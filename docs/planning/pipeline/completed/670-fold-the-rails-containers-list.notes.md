# Fold the rail's Containers list — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-670-fold-the-rails-containers-list.md
- **Pipeline spec:** 670-fold-the-rails-containers-list.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** Chad, 2026-10-06: "the containers showing up on the left are huge in number. if
  containers are persistent there in that pane we need to put them in a dropdown list that is
  collapsible".
- **Classification / tier:** feature; the rail (`marley_workbench`), no Zed crate.
- **Pre-flight:** green; no active pipeline; cargo idle.
- **Recall (§18.3):**
  - #614's notes: the rows stay out of the keys and the filter; `render_containers` is their only
    drawer.
  - #632: the Harness section folds on a header click (`Harness::toggle_folded`), in memory only.
  - #601: the rail keeps its own fields in Zed's saved sidebar blob and serializes through
    `MultiWorkspace::serialize`, from inside its own listeners (`groups_changed`,
    `toggle_expanded`).
  - The brain (consultation `8a3e6e450c244bc79699872f7e0da8a1`): nothing on this seam.
- **Discovery:** `rail.rs` `render_containers` (`:4322`), `RailState`/`SavedRail`/
  `read_rail_state`/`write_rail_state` (`:6112-6150`), `rail_state` (`:1070`), the rail's
  construction from the sidebar's blob (`:810-845`), `restore_serialized_state` (`:8680`); the
  project header's `Disclosure` (`:4583`); `quit_marley`/`launch_marley` in `script/e2e.sh`.

### Design
- **`rail.rs`** (Marley crate): `Rail::containers_open`; `RailState::containers_open`,
  `SavedRail::marley_containers_open` (serde default false), read and written beside
  `marley_rail_closed`, taken in construction and `restore_serialized_state`;
  `toggle_containers` flips it, serializes the window and notifies; `render_containers` draws a
  clickable header (`Disclosure` whose click stops there and toggles, CONTAINERS, the count) and
  the rows only while open.
- **`script/e2e/670-…`**: the scenario.
- **File manifest:** `crates/marley_workbench/src/rail.rs`; the scenario; docs.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001, 002, 004 | Starts a fresh window with the stand-in and the setting on | `670-01-folded` |
| REQ-003 | Clicks the header, then the chevron | `670-02-open`, `670-03-folded` |
| REQ-005 | Opens it, quits Marley, starts it again | `670-04-restored` |

### Risks
- **The chevron's click reaching the header too** would fold twice, a no-op: the chevron stops
  the click, and `670-03-folded` shows one fold.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request.

## Phase 2 — Code (2026-10-06)
### Built
- **`rail.rs`**: `ContainersFold { Folded, Open }` (default `Folded`, `toggled`); `Rail::containers`
  and `RailState::containers`, from and to `marley_containers_open` in the window's saved blob,
  taken at construction and in `restore_serialized_state`; `toggle_containers` flips it, calls
  `MultiWorkspace::serialize` and notifies; `render_containers` draws a clickable header (a
  `Disclosure` whose click stops there, CONTAINERS, the count) and the rows only while open.
- **`rail_tests.rs`**: the three tests that build or compare the saved blob carry the new field and
  key, so they still compile and still say what the blob holds.
- **The guides** (`guide/index.html`, `docs/marley/guide.md`).

### Deviations
- **An enum, not a `bool`**: a fifth bool in `Rail` trips `struct_excessive_bools` (L-669).

### Review
- The serialize from inside the rail's listener is the one `toggle_expanded` and `groups_changed`
  already make: `MultiWorkspace::serialize` spawns, so it reads the rail after the update ends.
- The header's click and the chevron's each fold once: the chevron stops its click.

### Gate
`just gate-diff` on the final tree (the scenario's fix included): GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
### The scenario
`script/e2e/670-fold-the-rails-containers-list.sh` (`compositor sway`), on the debug build: the
setting on, a fake `docker`, a stand-in `docker-proxy` for port 670, and the box's 13 containers.

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001, 002, 004 | `670-01-folded` | a fresh window: "› CONTAINERS 14" alone, no rows |
| REQ-001, 003 | `670-02-open` | after a click on the header: the chevron down, `:670 docker → 172.18.0.70:80` first, then the box's |
| REQ-002, 003 | `670-03-folded` | after a click on the chevron: folded, once |
| REQ-005 | `670-04-restored` | unfolded, quit, started with no path: open, the same rows |

### Fix
- The first run's `670-04-restored` came back folded: the relaunch passed the scenario's path,
  which Zed answers as an open request in a new window rather than a restore (L-601). The scenario
  now relaunches with `open_path ""`, as #601, #602 and #605 do; the second run's shot is open. No
  code changed.

### Focus
Headless sway: Hyprland's one Marley window (Chad's) before and after, no rule added, sway stopped
with the run's Marley; nothing was stopped.

## Phase 4 — Complete (2026-10-06)
- **Documented:** `CHANGELOG.md` (Added: the rail's Containers list folds); the architecture note's
  container ports; the in-app guide and `docs/marley/guide.md`. No Zed path touched.
- **Knowledge:** `L-claude-670-a-restore-scenario-relaunches-with-no-path-001`,
  `AD-claude-670-the-containers-list-starts-folded-and-each-window-keeps-its-fold-001`.
- **Brain:** consultation `8a3e6e450c244bc79699872f7e0da8a1` closed with
  `decisions/marleys-containers-list-starts-folded-and-each-window-keeps-its-fold`.
- **Ticket:** closed; the pair archived.
