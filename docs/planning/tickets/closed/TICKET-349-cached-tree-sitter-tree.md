# TICKET-349 — App-side cached tree-sitter tree (stop the per-caret bracket-match reparse)

- **Forge ticket:** #349 `18213b21-9822-44bf-aa93-3218db3a7501` (feature, M22/editor/syntax/performance, #340 follow-up)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `847b3f25-99a7-4e82-8158-79f970b6b724`
- **Pipeline doc:** ../../pipeline/completed/349-cached-tree-sitter-tree.spec.md
- **Source ticket:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (the #340 perf follow-up)
- **Status:** closed — shipped LOCAL: the tree cache + bracket-match reading it (the spike said BUILD, 19ms/10k). #329/#330 reroute → forge #363. GATE GREEN [diff] 15/15.

## Summary
The app holds no cached tree-sitter tree. The live tree lives only on the syntax worker thread, which sends back
per-line spans (`SyntaxResp{lines}`), never the tree — so every consumer that needs a NODE reparses a throwaway
`HighlightSession`: #340 bracket-match (every caret move — the hot path), #329 selection-ladder, #330
sticky-headers. #349 caches the tree app-side per `(nonce, version)` so a node query is microseconds, not a reparse.

**The profiling spike (release, the ticket's gate) DECIDED build:** the #340 parse-only path
(`matching_delimiters_in`) measured 3.4ms/1.8k lines, 8.6ms/4.5k, **19.1ms/10k — OVER a 16.7ms/60fps frame** at 10k
lines. A memo-missing per-caret reparse during rapid motion on a large file drops frames. Build Option A: the worker
(which already parses the tree for spans) clones its `Tree` into `SyntaxResp`; the app caches
`Option<(u64, BufferVersion, Tree)>`; a factored pure `matching_delimiters_from(&Tree, pos)` reuses it;
`refresh_bracket_match` reads the cache and falls back to the sync reparse on a `(nonce,version)` miss. `Tree: Send`
+ `Clone` confirmed.

## Acceptance
On a `(nonce,version)`-stable frame the bracket-match node query reads the cached tree (no reparse); an edit bumps
`version` → the next query refreshes once; the cached-tree query returns the SAME delimiters as a throwaway reparse
(equivalence). Full EARS criteria in the pipeline spec.
