---
pipeline_id: e1c967b8-f30b-45e1-a1d9-3df5dee48a82
ticket: docs/planning/tickets/open/TICKET-888-diff-review-and-turns.md
status: Phase 4 — Complete PASS
title: The guided review as the DiffReview board draws its guide
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Guide tab (#733) reads as the DiffReview board's guide: "Guide · from the agent" over the
title, "K of N reviewed" with a progress bar, and each section a card with its number and title,
Reviewed at its right, its summary and its file chips.

## Scope
### In
- `guide_tab.rs`: the header, the progress bar, the section card.

### Out (explicitly deferred)
- The diff review's header (Changes since main, files, viewed count, Send Review, Copy notes),
  viewed ticks, notes and findings in Zed's diff, Send to, and a terminal's turns with Keep and
  Undo: new views, TICKET-893.

## Reference (§20)
N/A — Marley-specific: Marley's guided review (#733); the DiffReview board.

### Prior art
- `GuideTab::render`.
- The brain (consultation 4dcd9eba).

## UI proof
Scenario `script/e2e/888-guide-tab.sh` (sway): #733's agent opens a guide of two sections, the
ask allowed for the session.
- `888-01-guide`: the eyebrow, the title, "0 of 2 reviewed" and its bar, the two cards (REQ-001).
- `888-02-ticked`: the first section ticked: "1 of 2 reviewed", the bar half, the card folded
  (REQ-001).
- `888-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — A reviewed section still folds to its title, as #733 made it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Guide tab shall show "Guide · from the agent" over its title, how many sections are reviewed with a bar, and each section as a card with Reviewed at its right, in Marley Night and Marley Day. | `888-01-guide`, `888-02-ticked`, `888-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
