# Containers move to a panel on the right — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-673-containers-panel-on-the-right.md
- **Pipeline spec:** 673-containers-panel-on-the-right.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-06 item 4 (quoted in the spec); built 2026-10-07.
- **Classification / tier:** feature; `marley_workbench`, plus the setting's words in
  `settings_content`, `settings_ui` and `default.json` (rows exist).
- **Pre-flight:** green; #672 committed; cargo idle.
- **Recall (§18.3):**
  - #614: container rows stay out of the rail's keys and filter; `render_containers` draws them.
  - #669: `marley.rail_containers`, off in every run's copy; a scenario turns it back on.
  - #670: the fold and `marley_containers_open` in the saved blob; its scenario's fake `docker`
    and stand-in `docker-proxy`.
  - Fleet (#607): set_active can come for a panel in a closed dock; Fleet asks the windows. Here a
    stray watch only keeps the scan the rail runs anyway.
- **Discovery:** `render_containers` (`rail.rs:4337` before #672), `ContainersFold`,
  `toggle_containers`, `RailState`/`SavedRail`; `render_port_row`, `port_row_buttons` and `PortMenu`
  (tied to `Rail`); `row_card`, `row_label`, `RowLine`, `Cut`, `ROW_GROUP`, `port_snapshot`,
  `container_line`, `url_label`, `stop_words`, `container_refused_toast`; `ports::watch`,
  `unwatch`, `containers`, `container_at`, `stop_container`; `browser::open_url_tab`;
  `FleetPanel` and `KnowledgePanel`'s `Panel` impls; priorities 20 and 21 taken, so 22.

### Design
- **`rail_containers.rs`** (new, `pub mod containers` in `rail.rs` by `#[path]`): `init` registers
  `ToggleContainers` and adds a `ContainersPanel` to each workspace; the panel keeps its workspace,
  its focus handle, whether it watches the scan, and a `Ports` observation. `set_active` watches
  or unwatches; release unwatches. `Panel`: right only, 320px, `IconName::Box` and `enabled` while
  `marley_layout` and the setting is on, priority 22, `starts_open` false. Rows through `row_card`
  with hover buttons and a right-click menu of its own (the rail's are bound to `Rail`).
- **`rail.rs`**: `render_containers`, `ContainersFold`, `toggle_containers`, `Rail::containers`,
  `RailState::containers` and `SavedRail::marley_containers_open` go; `write_rail_state` stops
  writing it; the tests in the tree that name it follow.
- **`marley_workbench.rs`**: `ToggleContainers` beside `ToggleFleet`; `rail::containers::init`.
- **The setting's words**: `settings_content/src/marley.rs` doc, `default.json` comment,
  `settings_ui/src/marley_page.rs` title Containers Panel and its description; rows updated.
- **Guide**: the container ports article and the settings table.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001, 002 | a fresh window with the stand-in and the setting on | `673-01-rail` |
| REQ-003 | clicks the status bar's Box button | `673-02-panel` |
| REQ-004 | double-clicks :673's row | `673-03-browser` |
| REQ-005 | turns the setting off | `673-04-off` |

### Risks
- `enabled` false hides the button; a panel left open when the setting goes off closes with it,
  as the Knowledge panel does when Rusty goes off.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall; discovery.
- [x] Mint the pair; the ticket in progress; the backlog row removed.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request.

## Phase 2 — Code (2026-10-07)
### Built
- **`rail_containers.rs`** (new, `rail::containers`): `init`, `shown`, `ContainersPanel` with
  `open`, `stop`, `render_row`, `render_buttons`, `close_while_hidden`; `Render`, `Focusable`,
  `EventEmitter<PanelEvent>`, a manual `Debug` (its subscriptions have none), `Panel`.
- **`rail.rs`**: the Containers section, `ContainersFold`, `toggle_containers`, the fold's field in
  `Rail`, `RailState` and `SavedRail`, and its key in `write_rail_state` removed; `SavedRail`'s doc
  says an older blob's key is left unread; `RailContainers` no longer imported.
- **`rail_tests.rs`**: the three blob tests drop the fold, so they still build and still say what
  the blob holds.
- **`marley_workbench.rs`**: `ToggleContainers`; `rail::containers::init`; `RailContainers`'s
  words.
- **The setting's words**: `default.json`, `settings_content/src/marley.rs`,
  `settings_ui/src/marley_page.rs` (Containers Panel); their three touchpoint rows.
- **The guide**: the container ports item.

### Deviations
- **The panel closes itself when hidden.** The exploratory run turned the setting off with the
  panel open: the button went, the panel stayed, empty. Zed's dock does not drop a visible panel
  whose `enabled` turns false; the panel now closes its dock once from a render while hidden, as
  the Knowledge panel does, and REQ-005 says so.

### Review
- `close_while_hidden` defers the close to after the panel's own update (it reads the panel), and
  resets when the setting comes back on (`SettingsStore` observation).
- `set_active` and release balance `ports::watch` and `unwatch` through `watching`.
- The row's double-click and the buttons' clicks: the buttons stop propagation, so the row never
  opens twice.

### Gate
`just gate-diff` on the tree with the scenario: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/673-containers-panel-on-the-right.sh` (`compositor sway`), on the debug build: the
setting on, the fake `docker`, the stand-in `docker-proxy` for port 673, and the box's 13
containers.

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001, 002 | `673-01-rail` | the rail: the project and its terminal, no CONTAINERS section; the status bar's box button after the debugger's |
| REQ-003 | `673-02-panel` | after a click on the box (now blue): the right dock's CONTAINERS 14, `:673 docker`, `127.0.0.1:673/`, `→ 172.18.0.73:80` first, then the box's |
| REQ-004 | `673-03-browser` | after a double-click on :673: a Browser tab on `http://127.0.0.1:673/` (refused, nothing listens), its row in the rail; the panel still open |
| REQ-005 | `673-04-off` | `rail_containers` off: no box button, the panel closed, the Browser tab wider |

Nothing was stopped. The exploratory runs before the gate found the box button's x (1343, not
1421, which opened the outline panel) and the panel left open when the setting went off (fixed,
above). Focus: headless sway; Hyprland's one Marley window (Chad's) before and after, no rule
added.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed: Containers moved to a panel on the right); the in-app
  guide's container ports item (before the gate); `marley_workbench.md`'s container ports; the three
  touchpoint rows.
- **Knowledge:** `L-claude-673-a-panel-whose-enabled-turns-false-stays-open-001`,
  `AD-claude-673-the-machines-containers-live-in-a-right-dock-panel-001`.
- **Brain:** `decisions/marley-lists-the-machines-containers-in-a-right-dock-panel-opened-from-the-status-bar`.
- **Ticket:** closed; the pair archived.
