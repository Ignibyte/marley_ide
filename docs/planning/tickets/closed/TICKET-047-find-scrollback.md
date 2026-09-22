# TICKET-047 — find in scrollback (cmd-F)

- **Forge ticket:** #47 `78f75b06-83b1-4bc1-8e98-d89742cf1434` (feature, M1.G — Block Workflows & Selection, seq-5)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `b78678e8-f4c0-49a2-b943-a9f174581c9a`
- **Pipeline doc:** ../../pipeline/active/find-scrollback.spec.md
- **Source ticket:** forge sprint #7 `eb842cb6-f4ab-4b27-89e0-78634476f304` (M1.G — Block Workflows & Selection)
- **Status:** closed

## Summary
cmd-F opens a find bar over the block output: highlights matches + Enter/Shift-Enter cycle-and-scroll.
Pure `find_matches(haystack, query)` (non-overlapping, ASCII-fold, empty→[]) + `match_navigation(len,
current, forward)` (wrap) in a new find.rs (cov/MSI 100); the overlay + highlight + scroll are the app
shim (mirror the palette #25 + viewport #32). Deps #25 + #32.

## Acceptance
`find_matches` + `match_navigation` at cov 100/MSI 100 (non-overlap "aa" in "aaa" → 1; empty → [];
case-fold; fwd/back wrap; len 0 → 0); the cmd-F overlay highlight + cycle (masked visual —
chad-verified); FULL gate GREEN. Full EARS in the pipeline spec.
