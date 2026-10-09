// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The Claude Code plugin's mod, shared by the Rustal Harness and Marley (M13, D169,
// D176). It reports an interactive session's state to the program that started it,
// through their agent report contract (D163): idle when the session starts, working
// while a main turn runs, idle when it ends, and a release when the session ends. When a
// main turn ends, it reports the run's token counts and the account's quota windows
// (TICKET-104).
//
// The host is chosen at session.start (TICKET-108): the harness when RH_BIN and RH_STATE
// are set, otherwise Marley when MARLEY_BIN and MARLEY_TERMINAL_ID are, and none in a
// session no person is at, such as a `claude -p` run inside another. An empty variable
// counts as unset. To the harness the mod runs `rh report` and `rh release`, puts every
// call Claude Code would ask a person about to `rh approve` (TICKET-101), and takes the
// harness's sends and interrupts from `rh listen`, confirming each with `rh delivered`
// (TICKET-102); they find the seat by the terminal they run in. To Marley it runs
// `MARLEY_BIN report` and `MARLEY_BIN release`, with the arguments `rh` takes after
// `--state ROOT`; Claude Code's own dialog decides the calls, and the session reads
// waiting while the dialog stands.
//
// A mod's MCP calls would wait for the harness's server to connect and for Claude Code's
// permission check, and Claude Code refuses them once the session is ending. Every mods
// API call is spelled at its call site, as Claude Code's loader requires.

const SOURCE = 'mod:claude-code'
const REPORT = { timeoutMs: 5000 }
// Claude Code gives all session.end hooks together 1.5 seconds: the reports still
// running get some of it, the release the rest.
const RELEASE = { timeoutMs: 1000 }
const DRAIN = 400
// `$.process.run`'s longest wait, ten minutes; `rh approve` withdraws its request at 590
// seconds and answers deny.
const DECISION = 600000
// The longest activity the contract takes, in characters.
const ACTIVITY = 120
// Wall-clock milliseconds keep the sequence rising across restarts and resumes.
let seq = Date.now()
// The host chosen at session.start: its program, the arguments before the verb, and
// whether it is the harness. None until then, and none in a session no person is at.
let host = null
// The running main turn's id, which `$.turn.abort` takes.
let turn = null
// Under Marley's host, the calls Claude Code's own dialog stands on, by their ids.
const asking = new Set()
// The reports waiting for the loop session.start begins, which runs them one at a time
// in the order the hooks raised them, so a host takes them in the order of their seq.
// They run from that loop because Claude Code aborts every mods call made on behalf of
// an event it has abandoned, as it abandons a denied call's `tool.call`, whichever `$`
// makes it.
const pending = []
// Wakes the loop when a report arrives while it waits.
let wake = () => {}
// Which session.start's loop runs the reports: a later session.start, as Claude Code
// raises again when it enables the plugin anew, retires the one before.
let loops = 0
// The run's token counts, its main turns' and its subagents': the input not read from
// the prompt cache, cache writes included, the output, and the cache reads.
const used = { input: 0, output: 0, cacheRead: 0 }

// The report's arguments for the run's usage and the quota windows Claude Code read from
// the last response; a window the contract wouldn't take is left out.
const measured = (rateLimits) => {
  const args = ['--input-tokens', String(used.input), '--output-tokens', String(used.output),
    '--cache-read-tokens', String(used.cacheRead)]
  const kinds = new Set()
  for (const window of Array.isArray(rateLimits) ? rateLimits : []) {
    if (kinds.size === 8 || typeof window.kind !== 'string' || !/^[a-z0-9_]{1,32}$/.test(window.kind)) continue
    if (kinds.has(window.kind) || !(window.percentUsed >= 0 && window.percentUsed <= 10000)) continue
    kinds.add(window.kind)
    const resets = typeof window.resetsAt === 'string' ? Date.parse(window.resetsAt) : NaN
    args.push('--quota', window.kind + ':' + window.percentUsed + (resets >= 0 ? ':' + resets : ''))
  }
  return args
}

