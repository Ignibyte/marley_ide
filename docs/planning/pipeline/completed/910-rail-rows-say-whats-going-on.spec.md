---
pipeline_id: 28519898-cf10-4a00-87dd-ee6ee86fad71
ticket: docs/planning/tickets/open/TICKET-910-rail-rows-say-whats-going-on.md
status: Phase 4 — Complete PASS
title: Rail rows say what is going on
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/B-rail-chrome.md, docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md]
---

## Title
A rail row's second line says what is going on, in its state's color: a thread row what its agent
does or asks, a shell row its failed command with Ask an agent or its folder, and a group header
its summary as pills worded "needs you", with the Default group's home tile and "quiet", as the
Rail and RailStates boards draw them.

## Scope
### In
- **Thread row line** (`marley_workbench/src/rail.rs`, Marley crate): `thread_subtitle` gives way
  to a line built from #907's `thread_activity` (`ThreadActivity`, in
  `crates/marley_workbench/src/thread_activity.rs` at the crate root). `live_statuses` returns each
  live conversation's activity beside its status; `render_thread_row` draws the activity's words
  in `RowState::line` of the same state (`RowState` lives in #905's `item_kind.rs`). Working: what the agent does ("Editing notes.md").
  Needs you: what it asks ("Allow <tool>?"). Done and unseen: what the turn left ("1 file
  changed", or "Finished"). Idle: "<model> · idle", or "<agent> · idle" when the model is not
  known (a thread with no live conversation).
- **Shell row lines** (`rail.rs`, Marley crate): `terminal_snapshot` gives a shell its folder in
  its project's root too (as `~/…`, from `marley_rail::working_directory_label`'s home branch), so
  an idle shell's line is its folder. `render_terminal_row` puts a shell's command line first and
  its folder second. `command_line` says a finished command as "done in 45 s".
- **Ask an agent on a failed shell row** (`rail.rs`, Marley crate): the failed command's line
  ends in an Ask an agent link when the newest live block failed, the shell waits at its prompt
  and no agent runs in that terminal: the same test as `send_block::chip`. The link calls
  `send_block::send(view, index, Request::Fix, …)`, the block header's Ask an agent (#826), as
  #908's failed chip in the bottom row does; it stops the click reaching the row.
  `CommandSnapshot` (`marley_rail`, Marley crate) carries the block's index.
- **Header summary as pills** (`marley_rail/src/marley_rail.rs` and `rail.rs`, Marley crates):
  #906's `GroupSummary { words, needs_you }` gains the counts by state the pills draw
  (`counts: SummaryCounts { needs_you, working, done }`); `words` stays as the pills' tooltip,
  worded "needs you" in place of "waiting". `project_name` draws a pink "N needs you" pill, an
  amber pill with the working count (tooltip "N working") and an accent "N done" pill, at the
  header's right. #906 already shows the summary on an open header; this draws it as pills
  there too.
- **"quiet", the row count and the Default tile** (`rail.rs` and `marley_rail.rs`, Marley
  crates): a header with nothing to count says, muted, "N rows" while open (the rows listed
  under it, #906 left it out) and "quiet" while folded. `header_icon` draws a projectless group's
  icon on a 20 px raised, rounded tile; the Default group's glyph is `IconName::MarleyHome`, other
  groups keep `ListTree`.

### Out (explicitly deferred)
- A running shell row's determinate progress ("1781 of 1813") and a ticking elapsed time. Nothing
  parses a command's own progress (A-core.md, TerminalSession, "Determinate build progress", M)
  and nothing in the rail ticks; both go to a follow-up ticket with that gap. The running line
  stays "<command> · running".
- Browser row states and tmux row states: deferred by #827 (Out, and its scope for `tmux_row`).
- The board's letter glyphs on the tiles (`CC`, `>_`, `tx`): deferred by #827 Out ("the icons
  stay").
- An agent terminal row's lines: they already come from the seat's own events (#519, #650).
- The filter field's look, the strip's flyout and its search popover (RailStates): a follow-up
  ticket for the strip.
- The Threads view rows' agent tiles and times (RailMenus): a follow-up ticket.
- The rail's menus and the strip's `+`: #911.

## Reference (§20)
- Warp, for the shell row's second line: Warp's vertical tabs give a shell row a muted second line
  holding `~`, a path or a branch (`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`,
  "Rows"). Marley keeps the folder there when the shell is idle, and puts the command's state
  above it.
- The rest is N/A — Marley-specific: the thread row's line, the header pills and the Default tile
  are Marley's own rail (#468, #542, #600, #827), drawn as the Rail board shows them.

### Prior art
- `thread_subtitle` (`rail.rs:11286`) formats `<agent> · <status.label()>`; `ThreadStatus::label`
  (`marley_rail.rs:425-436`) maps Done to "idle", so a done, unseen thread reads "idle" in the
  done color (`thread_row_state`, `rail.rs:11276`; `RowState::line`, `rail.rs:11020-11030`,
  moved with `RowKind` and the kind tints to `crates/marley_workbench/src/item_kind.rs` by #905).
  `render_thread_row` (`rail.rs:6445-6470`) sets the line's color from the state, not its words.
- `live_statuses` (`rail.rs:9367-9399`) already reads each live `ConversationView` and its root
  thread; the activity can be read in the same loop. A thread with no live conversation has only
  its metadata (`group_threads`, `rail.rs:9412`).
- #907's `thread_activity` / `ThreadActivity` (what a thread's agent does or asks, and since
  when), in `crates/marley_workbench/src/thread_activity.rs` at the crate root, is the one reader
  of a thread's activity; this ticket draws its words in the rail.
- `command_line` (`rail.rs:9694-9717`) builds `<cmd> · running`, `· exit N · <took>` in red, or
  `· done · <took>`; `command_snapshot` (`rail.rs:9655`) reads the newest live block.
  `RowLine::state` keeps the state whole beside a cut command (F-claude-551).
- A shell's second line is `working_directory_label` (`terminal_snapshot`, `rail.rs:9552-9562`),
  which returns `None` in the project root (`marley_rail.rs:1998-2019`): the blank idle line.
- `render_terminal_row` (`rail.rs:6609-6680`) chains subtitle, activity, quota, command line and
  running error, colors the first muted line by state, and pads to two lines (#796). `CardLine::
  Element` (#750) holds an element as a line.
- `send_block::chip` (`send_block.rs:192-240`) offers "Ask an agent" on the newest failed live
  block at the prompt with no agent in front, and calls `send(…, Request::Fix, …)`, which starts
  Claude Code when no agent runs (#826). #899's `ask_to_fix` (`send_selection.rs:225`) asks only
  about an editor's line and cannot send a terminal block (#908 found the same), so the block's
  `send` is the one to reuse, as #908's failed chip does.
- `summary` (`marley_rail.rs:1342-1380`) counts agents into five words ("waiting", "failed",
  "done", "working", "not reporting") joined as a string; it is set only while folded
  (`marley_rail.rs:1809`). #906 turns it into `GroupSummary { words, needs_you }` and sets it on
  an open header too; this ticket adds the counts. `project_name` (`rail.rs:10764-10788`) draws it as one XSmall label,
  conflict-colored on attention.
- `header_icon` (`rail.rs:674-697`) draws `IconName::ListTree` for every projectless group;
  `groups::DEFAULT` is "Default" (`groups.rs:33`); `IconName::MarleyHome` is Home's icon
  (`status_buttons.rs:240`).
- The boards: `Rail.dc.html` (the states × kinds grid; the three group headers, "5 rows", pink
  "1 needs you" and amber "2" pills, the Default home tile and "quiet") and `RailStates.dc.html`.
- Published material: none applies; this is Marley's own rail.

## UI proof
Scenario `script/e2e/910-rail-rows-say-whats-going-on.sh` (sway). Setup: projects `a` and `b`
in one window. Marley's own agent on #825's scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`),
scripted to hold a tool call in flight, then ask a permission, then write `notes.md` and finish.
In `a`, a shell runs `false`; another shell sits idle in `a`'s root. In `b`, #542's stand-in
Claude Code waits on the user and a second one works; the stand-ins sit first on the terminals'
PATH through the scenario's `.bashrc` (F-claude-903), so Ask an agent reaches a stand-in.
- `910-01-working`: the thread row reads the tool call in amber (REQ-001).
- `910-02-asks`: the thread row reads its question in pink (REQ-002).
- `910-03-done`: the thread finished, unseen: "1 file changed" in the done color with the green
  dot (REQ-003).
- `910-04-idle`: the thread seen: "<model> · idle", muted (REQ-004).
- `910-05-shells`: the failed shell reads "false · exit 1 · Ask an agent" in red; the idle shell
  reads its folder (REQ-005, REQ-006).
- `910-06-asked`: Ask an agent clicked: the stand-in's prompt holds "This command failed. Fix it:"
  and the block; the row did not change the selection (REQ-007).
- `910-07-headers`: `b` folded with a pink "1 needs you" pill and an amber "1"; `a` open with its
  pills; a quiet project open, reading "N rows"; the Default group with its home tile, folded,
  reading "quiet" (REQ-008, REQ-009, REQ-010).
- `910-08-day`: the rail in Marley Day, a thread row, the shells and the headers (REQ-001 to
  REQ-010).

## Locked-In Decisions
- D1 — A row's line and its color come from one state: the done color carries done words only.
- D2 — The thread row reads #907's `thread_activity`; if it lacks a part the row needs (the done
  turn's file count, the idle model), that part is added to `thread_activity`, not read twice.
- D3 — A shell row keeps #796's two lines: the command's state first, its folder second; a shell
  that ran nothing yet shows only its folder.
- D4 — The shell row's Ask an agent is the block's own button: the same condition as
  `send_block::chip` and the same `send(…, Request::Fix, …)`.
- D5 — The pills build on #906's `GroupSummary`, extended with counts; they count by the row
  states the tiles use: failed with attention counts as
  needs you, not reporting counts as working (`RowState::from_attention`).
- D6 — An empty summary reads "N rows" on an open header and "quiet" on a folded one, as the
  Rail board's first and third headers do.
- D7 — Every projectless group's icon sits on a tile; only the Default group's is the home glyph.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a thread's agent works, its rail row's second line shall say what it does, in the working color. | `910-01-working`, `910-08-day` |
| REQ-002 | WHILE a thread waits on the user, its rail row's second line shall say what it asks, in the needs-you color. | `910-02-asks`, `910-08-day` |
| REQ-003 | WHEN a thread finishes while unseen, its rail row's second line shall say what the turn left, in the done color. | `910-03-done` |
| REQ-004 | WHILE a thread is idle and seen, its rail row's second line shall read its model (or agent) and "idle", muted. | `910-04-idle` |
| REQ-005 | WHEN a shell's newest command fails, its rail row shall show the command, its exit and an Ask an agent link, in the error color. | `910-05-shells`, `910-08-day` |
| REQ-006 | WHILE a shell runs nothing, its rail row shall show its folder, its project's root included. | `910-05-shells` |
| REQ-007 | WHEN Ask an agent on a shell row is clicked, the system shall send the failed block to an agent with the fix request, as the block's own button does, and leave the rail's selection as it was. | `910-06-asked` |
| REQ-008 | A group header with rows that need the user, work or finished unseen shall show them as pills: "N needs you" in pink, the working count in amber, "N done" in the accent. | `910-07-headers`, `910-08-day` |
| REQ-009 | WHILE a group has nothing to count, its header shall say how many rows it lists when open, and "quiet" when folded. | `910-07-headers` |
| REQ-010 | The Default group's header shall show its home glyph on a tile. | `910-07-headers`, `910-08-day` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the thread line from `thread_activity`, the shell lines and Ask an agent link,
  `GroupSummary`'s counts and their pills, the row count, "quiet" and the Default tile; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
