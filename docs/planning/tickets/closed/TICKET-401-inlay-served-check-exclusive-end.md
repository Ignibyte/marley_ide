# TICKET-401 — Inlay served-check never passes with the file bottom on screen — store the exclusive end (#352 F5)

- **Forge ticket:** #401 4df45f2f-4975-4616-820d-055431588935 (bug, M28 follow-up)
- **Owner:** 97360104-5afc-4ae0-8e7c-f0ccaf0c2fe5
- **AAR:** be7a238a-465d-4333-9264-309f57a1b54c
- **Pipeline doc:** ../../pipeline/completed/401-inlay-served-check.spec.md
- **Source ticket:** #352 inspect ledger F5 (found pre-existing, preserved bit-for-bit there)
- **Status:** closed

## Summary
The inlay cache stores `rows = key.first_row..key.last_row` (app.rs apply site) where
`last_row = want_last` is an INCLUSIVE row clamped to `len_lines - 1` — an inclusive value in an
exclusive `Range` position. The served-check `rows.start <= first_row && end_row <= rows.end`
therefore can never pass when the file's last line is rendered (`end_row == len_lines` vs
`rows.end == len_lines - 1`), so a short/fully-visible file re-sends the IDENTICAL inlay request
once per LSP round-trip, forever. Waste only — hints render correctly; it never wedges. Fix: store
the EXCLUSIVE end (`want_last + 1`) in the cached range.

## Acceptance
With a file shorter than the viewport open and one inlay response applied, a subsequent refresh
tick sends NO second request (the cache serves it). The full EARS criteria live in the pipeline
spec.
