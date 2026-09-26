---
pipeline_id: 4ab49a90-0f86-4ebc-9814-ae748130e35a
ticket: docs/planning/tickets/open/TICKET-540-session-resume-after-restart.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Claude Code sessions resumed after a Marley restart"
type: feature
slice: prong 2, sessions (report 05 §3 item 3, report 01 §3 item 7); before the harness's passthrough terminals (C3)
references: [docs/planning/tickets/open/TICKET-519-claude-code-events-in-the-rail.md, docs/orca_architecture/05-terminal-and-workspace.md, docs/orca_architecture/01-agents-and-sessions.md, docs/marley/three-prong-plan.md]
---

## Title
After a Marley relaunch, a terminal that was running a Claude Code session at the quit runs
`claude --resume <id>` in the folder the session started in, with the id #519's `SessionStart`
reported, so the conversation comes back. A session that ended before the quit comes back as a
plain shell, as today.

## Scope
### In
- The session per terminal, from the `SessionStart` and `SessionEnd` frames #519's
  `marley_workbench::agent_events` receives (their `session_id`, `cwd`, `source` and `reason`):
  - `SessionStart` with source `startup`, `clear` or `fork`: the terminal's session becomes the
    event's `session_id`, and its folder the event's `cwd`;
  - `SessionStart` with source `resume`: the id is kept or replaced by the event's (a resume keeps
    its id unless `--fork-session`), and a new id takes the event's `cwd`;
  - `SessionStart` with source `compact`: nothing changes;
  - `SessionEnd` with reason `prompt_input_exit` or `logout`: the session is dropped; with `clear`
    or `resume`, the `SessionStart` that follows replaces it; with `other`, it is kept, since
    that is how the terminal closing at a quit reads;
  - the terminal's foreground process leaving Claude Code while Marley runs also drops it.
- Saved on the terminal's own row: two columns in `TerminalDb`'s `terminals` table
  (`marley_agent_session`, `marley_agent_cwd`), a migration appended to `MIGRATIONS` with a save
  and a get (`crates/terminal_view/src/persistence.rs`), written by `TerminalView::serialize` beside
  the working directory and read by `deserialize` (`crates/terminal_view/src/terminal_view.rs`).
  `TerminalView` gains the field and a setter shaped like `set_custom_title` (it marks
  `needs_serialize` and emits `ItemEvent::UpdateTab`). A Zed crate change in two files, with its
  touchpoint rows.
- At relaunch: a restored terminal with a saved session starts its shell in the session's folder
  (`deserialize` prefers it to the saved working directory), and the workbench types
  `claude --resume <id>` and Enter after the shell's startup handshake, as `start_cli` types
  `claude` (`crates/marley_workbench/src/agents.rs:188-222`, `STARTUP_TIMEOUT` 5 s).
- The id is typed only if it is a UUID (hex digits and dashes in the 8-4-4-4-12 shape), so a
  damaged row can type nothing else into a shell.
- One session resumes in one terminal per launch: the first restored terminal holding the id in
  the window's order; another terminal holding the same id comes back as a plain shell and drops it.
- The restored view keeps the session on its new row: Zed re-saves each restored item under its
  new id and deletes the rest (L-claude-494), so the session survives any number of relaunches.
- A setting, `marley.resume_agents` (default on), in `MarleySettingsContent`, and on the Marley
  page once #515 has added it.

### Out (explicitly deferred)
- Codex (`codex resume <id>`) and the other agents: #519 carries Claude Code's hooks only.
- The scrollback: Zed restores an empty terminal, and the resumed Claude Code redraws its own
  conversation. Restoring rows with their colours needs a grid serializer alacritty lacks (report
  05 §2.2).
