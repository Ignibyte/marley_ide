# TICKET-049 — terminal is bottom-anchored like a normal terminal

- **Forge ticket:** #49 `859c30cf-3c09-4d16-a3e2-fafd3d5ab4d5` (bug, M1.H — Real Terminal Feel, seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `caab5473-939a-44e8-9f49-01181c607f24`
- **Pipeline doc:** ../../pipeline/active/bottom-anchor.spec.md
- **Source ticket:** forge sprint #8 `0788d14d-41e4-4800-a3ff-ad6e4241ee86` (M1.H — Real Terminal Feel)
- **Status:** closed

## Summary
BUG (chad, live testing; confirmed via window capture): the pane top-packs its content, so short
output + the prompt sit at the TOP with empty space below — not a normal terminal. Fix: `.justify_end()`
on the pane `flex_col` so it bottom-anchors (prompt at the bottom, output above, empty at top). SHIM-
only (no pure surface) → validated by a window capture (the AX-verify step chad requested) + the static
gates. Deps #32 (viewport, already correct) + the pane render.

## Acceptance
The empty-prompt window capture shows the prompt at the BOTTOM (before/after); no alt-screen/scrollback
regression; static gates GREEN. Full multi-command feel is chad-verified. Full EARS in the pipeline spec.
