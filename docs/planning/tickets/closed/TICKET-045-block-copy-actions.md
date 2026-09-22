# TICKET-045 — Block actions: copy command / copy output

- **Forge ticket:** #45 `22c511c3-3508-404c-9cbb-71fd1a79e00b` (feature, M1.G — Block Workflows & Selection, seq-3)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `bcc0a203-3260-49d3-a586-d9199ff6e646`
- **Pipeline doc:** ../../pipeline/active/block-copy-actions.spec.md
- **Source ticket:** forge sprint #7 `eb842cb6-f4ab-4b27-89e0-78634476f304` (M1.G — Block Workflows & Selection)
- **Status:** closed

## Summary
Warp's signature: hover a command Block → copy its COMMAND or OUTPUT (no manual selection). Pure
`BlockCopy { Command, Output }` + `Block::copy_text(what)` (Command → command; Output → output_text())
in terminal_blocks (cov/MSI 100); the header's hover-revealed copy affordances + the clipboard write
are the app shim (reuses #44). Deps #36/#43 (header) + #44 (clipboard).

## Acceptance
`Block::copy_text` at cov 100/MSI 100 (Command arm; Output arm == output_text; the arms distinct); the
hover→click→paste (masked visual — chad-verified); FULL gate GREEN. Full EARS in the pipeline spec.
