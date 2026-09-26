---
pipeline_id: 4a40ed7e-fc22-43ae-a2e7-32e1eeea47a9
ticket: docs/planning/tickets/open/TICKET-519-claude-code-events-in-the-rail.md
status: Phase 4 — Complete PASS
title: "Claude Code's hook events in the rail"
type: feature
slice: prong 2, C1's first piece (a terminal's own Claude Code events into marley_fleet and the rail, ahead of the harness adapter)
references: [docs/orca_architecture/README.md, docs/orca_architecture/01-agents-and-sessions.md, docs/orca_architecture/05-terminal-and-workspace.md, docs/orca_architecture/06-cli-automations-skills.md, docs/planning/pipeline/completed/478-terminal-notifications.spec.md, docs/planning/pipeline/completed/482-claude-code-notifications-chip.spec.md]
---

## Title
Marley's Claude Code plugin reports each hook event to the terminal Claude Code runs in, and the
rail shows what that session is doing: working, waiting on a permission, idle or failed, with the
prompt, the tool in flight and the turn's last message under the agent's row. Today the row
guesses from 2 s of quiet (`marley_agent::agent_status`). The same events fill `marley_fleet`, so
`fleet_snapshot` stops being empty, and #508 (the approvals inbox) and #509 (per-turn diffs)
build on them.

## Scope
### In
- **The hooks.** `claude_plugin/marley/hooks/hooks.json` registers SessionStart,
  UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure and PermissionRequest (matcher
  `*` on the tool events), Stop, StopFailure, PostCompact, SubagentStart, SubagentStop and
  SessionEnd, each running `hooks/event.py`. PreCompact is left out: an aborted compact sends it
  alone (Orca, `src/main/claude/hook-settings.ts`). The `Notification` and `Stop` entries for
  `notify.sh` stay as they are.
