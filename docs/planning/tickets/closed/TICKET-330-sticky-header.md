# TICKET-330 — Sticky context header: the enclosing fn/impl pinned at the viewport top while you scroll

- **Forge ticket:** #330 4de9f2f2-86a7-4142-a54b-5a12ee1cba6d (feature, M21)
- **Owner:** claude (this session)
- **AAR:** dd0191e6-e9f6-438c-9cef-a442027258cc
- **Pipeline doc:** ../../pipeline/active/330-sticky-header.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331)
- **Status:** closed

## Summary
Scroll into the middle of a long function and the editor still shows where you are: the enclosing `fn`/`impl`/
`mod` header pinned at the viewport top, clickable to jump. The editor twin of the terminal `sticky_block`
(#185), powered by the #329 `enclosing_ranges` node API. A pure header-node collector (marley_syntax, filtered
by `Node::kind()`) + a pure `sticky_rows(headers, first_visible_row, max_depth)` pin decision (strictly-inside
+ header-scrolled-off, nested-stacked, capped at max_depth=2, the no-double-render edge). The header list is
cached per `(nonce, version)` — one reparse per edit, NOT per scroll frame — and `sticky_rows` filters per
frame. Rendered as an absolute overlay above `uniform_list` (`block_mouse_except_scroll`, not occlude — the
#185 no-scroll-dead-zone precedent), click = `open_and_place_caret` + NavStack. F8/find centering + caret
visibility account for the band height. `editor.sticky_header` default-on + a palette toggle.

## Acceptance
The header collector + `sticky_rows` at cov/MSI 100 (nested impl/fn, fn at top, free code → empty;
strictly-inside / at-edge-no-pin / nested-stack / depth-cap / never-panic). Headless: scroll a seeded 100-line
fn → the header pins at the right `first_visible_row`, unpins when its own row is at the top, click jumps +
NavStack; toggle off → no sticky. Live pixel drive env-blocked (screen locked) → units+mechanism. Full EARS in
the pipeline spec.
