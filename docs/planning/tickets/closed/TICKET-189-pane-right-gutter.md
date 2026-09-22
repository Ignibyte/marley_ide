---
id: forge#189 (790fa62a-c887-4395-a99f-7cf8309dac39)
title: M12.1 — split panes fit inside the window (right gutter); no overflow
status: closed
milestone: M12.1 — Cockpit polish & fixes
pipeline: 4b23e9ab-70f4-48ac-9db4-2130ebf6573b
---

The pane-tiling center band ends flush at the window's right edge; the rightmost pane bleeds/looks cut. Add a
right gutter so panes fit inside with padding. center_bounds.w insets by PANE_GUTTER; confirm split ratios sum
to 1.0 in design.
