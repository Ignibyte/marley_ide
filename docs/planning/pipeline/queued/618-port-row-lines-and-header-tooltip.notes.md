# A port row's clipped lines, and a header tooltip over its menu — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-618-port-row-lines-and-header-tooltip.md
- **Pipeline spec:** 618-port-row-lines-and-header-tooltip.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - F-claude-603-a-long-trailing-state-squeezed-the-rows-text-out-001: a row line's state is for a few short words.
  - #603's Test phase: the URL and unit clip at the default width while the hover buttons keep their room; `visible_on_hover` keeps its layout.
  - #606's Test phase: a header tooltip shown at the right-click stays over the menu until the pointer moves; gpui clears only a visible tooltip on mouse-down.
  - Zed withholds a trigger's tooltip while its menu is open (`dock.rs`, `PopoverMenu::trigger_with_tooltip`).
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.
