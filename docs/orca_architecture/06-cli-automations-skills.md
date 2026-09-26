# Orca survey 06: the agent-facing control surface

Area: the `orca` CLI, orchestration, automations, skills, the AI Vault, artifacts, agent hooks
and memory files. Source read at `/srv/stacks/orca-refs/orca`, HEAD `1c2cf120e3` (2026-09-25).
Orca paths are relative to that repo. Marley paths are under `/srv/stacks/marley_ide`.

## 1. Summary

Orca's agent surface is a 238-command CLI, `orca` (`orca-ide` on Linux), and no MCP server. Each
call is one JSON line over a 0600 Unix socket found through `<userData>/orca-runtime.json` (path
plus shared token). Every Orca terminal starts with its identity in env vars (`ORCA_TERMINAL_HANDLE`,
`ORCA_PANE_KEY`, `ORCA_WORKTREE_ID`) and a PATH shim; agents learn the CLI from thin skill stubs
that fetch a version-matched guide with `orca skills get`. On top: orchestration (Runs, Tasks,
Dispatches, a SQLite mailbox, gates; about 42k lines), cron automations with prechecks, the "AI
Vault" (history and FTS5 search over 20 agents' transcripts, no embeddings), cloud artifact links.
Take: (1) terminal identity in env, forwarded by Marley's bridge; (2) receipted input that proves a
turn started and never types into trust or approval prompts; (3) mail as a one-line pointer typed
into an idle agent; (4) per-project session history with resume; (5) refusals with `nextSteps`.

## 2. Features

### 2.1 The CLI and how it reaches the app

**What the user sees.** `orca` on macOS and Windows, `orca-ide` on Linux (GNOME's screen reader
owns `/usr/bin/orca`), registered under Settings → General → Orca CLI. `orca status --json`,
`orca open`, `orca --help`, `orca help <command>`.

**Underneath.**

- Launcher: `resources/linux/bin/orca-ide` runs the app's own Electron binary with
  `ELECTRON_RUN_AS_NODE=1` on `app.asar.unpacked/out/cli/index.js`. There is no separate Node.
- Entry: `src/cli/index.ts` parses argv against `COMMAND_SPECS` (`src/cli/specs/*.ts`) and
  validates the command and flags before any runtime lookup, so a typo never reports "Orca is not
  running". It then lazy-loads the RPC client (a comment counts it as "153 of the CLI's 199 eager
  modules") and only the handler group it needs (`src/cli/handler-group-manifest.ts`,
  `src/cli/dispatch.ts`).
