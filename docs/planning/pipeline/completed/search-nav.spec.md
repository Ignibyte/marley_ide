---
pipeline_id: 1b5af659-2657-458a-a6fc-0521fbe5ed74
ticket: forge#141 (dfc6cda0-499e-4304-96ae-c28f2118ff85) · local docs/planning/tickets/open/TICKET-141-search-nav.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: search UX polish — keyboard nav + activation [M8]
type: feature
milestone: M8 — Warp Chrome & Fidelity
references:
  - crates/marley_app/src/command_bar.rs or app.rs (PURE: move_selection)
  - crates/marley_app/src/app.rs (SHIM: selected index, ↑/↓/↵ routing, highlight, activate_search_hit)
---

## Title
The top search becomes navigable — ↑/↓ move a highlighted selection, ↵ opens it, and a result row activates
(file → viewer, session → focus) instead of only files being clickable.

## Scope
### In
- PURE `move_selection(current, len, delta)` (clamped index nav).
- SHIM: `top_search_selected`; ↑/↓/↵ in `handle_top_search_key`; a highlighted selected row; a shared
  `activate_search_hit` routing File→viewer + Session→focus-by-label.

### Out
- Action-hit activation (the label→action-string map is a follow-up, noted). Ranking changes. Fuzzy tweaks.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `move_selection` clamps to `[0, len-1]` (empty → 0); ↑/↓ step it; ↵ activates `hits[selected]`.
- D2 — `activate_search_hit`: File→`open_file_in_viewer`, Session→focus the pane whose session label matches
  (a label→PaneId map built from `panes_of_kind(Terminal)`), Action→no-op for now (follow-up). Clear+unfocus.
- D3 — the selected index resets to 0 whenever the query changes.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `move_selection(0,5,+1)` runs, it shall return 1; `move_selection(4,5,+1)`→4; `(0,5,-1)`→0; `(2,0,+1)`→0. | unit |
| REQ-002 (visual) | WHEN a query has results, one row shall be highlighted; driving `down` shall move the highlight. | driven capture |
| REQ-003 | WHEN ↵ is pressed on a File hit, it shall open in the viewer; on a Session hit, focus its pane. | code-review + driven |
| REQ-004 | gate GREEN, cov/MSI 100 on move_selection; the shim masked. | gate |

## Phase Plan
- **P2** — move_selection; the selected index + ↑/↓/↵ + highlight + activate_search_hit; test plan.
- **P3** — implement (command_bar/app.rs + app.rs).
- **P3.5** — 1 self-review: the clamp; the reset-on-change; the activation routing.
- **P4** — move_selection tests (cov/MSI 100) + a DRIVEN capture (highlight moves) + gate GREEN.
- **P5** — docs, AAR, archive, close #141.
