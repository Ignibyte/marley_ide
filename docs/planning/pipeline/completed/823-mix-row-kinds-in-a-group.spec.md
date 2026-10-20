---
pipeline_id: f1881ba4-9e47-40bf-bcea-4166da577559
ticket: docs/planning/tickets/closed/TICKET-823-mix-row-kinds-in-a-group.md
status: Phase 4 — Complete PASS
title: Any row goes anywhere in its group
type: feature
slice: the Marley layout's rail (#602 drag, #613 and #818 moves)
references: []
---

## Title
A group's terminals, Browser tabs and threads stop being three sections: a row drags to any place
among them, and the rail keeps that order (Chad, 2026-10-20: "you can reorder items inside of the
groups. You can move groups but not reorder there", then "mix any order").

## Scope
### In
- Under the window order (`marley.rail_order: "window"`, the default since #671), a project's own
  terminals (the main checkout's), its Browser tabs and its threads are one run: one `Run` for a
  drag, one order in the rail.
- The order is the one #602 already saves per group (`SavedOrder::rows`, one list of places for
  every kind); a row with no saved place keeps today's order after the placed ones: terminals,
  then Browser tabs, then threads.
- The keyboard, the selection and the switcher read the same walk, so they follow the new order.

### Out (explicitly deferred)
- A linked worktree's terminals stay under their worktree's row (#510).
- The attention order (`rail_order: "attention"`) keeps its sections by kind and class: it sorts
  by need, and a mixed run would fight its sort.
- Home's page, other tabs, the Files row and the ports keep their places; they do not drag.
- tmux sessions (#821, queued) are not rows yet.

## Reference (§20)
Upstream Zed: a pane's tab bar orders every kind of item in one list, a terminal beside an editor
beside a thread, and a drag moves a tab anywhere in it (`workspace` crate, `Pane::move_item` and
the tab bar's drop). The rail's rows mirror a group's tabs, so they take the same rule.

### Prior art
- #602's `rail_order.rs` already saves one mixed list per group and `drop_row` already builds it
  from all three kinds; only `marley_rail::run` and `walk` keep the kinds apart. No new storage.
- `marley_rail::place` and `move_to` are the sort and the move to reuse.
- Checked: nothing in gpui or Zed's `sidebar` crate orders mixed rows for us; Zed's Threads Sidebar
  lists threads only.

## UI proof
Scenario `script/e2e/823-mix-row-kinds-in-a-group.sh`, `compositor sway`: a project `repo` with a
terminal, a Browser tab and a Marley thread (#798's scripted agent).
- `823-01-sections` — the rows as today: terminal, Browser tab, thread (REQ-003).
- `823-02-line` and `823-03-thread-first` — the thread's row dragged over the terminal's: the drop
  line shows, and dropped, the thread is first (REQ-001).
- `823-04-browser-between` — the Browser tab's row dropped on the terminal's from below: it sits
  between the thread and the terminal (REQ-001).
- `823-05-restart` — Marley started again on the same profile: the same mixed order (REQ-002).

## Locked-In Decisions
- D1 — Only the window order mixes; the attention order keeps sections (see Out).
- D2 — The mixed run is a new `Run::Rows { project }`; worktree terminals keep
  `Run::Terminals { worktree: Some(..) }`.
- D3 — `ProjectSnapshot` gains `order: Vec<Selection>`, the saved places resolved to rows by
  `rail_order::arrange`; `marley_rail` sorts by it, so the pure crate needs no place strings.
- D4 — Worktree rows and their terminals come after the mixed run, where a worktree's section sat
  after the main checkout's terminals.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user drops a terminal's, Browser tab's or thread's row on another such row of the same group, the rail shall show the dropped row in that place, whatever the two rows' kinds. | `823-03-thread-first`, `823-04-browser-between` |
| REQ-002 | WHEN Marley starts again, the rail shall show the group's rows in the order the user left them. | `823-05-restart` |
| REQ-003 | WHILE the user has not reordered a group, the rail shall list its terminals, then its Browser tabs, then its threads, as before. | `823-01-sections` |
| REQ-004 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rail` (order, run, walk), `rail_order.rs` (arrange, drop), `rail.rs`
  (constructors); a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_rail.md`, the guide, ledger
  capture, close, archive, commit.
