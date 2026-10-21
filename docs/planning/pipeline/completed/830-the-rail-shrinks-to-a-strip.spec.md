---
pipeline_id: 8228bb68-b7cf-4777-a341-4a03fec52e68
ticket: docs/planning/tickets/open/TICKET-830-the-rail-shrinks-to-a-strip.md
status: Phase 4 — Complete PASS
title: The rail shrinks to a strip of project marks
type: feature
slice: The Marley look, W1.1 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
A button beside the rail's header folds the rail to a 60 px strip: one tile per project (its icon,
or its first letter on a tint), the active one ringed, a count on a project that needs you. Hovering
a tile shows that project's rows beside the strip. The search stays at the top as a button that
opens the filter as a popover, and Ctrl F still filters. The choice is the setting
`marley.rail_shrunk`, so it survives a restart. It comes from the design canvas's RailStates and
States boards. Today the rail is never narrower than 180 px.

## Scope
### In
- `marley.rail_shrunk: Option<bool>` (default `false`) in the settings content, `MarleySettings`,
  `default.json` and the Settings window's Layout section.
- The action `marley: toggle rail strip`. The rail header's new "Shrink the rail" button and the
  strip's "Open the rail" button run it, and it writes the setting to the user's settings file.
- The strip: a top cell the height of the rail header, holding Open; a search button in the
  filter row's place; then a tile per project in the rail's order.
- Each tile shows the project's icon (#564) or the first letter of its name on a tint of its own.
  The active project's tile has an accent ring. A count badge in the needs-you color shows how many
  of the project's rows wait on you. A click does what a click on the project's header does
  (activate, or open a closed project).
- The hover flyout: beside the tile, the project's header and rows as the open rail draws them,
  shown while the pointer is on the tile or the flyout.
- The search popover: the strip's search button or Ctrl F opens the rail's own filter field with
  the filtered rows, anchored at the strip's top; Escape, or Enter on a row, closes it.
- Width eases over 180 ms in both directions; under `reduce_motion` it jumps.
- `marley_rail::needs_you_count(project)`, a pure count of a project's rows whose class is
  `NeedsYou`.

### Out (explicitly deferred)
- A project's `logo.png` (TICKET-831). The strip shows what the open rail's header shows today.
- The Threads view in the strip (the strip lists projects; shrinking hides the view switch).
- Dragging tiles to reorder, a gear at the strip's foot, tmux and harness rows in the strip.
- A key binding for the toggle (palette and buttons only).

## Reference (§20)
N/A — Marley-specific. The rail is Marley's own sidebar (`marley_workbench::rail`); upstream Zed's
agents sidebar (`crates/sidebar`) folds groups but has no icon strip, and Warp's vertical tabs have
no shrunk form in `docs/warp_architecture/`. The behavior comes from the design canvas's
RailStates board (open, shrunk with a flyout, shrunk while searching) and its States board (the
180 ms width ease and the reduced-motion jump).

### Prior art
- The code we ship: `workspace::MultiWorkspace` draws the sidebar at `Sidebar::width` and
  re-renders whenever the rail notifies (`register_sidebar` observes it), so an eased width
  needs only `width()` computed from the toggle's `Instant` and `window.request_animation_frame()`
  while it runs. `App::reduce_motion()` (gpui) is Zed's reduced-motion switch, and gpui's own
  animation element follows it. `deferred(anchored())` is the rail's own overlay pattern (the
  empty-space menu, `rail.rs:11018`). `ui::PopoverMenu` covers the add-project menu.
- The rail already has each project's icon (`project_icons`, #564), its attention class
  (`marley_rail::project_attention`), the header click (`activate_workspace`,
  `open_closed_project`) and the filter (`filter_editor`, `focus_filter`, `FocusSidebarFilter` on
  `secondary-f`).
- Published: the activity bar of VS Code and the server strip of Slack and Discord fold a list to
  a column of marks with badges; the board follows that shape.
