# TICKET-422 — Pane-divider drag resizes the boundary you grabbed

- **Ticket:** LOCAL #422 (bug, M20)
- **Tags:** panes, layout, drag, divider
- **Created:** 2026-08-12
- **Status:** closed (2026-08-12 — shipped: every divider resizes its own boundary, band/axis/capture fixed; 2146 tests green; GATE GREEN [diff] 15/15)

## Summary

Chad's report: "drag and drop works only on the first item and it's very clunky." There is no
rail-row DnD (deferred v2, #398 spec) — this is the pane-divider resize drag. Divider handles
are painted for EVERY flattened leaf boundary (app.rs:20628-20654) but the sink
`resize_boundary` (layout.rs:198-206) only mutates a top-level 2-child Split, and
`resize_split` (layout.rs:150-153) no-ops for boundary ≥ 1 — so only the first divider ever
works; the rest are painted, hoverable, `.occlude()`-ing, and inert. Clunk factors in the same
block: the grab strip is drawn at `top(0)` over the title bar (the pane band starts at
TOP_BAR_H=30) and falls 30px short at the bottom; handles are always vertical 6px strips even
on a Vertical split's horizontal edge; no pointer capture (`dragging_divider`, app.rs:642,
:20656-20675). Fix: route the drag to the actual grabbed boundary (nested splits included),
draw handles on the correct axis within the correct band, and give the gesture capture
semantics. Recall pin L TICKET-020: layout.rs boundary index math — `checked_sub`, never
eager `- 1`.

## Acceptance

Headline: every divider in any split arrangement (including nested and Vertical splits)
resizes its own boundary under drag, with the grab strip aligned to the visible divider;
pinned by pure layout tests over nested-split routing. Full EARS in the queued spec
(`docs/planning/pipeline/queued/422-pane-divider-drag-routing.spec.md`).
