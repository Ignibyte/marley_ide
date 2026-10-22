---
pipeline_id: f54edc00-8085-4ec5-aeab-ea391efdc5d4
ticket: docs/planning/tickets/closed/TICKET-863-cards-above-the-bottom-row-and-the-debugging-row.md
status: Phase 4 — Complete PASS
title: The problems and panels cards above the bottom row
type: feature
slice: The Marley look, W1.4 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
In the Marley layout the bottom row's problems count opens a small card above it (errors,
warnings, Open in the drawer) instead of the drawer at once, and the panel group ends with a
button whose card lists every panel, where it opens and its key. The language servers and edit
predictions cards are Zed's, already cards in the theme.

## Scope
### In
- `crates/diagnostics/src/items.rs` (Zed crate): `MarleyProblemsCard`, a global; with it set the
  count opens the card.
- `panel_group.rs`: the Panels card.
- `marley_workbench.rs`/`panel_group.rs`: sets the global in the Marley layout.

### Out (explicitly deferred)
- The debugging row and the language servers card's Ask an agent: TICKET-901.
- Hints in the problems card: Zed's summary counts errors and warnings only.

## Reference (§20)
Upstream Zed's `diagnostics::items::DiagnosticIndicator`, `ui::ContextMenu`.

### Prior art
- A probe: the language servers button and the edit predictions button open cards; the problems
  count opens the Problems drawer at once.
- #862's `PanelGroup`, `panel_tabs`' moved panels.
- The brain (consultation 96cc50c8): nothing on this seam.

## UI proof
Scenario `script/e2e/863-problems-and-panels-cards.sh` (sway, rust-analyzer).
- `863-01-problems`: the count clicked: the card (REQ-001).
- `863-02-drawer`: Open in the drawer: the Problems drawer (REQ-001).
- `863-03-panels`: the group's last button: the Panels card (REQ-002).
- `863-04-day`: Marley Day, the Panels card (REQ-002).

## Locked-In Decisions
- D1 — The cards are Zed's `ContextMenu`, drawn in the theme like the other row cards.
- D2 — Where a panel opens, in the Marley layout: "tab" for one `panel_tabs` moves, "beside the
  rail" for Files, else its dock ("right dock", "bottom drawer").

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks the problems count in the Marley layout, Marley shall open a card above it with the errors and warnings and Open in the drawer, which opens the drawer. | `863-01-problems`, `863-02-drawer` |
| REQ-002 | The panel group shall end with a button whose card lists each panel, where it opens and its key, in Marley Night and Marley Day. | `863-03-panels`, `863-04-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
