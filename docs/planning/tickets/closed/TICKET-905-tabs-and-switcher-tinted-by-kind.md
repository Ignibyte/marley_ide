# TICKET-905 — Tabs and the Ctrl Tab switcher tinted by kind

- **Ticket:** LOCAL #905 (feature, the Marley look, center tabs and the switcher)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [905-tabs-and-switcher-tinted-by-kind.spec.md](../../pipeline/completed/905-tabs-and-switcher-tinted-by-kind.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/B-rail-chrome.md, boards Chrome and Splits)
- **Boards:** Chrome, Splits
- **Size:** S–M
- **Status:** closed

## Summary

Every center tab and every Ctrl Tab switcher row leads with the same grey tile, whatever the tab
holds. A thread tab shows its title's first letter, so "What changed…" reads "W". A terminal tab
shows two marks: Marley's `>_` tile and Zed's terminal icon after it. Only terminal tabs carry a
state dot. The Chrome and Splits boards tint each tile by kind: an agent's colour for a thread
(CC amber, CX violet, M orange, R teal), orange for a file and a Browser tab, teal for tmux. Each
tab and switcher row carries its state as a dot, and a switcher row's line says the project and
the state ("marley_ide · needs you", "repo · cargo run"). This ticket draws that, and makes the
kind tints, the agent glyphs and the state dots one shared palette that the rail, the tabs, the
switcher and the bottom row (#907 to #909) all draw from.

## Acceptance

Tabs and switcher rows are tinted by kind; a thread tab shows its agent's glyph; a terminal tab
shows one mark; thread, Browser and tmux tabs carry a state dot; the switcher's line says
"project · state". Shot in Marley Night and Marley Day.
