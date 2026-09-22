---
pipeline_id: 788d1a9e-78ae-4f3d-9bec-cc51406c4f40
ticket: forge#363 (9a5cfb51-3a9e-4930-8a21-63fb53e76f0d) · local docs/planning/tickets/open/TICKET-363-cache-ladder-and-headers.md
aar_id: d71ef05e-4670-45ab-99fb-4233ea8fea6e
status: Phase 5 — Complete PASS
title: Route #329 selection-ladder + #330 sticky-headers through the #349 tree cache (enclosing_ranges_from + all_headers_from + the hit-or-fallback reroute)
type: feature
milestone: M22
references: [syntax/lib.rs:488 enclosing_ranges (factor _from) / :685 all_headers (factor _from) / :593 matching_delimiters_in + :612 matching_delimiters_from (the #349 template) / :361 HighlightSession::tree(), app.rs:502 tree_cache Option<(u64,BufferVersion,Tree)> / :3678 refresh_bracket_match (the hit-or-fallback template) / :3488 step_selection_ladder [mutants::skip, reroute :3530-3532] / :3586 refresh_sticky_headers [mutants::skip, reroute :3615-3617] / :3821 tree_cache_key_for_test, headless_drive.rs:6650 t349 cache-hit drive / :6715 t349 invalidate drive / :89 tick_pump, #349 #329 #330 PR-04452c92]
---

## Title
Factor `enclosing_ranges_from(&Tree, ...)` + `all_headers_from(&Tree)` out of the session-taking originals (the
#349 pattern), then reroute `step_selection_ladder` (#329) and `refresh_sticky_headers` (#330) to read the cached
tree on an exact `(nonce, version)` hit — no reparse — falling back to the byte-identical session-parse on a miss.

## Scope
### In
- **marley_syntax (pure):** `pub fn enclosing_ranges_from(tree: &tree_sitter::Tree, byte_range: Range<usize>) ->
  Vec<Range<usize>>` (the walk from `enclosing_ranges` :495) + `pub fn all_headers_from(tree: &tree_sitter::Tree)
  -> Vec<Range<usize>>` (the `collect_headers` call from `all_headers` :689). The session-taking `enclosing_ranges`
  /`all_headers` become `session.tree.as_ref().map(|t| <_from>(t, …)).unwrap_or_default()` — behavior-identical,
  their existing tests stay green.
- **app.rs (the 2 reroutes, inside the already-`mutants::skip`'d fns):** each caller's
  `enclosing_ranges(&session, r)` / `all_headers(&session)` becomes `match &self.tree_cache { Some((cn, cv, tree))
  if *cn == nonce && *cv == version => <_from>(tree, …), _ => <the existing session-parse> }`. The miss branch is
  the UNCHANGED `HighlightSession::new + highlight_full + enclosing_ranges/all_headers` (byte-identical fallback).
- Equivalence units (`_from(&parse(src)) == <session-taking>(&session_of(src))` over a fixture sweep) + mock-clock
  cache-hit + miss-fallback headless drives.

### Out (explicitly deferred)
- Any change to `matching_delimiters`/bracket-match (#349's routing — UNCHANGED; REQ-349-unchanged).
- Any change to the tree-cache POPULATION (the pump path, #349) or the `(nonce, version)` scheme (PR-04452c92).
- Widening the caller-side Rust gate (the #340 M1 rule stays; #315 widens it later).
- An incremental-parse (tree-sitter `edit` reuse) — a separate #349 follow-up, not this ticket.

## Reference (§20)
N/A — Marley's own tree cache + node-range APIs. No Warp/Zed source read.

### Prior art
- **Our own code (the fix IS a #349 transplant, ×2):** `matching_delimiters_from`/`_in` (lib.rs:612/593) is the
  exact factor template; `refresh_bracket_match` (app.rs:3678) is the exact hit-or-fallback template with the
  exact-AND `(nonce, version)` guard (`PR-04452c92` — both fields load-bearing: a reload resets version→0 AND
  re-mints the nonce). `HighlightSession::tree()` (:361, #349) is the accessor. #363 mirrors these twice.
- **Our permissive deps:** tree-sitter owns the `Tree`/`Node`/`TreeCursor` walk (Apache/MIT, adoption) — already
  used by `enclosing_ranges`/`collect_headers`; the factor just splits the parse from the walk, no new adoption.
- **Behavior maps / published:** N/A — an internal caching refactor; no reference-app behavior.

## Locked-In Decisions
- **D1 — factor `_from(&Tree, …)`, keep the `_in`/session originals as `map(_from).unwrap_or_default()`** — a
  behavior-identical extraction (the existing enclosing_ranges/all_headers tests stay green, and become the
  `_in`-side of the equivalence). Combinator form (no early-return that leaves an uncoverable region — the
  node_kind_at lesson; `let Some…else{return}` is already total but the `.map(...).unwrap_or_default()` keeps the
  None-flow in a combinator).
- **D2 — the reroute mirrors `refresh_bracket_match` EXACTLY:** the hit arm `Some((cn,cv,tree)) if *cn==nonce &&
  *cv==version => _from(tree, …)`; the miss arm = the UNCHANGED session-parse (byte-identical — no new fallback
  code, just the existing `HighlightSession::new + highlight_full + enclosing_ranges/all_headers`). The exact-AND
  guard (both fields; PR-04452c92) — a stale tree is never read.
- **D3 — no new mutation surface.** Both callers are already `#[cfg_attr(test, mutants::skip)]` (verified) → the
  reroute inherits the skip (like #362 inside `replace_text_in_range`) → NO #361 gate-red, NO new skip. The pure
  `_from` factors (marley_syntax, cov-INCLUDED) are the cov/MSI-100 surface via the equivalence units.
- **D4 — the cache-hit drives use the mock-clock pump** (`tick_pump` — the tree_cache is pump-populated; a
  `run_until_parked` alone never caches it, so the hit branch would test VACUOUSLY — the #349/#334 lesson).
  Mirror `t349_bracket_match_reads_the_cached_tree_on_a_large_file_headless` (poll `tree_cache_key_for_test`
  non-None, then drive the gesture, assert the result).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-LADDER-FROM-EQUIV | `enclosing_ranges_from(&parse(src), r)` shall equal `enclosing_ranges(&session_of(src), r)` for every fixture. | pure equivalence unit over a fixture sweep (the #329 ladder fixtures: nested, caret, whole-file, reversed-range, no-tree) — cov/MSI 100. |
| REQ-HEADERS-FROM-EQUIV | `all_headers_from(&parse(src))` shall equal `all_headers(&session_of(src))` for every fixture. | pure equivalence unit (the #330 fixtures: nested, free-code, deep-nesting, no-tree) — cov/MSI 100. |
| REQ-LADDER-HIT | WHEN the tree is cached and ⌃W grows the selection, the ladder shall be built from the cached tree. | headless: large fixture, `tick_pump` until cached, ⌃W → the ladder's rungs are correct (the pure equiv proves `_from`; this proves the wiring reads the cache). |
| REQ-HEADERS-HIT | WHEN the tree is cached and sticky-headers refresh, they shall be built from the cached tree. | headless: `tick_pump` until cached, `refresh_sticky_headers_for_test` → the headers are correct on the hit. |
| REQ-MISS-FALLBACK | WHEN the cache misses (version bumped before the worker re-sends), the system shall reparse — byte-identical to today. | headless: edit to bump the version (no re-pump) → the ladder/headers still correct via the session-parse (mirror #349 T5). |
| REQ-349-UNCHANGED | Bracket-match's #349 routing shall be unchanged. | the #349 t349_* drives + matching_delimiters tests stay green. |

## Phase Plan
- **P2 Design** — settle the 2 factor signatures + the `.map(_from).unwrap_or_default()` delegation + the 2
  reroutes (miss = the unchanged session-parse) + the equivalence-unit sweep (reuse the existing #329/#330 test
  fixtures) + the mock-clock cache-hit/miss drives (mirror #349 T4/T5) + confirm no other callers ripple.
- **P3 Implement** — the 2 marley_syntax factors + delegations; the 2 app reroutes.
- **P3.5 Inspect** — ★ `_from == _in` (the equivalence); the exact-AND guard (both fields load-bearing); the miss
  branch byte-identical; no other callers; the reroute inside the skipped fns (no new mutants — confirm `cargo
  mutants --list`); the `_from`/delegation cov-100 (no uncoverable combinator).
- **P4 Validate** — the equivalence units + the mock-clock cache-hit/miss drives + the `--diff` gate.
- **P5 Complete** — CHANGELOG + editor.md (#329/#330 now cached) + the #349-cache doc.
