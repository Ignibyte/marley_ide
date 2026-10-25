---
pipeline_id: a5963ec7-a056-4cf8-b9f5-59c8badc3b9e
ticket: docs/planning/tickets/open/TICKET-922-bottom-drawer-as-the-board.md
status: Phase 4 — Complete PASS
title: The bottom drawer as the board draws it
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/D-panels-editor.md, docs/planning/pipeline/completed/838-the-bottom-drawer.spec.md]
---

## Title
The bottom drawer as the BottomPanel board draws it: Problems as a filterable list of rows, each
with a severity dot, the message, the place and Ask an agent to fix; a count pill on each tab; the
drawer's own Move to the side, Maximize and Close; the drawer as a rounded card inset from the
window's edges.

## Scope
### In
- **Problems as a list** (Marley crate). `ProblemsPanel` moves out of `drawer.rs` into a new
  `problems_panel.rs` and stops hosting Zed's `ProjectDiagnosticsEditor`. It draws:
  - a "Filter problems" field with a magnifier (a single-line `Editor`), which narrows the rows by
    message or file name, ignoring case;
  - one row per primary diagnostic of the project: a dot in the severity's colour (error, warning),
    the message's first line, the place (`main.rs:13`) and an "Ask an agent to fix" pill;
  - a click on a row opens the file with the caret at the place.
  The rows come from `Project::diagnostic_summaries`, then each path's buffer
  (`Project::open_buffer`) and its `diagnostics_in_range`, the calls Zed's diagnostics editor
  makes, refreshed on `project::Event::DiagnosticsUpdated`.
- **Ask an agent to fix** (Marley crate, `send_selection.rs`). `ask_to_fix` (#899) splits so its
  request ("Fix this: <place> (<message>)", typed with no Enter, to the one agent or through the
  picker) can be asked for a given path, line and message. A row's pill calls it.
- **Count pills on the tabs** (Marley crate, `drawer.rs` `tabs`). Problems carries its count in a
  pill, red while there are errors, muted otherwise. Debug carries "paused" while a session has a
  stopped thread, "running" while one runs, and no pill without a session.
- **The drawer's own buttons** (Marley crate, `drawer.rs` `tabs`). At the strip's right:
  - Move to the side, which sets the drawer's dock to the right; on the side it reads Move to the
    bottom and sets it back;
  - Maximize, which zooms the shown panel; while zoomed it reads Restore;
  - Close, which closes the drawer's dock.
- **The drawer's place follows `debugger.dock`** (Marley crate, `problems_panel.rs`). Problems
  reads its position from `debugger.dock` and writes it there, so Debug and Problems move as one,
  and the strip draws on whichever dock shows one of them.
- **The card shape** (Zed crate hunk, `crates/workspace/src/dock.rs`). A new global,
  `MarleyDockShape`, which `Dock::render` asks: while it answers `Card`, the dock's box is inset
  (10 px at the sides, 8 px below), rounded (14 px) and bordered all round. Marley answers `Card`
  for the dock that shows the drawer, in the Marley layout. The strip draws a grab handle (a short
  rounded bar) on the drawer's top edge while it is at the bottom.

### Out (explicitly deferred)
- The Output and Tasks tabs and their pills ("rust-analyzer", "0"): #838 Out, its decision 4. A
  follow-up ticket: "Output and Tasks in the bottom drawer".
- The debugger's panes as the board lays them out (controls over the call stack, Variables with
  types, Breakpoints with checkboxes and descriptions), and "Ask an agent about this stop": #838
  Out. A follow-up ticket: "The Debug tab as the board draws it". The debugger's own control strip
  (with its own Close) stays as Zed draws it until then.
- The "paused · name = …" chip on the stopped line: an editor change, not the drawer's. A
  follow-up ticket with the EditorPopups or Main boards.
