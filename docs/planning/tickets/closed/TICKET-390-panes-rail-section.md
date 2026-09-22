# TICKET-390 — Panes rail section: split views un-nest into the 4th section

- **Forge:** #390 `b15dd53b-3df2-4f00-a2ae-515afed6eaa6` (sprint #38 `fc972431`)
- **Type:** feature
- **Milestone:** M27
- **Status:** closed
- **Pipeline:** docs/planning/pipeline/queued/390-panes-rail-section.spec.md

## Summary
The rail gains chad's 4th fixed-order section — **Panes** (Editor · Terminal · Panes · Browser). Any
tab whose grid holds ≥2 cells lists there as an arrangement row "PANE n" (unit = arrangement, chad-
locked); its cell rows move under the arrangement (the #155 under-tab nesting dissolves); the origin
tab keeps its own section row. Arrangement click focuses the tab; Panes＋ → split the focused pane.
Display-first on coordinates (#394 adds ContentId later).

## Headline acceptance
Four section headers render; splitting a terminal makes "PANE 1" appear under Panes (and its cells
nest there, not under the tab); closing/unsplitting removes it; the origin tab row never moves.