- A history of past sessions per project with Resume (report 01 §3 item 7's history half).
- Remote terminals: Zed saves no working directory for them (`Terminal::working_directory` returns
  `None` for a remote terminal, `crates/terminal/src/terminal.rs:3059-3068`), and their sessions
  live on the remote host.
- Terminals held by the embedded rustal-harness (D10, prong 2's C3): there the process survives and
  Marley reattaches, and must not resume (the notes say what the harness changes).
- The flags a launch carried (the permission setting, once it exists): the resume types through the
  same launch input as `start_cli`, so a later launch recipe applies to both.

## Reference (§20)
Upstream Zed (`terminal_view`): `TerminalView::deserialize` brings a terminal back as a new shell at
its saved working directory with its custom title, and `serialize` saves both on the terminal's own
row. Marley keeps that and adds the agent's session to the row. Warp's Session Restoration restores
"your windows, tabs, panes, and recent Blocks automatically when you relaunch Warp" and says nothing
of the processes in them (docs.warp.dev/terminal/sessions/session-restoration). The behavior Marley
matches is Orca's cold restore (report 05 §2.2): a pane whose agent's session id its hooks reported
is relaunched with `claude --resume <id>` (`getAgentResumeArgv`, `src/shared/agent-session-resume.ts`),
and Claude's session folder is its first record's, because `claude --resume` finds sessions by the
directory they started in (report 01 §2.6, Orca #9361).

### Prior art
- **Behavior maps.** Report 05 §2.2 (Orca's daemon, cold restore, resume ids saved every 60 seconds
  and at quit), §2.8 (session restore), §3 item 3 (the landing: the session id in the terminal's row
  of Zed's database, or a Marley table keyed by item id; never resume one session twice); report 01
  §2.6 (a live pane's session id comes only from its hooks) and §3 item 7.
- **Published material.** Claude Code's CLI reference (code.claude.com/docs/en/cli-reference, read
  2026-09-25): `--resume` resumes a session by id or name and, like `--continue`, reuses the original
  session id; `--fork-session` makes a new one. Its hooks reference: `SessionStart`'s sources are
  `startup`, `resume`, `clear`, `compact` and `fork`; `SessionEnd`'s reasons are `clear`, `resume`,
  `logout`, `prompt_input_exit` and `other`; every hook receives `session_id` and `cwd`.
- **Code we already ship.** `crates/terminal_view/src/persistence.rs` (`TerminalDb`, its migrations
  at lines 415-454, `save_working_directory` 471, `save_custom_title` 511, `get_custom_title` 538);
  `crates/terminal_view/src/terminal_view.rs` (`serialize` 1933-1964, which skips task terminals and
  clean views; `deserialize` 1970-2027; `set_custom_title` 448-456 and `mark_needs_serialize`
  458-461, the pattern for the setter); `crates/terminal/src/terminal.rs:3059`
  (`working_directory` is the foreground process's folder, so while Claude Code runs the saved
  folder is its own); `crates/marley_workbench/src/agents.rs` (`start_cli` and Zed's
  `start_init_command_startup_handshake` and `write_init_command_after_startup`, which type into a
  shell only once it is ready); `marley_agent::launch_input` (`crates/marley_agent/src/marley_agent.rs:90`).
  Zed's own terminal agent threads (`crates/agent_ui/src/terminal_thread_metadata_store.rs`) keep a
  terminal thread's title, folders and working directory, and no session id or resume: nothing to
  take for this.

## UI proof
UI-AFFECTING (what a relaunched terminal shows).
`script/e2e/540-session-resume-after-restart.sh` (keys only, Hyprland's hidden workspace). A fake
`claude` first on the PATH (started as #481's scenario starts one) prints #519's `SessionStart`
frame with a fixed session id and its folder, then a line `session <id> in <cwd>`; started with
`--resume <id>` it prints `resumed <id> in <cwd>` and its own `SessionStart` (source `resume`); it
logs its argv and folder to a file, and on a trigger file prints `SessionEnd`
(`prompt_input_exit`) and exits. Steps: in terminal A, `cd sub` and start Claude Code from the
palette (`marley: new agent`), shot `540-01-running`; in terminal B, start and exit the fake;
`quit_marley`, `launch_marley`; shot `540-02-resumed` (A shows `resumed <id> in <repo>/sub`), shot
`540-03-plain-shell` (B shows a prompt); `quit_marley`, `launch_marley` again, shot
`540-04-second-relaunch` (A resumed again). Negative parts: B given A's id before a quit (one
resume only), a damaged id written into the profile copy's database between a quit and a launch
(`540-05-damaged`, a plain prompt), and the setting off.

## Locked-In Decisions
- D1 — The session rides the terminal's own row in Zed's database, not a Marley table. Item ids
  change at each launch and Zed re-saves and cleans up each item's own row (L-claude-494); a
  separate table would need a save and a cleanup hook Marley does not get. The price is a small
  change to two Zed files.
- D2 — The id comes only from #519's `SessionStart`, never from reading `~/.claude/projects` or the
  screen. A terminal whose Claude Code runs without Marley's plugin has no id and is not resumed.
- D3 — The resumed shell starts in the session's first folder, since `claude --resume` looks the
  session up by the folder it started in. The command typed is the program, the flag and a
  checked UUID, with no path to quote.
- D4 — `SessionEnd` with `other` keeps the session: a Marley quit closes the terminal under Claude
  Code, and that is the case to resume. An exit Chad makes (`prompt_input_exit`, `logout`) drops it.
- D5 — One session, one resume per launch: two Claude Code processes on one session file would
  interleave its transcript.
- D6 — Resume is on by default, with `marley.resume_agents` to turn it off, since it types into a
  shell on the user's behalf.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley relaunches and a restored terminal was running a Claude Code session at the quit, the system shall type `claude --resume <id>` into it once its shell is ready. | The fake's argv log; shot `540-02-resumed` |
| REQ-002 | WHEN such a terminal is restored, its shell shall start in the session's first folder. | The fake's folder log (`<repo>/sub`); shot `540-02-resumed` |
| REQ-003 | WHEN a terminal's Claude Code session ended before the quit (`prompt_input_exit` or `logout`), the system shall restore a plain shell and type nothing. | Shot `540-03-plain-shell`; no second entry in the argv log for B |
| REQ-004 | WHEN Marley relaunches again, a resumed session shall be resumed again. | Shot `540-04-second-relaunch`; the argv log |
| REQ-005 | WHEN two restored terminals hold the same session id, the system shall resume it in one of them only. | The argv log: one `--resume` for the id, in a part where B was given A's id |
| REQ-006 | WHEN a saved id is not a UUID, the system shall type nothing into the terminal. | The part with a damaged id: no argv entry, a plain prompt in the shot `540-05-damaged` |
| REQ-007 | WHERE `marley.resume_agents` is false, the system shall resume nothing. | The part with the setting off: no argv entry |

## Phase Plan
- **P1 Plan** — this spec, and the design and the test plan in the notes; `brain_ask` at
  promotion, and Chad's go-ahead, since the ticket is Deliberate.
- **P2 Code** — the touchpoint rows first; the columns, the migration and the field in
  `terminal_view`; the session bookkeeping and the resume in `marley_workbench`; the setting; fmt
  and clippy clean; a review of the diff.
- **P3 Test** — write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the touchpoint rows
  checked against what shipped, ledger capture, close the ticket, archive, commit.
