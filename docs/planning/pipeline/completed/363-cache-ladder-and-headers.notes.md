# Route #329/#330 through the #349 tree cache — Notes

- **Forge ticket:** #363 `9a5cfb51-3a9e-4930-8a21-63fb53e76f0d`
- **AAR:** `d71ef05e-4670-45ab-99fb-4233ea8fea6e`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-363-cache-ladder-and-headers.md
- **Pipeline spec:** 363-cache-ladder-and-headers.spec.md
- **pipeline_id:** 788d1a9e-78ae-4f3d-9bec-cc51406c4f40
- **Live on:** `73394fe` (FOURTH of the goal /work 360,361,362,363,364)

## Phase 1 — Plan
- **Request:** route the #329 selection-ladder + #330 sticky-headers through the #349 tree cache (the deferred D6
  perf half) — a feature/performance follow-up.
- **Classification / tier:** work pipeline; editor/syntax/performance; a pure-factor ×2 + an app-reroute ×2 (a
  verbatim #349 transplant).
- **Forge recall (§18.3):** bulletins none. `aar-open` → `d71ef05e`. `knowledge-context` (Plan) logged 13
  surfacings — the #349 tree-cache AD (`85bc90e1`) + `PR-04452c92` (the exact-AND (nonce,version) guard) + the
  mock-clock pump PRs + the #329/#330 node-range ADs. The governing prior art is #349 itself (the factor + the
  hit-or-fallback).
- **★ Recon (on `73394fe`) — fully de-risked, all seams verified:**
  1. The #349 factor template: `matching_delimiters_in(src,pos)` (lib.rs:593) parses then delegates to
     `matching_delimiters_from(&tree,pos)` (:612). Mirror ×2.
  2. `enclosing_ranges(session, r)` (:488) = `session.tree.as_ref() else return Vec::new(); <walk :495-…>`;
     `all_headers(session)` (:685) = `session.tree.as_ref() else return; collect_headers(root, &mut out)`. Factor
     `_from(&Tree, …)` = the walk half; the originals become `map(_from).unwrap_or_default()`. `HighlightSession::
     tree()` (:361) is the pub accessor.
  3. The hit-or-fallback template `refresh_bracket_match` (app.rs:3678): `match &self.tree_cache { Some((cn,cv,tree))
     if *cn==nonce && *cv==version => matching_delimiters_from(tree,pos), _ => matching_delimiters_in(&text,pos) }`.
     `tree_cache: Option<(u64,BufferVersion,Tree)>` (:502), pump-populated (:1368).
  4. ★ The 2 callers are BOTH already `#[cfg_attr(test, mutants::skip)]`: `step_selection_ladder` (:3488, reroute
     :3530-3532, nonce/version :3495-3496) + `refresh_sticky_headers` (:3586, reroute :3615-3617, nonce/version
     :3593-3594). So the reroute inherits the skip (the #362 pattern) → NO #361 gate-red, NO new skip needed.
  5. Only 2 callers of each API (grep-confirmed, both app.rs); the `_from` factor keeps the `_in` originals →
     NOTHING ripples. The drive template = `t349_bracket_match_reads_the_cached_tree…` (:6650) + `t349_edit_
     invalidates…` (:6715) via `tick_pump` (the mock-clock pump — the cache is pump-populated; run_until_parked
     alone won't cache → a vacuous hit-drive; the #349/#334 lesson).
- **Decisions:** D1 factor `_from`, originals delegate (combinator, cov-100); D2 reroute mirrors
  refresh_bracket_match (hit=_from, miss=unchanged session-parse, exact-AND guard PR-04452c92); D3 no new mutation
  surface (both callers already skip); D4 the cache-hit drives use the mock-clock pump.
- **Prior art:** #349 factor + hit-or-fallback (the transplant); tree-sitter owns the walk (adoption). §20 N/A.
- **EARS:** REQ-LADDER-FROM-EQUIV, REQ-HEADERS-FROM-EQUIV, REQ-LADDER-HIT, REQ-HEADERS-HIT, REQ-MISS-FALLBACK,
  REQ-349-UNCHANGED (see spec).

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture / approach.** A verbatim #349 transplant ×2: two pure factors in `marley_syntax` (the cov/MSI-100
surface) + two app reroutes inside the already-`mutants::skip`'d caller fns. §14: no panic (the factors are total,
the walk is bounded, the fallback is the unchanged parse). §20 N/A (Marley's own tree cache; tree-sitter walk is
adoption). Confirmed.

