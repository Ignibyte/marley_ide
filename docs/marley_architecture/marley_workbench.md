# `marley_workbench`

The Marley layout, written in the fork for the workbench shell's W2 (#438): the
`marley.layout` setting, the switch between Zed's sidebar and the rail, the rail itself, since
W5 (#441, #449) the terminal routing, and since W5c (#450) the New Agent picker and the Marley
keymap.
MIT OR Apache-2.0. It links Zed's GPL crates (`workspace`, `sidebar`, `terminal_view`,
`recent_projects` and others), so it builds and ships only as part of the fork
(AD-claude-438-marley-crates-may-link-zeds-gpl-crates-001).

Its lint table is rustal's (CONSTITUTION §14, #447), with two allows that gpui calls for:
`future_not_send`, since gpui's test and async contexts are not `Send`, and `unused_results`,
since gpui's registration calls return `&mut App` for chaining and `.log_err()` returns an
`Option`.

## The switch (`src/marley_workbench.rs`)

- `MarleySettings` reads `marley.layout`: `marley` by default since #460, or `zed`. The
  settings block is `settings_content::MarleySettingsContent`, and `default.json` has no
  `marley` block, so the enum's `#[default]` decides. `crates/zed`'s own tests write `zed`
  in `init_test_with_state`, before `initialize_workspace`, since they test Zed's layout.
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
- **The docks across a switch (#456).** Zed's docks move the Agent Panel in their own settings
  observers.
  - A dock the Agent Panel enters while visible opens on it and forgets the panel it showed;
    the dock it leaves closes when it was the one shown.
  - The switch's observer runs first, so it notes every workspace's docks before the move
    (`docks_before`), then settles them in a `cx.defer` after it (`settle_docks`). The dock the
    Agent Panel entered remembers the panel it showed and whether it was open, by persistent
    name, per workspace, in `LayoutState::displaced`.
  - The dock it leaves gets that panel back, open only if it was open both before the trip and
    as the Agent Panel left. This happens only while the Agent Panel was still the panel it
    showed, so a panel the user chose there in between stands, and so does a close.
  - The restore runs before the move's throttled save, so the saved state is the restored one.
- **Zed's layout presets (#451).** `workspace::UseClassicLayout` and `workspace::UseAgenticLayout`
  (from `title_bar`) rewrite the docks the Marley layout sets, `agent.dock` among them. In the
  Marley layout capture-phase listeners on each workspace's root stop them and show a toast
  saying the presets belong to Zed's layout, whose button (`use_zed_layout`) switches to it. In
  the Zed layout they go on to Zed's handlers.
- **The Marley settings page (#515).** `marley::OpenSettings` (`marley: open settings`) is registered
  on each workspace and dispatches `zed_actions::OpenSettingsPage { page: "Marley" }` through the
  window. The page itself lives in Zed's `settings_ui` (`src/marley_page.rs`, first in
  `settings_data`), since a page is data over `SettingsContent` that the settings UI owns: a
  Layout section (`marley.layout`, a dropdown through `strum` on `MarleyLayout`), since #516 an
  Agents section (`marley.redact_secrets_for_agents`), and a Privacy section (the telemetry keys,
  off by default since #514). A Marley feature with a setting adds its section there.
- **`MarleySettings`** holds `layout`, and since #516 `redact_secrets` (true unless
  `marley.redact_secrets_for_agents` says false) and `redaction_patterns`; it is `Clone`, no
  longer `Copy`.

## One Marley per data directory (`src/single_instance.rs`, #513)

- `zed`'s `main` runs Zed's single-instance check on the dev channel too: it binds
  `<data dir>/zed-dev.sock`, so `--user-data-dir` gives a Marley a socket of its own. It runs only
  where `socket_fits()`: a socket path of 108 bytes or more fails the bind as a running Marley
  would, so such a data directory starts without the check and logs a warning.
- When the check finds a Marley running, `hand_off(paths_or_urls)` sends it one datagram per
  argument, the URL Zed's listener opens (a path that exists canonicalized as `file://`; `file://`,
  `zed://`, `zed-cli://` and `ssh://` as given; anything else made absolute against this launch's
  working directory, so `path:line:column` keeps its suffix), or `zed://open` when there is none,
  which Zed handles as `FocusApp` (`activate_any_workspace_window`). A URL over 1,024 bytes, the
  listener's buffer, is refused before anything is sent. `main` prints the answer ("Marley is
  already running on <dir>; it was handed N of this launch's paths", or "… asked to come
  forward"), or the error on stderr, and exits.
- Whether the window comes to the front is the compositor's call: gpui asks for the activation
  token itself (L-claude-513-gpui-asks-for-activation-from-the-window-it-activates-001); #545
  would carry the launcher's token instead.

## The rail (`src/rail.rs`)

- It implements `workspace::Sidebar`, so the `MultiWorkspace` keeps the resize handle, the open
  state, persistence and the toggle actions.
