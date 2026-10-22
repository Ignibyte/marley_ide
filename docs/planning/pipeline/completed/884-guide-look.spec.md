---
pipeline_id: c6c80163-c335-4768-9d5d-7a34d532af78
ticket: docs/planning/tickets/open/TICKET-884-search-branches-history-extensions-guide.md
status: Phase 4 — Complete PASS
title: The Marley Guide in Marley's colours
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Marley Guide page takes Marley Night's and Marley Day's colours (the cat's coat for its
accent) for the dark and light schemes, and draws each article as a card, as the SearchMore board
draws "Marley Guide, in a tab".

## Scope
### In
- `crates/marley_workbench/guide/index.html`: the stylesheet's colour variables and the article
  card; no text changes.

### Out (explicitly deferred)
- The guide's "Try it" links that run a command, project search, the branch picker's notes,
  history in the Changes inspector and the extensions list: TICKET-891.

## Reference (§20)
N/A — Marley-specific: Marley's guide page (#599); the SearchMore board.

### Prior art
- `guide/index.html`'s `:root` variables and its dark-scheme block; `assets/themes/marley/
  marley.json` (Night and Day's values).
- The brain (consultation c22e7bbb).

## UI proof
Scenario `script/e2e/884-guide-look.sh` (sway): an offline Chromium; `marley: open guide`.
- `884-01-guide`: the guide in the dark scheme: Night's background, the nav, an article card, the
  accent (REQ-001).
- `884-02-day`: Marley Day, and the page in the light scheme when the tab follows it, else noted
  (REQ-001).

## Locked-In Decisions
- D1 — The page keeps following the browser's scheme: dark takes Night's values, light Day's.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Marley Guide shall draw in Marley Night's colours in the dark scheme and Marley Day's in the light, with each article a card. | `884-01-guide`, `884-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
