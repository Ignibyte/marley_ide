---
pipeline_id: e4bab4c5-29f3-46ce-85d1-701fd3cadec2
ticket: forge#54 (d57071a4-c0bd-407e-96c0-3c7688eb25e1) · local docs/planning/tickets/open/TICKET-054-search-core.md
aar_id: a9a135e6-5a33-4394-b5e7-4c77ee0e94a7
status: Phase 5 — Complete PASS
title: marley_search_core — the fuzzy ranking engine
type: feature
milestone: M2.A
references:
  - crates/marley_search_core/ (NEW crate — fuzzy_score + fuzzy_rank, wraps nucleo)
  - crates/marley_app/src/palette.rs (refactor field_score → marley_search_core::fuzzy_score)
  - crates/marley_app/Cargo.toml (nucleo → marley_search_core)
---

## Title
The M2 SEARCH seam: a shared, tested fuzzy matcher (`marley_search_core`) wrapping the `nucleo`
subsequence primitive. Extract it from the palette's local `field_score`, refactor the palette onto
it — ONE matcher — and give #57 fuzzy file-open its engine.

## Scope
### In
- `crates/marley_search_core/` (NEW crate, gpui-free PURE — cov/MSI 100; dep `nucleo = "0.5.0"`):
  - `pub fn fuzzy_score(text: &str, query: &str) -> Option<u32>` — the case-insensitive nucleo
    subsequence score (lowercase both, `Matcher::fuzzy_match`, `.map(u32::from)`); `None` when the
    query's chars don't occur in order. (EXACTLY the palette's current `field_score`, extracted.)
  - `pub struct Scored { pub index: usize, pub score: u32 }` (Debug/Clone/Eq).
  - `pub fn fuzzy_rank(candidates: &[&str], query: &str) -> Vec<Scored>` — empty query → every candidate
    in input order (score 0); else score each (a shared internal `Matcher`), KEEP only matches, sort by
    score DESC then index ASC.
  - private `score_with(matcher, text, query)` shared by both.
- Refactor `crates/marley_app/src/palette.rs`: delete local `field_score` + `use nucleo::…`;
  `filter_commands` calls `marley_search_core::fuzzy_score` per field (SAME multi-field-MAX + sort;
  behavior unchanged — the existing palette tests are the regression guard).
- `crates/marley_app/Cargo.toml`: REMOVE `nucleo` (now unused by marley_app → machete gate:9), ADD
  `marley_search_core`.

### Out
- The `SearchMixer` the palette comment mentions (multi-source mixing) — a later M2 ticket. Hand-rolled
  boundary/consecutive scoring — that's nucleo's job, not ours. `fuzzy_rank` over real files — #57.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — WRAP `nucleo` (already a workspace dep, the palette's proven matcher, and named in palette.rs's
  own comments) — NOT a hand-rolled matcher. Supersedes the ticket text's "no-deps, hand-rolled".
- D2 — `Scored.score: u32` (nucleo's score type; matches `ScoredCommand.score`) — NOT the ticket's i64.
- D3 — Refactor the palette in THIS ticket (one matcher) — the ticket asks to "REPLACE the palette's
  fuzzy logic". Behavior is preserved (fuzzy_score ≡ the old field_score), so palette tests stay green.
- D4 — Must remove `nucleo` from marley_app/Cargo.toml when its last use goes (else machete fails) — a
  load-bearing part of the change, not cleanup.
- D5 — `fuzzy_score` makes its own `Matcher`; `fuzzy_rank` reuses ONE across candidates (files can be
  many) — via the shared private `score_with`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `fuzzy_score(text, query)` is called and `query`'s chars occur in order in `text` (case-insensitive), it shall return `Some(score)`; otherwise `None`. | unit (match/non-match/case) |
| REQ-002 | WHEN `fuzzy_rank(candidates, "")` (empty query), it shall return every candidate as `Scored{index, score:0}` in input order. | unit |
| REQ-003 | WHEN `fuzzy_rank(candidates, query)` with a non-empty query, it shall return ONLY matching candidates, sorted by score DESC then index ASC. | unit (ranked, excluded, tie-break) |
| REQ-004 | WHEN the palette's `filter_commands` runs after the refactor, it shall produce the same ranking as before (behavior preserved). | the existing palette tests (regression) |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN incl. machete (nucleo removed cleanly) + cov/MSI 100 on marley_search_core. | gate exit 0 + receipt |

## Phase Plan
- **P2** — the crate skeleton, the `score_with`/`fuzzy_score`/`fuzzy_rank` shapes, the palette-refactor
  diff, the dep swap, mutation targets, test plan (incl. the palette regression).
- **P3** — the crate + the palette refactor + the Cargo.toml swap.
- **P3.5** — critic: the subsequence/case semantics, the sort comparator (desc+index), empty-query, the
  palette behavior-preservation, machete cleanliness, the nucleo-wrapper hollow-MSI caveat.
- **P4** — fuzzy_score + fuzzy_rank unit tests + the palette tests still green + gate GREEN. No UI.
- **P5** — docs (arch + crate-map), AAR, archive, close #54.
