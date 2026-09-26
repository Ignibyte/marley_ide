# Orca survey 01: agents and sessions

Read from `/srv/stacks/orca-refs/orca` at `1c2cf120e3` (2026-09-25). Paths are relative to that
repo root unless they start with `crates/` (Marley). Line counts are non-test TypeScript.

## 1. Summary

Orca runs agent CLIs as TUIs in PTYs (39 in its catalog), and Claude and Codex optionally as SDK
or app-server chats. It learns each agent's state from hooks it writes into 16 CLIs' user configs,
which curl every event to a loopback server keyed by a pane id in the PTY's env; titles, process
polling, screen scraping and keystroke inference fill the gaps. The state drives a needs-you-first
sidebar, a kanban board, an Agents feed that embeds the live terminal, and notifications that name
the tool or quote the last message. Around it: resume over 20 agents' transcripts, hibernation,
handoff, subagent rows, orchestration, and usage built on private endpoints. Well over 200,000
lines; most recent commits to the hook code are fixes. Worth taking: (1) hook-driven terminal
state, carried in-band by Marley's plugin, not over HTTP; (2) the Needs You model for #508; (3)
turn boundaries for #509; (4) notifications with content and unread; (5) Claude and Codex resume.

## 2. Features

### 2.1 The agent catalog and how a launch is built

**What the user sees.** An agent combobox in the new-workspace composer, the tab bar's + menu and
the dashboard lists the agent CLIs Orca found on `PATH`. Settings → Agents enables or disables
each one, sets a command override, default arguments and environment per agent, a default agent
for new workspaces, and one Agent Permissions switch (Yolo / Manual). The docs say Orca "works
with any CLI agent"; in the source that means any program typed into an Orca terminal. The launch
catalog itself is a closed union of 39 ids (`src/shared/tui-agent.ts`) and there is no way to add
an entry, only to override an existing entry's command (`agentCmdOverrides`).

**How it works.**

- `src/shared/tui-agent-config.ts` (370 lines) is the catalog. Per agent: `detectCmd` plus aliases
  and required commands (Agent Teams needs both `orca` and `claude`), `launchCmd` per platform,
  `expectedProcess` (the foreground process name that proves the agent is running; ZCode renames
  itself `zcode-cli`), and `promptInjectionMode`: `argv` (Claude, Codex, Cursor, Grok with `--`),
  `flag-prompt` (`--prompt`, OpenCode), `flag-prompt-interactive` (Gemini, Antigravity),
  `flag-interactive` (Copilot `-i`, because `--prompt` exits), `hermes-query`, or
  `stdin-after-start` (paste into the running TUI: Aider, Goose, Amp, Devin, Muse and others).
  Drafts (text placed in the input without sending) use `claude --prefill` or an env var read by
  Orca's Pi/OMP extension. Most fields carry a "Why" comment naming the bug that forced them:
  Codex needs a blind second Enter 1.2 s later, OpenCode a 20 s paste-readiness budget (measured
  under Windows ConPTY), Grok's animated logo never goes quiet, Antigravity expands long pastes at
  45 ms a line.
  This file is the most reusable thing in the area: a record of how each TUI misbehaves.
- The command line: `src/shared/tui-agent-launch-command.ts` builds
  `<override or launchCmd> <model/effort flags> <user args>`; `src/shared/tui-agent-startup.ts`
  (`buildAgentStartupPlan`) appends the prompt per injection mode, quoted for the target shell by
  `src/shared/tui-agent-startup-shell.ts` (388 lines, POSIX sh, PowerShell and cmd).
