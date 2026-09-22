# TICKET-048 — clear (cmd-K) + jump-to-block nav (⌘↑/⌘↓)

- **Forge ticket:** #48 `be79e95e-979b-4850-8a9c-2a73ae89211d` (feature, M1.G — Block Workflows & Selection, seq-6, FINALE)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `054d392c-b1b1-48a4-b0ca-55b2f566e103`
- **Pipeline doc:** ../../pipeline/active/block-nav.spec.md
- **Source ticket:** forge sprint #7 `eb842cb6-f4ab-4b27-89e0-78634476f304` (M1.G — Block Workflows & Selection)
- **Status:** closed

## Summary
cmd-K clears the shell; ⌘↑/⌘↓ jump the viewport between block boundaries. Pure
`block_boundary_rows(line_counts)` (per-block header content-rows) + `jump_target(boundaries, top,
forward)` (next/prev boundary, no wrap) in a new nav.rs (cov/MSI 100); the keymap bindings + the
jump-scroll (reuse #47 scroll_focused_to_row) + the `\x0c` clear are the app shim. CLOSES M1.G. Deps
#32 + #M1.B.

## Acceptance
`block_boundary_rows` + `jump_target` at cov 100/MSI 100 ([2,0,3]→[0,3,4]; fwd/back mid + both ends →
None); the ⌘↑/⌘↓ jump + cmd-K clear (masked visual — chad-verified); FULL gate GREEN. Full EARS in
the pipeline spec.
