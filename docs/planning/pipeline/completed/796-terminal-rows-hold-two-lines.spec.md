---
pipeline_id: 0e946e6f-6501-4848-adca-bbdf32c5ebd7
ticket: docs/planning/tickets/closed/TICKET-796-terminal-rows-hold-two-lines.md
status: Phase 4 — Complete PASS
title: A terminal's rail row holds two lines
type: bug
slice: Cross-cutting (the rail)
references:
  - docs/planning/pipeline/completed/551-command-end-from-outside.spec.md
  - docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md
  - docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md
---

## Title
A terminal's row in the rail is drawn at the two-line height whatever it shows under its title,
so the line a command adds (#551) or an agent's activity adds (#519) no longer grows the row and
pushes the rows under it down.

## Scope
### In
- Terminal rows, plain and agent: fewer than two lines under the title are made up with blank
  lines at the bottom, so the row takes `row_card`'s two-line height (3.5 rem) and its title stays
  at the top.
- The tests in `rail_tests.rs` that pin a terminal row at 2.75 rem, or a thread row at a
  terminal row's height, are brought to the new heights. No gate runs them (§7); they keep
  building and stop asserting what is no longer true.

### Out (explicitly deferred)
- A third line still grows the row: a running command's error (#572), an agent's quota line
  (#715) next to its activity, the Interrupted chip (#750) with two lines of text. Chad asked for
  two lines.
- Thread, Browser, port, tab, worktree, Files and inbox rows, the Threads view's list and the
  Containers panel keep their heights: their line count does not change while they are shown.

## Reference (§20)
Warp's vertical tab list, which the rail follows since #468
(`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`): each row is about 46px high
with a title over a muted second line, at one height down the list. Marley keeps the rows at one
height while a terminal's lines come and go, at its own two-line size.

### Prior art
- Behavior maps: `docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md` (one row
  height, title over a second line); `docs/zed_architecture/` has no rail row of this shape.
- Published material: none bears on a list row's height.
- The code we ship: `row_card` already has the two-line height (3.5 rem, #519) and its
  `CardLine::Text` with `RowLine::muted`; the fix reuses both and adds no new size. Zed's
  `ListItem` and `ThreadItem` were rejected for these rows in #468
  (AD-claude-468-the-rail-draws-its-own-rows-after-warps-tab-list-001).

## UI proof
Scenario `script/e2e/796-terminal-rows-hold-two-lines.sh`, under `compositor sway`, with
`marley.rail_order` at `window` so attention does not reorder the rows. A scratch repository with
a `sub` folder; three terminals in it: a shell that moves to `sub` and runs a command, a stand-in
Claude Code acting out hook events with the plugin's real `event.py` (as #519's does), and a shell
at the root with nothing run, under the other two.
- `796-01-before`: the shell in `sub` shows its folder line over a blank line; the agent's row
  shows its status over a blank line; both rows are as tall as the idle shell's (REQ-001).
- `796-02-command`: the shell runs `sleep 30`: its row shows `sub` over `sleep 30 · running`; its
  title and the rows under it sit where `796-01-before` has them (REQ-002).
- `796-03-activity`: the stand-in starts a tool: the agent's row shows its status over the tool;
  its title and the row under it sit where `796-02-command` has them (REQ-003).
- `796-04-root`: the shell ends its command and goes back to the root: its row loses the folder
  line and keeps its height, the command's line now first under its title (REQ-004).

## Locked-In Decisions
- D1 — The floor is two lines for terminal rows, made by blank lines after the real ones in
  `render_terminal_row`; `row_card` and its other callers are unchanged.
- D2 — The blanks go after the real lines, so what a row shows always starts right under its
  title.
- D3 — A third line still grows the row (Out, above).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a terminal's row shows fewer than two lines under its title the system shall draw the row at the two-line height, the lines it has under the title and blank space under them. | `796-01-before` |
| REQ-002 | WHEN a shell's command line appears under its folder line the system shall keep the row's height, and its title and the rows under it shall stay where they were. | `796-02-command` beside `796-01-before` |
| REQ-003 | WHEN an agent's activity line appears under its status the system shall keep the row's height, and its title and the rows under it shall stay where they were. | `796-03-activity` beside `796-02-command` |
| REQ-004 | WHEN a shell's folder line goes away the system shall keep the row's height and move the lines left up under the title. | `796-04-root` beside `796-03-activity` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the padding in `render_terminal_row`; the stale row-height assertions in
  `rail_tests.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
