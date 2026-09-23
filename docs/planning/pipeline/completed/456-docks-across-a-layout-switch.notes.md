# Each dock as it was across a layout round trip — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-456-docks-across-a-layout-switch.md
- **Pipeline spec:** 456-docks-across-a-layout-switch.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-456 (W6g), split from #451; autonomous per Chad's goal "lets continue
  working on the remaining tickets".
- **Classification / tier:** bug, medium; `marley_workbench` only.
- **Pre-flight:** no active pipeline, README marker present, cargo idle, every tool present.
- **Minted:** no queued pair; the doc pair is new, and the BACKLOG row is gone.
- **Recall (§18.3).**
  - #438 inspect S9 found the bug; #451's notes traced it to `dock.rs:638-700` and `:895-918`.
  - `L-claude-449-driving-keys-and-docks-in-a-gpui-test-001`: `TestPanel` stands in for a
    panel, and its activation priority sets its place in the dock.
  - `L-claude-439-focus-lands-in-a-dock-panel-only-once-its-dock-is-open-001`: open a panel
    the way a user does (`focus_panel`) before testing what it shows.
  - Brain: consultation `19367dc4412e4fbe897370a365b82e9f`, nothing on this seam.
- **Discovery.** One Explore sweep of the dock move. The spec's Reference and Prior art hold its
  facts. The traced round trip:
  - Zed layout: left `[Agent*]` open, right `[X*, …]` open.
  - To Marley: the left loses its active panel and closes; the right opens on the Agent Panel,
    and X is forgotten.
  - Back: the right loses its active panel and closes with none active; the left opens on the
    Agent Panel.

### Design
- **Where:** `crates/marley_workbench/src/marley_workbench.rs`, in the layout observer, with
  the memory in the switch's `LayoutState` global.
- **The types:**
  - `DockShown { open: bool, panel: Option<&'static str> }`: a dock's state, the panel by its
    persistent name.
  - `LayoutState::displaced: HashMap<EntityId, [Option<DockShown>; 3]>`: per workspace, one
    slot per dock in `Workspace::all_docks` order.
- **`layout_setting_changed`:**
  - before `apply_defaults`, capture every workspace of every `MultiWorkspace` window: a weak
    handle, and per dock its `DockShown` and whether it holds the Agent Panel
    (`Dock::has_agent_panel`);
  - after the sidebar swap, `cx.defer(move |cx| settle_docks(captured, cx))`.
- **`settle_docks`,** per captured workspace still alive, inside its window's update, when the
  Agent Panel's dock changed from `before` to `after`:
  - **remember:** the `after` dock now shows the Agent Panel, and before it showed another
    panel → store its `DockShown` before the move;
  - **give back:** the `before` dock has a memory and showed the Agent Panel before the move →
    activate the remembered panel (`panel_index_for_persistent_name`, `activate_panel`) and
    `set_open(memory.open && it was open before the move)`; the memory goes either way.
  - It prunes the memory of workspaces that are gone.
