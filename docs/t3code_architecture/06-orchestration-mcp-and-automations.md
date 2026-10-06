# T3 Code survey 06: MCP tools, orchestration and automations

Read from a local clone of `github.com/pingdotgg/t3code` at `17c0878941` (2026-10-06). Paths are relative to that
repo root unless they start with `crates/` (Marley).

## 1. Summary

T3 Code gives every provider session it starts one MCP server, `t3-code`, on the environment's own
HTTP port, behind a bearer credential minted for that session alone and revoked when the session
ends. The server carries 80 tools in eleven toolkits. The orchestrator toolkit lets an agent
delegate a task to a child agent on any installed provider (a Claude thread can hand work to
Codex), start top-level threads in a worktree it names, send to, wait on and interrupt other
threads, schedule recurring or webhook-triggered prompts, and ask the user for a secret through a
private card. A child's completion comes back to the parent as a one-line pointer, steered into
the parent's running turn or queued as its next one, and reading the result cancels the pointer.
Every write is capped by the caller's own permission and interaction modes, so an agent cannot
start or steer work broader than itself. Scheduled tasks run on a five-second scheduler with a
one-minute floor and a missed-run grace; webhook tasks verify HMAC signatures, cap bodies and
rates, keep a delivery log and render the request into the prompt through `{{body.path}}`
placeholders. After a server restart, an interrupted turn can continue with a note naming the
background work the restart cancelled. The repo's own rule ("Hit every surface") is that a
capability a user can trigger is a service method that an MCP tool and a scheduled task also
reach. Marley's MCP server already gives agents its terminals, blocks, ports and browser, and
Marley's plan gives agent hosting, messages and delegated work to rustal-harness. Worth taking:
(1) instructions in the `initialize` result telling agents when to use Marley's tools; (2) results
sized for Claude Code's MCP output limit, with paging and typed refusals; (3) an MCP tool that
starts a worktree agent through #510's path, capped by the caller's permission mode; (4) a pointer
typed into the launching agent when the launched one finishes, cleared once it reads the result;
(5) a continuation note for #540's resumed sessions; (6) a private secret card; (7) the "every
rail action is a tool" rule.

## 2. Features

### 2.1 One server, eighty tools, a credential per provider session

