---
pipeline_id: 48490af7-ab17-4347-88f6-3203359aed20
ticket: forge#186 (9ecc172c-c47d-4e41-aec9-35935ea656c5) · local docs/planning/tickets/open/TICKET-186-find-navigation.md
aar_id: 17d0ba55-e30d-4553-a785-7314d23e393f
status: Phase 5 — Complete PASS
title: M12 — scrollback find: next/prev navigation + "n of m" match count
type: feature
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/find.rs (PURE: scrollback_matches + match_label; find_matches/match_navigation already exist)
  - crates/marley_app/src/app.rs (SHIM: the "n of m" find-bar label + the active-match distinct tint)
---

## Title
Extend the #51 find bar: a live "n of m" match counter + an ACTIVE-match highlight distinct from the others, so
stepping through scrollback matches with Enter/⇧Enter (already wired in #51) shows WHERE you are.

## Scope
### In
- PURE `find.rs`: `scrollback_matches(rows: &[String], query) -> Vec<(usize, Range<usize>)>` — the ordered
  (row_index, byte_range) list of every match across all rows (reuses the tested `find_matches` per row; the
  pure core the shim's find_match_rows currently inlines). `match_label(current, total) -> String` — the
  "n of m" counter (1-based): "" when total == 0, else "{current+1} of {total}".
- SHIM `app.rs`: find_match_rows delegates to scrollback_matches; the find-bar label shows match_label(find_
  index, total) (was a bare total count); the pane render tints the ACTIVE match's row (matches[find_index].row)
  with a distinct brighter accent, the other match rows keeping the #51 find_bg.

### Out
- Per-CHARACTER active-match highlight (the current highlight is per-row; the active tint stays per-row too —
  a char-range highlight is a follow-up). Regex / whole-word find. Find across panes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `scrollback_matches` shall return every match as (row_index, range) across the rows, in row-then-column order; empty query / no rows → empty. | unit + mutation |
| REQ-002 | `match_label` shall render "{current+1} of {total}" (1-based), and "" when total == 0. | unit + mutation |
| REQ-003 (visual) | WHEN ⌘F is open with matches, the bar shall show "n of m" and Enter/⇧Enter shall move it (n changes) while the viewport scrolls to each match; the active match shall tint distinctly. | driven capture |
| REQ-004 | gate GREEN; the pure fns cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the two pure fns + the shim (label + active tint + find_match_rows delegation). P3.5 1-2 critics
(the match_label off-by-one/boundary; scrollback_matches ordering + the row-index; the active-tint row lookup +
find_index bounds; mutation). P4 unit + driven (repeated word, Enter to step, counter increments) + gate.
P5 docs.
