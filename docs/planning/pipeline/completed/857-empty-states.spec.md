---
pipeline_id: 044e8360-d72b-45e5-8ebb-2e15d3ff009d
ticket: docs/planning/tickets/open/TICKET-857-empty-and-error-states.md
status: Phase 4 — Complete PASS
title: Empty and error states as one card with one action
type: feature
slice: The Marley look, W6 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Where Marley has nothing yet or something it needs broke, it shows the States board's card: a
glyph tile, a title, a line, and one action. Six places: Home with no project (Open a Folder), the
rail's Threads and the Threads page with none (New Agent) or none matching (Clear the search), the
rail's filter matching nothing (Clear the filter), the Browser tab with no Chromium (Open the
guide), Forge's board unread (Retry, or Open Settings with no provider), and the rail's harness
stopped (Open Settings).

## Scope
### In
- `states.rs` (new): `EmptyState`. Its use in `home_page.rs`, `rail.rs`, `threads_page.rs`,
  `browser.rs`, `forge_board.rs`.

### Out (explicitly deferred)
- Rusty dropped (a Try Now needs a reconnect Rusty does not offer) and an agent CLI missing (no
  place shows it before a start; a check before the start is new behaviour): TICKET-894.

## Reference (§20)
N/A — Marley-specific: the States board.

### Prior art
- The texts each place shows today (an Explore pass); #870's `prompts::render_button`;
  `rail::new_agent_picker`, `workspace::Open`, `agent_ui::NewThread`, `OpenGuide`,
  `OpenSettings`, `ForgeBoard::refresh`.
- The brain (consultation 03949cea).

## UI proof
Scenario `script/e2e/857-empty-states.sh` (sway): a scratch repository, no harness command that
runs, no Forge provider, `MARLEY_BROWSER`/Chromium pointed at a missing file.
- `857-01-filter`: the rail filtered for "dpkg": the card and Clear the filter (REQ-001).
- `857-02-threads`: the rail's Threads view searched for "dpkg": the card and Clear the search
  (REQ-001). Not shot: its "No conversations yet" card, since the run's copy of the database keeps
  one Rusty thread (#877's finding); the Threads page draws the same card from the same
  component.
- `857-03-browser`: the Browser tab with no Chromium: the card and Open the guide (REQ-001).
- `857-04-forge`: Forge's board with no provider: the card and Open Settings (REQ-001).
- `857-05-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — One action per state; a broken state's tile and border take the error's tint.
- D2 — Each card says what the place said before, in the board's words where they are true.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a place has nothing to show or what it needs broke, Marley shall show one card with a glyph, a title, a line and one action, in Marley Night and Marley Day. | `857-01` to `857-05` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
