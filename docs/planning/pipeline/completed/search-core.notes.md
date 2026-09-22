# marley_search_core — the fuzzy ranking engine — Notes

- **Forge ticket:** #54 `d57071a4-c0bd-407e-96c0-3c7688eb25e1`
- **AAR:** `a9a135e6-5a33-4394-b5e7-4c77ee0e94a7`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-054-search-core.md

## Phase 1 — Plan
- **Request:** forge #54 (M2.A seq-2, auto-approved) — the shared fuzzy search engine.
- **Classification:** work pipeline, `feature`, a PURE new crate + a behavior-preserving refactor of
  the palette. No UI (the palette OVERLAY render is untouched; only its pure `filter_commands` seam).
- **Pre-flight discovery (KEY):** `palette.rs` already uses `nucleo = "0.5.0"` and its own doc comments
  name `marley_search_core::fuzzy_rank`/`SearchMixer` as the intended home. So the correct approach is
  EXTRACT nucleo into the new crate + refactor the palette onto it — NOT the ticket's original
  "hand-rolled, no-deps" (that predated seeing the code). Recorded as D1/D2/D3 (deviations from the
  ticket text, justified).
- **The palette's current `field_score`** (palette.rs:41): lowercase text+query, `Matcher::fuzzy_match`
  → `Option<u32>`. Becomes `marley_search_core::fuzzy_score` verbatim. `filter_commands` keeps its
  multi-field-MAX + sort; only the per-field call changes.
- **Machete trap (D4):** removing nucleo's last marley_app use REQUIRES removing it from
  marley_app/Cargo.toml or gate:9 (cargo-machete) fails on the unused dep.
