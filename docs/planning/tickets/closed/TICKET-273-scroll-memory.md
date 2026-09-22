# TICKET-273 — Per-file scroll memory + scroll_to_item

- **Forge ticket:** #273 0d3556f7-8948-4f21-904b-8f9a5dbb83bf (bug, M17)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 1ca979c4-1d8b-4994-9e56-33d2a8e332cb
- **Pipeline doc:** ../../pipeline/completed/273-scroll-memory.spec.md
- **Source ticket:** sprint #30 (forge)
- **Status:** closed

## Summary
Heal the disclosed #266 trade: the shared editor scroll handle carries
the offset across file-tab switches. Park each file's PIXEL offset on
`OpenFile.scroll_px` at the switch choke points (capture via the pub
`base_handle.offset()`, restore via `set_offset` — the test-only
`logical_scroll_top_index` is for asserts), clamp on shrink, and ship
`scroll_editor_to_row(row)` = non-strict `scroll_to_item(row, Center)` —
the one mechanism #270/#272/#212/#213 consume.

## Acceptance
Per-file offsets survive switch/open/close (pixel-exact); shrink clamps;
the helper centers off-screen rows and no-ops on visible ones; the #246
pane untouched. Full EARS in the spec.
