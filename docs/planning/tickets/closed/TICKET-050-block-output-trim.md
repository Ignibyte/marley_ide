# TICKET-050 — block output trims trailing blank rows (history stacks)

- **Forge ticket:** #50 `9cdc30e8-2f65-4a07-bbf1-fdc55b8ae936` (bug/critical, M1.H — Real Terminal Feel, seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `d60e56c1-ea66-4078-8a9c-0e6104a2e596`
- **Pipeline doc:** ../../pipeline/active/block-output-trim.spec.md
- **Source ticket:** forge sprint #8 `0788d14d-41e4-4800-a3ff-ad6e4241ee86` (M1.H — Real Terminal Feel)
- **Status:** closed

## Summary
CRITICAL: "every command clears the previous" — blocks don't stack. Each block captures the FULL grid
(term_to_styled_rows over all screen_lines, incl. trailing blank rows), so one short command is a
full-screen block that fills the viewport. Fix: a pure `trim_trailing_blank_rows(Vec<StyledLine>)` in
styled.rs (cov/MSI 100), applied to the block-output path in session.rs `ingest` only; `grid_styled_rows`
(alt-screen) keeps the full grid. Blocks then stack as scrollback. Deps block model #M1.A + #49.

## Acceptance
`trim_trailing_blank_rows` at cov 100/MSI 100 (trailing dropped, interior kept, all-blank→[], no-trailing
unchanged); a session test (a command's block output has no trailing blanks); FULL gate GREEN; rebuild +
window capture shows two commands STACKED as separate blocks (chad-typed). Full EARS in the pipeline spec.
