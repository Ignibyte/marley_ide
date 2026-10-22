---
pipeline_id: 3caaf04e-ef5a-4746-ae81-169a7a36c3e2
ticket: docs/planning/tickets/open/TICKET-875-rusty-brain-and-knowledge.md
status: Phase 4 — Complete PASS
title: Rusty's Brain tab, page and Knowledge as the boards draw them
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Rusty's Brain tab, a page and the Knowledge panel read as the BrainPage and Knowledge boards draw
them: the vault under a Vault heading (Favourites above it, #662), the page's outline under "On
this page", and the Knowledge panel as cards (Following with the page's tags, Backlinks with their
count, Links out with an unwritten page marked "not written yet").

## Scope
### In
- `brain.rs`: a Vault heading over the tree.
- `page.rs`: an "On this page" heading over the outline column.
- `knowledge_panel.rs`: the page view as three cards; a missing link's "not written yet".

### Out (explicitly deferred)
- The page's "Linked here" (the page view reads no backlinks; the Knowledge panel shows them).
- The Graph: its tab and settings already follow the board's shape (#647, #657).

## Reference (§20)
N/A — Marley-specific: Rusty's Brain tab (#678), page (#645, #656) and Knowledge panel (#646,
#655); the BrainPage and Knowledge boards.

### Prior art
- `brain.rs` `render_favourites` (its `ListSubHeader`, F-659's flex note), `page.rs`
  `render_outline`, `knowledge_panel.rs` `render_page`, `outgoing_row`, `section`.
- The brain (consultation ee22b1df): nothing on this seam.

## UI proof
Scenario `script/e2e/875-brain-screens.sh` (sway): #645's three made-up pages; Atlas links to the
renderer decision and to the unwritten `ideas/later`.
- `875-01-brain`: the Brain tab: Vault over the tree (REQ-001).
- `875-02-page`: Atlas: "On this page" over its outline (REQ-001).
- `875-03-knowledge`: the Knowledge panel: Following, Backlinks, Links out as cards, ideas/later
  "not written yet" (REQ-002).
- `875-04-day`: Marley Day, the page (REQ-001).

## Locked-In Decisions
- D1 — The panel's cards are bordered blocks inside its scroll, one per section, the count at the
  heading's end.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Brain tab shall title its tree Vault, and a page shall title its outline "On this page", in Marley Night and Marley Day. | `875-01-brain`, `875-02-page`, `875-04-day` |
| REQ-002 | The Knowledge panel shall show the page it follows, its backlinks and its links out as cards, an unwritten link marked "not written yet". | `875-03-knowledge` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
