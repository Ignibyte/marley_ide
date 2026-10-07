# TICKET-672 — Rusty's screens in the rail's header, and the row under it at the tab bar's height

- **Ticket:** LOCAL #672 (feature, the rail; Rusty in Marley R-D9)
- **Owner:** claude-opus-5-5, 2026-10-07 (Chad's request)
- **Pipeline doc:** ../../pipeline/completed/672-rusty-screens-in-the-rail-header.spec.md
- **Source ticket:** #644 (the Brain view and its header switch)
- **Status:** closed

## Summary
Two of Chad's 2026-10-06 items, which both reshape the top of the rail.

**Rusty's icons go up a level.** Chad: "I want all the rusty brain icons at the top in the same
location as Projects and the Folder with the +. Instead of clicking over to the other icon and
then seeing the other tabs." And, explaining it: "the rusty brain has a tab at the top left that
shows sub tabs. The idea would each sub tab just lives up in the top next to the others and it
shows its information when you click on it just as it does now except not two levels deep."
Today the header holds Projects and Brain (`render_view_switch`, `rail.rs:4159`), and the Brain
view adds a second row: Today, Graph, Tasks, Decisions, Memory, Skills, Secrets
(`BrainView::render_fixed_row`, `rusty/brain.rs:1008`). Those seven move into the header beside
Projects and Brain, in both views, and each still opens or focuses its center tab as now
(`brain.rs:382-433`). Brain keeps showing the vault tree and its search in the rail.

**The row under the header lines up with the tab bar.** Chad: "on filter it should be the same
height as the <- -> |Tab| of the main menu. Even when the icons expand?" The rail header already
matches the title bar (`platform_title_bar_height`). The row under it does not match the pane's
tab bar: the filter row (`render_filter`, `rail.rs:4497`) is `py_2` around an editor line, while
the tab bar is `Tab::container_height` (`DynamicSpacing::Base32`, `ui/src/components/tab.rs:79`).
The filter row, and the Brain view's search row that takes its place, get the tab bar's exact
height and a bottom border on the same line as the tab bar's. Nothing inside the row changes its
height: not the clear button that replaces the key hint, and not the Brain view's icons, which
this ticket moves out of that space.

## Open points for Plan
- **Width.** Ten buttons in the header (Projects, Brain, the seven, the folder `+` or New Page)
  at Chad's scale measure about 36px apiece, about 360px, against a rail of about 320px. Compact
  buttons with a smaller gap, or an overflow menu for whatever does not fit, decided against a
  shot at his scale. Linux window controls also sit in the header when the button layout asks.
- **The "group called Rusty" with a placeholder icon** from Chad's first message waits for
  tomorrow's talk (`docs/planning/intake/rail-and-center-tabs.md`). This ticket only flattens the
  icons.
- How the tab bar draws its bottom border (`ui::TabBar`), so the two lines meet exactly.

## Acceptance
While Rusty is connected, the header shows Projects, Brain and the seven screens in one row in
both views, and each opens or focuses its tab with one click. The Brain view has no second icon
row. The filter row and the Brain view's search row are the tab bar's height, with their bottom
line level with the tab bar's, whether the filter is empty or holds text. Proof: shots of both
views beside a pane's tab bar, and one with text in the filter.
