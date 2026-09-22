---
title: Terminal monospace font + accurate cell metric
status: promoted
raised_by: inspect critic (TICKET-030 #30, Phase 3.5)
milestone: M1.D / M1.E (terminal-font work)
ticket: forge#34 (2d48faf5-6191-4a84-8066-0e5af1b70b67)
pipeline_spec: docs/planning/pipeline/active/terminal-font.spec.md
---

## The gap
`marley_app` sets **no explicit monospace font** — `block_row` renders each output line as a plain
`String` attached via `.child(...)`, so the terminal text paints in the window's ambient
(proportional) font. Two consequences:

1. **Visual:** terminal output in a proportional font doesn't align in columns — command output,
   tables, TUI frames, and the caret column will look ragged. A terminal must render monospace.
2. **Cell metric (introduced in #30 resize-to-pane):** `plan_resize`'s `CellSize` is read via
   `window.text_system().em_advance(text_style.font(), …)` — i.e. the ambient font's `M` advance.
   With a proportional font there's no single cell width, so the computed `cols` won't match the
   glyphs actually drawn, and the alacritty grid + child winsize get a column count that doesn't
   line up with what's on screen. The resize **plumbing is correct**; only the input metric's
   fidelity is off.

## Why deferred (not fixed in #30)
Harmless today — panes only paint plain block-row text with no cursor addressing, so nothing
depends on exact column alignment yet. It becomes a real UX defect once terminal rendering matures
(ANSI color #31 wants aligned cells; the raw/TUI grid render #33 needs a true monospace grid for
vim/top/less).

## The fix (when promoted)
- Set an explicit monospace font (an OPEN font — clean-room: do not bundle a proprietary/Warp
  font; e.g. a bundled open mono or the platform monospace) as the terminal text style.
- Read `CellSize` from that SAME monospace font (its `em_advance` × its `line_height`), not from
  `window.text_style()`.
- Likely lands with #31 (color) or #33 (raw grid), or as its own small M1.E "terminal font" ticket.

## Related
- TICKET-030 (#30) resize-to-pane — introduced the cell-metric read (app.rs render).
- Sprint M1.D "The Daily Driver" #31 (color) / #33 (raw mode) both assume aligned monospace cells.
- The "stick to Warp visually (clean-room)" intent — the terminal font is part of that skin.
