# TICKET-336 — Horizontal scroll: an over-wide line's tail is unreachable (the defect ticket)

- **Forge ticket:** #336 c07703c9-e388-471e-b2f1-e005a8178310 (bug, M22)
- **Owner:** claude (this session)
- **AAR:** 2c3e5411-190b-4131-acff-2b625dbe46fb
- **Pipeline doc:** ../../pipeline/completed/336-hscroll.spec.md
- **Source ticket:** M22 "The editing bar" batch (#336–340 + the 11 that follow) — the FIRST of the batch
  ([m22-editing-bar.md](../../design-notes/m22-editing-bar.md) · [roadmap.md](../../../marley_architecture/roadmap.md))
- **Status:** closed

## Summary
The editor has **no horizontal axis at all**. `CodeViewState.scroll` is a row index, `clamp_scroll_px` takes
only `(px, total_rows, cell_h)`, and each row renders `.whitespace_nowrap()` — so a line wider than the code
area clips at the list edge and **its tail cannot be seen or reached by any means**. This is a defect, not a
missing convenience: VS Code clips too (`wordWrap: off` is its default), but it gives a horizontal scrollbar.

Deliberately **not soft wrap**. H-scroll keeps the row↔line identity 1:1 that the `uniform_list` render and
every `col_starts` rider assume, so it needs **no display map**; soft wrap breaks that identity and waits on
the M22 B-c chain. That is why this ships first and cheaply.

The shape is one seam, not N rider fixes: **one translated container per row** around the code region only
(`overflow_hidden`), so the caret bar, selection bands, #310 squiggles, find bands and #331 inlay-hint spans
ride the shift as children — for free. Only two conversions are outbound: the **click** (screen→content,
`+ scroll_x`) and the **caret follow** (content→screen). This is `AD-claude-editor-offset-column-model-001`
and `AD-claude-two-boundary-maps-for-phantom-text-001` applied in advance — keep ONE coordinate domain and
shift at ONE boundary, rather than teaching every rider to subtract an offset.

## Acceptance
A line wider than the viewport can be scrolled to its final glyph; the caret stays visible on every move; a
click at any scroll lands on the char under the pointer; the gutter never shifts; `scroll_x == 0` is
byte-identical to the pre-ticket render. Full EARS REQ-001..010 in the pipeline spec.
