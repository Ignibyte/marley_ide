---
pipeline_id: 80432f63-0d08-4fea-8f82-50c9ef9e441e
ticket: docs/planning/tickets/open/TICKET-840-needs-you-toasts-and-prompts.md
status: Phase 4 — Complete PASS
title: Needs you entries as cards
type: feature
slice: The Marley look, W3.4 (docs/marley/look-plan.md; toasts and prompts are #869)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The rail's Needs you section as the Inbox board draws it: a heading with the count and "longest
first", and each entry a card: its agent's icon on a tile, who and where on the first line with
how long it has waited at the right, what it asks under them, and its buttons in three weights:
the main answer filled, the others outlined, the quiet ones (Stop turn, Open) plain text.

## Scope
### In
- The heading: NEEDS YOU, the count in a pill, "longest first" at the right, muted.
- Each entry a bordered, rounded card on the panel's raised colour; the tile, who · where, waited,
  the ask (up to two lines), the chips (#568) and the buttons as today.
- The buttons' weights: Allow (or a question's first option) filled with the accent; Allow
  session, Deny, Refuse, Continue at Reset outlined; Stop turn and Open plain.

### Out (explicitly deferred)
- Toasts and prompts (TICKET-869).

## Reference (§20)
N/A — Marley-specific. The inbox is Marley's (#508, `rail.rs::render_inbox`); the board is
Inbox.

### Prior art
- The code we ship: `render_inbox`, `render_inbox_entry` (the icon by kind, `row_card`, the
  chips, the answers' buttons), `InboxAnswer` (`words`, `tooltip`), the inbox's order
  (marley_rail.rs:638, longest first since #508).
- Ledger: #833's scenario (a stand-in Claude Code acting out a permission request through the
  plugin's `event.py`); L-claude-836 (button labels).
- The brain (consultation e0db5975): nothing on this seam.

## UI proof
Scenario `script/e2e/840-needs-you-cards.sh` (sway): projects `b` and `c`, each running #833's
stand-in Claude Code, which asks to write a file; `b` asks first.
- `840-01-night`: the rail's NEEDS YOU 2 · longest first, b's card over c's, each with its tile,
  who · where, waited, the ask and Open (REQ-001, REQ-002).
- `840-02-day`: Marley Day (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — The weights come from the answer: `InboxAnswer::Allow` and a Codex question's first option
  filled; Stop turn and Open plain; every other answer outlined.
- D2 — The click behavior stays: a click on the card shows where the agent waits.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE agents wait on the user, the rail shall head its Needs you section with the count and "longest first", and draw each entry as a card with its tile, who and where, how long it waited and what it asks, in Marley Night and Marley Day. | `840-01-night`, `840-02-day` |
| REQ-002 | An entry's buttons shall be drawn in three weights: the main answer filled, the other answers outlined, and Stop turn and Open plain. | `840-01-night` (Open); the weights of answers no scenario can raise are read in the diff |

## Phase Plan
- **P1 Plan**, **P2 Code** (`rail.rs`), **P3 Test**, **P4 Complete**.