**★ The 2 factors (marley_syntax lib.rs) — settled against the full code:**
- `enclosing_ranges` (:488-517) is `let Some(tree)=session.tree.as_ref() else{return Vec::new()}; let Some(mut
  node)=tree.root_node().named_descendant_for_byte_range(start,end) else{return Vec::new()}; <the dedupe-climb loop
  :505-516>`. Factor `pub fn enclosing_ranges_from(tree: &tree_sitter::Tree, byte_range: Range<usize>) ->
  Vec<Range<usize>>` = everything from the `named_descendant` line through `ladder` (:495-516). The delegate:
  `enclosing_ranges(session, r) = session.tree.as_ref().map(|t| enclosing_ranges_from(t, r)).unwrap_or_default()`.
  The `let Some(node)…else{return Vec::new()}` moves INTO `_from` (covered by `enclosing_ranges_reversed_range_is_empty`
  + the equivalence sweep); the delegate's None arm covered by `enclosing_ranges_no_tree_is_empty`.
- `all_headers` (:685-692) is `let Some(tree)=session.tree.as_ref() else{return Vec::new()}; let mut out=Vec::new();
  collect_headers(tree.root_node(), &mut out); out`. Factor `pub fn all_headers_from(tree: &tree_sitter::Tree) ->
  Vec<Range<usize>>` = the 3 lines; delegate `all_headers(session) = session.tree.as_ref().map(all_headers_from).
  unwrap_or_default()`.