- Discovery: `src/cli/runtime/metadata.ts` reads `<userData>/orca-runtime.json`
  (`src/shared/runtime-bootstrap.ts`): `{runtimeId, pid, transports[], authToken, startedAt}`.
  userData on Linux is `$XDG_CONFIG_HOME/orca`, overridable with `ORCA_USER_DATA_PATH` (dev builds
  and parallel instances). The app writes the file 0600 (`src/main/runtime/runtime-metadata.ts`),
  and a watcher polls it every 10 s and republishes it once it no longer names a live runtime, for
  the case where a second instance slipped past the single-instance lock, wrote its own pid and
  exited (#7848; `src/main/runtime/runtime-metadata-ownership-watch.ts`).
- Transport: `src/cli/runtime/transport.ts` connects to `<userData>/o-<pid>-<suffix>.sock` (a named
  pipe on Windows), writes `{id, authToken, method, params, ...}` as one line, reads one response
  frame and closes. Server side: `src/main/runtime/rpc/unix-socket-transport.ts` (chmod 0600, 1 MiB
  per message, 32 connections, 30 s idle) and
  `src/main/runtime/runtime-rpc/runtime-rpc-request-admission.ts` (token check, then long-poll
  admission with a total cap and sub-caps, refusing with `runtime_busy`). During long polls
  (`terminal wait`, `orchestration check --wait`, `ask`) the server writes `{"_keepalive":true}`
  every 10 s; the client refreshes its timer on each and widens its own timeout to the requested
  wait plus 10 s (`src/cli/runtime/client.ts`, `resolveMethodTimeoutMs`).
- Responses: `{id, ok:true, result, _meta:{runtimeId}}` or
  `{id, ok:false, error:{code, message, data}, _meta}`. The client rejects a reply whose runtimeId
  differs from the metadata it read ("The Orca runtime changed while the request was in flight").
- The RPC methods under `src/main/runtime/rpc/methods/` (about 29k lines) also serve the desktop
  renderer, the mobile app and paired clients. The CLI is one more client of the app's own API.

**Good and bad.** Flag validation before connecting, per-method timeouts and the runtime-id check
are all worth copying. One request per connection keeps the server simple; every call re-reads
the metadata file and opens a socket, which is fine for a CLI.

**Size.** `src/cli` is about 20k lines of TypeScript plus 30k lines of tests. The socket server and
admission are a few hundred lines.

**Marley today.** No CLI. `marley_mcp` serves Streamable HTTP on 127.0.0.1 with a bearer in
`mcp-endpoint.json`, written 0600 before the bearer's bytes land and removed on quit
(`crates/marley_mcp/src/discovery.rs`), and the plugin's Python stdio bridge carries it to Claude
Code and to Zed's agents (`crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge`).
Discovery and permissions match Orca's in spirit.

### 2.2 The command surface

238 command specs, one hidden (`terminal stop`). Grouped:

| Family | Commands |
|---|---|
| Runtime and hosts | `open`, `serve`, `status`, `host name`, `host list`, `environment add/list/show/rm`, `diagnostics memory` (process memory, not agent memory), `agent-context` |
| Repos and projects | `repo list/add/show/set-base-ref/search-refs`; `project list/setups/setup-existing-folder/setup-clone/setup-create/setup-update/setup-delete` |
| Worktrees | `worktree list/show/current/create/set/rm/ps` |
| Terminals | `terminal list/show/read/send/wait/create/split/rename/switch/close` |
| Files | `file open/diff/open-changed` |
| Agents and sessions | `search`, `claude-teams`, `account add/list`, `agent hooks status/on/off/prepare-codex` |
| Orchestration (30) | `run-create/run-use/run-current/run-list/run-show`; `send`, `check`, `reply`, `inbox`, `ask`; `task-create/task-list/task-update`; `worker-start/show/read/stop/abandon/release/retain/list`; `dispatch`, `dispatch-show`, `request-show`; `gate-create/gate-resolve/gate-list`; `reset`; `coordinator-start/stop` (retired no-ops, aliased `run` and `run-stop`) |
| Automations (7) | `list`, `show`, `create`, `edit`, `remove`, `run`, `runs` |
| Skills (6) | `installed`, `share`, `list`, `get`, `install`, `update` |
| Artifacts (5) | `share`, `update`, `unshare`, `list`, `delete` |
| Built-in browser (80) | `goto`, `snapshot`, `click`, `fill`, `type`, `screenshot`, `tab ...`, `tab profile ...`, `cookie ...`, `storage ...`, `intercept ...`, `capture ...`, `console`, `network`, `set device ...`, `exec` |
| Other integrations | `computer ...` (14), `emulator ...` (16), `linear ...` (27), `vm recipe doctor` |

Each spec carries `summary`, `usage`, `notes`, `examples`, `aliases`, `destructive`, `hidden` and
`positionalArgs`. Help text, `agent-context --json`, typo suggestions and three parity tests read
the same table (`src/cli/registry-parity.ts`, `src/cli/cli-command-name-parity.test.ts`,
`src/cli/skill-guide-cli-parity.test.ts`). `src/cli/vocabulary-policy.ts` holds verbs to one
spelling (`rm` for deletion, `show` for single reads) and keeps aliases where agents guess:
`worktree remove` and `worktree delete` resolve to `worktree rm` because agents reach for git's
verbs.

Critique: 238 commands is a lot to teach. About 137 of them are integrations (browser, computer
use, emulators, Linear) that Marley covers through MCP or not at all. Orca answers the teaching
problem with guides that load one reference at a time (2.11).

### 2.3 Host scoping: local, SSH, paired server

**What.** One CLI acts on this machine, a registered SSH target, or a paired remote Orca server.
`orca host list` prints every machine with the selector that reaches it. Global flags
`--environment <name|id>` and `--pairing-code <code>` (env `ORCA_ENVIRONMENT`,
`ORCA_PAIRING_CODE`) pick a paired server. `--host local|ssh:<target>|runtime:<environment-id>`
names an execution host on commands that create or filter (worktree create, automations, project
setups). Grammar in `src/shared/execution-host.ts`.

**Underneath.**

- `src/cli/index.ts` resolves `--host runtime:<name>` to the environment id before the client
  exists, and refuses when it disagrees with an ambient `ORCA_ENVIRONMENT`. Paired servers are
  reached over WebSocket with per-device tokens and tweetnacl encryption
  (`src/cli/runtime/websocket-transport.ts`, `src/shared/remote-runtime-client.ts`).
- `src/cli/execution-host-flag.ts` and `host-selector-alternatives.ts` check the other axis when a
  name misses ("`--environment openclaw` is very often an SSH target") and put the known ids in
  `error.data`.
- Some commands never route to a paired server because their answer belongs to this machine:
  `account`, `artifacts`, `environment`, `host list`, `serve`, `agent`, `vm`, `agent-context`
  (`shouldIgnoreRemoteSelection` in `src/cli/index.ts`; `host list` and `environment list` reject
  the flags outright).
- SSH terminals: Orca installs a relay and an `orca` launcher on the remote host
  (`src/main/ssh/ssh-remote-cli-launcher.ts`). The remote `orca` sends argv, cwd, a filtered env
  and stdin through the relay socket (`src/relay/relay-orca-cli-channel.ts`); the desktop runs the
  command with `ORCA_CLI_CWD` set to the remote cwd, so `--worktree active` still resolves.
- Coverage: `terminal list`, `worktree list` and `worktree ps` return `hostScope` with the hosts
  they covered and the ones they did not, and the CLI adds the flag that reaches each omitted host
  (`src/cli/omitted-host-scope-selectors.ts`). A missing terminal counts as exited only when its
  host was covered; otherwise the verdict is `unverifiable`.

**Good and bad.** The cross-axis hints and the coverage rule stop an agent from reading "no
terminals" out of a partial answer. Three kinds of host behind two flags confuse users, and a lot
of Orca's error text exists to untangle them.

**Size.** A few hundred lines in the CLI plus the relay channel.

**Marley today.** None. `marley_remote` builds an `ssh` argv; an agent in a remote terminal cannot
reach the loopback MCP endpoint.

### 2.4 Output and errors for agent callers

- `--json` prints the RPC envelope. Human output is short text with no ANSI (`src/cli/format.ts`
  and the per-domain `*-format.ts` files).
- `src/cli/cli-error.ts`: text errors end with `Next step: ...` lines; JSON errors keep
  `error.data` with `nextSteps`, `validFlags`, `suggestions`, `knownEnvironments`,
  `validSelectorForms`.
- `src/cli/command-suggestion.ts`: unknown commands and flags are ranked by edit distance against
  the live spec table (distance 3 or less, at most 3 suggestions) and the error lists every valid
  flag. A destructive command is suggested only when the typed verb is within distance 1 of a
  destructive verb, because "suggestions flow into agents' recovery channel" (#6303). A synonym
  table covers renames (`--from` on the one verb that takes `--terminal`).
- `src/cli/worktree-selector-recovery.ts`: a selector miss names the bad value and the valid
  forms, including "a bare repository id is not a worktree id".
- Unknown mutation outcomes (`src/cli/orchestration-mutation-recovery.ts`,
  `src/cli/runtime/orchestration-recovery-command.ts`): the client stamps every durable mutation
  with a request id. On a timeout or dropped connection it prints that the mutation "may already
  have taken effect", a read-only query (`request-show --request <id>` or
  `worker-show --dispatch <id>`), and the exact retry command with `--retry-request <id>`, with
  `--pairing-code` and `--dispatch-capability` stripped so no secret lands in the retry line. The
  server keeps receipts keyed by caller fingerprint, request id and payload hash.
- `src/cli/skill-guide-cli-parity.test.ts` fails the build when a guide names a command or flag
  missing from the specs; its comment cites `orca emulator camera --webcam`, "documented for
  months without ever existing".

**Good.** This is the best-built part of Orca's agent surface: every refusal tells the agent what
to run next, in a form it can execute.

**Size.** Under 1k lines of core code, with many tests.

**Marley today.** `marley_mcp` answers a refusal with `isError: true` and
`{result: "refused", reason}` (`crates/marley_mcp/src/tools.rs`, `tool_error`). No `nextSteps`, no
list of valid values, no idempotency key on writes.

### 2.5 How an agent knows it is in Orca, and which terminal it is

Every Orca PTY gets these (`src/main/providers/local-pty-spawn-environment.ts`,
`src/main/ipc/pty/provider/local-configure.ts`, `src/main/ipc/pty/host-env/assembly.ts`):

- `TERM_PROGRAM=Orca`, `TERM_PROGRAM_VERSION`, and `FORCE_HYPERLINK=1` because supports-hyperlinks
  rejects unknown terminals and tools would drop OSC 8 links.
- `ORCA_TERMINAL_HANDLE`: a runtime handle allocated before spawn, so an agent can name itself in
  its first command without an RPC.
- `ORCA_PANE_KEY` (`<tabId>:<leafId>`, the layout leaf UUID that survives renderer reloads),
  `ORCA_TAB_ID`, `ORCA_WORKTREE_ID`. Setup hooks also get `ORCA_ROOT_PATH` and
  `ORCA_WORKTREE_PATH` (`src/main/setup-hook-env-vars.ts`).
- `ORCA_AGENT_HOOK_PORT`, `_TOKEN`, `_ENDPOINT`, `_VERSION` for the status hooks (2.14).
- PATH with the app's CLI directory first; on Linux a shim directory makes bare `orca` mean Orca
  inside its own terminals (`src/main/cli/orca-cli-child-path.ts`).
- Inherited identity is scrubbed: a terminal opened by an agent running inside Orca gets fresh pane
  identity (`removeUnspecifiedPaneIdentityEnv`), and the parent's hook coordinates are deleted
  before the new pane's are set.

The CLI answers "who am I" in this order (`src/cli/handlers/orchestration/terminal-identity.ts`):
an explicit `--from` or `--terminal`; `ORCA_TERMINAL_HANDLE`, checked live with
`terminal.resolveIdentity`; if that handle is stale, a remint from `ORCA_PANE_KEY` through
`terminal.resolvePane`. A structured session with no identity is refused rather than guessed,
because a guessed `check` would consume a sibling's mail. `active` and `current` map the cwd to the
enclosing managed worktree by longest path prefix (`src/cli/selectors.ts`). `orca agent-context
--json` prints the command table as a schema (v1) and needs no running app
(`src/cli/agent-context.ts`).

Critique: identity rests on environment variables that a long-lived shell keeps across an app
restart, which is why the live check and the pane-key remint exist.

**Marley today.** Marley's terminals carry `TERM_PROGRAM=zed` and `MARLEY_SHELL_NONCE` (shell
integration) and nothing that names the terminal. `terminal_list` ids are gpui entity ids
(`crates/marley_workbench/src/mcp.rs`, `terminal_list`) that the terminal never sees, and the
bridge forwards no caller identity, so an agent in a Marley terminal cannot ask for its own blocks.

