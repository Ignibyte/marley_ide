# TICKET-052 — capture full command output beyond the grid (block scrollback)

- **Forge ticket:** #52 `bd410f09-cc72-4bb4-860f-4a23cea5fcb3` (bug, M1.H/Terminal Polish; sprint #8)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `5b95f7a9-f442-4085-974a-ff0f49c7a764`
- **Pipeline doc:** ../../pipeline/active/block-scrollback.spec.md
- **Status:** closed

## Summary
A command exceeding the screen height keeps only the last screen (the rest scrolls off). alacritty already
keeps them (Config `scrolling_history: 10000`); read the FULL history into the FINISHED block. FIX:
`full_term_to_styled_rows` (reads `-history..screen`) + a Precmd (finish) full-capture in ingest (bounded,
one read). Integration-tested. Deps: the block model (#M1.A) + #50 (trim).

## Acceptance
A command > screen_lines → the finished block keeps ALL rows (integration test); a fitting command
unchanged; FULL gate GREEN (cov/MSI 100). Full EARS in the spec.
