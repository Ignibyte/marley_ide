---
id: forge#184 (eaa3b75a-ede5-43c3-a5c2-c447f867bcb4)
title: M12 — block folding: collapse/expand a command block's output
status: closed
milestone: M12 — The Agent Cockpit
pipeline: d023ffc5-1242-4669-baae-6812efc4fcfd
---

A chevron (▾/▸) on each command block header folds its output rows to the header (the header + exit glyph stay).
Per-block, keyed by block index, reset when the block list reshapes. Pure FoldState + fold_visible_rows in
nav.rs; shim draws the chevron + skips folded output rows (viewport shrinks with them).