### 2.6 Terminal verbs: read, send, wait

- `terminal read`: the accumulated output with escapes stripped (`stream`), or `--screen`, the
  rendered frame, the only faithful read of a TUI. The result names its `source` (`stream`,
  `screen`, `screen-unavailable`). Cursor paging with `nextCursor`, `oldestCursor` and `--limit`
  (`src/main/runtime/orca-runtime-resolve-terminal-pane.ts`).
- `terminal send --text ... --enter`: for an agent prompt the host frames the text as a bracketed
  paste, serializes sends per PTY, pins the PTY incarnation, applies per-agent submit timing, and
  returns a receipt whose stages are `input_accepted`, then `turn_started`. A turn start is proven
  by hook evidence (a new "working" sequence or a hook turn-start timestamp) or by output after the
  Enter (`src/main/runtime/agent-prompt-submission-verification.ts`); only Claude, Codex and
  Antigravity give a signal a receipt can settle on. `--wait-submit <s>` observes without
  resending; `--retry-request <id>` replays after a lost reply and is bound to the payload and the
  process incarnation. The guide's rule: never resend on silence.
- `terminal wait --for exit|tui-idle`: tui-idle combines first-party hook status, an explicit idle
  title (Claude's `✳` prefix), a known ready prompt in the screen text, and quiescence
  (`src/main/runtime/runtime-terminal-wait.ts`, `terminal-wait-detection.ts`). When the screen shows
  an update prompt, a trust-this-folder prompt, a cwd prompt, a hooks review or an approval prompt,
  the wait returns a `blockedReason` instead of idle
  (`RuntimeTerminalWaitBlockedReason` in `src/shared/runtime-terminal-contracts.ts`), so a caller
  never types a task into a dialog.
- `terminal create --command`, `split`, `rename`, `switch`, `close` (close verifies that every PTY
  stopped and reports `unverifiable` when it cannot).
- Launch prompts: `src/shared/tui-agent-config.ts` records per agent how the first prompt goes in:
  as argv (`claude "<prompt>"`, `codex "<prompt>"`), a flag, or a paste after readiness; drafts use
  `claude --prefill`. `src/main/runtime/rpc/methods/agent-launch-terminal-prompt.ts` explains why
  argv wins: the prompt is in the process at exec time, so there is no readiness race.

**Good and bad.** The receipt stages, the refusal to type into dialogs and the per-agent launch
table are hard-won; the comments cite dozens of issues. Dialog detection is text matching per
agent version and needs constant upkeep.

**Size.** A subsystem: a few thousand lines across runtime, shared and CLI.

**Marley today.** Read only: `terminal_list`, `terminal_blocks`, `terminal_read` (one block's
output, up to 2,000 lines). For shell work Marley's blocks with exit codes beat Orca's stream:
Orca reads OSC 133 marks only internally, for shell readiness and TUI lifecycle
(`src/main/daemon/terminal-shell-lifecycle-scanner.ts`), and gives agents no per-command blocks.
Marley has no screen read, send, wait or create.
`crates/marley_fleet/src/verbs.rs` already defines `SendRequest`, `ReadRequest`, `OpenRequest`,
`AnswerRequest` and `Receipt`, and `dispatch.rs` a `Deposited < Claimed < Started` delivery state,
all without handlers; `docs/marley/three-prong-plan.md` (D9, slice C4) names `session.send` and
`session.answer`.

### 2.7 Worktree handoffs and checkpoints

- `orca worktree create --name X --agent claude --prompt "..." [--setup run|skip|inherit]
  [--parent-worktree active | --no-parent] [--base-branch <ref>] [--issue N | --linear-issue ID]
  [--activate]` creates the checkout, starts the agent in the first terminal ("agent-first"),
  delivers the prompt, records parent and child lineage when run from inside a worktree, and stays
  in the background unless `--activate`. The JSON receipt carries `agentTerminalHandle`.
- The orca-cli guide separates a full handoff (worktree, agent and prompt; done when the send
  receipt says accepted; no monitoring) from supervised orchestration, and forbids creating Tasks
  for a handoff (`skill-guides/orca-cli.md`, "Full Handoffs"). Independent handoffs use
  `--no-parent` and the repo's default base, never the current feature branch unless asked.
- `orca worktree set --comment "..." --workspace-status todo|in-progress|in-review|completed`
  writes a one-line status on the workspace card. Agents set it at checkpoints; the guide says to
  read the current comment first so a human's context is not overwritten
  (`docs/site/content/docs/cli/worktree-checkpoints.mdx`).
- `orca worktree ps` gives per worktree: repo, branch, host, live terminal count, whether a PTY is
  attached, unread, preview.
- `orca worktree rm` also deletes the local branch, but keeps branches that predated the worktree
  and any branch it cannot prove merged. `orca.yaml` archive hooks run only with `--run-hooks`,
  and a failed hook blocks removal unless `--allow-failed-archive-hook`.
- Trust prompts: Orca pre-writes the trust markers Cursor, Copilot, Codex and Antigravity check at
  startup, so a new worktree path does not open a "trust this folder?" menu that would eat the
  pasted prompt (`src/main/agent-trust-presets.ts`). Claude is launched with
  `--dangerously-skip-permissions` by default (`docs/site/content/docs/model/agents-sessions.mdx`,
  "Launch defaults").

**Marley today.** None. TICKET-510 (worktree agents) and TICKET-511 (review and merge a worktree)
are queued, both as UI features.

### 2.8 Orchestration

**What the user sees.** Very little. Agents drive it through the `orchestration` skill. The human
sees agents in the experimental Agent Dashboard, and clickable `task_...` ids in terminal output
that focus the assigned terminal (`src/renderer/src/components/terminal-pane/terminal-orchestration-task-links.ts`).
There is no view of Runs, the task graph, gates or the inbox. The docs say to enable orchestration
under Settings → Experimental. In the source, Settings → Orchestration
(`src/renderer/src/components/settings/OrchestrationPane.tsx`) is a setup panel that installs the
CLI and the orchestration skill per agent and sets the nested worker depth
(`src/shared/nested-worker-depth.ts`, default 1, so workers cannot sub-dispatch); I found no
runtime gate on the RPC methods.

**Model** (`docs/site/content/docs/cli/orchestration.mdx`, `src/main/runtime/orchestration/types.ts`):

- Run: a namespace and the coordinator's inbox, bound to the coordinator's terminal. It never
  schedules or places workers.
- Task: spec, dependencies (JSON array), parent; status pending, ready, dispatched, completed,
  failed or blocked.
- Dispatch: one attempt of a task on one terminal or structured session; the lifecycle authority
  for `worker_done` and heartbeats.
- Message types: status, dispatch, worker_done, merge_ready, escalation, handoff, decision_gate,
  question, heartbeat; priorities normal, high, urgent; threads.
- Decision gate: a coordinator-owned question that blocks a task until `gate-resolve`.
- Storage: SQLite at `<userData>/orchestration.db`
  (`src/main/runtime/orca-runtime-automation-operations.ts`, `getOrchestrationDb`).

**Primitives for one agent steering others.**

- Spawn: `worker-start --task <id> | --spec <text>` with `--worktree current|new-child|new-top-level`,
  `--agent claude|codex|...`, optional `--model` and `--effort`, optional `--on <environment>` for
  a paired server. It composes placement, readiness, preamble injection and resource ownership,
  and exits 0 only when the worker is ready.
- Steer: `send --to dispatch:<id>`; a follow-up Dispatch into the same terminal with
  `worker-start --terminal <handle>`.
- Message: `send`, `reply --id`, and group addresses `@all`, `@idle`, `@claude`, `@codex`,
  `@worktree:<id>`, which resolve only within the sender's own Run.
- Ask and block: `ask --question --options --timeout-ms`; a timeout leaves the question pending and
  `--resume <message-id>` continues it.
- Wait: `check --wait --types worker_done,escalation,question --timeout-ms`, with JSON keepalive
  lines on stderr every 15 s.
- Observe: `worker-show` (with `observation.agentWait` when a worker is parked on a prompt only a
  human can answer, and the evidence: hook, prompt text or title), `worker-read --source
  auto|transcript|terminal`, `worker-list` (with `projection.liveness`, `attention`, and a literal
  `nextAction` argv per row).
- End: `worker-stop`, `worker-abandon` (fence without claiming the process stopped),
  `worker-release` (archive the output, then close only the coordinator-owned terminal),
  `worker-retain`.

**Delivery** (`src/main/runtime/orchestration/mailbox-pointer-*.ts`, `formatter.ts`,
`mailbox-pointer-eligibility.ts`). Messages stay in SQLite. When the recipient's agent is idle and
no `check --wait` is pending for those types, Orca types one line into its TUI, "You have 2
orchestration messages. Run `orca orchestration check`.", then Enter. The write is reserved and
settled so that a write that may have reached the wire is never retried as undelivered. `check`
returns the oldest unacknowledged FIFO batch and replays it until `--ack <deliveryId>`. A pending
`check --wait` preempts pointer typing.

**Worker contract.** The preamble (`src/main/runtime/orchestration/preamble.ts`) is pasted as the
worker's first prompt. It names the coordinator's handle and the task id and gives the exact
commands with `--from <handle> --dispatch-capability dcap_... --task-id ... --dispatch-id ...`
filled in: send `worker_done` exactly once with `--outcome succeeded|failed` and a three-sentence
body; send a heartbeat every 5 minutes; use `ask` instead of the agent's own question UI
(AskUserQuestion) because the coordinator cannot see it; check mail at checkpoints and before
`worker_done`; then go idle. Comments in the file record prompt lessons: rules sit beside the
example command because models skim trailing prose, and shouted rules "read as prompt injection"
to Claude Code (STA-8200). The capability is 32 random bytes, stored as a SHA-256 hash bound to the
pane key and process incarnation and rotated on re-dispatch
(`src/main/runtime/orchestration/db/dispatch-context/dispatch-capability.ts`), so a late message
from an old attempt cannot settle the current one.

Also present: federated workers on paired servers (`--on`, then routed by dispatch id), nested
coordinators, structured-session workers with no terminal.

**Good.** Correctness gets attention everywhere: fencing, idempotent receipts, proofs of
settlement, and the rule that absence is not proof of exit. Pointer delivery is the idea to keep:
it wakes an idle agent without a push channel, keeps message bodies out of the terminal, and asks
nothing of the agent but a shell.

**Bad.** Size and legacy weight: about 42k lines of non-test code (runtime/orchestration 23k, RPC
methods 12.5k, RPC glue 2.3k, CLI 2.7k, shared 1.8k) and about 69k lines of tests. The two runtime
directories have 137 commits since orchestration landed on 2026-04-28 (`c9391e203f`), and already
carry legacy authority, takeover, and schema migrations up to v33. The guide is long (a 197-line
kernel plus seven references), and correct behavior depends on agents following it. The guide
tells coordinators to wait with `--timeout-ms 900000` while Claude Code's Bash tool stops a
command at 10 minutes; the guide treats a timeout as a checkpoint and the pointer covers the gap,
but the default fights the most common coordinator. Agents are launched with full-autonomy flags
by default, so the permission prompts that would stall a worker mostly never appear.

**Marley today.** Not implemented. `marley_fleet` has the envelope (seats, states, attention,
staleness), the verb types and the delivery state machine. The three-prong plan gives work state
to rustal-brain and durable messages and actor states to rustal-harness (D7 to D11), with Marley
as the mechanism shell that renders and delivers.

### 2.9 Claude Code Agent Teams in native panes

`orca claude-teams [claude args]`, run inside an Orca terminal, starts Claude Code with
`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`, a fake `TMUX=/tmp/orca-claude-agent-teams/<team>,0,1`,
`TMUX_PANE=%1`, `TERM=screen-256color`, a per-team token, and a `tmux` shim first on PATH
(`src/main/runtime/claude-agent-teams-service.ts`, `claude-agent-teams-shim-env.ts`). The shim runs
`orca agent-teams-tmux ...`, which calls the RPC method `agentTeams.tmuxCompat`;
`claude-agent-teams-tmux-dispatcher.ts` implements about 20 tmux commands (`split-window`,
`send-keys`, `capture-pane`, `list-panes`, `select-layout`, `resize-pane`, `kill-pane`,
`display-message`, `show-options`, `has-session`, ...) against Orca's own splits. Teammates show as
child rows under the lead agent. About 800 lines; not on Windows or WSL.

**Marley today.** None. Claude Code's teams inside a Marley terminal would look for a real tmux.

### 2.10 Automations

**What.** An Automations page (host column, filters, search, last run, Rerun) and
`orca automations create --name --trigger hourly|daily|weekdays|weekly|<cron>|<RRULE> [--time
--day --timezone] --prompt --provider <agent> (--repo <sel> | --workspace <sel> | --project --host
| --project-host-setup) [--precheck "<cmd>" --precheck-timeout] [--reuse-session | --fresh-session]
[--missed-run-grace-minutes] [--disabled]`, plus `run`, `runs`, `edit`, `remove`.

**Underneath** (`src/shared/automations-types.ts`, `src/main/automations/`):

- `service.ts` ticks every 60 s over enabled records whose `nextRunAt` has passed. Schedules are
  RRULE with a timezone. A missed run within the grace window runs once; beyond it the run is
  recorded `skipped_missed`. An unresolvable target folds repeated refusals into one row with an
  occurrence count, so a `*/5` schedule does not write 288 rows a day and push real history out of
  retention.
- Precheck (`precheck-runner.ts`): a bounded shell command on the target host before scheduled runs
  only; a non-zero exit records `skipped_precheck` with stdout and stderr.
- Dispatch: the renderer creates a worktree per run (`new_per_run`) or uses an existing workspace,
  launches the agent with the prompt as a background session, and marks the run `dispatched`
  (`src/renderer/src/hooks/automation-dispatch-handler.ts`). Headless `orca serve` has its own
  dispatcher. `--reuse-session` submits the prompt into the previous live automation terminal and
  counts the run done only after it sees working, then done.
- Completion (`runtime-terminal-run-observer.ts`): the pane must leave idle first (a reused pane is
  still idle from the last run), then a tui-idle wait re-arms for up to 6 hours, then up to the
  last 2,000 lines go into a bounded `outputSnapshot`. A blocked prompt fails the run with the
  reason.
- Usage (`run-usage-collection.ts`): tokens and estimated cost from the Claude and Codex usage
  stores, matched by session and time window, with `ambiguous_session` when the match is unclear.
- Owners: each record names its scheduler owner (`local_host_service`, `ssh_bridge`,
  `remote_host_service`). Edits carry an owner precondition and the SSH target's registration
  generation, so a record cannot silently re-point at a host that was removed and re-added
  (`src/cli/automation-destination.ts`, `automation-owner-conflict-recovery.ts`).
- External managers: the page also lists jobs from Hermes's cron and OpenClaw's own schedulers
  (`external-manager*.ts`, `hermes-cron-*.ts`).

**Good and bad.** Prechecks, missed-run grace, refusal folding and the busy-edge rule on reused
panes are thought through. Completion is inferred from idle heuristics, and the UI is 25k lines of
React for a scheduler.

**Size.** 4.6k lines main, 2.9k shared, 25k renderer, 0.8k CLI.

**Marley today.** None. Zed tasks run commands on demand. Chad has Claude Code's `/loop`, the cloud
`schedule` skill and Rusty tasks; none starts an agent in a Marley terminal.

### 2.11 Skills

**What Orca ships.** Seven skills under `skills/`: `orca-cli`, `orchestration`, `computer-use`,
`orca-linear` (plus the legacy name `linear-tickets`), `orca-emulator`, `orca-emulator-android`,
`orca-per-workspace-env`.

**Three layers.**

- `skill-guides/<name>.md` and `skill-guides/<name>/references/*.md` hold the procedures.
  `config/scripts/generate-bundled-skill-guides.mjs` embeds them in the CLI as
  `src/cli/bundled-skill-guides.ts`, and `orca skills get <name> [--full | --references |
  --reference <name>]` prints them (`src/cli/handlers/skill-guide-get.ts`). Each guide's kernel ends
  with an "action gates" table that says which reference to load in which situation, so an agent
  reads the kernel plus one reference.
- `skill-stubs/<name>.md` and `skill-stubs/_shared/cli-resolution.md` are the source for the
  installable stub.
- `skills/<name>/SKILL.md` is generated from the stub body plus the guide's own frontmatter (kept
  byte-identical by the generator), published in github.com/stablyai/orca and installed with
  `npx skills add https://github.com/stablyai/orca --skill <name> --global`. The frontmatter
  description holds the trigger phrases. The body picks the executable
  (`ORCA_CLI_COMMAND`, then `orca-dev` in a dev checkout, then `orca-ide` on Linux, else `orca`),
  tells the agent to run `ORCA skills get <name>`, and forbids guessing flags.

