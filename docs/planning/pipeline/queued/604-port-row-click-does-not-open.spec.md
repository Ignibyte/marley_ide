---
pipeline_id: b3eb9bf7-f482-4194-8739-60cc2004b2cb
ticket: docs/planning/tickets/open/TICKET-604-port-row-click-does-not-open.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A click on a port row opens nothing"
type: feature
slice: workbench shell (the rail's port rows), after #521
references: [docs/planning/pipeline/completed/521-ports-per-project.spec.md, docs/planning/pipeline/completed/453-rail-keyboard-and-reorder.spec.md]
---

## Title
A single click on a port row marks it instead of opening its URL in a Browser tab; the row's Open
button, a double-click and Enter open it.

## Scope
### In
- **A single click** on a port row's body puts the rail's keyboard cursor on the row (#453's
  `Focus::cursor`), with the focus in the rail, and opens nothing.
- **Opening** stays on the Open button (unchanged), a double-click on the row, and Enter while the
  cursor is on it (#453 runs a row's click handler on Enter, so Enter gets the open handler
  explicitly).
- **The row's tooltip** gains "Double-click to open in a Browser tab".

### Out (explicitly deferred)
- Any other row's click (terminal, Browser tab, thread, worktree rows keep opening on one click).
- Telling a listener with no web page apart before opening (an HTTP probe).

## Reference (§20)
N/A — Marley-specific: port rows are Marley's own (#521). The pattern follows Zed's own lists
where a single click selects and a second click or Enter opens (the project panel's entries with
`project_panel.open_file_on_double_click`), kept here as selection on one click and open on two.

### Prior art
- **Behavior maps.** #521's spec (the row's click opens its URL); #453's spec (the rail's cursor,
  Enter runs the click handler).
- **Published material.** None needed.
- **Code we already ship.** gpui's `ClickEvent::click_count()` (the terminal row already uses it
  for rename on a double-click, `rail.rs` `render_terminal_row`); the rail's `Focus::cursor` in
  `marley_rail`; `render_port_row`'s row `on_click` (`rail.rs` around 4060).

## UI proof
The scenario `script/e2e/604-port-row-click-does-not-open.sh` (sway) starts a server in the
scratch project, single-clicks its port row and shoots the rail with the row marked and no
Browser tab (`single.png`), presses Enter and shoots the Browser tab (`enter.png`), closes it,
double-clicks the row and shoots the tab again (`double.png`).

## Locked-In Decisions
- D1 — One click marks, two open; Enter and the Open button open.
- D2 — Only port rows change.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user single-clicks a port row, the rail shall put its cursor on the row and open no Browser tab. | Shot `single.png` |
| REQ-002 | WHEN the user presses Enter with the cursor on a port row, Marley shall open its URL in a Browser tab of the project. | Shot `enter.png` |
| REQ-003 | WHEN the user double-clicks a port row, Marley shall open its URL in a Browser tab of the project. | Shot `double.png` |
| REQ-004 | WHEN the user clicks the row's Open button, Marley shall open its URL as before. | Review of the diff |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes.
- **P2 Code** — `render_port_row`'s click and the rail's Enter for a port row; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the guide page and the walkthrough (§21); the ledger; close,
  archive, commit.