// The host's command for a report of `state`, naming the conversation the session runs
// now, which /clear and /resume change without a new session.start.
const report = (state, id) => {
  seq = Math.max(seq + 1, Date.now())
  const argv = [host.program, ...host.before, 'report', '--source', SOURCE, '--seq', String(seq), state]
  if (typeof id === 'string' && id) {
    argv.push('--session-id', id, '--resume-arg=claude', '--resume-arg=--resume', '--resume-arg=' + id)
  }
  return argv
}

// Queues a report of `state`, with its activity and, when `measure` is set, the run's
// usage and quota; it resolves once the report has run or failed. With no state it
// resolves once the reports before it have.
const send = (state, { activity = null, measure = false } = {}) =>
  new Promise((done) => {
    pending.push({ state, activity, measure, done })
    wake()
  })

// Resolves with `sent` or after `ms`, whichever comes first, so a hook never spends its
// own time limit waiting on a host; the report still runs in its turn.
const awhile = (sent, ms = REPORT.timeoutMs) => {
  let timer = null
  const passed = new Promise((resolve) => {
    timer = setTimeout(resolve, ms)
  })
  return Promise.race([sent, passed]).finally(() => clearTimeout(timer))
}

// What a call acts on, from its input: a command, a file, a URL, a search, or the first
// question it asks.
const subject = (input) => {
  if (!input || typeof input !== 'object') return ''
  const questions = Array.isArray(input.questions) ? input.questions : []
  const question = questions[0] && typeof questions[0] === 'object' ? questions[0].question : null
  for (const value of [input.command, input.file_path, input.notebook_path, input.url, input.query,
    input.pattern, input.path, question]) {
    if (typeof value === 'string' && value) return value
  }
  return ''
}