The design note behind it, added in `31f643ca42` (`notes/skill-guide-indirection-design.md`, since
folded into the freshness design): "Version-sensitive content must not live in distributed files;
only discovery metadata should." It cites vercel-labs/agent-browser, WrenAI and zerolang doing the
same.

**Install and update.** `orca skills install|update` runs `npx --yes skills add|update`
(vercel-labs/skills) and always passes `--agent` for the detected agents, because `-y` without one
installs into "all ~75 known agents" (`src/shared/agent-feature-install-commands.ts`). The app
scans skill homes for Codex, Claude, `.agents/skills`, Grok, OpenCode, Pi, OMP, Hermes, Prime Agent,
Gemini and plugin caches (`src/main/skills/skill-discovery-sources.ts`) and places copies per
provider (`src/shared/skill-install-providers.ts`: Claude `.claude/skills`, Codex reads
`.agents/skills`, Cursor `.cursor/skills`, Gemini `.gemini/skills`, and others). Freshness: the file
digests of every published revision sit in `resources/skills/snapshot-registry.json` and
`release-mapping.json`, so Orca can tell a current, outdated, newer or locally modified copy, and
offers an update only when every placement of that name is safe to rewrite.

**Sharing.** Skills → Share skills publishes one immutable bundle (Agent Plugins 1.0.0 layout)
behind an unlisted, revocable bearer link on Orca Cloud (GCS and Postgres, provisioned with
Terraform). Recipients install without signing in, into global, worktree, folder, WSL, SSH or
paired-host scope. Agents can publish only named skills, and only after a default-off setting
(`orca skills installed`, `orca skills share`). The implementation checklist
(`docs/agent-skill-sharing-implementation-checklist.md`) runs to 1,676 lines.

