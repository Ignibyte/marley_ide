# TICKET-363 — Route #329 selection-ladder + #330 sticky-headers through the #349 tree cache

- **Forge ticket:** #363 `9a5cfb51-3a9e-4930-8a21-63fb53e76f0d` (feature, M22/editor/syntax/performance/349-followup)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `d71ef05e-4670-45ab-99fb-4233ea8fea6e`
- **Pipeline doc:** ../../pipeline/active/363-cache-ladder-and-headers.spec.md
- **Source ticket:** the follow-up goal `/work 360,361,362,363,364` (the deferred D6 half of #349)
- **Status:** closed

## Summary
#349 shipped the app-side tree cache (`tree_cache: Option<(nonce, version, Tree)>`, populated on the pump) and
routed bracket-match through it. Two other consumers still reparse a throwaway `HighlightSession` per query:
`step_selection_ladder` (#329, ⌥↑/⌃W) calls `enclosing_ranges(&session, ...)`, and `refresh_sticky_headers`
(#330, on scroll) calls `all_headers(&session)`. #363 mirrors #349 exactly: factor `enclosing_ranges_from(&Tree,
...)` + `all_headers_from(&Tree)` out of the session-taking originals (the `_in` originals delegate,
behavior-identical), then reroute the two callers to read the cached tree on an EXACT `(nonce, version)` hit → the
`_from` variant (microseconds, no reparse), else the byte-identical session-parse fallback (the same exact-AND
guard as `refresh_bracket_match`; PR-04452c92). Both callers are already `#[cfg_attr(test, mutants::skip)]`, so
the reroute adds no mutation surface — the pure `_from` factors carry cov/MSI 100 via equivalence units.

## Acceptance
`enclosing_ranges_from`/`all_headers_from` are byte-identical to their session-taking originals (equivalence
units); on a cache hit the ladder/headers are built from the cached tree (headless drives via the mock-clock
pump); a miss falls back to the byte-identical reparse; bracket-match's #349 routing is unchanged.