- **Hollow-MSI caveat (from #53 `PR-claude-method-call-code-mutation-hollow-001`):** `fuzzy_score` is a
  thin nucleo wrapper (method calls) → few viable mutants; `fuzzy_rank`'s empty-query branch + sort
  comparator carry the real mutants. Behavioral asserts + coverage guard fuzzy_score.
- **AAR id:** `a9a135e6-5a33-4394-b5e7-4c77ee0e94a7`.

## Phase 2 — Design

### PURE — `crates/marley_search_core/src/lib.rs` (NEW)
```rust
use nucleo::{Config, Matcher, Utf32Str};

/// A candidate's index in the input plus its fuzzy score (higher = stronger match).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scored { pub index: usize, pub score: u32 }

/// The case-insensitive subsequence score of `query` against `text` via `nucleo`; `None` when the
/// query's characters do not occur in order.
pub fn fuzzy_score(text: &str, query: &str) -> Option<u32> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    score_with(&mut matcher, text, query)
}

/// Rank `candidates` against `query`: an empty query returns every candidate in input order
/// (score 0); otherwise the MATCHING candidates, sorted by score descending then input index ascending.
pub fn fuzzy_rank(candidates: &[&str], query: &str) -> Vec<Scored> {
    if query.is_empty() {
        return candidates.iter().enumerate().map(|(index, _)| Scored { index, score: 0 }).collect();
    }
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut scored: Vec<Scored> = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, text)| score_with(&mut matcher, text, query).map(|score| Scored { index, score }))
        .collect();
    scored.sort_by(|a, b| b.score.cmp(&a.score).then(a.index.cmp(&b.index)));
    scored
}

/// The shared nucleo subsequence primitive, using a caller-owned matcher (so `fuzzy_rank` reuses one).
fn score_with(matcher: &mut Matcher, text: &str, query: &str) -> Option<u32> {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    let mut text_buf = Vec::new();
    let mut query_buf = Vec::new();
    matcher
        .fuzzy_match(Utf32Str::new(&text_lower, &mut text_buf), Utf32Str::new(&query_lower, &mut query_buf))
        .map(u32::from)
}
```
`Cargo.toml`: `[dependencies] nucleo = "0.5.0"`; no dev-deps needed.

### Palette refactor — `crates/marley_app/src/palette.rs`
- DELETE `use nucleo::{Config, Matcher, Utf32Str};` and the `field_score` fn (lines ~39-52).
- `filter_commands`: drop `let mut matcher = …;`; change `field_score(&mut matcher, field, query)` →
  `marley_search_core::fuzzy_score(field, query)`. Everything else (empty-query, multi-field MAX, the
  `sort_by(score desc, index)`) UNCHANGED → behavior identical.
- Update the module doc comments (lines 1-6) — `fuzzy_rank`/`fuzzy_score` are now real.

### File manifest
- NEW `crates/marley_search_core/Cargo.toml` (nucleo dep).
- NEW `crates/marley_search_core/src/lib.rs` (Scored, fuzzy_score, fuzzy_rank, score_with, tests).
- MODIFY `crates/marley_app/src/palette.rs` (drop local nucleo + field_score; call fuzzy_score).
- MODIFY `crates/marley_app/Cargo.toml` (REMOVE `nucleo = "0.5.0"`; ADD `marley_search_core = { path = "../marley_search_core" }`).

### Mutation Targets
- REALITY (confirmed at inspect via `cargo mutants --list`): cargo-mutants 27.1.0 emits ONLY
  whole-function `FnValue` mutants here — `fuzzy_score`/`score_with` → `None`/`Some(0)`/`Some(1)`;
  `fuzzy_rank` → `vec![]` (and an UNVIABLE `vec![Default::default()]`, excluded since `Scored: !Default`).
  It does NOT isolate the `sort_by` comparator, the `.then` tiebreak, the empty-query `if`, or the
  `filter_map`. So MSI is carried by killing those whole-fn replacements; REQ-001..003 do that.
- REQ-003 is therefore a BEHAVIORAL order-guard + a coverage driver for the sort/filter_map lines (top
  match NOT at index 0, non-match excluded) — NOT a comparator-mutant killer (that mutant isn't
  generated). Same hollow-MSI family as #53 (`PR-claude-method-call-code-mutation-hollow-001`):
  correctness rides on coverage + behavioral asserts, not the mutation count.

### Regression Test Plan (pure unit; no UI)
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `fuzzy_score_matches_case_insensitively` — `fuzzy_score("Main.rs","MN").is_some()`, `fuzzy_score("abc","xyz").is_none()`, `fuzzy_score("MAIN","main").is_some()` | unit |
| REQ-002 | `fuzzy_rank_empty_query_is_identity` — `fuzzy_rank(&["a","b","c"], "")` → indices `[0,1,2]`, all score 0 | unit |
| REQ-003 | `fuzzy_rank_orders_and_excludes` — `fuzzy_rank(&["other","src/main.rs","main"], "main")` → indices `[2,1]` (exact "main"@2 outranks the gapped "src/main.rs"@1; "other" excluded — top is NOT index 0) | unit |
| REQ-003b | `fuzzy_rank_ties_break_by_index` — `fuzzy_rank(&["ab","ab"], "ab")` → `[0,1]` (equal score → input index order) | unit |
| REQ-004 | palette `filter_commands` ranking unchanged | existing palette tests (regression) |
| REQ-005 | gate GREEN incl. machete (nucleo removed) + cov/MSI 100 on marley_search_core | gate |

Uncoverable: none. nucleo's internal scoring is a dep (not covered/mutated by us); we assert ORDER +
match/non-match, never exact score values (they're version-local).

### Risks / decisions
- D-2.1 The palette now makes a `Matcher` per field call (fuzzy_score is self-contained) vs the old
  one-matcher-per-filter — negligible for ~10 commands, behavior identical. D-2.2 REQ-003 asserts
  nucleo ORDERING ("main" exact > "src/main.rs" gapped) — if a nucleo version flips it, adjust the
  fixture (order, not exact score, is the contract). D-2.3 machete removal of nucleo is load-bearing
  (D4). D-2.4 The `.then(index)` tiebreak is belt-and-suspenders (sort_by is stable) — mirrors the
  palette's proven pattern; cargo-mutants doesn't isolate it, so no surviving mutant.

## Phase 3 — Implement
- **Built:** NEW `crates/marley_search_core/{Cargo.toml, src/lib.rs}` — `Scored`, `fuzzy_score`,
  `fuzzy_rank`, private `score_with` (verbatim from the design). Refactored `palette.rs` — dropped
  `use nucleo::…` + the `field_score` fn; `filter_commands` now calls `marley_search_core::fuzzy_score`;
  module doc updated. `marley_app/Cargo.toml` — `nucleo` REMOVED, `marley_search_core` ADDED.
- **Deviations:** none from the design. (Ticket-text deviations D1–D5 already logged in Phase 1.)
- **Verification:** `cargo fmt`; `cargo check --workspace` 0 err (marley_search_core + marley compile);
  **`cargo nextest run -p marley` → 115 passed** (the palette regression — behavior preserved). The
  crate package is `marley` (dir `marley_app`). search_core tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (16-query palette-equivalence probe + real cargo-mutants + real nucleo-order check +
  machete/clippy/rustdoc). Verdict: **SHIP** — no HIGH/MED. cov 100 (121/121 regions), MSI 100
  (8 mutants / 7 viable / 7 caught / 0 missed / 1 unviable `vec![Default::default()]`).
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | LOW | The notes' "Mutation Targets" described mutants cargo-mutants does NOT generate (comparator→Equal, empty-query `if`, filter_map) — real output is whole-fn `FnValue` mutants only, so REQ-003's "catch the comparator mutant" rationale defended a non-existent mutant. Harmless (MSI 100) but would mislead validate. | REAL (doc accuracy) | REWORDED the notes' Mutation Targets (done) — REQ-003 reframed as a behavioral order-guard + coverage driver. |
  | F2 | NIT | `Scored` could derive `Copy` (two ints) for #57 ergonomics; keep `Default` OFF (a `Default` makes the `vec![Default::default()]` mutant viable for no gain). | Accept | Add `Copy` in Phase 4 (folded with the tests — the phase-gate hook gates crate-src edits to the active phase; validate edits lib.rs anyway). |
- **Verified (probe):** REQ-003 real nucleo-0.5.0 order = `[2,1]` (`main`@2 score 114 > `src/main.rs`@1
  score 109; `other` excluded) → fixture CORRECT. `fuzzy_score` case-insensitive both ways, no panics.
  **Palette behavior IDENTICAL** — 16-query ids+scores match old `field_score` (nucleo scoring is
  matcher-reuse-independent); `cargo nextest -p marley` 115 pass. **machete CLEAN** (nucleo removed from
  marley_app, sole owner now marley_search_core); clippy/fmt/rustdoc `-D warnings` clean. §20 trivial.
- **No code defect** — F1 doc (done), F2 nit (Phase 4).

## Phase 4 — Validate
- **F2 applied:** `Scored` now derives `Copy` (kept `Default` off).
- **Tests added** (`lib.rs`): `fuzzy_score_matches_case_insensitively` (REQ-001), `fuzzy_rank_empty_query_is_identity`
  (REQ-002), `fuzzy_rank_orders_by_score_and_excludes_non_matches` (REQ-003 — `[2,1]`),
  `fuzzy_rank_ties_break_by_index` (REQ-003b). Palette regression = the existing `-p marley` suite (REQ-004).
- **Runs (actual):** `cargo nextest run -p marley_search_core` → 4 passed; `-p marley` → 115 passed
  (palette unchanged).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **8 caught / 0
  missed → MSI 100.0%**, **machete PASS** (nucleo cleanly removed from marley_app). Receipt written.
- **UI:** N/A — library crate + the palette's PURE seam only (overlay render untouched).
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added`; new arch doc `marley_search_core.md`; crate-map node + row.
- **Knowledge:** aar-submit `completed` (5). Reused the #53 hollow-MSI rule (no new rule). Win: the
  pre-flight caught that palette already used nucleo + named the target crate, so the ticket became a
  clean EXTRACT+refactor (one matcher) instead of a redundant hand-rolled matcher; behavior preserved
  (16-query equivalence probe).
- **Ticket:** forge #54 → done; local doc → closed/; pair archived. **2/6 of M2.A.**