- **`hooks/event.py`**, Python 3 with the standard library only, as the bridge is. Outside a
  Marley terminal (`TERM_PROGRAM` is not `zed`) it prints `{}`. Inside, it reads the hook's JSON
  from stdin and prints `{"terminalSequence": "ESC ] 777 ; notify ; marley-event ; <base64> BEL"}`.
  The base64 holds a summary: `v` (1), `event`, `session_id`, `prompt_id`, `agent_id`,
  `permission_mode`, `cwd`, `transcript_path`, and by event `prompt` (UserPromptSubmit); `tool`
  and `preview` (the tool events and PermissionRequest: the tool input's `file_path`, `command`,
  `pattern`, `url`, `query` or `description`, `AskUserQuestion`'s first question's text, else the
  input's first string field); `tool_use_id` (the tool events; PermissionRequest carries none);
  `message` (Stop, StopFailure and SubagentStop's `last_assistant_message`); `error`
  (StopFailure); `source` (SessionStart); `trigger` (PostCompact); `reason` (SessionEnd); and
  `is_interrupt` (PostToolUseFailure). Prompt and message are cut to 300 characters, the preview to 200, and
  the JSON to 2,900 bytes, dropping an overlong transcript path and working directory, then
  message, then preview, then prompt while it is over, so the
  sequence stays under Claude Code's 4,096-byte cap and Marley's 4 KiB scanner cap. A failure of
  any kind prints `{}` and exits 0.
- **The plugin's version** moves from 1.1.0 to 1.2.0 (`plugin.json`, `marketplace.json`), and
  `claude_plugin::FILES` ships `event.py` as a program. (The "Update Marley's plugin" chip for an
  older install is slice 2, #547.)
- **The terminal.** Zed's `TerminalView` does not mark a notification titled `marley-event` as
  a bell (the one Zed change, inside #478's arm), and `marley_workbench::notifications` hands its
  body to the new `agent_events` module instead of showing it on the desktop.
- **The model.** `marley_agent::claude_events` (pure) decodes a body into a hook event and folds
  it into `marley_fleet` events for the terminal's seat by the rules of D6.
  `marley_workbench::agent_events` keeps the app's one `FleetSnapshot`: a seat per terminal whose
  Claude Code has sent an event, keyed by the terminal's id as `terminal_list` gives it, ended when
  Claude Code leaves the foreground or the terminal closes.
- **The rail.** An agent row whose terminal has sent events takes its state from them: `working`,
  `waiting`, `idle` or `failed` (`no update in N m` for a quiet working seat is slice 2, #547).
  The status line counts running subagents, a second line shows the prompt,
  and a third shows the tool in flight with its preview, what the session waits on, the last
  message, or the error. A row with no events keeps today's quiet-timer status.
- **Slice 2, #547:** `fleet_snapshot` fed from the app's snapshot, the update chip, and the
  30-minute `no update` form (D7, D9's chip, the design's items 7 and 8).

### Out (explicitly deferred)
- Notifications that say what happened (report 01, item 4; #538): `notify.sh` keeps its fixed
  sentences here.
- Attention order, summaries and unread marks in the rail (report 01, item 5; #542).
- Inferring an interrupt from Ctrl+C or Escape typed into the terminal (report 01 §2.3). Orca's
  recording of Claude Code 2.1.280 shows an Escape fires no hook, even in the middle of a tool
  (`src/shared/__fixtures__/claude-cancel-shell-hooks.jsonl`), so the row moves on only at the
  next event, and reads `no update` after 30 minutes. It needs a Zed touch in the terminal's input
  path and its own measurement on 2.1.283.
- Codex's, Gemini's and OpenCode's hooks; remote hosts (the plugin installed there, tmux's
  passthrough); an authenticated agent frame (D4).
- Session resume from the reported `session_id` (report 01, item 7; #540).
- The approvals inbox (#508) and per-turn diffs (#509), which read these events.

## Reference (§20)
- **Warp:** its agent notifications give each tab an icon for its agent's state, "working,
  blocked, completed, or errored", and support Claude Code "via notification plugin"
  (docs.warp.dev/agent-platform/capabilities/agent-notifications/). Marley's rail rows show the
  same four states, from its own plugin's hooks. No Warp code and no Warp plugin was read.
- **Upstream Zed:** the Agent Panel's thread status, which the rail already shows for threads
  through ACP (`marley_rail::thread_status`: waiting for confirmation, error, generating).
  Terminal rows now reach the same states.
- **Orca:** hook-driven agent state (report 01 §2.2, §2.3 and item 1; report 05 §2.5 and item 1;
  report 06 §2.14 and item 3), carried in-band here instead of over Orca's loopback server.

### Prior art
- **Reports.** Report 01 gives Orca's Claude event set (`src/main/claude/hook-settings.ts`), its
  mapping to states (`src/shared/agent-hook-listener/providers/claude-events.ts`), the preview
  keys (`src/shared/agent-hook-listener/tool-input-preview.ts`), the prompts a harness injects
  (`src/shared/harness-injected-user-turns.ts`) and the 30-minute decay
  (`src/shared/agent-status-freshness.ts`). Orca's commit `02ba70a847` explains why its hook
  prints `{}` first: Claude Code reads empty stdout as no decision, but compat consumers such as
  cursor-agent, which import `~/.claude/settings.json`, fail a permission gate closed on it. A
  plugin's hooks are not in that file; `event.py` prints `{}` outside Marley all the same.
- **Published material.** Claude Code's hooks reference (code.claude.com/docs/en/hooks): the
  common input fields (`session_id`, `transcript_path`, `cwd`, `permission_mode`, `prompt_id`,
  `agent_id`), `terminalSequence` as a top-level output field, a PermissionRequest hook's exit 0
  with empty stdout meaning no decision, StopFailure's error types, PostCompact's `manual` and
  `auto`, SessionStart's `startup`, `resume`, `clear`, `compact` and `fork`. The installed Claude
  Code 2.1.283, its bundle read and not run: `terminalSequence` allows OSC 0, 1, 2, 9, 99 and 777
  and BEL, refuses a sequence over 4,096 bytes, keeps only printable characters of a payload, and
  wraps the sequence for tmux itself; the payload fields per event are the ones listed above.
- **Code we already ship.** `marley_dcs::NotificationScanner` finds OSC 777 across reads with a
  4 KiB cap (`crates/marley_dcs/src/notification.rs:11`, `:107`), and Zed's terminal already
  emits `Event::MarleyNotification` (#478), so the frame needs no new escape and no new terminal
  event. `marley_fleet` already models the states (Working, Waiting with a `Question`, Idle, Error
  kept apart from Idle, Done), the fold (`apply`) and staleness (`is_stale`); this ticket feeds it
  rather than adding a model. `base64` and `serde_json` are workspace dependencies already.

## UI proof
UI-AFFECTING: the agent's rows in the rail, and the agent bar's chip.
`script/e2e/519-claude-code-events-in-the-rail.sh` (`compositor sway`, for the chip's click).
A fake `claude`, a Python script first on the PATH (the rail names it `claude` through its rule
for interpreters), acts out a session: at each Enter the scenario presses, it runs the plugin's
real `hooks/event.py` with a recorded payload on stdin and writes the answer's
`terminalSequence` to its terminal, as Claude Code does. Shots:
- `519-00-before-events`: output and no event yet; the row reads `Claude Code · waiting` from the
  quiet timer.
- `519-01-working`: UserPromptSubmit ("Add a README to the project"), then PreToolUse
  (Bash, `ls -la`): `working`, the prompt, `Bash: ls -la`.
- `519-02-waiting`: PreToolUse (Write, `README.md`) and PreToolUse (Read, `src/lib.txt`) in
  parallel, PermissionRequest for the Write, then the Read's PostToolUse: still `waiting`, and
  what it asks. The Write's PostToolUse follows and ends the wait.
- `519-03-subagent`: SubagentStart and a subagent's PreToolUse (Grep): `working · 1 subagent`,
  the lead's prompt unchanged and the subagent's tool not on the lead's line.
- `519-04-idle`: Stop with a last message: `idle`, the prompt, the message on one line.
- `519-05-injected`: a `<task-notification>` prompt, then the post-compaction continuation: the
  user's prompt still on the row.
- `519-06-failed`: a new prompt, then StopFailure (`rate_limit`): `failed` and `rate_limit`.
No shot from 519-01 to 519-06 shows a bell dot, and `busctl --user monitor
org.freedesktop.Notifications` during the run records no `Notify` (L-claude-478).
Test also runs a real Claude Code once, in a Python pty with `--plugin-dir` (L-claude-482), to
prove 2.1.283 writes `marley-event` frames for UserPromptSubmit, PreToolUse and Stop; a real
model's replies cannot be held steady for shots.

## Locked-In Decisions
- D1: In-band frames, not a POST to `marley_mcp`. The frame arrives in the terminal it belongs
  to, over SSH too, with no endpoint file, token or terminal id to pass (report 05, open question
  2; report 01 §4 skips Orca's loopback server). The price is a 4,096-byte frame and nothing in
  print mode (L-claude-482).
- D2: An OSC 777 notify under the reserved title `marley-event`, its body the base64 of a JSON
  summary. Claude Code allows OSC 0, 1, 2, 9, 99 and 777 only, and Marley's scanner reads OSC 777
  already.
- D3: The hook is Python, beside the bridge. It must pick fields and bound the frame, which `sh`
  cannot do without `jq`, and the plugin needs `python3` for its bridge already (#491).
- D4: A frame is display data, unauthenticated (PR-claude-474: showing a field needs no check).
  The shell's nonce is unset before the user's files run, and a nonce in the environment would be
  readable by every program in the terminal anyway. An event counts only while Claude Code is the
  terminal's foreground program, which drops a frame that a `cat` of an old log replays. An action
  that turns an event into input checks authenticity first; this ticket has none.
- D5: The state lives in `marley_fleet` (three-prong plan D7). Claude's specifics (prompt, tool,
  preview, message, error, subagent count, session id, transcript path, permission mode) ride as
  the seat's labels. The rail and `fleet_snapshot` read the same snapshot.
- D6: The rules, Orca's (report 01, item 1) with an error kept red:
  UserPromptSubmit, PreToolUse, PostToolUse and PostToolUseFailure make the seat Working; a
  UserPromptSubmit sets the prompt unless a harness injected it (a leading tag or prefix from
  Orca's observed list), and the post-compaction continuation ("This session is being continued
  from a previous conversation") changes nothing. PermissionRequest, and PreToolUse of
  `AskUserQuestion`, make it Waiting with a `Question` naming the tool and its preview. Stop, a
  manual PostCompact, a PostToolUseFailure with `is_interrupt`, and SessionStart from `startup`,
  `resume`, `clear` or `fork` make it Idle (a SessionStart sets no prompt; `compact` and an
  automatic PostCompact change nothing). StopFailure makes it Error with the error type.
  SessionEnd, Claude Code leaving the foreground, or the terminal closing ends it (Done). An event
  with `agent_id` comes from a subagent: SubagentStart and SubagentStop move the count, and the
  lead's state stays, except that a subagent's PermissionRequest makes the seat wait too. A
  permission wait ends when the tool it asked for finishes (its PostToolUse or
  PostToolUseFailure, paired through the PreToolUse before it with the same tool and input, which
  carries the `tool_use_id`) or when the turn ends. Another tool finishing meanwhile does not end
  it: tools run in parallel (report 05 §2.5).
- D7: A working seat that has gone 30 minutes without an event, with Claude Code still in the
  foreground, reads `no update in N m`, never idle or done (Orca's decay). The rule covers working
  seats only: an idle session at its prompt is quiet by right, and a waiting one still waits on
  the user. `marley_fleet::is_stale` flags every seat that is not Done, so the rail applies the
  30 minutes itself.
- D8: No new Zed event. The frame rides `Event::MarleyNotification`, and the one Zed change is
  the view's bell arm leaving the reserved title unmarked.
- D9: The plugin's version moves, and the chip offers the update. Claude Code runs an installed
  plugin from its cache, a copy per version (`~/.claude/plugins/cache/...`, named in
  `installed_plugins.json`), so a new hook reaches an existing install only through an update.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Claude Code in a Marley terminal submits a prompt, the terminal's rail row shall read `working` and show the prompt. | Shot `519-01-working` |
| REQ-002 | WHILE a tool runs, the row shall show the tool and its input preview. | Shot `519-01-working` |
| REQ-003 | WHEN Claude Code asks for a permission, the row shall read `waiting` and show the tool it asks for until that tool finishes or the turn ends, whatever other tool finishes meanwhile. | Shot `519-02-waiting` |
| REQ-004 | WHILE a subagent runs, the row shall count it and keep the lead's state and prompt. | Shot `519-03-subagent` |
| REQ-005 | WHEN a turn ends with Stop, the row shall read `idle` and show the last message on one line. | Shot `519-04-idle` |
| REQ-006 | WHEN a harness-injected prompt or the post-compaction continuation arrives, the row shall keep the user's prompt. | Shot `519-05-injected` |
| REQ-007 | WHEN a turn ends with StopFailure, the row shall read `failed` and show the error type. | Shot `519-06-failed` |
| REQ-008 | WHEN a `marley-event` frame arrives, Marley shall post no desktop notification and mark no bell. | Shots `519-01` to `519-06`; the run log's `busctl` record |
| REQ-009 | WHILE an agent terminal has sent no event, its row shall keep the quiet-timer status. | Shot `519-00-before-events` |
| REQ-012 | WHEN `event.py` runs outside a Marley terminal it shall print `{}`, and inside one its answer shall stay under 4,096 bytes for any payload. | The run log: setup runs the hook without `TERM_PROGRAM=zed` and with a 100 KB payload, and logs each answer's size |
| REQ-014 | WHEN a real Claude Code 2.1.283 runs with the plugin in a Marley-like terminal, its output shall carry `marley-event` frames for UserPromptSubmit, PreToolUse and Stop. | Test's live check (L-claude-482's pty method); its log |
| REQ-015 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, re-verify every seam, ask the brain, and run the live pty
  check against the installed Claude Code before any code, since the ticket stands on
  `terminalSequence` reaching the terminal for these events.
- **P2 Code:** the ledger row first (`crates/terminal_view/src/terminal_view.rs`); the hook and
  `hooks.json`, the version and the chip's update; `marley_agent::claude_events`;
  `marley_workbench::agent_events`, the notifications route and the rail's rows; the snapshot
  handed to the server. fmt and clippy clean; a review of the diff.
- **P3 Test:** write and run the scenario, read every shot, run the live check,
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
