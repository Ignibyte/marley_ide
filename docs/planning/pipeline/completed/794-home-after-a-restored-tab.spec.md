---
pipeline_id: 0644e057-859e-478e-aa8f-c91e4ca59566
ticket: docs/planning/tickets/open/TICKET-794-home-missing-after-a-restored-tab.md
status: Phase 4 — Complete PASS
title: Home stays first in the Default group after a relaunch restores its tabs
type: bug
slice: Home (#700, #701)
references: []
---

## Title
Home goes missing from the Default group when a relaunch restores the group's saved tabs: Zed's
session restore swaps in the saved center pane after Marley put Home in the old one, and nothing
puts it back. Seen on the Mac with Onboarding (2026-10-16); reproduced on Linux on 2026-10-29 with
a terminal saved in Default.

## Scope
### In
- A `workspace::Event::CenterRestored`, emitted once by `Workspace::load_workspace` after it swaps
  in the restored center.
- Home's `init` handles it: in the Default group's workspace, `ensure` puts Home first.

### Out (explicitly deferred)
- Saving Home's page in the session: Home is made fresh each start, as #701 decided.
- #795, which waits on the Mac.

## Reference (§20)
Upstream Zed (`workspace`): `load_workspace` deserializes the saved center group, removes the
panes the workspace was made with (`remove_panes`) and installs the restored group. Zed keeps that
order; Marley adds one event after it so an observer can act on the restored panes. Zed's own
items (Welcome, Onboarding) are opened by the caller before or after the restore, never into it,
so Zed has no analog of an item that must survive the swap.

### Prior art
- Zed's `workspace` crate: `Event::PaneAdded` fires while each saved pane is built, before the
  swap, when the old center (with Home) is still in place, so `ensure` finds Home and does
  nothing; `ActiveItemChanged` fires after the swap but on every tab change, so re-adding Home on
  it would bring back a Home the user closed. `is_restoring()` is false again by the time either
  is delivered. No existing event marks the end of a restore.
- Marley: `home_page::ensure` already puts Home first in a pane that holds other tabs (#703), and
  `fill_shown` and `fill_emptied` are the two existing triggers.
- F-claude-700 (Home took Zed's start workspace with its Onboarding tab): the start workspace is
  Home's; this ticket keeps that.

## UI proof
`script/e2e/794-home-after-a-restored-tab.sh` (sway):
- `794-01-before`: the Default group shown from the rail, a terminal opened in it beside Home.
- `794-02-restored`: after a quit and a launch with no folder named (the session restores), the
  Default group with its terminal, Home first in its tabs and listed in the rail (REQ-001).
- `794-03-closed`: Home closed by hand, then another tab chosen: Home stays closed (REQ-002).

## Locked-In Decisions
- D1 — A new `workspace::Event` variant, not a Marley-side record of the Home page it added: the
  event says exactly what happened, and nothing in the tree matches `workspace::Event`
  exhaustively, so the variant breaks no match.
- D2 — Home is added only in the Default group's workspace (`groups::is_home_workspace`), first and
  without taking the active tab, as `ensure` does now.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a relaunch restores saved tabs into the Default group, Marley shall put Home first in the group's tabs, and the rail shall list it. | `794-02-restored` |
| REQ-002 | WHEN the user closes Home while the Default group holds other tabs, Marley shall not put it back until the group is shown again or emptied. | `794-03-closed` |
| REQ-003 | `Workspace::load_workspace` shall emit `Event::CenterRestored` once, after the restored center is installed, and only when a center was restored. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the event in `workspace`, its ledger row, Home's handler; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
