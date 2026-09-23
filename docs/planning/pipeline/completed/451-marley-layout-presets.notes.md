# Zed's layout presets in the Marley layout — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-451-marley-layout-presets.md
- **Pipeline spec:** 451-marley-layout-presets.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint and split
  · [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-451 (W6b), split from #442; autonomous per Chad's goal "lets continue
  working on the remaining tickets".
- **Classification / tier:** bug, small; `marley_workbench` only.
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Split at promotion.** The ticket held three fixes; two need research of their own:
  - **A first terminal** is TICKET-455. `Workspace::is_restoring` is set inside the task
    `load_workspace` spawns (`workspace.rs:7857-7862`), a turn after the workspace exists, so it
    cannot tell the rail at `WorkspaceAdded` whether saved terminals are on their way.
  - **The docks across a round trip** is TICKET-456. A dock that gains the Agent Panel while it
    was visible shows it and forgets the panel it showed, and one that loses its active panel
    closes (`dock.rs:638-700`, `:895-918`); the fix needs per-workspace memory across both
    switches.
  - #451 keeps the presets, and the ticket file is renamed for it.
- **Recall (§18.3).**
  - #441's capture (`AD-claude-441-the-marley-layout-routes-terminals-without-touching-zed-001`)
    and #449's (`AD-claude-449-terminal-keys-catch-zeds-actions-and-the-keymap-waits-for-new-keys-001`):
    catching an action at the workspace's root routes the palette, the menus and bindings alike.
  - `PR-claude-input-handler-overlay-arms-must-stop-propagation-001`, in spirit: the listener
    stops propagation only when it handles the action.
  - Brain: consultation `f83cabb0f43649d69d5d388980b4972d`, nothing on this seam.

### Design
- **Where:** the crate root (`marley_workbench.rs`), which owns the layout: `init` gains a
  `cx.observe_new::<Workspace>` whose action renderer adds two `capture_action` listeners,
  `layout_preset::<UseClassicLayout>` and `layout_preset::<UseAgenticLayout>`, one generic
  handler.
- **The handler:** in the Marley layout, `cx.stop_propagation()` and
  `workspace.show_toast(Toast::new(NotificationId::unique::<LayoutPresets>(), …).on_click(
  "Use Zed's Layout", |window, cx| window.dispatch_action(UseZedLayout.boxed_clone(), cx)), cx)`.
  In the Zed layout it returns.
- **Message:** "Panel Layout presets belong to Zed's layout: the Marley layout places its own
  panels."
- **Manifest:** `title_bar` joins the dependencies for the two action types; it depends on no
  Marley crate.
- **File manifest.** Marley only: `crates/marley_workbench/src/marley_workbench.rs`,
  `marley_workbench_tests.rs`, `Cargo.toml`; docs at Complete: `CHANGELOG.md`,
  `docs/marley_architecture/marley_workbench.md`, `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: Marley layout; each preset dispatched from the center pane: the settings file is unwritten, `AgentSettings::dock` stays right, the workspace has the toast, a stand-in handler does not run |
| 002 | driven: the toast's button (through the notification view's `primary_on_click`, or dispatching `marley::UseZedLayout` as the button does) switches the layout |
| 003 | driven: Zed layout; each preset reaches a stand-in handler registered as Zed's is, and no toast shows |
| 004 | `script/gates.sh --diff` |

The tests register a stand-in for Zed's handler with `workspace.register_action` rather than
`title_bar::init`, which builds a title bar per workspace with globals these tests do not set.

### Risks
- **Focus.** A preset chosen from the title bar's menu dispatches from whatever had focus
  before the menu; with nothing focused the dispatch starts at the window's root, where
  neither this listener nor Zed's handler is, and nothing happens, as in Zed.

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] the listeners · [x] the toast · [x] `Cargo.toml` ·
  [x] tests · [x] review · [x] clippy at deny level · [x] coverage.
- **Built** in `crates/marley_workbench/src/marley_workbench.rs`: an `observe_new::<Workspace>`
  in `init` whose action renderer adds `layout_preset::<UseClassicLayout>` and
  `layout_preset::<UseAgenticLayout>`; `layout_preset` shows the toast (id `LayoutPresets`)
  in the Marley layout; `use_zed_layout`, the toast's button, calls `write_layout`.
- **Deviations from the design.**
  - `title_bar` joins the dev-dependencies too, with `test-support`: the crate's dev-dependencies
    turn on `remote`'s test support, whose `Mock` connection `title_bar` matches only under its
    own `test-support` (`title_bar.rs:617`).
  - REQ-002's test calls `use_zed_layout`, the function the toast's button runs, instead of
    clicking the button: the notification's primary button carries no debug selector to find it
    by. The button's wiring is the one `on_click` call in `layout_preset`.
  - Two `workspace::Workspace` paths in the test harness lost their qualification: the crate
    root imports `Workspace` now.
- **Review against the criteria.** The listener runs inside the workspace's update and touches
  only the workspace it is given (`show_toast`) and the settings; the button runs from the
  notification and only writes the setting. `show_toast` replaces an earlier toast with the same
  id, so repeated presets keep one. No Warp source, no Zed code carried over, no Zed crate
  changed.
- **Negative check.** Without the two listeners,
  `in_the_marley_layout_zeds_layout_presets_only_explain` fails.
- **Checks.** `cargo clippy -p marley_workbench --all-targets --all-features -- -D warnings`
  clean. `cargo nextest run -p marley_workbench` under coverage: 81 passed; 1807 of 1807 lines.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 · [x] REQ-002 · [x] REQ-003 · [x] the gate ·
  [x] the live drive (not run; why below).
- **Tests** (`marley_workbench_tests.rs`):

  | REQ | Test |
  |---|---|
  | 001 | `in_the_marley_layout_zeds_layout_presets_only_explain`: both presets from the center pane; the toast shows, the stand-in handlers never run, the Marley defaults hold |
  | 002 | `the_toasts_button_switches_to_zeds_layout`: `use_zed_layout`, the button's function, switches the layout |
  | 003 | `in_the_zed_layout_the_presets_reach_zed`: both stand-ins run, in order, and no toast shows |
  | 004 | `script/gates.sh --diff` |
- **Runs.** `cargo nextest run -p marley_workbench`: 81 passed. The three tests start no
  process and use no timer, so no flake loop.
- **Gate.** `script/gates.sh --diff`: 19 passed, 0 failed, `GATE GREEN [diff]`.
- **Live drive: not run.** It needs clicks in the title bar's menu. At 01:12 the desk looked as
  it has all night: stay-awake on, and his fullscreen Teams window on the workspace the
  headless output borrows. The drive is owed with the others.
- **Pre-existing failures:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket
  · [x] archive · [x] commit.
- **Documented.** `CHANGELOG.md` (Fixed); `docs/marley_architecture/marley_workbench.md` (the
  presets under the switch; the known limits, the right dock now pointed at #456);
  `docs/marley/workbench-shell.md` (W6b shipped, W6f and W6g queued). No Zed path changed.
- **Knowledge appended.**
  - `AD-claude-451-zeds-layout-presets-explain-themselves-in-the-marley-layout-001`
  - `L-claude-451-catching-a-zed-crates-actions-in-a-marley-test-001`
  - No `F-` block: Code and Test found no bug.
- **Brain.** Consultation `f83cabb0f43649d69d5d388980b4972d` closed with `brain decide`:
  `decisions/in-the-marley-layout-zeds-panel-layout-presets-explain-themselves-instead-of-rewriting-its-docks`,
  follow-up by 2026-10-07.
- **Closed and archived.** TICKET-451 in `tickets/closed/`; this pair in `pipeline/completed/`;
  TICKET-455 and TICKET-456 point at the completed spec.