- **File manifest.** Marley only: `crates/marley_workbench/src/marley_workbench.rs` and
  `marley_workbench_tests.rs`. Docs at Complete: `CHANGELOG.md`,
  `docs/marley_architecture/marley_workbench.md`, `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven, in `marley_workbench_tests.rs`: a window with an Agent Panel (`AgentPanel::test_new`) open on the left and a `TestPanel` (priority 5) open on the right, in the Zed layout; to Marley and back: the right open on the `TestPanel`, the left open on the Agent Panel. The same with the right dock closed before the trip: closed, `TestPanel` active |
| 002 | driven: in the Marley layout, a second `TestPanel`-like panel shown on the right; back to Zed: the right shows it |
| 003 | driven: in the Marley layout, the right dock closed; back to Zed: closed, the first panel active |
| 004 | driven: the left dock closed before the trip: the right unchanged through both switches |
| 005 | `script/gates.sh --diff` |

### Risks
- **Observer order.** The capture relies on the switch's observer running before the docks',
  which holds because `init` registers it before any window. The first test shows it either
  way: a capture taken after the move sees the moved state and remembers nothing.
- **Two test panels.** Every `TestPanel` has the persistent name "TestPanel", so the REQ-002
  test needs a second panel with a name of its own, or the two must be told apart another way.
  The Code phase picks one, for example Zed's `TestPanel` for one and the Agent Panel's
  neighbor from the harness for the other.
- **Decisions for the brain at Complete:** D1 (Zed's move kept, the memory from outside) and
  D3 (the user's choice in between stands).

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] the memory · [x] the capture · [x] the settle ·
  [x] tests · [x] fmt · [x] clippy · [x] review.
- **Built** (`crates/marley_workbench/src/marley_workbench.rs`):
  - `DockShown`, `DocksBefore`, and `LayoutState::displaced`;
  - `layout_setting_changed` notes every workspace's docks with `docks_before` before
    `apply_defaults`, then defers `settle_docks`;
  - `settle_docks` settles each live workspace in its window and prunes the memory of dead or
    empty entries;
  - `settle_workspace` remembers and gives back as designed.

  The iterator chains keep a window that is not a `MultiWorkspace`, and a workspace dropped in
  between, off lines of their own that no test reaches.
- **Tests** (`crates/marley_workbench/src/marley_workbench_tests.rs`): `open_with_docks` builds a
  window in the Zed layout. It has the Agent Panel open on the left and two `TestPanel`s on the
  right (priorities 5 and 6), the first shown. The five tests read the docks by panel id, since
  both `TestPanel`s share the persistent name "TestPanel".
- **Deviations.** None from the design.
- **Review.**
  - Compile: the crate root now imports `AgentPanel`, which made two qualified paths in the
    harness redundant (`unused_qualifications`). And `workspace::dock::DockPosition` imported
    into the tests shadowed the `settings::DockPosition` the older tests use; it is named in
    full at its one use.
  - clippy: four `semicolon_if_nothing_returned` in closures.
  - Re-entrancy: `settle_docks` runs in a `cx.defer`, and each workspace settles in
    `AnyWindowHandle::update`, which leases no root. Only a dock is updated.
  - The observer order is shown by the tests themselves: a capture taken after the move would
    remember nothing, and the round trip would fail.
- **So far:** clippy clean; `cargo llvm-cov nextest -p marley_workbench`: 125 of 125 passed,
  100% of lines and functions.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-004 tests · [x] negative checks ·
  [x] the crate suite · [x] the live drive (not run; why below) · [x] the gate.
- **A late change, from Phase 2's closing review.** The "remember" rule first required the move
  to have made the Agent Panel the dock's shown panel. That forgot the panel when the Agent
  Panel arrived hidden and the user showed it there later, and the way back then closed the dock
  empty. The give-back condition already guards the user's choices, so the entered dock now
  remembers whenever the Agent Panel enters it. The spec says so, and
  `a_dock_the_agent_panel_was_shown_in_later_gets_its_panel_back` pins it.
- **Tests:** six driven tests in `crates/marley_workbench/src/marley_workbench_tests.rs`.

  | REQ | Tests |
  |---|---|
  | 001 | `a_round_trip_leaves_each_dock_as_it_was`, `a_dock_closed_before_the_trip_comes_back_closed_on_its_panel`, `a_dock_the_agent_panel_was_shown_in_later_gets_its_panel_back` |
  | 002 | `a_panel_chosen_during_the_trip_stays_chosen` |
  | 003 | `a_dock_closed_during_the_trip_stays_closed` |
  | 004 | `a_hidden_agent_panel_moves_without_moving_anything_else` |
  | 005 | the gate |
- **Negative checks**, the file restored and checked by checksum:
  - the deferred settle removed: the round trip fails with the right dock `(false, None)`,
    which is the reported bug;
  - given back even when the Agent Panel was not what the dock showed: the user's chosen panel
    is overridden (`left: …52`, `right: …53`);
  - opened again even when the user closed it: the close is undone (`left: (true, …)`).
- **Run:** `cargo nextest run -p marley_workbench`: 126 tests run, 126 passed.
- **Gate.** The first `script/gates.sh --diff` was red at gate:2. The late test had a
  `semicolon_if_nothing_returned`, since clippy was not rerun after it was written. After the
  fix: 19 passed, 0 failed, `GATE GREEN [diff]`. gate:3 ran 428 tests; gate:4 ran 126, with
  100% of lines and functions. The receipt matches the tree (`1131f87e…`).
- **Live drive: not run.** Chad is at the desk: his windows are on ws3, which the headless
  output borrows, and the session is not idle. The drive needs clicks, so it is owed with W3 to
  W6's.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket ·
  [x] archive · [x] commit.
- **Documented:**
  - `CHANGELOG.md`, under Fixed: the docks across a layout round trip;
  - `docs/marley_architecture/marley_workbench.md`: the docks across a switch, 29 tests in
    `marley_workbench_tests.rs` (the old count of 20 was stale), and the known limits;
  - `docs/marley/workbench-shell.md`: W6g shipped.

  No Zed path changed, so there is no `zed-touchpoints.md` row.
- **Knowledge:**
  - `F-claude-456-a-layout-round-trip-closed-the-right-dock-and-lost-its-panel-001`;
  - `AD-claude-456-a-layout-round-trip-gives-each-dock-back-its-panel-001`;
  - `L-claude-456-acting-around-zeds-own-settings-observers-001`.
- **Brain:** consultation `19367dc4412e4fbe897370a365b82e9f` closed as
  `decisions/a-layout-round-trip-gives-each-dock-back-its-panel`, with a follow-up by
  2026-10-07.
- **Ticket:** #456 is closed, and no BACKLOG row is stale.
