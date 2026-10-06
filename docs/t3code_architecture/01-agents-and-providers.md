# T3 Code survey 01: agents, providers and sessions

Read from a local clone of `github.com/pingdotgg/t3code` at `17c0878941` (2026-10-06). Paths are relative to that
repo root unless they start with `crates/` (Marley). Line counts are non-test TypeScript.

## 1. Summary

T3 Code never runs an agent's TUI. Its server drives each provider headless through the most
structured channel the vendor offers: Claude Code through `@anthropic-ai/claude-agent-sdk`, Codex
through `codex app-server`, Cursor through `@cursor/sdk`, Pi over `pi --mode rpc`, OpenCode through
its own server, and Grok, Antigravity, Devin and any registry agent over ACP. Every adapter
normalizes into one event-sourced thread model (app threads, runs, provider threads, execution
nodes) and declares capability flags, so the orchestrator can switch model or provider inside one
thread, fork, rewind with file restore, continue after a restart, and resume after a usage limit,
with a written policy for each provider that lacks a primitive. Around that sit provider instances
(several accounts per provider), four permission modes with Full access as the initial default, a
Usage page built from the CLIs' own transcripts, version advisories and updates for the CLIs, an
importer for past Claude Code and Codex conversations, and headless `claude -p` calls for titles,
branch names and commit text. The adapters alone are about 39,900 lines and `apps/server/src/provider`
another 40,400. Marley's model is the opposite: the vendor's TUI in a terminal, watched through
hooks, the Codex App Server and a plugin. Much of what T3 builds the TUIs already give Chad (model
switching, `/rewind`, a message queue, `/compact`, `/goal`, skills), and Zed's Agent Panel has
queueing, steering and rewind for ACP agents. What is worth taking: (1) a Limited state on an agent
row with a continue-at-reset, for Codex first; (2) restart continuation that also covers Codex and
continues a turn the quit cut off; (3) handing a session to the other agent with T3's budgeted
history selection; (4) a per-project list of past Claude Code and Codex sessions with search and
Resume; (5) answering Codex's own questions and app-access requests from the inbox; (6) a Usage tab
read from the same transcripts; (7) reverting a turn's files from its Turns row, under T3's
isolation rule.

## 2. Features

### 2.1 How each provider runs

**What the user sees.** One model picker per thread that lists every enabled provider instance and
its models. Settings → Providers enables Codex, Claude, Cursor, Grok Build, OpenCode, Antigravity
and Pi, plus any agent from the ACP Registry or a local ACP command (`docs/user/install.md`,
`docs/user/providers-acp.md`). No terminal shows an agent.

**How it works.** One adapter per driver under `apps/server/src/orchestration-v2/Adapters/`, each
implementing the contract in `ProviderAdapter.ts` and emitting a `ProviderCapabilities` record at
session start (`docs/orchestration-v2/provider-capability-system.md`).

| Provider | Transport | Adapter (lines) |
|---|---|---|
| Claude Code | Agent SDK `query()`, which runs the `claude` binary (`pathToClaudeCodeExecutable`) | `ClaudeAdapterV2.ts` (8,000) |
| Codex | `codex app-server` JSON-RPC; bindings regenerated per Codex release | `CodexAdapterV2.ts` (6,935) |
| Cursor | `@cursor/sdk` local agent runtime | `CursorAdapterV2.ts` (2,684) |
| OpenCode 1.x | a T3-owned chat server per thread | `OpenCodeAdapterV2.ts` (3,798) |
| OpenCode 2.x | one server per instance, a `t3-code-<thread>` MCP entry per thread | `OpenCode2AdapterV2.ts` (4,271) |
| Pi | `pi --mode rpc`, line-delimited JSON (`PiRpc.ts:4`) | `PiAdapterV2.ts` (3,017) |
| Grok, Antigravity, registry agents | ACP v2 preview, else v1 | `AcpAdapterV2.ts` (8,010) and three thin ones |
| Devin | ACP, its commands run in T3's own terminals | `DevinAcp.ts` |

