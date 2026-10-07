---
pipeline_id: 328ae79a-156d-410e-930c-22dc341eeb86
ticket: docs/planning/tickets/closed/TICKET-670-fold-the-rails-containers-list.md
status: Phase 4 — Complete PASS
title: "Fold the rail's Containers list"
type: feature
slice: the rail's ports (#521, #614, #669)
references: [docs/planning/pipeline/completed/614-container-ports-in-the-rail.spec.md, docs/planning/pipeline/completed/669-test-runs-list-none-of-the-machines-containers.spec.md]
---

## Title
Fold the rail's Containers list. The rail lists every container port no project holds under a
CONTAINERS label (#614); on the dev box that is 13 rows, which push the Harness section down. The
label becomes a header with a chevron and the count, a click folds or unfolds the list, it starts
folded, and the window's saved state keeps it as it was left. Chad asked for it on 2026-10-06.

## Scope
### In
- **The header**: a chevron (Zed's `Disclosure`), CONTAINERS, and the number of ports; the whole
  header and the chevron each fold or unfold the list once per click.
- **Folded**: the header alone; no container row.
- **Remembered**: `marley_containers_open` in the window's saved sidebar blob, beside
  `marley_rail_closed`; a window with none starts folded.

### Out (explicitly deferred)
- **The Harness section's fold** (#632), which has no chevron and is not remembered: not asked.
- **Keys**: container rows stay out of the rail's keys and its filter, as #614 left them.

## Reference (§20)
Upstream Zed — the `ui` crate's `Disclosure`, which Marley's project headers already fold with
(`rail.rs`, `marley-rail-disclosure`), and Zed's sidebar blob, which the rail already extends with
its own fields (`write_rail_state`).

### Prior art
- **Behaviour maps:** none name a container list; the project headers' fold is the house shape.
- **Published material:** none applies.
- **The code we ship:** `ui::Disclosure` (chevron right folded, down open, `IconSize::Small`,
  muted); `RailState`/`SavedRail`/`write_rail_state` (the rail's fields in the window's saved
  blob); `toggle_expanded` (fold, then `MultiWorkspace::serialize`). Nothing new is needed.

## UI proof
`script/e2e/670-fold-the-rails-containers-list.sh` (`compositor sway`): a fake `docker` and a
stand-in `docker-proxy` for port 670, with `marley.rail_containers` on. Shots: `670-01-folded`
(a fresh window: the header with the chevron right and the count, no rows); `670-02-open` (after a
click on the header: the rows, :670 first, the chevron down); `670-03-folded` (after a click on
the chevron); `670-04-restored` (unfolded, then Marley quit and started again: still open).

## Locked-In Decisions
- D1 — **Starts folded**: the list is the machine's, and Chad's complaint is its length.
- D2 — **Per window, in the saved sidebar blob**, as the rail's width and closed state are.
- D3 — **The count is of the ports listed**, the rows the header hides.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Containers header shall show a chevron, CONTAINERS and the number of ports it lists. | `670-01-folded`, `670-02-open` |
| REQ-002 | WHILE the list is folded, the rail shall show the header and no container row. | `670-01-folded`, `670-03-folded` |
| REQ-003 | WHEN the header or its chevron is clicked, the rail shall fold or unfold the list, once. | `670-02-open`, `670-03-folded` |
| REQ-004 | WHEN a window has no saved state for it, the list shall start folded. | `670-01-folded` |
| REQ-005 | WHEN Marley starts again, the window's list shall be folded or open as it was left. | `670-04-restored` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rail.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture note, ledger capture, the brain
  decision, close, archive, commit.
