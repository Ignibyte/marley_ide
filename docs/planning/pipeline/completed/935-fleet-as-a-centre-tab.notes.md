# The Fleet tab with the snapshot beside the list — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-935-fleet-as-a-centre-tab.md
- **Pipeline spec:** 935-fleet-as-a-centre-tab.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, Fleet).
- **Classification / tier:** feature, S. Marley crate only (`marley_workbench`). The review sized
  it M because it read Fleet as a dock panel to be turned into a tab; #748 already made it one,
  so only the layout inside changes.
- **Recall (§18.3):**
  - AD-claude-608: the reads keep full detail only of selected agents; so nothing is selected for
    the user when the right column is empty (D4).
  - AD-claude-773 and #748: Fleet's tab is a `PanelTab` saved by the panel's name; the panel stays
    the dock's. D1 keeps that.
  - F-claude-747-a-panel-tab-read-its-panel-while-the-panel-updated-001: a hosted panel must not
    be read by its tab while the panel updates. This change stays inside `FleetPanel::render` and
    reads nothing of the tab.
  - F-claude-747-a-panel-hid-the-panes-drop-target-001: the tab draws its own drop layer; the
    measuring `canvas` must not block the mouse (it paints nothing and has no hitbox).
  - F-claude-865 (a hook called from a render never reads that view): the canvas's prepaint
    callback only writes a `Cell`; it reads no entity.
  - #608's scenario drags the split at measured places; `935-03-under` only shows the handle.
  - No failure or prevention rule names `fleet.rs`'s layout.
- **Discovery:** one file, `crates/marley_workbench/src/fleet.rs`: `FleetPanel` (a new
  `body_width` field and `Arrangement`), `FleetPanel::render` (the header and the choice),
  `render_beside` and `render_under` (the body), `render_handle` (used only under). `render_list`,
  `render_selected`, `render_snapshot` and `panel_tabs.rs` are unchanged.
- **Decisions:** D1–D5 in the spec.

### Design
- `enum Arrangement { Beside, Under }` with `fn for_width(width: Pixels) -> Self` against
  `const BESIDE_FROM: Pixels = px(760.)`. `FleetPanel` keeps `body_width: Rc<Cell<Option<Pixels>>>`
  and `arranged: Arrangement` (what the last frame drew); the first frame, with no width yet, draws
  `Under`.
- In `render`, the body (`marley-fleet-body`) gets a first child `canvas(move |bounds, window, _|
  { width.set(Some(bounds.size.width)); if Arrangement::for_width(bounds.size.width) != drawn {
  window.refresh(); } }, |_, (), _, _| {})` with `.absolute().size_full()`, as `rail_strip.rs`
  measures. `render` reads `body_width` to choose, stores it in `arranged`, and draws. The width
  does not depend on the arrangement, so the choice cannot swing back and forth.
- `render_beside(list, snapshot, cx)`: `h_flex().flex_1().min_h_0()` with two children, each
  `flex_1().min_w_0().h_full().overflow_y_scroll()` and an id (`marley-fleet-list` already on the
  list; `marley-fleet-snapshot` on the column); the snapshot column `border_l_1()` in
  `border_variant` and `bg(editor_background)`; without a snapshot, a muted XSmall line padded
  `p_2`.
- `render_under`: today's `v_flex` with `on_drag_move::<DraggedFleetSplit>`, the list, the handle
  and the snapshot at `relative(ratio)`.
- Header: `Headline::new("Fleet").size(HeadlineSize::Small)` and the `summary` label muted
  (`LabelSize::Small`), `px_3 py_2`, then the divider as now.
- File manifest: `crates/marley_workbench/src/fleet.rs` (Marley crate). No Zed path, so no
  `docs/marley/zed-touchpoints.md` row.

### Visual check plan
- Setup as #848's scenario: a scratch repository, `profile_setting marley.fleet
  '{"providers": [{"kind": "pseudo"}], "hosts": []}'`, `marley.tmux_hosts '[]'`, `open_path`.
- REQ-002: trust the project, `marley: toggle fleet`; `935-02-none` before any click: the header
  and the right column's line.
- REQ-001, REQ-003: click build-1's row (its place from the first run's shot, `BUILD_X`,
  `BUILD_Y`); `935-01-beside`: header, host cards on the left, build-1's snapshot (work item, run,
  resources, tokens) on the right.
- REQ-004: `pane: split right` with the Fleet tab in front (#747 moves the panel tab into the new
  pane); `935-03-under`: the half-width Fleet with the snapshot under the list and the handle.
  If the move leaves the Fleet in a pane 760 px or wider on the 1600 px output, split once more.
- REQ-001 in Day: taken before the split, right after `935-01-beside`:
  `profile_setting theme '"Marley Day"'`, `935-04-day`, then the theme back to Marley Night for
  the split.
- REQ-005: the diff review confirms `impl Panel`, `ToggleFleet`, `commands.rs` and
  `panel_group.rs` untouched; `935-01-beside` shows the tab titled "Fleet" opened by the action.
- Not reachable by the scenario: the Zed layout's dock. It takes the `Under` path by width, the
  same code `935-03-under` shows.

### Risks
- `window.refresh()` from a prepaint callback asks for another frame; if the measured width sits
  exactly at the threshold during a pane drag, each frame may redraw once. Acceptable: no state
  changes, and the choice is stable once the drag stops.
- The selection's snapshot column scrolls on its own; `render_snapshot`'s content is sized for a
  320 px dock and will look sparse in a wide column. That is #848's deferred look, not this slice.
- `pane: split right` on a panel tab: #747's `move_split` moves the tab; the scenario checks the
  shot shows Fleet in the right half, not an empty pane.

## Phase 2 — Code
- **Built:** `fleet.rs` only: `BESIDE_FROM`, `Arrangement` with `for_width`,
  `FleetPanel.body_width`, the measuring `canvas` behind the body, `render_beside`,
  `render_under` (#608's split, moved), and the header's `Headline` with the summary beside it.
- **Deviations:** none. The redraw on a changed arrangement is `window.refresh()` from the
  canvas's prepaint, since the panel's own `cx` is not there.
- **Review:** `render_list` and `render_selected` are unchanged; the `Panel` impl, `ToggleFleet`,
  `fleet_shows` and #748's tab are untouched (REQ-005). Before the first prepaint the body draws
  under, then beside at the next frame when wide.
- **Gate:** clippy clean at the first run. `just gate-diff`: 17 passed, 0 failed.

## Phase 3 — Test
- **Scenario:** `script/e2e/935-fleet-as-a-centre-tab.sh` (sway), run in full; every shot read.
- `935-02-none`: `marley: toggle fleet` opens the Fleet tab at full width: "Fleet" headline, "3
  agents on 2 hosts · Pseudo provider", the two host cards on the left, "Select an agent to see
  its snapshot here." in the right column. REQ-002, REQ-003, REQ-005.
- `935-01-beside`: build-1 clicked: its row lit, and its snapshot (header, work item, run strip,
  resources, tokens) in the right column. REQ-001.
- `935-03-under`: `pane: split right` moves the Fleet into the right half: the list over the
  snapshot with the split line between. REQ-004.
- `935-04-day`: in Marley Day, the Fleet opened again at full width, build-1 still selected, beside.
  The first run closed the split's items, which closed the Fleet too, and shot the terminal; the
  scenario opens the Fleet again before the shot. REQ-001, REQ-003.

## Phase 4 — Complete
- CHANGELOG entry; `docs/marley_architecture/marley_workbench.md` (the Fleet panel as cards);
  the look-plan row. No Zed path, so no touchpoint row.
- Brain: the consultation closed with `brain decide`.
