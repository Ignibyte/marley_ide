---
id: forge#185 (e0611622-933f-45bf-b53f-6d778b21f960)
title: M12 — sticky command header while scrolling a long block
status: closed
milestone: M12 — The Agent Cockpit
pipeline: d3b50a15-4bd3-4715-b458-cffd497b64bb
---

When you scroll deep into a long command's output, its command header pins to the pane's top edge (a sticky
overlay row) until the next block's header scrolls in. Pure sticky_block(counts, folds, viewport_top) over the
#184 fold-aware rows (Some when the top row is a block's output, None at a header/prompt); shim renders the
overlay header.