- Hints and information as rows (the board's green dot): Problems lists what Zed's diagnostics
  list (errors, and warnings while `diagnostics.include_warnings` is on), so the rows match the
  tab's count.
- Zed's diagnostics excerpts inside the drawer: the list replaces them in the Marley layout. Zed's
  layout keeps its Project Diagnostics tab.

## Reference (§20)
Upstream Zed: `workspace::dock` (one panel shown per dock; a panel's position, zoom through
`workspace::ToggleZoom` and `PanelEvent::ZoomIn`/`ZoomOut`, and the dock's open state are kept as
Zed handles them), `diagnostics` (which diagnostics count as problems, and how a jump puts the
caret at a diagnostic: `DiagnosticBlock::jump_to`), and `debugger_ui` (`debugger.dock` as the
Debug panel's place, its `set_position`). The look is the design canvas's BottomPanel board.

### Prior art
- `drawer.rs` (#838): `MarleyDockHeader`'s hook `tabs` draws the strip (`drawer.rs:59-100`),
  "Problems N" as button text (`:89-93`), only for the bottom dock (`:65`); `ProblemsPanel`
  hosts `ProjectDiagnosticsEditor` (`:126-150`) and is valid only at the bottom (`:196-198`).
  `count` already reads `Project::diagnostic_summary` (`:153-156`).
- `crates/workspace/src/dock.rs`: `MarleyDockHeader` asked in `Dock::render` (`:1283-1300`,
  `:1380`); the dock's box draws `border_t_1` flush at the bottom (`:1359-1376`); `Dock::set_open`
  (`:586`), `set_panel_zoomed` (`:604`), the subscription that moves a panel when its `position`
  changes after a settings change (`:646-712`) and that turns `PanelEvent::ZoomIn`/`ZoomOut`/
  `Close` into the workspace's zoom and the dock's open state (`:738-788`); `Panel::is_zoomed`
  and `set_zoomed` default to nothing (`:76-82`).
- `crates/debugger_ui/src/debugger_panel.rs`: position from `DebuggerSettings::dock` (`:1522`),
  valid anywhere (`:1526`), `set_position` writes `debugger.dock` and turns its panes' axes
  (`:1530-1548`); `toggle_zoom` emits `ZoomIn`/`ZoomOut` (`:1276-1290`); its control strip has a
  close button dispatching `ToggleBottomDock` (`:670-679`).
- `crates/diagnostics/src/diagnostics.rs`: `update_stale_excerpts` opens each path's buffer
  (`:351-394`), `update_excerpts` reads `diagnostics_in_range` and the severity bound from
  `include_warnings` (`:509-560`); `diagnostic_renderer.rs` `jump_to` (`:323-337`).
- `send_selection.rs`: `ask_to_fix` (`:225-284`) and `line_and_problem` (`:288`), #899.
- `debug_row.rs` (#901): a session with a stopped thread, from the `DapStore` (`:76-105`).
- `diagnostics/src/items.rs`: `MarleyProblemsCard` (#863), the bottom row's count opening a card
  with Open in the drawer; unchanged here.
- Published material and the behaviour maps: nothing on a drawer of docks; `docs/zed_architecture/`
  describes the dock as above. No crate we build already draws diagnostics as a list of rows.
- AD-claude-838 rejected "a Problems list of Marley's own". The board asks for the list, so this
  ticket supersedes that half of the decision; Phase 4 records the new AD.

## UI proof
Scenario `script/e2e/922-bottom-drawer-as-the-board.sh` (sway). Setup: L-claude-838's Cargo crate
with syntax errors in two files (rust-analyzer, its target in `$E2E_WORK/target`), #838's C
program with `.zed/debug.json` for GDB, and #899's stand-in `claude` first on the terminals' PATH
through the scenario's `.bashrc`, started in a terminal of the window so it is the window's one
agent.
- `922-01-problems`: the drawer on Problems: the inset card with its grab handle, the tabs with
  their pills (Problems red), the three buttons, the filter field and a row per problem (REQ-001,
  REQ-005, REQ-009).
- `922-02-filtered`: a word typed in the filter: only the matching rows (REQ-002).
- `922-03-opened`: a row clicked: its file open with the caret on its line (REQ-003).
- `922-04-asked`: a row's Ask an agent to fix clicked: the stand-in's prompt holds "Fix this:",
  the place and the message (REQ-004).
- `922-05-paused`: `debugger: start`, the GDB config: the Debug tab lit with its "paused" pill
  (REQ-005).
- `922-06-maximized`: Maximize clicked: the drawer fills the window and the button reads Restore
  (REQ-006).
- `922-07-side`: Restore, then Move to the side: the drawer in the right dock with its strip; the
  button reads Move to the bottom (REQ-008).
- `922-08-bottom`: Move to the bottom: the drawer back at the bottom (REQ-008).
- `922-09-closed`: Close clicked: the drawer gone, the editor full height (REQ-007).
- `922-10-day`: Marley Day, Problems again (REQ-001, REQ-005, REQ-009).

## Locked-In Decisions
- D1 — Problems is a list of Marley's, read through the project's diagnostic summaries and buffer
  snapshots as Zed's diagnostics editor reads them. It supersedes AD-claude-838's "Zed's own
  diagnostics editor" for the Marley layout; Zed's layout keeps Zed's diagnostics tab.
- D2 — The rows show what Zed's diagnostics show (errors, and warnings while
  `diagnostics.include_warnings` is on), so the list and the Problems pill agree.
- D3 — The drawer's place is `debugger.dock`. Problems follows it, so one setting moves the drawer;
  Move to the side writes "right", Move to the bottom "bottom". The default stays the bottom
  (AD-claude-774).
- D4 — Maximize focuses the shown panel and dispatches `workspace::ToggleZoom`, so the zoom is
  Zed's. Problems implements `is_zoomed`, `set_zoomed` and a `ToggleZoom` handler as the Debug
  panel does.
- D5 — Close sets the drawer's dock closed (`Dock::set_open(false)`), reading the dock's state, not
  the focus (F-claude-898).
- D6 — The card shape is a Zed hunk, a global the dock asks, answering an enum (`Flush`, `Card`),
  not a bool. Without the global, or in Zed's layout, the dock is Zed's.
- D7 — The pill's wording is "Ask an agent to fix", #899's words for the same request; the board's
  "Fix with an agent" is placeholder text.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the drawer shows Problems in the Marley layout, it shall list one row per problem with a severity dot, the message's first line, the place and an Ask an agent to fix pill, under a Filter problems field, in Marley Night and Marley Day. | `922-01-problems`, `922-10-day` |
| REQ-002 | WHEN the user types in the Filter problems field, the list shall show only the rows whose message or file name holds the typed text. | `922-02-filtered` |
| REQ-003 | WHEN the user clicks a problem's row, Marley shall open its file with the caret at the problem's line. | `922-03-opened` |
| REQ-004 | WHEN the user clicks a row's Ask an agent to fix, Marley shall type "Fix this:", the place and the message at the window's terminal agent, with no Enter. | `922-04-asked` |
| REQ-005 | WHILE the drawer shows, its Problems tab shall carry the problem count in a pill, red while there are errors, and its Debug tab shall carry "paused" while a debug session's thread is stopped. | `922-01-problems`, `922-05-paused`, `922-10-day` |
| REQ-006 | WHEN the user clicks Maximize, the shown panel shall fill the window and the button shall read Restore; a click on Restore shall put it back. | `922-06-maximized`, `922-07-side` |
| REQ-007 | WHEN the user clicks Close, the drawer shall close. | `922-09-closed` |
| REQ-008 | WHEN the user clicks Move to the side, the drawer shall move to the right dock with its strip, and Move to the bottom shall move it back. | `922-07-side`, `922-08-bottom` |
| REQ-009 | WHILE the drawer shows at the bottom in the Marley layout, it shall be a rounded card inset from the window's edges, with a grab handle on its top edge, in Marley Night and Marley Day. | `922-01-problems`, `922-10-day` |
| REQ-010 | WHILE Zed's layout is on, the bottom dock shall keep Zed's flush shape and the diagnostics shall keep Zed's tab. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the `dock.rs` hunk with its ledger row first; `problems_panel.rs`, `drawer.rs`,
  `send_selection.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19, the AD that
  supersedes half of AD-claude-838), close the ticket, archive, commit.