// One line naming the tool and what it acts on, as the contract takes an activity:
// control characters and runs of white space become one space, and a line longer than
// 120 characters is cut with an ellipsis.
const acting = (tool, input) => {
  const thing = subject(input)
  const line = (thing ? tool + ': ' + thing : String(tool))
    .replace(/[\u0000-\u001f\u007f-\u009f]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
  const characters = Array.from(line)
  return characters.length > ACTIVITY ? characters.slice(0, ACTIVITY - 1).join('') + '…' : line
}

export function register(on) {
  on('session.start', async ($, e, next) => {
    host = null
    try {
      if (e.isInteractive) {
        const rh = await $.env.get('RH_BIN')
        const root = await $.env.get('RH_STATE')
        if (rh && root) {
          host = { program: rh, before: ['--state', root], harness: true }
        } else {
          const marley = await $.env.get('MARLEY_BIN')
          const terminal = await $.env.get('MARLEY_TERMINAL_ID')
          if (marley && terminal) host = { program: marley, before: [], harness: false }
        }
      }
    } catch {}
    const loop = ++loops
    wake()
    if (!host) return next(e)
    // The reports' loop runs as long as the session.
    void (async () => {
      while (loop === loops) {
        const item = pending.shift()
        if (!item) {
          await new Promise((resolve) => {
            wake = resolve
          })
          continue
        }
        if (item.state) {
          try {
            let id = null
            try {
              id = await $.session.id()
            } catch {}
            const argv = report(item.state, id)
            if (item.activity) argv.push('--activity', item.activity)
            if (item.measure) {
              // The windows are a subscription's; any other login reads none.
              let rateLimits = []
              try {
                rateLimits = (await $.session.usage()).rateLimits
              } catch {}
              argv.push(...measured(rateLimits))
            }
            await $.process.run(argv, REPORT)
          } catch {}
        }
        item.done()
      }
    })().catch(() => {})
    await awhile(send('idle'))
    // `rh listen` runs as long as the session and prints each send or interrupt once: a
    // send becomes the person's words, an interrupt ends the running turn, and each is
    // reported once acted on. Marley takes neither yet.
    if (host.harness) {
      const { program: rh, before } = host
      void (async () => {
        const listen = $.process.spawn({ argv: [rh, ...before, 'listen', '--source', SOURCE] })
        let buffer = ''
        for await (const piece of listen) {
          if (piece.stream !== 'stdout') continue
          buffer += piece.text
          let end
          while ((end = buffer.indexOf('\n')) >= 0) {
            const line = buffer.slice(0, end)
            buffer = buffer.slice(end + 1)
            let command = null
            try {
              command = JSON.parse(line)
            } catch {
              continue
            }
            const delivered = [rh, ...before, 'delivered', '--source', SOURCE, '--delivery', command.delivery]
            if (command.kind === 'send') {
              // It resolves once the turn starts, after the session is next idle.
              void $.prompt
                .submit({ text: command.text, asUser: true })
                .then(() => $.process.run(delivered, REPORT))
                .catch(() => {})
            } else if (command.kind === 'interrupt' && turn) {
              try {
                await $.turn.abort({ turnId: turn })
                await $.process.run(delivered, REPORT)
              } catch {}
            }
          }
        }
      })().catch(() => {})
    }
    return next(e)
  })
  // A subagent's run raises no turn.start, and its turn.complete carries its agentId.
  on('turn.start', async ($, e, next) => {
    turn = e.turnId
    if (host) await awhile(send('working'))
    return next(e)
  })
  // Claude Code sums a turn's requests in its `usage`, and a subagent's turn carries
  // its own.
  on('turn.complete', async ($, e, next) => {
    if (e.usage) {
      used.input += (e.usage.input_tokens || 0) + (e.usage.cache_creation_input_tokens || 0)
      used.output += e.usage.output_tokens || 0
      used.cacheRead += e.usage.cache_read_input_tokens || 0
    }
    if (!e.agentId) {
      turn = null
      asking.clear()
      // A turn an API error ended leaves the session at its prompt, failed, and an
      // interrupted one says so on the seat until the next turn starts.
      if (host) {
        const activity = e.isAborted ? 'the last turn was interrupted' : null
        await awhile(send(e.reason === 'error' ? 'error' : 'idle', { activity, measure: true }))
      }
    }
    return next(e)
  })
  // A call Claude Code would put to a person goes to the harness, as an M12 session's
  // requests do (D156): approve runs it, deny refuses it, and cancel refuses it and ends
  // the turn. The hook catches nothing from `rh approve`, so any failure reaches the
  // `.catch`, which denies the call. Under Marley's host Claude Code's own dialog decides,
  // and the session reads waiting, with the call as its activity, until the call resolves;
  // the hook doesn't wait for that report, so a slow host can't run it into the `.catch`.
  on('tool.check', async ($, e, next) => {
    const verdict = await next(e)
    if (verdict.decision !== 'ask' || !e.tool_use_id || !host) return verdict
    if (!host.harness) {
      if (!asking.has(e.tool_use_id)) {
        asking.add(e.tool_use_id)
        void send('waiting', { activity: acting(e.tool, e.input) })
      }
      return verdict
    }
    const asked = [host.program, ...host.before, 'approve', '--source', SOURCE, '--call', e.tool_use_id,
      '--tool', e.tool]
    const input = JSON.stringify(e.input === undefined ? null : e.input)
    const run = await $.process.run(asked, { stdin: input, timeoutMs: DECISION })
    if (run.exitCode !== 0) throw new Error('rh approve exited with status ' + run.exitCode)
    const answer = JSON.parse(run.stdout)
    if (answer.decision === 'approve') return { decision: 'allow', reason: 'approved through the harness' }
    if (answer.decision !== 'deny' && answer.decision !== 'cancel') throw new Error('rh approve gave no decision')
    if (answer.decision === 'cancel' && turn) await $.turn.abort({ turnId: turn })
    return { decision: 'deny', reason: answer.reason || 'denied through the harness' }
  }).catch(($, e, next) => ({ decision: 'deny', reason: 'the harness could not decide: ' + next.error.kind }))
  // A call whose dialog stood under Marley's host has resolved, approved or denied, and
  // the session works on while its main turn runs.
  on('tool.call', async ($, e, next) => {
    try {
      return await next(e)
    } finally {
      if (e.tool_use_id && asking.delete(e.tool_use_id) && host && turn) void send('working')
    }
  })
  // /clear and /resume end a conversation, not the session.
  on('session.end', async ($, e, next) => {
    if (host && e.reason !== 'clear' && e.reason !== 'resume') {
      await awhile(send(null), DRAIN)
      try {
        await $.process.run([host.program, ...host.before, 'release', '--source', SOURCE], RELEASE)
      } catch {}
    }
    return next(e)
  })
}
