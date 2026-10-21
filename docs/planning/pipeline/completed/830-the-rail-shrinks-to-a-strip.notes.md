# The rail shrinks to a strip of project marks — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-830-the-rail-shrinks-to-a-strip.md
- **Pipeline spec:** 830-the-rail-shrinks-to-a-strip.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #829; Chad's overnight goal (2026-10-20): build the look
  queue until he says stop.
- **Classification / tier:** feature, M–L. Marley crates, plus the settings content and the
  Settings page (Zed crates with existing Marley rows).
- **Recall (§18.3):**
  - F-claude-438-a: `has_notifications` must read the snapshot that `refresh` keeps. The badge
    count is computed from that same snapshot, never from state that only `render` writes.
  - L-claude-474: an occluding child ends its group's hover. The flyout is a deferred overlay
    with its own `on_hover`; the tile tracks its own.
  - L-claude-644: the rail forwards focus into the shown view; the popover's filter editor
    takes focus explicitly when it opens.
  - The brain (consultation f4468802aeff4032bf8b49ad663c6808): nothing on this seam.
- **Discovery:** an Explore map of the rail (this session):
  - Width: `DEFAULT_WIDTH` 260, `MIN_WIDTH` 180, `MAX_WIDTH` 600 (`rail.rs:104-106`); `width` and
    `width_set_by_user` (:135-137); `impl Sidebar for Rail` (:10820), `width()` :10821,
    `set_width()` :10825 (it clamps and mirrors to Zed's sidebar); saved through
    `serialized_state` (:10907) and restored at :10940.
  - `MultiWorkspace` draws the sidebar at `width()` with a 6 px resize handle
    (`multi_workspace.rs:2071-2120`) and observes the sidebar (:413), so a rail notify
    re-renders it.
  - Header `render_header` (:4867): window controls, the Projects | Threads switch, a spacer, and
    `render_add_project` (:5474). Filter `render_filter` (:5427), editor `filter_field` (:1272),
    `focus_filter` (:3577), `cancel` (:3592); `secondary-f` → `agents_sidebar::FocusSidebarFilter`
    in context `MarleyRail && !Picker` (`marley_workbench/keymap.json:297`).
  - Groups: `render_blocks` (:4906) walks `marley_rail::rail_rows`; `render_project_row` (:5505)
    clicks into `activate_workspace` or `open_closed_project`; the icon comes from
    `project_icons` via `header_icon` / `project_icon` (:666-690); `header_block` puts rows under
    their header.
  - Attention: `marley_rail::{terminal_attention, thread_attention_class, project_attention}`
    (`marley_rail.rs:1060-1100`), `Attention::NeedsYou`.
  - Settings: `MarleySettings` (`marley_workbench.rs:432`, `from_settings` :798); the bool
    pattern is `rail_containers`, which touches `settings_content/src/marley.rs:187`,
    `marley_workbench.rs:509/870`, `assets/settings/default.json:1844` and
    `settings_ui/src/marley_page.rs:101-122`.
  - Reduced motion: `App::reduce_motion()` (`gpui/src/app.rs:1328`), set from `reduce_motion`
    (`zed.rs:2091`).
- **Decisions:** D1 to D7 in the spec.

### Design
- **Setting:** `MarleySettingsContent.rail_shrunk: Option<bool>` with a doc ("Whether the rail is
  folded to a strip of project marks (#830)"). `MarleySettings.rail_shrunk: bool`, from
  `marley.rail_shrunk.unwrap_or(false)`. In `default.json`'s `marley` block,
  `"rail_shrunk": false` with a comment. A Layout section item "Shrink the Rail" in
  `marley_page.rs` (the section's array grows by one).
- **Action:** `ToggleRailStrip` in `marley_workbench`'s `actions!(marley, [...])`, registered on
  the rail. It writes the setting's negation with
  `settings::update_settings_file(fs, cx, |content, _| content.marley.get_or_insert_default()
  .rail_shrunk = Some(!current))`, where `fs` comes from the workspace's `app_state().fs`.
- **State in `Rail`:** `shrunk: bool` (what is drawn) and `strip_motion: Option<(Instant, bool)>`
  (the start and its direction). `cx.observe_global::<SettingsStore>` compares
  `MarleySettings::get_global(cx).rail_shrunk` with `shrunk`. On a change it sets `shrunk`, starts
  the motion unless `cx.reduce_motion()`, closes any flyout or popover, sets `threads.shown =
  false` (D7), and notifies. The observer is set in `Rail::new` and kept in the rail's
  subscriptions; `Rail::new` reads the setting so a restart starts shrunk.
- **Width:** `STRIP_WIDTH = px(60.)` and `STRIP_EASE = Duration::from_millis(180)`. `width()`
  returns the eased value between `STRIP_WIDTH` and `self.width`:
  `t = elapsed / STRIP_EASE` clamped to 1, eased `1 - (1 - t)^3`. Once `t` reaches 1 it returns
  the end value. `set_width` while `shrunk` returns without changing `width` (D2). In `render`,
  while the motion runs, `window.request_animation_frame()`, and the motion is cleared once it
  has ended.
- **Render:** `Rail::render` branches on `shrunk`: `render_strip(window, cx)` instead of the
  header, filter, inbox and rows. The root keeps its key context, actions and focus.
  - Top cell: the header's height (`platform_title_bar_height(window) + px(1.)`, bottom border),
    centered `IconButton("marley-rail-open", IconName::ThreadsSidebarLeftClosed)` with the tooltip
    "Open the Rail", dispatching `ToggleRailStrip`.
  - Search cell: `Tab::container_height` tall, `IconButton("marley-rail-strip-search",
    IconName::MagnifyingGlass)` with the tooltip for `FocusSidebarFilter`; its click opens the
    popover.
  - Tiles: for each `Row::Project` of `marley_rail::rail_rows`, `render_strip_tile(index, group,
    icon, cx)`, a 36 px rounded tile, scrolling as a `v_flex().id(..).overflow_y_scroll()`. The
    face is `img(icon).size_6()` or a `Label` of the uppercase first char on
    `cx.theme().players().color_for_participant(hash(name)).background`. Active ring:
    `border_2().border_color(accent)` when the group holds the window's active workspace (the same
    test `render_project_row` uses for its active state; the Code phase reads it there). Badge:
    absolute top-right, 16 px round, `status().conflict` fill, count text, when
    `needs_you_count > 0`. `.debug_selector`s `marley-rail-strip-tile-{index}` and
    `marley-rail-strip-badge-{index}`. Click: the header's own click path.
  - Hover: `.on_hover` on a tile sets `strip_flyout = Some(index)` and `tile_hovered`; the
    flyout's own `.on_hover` sets `flyout_hovered`; the flyout closes when both are false.
  - Flyout: `deferred(anchored().position(tile_bounds.top_right()))`. The tile's bounds come from
    `on_children_prepainted`, or from the element's own `canvas` prepaint stored in a
    `Rc<Cell<Bounds>>`; Code picks whichever the rail already uses. Its child is a 280 px
    `v_flex` on `elevated_surface_background` with border and shadow, holding the project's block
    from `render_blocks`. `render_blocks` is split so one project's block can be built by index:
    `render_project_block(index, ...)` serves both.
  - Search popover: `strip_search_open: bool`. Open from the button or `focus_filter` while
    `shrunk`, by setting the flag, focusing `filter_editor` and notifying.
    `deferred(anchored().position(strip top-right))`, 300 px wide, max 70 % of the window's
    height, holding `render_filter(cx)` and `render_rows(render_blocks(cx), cx)`. `cancel` while it
    is open closes it (and clears the filter as `cancel` does today); `confirm` closes it after
    acting. A click outside closes it (`on_mouse_down_out`).
- **`marley_rail`:** `pub fn needs_you_count(project: &ProjectSnapshot) -> usize`: the terminals
  and threads whose attention class is `NeedsYou`.
- **Header button:** in `render_header`, before `render_add_project`, `IconButton(
  "marley-rail-shrink", IconName::ThreadsSidebarLeftOpen)` with the tooltip "Shrink the Rail",
  dispatching `ToggleRailStrip`; shown in both views.
- **File manifest:**
  | File | Owner | Change |
  |---|---|---|
  | `crates/marley_workbench/src/rail.rs` | Marley | the strip, tiles, flyout, popover, motion, button |
  | `crates/marley_workbench/src/marley_workbench.rs` | Marley | the setting, the action |
  | `crates/marley_rail/src/marley_rail.rs` | Marley | `needs_you_count` |
  | `crates/settings_content/src/marley.rs` | Zed crate (Marley file) | `rail_shrunk` |
  | `assets/settings/default.json` | Zed asset | `"rail_shrunk": false` |
  | `crates/settings_ui/src/marley_page.rs` | Zed crate (Marley file) | the Layout item |
  | `docs/marley/zed-touchpoints.md` | docs | clauses on the three rows |
  | `script/e2e/830-the-rail-shrinks-to-a-strip.sh` | Marley | the scenario (Test) |

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001 | two projects; a click on `marley-rail-shrink` (its point read from the first shot) | `830-01-open`, `830-02-strip-night` |
| REQ-004/005 | `b`'s stand-in waiting (#542's FIFO), `a` active | `830-02-strip-night` |
| REQ-006 | pointer moved onto `b`'s tile | `830-03-flyout` |
| REQ-007 | pointer parked in the terminal, Ctrl F over the rail's focus, `wait` typed | `830-04-search` |
| REQ-004 | Escape; theme written to Marley Day | `830-05-strip-day` |
| REQ-003 | Marley quit (`ctrl-q`) and relaunched by the scenario on the same profile; `rail_shrunk` printed from the profile's settings | `830-06-restart` |
| REQ-002 | a click on `marley-rail-open` | `830-07-open-again` |
| REQ-008 | not reachable in a shot: motion. Read in the review of the diff, and the strip's shots taken after 180 ms | — |

If the scenario harness cannot relaunch Marley inside one run, REQ-003 is shown by a second
scenario run on a profile whose `settings.json` has `marley.rail_shrunk: true`, which starts in
the strip. The setting's write is shown by the printed file after the click.

### Risks
- The flyout's anchor position: gpui's `anchored` positions in window coordinates, so the tile's
  bounds must come from its prepaint. A wrong origin draws the flyout at the window's corner; the
  shot shows it.
- Ctrl F focus: `FocusSidebarFilter` reaches the rail only while it has focus or through the
  workspace's sidebar focus. In the strip, the filter editor lives in the popover, so focusing it
  must open the popover first.
- A strip wider than the tile, with the window's left controls (macOS traffic lights) in the top
  cell: on Linux the box draws none; macOS is shelved.
- The ease reads `Instant::now()` in `width()`, which `MultiWorkspace` calls inside its render.
  That is a read, not an update, so it is safe.

## Phase 2 — Code
- **Built:** `rail_strip.rs` (new, `#[path]` submodule of `rail`): `Strip` (shrunk, the ease's
  `Instant`, the flyout's project, `Hover { tile, flyout }`, the flyout's close task, the
  popover's flag, the painted bounds of tiles and the search button), `tick`, `width`,
  `follow_strip_setting`, `open_strip_search`/`close_strip_search`, `leave_flyout` (150 ms grace),
  `render_strip`, `render_strip_tiles`/`render_strip_tile`, `strip_badge`, `render_strip_flyout`,
  `render_strip_search`. In `rail.rs`: the `strip` field, the settings observer calls
  `follow_strip_setting`, `Sidebar::width` eases, `set_width` is ignored while shrunk, `render`
  ticks and branches, the header's Shrink button, `render_blocks(only)`, and `focus_filter`,
  `cancel`, `confirm` and focus-out handling for the popover. `marley_rail::needs_you_count`.
  The setting end to end: `settings_content` field, `MarleySettings.rail_strip: RailStrip`
  (`Open`/`Shrunk`), `default.json`, the Settings page's Layout item; the action
  `ToggleRailStrip` with `toggle_rail_strip` at the crate root beside `write_layout`.
- **Deviations:** `MarleySettings` holds a `RailStrip` enum, not a bool (clippy's
  `struct_excessive_bools`, as `RailContainers`); the hover flags became a `Hover` struct for
  the same lint. `toggle_rail_strip` lives at the crate root: a `pub` or `pub(crate)` function
  in the private `strip` module trips either `unreachable_pub` or `redundant_pub_crate`. The
  search popover opens from its button in the scenario (Ctrl F needs the rail's focus, which a
  click on the strip gives anyway).
- **Review of the diff:** found a real bug, the ease's frames were asked for only inside
  `render_strip`, so opening the rail would have jumped instead of easing. `Strip::tick` now runs
  at the top of `Rail::render` in both forms. (F block at Complete.)
- **Gate:** first run red on clippy only (11 findings in the new code, plus `too_many_lines` on
  `from_settings` from the three-line field); all fixed at source. Second run
  `script/gates.sh --diff` 17/17.

## Phase 3 — Test
- **Scenario:** `script/e2e/830-the-rail-shrinks-to-a-strip.sh` (sway, 1600×1000, debug build of
  the Code tree). The first run was red only in the scenario: the tile point was A's, "wait"
  matched no row, the settings printout's regex was mangled, and the quit waited on #550's
  question about b's working agent. Fixed in the scenario
  (`ask_before_ending_a_working_agent` false); the second run exited 0 with seven shots.
- `830-01-open`: the open rail's header now ends in add-project and the new Shrink button
  (REQ-001's starting point).
- `830-02-strip-night`: after the click, a 60 px strip: Open at the top, the search button, and
  tiles D (Default), A (an accent ring: the shown project) and B with a pink badge "1" (b's
  waiting stand-in). The printout read `rail_shrunk after the click: True` (REQ-001, REQ-003's
  write, REQ-004, REQ-005 met).
- `830-03-flyout`: the pointer on B. Beside the strip, a panel with b's header and its Claude Code
  row, "waiting · Write the README · Permission for Write: README.md" (REQ-006 met).
- `830-04-search`: the strip's search button, then `claude` typed. A popover beside the strip with
  the filter and b's matching row highlighted; the strip shows only the matching project's tile,
  since its tiles follow the same filter (REQ-007 met).
- `830-05-strip-day`: Marley Day. Light tiles, the ring in the Day accent, the badge in Day's pink
  (REQ-004 met).
- `830-06-restart`: after `zed: quit` and a relaunch on the same profile, Marley comes back in the
  strip (D, B; the handed-over `a` is not restored by a restart, as before) (REQ-003 met).
- `830-07-open-again`: Open clicked. The rail is back at its width with the filter, the groups and
  the Shrink button (REQ-002 met).
- REQ-008 (the ease): not reachable in a shot. Read in the diff: `Strip::width` eases 180 ms
  cubic-out from the toggle's `Instant`, and `tick` asks for frames in both forms. Under
  `reduce_motion` the motion is `None`, so it jumps.
- Seen in passing, pre-existing — not in scope: the box's own tmux sessions (`tessera`,
  `lotline`) show under Default in the scenario's rail (#808 reads the user's tmux server).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #830); the guide's rail section; the rail's note in
  `docs/marley_architecture/marley_workbench.md` (The strip); `docs/marley/look-plan.md` W1.1
  done; the touchpoint rows for `settings_content/src/marley.rs`, `default.json` and
  `marley_page.rs` checked against what shipped.
- **Knowledge:** F-claude-830-an-ease-asked-for-frames-only-in-one-of-its-two-forms-001,
  PR-claude-830-an-eased-value-asks-for-frames-where-every-form-renders-001,
  L-claude-830-a-crate-wide-function-from-a-private-submodule-lives-at-the-crate-root-001.
- **Brain:** decisions/the-rails-strip-is-the-setting-marleyrail_shrunk-eased-in-sidebarwidth
  (consultation f4468802; follow-up 2026-11-03).
- **Ticket** closed; pipeline archived; committed on `marley/workbench-shell`.