**Plugins.** `/srv/stacks/orca-refs/orca-workflow-skills/orca-plugin.json` declares
`contributes.skills` (two skills, `change-handoff` and `repository-review`), but
`src/shared/plugins/plugin-marketplace.ts` hides the `skills` category because skill contributions
"were deferred", so this build cannot install that plugin.

**Good and bad.** The stub plus served guide, the parity test and the digest-based freshness are
sound. Installs depend on a third-party npm CLI and the network; the sharing stack is large for
what it does; `src/main/skills` alone is 16k lines.

**Marley today.** The Claude Code plugin has hooks and the MCP bridge and no skill. Chad's skills
live in Rusty's skill store (a git repo) and are linked into Claude Code's skills directory, with
`skill_list`, `skill_view`, `skill_scan`, `skill_approve` and
`skill_reject` in rusty-mcp.

### 2.12 AI Vault: agent session history and search

"AI Vault" is Orca's internal name for the Agent Session History panel and the search behind it.
It is a session archive: an index over the transcripts that the agents themselves write. It holds
no curated knowledge and no prompt library.

**What the user sees.** Right sidebar → Agents → Agent Session History
(`docs/site/content/docs/agents/session-history.mdx`). Scope Workspace, Project or All; filter by
agent (20 in `src/shared/ai-vault-types.ts`: Claude, Codex, Cursor, Gemini, OpenCode, Grok, Devin,
Kimi, Pi, OMP, Hermes, Copilot, Cline, and more); group by project, folder or agent. Details show
branch, model, message count, the untruncated first prompt with Copy, the latest turns and
subagents. Actions: Resume (runs `claude --resume <id>` or `codex resume <id>` in a new terminal at
the session's cwd), copy the resume command, copy the session id or log path, open or reveal the
log, open the cwd as a workspace; dragging a row into the workspace also resumes it. Terminal tab
titles follow each agent session's own title
(`src/renderer/src/components/AiVaultTabTitleSyncGate.tsx` and
`src/renderer/src/lib/ai-vault-tab-title-sync.ts`: re-resolve after input goes quiet, every 20 s
while a title is missing, every 5 min otherwise).

**Underneath.**

- Scanners (`src/main/ai-vault/`, about 20k lines): one parser per agent format (Claude JSONL under
  `~/.claude/projects/<encoded cwd>/`, Codex `~/.codex/sessions`, OpenCode's `opencode.db` read in a
  worker thread, Devin's `sessions.db`, ...), run in a scanner child process with a persisted parse
  cache. SSH hosts are scanned through the relay (`src/relay/ai-vault-*.ts`).
- Index (`src/main/ai-vault-search/`, 6.5k lines, schema in `session-search-schema.ts`): SQLite at
  `<dataRoot>/ai-vault/session-search.sqlite` with tables `sessions`, `files` (byte offsets for
  append-only incremental reads, plus a retry state), `messages`, and one FTS5 table
  `messages_fts(user_text, assistant_text, tool_text, identifiers)` tokenized `unicode61
  tokenchars '_.-/+'` so paths and identifiers stay whole, with a split-identifier column beside
  them. Ranking is bm25 with column weights; the conversation scope zeroes the tool column. A query
  tries the phrase, then all words, then any word, with typo repair and the operators `repo:` and
  `path:`. Tool output past 3,072 characters per row is not indexed. No embeddings. The index is a
  cache: a schema bump drops and rebuilds it.
- Consent: off by default (`src/shared/ai-vault-search-settings.ts`, `{enabled, historyDays}`),
  because "building the index reads every transcript on the machine". Clearing deletes the
  database and never the transcripts.
- Paging: opaque cursors fenced by index generation and database identity; a `stale-cursor` answer
  sends the client back to page 1.
- Exposure: paired and relay clients get snippets but no file paths or resume commands
  (`docs/reference/agent-session-search-contract.md`).
- For agents: `orca search "<query>" [--scope conversation|all] [--agent] [--path] [--since]
  [--sort relevance|newest] [--fresh] [--json]` and `orca search --index-status`. Hits carry agent,
  title, cwd, branch, a snippet with `[[ ]]` marks and, on the same machine, the resume command and
  file path. No command returns a whole transcript. On a synthetic 10.5 MB corpus the text queries
  in the default scope take 5 to 9 ms at p50
  (`docs/reference/agent-session-search-query-tuning.md`).

**Good and bad.** One place to find and resume any agent's past session, the first-prompt card,
and explicit consent and exposure rules. The cost is 20 parsers for formats Orca does not own; the
two directories have 142 commits.

**Size.** About 37k lines of non-test code including the renderer panel.

**Marley today.** None. Rusty covers another need: curated brain pages with links, tags, timelines
and semantic search, tasks, memories, notes, and `search_conversations` over Rusty's own agent
conversations. A SessionEnd hook archives some projects' Claude Code transcripts into the brain.

### 2.13 Artifacts

**What.** `orca artifacts share ./report.html` (or `.md`) returns a public view link on Orca's
service; `update` and `unshare` find the artifact by the same local path; `list` pages;
`delete <id>`. In the app: Share as artifact on an open HTML or Markdown file, an Artifacts page,
and a share action for local HTML in the browser pane. Publishing needs a signed-in Orca account
and a device-wide setting that only a human can turn on; no CLI flag or RPC grants it, a denied
share fails with `artifact_sharing_disabled`, and the guide says not to retry.

**Underneath.** `src/main/artifacts/artifact-publisher.ts` (a queue per source file, a create-intent
store so a retry after a lost reply does not create a second artifact, edit tokens stored per
profile and never printed), RPC `artifacts.*` (`src/main/runtime/rpc/methods/artifacts.ts`), CLI
`src/cli/handlers/artifacts.ts`, UI `src/renderer/src/components/artifacts/`. Relative assets are
not uploaded.

**Size.** 1.2k lines main, 2.3k renderer, plus the cloud service outside this repo.

**Marley today.** None. Chad publishes with claude.ai artifacts.

### 2.14 Agent hooks and memory files

- Memory files: Orca leaves `CLAUDE.md` and `AGENTS.md` alone and writes none into projects
  (`docs/site/content/docs/agents/hooks-memory.mdx`; no writer in the source). It mirrors
  `~/.codex/AGENTS.md` into the Codex homes it manages per account, so those sessions still read it
  (`src/main/codex/codex-home-paths.ts`). Projects may carry `orca.yaml` (setup and archive hooks,
  per-workspace environment recipes) and a per-user `.orca/issue-command`
  (`src/main/issue-command-file.ts`). Repository settings can create a starter `.mcp.json`
  (`MCP_STARTER_CONFIG` in `src/shared/mcp-config.ts`).
- Status hooks: Orca writes managed entries into the user's global `~/.claude/settings.json` (and
  the Codex, OpenCode, Pi, Gemini and other equivalents) that run
  `~/.orca/agent-hooks/claude-hook.sh`. The script first prints `{}` ("Claude-compatible permission
  hooks fail closed on empty stdout", #14818), skips Claude background jobs (`CLAUDE_JOB_DIR`), and
  exits when `ORCA_AGENT_HOOK_PORT`, `ORCA_AGENT_HOOK_TOKEN` or `ORCA_PANE_KEY` is unset, so it does
  nothing outside Orca (`src/main/claude/hook-script.ts`).
- Claude events registered (`src/main/claude/hook-settings.ts`): SessionStart, UserPromptSubmit,
  Stop, StopFailure, SubagentStart, SubagentStop, TeammateIdle, PreToolUse, PostToolUse,
  PostToolUseFailure, PermissionRequest, PostCompact, and SessionEnd on versions that support it.
  The comments say why: SessionStart so a resumed idle session shows before the first prompt;
  StopFailure because API errors skip Stop; PostCompact because a manual `/compact` ends idle with no
  Stop; PreCompact deliberately not mapped to working, because an aborted compact emits it alone.
  The ingest takes `last_assistant_message` from the payload or reads the tail of `transcript_path`
  (`src/shared/agent-hook-listener/providers/claude-tool-fields.ts`).
- Transport: loopback HTTP with a per-launch token. The endpoint is also written to
  `<userData>/agent-hooks/endpoint.env` and re-sourced on every hook call, so a Claude Code session
  that outlives an Orca restart reaches the new server
  (`src/main/agent-hooks/server/server-runtime-env.ts`). WSL and SSH need relays.
- `orca agent hooks status|on|off` toggles the managed entries without a restart.

**Size.** `src/main/agent-hooks` is 11.6k lines, plus a hook service per agent.

**Marley today.** The plugin (`crates/marley_workbench/claude_plugin/marley/hooks/hooks.json`) hooks
Notification (`permission_prompt`, `idle_prompt`) and Stop. `notify.sh` returns a
`terminalSequence` holding an OSC 777 notify, which Claude Code writes to its own terminal, so the
notice travels in-band, works over SSH and needs no socket. It carries one line of text and no
structure.

### 2.15 No MCP server

Orca exposes nothing over MCP. It reads a repository's `.mcp.json`, `.cursor/mcp.json`,
`.claude.json` and `.claude/mcp.json` and lists their servers in repository settings with secrets
masked (`src/shared/mcp-config.ts`, `src/shared/mcp-server-inspection.ts`). Open issue #13079,
"Expose Orca as an MCP server for session and workspace control", asks for an `orca mcp` stdio
server built on the same runtime client, for MCP clients with no shell; it has no maintainer reply.
Screenshots show the cost: `orca screenshot --json` returns the image as base64 inside JSON, which
a model cannot look at unless it writes the file and opens it (`src/cli/handlers/browser-nav.ts`).

### 2.16 CLI versus MCP, capability by capability

| Capability | Orca (CLI) | Marley (MCP today) | Better fit for Marley | Why |
|---|---|---|---|---|
| Discover the surface | `--help`, `agent-context --json`, stubs plus `skills get` | `tools/list` with input and output schemas | MCP | The server describes itself each session, so descriptions cannot drift from the running build; Orca needed stubs, a generator and a parity test to get the same property |
| Say who is calling | env vars the CLI reads in the agent's shell | nothing | Both: env var in the terminal, forwarded by the bridge | A stdio MCP server inherits the agent's environment, so the bridge can read a terminal id and send it as a header |
| Read terminal output | `terminal read` (stream, `--screen`, cursors) | `terminal_blocks`, `terminal_read` | MCP | Marley's blocks already beat Orca's stream; add a rendered-screen read for TUIs |
| Type into a terminal with proof | `terminal send` with receipt stages | none | MCP write tool behind a grant | Claude Code permissions name each MCP tool, and Marley's grant table is deny-by-default; a CLI call falls under Bash rules that match command prefixes |
| Wait for idle or exit | `terminal wait` long poll with keepalives | none | MCP, short waits | A blocking call holds the caller's turn open; Claude Code's Bash tool stops at 10 minutes |
| Wake an idle agent with mail | pointer typed into its TUI | none | Terminal input, under either protocol | MCP gives a server no way to start a turn in an idle client; only input into the agent's terminal (or a session runtime such as rustal-harness) does |
| Spawn a worktree agent | `worktree create --agent --prompt` | none (TICKET-510 is UI) | MCP write tool sharing the rail's code path | The same creation path for human and agent |
| Images | base64 in JSON, or a saved path | `browser_look` returns an image block | MCP | The model sees the image directly |
| Scheduled and scripted runs | shell commands, prechecks, the automation scheduler | none | CLI | cron, systemd timers, prechecks and `jq` live in shells; a thin CLI over the MCP endpoint gives that reach without a second API |
| Agents in SSH terminals | relay forwards argv to the desktop | endpoint on loopback only | Bridge over an ssh reverse forward, later | Needs marley_remote |
| Agents with no shell | cannot reach Orca | Zed's agents reach `marley` as a context server | MCP | Issue #13079 asks Orca for exactly this |

### 2.17 The vault, skills and orchestration against Rusty and rustal-brain

- Memory and knowledge: Orca has no agent memory store. Worktree comments are the only durable
  notes agents write, and the vault holds raw transcripts. Rusty holds curated knowledge (brain
  pages with links and semantic search, memories, and decisions through `brain_ask`,
  `brain_decide` and `brain_follow_up`). Marley should not grow a memory store. A transcript index
  in Marley would be a disposable cache for "find that session", and anything worth keeping would
  go to Rusty through `brain_capture`.
- Skills: Orca solves distributing its own skills to a dozen agents and sharing skills through its
  cloud. Rusty solves Chad's own skills: a git store with scan, approve and reject, linked into
  Claude's skills directory. Marley needs neither distribution nor sharing; it needs one skill of
  its own in the plugin, and at most a read-only view of which skills each agent sees.
- Work state: Orca keeps Runs, Tasks and gates inside the IDE. rustal-brain keeps tickets, sprints,
  runs, phases, gates and evidence in Postgres, and rustal-harness keeps actor states and messages
  with delivery ids. Marley's plan keeps that authority outside Marley (three-prong plan D7, D11).
  What transfers from Orca is mechanism: the worker preamble contract, pointer delivery, a
  capability per dispatch, idempotent mutation receipts, and the `live`, `unverifiable`, `exited`
  verdicts.

## 3. Bring to Marley

Ranked by value for the effort. S is a day, M a few days, L a week or more.

1. **Terminal identity in the environment, forwarded by the bridge.**
   - Why: every agent-to-agent verb needs to know who is calling and which terminal is its own.
     Orca allocates the handle before spawn so the agent's first command already has it (2.5).
   - How: `marley_terminal` sets `MARLEY_TERMINAL_ID` (a UUID minted when the terminal is created
     and kept through session restore, not the gpui entity id) and `MARLEY_PROJECT` at spawn, and
     strips inherited values when a terminal is opened from inside an agent. The bridge reads them
     from its own inherited environment and sends a `Marley-Caller-Terminal` header; `marley_mcp`
     passes the caller to tools; `terminal_list` marks `self: true`; `terminal_blocks` and
     `terminal_read` default to the caller's terminal. Set `FORCE_HYPERLINK=1` as Orca does, so Node
     tools emit OSC 8 links (useful for TICKET-503).
   - Size: S.
   - Hard: Zed's Agent Panel agents have no terminal; give them a thread id or nothing. A shell that
     outlives a Marley restart holds a stale id; refuse it with a next step instead of guessing.

2. **Receipted input, waits and a screen read (plan D9 and slice C4; not in the sprint).**
   - Why: today an agent can watch another agent but not drive it. Orca's receipts separate "typed"
     from "the agent started working" (2.6).
   - How: `marley_mcp` write tools behind a `terminal.write` grant.
     `terminal_send {terminal, text, enter, request_id}` pastes with bracketed paste, sends Enter as
     a second write after a per-agent delay, and returns `{accepted, turn_started}`, with
     `turn_started` taken from the plugin's UserPromptSubmit event for that terminal (item 3).
     `terminal_wait {terminal, for: idle|exit, timeout_ms}` capped near 60 s, using plugin status
     and block state (a finished block is an exit). Refuse with `blocked_reason` when the screen
     shows Claude Code's trust or permission prompt. `terminal_screen` returns the visible grid as
     text. Build on `marley_fleet`'s `SendRequest`, `Receipt` and `DeliveryState`.
   - Size: M.
   - Hard: dialog detection is text matching per agent version. A typed prompt is an instruction
     channel, so keep it grant-gated and show each send in the target terminal's block list.

3. **Structured hook events in-band, for the approvals inbox and per-turn diffs (TICKET-508 and
   TICKET-509, queued).**
   - Why: the inbox needs each permission request's tool and input; per-turn diffs need turn start
     and end. The plugin sends one line of notification text today (2.14).
   - How: add SessionStart, UserPromptSubmit, PermissionRequest (matcher `*`), Stop, StopFailure,
     PostCompact, SubagentStart and SubagentStop to the plugin's `hooks.json`. Claude Code lets a
     hook's `terminalSequence` carry only OSC 0, 1, 2, 9, 99, 777 and BEL (Marley's own finding in
     `docs/planning/pipeline/completed/478-terminal-notifications.spec.md`), so carry the event as
     an OSC 777 with a reserved title, for example `777;notify;marley-event;<base64 JSON>`, where the
     JSON holds event, session id, tool name, a bounded input preview and `transcript_path`.
     `marley_terminal` consumes that title instead of showing a notification. This stays in-band
     and works in SSH terminals with no relay, where Orca needed loopback HTTP, an endpoint file,
     and WSL and SSH relays.
   - What Orca teaches 508: PermissionRequest carries the request; mark a worker "waiting on a
     human" only with evidence and treat no evidence as unknown (`observation.agentWait`); the
     screen-text blocked reasons (trust folder, update prompt, approval prompt) catch prompts that
     no hook reports; a permission hook must print `{}` because Claude-compatible permission hooks
     fail closed on empty stdout (#14818).
   - What Orca teaches 509: UserPromptSubmit opens a turn; Stop closes it, but StopFailure closes a
     turn that died on an API error and PostCompact closes a manual `/compact`, neither of which
     emits Stop; PreCompact must not open one. `last_assistant_message`, or the tail of
     `transcript_path`, labels the turn.
   - Size: S to M.
   - Hard: confirm that Claude Code honors `terminalSequence` for every event the plugin needs, not
     only Notification and Stop. The field is in Claude Code's hook schema but not its public docs,
     and `claude -p` drops it (Marley lesson L-claude-482). Bound the payload size.

4. **Worktree agents with handoff semantics and a checkpoint line (TICKET-510 and TICKET-511,
   queued).**
   - Why: worktree plus agent plus prompt is Orca's core flow (its parallel-agents recipe calls it
     "Orca's killer move"), and agents use it to hand work off, not only humans (2.7).
   - How: the + menu's worktree entry and an MCP write tool
     `worktree_create {project, name, base?, agent, prompt, request_id}` share one code path. Start
     the agent with the prompt in argv (`claude "<prompt>"`) so there is no paste race; record the
     parent project so the rail can show the relation; return the new terminal id; make the call
     idempotent on `request_id` so a lost reply does not create two worktrees. Add
     `project_comment {text, status?}` so an agent can write a one-line checkpoint on its rail row.
     For 511: delete the branch only when git proves it merged (`git branch -d`), never a branch that
     existed before the worktree, and run a configured archive hook before removal.
   - Size: M on top of 510.
   - Hard: Claude Code's folder-trust prompt on each new worktree path; either pre-trust the path
     or detect the prompt before sending anything.

5. **Session history per project, with resume (new crate, the rail).**
   - Why: resuming an earlier Claude Code session in the right project takes `claude --resume` and
     a guess at the id today; Orca makes it one click from the project.
   - How: a `marley_sessions` crate scans `~/.claude/projects/*/*.jsonl` and `~/.codex/sessions/`
     (Claude Code and Codex only) and matches each session's cwd to rail projects. A project row
     lists sessions with title, age, first prompt and Resume, which opens a Marley terminal running
     `claude --resume <id>` in that cwd; terminal tab titles follow the session title. Later, an
     FTS5 index on Orca's template (unicode61 with `_.-/+` token characters, one FTS table with
     user, assistant, tool and identifier columns, append-only incremental reads, off until Chad
     turns it on) and an MCP `sessions_search` tool that returns snippets and resume commands.
   - Size: M for the list and resume, L with the index.
   - Hard: Claude Code's transcript format is undocumented and changes; keep the parser narrow. Keep
     the index a cache; Rusty holds what is worth keeping.