- Ledger: F-claude-438-a (a sidebar flag must read state `refresh` keeps, not state only
  `render` writes), L-claude-474 (an occluding child ends its group's hover; the flyout tracks its
  own hover), L-claude-644 (the rail forwards focus into the shown view; the popover's filter takes
  it the same way), L-claude-828 (switch the theme live in a scenario). The brain had nothing on
  this seam.

## UI proof
Scenario `script/e2e/830-the-rail-shrinks-to-a-strip.sh` (sway): two projects, `a` and `b`, with
`b`'s stand-in Claude Code waiting on the user, borrowing #542's FIFO stand-in and private
notification bus.
- `830-01-open`: the open rail with the new Shrink button in its header (REQ-001).
- `830-02-strip-night`: after the click, the 60 px strip in Marley Night: Open, search, tile `a`
  ringed, and tile `b` with badge 1 (REQ-001, REQ-004, REQ-005).
- `830-03-flyout`: the pointer on `b`'s tile, the flyout with `b`'s header and its waiting row
  (REQ-006).
- `830-04-search`: the strip's search button, `claude` typed, the popover with the filter and the matching
  row (REQ-007).
- `830-05-strip-day`: the theme switched to Marley Day, the strip (REQ-004, REQ-005).
- `830-06-restart`: Marley quit and started again on the same profile, still the strip; the
  scenario prints `marley.rail_shrunk` from the profile's settings (REQ-003).
- `830-07-open-again`: the strip's Open clicked, the rail back at its width (REQ-002).

## Locked-In Decisions
- D1 — The state is the setting `marley.rail_shrunk`. The buttons and the action write it with
  `settings::update_settings_file`, and the rail follows the settings store, so a hand edit of
  `settings.json` folds the rail too.
- D2 — The strip is 60 px. While shrunk the resize handle's drag is ignored, and the open width the
  user set is kept for the return.
- D3 — A tile is 36 px with rounded corners. Without an icon, it shows the first letter on the
  theme's player color for a hash of the project's name. The active project gets a 2 px accent
  ring, and the badge uses the theme's `conflict` color, the needs-you pink of Marley Night.
- D4 — The flyout shows the project's block as the open rail would draw it, in a 280 px panel, so
  rows keep their clicks, menus and states without a second row renderer.
- D5 — The search popover holds the rail's own filter editor and filtered rows. Focus moves into
  the editor when the popover opens, and closing it clears nothing the filter keeps.
- D6 — The ease is 180 ms ease-out, computed from the toggle's `Instant`; with
  `cx.reduce_motion()` it jumps.
- D7 — Shrinking while the Threads view shows switches the rail to Projects.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks the rail header's Shrink button, the rail shall fold to a 60 px strip. | `830-01-open`, `830-02-strip-night` |
| REQ-002 | WHEN the user clicks the strip's Open button, the rail shall return to the width it had. | `830-07-open-again` |
| REQ-003 | WHEN the rail is shrunk, `marley.rail_shrunk` shall be `true` in the user's settings and the strip shall return after a restart. | `830-06-restart`; the printed setting |
| REQ-004 | The strip shall show a tile per project, its icon or first letter, with the active project ringed, in Marley Night and Marley Day. | `830-02-strip-night`, `830-05-strip-day` |
| REQ-005 | WHILE a project has rows waiting on the user, its tile shall show their count on a badge. | `830-02-strip-night` |
| REQ-006 | WHILE the pointer is on a tile, the strip shall show that project's header and rows beside it. | `830-03-flyout` |
| REQ-007 | WHEN the user presses Ctrl F or the strip's search button, the strip shall open the filter with its matching rows in a popover. | `830-04-search` |
| REQ-008 | The fold and the return shall ease over 180 ms, and jump under reduced motion. | The review of the diff (a shot cannot hold motion) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the setting end to end, the action, the strip, the tiles, the flyout, the search
  popover, the ease, `needs_you_count`, and the touchpoint rows; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the rail's architecture note (§21), ledger capture
  (§19), close the ticket, archive, commit.
