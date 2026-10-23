---
pipeline_id: 8e3b1061-7d90-4326-b75d-9318ea5b2c9a
ticket: docs/planning/tickets/open/TICKET-913-dialogs-wait-and-second-launch.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Wait before closing, a second launch says so, the tmux close prompt
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/G-system.md, docs/marley/design-review-2026-10-23/B-rail-chrome.md]
---

## Title
The close dialog for a working agent's tab offers Wait, which closes the tab once the agent's
turn ends, and its words name the agent, the turn and the undo hold. A folder handed over by a
second launch opens with a notice in the running Marley. The tmux close prompt shows a `tx` tile,
the session's host, its windows and the program it runs, over Cancel · Hide · Close Session.

## Scope
### In
- `close_guard.rs` (Marley), the one-agent tab close:
  - The title "<Agent> is still working"; the detail "“<turn>” is in a turn. Close keeps it
    running for <N> s so Undo can bring it back." The turn is the seat's title; with no title,
    "Its turn"; with `marley.undo_close_seconds` at 0, the second sentence is left out.
  - The buttons Close, Wait and Cancel (drawn Cancel · Wait · Close), when the agent has a seat
    that reports its turns (Claude Code's events, #519; Codex's App Server, #650). Without a seat
    the dialog keeps Close, Show and Cancel.
  - Wait: the tab stays; a toast "<Agent> in <project> closes when its turn ends" with Keep It
    Open, which ends the wait. A `cx.observe_global::<AgentEvents>` watcher closes the tab through
    the pane's own close once the seat leaves Working (Idle, Error, or the seat ends). A Waiting
    seat (a question) keeps the wait. Closing the tab by hand ends the wait.
- `close_guard.rs` and `prompts.rs` (Marley), the tmux close prompt:
  - `prompts.rs` gains a head a caller sets for its next prompt: a glyph tile and a line under the
    title (`PromptHead`, a `Global` keyed by the prompt's message, removed when the prompt is
    answered). The card draws the tile left of the title and the line under it, then the detail
    as the body.
  - `ask_tmux` sets the head: tile `tx` in the board's tmux teal (the theme's
    `terminal_ansi_cyan`, #7dd3c0 in Marley Night, on the same colour faded), the title "Close the tmux session “<name>”?", the line "On this machine" or "On <host>",
    "N window(s)", and "<program> is running" when the session's pane runs something other than
    its shell. The body: "Close Session ends what runs in it. Hide detaches the tab; the session
    keeps running and its row stays in the rail."
  - `tmux_sessions.rs` (Marley): `TmuxSession` keeps the pane's foreground program
    (`pane_current_command`, already in `PANE_FORMAT`) as `command: Option<String>`, none for a
    shell.
- The second-launch notice:
  - `crates/zed/src/zed/open_listener.rs` (Zed crate): `listen_for_cli_connections` hands each
    `file://` URL it reads to `marley_workbench::single_instance::note_handed` before it opens.
  - `crates/zed/src/main.rs` (Zed crate): `handle_open_request`'s open-paths task, after
    `open_paths_with_positions`, calls `marley_workbench::single_instance::announce_handed` with
    the window and the paths.
  - `single_instance.rs` (Marley): `note_handed` keeps the paths in a `Mutex` (the listener is a
    thread of its own); `announce_handed` takes the paths it noted and shows a toast in that
    window's workspace: "Opened <folder name> from another launch. Only one Marley runs on a data
    folder. The second launch handed you its folder and quit." with Dismiss. A path the launch's
    own arguments opened was never noted, so the first start shows nothing.

### Out (explicitly deferred)
- The quit dialog's wording ("Quit with 2 agents working?", which agents resume) and its Show
  button; the Rusty-not-connected dialog; the setting-change proposal as a centred dialog: a
  follow-up ticket for the Dialogs board's remaining looks.
- Wait for a window's close or a quit: they close many terminals at once, and #550 holds only a
  tab's close.
- A countdown on the waiting toast (#550's Out keeps it off the hold's toast too).
- The tmux window that is current ("window 1 of 2"): tmux's listing gives the count, not which
  window the attach shows. The line says "2 windows".
- The command's arguments in the tmux line: `pane_current_command` names the program only.
- A notice for a hand-off with no folder (it only brings a window forward) or for `marley <verb>`.
- Welcome to Marley: TICKET-912.

## Reference (§20)
N/A — Marley-specific: the close guard (#550, AD-claude-550), the tmux attach and its prompt
(#821), and Marley's single-instance hand-off (#513) are Marley's own; upstream Zed keeps its
prompt renderer (`ui_prompt`, #870's hook) and its open listener, which this extends by one call
each.

### Prior art
- `close_guard.rs:103-160` `guard`: one prompt per close, `["Close", "Show", "Cancel"]`, the hold
  after Close; `:166` `tmux_attach`, `:181-226` `ask_tmux` with `["Close Session", "Hide",
  "Cancel"]`; `:263` `working_status` reads the seat (`agent_events::seat_of`) or the quiet timer;
  `:290` `question` builds today's words.
- `prompts.rs:44` `card` (#870) already draws the buttons right to left and gives Close Session
  the danger role, Hide plain and Cancel quiet: in the Marley layout the order is already Cancel ·
  Hide · Close Session. The review judged the order from `close_guard.rs` without a run; this
  ticket's shot confirms it.
- `crates/ui_prompt/src/ui_prompt.rs:31-45`: `MarleyPrompt` carries the message, detail, actions,
  cancel and active index only, so the tile and line come from Marley's side (`PromptHead`), not
  a new Zed field.
- `agent_events.rs:143` `seat_of`; `marley_fleet::Session { title, state, … }`
  (`session.rs:62`); `resume.rs:134` and `mcp.rs:169` observe `AgentEvents` as a global, the
  pattern for the wait's watcher.
- `single_instance.rs:106` `hand_off` and `:152` `handed_line` (what the second launch prints);
  `zed/open_listener.rs:429-458` the listener thread; `main.rs:1033-1047` the request loop and
  `:1327-1355` the open-paths task. #870's notes: "the second launch shows nothing in the running
  Marley".
- `toasts.rs` (#869): a toast's first sentence is its title and the rest its line.
- `tmux_sessions.rs:41` `PANE_FORMAT` reads `pane_current_command`; `:652` `session_words`.
- Ledger: AD-claude-550, L-claude-821-a-prompts-buttons-stand-in-a-column-001 (before #870's
  card), L-claude-768-a-prompts-buttons-move-with-its-detail-001.

## UI proof
Scenario `script/e2e/913-dialogs-wait-and-second-launch.sh` (sway): project `repo`; #550's
stand-in `claude` first on the terminals' PATH through the scenario's `.bashrc`, running the
plugin's real `event.py` so each Return moves its seat (working, then idle); a tmux server of the
run's own under `$XDG_RUNTIME_DIR` (#808) with a session `build` running `sleep 600` in `repo`;
`marley.tmux_hosts` naming this machine only; a second folder `repo-b`.
- `913-01-close`: the stand-in working, its tab closed: "Claude Code is still working", the
  turn and the hold, Cancel · Wait · Close (REQ-001, REQ-002).
- `913-02-waiting`: Wait clicked: the tab still open, the waiting toast (REQ-003).
- `913-03-closed`: Return in the stand-in, its seat idle: the tab gone (REQ-003).
- `913-04-tmux`: `build` attached and its tab closed: the `tx` tile, the title, "On this machine
  · 1 window · sleep is running", the body, Cancel · Hide · Close Session (REQ-004).
- `913-05-handed`: the run's binary started again on the same profile with `repo-b` (#513's
  `second_launch`): `repo-b` open, the notice in the running Marley (REQ-005).
- `913-06-day`: Marley Day, the close dialog again (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — Wait takes Show's place only where it can work: one agent closed from its tab, with a seat
  that reports turns. The quiet timer cannot tell a turn's end from a pause, so without a seat the
  dialog keeps Show.
- D2 — The wait closes through Zed's own close path, so the guard runs again and finds the agent
  idle; nothing bypasses the guard.
- D3 — A seat waiting on the user keeps the wait: the turn has not ended, and the question shows in
  the inbox.
- D4 — The tile and the line under a prompt's title are Marley's (`PromptHead`), keyed by the
  message: `ui_prompt` stays as #870 left it.
- D5 — A hand-off is recognised by where its URL came from (the socket's listener), never by
  timing, so the first start's own paths never show the notice.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a working agent's tab is closed, the dialog shall be titled "<Agent> is still working" and say which turn it is in and how long Close keeps it for Undo, in Marley Night and Marley Day. | `913-01-close`, `913-06-day` |
| REQ-002 | WHERE the agent has a seat that reports its turns, the dialog shall offer Cancel, Wait and Close in that order. | `913-01-close`, `913-06-day` |
| REQ-003 | WHEN Wait is chosen, the tab shall stay open with a waiting toast, and WHEN the seat's turn ends the tab shall close. | `913-02-waiting`, `913-03-closed` |
| REQ-004 | WHEN an attached tmux tab is closed, the prompt shall show a `tx` tile, "Close the tmux session “<name>”?", a line with the host, the windows and the running program, its body, and Cancel · Hide · Close Session. | `913-04-tmux` |
| REQ-005 | WHEN a second launch hands the running Marley a folder, that folder shall open with a notice "Opened <name> from another launch" and Dismiss. | `913-05-handed` |
| REQ-006 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the close dialog and Wait, `PromptHead` and the tmux prompt, the hand-off notice
  and its two Zed hunks with their rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