6. **A mailbox with pointer delivery (slice C4, before or with rustal-harness).**
   - Why: the one Orca mechanism that lets an agent wake another idle agent and have it pull a
     message, with no push channel (2.8).
   - How: messages stored durably, in rustal-harness's messages once it is embedded and until then
     in a small SQLite table beside `marley_fleet`. Delivery: when the recipient terminal's agent is
     idle and not in a wait, `terminal_send` types one line ("1 Marley message. Call the marley tool
     messages_check."); `messages_check` returns the batch and `messages_ack` acknowledges it. `ask`
     sends a question and waits on the answer id, resumable. Reuse Orca's worker contract wording:
     report done once with an outcome, ask through the mailbox instead of a local question UI, check
     mail at checkpoints.
   - Size: L.
   - Hard: the plan puts messages in rustal-harness, so a Marley-only mailbox may be thrown away.
     The pointer must carry nothing but "check mail"; the body is data, never instructions.

7. **Agent-grade refusals and idempotent writes in `marley_mcp`.**
   - Why: an agent recovers from a refusal only when the refusal says what to do (2.4).
   - How: extend `tool_error` in `crates/marley_mcp/src/tools.rs` to
     `{result: "refused", code, reason, next_steps: [...], valid: [...]}`; an unknown tab or terminal
     id lists the valid ones; write tools accept `request_id` and replay the first outcome; when
     remote seats arrive, report `unverifiable` rather than `exited` for hosts a snapshot did not
     cover. Add a test that the plugin's text and any guide name only registered tools.
   - Size: S.

8. **A thin `marley` CLI over the MCP endpoint.**
   - Why: cron, systemd timers, shell prechecks and scripts need a CLI; Orca's automations and
     handoffs are shell-native.
   - How: `marley tools` and `marley call <tool> --json '<args>'`, generated from the registry and
     reading `mcp-endpoint.json`. A subcommand of the existing bridge is enough; no second API.
   - Size: S.
   - Hard: keep the registry the single source; do not grow per-tool flags.

9. **A `marley` skill as a stub, with the guide served by the server.**
   - Why: procedures such as "hand off to a worktree agent", "pick, fix, check" and "turn a
     recording into a Playwright test" change with Marley. A stub in the plugin plus a
     `marley_guide {topic}` tool (or MCP prompts) keeps them matched to the running build (2.11).
   - Size: S.

10. **Claude Code Agent Teams as Marley splits.**
    - Why: teammates would appear as Marley terminals with status instead of a tmux session inside
      one pane (2.9).
    - How: a Marley launch option sets `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`, a fake `TMUX` and
      `TMUX_PANE`, and a `tmux` shim that forwards the ~20 commands Orca implements to Marley through
      the MCP endpoint.
    - Size: M.
    - Hard: it tracks an experimental Claude Code feature, and rustal-harness owns a tmux server of
      its own; decide which one owns tmux.

11. **Scheduled agent runs.**
    - Why: recurring triage and review prompts. Orca's precheck, missed-run grace and refusal
      folding are the parts to copy (2.10).
    - How: later. The cheap path is systemd user timers calling `marley call worktree_create ...`
      (items 4 and 8) with the precheck in the unit, and a run log shown in the rail.
    - Size: M for timers and a log, L for an in-app scheduler.

12. **Clickable terminal ids in terminal output.**
    - Why and how: once ids exist (item 1), a printed terminal or message id focuses its terminal,
      as Orca's `task_...` links do.
    - Size: S.

## 4. Skip

- Orca's orchestration database (Runs, Tasks, Dispatches, gates, federation, legacy layers): about
  42k lines plus 69k lines of tests, and rustal-brain and rustal-harness already hold those roles in
  Marley's plan.
- Artifacts on a vendor cloud: public links through a third-party account; claude.ai artifacts
  cover Chad.
- Skill sharing through Orca Cloud bundles: Terraform, GCS and Postgres for a job Rusty's git store
  does.
- Installing skills with `npx skills`: a third-party npm CLI, network at install time, and a default
  that installs into every known agent.
- Parsers for 20 agents' transcripts: Marley's plans name Claude Code and Codex; start with those
  two.
- The Linear, computer-use, emulator and VM command families: integrations outside Marley's scope;
  an MCP server can add any of them.
- `agent-context --json`: `tools/list` already gives this.
- Launching every agent with full-autonomy flags by default: it contradicts the approvals inbox and
  Marley's deny-by-default grants.
- Managed hook entries in the user's global `~/.claude/settings.json`: Marley's plugin is scoped and
  removable.
- Pairing codes and the encrypted WebSocket for the CLI: remote control belongs to another report
  and is not needed for local agents.
- Orca's retired and legacy orchestration commands and compatibility receipts: history to avoid.

## 5. Open questions for Chad

1. MCP only, or MCP plus a thin `marley` CLI for scripts, timers and agents in SSH terminals?
2. A Marley-local mailbox now, replaced when rustal-harness is embedded, or wait for
   rustal-harness itself, paused since 2026-09-14?
3. Where should a transcript index live: in Marley as a disposable cache beside the rail, or in
   Rusty, which already archives some transcripts through the SessionEnd hook?
4. What permission mode should worktree agents start in: Orca's full-autonomy flags, or Claude
   Code's normal prompts routed to the new inbox?
5. Should Claude Code Agent Teams open as Marley splits through a tmux shim, given that
   rustal-harness also runs its own tmux server?