The Claude adapter builds its options in `makeClaudeQueryOptions`
(`ClaudeAdapterV2.ts:805-934`): the `claude_code` system-prompt preset with T3's runtime note and,
when T3's MCP server is attached, its orchestration instructions appended
(`apps/server/src/provider/RuntimeInstructions.ts`, `T3OrchestrationInstructions.ts`);
`includePartialMessages`; summarized thinking; `canUseTool` for approvals and `onUserDialog` for
questions; `resume` or a fixed `sessionId`; `resumeSessionAt` for rollback; `forkSession` for forks.
Each thread gets its own authenticated HTTP MCP endpoint named `t3-code`, with a 65-minute tool
timeout so its wait tools can block, and read-only sandboxes pre-approve only the read-only T3 tools
(`claudeMcpQueryOverrides`, `ClaudeAdapterV2.ts:966-994`). Codex turns use `turn/start`,
`turn/steer`, `turn/interrupt`, `thread/resume` and `thread/fork` (`CodexAdapterV2.ts:6236`, `:6844`).

Capabilities drive every feature: `supportsActiveSteering`, `canRollbackThread`,
`providerCanRollbackConversation`, `supportsDeltaHandoff` and so on. A missing one has a declared
fallback: steering becomes interrupt-and-restart, rollback restores files and starts a fresh
provider thread, a fork becomes a portable handoff. Claude declares active steering and queued
messages but no subagent thread ids (`ClaudeAdapterV2.ts`, capability block near line 200).

**Good.** The capability record plus a written degradation policy per feature is a clean way to
support providers of uneven depth. Codex is treated as the reference protocol and weaker providers
are adapted to it with app-owned ids. **Bad.** The cost is size: two adapters of 8,000 lines, and
release notes in which most provider work is catching up with a CLI's latest protocol (v0.0.45:
"regenerate protocol bindings for Codex 0.159", "Grok CLIs older than 1.0.13 are marked broken").

