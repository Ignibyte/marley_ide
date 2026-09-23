# New Agent from the keyboard — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-450-new-agent-from-the-keyboard.md
- **Pipeline spec:** 450-new-agent-from-the-keyboard.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint the pair ·
  [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-450, split from #449 at its promotion; autonomous per Chad's goal "lets
  continue working on the remaining tickets".
- **Classification / tier:** feature, medium; `marley_workbench` plus one hunk and one test in
  `crates/zed/src/zed.rs`.
- **Pre-flight:** no active pipeline, README marker present, tools present; another project's
  cargo held the box during planning, and no cargo was needed.
- **Recall (§18.3).**
  - `PR-claude-new-chord-shadowed-by-hardcoded-key-001`: the chord sweep in the spec checks
    every context of every keymap Zed ships, and Hyprland's configuration.
  - `PR-claude-unmodified-terminal-chords-yield-to-the-pty-001`: `ctrl-alt-n` is modified. A
    TUI in a Marley terminal that uses `C-M-n` (Emacs's `forward-list`) loses it to the
    Workspace binding, as it loses Zed's other Workspace chords.
  - `L-claude-440-testing-agent-clis-without-a-pty-001`: display-only terminals, a search path
    in a temporary directory, and the write log; no test writes `claude` into a real shell,
    since the real Claude Code is on the dev box's `PATH`.
  - Brain: consultation `5bac9db68beb47e19ec5491c30e9e3a1`, nothing on this seam.
- **Minted:** no queued spec existed. BACKLOG's 450 row is removed; the ticket is in progress.

### Design
- **`crates/marley_workbench/src/agents.rs`** (`pub mod agents`, tests in `agents_tests.rs`
  through `#[path]`): what Marley can start and how, shared by the rail and the picker.
  - `Launcher { search_path: Option<OsString>, terminal_factory: TerminalFactory }`, a gpui
    global with a `Default` of the process's `PATH` and `Project::create_terminal_shell`, read
    through `launcher(cx)` (the default when no global is set). It replaces the rail's two
    fields; tests set it.
  - Moved from `rail.rs`: `AgentIcon`, `agent_icon` and `agent_choices` (Zed's agents),
    `agents_on_path` and `agent_icon_name` (the CLIs). `agent_choices` becomes `thread_agents`
    and `agents_on_path` becomes `installed_clis`, returning what they return today.
  - `start_thread(workspace: &mut Workspace, agent, window, cx) -> anyhow::Result<()>` and
    `start_cli(workspace: &mut Workspace, kind, window, cx)`: the bodies of
    `Rail::new_agent_thread` and `Rail::new_agent` after their `activate_workspace`, which stays
    in the rail. `start_cli` makes one `add_center_terminal` call with the launcher's factory
    and the same handshake, timeout and `detach_and_prompt_err` as today.
  - `init(cx)`: on every new workspace, `register_action` for `marley::NewAgent`, which returns
    at once while `DisableAiSettings` says AI is off and otherwise calls
    `workspace.toggle_modal` with a `NewAgentPicker`.
  - `NewAgentPicker`, a `ModalView` around `Picker<NewAgentDelegate>` (`Picker::uniform_list`).
    The delegate holds the workspace's weak handle, the choices (`thread_agents` for the
    workspace's project, then `installed_clis`), the matches and the selected index.
    - `update_matches`: every choice in order for an empty query, else `fuzzy::match_strings`
      over the names on the background executor.
    - `render_match`: a `ListItem` with the agent's icon, the name with its match highlights,
      and a muted end label, "Thread" or "Terminal", since Claude Code can be both.
    - `confirm`: the workspace runs `start_thread` or `start_cli` (an error to a prompt), and
      the modal is dismissed; `dismissed` emits `DismissEvent`.
- **`NewAgent`** joins `UseMarleyLayout` and `UseZedLayout` in the crate root's
  `actions!(marley, …)`; its doc comment is the palette's description. The `marley` namespace
  is already in `zed.rs`'s `test_action_namespaces` list.
- **The keymap.** `crates/marley_workbench/keymap.json` (`include_str!`):
  `[{ "context": "Workspace", "bindings": { "secondary-alt-n": "marley::NewAgent" } }]`.
  In the crate root, `pub fn load_keymap(cx)` calls `load_keymap_from(KEYMAP, cx).log_err()`.
  `load_keymap_from` maps `KeymapFile::load`'s three results: bindings tagged
  `KeybindSource::Default` and bound, or an error naming the failure. The input is a parameter
  so a test can feed it a broken keymap (REQ-007).
- **`crates/zed/src/zed.rs`.** At the end of `load_default_keymap`, after `specific-overrides`:
  a `// Marley:` comment and `marley_workbench::load_keymap(cx);`. A test beside
  `test_disable_ai_filters_keybindings`, on `init_keymap_test`: `load_default_keymap` binds
  `marley::NewAgent`, and after `reload_keymaps` with no user bindings it is bound again. Its
  row goes into `docs/marley/zed-touchpoints.md` before the edit (the hook enforces it).
- **`rail.rs`** keeps its menu and its `activate_workspace`, and calls `agents::` for the
  choices, the icons, the launcher and the two launches. Its tests set the launcher global in
  place of the two fields (four sites).
- **Manifest.** `picker` and `fuzzy` join the dependencies; `menu` joins the dev-dependencies if
  the tests dispatch `menu::Confirm`.
- **File manifest.**
  - Marley: `crates/marley_workbench/src/agents.rs`, `agents_tests.rs` (new);
    `marley_workbench.rs` (the action, `load_keymap`, `mod agents`, `agents::init`);
    `rail.rs`, `rail_tests.rs`; `marley_workbench_tests.rs` (the display-only terminal helpers
    move there from `rail_tests.rs`, so both test files use them); `keymap.json`;
    `Cargo.toml`.
  - Zed: `crates/zed/src/zed.rs` (the hook and its test).
  - Docs at Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_workbench.md`,
    `docs/marley/workbench-shell.md` (W5c, D4, D7, the touchpoint table),
    `docs/marley/zed-touchpoints.md` (the row, written during Code).

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: a project with one configured external agent and a search path holding `claude` and `codex`; `NewAgent` from the center opens the picker listing Zed Agent, the external agent, Claude Code, Codex, in that order, with their end labels |
| 002 | driven: an `AgentPanel::test_new` in the project (the #439 harness); confirming Zed Agent closes the picker and opens a new thread, focused, in that panel |
| 003 | driven: the launcher's factory makes display-only terminals; confirming Claude Code closes the picker and adds one center terminal whose write log is `claude\r` |
| 004 | driven: the query `cod` leaves Codex alone; the empty query lists everything again |
| 005 | driven: with `disable_ai` on, `NewAgent` opens no modal |
| 006 | driven: after `load_keymap`, pressing `ctrl-alt-n` opens the picker; with a user binding on `ctrl-alt-n` bound after it, pressing it runs the user's action and opens nothing |
| 007 | unit: `load_keymap_from` returns an error, and binds nothing, for text that is not JSON and for a keymap naming an unknown action |
| 008 | `crates/zed/src/zed.rs`: `load_default_keymap` binds `marley::NewAgent`, and `reload_keymaps` binds it again |
| 009 | the rail's #439 and #440 tests, unchanged but for setting the launcher |
| 010 | review; gate:16; `script/gates.sh --diff` |

### Risks
- **The rail's move onto `agents`.** The rail's menu and launches must behave as before; its
  #438, #439 and #440 tests are the check, run unchanged but for the seam.
- **Global seam in tests.** A test that sets no launcher gets the real `PATH` and real shells,
  as the rail's tests do today. Every test that confirms a CLI sets a display-only factory,
  so no agent really starts.
- **The zed crate's tests.** Touching `zed.rs` puts `crates/zed` in the gate's scope, so the
  gate runs its test suite; the #438 slice did the same.
- **`Picker` focus in tests.** Actions reach the picker only while it has focus; the tests
  confirm through the delegate or dispatch from the picker's focus handle, whichever the
  harness supports (W3's lesson: menus focus on a platform frame).

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] ledger row · [x] `agents.rs` · [x] crate root ·
  [x] `keymap.json` · [x] `rail.rs` · [x] `Cargo.toml` · [x] `zed.rs` and its test · [x] tests ·
  [x] review · [x] clippy at deny level · [x] coverage.
- **Built.**
  - `crates/marley_workbench/src/agents.rs`, as designed: `Launcher` and `launcher`,
    `AgentIcon`, `installed_clis`, `cli_icon`, `thread_icon`, `thread_agents`, `start_thread`,
    `start_cli`, `init`, the `NewAgent` handler and the picker (`NewAgentPicker`,
    `NewAgentDelegate`, `Choice`, `Start` with `place` and `run`).
  - The crate root: `pub mod agents`, `NewAgent` in `actions!(marley, …)`, `agents::init`,
    `load_keymap` and `load_keymap_from`; `crates/marley_workbench/keymap.json`.
  - `rail.rs` on `agents`: its two fields, the factory type, the timeout, `AgentIcon` and the
    four helpers left; `new_agent` and `new_agent_thread` show the workspace and call
    `agents::start_cli` and `agents::start_thread`.
  - `crates/zed/src/zed.rs`: `marley_workbench::load_keymap(cx)` last in
    `load_default_keymap`, and `test_reload_keymaps_binds_the_marley_keymap`; the ledger row
    for `zed.rs` extended first (one row per path).
  - Tests: `agents_tests.rs`, nine; two loader tests in `marley_workbench_tests.rs`, where the
    display-only terminal helpers, `programs_in`, `search_agents_in`,
    `use_display_only_terminals`, `add_agent_panel` and `configure_agents` now live for both
    test files. The rail's CLI icon test moved to `agents_tests.rs`.
- **Deviations from the design.**
  - Beyond the plan, `a_query_that_matches_nothing_starts_nothing` and
    `the_arrow_keys_pick_another_agent`; the second reaches `set_selected_index`, which no
    other test moves.
  - The key test binds a test action of its own for the user's binding, so it sees the
    user's action run and not just the picker stay shut.
- **Review against the criteria.**
  - Re-entrancy: the `NewAgent` handler runs inside the workspace's update and reads only the
    project, the agent servers and the settings; `confirm` runs inside the picker's update and
    updates the workspace and the modal, neither leased then. Every path is driven through
    real dispatch (`menu::Confirm`, `menu::SelectNext`, the chord).
  - Errors: a start that fails reaches a prompt (`detach_and_prompt_err`); a Marley keymap
    that does not load is logged and binds nothing. It is compiled in, and a unit test loads it.
  - Provenance: no Warp source. The moved code is Marley's own (#439, #440). The delegate is
    written from `PickerDelegate`'s contract; its empty-query branch lists every choice in
    order because `fuzzy::match_strings` sorts by score, the idiom that API calls for, not a
    copied body.
  - Upstream discipline: in `zed.rs`, one call with a `// Marley:` comment and one test with its
    own; the row names both.
- **Negative checks.** Without the `zed.rs` line, `test_reload_keymaps_binds_the_marley_keymap`
  fails; without the AI guard, `with_ai_disabled_new_agent_opens_nothing` fails.
- **Checks.** `cargo clippy -p marley_workbench --all-targets --all-features -- -D warnings`
  clean. `cargo nextest run -p marley_workbench`: 67 passed before the arrow-key test, which
  passes too. `cargo nextest run -p zed`: 93 passed, 1 skipped by Zed. The rail's tests pass
  unchanged but for the seam (REQ-009).

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-009 · [x] the gate · [x] the live drive
  (not run; why below).
- **Tests.**

  | REQ | Test |
  |---|---|
  | 001 | `agents::tests::the_picker_lists_zeds_agents_then_the_installed_clis`: Zed Agent, stub, Zeta Code (the registry's name and SVG icon), Claude Code, Codex, with their places; the picker has focus |
  | 002 | `choosing_a_zed_agent_starts_a_thread_in_the_projects_panel` (`menu::Confirm`) |
  | 003 | `choosing_an_agent_cli_starts_it_in_a_new_center_terminal`: the write log is `claude\r` alone; `the_arrow_keys_pick_another_agent`: `menu::SelectNext` four times, then `codex\r` |
  | 004 | `a_query_leaves_only_the_agents_it_matches`; `a_query_that_matches_nothing_starts_nothing` |
  | 005 | `with_ai_disabled_new_agent_opens_nothing` |
  | 006 | `the_marley_keymap_opens_the_picker_and_a_users_binding_wins` (Linux keys) |
  | 007 | `the_marley_keymap_binds_new_agent_as_a_default`; `a_marley_keymap_that_fails_to_load_binds_nothing` |
  | 008 | `zed::tests::test_reload_keymaps_binds_the_marley_keymap` |
  | 009 | the rail's tests, on the launcher seam |
  | 010 | `script/gates.sh --diff`, gate:16 |
- **Runs.** `cargo nextest run -p marley_workbench`: 68 passed. The eleven agent and keymap
  tests five times in a row: 11 passed each time. `cargo nextest run -p zed`: 93 passed, 1
  skipped by Zed (Phase 2).
- **Gate.** `script/gates.sh --diff`: 19 passed, 0 failed, `GATE GREEN [diff]`. Coverage
  1717 of 1717 lines in `marley_workbench` (`agents.rs` 310 of 310).
- **Live drive: not run.** It needs keys. At 00:38 the desk looked as it did for #441 and
  #449: stay-awake on, so nothing shows Chad away, and his fullscreen Teams window on the
  workspace the headless output borrows. The key test presses the chord through the real
  Marley keymap in its place; the drive is owed with the others to the next headless session
  while he is away.
- **Pre-existing failures:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket
  · [x] archive · [x] commit.
- **Documented.** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (the
  intro, the rail's New Terminal and `+` menu on the launcher, a section for the agents module
  and the key, the tests); `docs/marley/workbench-shell.md` (W5c shipped, the W5 row, D4, D7,
  the touchpoint row's lines). The `zed.rs` row in `docs/marley/zed-touchpoints.md`, written
  in Code, names both hunks as shipped; gate:16 passed.
- **Knowledge appended.**
  - `AD-claude-450-new-agent-is-a-picker-behind-the-marley-keymap-001`
  - `L-claude-450-driving-a-picker-and-a-keymap-in-a-marley-test-001`
  - No `F-` block: Code and Test found no bug.
- **Brain.** Consultation `5bac9db68beb47e19ec5491c30e9e3a1` closed with `brain decide`:
  `decisions/new-agent-from-the-keyboard-is-a-picker-behind-the-marley-keymaps-secondary-alt-n`,
  follow-up by 2026-10-07.
- **Closed and archived.** TICKET-450 in `tickets/closed/`; this pair in `pipeline/completed/`.
