---
pipeline_id: 40d53bdf-c451-4197-a11d-c5e75a13523a
ticket: docs/planning/tickets/open/TICKET-853-splits-and-ports.md
status: Phase 4 — Complete PASS
title: The focused pane outlined in the accent
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
With the center split, the focused pane has a 2 px outline in the accent (the cat's coat), as the
Splits board draws it; a single pane has none.

## Scope
### In
- `assets/settings/default.json`: `active_pane_modifiers.border_size` 2 with a `Marley:`
  comment.
- `crates/workspace/src/pane_group.rs`: the active pane's border painted in
  `pane_focused_border` rather than `border_selected` (a `Marley:` hunk).
- `assets/themes/marley/marley.json`: `pane.focused_border` the accent in both themes.

### Out (explicitly deferred)
- The Ctrl Tab switcher, the port rows, the Containers panel and Collab: TICKET-883.

## Reference (§20)
Upstream Zed, `workspace`: `active_pane_modifiers.border_size` already draws an inset border
round the active pane of a split (`PaneAxisElement::paint`), in `border_selected`.

### Prior art
- `pane_group.rs` `PaneAxisElement::paint`; `pane_focused_border`'s one other reader is the
  debugger's focused sub-pane, which wants the same accent.
- The brain (consultation 51dc9072).

## UI proof
Scenario `script/e2e/853-focused-pane.sh` (sway): a scratch repository with two files.
- `853-01-split`: a file split right, the right pane focused: its accent outline, none on the
  left (REQ-001).
- `853-02-other`: the left pane focused: the outline moved (REQ-001).
- `853-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — A setting default rather than code: `border_size: 0` turns it off again.
- D2 — Zed's `border_selected` keeps its other uses; only the pane border changes colour key.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the center is split, the focused pane shall have a 2 px outline in the accent and the other panes none, in Marley Night and Marley Day. | `853-01-split`, `853-02-other`, `853-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
