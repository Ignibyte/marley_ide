# App-side cached tree-sitter tree — Notes

- **Forge ticket:** #349 `18213b21-9822-44bf-aa93-3218db3a7501`
- **AAR:** `847b3f25-99a7-4e82-8158-79f970b6b724`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-349-cached-tree-sitter-tree.md
- **Pipeline spec:** 349-cached-tree-sitter-tree.spec.md
- **pipeline_id:** 94aa473e-c3d3-4233-b845-a623e175658a · on `312b643`

## Phase 1 — Plan

**Request:** cache the tree-sitter tree app-side so the per-caret bracket-match (#340) node query is microseconds,
not a throwaway reparse. #340 follow-up, perf. CONDITIONAL — "do this only if profiling wants it."

**Classification / tier:** work pipeline, one shippable slice (the cache + the worker handoff + bracket-match
reading it; #329/#330 reroute is the in-scope shared win, trimmable to a follow-up if it balloons).

**Forge recall (§18.3):** no bulletins. `knowledge-context` (Plan) surfaced 13 nodes (3 ADRs incl. the #340/#268
syntax decisions, 3 distilled lessons, 5 prevention-rules incl. `ba51b76d` the tree-answers-directly rule I
recorded on #346, 2 failures). Logged to the AAR.

### ★ THE PROFILING SPIKE — D1 = BUILD (the decisive recon)

A release (`--release`, opt-level 3) best-of-N micro-bench of the #340 parse-only path
`marley_syntax::matching_delimiters_in(src, byte_pos)` (which reparses `src` from scratch each call — the exact
per-caret cost #349 targets). Fixture = `crates/editor/src/find.rs` (910 lines, real Rust) repeated to size. Scratch
bench crate in scratchpad (path-deps marley_syntax); NOT committed; repo tree confirmed untouched.

| lines | KB | best ms | median ms | vs 16.7ms/60fps frame |
|-------|----|---------|-----------|------------------------|
| 1,822 | 87 | 3.24 | 3.37 | ✓ under (matches the ticket's ~4ms/2k baseline) |
| 4,555 | 218 | 8.35 | 8.60 | ~half a frame |
| 10,021 | 479 | **18.64** | **19.10** | **✗ OVER frame** |

**Verdict: BUILD.** At 10k lines a memo-missing per-caret reparse (19ms) exceeds a 60fps frame → dropped frames on a
held arrow over a large file. #340's `(nonce, version, caret)` memo absorbs a STATIONARY caret (a hit is free) but
NOT rapid motion (each distinct position misses → reparses). The ticket's own bar ("realistic 5k-10k line file shows
jank") is met. This is NOT a #339-style dissolve — the measurement affirmatively says build. (Had 10k stayed
sub-frame, D1 would be MEASURED-DEFER.)

### Recon (the BUILD path — verified live on `312b643`)

- **`tree_sitter::Tree: Send` — CONFIRMED:** `unsafe impl Send for Tree {}` (tree-sitter 0.26.11
  binding_rust/lib.rs:3908) + `Sync` (:3909). The tree can cross the worker→app mpsc channel → Option A type-viable.
- **`tree_sitter::Tree: Clone` — CONFIRMED:** `impl Clone for Tree` (lib.rs:1521) = a cheap `ts_tree_copy` refcount
  bump (NOT a deep reparse). The worker clones its `session` tree into the resp while keeping its own for the next
  incremental parse.
- **The worker boundary:** the syntax worker (app.rs:13311) calls `session.highlight_full(&req.text)` (:13339) —
  which PARSES the tree AND builds spans — then sends `SyntaxResp{generation, nonce, version, lines}` (:13383),
  the spans only, never the tree. `SyntaxResp` ALREADY carries `nonce` + `version` (the cache key is free).
- **The cache pattern (mirror, don't invent — D5):** `syntax_cache: Option<(u64, BufferVersion, SyntaxLines)>`
  (app.rs:497) is the EXACT shape for the tree cache; `sticky_headers` (app.rs:305) is a second `(nonce, version)`
  instance. Refreshed in the render/pump path.
- **The consumers:** `refresh_bracket_match` (app.rs:3602) calls the pure `matching_delimiters_in` (the reparse),
  keyed `bracket_match_key: (u64, BufferVersion, usize)` (:315). #329 selection-ladder + #330 sticky-headers
  similarly reparse a throwaway session (the shared win reroutes them too).
- **The query factor (D3):** `matching_delimiters_in` (marley_syntax lib.rs:556) = `Parser::new → parse(src) →
  root → (before-then-at) delims_at`. The reusable half is `root → delims_at` → a new pub
  `matching_delimiters_from(&Tree, byte_pos)`; `matching_delimiters_in` = `parse + matching_delimiters_from`. The
  tree is PRIVATE (`HighlightSession.tree`); surface via a `pub fn tree(&self) -> Option<&Tree>` accessor (P2 picks
  accessor vs `highlight_full` co-return).

**Decisions recorded:** D1-BUILD (spike evidence), D2-OPTION-A (Tree Send+Clone; worker returns its tree),
D3-FACTOR-THE-QUERY, D4-HIT-OR-FALLBACK (the invalidation correctness — a stale tree is silent-wrong; exact
(nonce,version) key + always-correct sync fallback), D5-MIRROR-NOT-INVENT. See the spec.

**The load-bearing risk (for inspect):** a STALE cached tree = silently-wrong bracket highlights with no crash (the
class the #339 `char_at` debug_assert guards). The (nonce,version) key must be exact and the fallback always correct
— which it is, because `matching_delimiters_in` (the fallback) is the current shipped behavior.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Approach.** Option A (D2): the syntax worker already parses the tree for spans; it now clones that tree
(`Tree: Clone` = a cheap `ts_tree_copy` refcount bump) into `SyntaxResp`, which crosses the mpsc (`Tree: Send`).
The app caches `Option<(u64, BufferVersion, Tree)>` mirroring `syntax_cache` (D5), and `refresh_bracket_match`
reads it on an exact `(nonce, version)` hit (µs, no reparse, no `text` materialization) or falls back to the
shipped sync `matching_delimiters_in` on any miss (D4). §14 holds (the query is total, `None` on parse-fail);
§20 = N/A (Marley's own worker→app seam; tree-sitter is adoption, confined to marley_syntax).

**★ D2b — the crate boundary (a recon-driven refinement).** `marley_app` does NOT dep `tree_sitter` and never
names `tree_sitter::` (it confines tree-sitter to marley_syntax, exactly as `regex` is confined to find.rs). So
naming `Option<tree_sitter::Tree>` in `SyntaxResp` would force a NEW cross-boundary dep. Instead, marley_syntax
**re-exports** `pub use tree_sitter::Tree;` and the app names `marley_syntax::Tree` — an OPAQUE handle it only
ever passes back to `marley_syntax::matching_delimiters_from`. No new dep; the tree stays in the syntax crate's
vocabulary.

**★ D6-SCOPE — ship the cache + bracket-match; DEFER the #329/#330 reroute (a bounded follow-up).** The spike
measured the PER-CARET bracket-match reparse (#340) — the hot path, the one at 19ms/10k lines. #329
selection-ladder (`step_selection_ladder`, app.rs:3426 → `enclosing_ranges`) fires only on ⌥↑/⌃W (a deliberate,
infrequent gesture); #330 sticky-headers (`refresh_sticky_headers` app.rs:3547 → `HighlightSession::new` +
`highlight_full` + `all_headers(&session)`) fires on scroll and is ALREADY `(nonce,version)`-gated (a static
caret / bare scroll costs nothing). Neither is the per-caret hot path, and each reroute needs its OWN tree-taking
factor (`enclosing_ranges_from(&Tree)` / `all_headers_from(&Tree)`) + hit-or-fallback site. Ship the measured win
first; file a follow-up for the two.

**The pieces (exact):**
1. **`HighlightSession::tree` accessor (marley_syntax):** `pub fn tree(&self) -> Option<&tree_sitter::Tree> {
   self.tree.as_ref() }`. The field is `tree: Option<tree_sitter::Tree>` (lib.rs:327), `None` on a fresh session
   (pre-parse), `Some` after `highlight_full`/`highlight_incremental`.
2. **Re-export (marley_syntax):** `pub use tree_sitter::Tree;` (D2b) so the app names `marley_syntax::Tree`.
3. **`matching_delimiters_from` (marley_syntax, pure — D3):**
   `pub fn matching_delimiters_from(tree: &tree_sitter::Tree, byte_pos: usize) -> Option<(Range<usize>,
   Range<usize>)> { let root = tree.root_node(); byte_pos.checked_sub(1).and_then(|before| delims_at(&root,
   before)).or_else(|| delims_at(&root, byte_pos)) }`. `matching_delimiters_in` becomes `…parser setup…; let tree
   = parser.parse(src, None)?; matching_delimiters_from(&tree, byte_pos)` — BEHAVIOR-IDENTICAL (the #340 tests
   stay green; the doc's "#349 is the follow-up" line updates at P5).
4. **`SyntaxResp.tree: Option<marley_syntax::Tree>` (app.rs:13383):** the worker sets `tree:
   session.tree().cloned()` in the `.send(SyntaxResp{…})` (app.rs:13343) — after the highlight, so the tree is the
   one just parsed. The clone is a refcount bump; the worker keeps its own tree for the next incremental parse.
5. **`tree_cache: Option<(u64, marley_editor::BufferVersion, marley_syntax::Tree)>` (app.rs, near syntax_cache:497)
   + init `tree_cache: None` (near :2031).** The pump (app.rs:1356, in the SAME accept-gate that stores
   `syntax_cache`) adds `if let Some(tree) = resp.tree { view.tree_cache = Some((resp.nonce, resp.version, tree));
   }` (`resp.tree` is a distinct field, accessible after `resp.lines` moves).
6. **`refresh_bracket_match` hit-or-fallback (app.rs:3610, mutants::skip — D4).** Replace the single reparse
   (:3635-3639) so `text` is fetched ONLY on the miss path (the hit path is text-free — the real win):
   ```
   let pair = match &self.tree_cache {
       Some((cn, cv, tree)) if *cn == nonce && *cv == version =>
           marley_syntax::matching_delimiters_from(tree, caret_byte),   // µs — cache hit
       _ => {                                                            // fallback: no/stale cache, small file
           let Some(text) = self.active_editor().map(|s| s.active_buffer().text()) else { return false; };
           marley_syntax::matching_delimiters_in(&text, caret_byte)     // shipped behavior, always correct
       }
   };
   self.bracket_match = pair;
   self.bracket_match_key = Some((nonce, version, caret_byte));
   true
   ```
   The `bracket_match_key` memo (:3632) still short-circuits a stationary caret BEFORE any query, so a static
   caret is a no-op either way. A stale tree cannot be read: the arm guard is exact (`*cn == nonce && *cv ==
   version`); every other case takes the byte-identical sync fallback.

**Invalidation walk (D4, the load-bearing correctness):**
- Caret moves, no edit → version stable, nonce stable → `tree_cache` matches → HIT (µs). ← the win.
- Edit bumps version → `tree_cache` holds `(nonce, OLD_version)` ≠ current → MISS → sync reparse of the edited
  text (correct) until the worker re-sends with `NEW_version` and the pump refreshes `tree_cache`.
- Small file (no worker; sync span path) → `tree_cache` never populated → always MISS → sync reparse (correct;
  small files parse <1ms, sub-frame). The win is for LARGE files, where the worker runs AND the reparse is
  expensive — exactly where the cache is populated.
- Non-Rust / no editor → the existing gates (:3624/:3619) drop the cache before the query is reached.

**File manifest:**
| File | Change |
|---|---|
| `crates/syntax/src/lib.rs` | ADD `pub use tree_sitter::Tree;` (re-export); ADD `HighlightSession::tree()` accessor; ADD pub `matching_delimiters_from(&Tree, pos)` + refactor `matching_delimiters_in` to delegate. (+ P4 tests.) |
| `crates/marley_app/src/app.rs` | `SyntaxResp.tree` field + the worker's `session.tree().cloned()`; `tree_cache` field + init; the pump store; `refresh_bracket_match` hit-or-fallback. All app plumbing = `mutants::skip`/cov-excluded. |

**Regression Test Plan:**
| # | Test | Kind / home | Proves |
|---|------|-------------|--------|
| T1 | `matching_delimiters_from` equivalence: for a fixture with nested `()[]{}`, sweep every byte position and assert `matching_delimiters_from(&parse(src), p) == matching_delimiters_in(src, p)` | pure unit, marley_syntax | REQ-CORRECTNESS + cov/MSI 100 on the factored fn |
| T2 | the existing #340 `matching_delimiters_in` tests | pure unit, marley_syntax (existing) | REQ-FACTOR-IDENTICAL (stay green) |
| T3 | `HighlightSession::tree()`: `None` on a fresh `new(Rust)`; `Some` after `highlight_full`; the returned tree's `root_node().kind()` is `source_file` | pure unit, marley_syntax | the accessor (cov/MSI 100) |
| T4 | ★ headless cache-HIT: open a LARGE Rust file, let the worker respond (`run_until_parked`), move the caret onto a `(`, assert the pair; a parse-count probe (a `#[cfg(test)]` AtomicUsize in `matching_delimiters_in`, or an instrumented seam) shows the reparse counter did NOT increment on the cache-hit query | headless drive, app | REQ-CACHE-HIT |
| T5 | ★ headless INVALIDATE: after a HIT, type a char (bumps version), immediately move the caret — the pair is CORRECT for the edited text (the sync fallback ran; not the stale tree) | headless drive, app | REQ-CACHE-INVALIDATE |

The pure `matching_delimiters_from` + `tree()` = cov/MSI 100 in marley_syntax. The app plumbing (SyntaxResp field,
tree_cache, pump store, refresh_bracket_match) is `mutants::skip`/cov-excluded → the T4/T5 headless drives carry
it. **NO live synthetic drive** (headless = cargo test, safe). ⚠️ the parse-count probe (T4) may need a small
`#[cfg(test)]` instrumentation hook in marley_syntax (an AtomicUsize incremented in `matching_delimiters_in`, read
by the drive) — settle the exact mechanism at P4 (a counter is cleanest; alternatively assert the pair is correct
+ trust the code path, but the ticket wants proof of NO reparse).

**Risks / decisions:** (a) ★ stale tree = silent wrong-highlight — D4's exact key + always-correct fallback; a
debug-assert is NOT needed (the fallback is correct, not just loud) but the arm guard must be exact. (b) worker
clone = refcount bump (`ts_tree_copy`), ~free. (c) #329/#330 reroute DEFERRED (D6 — a follow-up). (d) the app
names `marley_syntax::Tree` (D2b re-export), no tree_sitter dep. (e) small-file sync path never populates
tree_cache → always the correct fallback (documented, not a bug). §14 honored; §20 = N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest, 2 files. `cargo fmt --all`; `cargo check --workspace` CLEAN; `cargo clippy -p marley_syntax
-p marley --all-targets -D warnings` rc 0 (only the pre-existing `block v0.1.6` future-incompat warning). Diff =
exactly syntax/lib.rs + app.rs.

- **crates/syntax/src/lib.rs** — ADDED `pub use tree_sitter::Tree;` (D2b re-export, with the other pub re-exports
  at the crate head, doc'd); ADDED `HighlightSession::tree(&self) -> Option<&tree_sitter::Tree>` (after `new`);
  ADDED `pub fn matching_delimiters_from(&Tree, byte_pos)` (the `root → delims_at` before-then-at half) and
  REFACTORED `matching_delimiters_in` to `…parser setup…; let tree = parser.parse(src, None)?;
  matching_delimiters_from(&tree, byte_pos)` — behavior-identical. Updated `matching_delimiters_in`'s doc (its
  "#349 is the follow-up" line now says #349 shipped + this fn is the cache-miss fallback).
- **crates/marley_app/src/app.rs** — `SyntaxResp` gained `tree: Option<marley_syntax::Tree>`; the worker's
  `.send(SyntaxResp{…})` sets `tree: session.tree().cloned()` (after the highlight); ADDED `tree_cache:
  Option<(u64, BufferVersion, marley_syntax::Tree)>` field (after syntax_cache) + init `tree_cache: None`; the pump
  binds `let resp_tree = resp.tree;` before the `resp.lines` move and stores `if let Some(tree) = resp_tree {
  view.tree_cache = Some((resp.nonce, resp.version, tree)); }` right after `syntax_cache`; `refresh_bracket_match`
  now matches `&self.tree_cache` — an exact `(nonce, version)` hit calls `matching_delimiters_from(tree,
  caret_byte)` (no `text` fetch), any miss fetches `text` + falls back to `matching_delimiters_in`.

**Deviations from design:** none. The pump binds `resp_tree` before the `lines` move (the design's belt-and-
suspenders option — `resp.tree` is a distinct field but the explicit bind is clearest). The `marley_syntax::Tree`
re-export (D2b) avoided adding a `tree_sitter` dep to marley_app (confirmed: no new Cargo.toml dep). The app plumbing
(the resp field, the worker clone, the cache field/pump, refresh_bracket_match) all sits in `mutants::skip`/cov-
excluded app.rs; the pure `matching_delimiters_from` + `tree()` accessor carry the marley_syntax cov/MSI 100
obligation (Phase 4).

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 independent critics** (general-purpose), parallel, distinct lenses — scaled to the cross-thread change. Both
verified the WORKING TREE untouched (only the 2 `.rs` + docs). `cargo check --workspace` exit 0.

- **Critic 1 — invalidation correctness + behavior-preservation:** CLEAN on all of (a) stale-tree, (b) pump key,
  (c) Code==old factor, (d) clone timing. No reachable stale-tree read.
- **Critic 2 — Send/thread-safety + crate boundary + purity/docs + no-double-parse:** CLEAN on all of (a)–(e),
  verified at the tree-sitter C level.

### Findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| ★ | (insight) | **The exact-AND guard `*cn == nonce && *cv == version` is NOT belt-and-suspenders — BOTH halves are load-bearing.** `reload_active` (editor_surface.rs) RESETS `version` to `initial()` (0) AND re-mints `nonce`, so version is NOT globally monotonic across a reload. A version-only guard would false-HIT a just-reloaded v0 buffer against a cached `(nonceB, 0, treeB)` (0==0) → stale read; a nonce-only guard would false-HIT the pre-edit tree after an edit (nonce stable, version bumped). | **CONFIRMED CORRECT (Critic 1).** The implemented guard is exactly right; the critic traced both bug variants it prevents. Recorded as `PR-claude-cache-key-both-fields-load-bearing-when-a-reset-breaks-monotonicity-001`. No code change — the guard is already exact-AND. |
| F1 | LOW | `SyntaxResp.tree` doc's parenthetical "it always does after a highlight" slightly OVERCLAIMS — a parse failure yields None post-highlight too (the "a doc must not claim what the code doesn't keep" class, #337 F5). | **FIXED at source.** Rewrote the doc: "`None` when the session holds no tree — a parse failure, which the pump treats as no cache update". Accurate now. `cargo check -p marley` clean. |
| F2 | LOW | `tree_cache` not proactively cleared on nav-to-terminal / non-Rust (only `bracket_match`/`bracket_match_key` drop). | **ACCEPT — mirrors `syntax_cache`'s own lifecycle (D5); safe.** At most one refcounted tree held, overwritten on the next worker resp, and the exact-AND guard prevents any wrong read on return. Clearing it would DIVERGE from the established pattern. |
| — | (verified) | `Tree::clone` = `ts_tree_copy` (one atomic refcount bump + a tiny struct alloc, NOT a reparse/deep-copy) — the perf win does not migrate to the worker. Concurrent app-read vs worker-reparse is SOUND (atomic refcounts + copy-on-write edits; the worker's later `tree.edit()` can't mutate a subtree the app's clone sees). | **CLEARED (Critic 2, C-level).** |
| — | (verified) | No `tree_sitter` dep added to marley_app (the `marley_syntax::Tree` re-export); the app names the tree opaquely, never calls a tree-sitter method; docs link only pub items (no `[link]` to the private `HighlightSession.tree` field); one parse (the tree is the SAME `highlight_full`/incremental output, cloned); tree_cache mirrors syntax_cache in the same accept-gate. | **CLEARED (Critic 2).** |

**Result: 0 CRITICAL/HIGH/MEDIUM. 1 LOW fixed (the doc overclaim), 1 LOW accepted (mirrors syntax_cache), and a
KEY INSIGHT confirming the guard's exact-AND is load-bearing (both halves) — recorded as a prevention rule.** The
ticket's sole load-bearing risk (a silent stale-tree wrong-highlight) is genuinely closed: exact guard, both
fields provably necessary (the reload path), byte-identical fallback, consistent pump key. Lenses: stale-tree,
pump-key, Code==old, clone-timing, Send/thread-safety, refcount-clone, crate-boundary, purity/docs, no-double-parse.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests written** (T1/T3 + T4/T5 + 1 `#[cfg(test)]` hook; T2 = the existing #340 tests stay green):

| # | Test | File | Proves |
|---|------|------|--------|
| T1 | `t349_from_equals_in_at_every_byte` | syntax/lib.rs | REQ-CORRECTNESS — `matching_delimiters_from(&tree, p) == matching_delimiters_in(src, p)` for every byte of `fn f(a: [u8; 4]) { g((x)); }` + a spot pair; covers `matching_delimiters_from` end-to-end |
| T2 | the 7 existing `bm_*` #340 tests | syntax/lib.rs | REQ-FACTOR-IDENTICAL — stay green (the refactor is behavior-identical) |
| T3 | `t349_tree_accessor_none_then_some` | syntax/lib.rs | the `tree()` accessor — None pre-parse, Some after highlight_full, `source_file` root |
| T4 | `t349_bracket_match_reads_the_cached_tree_on_a_large_file_headless` | headless_drive.rs | ★ REQ-CACHE-HIT — a >1000-line file's worker tree lands in the cache, its key == the live buffer → the exact-AND guard's HIT arm fires + the pair is correct |
| T5 | `t349_edit_invalidates_the_cached_tree_and_falls_back_headless` | headless_drive.rs | ★ REQ-CACHE-INVALIDATE — an edit bumps the version → cache stale → the guard MISSES → the sync fallback lights the correct post-edit pair (no stale-tree read) |
| hook | `tree_cache_key_for_test` (app.rs, `#[cfg(test)]`) | app.rs | reads the cache's (nonce, version) so a drive asserts the key-match that IS the hit-arm guard |

**Runs (`CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley_syntax` → **77 passed, 0 skipped** (T1/T3 + the 7 #340 bm_ tests + the rest).
- `cargo nextest run -p marley` → **727 passed, 2 skipped** (T4/T5 + no regression; was 725, +2).

**★ The headless-drive fix (a real catch — the pump is a mock-clock timer).** T4/T5 FIRST FAILED (`tree_cache` empty)
because `run_until_parked()` alone does NOT drive the pump — the pump is a TIMER on the mock clock, and the worker
is a REAL OS thread, so the tree never reaches the cache without advancing the clock. Fixed by mirroring
`syntax_async_large_file_lands_off_thread_headless`: a poll loop with real-time `sleep` (the worker thread) +
`executor().advance_clock` (the pump timer) until `tree_cache_key == live (nonce, version)`. This IS the standing
`tick_pump` lesson (headless_drive.rs:67-76) — recorded there, hit here.

**Mutation (`cargo mutants --in-diff` on syntax/lib.rs):** **52 mutants: 2 caught, 50 unviable, 0 MISSED → MSI 100.**
The 2 caught = `HighlightSession::tree -> None` (killed by T3) + `matching_delimiters_in -> None` (killed by the #340
exact-pair asserts). The 50 unviable = cargo-mutants' `Range::new()`/`Range::from()`/`Range::from_iter()`
return-replacements — none compile (`std::ops::Range` is a `start..end` struct, not constructible those ways),
legitimately excluded. `matching_delimiters_from`'s behavior is pinned by T1's equivalence sweep. app.rs (the
plumbing) is `mutants::skip` → not mutated.

**NO LIVE SYNTHETIC DRIVE — stated.** chad may be at the machine; a live keyboard/screencapture drive is off-limits
AND unnecessary. T4/T5 are `cargo nextest` headless drives through the REAL `refresh_bracket_match` (the worker→pump→
cache→hit/fallback path); T1 is the pure equivalence. headless-drive ≠ live-drive.

**FULL `--diff` gate → `GATE GREEN [diff]` — 15/15.** Receipt `1242033acef59da9a9da1a25ede9f59558560fcc`. gate:4
coverage 100% lines (the pure fns + accessor tested; app.rs excluded), gate:5 MSI 100, gate:14 docs PASS. No
pre-existing failures excluded; the `search_open…` flake did not recur this run.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21):**
- CHANGELOG.md — `### Changed` entry for #349 (the tree cache; the spike numbers; hit/fallback; the exact-AND
  guard's two-fields-load-bearing; the #329/#330 follow-up).
- `docs/marley_architecture/editor.md` — the bracket-match cadence section gained a `M22 #349` paragraph (the
  worker returns its tree, the app caches `(nonce, version, Tree)` like syntax_cache, `matching_delimiters_from`
  on a hit / `matching_delimiters_in` on a miss, the load-bearing exact-AND guard, the `marley_syntax::Tree`
  re-export confining tree-sitter, #329/#330 deferred). `matching_delimiters_in`'s code doc was updated in Phase 3.

**Capture (forge wired):**
- **aar-submit** 847b3f25 — outcome completed, effectiveness 4, 1 novel finding, 13 verdicts. Lessons: (a) THE
  SPIKE GATED THE BUILD WITH A NUMBER (3.4/8.6/19.1 ms at 1.8k/4.5k/10k lines; 10k>16.7ms/frame → BUILD; else a
  MEASURED-DEFER, the #339 precedent — profile before a cross-thread cache). (b) the exact-AND cache guard's two
  fields BOTH load-bearing (reload resets version to 0 AND re-mints nonce → PR-04452c92). (c) the mock-clock pump
  trap (a real P4 catch — `run_until_parked` doesn't drive the pump timer; poll with sleep+advance_clock, the
  `tick_pump` lesson). (d) `Tree: Send` + `Clone`=`ts_tree_copy` (refcount bump, perf win stays on the worker);
  the `pub use tree_sitter::Tree` re-export confines tree-sitter (no new app dep). (e) `std::ops::Range` return
  mutants unviable (not `Range::new()`/`from()`/`from_iter()`-constructible) → 50/52 unviable, 2 viable `-> None`
  caught (MSI 100).
- **prevention-rule-record** (at inspect): `PR-claude-cache-key-every-field-load-bearing-when-a-reset-breaks-mono-001`
  (04452c92) — verify each field of a multi-field cache key is load-bearing by tracing what resets/re-mints the
  other.
- **failure-record:** none — the T4/T5 mock-clock miss was a TEST-authoring error caught+fixed in-phase, not a
  shipped code bug.
- **★ Follow-up filed:** forge **#363** (9a5cfb51-3a9e-4930-8a21-63fb53e76f0d) — "Route #329 selection-ladder +
  #330 sticky-headers through the #349 tree cache" — the deferred D6 consumers; each needs a tree-taking factor
  (`enclosing_ranges_from`/`all_headers_from`) + a hit-or-fallback site mirroring `refresh_bracket_match`.

**Close + archive:** forge #349 closed; TICKET-349 → closed/; pair archived active/ → completed/; spec Phase 5 PASS.

⚠️ **/commit — the receipt HOLDS.** The Phase-5 edits are docs-only (CHANGELOG + editor.md); the source
(syntax/lib.rs + app.rs + headless_drive.rs) is UNCHANGED since the Phase-4 receipt `1242033a…` was minted → a
fingerprint verify is valid (no `.rs` staged past the receipt).

**Status: Phase 5 — Complete PASS.**
