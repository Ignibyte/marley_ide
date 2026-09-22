# TICKET-214 — Honor OSC 8 explicit hyperlink escapes in terminal output

- **Forge ticket:** #214 (c9d31e78-86c0-4eda-9692-75876d29b5b5) (feature, M12.2)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018 (autonomous /work 195-222 run)
- **AAR:** 91bfb0b8-a74d-45a1-af6a-08544f740b7c
- **Pipeline doc:** ../../pipeline/active/warp-osc8-hyperlinks.spec.md
- **Source ticket:** sprint #25 M12.2 (f3ecc094) — deferred follow-up to #196 (clickable links)
- **Status:** closed

## Summary
#196 shipped text-scan link detection (`marley_app/src/links.rs::scan_links` + `open_url`) but
does NOT honor OSC 8 explicit hyperlinks (`ESC]8;;URI ST … text … ESC]8;; ST`), where the display
text differs from the target URI — links.rs:7-8 explicitly deferred this to "a separate
`terminal_blocks` change." This ticket carries the alacritty grid cell's `hyperlink()` URI through
`terminal_blocks::StyledRun` + `coalesce_row` (a run breaks on a hyperlink change), and makes the
`marley_app` render prefer an explicit OSC 8 hyperlink over the text-scan heuristic for that run.
One cross-crate slice on the #196 foundation; pure seams at cov/MSI 100 + masked grid-read/render shims.

## Acceptance
A program emitting an OSC 8 hyperlink (`printf '\e]8;;https://example.com\e\\click\e]8;;\e\\\n'`)
renders `click` as a clickable link opening `https://example.com`; `output_text` stays byte-identical;
old (no-hyperlink) output coalesces exactly as before. Full EARS criteria (REQ-001..006) in the
pipeline spec.
