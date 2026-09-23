# `marley_workbench`

The Marley layout, written in the fork for the workbench shell's W2 (#438): the
`marley.layout` setting, the switch between Zed's sidebar and the rail, and the rail itself.
MIT OR Apache-2.0. It links Zed's GPL crates (`workspace`, `sidebar`, `terminal_view`,
`recent_projects` and others), so it builds and ships only as part of the fork
(AD-claude-438-marley-crates-may-link-zeds-gpl-crates-001).

Its lint table is rustal's (CONSTITUTION §14, #447), with two allows that gpui calls for:
`future_not_send`, since gpui's test and async contexts are not `Send`, and `unused_results`,
since gpui's registration calls return `&mut App` for chaining and `.log_err()` returns an
`Option`.

## The switch (`src/marley_workbench.rs`)

- `MarleySettings` reads `marley.layout`: `zed` by default, or `marley`. The settings block is
  `settings_content::MarleySettingsContent`.
- `init`, the first line of `crates/zed`'s `initialize_workspace`, reads Zed's own
  `terminal.button` and `agent.dock` defaults before anything patches them, applies the current
  layout's defaults, observes the settings store and registers `marley::UseMarleyLayout` and
  `marley::UseZedLayout`. The actions write the setting to the user's settings file, and write
  nothing when the layout already matches.
- `register_sidebar` builds and registers a window's sidebar for the current layout. `crates/zed`
  calls it where it used to build Zed's sidebar; the settings observer calls it for every window
  when the layout changes.
  - Into the Marley layout: the rail keeps Zed's sidebar and whether it was open. The window's
    sidebar opens, unless AI is off, when no sidebar is drawn at all.
  - Back to the Zed layout: the kept sidebar is registered again and opened or closed as it was.
    A window that opened in the Marley layout gets a new Zed sidebar with the saved state the
    rail kept for it.
  - A sidebar that held focus hands it to its replacement, and Zed's thread-switcher overlay is
    cleared.
- In the Marley layout the defaults are `terminal.button: false` and `agent.dock: right`,
  patched below the user's settings, so a user value wins in both layouts.

## The rail (`src/rail.rs`)

- It implements `workspace::Sidebar`, so the `MultiWorkspace` keeps the resize handle, the open
  state, persistence and the toggle actions.
- It follows every workspace and every listed terminal view. On each event it rebuilds the
  snapshot and redraws only when the pure `marley_rail::RailSnapshot` changed. `render` reads
  nothing but the rail, so a background terminal's output never redraws the window through it,
  and `has_notifications` reads the stored snapshot, so it holds while the rail is closed.
- It lists the groups that have an open workspace, named through Zed's public functions
  (`compute_disambiguation_details`, `ProjectGroupKey::display_name`).
- A project header has a chevron to fold it, a `+` menu with New Terminal and an attention dot;
  a terminal row has the title, the working directory and a bell dot. A header click shows the
  project. A terminal row click shows its project, activates and focuses the terminal and
  clears its bell. New Terminal starts where Zed's own would
  (`terminal_view::default_working_directory`), through `Project::create_terminal_shell`; the
  tests replace that factory with a display-only terminal.
- The header is the title bar's height and draws the window controls the title bar leaves to a
  left-hand sidebar. Its Add Project button opens Zed's recent-projects popover.
- The handlers take the rows' weak handles and return a `Result`, which the click sites log.
- While it stands in, it answers `serialized_state` with Zed's sidebar's state. It saves nothing
  of its own yet (#442).

## Threads (#439)

- **Rows.** Each group's threads come from `ThreadMetadataStore`:
  - the group's main worktree paths;
  - the same paths as folder paths, for rows from before main paths were kept;
  - each member workspace's roots.

  They are deduplicated by thread id and sorted newest first. Archived threads never show,
  and a draft shows only while its panel shows it. Rows are drawn with Zed's `ui::ThreadItem`,
  indented under the project.
- **Status and attention.** Status comes from the live conversations in each member
  workspace's Agent Panel, joined to rows by thread id; a thread with no live conversation
  shows as done. The rail keeps each thread's last status, so a run that ended while the
  thread was not shown lights its dot, until the thread is the displayed workspace's
  visible Agent Panel thread.
- **Clicks.** A thread row shows its project and opens the thread, focused, in that project's
  Agent Panel (`load_agent_thread`, then `focus_panel`). New Agent Thread lists the Zed Agent,
  then the project's agent servers by name, with the agent registry as the fallback for names
  and icons. It calls the chosen project's panel directly, since a dispatched action would
  reach the displayed project.
- **What it follows.** Beyond W2's subscriptions:
  - each Agent Panel's events, and focus entering or leaving it;
  - each live thread's status, title and confirmation events, but not streamed output;
  - each project's agent servers;
  - the metadata store.

  The subscriptions carry the window, since focus is read at every rebuild.
- **`is_threads_list_view_active` stays `false`.** `true` would make Zed treat every thread
  in the window as seen while the rail is open. That would silence the OS pop-ups and sounds
  for the Agent Panel's terminal threads too, which the rail does not list, and swap the title
  bar's project button
  (AD-claude-439-the-rail-does-not-claim-zeds-threads-list-001).

## Tests

`src/marley_workbench_tests.rs` (the switch, 11 tests) and `src/rail_tests.rs` (the rail, 16,
and 11 thread tests in `rail::tests::threads`): driven gpui tests on `MultiWorkspace::test_new`
over a FakeFs, clicking elements found by debug selector. The thread tests run on
`init_agent_test`, which layers Zed's agent test setup (`agent_ui::test_support`, with a
thread database per test) over the window. They put an `AgentPanel::test_new` in each project
and drive threads through `acp_thread::StubAgentConnection`: a turn that stays open until
`end_turn`, or a tool call waiting on a permission.

## Known limits

- Zed's Panel Layout presets misread the Marley layout until #441 hides them.
- The settings UI shows the patched values as the defaults: in the Marley layout a stored
  `terminal.button: false` looks like the default and has no reset control.
- A layout round trip with the Agent Panel open can close the right dock; each round trip
  adds a subscription pair on the kept Zed sidebar; a window restored in the Marley layout
  saves a partial state before its restore finishes. All three are in #442's notes.
- Agent CLIs in rail terminals, routing, keyboard navigation and the rail's own saved width and
  closed state are W4 to W6 (#440 to #442).
- The thread rows have not been seen live: #439's drive would have moved Chad's windows off his
  monitor. Their look is owed to the next headless capture.