**What the user sees.** Nothing directly. An agent in a T3 thread finds tools named
`mcp__t3-code__*` (or the harness's normalized prefix), and T3's timeline shows each call with a
summary.

**How it works.**

- `apps/server/src/mcp/McpHttpServer.ts:765` merges eleven toolkits onto one Streamable HTTP
  endpoint, `/mcp`, protocol revision 2025-06-18, on the environment server's own port. Counted
  from `Tool.make(` in `apps/server/src/mcp/toolkits/*/tools.ts`: orchestrator 16, thread 17,
  preview 19, project 7, pullRequests 5, device 4, attachment 3, worktree 3, environment 2, html 2,
  previewControls 2.
- Before `ProviderSessionManager` opens a provider session it asks `McpSessionRegistry` for a
  credential (`apps/server/src/mcp/McpSessionRegistry.ts:124-150`): 32 random bytes, stored only as
  a hash, scoped to the environment, the thread, the provider instance and a fresh provider
  session id, with a capability set (`orchestration`, `worktree`, `pull-requests`, plus `preview`
  unless the user withheld browser access, plus `device` when granted). The scope's
  `requestNamespace` is the provider session id, so two callers reusing a `clientRequestId` cannot
  collide.
- A credential lives while its session shows signs of life: MCP traffic and every provider turn
  `touch` it, and one with no sign of life for 24 hours is pruned
  (`McpSessionRegistry.ts:82`). Stopping the session revokes it at once. The comment at
  `McpSessionRegistry.ts:78-80` says why the bound matters: `/mcp` sits outside the environment's
  own auth stack, so on a server reachable from other machines the token is the only guard.
- The auth middleware (`McpHttpServer.ts:99-125`) resolves the bearer to an `McpInvocationScope`
  and logs a rejected credential with its reason, because an agent that loses its credential
  otherwise loses the whole toolkit with nothing on the server to say why.
- Injection per provider (`docs/orchestration-v2/orchestrator-mcp-server.md`, "Provider
  Injection"): Codex app-server gets `-c mcp_servers.t3-code.url=...` and the token in an
  environment variable; the Claude Agent SDK gets an `http` server with an `Authorization` header
  and `mcp__t3-code__*` added to its allowed tools; Cursor's SDK and ACP agents get it in their
  session options; Pi, which has no MCP client, gets a generated extension that registers each
  tool under the same prefix.
- `McpInvocationContext.ts:29-34` already defines a second kind of caller, "An agent T3 Code did
  not launch, signed in through MCP OAuth", with a permission ceiling chosen when it was approved.
  At this commit nothing issues such a scope: the registry sets `client: undefined` on every
  credential (`McpSessionRegistry.ts:139`).

**Good.** One credential per provider session, hashed, with capabilities that follow the user's
settings, is the right grain: the server always knows which thread is calling and what it may do.

**Bad.** Eighty tool definitions go to every session, and some descriptions run past a thousand
characters (`delegate_task` at `toolkits/orchestrator/tools.ts:61`). The orchestration doc still
says "eleven orchestration tools"; the toolkit has sixteen.

**Size.** 9,676 non-test lines under `apps/server/src/mcp/`, of which `OrchestratorMcpService.ts`
is 2,353; contracts in `packages/contracts/src/orchestratorMcp.ts` (661) and
`threadMetadataMcp.ts` (122).

**Marley today: has part.** Marley's MCP server serves every agent on the machine through one
endpoint and one bearer that changes at each start ("Marley's MCP server", "What it serves, and
where"). The caller is named by the bridge's `Marley-Terminal`, `Marley-Project` and `Marley-Cwd`
headers (#520; `crates/marley_mcp/src/transport.rs:586`), which any holder of the bearer can set.
Grants are global: Marley grants `browser.write` and nothing else ("Grants and what keeps an agent
in check").

### 2.2 Who may do what: capabilities, the privilege ceiling and the live-run rule

**What the user sees.** An agent in Supervised mode that tries to start a Full access child, or to
steer a thread that runs with broader permissions, gets a refusal and tells the user why.

**How it works** (`apps/server/src/mcp/threadAccess.ts`).

- `readCaller` (line 42) loads the calling thread and takes its runtime and interaction modes as
  the caller's limits.
- `assertTargetWithinLimits` (line 83) refuses any change to a thread whose modes are broader than
  the caller's. `delegate_task`, `create_threads` and `t3_thread_send` apply the same rule to the
  modes they set: a child may stay equal or narrow (`full-access` to `approval-required`,
  `default` to `plan`) and never widen (`docs/orchestration-v2/orchestrator-mcp-server.md`,
  "Policy And Idempotency").
- `assertLiveCaller` (line 94) makes every mutation need the caller's live run: an agent whose turn
  ended, or whose thread was archived, cannot keep acting through a credential it still holds.
- `readFullAccessCaller` (line 124) gates what changes the environment itself (projects,
  preferences, launching outside a project): the caller must be a full-access, default-mode thread.
  `t3_thread_launch` also requires it (`toolkits/project/handlers.ts:48-55`). T3's default
  permission mode for new threads is Full access (`docs/user/permission-modes.md`), so most
  threads qualify.
- An agent may answer another thread's pending question (`t3_pending_request_respond`) but never a
  permission request: the tool says so and the service refuses it
  (`toolkits/thread/tools.ts:165-170`).
- Refusals are typed: `capability_denied`, `parent_not_active`, `runtime_mode_escalation_denied`,
  `thread_not_sendable` and ten more, each with a message that says what to pass instead ("Pass
  projectId: this MCP client is not running inside a T3 thread.", `threadAccess.ts:147`).

**Good.** The ceiling is the rule any agent-to-agent tool needs, and the split between questions
(an agent may answer) and approvals (only a human may) is the line Marley's question route (#570)
draws for its future manager.

**Marley today: has part.** Marley reads the caller's permission mode for one decision: with the
click consequence on, an agent with no permission prompt of its own waits before a consequential
click ("The click consequence", #571). `terminal_type` refuses to type at the shell's prompt or
into another agent CLI (#525). Marley has no tool that starts or steers another agent, so it has
no general ceiling yet.

### 2.3 Telling the agent: instructions with the tools

**What the user sees.** Agents use T3's browser and delegation tools without being told to, and do
not fall back to Playwright or a shell `git worktree add` when T3's tools exist.

**How it works.** `apps/server/src/provider/T3OrchestrationInstructions.ts` (111 lines) holds two
texts. `T3_CODE_ORCHESTRATION_INSTRUCTIONS` (line 3) says when to use `delegate_task` against a
native subagent, that `t3_thread_launch` and `create_threads` are only for threads the user asked
for, how to pass a schedule, to call `request_secret` and never ask for a secret in chat, how to
pick a workspace before launching, and to make "one bounded direct attempt" at a T3 tool before
concluding it is missing, since some harnesses attach MCP servers lazily.
`T3_CODE_BROWSER_TOOL_INSTRUCTIONS` (line 38) tells the agent to call `preview_status` first, then
`preview_open`, and to use another browser only when T3's tools are absent or refuse. Claude gets
the text as system prompt (`t3OrchestrationSystemPrompt`, line 109); providers with no system
prompt get it wrapped in `<t3_code_orchestration_instructions>` before the first user message;
ACP agents get it again whenever the tools or the interaction mode change.

**Good.** The rules sit next to worked examples, which is the lesson Orca's preamble recorded
(models skim trailing prose). The text says what a tool is not for as often as what it is for.

**Marley today: lacks.** Marley's `initialize` result carries protocol version, capabilities and
`serverInfo` and no `instructions` (`crates/marley_mcp/src/dispatch.rs:107-113`). The Claude Code
plugin ships hooks and the bridge and no skill (`crates/marley_workbench/claude_plugin/marley/`).
Orca's report 06 proposed a skill stub (its item 9); it was not built. An agent learns Marley's
tools from their descriptions alone.

### 2.4 Results sized for the agent

**What the user sees.** Large tool results arrive whole enough to use, with a note saying what was
left out and how to get it.

**How it works.**

- The comment at `McpHttpServer.ts:132-139` records two Claude Code behaviors T3 designs around:
  a result over Claude Code's MCP output limit is moved to a file and the agent gets a notice in
  its place, and when a result has both `structuredContent` and text blocks Claude Code shows the
  model the structured part. So T3 keeps a browser snapshot near 20 KB
  (`MAX_SNAPSHOT_TEXT_BYTES = 20_000`, line 140), cuts names, logs and page text in a fixed order,
  and lists what it omitted with the tool to reach it (`preview_evaluate`).
- `t3_thread_read` pages a long item with `itemId`, `textOffset` and `nextTextOffset`
  (`packages/contracts/src/orchestratorMcp.ts:339`, `:404`) and a timeline with `afterPosition`
  and `nextPosition`, so an agent reads a long transcript in pieces instead of getting the end of
  it.
- Retries are safe where it matters: a `clientRequestId` derives stable command, thread and
  message ids within the provider session, so a lost reply does not create a second thread or a
  second message (`orchestrator-mcp-server.md`, "Policy And Idempotency").

**Marley today: has part.** `terminal_read` returns at most 2,000 lines and 256 KiB with the end
kept and says whether the start was cut ("The tools"). 256 KiB is well over Claude Code's default
MCP output limit (25,000 tokens, `MAX_MCP_OUTPUT_TOKENS`), so a long build log goes to a file or is
cut by Claude Code, not by Marley, and there is no offset to read the rest. Refusals are
`{"result": "refused", "reason": ...}` with no code or next step
(`crates/marley_mcp/src/tools.rs:61`); Orca's report 06 item 7 proposed codes, next steps and
`request_id` replay, and they are still open.

### 2.5 Delegated tasks across providers

**What the user sees.** In a Claude thread the user says "have Codex review this", and a child
thread appears under the parent's Agents view, runs on Codex, and its result comes back into the
Claude conversation. Stop on the parent stops the child.

**How it works.**

- `orchestrator_capabilities` lists provider instances and their live models, from the same
  catalog as the composer, and marks those that cannot run a child (no V2 adapter, disabled,
  missing binary, signed out).
- `delegate_task` (`toolkits/orchestrator/tools.ts:61`) creates a child thread and run with only
  the task prompt and an optional role (`implementation`, `research`, `review`, `design`, `test`,
  `general`); the parent's history is not copied. Provider, model and modes inherit unless the
  call narrows them. `mode: "wait"` blocks up to `timeoutMs` and a timeout never cancels the
  child; `mode: "async"` returns at once. It needs an active parent run owned by the calling
  session and becomes the V2 command `delegated_task.request`.
- `task_status` reads the task from the parent's projection and refuses a task id from another
  parent. `task_cancel` stops the child like a user Stop (interrupts the turn, holds queued turns,
  ends PR watches) and then every task the child delegated.
- Finalization runs from persisted events, so a child that finishes during a server restart is
  still delivered; a stored `subagent_result` transfer makes finalization idempotent. A failed
  child reports its provider error before any progress text.
- The tool text steers review loops: each review round is a new `delegate_task` carrying the
  original brief, prior findings and open objections, with its own `clientRequestId`, and never a
  `t3_thread_send` to the old child.

**Good.** Cross-provider delegation is the one thing no single CLI does: Claude Code's subagents
run Claude models inside the same process. The child gets a prompt, not a transcript, which keeps
its context clean and its behavior predictable.

**Bad.** The orchestrator core it rests on is large:
`apps/server/src/orchestration-v2/Orchestrator.ts` alone is 10,939 lines.

**Marley today: lacks, and it belongs to the harness.** Marley starts no agent on an agent's
request. rustal-harness already has delegated work with explicit offer, accept and finish, and
parent-child lifetimes (its `docs/WORK.md`), and agent messages between its sessions (its
`docs/AGENT_MESSAGES.md`). Claude Code in a Marley terminal can run `codex exec` through Bash,
which gives a one-shot answer with none of the lifecycle above.

### 2.6 Completion delivered to the parent

**What the user sees.** The parent agent ends its turn after delegating. When the child finishes,
the parent wakes with a timeline row such as `Delegated task "Review auth" finished` and carries
on with the result.

**How it works.**

- When a child reaches a terminal state, the orchestrator offers a delivery to the parent
  (`Orchestrator.ts:994`, `offerDelegatedCompletionDelivery`). The message is a pointer, not the
  result: "Delegated task X reached a terminal state. Use task_status with taskId X to read the
  result." (`Orchestrator.ts:522-527`). It is steered into the parent's active turn where the
  provider supports steering and queued as the parent's next turn otherwise.
- Deliveries are grouped per parent run in a cohort. When the parent reads a result with
  `task_status` or an untruncated `t3_thread_read`, that task leaves the pending delivery; when
  none is left and the delivery has not started, its queued run is cancelled
  (`Orchestrator.ts:1940-2030`). An agent that already polled is not woken for nothing.
- Delivery is at least once, since the provider's acceptance and T3's receipt cannot commit
  together; recovery reuses the message id so a retried steer does not duplicate the timeline item
  (`apps/server/src/orchestration-v2/NotificationMailbox.ts:7-26`).
- The timeline shows the wake as a notification row, not as a user message:
  `apps/server/src/orchestration-v2/Notification.ts` turns one or more reports into "Delegated
  task ... finished", "3 of 5 delegated tasks failed: ...", or for background work the provider
  reported, `Command "cargo test" finished (exit 1)`.
- The tool text tells the agent to end its turn rather than poll or start a watcher: the
  notification will wake it.

**Good.** The pointer plus acknowledgement is a better form of Orca's mailbox pointer: the wake is
cancelled when it is no longer news, and the body stays out of the prompt.

**Marley today: lacks.** The nearest piece is Review notes to the agent (#522), which pastes only
when Claude Code idles at its prompt and refuses while it works or asks ("Review notes to the
agent"). rustal-harness has `message_send`, `message_inbox` and `message_receive` for its own
sessions, with one landing per delivery and one receipt.

### 2.7 Threads that agents start, send to, wait on and interrupt

**What the user sees.** "Open three threads, one per package, and fix the lint in each" produces
three threads in the sidebar, marked as created by an agent.

**How it works.**

- `create_threads` makes one to twenty ordinary threads sharing the caller's project, branch and
  worktree (`packages/contracts/src/orchestratorMcp.ts:246`); each may override provider, model
  and modes within the ceiling.
- `t3_thread_launch` (`toolkits/project/tools.ts:102`) makes one thread bound to a workspace before
  its agent starts: `worktree` with `baseRef`, `branch` and `startFromOrigin`, `existing_worktree`
  with a path, or `root`, which is also the default and does not inherit the caller's worktree.
  `scratch: true` launches with no project, in a folder of its own. The instructions say why the
  binding matters: an agent told to run `git worktree add` in its prompt leaves T3's record
  pointing at the old folder.
- `t3_thread_send` sends in one of four modes: `auto` (start an idle thread, steer a steerable
  turn, else queue), `queue`, `steer` and `restart` (interrupt and start again).
  `t3_thread_wait` waits for a run to end and a timeout never interrupts it. `t3_thread_interrupt`
  stops the newest interruptible run.
- Provenance is kept: threads and user-role messages an agent made carry `createdBy: "agent"` and
  `creationSource: "mcp"`, so the UI tells an agent's message from the user's.

**Bad.** `t3_thread_launch` has no idempotency key; the instructions tell the agent to list threads
before retrying a launch whose reply was lost.

**Marley today: lacks.** New Agent in Worktree (#510) makes a worktree, writes
`branch.<b>.base`, copies `.worktreeinclude` files, offers the setup command, gives the worktree a
port slot and starts the agent with the prompt on its command line ("Worktree agents"). Only the
`+` menu reaches it. Orca's report 06 item 4 proposed the same path as an MCP tool; that half was
not built.

### 2.8 Reading other threads, their queues and their questions

**How it works.** `t3_thread_list` and `t3_thread_search` find threads in a project;
`t3_thread_read` reads any thread's timeline in a `messages` or `activity` view. The thread toolkit
(`toolkits/thread/tools.ts`) adds queue tools (list, read, edit, cancel, reorder, promote a queued
message to a steer), pending-question tools (list, read, answer), a thread's model selection,
forks and merge-back from a fork, and `t3_thread_organize` (pin, snooze, settle, archive, mark
unread).

**Marley today: has part.** An agent reads every terminal's blocks and screen (`terminal_list`,
`terminal_blocks`, `terminal_read`, `terminal_screen`), and `fleet_snapshot` lists every Claude
Code session in Marley's terminals with what it is doing (#547). Claude Code keeps its own message
queue and transcripts, so the queue and history tools have no counterpart to build in Marley.

### 2.9 Workspaces, projects and the environment from MCP

`t3_worktree_status`, `t3_worktree_list` and `t3_worktree_handoff` (move the calling thread into a
new worktree and run the project's setup script there); `t3_project_list`, `_read`, `_create`
(including a new repository from a title alone), `_update` (which can rewrite the project's
scripts, including `runOnWorktreeCreate`), `_delete`, `_clone`; and `t3_environment_read` and
`t3_environment_preferences_update`. Project and environment changes need a full-access caller
(`toolkits/project/handlers.ts:35-40`). Project scripts from `t3.json` are readable and writable
through these tools; no tool runs one.

**Marley today: lacks.** Marley has no project or settings tools; agents read `.zed/marley.json`
and run commands themselves.

### 2.10 Pull request tools and watches

`link_pull_request`, `unlink_pull_request`, `list_thread_pull_requests`, `watch_pull_request` and
`unwatch_pull_request` (`toolkits/pullRequests/tools.ts:256-334`). A watch checks every two minutes
and wakes the agent through the same delivery path when a check fails, required checks pass,
someone else comments or reviews, or the branch starts to conflict; the tool text says a wake "is
news, not a merge decision". Report 03 covers the watch itself.

**Marley today: has part.** The project header shows the branch's pull request and its state from
`gh`, read every two minutes (#531). No agent tool reads it and nothing wakes an agent.

### 2.11 Secrets the agent never sees

**What the user sees.** The agent says it needs GitHub's webhook secret; a card appears in the
thread with a password field. The value never shows in the transcript.

**How it works.** `request_secret` (`toolkits/orchestrator/tools.ts:151-163`) opens the card and
waits. The value stays in the app; the agent gets a `secretRef` it can pass, once, to a tool that
takes one (today `schedule_task`'s `signature.secretRef`). The tool text forbids asking for a
secret in chat or inventing one.

**Marley today: has part.** When ssh in an agent's terminal needs a passphrase, Marley shows Zed's
password dialog naming the agent and project and hands the answer to ssh alone (#596). System One's
key goes to the keyring through its own Set Key ("Turning it on"). Rusty's Secrets tab behind its
PIN is queued (TICKET-667). Nothing lets an agent ask for a value it can use without reading it.

### 2.12 Scheduled tasks

**What the user sees.** Settings → Scheduled tasks lists recurring prompts per environment and
project, with run now, pause, resume and delete; an agent can create one with `schedule_task`
("every weekday at 09:00, triage new issues").

**How it works.**

- One scheduler (`apps/server/src/scheduling/Scheduler.ts:60`) ticks every five seconds over every
  registered source; a slow source cannot block another or overlap itself.
- Schedules are an interval (floor one minute, enforced again on old rows at
  `apps/server/src/scheduledTasks/Schedule.ts:20`), a fixed time with optional weekdays in the
  environment's time zone, or a webhook. A fixed-time run more than ten minutes late is skipped
  and re-aimed at its next slot (`Schedule.ts:81`), so a machine that wakes at noon does not fire
  the 09:00 triage.
- `runTask` (`ScheduledTaskService.ts:710`) refuses to overlap a task with itself: a scheduled
  fire while the last run is going is dropped, a manual one is refused with a message. It rereads
  the row before dispatch so a pause or delete in between wins, and derives the command and message
  ids from the task, the time and the trigger, so a fire is never dispatched twice.
- A task bound to a thread posts into it in `queue` mode: "Scheduled prompts must not interrupt
  tools in the bound thread" (`ScheduledTaskService.ts:826`). By default an agent-created task
  binds to the thread that created it, so an orchestrating agent sees every trigger and can
  dedupe against work in flight; unbound tasks launch a fresh thread per run.
- `run_scheduled_task_now` lets an agent fire one; it needs a full-access caller.

**Size.** 2,213 non-test lines in `apps/server/src/scheduledTasks/`, 64 in `scheduling/`, contracts
339.

**Marley today: lacks.** Claude Code's `/loop` repeats a prompt inside a running session and its
`/schedule` routines run in Anthropic's cloud, not in a terminal on the box. Orca's report 06
item 11 left scheduled runs for later (systemd timers through a CLI); not built.

### 2.13 Webhook automations

**What the user sees.** A task with the trigger On webhook has a URL to paste into GitHub. Each
request starts a run whose prompt is the task's template filled from the request; Deliveries shows
recent requests and the prompt each produced.

**How it works.**

- The route is `/api/hooks/:hookId/:token` (`ScheduledTaskService.ts:58`): the token in the path
  is the URL's secret, and Rotate issues a new one. A public URL needs T3 Connect's managed tunnel;
  the relay can hold requests for up to 24 hours while the machine is offline
  (`infra/relay/src/hooks/HeldHooks.ts`, `docs/user/project-settings.md`).
- Limits (`ScheduledTaskService.ts:59-68`, `webhookRoute.ts:18`): 1 MiB bodies, checked on the
  header and on the reader for chunked bodies; 60 accepted requests a minute per task, enforced on
  the machine as well as the relay "because the tunnel hostname is public too"; 20 runs queued per
  task; 50 deliveries kept, with body and rendered prompt each capped at 64 KiB.
- Signatures (`webhookVerification.ts`): HMAC-SHA256 over the raw body bytes, hex or base64, with
  an optional prefix (`sha256=` for GitHub), compared in constant time over hashes so length does
  not leak.
- Templates (`webhookTemplate.ts:1-21`) are `{{path}}` lookups only, with no conditionals or
  filters "so it never grows into a template language": `{{body.a.b.0}}`, `{{headers.name}}`,
  `{{query.name}}`, `{{body}}`, `{{request}}`. Credential-named headers and query values are
  redacted wherever the whole set renders (`CREDENTIAL_NAME`, line 40); naming one gives its raw
  value.
- A delivery is keyed by its id, so it never dispatches twice; a queued delivery whose task was
  paused, replaced or switched to another trigger is skipped with the reason logged.

**Good.** The hardening is thorough and each limit has a stated reason.

**Bad.** The request body reaches the prompt as plain text with no marking as untrusted input, and
a task an agent creates binds by default to that agent's thread, which in T3 usually runs with Full
access. With the signature check off, anyone holding the URL writes into a full-access agent's
prompt.

**Marley today: lacks.**

### 2.14 Restart continuation and the cancelled-work note

**What the user sees.** With Settings → General → Continue threads after restarts on (off by
default), a thread that was mid-turn when the server stopped carries on after it starts again. A
thread whose turn had ended but whose background work was cut gets a note on its next turn.

**How it works.** `apps/server/src/orchestration-v2/RestartContinuation.ts` resumes only an
unfinished root turn; Codex resumes it natively, other providers get the prompt "Continue where you
left off." (line 22). Background work the restart cancelled (subagents, background shell commands,
monitors) is collected from the run's turn items and reported to the agent as "Note: the T3 server
restarted, and this background work was cancelled before it finished. It will not report back:"
followed by one line per item (`RestartBackgroundNote.ts:90-97`). Queued runs that never started
are held behind the resumed one.

**Good.** The agent is told that the work it is waiting for will never report, instead of waiting
on a background shell that no longer exists.

**Marley today: has part.** Quit Marley while Claude Code runs and the next launch runs
`claude --resume` with the same session in its starting folder (#540). The conversation comes back;
an interrupted turn does not continue, and nothing tells Claude Code that its background shells and
monitors died with the old process.

### 2.15 Background-work notices

`Notification.ts` also names background work the provider reported finishing while no turn ran:
`Command "npm run build" finished (exit 0)`, `Monitor "deploy log" reported new output`, or "4
background tasks failed". The row opens the subagent's thread when there is exactly one.

**Marley today: has part.** Notifications say what happened for Claude Code, Codex and OpenCode
(#538, #552), and a row counts running subagents (#519). Marley has no notice for a background
shell's end inside an agent.

### 2.16 The bridges

ACP agents must support stdio MCP servers while HTTP support is uneven, so `t3 acp-mcp-bridge`
(`apps/server/src/mcp/AcpMcpStdioBridge.ts`) relays JSON-RPC lines to the HTTP endpoint and
replays the session id and protocol version, and `AcpMcpOverAcpBridge.ts` tunnels MCP through ACP
itself. When an ACP agent accepts the server but hides its tools, the instructions give a shell
fallback (`acp-mcp-call <tool> '<json>'`).

**Marley today: has.** Marley's Python bridge does the stdio-to-HTTP job, answers `initialize`
itself when Marley is not running and sends `notifications/tools/list_changed` when Marley comes
and goes ("The bridge").

### 2.17 "Hit every surface"

`AGENTS.md:65-76` lists the places a change must reach before it is done. One line reads: "A
capability a user can trigger is usually one an agent should reach through MCP tools, and
scheduled tasks run the same paths. That only works when it is a service method, not handler
code." `AGENTS.md:153` adds that a WebSocket handler, HTTP route or MCP tool decodes input, calls
one service method and maps errors. Release 0.0.45 carried the doc change "server features are
services, and handlers stay thin" (#14613). Its other entries in the list ask for the way out of
every way in ("Snooze needs unsnooze") and a decision per provider.

**Marley today: has part.** Marley's MCP tools arrive ticket by ticket. Several rail actions have
no tool: worktree Review, Merge and Remove (#511, #589), port Stop and Restart (#521, #615), launch
configs (#527), Archive and Delete Thread (#605, #616), Send Review to Agent (#522).

### 2.18 Against rustal-harness and Rusty

- Agent hosting, delegated work, messages between sessions, a manager that answers for the owner,
  and the coordinator loop are rustal-harness's (Chad, 2026-10-02: harness-side work goes to the
  harness; Marley keeps the drawing side). T3's `delegate_task`, completion cohorts, thread send
  and wait, and scheduled sessions map onto that, and the harness already has offer, accept and
  finish with parent-child lifetimes, and agent messages with delivery ids and receipts.
- What touches Marley's own objects stays Marley's: the rail, its terminals, #510's worktree
  agents, the Browser tab, ports. An agent-callable launch of a worktree agent, a pointer typed into
  a Marley terminal, the size of Marley's tool results, the instructions its server sends, and
  #540's resume belong here.
- Secrets have a home coming in Rusty (TICKET-667). Decisions and follow-ups are Rusty's; T3 has
  nothing like them.

## 3. Bring to Marley

Ranked by use to Chad, who runs Claude Code and Codex in Marley's terminals. S is a day, M a few
days, L a week or more.

1. **Instructions in Marley's `initialize` result.**
   *Why.* Marley serves about thirty tools, and agents learn them only from per-tool descriptions.
   A short text tells Claude Code and Codex which tool to reach for: `terminal_blocks` and
   `terminal_read` before asking the user to paste output, `terminal_run` for a command whose block
   Chad should see, `browser_tabs` then `browser_snapshot` before driving a page, the Browser tab
   instead of starting Playwright or another Chromium, `browser_pick` when a line names a pick,
   never typing into another agent. T3's two texts are the model: rules beside one example each,
   and what each tool is not for (`apps/server/src/provider/T3OrchestrationInstructions.ts:3-49`).
   *Seam.* `crates/marley_mcp/src/dispatch.rs`, `initialize_result`; the bridge replays the
   server's `initialize` and so forwards the field.
   *Size.* S.
   *Hard.* Confirm on the installed Claude Code and Codex that each puts the MCP `instructions`
   field into the model's context; if one does not, the plugin carries the same text as a skill
   (Orca's report 06 item 9). Keep it under about 2 KB, and only name tools that are listed (the
   find tools come and go with System One's settings).

2. **Results that fit Claude Code's output limit, with paging and typed refusals.**
   *Why.* A `terminal_read` of a long cargo build can return 256 KiB, more than twice Claude Code's
   default MCP limit of 25,000 tokens, so Claude Code cuts it or moves it to a file and the agent
   loses the part it needed. T3 bounds results near 20 KB, says what it left out, and pages long
   text with an offset.
   *Seam.* `crates/marley_mcp`: `terminal_read` gets a default near 20 KB, `offset` and
   `next_offset` (lines or bytes), and `omitted` notes; `browser_snapshot` and `browser_console`
   get the same treatment. Refusals become `{result, code, reason, next_steps}` with the valid ids
   listed where an id was wrong, and write tools take a `request_id` that replays the first
   outcome. This is Orca's report 06 item 7 plus T3's paging (`McpHttpServer.ts:132-140`,
   `orchestratorMcp.ts:339`, `:404`).
   *Size.* S.
   *Hard.* Keep the end of the output first, since errors sit there, and page backwards.

3. **An MCP tool that starts a worktree agent through #510's path.**
   *Why.* Chad's own pattern, "have another agent do X in a worktree", today needs him at the `+`
   menu. Claude Code could call `agent_launch` instead, and the new agent would show in the rail
   with its worktree, port slot and drift chip as if Chad had started it.
   *Seam.* `crates/marley_mcp` (a write tool behind a new `agents.launch` grant) calling the
   function `crates/marley_workbench` uses for New Agent in Worktree. Arguments after T3's
   `t3_thread_launch`: `agent` (`claude` or `codex`), `workspace` (`worktree` with `base` and
   `branch`, `existing_worktree`, or `root`, with `root` as the default and never the caller's
   worktree implied), `prompt`, `request_id` (T3 lacks one; Marley should not), and `setup` (the
   offered install command). It returns the terminal id. Two rules from T3: the launched agent's
   permission mode is never broader than the caller's (`threadAccess.ts:83`; Marley reads the
   caller's mode as the bypass chip does, #532), and the caller must be in a turn. The bridge
   should send a per-terminal token Marley mints at spawn (as `MARLEY_TERMINAL_ID` is minted), so
   one terminal cannot claim to be another; T3's per-session credential is the model
   (`McpSessionRegistry.ts:124-150`).
   *Size.* M.
   *Hard.* Claude Code asks before each MCP call by default, so in ask mode Chad approves every
   launch in the caller's terminal; a bypass caller launching a bypass child needs a card under the
   caller's terminal, as #525's write card does. The open question below asks whether this tool is
   Marley's or the harness's.

4. **A pointer typed into the launching agent when the launched one finishes.**
   *Why.* Without it the caller polls `terminal_blocks` or Chad relays the result by hand. T3's
   delivery is the pattern: a one-line pointer, typed only into an idle prompt, withdrawn once the
   caller has read the result.
   *Seam.* `crates/marley_workbench`: when a terminal started by item 3 reports `Stop` (plugin
   events, #519) or idle (Codex App Server, #650), and the caller's Claude Code is idle at its
   prompt (#522's `ready` test), type "Agent `agent/lint` (terminal …) finished. Read it with
   `agent_status`." and Enter; a new `agent_status` tool returns the state, the last message from
   the plugin's events and the worktree's diff stat, and marks the pointer read so a pending one is
   dropped. Text after `Orchestrator.ts:522-527` and `Notification.ts`.
   *Size.* M, after item 3.
   *Hard.* Typing into an agent is an instruction channel: the pointer carries ids only, never the
   child's words. Never type while the caller works or asks (#508's rule). rustal-harness already
   delivers messages between its own sessions; this covers only agents Marley started.

5. **A continuation note for #540's resumed sessions.**
   *Why.* After a Marley restart, a Claude Code that was mid-turn comes back idle with no word about
   the background shells and monitors that died with it, and may wait on output that will never
   come.
   *Seam.* `crates/marley_workbench`'s resume path (#540) and the plugin's hook events (#519): if
   the session's last event opened a turn with no `Stop`, show a chip on the row, "Interrupted ·
   Continue", which types T3's prompt ("Continue where you left off.") plus, when the plugin saw
   background commands start without an end, the note in `RestartBackgroundNote.ts:90-97`. Off by
   default as in T3 (`docs/user/updating.md`).
   *Size.* S for the chip and prompt; the background list depends on what Claude Code's hooks
   report for background shells, to be checked.
   *Hard.* A turn that ended on an interrupt fires no hook (Orca's finding), so "mid-turn" is a
   guess; the chip asks rather than acts.

6. **A secret card.**
   *Why.* When an agent needs an API key for a command, Chad pastes it into the conversation today,
   and it lands in Claude Code's transcript.
   *Seam.* A `secret_request {name, purpose}` tool in `crates/marley_mcp` that shows Zed's
   password dialog over the caller's terminal, headed with the agent and project as #596's ssh
   dialog is, and returns a one-use reference; `terminal_run` takes `env: {NAME: ref}` and passes
   the value into that command's environment only, and #516's redaction gets the value as a
   literal pattern for the terminal's life. T3's tool text is at
   `toolkits/orchestrator/tools.ts:151-163`.
   *Size.* M.
   *Hard.* Where the value lives (keyring, or Rusty's Secrets, TICKET-667) and for how long.
   `terminal_run` types at the shell's prompt, so the variable must reach the command without
   appearing on the typed line or in shell history (#553 keeps agent commands out of history).

7. **A pull request watch an agent can start.**
   *Why.* "Babysit this PR" without a sleeping agent: Marley already reads the branch's pull request
   through `gh` every two minutes (#531).
   *Seam.* `pr_watch` and `pr_unwatch` tools in `crates/marley_mcp`; wakes through item 4's pointer
   on a failed check, required checks passing, someone else's comment or review, or a new conflict;
   T3's tool text at `toolkits/pullRequests/tools.ts:300-313` ("a wake is news, not a merge
   decision"). Report 03 covers the watch's checks.
   *Size.* M, after item 4.

8. **"Every rail action is a tool", as a Plan-phase question.**
   *Why.* T3 holds itself to it and its agents can do what its users can. Marley's agents cannot
   review, merge or remove a worktree, stop a port's server or run a launch config, though Chad can.
   *Seam.* The Plan template (`docs/planning/pipeline/`): each ticket that adds a rail or menu
   action names the MCP tool that reaches the same function, or says why none should. Not
   CONSTITUTION; T3 calls its own list "good defaults".
   *Size.* S, then a small cost per ticket.

9. **Scheduled prompts and webhooks, if the harness takes them.**
   *Why.* A weekday-morning triage prompt, or "when CI fails, wake the agent", without Chad
   starting it.
   *Seam.* rustal-harness, which hosts unattended sessions; Marley would list tasks and deliveries
   and open a run's session. T3's design to copy: queue mode into a bound session so a trigger never
   interrupts a tool; no overlap with the task's own last run; a one-minute floor; a ten-minute
   grace for fixed times, then skip to the next slot; ids derived from task, time and trigger;
   webhook HMAC over raw bytes in constant time; token in the path with rotation; body, rate and
   queue caps; a delivery log with the rendered prompt; `{{path}}` lookups only. Add what T3 lacks:
   mark the request as untrusted input in the prompt, and refuse webhook tasks bound to an agent
   that runs without prompts.
   *Size.* L.
   *Hard.* A GitHub webhook needs a public URL; the box has none without Tailscale Funnel, and
   item 7 covers most of the use.

## 4. Skip

- The event-sourced orchestrator, thread store and command receipts (`Orchestrator.ts`, 10,939
  lines, and `ThreadManagementService`): Marley has no thread store, and rustal-harness keeps the
  journal and receipts in Marley's plan.
- `delegate_task` and `create_threads` as Marley tools: cross-provider delegation with lifetimes is
  the harness's delegated work; batches of agents sharing one checkout collide, and #510's
  worktrees are the right shape.
- Agent tools that create, clone, delete or reconfigure projects and change environment settings
  (`t3_project_*`, `t3_environment_preferences_update`): large blast radius for little use; an agent
  that can rewrite a project's `runOnWorktreeCreate` script can plant a command for the next
  worktree.
- Agents organizing the user's view (`t3_thread_organize`: pin, snooze, settle, mark unread) and
  editing queues: T3's GUI owns those; Claude Code keeps its own queue in the terminal.
- Fork and merge-back context transfers: Claude Code and Zed's Agent Panel own conversation state.
- T3 Connect's relay holding webhooks while offline: a hosted service Marley does not run.
- The ACP shell fallback (`acp-mcp-call`): a workaround for ACP agents that hide injected tools;
  Marley's terminal agents load MCP servers directly.
- `html_render`, `preview_*` and `device_*`: reports 02 and 05.

## 5. Open questions

1. Should agents be able to start agents in Marley (item 3), and is that tool Marley's, over #510's
   path, or rustal-harness's? *Default: Marley's, since the agent lives in a Marley terminal and
   the rail; the launched agent never runs broader than its caller; nothing built until Chad
   agrees.*
2. Should Marley type wake pointers into idle Marley-terminal agents (item 4), or leave every
   message between agents to the harness? *Default: Marley types pointers only for agents it
   launched through item 3.*
3. After a restart, should Marley continue an interrupted Claude Code turn by itself? *Default: no;
   a chip on the row offers it, as T3's setting is off by default.*
4. Where does a secret an agent asked for live, and for how long (item 6)? *Default: nothing built
   until Rusty's Secrets (TICKET-667) lands; then Rusty holds it.*
5. Who owns scheduled prompts and webhooks: rustal-harness, systemd timers through a CLI (Orca's
   suggestion), or nobody yet? *Default: nobody yet; Marley builds no scheduler.*
6. Does "every rail action is a tool" go into the Plan template or CONSTITUTION? *Default: the Plan
   template, as a question each ticket answers.*