- The environment: the agent's default env (`agentDefaultEnv`, e.g. `GOOSE_MODE=auto`) plus
  Orca's pane identity on every PTY it spawns (`src/main/ipc/pty/ipc/spawn-env.ts`):
  `ORCA_PANE_KEY` (`tabId:leafId`, the pane's stable id), `ORCA_TAB_ID`, `ORCA_WORKTREE_ID`,
  `ORCA_AGENT_LAUNCH_TOKEN` for launched agents, and the hook server coordinates
  `ORCA_AGENT_HOOK_PORT/TOKEN/ENV/VERSION/TRANSPORT/ENDPOINT`
  (`src/main/agent-hooks/server/server-runtime-env.ts`). Codex also gets `CODEX_HOME` pointing at
  an Orca-managed home (see 2.12).
- The working directory is the worktree root. A resumed session's recorded subdirectory can be
  passed as `cwd` on the launch intent, which forces a terminal launch.
- Delivery: the PTY runs the user's shell with Orca's rc wrappers, whose prompt prints
  `ESC ] 777 ; orca-shell-ready BEL` (`src/main/daemon/daemon-bash-shell-ready-rcfile.ts`,
  `src/main/shell-ready-marker-scanner.ts`); the agent command is typed only after that marker
  (`startupCommandDelivery: 'shell-ready'`), so slow rc files cannot eat it.
- Two launch modes. `src/main/agent-launch/agent-launch-mode.ts` decides between `terminal` (the
  TUI in a PTY) and `structured` (Orca's native chat, Claude and Codex only) from the user's
  default-view setting and host facts; a remote host, WSL, a reused terminal, a custom launch
  command, a `cwd` outside the root or Codex on Windows downgrade to a terminal, with a receipt
  that says why. `agent-launch-executor.ts` orders worktree creation, the host's
  structured-support check and surface creation, and falls back to a terminal only on a
  definitive structured refusal. `agent-launch-prompt-delivery.ts` reports what became of the
  first prompt: `journaled` (a structured message id), `handed-to-terminal` (argv, or a PTY write
  that returned) or `not-delivered`. One `AgentLaunchIntent` type (`src/shared/agent-launch-intent.ts`)
  serves the UI, mobile, the CLI and orchestration; the executor's header admits that only the
  `agent.launch` RPC uses it so far and the other surfaces "still start agents their own way".
- First-launch trust menus. `src/main/agent-trust-presets.ts` pre-writes the file each CLI writes
  after a user accepts "trust this folder?": Cursor's
  `~/.cursor/projects/<slug>/.workspace-trusted`, Copilot's `trustedFolders` in
  `~/.copilot/config.json`, Antigravity's `trustedWorkspaces` in
  `~/.gemini/antigravity-cli/settings.json`, and Codex's `[projects."<path>"] trust_level =
  "trusted"` in both `~/.codex/config.toml` and Orca's managed home. The reason given: the menu
  reads single keys, so the bracketed paste carrying the first prompt picks a menu option or quits
  the agent, and every new worktree is a new path. `remote-agent-trust-presets.ts` does the same
  over SFTP (Antigravity missing, recorded as a known gap).
- Permissions. `src/shared/tui-agent-permissions.ts` maps each agent to its bypass switch (Claude
  `--dangerously-skip-permissions`, Codex `--dangerously-bypass-approvals-and-sandbox`, Gemini and
  others `--yolo`, Droid `--auto high`, Grok `--permission-mode bypassPermissions`, Qwen
  `--approval-mode yolo`, Goose by env). `DEFAULT_TUI_AGENT_ARGS` in `tui-agent-launch-defaults.ts`
  is that table: bypass is what a user gets until they choose Manual. The Agent Permissions switch
  stores nothing itself; it rewrites every untouched agent's arguments to the flag or to empty,
  and leaves an agent with custom arguments alone ("mixed"). The structured path reads the same
  bit from the arguments string to decide whether a native-chat session bypasses approvals.

**Good.** The catalog records per-TUI quirks with reasons; injection modes keep multi-line prompts
out of keystrokes where the CLI takes argv; the launch receipt never claims a delivery it cannot
see; trust pre-seeding fixes a real problem for agents in fresh worktrees.
**Bad.** Bypass by default, documented as "the worktree itself is the sandbox" while the Callout
on the same site says a worktree is not a sandbox. Orca edits other tools' config files (Copilot,
Cursor, Codex, Antigravity). Launch paths are not unified yet.
**Size.** About 4,100 lines (catalog, launch, injection); trust presets 390.
**Marley today: has part.** `crates/marley_agent` knows four agents (claude, codex, gemini,
opencode) by the command's leading program; `crates/marley_workbench/src/agents.rs` waits up to
5 s for the shell to say it is ready, then types `<program>\r` (`launch_input`): no arguments,
environment, prompt, draft or trust handling. Zed agents start as Agent Panel threads.

### 2.2 Agent status from hooks

**What the user sees.** A state glyph per agent on terminal tabs, sidebar worktree cards and the
dashboard, plus a live "tool step" (wrench, tool name, input preview such as the file path or
command), the latest assistant message, the pending question's text, and child rows for
subagents and teammates.

**Which agents.** Managed hooks are installed for 16 CLIs (`AGENT_HOOK_TARGETS` in
`src/shared/agent-hook-types.ts`; installers in `src/main/agent-hooks/managed-agent-hook-registry.ts`,
one `hook-service.ts` per agent directory): Claude, OpenClaude, Codex, Gemini, Antigravity, Amp,
Cursor, Droid, Command Code, Grok, Copilot, Hermes, Devin, Kimi, Muse, ZCode. OpenCode and
OpenCode 2 get a JS plugin loaded from a per-PTY config overlay (`src/main/opencode/hook-service.ts`);
Pi, OMP and Prime Agent get an extension (`src/main/pi/agent-status-extension-source.ts`). All
post to one listener. Installation runs at startup for detected, enabled CLIs while Settings →
Agents → Agent status hooks is on (default on); `orca agent hooks status|on|off` from the CLI.
Startup never removes hooks, because the files are user-global and a second Orca profile with the
switch off would strip them from the others (`managed-agent-hook-controls.ts`, STA-5679).

**Where they go.** Into each tool's own user config: the `hooks` arrays of `~/.claude/settings.json`
(merged beside the user's entries, `src/main/claude/hook-settings.ts`), Cursor's `hooks.json`,
and so on. For Codex, Orca writes `hooks.json` into its own managed `CODEX_HOME` and, when the
selected account is the system default, appends its entry to `~/.codex/hooks.json` as well; in
both it adds a per-hook trust entry to `config.toml` (a precomputed hash, or a grant made through
a `codex app-server` session), so Codex 0.129+ does not ask for `/hooks-approve` (`src/main/codex/codex-hook-definition.ts`,
`codex-real-home-hook-install.ts`, `codex-hook-trust-grant.ts`). Each
entry's command is a wrapper that runs `~/.orca/agent-hooks/<agent>-hook.sh` only if it exists,
so a user who deletes Orca keeps a harmless no-op (`runtime-home-hook-command.ts`).

**Events.** Claude: SessionStart, UserPromptSubmit, Stop, StopFailure, SubagentStart,
SubagentStop, TeammateIdle, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest,
PostCompact, and SessionEnd on CLI versions that support it; `Notification` is not used. Codex:
SessionStart, UserPromptSubmit, PreToolUse, PermissionRequest, PostToolUse, SubagentStart,
SubagentStop, Stop. PreCompact is left out on purpose: an aborted compact emits it alone and
would strand the pane in "working" (comment in `hook-settings.ts`, STA-4613).

**What a hook does** (`src/main/claude/hook-script.ts`, `src/main/agent-hooks/hook-post-command.ts`,
`hook-stdin-contract.ts`). Print `{}` first (some Claude-compatible consumers fail a permission
hook closed on empty stdout, #14818); read the payload from stdin; exit if `CLAUDE_JOB_DIR` is set
(a background job inherited the dispatching pane's env, #9236); re-source
`$ORCA_AGENT_HOOK_ENDPOINT`; then
`curl -X POST http://127.0.0.1:$ORCA_AGENT_HOOK_PORT/hook/claude` with an
`X-Orca-Agent-Hook-Token` header and a base64 metadata header holding the pane key, tab id,
launch token and worktree id (0.5 s connect, 1.5 s total). If the post fails, the hook appends a
JSON line to `<endpoint dir>/spool/pane-<id>.jsonl` (5 MiB cap, reset after 7 days, tool events
skipped), which the server replays at its next start. Every event therefore spawns a shell,
`cat`, `base64`, `tr` and `curl`. On Windows the hook is a `.cmd` calling `curl.exe`, or an
encoded PowerShell launcher; 18 of the 53 top-level source files in `src/main/agent-hooks` exist
for WSL or Windows.

**The listener** (`src/main/agent-hooks/server.ts` → 35 files in `server/`). An HTTP server on
127.0.0.1, random port, a fresh random-UUID token per start. It writes
`<userData>/agent-hooks/endpoint.env` (mode 0600 in a 0700 directory) so an agent in a PTY that
outlived an app restart finds the new port on its next hook; hydrates `last-status.json` (rows up
to 7 days old, marked `restoredUnconfirmed` so they never read as live); drains the spool before
binding. Normalization is in `src/shared/agent-hook-listener.ts` and
`src/shared/agent-hook-listener/providers/*.ts` so the SSH relay (`src/relay/agent-hook-server.ts`,
a Node process Orca deploys on the remote) and the WSL relay run the same code and forward
`agent.hook` JSON-RPC notifications over the SSH multiplexer.

**The mapping for Claude** (`src/shared/agent-hook-listener/providers/claude-events.ts`, 322 lines).
UserPromptSubmit, PreToolUse, PostToolUse and PostToolUseFailure → `working`. PermissionRequest,
and PreToolUse of `AskUserQuestion` → `waiting`. Stop, StopFailure (a `failure` verdict) and a
manual PostCompact → `done`. SessionStart with source startup/resume/clear → a `done` row flagged
`sessionBoundary`, which notifications, unread and stats ignore. Events that carry `agent_id` come
from a subagent or teammate: they update a per-pane roster and keep the pane `working` while
children run; `background_tasks` and `session_crons` in the Stop payload keep it `working` or
`monitoring` after the lead's turn ends. What the row keeps (`AgentStatusEntry` in
`src/shared/agent-status-types.ts`): state, prompt, tool name, tool input preview (a per-tool key
table in `tool-input-preview.ts`: `file_path` for Edit, `command` for Bash, `url` for WebFetch,
across 70 tool names from all agents), the full AskUserQuestion JSON as `interactivePrompt`,
`last_assistant_message` from Stop (else read from `transcript_path`), a capped state history,
subagent snapshots, the provider session id (for resume), the model, and `mainAgent` (the lead's
own state beside the combined one).

**The store.** `docs/reference/agent-status-store.md` states the rule: each execution host keeps
one status store (the hook server's), hook posts, relay posts, main's own OSC parse and
structured sessions all write it, and the renderer, `orca worktree ps` and the phone only read it.
Precedence is decided once at write time.

**Good.** Rich, attributable status (who, which tool, what question), survives app restarts and
works over SSH and WSL. Reader-side rules (30-minute decay, unread, dismissal) are kept apart from
the store.
**Bad.** Heavy: 11,600 lines in `src/main/agent-hooks`, 7,700 in the shared listener and 9,700 in
the status model (`agent-status-*.ts`, `agent-title-*.ts`). Of about 190 commits that touched
the hook area since June, 122 are fixes. Most of the machinery (launch tokens and their hashes,
retired-pane fences, pane-key aliases, cancel latches, per-connection watermarks, hydrated rows
that must not look live) exists because the hook runs in a separate process that must find the
right pane over HTTP after restarts, relays and pane reuse. Orca also rewrites 16 tools' global
configs. One doc is wrong: `agents/claude-code.mdx` says a status-line hook "emits OSC title events
Orca uses for state dots"; the status line only posts rate limits (2.12) and state comes from the
lifecycle hooks.
**Marley today: has part.** The plugin (`crates/marley_workbench/claude_plugin/marley/hooks/`)
registers Notification (`permission_prompt`, `idle_prompt`) and Stop, and each hook answers with a
`terminalSequence` that makes Claude Code print an OSC 777 notify with a fixed sentence
("<project> needs your permission"). Nothing machine-readable reaches the rail: a terminal's agent
row reads `working` while output arrives and `waiting` after 2 s of quiet or a bell
(`marley_agent::agent_status`, `WAITING_AFTER`). Agent Panel threads get real status from ACP
(`marley_rail::thread_status`: pending tool call, error, generating). `crates/marley_fleet` has a
pure session model (Starting, Working, Idle, Waiting, Error, Done, a structured `Question`,
attention ranking) served by `marley_mcp` as `fleet_snapshot`, but nothing in
`crates/marley_workbench` feeds it: `mcp.rs` starts the server with `FleetSnapshot::default()`.

### 2.3 Status without hooks, and the fallbacks behind the hooks

Orca stacks several weaker signals under the hooks and lets a fresh hook row win.

- **OSC 9999.** `ESC ] 9999 ; <JSON status payload> BEL` in any PTY's output is parsed in main,
  stripped from what the terminal shows, and written to the same store
  (`src/shared/agent-status-osc.ts`, 131 lines, 64 KiB cap on a split frame). Nothing Orca ships
  emits it outside tests; it is the open channel for any CLI to report its own state.
- **Terminal titles** (`src/shared/agent-title-status.ts`, `agent-title-core.ts`). OSC 0/2 titles
  are classified `working`, `permission` or `idle`: Claude's `✳` means idle, braille or
  quarter-circle spinner frames mean working (Claude 2.1.228 switched glyphs and briefly read as
  "exited", #13889), Gemini's `✦ ⏲ ◇ ✋`, keywords (ready, idle, done / working, thinking,
  running) with boundary regexes that skip paths like `~/codex/ready`, and "action required",
  "permission", "waiting". A title that reverts to a shell prompt after idle means the agent
  exited.
- **Foreground process.** `src/shared/agent-process-recognition.ts` names the agent from the
  process table (names, aliases, node or python entrypoints such as
  `node_modules/@openai/codex/`); the renderer polls one shared, TTL-deduped process snapshot on a
  common grid so N panes cost one `ps` (`agent-completion-poll-interval.ts`). The agent process
  leaving the foreground counts as a completion.
- **Output scraping** for Command Code, whose hooks cover only PreToolUse, PostToolUse and Stop,
  so a turn's start is read from the screen (`src/shared/command-code-output-status.ts`: its
  rotating "Thinking…", "Pondering…" status words and its idle composer).
- **Synthetic titles** (`src/shared/synthetic-agent-title.ts`): for Codex, Cursor, OpenCode, Pi,
  OMP, Droid, Hermes, Devin and ZCode, Orca rewrites the tab title from hook state ("Codex - action
  required", "Codex ready") and animates its own spinner.
- **Interrupt inference** (`src/shared/agent-interrupt-intent.ts`). Orca measured that current
  Claude sends no hook on a cancel, so Ctrl+C typed into the pane (and Escape for agents where
  Escape does not also close an overlay; a second Escape for OpenCode and Copilot) marks the turn
  `done` with a cancellation verdict after 500 ms, re-checked in main against a baseline. The
  Claude docs describe an `is_interrupt` flag on Stop; Orca keeps reading it for builds that send
  it.
- **Answer inference** (`src/shared/agent-question-answered-intent.ts`). Enter or a digit typed
  into a pane showing a single-question AskUserQuestion clears the waiting state before the next
  hook arrives.
- **Decay** (`src/shared/agent-status-freshness.ts`, `src/renderer/src/lib/agent-row-decay-state.ts`).
  A non-done row older than 30 minutes becomes `unverifiable` ("No update in 34m", a dashed amber
  ring) while the PTY is alive, `idle` once it is not. It never becomes `done`.

**Good.** "Unverifiable" says what Orca knows instead of guessing; interrupt and answer inference
fix the two cases where hooks are silent. **Bad.** The title ladder is glyph trivia per TUI and
breaks with their releases. Errors are weak: a Claude `StopFailure` (rate limit, overload, auth)
is stored as a `failure` verdict on the row's `mainAgent`, but I found no renderer that reads it,
so the row shows done and the notification says "finished"; `marley_fleet` already treats Error
as its own state, which is the better model. **Marley today: has part** (quiet timer and bell
only).

### 2.4 Where agent state shows

- The shared glyph set (`src/renderer/src/components/AgentStateDot.tsx`, `AgentWorkingSpinner.tsx`,
  `AgentQuestionIcon.tsx`): working = yellow CSS ring spinner (phase-synced across mounts, static
  under reduced motion), monitoring = Activity icon, waiting/needs attention = `MessageCircleQuestion`
  in the `--agent-question` amber, done = emerald check on the dashboard and emerald dot in the
  sidebar, blocked/interrupted/failed = red dot, idle = grey dot, unverifiable = dashed amber ring.
  Two glyphs sit side by side: the agent's icon (who) and the state (what).
- Sidebar worktree cards list their agents inline with a summary ("3 working, 1 waiting",
  `worktree-card-agent-summary.ts`) and child rows. The "Smart" sort
  (`src/renderer/src/components/sidebar/smart-attention.ts`) orders worktrees: 1 needs you, 2 done
  in the last 30 minutes, 3 working, 4 unverifiable, 5 idle, ties by the attention time.
- Terminal tabs carry the same states (`tab-bar/terminal-tab-activity-status.ts`).
- The Agent Dashboard (Settings → Experimental; `src/renderer/src/components/dashboard/`,
  `dashboard-popout/`, 7,000 lines): a kanban of Needs You (waiting or blocked), Working, Done,
  and Idle (hidden by default). Cards show the conversation name, the last user and agent
  messages, the pending question's summary, a subagent disclosure, the host badge for SSH or
  remote runtimes, and age. Needs You cards tint amber, Done cards green. Clicking a card opens a
  dialog holding a live xterm view of that agent's pane that accepts input
  (`AgentTerminalDialog.tsx`, `AgentTerminalPreview.tsx`), so a prompt can be answered without
  leaving the board. It can pop out into its own window.

**Marley today: has part.** Rail rows show a status word and an attention dot; there is no sort by
attention, no per-project summary, no preview of the tool or message, and no board.

### 2.5 Notifications, unread and the Agents feed

**Triggers.** Per pane, `src/renderer/src/components/terminal-pane/agent-completion-coordinator.ts`
merges three sources: a hook `done` after `working`, a title going from working to idle, and the
agent process leaving the foreground. A hook `waiting` or `blocked` raises "needs input" at once
(`agent-completion-hook-observer.ts`); a `done` from Pi, OMP and goal-style agents waits 1.5 s so a
resumed milestone can cancel it; a lead turn that ends while background work keeps the row
`working` still notifies at the turn's end. A BEL from a pane that is not visible raises a
`terminal-bell` notification. Structured chats notify on every settled turn
(`native-chat/structured-attention-dispatch.ts`).

**Policy** (`src/renderer/src/attention/`, 450 lines). Provider-neutral: a surface adapter answers
"is a session alive", "is this surface on screen", "which group holds it", so terminals and chats
share one policy. A settled turn earns an unread marker only when its surface is not the one on
screen, and the marker is written before delivery, so a suppressed banner still leaves it. Reasons
are recorded (agent completion, terminal bell, manual mark-unread). Viewing acknowledges by
comparing the turn's `stateStartedAt` with the last acknowledged time, so repeated same-state
pings do not re-light it; a workspace stays unread while a hidden sibling still wants attention.

**Delivery** (`src/main/notifications/notification-delivery-service.ts`). Light the tray attention
dot when the window is hidden; fan out to a paired phone with the agent's state (so a push can say
"needs input" or "finished"); then the desktop gates: notifications on, the per-source switch
(agent finished, terminal bell), suppress-when-focused for the active worktree, a burst cooldown
per worktree (a finish and a bell in one chunk make one banner), and on macOS an authorization
check that falls back to an in-app toast. Text (`src/main/ipc/notification-options.ts`): title
`<repo> / <worktree> - Claude finished` (or `needs input`, `stopped`), body the last assistant
message cut to 180 characters, else `Using <tool>: <input>`. A custom sound file (MP3, WAV, OGG,
M4A, AAC, FLAC) with volume; the macOS Dock badge carries the unread count; Mark Unread on a
worktree's context menu or a feed row puts a marker back, and it lapses when the user moves on or
the agent starts a new turn (`computeLapsedManualUnreadProtections`).

**The Agents feed** (`src/renderer/src/components/activity/`, 6,000 lines). The bell in the
sidebar header switches the sidebar to a list of threads, one per agent pane, built from each
pane's state history (entries into done, waiting and blocked; working only while live), grouped by
status (waiting, blocked, permission, interrupted, working, monitoring, unverifiable, failed,
done, idle) or by project, worktree or agent, with unread badges, a hover card with the response
preview, Mark unread, Open, and "Clear completed". The selected thread's live terminal is
portaled into the detail pane (`activity-terminal-portal.ts`), so an agent can be answered from
the list. The docs still describe the header bell and the Agents page as two surfaces; the code
merged them.

**Good.** The attention module is the best-factored part of the area. Notification bodies carry
content. **Bad.** Six delivery gates plus renderer-side arbitration between hook, title and
process evidence (`use-notification-dispatch.ts`) make "why didn't I get pinged" hard to answer.
**Size.** Attention 450; completion and dispatch in the terminal pane 3,000; main notifications
1,300; feed 6,000.
**Marley today: has part.** `crates/marley_workbench/src/notifications.rs` turns OSC 9 and OSC 777
into a desktop notification unless that terminal is focused in the active window, and a click
shows the terminal. Terminal rows carry the bell; thread rows carry an attention dot when a run
ends unseen. No per-turn unread, no feed, no content beyond the plugin's fixed sentence, no sound
choice, no cooldown.

### 2.6 Session history ("AI Vault") and resume

**What the user sees.** Right sidebar → Agents: "Agent Session History", every past session of 20
agents on the machine, scoped to the workspace, the project or everything, searchable, sorted by
last update or creation, grouped by project, folder or agent. A row opens details (cwd, branch,
model, message count, the untruncated first prompt with Copy, the latest turns) and offers Resume,
Copy resume command, Copy session ID, Copy or Open or Reveal log, Open cwd, Delete, and Continue
in New Session; a row can be dragged into the workspace to resume it.

**How it works.** One table (`AI_VAULT_AGENT_SOURCES` in
`src/main/ai-vault/session-scanner-agent-sources.ts`) gives each agent's root and file predicate:
Claude `~/.claude/projects/*.jsonl` (subagent files read on demand), Codex `$CODEX_HOME/sessions`
plus Orca's account homes (titles from `session_index.jsonl`), Gemini `~/.gemini/tmp`, Copilot
`~/.copilot/session-state`, Cursor `~/.cursor/projects/**/agent-transcripts`, OpenCode's
`opencode*.db` SQLite (read in a worker), Devin's `sessions.db`, and JSON or JSONL stores for Grok,
Hermes, Rovo, Pi, Prime Agent, OMP, OpenClaw, Droid, Cline, Kimi, Muse and Antigravity; the usual
`*_HOME` overrides are honored. Scans run in a forked child process (384 MB heap, below-normal
priority, exits after 10 idle minutes, 130 s timeout, a circuit that opens after three faults),
keep the newest 2×limit files per agent by mtime (limit 250 by default), and parse append-only
JSONL incrementally from the last byte offset. A 4096-entry parse cache validated by mtime and
size persists to `<userData>/ai-vault/session-parse-cache.json`; results are cached 60 s. The
renderer rescans on mount, scope change, window focus and when a hook reports a session id it has
not seen (at most every 30 s). Claude's title is its `custom-title`, else the latest `ai-title`,
else the first prompt that is not harness-injected; its cwd is the first record's, because
`claude --resume` finds sessions by their start directory (#9361). A session maps to a worktree by
longest cwd-prefix match; nothing records which pane started it. Optional full-text search
(`src/main/ai-vault-search`): SQLite FTS5 with bm25 ranking, reindexed every 20 s, phrase then AND
then typo repair then OR, forks with the same first messages collapsed. On SSH hosts the relay runs
the same scanner remotely. Delete moves the transcript to the OS trash after checking the path
against the agent's roots, and is refused for Codex, Kimi, Antigravity and OpenCode whose indexes
or databases would keep a dangling entry.

Resume: a live pane's provider session id comes only from its hooks
(`extractAgentProviderSession` in `src/shared/agent-session-resume.ts`); Cursor, Hermes, Amp and
Command Code report none, so their live panes cannot resume. `getAgentResumeArgv` builds
`claude --resume <id>`, `codex resume <id>`, `gemini --resume <id>`, `opencode --session <id>`,
`pi --session <file>`, `copilot --resume=<id>` and so on; the resume plan keeps the user's command
override, arguments and env, so a bypass flag carries over, and for Claude any `--resume`, `-r`,
`--continue` or `-c` already in the base command is removed first. The command is typed into a new
tab's shell. A Codex resume first checks which account home owns the rollout file and links,
copies or moves it, or drops the resume and starts fresh when it cannot tell.

**Good.** One table drives both discovery and the delete guard; scan cost is bounded four ways;
identity comes from the provider's own hook payload. **Bad.** Twenty reverse-engineered formats with
no upstream contract; cwd-prefix mapping files a session started above the worktree under the
wrong one; the Pi resume command differs between scanned rows and live panes.
**Size.** About 26,000 lines in main (`ai-vault` 19,900, `ai-vault-search` 6,500), 10,500 in the
renderer panel, 1,800 shared; the tests are larger than the code.
**Marley today: lacks it.** Zed keeps a history of Agent Panel threads (the rail lists them from
`ThreadMetadataStore`); nothing lists or resumes Claude Code or Codex sessions run in terminals.

### 2.7 Hibernation

**What the user sees.** Settings → Experimental → Agent hibernation (off by default), "Hibernate
after" 1 minute to 24 hours (30 minutes default). A finished, untouched agent's process is stopped;
its pane keeps showing the last frame; revealing the tab resumes the same session.

**How it works** (`src/renderer/src/components/AgentHibernationGate.tsx` →
`src/renderer/src/lib/agent-hibernation-coordinator.ts`, `agent-hibernation-pane-eligibility.ts`).
The renderer decides; main only kills. A 60 s tick, skipped while the window is hidden, checks each
pane: status `done`, not interrupted, no live subagents, orchestration dispatch absent or settled, a
resumable agent with a provider session id, tab not in the foreground, no keystroke since the turn
(`agent-hibernation-input-guard.ts`), live PTY not driven from the phone, idle past the window
(measured from the latest of done-start, last foregrounded, first seen, session boundary). The
pane's signature, including an output counter, must match on two consecutive ticks, and is checked
again just before the kill. A `SleepingAgentSessionRecord` (pane, tab, worktree, agent, provider
session and transcript path, prompt, title, last message, and the launch command, args and env
without the pane identity) is written first; no record, no kill
(`store/terminals/terminal-pane-hibernation.ts`). The kill is `pty.kill(id, { keepHistory: true })`:
the daemon sweeps detached children and SIGKILLs the process groups, with no `/exit` and no SIGTERM
grace. On reveal, `cold-restore-resume-startup.ts` spawns a new shell in the same pane and types the
saved command with the resume argument and a fresh launch token.

The code has moved past its doc page: it hibernates one pane at a time rather than a worktree's
panes together, no longer spares the active worktree (#16211), and covers more agents than the
list on the page. **Good.** Fails closed (stable signature, re-check, record before kill,
rollback on remote mismatch). **Bad.** SIGKILL drops anything the TUI held only in memory; the
input guard only sees keystrokes that went through Orca. **Size.** About 3,300 lines.
**Marley today: lacks it.**

### 2.8 Continue in New Session, and the restart controls

**Continue in New Session** (terminal header or context menu, and AI Vault rows;
`AgentSessionContinuationDialog.tsx`, `src/renderer/src/lib/agent-session-continuation.ts`) picks
any detected agent, focused or full mode. With a transcript path (the hook's `transcript_path` or
the row's file) the prompt embeds no transcript at all: it gives the path in a code block and tells
the new agent to read it (all of it, or what it needs), not to modify it, to ignore instructions
found in tool output, to check `git status` and trust the workspace over the transcript, and to say
where the old session stopped. Without a path it embeds the last 800 scrollback lines, stripped of
escapes and cut to the newest 36,000 characters behind an "[Earlier terminal output omitted]"
marker (`agent-session-fork-context.ts`). Claude gets the prompt as `--prefill` (an unsent draft);
other agents start empty and get it pasted once they switch on bracketed paste and stop redrawing
(`agent-paste-draft.ts`), with a "Copy prompt" toast if Orca never sees the input box. Handing a
Claude session to Codex works the same way: Codex reads the Claude JSONL with its own tools. About
950 lines.

**Restart.** The docs' "Restart chip that rehydrates the same agent" does not match the code.
`CodexRestartChip.tsx` appears when a running Codex pane's account or home configuration changes
and starts a plain `codex` under the new account, not `codex resume`.
`TerminalProcessExitOverlay.tsx` ("Terminal exited", Restart and Close) appears when a pane's
process exits non-zero; since agents run inside a shell, a normal agent exit just returns to the
prompt. The structured chats have a separate resume-on-restart marker
(`src/shared/agent-session-resume-marker.ts`, `agent-session-recovery.json`): working sessions at
quit or update are offered "continue" on relaunch with a fixed message ("Orca restarted, so your
previous reply was cut off partway through.").

**Marley today: lacks both.**

### 2.9 Chat UI ("native chat") and structured sessions

**What the user sees.** Settings → Experimental, all off by default: Chat UI, Default view (terminal
or chat), and "Use updated structured native chat". A chat tab shows the transcript (messages,
reasoning, folded tool runs, per-edit diff cards with Copy diff, subagent and background-task runs,
the task list, turn timing), a composer (`/` menu of the agent's commands plus discovered skills,
model, effort and fast-mode pickers, a context-usage ring, image paste and file drop, Stop), and
cards for questions (multi-question stepping with options, multi-select, free text) and approvals
(title, reason, blocked path, the matching ask rule, the plan for ExitPlanMode). Cmd/Ctrl+Shift+J
flips a terminal agent tab between xterm and chat.

**How it works: two lanes, chosen per tab** (`resolveAgentLaunchRoute` in
`src/renderer/src/lib/agent-launch-routing.ts`, blockers in
`src/shared/structured-native-chat-launch-route.ts`).

- **Terminal-backed chat**, the lane the docs describe. The agent runs as its normal TUI. Main
  tails the provider's own JSONL (`src/main/native-chat/transcript-watch*.ts`, 1 s poll plus
  debounced `fs.watch`), found through the hook's `transcript_path`, then `~/.claude/projects`,
  Orca's managed `CODEX_HOME/sessions` and so on; per-agent decoders exist for Claude, Codex,
  Grok and OMP. Sends are PTY bytes: single lines as typed, multi-line text as bracketed paste and
  a delayed `\r`, images as pasted file paths, Stop as ESC, Codex `/model <id>` typed key by key,
  AskUserQuestion answered with option digits, arrows and Tab in groups 500 ms apart
  (`src/shared/native-chat-ask.ts`; pasting option labels committed the first option, STA-1860).
  A pane with no transcript falls back to ANSI-stripped scrollback marked approximate.
- **Structured sessions**, no PTY, local host only, Claude and Codex only. Claude runs through
  `query()` from `@anthropic-ai/claude-agent-sdk` 0.3.251 against the user's installed `claude`
  (`src/main/claude/claude-stream-json-connection.ts`), with Orca's own spawner so it owns the pid;
  prompts are `SDKUserMessage`s on an open async queue, controls are SDK calls (interrupt,
  set_model, set_permission_mode, stop_task, context usage). Codex runs `codex app-server` with
  `CODEX_HOME` pinned (`src/main/codex/codex-app-server-connection.ts`): newline-delimited
  JSON-RPC, `thread/start` or `thread/resume`, `turn/start` with a client message id,
  `turn/interrupt`, `thread/compact/start`, `thread/revert`. A send counts as delivered only when
  the provider echoes it.
- **The journal** (`src/main/native-chat/agent-session-journal/`): one SQLite database per session at
  `<userData>/agent-session-journal/<hash(workspace)>/<hash(session)>/journal.db`, outside the
  worktree. Append-only rows of message, tool-call, diff, approval, question, status and turn
  items; write-ahead submissions; epochs bound to a provider handle; payloads over 16 KiB kept as a
  head plus length and SHA-256. Desktop and the phone read it by (epoch, sequence) cursor; after a
  crash, pending submissions become "unknown" and are matched against provider history, never
  re-sent; prompts are answered by compare-and-set, so two devices cannot both answer.
- **Approvals and questions**, structured lane only. Claude's `can_use_tool` becomes an approval or
  (for AskUserQuestion) a question item; Allow returns the input, "Allow for this session" returns
  the SDK's suggested permission updates, Deny a message, Stop a deny with interrupt
  (`claude-structured-prompt-replies.ts`). Codex `requestApproval` requests map to its
  `availableDecisions`; MCP elicitation and other requests get safe automatic answers. There is no
  queue of pending prompts across sessions: a pending prompt makes its session "attention", and in
  a tab only the oldest one shows. With Orca's bypass default, approval cards appear only for users
  who chose Manual.
- **Per-turn diffs.** Edits are rebuilt from tool calls, never from git
  (`src/shared/native-chat-edit-normalize.ts`): Claude Edit/MultiEdit/Write snippets diffed with LCS
  and without line numbers, Claude's `toolUseResult.structuredPatch` from the transcript (the only
  source with real line numbers, so terminal-lane only), Codex `*** Begin Patch` envelopes and
  structured `fileChange` items with the app-server's own diffs. A per-turn rollup ("N changed
  files", +/−) appears only for structured Codex turns (`native-chat-turn-diffs.ts`). No card can
  revert. Rewind (`thread/revert`) is Codex-only, conversation-only as far as Orca's code goes, and
  has no caller in the UI.

**Good.** Honest delivery semantics (write-ahead, echo-confirmed, no silent resend), durable
prompts answered once, one status projection shared by sidebar, CLI and phone. **Bad.** The
terminal lane rests on undocumented transcript fields and TUI keystroke choreography; the
structured lane pins SDK and CLI versions, calls SDK methods missing from its typings, and matches
English error strings; parity gaps everywhere (two agents, local only, rollups for Codex only).
**Size.** About 25,700 lines in `src/main/native-chat`, 27,500 in the renderer component
directory, plus 6,400 (`claude-structured-*`) and 8,000 (`codex-structured-*`); tests are larger.
**Marley today: has the structured half through Zed.** Zed's Agent Panel runs Claude Code
(`claude-agent-acp`, built on the same Agent SDK) and Codex (`codex-acp`) as ACP agents, shows
tool calls with permission prompts that `AcpThread::authorize_tool_call` answers in place, and
keeps the edits a thread makes through Zed in `action_log` with keep and reject per hunk, which is
more than Orca's cards offer. What Marley lacks is Orca's terminal half (a chat view over a TUI's
transcript) and a durable, multi-device journal.

### 2.10 Subagents, teammates and Claude Agent Teams

**What the user sees.** Indented child rows under a lead agent in the sidebar, the dashboard and
the feed, one per live subagent or teammate, with their own state glyphs; clicking one focuses the
lead's pane (children have no pane of their own). With Agent Teams enabled, each teammate gets a
real Orca split pane.

**How it works.** Claude children are tracked from hooks: `src/shared/claude-subagent-roster.ts`
keeps a per-pane map keyed by `agent_id`, fed by SubagentStart, SubagentStop, TeammateIdle and a
child's StopFailure (`providers/claude-lifecycle-events.ts`); one-shot children (`a<hex>` ids)
leave on SubagentStop, teammates (`a<name>-<hex>`) go idle instead. The lead's Stop payload carries
`background_tasks`, which Orca treats as an inventory that recreates, keeps or drops rows. Only
working rows keep the pane spinning; the roster caps at 32 rows. After a restart, rows restored
from disk are swept only when the daemon proves the PTY is gone
(`src/main/agent-hooks/restored-subagent-liveness-sweep.ts`); before that sweep existed a dead PTY
left a pane "working" forever. Codex has no child hooks, so Orca tails the parent rollout for
`sub_agent_activity` records and each child's own rollout file, polled every second while hooks
flow (`src/shared/codex-subagent-transcript.ts`). Structured sessions read `task_started` and
`task_notification` frames instead. `src/shared/agent-status-child-work*.ts` (20 files, 2,900
lines) is one provider-neutral record for anything a main agent spawned (agent, workflow,
command, monitor), with invocation fences, a legality matrix and caps, because each past failure
got its own module.

Agent Teams: the picker's `claude-agent-teams` entry runs `orca claude-teams`, which asks the
runtime for a team env and starts `claude --teammate-mode auto`. The env puts a fake `tmux` first
on `PATH` (`~/.orca/claude-agent-teams-bin/tmux`, 11 lines of sh that forward to the Orca CLI with
a 32-byte team token), sets `TMUX`, `TMUX_PANE` and `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`, and
removes `TERM_PROGRAM` (`src/main/runtime/claude-agent-teams-service.ts`). Claude's tmux backend
then drives Orca: `split-window` becomes an Orca split, `send-keys` a terminal send, `capture-pane`
a 1,000-line read, `kill-pane` a close that refuses the leader
(`claude-agent-teams-tmux-dispatcher.ts`); `respawn-pane -k`, which an Orca PTY cannot do, is
emulated by closing the placeholder and splitting again under the same pane id. Not on Windows or
WSL; team state is in memory.

**Good.** The tmux shim gives native teammate panes with no change to Claude, in about 1,000
lines. **Bad.** Rosters rest on undocumented id shapes and `background_tasks` semantics of specific
Claude releases (2.1.21x, 2.1.280) and on Codex's rollout file layout.
**Size.** About 11,000 lines across rosters, child work, background tasks and Agent Teams.
**Marley today: lacks it.** The rail shows one row per terminal or thread; nothing shows a
terminal agent's children.

### 2.11 Fan-out, orchestration and automations

**Fan-out as the docs describe it is a recipe, not a feature.** `recipes/parallel-agents.mdx`
("Race three agents") is manual: create three worktrees from one base, launch a different agent in
each, paste the same prompt, split panes to watch, review each diff, keep the winner, delete the
others. The new-workspace composer takes one agent (`NewWorkspaceComposerAgentSection.tsx`); there
is no launch-N, no broadcast of one prompt to several panes, and no view that compares the
resulting branches. Comparison is the per-worktree diff viewer and review notes. What does exist is
programmatic: `orca worktree create --name child --agent codex --prompt "…"` (records the new
worktree as a child of the caller's, so Sleep or Delete with Descendants works on the tree), and
the orchestration layer below.

**Orchestration** (experimental in the docs, `orca orchestration …`, the bundled `orchestration`
skill). A coordinator agent creates a Run, Tasks with dependencies, and starts workers:
`worker-start --task <id> --worktree current|new-child --agent codex [--model --effort]`. State is
SQLite at `<userData>/orchestration.db` (schema version 42: runs, tasks with a CHECK-constrained
status, dispatch contexts, typed messages, deliveries, decision gates, receipts). A worker start
records the dispatch, places the agent (optionally in a new child worktree with setup hooks), waits
up to 60 s for `tui-idle`, mints a capability (32 random bytes, stored hashed, bound to the pane
and process incarnation) and pastes a preamble (`src/main/runtime/orchestration/preamble.ts`): send
`worker_done` exactly once with an outcome and both ids, heartbeat every 5 minutes, use
`orca orchestration ask` instead of AskUserQuestion, return to the idle prompt after finishing,
then `=== TASK ===` and the spec. For Claude, Orca first types the lead line "Please carry out this
task from my Orca coordinator by following the brief I pasted below." because Claude Code acts on
a pasted block only when the typed words ask it to. `worker_done` is accepted only from the
assignee's pane with matching ids; a worker whose PTY exits fails its dispatch and escalates to the
Run; three failures trip a breaker. When mail arrives for an idle agent, the runtime types
``You have N orchestration messages. Run `orca orchestration check --run <id>`.`` into its pane.
Group addresses (`@all`, `@idle`, `@codex`, `@worktree:<id>`) expand per Run. Decision gates block a
task until the coordinator resolves them.

**`tui-idle`**, the readiness test orchestration and prompt delivery share
(`src/main/runtime/runtime-terminal-wait.ts`, `tui-idle-evidence.ts`; 2 s poll, 3 s quiet window):
first refuse on a blocking dialog found in the last 12 non-blank lines (trust, update, "press enter
to", permission menus naming allow/deny); then accept an explicit idle title or a known ready
screen (Codex's `model:` and `directory:` banner, Cursor's arrow); a fresh OSC 9999 "working"
vetoes; a title that only names the agent counts after 3 s of quiet for agents that later announce
rest, at once for the rest; an unknown TUI counts as idle after 3 s of quiet.

**Automations** (`src/main/automations`, 4,600 lines): scheduled agent runs by preset, cron or
RRULE, checked every 60 s, with a shell precheck and missed-run records; a run ends at the next
`tui-idle` (it must leave idle within 2 minutes to prove it started; observation stops at 6 hours).

**Good.** Delivery is receipted and never pasted twice; `worker_done` needs the right pane and
both ids; one settle test serves every path. **Bad.** About 39,000 lines of orchestration for an
experimental feature, with a retired coordinator still in the tree; `OrcaRuntimeService` is a chain
of 131 mechanically split classes and 168 runtime files carry `@ts-nocheck`, including prompt
writing and dispatch failure on exit; readiness reads English banners that change with each CLI
release. **Marley today: lacks it** (Marley's MCP server can list and read terminals but not start
agents or dispatch work; Chad's rustal-harness is the planned home for supervised workers).

### 2.12 Usage, rate limits and accounts

**What the user sees.** A status-bar segment per provider (icon, a mini-bar for the tightest
window, a percentage per window); a Usage popover listing Claude, Codex, Gemini, Kimi, Grok,
OpenCode Go and MiniMax with reset countdowns, highest usage first; bars go yellow at 60% and red
at 80% in the popover and switcher (there is no "80% warning chip" as the docs say). An account
switcher for Claude and Codex with each account's usage; Codex "rate-limit reset credits" that
can be spent from desktop or phone; a Stats page with estimated cost.

**Where the numbers come from.** Not from local files alone: the docs' "No API calls, no extra
auth" holds only for the first Claude source below.

- Claude, in order: (1) the managed `statusLine` script, installed only into an empty slot, which
  prints nothing and posts Claude's `rate_limits` JSON (5-hour and 7-day windows with `resets_at`,
  present for Claude.ai subscribers after the first response) to the hook server at most every
  15 s per pane (`src/main/claude/statusline-script.ts`, `src/shared/claude-statusline-rate-limits.ts`);
  (2) `GET https://api.anthropic.com/api/oauth/usage` with the account's OAuth token, sending
  Claude Code's user agent (`src/main/rate-limits/claude-oauth-usage-request.ts`), which also
  gives the weekly Fable window; (3) a hidden `claude` session in a scratch directory that types
  `/usage`, presses Enter every 800 ms, answers trust prompts and reads the panel with regexes
  (`claude-pty.ts`, not on Windows), used when OAuth fails or omits Fable; (4) Orca refreshing
  managed accounts' OAuth tokens itself, five minutes before expiry.
- Codex: (1) `codex app-server` spawned read-only with `CODEX_HOME` set, `account/rateLimits/read`
  over JSON-RPC, under a per-home lock because two processes refreshing one home would spend a
  single-use refresh token twice (`codex-rpc-rate-limit-probe.ts`, `codex-cli/codex-home-process-lock.ts`);
  (2) `https://chatgpt.com/backend-api/wham/usage` with the token from `auth.json` and Codex's
  client headers; (3) a hidden `codex` session typing `/status`. Rollout files are not used.
- Others are HTTP calls: Gemini's internal quota API after Orca refreshes `~/.gemini/oauth_creds.json`
  with a client secret it extracts from the installed CLI by regex; Kimi, Grok, OpenCode Go and
  MiniMax (cookie or API key, stored with Electron `safeStorage`).
- Polling every 15 minutes, only while the window is visible and focused, plus a refresh on focus
  when data is over 5 minutes old; state is in memory only; last good numbers stay on screen,
  marked stale, for 30 minutes (24 hours after a 429). Reset times always come from the provider;
  scraped text like "resets in 2h 13m" is parsed by `claude-pty-reset-parser.ts`.

**Accounts.** Claude: each account's credentials live under `<userData>/claude-accounts/<id>/auth/`
(Keychain on macOS, `.credentials.json` 0600 elsewhere); adding one runs `claude auth login` with a
temporary `CLAUDE_CONFIG_DIR`. Switching is not a pointer rewrite as the docs say: Orca copies the
chosen account's credentials into the one place every Claude on the machine reads
(`~/.claude/.credentials.json`, both macOS Keychain items, the `oauthAccount` field of
`~/.claude.json`), first saving any tokens a running Claude refreshed back to the account they
belong to (`runtime-auth/runtime-auth-sync.ts`, `runtime-auth-readback.ts`). The "guard" is an
app-wide switch flag that refuses a second switch and Claude launches during a switch, plus a
persisted list of live Claude processes while which Orca never refreshes the active token itself
(`claude-accounts/live-pty-gate.ts`). Running sessions keep their in-memory token over credentials
that now belong to another account; the switcher tells users to restart them. Codex: each account
is a full Codex home at `<userData>/codex-accounts/<id>/home/`, and switching only changes which
`CODEX_HOME` the next launch gets; `codex-config-mirror.ts` copies `config.toml` both ways between
`~/.codex` and the account home (including `/model` changes made inside an Orca-launched Codex,
despite a comment saying Orca never writes `~/.codex`), symlinks skills, hooks, plugins, prompts and
`AGENTS.md`, and hardlinks other homes' session files so `/resume` finds them. Reset credits are
spent through `/wham/rate-limit-reset-credits/consume` with an idempotency key and a persisted
attempt ledger that refuses a redeem if the offer the user saw has changed
(`codex-reset-credit-coordinator.ts`, `codex-reset-credit-ledger.ts`).

**Cost.** `src/main/stats` only counts agents, agent time and PRs. Estimated cost comes from
opt-in scanners of `~/.claude/projects` and Codex session files (`src/main/claude-usage`,
`codex-usage`) priced from hardcoded tables, flagged "inferred pricing" for unknown models.

**Good.** The statusline feed gives near-live Claude quota for free; stale data is kept visible
and labeled; Codex account homes isolate cleanly; single-use refresh tokens are handled with care;
the reset-credit ledger is the right shape for spending something scarce. **Bad.** It rests on
private endpoints called with the vendors' own client identities, on scraping hidden TUI sessions
with timers and English regexes, and on rewriting the user's shared Claude credentials (including
an unlocked read-modify-write of `~/.claude.json`); credentials pass through a `security -w`
command line on macOS. `RateLimitService` is one class spread over a 14-level inheritance chain.
**Size.** About 49,000 lines including the renderer: `rate-limits` 10,500, `codex-accounts` 7,500,
`claude-accounts` 4,700, usage scanners 6,000.
**Marley today: lacks it.**

### 2.13 Smaller agent features the docs do not cover

- **Sending text into a running agent** (`src/renderer/src/lib/running-agent-targets.ts`,
  `active-agent-terminal-send-readiness.ts`). Review notes and other "send to agent" actions list
  the worktree's agents as targets and disable any whose row is stale, whose PTY is gone, or that
  is waiting on permission, because typed text would answer the permission prompt. Before sending,
  the host is asked `terminal.agentStatus`; older runtimes fall back to waiting for `tui-idle`.
- **How a prompt is typed into a TUI** (`src/shared/agent-prompt-injection.ts`,
  `src/main/runtime/orca-runtime-write-terminal-agent-prompt.ts`). For agents that take no argv
  prompt, wait for `tui-idle` (up to 60 s) and do not paste if the wait fails, since the paste
  would answer whatever dialog is showing. Then one bracketed-paste write (`ESC[200~ … ESC[201~`)
  with every ESC byte inside the text replaced by a literal `<ESC>` so the text cannot end the
  paste early; a check that no permission prompt is showing, before the paste and again before
  Enter; for Claude and Codex, Enter after the cursor-show sequence `ESC[?25h` and 1.5 s of quiet
  (8 s cap), for others after 500 ms plus the paste's ingest time (about 4 KB/ms on macOS and
  Linux, 64 B/ms under Windows ConPTY); receipts `input_accepted` and, where hooks can confirm it,
  `turn_started`; never a second paste when confirmation does not come. For Claude, Orca types a
  short lead line before the paste because Claude Code acts on pasted text only where the typed
  words ask it to (`pasteNeedsTypedRequest`); for Codex it must not, because Codex drops typed
  text that shares the paste's write (STA-8200).
- **Prompt cache timer** (Settings → Agents; `AgentCacheTimerSection.tsx`, `promptCacheTtlMs`, 5
  minutes default, 1 hour option): a countdown in the sidebar after a Claude agent goes idle,
  until its prompt cache expires and the next message pays for the full context again.
- **Branch rename on first work** (`src/main/agent-hooks/first-work-branch-rename.ts`, 322 lines):
  a new worktree gets a placeholder branch like `you/Nautilus`; on the agent's first prompt Orca
  runs a headless agent (`claude -p --permission-mode plan`, or the user's chosen CLI and model,
  `src/shared/commit-message-agent-specs-primary.ts`) to name the branch from the prompt, renames
  the branch, the folder and the sidebar title, and shows a "rename failed" badge with the CLI's
  output when it fails.
- **Keep awake** (`src/main/agent-awake-service.ts`): modes On, Agent and Off (also a coffee icon
  in the status bar). In Agent mode, while any hook-reported agent works, Electron's
  `powerSaveBlocker('prevent-display-sleep')`, a macOS assertion, and on Linux
  `systemd-inhibit --what=sleep:handle-lid-switch --why="Agents are working" --mode=block`
  (`linux-lid-sleep-assertion.ts`). A status older than 2 hours stops counting.
- **Stats** (`src/main/stats/`): `agent_start` and `agent_stop` events from status transitions and
  `pr_created`, aggregated into agents spawned, total agent time and PRs created (the cost column
  is in 2.12).
- `src/main/agent-state-file-reader.ts` (19 lines), named in the brief, is only a bounded JSON
  reader (4 MiB, a million tokens, depth 128) used on Codex state files; it plays no part in
  status detection.

## 3. Bring to Marley

Ranked. Items 2, 3 and 6 are in the queued sprint (#508, #509, #510); each says what Orca teaches
the ticket. Item 1 is what they stand on.

1. **Hook-driven state for agents in terminals, carried in-band by the plugin.**
   *Why.* The inbox, per-turn diffs, useful notifications, an attention-ordered rail and a real
   `fleet_snapshot` all need to know what a terminal's Claude is doing; today the rail guesses
   from 2 s of quiet. Orca shows what the hook stream gives: turn start and end, the tool and its
   input, the pending question, the last message, children, the session id and transcript path.
   *Seam.* The plugin's `hooks/hooks.json` registers Orca's Claude set: UserPromptSubmit,
   PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Stop, StopFailure,
   SessionStart, SessionEnd, SubagentStart, SubagentStop, PostCompact (not PreCompact). Each hook
   answers with a `terminalSequence` carrying a Marley frame, either a new hook name in the
   existing DCS codec (`crates/marley_terminal/src/dcs.rs`, found by `marley_dcs`'s scanner in
   `vendor/alacritty_terminal/src/marley_hooks.rs`) or an OSC with a JSON body like Orca's OSC
   9999: event, `session_id`, `prompt_id`, tool name, a short input preview (Orca's per-tool key
   table), `last_assistant_message` cut to a few hundred characters, `transcript_path`. The
   terminal emits an agent-state event, `crates/marley_rail` keeps the row, and
   `marley_workbench` also feeds `marley_fleet` so the MCP `fleet_snapshot` stops being empty.
   In-band beats Orca's HTTP server here: the terminal that receives the bytes is the pane, so
   Orca's pane-key env, launch tokens, endpoint file, spool, fences and SSH relay have no
   counterpart; frames reach Marley over Zed's SSH remoting unchanged because they ride the PTY
   stream; and `notify.sh`'s `TERM_PROGRAM` check already keeps other terminals silent.
   *Rules to copy.* UserPromptSubmit, PreToolUse, PostToolUse → working; PermissionRequest and
   PreToolUse of AskUserQuestion → waiting; Stop → done; StopFailure → failed (Orca records it and
   then shows done; keep it red, as `marley_fleet` already models Error); SessionStart with source
   startup, resume or clear → an idle boundary that never notifies; SessionStart compact ignored;
   the post-compaction "This session is being continued…" UserPromptSubmit ignored, and other
   harness-injected prompts (`<task-notification>`, `<system-reminder>`, `<teammate-message>`;
   the observed list is in `src/shared/harness-injected-user-turns.ts`) count as work but never
   replace the user's prompt as the row's label; a manual PostCompact → done, since `/compact`
   ends without Stop; events carrying `agent_id` come from children and leave the lead's state
   alone, except that a child's permission prompt makes the row wait; `background_tasks` on Stop
   keeps the row working;
   30 minutes of silence → "no update in N m", never done; Ctrl+C typed into the terminal during
   a turn → interrupted after 500 ms. Orca measured that current Claude sends no hook on a cancel
   and ignores a bare Escape for Claude because Escape also closes overlays; the hook docs
   describe a Stop with `is_interrupt: true`. Test which is true on the installed Claude before
   relying on either.
   *Size.* M.
   *Hard.* Authenticity: the shell nonce is taken out of the environment before user files run
   (`crates/marley_terminal/src/shell_integration.rs`), so a program printing the frame can fake
   agent state; either export a separate agent nonce for the plugin to echo, or accept, as Orca
   does for OSC 9999, that status is unauthenticated display data. Confirm where
   `terminalSequence` belongs in the hook output and on which events it is honored (the plugin
   uses it at the top level today; the docs summary I got places it under `hookSpecificOutput`).
   Accept frames on the alternate screen too (shell hooks there are skipped today). Each tool call
   costs one `sh` and `printf`, far cheaper than Orca's `curl` per event.

2. **The approvals inbox. Queued as #508.** What Orca teaches the ticket:
   - One "needs you" state (waiting or blocked), one amber question glyph and one count
     everywhere (`AgentQuestionIcon`, the `--agent-question` token). Sources: item 1 for terminal
     agents, ACP `ToolCallStatus::WaitingForConfirmation` for Agent Panel threads.
   - An entry names the agent and project and says what is asked: the tool and its input
     (`Bash: rm -rf build`) or the AskUserQuestion's question and options (Orca keeps the full JSON
     on the row as `interactivePrompt`), with the wait's age. Oldest first.
   - ACP entries can be answered in place with the offered options (`AcpThread::authorize_tool_call`),
     which Orca offers only for its own structured chats. Terminal entries open the terminal, as
     Orca's dashboard does with its live-terminal dialog and its feed with a portaled terminal.
   - An entry clears on the session's next hook (PostToolUse, Stop, UserPromptSubmit), on Enter or
     a digit typed into a terminal showing a single question (Orca's answer inference), or when
     the terminal closes.
   - Never paste picks, annotations or recordings into a terminal whose agent is waiting: the text
     would answer the prompt (Orca disables such targets, `running-agent-targets.ts`).
   - One "needs input" notification per wait.
   *Size.* M after item 1.

3. **Per-turn diffs for Claude Code in a terminal. Queued as #509.** What Orca teaches:
   - Open a turn on UserPromptSubmit (keep `prompt_id`); close it on Stop, StopFailure, an
     inferred interrupt, or a manual PostCompact. A harness-injected prompt (a background task's
     `<task-notification>`, a teammate message) also starts agent work that can edit files, but the
     user did not ask for it: list it as its own turn marked as such, titled by what injected it,
     not by the injected text; the post-compaction continuation is not a turn.
   - Subagents and background tasks can keep editing after the lead's Stop (Orca's
     `background_tasks` inventory keeps the row working); keep the turn open until that drains,
     or late edits land in the next turn.
   - Orca rebuilds edits from tool calls (Edit and Write snippets diffed without line numbers;
     Claude's `toolUseResult.structuredPatch` from the transcript for real ones), shows cards with
     no revert, and has a per-turn rollup only for structured Codex. Marley's planned tree
     snapshots are stronger because they catch edits made through Bash (sed, formatters,
     generators) that tool-call reconstruction misses; tool calls can still attribute hunks.
   - Zed's `action_log` already gives keep and reject per hunk for Agent Panel threads; the terminal
     turn view should reach the same bar with Zed's multibuffer diff.
   *Size.* M to L. *Hard.* Snapshotting a large tree per turn; concurrent agents in one worktree
   share the tree, which is an argument for #510's worktree-per-agent.

4. **Notifications that say something.** *Why.* Today's banner is a fixed sentence
   ("<project> needs your permission"), so every alert means opening the terminal to find out
   what happened. Title `<project>: Claude finished` (or needs input, failed, stopped); body the
   Stop's `last_assistant_message` cut short, else `Using <tool>: <input>`; the unread marker
   written even when the banner is suppressed; acknowledged by viewing, keyed on the turn's start
   so repeated pings do not re-light it; a per-project burst cooldown so a finish and a bell make
   one banner; a session boundary never notifies.
   *Seam.* Item 1's frames into `crates/marley_workbench/src/notifications.rs`, replacing
   `notify.sh`'s fixed sentence. *Size.* S. *Hard.* Nothing beyond item 1.

5. **Rail: attention order, summaries, tool step.** *Why.* With several projects open, the rows
   that need Chad should come first without expanding every project. Orca's Smart sort (needs
   you, done and unseen, working, not reporting, idle) for projects and rows; a collapsed
   project's summary ("2 working, 1 waiting"); under a working row, the tool and its input; "no
   update in 34m" rather than a guess. *Seam.* `crates/marley_rail` (pure; it already has
   `thread_attention`) and `marley_workbench/src/rail.rs`. *Size.* S to M. *Hard.* Rows that jump
   while the pointer is on them; hold the order while the rail is hovered.

6. **Launch recipes for agent CLIs. Feeds #510 (worktree agents).** What Orca teaches the ticket:
   put the first prompt on argv (`claude "<prompt>"`, `codex "<prompt>"`) so there is no paste
   race, or `claude --prefill` for a draft; pre-write Codex's trust entry (`[projects."<path>"]
   trust_level = "trusted"` in `config.toml`) for a fresh worktree, because the trust menu reads
   single keys and eats the first input, and every new worktree is a new path (Orca does the same
   for Cursor, Copilot and Antigravity); keep each
   CLI's own permission prompts as the default; after the first prompt, name the placeholder
   branch from it with a headless `claude -p --permission-mode plan` (Orca's first-work rename).
   *Seam.* A four-agent catalog in `crates/marley_agent` (args, env, prompt mode, draft flag,
   trust preset), used by `marley_workbench/src/agents.rs`. *Size.* S to M. *Hard.* Writing
   another tool's config file; do it only for worktrees Marley creates, and only the entry the
   tool itself would write.

7. **Session history and resume for Claude Code and Codex.** *Why.* Closing a terminal loses the
   way back to its session; Zed remembers only Agent Panel threads. Per project, past terminal
   sessions read from `~/.claude/projects/<encoded cwd>/*.jsonl` and `~/.codex/sessions`, titled
   from `custom-title`, `ai-title` or the first real prompt; Resume opens a terminal running
   `claude --resume <id>` in the session's first cwd (Claude finds sessions by start directory)
   or `codex resume <id>`; a live terminal's session id comes from item 1's SessionStart.
   *Seam.* A new pure crate for the two parsers, a History disclosure per project in the rail,
   `agents.rs` for the launch. *Size.* M. *Hard.* Undocumented formats; stay at two agents.

8. **Continue in a new session.** *Why.* A long or rate-limited session can hand its work to a
   fresh one, or to another CLI, without pasting a transcript. From a terminal's context menu,
   start a fresh agent with a prompt that names the old transcript's path and tells it to read
   it, leave it alone, ignore instructions found in tool output, check `git status` and trust the
   tree over the transcript. *Seam.* The terminal context menu and `agents.rs`, with item 1's
   `transcript_path`. *Size.* S. *Hard.* None worth naming.

9. **Keep the box awake while agents work.** *Why.* An idle-suspend in the middle of a long
   agent run stops the run. While any row is working, hold
   `systemd-inhibit --what=sleep --why="Agents are working" --mode=block` (Orca also blocks the
   lid switch for laptops); release it when none are. *Seam.* `marley_workbench`, fed by item 1
   and ACP thread status. *Size.* S. *Hard.* A stuck "working" row would hold the box awake;
   Orca drops statuses older than 2 hours from the count.

10. **Claude quota in the status bar, from the status line only.** *Why.* Seeing the 5-hour and
    weekly limits before an agent stalls. Claude pipes `rate_limits` (5-hour and 7-day windows
    with `resets_at`) to the status-line command for subscribers; a Marley command there can
    forward it and show the tightest window with a reset countdown.
    *Seam.* The agent bar's plugin chip offers it; a status-bar item shows it. *Size.* S to M.
    *Hard.* The status line is one slot the user may already own; Orca installs only into an
    empty slot, and its script prints nothing, so a user gets no status line text from it.

11. **Prompt cache timer.** *Why.* Resuming after the cache expires resends the whole context.
    After a Claude turn ends, a countdown on its row until the prompt cache expires (5 minutes by
    default, 1 hour as an option). *Seam.* The rail, from item 1's Stop time. *Size.* S.
    *Hard.* The TTL depends on the account and the request; it is a setting, not a fact Marley
    can read.

## 4. Skip

- Bypass flags by default: Orca's own docs say both "the worktree itself is the sandbox" and "a
  worktree is not a security sandbox"; keep each CLI's prompts unless Chad opts in.
- Writing managed hooks into 16 CLIs' user configs and trust hashes into Codex's config: Marley's
  opt-in plugin is the cleaner seam.
- The loopback hook server with endpoint file, spool, launch tokens, pane fences, aliases and
  relays: in-band frames make it unnecessary.
- Terminal-title glyph classification and process-table polling for 20 TUIs: per-release trivia;
  keep the 2 s quiet heuristic only as the fallback for agents without the plugin.
- Orca's own Agent SDK and app-server hosts, the session journal and the transcript-tailing chat
  view: Zed's Agent Panel with ACP agents already gives structured sessions, prompts and per-hunk
  keep and reject.
- Claude account hot-swap: it rewrites the shared Claude credentials under running sessions.
- Usage from private endpoints and hidden TUI sessions: vendor client identities, English regexes,
  fragile and likely against terms.
- The 39-agent catalog and the 20-format history scanner: breadth costs more than four agents need.
- The kanban dashboard and its pop-out window: the rail and the inbox cover it.
- Orchestration Runs, capabilities and mailbox typing: 39,000 lines, experimental, and
  rustal-harness is the planned home for supervised workers.
- The Agent Teams tmux shim: it needs no change to Claude, but it only matters if Chad runs
  Claude Agent Teams.
- Hibernation, for now: SIGKILL after 30 idle minutes pays off with dozens of agents; revisit with
  #510 if worktree agents pile up.
- Automations: a scheduled `claude -p` in a systemd timer does the same.

## 5. Open questions

1. For a Claude prompt in a terminal, should the inbox only open the terminal, or answer it? A
   `PermissionRequest` hook can return allow or deny, so the plugin could wait on Marley's MCP
   server for the choice, with Claude's own prompt as the fallback on timeout. Orca never answers
   a terminal agent's prompt remotely.
2. Agent frames: a new name in the nonce-checked DCS codec, with a nonce the plugin can read, or an
   unauthenticated JSON OSC like Orca's 9999 that any CLI could emit?
3. Where should Claude Code live by default, the Agent Panel over ACP (prompts and diffs already
   structured) or terminals? The answer sets how far items 1 and 3 go.
4. Permission posture for #510's worktree agents: each CLI's own prompts, or a per-project opt-in
   bypass?
5. Does Chad run more than one Claude or Codex account? If so, Codex's per-account `CODEX_HOME` is
   the one piece of Orca's account work worth copying.