- It follows every workspace, every listed terminal view and every Browser tab (#504). On each
  event it rebuilds the snapshot and redraws only when the pure `marley_rail::RailSnapshot`
  changed. `render` reads nothing but the rail, so a background terminal's output never redraws
  the window through it, and `has_notifications` reads the stored snapshot, so it holds while the
  rail is closed.
- It lists the groups that have an open workspace, named through Zed's public functions
  (`compute_disambiguation_details`, `ProjectGroupKey::display_name`) in `crate::group_names`,
  which the browser tools share for a tab's project (#574).
- A project header has a chevron to fold it, a `+` menu with New Terminal, New Browser Tab
  (#500: `Rail::new_browser_tab` shows the project, then `browser::new_tab`, as Ctrl+T), New
  Agent Thread and the agent CLIs, and an attention dot;
  a terminal row has the title, the working directory and a bell dot. A header click shows the
  project. A terminal row click shows its project, activates and focuses the terminal and
  clears its bell. New Terminal starts where Zed's own would
  (`terminal_view::default_working_directory`), through the terminal factory of
  `agents::Launcher`: `Project::create_terminal_shell`, unless a test sets a display-only one.
- **The rows' look (#468),** after Warp's vertical tab list
  (`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`). Marley draws the rows
  itself, not with Zed's `ListItem` or `ThreadItem`:
  - `row_frame`: every row keeps a 1px border, clear unless the row is selected, when it is a
    card, `ghost_element_selected` inside a `raised` border, so the selection moves nothing.
    `raised` is the theme's text color at 10%, a step lighter than what it sits on in a dark
    theme and darker in a light one; `border` and `element_background` match the fills under
    them in One Dark.
  - `row_card`, for terminal, Browser and thread rows: `h_11`, a `size_7` round icon container in
    `raised`, the title over the second line when there is one. A shell's icon is a `>_` in the
    buffer font (`rail_terminal_icon`), since each of Zed's terminal icons boxes the prompt; an
    agent CLI's is its own.
  - A project header is `h_8`, its name in the small muted type of a section label unless
    selected. A `Divider` runs above every project after the first row, in the project's
    wrapper (`marley-rail-divider-{index}`).
- **Rename and close (#452).** A terminal row's right-click menu has Rename and Close, a
  double-click renames, and a close button swaps in for the bell's slot under the pointer
  (gpui's `visible_on_hover` and `group_hover` on the row's hover group, since #468). Rename shows the terminal, then runs Zed's own
  `TerminalView::rename_terminal`, which edits the name in the tab and keeps it through
  `set_custom_title`. Close goes through the pane (`close_item_by_id`, `SaveIntent::Close`), so
  Zed asks first while a task runs. A custom title beats an agent CLI's own on its row.
- **Browser rows (#504).** Each `BrowserView` in a project's workspaces is a row after its
  terminals (`member_browsers`, in the order the workspace lists its items), keyed by the view's
  entity id. Its title is the tab's own text, and its second line the URL's host and port
  (`host_and_port`; none for `about:blank`). The icon is the page's (`img`, 14 px), a spinning
  `LoadCircle` while the main frame loads, or `ToolWeb`. At the end come `Crosshair` and the
  tray's picks, `Pencil` and the page's annotations, each above zero, then the agent's `Sparkle`,
  and on hover a close button that closes the tab through its pane (`close_item_by_id`), which
  closes the page. A click or Enter shows the tab with the focus (`activate_browser`).
  `active_rows` reports the displayed workspace's active item as `Focus.browser` when it is a
  Browser tab, so its row is selected while it is in front.
  - The rail reads the hub only through `BrowserHub::try_global`, so it never starts the
    browser. It follows each tab's `ItemEvent`s and the hub's `PageInfoChanged`,
    `PageStatusChanged`, `PageOpened` and `PageClosed` (`follow_browsers`), never the hub's
    notify, which fires on every frame.
  - The page's image lives in `Snapshot.favicons`, beside the pure snapshot, and its id in
    `BrowserSnapshot.icon`, so an icon's arrival changes what `refresh` compares.
- **Keys and reorder (#453).** The key context is `MarleyRail menu`, and the rail answers Zed's
  `menu::SelectNext`, `SelectPrevious`, `SelectFirst`, `SelectLast`, `SelectParent`,
  `SelectChild` and `Confirm`. Zed binds up, down, Home, End and Enter to them with no context,
  and left and right in `menu`, so the rail binds no key. Each moves `cursor`, the keyboard's
  row, which `refresh` hands to `marley_rail` while the rail holds focus; a focus-out
  subscription drops it. Left folds an open project or climbs to its header, right unfolds, and
  Enter runs the row's click handler. A project header's right-click menu has Move Project Up
  and Move Project Down (`MultiWorkspace::move_project_group_up`, `move_project_group_down`),
  disabled at the ends, and after a separator Clear Browser Data… (#581) and Remove Project
  (#507: `MultiWorkspace::remove_project_group`, as Zed's sidebar's Remove calls it), which closes
  the project's workspaces after their save prompts and so stops its Chromium.
  `project_context_menu` builds it. Zed's `multi_workspace::FocusWorkspaceSidebar` (`ctrl-alt-;`) focuses
  the rail as it does Zed's sidebar.
- **The filter (#457).** A single-line editor under the header ("Filter…") narrows the rail as
  Zed's Threads Sidebar's filter does. `build_snapshot` matches each project's name and each
  terminal's, Browser tab's and thread's title with Zed's own `fuzzy_match_positions` (a substring match that
  ignores ASCII case), and `marley_rail` decides what shows. A header's name and a terminal's
  title and a thread's draw with `HighlightedLabel`.
  - While filtering, the chevrons are left out and `fold` does nothing. "No matches" draws when
    nothing is left. A clear button replaces the key hint (`KeyBinding::for_action_in`).
  - Each edit (`EditorEvent::BufferEdited`) refreshes, then puts the cursor on
    `marley_rail::first_match`.
  - Keys: the Marley keymap binds `secondary-f` to Zed's `agents_sidebar::FocusSidebarFilter`
    in `MarleyRail && !Picker`. In the field, up, down and Enter reach the rail's `menu`
    handlers because the single-line editor propagates `editor::MoveUp`, `editor::MoveDown`
    and `editor::Cancel`.
  - `menu::Cancel` clears the filter, or from an empty field focuses the rows, and otherwise
    propagates. The text is not saved.
- **The switcher (#454).** The rail's `Sidebar::toggle_thread_switcher` opens `RailSwitcher`
  (`src/rail_switcher.rs`, the rail's `switcher` module) over the window's terminals and
  threads, as Zed's sidebar opens its thread switcher.
  - Zed's `AgentPanel` binding routes `ctrl-tab` there, and the Marley keymap binds `ctrl-tab`
    and `ctrl-shift-tab` in the rail's block. The center panes keep Zed's tab switcher.
  - The rail puts the view in the `MultiWorkspace`'s sidebar overlay and focuses it, since the
    overlay does neither. It needs two rows, and opens on the second, or on the last with
    `select_last`.
  - The view's key context is Zed's `ThreadSwitcher`, so Zed's bindings step it. The release
    of the modifiers it opened with, Enter or a click confirms, and the rail opens the row
    through `open_row`, as Enter on the rail does. Escape cancels and hands focus back;
    focus-out cancels and leaves it.
  - Recency: `refresh` notes each change of `marley_rail::window_row`, the row that holds the
    window's focus, in `shown_at`, pruned to the rows that exist. The switcher's own focus is no
    row, so opening it notes nothing. `thread_item` and `terminal_icon` draw the switcher's rows;
    the rail's own are Marley's cards (#468, below).
- **Next and Previous Project and Thread (#459).** The `MultiWorkspace` forwards Zed's four
  actions to `Sidebar::cycle_project` and `cycle_thread`, open or closed, through
  `SidebarHandle`'s `window.defer`. The rail asks `marley_rail::cycle_project` or `cycle_row`
  for the row and opens it through `open_row`, as a click does. Activation takes focus out of
  the rail, so the next action starts from the row the window then shows.
- The header is the title bar's height and draws the window controls the title bar leaves to a
  left-hand sidebar. Its Add Project button opens Zed's recent-projects popover.
- The handlers take the rows' weak handles and return a `Result`, which the click sites log.
- While it stands in, it answers `serialized_state` with Zed's sidebar's state, with fields of
  its own added (#442): `width` and `width_set_by_user`, the names Zed's sidebar reads, and
  `marley_rail_closed`, which Zed's ignores. Every other field is kept, so Zed's sidebar still
  restores from the blob.
  - **Closed-rail memory.** The observer the rail puts on the `MultiWorkspace` notes, while
    AI is on, whether the sidebar is open; `serialize_now` reads the blob a turn after the
    update that closed it. A restored blob that says closed closes the rail again once the
    restore's update is over (`window.defer`), since the Marley layout builds every window's
    rail open.
  - **One width.** `set_width` forwards the rail's clamped width to the kept Zed sidebar; a
    rail built over one starts at its width; a restored blob's user width sizes the rail;
    `take_zed_sidebar` hands a fresh Zed sidebar the blob with the rail's width in it.
  - `serialized_state` and `restore_serialized_state` run inside the `MultiWorkspace`'s update,
    so they touch only the rail's fields and the kept Zed sidebar.

## Threads (#439)

- **Rows.** Each group's threads come from `ThreadMetadataStore`:
  - the group's main worktree paths;
  - the same paths as folder paths, for rows from before main paths were kept;
  - each member workspace's roots.

  They are deduplicated by thread id and sorted newest first. Archived threads never show,
  and a draft shows only while its panel shows it. Rows are drawn as the terminal rows are
  (#468): the agent's icon in the round container, the title, and a second line
  `<agent> · <status>` (`agents::thread_agent_name`, `ThreadStatus::label`), with the status at
  the row's end as Zed's `ThreadItem` draws it.
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
  - the metadata store;
  - each project's folders (#458): `WorktreeAdded`, `WorktreeRemoved`, `WorktreeOrderChanged`
    and `WorktreePathsChanged`, as Zed's Threads Sidebar follows them, and no other project
    event. The rebuild is deferred (`defer_in`): the project reports a folder before the
    `MultiWorkspace` rekeys its group, and a rebuild in between would find the project in no
    group and forget its rows' recency and attention dots.

  The subscriptions carry the window, since focus is read at every rebuild.
## Agent CLIs (#440)

- **Recognition.** A terminal whose foreground argv names a known agent CLI
  (`marley_agent::agent_kind_of`) is an agent row. It carries the agent's icon and the title
  the CLI sets over OSC, falling back to the agent's name, and its second line is
  `status_line`: the agent and whether it is working or waiting. The foreground command comes
  from `Terminal::foreground_process_command_name`, through a seam the tests replace, since a
  display-only terminal has no process.
- **Status.** The rail notes when each terminal last wrote output (a view's `Wakeup`, which
  the view also sends for a bell). Each output re-arms an agent terminal's quiet timer, which
  refreshes the rail once `WAITING_AFTER` has passed, on the executor clock. Both maps are
  pruned to the open terminals.
- **The `+` menu.** After New Agent Thread, an "Agent CLIs" header lists each CLI
  `which::which_in` finds on the launcher's search path, the process's `PATH` unless a test
  sets another. Choosing one runs `agents::start_cli` in that project: a center terminal where
  New Terminal would, and once the shell's startup handshake completes, or after 5 s,
  `write_init_command_after_startup` writes the program's name and Enter. An error reaches a
  prompt.

- **`is_threads_list_view_active` stays `false`.** `true` would make Zed treat every thread
  in the window as seen while the rail is open. That would silence the OS pop-ups and sounds
  for the Agent Panel's terminal threads too, which the rail does not list, and swap the title
  bar's project button
  (AD-claude-439-the-rail-does-not-claim-zeds-threads-list-001).

## Routing (#441)

`src/routing.rs`, installed on every workspace by `init`. In the Marley layout nothing opens the
bottom Terminal Panel; in the Zed layout everything passes through to Zed. Both halves read the
layout at each call, so a switch reinstalls nothing.

- **Tasks.** `RoutedTerminals` replaces Zed's `TerminalProvider` when the workspace announces
  the Terminal Panel (`workspace::Event::PanelAdded`), after the panel's `load` has installed
  Zed's. Every task still runs through `TerminalPanel::spawn_task`, so Zed's rules for reruns,
  reuse and concurrent runs hold; in the Marley layout the provider sets
  `reveal_target: Center` first.
  - The workspace calls a provider while it is being updated, and `spawn_task` reads the
    workspace, so the spawn waits for the window's next turn.
  - A task reruns in its last terminal. In the Marley layout the provider first moves the
    task's terminals out of the panel into the active pane (`workspace::move_item`, as a tab
    drag does), so a task that last ran in the Zed layout reruns in the center.
  - The answer is the task's exit status: `Some(Err)` when it cannot start, `None` when its
    window or terminal went first.
- **New Terminal and Open in Terminal.** Capture-phase listeners on the workspace's root
  (`register_action_renderer`, `capture_action`) see `workspace::NewTerminal` and
  `workspace::OpenTerminal` before Zed's handlers. In the Marley layout they stop propagation
  and open a center terminal through `TerminalPanel::add_center_terminal`: New Terminal where
  Zed's would start (`default_working_directory`), Open in Terminal in the folder it names, a
  local shell for `local: true`. An error reaches a prompt. Everything dispatched inside the
  workspace passes these listeners, the Terminal Panel's own `+` included when the panel is
  opened by hand.
- **A first terminal (#455).** The same `observe_new` hook opens one center terminal at a
  project's root through `open_center_terminal`, when a folder project is opened fresh in the
  Marley layout and its center has none.
  - Fresh means `Workspace::opened_from_saved_state() == Some(false)`. That is a Zed touchpoint,
    recorded by `new_local` from its own lookup of the project's saved state, since nothing
    outside `crates/workspace` can tell a fresh workspace from a restored one.
  - A project opened from saved state reports `Some(true)` and keeps what it saved. Its restore
    also swaps in the saved center, which would drop a seed whenever that center has panes.
  - A workspace made any other way reports `None`, as `Workspace::test_new` does, so the test
    harness's windows get nothing.
- **The toggles (#449).** Three more capture listeners take `terminal_panel::Toggle`
  (`` ctrl-` ``), `terminal_panel::ToggleFocus` and `workspace::ToggleBottomDock` (`ctrl-j`)
  in the Marley layout and run one toggle between the code and the center terminals:
  - from a focused center terminal, the center item used last that is not a terminal, by the
    panes' activation history, or nothing when there is none;
  - from anywhere else, the center terminal used last (`recent_active_item_by_type`), or a
    new one where New Terminal would start.

  `ToggleBottomDock` is taken only while the bottom dock is closed and would show the
  Terminal Panel: its active panel, or with none active, its first. Docks sort panels by
  activation priority and the Terminal Panel's is the lowest of Zed's bottom panels. The check
  reads the dock and asks no panel whether it is enabled, since the listener runs inside the
  workspace's update. An open dock, or one whose active panel is another, goes to Zed.
  Catching the actions rather than rebinding the keys routes Zed's bindings, a user's own, the
  palette and the menus alike, with no keymap and no Zed change.

## Agents and the New Agent key (#450)

`src/agents.rs` holds what Marley can start and how, for the rail's `+` menu and the picker
alike.

- **The choices.** `thread_agents`: the Zed Agent, then the project's agent servers by name,
  each with `thread_icon`. `installed_clis`: `marley_agent`'s CLIs that the search path holds,
  each with `cli_icon`.
- **The launches.** `start_thread` opens a thread of the agent in the workspace's Agent Panel,
  focused. `start_cli` opens a center terminal and writes the program's name once the shell is
  ready. Both act on the workspace they are given; the rail shows its workspace first.
- **The seam.** `Launcher`, a global holding the search path and the terminal factory, read
  through `launcher(cx)`: the process's `PATH` and `Project::create_terminal_shell` unless a
  test sets its own, so no test starts a real agent.
- **The picker.** `marley::NewAgent` opens a `Picker` in the workspace's modal layer with Zed's
  agents, then the CLIs, each marked "Thread" or "Terminal", since Claude Code can be both.
  Typing filters them fuzzily; an empty query keeps their order. Choosing one starts it in the
  workspace the picker opened in; an error reaches a prompt. With AI disabled the action opens
  nothing.
- **The key.** `keymap.json` binds `secondary-alt-n` (`ctrl-alt-n`, `cmd-alt-n` on macOS) to
  `marley::NewAgent` in the Workspace context. `load_keymap`, called last in `crates/zed`'s
  `load_default_keymap`, binds it tagged `KeybindSource::Default`, so every keymap reload binds
  it again, it wins over Zed's defaults at the same depth, and a user binding on the same keys
  wins over it. A keymap that fails to load binds nothing and is logged. No Zed default uses
  the chord in any context; JetBrains's base keymap does, and wins inside its editors.

## The block keys (`src/blocks.rs`, #473)

- `marley::PreviousBlock` and `marley::NextBlock` are caught at each workspace's root with
  `register_action_renderer`, as `routing` catches its actions, and act on the terminal view
  that holds focus: the active item of a center pane or of a Terminal Panel pane. With none, they
  do nothing.
- The handler calls `Terminal::sync` first, so a key pressed before the next frame sees where
  the one before it went, then scrolls to the bottom and up by `marley_terminal::block_scroll`'s
  offset.
- The Marley keymap binds them to `secondary-up` and `secondary-down` in `Terminal`, keys Zed's
  defaults leave unbound there (AD-claude-449's rule for the Marley keymap).

## The agent bar (`src/agent_bar.rs`, #477)

- `init` sets Zed's `terminal_view::MarleyTerminalFooter` to the bar's renderer, which every
  `TerminalView` calls below its grid with a `MarleyFooterContext`; the view's root is a flex
  column, so the bar takes its rows from the grid.
- `contents` is what the bar shows: the agent from `foreground_process_command_name` through
  `marley_agent::agent_kind_of`, the folder from `working_directory`, and the branch of the
  innermost repository in the project's git store that holds the folder (`branch_for`). No agent,
  no bar, except the one-row strip that holds a terminal's offer (#503).
- The bar draws `agents::cli_icon`, the agent's name, Attach File, Rich Input, the microphone
  and #482's chip ("Connect Claude Code to Marley" since #500, its tooltip naming the plugin's
  notifications and Marley's tools for its terminals and Browser tabs) at the left, and the folder (`~` for home) and branch at the right. The
  footer is a column: the rich input's editor, while it is open, sits above the bar.
- Attach File (#479): the bar's `+` and `marley::AttachFile`, which `init` registers on every
  workspace for the focused terminal (`blocks::focused_terminal`), both call `attach`. It opens
  `Workspace::prompt_for_open_path` for files, several at once, with the project's lister, so a
  remote project's chooser lists the machine its terminals run on, and types the chosen paths
  with `TerminalView::add_paths_to_terminal`, as a drop does. A cancel types nothing.

## Terminal links and the offered URL (`src/links.rs`, #503)

- `init` sets Zed's `terminal_view::MarleyTerminalUrl`, which a terminal view's `Event::Open` asks
  with its `MarleyFooterContext` before `cx.open_url`. It gets Ctrl+click URLs, and since #503
  also an OSC 8 link's http or https target on a plain click. `open_clicked` leaves a URL that is
  not http or https to Zed.
- `destination`: over SSH (a remote project, or a foreground program among `ssh`, `mosh-client`,
  `autossh` and `et`) every URL goes to the system browser as printed. Otherwise
  `marley.terminal_links` (`MarleySettings.terminal_links`) decides: `local_in_browser_tab`, the
  default, sends a local URL to a Browser tab and any other to the system browser;
  `all_in_browser_tab` and `system_browser` send every one to one place. Shift, read from the
  window when the event is handled, takes the other place. A local URL opens as
  `address::local_url` gives it, so `0.0.0.0` opens at `127.0.0.1`.
- `open`: the system browser through `cx.open_url`. A Browser tab is opened through `window.defer`,
  since the URL arrives while its terminal view is being updated, and a tab added then would
  deactivate that view, the pane's front item, and update it again. `browser::open_url_tab`
  brings forward a tab of the workspace whose address is the same URL, or adds one to the active
  pane with the focus.
- The offer:
  - `follow` subscribes every terminal view to its own `Wakeup`s. A scan runs 500 ms after the
    first (`schedule_scan`, one pending per view, dropped with the view) and reads the last 200
    logical lines through `address::printed_local_urls`. It skips a terminal over SSH.
  - `ServedUrls` keeps each view's URLs, each port's newest print, 16 at most. While any
    terminal holds one, `watch_ports` reads `ports::listening_ports_in("/proc/net")` every 2 s
    off the main thread, and `take_ports` redraws each terminal whose offer changed.
  - `offer` is the terminal's newest URL whose port listens, and none over SSH.
  - `offer_button` is a `SplitButton`: its label opens the URL as a Ctrl+click would, and its
    arrow's `PopoverMenu`, opening upward, holds Open in Browser Tab, Open in System Browser and
    Copy URL.
  - `agent_bar::render` puts it before the folder chip, or alone in a one-row strip under a
    terminal with no agent.
- The link menus (#579):
  - `link_menu`, which `init` sets as Zed's `terminal_view::MarleyTerminalLinkMenu`, fills a
    `ContextMenu`. It gives a header of the URL (shortened to 60 characters), Open in Browser
    Tab (not over SSH), Open in System Browser and Copy Link, the target for an OSC 8 link.
  - While the user's settings file names no `terminal_links`, it adds "Always Open Local Links in
    a Browser Tab" and "… in the System Browser". They write it with `update_settings_file`,
    which ends the offer.
  - The terminal view shows these entries alone for one plain click on a link in the text, and
    first in its right-click menu for any link. Both open through `destination` and `open`.
- Programs that open a URL themselves (#561):
  - `mcp::offer_browser_opener` writes `bin/marley-open-url` beside the bridge's copy
    (`<data_dir>/mcp/`), and gives its path to `marley_terminal::shell_integration::set_browser_opener`
    at start and on every settings change, or none under `system_browser`. The terminal builder
    exports it as `BROWSER` for every terminal that is not remote, unless `terminal.env` names a
    `BROWSER`.
  - The opener, run with a URL, reads the endpoint beside its own data directory (or
    `$MARLEY_MCP_ENDPOINT`), so it reaches the Marley that wrote it. It calls
    `browser_open_url {url, directory}` within 5 s, and otherwise execs `xdg-open` without
    `BROWSER`.
  - `browser_tools::open_url` answers before the browser is up. It checks `address::agent_url`,
    then `links::browser_tab_url` (#503's rule, with no SSH and no key), then `holding(directory)`
    and `window_of`. It shows the workspace in its window and opens the tab with
    `open_url_tab`, in separate updates through the window's `AnyWindowHandle`.

## Autosuggestions (`src/autosuggest.rs`, #484)

- `init` sets Zed's `terminal_view::MarleyTerminalSuggestion`, which the element calls in
  `prepaint` for the text it paints dimmed at the cursor. The typed text is the cursor line's
  cells from `AnchoredBlocks::input_start` to the cursor, while the shell waits at its prompt, the
  view is on the live screen and not the alternate one, vi mode is off and nothing follows the
  cursor. The lookup takes the terminal's own verified commands, newest first, then its shell's
  history file, which `HistoryFiles` reads once per path in the background, deferred out of the
  frame; a missing file holds none.
- `marley::AcceptSuggestion`, bound to `right` in `Terminal` and handled on every workspace,
  types the rest through `Terminal::input`, and without a suggestion calls `cx.propagate()`, so
  → reaches the program.

## Rich input (`src/rich_input.rs`, #481)

- `Prompts`, a global, holds an editor per terminal view (auto height, one to eight lines,
  soft wrap), made the first time it opens and dropped with the view, and whether it shows.
- `marley::RichInput` is bound to `ctrl-g` in `Terminal` and handled on every workspace: with a
  CLI agent in the focused terminal (`agent_bar::agent_in`) it opens that terminal's editor and
  focuses it; otherwise it calls `cx.propagate()`, and the key goes on to the terminal, which
  sends it to the program.
- In the editor's `MarleyRichInput` container, Enter (`marley::SendRichInput`) pastes the text
  with `Terminal::paste`, bracketed when the program asked for it, sends `\r`, clears and closes
  the editor and focuses the terminal; Escape (`marley::CloseRichInput`) closes it with the draft
  kept; Shift-Enter is `editor::Newline`. The container stops the key events the terminal view
  would send its program, chords and keys that type nothing, and lets text through to the
  editor.

## Sending the selection to an agent (`src/send_selection.rs`, #549)

- `init` registers `marley::SendSelectionToAgent` on every workspace and, through
  `register_action_renderer`, a capture of Zed's `agent::AddSelectionToThread` (`ctrl->`) that
  acts only in the Marley layout and only when the send goes ahead; every other case (no focused
  file editor, no agent) lets the action on to Zed's Agent Panel. A `FocusOrder` global follows
  the terminals' focus-ins, 64 at most.
- `send_selection` takes the active item as an `Editor` when it holds the focus, its newest
  selection as an absolute path and 1-based lines (`selection_of`, through the multi-buffer
  snapshot's `range_to_buffer_range` and the file's `LocalFile::abs_path`; `line_span` leaves out
  a last line the selection ends at the start of; a caret is the file alone). `agent_targets`
  walks the terminals of the window's workspaces, center and docked, whose foreground program is
  a known agent (`agent_bar::agent_in`), newest focus first; it reads the workspace the action
  runs in through the `&Workspace` it is given, since that one is being updated. One target is
  sent; several open `TargetPicker`, Zed's `Picker` in a `ModalView`, rows "agent · project ·
  state" (the state from the seat, when there is one).
- `reference` gives Claude Code `@path#L<a>-<b>` and the other agents `path:<a>-<b> `, the path
  relative to the agent's `Terminal::working_directory` when the file lies under it.
- `send` runs after the update it starts in: rich input open on the terminal
  (`rich_input::is_open`) gets the text through `rich_input::insert`, the terminal brought to the
  front first; a seat in `State::Waiting` gets nothing and a toast; otherwise the window
  activated, the terminal revealed (`browser::reveal_terminal`) and focused, one
  `Terminal::paste`, no Enter.

## Voice (`src/voice.rs`, #480)

- `Voice`, a global, holds the `voxtype` found on the PATH at `init` (a lazy background
  future), the `VoiceState` (idle, recording, transcribing) and the follower: a task running
  `voxtype status --follow --format json` with `kill_on_drop`, whose lines' `class` becomes the
  state (streaming counts as recording, anything else as idle).
- The follower starts the first time a microphone is drawn, deferred out of the render
  (`follow_once_drawn`), and again on a toggle after it ended; a render never restarts an
  ended one. When it ends, the state is idle.
- `toggle` makes sure the follower runs and runs `voxtype record toggle`; a failure shows in
  the workspace. The agent bar's microphone focuses its terminal and toggles, and
  `marley::ToggleDictation`, registered on every workspace, toggles from the command palette.
- `run_program` in the crate root runs `claude` and `voxtype` for both adapters.

## Notifications (`src/notifications.rs`, #478)

- `init` names the app `Marley` for the desktop's notifications, answers their clicks, and
  subscribes every new `TerminalView` to its terminal's `Event::MarleyNotification`.
- `notify` skips the focused terminal of the active window; otherwise it posts a
  `SystemNotification` tagged by the view, titled by the escape or, for an OSC 9, by the tab.
  The view has already set its bell, which marks its tab and its rail row.
- `show_sender` answers a click: it activates the view's window, its workspace in the
  multi-workspace and its item, and clears the bell, as the rail's `activate_terminal` does.

- A notification titled `marley-event` (`marley_terminal::AGENT_EVENT_TITLE`) is a Claude Code
  hook event, not one for the user (#519): `init`'s subscription hands its body to
  `agent_events::on_frame` instead of `notify`, and registers `agent_events::forget` for the
  view's release. Zed's view leaves such a notification unmarked, so no bell either.

## Claude Code's hook events (`src/agent_events.rs`, #519)

- `AgentEvents`, a global made at the first frame, holds one `marley_fleet::FleetSnapshot`: a
  seat per terminal view whose Claude Code has sent an event, keyed by the view's entity id as
  `terminal_list` gives it. `seat(view)` gives the seat until its session ends.
- `on_frame` drops a frame unless `agent_bar::agent_in` says Claude Code is the terminal's
  foreground program (a `cat` of an old log moves nothing; spec D4), decodes it, folds it
  against the seat with `marley_agent::claude_events::fold`, and applies the events. A frame
  that does not decode is logged at debug and dropped.
- `forget` removes a closing view's seat by folding the others again from nothing, since the
  reducer never removes a seat.
- `end(terminals)` (#547) ends each live seat named, and the rail's refresh names every listed
  terminal without Claude Code in the foreground, so a Claude Code that left without a
  `SessionEnd` reads `done`. A failed seat is left as it is: the reducer keeps it failed on
  `Ended`, and ending it again would only notify the rail into ending it again.
  `next_quiet_change` gives the delay to the moment a working row's `no update in N m` next
  changes, which the rail's one timer (`minute_timer`) waits for.
- `mcp::publish`, an observer of the global, sends each snapshot down a channel to one background
  task that replaces the server's `ServerData.snapshot` and calls `transport::signal_change`, so
  `fleet_snapshot` and `fleet://snapshot` answer the same seats and the main thread never waits
  on the server's lock (#547).
- The rail observes the global (`observe_global_in`) and refreshes. `terminal_snapshot` reads
  the seat of a Claude Code row: its status from `seat_status`, its second line from
  `seat_line` (the state, the subagents, the user's prompt; the icon names the agent), and a
  third line from `seat_activity`. `row_card` takes the lines under the title, and a row with
  a third line is 3.5 rem tall; every other row keeps `h_11`. A row without a seat keeps the
  quiet timer's reading.

## Asking before a close ends a working agent (`src/close_guard.rs`, #550)

- Zed's close paths ask `workspace::MarleyCloseGuard`, which `close_guard::init` sets: a tab's
  close at the head of `Pane::close_items` (the rail's Close and `ctrl-shift-w` included), the
  quit once in `prepare_windows_to_quit`, a window's close once in `prepare_window_to_close`,
  and a replace (the rail's project removal) in `prepare_to_close`. Each hands over the items
  about to close; `false` cancels as Zed's own Cancel does.
- `working_agents` keeps the terminals whose foreground program is a known agent that is
  working: the seat's `Starting`, `Working` or `Waiting` for Claude Code, else the quiet timer's
  `Working` (a `Wakeup` within two seconds, no bell). With none, the close goes on. Otherwise one
  question at a time names each agent (`Close`/`Quit`/`Close Window`, `Show`, `Cancel`).
- A tab's close of a working terminal holds the view (its pane, its workspace, a deadline of
  `undo_close_seconds`) with a toast; Undo dispatches `marley::UndoCloseTerminal`, which
  `ctrl-shift-t` also sends (Marley's keymap, the Workspace context; it propagates to Zed's
  Reopen Closed Item when nothing is held) and which re-adds the newest view to its pane. At the
  deadline the view drops and its PTY ends as a close ends it.

## Pushes to the phone (`src/push.rs`, #535)

- `notifications::init` hands each `marley-event` frame's seat change (`agent_events::on_frame`
  gives the state before and the seat after) to `push::on_change`, which asks `TurnEvent::of_change`
  whether it is one to push (a wait that starts, a turn that ends, a turn that fails).
- It pushes only with `MarleySettings::push` set (`marley.push.url` and `topic`), only while the
  user is not looking at the terminal (`notifications::looking_at`, the gate desktop banners use),
  only to a loopback host and an ntfy topic (`target_url`), and at most once per project in 5
  seconds. The line is `marley_agent::event_line` over the last folder of the seat's `cwd`.
- `post` runs on a background task: the token from `token_file`, refused when group or others
  can read the file; `Title: Marley`, `Priority` 4 (needs input, failed) or 3 (finished), `Tags`
  `question`, `white_check_mark` or `x`, the line as the body. `report` logs a refusal, and a
  failure too, with one toast (`Pushes::failing`) until a post succeeds.

## Marley's MCP server (`src/mcp.rs`, #491, #501)

- Since #520 every call carries its `marley_mcp::Caller` (the bridge's `Marley-Terminal`,
  `Marley-Project` and `Marley-Cwd` headers): `terminal_list` marks the row whose
  `Terminal::marley_terminal_id` is the caller's `self` and gives each row's `terminal_id`, and
  `terminal_of` reads the caller's own terminal when a call names none, refusing with the next
  step when the caller has none. The context server registered for Zed's agents blanks both
  variables, so a Marley started from a Marley terminal hands its agents no parent's identity.

- `start`, which `zed`'s `main` calls after `initialize_workspace` (Zed's tests run
  `initialize_workspace`, and must not start a server), spawns `marley_mcp`'s server once per
  process, writes `mcp-endpoint.json` into `paths::data_dir()` on the background executor, and
  removes it in `on_app_quit`. A server or a file that fails is logged and shown once, as a toast
  in the first workspace; Marley runs on without it.
- The server hands each call that is the app's to an unbounded channel, and a foreground task
  answers it in a top-level update:
  - `terminal_list`: every `TerminalView` in every window, the center panes' and the terminal
    panel's, with its id (the view's entity id, as the rail's), its tab title, its project (the
    first visible worktree), its working directory, the running block's command and how many
    blocks it holds;
  - `terminal_blocks`: the newest 50 blocks, at most 500, each with its command, whether the
    command was verified, whether it runs, its exit code, the prompt's `pwd`, its start and
    duration from `AnchoredBlocks::times` (a running block's duration is how long it has run),
    and `Terminal::block_output_kept`;
  - `terminal_read`: `Terminal::block_output`, the last 2,000 lines and at most 256 KiB, and
    whether anything was left out.
- **Redaction (#516).** `start` builds the `AgentRedaction` global from `MarleySettings` and
  rebuilds it on each `SettingsStore` change that alters `redact_secrets` or
  `redaction_patterns`, with an app notification naming a pattern that did not compile.
  `agent_redactor(cx)` answers its `Arc<Redactor>`, `None` while redaction is off, and the
  built-in rules alone before `start` ran, so nothing leaves unredacted by accident.
  `terminal_blocks` runs each command through it and `terminal_read` the command and the whole
  output, before `tail` cuts it (a key whose BEGIN line fell before the cut would pass
  otherwise); both answer `redacted`, the count. `browser_tools`' `browser_console` runs each
  entry's text through it.
- Since #501, once the server runs, `offer_to_zeds_agents` writes the bridge
  (`claude_plugin::BRIDGE`) to `<data dir>/mcp/marley-mcp-bridge` off the main thread and adds
  `context_servers.marley` to Zed's default settings: a stdio server running it, with
  `MARLEY_MCP_ENDPOINT` naming the endpoint file. Zed's context server store then runs it for
  each local project, where the Zed Agent's Write profile (`enable_all_context_servers`) takes
  its tools from, and `mcp_servers_for_project` hands it to each external agent's `session/new`.
  No bearer goes into a setting; a user's own `context_servers.marley` replaces the default.

## Terminal ids across a restore (`src/terminal_ids.rs`, #575)

- `init` reads `MarleyTerminalIdsDb`'s table, `marley_terminal_ids(workspace_id, item_id,
  terminal_id)` (its own domain after `WorkspaceDb`, keyed by the pair with the workspace's
  cascade and no `UNIQUE(item_id)`), into `KnownIds`, and sets `terminal_view`'s
  `MarleyTerminalIdentity` hook, before any window restores its terminals.
- `TerminalView::serialize` saves the terminal's id through `save` (memory, then the table, the
  task awaited with Zed's own saves, so a quit keeps it); `added_to_workspace` moves it through
  `moved`; `cleanup` deletes the rows of the items not loaded (`delete_unloaded_items`);
  `deserialize` reads `saved`, which answers from memory, then from the table, and opens the
  shell with `Project::create_terminal_shell_restoring`.
- The restore reads memory because the terminal panel runs `TerminalView::cleanup` with the
  panel's items alone, which can delete a center terminal's row between two terminals' restores.
- `create_terminal_shell_restoring` puts the id under `MARLEY_RESTORED_TERMINAL_ID` for a local
  terminal, and the project sets the key empty for every other one; `TerminalBuilder::new` takes
  the value and leaves the key empty, so a well-formed value becomes the terminal's id and no
  program, and no split rebuilt from the template, sees it.

## The browser's agent tools (`src/browser_tools.rs`, #492, #493, #574)

- `mcp.rs` hands each `browser_*` call to `browser_tools::answer`, which starts the caller's
  project's browser again when it failed and answers the call from a task of its own once no
  project's browser is starting (`browser::settled`, 20 seconds at most). Asking starts no
  browser that is not running (#507): `browser_navigate`'s new page starts its project's. A call
  acts on the page its
  `tab` names, a target id from `browser_tabs`, waiting up to five seconds for a page still
  being attached (#493). A write tool first calls `browser::show_for_agent`, which brings the
  page's tab to the front of its pane unless that pane has the focus, and gives a page with no
  tab a tab. Every answer names its tab.
- **The caller's project (#574).** `answer` works out the call's `Scope` once, from its
  `marley_mcp::Caller`: the workspace of the caller's terminal (`mcp::caller_terminal`), else the
  local workspace one of whose own folders (`Workspace::root_paths`, so a linked worktree finds
  its own) holds `Marley-Project`, else `Marley-Cwd`, the longest folder winning; the scope is that
  workspace's project group in its window, every held workspace with its `project_group_key`,
  named as the rail names it (`crate::group_names`). A call that names no tab acts on the page of
  the scope's tab the user focused last, else of its newest (`BrowserHub::focused_among` over
  `browser::tab_workspaces`); with no tab of the scope showing a page it is refused at once,
  naming `browser_navigate`. A caller in no project gets the page whose tab the user focused last
  anywhere, the newest page when the user focused none, as before. `browser_navigate` opens a new
  page with `new_tab`, or when the caller's project has no tab showing a page (for a caller in no
  project, when the browser has none); a page opened for a caller's project gets its tab in the
  caller's own workspace. Since #507 that page is made in the caller's project's browser
  (`caller_project`: the scope's `home`, else the workspace the active window shows), started by
  `BrowserHub::browser_for` when it is not running, and a named tab acts in whichever project's
  browser holds it, since each `Page` carries its own browser's connection.
- `browser_tabs` lists each page of every project's browser: its id, title, URL (with
  secret-looking values hidden), whether it loads, its `project` (the rail's name for its tab's
  workspace; for a page with no tab, its browser's project, #507), whether the user focused it
  last (`focused`), and `default`, the one a call from this caller that names no tab acts on.
- `browser_annotate` (#498, a write tool) draws the agent's box around a ref's element, scrolled
  into view first (`ref_origin`, which `place` shares, then `Page::border_box` plus the scroll
  from `Page::viewport`), or over an area of the viewport, and names the annotation; `clear`
  removes the agent's own. `browser_annotations` lists the page's boxes in page coordinates,
  with their notes, makers and times.
- `browser_recordings` and `browser_recording {id, frame?}` (#499) read the recordings' files
  off the main thread, before the browser needs to show: the list (each id, tab, URL, title,
  time, length, frame and entry counts), and one timeline with frame `frame`, from 1, as the
  image. Since #506 `timeline_for_agents` passes every string of the timeline through
  `agent_redactor` whole, and `browser_recording` then cuts each fill's text to `FILL_BUDGET`
  (1,000).
- `browser_draft_test {id}` (#506) reads the recording off the main thread, redacts it (a fill
  the redactor changes becomes secret) and drafts the test with `playwright::draft`. It writes
  nothing. It suggests `<project>/<testDir>/<slug of the title>-<id>.spec.ts`, where `testDir`
  is the plain relative path the project's `playwright.config.*` sets (`test_dir_in`), else
  `tests`. It answers the test, that path, the project, the variables, the skips, the start,
  `run` (`npx playwright test <path>`) and a note when the recording names no project or the
  project has no config.
- `browser_picks` and `browser_pick {id}` (#496) answer from the hub alone, before the browser
  needs to show, so a pick outlives its page and a restart: the list gives each pick's id, tab,
  URL and title, summary, caption and whether it was sent; `browser_pick` gives the pick with
  its bundle, and its crop as the image. Since #518 both answer from `pick_for_agents`, a copy
  whose page title, caption, blockers and listeners' events, and whose bundle's name, text, HTML,
  nearby texts, selection and locator values, pass through `agent_redactor` whole and are then
  cut to `pick`'s budgets (the HTML 4,096 characters, each text 200, the selection 500). Each
  listener's script URL goes through `redact_url`, and the summary is made again from the
  redacted bundle.
- `browser_check_pick {id}` (#505) is a write tool, answered after the browser shows: it acts in
  the pick's own tab through `page_of`, brings it forward with `show_for_agent`, says "checking
  pick N" and "checked pick N" in the Agent chip, and runs the hub's `check_pick`. It answers
  whether and by what the element was found, the changes, the element now, and its crop as the
  image, from `pick_for_agents`. That function redacts each page text of a pick and its check
  whole (`redacted`) before cutting it (`within_budgets`). It makes the check's change lines
  again from the two redacted bundles, then cuts each to `CHANGE_BUDGET`, so a secret in a text
  change reaches no agent.
- The read tools: `browser_look` (the hub's URL and title, `Page::viewport`,
  `focused_element`, the selection unless a password field has the focus, and
  `Page::screenshot` as the image), `browser_snapshot` (the main frame's tree, each same-site
  frame the main tree does not hold, and each cross-site iframe's session's tree, whose refs the
  hub keeps until the next snapshot), and `browser_console` and `browser_network` (the hub's
  rings).
- The write tools: `browser_navigate` (`address::agent_url`, then `BrowserHub::navigate_task`,
  which answers once the page has loaded), `browser_back` (`go_task`), `browser_click` (a ref
  scrolled into view and placed at its box's middle, through its iframe's owner when it is in
  one, or a point; the pointer moves, then each press and release), `browser_type` (a click on
  the ref, then each character as a key press), `browser_press` (`input::chord`) and
  `browser_scroll` (a wheel turn at the viewport's middle, or a ref into view). Each shows its
  action in the Agent chip.
- The hub observes the page and each cross-site iframe as it attaches (`Target.setAutoAttach`),
  keeps their sessions, the console and network rings, the snapshot's refs, the calls waiting
  for a load (fired when the main frame stops loading or moves within its document) and the
  agent's last action. The toolbar's Agent chip shows that action while it runs and for five
  seconds after it ends; a typed text shows as its length, never itself.

## Marley's plugin for Claude Code (`src/claude_plugin.rs`, `claude_plugin/`, #482)

- The plugin lives as files in the crate: a local marketplace named `marley` and the plugin,
  whose `hooks/hooks.json` runs `hooks/notify.sh` for `permission_prompt`, `idle_prompt` and
  `Stop`. The script answers only where `TERM_PROGRAM` is `zed`, with a `terminalSequence`
  holding an OSC 777 notify, which the interactive Claude Code writes to its terminal (print mode
  drops it).
- `ClaudePlugin`, a global, holds where the plugin goes (Marley's data directory), Claude Code's
  configuration directory, the `claude` to run, and whether `installed_plugins.json` lists
  `marley@marley`, read in the background at `init`.
- `install` writes the plugin, runs `claude plugin marketplace add` unless
  `known_marketplaces.json` has `marley`, then `claude plugin install marley@marley`, and shows a
  toast or the error. The agent bar's chip calls it.
- Since #491 (version 1.1.0) the plugin declares an MCP server, `marley`, in `marley/.mcp.json`:
  `bin/marley-mcp-bridge`, Python 3 with the standard library only. It reads the endpoint file
  (`$MARLEY_MCP_ENDPOINT`, else `${XDG_DATA_HOME:-~/.local/share}/marley/mcp-endpoint.json`),
  sends the bearer only to a loopback `http` URL, and passes each JSON-RPC message to Marley in a
  session of its own, replaying the client's `initialize` when Marley starts over and closing the
  session when its input ends. With no Marley it answers `initialize` and lists no tools; a thread
  checks every two seconds whether Marley answers and sends `notifications/tools/list_changed`
  when that changes. The root `.gitignore` ignores every `.mcp.json`, since a local one carries
  bearers, with an exception for this one.
- Since #519 (version 1.2.0) `hooks.json` also runs `hooks/event.py` for SessionStart,
  UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure and PermissionRequest (matcher
  `*`), Stop, StopFailure, PostCompact, SubagentStart, SubagentStop and SessionEnd. Python 3
  with the standard library, it answers `{}` outside `TERM_PROGRAM=zed` or on any failure, and
  inside a `terminalSequence` of an OSC 777 notify titled `marley-event`, the base64 of a
  summary of at most 2,900 bytes (the prompt and message cut to 300 characters, the tool's
  preview to 200; an overlong path goes first, then those fields), which keeps the sequence
  under Claude Code's 4,096-byte cap and Marley's scanner's 4 KiB. `FILES` ships it as a
  program; it takes about 14 ms a call.
- Since #547 `ClaudePlugin` keeps the installed version (`installed_version_in`, the user-scope
  entry of `marley@marley`), and `needs_update` compares it with the embedded manifest's
  (`shipped_version`, `semver`); an unparseable or newer one needs none. The chip then reads
  "Update Marley's plugin", and `update` writes the marketplace again (`write_marketplace`,
  shared with `install`), runs `claude plugin marketplace update marley` (or `add` when Claude
  Code forgot it) and `claude plugin update marley@marley`, and shows a toast or the error.
  `MARLEY_CLAUDE` names the `claude` to run: the app's PATH can come from the login shell, which
  finds the real one before a scenario's stand-in.

## The Browser tab (`src/browser.rs`, #488 to #490, #493 to #499, #504)

- `BrowserHub` is one entity per app, behind a global: a `ProjectBrowser` for each project's
  Chromium (#507: its `BrowserProject`, state, connection, the pages it is attaching or closing,
  the agents' placements, its start's generation and task, and a pending stop) and a
  `PageState` for each page of them (#493): the `Page`, the start that attached it, its newest
  frame, title and URL, its loading, history and dialog, its iframes, rings and refs, the
  agent's last action, its viewport and how many tabs draw it. A `BrowserProject` is Zed's
  project group (`ProjectGroupKey`): its main folders and host, the key `service::project_key`
  makes of them, and Zed's name for it. Making the hub starts nothing; `browser_for` starts a
  project's browser when it has none or it failed, from `marley: open browser`, `New Browser
  Tab`, a terminal URL's tab, a restored tab (`deserialize` reads the project Zed hands it,
  whose folders are added before its items) or an agent's new page. The process's first start
  moves the profile of earlier builds to its project, once, after closing that profile's
  Chromium (`move_legacy_profile`, a shared task every start waits for); then the project's
  `project.json` is written. `start` connects through the project profile's
  `DevToolsActivePort` when a Chromium answers there; otherwise, unless the unit is up, it
  removes a stale endpoint file, starts the unit and waits up to fifteen seconds, failing early
  when the unit stops. It then turns on target discovery and attaches the pages the
  browser lists, each once and each in a task of its own, with the page's observers on before
  the page is announced; a start opens no page (#494). The event loop routes each
  event to the page whose session it came from, or whose iframe's; attaches each `page` target
  discovery reports later (`targetCreated`, which carries `openerId` for a page a page opened);
  drops a page on `targetDestroyed` or `targetCrashed`, and one that went while it was being
  attached (`closing`); decodes each frame off the main thread, keeps it for its page and
  acknowledges it; follows each page's URL and title (asking for the title after
  DOMContentLoaded, load and same-document navigations, since no target event reports it); and
  fails with "The browser closed its connection." when the socket ends. A generation number
  drops a superseded start's late results: one counter for every project's starts, so a number
  names its browser and each result's guard reads `is_current(generation)`. A page streams
  while a tab draws it and its size is known.
- **A project's browser stops when the project leaves (#507).** `init` observes each
  `MultiWorkspace`: its `WorkspaceAdded`, `WorkspaceRemoved` and `ProjectGroupsChanged`, and its
  release when a window closes, each defer `review_browsers` (the event arrives inside the
  window's own update). A project is live while any window lists its group or holds a workspace
  of it, or while a Browser tab of it sits in a held workspace, which keeps a workspace whose
  folders changed on its tabs' browser. A browser whose project is not live gets a numbered
  stop; two seconds later it stops if that stop is still pending and the project is still not
  live: its pages go, their tabs with them, and `stop_chromium` sends `Browser.close`, waits up
  to five seconds for the unit to stop, then runs `systemctl --user stop` for whatever is left.
  `on_app_quit` sets `quitting` and drops the pending stops, so a quit, by the palette or the
  last window's close, leaves every unit running for the restored tabs.
- **Clear Browser Data (#581).** `clear_project_browser_data`, from the rail's project menu or
  the action `marley::ClearProjectBrowserData`, asks with Zed's prompt (Warning; Clear, then
  Cancel, which Escape picks), naming the workspace's project. On Clear it closes every Browser
  tab of the project in every window (`close_project_tabs`, over `close_views`, which forgets
  each tab's page so its removal closes nothing), from the app rather than inside the window's
  update, and runs `BrowserHub::clear_browser_data`: `forget_browser`, then `stop_chromium` on
  the browser's connection (or a new one, for a Chromium an earlier Marley left running), then
  `service::remove_profile_in` off the main thread. The clear is a `Clearing`, a shared task the
  hub keeps in `clears` while it runs: `start` waits for the project's before anything else, and
  a clear asked while one runs gets that one. A toast in the workspace reports it, or the error
  text.
- **Tabs as pages (#493).** The hub emits `PageOpened` once a page is attached and
  `PageClosed` when it goes, each naming the page's target id, and a subscription made with the
  hub's global answers them. A tab that shows the page already keeps it. A page a start found
  (`listed`, #494) goes to a tab of its project opened while the browser started, or else waits
  without a tab until a restored tab claims it or `marley: open browser` gives it one. Any other page gets a
  tab of its own. A page a page opened goes beside
  its opener's tab, with the focus. A page an agent's call opened for a project (#574) goes to
  the workspace the hub keeps for it (`placements`, which `create_page_task` records when
  `Target.createTarget` answers, several round trips before the page can attach, and which goes
  with a failed attach, a start or the page): after that workspace's tab the user focused last,
  else its newest, else in its active pane. Any other page goes after its project's tab the user
  focused last, or else after its project's newest Browser tab, or else into a workspace of its
  project (#507: the active window's when it shows the project, else another window's, else any
  held one). None of these takes the focus: in a pane that has
  the focus it
  joins the tab bar behind the active tab, since Zed gives a lost focus to the pane's new front
  item; when no Browser tab is open and the active pane shows other work with the focus in it,
  the page opens in a pane split to its right, and the focus goes back where it was. A new page
  takes the viewport of the page it opens beside, so a page behind another tab lays out as it
  will show. `PageClosed` closes each tab of the page, which first forgets the page so that its
  removal closes nothing. A registry of weak `BrowserView`s finds a page's tab.
- `BrowserView` is one page's tab, a `workspace::Item` that holds the page's target id (none
  while it waits for one), its project's key (#507: the page's project, or its workspace's while
  it waits), its workspace and its window: its text is the page's title (else
  "Browser"), its tooltip the URL, its icon the globe. It draws the hub's state ("Starting
  Chromium…", "Connecting to Chromium…", "Opening a page…", or the reason it stopped, with how
  to try again) or its page. It counts as its page's viewer from its first paint in front of its
  pane until `Item::deactivated` or its release, so a page behind another tab stops streaming.
  Its focus moves its page to the end of the hub's focus history (#574: each page once, the newest
  last, 64 at most, a page that goes dropped), which the agent tools' defaults read: `focused()`
  is its newest live page, else the newest page, and `focused_among` the same within a set. `Item::on_removed`, which Zed
  calls on a close and on a move between panes alike, defers a check past the effect cycle and
  closes the page (`Page::close`) only when no pane holds a tab of it: a move removes the tab and
  adds it again in one update, before the deferred check runs. The tab frees each frame from the
  window's atlas two paints after it was first drawn, and both kept frames on release, as Zed's
  screen-share view does, since the window may present the last frame again.
- **Saved and restored (#494).** `BrowserView` is a `SerializableItem` of kind `MarleyBrowserTab`,
  registered in `browser::init`. The workspace's layout holds the item alone; the tab's page id,
  URL and title go in its own table, `marley_browser_tabs`, of the `db` domain
  `MarleyBrowserTabsDb` (after `WorkspaceDb`, its rows deleted with their workspace), keyed by
  workspace and item: item ids repeat across launches, so a second migration rebuilt the table
  without the first's `UNIQUE(item_id)`, rows and all (#576). Zed saves the item when it joins a
  workspace and on `UpdateTab`, which the tab emits when its page's URL or title changes and when
  it takes a page; `cleanup` is Zed's `delete_unloaded_items`. `deserialize` builds a tab that
  claims its saved page id at once, so the start's `PageOpened` for that page finds it, and shows
  the saved title and URL until the page is back. The tab's restore task waits for the hub to show
  its pages and for the start's attaches: a page that is back is kept, and otherwise the saved URL
  opens in a new page, which the tab takes.
- `PageElement` reports the tab's size and the window's scale to the hub in `prepaint` (the
  page is laid out again only when either changes) and paints the frame from the tab's top
  left at its own size, so a frame from before a resize is neither stretched nor squeezed.
- `marley::OpenBrowser` ("marley: open browser") gives a tab to each page that has none (a
  start's that no tab claimed, and those whose tab closed with its window), the first with the
  focus; else activates the workspace's tab of the page the user focused last, or its first
  Browser tab, a restoring one included; else opens a tab and a blank page for it. A tab opened
  while the browser starts waits, in a task it holds, for the start's pages, through a failure
  and the start after it, and opens a blank page when none came to it. It restarts a hub that
  failed, and the tabs of the old pages close. `marley::NewBrowserTab` ("marley: new
  browser tab", Ctrl-T in `MarleyBrowser`) opens a tab after the active one with the focus in
  its address bar, and a blank page for it. The tab takes the page's id when
  `Target.createTarget` answers, which is before the page is attached, so the page's
  `PageOpened` finds the tab and opens no second one; an address typed before the page is
  attached is gone to once it is.
- **Input (#489).** `PageElement` inserts a hitbox and, in `paint`, registers the tab's input
  handler and its mouse listeners: a press in the page focuses the tab and records where it
  landed (an input method opens its window there, since CDP reports no caret); moves and the
  release reach the page wherever the pointer is while a button the page got is held; the
  wheel counts 100/3 CSS pixels a line with the sign turned. A point maps to CSS pixels through
  the frame's metadata (its DIP width over the width it is drawn at). The tab's `key_down`
  sends each key `marley_browser::input` maps, after Zed's bindings; Ctrl+V inserts the
  clipboard's text, and Ctrl+C and Ctrl+X read the selection onto the clipboard before their
  key goes out, so a cut cannot empty it first. As `EntityInputHandler`, the tab turns a
  preedit into the page's composition and a commit into inserted text. The hub sends input in
  the order it came, keeps the held buttons, and times the first frame after each press or key.
- **Navigation (#490).** A toolbar sits over the page: back and forward (`IconButton`s, disabled
  at the ends of the history), reload, which is a stop button while the main frame loads, and
  the address bar, a single-line `Editor` in a `MarleyAddressBar` key context that shows the
  page's URL, or where a navigation it asked for is going, whenever it does not have the focus.
  While the main frame loads, a 2-pixel accent bar runs over the toolbar's lower edge, drawn
  absolutely so a load never resizes the page. The hub follows the main frame's
  `frameStartedLoading` and `frameStoppedLoading` (its frame id is the target's), its
  `frameNavigated` and `navigatedWithinDocument`, which read the history again, and the page's
  dialogs; it emits `BrowserEvent` (`PageInfoChanged`, `DialogOpened`, `DialogClosed`). The
  page gets keys only while its own focus handle has the focus, so the address bar and a
  prompt's field keep theirs. The keys live in `keymap.json`: `MarleyBrowser` binds Ctrl-L
  (`marley::FocusAddressBar`), Alt-Left and Alt-Right (`BrowserBack`, `BrowserForward`), and
  Ctrl-R and F5 (`BrowserReload`), which beat Zed's workspace bindings for those keys while the
  tab has the focus; `MarleyAddressBar > Editor` binds Enter (`GoToAddress`) and Escape
  (`RestoreAddress`).
- **Dialogs (#490).** A JavaScript dialog is Zed's `AlertModal` over an occluding layer on the
  page, in a `MarleyBrowserDialog` key context: "<host> says" and the message, a single-line
  field with the default for `prompt`, Cancel (not for `alert`) and OK; `beforeunload` reads
  "Leave this page?" with Leave. Enter answers OK and Escape Cancel, from the card or the field
  (`AnswerDialog`, `DismissDialog`). The dialog takes the focus only from inside the tab, and
  the tab's focus goes to it while the page waits; a navigation, back, forward or reload
  answers an open dialog with Cancel first, as Chrome closes a page's dialog when the page is
  left.
- **Select lists (#495).** The hub watches each page's session once the page is attached, and
  each cross-site iframe's once it is observed (`Page::watch_selects`). A `Runtime.bindingCalled`
  from the select world becomes the page's `select` (the session, the listener's context, the
  request) and `BrowserEvent::SelectOpened`. The tab opens a `ContextMenu` for it only within a
  second of the user's own press or key in the page, so an agent's click opens nothing: a header
  per group and a separator after one, a checked entry for the selected option, disabled ones
  greyed. The menu starts on the current option, takes the focus, and is drawn with
  `deferred(anchored())` at the select's bottom left, from the frame's mapping at the last paint
  and, for a press, the difference between the press in the page and in its frame, which places
  it right in any frame. A choice calls the hub's `choose_option`, a dismissal its
  `dismiss_select`, and either gives the page the focus back.
- **Picks (#496).** The toolbar's Crosshair button, lit while the page picks, and
  `marley::PickElement` (Ctrl-Shift-C in `MarleyBrowser`, and in `MarleyBrowser > Editor`, where
  Zed's `!Terminal` binding of the key would otherwise win) call the hub's `set_picking`, which
  turns on the page's Debugger domain the first time (its `scripts` map is `None` until then)
  and inspect mode (`Page::set_inspect`); the page takes the focus. `MarleyBrowser` binds Escape
  to nothing, so it reaches the tab's `key_down` rather than Zed's `workspace::Unfollow`: in pick
  mode it ends the mode, and otherwise it goes to the page. `Overlay.inspectNodeRequested` ends
  pick mode and reads the pick in a task (`capture_pick`, then `crop`); the session's `Pick`s,
  numbered from 1, keep their tab, redacted URL, title, summary, caption, whether they were sent
  and the bundle, and `BrowserEvent::PickStaged` gives the tab's caption field the focus. The
  tray, under the toolbar while its page has picks, lists them newest first in rows of one
  height: the pick's number and summary, then a caption field (`MarleyPickCaption`, where Enter
  is `marley::SendPick`), Send and Discard, or what was sent. Send types
  `[browser pick N: <summary> on <host/path>; browser_pick id N] <caption>` with
  `Terminal::paste` into the terminal the user focused last (`LastTerminal`, kept by an
  `on_focus_in` on every `TerminalView`), after bringing its tab to the front of its center pane
  or Terminal Panel, in whichever workspace of the window holds it, and focusing it; that work
  is deferred, since the terminal's pane may hold the tab itself. A sent pick's Discard takes it
  out of the tray and keeps it for the agent. A pick that did not read, or a Send with no
  terminal used yet, says so in the tray.
- **Listener sources (#497).** Once a pick is staged, its capture task reads each distinct source
  map of its listeners (`original_positions`, `load_map`; the parse and the scans on the
  background executor, as `futures::future::lazy`) and `pick_sources` sets each listener's
  `original`, its `file` from `find_source` in the project of the pick's tab: an absolute
  source inside a worktree, else the longest suffix of the source's path, down to two
  components, that `entry_for_path` finds as a file (a lone name only when that is all the
  source names). The row shows the first listener whose file is in the project, else the first
  with a script: its event, then `src/app.ts:2` as a link, or the original source's or the
  script's name and line muted, or `…` while the maps are read; the tooltip lists every
  listener. The link opens the file with `Workspace::open_path` and
  `Editor::go_to_singleton_buffer_point` at the line, from a task, since the file opens in the
  tab's own pane; a file gone since, or an open that fails, says so in the tray.
- **The component's source (#518).** The same task maps a React 19 pick's `_debugStack` frames
  (`stack_source`): through each frame's script's source map, the first frame that is not React's
  own (`StackFrame::is_reacts`) and whose original source lies outside `node_modules`, or, for a
  script without a map, whose URL does. `pick_sources` sets it as the component's source with its
  `file` from `find_source`, and gives a React 18 `_debugSource` its `file` the same way.
  `MapCache` loads each map once per pick, for the listeners and the stack together.
- **Checks (#505).** `BrowserHub::check_pick(id)` runs a pick's check in the pick's own tab:
  pick mode off first, then `check_element`, which finds the element again (`Page::refind`),
  scrolls it into view on the page's session, gives the scroll `SCROLL_SETTLE`, reads it through
  `capture_pick` and crops it. The result is `Pick.check: Option<PickCheck { found_by, changes,
  bundle, crop, checked_at }>`, the latest only, and a check changes nothing the rail shows. Each
  tray row ends in `render_check`: the latest check's verdict ("2 changes", "no change", "not
  found"), which opens the comparison, and Check ("Checking…" while it runs). The user's Check
  opens the card when it answers; an agent's does not. The card (`render_comparison`,
  `render_crop`) is an absolute child of the page area at its top right, occluding. It holds the
  pick's number and summary, what found it again, the crop at the pick beside the crop now, and
  the change lines, each cut to 300 characters on the card only. The tab decodes both crops once,
  when the card opens (`open_comparison`), drops them from the window's atlas when it closes, and
  at each render follows a newer check of its pick, or closes on a discarded one
  (`refresh_comparison`).
- **Annotations (#498).** A page's `annotations` live in the hub (`Annotation { id, page_box,
  note, maker, made_at }`, numbered across the session) until its main frame shows another
  document (`frameNavigated`); a fragment or history move keeps them. The view's render reads
  the frame and its metadata together and places each box from them (`Placement`: the scroll,
  the pinch scale, the top offset and the DIP over the drawn size), as absolute children of the
  page area, which clips: a border and a light fill in the user's color (the theme's warning) or
  the agent's (its accent, with the Sparkle icon on the note), the note on a chip above the box.
  Nothing reaches the page to draw, and a box has no hitbox, so the page under it still takes
  the pointer. Annotate mode (`AnnotateMode`, the toolbar's Pencil, `marley::Annotate`) has
  `PageElement` take the press, move and release as a drag in document points while the wheel
  still scrolls the page; the release opens a note field (`MarleyAnnotationNote`, Enter
  `marley::KeepAnnotation`, Escape `marley::DropAnnotation`), and Escape in the page ends the
  mode. A chip occludes the page under it: its click selects the box and gives the page the
  focus, and Delete or Backspace then removes it; a press in the page drops the selection.
- **The flight recorder (#499).** Each page's `recorder` is fed by `record_entry` only while a
  tab draws the page: the hub's `mouse_press`, `wheel`, `send_key`, `copy_selection` and
  `insert_text` (`key_entry` counts a key that types, or a single character that is not a
  letter or a digit, which is how AltGr's characters arrive, and names any other key or
  shortcut), `agent_ended`, `observed` (console entries; requests as they start, then their
  status or failure), the main frame's `frameNavigated`, a snapshot after each load
  (`record_snapshot`, spawned so the event loop does not wait on the tree), and `show`, which the
  frame loop hands each frame's base64 back to. `record(target, dir, project)` takes the minute,
  adds a snapshot of the moment and writes it off the main thread into `recordings_dir()`,
  `browser/recordings` under Marley's data directory. The toolbar's red dot (`IconName::Circle`
  in the error color) and `marley::RecordThis` save it, and a toast in the tab's workspace names
  the recording or the failure. Since #506:
  - `watch_actions` runs beside `watch_selects` for the page's own session, and
    `Runtime.bindingCalled` goes by the binding's name to `action_reported` or
    `select_requested`;
  - an action the listener reports is kept through `record_entry` like any entry, and a move
    within the document is a navigation with `within`;
  - `record_this` passes the tab's project root, its workspace's first visible worktree.
  An agent's own clicks and keys go to the page as the same trusted input as the user's, so the
  listener records them as actions too; the Agent chip's text says whose they were.
- **What the rail reads (#504).** `BrowserHub::try_global` answers the hub when something made
  it, and never makes one. `BrowserEvent::PageStatusChanged { target }` says that a page's
  loading, its icon, its picks or annotations, or the agent's mark changed, beside
  `PageInfoChanged` for the title and URL. After each `Page.loadEventFired` whose origin has no
  icon yet, `read_favicon` asks `Page::favicon_href`, loads the bytes (or decodes a `data:` URL),
  names their format, and keeps `Favicon { origin, image }` on the page when the generation and
  the origin still match. `navigated` drops it when the page goes to another origin or to a URL
  with none (`about:blank`, `file://`), as Orca's `browserNavigationLeavesFaviconOrigin` does. A
  read that fails logs at debug and leaves the globe. `agent_ended` sets `agent_unseen` when the
  page has no viewer, and `add_viewer` clears it.

## Tests

`src/marley_workbench_tests.rs` (the switch, the keymap loader, persistence and the docks across
a switch, 29 tests) and
`src/rail_tests.rs` (the rail, 49, 14 thread tests in `rail::tests::threads`, and 5 agent-CLI
tests in `rail::tests::agents`, which search a temporary directory for programs and read the
new terminal's write log): driven gpui tests on `MultiWorkspace::test_new`
over a FakeFs, clicking elements found by debug selector. The keyboard tests dispatch the
`menu` actions from the focused rail; one binds Zed's Linux default keymap and presses
`ctrl-alt-;`, the arrows and Enter. The filter tests type into the field with `simulate_input`;
two bind Zed's keymap and the Marley keymap and press `ctrl-f`, in the rail and in the
add-project picker. The switcher tests hold `ctrl` with `simulate_modifiers_change` before
opening, as a hand does, and let go of it to confirm. The thread tests run on
`init_agent_test`, which layers Zed's agent test setup (`agent_ui::test_support`, with a
thread database per test) over the window. They put an `AgentPanel::test_new` in each project
and drive threads through `acp_thread::StubAgentConnection`: a turn that stays open until
`end_turn`, or a tool call waiting on a permission.

`src/routing_tests.rs` (the routing, 20 tests) runs real shells and tasks, as Zed's own panel
tests do, with the executor allowed to park. Each window loads the Terminal Panel through
`TerminalPanel::load` and adds it, as `crates/zed` does, so Zed's provider is in place first
and the routing's has to replace it. Actions are dispatched from the focused center pane, below
the workspace's root. The toggle tests stand `workspace::item::test::TestItem` in for an editor
and a `TestPanel` in for another bottom panel. One binds Zed's real Linux default keymap
(`KeymapFile::load_asset_allow_partial_failure`) and presses the keys.

`src/agents_tests.rs` (the picker, 9 tests) opens the picker by dispatching `marley::NewAgent`
from the center pane and drives it with `menu::SelectNext` and `menu::Confirm`, over a project
with an Agent Panel, two custom agents (one named and drawn by the registry) and two fake CLIs
in a temporary directory. The harness in `marley_workbench_tests.rs` holds what both test
files use: the display-only terminal factory, `programs_in`, `search_agents_in`,
`use_display_only_terminals`, `add_agent_panel` and `configure_agents`. The keymap's hook
has its own test in `crates/zed/src/zed.rs`, `test_reload_keymaps_binds_the_marley_keymap`.

`src/agent_bar_tests.rs` (the agent bar, #482's chip and Attach File, 10 tests) and
`src/notifications_tests.rs` (5) run a real PTY whose shell `exec`s `claude`, a link to `sleep`
on a scratch PATH (`fake_claude_bin`), in a real scratch folder that the project's FakeFs
repository covers, on branch `main`. The tty echoes the spaces a test sends, so the terminal
reads its foreground process again. The chip's tests install with a fake `claude` that logs its
arguments. Attach File's tests click the `+` and answer the test platform's path prompt with
`simulate_path_prompt_response`, then read the terminal's PTY write log.
`src/claude_plugin_tests.rs` (4) writes the plugin into a scratch directory and runs its hook
script.

Since #483 no test is added (CONSTITUTION §7): the files above stay and keep building, and each
change is proven by an e2e scenario in `script/e2e/`. `480-voice-input.sh` drives the
microphone through a fake Voxtype whose `record toggle` moves its status on, and
`481-rich-input.sh` types into the rich input and reads what a stand-in agent prints;
`484-autosuggestions.sh` types prefixes at a bash with a history file of its own.

## Known limits

- A key Zed binds above the Browser tab goes to Zed, not the page: Ctrl-S saves, Ctrl-W closes
  the tab. Escape reaches the page since #496. Pick mode runs in the page's own session, not a
  cross-site iframe's, so picking inside such an iframe is outside #496. A page whose framework
  delegates its events (React's root listener) shows the framework's listener, whose source is
  the framework's code, not the handler the app wrote (#497). Since #518 `browser_pick` gives
  agents the React component and the file it was written at; the tray still shows the listener.
- A check looks for its element in the page's main document only: a pick made in a cross-site
  iframe or a shadow root checks as not found (#505). The crop at a check, like the pick's, is
  the page's pixels, so the card and `browser_check_pick`'s image show the visible text
  unredacted.
- The tray's Send types a pick's summary as the page shows it (#496). Redaction covers what
  agents read through Marley's tools (#516, #518). A Send is typed at the user's prompt, where
  the user sees it before Enter, as with #549's selection.
- A Browser row keeps a page's first icon while the page stays on its origin: an icon the page
  swaps without a navigation (an unread count), or another page of the same origin with an icon
  of its own, still shows the first. A page whose load event never comes keeps the globe.
  Browser tabs are not in the rail's switcher, and a Browser row lights no attention dot on a
  folded project (#504).
- A Browser tab's page outlives its window: closing a window, or quitting, closes no page. A
  tab restored at launch takes its page back (#494); a page no restored tab claims gets a tab
  the next time `marley: open browser` runs, or when an agent acts in it. A navigation in the
  moment before a quit may go unsaved, since Zed throttles item saves. History across a
  Chromium restart, scroll positions and form contents come back only when the page lived on.
- The title bar's Panel Layout submenu still lists Classic and Agentic in the Marley layout,
  with "Custom" checked; choosing one explains itself (#451), and hiding it needs a
  `title_bar` touchpoint.
- Vim's `:!` and external agents' login terminals still open in the Terminal Panel: they call
  the panel directly. While such a panel is open, `` ctrl-` `` still toggles the center
  terminals, and `ctrl-j` closes the dock.
- A Terminal Panel moved to a side dock by `terminal.dock` still opens with that dock's toggle
  (`ToggleLeftDock`, `ToggleRightDock`).
- The settings UI shows the patched values as the defaults: in the Marley layout a stored
  `terminal.button: false` looks like the default and has no reset control.
- Each layout round trip adds a subscription pair on the kept Zed sidebar, and a window
  restored in the Marley layout saves a partial state before its restore finishes
  (`docs/planning/intake/rail-internals.md`). The docks' memory of a round trip lives in memory
  only, so a window saved in one layout and restored in the other starts without it.
- A restored window builds its rail open and closes it once the restore is over, through
  `close_sidebar`, which records Zed's "Sidebar Toggled" event; Zed has no silent close.
- The filter ignores case for ASCII letters only, as Zed's does, and vim's `/`, which reaches
  Zed's sidebar filter, does not reach the rail's.
- A first terminal comes only with a folder opened fresh. A project opened before, in either
  layout, reopens as it was saved, and switching a window to the Marley layout seeds nothing.
- `ctrl-tab` in a center pane stays Zed's tab switcher, over that pane's items. The rail's
  switcher shows no preview while cycling, and its recency lives in memory only.
- Vim's `ThreadsSidebar` bindings (`] p`, `[ p` and the rest) do not reach the rail, whose key
  context is `MarleyRail`.
- The thread rows, the agent rows, the routing, the terminal keys, the New Agent key, the
  rail's persistence, its keys and reorder, its filter, its switcher, the first terminal, a
  project's folder changes and Next and Previous Project and Thread have not been seen live:
  the drives for #439 and #440 would have moved Chad's windows off his monitor, and the later
  ones need input or a relaunch. They are owed to the next headless capture.
