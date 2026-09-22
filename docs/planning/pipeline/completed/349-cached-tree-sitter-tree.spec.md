---
pipeline_id: 94aa473e-c3d3-4233-b845-a623e175658a
ticket: forge#349 (18213b21-9822-44bf-aa93-3218db3a7501) · local docs/planning/tickets/open/TICKET-349-cached-tree-sitter-tree.md
aar_id: 847b3f25-99a7-4e82-8158-79f970b6b724
status: Phase 5 — Complete PASS
title: App-side cached tree-sitter tree — stop the per-caret bracket-match reparse (#340 follow-up); the profiling spike said BUILD (19ms/10k lines > frame)
type: feature
milestone: M22
references: [marley_syntax matching_delimiters_in (lib.rs:556, parse-only, its doc names #349), HighlightSession.tree (lib.rs private), app.rs SyntaxResp (:13383) / the worker (:13311) / refresh_bracket_match (:3602) / bracket_match_key (:315) / syntax_cache (:497) / sticky_headers (:305), tree-sitter 0.26.11 Tree Send (lib.rs:3908) + Clone (:1521), #340 bracket-match AD, #329 selection-ladder, #330 sticky-headers, #274 incremental-parse worker, #268 syntax cache]
---

## Title
Cache the tree-sitter tree app-side, per `(nonce, version)`, so the #340 bracket-match node query (and #329/#330)
reuse the tree the syntax worker already parsed instead of reparsing a throwaway `HighlightSession` on every caret
move. **The profiling spike (the ticket's own gate) measured the reparse OVER a 60fps frame at 10k lines → BUILD.**

## Scope
### In
- `SyntaxResp` gains an `Option<tree_sitter::Tree>` — the worker clones the tree it ALREADY parsed for spans
  (`Tree: Send + Clone`, confirmed) and sends it alongside the per-line spans.
- An app-side `tree_cache: Option<(u64, BufferVersion, tree_sitter::Tree)>`, populated by the pump that drains
  `SyntaxResp`, keyed exactly like `syntax_cache` (`(nonce, version)`).
- A pure `marley_syntax::matching_delimiters_from(&tree_sitter::Tree, byte_pos)` — the `root → delims_at`
  before-then-at query FACTORED OUT of `matching_delimiters_in` (which becomes `parse + matching_delimiters_from`,
  behavior-identical). Reuses a cached tree in microseconds.
- `refresh_bracket_match` reads `tree_cache` on a `(nonce, version)` HIT → `matching_delimiters_from`; on a MISS
  (small file / pre-worker-response / stale version) → the existing sync `matching_delimiters_in`. Correctness is
  identical either way; only the cost differs.
- The SHARED WIN: route #329 selection-ladder + #330 sticky-headers through the cached tree too (they reparse the
  same throwaway session today) — IF the reroute stays small (P2 confirms; else a bounded follow-up).

### Out (explicitly deferred)
- Incremental reparse ON the app thread (Option B) — rejected: doubles edit-parse work; the worker already has
  the tree.
- Changing WHAT bracket-match / #329 / #330 compute — byte-identical results; this is purely where the tree comes
  from.
- A tree cache for non-Rust or for consumers that only need spans (`syntax_cache` already serves spans).

## Reference (§20)
N/A — Marley-specific. This is Marley's own syntax/perf seam (the worker→app boundary + the app-side node-query
consumers). No Warp/Zed source read. tree-sitter's `Tree`/`Node`/`descendant_for_byte_range` are ADOPTION (outside
the wall).

### Prior art
1. **★ THE PROFILING SPIKE (the highest-yield leg — it GATES the ticket, and it said BUILD).** A release micro-bench
   of the #340 parse-only path `matching_delimiters_in` (best-of-N over a real-crate-file fixture repeated to size):
   **1.8k lines → 3.4ms · 4.5k → 8.6ms · 10k → 19.1ms.** The 16.7ms/60fps frame budget is CROSSED at 10k lines. The
   ticket's conditional ("realistic 5k-10k line file shows jank") is met; #340's memo absorbs the stationary-caret
   common case but not a memo-missing held-arrow on a large file. So this is not a dissolve (unlike #339
   D-EMPTY-ADVANCE) — the measurement says build.
2. **OUR OWN CODE (the seams are built):** `syntax_cache: Option<(u64, BufferVersion, SyntaxLines)>` (app.rs:497) is
   the EXACT cache shape to mirror for the tree; `sticky_headers` (app.rs:305) is a second instance of the same
   `(nonce, version)` keying. `matching_delimiters_in` (marley_syntax lib.rs:556) is the parse-then-query whose
   query half factors out. `SyntaxResp` (app.rs:13383) already carries `nonce` + `version` — the cache key is free.
3. **tree-sitter 0.26.11 (ADOPTION):** `unsafe impl Send for Tree {}` (binding_rust/lib.rs:3908) — the tree can
   cross the worker→app mpsc channel; `impl Clone for Tree` (lib.rs:1521) is a cheap `ts_tree_copy` refcount bump —
   the worker clones into the resp while keeping its own for incremental parse. Both gates for Option A clear.

## Locked-In Decisions
- **D1-BUILD (the spike's verdict, with evidence).** The parse-only reparse crosses a 60fps frame at 10k lines
  (19.1ms measured, release); at 4.5k it is half a frame (8.6ms). #340's memo absorbs a stationary caret but a
  memo-missing held-arrow on a large file drops frames. BUILD the cache. (Had the 10k number stayed sub-frame, D1
  would be MEASURED-DEFER — the #339 precedent — but it did not.)
- **D2-OPTION-A (worker returns its tree).** The worker already parses the tree for spans; it clones the tree
  (`Tree: Clone`) into `SyntaxResp` (`Tree: Send`). App caches `Option<(u64, BufferVersion, Tree)>`. NO double-parse
  (Option B's cost). The private `HighlightSession.tree` is surfaced by a `pub fn tree(&self) -> Option<&Tree>`
  accessor (or `highlight_full` co-returns it — P2 picks).
- **D3-FACTOR-THE-QUERY.** `matching_delimiters_from(&Tree, pos)` is the pure `root → delims_at` half extracted from
  `matching_delimiters_in`; the latter becomes `parse + matching_delimiters_from` (behavior-identical, its tests
  stay green). cov/MSI 100 on the extracted query.
- **D4-HIT-OR-FALLBACK (invalidation correctness — the load-bearing risk).** `refresh_bracket_match` uses the
  cached tree ONLY when `tree_cache`'s `(nonce, version)` matches the current buffer's; ANY mismatch (a small file
  with no worker, a version bumped by an edit before the worker re-sent, a project switch) falls back to the sync
  `matching_delimiters_in`. A stale tree is a silent wrong-highlight, so the key check is exact and the fallback is
  always correct. Mirror `syntax_cache`'s own gate; add a debug-assert-loud check if a stale read is ever possible.
- **D5-MIRROR-NOT-INVENT.** The tree cache reuses the established `(nonce, version)` pattern (`syntax_cache`,
  `sticky_headers`) — same key tuple, same refresh cadence — NOT a new caching scheme.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-CACHE-HIT | WHEN the bracket-match query runs on a frame whose `(nonce, version)` matches the cached tree, the system shall read the cached tree and NOT reparse. | a parse-count probe (a `#[cfg(test)]` counter / an instrumented `matching_delimiters_from` vs `_in`): a cache-hit path invokes NO `Parser::parse`. |
| REQ-CACHE-INVALIDATE | WHEN an edit bumps `version`, the system shall NOT use the now-stale cached tree — the next query falls back to a fresh parse until the worker re-sends. | headless: edit → the immediate bracket-match uses the sync path (version mismatch), returns the CORRECT pair for the edited text. |
| REQ-CORRECTNESS | The cached-tree query shall return the SAME delimiter pair as a from-scratch `matching_delimiters_in` for every caret position. | pure equivalence unit: for a fixture + a sweep of byte positions, `matching_delimiters_from(&parse(src), p) == matching_delimiters_in(src, p)`. |
| REQ-FACTOR-IDENTICAL | `matching_delimiters_in` shall behave byte-identically after being refactored to `parse + matching_delimiters_from`. | the existing #340 `matching_delimiters_in` tests stay green. |

## Phase Plan
- **P2 Design** — settle the `HighlightSession.tree` accessor vs co-return; the `SyntaxResp.tree` field + the
  worker clone; the `tree_cache` field + the pump store; the `matching_delimiters_from` factor; the
  `refresh_bracket_match` hit-or-fallback; the #329/#330 reroute (in-scope if small, else a bounded follow-up); the
  test plan (the pure equivalence + factor units in marley_syntax cov/MSI 100; the parse-count + invalidate as
  headless drives / a probe, since the cache plumbing is the app shim).
- **P3 Implement** — the accessor + the resp field + the cache + the factored query + the reroute.
- **P3.5 Inspect** — ★ the (nonce,version) invalidation (a stale tree = silent wrong highlight — the #339 loud-assert
  class); the Send/Clone safety (no aliasing, the clone is a refcount bump not a deep copy); no double-parse; the
  factor is behavior-identical; no second cache pattern.
- **P4 Validate** — the equivalence + factor units + the parse-count-hit + the invalidate drive + the `--diff` gate.
- **P5 Complete** — CHANGELOG + editor.md (the cache + the 3 consumers off the reparse path) + the perf note (the
  spike's numbers); update `matching_delimiters_in`'s doc (its "#349 is the follow-up" line now points at the
  shipped cache).
