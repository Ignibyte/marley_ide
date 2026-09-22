---
pipeline_id: 60882aa4-7eac-4ad2-8d6d-3341ea2d9637
ticket: forge#131 (b0ed44a3-66fd-4470-867d-43416ab2538e) · local docs/planning/tickets/open/TICKET-131-focus-accent.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: pane focus navigation + the Warp focus accent (M6)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/workspace.rs (PURE: focus_neighbor)
  - crates/marley_app/src/keymap.rs (PURE: ⌘⌥-arrow bindings)
  - crates/marley_app/src/app.rs (SHIM: dispatch focus-*, the accent edge)
---

## Title
Move focus between panes with ⌘⌥-arrow, and mark the focused pane with a Warp-style bright left+top edge
accent instead of a full cyan border.

## Scope
### In
- PURE `Workspace::focus_neighbor(dir)` (reuses the tested `PaneGroup::neighbor`).
- PURE keymap: 4 ⌘⌥-arrow bindings → `focus-left/right/up/down`.
- SHIM: dispatch the 4 actions; the focused pane's left+top accent edge.

### Out
- Geometric `pane_in_direction` (reusing the tree neighbor — see D1). Reworking split/close.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — REUSE the tested tree-based `PaneGroup::neighbor` (no render-time rects needed in the key handler)
  rather than a new geometric `pane_in_direction`. `focus_neighbor(dir)` wraps it: focus the neighbor, or
  return false at a grid edge.
- D2 — the focus affordance is a subtle `border_1` frame on every pane PLUS a left+top accent edge on the
  focused pane (matching the Warp screenshots), not a full accent border.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `focus_neighbor(dir)` has a neighbor that way, it shall focus it and return true. | unit |
| REQ-002 | WHEN there is no neighbor that way (grid edge), `focus_neighbor` shall return false and leave focus. | unit |
| REQ-003 | WHEN a ⌘⌥-arrow is pressed, `action_for` shall resolve the matching `focus-<dir>` action. | unit |
| REQ-004 (visual) | WHEN a pane is focused, it shall show a left+top accent edge, not a full border. | live capture |
| REQ-005 | gate GREEN, cov/MSI 100 on focus_neighbor + the keymap; the shim masked. | gate |

## Phase Plan
- **P2** — focus_neighbor; the 4 keymap bindings; the accent-edge render; test plan.
- **P3** — implement (workspace.rs + keymap.rs + app.rs).
- **P3.5** — 1 self-review: the neighbor arms; the 4 bindings; the accent edges.
- **P4** — focus_neighbor + keymap tests (cov/MSI 100) + a LIVE capture (the accent edge) + gate GREEN.
- **P5** — docs, AAR, archive, close #131.