**Marley today: has part.** Any CLI runs in a Marley terminal; four are recognized (Agent CLIs in
terminals, #440). Codex can run against its own App Server with Marley joined as a second client
(Permission modes, "Codex's own App Server", #650). Zed's Agent Panel runs ACP agents, the ACP
Registry included (`crates/project/src/agent_registry_store.rs:20`).

### 2.2 Provider instances and accounts

**What the user sees.** Several instances of one provider, each with a name, binary path, launch
arguments, environment variables (values marked Sensitive are never shown again) and, for Claude, a
`CLAUDE_CONFIG_DIR`; for Codex, a `CODEX_HOME` and an optional shadow home. Recipes cover a second
Claude login, OpenRouter (`ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN`, an empty
`ANTHROPIC_API_KEY`) and Claude Code Router (`docs/user/providers-claude.md`). An existing thread
can switch between instances that share a continuation key (`docs/user/providers-codex.md`).

**How it works.** Work is routed by instance, never by driver, so two accounts on one driver share
no session or catalog state (`docs/internals/providers.md`). The Codex shadow home is the clever
part (`apps/server/src/provider/Drivers/CodexHomeLayout.ts`, 422 lines). The shadow directory keeps
its own `auth.json` and `models_cache.json` as real files and symlinks `sessions`,
`archived_sessions`, `sqlite`, `skills`, `plugins` and the other shared entries to the main home
(lines 19-34). Both instances carry the continuation key `codex:home:<shared home>` (lines 55, 64),
so a second ChatGPT account can resume the first account's sessions. Antigravity gets a separate
profile per instance with ambient Google credentials stripped (`antigravityAuthSupport.ts`).

**Good.** The shadow home is a small, file-level trick that makes "continue this Codex session on my
other account" possible without copying anything. **Bad.** The settings surface is large, and the
OpenRouter recipe depends on clearing a cached login first.

**Marley today: lacks.** Marley runs one `claude` and one `codex`, the ones `MARLEY_CLAUDE` and
`MARLEY_CODEX` name or the first on the PATH (The footer and the agent bar, #648). A launch config
can type `CLAUDE_CONFIG_DIR=… claude` (Launch configs, #527), but the plugin check, the IDE lock
file and the version check all read Marley's own `CLAUDE_CONFIG_DIR`.

### 2.3 Permission modes and plan mode

**What the user sees.** A per-thread mode in the composer: Supervised, Auto-accept edits, Auto (the
provider's own automatic review) and Full access, with a default in Settings and per-project
overrides. "The initial default is **Full access**" (`docs/user/permission-modes.md:8`). A separate
Plan and Build toggle sets the interaction mode.

**How it works.** `DEFAULT_RUNTIME_MODE` is `"full-access"`
(`packages/contracts/src/providerPolicy.ts:32`).
Claude maps Supervised to `default`, Auto-accept edits to `acceptEdits`, Auto to Claude's `auto`
mode, Full access to `bypassPermissions`, and Plan to `plan`; a read-only sandbox becomes `dontAsk`
(`permissionModeForClaudeRuntimePolicy`, `ClaudeAdapterV2.ts:1502-1546`). Codex maps the four to
`approvalPolicy` `untrusted` / `on-request` / `on-request` / `never`, sandbox `readOnly` /
`workspaceWrite` / `workspaceWrite` / `dangerFullAccess`, and Auto sets
`approvalsReviewer: "auto_review"` (`CodexAdapterV2.ts:655-690`). OpenCode stores "always"
approvals for a whole project, so T3 replies `once` even in Full access, which keeps a supervised
thread on a shared server from inheriting another thread's grant (`docs/internals/providers.md`).
For ACP agents without a native plan mode, T3 sends a plan-mode instruction block instead
(`T3OrchestrationInstructions.ts`).

**Good.** One vocabulary across providers, with the mapping in one function each. **Bad.** Full
access as the default for every new thread.

**Marley today: has part.** `marley.claude_code_permissions` is `ask` or `bypass` and
`marley.codex_permissions` is `ask` or `full_access`, per project by folder, and a chip marks an
agent that runs without prompts (Permission modes, #532). The TUIs switch modes themselves
(Shift+Tab in Claude Code). Marley offers neither Claude's `acceptEdits` and `auto` nor Codex's
`auto_review` as a launch default.

### 2.4 Switching model or provider inside a thread

**What the user sees.** The model picker stays live in a running thread. Picking another model of
the same provider applies on the next turn or restarts the session; picking another provider
continues the same thread with the new agent, which receives the conversation so far
(`docs/user/portable-handoffs.md`).

**How it works.** `decideProviderSessionTransition`
(`orchestration-v2/ProviderSessionTransitionPolicy.ts:35-96`) returns one of `reuse`,
`switch_model_in_session`, `restart_and_resume`, `create_with_handoff` or `reject`; the adapter
classifies a same-provider model change (`ProviderSwitchService.ts`, 219
lines). A change of provider creates a context transfer and, when native continuity is impossible,
a handoff. Returning to a provider used earlier resumes its old provider thread with a delta of
what happened elsewhere (`docs/orchestration-v2/provider-switching-and-context.md`).

The handoff is a budgeted selection of whole items, not a model-written summary
(`ContextHandoffBudget.ts`, 282 lines):

- The budget is the model window (128,000 when unknown) minus what the native context already
  holds, minus the new input, minus a reserve of 16,000 tokens or a quarter of the window, capped
  at `T3CODE_CONTEXT_HANDOFF_TOKEN_CAP` (16,000 by default) and 64 KB (`handoffBudget`, lines
  111-138). Tokens are counted as one UTF-8 byte each, deliberately pessimistic.
- `selectHistory` (lines 227-270) adds the latest user request, then the latest assistant answer,
  then the first user request, then walks back from the newest item. An item that does not fit is
  left out whole. Commands carry their exit code and output; file changes carry the file name.
- Each item is labelled `[Historical user; …]`, and a preamble says historical material is context,
  not a new request or a higher-priority instruction.
- `handoffCoverage` (lines 272-282) tells the agent how to fetch what was left out through T3's
  `t3_thread_read` tool, with paging.

**Good.** The selection order (latest ask, latest answer, original ask, then recency) and the
"context, not instructions" framing are both small and right. **Bad.** The machinery behind it
(transfers, coverage, supersession) only pays off inside T3's event store.

**Marley today: has part.** Inside one agent, `/model` in Claude Code and Codex's own model switch
work in the terminal, and the Agent Panel switches models for its agents. Nothing carries a Claude
Code session over to Codex or back. Orca's survey proposed "Continue in a new session" (report 01,
item 8); it has not been built.

### 2.5 Fork and merge-back

**What the user sees.** Fork from a finished turn into a new thread; the first message there picks
the provider. A fork can be brought back into its source thread.

**How it works.** `ThreadForkService.ts` (139 lines) accepts a source run that is completed,
waiting, failed, interrupted or cancelled, never one in progress or rolled back
(`isForkableSourceRunStatus`). It records lineage and a pending transfer and chooses nothing yet;
the first run resolves it through a native fork when the provider and source refs allow (Claude's
`forkSession`, Codex's `thread/fork`), or a portable handoff otherwise. Merge-back sends a delta of
the fork's items, each compacted to 240 characters (`ContextHandoffService.ts`, `compactText`).

**Marley today: has part.** Claude Code forks a session itself (`claude --resume <id>
--fork-session`). Marley has no fork action on an agent row.

### 2.6 Edit from here: rewind with file restore

**What the user sees.** "Edit from here" under a sent message rewinds the conversation to before
it, with "Revert and keep changes" or "Revert files too". File restore is offered only for a thread
in its own worktree, and is "refused when another thread or agent session also uses that
directory, a folder inside it, or a folder that contains it" (`docs/user/composer.md:113-117`).

**How it works.** Files come from T3's checkpoints, hidden git refs per turn (report 03 covers
them). The conversation rewinds natively: Claude resumes at an earlier message with
`resumeSessionAt` (`ClaudeAdapterV2.ts`, `rollbackThread` near line 7756); Codex pages
`thread/turns/list` to find the boundary turn and rolls back before it
(`apps/server/src/provider/CodexThreadRevert.ts`). A provider that cannot roll back its conversation
must refuse before any file changes. `CheckpointRestoreSafety.ts` (88 lines) runs the isolation
check twice, at admission and again before the provider rollback: it resolves real paths for every
other thread's worktree, checkpoint scopes and live session folders, and refuses when either
contains the other.

**Good.** The isolation rule is what makes file restore safe, and it is a short, portable check.

**Marley today: has part.** Claude Code's `/rewind` restores its conversation and the edits its own
tools made; Codex's Esc Esc backtrack rewinds the conversation only; Zed's Agent Panel rewinds and
restores checkpoints (`crates/acp_thread/src/acp_thread.rs:4380`). Marley's Turns rows (Per-turn
diffs, #509) already hold each turn's start and end as commits under `refs/marley/turns/`, shell
edits included, but offer no revert.

### 2.7 Continuing after a restart

**What the user sees.** Settings → General → "Continue threads after restarts", off by default:
after an update, a crash or a reboot, a thread that was mid-turn resumes on its own
(`docs/user/updating.md:13`). Queued messages wait behind it, held until the user presses Resume.

**How it works.** `RestartContinuation.ts` (186 lines). `restartContinuationRun` (line 28) picks a
thread only when its newest run was running on a strong native thread reference with a live turn.
`continueRestartedRun` (line 90) then sends "Continue where you left off." (line 22), prefixed by a
note naming any background work the restart cancelled. It does nothing when the user had asked the
run to stop, when a newer user message exists, when the thread is archived, or when the cut run was
a maintenance command such as `/compact`.

**Marley today: has part.** Claude Code sessions come back after a restart as
`cd <folder> && claude --resume <id>` (#540, `crates/marley_workbench/src/resume.rs:1-10`). Codex is
not resumed, and a turn the quit cut off waits for the user to type.

### 2.8 Usage limits: Limited, Resume at reset, snooze

**What the user sees.** A thread stopped by a usage limit reads **Limited**, with the window that ran
out and its reset time. "Resume at reset" schedules a continuation; "Auto-resume limited threads"
does so by default; "Snooze until reset" hides the thread until then; the two combine
(`docs/user/thread-sidebar.md:187-210`). For Claude, T3's docs note that "Claude Code holds the turn
until that window reopens, so it can keep showing as working" (`docs/user/providers-claude.md:60`);
for Codex, "Send the message again after the reset" (`docs/user/providers-codex.md:98`).

**How it works.** The Claude adapter reads the SDK's `rate_limit_event` (window, status, `resetsAt`,
overage) into the thread (`ClaudeAdapterV2.ts:5563-5625`); Codex reports
`account/rateLimits/read` and `account/rateLimits/updated` (`provider/codexUsageLimits.ts:2-3`).
`UsageLimitRecoveryWorker.ts` (115 lines) arms a recovery keyed on the run and the reset time, and
when the reset passes sends "Continue where you left off." as a user message (lines 11-70). It
skips a thread that is settled, snoozed, waiting on a request, or whose reset was already past when
the failure arrived, so a stale window cannot start a retry loop. Sending a message, archiving or
settling cancels the pending continuation. Overdue continuations run after a server restart.

**Good.** The identity of one recovery (thread, run, reset time) and the stale-window guard are
the parts a port must keep.

**Marley today: has part.** Marley records a failed Claude turn's error type, `rate_limit` among them
(`crates/marley_agent/src/claude_events.rs:52`), and the rail shows `failed` with it. Codex's row
shows `failed` with the turn's error when it runs on its App Server (#650). Neither row shows when
the limit resets, and nothing continues. Harness sessions show their quota windows (The harness's
sessions, #640).

### 2.9 Questions and approvals from the provider

**What the user sees.** Approvals and questions answered in the thread on any client, a subagent's
included (its parent asks). Codex can ask a question and keep working; the answer becomes a new
message that reaches the running turn or starts another (`docs/user/providers-codex.md`). App-access
requests from Codex tools offer once, this session or always. Questions that take free text also
take attached files (`docs/user/question-attachments.md`).

**How it works.** Claude: `canUseTool` and `onUserDialog` callbacks resolved from the UI. Codex has
two forms: the blocking `item/tool/requestUserInput` server request, answered with the answers
(`CodexAdapterV2.ts:5085`), and an `agentMessage` with `delivery: "async"` and questions, answered
by a user message (`:4608`). The answer path validates required answers and commits the resolution
and the message in one transaction, so a repeat posts nothing twice (`docs/internals/providers.md`,
"Protocol traps"). App-access elicitations are parsed for their persistence options
(`provider/CodexMcpElicitation.ts`).

**Marley today: has part.** The inbox lists Claude Code's permissions and questions (opened in the
terminal) and answers Codex's command, file-change, permission and MCP-server requests from its App
Server (The rail, "Needs you", #508, #651). "A question of Codex's own is listed as 'Waits on an
answer' and opens the terminal", and an MCP request offers Deny and Dismiss only.

### 2.10 Subagents and delegation

**What the user sees.** Subagents appear under **Agents** as read-only child threads with their tool
activity and final result; Stop on the parent stops them (`docs/user/thread-sidebar.md`). An agent
can also delegate to another provider and model through T3's `delegate_task` tool, and the result
comes back to the parent as a message.

**How it works.** `SubagentProjection.ts` (255 lines) makes child app threads titled from the
subagent's description (`subagentThreadTitle`, 72 characters). The Claude adapter finds a
subagent's launching tool call from `parent_tool_use_id` in the CLI's session storage
(`ClaudeAdapterV2.ts:763-795`). Delegated completions are delivered by
`ProviderContinuationService.ts` as "Delegated task … reached a terminal state". Report 06 covers
the MCP tools.

**Marley today: has part.** The rail counts running subagents and tracks their tools in flight
(#519); there is no row per subagent.

### 2.11 Provider-native commands surfaced in the GUI

**What the user sees.** `/goal` with a goal row and progress above the composer, `/compact` and a
context meter, "Auto-compact after" for Claude (an integer from 100,000 to 1,000,000), `$` for
skills, slash commands from the provider, and "Restart agent session" after adding skills, plugins or
MCP servers (`docs/user/composer.md`).

**How it works.** T3 mirrors provider state rather than implementing it: Codex runs several native
goal turns inside one T3 run; Claude's goal state is read from synthetic command output and Stop
hook feedback (`docs/internals/providers.md`). Auto-compact passes Claude's `autoCompactWindow`
setting through the SDK (`ClaudeAdapterV2.ts:858-862`).

**Marley today: has.** In a terminal the TUI provides every one of these. A goal line on the agent's
row would be a refinement at most.

### 2.12 The Usage page

**What the user sees.** Usage (`mod+u`): tokens, cache savings, model breakdowns and API-equivalent
cost across Codex, Claude Code, Grok, OpenCode, Antigravity and Cursor, per environment, for 24 hours
to 90 days; editable model prices with "Map to" for preview names; **Limits**, which pools every
subscription account per provider into one bar per window with each account a segment; a CLIProxyAPI
hub connection; "Use reset" for banked reset credits; a phone widget (`docs/user/usage.md`).

**How it works.** `apps/server/src/usage/` (4,478 lines). `usageTranscripts.ts` holds pure
line-at-a-time parsers of the CLIs' on-disk transcripts (`~/.claude/projects/**/*.jsonl`,
Codex's `sessions`), with a dedupe key per record and a speed tier (standard, fast, ultrafast).
Prices come from LiteLLM's `model_prices_and_context_window.json`, "the same table `ccusage`
prices against" (`usagePricing.ts:1-8`). Claude's limits come from the SDK's `get_usage` control
response (`provider/claudeUsageLimits.ts:157`); Codex's from the App Server. Reset credits read the
CLI's OAuth token and call `api.anthropic.com/api/oauth/usage` (`provider/claudeResetCredits.ts:24`,
`:176`), which is why the feature is off on macOS, where the token sits in the keychain.

**Good.** Reading the transcripts the CLIs already write means the numbers cover work done outside
T3 too. **Bad.** Reset credits and the hub use private endpoints and account pooling.

**Marley today: lacks.** Claude Code's `/usage` and Codex's `/status` show current limits inside
the TUI. Marley shows a token count on a Codex row (#650) and quota for harness sessions (#640); it
has no history of tokens or cost.

### 2.13 Past sessions: import and search

**What the user sees.** The welcome wizard finds directories Claude Code or Codex has worked in,
groups clones by remote, preselects git repositories active in the last 30 days with at least three
conversations, and imports those conversations as threads that continue the native session
(`docs/user/welcome-wizard.md`). The command palette searches threads across environments,
messages included, after two characters (`docs/user/thread-sidebar.md`).

**How it works.** `apps/server/src/project/AgentSessionScanner.ts` (1,495 lines) reads each
transcript's `cwd` from `<home>/projects` for Claude (line 926) and `<home>/sessions` for Codex
(line 978), with hard limits (5,000 transcripts per source, 1 MB scanned per transcript, 200
messages kept per import: the first user prompt and the newest 199, line 494). The importer
attaches the native session id as a strong reference with a resume cursor, so the next message
resumes it (`AgentSessionImporter.ts:319-347`).

**Marley today: has part.** `claude --resume` and `codex resume` open each CLI's own picker of
sessions for the folder. Zed's Agent Panel imports native sessions from ACP agents that list them
(`crates/agent_ui/src/thread_import.rs`). Marley's rail lists no past terminal sessions and searches
none.

### 2.14 Provider versions and updates

**What the user sees.** A warning when an installed CLI version has known problems with this T3
release, the recommended version or range, and **Update now**, which runs the installer that owns
the CLI; **Update all** across machines (`docs/user/install.md`, `docs/user/updating.md`).

**How it works.** `apps/server/src/provider/model-manifest.json` carries compatibility policies:
per driver and T3 version range, a list of CLI version ranges each marked `supported`, `graceful`,
`unsupported` or `broken` (for Claude Code: `>=2.1.280` supported, `>=2.1.111 <2.1.280` graceful).
The manifest ships in the bundle and is refreshed from GitHub `main` at run time
(`ModelManifest.ts:42`); a newer bundle outranks an older fetched copy by `updatedAt`
(`docs/internals/model-manifest.md`). Updates run Homebrew or npm only when the binary's real path
proves that installer owns it; otherwise `claude update`, `codex update` and the like
(`providerMaintenance.ts`, 856 lines; `providerMaintenanceRunner.ts`, 523).

**Marley today: has part.** Marley reads `claude --version` and `codex --version`, keeps the
versions it checked in a compiled table, and shows chips such as "Untested Claude Code 2.2.0"
(The footer and the agent bar, #648).

### 2.15 Text generation by a provider

**What the user sees.** Thread titles, worktree branch names, commit messages and pull request text
written by a model; branch naming takes a static prefix (`t3/` by default), a model-chosen semantic
prefix (`feat/`, `fix/`) or custom instructions (`docs/user/project-settings.md`).

**How it works.** `apps/server/src/textGeneration/` (2,897 lines), one implementation per provider.
For Claude it spawns `claude -p --output-format json --json-schema <schema> --model <id>
--settings <json> --tools "" --disable-slash-commands --strict-mcp-config --permission-mode dontAsk`
with the prompt on stdin (`ClaudeTextGeneration.ts:175-195`): no tools, no MCP servers, typed JSON
out.

**Marley today: has part.** Zed writes commit messages through its own model provider. Worktree
agents get the branch name Zed generates, `agent/<name>` (Worktree agents, #510).

## 3. Bring to Marley

1. **A Limited state and continue at reset.** *Why.* An agent that stops on a usage limit stops the
   whole evening's work, and today the row says `failed` with no time. Show `limited · resets in
   1 h 35 m` on the row and in Needs you, with Continue at Reset (and an opt-in setting to arm it
   for every limit). At the reset, send "Continue where you left off." Keep T3's guards from
   `UsageLimitRecoveryWorker.ts:11-70`: one recovery per (session, turn, reset time), nothing when
   the reset had already passed at the failure, nothing once the user typed, closed the terminal or
   the agent waits on a request. Codex comes first: its App Server gives the reset time
   (`account/rateLimits/updated`) and takes `turn/start`, so nothing is typed into the TUI. For
   Claude Code, check on the box whether the TUI stops on the limit or holds the turn as T3's SDK
   path does; if it stops, the reset time comes from the status line's `rate_limits` and the
   continuation is typed only while the session is idle, as review notes are (#522). *Seam.*
   `crates/marley_agent` (the state), `marley_workbench/src/codex_server.rs`, the rail. *Size.* M.
   *Hard.* Claude's reset time; a laptop asleep at the reset (fire overdue continuations on wake).

2. **Restart continuation for every agent, and for a cut turn.** *Why.* #540 brings Claude Code back
   but leaves Codex behind and leaves a cut-off turn idle. Resume Codex with `codex resume <id>` (the
   thread id is known from the App Server or the session log), and when the session's last state
   before the quit was `working` and the user had not interrupted, start it with "Continue where you
   left off." (`claude --resume <id> "…"`), as `RestartContinuation.ts:28-170` does, with its
   exclusions (an interrupt, a newer prompt, `/compact` and other maintenance commands). *Seam.*
   `crates/marley_workbench/src/resume.rs`. *Size.* S. *Hard.* Knowing the turn was mid-flight at
   quit: Marley already holds the row state from the hook events.

3. **Hand a session to the other agent.** *Why.* When Claude Code is limited or stuck, Codex can
   take the work, and back. Add "Continue in Codex" / "Continue in Claude Code" to an agent row's
   menu: read the session's transcript (Claude's `transcript_path` from the hook events, Codex's
   session file), select history with T3's `selectHistory` order and byte budget
   (`ContextHandoffBudget.ts:111-270`), frame it as context rather than instructions, name the
   transcript path for the rest, and start the other CLI in a new terminal of the same folder with
   that as its first prompt on argv. This replaces Orca's "read the transcript yourself" prompt
   with a selection that survives a small context. *Seam.* A pure module in `crates/marley_agent`
   (parse, select, render), `marley_workbench/src/agents.rs` for the launch. *Size.* M. *Hard.* Two
   undocumented transcript formats; argv length (pass a file path when the selection is large).

4. **Past sessions per project, with search and Resume.** *Why.* Closing a terminal loses the way
   back to its conversation unless Chad remembers the picker. A "Sessions" disclosure per project
   lists Claude Code and Codex sessions whose `cwd` is in the project, newest first, titled from
   the first real prompt, with Resume (a terminal running `claude --resume <id>` or
   `codex resume <id>` in that folder) and a text search over prompts and final answers. T3's
   scanner gives the reading rules and limits (`AgentSessionScanner.ts`: per-source caps, the first
   prompt plus the newest messages). *Seam.* The same transcript module as item 3, the rail,
   `agents.rs`. *Size.* M. *Hard.* Index size for search; keep it to prompts and final messages.

5. **Answer Codex's own questions and app-access requests from the inbox.** *Why.* #651 answers
   approvals but sends Codex's questions to the terminal. Through the App Server Marley already
   joins: answer a blocking `item/tool/requestUserInput` with its options, answer an async question
   as a user message (`turn/steer` while the turn runs, `turn/start` after), and offer once, session
   or always on app-access elicitations, as `CodexAdapterV2.ts:4608`, `:5085` and
   `CodexMcpElicitation.ts` do. *Seam.* `marley_agent/src/codex_events.rs`,
   `marley_workbench/src/codex_server.rs`, the inbox. *Size.* S to M. *Hard.* Async questions have
   no pending request to close; clear the entry when Codex's next turn starts.

6. **A Usage tab read from the transcripts.** *Why.* Tokens and cost per day, project and model,
   across terminal sessions, without leaving Marley; the TUIs show only current limits. Port the
   pure parsers of `usageTranscripts.ts` (530 lines) and the aggregation, and show limits beside
   them: Codex from `account/rateLimits/read` on its App Server, Claude from the status line's
   `rate_limits`. Prices need LiteLLM's table: bundle a snapshot rather than fetch it. *Seam.* A
   center tab in `marley_workbench`, a pure crate for the parsers. *Size.* M. *Hard.* Deduping
   records across resumed and forked transcripts (T3's `dedupeKey`).

7. **Revert a turn's files from its Turns row.** *Why.* Claude Code's `/rewind` misses edits made
   through shell commands, and Codex restores no files; Marley's turn commits already hold both.
   Add "Restore files to before this turn" on a turn row, restoring the turn's changed paths from
   its start commit, and refuse unless the agent works in a worktree no other terminal, agent or
   project folder contains or sits inside, with T3's two-time check
   (`CheckpointRestoreSafety.ts`). Tell the user the agent's conversation is unchanged and offer
   the agent's own rewind. *Seam.* `marley_workbench` (Per-turn diffs, #509). *Size.* M. *Hard.*
   Later turns that touched the same files; restore only the newest turn by default.

8. **More permission modes at launch.** *Why.* Between ask and bypass sit Claude Code's
   `acceptEdits` and `auto` and Codex's automatic review, which T3 maps in two functions
   (`ClaudeAdapterV2.ts:1502-1546`, `CodexAdapterV2.ts:655-690`). Add them as values of
   `marley.claude_code_permissions` and `marley.codex_permissions`, per project as today, and let
   the chip name them. *Seam.* #532's settings and `agents.rs`. *Size.* S. *Hard.* Check the flag
   names against the installed CLI versions (#648's table).

9. **Subagent rows.** *Why.* A count says little when three subagents run. List each under the
   agent's row with its description and the tool in flight, from `SubagentStart`/`SubagentStop`
   and the `subagent_tool:` keys Marley already tracks, titled as `subagentThreadTitle` does.
   *Seam.* `crates/marley_agent/src/claude_events.rs`, the rail. *Size.* S to M. *Hard.* Subagents
   that never send a stop event; clear them with the turn.

10. **Agent profiles.** *Why.* Only if Chad runs a second account or a router: named variants of
    an agent CLI with their own environment (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`,
    `ANTHROPIC_BASE_URL`) in the `+` menu, the New Agent picker and launch configs, with the
    plugin and IDE checks reading the profile's own config directory. The Codex shadow-home layout
    (`CodexHomeLayout.ts:19-64`) lets a second account resume the first one's sessions. *Seam.*
    `marley_agent`'s catalog, `agents.rs`. *Size.* S to M. *Hard.* Every place Marley reads Claude
    Code's config directory today.

Smaller things worth a day each: status levels in #648's table (`supported`, `graceful`,
`unsupported`, `broken`, from `model-manifest.json`) instead of tested or untested; an Update
action on the version chip that runs `claude update` or `codex`'s own updater in a terminal; a
worktree agent's branch named from its first prompt with T3's no-tools `claude -p --json-schema`
recipe (`ClaudeTextGeneration.ts:175-195`), behind a setting with a static prefix.

## 4. Skip

- Running agents headless through SDKs and app servers as the main surface: Marley's model is the
  vendor's TUI in a terminal, and Zed's Agent Panel already is the structured route for ACP agents.
- The event-sourced orchestrator, provider threads, coverage records and capability negotiation:
  they exist to keep a GUI conversation alive across providers inside a server Marley does not
  have. Take the policies (items 1 to 3), not the store.
- Adapters for Cursor, Grok, Antigravity, Pi, OpenCode servers and Devin: Chad runs Claude Code
  and Codex.
- T3's ACP Registry support: Zed has the registry (`crates/project/src/agent_registry_store.rs:20`).
- Full access as the default mode: Marley keeps each CLI's prompts unless Chad opts in (#532).
- Claude reset credits through `api.anthropic.com/api/oauth/usage` with the CLI's OAuth token, and
  CLIProxyAPI hubs: private endpoints and account pooling.
- Fetching the model manifest from GitHub at run time: a network call Marley does not need for a
  table that changes when Marley is rebuilt anyway.
- Managed Codex installs and ChatGPT sign-in run by the app, and the welcome wizard: the box is set
  up, and the CLIs own their logins.
- Cross-provider `delegate_task`: supervised workers belong to rustal-harness; report 06 covers the
  tool surface.
- The composer's `/goal` row, `$` skills menu, "Restart agent session" and the auto-compact setting:
  the TUIs give Chad these in the terminal.

## 5. Open questions

1. Does Claude Code's TUI stop a turn on a usage limit on Chad's plan, or hold it until the window
   reopens as T3's SDK path does? *Default: build item 1 for Codex first and check Claude on the
   box.*
2. Should a limited agent continue at the reset by itself, or only when Chad presses Continue at
   Reset? *Default: the button only; auto-continue is a setting, off.*
3. Does Chad run more than one Claude or Codex account, or a router such as OpenRouter? (Orca's
   survey asked the same.) *Default: no profiles (item 10 waits).*
4. When a session is handed to the other agent, should the old terminal stay open beside the new
   one? *Default: yes, a new terminal in the same folder, the old one kept.*
5. Should the Usage tab show API-equivalent cost, which needs a price table, or tokens only?
   *Default: tokens only until Chad asks for prices; no network fetch either way.*
6. May "Restore files to before this turn" run in a project's main checkout when no other agent
   works there? *Default: no; worktree agents' worktrees only, as T3 does.*