- Plain-backtick docs on both `_from` (mirror `matching_delimiters_from`: "the parse-free half of
  [`enclosing_ranges`] — for a consumer holding a cached tree (M22 #349/#363)").

**★ The 2 app reroutes (app.rs, inside the `#[cfg_attr(test, mutants::skip)]` fns):**
- **step_selection_ladder (:3530-3532)** — `text`/`byte_range` are already locals (bound :3518-3522 before the
  call), `nonce`/`version` in scope (:3495-3496). Replace the 3 lines with:
  ```
  let byte_rungs = match &self.tree_cache {
      Some((cn, cv, tree)) if *cn == nonce && *cv == version => {
          marley_syntax::enclosing_ranges_from(tree, byte_range.clone())
      }
      _ => {
          let mut session = marley_syntax::HighlightSession::new(marley_syntax::Lang::Rust);
          session.highlight_full(&text);
          marley_syntax::enclosing_ranges(&session, byte_range)
      }
  };
  ```
  (`byte_range.clone()` in the hit arm, moved in the miss arm — `Range<usize>` isn't `Copy`; both arms lexically
  reference it, so the hit clones and the miss moves; `byte_range` unused after → compiles.)
- **refresh_sticky_headers (:3609-3633)** — ★ THE ONE RESTRUCTURE. Currently the `all_headers(&session)` call is
  INSIDE `self.active_editor().map(|s| {...})` (a `&self` borrow), so `self.tree_cache` can't be read there.
  Mirror `refresh_bracket_match`: compute the header BYTE RANGES via a match FIRST (the miss arm reads `text` via
  a nested `self.active_editor()` — a shared `&self`, OK alongside the `&self.tree_cache` match scrutinee, exactly
  as refresh_bracket_match does), THEN the row-mapping in a separate `active_editor().map` (the match borrow ended
  when `header_ranges` was bound):
  ```
  let header_ranges: Vec<std::ops::Range<usize>> = match &self.tree_cache {
      Some((cn, cv, tree)) if *cn == nonce && *cv == version => marley_syntax::all_headers_from(tree),
      _ => {
          let Some(text) = self.active_editor().map(|s| s.active_buffer().text()) else {
              return self.sticky_headers.take().is_some();
          };
          let mut session = marley_syntax::HighlightSession::new(marley_syntax::Lang::Rust);
          session.highlight_full(&text);
          marley_syntax::all_headers(&session)
      }
  };
  let Some(spans) = self.active_editor().map(|s| {
      let buffer = s.active_buffer();
      header_ranges.iter().map(|r| { <the unchanged row-mapping :3620-3628> }).collect::<Vec<(usize, usize)>>()
  }) else { return false };
  self.sticky_headers = Some((nonce, version, spans));
  true
  ```

**File manifest (3 files):**
- `crates/syntax/src/lib.rs` — `enclosing_ranges_from` + `all_headers_from` (+ the 2 delegations) + 2 equivalence
  units.
- `crates/marley_app/src/app.rs` — the 2 reroutes (inside the skipped fns).
- `crates/marley_app/src/headless_drive.rs` — the cache-hit + miss drives.

**★ Test plan.**
| # | Proves | Test |
|---|---|---|
| U1 | REQ-LADDER-FROM-EQUIV (cov/MSI 100) | `enclosing_ranges_from_equals_the_session_taking` — for each #329 fixture src (nested, caret-in-ident, whole-file, reversed-range, no-tree via an empty session): `enclosing_ranges_from(&parse(src), r) == enclosing_ranges(&session_of(src), r)`. |
| U2 | REQ-HEADERS-FROM-EQUIV (cov/MSI 100) | `all_headers_from_equals_the_session_taking` — for each #330 fixture (nested, free-code, deep-nesting): `all_headers_from(&parse(src)) == all_headers(&session_of(src))`. |
| T1 | REQ-LADDER-HIT | headless `ladder_reads_the_cached_tree_headless`: large fixture, poll-until-cached (mock-clock pump, mirror t349 :6650), caret in a nested expr, `expand-selection` → assert the selection grew (`selection_ladder_pos_for_test` advances / the selection widened). The HIT arm fires because the cache is populated. |
| T2 | REQ-HEADERS-HIT | headless `sticky_headers_read_the_cached_tree_headless`: poll-until-cached, `refresh_sticky_headers_for_test()` → `sticky_rows_for_test(row)` returns the enclosing header(s). |
| T3 | REQ-MISS-FALLBACK | headless `ladder_falls_back_on_a_cache_miss_headless`: cache the tree, then EDIT to bump the version WITHOUT re-pumping (tree_cache holds the OLD version → the exact-AND guard misses) → `expand-selection` still grows (via the session-parse). Mirror t349_edit_invalidates (:6715). |
| — | REQ-349-UNCHANGED | the existing `enclosing_ranges_*`/`all_headers_*` units + the `t349_*` drives + `matching_delimiters` tests stay green (the delegation is behavior-identical). |

- **Uncoverable-honestly:** the app reroute is inside the `mutants::skip`'d fns (no mutation surface — confirm via
  `cargo mutants --list` at validate) + app.rs cov-excluded; the pure `_from` factors (marley_syntax, cov-included)
  are the cov/MSI-100 surface via U1/U2. The drives (T1-T3) carry the wiring proof.

**Risks / decisions.** (a) ★ the sticky borrow un-nest (mirror refresh_bracket_match's match-then-map — settled
above). (b) the exact-AND `(nonce, version)` guard (PR-04452c92 — both fields). (c) the miss branch byte-identical
(the unchanged `HighlightSession::new + highlight_full + enclosing_ranges/all_headers`). (d) `byte_range.clone()`
in the ladder hit arm (Range isn't Copy). (e) the mock-clock pump for T1/T2 (`for 0..100` poll + advance_clock —
run_until_parked alone won't cache → vacuous; the #349/#334 lesson; this is the Shape-2 poll #364 will harden — reuse
as-is). (f) no other callers ripple (grep-confirmed, both app.rs). §14; §20 N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

**Built to the manifest — 3 files, all `cargo check`/`clippy -D warnings`/`fmt` clean (only the pre-existing
`block v0.1.6` future-incompat dep note; diff = exactly lib.rs + app.rs + headless_drive.rs).**

**Layer 1 — `crates/syntax/src/lib.rs` (the 2 pure factors + delegations + 2 equivalence units):**
- `enclosing_ranges_from(tree: &tree_sitter::Tree, byte_range: Range<usize>) -> Vec<Range<usize>>` = the walk
  (`named_descendant_for_byte_range` → the dedupe-climb ladder loop), extracted verbatim; `enclosing_ranges`
  delegates `session.tree.as_ref().map(|t| enclosing_ranges_from(t, byte_range)).unwrap_or_default()` (combinator,
  no early-return uncoverable region). The `let Some(node)…else{return Vec::new()}` moved INTO `_from`.
- `all_headers_from(tree: &tree_sitter::Tree) -> Vec<Range<usize>>` = `let mut out = Vec::new(); collect_headers(
  tree.root_node(), &mut out); out`; `all_headers` delegates `session.tree.as_ref().map(all_headers_from).
  unwrap_or_default()` (fn-item coercion).
- Plain-backtick docs on both `_from` (mirror `matching_delimiters_from`).
- 2 equivalence units after `all_headers_deep_nesting_does_not_overflow`: `t363_enclosing_ranges_from_equals_the_
  session_taking` (sweeps `"fn f(){ g(x); }"` @ 10..11/10..10/0..15, `"mod m { impl T { fn f(){} } }"` @ 17..25, +
  the reversed 5..2 degenerate — all `_from(&s.tree().unwrap(), r) == enclosing_ranges(&s, r)`) + `t363_all_
  headers_from_equals_the_session_taking` (`"mod m…"`, `"let x = 1;"`, `"trait T {…}"`). Reuse the existing
  `parsed(src)` helper + the pub `tree()` accessor. RAN: 12 passed.

**Layer 2 — `crates/marley_app/src/app.rs` (the 2 reroutes, inside the already-`#[cfg_attr(test, mutants::skip)]`
fns — no new mutation surface, D3):**
- `step_selection_ladder` (:3488): the `enclosing_ranges(&session, byte_range)` call replaced with the
  hit-or-fallback `match &self.tree_cache { Some((cn, cv, tree)) if *cn == nonce && *cv == version =>
  enclosing_ranges_from(tree, byte_range.clone()), _ => <the unchanged session-parse> }` (`byte_range.clone()` in
  the hit arm — `Range` isn't `Copy`; the miss arm moves it).
- `refresh_sticky_headers` (:3596): ★ THE BORROW UN-NEST (mirror `refresh_bracket_match`) — compute
  `header_ranges: Vec<Range<usize>>` via the `match &self.tree_cache { hit => all_headers_from(tree), miss => { let
  Some(text) = self.active_editor()…else return…; session-parse } }` FIRST (the tree-cache read OUT of the
  row-mapping's `active_editor()` borrow), THEN the unchanged row math in a separate `self.active_editor().map(|s|
  …)`. The 2 shared `&self` borrows (the match scrutinee + the nested `active_editor()` in the miss arm) compile,
  exactly as `refresh_bracket_match` does. The exact-AND `(nonce, version)` guard (PR-04452c92 — both fields).

**Layer 3 — `crates/marley_app/src/headless_drive.rs` (3 drives, mirror `t349_*`'s mock-clock poll):**
- `ladder_reads_the_cached_tree_on_a_hit_headless` (REQ-LADDER-HIT): a >1000-line fixture with a nested
  `(1 + 2)`, poll-until-cached (`cache_matches_live` — real sleep + advance_clock, the #349/#334 pump idiom), caret
  at the `1` byte (a bare caret → `editor_selection` None), `dispatch_for_test("expand-selection")` → the ladder is
  built on the HIT arm → `selection_ladder_pos_for_test().is_some()` + the selection widened (`e > s`).
- `sticky_headers_read_the_cached_tree_on_a_hit_headless` (REQ-HEADERS-HIT): a `fn wrapper` spanning >1000 rows,
  poll-until-cached, then ★ CLEAR the sticky cache via an OFF/ON toggle so the next refresh can only repopulate
  through the hit arm (the buffer version is unchanged, the tree is cached) — `refresh_sticky_headers_for_test()`
  returns true (repopulated via `all_headers_from`), `sticky_rows_for_test(500)` pins the fn header at row 0. The
  toggle-clear makes it NON-VACUOUS: without it the refresh would early-return on the already-cached version and
  the drive would assert nothing about the hit arm (the #362 vacuous-drive lesson applied preemptively).
- `ladder_falls_back_on_a_cache_miss_headless` (REQ-MISS-FALLBACK): poll-until-cached, then edit at the END (bump
  version to V+1 WITHOUT re-parking → the cache holds the stale V tree; assert `cache_ver != live_ver`), caret at
  the unchanged `1` byte, `expand-selection` → the guard MISSES → the session-parse fallback still grows the
  selection. Mirror `t349_edit_invalidates`.

**Deviations from design:** none material. The drives use the existing `editor_selection` helper (:437, returns
`Option<(usize,usize)>`, None = bare caret) for the widen assertion rather than a bespoke accessor; the sticky
drive adds the OFF/ON-toggle clear (a rigor improvement over the design's bare "refresh then assert" — makes the
hit arm provably the populating path). `dispatch_for_test("expand-selection")` → `grow_selection` →
`step_selection_ladder(true)` is `&mut self`-only (no `cx`), so it runs inside a plain `window.update`.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 critics (Agent general-purpose), parallel — scaled to a small verbatim-transplant change. Both READ-ONLY +
`cargo check`/`cargo mutants --list` only (HARD RULE: never git checkout/stash/restore/reset). Lenses: Critic 1 =
the pure factors + equivalence units (marley_syntax); Critic 2 = the app reroutes + the 3 drives (correctness /
borrow / non-vacuous / no-new-mutants / provenance).**

### Findings ledger

**F1 [LOW → FIXED] the t363 equivalence units are TAUTOLOGICAL for the factor-internal walk mutants** (Critic 1) —
crates/syntax/src/lib.rs:1817/1844. **REAL.** Because both `enclosing_ranges` and `all_headers` now DELEGATE to
`enclosing_ranges_from`/`all_headers_from`, an equivalence assert `_from(tree, r) == wrapper(&session, r)` routes
BOTH operands through `_from` — so a mutant INSIDE `_from` (the ladder body, the `ladder.last() != Some(&r)` dedup,
the `collect_headers` descent) moves both sides equally and the equality still holds → the mutant SURVIVES *as far
as t363 is concerned*. MSI is genuinely 100 (Critic 1 traced each factor-walk mutant to a killer), but the killers
are the RETAINED #329/#330 hardcoded-vector tests (`enclosing_ranges_ladder_on_fixture` etc.) reached THROUGH the
delegation — NOT t363. So the ticket's "the equivalence units are the cov/MSI-100 surface for `_from`" was
misattributed, and a latent hazard existed: deleting the #329/#330 tests as "redundant now that t363 proves
equivalence" would silently collapse the factor's MSI to ~0. **FIX (source, §0 — no gate weakened):** added an
INDEPENDENT hardcoded-vector assertion to each t363 unit — `enclosing_ranges_from(tree, 10..11) == vec![10..11,
9..12, 8..12, 8..13, 6..15, 0..15]` (the deduped root pair exercises the `!=` false branch) + the reversed-range →
`Vec::new()`, and `all_headers_from(tree) == vec![0..29, 8..27, 17..25]`. These call `_from` DIRECTLY (not via the
wrapper), so a factor-internal mutant now fails the literal → t363 independently kills the walk mutants; the
equivalence sweep is retained as the DELEGATION contract (kills the wrapper-body mutants). 12 units green. Critic 1
noted #349 (`matching_delimiters_in`→`_from`) carries the SAME latent tautology — recorded as a prevention rule.

**F2 [LOW → COMMENT REFINED] the 3 drives are integration/wiring proofs, not arm-discriminators** (Critic 2) —
crates/marley_app/src/headless_drive.rs (the three #363 drives). **REAL but by-design + accepted precedent.**
Because `_from == wrapper` by construction (F1's equivalence), the hit and miss arms produce the IDENTICAL ladder/
headers — so no drive can behaviorally distinguish "read the cached tree" from "reparsed" by observing the result;
all three would pass unchanged against pre-#363 code, and T3 would pass even against a version-ignoring guard (the
end-append is a semantic no-op for the `(1 + 2)` region). This EXACTLY mirrors the accepted #349 T4/`t349_edit_
invalidates` pattern: arm-selection is proven "by construction" from the `cache_matches_live` precondition (the
guard condition IS the polled key match) + the reroute being present, and arm-CORRECTNESS is carried by the t363
equivalence units + the exact-AND guard's `mutants::skip`. **FIX:** none to the tests (they correctly prove the
REQ — the wiring works end-to-end on a confirmed-hit / confirmed-stale cache); REFINED T3's comment which
overclaimed "proves a stale tree is never read" → now states it fixes the MISS arm via the asserted-stale guard,
not a divergent value, and points at the equivalence units for arm-correctness. (A genuinely arm-discriminating
miss test — a stale tree whose region the edit DOES perturb — is possible but adds real complexity for a
`mutants::skip`'d shim that mirrors accepted precedent; not pursued. Noted for a future hardening if ever wanted.)

**F3 [LOW/nit → REJECTED] the hit-arm `byte_range.clone()` is avoidable** (Critic 2) — app.rs:3535. Both match arms
are mutually exclusive, so the hit arm COULD move `byte_range` like the miss arm. **REJECTED as a non-issue:** the
clone is two `usize`s once per deliberate ⌃W gesture (never a hot path), and cloning in the hit arm keeps the miss
arm's `enclosing_ranges(&session, byte_range)` move lexically clean — arguably clearer than a borrow dance. Critic
2 itself rated it "not worth changing." Kept.

### Verified CLEAN (no change)
- **Behavior-identical factor** (Critic 1 Lens 1): `enclosing_ranges_from` is the byte-for-byte original walk (the
  no-node/degenerate arm moved in intact); the delegations preserve the None-tree → `Vec::new()` flow.
- **Cov-clean combinator** (Critic 1 Lens 3): `.map(...).unwrap_or_default()` — both Option arms live (fresh
  session None / parsed Some), no `node_kind_at`-style uncoverable region; the dedupe FALSE branch is exercised.
- **Docs + provenance** (Critic 1 Lens 4): valid shortcut intra-doc links, no `unwrap`/`expect`/panic on a
  reachable path, tree-sitter walk = adoption (permissive), no AGPL/Zed/Warp source.
- **Exact-AND `(nonce, version)` guard, both fields** (Critic 2 Lens 1): both reroutes `*cn == nonce && *cv ==
  version`, sourced from the same `active_nonce()`/`active_buffer().version()` the cache is keyed by; identical in
  shape to `refresh_bracket_match` (PR-04452c92).
- **Miss branch byte-identical** (Critic 2 Lens 2): the unchanged `HighlightSession::new + highlight_full +
  enclosing_ranges/all_headers`; `byte_range.clone()` in hit / moved in miss, no use-after-move.
- **Sticky borrow un-nest SOUND** (Critic 2 Lens 3): the match yields an OWNED `Vec`, so the `&self.tree_cache`
  borrow ends (NLL) before the row-mapping's fresh `active_editor()` borrow; the version-gate early-return + the
  final `self.sticky_headers = Some(...)` preserved; mirrors `refresh_bracket_match`.
- **No new mutants** (Critic 2 Lens 5): `cargo mutants --list -f app.rs` → ZERO mutants in `step_selection_ladder`
  / `refresh_sticky_headers` (both `#[cfg_attr(test, mutants::skip)]` → the reroute inherits the skip — the #362
  case, not the #361 case). Re-confirm at the Phase-4 gate.
- **Verb + accessors** (Critic 2 Lens 6): `dispatch_for_test("expand-selection")` → `grow_selection` →
  `step_selection_ladder(true)`; all `_for_test` accessors exist with the called signatures.
- **Completeness**: the only `enclosing_ranges`/`all_headers`/`HighlightSession::new` consumers in app.rs are the
  three tree_cache readers (bracket-match #349 + ladder #363 + sticky #363, all rerouted) + the worker producer
  parse (which legitimately parses — it FEEDS the cache). No fourth consumer left un-rerouted.

**Post-fix checks:** `cargo nextest -p marley_syntax -E 'test(t363)+test(enclosing_ranges)+test(all_headers)'` → 12
passed; `cargo fmt` clean; `cargo check -p marley_syntax -p marley --all-targets` clean (only the pre-existing
`block v0.1.6` dep note). Diff = the same 3 source files.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**The tests were written in Phase 3 + the F1 inspect fix; Phase 4 RAN them + the full gate. All green.**

**1. The 2 equivalence units + the 3 headless drives** (`cargo nextest -p marley_syntax -p marley -E 'test(t363) +
test(ladder_reads_the_cached_tree) + test(sticky_headers_read_the_cached_tree) + test(ladder_falls_back_on_a_cache_
miss)'`): **5 passed, 824 skipped.**
- `t363_enclosing_ranges_from_equals_the_session_taking` ✓ (the F1 hardcoded `_from(tree,10..11)==vec![10..11,
  9..12,8..12,8..13,6..15,0..15]` + reversed→`Vec::new()` + the equivalence sweep)
- `t363_all_headers_from_equals_the_session_taking` ✓ (hardcoded `all_headers_from(tree)==vec![0..29,8..27,17..25]`
  + the equivalence sweep)
- `ladder_reads_the_cached_tree_on_a_hit_headless` ✓ (REQ-LADDER-HIT — mock-clock pump, hit arm, selection grew)
- `sticky_headers_read_the_cached_tree_on_a_hit_headless` ✓ (REQ-HEADERS-HIT — OFF/ON toggle-clear then hit-arm
  repopulate, row 500 pins the fn header)
- `ladder_falls_back_on_a_cache_miss_headless` ✓ (REQ-MISS-FALLBACK — end-edit → asserted-stale cache → the
  session-parse fallback grows)
  The 3 drives ran in ~0.2s each (the mock-clock pump caches quickly on these fixtures).

**2. Regression** (`cargo nextest -p marley_syntax -p marley_editor -p marley`): **1069 passed, 2 skipped, 0
failed.** The #329 ladder + #330 sticky + #349 bracket-match/`matching_delimiters`/`t349_from_equals_in` tests all
stay green — the delegation is behavior-identical. (The known `search_open…` flake did not even fire this run.)

**3. NO LIVE SYNTHETIC DRIVE — stated.** #363 is a PURE performance reroute (read the #349 cached tree on a hit
instead of reparsing a throwaway `HighlightSession`); the OUTPUT is byte-identical (proven by the equivalence
units), so the rendered selection-ladder + sticky-headers are visually unchanged from pre-#363 — there is no visual
delta to capture. The 3 headless drives ALREADY exercise the REAL editor door → the real reroute (`dispatch_for_
test("expand-selection")` → `grow_selection` → `step_selection_ladder` reading `self.tree_cache`; `refresh_sticky_
headers_for_test` reading `self.tree_cache`) on a confirmed-hit / confirmed-stale cache. A live `drive.swift` run
would render the identical ladder/headers AND is off-limits (chad may be at the machine → live synthetic input
hits HIS frontmost window). The headless drives are the complete proof.

**4. THE FULL `--diff` GATE → GATE GREEN [diff], 15/15, first run.** Receipt
`13dd70c980d3c243a593ac4ebee7a08561ac3a97`. All 15 PASS incl.:
- **gate:4 coverage ≥ 100%** — `enclosing_ranges_from` + `all_headers_from` (marley_syntax, cov-INCLUDED) fully
  exercised by the hardcoded + equivalence units + the retained #329/#330 tests; app.rs reroutes cov-excluded.
- **gate:5 mutation MSI 100%** — the factor walk mutants are killed by the F1 HARDCODED literal asserts (the
  tautology is broken — t363 now independently kills them); the 2 app reroutes enumerate ZERO mutants (inside the
  already-`#[cfg_attr(test, mutants::skip)]` `step_selection_ladder`/`refresh_sticky_headers` — the #362
  inherited-skip case, NOT #361). No survivor, no new skip needed.
- **gate:14 docs** — the plain-backtick `_from` rustdoc + the shortcut intra-doc links resolve.
- **gate:15 visual/AX** — 158 harness units green (no visual change; the reroute output is byte-identical).

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21):** CHANGELOG.md `### Changed` entry (the #329/#330-now-cached reroute, the two `_from` factors, the
tautology-avoiding hardcoded units) + docs/marley_architecture/editor.md — a `**M22 #363**` note appended to the
#349 tree-cache section (the factor+hit-or-fallback ×2, the sticky borrow un-nest, the three cache readers are now
`refresh_bracket_match`/`step_selection_ladder`/`refresh_sticky_headers`, the tautology lesson). Both DOCS-only →
no receipt stale (source unchanged since receipt `13dd70c9`).

**Capture (forge wired):** aar-submit `d71ef05e` — outcome completed, effectiveness 4 (13 verdicts written, 1
novel finding, distillation/confidence-drift/pattern-emergence enqueued). HEADLINE lessons:
- ★★ The #349 transplant held ×2 — a verbatim factor + hit-or-fallback reuse; the recon named BOTH callers already
  `mutants::skip`'d, so D3 (no new mutation surface) was known BEFORE code → no #361 gate-red.
- ★★ The inspect critic caught a REAL tautology (F1): a `_from == wrapper` equivalence test is tautological for
  the factor's OWN mutants (both operands route through `_from`) → those were killed only by the retained #329/#330
  hardcoded tests, not t363; MSI stayed 100 but the credit + deletion-safety were wrong. FIX: an INDEPENDENT
  hardcoded-vector assert per t363 unit. Latent in #349 (`matching_delimiters_in`→`_from`) too. →
  `PR-claude-delegation-equivalence-test-tautological-001` (`d5e67e83`, recorded at inspect).
- ★ The sticky borrow un-nest (the `&self.tree_cache` read OUT of the `active_editor().map` closure; the match's
  owned `Vec` releases the borrow before the row-map's fresh `&self`) — settled at design, compiled first try.
- The mock-clock pump drives reused as-is (real sleep + advance_clock, poll-until-`cache_matches_live`); the sticky
  drive added an OFF/ON toggle-clear so the hit arm is provably the populating path (#362 vacuous-drive lesson
  applied preemptively).
- F2: hit and miss are equal BY DESIGN, so no drive can behaviorally discriminate the arms — they are integration/
  wiring proofs (arm-selection by-construction from the `cache_matches_live` precondition), mirroring the accepted
  #349 T4/T5. T3's comment was refined to match.

**failure-record:** NONE — F1 was caught + fixed in-phase (MSI never went red); the vacuous-drive risk was
pre-empted. No shipped defect. **architecture-decision-record:** none — the #349 tree-cache AD (`85bc90e1`) already
covers the pattern; #363 is an application, not a new decision. **follow-up ticket:** none (#363 spawned none; #364
is the next in-goal ticket).

**Close + archive:** forge #363 closed (`ticket-close` → done); TICKET-363 → `tickets/closed/` (status: closed);
the pipeline pair → `pipeline/completed/`; spec Phase 5 PASS.

**Status: Phase 5 — Complete PASS.**
