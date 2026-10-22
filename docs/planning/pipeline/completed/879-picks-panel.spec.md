---
pipeline_id: c4a1182f-b14e-4cb1-b2f1-4d125ef2c785
ticket: docs/planning/tickets/open/TICKET-879-browser-tools-and-panels.md
status: Phase 4 — Complete PASS
title: The Browser's picks as a panel of cards
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Browser tab's picks leave the tray above the page for a panel at its right, "Picks for the
agent", as the Browser board draws it: each pick a card with its number and summary, where its
listener is, a Note field and a primary Send (or what was sent), with Check and Discard.

## Scope
### In
- `browser.rs`: `render_tray` as the panel, `render_pick_row` as a card; the page and the
  panel side by side.

### Out (explicitly deferred)
- "Goes with it" and "To": a pick's screenshot and source go with every send today, and Send
  types into the terminal used last; choosing them is a feature of its own.
- The console drawer (Console, Network, Recordings, Send to the agent), held clicks, dialogs,
  Playwright scripts, recordings, take over and clearing a project's data: TICKET-887.

## Reference (§20)
N/A — Marley-specific: Marley's element picker (#496, #505, #518); the Browser board.

### Prior art
- `render_tray`, `render_pick_row`, `render_check`, `render_listener_place`, the caption
  editors and `SendPick`; #870's `prompts::render_button`.
- The brain (consultation 9f64d7aa).

## UI proof
Scenario `script/e2e/879-picks-panel.sh` (sway): an offline Chromium, a page served in the run
with a card and a button.
- `879-01-picked`: the button picked: the panel at the page's right with the pick's card, its
  Note field and Send (REQ-001).
- `879-02-sent`: a note typed and sent: the card says what was sent (REQ-001).
- `879-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The panel is 320 px and shows only while the tab has a pick or a pick error, as the tray
  did; the page takes the rest of the width.
- D2 — A pick's actions keep their ids, keys and tooltips.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Browser tab has picks, it shall show them in a panel at the page's right, each a card with its number, summary and place, a Note field and a primary Send or what was sent, in Marley Night and Marley Day. | `879-01-picked`, `879-02-sent`, `879-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
