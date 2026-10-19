# `marley_workbench`

The Marley layout, written in the fork for the workbench shell's W2 (#438): the
`marley.layout` setting, the switch between Zed's sidebar and the rail, the rail itself, since
W5 (#441, #449) the terminal routing, and since W5c (#450) the New Agent picker and the Marley
keymap.
On macOS `load_keymap` also binds `keymap-macos.json` after `keymap.json` (#789): Cmd twins of
Marley's own Ctrl keys, in the same contexts. The shell keys and Ctrl-G have no twin.
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
  `<data dir>/marley-dev.sock` (`single_instance::socket_path`, which Zed's listener calls since
  #760; `zed-dev.sock` before), so `--user-data-dir` gives a Marley a socket of its own. It runs only
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
- The launcher's activation token travels with the hand-off (#545). `keep_activation_token()`,
  first in `main`, reads `XDG_ACTIVATION_TOKEN` before gpui's Wayland client takes it out of the
  environment; `send` puts `zed://marley-activation-token/<token>` ahead of the paths. The running
  Marley's listener thread (`open_listener.rs`) reads it back with `activation_token_in` and hands
  it to `gpui::set_next_activation_token` instead of opening it, and the next Wayland `activate`
  uses it once. A compositor that checks tokens then brings the window forward as the launcher's
  click; without a token, gpui asks for its own, which such a compositor refuses while another
  window has the focus (L-claude-513-gpui-asks-for-activation-from-the-window-it-activates-001).

## The rail (`src/rail.rs`)

- **#713, the rail's cleanup.** `OutputGate` keeps what the rail last built for the rows terminal
  output touches: each terminal's row before the passes mark it, each center pane's tabs, each
  workspace's active pane and tabs, and what the rail reads of its window (`WindowSeen`). A
  wakeup, a terminal's `UpdateTab`, a pane's `ChangeItemTitle`, a workspace's
  `ActiveItemChanged` and the window's notify refresh only when theirs changed (`row_changed`,
  `pane_changed`, `workspace_changed`, `window_changed`), at the end of the effect cycle
  (`refresh_soon`), so one change makes one refresh; every other event refreshes as before. Each
  refresh logs `rail <id> refresh` at debug. A rail built with its window (no kept Zed sidebar)
  holds the window's saves (`MultiWorkspace::marley_hold_serialization`, set by
  `register_sidebar` before the first open) until no group its saved state lists is still
  pending, or until the settle for a window no restore reaches, and at most 10 s
  (`SAVES_HELD_AT_MOST`); the release is logged at info and deferred out of any update. During
  the hold a reorder keeps the saved places of the groups not back yet (`keep_unlisted`, and
  every saved row). `register_sidebar` and the restored close use the silent
  `restore_open_sidebar` and `restore_closed_sidebar`, and the window keeps one sidebar's
  subscriptions. Home's page holds the rail weakly and follows the window's current one.

> Since #644 the rail has two views under its header, Projects (all that follows) and Brain, Rusty's
> vault; see [The rail's Brain view](#the-rails-brain-view-srcrustybrainrs-srcrustyrs-srcrailrs-644).

- It implements `workspace::Sidebar`, so the `MultiWorkspace` keeps the resize handle, the open
  state, persistence and the toggle actions.
- It follows every workspace, every listed terminal view and every Browser tab (#504). On each
  event it rebuilds the snapshot and redraws only when the pure `marley_rail::RailSnapshot`
  changed. `render` reads nothing but the rail, so a background terminal's output never redraws
  the window through it, and `has_notifications` reads the stored snapshot, so it holds while the
  rail is closed.
- It lists every project group of the window, named through Zed's public functions
  (`compute_disambiguation_details`, `ProjectGroupKey::display_name`) in `crate::group_names`,
  which the browser tools share for a tab's project (#574).
- **Closed projects (#606).** A group the window holds no workspace of (a restart reopens only the
  shown one) gets a header-only `ProjectSnapshot { closed: true }` and a `GroupEntry { closed:
  true }` whose `workspace` is `WeakEntity::new_invalid()` (`push_closed`). Its header dims its
  name (`Color::Disabled`) and icon, keeps the chevron's room with an invisible, disabled
  `Disclosure`, has no `+`, takes its element ids from `closed_id` (a hash of its key, since every
  invalid handle shares one entity id) and says "Not open. Click to open it." A click, or Enter
  through `open_row`, runs `open_closed_project`: `MultiWorkspace::find_or_create_workspace` with
  `OpenMode::Activate` and `remote_connection::connect_with_modal`, as the Threads Sidebar's
  `open_workspace_for_group` does; a failure is a toast. Its menu disables Clear Browser Data…,
  and `follow_icons` searches a closed local group's folder for its icon.
- **Rows under a closed project (#617).** `push_closed` lists a closed group's threads and ports:
  `listed_threads` and `group_threads` take an optional listed workspace, the store's
  `entries_for_main_worktree_path` and `entries_for_path` need only the key's path list and host,
  and with no workspace the agent's icon and name are read against the shown workspace's project.
  `ThreadEntry.closed` and `PortMenu.closed` carry the group's key. `header_folds` gives a closed
  header with rows its `Disclosure`, whose click stops before the header's own click opens the
  project. Opening goes through `Rail::in_workspace`: an open project's workspace is shown and the
  action runs at once; a closed one's `open_closed` (the folders to open, the key as the
  provisional key) is awaited first. A thread opens its own folders, as the Threads Sidebar's
  `open_workspace_and_activate_thread` does, then `load_thread`, which loads the Agent Panel with
  `AgentPanel::load` and adds it when the new workspace has none yet. `open_port` and `show_logs`
  open the key's folders. In `ports.rs`, `project_folders` gives a closed local group its key's
  folders and `project_names` names closed groups; `marley_rail::cycle_row` passes over a closed
  project's rows.
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
    `raised`, the title over the second line when there is one. Since #796 `render_terminal_row`
    pads its lines to two with blank ones, so a terminal row is always the two-line 3.5 rem and a
    line that comes or goes moves nothing. A shell's icon is a `>_` in the
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
- **Move to another project (#613).** A terminal row's menu has Move to Project, a submenu of the
  window's other open groups (`move_targets`, computed at render so the menu reads no entity), and
  a header takes a dragged terminal row of another project (`Rail::takes_terminals` on the
  header's frame, not its block, since the block's rows take the same `DraggedRailRow` for a
  reorder; `DraggedRailRow::moves_to`). Both run `move_terminal`, deferred through
  `window.defer`: `pane_for` the view, the target workspace's active pane, Zed's
  `workspace::move_item` to its end, then the target shown. The same view moves, so its
  `Entity<Terminal>` and every Marley global keyed by its entity id stay; Zed's
  `added_to_workspace` (Marley hunk) re-points its `workspace` and `project` and resubscribes, and
  moves its database row and Marley id as for any add (#575), so it restores under its new project.
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
    notify, which fires on every frame. A click the hub begins or stops holding (#571) is a
    `PageStatusChanged` too, for the inbox (#508).
  - The page's image lives in `Snapshot.favicons`, beside the pure snapshot, and its id in
    `BrowserSnapshot.icon`, so an icon's arrival changes what `refresh` compares.
- **Port rows (#521).** `build_snapshot` gives each project the listeners `ports::Ports` holds
  for its group (`port_snapshots`): the title `:<port> <name>`, matched by the filter as a title
  is, the URL as the second line (`marley_browser::ports::url`), and a tooltip with the command
  line, `in <cwd>` and `pid <pid>`, and since #603 `user service <unit>` or `system service
  <unit>` with the service's unit as a line of its own under the URL (`PortService`). Stop's
  tooltip (`stop_words`, `Tooltip::with_meta`) says it stops the user service, the system service
  or the process, with the command or the signal, and `stop_port` shows a refusal as a toast with
  Copy Command (`refused_toast`). `render_port_row` draws `Server` in the row's round icon, and
  on hover (`visible_on_hover`) Open (`ToolWeb`), Copy and Stop, each in a div with its own
  debug selector (`marley-rail-port-open-<port>` and so on). Since #618 the buttons are an
  absolute strip over the row's end (the row is `relative()`), on `panel_background` blended
  with the row's hover or selected fill so it covers the text, and the text has the row's width.
  `RowLine.cut` (`Cut::End`, `Start`, `Middle`) picks the label's truncation: the URL line is
  `url_label` (no `http://` before `127.0.0.1:` or `localhost:`) cut in the middle, the unit line
  cut at its start. The row's tooltip begins with the whole URL. A closed header's tooltip is
  built in its `right_click_menu` trigger behind `!menu_open`, as the port row's is (#615).
  - A double-click on the row, Enter, or Open runs `open_port`: the group's workspace is shown
    (`activate_workspace`), then `browser::open_url_tab` opens the URL there, or brings forward
    the tab already on it. Since #604 one click runs `mark_row` instead, which focuses the rail
    and puts `cursor` on the row, so Enter (`confirm` → `open_row`) opens it; the row's tooltip
    ends with that. The buttons stop the click's propagation, so the row does not act on it a
    second time.
  - Copy writes the URL to the clipboard. Stop runs `ports::stop` off the main thread and shows a
    toast in the window's workspace when the process no longer listens there or the signal
    failed; the row goes with the next scan.
  - The rail keeps the scan running only while it shows. `watch_ports_while_shown` calls
    `ports::watch` once the rail is the window's sidebar, open, with AI on, and
    `ports::unwatch` once it is not. The observer the rail puts on its `MultiWorkspace` runs
    it, and so does one on the settings, since turning AI back on shows an open rail with no
    word from the `MultiWorkspace`. `cx.on_release` unwatches a rail that goes while watching. The rail observes the `Ports` global (`observe_global_in`),
    so it rebuilds when a scan changed what listens.
- **The inbox (#508).** `build_snapshot` gathers, for each project group, what waits on the
  user (`inbox_entries`): each Agent Panel conversation's `pending_tool_call`, through
  `thread_entry` (the tool call's label on one line, and the agent's name and icon from
  `agents::thread_agent_name` and `thread_icon`, as its thread's row has them); each center
  terminal whose `AgentEvents` seat is `State::Waiting`, with the seat's question or "Waits for
  you"; and each Browser tab whose page holds a click (`BrowserHub::pause_sentence`), once a page.
  Each is a `marley_rail::InboxEntry` in `RailSnapshot.inbox`, with an `InboxTarget` beside it in
  the workbench's snapshot: for a thread, the thread's key, the conversation's weak handle, the
  session, the tool call and the allow-once and reject-once options with their kinds
  (`first_option_of_kind`), or no options when the prompt lacks either or carries
  `sandbox_authorization_details`; for a terminal, its id; for a click, the page.
  - `note_inbox` keeps when the rail first saw each key (`inbox_seen`, pruned at each refresh),
    sorts the entries by it and gives each its age through `waited_words`. While any entry shows,
    a 30-second timer refreshes the rail, so the ages move.
  - `render_inbox` draws "Needs you" and the count between the filter and the rows, and each entry
    as a `row_card` with its age at the end; Deny (Refuse for a click) and Allow sit under an
    entry that answers in place. The buttons stop their click's propagation, so the card's click
    does not open the entry as well.
  - Allow and Deny on a thread run `answer_thread`: out of the rail's update (`defer_in`), it finds
    the conversation's thread view again (`thread_view(&session)`) and calls the panel's own
    `authorize_tool_call` with the option and its kind, so a closed thread answers nothing. Allow
    and Refuse on a click call `BrowserHub::answer_pause`. A click on an entry runs
    `open_inbox_entry`: the thread (`open_thread`), the terminal (`activate_terminal`), or the tab
    with the focus on its card (`browser::show_paused`).
  - The rail hears of a thread's prompt from the thread's `ToolAuthorizationRequested` and
    `ToolAuthorizationReceived`, of a seat's wait from its observer on `AgentEvents`, and of a held
    click from the hub's `PageStatusChanged`.
  - **Risk chips (#568).** While the use `inbox` is on, each tool entry is read into a `Waiting`:
    a thread's call through `thread_waiting` (its kind's class, the raw input's `command` or the
    label with its Markdown escapes taken out, its locations and the raw input's `path`, a
    terminal tool's `cd`), a seat's wait through `seat_waiting` (`Permission for <Tool:
    preview>` split into the tool and its preview, the seat's `cwd` label, or a question). `mark`
    classifies it (`marley_agent::risk`, with `secret` from `mcp::model_redactor`), sets its
    chips and level, and keeps its `RiskAsking` in the snapshot: the `Asking` with the tool's
    name, the agent, the project's name and `code found` as facts and the ask as text, and
    `system_one::nouls_verdict` when the rules marked it. A held click's entry takes the chip of
    its class (`BrowserHub::pause_class`) and nothing more. `follow_risk`, at the end of each
    refresh, logs each new tool entry once, a `rules` row for a chipped one and a call
    (`INBOX_RISK`) for the rest, whose reading (`risk_reading`: the nouls that hold, and the
    urgency's expected level rounded by comparison) lands by key and ask and refreshes; when an
    entry with a row leaves, its outcome says `allowed` or `denied from the inbox`, or `cleared
    elsewhere`, and after how long. `note_inbox` adds a reading's chips in `suggest` and `act`,
    lets it raise the level in `act`, and sorts by `(Reverse(level), first seen)` while the use
    is on. `render_chips` draws the chips on the line under the card, before #508's buttons: the
    rules' plain, in the error color at level 5, a reading's with a question mark in `suggest`
    and a dashed border in `act`, with its probability in a tooltip.
  - **Who should answer (#570).** `RiskScope` names which uses read the entries, and `mark`
    classifies each tool entry once for both: #568's chips and level while `inbox` is on, and,
    while `question_route` is, `route::classify` over those chips, the tool's name and a
    question's options (`seat_waiting` carries them and the seat's `prompt` label), its rule's
    mark on the entry and a `RouteAsking` in the snapshot (facts `tool`, `agent`, `project`,
    `chips`; texts `ask`, `options`, `prompt`; the rules' `choice_verdict`). A held click is the
    owner's by its class. `follow_route`, after `follow_risk`, notes whether each entry's terminal
    is the rail's focused terminal holding the focus, or its thread the one the panel shows with
    the focus, in an active window; logs `owner` or `agent after … s` for an entry with a row that
    leaves (`owner` too when an inbox button answered it); and logs a `rules` row or starts one
    ask (`QUESTION_ROUTE`) per new entry, whose reading (`route_reading`: the choice at or above
    the floor, else `unclear`) lands by key and ask. `note_inbox` puts a reading's route on its
    entry in `suggest` and `act`, sets `route_suggests`, and in `act` sorts by `(Reverse(level),
    rank, first seen)`, an entry not marked yet ranking as unclear. `render_route` draws the mark
    after the chips, with "Marley's rule: …" or "System One: … (0.88)" in its tooltip.
- **Keys and reorder (#453). The key context is `MarleyRail menu`, and the rail answers Zed's
  `menu::SelectNext`, `SelectPrevious`, `SelectFirst`, `SelectLast`, `SelectParent`,
  `SelectChild` and `Confirm`. Zed binds up, down, Home, End and Enter to them with no context,
  and left and right in `menu`, so the rail binds no key. Each moves `cursor`, the keyboard's
  row, which `refresh` hands to `marley_rail` while the rail holds focus; a focus-out
  subscription drops it. Left folds an open project or climbs to its header, right unfolds, and
  Enter runs the row's click handler. A project header's right-click menu has Move Project Up
  and Move Project Down, disabled at the ends (since #602 they move the project in the rail's own
  header order, `move_project`, rather than through `MultiWorkspace::move_project_group_up`), and after a separator Clear Browser Data… (#581) and Remove Project
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

## Ports (`src/ports.rs`, #521)

- `Ports` is a global: the listeners of each project group (`ProjectGroupKey`), each with the
  project folder that holds its working directory (`ProjectListener`), a count of the open rails
  (`watchers`) and whether the scan runs.
- `watch` counts an open rail in and starts the scan when none runs; `unwatch` counts it out. The
  scan (`scan_while_watched`) is one foreground task. Each round it reads every window's project
  groups and each local member workspace's root paths on the main thread (`project_folders`),
  then reads `/proc` and attributes the listeners on the background executor, and waits three
  seconds, or 30 after a round that took over 500 ms. It ends in the first round that finds no
  rail open.
- gpui's `global_mut` and `default_global` tell every observer of the global, changed or not, so
  the round reads `Ports` through `try_global` and writes it only when the listeners changed.
  A scan that fails leaves the last listeners and logs a warning.
- On macOS (#788) a round reads the listeners through the `process` adapter. `mac_listening` runs
  `/usr/sbin/lsof -nP -iTCP -sTCP:LISTEN -F pctn`, read whatever its status, since lsof exits 1
  when nothing matches. `mac_listeners` adds `lsof -d cwd` and `/bin/ps` for those pids, at most
  three programs a round. `proxied` is empty, and a test's `proc_root` stands for nothing
  listening. `stop` finds the pid in a fresh `mac_listeners` and signals through `stop_among`.
  `list`, the agents' `ports_list`, runs `listing`, one function per platform.
  `links::watch_ports` takes the footer's ports from `reachable_ports(mac_listening())`.
- `attribute` gives each listener to the group whose folder is the deepest one holding its
  working directory, compared by path components, across every window; a listener in no folder
  is dropped. So are Marley's own: its pid, any process named `marley` (another Marley's MCP
  server, a project Chromium's relay, which runs Marley's executable), and any whose command line
  names `<data dir>/browser`, a Chromium an earlier build started on a port.
- `list` scans at once for `ports_list`, and `project_names` names each group as the rail does
  (`crate::group_names`).
- `stop` (#603) scans at once on the background executor. When the pid no longer listens on the
  port, it answers `Stop::Signalled(Stopped::NotListening)`. A listener in a systemd service goes
  to `stop_unit`, which runs `systemctl [--user] stop <unit>` through `process::output` (the
  system's through polkit, which asks the desktop's agent) and answers `Stop::Unit`, or
  `Stop::Refused` with systemd's reason (`refusal_reason`: the first line of its stderr, after
  `Failed to stop <unit>: `). Any other listener goes to `marley_browser::ports::stop_in`
  (SIGTERM). `attribute` and `stop` both treat a listener in the unit Marley itself runs in
  (`own_service_in`) as a process, since stopping that unit would stop Marley. `hand_command`
  gives the command that stops a service by hand, which a refusal's toast offers to copy.

- **Container ports (#614).** The scan's background part also runs
  `marley_browser::containers::proxied_ports_in` (each `docker-proxy` command line, split on NULs
  and spaces); `container_ports` takes those, and the user's Podman helper listeners
  (`Engine::of_helper`), as container ports. `ask_engines` asks `docker ps --format '{{json .}}'`
  or `podman ps --format json` through `process::output` when an engine's ports changed or its
  answer (`EngineAnswer`) is 30 s old, a refusal kept as its stderr's last line;
  `attribute_containers` gives a port whose container's Compose folder (`com.docker.compose.
  project.working_dir`) a project holds to that project (`deepest`), the rest to
  `Ports.containers`. `ProjectListener.container` (`ContainerRef { engine, name, target, refusal
  }`) rides to the rail's `PortSnapshot.container`. `attribute` passes `HELPERS` by and clears an
  engine's own unit (`ENGINE_UNITS`), as `stop` does, so no port's Stop reaches `docker.service`.
  `stop_container` runs `<engine> stop <name>` (`ContainerStop`), or refuses a port the engine did
  not name with a command that finds it by port. `Ports::containers` is listed by the Containers
  panel (#673; the rail's CONTAINERS section of #614 and its fold of #670 are gone, and a saved
  blob's `marley_containers_open` is left unread): `rail_containers.rs`, the rail's child module
  `rail::containers`, so it draws its rows with the rail's `row_card`, `port_snapshot`,
  `container_line`, `url_label` and `stop_words`. `ContainersPanel` is a right-dock `Panel` added to
  every workspace (`ToggleContainers`, `IconName::Box`, priority 22, closed at start); its `icon`
  and `enabled` follow `shown` (the Marley layout and `MarleySettings::rail_containers`, #669), and
  a render while hidden closes its dock once, deferred, as the Knowledge panel does. Its rows open
  in a Browser tab of the panel's workspace on a double-click or Open, and Copy URL and Stop (the
  rail's `stop_container` and `container_refused_toast`) are on hover and in the right-click menu.
  `set_active` holds a `ports::watch` while the panel is open, released on close or release, so
  the scan also runs with the rail closed; a project's containers still show under it in the
  rail.

- **Restart, state and logs (#615).** Each scan asks `unit_states` for every service row's unit
  (one `systemctl show -p ActiveState,SubState` per manager, blocks matched by the order asked,
  since `Id` answers the real name) into `Ports.units`; `unit_word` turns a state into the row's
  word. `begin_restart` records a `Restarting { key, port, service, folder, since }` before
  `restart_service` runs `systemctl [--user] restart`; `keep_restarting` gives such a port a row
  of pid 0 while it is quiet, until it listens, the unit is inactive, or `RESTART_KEPT` (ten
  minutes) passes. `stop_service` stops a kept row's unit; both refuse an engine's own unit. In
  the rail, `PortMenu` builds the row's `right_click_menu` (Open in a Browser Tab, Copy URL, then
  Restart Service, Stop Service and Show Logs, Stop Container, or Stop Process); its trigger
  builds the row's tooltip only while the menu is closed. `show_logs` runs `journalctl [--user]
  -u <unit> -f` in a new terminal of the project's first folder through `start_in_terminal`. The
  unit line is a `RowLine` with the word as its state, red for `failed`; `port_row_buttons` holds
  the hover buttons, whose Stop stops a kept row's unit.

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
- **Archive (#605).** The row's end holds an Archive `IconButton` shown on hover, with the status
  mark laid over it and hidden on hover, as a terminal row's bell gives way to its close button;
  the card sits in a `right_click_menu` with Archive Thread. Both run `archive_thread`, which calls
  `ThreadMetadataStore::archive(thread_id, None, cx)` as Zed's history view does; the button stops
  the click, so the row does not open the thread. The store's notify refreshes the rail, which
  lists no archived thread, a tabbed one included (`group_threads`' tabbed loop filters on
  `archived`, #777). A `ThreadTab` subscribes to the store's `ThreadArchived` and closes on its
  own thread's, as the Agent Panel lets its retained thread go, and `deserialize` refuses an
  archived thread as it refuses a deleted one. Every Marley opener unarchives first, as
  `load_agent_thread` does: `threads_page::open_thread` (the Threads page, Reopen Thread…, Talk
  to), `thread_tab::open_saved` and the rail's `show_thread`, before it brings an open tab
  forward; `assistant::latest_conversation` skips archived records (#777).
- **Delete and archived threads (#616).** The thread row's menu has Delete Thread…:
  `delete_thread` asks with `window.prompt`, then, in the window's update, closes the thread's
  tabs in every workspace (`thread_tab::close_for`, #777: an open tab would save it again), every
  Agent Panel's
  `remove_thread_without_activating_draft` (a live conversation would save the thread again, and
  a draft in its place would be a thread the user did not start), the store's `delete`, then
  `thread_worktree_archive::cleanup_thread_archived_worktrees` and the agent's
  `AgentSessionList::delete_session` through the panel's `AgentConnectionStore`, where the list
  supports it. A project's menu (`HeaderMenu` now carries `archived`, from `archived_threads`:
  `archived_entries` matched by main folders and host, newest first, twenty at most) has Archived
  Threads, one level deep, since Zed's menu closes a nested submenu's parents only when that
  submenu was clicked; choosing one runs `open_thread_with`, which `open_thread` shares, and
  `load_agent_thread` unarchives it. `agent` is a normal dependency for `ThreadStore`.
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

- **The terminal an agent starts in (#537).** Every agent launch opens its terminal through
  `start_in_terminal` with `agent_env()`, `marley_agent::GIT_PROMPTS_OFF`
  (`GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`), handed to Zed's
  `Project::create_terminal_shell_with_env` at spawn and never typed; a launch config's Terminal
  item, New Terminal and the Playwright script's terminal pass none.
- **ssh's passphrases (#596).** `start_in_terminal` takes the agent (`Option<AgentKind>`) and
  works the variables out itself. For an agent in a local project it first makes Zed's
  `askpass::PasswordProxy` (`passphrase_proxy`: a 0700 temporary folder with a socket and
  `askpass.sh`, which runs Marley's executable as `marley --askpass=<socket>`), adds
  `SSH_ASKPASS` and `SSH_ASKPASS_REQUIRE=force` to `GIT_PROMPTS_OFF` (which carries
  `GIT_ASKPASS=` so git never borrows the helper), opens the terminal, and moves the proxy into
  the terminal's `on_release`, so the socket lives exactly as long as the terminal. Each prompt
  runs `ask_passphrase` on the foreground: the terminal is looked up through `mcp::terminals`
  before any window update (a window being updated reads as gone through its handle), revealed,
  and Zed's `AskPassModal` opens in its workspace headed `marley_agent::ssh_dialog_title`. A
  dismissed dialog drops its sender; the proxy answers `Continue(Err)`, writes nothing and closes
  the connection, and ssh reads an empty passphrase and fails. A second prompt while a password
  dialog is open in that workspace is refused rather than toggled. A socket path over
  `sun_path` leaves the terminal without the helper, logged. `agents::init` fixes the askpass
  program to `current_exe` once per process, before an install can replace the file.
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
- **Permission modes** (#532). `MarleySettings.agent_permissions` (`agents::AgentPermissions`)
  holds the two defaults and `agent_permissions_by_project`'s entries, their folders through
  `system_one::folder_path`; `launch_mode(kind, folders)` takes the longest entry whose folder
  holds one of the project's main folders and sets that agent, else the default. `start_cli`
  asks it for a local project's main folders (`project_group_key`), none for a remote one, and
  passes the mode to `launch_input`. `terminal_snapshot` sets `TerminalAgent.mark` from
  `marley_agent::permission_mark` with the seat's `PERMISSION_MODE_LABEL` and the terminal's
  `marley_foreground_argv`; `render_terminal_row` draws `permission_chip`, a pill as the inbox's
  chips are, in `Color::Warning` with the mark's tooltip, before #569's `stall_flag`.
- **Threads' modes** (#779). `agents::thread_modes` maps `claude-acp`, `codex-acp` and each Marley
  entry `assistant::entry_agents` names (with the CLI its adapter runs) to `bypassPermissions` or
  codex-acp's `agent-full-access` while `launch_mode(kind, &[])` asks for no prompts;
  `sync_thread_modes`, on the settings and the `Assistant` global, writes the map into the
  in-memory defaults as `marley.agent_default_modes` when it differs from the `ThreadModes`
  global. A hunk in `agent_servers` (`marley_default_mode`) puts it before the agent's own
  `default_mode` in `AcpConnectionDefaults`, which Zed applies with `session/set_mode` after
  `session/new`; Zed's `ModeSelector` saves every pick as that `default_mode`, which is why
  Marley's comes first. The entries' `session_meta` sends `allowDangerouslySkipPermissions: true`,
  so the adapter keeps Bypass Permissions among their modes.
- **Remote Control** (#757). `AgentPermissions` also holds `claude_code_remote_control`, and
  `launch_mode` sets the mode's `remote_control` for Claude Code only. `terminal_snapshot` sets
  `TerminalAgent.remote` through `runs_remote` (`marley_agent::remote_control` over the
  foreground's arguments); `terminal_marks` draws `remote_chip`, a muted pill, after the
  permission chip.

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

### Where the agent starts (#735)

- **`NewAgent { folder }`** is a data action now (read through `NewAgentFields`), re-exported as
  `crate::NewAgent`; a bare binding builds from `{}`, so `ctrl-alt-n` is unchanged. With a folder
  the picker asks only which agent.
- **Two steps in one picker.** The delegate keeps `chosen: Option<(Start, name)>`; confirming an
  agent with no `fixed` folder sets it, then a deferred `set_query("")`, `refresh_placeholder` and
  `refresh` list the `places` instead of the choices. `places_for` builds them while the workspace
  is being updated, so it reads that workspace through its reference and only the others through
  their entities: the guess (`guess`: the active terminal's `working_directory()`, else the active
  item's worktree root, else `thread_tab::default_folder`), the window's open projects, Browse….
  `load_recent` adds up to eight local recent projects from `WorkspaceDb` before Browse….
- **Starting.** `Start::run_in(folder)`: a thread through `thread_tab::start` (#734), a CLI
  through `launch_input` and `start_in_terminal` with the folder, as a launch config starts one.
  Browse… runs from `window.defer`: Zed's path prompt is a modal that replaces the picker, and the
  modal layer reads the picker, which the confirm is updating.
- **Open Agent Here.** The rail's terminal row passes the terminal's `working_directory()`; the
  project panel's entry (a Zed touch, `zed-touchpoints.md`) builds `marley::NewAgent` by name with
  the entry's folder. `Rail::new_agent_picker` shows the row's workspace first; Home's NEW AGENT
  card opens the picker on Home.

## Worktree agents (`src/worktree_agents.rs`, #510)

- The rail's `+` has New Agent in Worktree (`worktree_agent_entries`), a submenu of the installed
  CLIs right after New Agent Thread and before the Agent CLIs header (#598), when `worktree_agents::offered` finds the project local and its first folder a git
  repository's work directory. A choice runs `open_prompt`, whose plan is the repository, the
  main checkout's branch (its commit when detached; from a linked member, the `is_main` entry of
  its `linked_worktrees`) and a name from `worktree_names::generate_worktree_name` past the
  worktrees' folders and the `agent/` branches, and opens the `WorktreePrompt` modal: an
  auto-height editor in `MarleyWorktreePrompt` (Enter is `marley::StartWorktreeAgent`,
  Shift-Enter a new line, Escape `menu::Cancel`) over `agent/<name> from <base>`.
- The create, spawned on the window once the modal closes: one at a time per project (the
  `WorktreeAgents` global's `creating`, a toast for a second); a folder check with `fs.metadata`
  at `path_for_new_linked_worktree`, three names at most, since Zed's rollback of a refused create
  removes the target folder with force; the seed's mark; Zed's
  `create_worktree_workspace_on_branch` with `ExistingBranch { name: base }` and
  `agent/<name>`, whose failures Zed's toast reports; the mark cleared either way; `git config
  branch.<branch>.base <base>` through `util::command` in the new worktree (a failure is a toast,
  and the agent still starts); `agents::start_cli_with_prompt` in the new workspace, which reads
  the project's permission mode and types `marley_agent::launch_line`.
- `routing::seed_first_terminal` asks `take_seed_skip`, which matches the mark by the root's
  last two parts, since `new_local` canonicalizes a root.
- The rail's rows: `group_worktrees` reads each member's repository (`member_git`: the
  repository whose work directory is the member's first folder), and `worktree_rows` makes the
  union of the members' `linked_worktrees` and each member that is a linked worktree, minus the
  main checkout and `.claude/worktrees/`, named by `project::linked_worktree_short_name`, with
  the branch or a short commit. A linked member's terminals are tagged with its folder;
  `note_focus` names the displayed workspace's worktree. The rows need no git call: the snapshots
  hold them. `follow_folders` also follows each project's `GitStore` (`changes_the_worktrees`:
  a repository added or removed, and its worktrees, `HEAD` or branches). `render_worktree_row`
  draws the branch icon, the name (muted while not open) and the branch; a worktree's terminals
  sit deeper. `open_worktree` shows an open worktree's workspace, else runs Zed's
  `handle_switch_worktree` on the project's workspace.

- **Several agents from one prompt** (#770). `open_prompt` takes the installed CLIs other than
  the one chosen (the rail's submenu passes them), and `WorktreePrompt` draws a `Checkbox` for each
  under "Also start:", all clear. `kinds()` is the chosen agent, then the ticked ones in the
  submenu's order; `Plan::branches` names `agent/<name>` for one and `agent/<name>-<program>` for
  each of several (`suffixes`). `start` holds `creating` for the whole set, finds the set's names
  with `free_folders` (the first of three base names whose folders are all empty), and awaits
  `create` for each agent in turn, so a failed create's toast does not stop the next. `free_name`
  counts a set's base name as taken, stripping an agent program's suffix.

## A worktree's drift (`src/worktree_git.rs`, `src/rail.rs`, #560)

- `worktree_git` is the git adapter of the worktree rows, which #511 extends: every call goes
  through `git(main, args)`, `util::command::new_command` of `MARLEY_GIT` or `git` with Zed's four
  flags (`-c core.fsmonitor=false -c log.showSignature=false --no-optional-locks --no-pager`), the
  main checkout as its folder and `GIT_TERMINAL_PROMPT=0`, whose output the caller reads, since
  Zed's `run_raw` takes `merge-tree`'s exit 1, its answer, for an error. `recorded_base` reads
  `branch.<b>.base` (`config --get`, exit 1 none). `summary(main, branch_tip, base_tip, base)` runs
  `merge-base`, `rev-list --count <branch>..<base>` and `merge-tree --write-tree --name-only -z
  --no-messages --merge-base`: 0 is clean, 1 gives the files after the tree id, NUL-separated,
  and 129 with git's refusal words (before 2.38) is `conflicts: None`.
- The rows' facts: `member_git` keeps the member's branch as git names it, its `HEAD` and its
  repository; `worktree_rows` keeps each worktree's branch and `HEAD` (`WorktreeFacts`), and
  `group_worktrees` puts them on the `WorktreeEntry` with the main checkout and the group's
  repository, the main checkout's when it is open.
- The cache: the Rail's `drift` (per worktree's folder: the base the last run read, the tips it
  read for, the drift), `drift_runs` (one per main checkout) and `drift_unsupported`. `refresh`
  calls `note_drift`, which puts each kept drift on its `WorktreeSnapshot`, before its compare,
  and `follow_drift` last: the worktrees no longer listed are dropped, and while the rail shows
  (`watching_ports`), a repository `Repository::is_trusted` finds trusted with a worktree unread
  or whose tips moved gets a run unless one is in flight. The tips are the worktree's `HEAD`
  and its base's tip from the repository's `branch_list` (a local branch), or the base itself
  when #510 recorded a detached main checkout's commit (`drift_tips`, compared by `same_tips`
  without allocating). `drift_run` waits `DRIFT_DEBOUNCE` (a second), reads the rows, the kept
  states, the branches and `default_branch(false)` on the main thread, checks the trust again,
  and runs `read_drifts` on the background executor: each worktree's base, its record else the
  default branch, and a summary unless the base and the tips are the ones kept. The results are
  kept, one warning per repository is logged when `--write-tree` is refused, and the rail
  refreshes; a tip that moved during the run is found by that refresh. Zed flips a repository's
  trust without an event, so the first run after a trust grant waits for the next rebuild.
- `drift_chip` joins the worktree row's card: a bordered pill, `N behind` muted or `N conflicts`
  in the warning color after `IconName::GitMergeConflict`, with `DriftSnapshot::tooltip`, and
  `marley-rail-drift-<name>` for scenarios.

## Review and merge from a worktree's row (`src/worktree_git.rs`, `src/rail.rs`, #511)

- Since #511 `summary` counts both ways in one `rev-list --left-right --count
  <branch_tip>...<base_tip>`, and `Drift` carries `ahead`. `merge_owner(main, fs)` gives
  `MergeOwner::Workflow` or `Marley`: `git config --get marley.merge` when it is one of the two
  (another value is logged), else a `workflow.toml` at the main checkout's root, read through
  Zed's `Fs` and parsed with `toml`, holding a `[project]` table (what `rw init` writes).
  `ready_to_merge(main, worktree, branch, fs)` refuses a workflow's repository, a branch with no
  recorded base or a base that is a commit, then runs `merge_checks` in the spec's D3 order
  (`symbolic-ref --quiet --short HEAD` on the base; `status --porcelain --untracked-files=no`
  empty in the main checkout, then in the worktree; `rev-list --count
  refs/heads/<base>..refs/heads/<branch>` above zero) and gives the base and the count.
  `merge(main, branch, base)` runs `merge --no-ff --no-edit -m "Merge branch '<branch>' into
  <base>" refs/heads/<branch>` (the full ref, so a tag of the same name is never merged; the title,
  since git's own would quote the ref) and gives `rev-parse --short HEAD`; a failure lists `diff
  --name-only --diff-filter=U`, runs `merge --abort` whenever `MERGE_HEAD` exists, and says the
  conflict's files or git's words. `merge_checks` reads `status --porcelain` untrimmed, since a
  line's status letters may be spaces. `is_commit` lives here.
- `DriftState` keeps `recorded` (the base is #510's record) and `owner`; `read_drifts` reads the
  owner once per run and sets both on kept states too. A marker changed without a commit is seen
  at the next run, a tip's move; Merge reads it again.
- `render_worktree_row` joins the branch and `DriftSnapshot::ahead_words` on the second line, and
  wraps the row in a `right_click_menu` built when it opens from the kept state through a weak
  `Rail`, running no git: Review, then `merge_line`'s `MergeLine::Merge` entry ("Merge N commits
  into <base>…") or a `ContextMenu::label` (nothing to merge, no base recorded, the workflow
  merges here, a commit base, no branch, Zed's trust not given yet, the state not read yet).
- `review_worktree` reads the recorded base in the background, else
  `Repository::default_branch(true)` as `git: diff branch` does; `start_review` opens the worktree
  as its row's click does when it is not open, its folder first marked with
  `worktree_agents::skip_seed` so routing's first terminal does not land over the diff, and keeps a
  `PendingReview` for `REVIEW_WAIT`; `take_review`, last in `refresh`, finds the worktree's
  workspace and its own repository (`member_git`), drops a mark the seed did not take
  (`drop_seed_skip`) and, deferred, shows the workspace and calls Zed's
  `BranchDiff::deploy_branch_diff_with_base_ref` (made `pub`, the one Zed touch).
- `merge_worktree`: `merge_target` (the branch, the main checkout, Zed's trust, and `unsaved_in`,
  the window's dirty buffers inside the main checkout and outside its listed worktrees),
  `ready_to_merge` in the background, `window.prompt` ("Merge N commits into <base>?", the branch
  and the main checkout in its detail; Merge, Cancel), both again with the base and the count
  compared to the prompt's, then `merge`; a toast in the displayed workspace (`WorktreeMerge`), or
  `detach_and_prompt_err` "Could not merge <name>".
- Remove (#589): `RemoveLine` from `remove_line` (Remove, or in a `MergeOwner::Workflow`
  repository with commits ahead a note), after a separator. `remove_worktree`: `remove_target` on
  the UI thread (the entry and `agent_ui::thread_worktree_archive::build_root_plan` over the
  window's workspaces, which needs a project holding the worktree and Zed's creation record);
  `worktree_git::uncommitted` (`status --porcelain --untracked-files=normal`); a warning prompt,
  Remove or Remove Anyway; `MultiWorkspace::remove([member], KeepProject)` through the
  `MultiWorkspace` handle, `false` ending it; `thread_worktree_archive::remove_root`, which
  verifies the record, releases the folder, runs `git worktree remove --force` and rolls back on
  failure; then `worktree_git::end_branch(main, branch, recorded base)`: `branch -d`, else the oid,
  `merged_into` each of the base, `origin/HEAD` and `HEAD` (`merge-base --is-ancestor`,
  `merge-tree --write-tree` equal to the target's tree, `cherry` all `-`) and
  `update-ref -d refs/heads/<b> <oid>`, then `config --remove-section branch.<b>`; `BranchEnd`
  names the outcome for the toast.
- The teardown (#591): after the question and while the worktree's workspace is open,
  `worktree_agents::tear_down(member, name, cx)`: `teardown_tasks` resolves each folder's
  `TaskHook::RemoveWorktree` templates (`Inventory::templates_with_hooks`, the folder's and the
  global ones) with `WorktreeRoot` and `MainGitWorktree`, as Zed's `run_create_worktree_tasks`
  does; each goes to `Workspace::spawn_in_terminal` and is raced against `TEARDOWN_FOR` (two
  minutes); a non-zero exit, an error or the deadline asks Remove Anyway or Cancel, and Cancel ends
  Remove with the worktree and its terminals as they were.

## A worktree agent's environment (`src/worktree_include.rs`, `src/worktree_agents.rs`, #585)

- `worktree_include::copy_included(main, worktree, fs)` reads `.worktreeinclude` through `Fs`,
  builds an `ignore::gitignore::Gitignore` rooted at the main checkout, and takes each entry
  `worktree_git::ignored_entries` lists (`ls-files --others --ignored --exclude-standard
  --directory -z`, split on NULs, never trimmed): a file a line matches
  (`matched_path_or_any_parents`); a wholly ignored directory a line matches, or one a line
  reaches by Claude Code's rule (`Reach::Named` for `**/<name>` and slash-less lines, by the
  directory's names; `Reach::Under` for a line with a slash, by its leading literal names), walked
  with each file checked against the matcher, so a `!` line keeps a file out; a folder holding a
  `.git` and a link to a folder are not walked. Sizes are summed before any write; an entry past
  the budget (100 MB, 10,000 files) is skipped whole and named with its size. Files go through
  `Fs::create_dir` and `Fs::copy_file` (`overwrite: false, ignore_if_exists: true`).
- `worktree_agents::create` copies after Zed's create resolves and before `write_base`, a toast
  under `NotificationId::unique::<Included>()` naming what was left out. Zed's own
  `create_worktree` tasks start inside the create and are never awaited, so they run alongside the
  copy.
- The setup offer: `LOCKFILES` (Orca's table), `setup_offer(main, base, fs)` (a `package.json`,
  one manager's lockfiles, no `create_worktree` task in the `.zed/tasks.json` committed at the
  base, read by `worktree_git::committed_file` and parsed with
  `settings::parse_json_with_comments::<TaskTemplates>`; checked when `marley.worktreeSetup` keeps
  the command), read in the background when the prompt opens unless the user's global tasks have
  such a hook (`global_create_worktree_tasks`, the inventory's `TaskSourceKind::AbsPath` hooks).
  The prompt draws a `Checkbox`; `Launch { prompt, setup }` carries both to `start`; `create` keeps
  the choice (`remember_setup`, the command or `none`, only when an offer showed), then
  `start_cli_with_prompt(.., Some(command), ..)` types `marley_agent::launch_line_after`, and
  `agent_trust::watch` waits `WATCH_AFTER_SETUP` (15 minutes) instead of `WATCH_FOR`.
- `routing::with_marley_paths`, in `RoutedTerminals::spawn` in both layouts, adds
  `MARLEY_ROOT_PATH` (from `ZED_MAIN_GIT_WORKTREE`) and `MARLEY_WORKTREE_PATH` (from
  `ZED_WORKTREE_ROOT`) to a task's environment, a task's own values kept.
- The port slot (#590): `worktree_agents::record`, after the copy, writes the base and then
  `worktree_git::assign_slot(worktree, branch)`: `git worktree list --porcelain` parsed with Zed's
  `git::repository::parse_worktrees_from_str`, the other listed worktrees' branches, their
  `branch.<b>.marleyslot` keys from `git config --get-regexp`, the lowest slot from 1 none holds,
  written with `git config`; a failure is a toast. `worktree_git::slot_of(folder, fs)` answers
  `None` unless the folder's `.git` is a file, then reads the branch (`git symbolic-ref`) and its
  slot. `worktree_agents::give_slot_reader(fs)`, called for each new workspace from `init`'s
  `observe_new` with the workspace's `app_state().fs`, hands `slot_of` to
  `marley_terminal::ports::set_slot_reader`, so the reader has the app's `Fs` before the
  workspace's first terminal.

## Claude Code's trust question in a new worktree (`src/agent_trust.rs`, #587)

- `agents::start_cli_with_prompt` returns `Task<Option<WeakEntity<Terminal>>>`, the terminal
  once the launch line is written, its errors still prompted (`prompt_err`);
  `worktree_agents::create` awaits it and, for Claude Code, calls `agent_trust::watch` with the
  terminal, the worktree's workspace, the workspace that started it and the worktree's name.
- The `TrustWatches` global holds one `TrustWatch` entity per terminal, by the terminal's id. It
  subscribes to the terminal's `Wakeup` and reads the screen half a second after output, one read
  at a time. A read finds the question only while `marley_agent::trust::footer_on_screen` finds
  its footer among the visible rows (`with_renderable_cells`, one row per `point.line`), since
  `last_n_non_empty_lines` reaches into the scrollback, where an answered question stays; it then
  parses `last_n_non_empty_lines(24)`, whose wrapped rows are joined. The watch ends after a
  minute while the question has not shown, when the question leaves the screen, or when the
  terminal is released (`observe_release`); `finish` dismisses its notifications and removes it,
  deferred.
- When the question shows: under `claude_code_worktree_trust: follow_zed`, with
  `TrustedWorktrees::can_trust` true for the worktree's first folder, it answers and shows a toast
  in the starting workspace under an id of its own (no autohide); otherwise an app notification
  (`show_app_notification`, `MessageNotification` with a built body: the headline and the
  warnings in the warning color), Trust Folder when `answer_keys` gives keys, and Show Terminal.
- An answer re-reads the screen, writes `answer_keys` once with `Terminal::input`, and after a
  second and a half either finds the question gone or shows the "still asking" notification,
  with an id of its own (a `MessageNotification` button dismisses its own id after its handler,
  deferred), with Show Terminal alone. Show Terminal activates the worktree's workspace in the
  window's `MultiWorkspace` and the terminal's `TerminalView` among its items.

## The block keys and the block menu (`src/blocks.rs`, #473, #554)

- `marley::PreviousBlock` and `marley::NextBlock` are caught at each workspace's root with
  `register_action_renderer`, as `routing` catches its actions, and act on the terminal view
  that holds focus: the active item of a center pane or of a Terminal Panel pane. With none, they
  do nothing.
- The handler calls `Terminal::sync` first, so a key pressed before the next frame sees where
  the one before it went, then scrolls to the bottom and up by `marley_terminal::block_scroll`'s
  offset.
- The Marley keymap binds them to `secondary-up` and `secondary-down` in `Terminal`, keys Zed's
  defaults leave unbound there (AD-claude-449's rule for the Marley keymap).
- Since #554 they select. `terminal_view::MarleyBlockSelection` holds each terminal's selected
  block (by the terminal's entity id) with `AnchoredBlocks::inputs` at that moment; `selected`
  answers only while the count is unchanged, so the first input ends it. `step`: with none
  selected, `PreviousBlock` selects the newest block and `NextBlock` scrolls to the next as
  before; with one, back or forward a block, forward past the last clearing it; `reveal`
  scrolls the block's first line into view when it is off screen. On the alternate screen the
  key propagates. `Terminal && MarleyBlockSelected` binds `up`, `down`, `escape`
  (`ClearBlockSelection`) and `ctrl-shift-i` (`ReinputBlock`). The view's key context carries
  `MarleyBlockSelected` and the element outlines the block while it is selected.
- The block menu: `terminal_view::MarleyTerminalBlockMenu` is set to `block_menu`, which Zed's
  `deploy_context_menu` asks after its own items with the block under the click
  (`terminal_element::marley_block_at`). It selects the block and adds a `Block` header, the four
  copies (`copy`: the command, `Terminal::block_output`, both, or `AnchoredBlock::markdown` with
  the block's `BlockTimes`) and Reinput and Reinput with sudo (`reinput`: Ctrl-U, the command,
  no return), disabled unless `AnchoredBlocks::rerun_offered`.

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
  - Since #586 `open_url` is async: a text `address::local_page` reads (a `file:` URL, or a path
    joined to the program's folder, named `.html` or `.htm`) is checked with Zed's
    `Fs::metadata` off the main thread (a file, neither a folder nor a FIFO), turned into a URL
    with `Url::from_file_path` and, unless `links::pages_in_browser_tab` finds
    `terminal_links: system_browser`, opened by the same tail, `open_in_tab`; "not a local HTML
    page" otherwise. Any other text takes the path above. An agent calls the tool with Marley's
    bearer as the opener does, so this is the whole of what it opens beyond http and https;
    `browser_navigate` keeps `agent_url` (plan D15).

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
  → reaches the program. While the shell's prompt editor holds the line
  (`rich_input::holds_line_of`) it propagates at once, so the grid's text behind the editor is
  never completed (#637).
- `suggestion_for(line, terminal, cx)` is the lookup for any line; the grid's `suggestion` passes
  the typed text, and the prompt editor its own text (#637).

## Rich input (`src/rich_input.rs`, #481)

- The agent bar's pencil (`agent_bar::rich_input_button`) toggles since #677:
  `rich_input::is_open_for_agent` (the view's prompt open, its target an agent) sets its
  `toggle_state` and its tooltip's title, Hide Rich Input, and a click reads it again and calls
  `rich_input::hide` (Escape's `close`: the draft kept, the terminal focused) or `open`.
- `Prompts`, a global, holds an editor per terminal view (auto height, one to eight lines,
  soft wrap), made the first time it opens and dropped with the view, and whether it shows.
- `marley::RichInput` is bound to `ctrl-g` in `Terminal` and handled on every workspace: with a
  CLI agent in the focused terminal (`agent_bar::agent_in`) it opens that terminal's editor and
  focuses it; since #624, with none and the shell at a prompt off the alternate screen
  (`target_of`), it opens it for the shell (`Target::Shell`), its text the line typed at the
  prompt (`autosuggest::typed_text`); otherwise it calls `cx.propagate()`, and the key goes on to
  the terminal, which sends it to the program. The footer of a terminal with no agent
  (`agent_bar::footer_without_agent`) draws the editor too since #624.
- In the editor's `MarleyRichInput` container, Enter (`marley::SendRichInput`) pastes the text
  with `Terminal::paste`, bracketed when the program asked for it, sends `\r` after
  `terminal_drive::AFTER_PASTE` (`paste_then`, #594), for the shell after a Ctrl-U that clears
  its line (#624), clears and closes
  the editor and focuses the terminal; Escape (`marley::CloseRichInput`) closes it with the draft
  kept; Shift-Enter is `editor::Newline`. In the shell's editor the container's key context adds
  `MarleyShellInput`, where Ctrl-C is `marley::ClearRichInput` (#627), which empties it. The container stops the key events the terminal view
  would send its program, chords and keys that type nothing, and lets text through to the
  editor.

## Agent prompts in a tab (`src/agent_editor.rs`, #649)

- `marley.agent_editor_in_tab` (`MarleySettings::agent_prompts`, an `AgentPrompts`: `Overlay`, the
  default, or `InTab`) decides where an agent's prompt opens on Ctrl-G in the terminals Marley
  opens for agents.
- `mcp::offer_agent_editor` writes `bin/marley-edit` (`agent_editor::HELPER`, embedded) beside the
  opener at start, and `set_helper` publishes its path unless it holds whitespace: Claude Code,
  Gemini CLI and OpenCode split `$VISUAL` on spaces. Since #790 a data directory with whitespace,
  as macOS's `Application Support` always has, gets a link in its place:
  `spaceless_in(paths::state_dir(), helper)` makes
  `<state dir>/agent-editor/<hash of the helper's path>/marley-edit` again at each start, off the
  main thread. `marley-edit` finds its endpoint beside `realpath(__file__)`, so the link reaches
  the Marley that wrote it. Only a link that cannot be made, or a state folder with whitespace too,
  logs the warning and keeps the overlay.
- `agents::start_in_terminal`, for an agent in a local project while `editor_path` gives the
  helper, adds `VISUAL`, `EDITOR` and `MARLEY_AGENT_EDITOR` (`add_env`) and records the new
  terminal (`give`, `forget` on its release). The shell integration exports the helper again after
  the user's files.
- `rich_input`'s `RichInput` action and the agent bar's button, for a terminal `takes_key` accepts
  (the switch on, the terminal given the helper), write the agent's own key
  (`AgentKind::editor_key`) into the terminal with `press_key` and open no overlay. Any other
  terminal keeps #481's overlay.
- The helper, run by the agent with its prompt's file as the last argument, reads the endpoint as
  the opener does, sends `Marley-Terminal` from `MARLEY_TERMINAL_ID`, calls `prompt_open`, then
  `prompt_wait` in 20 s rounds until `closed` (`editor_open` and `editor_wait` until #704), and
  exits 0; any failure prints one `marley-edit:`
  line (the refusal's `reason` when Marley gave one) and exits 1, the file untouched.
- `answer` serves both tools. `open` finds the calling terminal (`mcp::caller_terminal`), opens the
  file with `Workspace::open_abs_path` (not a preview, with the focus) through the window's handle,
  keeps the item's `on_release` subscription in the `Edit` and drops the item handle, as Zed's
  `--wait` does. The release defers `ended`, which marks the edit closed, wakes the live waiters,
  keeps an edit nobody waits on for a minute, and activates the terminal view in its workspace.
  `wait` answers at once when closed, or races a oneshot against the executor's timer.

## The prompt editor by default (`src/rich_input.rs`, #627)

- `marley.prompt_editor` (`MarleySettings::prompt_editor`, a `PromptEditor`: `AtEveryPrompt`, the
  default, or `OnCtrlG`) decides whether the shell's editor follows the prompt.
- `rich_input::init` observes each new `TerminalView`: its terminal (`follow_prompt`) and its
  focus (`take_focus_at_prompt`). `Shells`, a global, keeps per view whether the shell waited at a
  prompt at the last notify, the terminal's block count then, and whether Escape dismissed the
  editor; the entry goes with the view.
- `follow_prompt` counts a prompt as arrived when the shell waits now and did not before, or when
  the block count changed: a quick command starts and ends between two notifies. An arrival clears
  the dismissal and, when the view or its editor holds the focus, opens the editor for the shell.
  Leaving the prompt (a command, the alternate screen) closes the editor and gives the focus back
  to the terminal if the editor held it. Both hooks run inside the view's update, so each open,
  close and focus change is deferred to the window.
- `take_focus_at_prompt` opens the editor when the terminal gets the focus at a prompt it was not
  dismissed at. `close` marks the shell's editor dismissed until the next prompt.
- Since #573 the shell's editor subscribes to its own `BufferEdited`: the text goes to
  `typed_line::changed` and `paint_hint` draws `english::hint_for`'s words after the text as an
  `Inlay::edit_prediction` with the reserved id `usize::MAX - 573`, and the warning range in the
  theme's warning colour through `highlight_text` under `HighlightKey::Editor`; an agent's editor
  shows neither. `send` calls `typed_line::entering` first. In `MarleyShellInput > Editor`,
  Ctrl-Shift-Enter is `marley::AskAgent`.
- Since #637 the inlay carries #484's suggestion first: `shell_hint` asks
  `history_suggestion` (`autosuggest::suggestion_for` over the editor's text, only while the editor
  has one cursor at the end of its text with nothing selected), and `hint_for` only without one,
  the grid's order. The subscription repaints on `SelectionsChanged` too, so the suggestion follows
  the cursor. In `MarleyShellInput > Editor`, `right` is `marley::AcceptSuggestion`; the footer's
  handler (`accept_suggestion`) recomputes the suggestion and appends it with `Editor::insert`, and
  without one calls `cx.propagate()`, so gpui dispatches the next binding for the key,
  `editor::MoveRight`.
- Since #783 Up and Down are `marley::RichInputUp` and `marley::RichInputDown` in
  `MarleyRichInput > Editor && !showing_completions && !showing_code_actions`. Zed's
  `Editor::move_up` and `move_down` call `cx.propagate()` when the cursor stays put, and the next
  binding for the key inside a terminal view is the Terminal context's `terminal::SendKeystroke`,
  so the footer's `arrow` never propagates: it calls the editor's own move, then
  `cx.stop_propagation()`. In the shell's editor, with one empty selection on the display row the
  text starts on (Up) or ends on (Down; the buffer end's row, since the hint inlay can wrap), it
  steps `Prompt::walk` instead: a `Walk` of `autosuggest::history` taken at the first Up, blank
  commands and repeats of a newer one left out, the index shown, and the draft Down gives back past
  the newest. `walk` stores the walk before `set_text`, so the `BufferEdited` it raises sees the
  recalled command; `edited` ends a walk whose command no longer matches the text. `open_for`
  starts each opening with no walk.
- Since #784 the Terminal context's other keys stay in the editor too. Two ways out were found by
  evaluating each binding of both keymaps against the editor's context stack as gpui does: a key
  the Editor context leaves unbound falls to the Terminal's `SendKeystroke`, `SendText` or
  `Paste` (Ctrl-W, Ctrl-E, Ctrl-B, Ctrl-R, Alt-B, Alt-F, Ctrl-Shift-V), and Zed's unscoped Ctrl-O
  and Ctrl-Q rank at the deepest context, above the Terminal's override. `MarleyRichInput >
  Editor` binds the readline moves to Zed's editor actions with the arguments Zed gives
  Ctrl-Backspace, End, Ctrl-Left and Ctrl-Right, and Ctrl-R, Ctrl-O and Ctrl-Q to `null`, which at
  the same depth and the same default source suppresses both. Ctrl-Shift-V is
  `marley::RichInputPaste`: `paste` puts text in with `Editor::paste`, and from an agent's editor
  propagates an image to `terminal::Paste`, which sends the agent Ctrl-V. Ctrl-Shift-C is
  `marley::RichInputCopy`: `copy` dispatches `editor::Copy` when the editor has a selection
  (`Editor::copy` is not public) and otherwise propagates to `terminal::Copy`, the grid's
  selection.
- `send` (#635) writes Ctrl-U to clear readline's line only when something may be on it (input
  typed since the prompt, or not known): on an empty line readline rings the bell at Ctrl-U, and
  the bell is Zed's dirty mark. It clears the view's bell, as Zed clears it on any input from the
  view. Marley's keymap binds Ctrl-Shift-W in `MarleyRichInput > Editor` to
  `pane::CloseActiveItem`: gpui ranks an unscoped binding at the deepest context, so Zed's
  unscoped `workspace::CloseWindow` would otherwise outrank the Terminal binding there.
- `holds_focus(view, window, cx)` (#634) is whether a terminal view has the keys: its own focus
  handle, or the open editor of its terminal. The view's handle alone does not count the footer:
  Zed's terminal element tracks the same handle on the grid, gpui takes the last element to track
  a handle as its place, and the footer sits beside the grid. The terminal toggle
  (`routing::toggle_terminal`), the block keys (`blocks::focused_terminal`),
  `notifications::looking_at` and the rail's focused terminal ask it.

## The shell's completions (`src/shell_completions.rs`, #625)

- `ShellCompletions { terminal, workspace }`, an `editor::CompletionProvider` that `rich_input`
  gives the shell's editor (an agent's gets none). From the buffer it takes the line before the
  cursor and its last word; history (`autosuggest::history`, the shell's verified commands then
  its history file, newest first) and the project's task commands
  (`TaskInventory::list_tasks`, command and arguments joined) that start with the line, twelve at
  most, replace the line; the word's folder part listed under the prompt's folder
  (`AnchoredBlocks::prompt_folder`, else the terminal's) gives the entries starting with the rest,
  folders with `/`, hidden ones only for a `.`, replacing the word. It sorts and filters itself
  and answers incomplete, so each key asks again; no character triggers it.
- `keymap.json`: in `MarleyRichInput > Editor && !showing_completions`, Tab is
  `editor::ShowCompletions` and Enter `SendRichInput`, so Zed's own Enter and Tab take an entry
  while the menu shows.

## A command's colours (`src/prompt_colors.rs`, #626)

- `init` loads "Shell Script" (Zed's bash grammar) from the first workspace's project into a
  `ShellLanguage` global and sets `terminal_view::MarleyPromptColoring`, which the element asks
  once a frame: the line typed at the prompt (`autosuggest::typed_text`, so none while scrolled
  back or on the alternate screen), highlighted with `Language::highlight_text`, each run's
  `HighlightId` given its colour by the theme's `syntax().get`, and its bytes mapped to columns
  from `AnchoredBlocks::input_start`. The element paints those cells in those colours
  (`marley_layout_grid`); the grid's cells are untouched.
- `rich_input` gives the shell's editor's buffer the same language (`shell_language`).

## A block's filter (`src/block_filter.rs`, #528)

- `init` sets `terminal_view::MarleyTerminalOverlay` (the open panel of the view, found by the
  view's id alone, since the view is rendering) and `MarleyBlockFilter` (the Filter button's
  action), keeps a `Filters` global of one `FilterPanel` entity per terminal view (dropped with
  the view), and handles `FilterBlock` at the workspace's root: the selected block
  (`MarleyBlockSelection`), else the newest in `marley_terminal::visible_spans`.
- `FilterPanel`: the query and context fields (single-line editors), the list (a read-only
  multi-line editor, no gutter), `Toggles` (case, regex, invert), the count and the error.
  `refilter` takes `Terminal::block_output` and runs `marley_terminal::filter::filter_lines` off
  the main thread (`futures::future::lazy`); `follow` filters a running block again at most every
  250 ms on its terminal's `Wakeup`; `fill_list` writes the rows (`--` for a gap) into the list's
  buffer and highlights the matches (`HighlightKey::BufferSearchHighlights`).
- The panel covers the grid (`absolute`, `inset_0`, `occlude`) in key context `MarleyBlockFilter`;
  its container stops every key that types no text, as the rich input's does; Escape
  (`MarleyBlockFilter > Editor`) and `FilterBlock` close it and give the terminal the focus.

## Bookmarks and find within a block (`src/bookmarks.rs`, #559)

- The marks are data in `terminal_view::MarleyBlockMarks`, keyed by the terminal's entity id:
  the bookmarked blocks, which the element ticks at its right edge, and the block the search is
  held to, which the element outlines and the view's `find_matches` keeps the matches of. `init`
  sets it, drops a terminal's marks when the `Terminal` is released, and sets the element's two
  per-block hooks, each composed from its parts: `MarleyBlockChip` is the bookmark, then
  `send_block::chip`; `MarleyBlockExtras` is Bookmark, Find, then `workflows::block_buttons`.
- `ToggleBookmark` (`ctrl-shift-b` in `Terminal`) and `FindInBlock` (`ctrl-shift-f` in
  `Terminal && MarleyBlockSelected`) act on `block_filter::block_to_filter`: the selected block,
  else the newest in view. `PreviousBookmark` and `NextBookmark` (`alt-up`, `alt-down`) run
  `marley_terminal::block_scroll` over the marked blocks, and propagate with no mark or on the
  alternate screen, so the keys reach the program.
- `find_in`, deferred past the workspace's update, sets the scope and dispatches
  `zed_actions::buffer_search::Deploy::find()` on the view's focus handle, or clears a scope set
  on that block already; either way it emits `SearchEvent::MatchesInvalidated`, so an open bar
  searches again. The view's `search_bar_visibility_changed` clears the scope when the bar closes.
- `blocks.rs` adds Bookmark (or Remove Bookmark) and Find in Block after Send to Agent.

## Runnable commands in the Markdown preview (`src/markdown_commands.rs`, #530)

- `init` sets `markdown_preview::MarleyCodeBlockAction`, which the preview passes to its
  `MarkdownElement` as a code-block action; the `markdown` crate puts the action's element first
  in each code block's hover row, before Copy, under an id of the block's own.
- `button` gives Insert in Terminal for a shell block with text: a bare fence, or a fence whose
  info string's first word is `sh`, `shell`, `bash`, `zsh` or `fish`.
- `insert` takes `browser::last_terminal_with_window`, the terminal the focus entered last. It
  types nothing, with a toast in the preview's workspace, when there is none, when its shell is
  not at its prompt (`AnchoredBlocks::at_prompt`; the toast names the running block's command or
  the foreground program), or for several lines while `Modes::BRACKETED_PASTE` is off. Otherwise,
  after the click's update, it activates the terminal's window when it is another, brings the tab
  forward (`reveal_terminal`), focuses it, and sends Ctrl-U and `Terminal::paste` of the text,
  with no carriage return.

## Block headers (`src/block_headers.rs`, #628)

- The terminal element takes each verified block's prompt rows on screen
  (`marley_terminal::prompt_rows`) and asks the `terminal_view::MarleyBlockHeader` hook, which
  `init` sets, with the block's index and the row height. When it gets an element it leaves those
  rows' cells out and lays the element over the same rows, first of the block elements, so the
  pill and the hover actions draw over it.
- `header` gives none while `MarleySettings::block_headers` is `ShellPrompt` (`Native` unless
  the setting is `false`, since #630); otherwise a column the rows' height. Over two rows or more
  its first row is the place, muted, and its second the command's first line in the buffer font
  (`…` when the command has more lines); over one row the command comes first and the place
  after it. The command keeps its width and the place truncates. The press and its release stop
  there, so the terminal under it starts no selection. Since #812 the press first focuses the
  header's view (`view.focus_handle(cx)`, the handle the terminal element focuses on a press), so
  a header in another pane takes the focus there.
- The place (#630) is `folder_label` of the block's `pwd` (`~` for home; past two folders the
  last two after `…`) and the branch `Branches` recorded for the block: `init` observes each new
  view's terminal, and `record_branches` stores, once per started block, the branch of its
  folder's repository in the view's project (`agent_bar::branch_of`, the agent bar's lookup made
  a function). A released view's entries go.
- `init` sets `terminal_view::MarleyBlockSpacing` from `MarleySettings::block_density` (#631),
  and again on each settings change that moves it: `Comfortable` is half a row between blocks
  and a second header row over a one-row prompt, `Compact` none. The element asks the hook for
  the header with that row more, so `header`'s two-row layout draws, and inserts the space
  through its display-row map (`marley_terminal::RowMap`).

## The sticky command header (`src/sticky_header.rs`, #529)

- The terminal element picks the block, `marley_terminal::sticky_block` over its spans: the top
  one while the view is scrolled back, when it covers row 0 and started above it. It asks the
  `terminal_view::MarleyStickyHeader` hook, which `init` sets, lays the answer over row 0 and
  paints it last among the block elements.
- `header` gives none while `MarleySettings::sticky_command_header` is off; otherwise a row on the
  terminal's background with a bottom border: `$ ` and the command's first line in the buffer
  font, truncated, the state (`running`, a check, `exit N`) and an up arrow. A left press jumps,
  as Zed's editor sticky headers do (`blocks::reveal`, the block's first line at the top), and
  the press and its release both stop there, so the terminal starts no selection and its
  plain-click listener (#579) opens no link menu; since #812 the press focuses the view first, as
  the block header's does. With `block_headers` `Native` the command has
  no `$ `, as the native header has none (#629).

## Workflows (`src/workflows.rs`, #558)

- A workflow is a Zed task: `label`, `command`, `cwd` (`$ZED_WORKTREE_ROOT` when the block ran
  there) and, under a `marley` key Zed ignores, `parameters`, a default and a description per
  `{{name}}`. There is no store of Marley's own.
- `block_buttons`, part of the `terminal_view::MarleyBlockExtras` hook `bookmarks` sets (#559):
  Save as Workflow (`IconName::Book`) for a finished block whose command is one line and was
  verified by the shell's nonce. `SaveAsWorkflow` takes
  the selected block, else the newest on screen (`block_filter::block_to_filter`). Both defer
  `open_editor`, since the action runs while the workspace is leased.
- `WorkflowEditor` (`MarleyWorkflowEditor`): the name, the command from
  `marley_terminal::workflow::guess` (the branch from the block's facts, a path when it exists
  under the block's folder), a default and a description row per parameter, rebuilt as the
  command is edited, and a checkbox for the global file. `write_workflow_in` reads the file or
  starts one, refuses a label it has, and adds the task with
  `settings_json::append_top_level_array_value_in_json_text`, so the file is edited as text and
  its comments stay; it runs off the main thread, then a toast names the file.
- `fill`, which `RoutedTerminals::spawn` awaits inside its `window.spawn`: a task whose command or
  arguments name a `{{…}}` opens `ParameterPrompt` (`MarleyWorkflowPrompt`), prefilled from the
  session's `LastValues`, else the files' `marley.parameters`; Enter (`RunWorkflow`) fills the
  command, the arguments and the label with `workflow::substitute`, and Escape drops the answer's
  sender, so the spawn resolves to none and nothing runs.

## Sending a block to an agent (`src/send_block.rs`, #555)

- `text_for`: the block's output through `mcp::agent_redactor` first; inline when that is at most
  32 lines and 4 KiB, or the output is gone (`AnchoredBlock::markdown`, the whole redacted again);
  else `[terminal <id> block <n>: <command, redacted>, exit <code>; terminal_read terminal=<id>
  block=<n>] `, the id the view's entity id `terminal_list` gives.
- `targets`: `send_selection::agent_targets` of the view's workspace, less the block's own view.
  `send` defers to `send_now` (the key's action runs while the workspace is being updated): a toast
  for none, `send_selection::send_text` for one, `TargetPicker` ("Send the block to…") for several.
- `chip`, part of the `terminal_view::MarleyBlockChip` hook `bookmarks` sets (#559): a tinted
  `Ask the agent` button for the newest block when it is Finished with a non-zero exit,
  `AnchoredBlocks::at_prompt`, no agent in front (`agent_bar::agent_in`) and a target exists; its click sends. The element places it on
  the block's first row before the pill, or on its last row when the first is above the screen.
- `blocks.rs` puts Send to Agent first in the Block section and handles `SendBlockToAgent`
  (`ctrl-shift-enter` in `Terminal && MarleyBlockSelected`) for the selected block.

## Marley as Claude Code's IDE (`src/claude_ide.rs`, #653)

- `marley.claude_code_ide` (`MarleySettings::claude_code_ide`, a `ClaudeCodeIde`, off by
  default). `init` keeps each local workspace with its project's entity id (`observe_new`, which
  defers its `reconcile`, since the workspace is still being built; `on_release`), and runs
  `reconcile` on each settings change and each change of #648's checks
  (`agent_versions::observe`). `reconcile` serves every local project while the switch is on and
  `agent_versions::is_on(&CLAUDE_IDE_CONNECTION)`, and stops every other server; it also refreshes
  the state the servers' threads judge clients by (the allowed rows and the installed version,
  `agent_versions::found`).
- `start` spawns the project's `IdeServer`, sets the port in `marley_terminal::ide`, and writes
  the lock file off the main thread into `ClaudePlugin::config_dir`'s `ide` folder
  (`write_lock_in`: the folder made 0700 only when missing, the stale Marley locks swept, the file
  written 0600 with `discovery::write_endpoint_file_in` under a temporary name and renamed). A
  project's `WorktreeAdded` or `WorktreeRemoved` rewrites it (`refolder`); `stop` and the quit
  hook remove it.
- A client's messages come to the main thread through one channel: its version (logged with its
  parts), its process (`ide_connected`'s pid, its chain read on the client's thread with
  `agent_reports::process_chain` and matched by `agent_reports::terminal_on_chain`), its leaving,
  and its tool calls (`answer`).
- The selection: the workspace's `ActiveItemChanged` makes a local file editor the project's
  last one (`follow_editor`, which subscribes to its `SelectionsChanged`); a change rests 100 ms
  (`settle_then_send`, one task per project) and goes as `selection_changed` to the clients whose
  version takes it, unless it is the one sent last (the workspace reports `ActiveItemChanged`
  far more often than its item changes). A client that names its process gets the current one.
  `editor_selection` reads the newest selection through the multi-buffer snapshot, its text and
  its places in UTF-16.
- The tools: `getWorkspaceFolders` from the lock file's folders, `getCurrentSelection` from the
  last file editor, `getLatestSelection` from the last sent, `getOpenEditors` from the
  workspace's file editors, and `getDiagnostics` for one file or for every file whose summary has
  errors or warnings, each opened (`Project::open_buffer`) and read for its groups' primary
  entries.
- `mention(view, path, lines)` is send selection's way in: the link whose client runs in the
  terminal `view` gets `at_mentioned` when its version takes mentions.

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
- `send` gives the reference to `send_text` (#555 split it out), which runs after the update it starts in: rich input open on the terminal
  (`rich_input::is_open`) gets the text through `rich_input::insert`, the terminal brought to the
  front first; a seat in `State::Waiting` gets nothing and a toast; otherwise the window
  activated, the terminal revealed (`browser::reveal_terminal`) and focused, one
  `Terminal::paste`, no Enter.
- Since #653 `send` first offers the selection to Claude Code's IDE link: for a Claude Code
  target that does not wait, without rich input open, `claude_ide::mention` sends `at_mentioned`
  (the lines from 0) to the link of its terminal when that link takes mentions, and the terminal
  is brought to the front and focused; nothing is typed. Every other case goes to `send_text`.
- Since #522 the picker is shared: `TargetPicker::new(rows, placeholder, on_pick, ..)`, each
  `Row` a label and a `Pick` (`Agent(Target)` or `Copy`), and `on_pick` what the confirm does.
  `Target` gains `ready` (its seat in `State::Idle`); `Target`, `agent_targets` and the picker are
  `pub(crate)`.

## An agent that drives a running program (`src/terminal_drive.rs`, #525)

- A `Drives` global keeps each terminal's control state by its view: `generation`, the foreground
  program it belongs to (the PTY's foreground process group leader, `Terminal::pid`, none when it
  is the shell's own pid), `approved`, `taken_over`, the last write and a `Pending` write with its
  oneshot. `drive()` brings a terminal's generation up to date on each read: a new program
  advances it and clears the approval.
- `screen` (for `terminal_screen`): the terminal (`mcp::terminal_of`), its `last_content` cells
  grouped into rows by line (a wide character's spacer skipped), through `mcp::for_agents`, with
  the cursor, the size, `ALT_SCREEN`, `scrolled` (the user scrolled back), the program's name, and
  the control state.
- `type_into` (for `terminal_type`): `check` refuses a missing `generation`, nothing to type, more
  than 4,096 bytes, a key name that parses to a plain character (it goes in `text`), no program in
  the foreground, an agent CLI there (`agent_bar::agent_in`), another generation, a take-over, or a
  write already waiting; `marley.agent_terminal_writes` decides whether to ask. Asking sets
  `Pending` (the caller's words from `click_pause::Who`), notifies the view, shows a toast with
  Show (`browser::reveal_terminal`), and races the answer against 25 seconds; Allow types only if
  the generation is the same and no one took over. `write` pastes the text (`Terminal::paste`),
  then sends each key (`Keystroke::parse`, `Terminal::try_keystroke`) and `\r` once
  `AFTER_PASTE` (200 ms) has passed, through `paste_then` (#594): a program that reads a
  bracketed paste to its end marker, as Python's REPL does, takes whatever arrived with it as
  text. It keeps the last write and answers after the Enter.
- `footer`, called from `agent_bar`'s footer for a terminal with no agent CLI in front: the card
  (Allow and Deny first, since the toast stacks over the footer's right end (#593); then the
  caller, the program, what it would type) or the bar (who typed what, while that program is
  still in the foreground, read through `program_of(context.terminal)` since the footer renders
  inside its view (#595); Take Over and a close button,
  or "You have control" and Hand Back). `Drive::control` is `Control::Agents`, `Quiet` or `Taken`
  (#810): the close button (`close_bar`) and Hand Back leave `Quiet`, where the footer draws
  nothing, and the next write or run turns it back to `Agents`. `marley::TakeOverTerminal`
  (Ctrl-I in `Terminal`) toggles the take-over, advancing the generation, while an agent has typed
  into the focused terminal's program, and otherwise propagates so the key reaches the program.
- `mcp.rs` grants `terminal.write` at start beside `browser.write`, answers `terminal_screen` in
  line and hands `terminal_type` to its own task; a closed terminal's state goes on release.
- `run_at_prompt` (for `terminal_run`, #556): `check_run` refuses an empty, long or multi-line
  command, a take-over, a card already waiting or a run in flight, and `prompt_refusal` a program
  or an agent CLI in the foreground, a prompt not signed by the terminal's own shell
  (`prompt_shell()`, PR-claude-474), and anything typed at it (nothing since the prompt when
  `input_start()` is unset, else `autosuggest::typed_text`). `marley_terminal::agent_commands::
  verdict` with the two lists and `marley.agent_commands_outside_lists` decides whether to ask:
  the card reads `<who> wants to run <command>` with Run and Refuse, and the toast has Show;
  `RunAgentCommand` and `RefuseAgentCommand` (Enter and Escape in `Terminal`) answer a run card
  in the focused terminal and otherwise propagate. `type_run` checks the prompt again and types
  Ctrl-U, the command and a return, as Rerun does; the wait polls every 50 ms for the first
  verified block past the count at typing whose command is the one typed, which joins `runs`
  (the mark, `terminal_blocks`' `agent`), and for its end, within 25 seconds of the call and the
  run's `wait_seconds` of the typing. The answer reads the block as `terminal_read` does.
  Where the terminal's blocks keep agents out of history (#553, the setting off when the terminal
  started), the command is typed after a space, and `agent_blocks` gives the suggestions the
  runs to skip; `init` keeps `terminal::MarleyAgentHistory` with the setting.
  `agent_mark` draws the sparkle before the pill through `bookmarks::chip`; the bar reads `<who>
  ran <command>` with Take Over while the run's block runs, and Ctrl-I takes over once an agent
  ran a command there.

## A project's launch configs (`src/launch.rs`, #527)

- The file is `.zed/marley.json` (`LAUNCH_FILE`), read with `settings::parse_json_with_comments`
  into `LaunchFile { launch: IndexMap<String, ConfigFile> }` (unknown keys refused), then
  `checked`: each item exactly one of `terminal`, `agent` (a name in `AgentKind::ALL` by its
  program) and `browser` (`marley_browser::address::agent_url`, http and https only), a `cwd` of
  normal components only, at most one `focus`. A `LaunchConfigs` global keeps each folder's
  `Configs` (the configs, or the first error): `init` reads a workspace's folders as it opens and
  again on `project::Event::WorktreeAdded`, or `WorktreeUpdatedEntries` naming the file, through
  the project's `Fs`.
- The rail's `launch_entries` puts them after the Agent CLIs under a Launch header (a broken
  file: one disabled entry with the first 90 characters of the error); `Rail::launch` activates the
  workspace and calls `launch::run`.
- `run`: `text` (one line an item) and its SHA-256 in hex; `KeyValueStore::global(cx)
  .scoped("marley-launch")` keyed by the folder and the config's name; a text not approved asks
  (Run, Cancel; "changed since you approved it" when another hash is kept), and Run writes the
  hash before `open_items`. The prompt gets the text through `verbatim` (#592), a Markdown code
  block fenced one backtick longer than any run inside, since Zed's prompt renders its detail as
  Markdown; the hash is still taken over the plain text.
- `open_items` opens each item in the active pane (`agents::start_in_terminal` with the item's
  folder and `marley_agent::send_payload(command)` or `agents::launch_input(kind)`, the terminal's
  view found by its terminal and titled with `set_custom_title`; `browser::open_url_tab`, which now
  returns its tab, after `wait_for_port` for a loopback URL: `marley_browser::address::accepts`
  every half second, 30 seconds at most). A `split` item moves into a pane split off the previous
  item's (`Workspace::split_pane`, `workspace::move_item`), since the active pane follows a focus
  change that may not have landed. The `focus` item, else the first, has the focus at the end.
- `agents::start_cli_with_prompt` now takes `&str` and `Option<&str>` and is a wrapper over
  `start_in_terminal(workspace, directory, input)`, which opens a center terminal and types the
  input after the shell's handshake; `launch_mode` and `launch_input` give an agent's line.

## Review notes to the agent (`src/review_notes.rs`, #522, #729)

- `init` registers Zed's `editor::actions::SendReviewToAgent` on every workspace; the diffs'
  toolbar button focuses the diff and dispatches it. `open_picker` takes the active item as an
  `Editor` (`act_as`, which a project diff and a branch diff answer with their right-hand
  editor) and its `unsent_review_notes`; the rows are `send_selection::agent_targets` whose
  working directory holds every note's file (`holds_every_file`), labeled "agent · project ·
  ready | working | asking for permission | no idle signal", then Copy notes.
- `deliver`: Copy writes `prompt(notes, None)` to the clipboard; an agent not `ready` gets a
  toast; a ready one, after the picker's update, gets its window activated, its terminal revealed
  and focused, `paste_then` of `prompt(notes, cwd)` with `\r` after the pause (#594), and the
  diff's editor
  `mark_review_notes_sent(ids)`.
- `prompt` turns the notes into `ReviewLine`s with absolute paths for `message_of`, which makes
  each file relative to the agent's folder when it lies under it, hands the notes to
  `marley_agent::review_prompt`, and wraps them with `review_message` in
  `MarleySettings.review_notes_template` (#732, `marley.review_notes_template`, blank taken as
  unset), so the paste and Copy carry the same message.
- Since #729 `refused` (the toast for an agent not ready) and `paste_and_submit` (the deferred
  paste with Enter, then an `after` callback) serve the diff's notes and `send_file_notes`, which
  opens the same picker for one Markdown file's notes from the review tab (rows whose folder holds
  the file) and runs the tab's `on_sent` once an agent took them. `reply_message` wraps a reply's
  notes in the same template.
- The Zed side: `DiffReviewFeatureFlag::enabled_for_all`; `StoredReviewComment::sent`; the count
  of unsent notes behind `ReviewCommentsChanged` and the button; `ReviewNote`,
  `Editor::unsent_review_notes` (anchors to points, `point_to_buffer_point`, the buffer file's
  `LocalFile::abs_path`, rows from 1) and `Editor::mark_review_notes_sent`; the row's Sent label.

## Voice (`src/voice.rs`, #480, #642)

- Everything here waits for `MarleySettings::dictation` (`marley.voice.enabled`, read "off unless
  on" by `Dictation::from_content`, #642). Off, `microphone` in `agent_bar.rs` returns nothing
  before it reads `Voice` or calls `follow_once_drawn`, and `marley::ToggleDictation` answers with
  a toast (`NotificationId::unique::<Voice>()`) that names the Voice section, so no `voxtype`
  runs. `init` observes the settings store, and while off `stop_following` drops the follower,
  which kills `voxtype status`, and resets `following` and the state itself, since a dropped task
  never reaches its own reset. Turned on, the next microphone drawn starts a follower again; each
  `TerminalView` redraws its bar on a settings change.

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
  subscribes every new `TerminalView` to its terminal's `Event::MarleyNotification` through
  `watch`; it also observes the view and watches again when its terminal's entity changes, since a
  task's Rerun hands the view a new terminal through `TerminalView::set_terminal` (#543).
- `notify` skips the focused terminal of the active window; otherwise it posts a
  `SystemNotification` tagged by the view, titled by the escape or, for an OSC 9, by the tab.
  The view has already set its bell, which marks its tab and its rail row.
- `show_sender` answers a click: it activates the view's window, its workspace in the
  multi-workspace and its item, and clears the bell, as the rail's `activate_terminal` does.
- Claude Code's events (#538): `on_seat_change`, beside `push::on_change` for each frame that is
  not a `SessionStart`, takes the event `TurnEvent::of_change` gives. In a terminal the user is
  not looking at it marks the view in `Attention` (the rail's dot, which the rail observes) and,
  past the project's five-second cooldown, posts `event_line` over
  `claude_events::banner_body`. `notify` keeps the same cooldown, by the view's workspace. `seen`,
  on the view's focus-in and its window's activation, clears the mark, and the release drops it.
- `notify_stall` (#569) posts the stall kind's banner, `<project>: Claude Code may be stuck`,
  behind the same focus rule, under a tag of its own (`marley-stall-<id>`), so it neither replaces
  nor is replaced by the terminal's other banners, such as Claude Code's own. `post` is what the
  two share, and a click shows the terminal as the others' does.

- A notification titled `marley-event` (`marley_terminal::AGENT_EVENT_TITLE`) is a Claude Code
  hook event, not one for the user (#519): `init`'s subscription hands its body to
  `agent_events::on_frame` instead of `notify`, and registers `agent_events::forget` for the
  view's release. Zed's view leaves such a notification unmarked, so no bell either.

## A long command's end (`src/command_watch.rs`, #551)

- `init` observes every new `TerminalView`: it takes the view's baseline (the last finished
  block's index, so a split tells nothing of the ends before it), then checks the terminal's last
  block on each notify of the terminal and each `Event::TitleChanged`, which the foreground
  process's refresh emits when the password flag changes. The view's release drops its entry.
- `check` leaves a terminal alone while an agent CLI runs there (`agent_in`) or when the block's
  own command names one (`marley_agent::agent_kind_of`): the agent's block ends after the agent
  has left the foreground, and #538 words its banners. A finished block newer than the last one
  told, at or over `marley.long_command_seconds` (0 is never), posts `done in <d>` or `exit <n>
  after <d>` titled with the command; a running block whose PTY reads with echo off
  (`Terminal::marley_foreground_reads_password`) posts `waiting for a password` once per block.
  Both go through `notifications::notify`, so the focus rule and the project's five-second
  cooldown hold, and both set the view's unread mark (`notifications::mark_unread`) first.
- The rail's plain terminal rows carry the same block as `CommandSnapshot` (`command_snapshot` in
  `rail.rs`): the command on one line, cut to 60 characters, and a state `command_line` words.
  The line is a `RowLine` whose `state` renders in a label of its own after the truncating
  command, so a long command never hides it; a non-zero exit is drawn in the error color. An
  agent's block has no line, by `check`'s test.

## A running command's printed error (`src/running_errors.rs`, #572)

- `init` observes every new `TerminalView`. While `marley.system_one.uses.running_error` is not
  `off`, an `Event::Wakeup` schedules one read 500 ms later unless one is pending; each notify of
  the terminal checks whether the block being read is still the running last block; the view's
  focus logs `seen` for a flagged call; a settings change to `off` drops every watch and mark.
- A read takes the running last block unless an agent CLI runs there or the block's command
  names one (L-claude-551), an SSH client runs (`links::ssh_in`) or the project is remote. It
  reads `Terminal::marley_lines_since` from where the last read ended (the block's
  `output_start` at a new block) and scans each line with `marley_terminal::running_errors::scan`.
  A failure opens the block's `Episode`, a recovery closes it, and an open line is asked through
  `system_one::ask(RUNNING_ERROR, ..)` with the lines around it and the facts (the program, how
  long it runs, the shapes in the last 40 lines). The reading, in `act`, opens or closes the
  episode as a shape would; in `suggest`, it opens a questioned one and closes only a questioned
  one; in `shadow`, it is only logged. A grace timer fails a suspect five seconds after its line.
- A flag records a `rules` row when the shapes made it, sets the view's mark in `ErrorMarks`
  (the global the rail observes, written only when a mark changes, where `Watch`, changed at
  every read, is observed by nothing) and, unless it is questioned or the terminal is in front,
  marks the view unread and posts `<project>: <command> printed an error` over the line through
  `notifications::notify`. A recovery posts `<project>: <command> recovered` when its flag was
  told. The block's end clears the mark silently. The first of `seen`, `recovered` or `ended` is
  the flag's call's outcome.
- The rail draws the mark (`terminal_marks`, `running_error_mark`: `Close` in the error color,
  a `?` when questioned, the line as its tooltip) and the line in red under #551's command line,
  and its filter matches the line.

## English at the prompt (`src/english.rs`, #557)

- `init` reads the search path's program names off the main thread (`Commands`, from
  `agents::launcher`) and registers `marley::AskAgent` (Ctrl+Shift+Enter in `Terminal`).
- `is_command`: a builtin (`marley_terminal::english::BUILTINS`), a program on the search path,
  or the first word of the terminal's own verified commands; before the path is read, every word,
  so no hint shows early. `read_line` decides with it.
- `hint` returns ` · ctrl-shift-enter asks the agent` for a typed English line while
  `marley.english_hint` is on; `autosuggest`'s hook closure shows it where no history suggestion
  applies, and `AcceptSuggestion`, which recomputes only the history's, never types it.
- `ask_typed` (the key) takes an English line only, clears the shell's line with Ctrl-U, and
  `ask` sends it: one agent terminal of the window (`send_selection::agent_targets`) gets it as a
  paste and a return (`send_text` with `submit`); several, #549's picker; none, Claude Code
  through `agents::start_cli_with_prompt`. A command's key reaches the shell.
- `asks_on_127` makes #555's chip show on the newest block that ended with 127 when its verified
  command reads as English, even with no agent running; its click asks with the command
  (`ask_later`, deferred past the element's update).
- Since #573, `hint_for(line, terminal)` gives the words for any line, the grid's or the prompt
  editor's: #573's reading from `typed_line::shown` where one is kept for exactly that line
  (`request`, `comment`, `command_then_english`, with `?` in suggest; `command` hides #557's hint
  in act), else #557's; in act a command followed by English carries `arguments`, the byte range
  of the words after the first, for the editor's warning colour. `ask_typed` reads the shell
  editor's line while it is open (`rich_input::shell_text`, then `clear_shell`), takes a line
  that reads as English or whose shown reading offers the agent (`offers_agent`), and writes
  `asked the agent`. `open_case`, `reads_as_english` and `command_source` serve `typed_line`.

## The typed line (`src/typed_line.rs`, #573)

- `Lines`, a global keyed by the terminal's entity id: the line in front, the quiet timer, the
  call about the line (its row, text, kind and the next block's index), the last line asked about
  with its kind, a call entering the shell and a call whose block runs, and the view's workspace.
- The grid's line is read in `autosuggest`'s hook, at each frame (`follow_grid`), since typing
  reaches the terminal as wakeups that notify no observer; it is skipped while a shell editor
  holds the terminal's line (`rich_input::holds_line_of`). The editor's line comes from its edit
  subscription. Either goes through `changed`: a call about another line gets `edited` or
  `cleared`; an open line (`english::open_case`), with the use on and nothing the redactor finds,
  arms a 250 ms timer.
- `settled` asks, when the line is unchanged: the last line asked about reuses its kind with no
  call; an unlisted project (`system_one::detail`) makes none; otherwise `system_one::ask(TYPED_LINE)`
  with the line as text and facts (the project, the first word and where it is known from, the
  word count, the markers, a history match, the last exit code) and #557's reading as the verdict.
  The ask's task is detached, so its row is always written; `answered` keeps a model's clear
  choice for the line still in front, notifies the terminal and repaints the editor's hint
  (`rich_input::refresh_hint`), or logs `dropped`.
- Outcomes: `entering` (the editor's send, or the grid's prompt left) holds the call until a block
  past its index opens with its command (`entered`), then `exit N` when that block ends, both seen
  from an observer of the terminal; `asked_agent` writes `asked the agent`. A settings change
  drops the kept readings and timers.

## The agents' versions (`src/agent_versions.rs`, #648)

- `AgentVersions`, a global with one `Check` each for Claude Code and Codex (the path found, its
  identity: canonical path, size and modification time; what was found; when the last check
  ended; whether one runs). `init` checks both, and redraws the windows when the settings move the
  allowed rows.
- `check(kind)` finds the program off the main thread (`MARLEY_CLAUDE` or `MARLEY_CODEX`, else
  `which_in` on the launcher's search path) and, when its identity changed, runs `--version`
  through `process::output` with the program's folder first on `PATH`, raced against 5 s
  (`process::output` kills its child when dropped), reading 4 KiB of stdout, then stderr. It logs
  `agent versions: <name> <version> at <path>` (or what went wrong) when the outcome changes, and
  redraws the windows when a verdict moved.
- `chip(kind, context, cx)` asks for a check when the last is 10 s old or more (as
  `agent_notify` does), and draws a warning chip under a local terminal whose agent has a row
  `Off` for any reason but `NotChecked`; its tooltip is `versions::reasons`, its click dispatches
  `zed_actions::OpenSettingsAt` at `marley.allow_untested_versions.<id>`.
- `prompt_reading(terminal, cx)`: `Recognized` for a remote terminal and while
  `claude_prompt_tags` is on or allowed, else `AllTyped`. `agent_events::on_frame` reads it once
  per frame and hands it to `claude_events::fold_with`, `after_fold`'s outcome note and
  `turns::on_event`'s turn title.
- `allowed_in` reads `marley.allow_untested_versions` into `MarleySettings`; the e2e harness
  allows `claude_prompt_tags` in each run's copy and names a missing `MARLEY_CODEX`.
- Since #650 `is_on(integration, cx)` and `program(kind, cx)` (the path the check found) serve
  `codex_server`, and the chip lists `codex_app_server` only while `marley.codex_app_server` is
  on.
- Since #720 `outdated(kind, cx)` lists the integrations `wanted` (now `pub(crate)`) that the
  found version is older than (`Range::older`: the release below the range's `from`; a newer
  untested version is not older). `warn_if_outdated` runs after each stored check and on every
  settings change (turning a part on can make the version too old for it): once per program and
  version, kept in `AgentVersions::warned` for the session and in the key-value store's
  `marley-agent-versions` scope (`claude-2.1.280`), an app notification names the version, each
  part with the version it needs, and `versions::update_command` (`claude update`,
  `codex update`), with Open MCP Servers. `show_app_notification` keeps it for windows opened
  later, so a check that ends before the first window still shows.

## MCP Servers (`src/mcp_servers.rs`, #720)

- `marley::OpenMcpServers` and Home's MCP Servers button open the tab, one per group: `open`
  brings the group's tab forward and refreshes it, else adds one to the center.
- `refresh` looks up the open projects once the caller's update has ended (`spawn_in`, then
  `update_in`; the action runs while its workspace is updated, and `window_workspaces` reads every
  workspace), subscribes to each project's `ContextServerStore` for `ServerStatusChangedEvent`,
  and reads the CLIs' files in `background_spawn`: `claude_rows_in(.claude.json, folders)` and
  `codex_rows_in(config.toml)`.
  - `.claude.json` is in `CLAUDE_CONFIG_DIR` when set and not empty, else in the home folder (as
    Claude Code 2.1.296 places it). Only `mcpServers` and `projects` are deserialized, each entry
    as a borrowed `RawValue`; a folder's entry is parsed only for an open folder, and each server
    on its own, so an entry Marley cannot read becomes a row saying so. `.mcp.json` servers are
    approved or denied by the folder's `enabledMcpjsonServers` and `disabledMcpjsonServers`.
  - Codex's `[mcp_servers.<name>]` tables through `toml_edit`: `command` and `args` or `url`,
    `enabled`, the names of `env`, `env_vars`, `http_headers`, `env_http_headers`, and
    `Authorization` for `bearer_token_env_var`.
  - Env and header maps are read as `Names`, a map visitor that keeps the keys and skips each value
    with `IgnoredAny`, so no value is held in a row. A command line goes through
    `Redactor::new(&[])` (the built-in rules) and a URL through `redact_url`.
- `render` draws Agent CLIs from `agent_versions` (program, version, each integration's state:
  needs a newer version, not turned on, on, or off on this version, with `Range::words`), the two
  CLIs' rows, and Zed's rows from the stores (`server_ids`, `is_server_enabled`,
  `status_for_server`, `settings_for_server`), one per name with each project's status, `marley`
  and `rusty` marked Marley's. Configure in Settings dispatches `OpenSettingsAt` at
  `context_servers`.
- `mcp::window_workspaces(window, cx)` (moved from `turn_undo`, #714) is every workspace of
  Marley's windows from inside one window's update.
- Since #768 a Claude Code or Codex row carries `stored: Option<Stored>` (its `Place`, the
  command and raw arguments or the URL, the names of the env variables or headers the form edits,
  and `seen`, the entry's hash as read: `entry_hash` of its compact JSON, or of its TOML item's
  text), never a value. The tab keeps the open folders for Add Server's Where.

## Editing a CLI's MCP servers (`src/mcp_server_form.rs`, #768)

- `Place` (`ClaudeUser`, `ClaudeLocal(folder)`, `ClaudeProject(folder)`, `Codex`) names the file:
  `.claude.json`, the folder's `.mcp.json`, Codex's `config.toml`.
- `change_text(text, place, name, change, seen)` is the `edit` handed to
  `safe_write::replace_in(file, backup: true, …)`. `check` compares the entry's hash now with
  `seen`: a different one, a missing one for an edit, or a name already there for an add refuses
  ("changed since the tab read it; Refresh and try again"). Only the entry is compared, since
  Claude Code rewrites `.claude.json` read-modify-write while it runs.
  - JSON (`claude_change`): the whole file as a `serde_json::Value` with `preserve_order`, the
    entry changed in place (`shift_remove`, `insert` keeps a key's position), written with
    `to_string_pretty` (two spaces) and the original final newline. A local scope needs the
    folder's `projects` entry already there: Claude Code fills a new one's keys itself.
  - TOML (`codex_change`): `toml_edit`, `[mcp_servers]` made implicit when missing, the entry's
    table edited in place; `env` or `http_headers`, inline or a table, keeps its style and
    order; `enabled`, `env_vars` and `bearer_token_env_var` stay.
  - An edit starts from the stored entry, so keys the form does not know stay; a pair's `Keep`
    copies the stored value inside the writer, the only place a value is read.
- `McpServerForm`, a `ModalView` opened by `McpServers::open_form` through `toggle_modal`: name
  and Where for an add, Command or URL, the pairs (a stored one is a label, a masked value editor
  reading "set; type to replace", and Clear; a new one two editors), Save (`menu::Confirm`). The
  arguments (`shlex::try_join`) and the URL show as the tab shows them, through the redaction
  rules and `redact_url`; a field left as shown keeps what is stored, and an edited one that
  still holds `[redacted: …]` or `…` is refused. Save writes in `background_spawn`, then
  refreshes the tab, shows a toast and closes; a refusal stays in the form.
- `remove(page, workspace, name, stored, window, cx)`: `window.prompt` (Remove, Cancel), then
  the same writer with `Change::Remove`; an error goes to `show_error`.

## Codex's own App Server (`src/codex_server.rs`, #650)

- `marley.codex_app_server` (`MarleySettings::codex_app_server`, a `CodexAppServer`: `Off`, the
  default, or `On`). `prepare(project, folder, cx)` gives a `Prepared` (the program #648's check
  found, a socket `marley-<pid>/codex/<n>.sock` in `XDG_RUNTIME_DIR` or Zed's temporary folder,
  the folder) when the switch is on, the project local, the system Linux, `CODEX_APP_SERVER` on,
  and every path free of whitespace and the socket short enough; otherwise `None`, logged.
- `agents::agent_line` builds a Codex launch's two lines, with and without the server
  (`Joining`); `start_cli_with_prompt` and `launch_input` (launch configs) use it.
  `start_in_terminal` with a `Joining` calls `start` once the terminal exists, and types the plain
  line when the server's socket does not come up in 5 s.
- `start`: the socket's folder made 0700; the server's environment, as Zed's terminal builder
  makes the terminal's (the folder's `directory_environment`, waited on for 3 s, the worktree's
  ports, the terminal settings' `env`, the agent's variables, `MARLEY_TERMINAL_ID` and
  `MARLEY_PROJECT`); `process::serve` of `<codex> app-server --listen unix://<socket>` in the
  folder; the process task, which keeps stderr's last line, reports the server's own end, and on a
  stop sends SIGTERM, then kills after 2 s, then removes the socket (and, for 0.158's link, the
  socket under `/tmp/codex-daemon-<uid>/` and its lock). The `Server` is kept in `CodexServers` by
  the terminal and dropped with it; the quit hook kills every server at once and removes
  `marley-<pid>`.
  The task holds `util::command::Child`, smol's on Linux and Zed's own posix_spawn child on
  macOS, so `last_line` reads any `AsyncRead` (#786).
- The follow (`follow`): it waits for the TUI's connection, an accepted socket under the path in
  `/proc/net/unix` (state 03), then 1 s, so the TUI's `initialize` names the originator. It
  connects (`smol`'s `UnixStream`, `async_tungstenite` at `ws://localhost/rpc`), initializes as
  `marley` with the experimental API and the delta notifications opted out, checks the server's
  `userAgent` version against `CODEX_APP_SERVER`, and sends `initialized`. Each second it lets
  the lead go when Codex left the foreground (`thread/unsubscribe`), looks for a lead with
  `thread/loaded/list` and `thread/read`, and resumes a lead not yet resumed once it is idle
  (`thread/resume`, `excludeTurns`), never during a turn. A `thread/started` lead becomes the
  lead; any other thread Marley was attached to is unsubscribed. Lead notifications fold into the
  seat (`agent_events::fold_codex`); other lead messages mark it heard from, once per 5 s. The
  server's end, or the connection's, fails the seat with the last stderr line.
- The client (`Client`): one channel carries the responses, the notifications, the connection's
  close and the server's end; a call waits 5 s for its id and keeps what comes meanwhile. A
  message with an id and a method is a server request, dropped unanswered, since any answer
  resolves it and an error reads as a denial.
- The readers: `agent_events::seat_agent` and `seat_of` give a seat by its agent label; the
  rail's row, `note_claude_code`, the inbox entry and its icon, the close guard and the Browser
  tab's Send read Codex's seats too; the stall watch and the inbox's risk and route marks keep to
  Claude Code's.
- Approvals (#651): a server request becomes `Event::Request`; the follow keeps the lead's
  requests of the four methods (`Shown`: the request, a file change's paths from its
  `item/started`, the decision sent) with its connection's generation, and publishes them in
  `CodexRequests`, which the rail observes (`requests_of`). Any other request is left to the TUI.
  `answer(terminal, generation, request, decision)` puts an `Event::Answer` on the follow's own
  channel; `answered` sends the response only while the same request, with the same params, waits
  unanswered on that generation, then marks it sent. A request leaves on `serverRequest/resolved`,
  a new lead, the lead's close, Codex leaving the foreground, the follow's end and the terminal's
  release.
- The inbox (`rail.rs`): a terminal with requests lists one `InboxKind::Codex` entry each
  (`codex_entry`, keyed by the view, the generation and the request id) in place of the seat's
  wait; its buttons come from the request's decisions (`inbox_answer`), wrap under the entry, and
  read `Sent: <decision>` once sent; `codex_waiting` gives #568's marks; a click sends through
  `codex_decision`, and the entry's body shows the terminal.
- **Usage limits (#715).** `heard` takes `account/rateLimits/updated` before its thread filter:
  `RateLimits::decode` passes over any bucket but `codex`, `Following.rate_limits` merges each
  sparse update, and the merged windows fold into the lead's seat (`Input::RateLimits`), again
  after a new lead, whose seat starts with no labels. The follow publishes its channel at its
  start, so `generation_of` and `continue_lead` reach it before any request. `Event::Continue`
  carries a `limits::Limited`; `continued` sends `turn/start` with `RESUME_PROMPT` only while the
  generation, the lead's id, its idle state, an empty request list and the seat's limited turn
  all still match, since a `turn/start` during a turn adds to that turn; a refusal or a failed
  call is logged. `Following::fold` hands the seat to `limits::on_seat` after every fold, and the
  follow's end and the terminal's release call `limits::forget`.
- **`src/limits.rs` (#715, #765).** A `Limited` names its agent, the Codex follow's generation
  (0 for Claude Code), the session (Codex's thread, Claude Code's session id), the failed turn and
  the reset (`of_codex`, `of_claude`); its `LimitKey` leaves the reset out. An arm keeps the view's
  id and the terminal's weak handle, since Continue in Needs you runs inside the window's update.
  `send` continues a Codex through its follow and types a Claude Code's continuation
  (`type_continuation`: `paste_then` with the resume prompt and Enter, only while the seat still
  reads the same limit). `agent_events::on_frame` and `agent_reports` hand a Claude Code seat to
  `on_seat` after it moves. The `Arms` global holds one armed `Limited` per terminal entity, the limits already continued or cancelled
  (`handled`), and whether a check runs. `arm` starts a check at the earliest reset, 30 s at most,
  against the wall clock (`now_ms`), so a reset that passed during sleep fires at the next check;
  `fire_due` sends each due arm once. `on_seat` drops an arm the seat no longer matches (a turn
  started, a wait, the thread closed or changed) and, with `ContinueAtReset::Always`, arms a new
  limit with a known reset unless it was handled. `cancel` records the limit as handled, so the
  setting does not arm it again; `continue_now` sends at once.
- **The rail (#715).** `usage_of` gives each agent row its `Usage`: the quota windows through
  `Signals::of` (a percent above 100 reads as 100, `used_percent_of`) and a Codex's `limit`.
  `codex_line` adds the limit's reset; a limited row's line is drawn in the warning colour;
  `usage_parts` gives the quota line (`quota_line`, the kind as `kind_words` writes it, `5 h` or
  `7 d`) and the tooltip, titled `<agent> usage`, every window by name (`window_words`); harness
  rows share `quota_line`. `next_reset_change` arms the minute timer for the next change of any
  shown countdown. A limited Codex is an `InboxTarget::Limit` entry (`limit_entry`, keyed by the
  view, the generation and the turn): Continue at Reset, Continue once the reset passed,
  `Continues at <clock>` with Cancel while armed (`clock_words`, local time), or no button; the
  rail observes `Arms`.

## Codex's and OpenCode's notifications (`src/agent_notify.rs`, #552)

- `AgentNotify` knows whether Codex's `config.toml` (`CODEX_HOME` or `~/.codex`) asks its TUI for
  notifications Marley shows, and the state of Marley's plugin for OpenCode (`XDG_CONFIG_HOME` or
  `~/.config`, then `opencode/plugins/marley.js`): missing, older or current by its first line.
  It reads both through `smol::unblock` at start, when a Codex or OpenCode bar draws two seconds
  after the last read, and after each write, and redraws only on a change.
- `chip` gives the bar Turn on Codex notifications, Connect OpenCode to Marley or Update Marley's
  plugin for OpenCode; a click writes off the main thread (`configure_codex` with `toml_edit`,
  every other key and comment kept, a list of events for `notifications` left as it is; the plugin
  whole) and shows a toast or the error.
- Since #755 both writes go through `safe_write::replace_in(path, backup, edit)`, the helper for a
  file another program reads. It follows a symlink to the real file (a dangling one names the
  file to create), reads the bytes and the identity (device, inode, mode, length, modification
  time), and makes the new text; equal text writes nothing (`Replaced::Unchanged`). The new text
  goes to `.<name>.marley-<uuid>.tmp` beside the real file (`create_new`, the original's mode or
  0644 for a new file, synced). With `backup` (Codex's config, not OpenCode's plugin, which is
  Marley's whole file) the old bytes go to `<path>.marley-backup` beside the path as named. The
  file is read again, and a change in bytes or identity refuses the write ("changed while Marley
  wrote it; nothing was written"); otherwise the temp is renamed into place and the folder synced.
  A failure removes the temp. Marley's own files (`tasks.json`, the programs `mcp.rs` writes, the
  fixed-name temps in `clients.rs`, `service.rs` and `relay.rs`) do not use it yet.
- `agent_plugins/opencode/marley.js` writes an OSC 777 notify to `/dev/tty` on `session.idle`,
  `permission.asked` and `session.error` while `TERM_PROGRAM` is `zed`, so it reaches the terminal
  whatever the process's stdout is, and never throws; #478's path shows it.

## A note when a key is taken from a terminal (`src/shortcut_note.rs`, #563)

- `taken(action, did, terminal, workspace, window, cx)`, called where Rich Input, the block keys,
  New Agent (from a terminal) and the take-over act: the key as the terminal's own contexts bind it
  (`Window::bindings_for_action_in` with the terminal's focus handle, the last binding; the
  window's own lookup reads the frame's root contexts and misses a `Terminal` binding), a toast
  deferred into the workspace with Open Keymap, and the action's name in the key-value store's
  `marley-shortcut-note` scope, so the note shows once per data directory. A `Shown` global saves
  the store a read in the same session.

## Remote terminals (`src/remote.rs`, #543)

- `marley: open remote terminal` opens a picker of Zed's saved SSH hosts
  (`RemoteSettings::ssh_connections`), each through `marley_remote::saved_target`; an entry that
  fails its checks is left out. Confirming refuses a project not on this machine (its task would
  run ssh on the project's host), then schedules a task resolved with the id base `marley-remote`:
  `marley_remote::remote_terminal_command` with a fresh `SessionName`, each word quoted for the
  system shell (a task's arguments reach it as shell text), `MARLEY_SSH` in place of `ssh` when
  set, revealed in the center, reusing its terminal, with the summary line on and the command line
  off. `routing.rs` places it and reruns it in place, so Rerun attaches the same session.
- `is_remote(terminal)` finds the id base on the terminal's task. `agent_events::on_frame` takes a
  remote terminal's frames although its foreground is ssh, and the rail's `remote_claude` takes a
  remote terminal with a seat as Claude Code's, so its row shows the seat and `note_claude_code`
  leaves the seat to the host's `SessionEnd`.
- Since #641 a link that dies is known dead. `remote_terminal_command` gives ssh the keepalive
  (`marley_remote::keepalive_options`) and a connect timeout. `routing.rs` calls `starting` before
  a remote run (its `Backoff::up_since` is now, and the task's down entries go) and hands the run's
  end to `supervise`. An exit of 255, ssh's own error status, turns the terminal down: the
  `Links` global keeps a `Down` by the ended terminal's entity, the terminal's input is held
  (`Terminal::marley_hold_input`, Zed's `terminal.rs`), a ticker draws its line each second, and
  `check_until_up` checks the host with `link_check_command` through `process::output`
  (`MARLEY_SSH` honoured) on `check_delay`'s backoff, the failures reset by a drop after
  `STABLE_LINK`. `Answers` calls `reattach`: `TerminalPanel::spawn_task` with the provider's own
  unprepared task and `reveal: Never`, followed by a detached task that drops the old entry and
  supervises the new terminal (the old terminal's release ends the checks, so they cannot follow
  it). `Down` waits again with ssh's reason; `Stopped` leaves the line saying why. The entry goes
  with the old terminal (`observe_release`), so a Rerun or a closed tab ends the checks.
  `link_overlay`, asked first by `block_filter::overlay`, draws the dim, occluding cover and its
  line. `rich_input::send` leaves a held terminal's prompt in the editor. Any other exit removes
  the task's backoff.

- Since #808 `open`'s second half is `open_words(host, label, id_base, words, …)`, which
  `tmux_sessions` calls too: with a host it puts `MARLEY_SSH` in the first word's place and records
  the host in `Links`; before scheduling it shows a terminal whose task has the same resolved id
  and still runs (`TaskStatus::Running`), since Zed's `spawn_task` holds a second run of such a
  task until the first ends. `Host` and `saved_hosts` are `pub(crate)`, and `TMUX_ID_BASE`
  (`marley-remote-tmux`) sits under `ID_BASE`.

## tmux sessions in the rail (`src/tmux_sessions.rs`, `src/rail.rs`, #808)

- `init` sets the `TmuxHosts` global and runs `follow` now and on each `SettingsStore` change:
  `marley.tmux_hosts` from the merged settings, each name trimmed and deduplicated, placed by
  `place_of` as `Place::Here` (`local`), `Place::Host` (the `saved_hosts` entry whose label, then
  whose host, it is) or `Place::Unknown` (a `Listing::Failed` at once). An unchanged list returns
  without touching the global; a host whose name and place are unchanged keeps its `TmuxHost`
  (listing and fold). The poll task runs only while a host is named.
- `list_all`, every `LIST_EVERY` (10 s) and once `LIST_AFTER_NEW` after a +, lists every host at
  once (`join_all`), each `list` raced against `LIST_TIMEOUT` (20 s): `tmux list-panes -a -F
  PANE_FORMAT` through `process::output`, here, or behind `MARLEY_SSH` or `ssh` with a connect
  timeout and `harness_hosts::ssh_arguments` (`-T`, `BatchMode`, the keepalive, `--`), the remote
  words through `shell_word`. `read_listing` takes "no server running" and "error connecting to"
  as no sessions, and any other failure as its first printed line. `parse` groups the panes by
  session in tmux's order and marks the session with the first pane whose command
  `marley_agent::agent_kind_of` names. Only a listing that differs, for a host still at the place
  it was listed for, is written, so an idle poll redraws nothing.
- `attach` (`tmux attach-session -t =NAME`) and `new_session` (`tmux new-session -A -s` a
  `SessionName`, then `list_soon`) go through `open`: here `env -u TMUX` and the tmux words under
  the id base `marley-tmux`; on a host `ssh -t`, the keepalive, the connect timeout,
  `ssh_command`'s port and destination, and the tmux words through `shell_word`, under
  `remote::TMUX_ID_BASE`, so #641's supervision and `is_remote` apply. The tab's label is the host's
  label and the session's name.
- The rail observes `TmuxHosts` (`observe_marks`, a notify) and `render_tmux` puts a
  `tmux_section` per host after `render_harness`'s sections, none while filtering: a header
  (`TMUX · label`, `heading_words`, a `+` `IconButton` that stops the header's fold click) and a
  `tmux_row` per session unless folded: `row_card` with `agents::cli_icon` or
  `IconName::Terminal` and `session_words`. A row runs `attach_tmux`, the + `new_tmux_session`,
  each in the Home group (`groups::in_group`), an error shown by `report`'s prompt.

## A project's own icon (`src/project_icons.rs`, `src/rail.rs`, #564)

- `icon_in(root)`, blocking: Orca's fifteen names by extension (PNG, SVG, WebP, ICO), then the
  icon `index.html`, `public/index.html` or `src/index.html` declares (`<link rel="icon">`, any
  attribute order and quoting; a scheme, `//`, `..` or `data:` skipped; resolved against the
  page's folder, `public/`, then the root). The first file at most 256 KiB that decodes wins, made
  32 px on its long side as a BGRA `RenderImage`; each skip is one log line.
- The rail keeps `ProjectIcon`s by the group's first folder: searched through `smol::unblock`
  when a local group first shows, again on `WorktreeUpdatedEntries` touching a candidate, a page or
  the file it shows, at most once a second; `project_icon` draws it at 16 px before the name.

## The harness's sessions (`src/harness.rs`, #534)

- `marley.harness` (`MarleySettings::harness`, Zed's `ContextServerCommand`) names rustal-harness's
  MCP server; `init` follows the setting, starting a run when it names a command and dropping the
  run, and with it the server, when it changes.
- The `Harness` global holds the command, the `Connection` (`Connecting`, `Connected`,
  `Down(reason)`), the fleet (`FleetSnapshot`), the server while connected, the section's fold,
  and a minute counter that redraws a working session's quiet line.
- The run (`follow`, `connected`): `ContextServer::stdio` and `start` within 5 s; `seed` reads
  `fleet_snapshot`'s structured content into a `FleetSnapshot` (the harness's conformance check
  asserts its snapshot is Marley's fold of its events) and keeps its `cursor`; every second
  `fleet_events { after }` pages are folded with `apply` until one is empty. An `isError` result
  whose text starts `resync_required` re-seeds. Any other failure, or a call over 5 s (`call`),
  ends the connection: `Down` with the root cause's first line, then a new start after 1, 2, 4 …
  60 s. Zed's client sees no server exit, so the failing poll is the sign. The global changes only
  when events arrive, the connection moves or the minute passes while a session works.
- `HarnessView`, a workspace item titled with the session's title: `session_read { id, range:
  { mode: tail, lines: 500 } }`'s lines in the buffer font, read when the seat's `last_event_ms`
  or state changes and every 2 s while it is not done, one read at a time; a failed read shows in
  red above the kept lines. `open` shows the one open for that id or adds one to the center.
- Since #632 the source is `Source::{Off, Command, Embedded}`, `marley.harness` first, then
  `marley.embedded_harness` (`EmbeddedHarness`). `embed` finds `rh` (`MARLEY_RH`, else
  `which::which`, off the main thread), checks the root `<data dir>/harness`'s socket path against
  107 bytes, and loops: `serve` runs `rh --state <root> serve` through
  `process::follow_with_errors` (stderr kept), waits 10 s for the JSON ready line, then sets
  `Runtime::Running` and starts `follow` on `rh --state <root> mcp` once (`follow_embedded`), reads
  stdout to its end, and gives the exit and stderr's last line; a run that never got ready is
  killed. A `serve` that says another runtime owns the root counts as running elsewhere: Marley
  follows it and looks again after a minute. Between runs it waits 1, 2, 4 … 60 s. The `Runtime`
  (`Starting`, `Running`, `Stopped`, `Missing`) is in the `Harness` global and speaks first in the
  rail's header, whose tooltip holds the whole reason; the rows are stale while it is not running.
- `shown_prompt` drops the `[wait <key>, generation <id>]` the harness adds to an actor's prompt,
  for the rail's rows and the inbox.
- In `rail.rs`: `observe_marks` observes `Harness`; `render_harness` draws the section after the
  containers, outside the model, its keys and the filter (hidden while filtering), with
  `harness_row` per seat (a dot, the title, the question, `no update in N m` past
  `no_update_after_minutes` for a working seat, or the state; `· stale` and muted while not
  connected); `harness_entries` adds each seat with a question to the inbox (`InboxKind::Harness`,
  `InboxTarget::Harness(id)`, the ask its prompt and options); `open_harness` opens the tab in the
  displayed workspace.
- Since #640 a session's labels say more (the harness's MREQ-005 to MREQ-007, D173, D174).
  `Signals::of(&labels)` in `harness.rs` is pure: `state.source` as `StateSource` (`protocol`,
  `reported` and `runtime` are declared; `detected` and any other word are not), the
  `progress.percent` and `progress.activity` pair, each `quota.KIND.percent_used` with its
  `quota.KIND.resets_at_ms` (most used first), and `quota.account` unless it holds an `@`; a
  value that does not parse is left out (`percent_of`: a finite number from 0 to 100, rounded).
  No source is declared, as the harness sent before. `harness_row` mutes the dot of a state not
  declared and puts the source's word, and `stale`, in the first line's end slot; it adds a
  progress line and a line for the most-used window (`62 % · resets in 1 h 35 m`, `used_words` and
  `reset_words`), and a tooltip (`Tooltip::with_meta`: the source's sentence, every window, the
  account). `harness_entries` skips a state not declared, so only a declared question reaches the
  inbox. The minute bump in `connected` also runs while a shown reset lies ahead. `row_card` is
  `rems(4.5)` tall for three lines under the title.

- **Writes (#689).** `harness_writes` reads `marley.harness_writes` from the merged settings.
  - `writes_on` is that switch with a harness source that is not `Off`. `Source::Embedded
    { writes }` makes a change restart the embedded follow, and `follow_embedded` adds `--grant
    write`.
  - `request` sends one tool call within 5 s, shared by `call` (reads) and `call_write`.
    `call_write` reads `{result: accepted, value}` or `{result: refused, reason}` from the
    structured content or the text, whether or not `isError` is set.
  - `HarnessView` holds `send_editor` (`Editor::single_line`), `status`, `views` and `writing`.
    `render_controls`, shown while `writes_on`, draws the question's option buttons
    (`answer`: `session_answer` with the full prompt), the send line (`send`:
    `session_send` with a new UUID; Enter arrives as `menu::Confirm` on the view), and Views
    (`surface`: `session_surface_to_human`, `view_line` quoting each argument). Each view gets
    Open and Copy, and its line is truncated after them. `open_view` (#690) types the quoted line
    into a new terminal of `HarnessView.workspace`, the workspace `open` was called in, through
    `agents::start_in_terminal`.
  - `OpenHarnessSession` opens `open_session_picker`, whose `OpenSessionDelegate` offers the
    names in `<data dir>/harness/profiles` while the harness is embedded (`profile_names`,
    read on the background executor) and the typed name. Confirm calls `session_open` and
    opens the returned id's tab. `filter_palette` lists the action only while `writes_on`.

- **The Manager entry (#694).** Per harness since #758 (below). `sync_manager_entry` runs after
  `seed`, after each page of events, and in `follow_setting`.
  - While `writes_on` and a seat's labels hold `role: manager`, it puts
    `agent_servers["Manager"]` in the in-memory defaults: a `Custom` server with
    `seat_command`'s program and arguments plus `acp`.
  - Otherwise it takes the entry out.
  - `Harness.manager_entry` keeps the command in place, so the defaults change only when it does.
- **The manager's notices (#688).** Per harness since #758 (below). `follow_thread` runs after
  each fleet poll while `has_manager` and `writes_on`.
  - `read_thread` pages `thread_read {after}` to its end (`ThreadPage`, at most `THREAD_PAGES`),
    keeping `thread_record`s whose author is `manager` as `ManagerRecord`s.
  - The first read only sets `Harness.thread_cursor`, and a resync clears it.
  - `manager_posted` raises the new records unless `manager_in_front` (the active window's
    workspace shows the Agent Panel with `Agent::Custom { id: "Manager" }` selected): a
    `SystemNotification` each, tag `marley-manager-<id>`, and `Harness.unread`. Unread clear on
    the first pass that finds the thread in front.
  - The rail's `manager_entries` lists them as `InboxKind::Harness` entries.
    `InboxTarget::Manager` opens a listed Manager thread or starts one.
- **A seat in one step (#691, `src/harness_seat.rs`).**
  - `Harness::seat_command` is the followed command before its `mcp`, or the embedded
    `Harness.rh` with `--state <root>`.
  - `NewHarnessSeat` opens `open_form`'s `NewSeatModal`: three single-line editors and the
    `SeatAgent`.
  - `create` runs `seat add NAME --agent … --cwd … [--role …]`, then `seat start NAME`, through
    `process::output` (`run_seat`). Exit 0 gives the JSON, and otherwise the shown refusal is
    stderr's last line without `rh: `.
  - On success `harness::open_in_home` shows the id's tab in the Home group (`groups::in_group`),
    which #689's picker now uses too.
  - `filter_palette` lists the action with `OpenHarnessSession`.
  - `seat_add` (#692) is `answer_seat_add`:
    - `tool_off` unless `writes_on`, and `seat_of` checks the arguments;
    - `settings_change::ask_user` asks (the question and wait `ask_then_write` uses too);
    - `run_seat` runs `seat add`, and `refusal_of` passes on any `[a-z_]+` code before the first
      `: ` (#693; exit 2 is Marley's own usage error);
    - the answer is `starting`, then `seat start` runs, with `SeatStartFailed` as a
      notification.

## Home's own page (`src/home_page.rs`, `src/rail.rs`, #701)

- `MarleyHome` (an `Item`: Home, `ListTree`) is drawn with `card`, `row` and `muted`, which live
  here since #782 (`agent_activity`, `mcp_servers`, `threads_page` and `rusty::home_tab` use them).
  - `home_page::ensure` adds it at index 0 of the active pane through `add_item_inner(…, activate:
    empty, …)`, taking the active tab only from an empty pane.
  - #699's hooks carry it: `ItemRemoved` calls `fill_emptied` (Home's last tab closed), and
    `fill_shown` calls `ensure` when the shown workspace is Home (`groups::is_home_workspace`).
    Since #782 `home_page::init` registers both, just before `rusty::init`, where
    `rusty::home_tab::init` did.
  - `ensure` runs inside the workspace's or the `MultiWorkspace`'s update, so `MarleyHome::new`
    reads neither: `ensure` hands it the `Fs`, and the rail is looked up in `render`.
- The cards:
  - **Start:** New Terminal is the rail's path (`TerminalPanel::add_center_terminal` with
    `default_working_directory` and the launcher's factory). Open Folder…, Clone Repository… and
    Command Palette dispatch `workspace::Open::DEFAULT`, `git::Clone` and
    `command_palette::Toggle` from the page's focus handle.
  - **New Agent:** `agents::installed_clis` on the launcher's search path, each button
    `agents::start_cli` on Home's workspace. Zed's Agent Panel runs no thread in a folderless
    workspace, so Home has no New Agent Thread.
  - **Recent Projects:** `WorkspaceDb::recent_project_workspaces`, local ones, at most 8, read
    once; a click calls `open_workspace_for_paths(OpenMode::Activate, …)`.
  - **Agents at Work:** `Rail::agents_at_work` (`AtWork`: threads not Done, agent terminals not
    Idle, the terminal's status named "<agent> · <state>"), with the rail observed once found;
    a click calls `Rail::open_selection`.
  - **Configure:** `OpenSettings`, `OpenSettingsAt { path: "marley.layout" }`, `OpenKeymap`,
    `Extensions` and `OpenGuide`.
- Every handler that updates the workspace is a plain closure over a weak handle, never
  `cx.listener`. The workspace's updates read its items, the page among them.
- `groups::make` names Home and Rusty by their kind; only named groups are numbered.
- #782 (Chad, 2026-10-13: "combining the rusty home page with the marley home page and then just
  show rusty items if it is configured"): `MarleyHome::rusty: Option<Entity<RustyCards>>`, made
  and dropped by `follow_rusty` on the `rusty::Rusty` global (`rusty::is_on`), rendered as a child
  after Recent Projects | Agents at Work; Configure moved under Agent Activity. The rail marks the
  page `TabKind::Home` in `member_tabs`, and `marley_rail::walk` lists it first under its group.

## Home and Rusty from the start (`src/rail.rs`, `src/groups.rs`, #700)

- `Rail::ensure_groups` runs at the end of each `refresh` and from `rusty_changed` while Rusty is
  on. For Home, and for Rusty while it is on, it skips a kind the window holds or is making
  (`groups::has_kind`, which reads `Groups::waiting` too), or whose restored record may still come
  back. Then, deferred, it either claims the start workspace for Home or calls
  `groups::with_group`.
  - **May come back** (`groups::pending_kind`): once the window's saved state is read
    (`StartupGroups::restored`, set in `restore_serialized_state`), one of its own saved group ids
    still pending. Before that, any pending record of the kind, until `SETTLE` (3 s) has passed
    since the rail was built; `StartupGroups::_settle` refreshes then. Zed applies a restored
    window's sidebar state after awaits that follow the window's opening, and `Groups::pending`
    keeps the records of windows never restored.
  - **The start workspace** (`start_workspace`): the shown workspace when it is local, folderless,
    and no group's or pending record's. `groups::claim` records it as Home, keeping any tab Zed
    opened there, such as its onboarding page.
- `rail_groups` lists Home, Rusty, Zed's project groups, then named groups, each group kind in the
  window's saved order, before `marley_rail::place` applies a dragged order.
- `group_context_menu` gives Home a label, as Rusty's (#675).

## Zed's editors for agents, and the area modes (`src/editor_tools.rs`, `src/agent_control.rs`, #704)

- **`agent_control.rs`.** `Area::Editors` reads its `MarleyAgentControlMode` from
  `marley.agent_control.editors` in the merged settings (`ask_first` when unset).
  - `admit(call, area, acts, what, cx)`: `off` refuses `agent_control_off`; a read passes; an act
    under `allow` passes. Under `ask_first` it passes when this run allowed the session (area,
    caller key, project key) or the project was allowed for good; otherwise it asks, as it always
    does under `ask_every`.
  - The caller key is `terminal:<id>`, else `client:<name>`, else `agent`. The project key is the
    caller's `project`, else its `cwd`.
  - The question is a `MessageNotification` built with Allow for This Session, Always for This
    Project and Deny as content buttons. Its answer goes through a one-shot slot, it waits
    `ANSWER_WAIT` (25 s), and it is dismissed after.
  - Always is saved in the KV store (`marley-agent-control`/`always`), and read back at `init`.
- **`editor_tools.rs`.** `open_editors` walks every window's `MultiWorkspace`, each workspace's
  panes and their `Editor` items with a singleton local buffer, once each. An editor's id is its
  entity id.
  - `list`: the path, the project (the visible worktree holding it), `is_dirty`, the language,
    whether it is its pane's active item, and the disjoint selections as 1-based points.
  - `read`:
    - `agent_activity::log` first, so the read is listed, refused or not;
    - the secret globs (`marley.agent_control.secret_globs`, else `SECRET_GLOBS`) through
      `PathMatcher::new_lenient` over the file name; a match is refused `secret_file`;
    - the buffer's text through `mcp::for_agents`, paged by `page_from` and `fill_forward`, with
      `next_line`.
  - `open`: only a path under a visible worktree (`project_holding`), else `outside_projects`. The
    area admits it as an act. Then the window shows that workspace (`MultiWorkspace::activate`)
    and opens the file (`open_abs_path`, focused), and `go_to_singleton_buffer_point` places the
    cursor.
- `agent_activity::log` is `gate` without the kill switch, for reads.
- **Since #705:**
  - `agent_control::Level { Read, Act, Sensitive }` replaces `acts`. A Sensitive call asks every
    time unless the area is `allow`.
  - `editor_edit` (`edit`) finds `old_text`'s exact matches in `buffer.text()`. None is
    `no_match`; several without `replace_all` are `ambiguous`. It edits them in one
    `start_transaction`/`edit`/`end_transaction_with_source(Agent)`, as Zed's agent does.
  - `editor_save` (`save`) runs `Project::save_buffer` on the editor's workspace's project.
  - `not_secret` guards read and edit alike. `OpenEditor` carries its workspace.
- **`lsp_tools.rs` (#712).** `editor_tools::answer` routes the four language server tools here;
  each is logged (#703) and admitted at `Level::Read`, then answered from a spawned task.
  - `open(path)`: `not_secret`, `editor_tools::project_holding` and `find_project_path` (else
    `outside_projects`), the file's existence off the main thread (else `no_file`),
    `Project::open_buffer` (no tab), then `register_buffer_with_language_servers`, its handle held
    in `Opened` until the answer is built, and a poll every 200 ms, up to 15 s, until
    `LspStore::language_server_ids_for_opened_buffer` names a server (else
    `no_language_server`). `was_open` records whether a server had the file before the call.
  - `at_position`: the 1-based `line`, then `symbol` (its first byte offset on the line, else
    `no_symbol` with the line's text) or `column` (bytes, clipped to the line), as
    `editor_list` counts.
  - `definition`, `references` and `hover` run `Project::definitions`, `references` and `hover`
    raced against 10 s (`within`, else `lsp_timeout`). `location_json` gives the path, the 1-based
    start and end and the trimmed line (200 characters) through `mcp::for_agents`, or `[secret
    file]` for a file the globs match. Hover joins its blocks, masked, cut at `PAGE_BYTES`.
  - `editor_diagnostics` with a path pulls afresh (`pull_diagnostics_for_buffer`), and a file the
    call opened waits up to 3 s for its first ones; without a path it reads the files
    `diagnostic_summaries` counts errors or warnings in, for the project named, else the caller's
    (`browser_tools::caller_workspace`), else every local one, hiding secret files and counting
    them. `entries_of` takes each group's primary entry and leaves out Marley's own (source
    `marley`, #623). `page_of` pages at 200 entries or `PAGE_BYTES` of JSON with `total`,
    `start` and `next`.

## The Agent Panel's threads for agents (`src/thread_tools.rs`, #706)

- **`live`** collects every window's workspaces' `AgentPanel::conversation_views`, each
  conversation once. A thread's id is `parent_id().to_key_string()`, as the rail's rows use.
- **`list`** answers each thread's `title` (else "New Thread") and its `agent`
  (`agents::thread_agent_name`). `project` is the first visible worktree's name, and `running`
  means `ThreadStatus::Generating`. `pending` is the tool's label, with `answerable` false for a
  sandbox escalation.
- **`read`** logs the read (#703), then runs `AcpThread::to_markdown` through `for_agents` and
  `page_from`.
- **`post`** refuses `draft_in_progress` while the message editor holds text. Otherwise it calls
  `MessageEditor::insert_text`, then `ThreadView::send` in the thread's window, so a running
  thread queues the post.
- **`give_answer`** is #508's `answer_thread`: AllowOnce or RejectOnce through
  `authorize_permission_request`. A sandbox escalation is `sandbox_escalation`. A request gone by
  then is `no_pending`.
- **The area.** All four pass `agent_control::Area::Threads` (`marley.agent_control.threads`).
  List and read are Read, post is Act, and answer is Sensitive.

## The rustal-ste skill for agents (`src/rustal_skill.rs`, #725)

- **The files.** `FILES` embeds `agent_skills/rustal-ste/` (a copy of Rusty's store). `digest`
  names the plugin folder, and the Codex copy's `.marley-owned` mark holds the same digest.
- **`reconcile`** runs on start and on every settings change, as `marley.rustal_ste_skill` says.
  - On:
    - `agent_skills::register_builtin_skill` (Zed's hook, #725);
    - `install_plugin_in` (`<data>/claude-code/rustal-ste/<digest>/`, read-only, staged and
      renamed), then `identity::set_skill_plugin`;
    - `write_codex_copy_in` (`$CODEX_HOME/skills`).
  - Off: unregister, clear the plugin folder, and `remove_codex_copy_in`, only when the mark is
    there.
- **`plugin_folder`** gives the digest path for the Marley entry's `CLAUDE_CODE_PLUGIN_DIRS`
  (`assistant.rs`). Terminals take it through `identity::plugin_dirs`, after the shared plugin.

## Following Omarchy's theme (`src/omarchy_theme.rs`, `omarchy/`, #759)

- **The files.** `omarchy/marley.json.tpl` is a Zed theme family, "Omarchy", written in Omarchy's
  template language (`{{ key }}`, `{{ mix a b N% }}`, two-digit alpha suffixes); its first line is
  the marker comment. `omarchy/theme-set-hook.sh` copies the render to `omarchy.json` in Marley's
  themes folder through a temporary file, and skips a render with `{{` left. Both are embedded with
  `include_str!`.
- **`OmarchyPaths`** puts everything under `MARLEY_OMARCHY_HOME`, else the home folder:
  - the template in `.config/omarchy/themed/`;
  - the hook as `.config/omarchy/hooks/theme-set.d/marley-theme`;
  - the render at `.local/state/omarchy/current/theme/marley.json`.

  `present` wants the current theme's `colors.toml` and the config folder.
- **`reconcile`** runs at start and on each settings change. It acts only when
  `marley.omarchy_theme` differs from what the `OmarchyTheme` global last applied. While an apply
  runs, `running` holds the next one back; the apply reconciles again when it ends.
  - On, it runs `install_in`: the template while missing or marked, the hook with its two paths
    quoted by `shlex` (mode 755), and a render newer than the copy. At start it stops there.
  - Turned on later, it also:
    1. runs `omarchy-theme-refresh` (`MARLEY_OMARCHY_REFRESH`) through `process::run_program`
       when no render is there;
    2. checks the switch again;
    3. calls `select`: `load_user_theme`, `reload_theme`, the replaced selection kept as JSON in
       the key-value store (scope `marley-omarchy-theme`), and `theme` written as "Omarchy".
  - Off: the kept selection goes back while `theme` still names "Omarchy", then `remove_in`
    deletes the hook, the copy, and the template while marked.
- **At each switch** Zed's own `watch_themes` reloads `omarchy.json`; Marley takes no part.
- **Errors** go to a toast in every workspace (`show_warning`) and to the log.

## Palette actions for agents (`src/action_tools.rs`, #707)

- **Who may run what:** `ALLOWED` (37 names) and the user's
  `marley.agent_control.actions_allowed` run under `Area::Actions` (`action_run` is Act).
  `refused` is over both. It refuses a name whose namespace is in `REFUSED_NAMESPACES` or whose
  local name holds a word from `REFUSED_WORDS`, and it is checked again on the built action's
  `name()`.
- **`list`** answers the allowed actions with `cx.action_documentation()`, plus `unknown`: the
  allowlisted names `cx.all_action_names()` lacks. The scenario keeps that empty, so an upstream
  rename shows.
- **`target`:** the workspace whose visible worktree's name or path is `project`, else the active
  window's shown workspace, else the first window's.
- **`run`** builds the action with `cx.build_action`. When the target isn't shown, it activates
  that workspace and waits for `on_next_frame`. Then `dispatch_in` focuses the active pane if
  nothing is focused. It refuses `not_available` when `window.is_action_available` says so;
  otherwise it calls `window.dispatch_action`.

## Agent activity and the kill switch (`src/agent_activity.rs`, `src/mcp.rs`, `marley_mcp`, #703)

- **The gate.** `mcp::answer`, after `permits`, runs `agent_activity::gate` for a write-tier tool
  (`marley_mcp::lookup`'s `Tier::Write`) whose family `tools/list` serves. The unlisted editor
  family serves `marley-edit`, the user's `$EDITOR` (#649).
  - `gate` names the caller (`click_pause::Who`) and the summary: the first of `command`, `text`,
    `url`, `key`, `keystrokes`, `name`, `path`, else the arguments, cut to one line of 160 chars and
    redacted with `model_redactor`.
  - It sets `AppCall::on_answer`, so the outcome (done, or refused with the code) is logged however
    the tool answers.
  - While `MarleySettings.agent_control` is `Stopped` it refuses with `agent_control_stopped` and
    the step to resume.
- **`marley_mcp::AppCall::on_answer`** keeps an `AnswerHook`, a `Send` `FnOnce`. `answer` runs it
  with the converted result before sending.
- **The log.** The hook sends each `ActivityRow` on an unbounded channel. One background task first
  reads today's `<data>/agent_control/activity-<day>.jsonl` (`read_day_in`), then appends each row
  in order (`append_in`, 0600) and forwards it. A foreground task pushes rows into the
  `AgentActivity` global (500 kept), which the tab and Home's card observe.
- **The switch.** `marley.agent_control.stopped` (`MarleyAgentControlContent`, the Settings page's
  Agent Control section) is read as `AgentControl { Running, Stopped }`. `StopAgentControl`,
  `ResumeAgentControl` and the Stop/Resume buttons write it through `update_settings_file`.
- **The views.** `AgentActivityView` is an `Item` opened in the Home group by
  `OpenAgentActivity` (`open_later`). `render_state` and `render_rows` are shared with Home's
  AGENT ACTIVITY card.
- **Home's page (`home_page::ensure`).** After inserting the page at index 0, it activates the tab
  that was active. `Pane::add_item_inner` moves no active index for an item it doesn't activate,
  so the page used to take the front.

## The Rusty group's + menu and its never-empty rule (`src/rusty/home_tab.rs`, `src/groups.rs`, #699)

- **The + menu.** Zed's `pane.rs` gains `MarleyNewItemMenu`, a global function from the + menu and
  the pane's workspace to the menu, which `default_render_tab_bar_buttons`' builder starts from
  (a touchpoint row). `home_tab::init` sets it to `rusty_links`. For a Rusty group's workspace
  (`groups::is_rusty_workspace`) while Rusty is on, it adds Home (`open_later`), each
  `Screen::ALL` entry (`RustyHome::open_screen`), then Open Page… and the three captures, whose
  handlers dispatch `OpenPage`, `CaptureToToday`, `CaptureToInbox` and `CaptureUrl` as Zed's own
  `action` entries do there (the menu sets no action context). A separator comes before Zed's
  entries. Any other workspace's menu comes back unchanged.
- **Never empty.** `home_tab::fill` calls `ensure` when the workspace is the Rusty group's, holds
  no item, and Rusty is on. It runs:
  - on each workspace's own `ItemRemoved` (`subscribe_in(&cx.entity(), window, …)`);
  - through `fill_shown`, on the shown workspace, on each `MultiWorkspace`'s
    `ActiveWorkspaceChanged` and on the `Groups` global, which covers a restored group adopted by
    the rail.

  The home page is still not serialized: a restored group comes back empty and is filled when shown.

## One row for a thread in a center tab, and the keys at start (`src/rail.rs`, `src/thread_tab.rs`, #702)

- `ThreadTab::thread_key` is the conversation's `parent_id().to_key_string()`, the rail's key for
  thread rows. `member_tabs` skips a `ThreadTab`, as it skips terminals and Browser tabs;
  `active_rows` takes no tab row for an active `ThreadTab`, and `note_focus` marks the thread row
  of the active `ThreadTab` (`active_thread_tab`) before the panel's focused thread.
- **Focus after a background group.** #702's rail-side `keep_focus` was replaced in #708 by three
  Zed hunks (`zed-touchpoints.md`, the `workspace.rs` and `zed.rs` rows): a workspace added behind
  the shown one gives the focus back right after `MultiWorkspace::add`, `initialize_workspace`
  focuses a new workspace only when `marley_is_shown()`, and a workspace's focus-lost listener
  acts only for the shown one. Groups made at start (#700) and reopened after a relaunch (#601)
  leave the window's focus alone.

## New Thread with Context (`src/handoff.rs`, `src/thread_tab.rs`, #717)

- `NewThreadWithContext` (registered by `handoff::init`, which `thread_tab::init` calls) acts on
  the active thread tab, else the Agent Panel's thread, and needs one of the person's messages. The
  thread tab's menu offers it in both layouts.
- `AgentPickerDelegate` lists `agents::thread_agents` without Manager entries (seat entries are
  already out), the old thread's agent first.
- `items_of` turns the live `AcpThread`'s entries into `marley_agent::handoff::Item`s: user
  messages' markdown, assistant `Message` chunks (not `Thought`), tool calls as
  `label.read(cx).source()` and `status()` with paths from `diffs()` and `locations`. Each item and
  `to_markdown` pass through `mcp::agent_redactor`.
- `start` writes the whole conversation with `write_handoff_in(<data dir>/handoffs, …)`, mode 0600,
  on the background executor, then builds the block (`handoff::block`, `BUDGET`) and calls
  `thread_tab::open_beside`: a `ThreadSpec` with `initial: Some(ContentBlock { auto_submit: false })`,
  added to the pane right of the active one, split off it when there is none.

## Drafted messages (`src/drafts.rs`, #769)

- `mcp::answer` routes `draft_message` to `drafts::answer`, past the Write gate (logged, and refused
  while agent control is stopped). The tool answers once the tab opens: an app call has 30 seconds,
  and a draft waits on the person.
- A `terminal` target is `mcp::terminal_with_id` checked with `send_selection::target_of` (an agent
  CLI must be its foreground program, else `not_an_agent`). A `thread` target is the first
  `thread_tab::conversations_of` view, in any of `mcp::all_workspaces`, whose
  `parent_id().to_key_string()` is the id, so a thread in a tab counts as well as a panel's.
- `DraftView` is a center item in the target's workspace: a header naming `click_pause::Who`'s words
  and the target, a multi-line `Editor` with the text, Send and Drop. Drop emits `CloseItem`.
- Send reads the editor and checks the target again. A terminal goes through
  `send_selection::send_text(…, submit: true, …)` unless its seat waits; a thread gets the text
  inserted into its message editor and `ThreadView::send`, deferred, unless the person has text
  there. A refusal stays on the tab as a note; a send shows a toast and closes the tab.
- **The window being updated.** Send runs inside the draft's own window update, which
  `browser::window_of` cannot see (`cx.windows()` leaves it out), so `window_holding` falls back to
  that window's handle.

## A thread that starts in a tab (`src/thread_tab.rs`, `src/rail.rs`, Zed's `agent_ui`, #734)

- `NewAgentThread { agent, folder }` (a data action read through `NewAgentThreadFields`, as
  `rusty::OpenPage`) runs `thread_tab::start`, which the rail's New Agent Thread calls directly on
  the row's workspace. `start` takes the panel's selected agent and `default_folder` (the first
  root, else the home folder) when they are absent, checks the folder is a directory, and joins it
  to the project as a hidden worktree when `needs_worktree` says so: the project is local, no
  worktree holds it, and it neither is nor holds the home folder. Then `open_thread` builds the
  `ConversationView` as the panel does (`Agent::server`, the panel's `connection_store`,
  `ThreadStore::global` for Zed's agent, `AgentThreadSource::Sidebar`) with `work_dirs` = the folder
  and `marley_own_folders` set, and adds a `ThreadTab`.
- **The hidden worktree lives in the tab.** `WorktreeStore` keeps a strong handle only for visible
  worktrees, so the `Entity<Worktree>` `find_or_create_worktree` returns is held by
  `ThreadTab::_folder` and goes with the tab.
- **Zed's side** (`zed-touchpoints.md`): `MarleyThreadHost { shows, reveal }`, a global `init`
  sets to `shows_view` (the workspace's active item is a tab on that view) and `activate_for`;
  `ConversationView::is_visible_in_agent_panel` asks `shows`, and a notification's accept asks
  `reveal` before opening the panel; the thread store files a view with `marley_own_folders` under
  its thread's `work_dirs()` rather than the project's visible folders.
- **The rail.** `tab_homes` maps each tab's thread to its workspace; `group_threads` takes a thread
  tabbed in another group out of a group's rows, keeps a draft tabbed here, and lists a projectless
  group's tab threads from the store. `conversations_of` (the panel's views and the tabs' others)
  feeds `live_threads`, `live_statuses` and `inbox_entries`, so a tab thread has a status and its
  waits reach Needs you.

### Restored after a restart (#736)

- `ThreadTab` is a `SerializableItem` (`MarleyThreadTab`). `serialize` (on `UpdateTab`) saves a
  `SavedThreadTab` (the thread id, the agent's id, the root thread's `work_dirs()` or the record's
  folders, `marley_own_folders`, the title) as JSON in `marley_thread_tabs`, Marley's own table in
  the workspace database, after `rusty/graph_store.rs`; `SavedThreadTabs` holds them in memory from
  `init` so `deserialize` finds one at once.
- `deserialize` refuses a tab whose thread has no record (Zed shows nothing, the cleanup drops the
  row), waits up to ten seconds for the workspace's Agent Panel (Zed adds panels while it restores
  items) for its connection store, re-joins a #734 folder that needs a hidden worktree, and builds
  the view through `build_view`, which `open_thread` shares, with the record's session id, so the
  agent loads or resumes the session.

## Harnesses on other hosts (`src/harness_hosts.rs`, #740)

- `marley.harnesses` (`MarleyHarnessHostContent { name, ssh, state, rh }` in `settings_content`)
  becomes `HarnessHost`s through `from_content`: the named, rooted entries, first of each name; an
  `ssh` that `marley_remote::parse_ssh_target` refuses drops its entry with a log line.
- `HarnessHost::base` is `rh --state ROOT`, or `ssh -T -o BatchMode=yes <keepalive> [-p] -- HOST`
  and the remote words through `shell_word`; `mcp_command(writes)` adds `mcp [--grant write]`.
- The `Hosts` global holds a `Followed` per host (connection, `FleetSnapshot`, server, fold, the
  follow task). `follow_hosts`, on every settings change, keeps a host whose entry and grant are
  unchanged, restarts a changed one and drops the rest; it reads the merged settings as
  `harness_writes` does, so `MarleySettings` has no field for it.
- `harness::Slot { Primary, Host(name) }` names a harness. `follow`, `connected` and `seed` take
  it and write through `set_connection`, `change_seats` and `bump_minute`; each harness's manager
  thread is followed on its own connection (#758). `seats_of`, `server_of`, `seat_command_of` (with whether the
  words cross a remote shell, so `Seat::commands` quotes them) and `writes_on_slot` read either.
- `HarnessView` keeps its slot: its observer (`Harness` or `Hosts`), reads, answers, sends and
  controls go to that harness, and its tab title adds ` · name`.
- The rail draws `render_harness` as one `harness_section` per harness; rows carry the slot in
  their click and element ids, `InboxTarget::Harness(slot, id)` and the inbox key
  (`harness:NAME:ID:ASK` for a host, the old shape for the primary).
- After a failure `follow` leaves `Connection::Down(reason)` in place while it waits and tries
  again, so the header keeps the reason; `Connecting` is the first try only.

## A thread on a harness seat (`src/harness_seat.rs`, `src/thread_tab.rs`, #742)

- A seat's thread is a custom agent server in the settings' in-memory `agent_servers`, as the
  Manager's is (#694), named `<profile> · <harness>` (`· the harness` for the first one):
  `ensure_seat_entry` builds `seat_command_of(slot)` + `acp --seat PROFILE`, quoting the words
  that cross the host's shell and resolving a bare `ssh` or `rh` on Marley's own search path, since
  Zed runs an agent server in the project's shell environment. `SeatEntries` keeps each entry's
  command and the folders New Agent gave seats.
- `seat_of_entry` reads a name back to the harness and profile. `thread_tab::open_saved` and the
  restore call it to make the entry again and skip the worktree; `agents::thread_agents` leaves seat
  entries out of the agents it lists.
- `open_seat_thread` brings forward a tab on the entry (`thread_tab::activate_agent`) or calls
  `thread_tab::start_on_seat`: a `ThreadSpec` on the seat's folder with `own_folders`, no `is_dir`
  check and no worktree, opened one tick later so the agent server store has the entry. The folder
  is the one New Agent gave, else `harness::root_of(slot)`.
- New Agent: `Start::seat()` maps Claude Agent's thread (`claude-acp`) to `SeatView::Thread`, and
  `start_agent_on` opens the thread from `seat start`'s `profile`. The rail's `with_seat_menu` gives
  a row with a `profile` label, while its harness takes writes, **Open as Thread**.

## A manager on each harness (`src/harness.rs`, `src/thread_tab.rs`, `src/rail.rs`, #758)

- **Entries.** `manager_entry_name(slot)` is `Manager` for the first harness and
  `Manager · <name>` for a host. `sync_manager_entries`, after every `change_seats`, in
  `follow_setting` and at the end of `follow_hosts`, puts one in the in-memory `agent_servers` for
  each slot that takes writes and has a `role: manager` session, with `manager_command(slot)`:
  `seat_command_of(slot)` + `acp`, a bare program resolved through `which`. `Harness.manager_entries`
  maps each name to its command, and the defaults change only when the map does. An entry
  `ensure_manager_entry` made for a saved thread stays while its harness's connection is not
  `Connected`, since the fleet is seeded before that.
- **`manager_of_entry`** reads a name back to its slot while that harness is followed.
  `harness_seat::seat_of_entry` and `ensure_seat_entry` refuse the profile `Manager`.
- **Following.** `ManagerFollow { cursor, unread }` lives in `Harness.manager` for the first
  harness and in `Followed.manager` for a host, so it goes with the host. `follow_thread(slot)`
  runs after every slot's fleet pass; a slot without a manager or writes has its `ManagerFollow`
  reset. `manager_posted(slot)` titles its notice with the entry's name and tags it
  `marley-manager-[<name>-]<id>`. `manager_in_front(slot)` reads the entry's panel selection or
  `thread_tab::shows_agent`.
- **The rail.** `manager_entries` walks `harness::slots`, keyed `manager:<id>` or
  `manager:<name>:<id>`, under `Harness` or `Harness · <name>`; `InboxTarget::Manager(Slot)` opens
  `thread_tab::open_manager_thread` in the shown workspace. `harness_row` passes `role: manager` to
  `with_seat_menu`, which adds **Open Manager Thread**; `shown_workspace` reads the rail's window.
- **`thread_tab::open_manager_thread`**: an open tab on the entry comes forward
  (`activate_agent`); else the store's newest non-draft record on it opens (`open_saved`); else
  one tick later `retained_session` asks the harness project's connection for `session_list` and
  takes its first session, and the new tab's `ThreadSpec.session` loads it, titled with the entry.
  `thread_tab::start` sends a manager entry here, so New Agent and `marley agent-thread` do too.
- **The harness project (D6).** `harness_project_for` builds a seat's or a manager's view on the
  `HarnessProject` global: `Project::local` with no folders, from the workspace's app state, and
  its own `AgentConnectionStore`, made on first use and logged at each view. `build_view` takes it
  for those agents in every caller; `open_saved` and the restore skip the worktree for them
  (`ensure_harness_entry`).

## Agents that start agents (`src/delegation.rs`, #716)

- `mcp::answer` routes `agent_start` to `delegation::answer_start` and `agent_status` to
  `answer_status`; `agents.write` is granted at start. `checked` takes the request (`Request::of`:
  agent, prompt up to 8192 bytes, place, view, request id as a UUID), the caller from
  `mcp::caller_terminal` (the terminal's token checked, #755), `harness::writes_on`, the first
  harness's `seat_command_of` and the caller's folder. `agent_control::admit` asks with
  `Area::Agents` at `Level::Sensitive` (the setting `marley.agent_control.agents`).
- After Allow it answers `starting` at once, since the server waits 30 seconds and a seat takes up to
  a minute, and `run` goes on: the worktree through `worktree_agents::make_for_seat` for
  `worktree` (the creation half of `create`, split for it), `harness_seat::add_and_start` with the
  labels `started_by = marley:<terminal id>` and `delegation = <request id>` (`Seat::commands`
  writes a `--label` each), the view (`open_view`: `open_seat_thread`, or `attach_line` in a terminal
  that `beside_caller` moves into a split right of the caller's pane), then `send_prompt`:
  `session_send` with the request id as its delivery, sent again every 2 seconds for up to 120 while
  the seat refuses it. A failure is a notification.
- `harness::change_seats` ends with `delegation::follow`, which on the first harness runs `pass`:
  the callers' titles for the rail (`Delegations.callers`, worked out here because the rail draws
  inside its window's update, which hides that window's terminals); each seat whose `started_by`
  names a Marley terminal, whose `delivery.state` is `ended` and whose `delegation.told` is unset
  (or set by Marley and not yet published) gets `point`: the line pasted into the caller with
  `terminal_drive::paste_then` and Enter, without moving the focus, once `send_selection::target_of`
  reads it ready, then `mark_told` (`session_label`). A caller no open terminal holds for 10
  seconds is an `Orphan`. While anything waits, `watch` runs `pass` every 2 seconds.
- `answer_status` finds the seat by its `delegation` label, reads its last 40 lines with
  `session_read` (redacted with `mcp::agent_redactor`) and answers the state, the delivery's state
  and outcome, and `told`, marking an ended one told.
- The rail: `harness_row` adds "for <caller>" under a delegated seat; `harness_entries` lists each
  orphan as an `InboxTarget::Delegation`, which opens the seat and marks it told; the rail observes
  `Delegations`. `harness::write_tool` and `read_tool` are the crate's calls to a harness tool from
  outside a session's tab.

## Scheduled runs (`src/schedule_runs.rs`, #719)

- rustal-harness keeps the tasks, runs each in a session it opens from the task's profile and stops
  after the run, and records the outcome (TICKET-118, its `docs/SCHEDULE.md`); Marley builds no
  timer. The global `Schedules` holds, per `Slot`, the tasks from the last `schedule_tasks` read and
  what Marley did about the findings: each one's first line, the ones being read, the ones given a
  notice or waiting for their line to get it, and the ones the user opened.
- `harness::connected` calls `schedule_runs::poll` after `follow_thread`; it reads `schedule_tasks`
  through `harness::read_tool` once 15 seconds have passed. An error, an unknown tool among them,
  leaves no tasks: a harness without the scheduler shows none.
- `harness::change_seats` ends with `schedule_runs::follow`: each session labelled
  `schedule.outcome: finding` without `finding.seen` is a finding. A finding whose first line is
  unknown is read with `schedule_run`; one whose session last changed after `Schedules` was made
  (Marley's first connection) gets one desktop notice, tagged by run, once its line is read.
- The rail: `harness_section` leaves out sessions labelled `schedule.run` and adds `schedule_row`
  per task after the sessions (the schedule and the next run, or "paused"; the last run's outcome
  and first line, colored by `outcome_of`); a click opens `ScheduleView` in the Home group
  (`open_history`). `finding_entries` lists each finding as an `InboxTarget::Finding`, which opens
  the run's session and calls `mark_seen`: the run is seen locally at once, and `session_label`
  sets `finding.seen` (a first segment the harness does not reserve; it reserves `schedule.`). The
  rail observes `Schedules`.
- `ScheduleView` reads `schedule_runs {task, limit: 32}` when it opens and again when the task's
  `last_run` changes; each row, newest first, shows the run's local time, its outcome and its first
  line, and a click opens the run's session with `harness::open_in_home`.

## A new agent on a harness (`src/harness_seat.rs`, `src/agents.rs`, #741)

- The New Agent picker's `Place::Harness { slot, name }` follows Browse…, one per
  `harness_seat::seat_slots`, listed only for a CLI choice `Start::seat_kind` maps to a
  `SeatAgent`. Choosing it sets the delegate's `remote`; the picker's query is then the folder,
  one entry, confirmed only when it starts with `/`.
- `harness_seat::start_agent_on` shows a toast, then `add_and_start` runs `seat add` and `seat
  start` through `seat_command_of(slot)` under `seat_stem`'s name (`claude-other`), skipping the
  slot's session titles and moving on at `seat_exists`, up to `-9`. `attach_line` takes `seat
  start`'s `tmux` view, else `native`, and the line is typed into a terminal of the workspace in
  its default folder (`agents::start_in_terminal`). Any refusal is `not_started`'s notification.
- A view's line comes from `harness::view_command(slot, argv)` →
  `harness_hosts::terminal_line`: the argv through `shell_word`, and for a host over ssh, `ssh -t`
  with the keepalive and connect timeout, `--`, the destination and that line as one remote word,
  every word quoted again for the local shell. `HarnessView::surface` keeps these lines.

## Buttons at the right of the status bar (`src/status_buttons.rs`, #739)

- `RustalButtons`, a `Render + StatusItemView` with its workspace's weak handle, is added to every
  workspace's status bar by `init` (`observe_new` + `add_right_item`), so no line of `crates/zed`
  changes; it lands just left of Zed's dock buttons. It observes the settings, `rusty::Rusty` and
  `assistant::Assistant`, and draws Home always, Rusty while `rusty::is_on` and `is_connected`,
  Threads while AI is on, and Marley while `assistant::marley_present`.
- The clicks, plain closures over the weak workspace (PR-claude-701): `home_page::open_here`,
  `rusty::open_home_here` (`home_tab::open_here`), `threads_page::open`,
  `assistant::talk_to_marley`. Each `open_here` brings the workspace's page forward or adds one to
  its center. `hide_setting` is `None`: each button follows its feature's own switch.
- The rail's `render_rusty_button` went, with the re-export of `OpenHome` and `open_later` it used.

## The Threads page (`src/threads_page.rs`, #737)

- `OpenThreads` (`marley: open threads`) brings the workspace's `ThreadsPage` forward or adds one to
  its center. The page observes Zed's `ThreadMetadataStore` and its search editor, and redraws on
  either.
- `sections` takes the store's entries (drafts left out; the entries hold the archived ones, each
  once, #754), newest `updated_at` first, keeps those whose title or folder paths hold the search
  (case-insensitive) and whose agent is the chip's, and splits them: Marley and Rusty
  (`assistant::is_assistant`), All conversations, Archived (shown on its button). The chips list
  every agent the store holds.
- **CLI sessions (#754).** `Row` names its `Target` (`Thread { thread_id, agent }` or
  `Cli(Arc<CliSession>)`), the moment it sorts by, and a search snippet; `icon`, `detail` and
  `key` draw it for the page, the rail's Threads view and Reopen Thread…, and `open_row` opens
  either. `cli_rows` makes rows of `cli_sessions::listed`, leaving out a session whose id a store
  entry holds as its session id; `conversations` (unarchived conversations, as its doc says, and
  CLI rows by title and folder) feeds the rail and the picker, and `sections` adds the CLI rows to
  All conversations by time, with a `Chip::Cli` per CLI (`Claude Code CLI`, `Codex CLI`) and the
  prompts searched (`CliSearch::Prompts`, the snippet through `mcp::for_agents`). The page
  observes the `CliSessions` global; opening it, the rail's Threads view and Reopen Thread… call
  `cli_sessions::refresh`.

## Past CLI sessions (`src/cli_sessions.rs`, #754)

- `refresh` scans off the main thread, unless one runs or the last is under 30 s old, with
  `marley.list_cli_sessions` on: `scan_in(claude_dir, codex_dir, cache)` lists Claude Code's
  `<config>/projects/*/*.jsonl` and Codex's `<home>/sessions/YYYY/MM/DD/rollout-*.jsonl`, the newest
  500 per CLI by modification time, reads each file whole, or its first MiB and last 256 KiB
  (`Windows`), and parses it with `marley_agent::cli_sessions`; a file the cache holds at the same
  size and time is not read again. Codex's titles come from `session_index.jsonl` each scan.
  `CliSessions` holds the list, newest first, and the cache between scans; `listed` gives nothing
  while the switch is off.
- `reopen(workspace, session, window)`: `agent_events::view_running` (a seat of the session's CLI,
  not done, whose `session_id` or `thread` label is the session's) finds a terminal among the
  window's workspaces, which `browser::reveal_terminal` brings forward. Otherwise the folder is
  checked off the main thread, and `agents::resume_input` (the resume line with the project's
  permission arguments, and a Codex App Server through `Joining` when #650's switch is on) goes to
  `start_in_terminal` in that folder; a missing folder or an id the CLI would not take is a toast.
- A row's click (a plain closure over the page's weak workspace, PR-claude-701) runs
  `open_thread`: a tab with the thread anywhere in the window comes forward (its workspace shown,
  `thread_tab::activate_for`); a panel showing or retaining it shows it; else
  `thread_tab::open_saved` opens it in a tab of the page's workspace, as #736 restores one.

## An agent thread in a center tab (`src/thread_tab.rs`, `src/rail.rs`, #697)

- `ThreadTab` (an `Item`, titled as the thread, `IconName::ZedAssistant`) hosts the Agent Panel's
  own `ConversationView` entity, so the thread keeps running and its history while it sits in the
  center. Zed's test-only `ThreadViewItem` is the pattern; no Zed path is touched.
  - It observes the view (title changes re-render the tab) and subscribes to the panel: on
    `ActiveViewChanged`, when the panel shows its view again, it emits `ItemEvent::CloseItem`, so a
    view is never drawn in two places.
  - Its `Focusable` is the active thread's `message_editor`, which the panel focuses on activation
    (`activation_focus_handle` is `pub(crate)` in `agent_ui`); the view's own handle takes no typing.
  - `tab_extra_context_menu_actions` gives the tab "Move Thread to Panel".
- `OpenThreadInCenter` (`marley: open thread in center`) runs `open_in_center`. It refuses with a
  toast when the window has no panel, no active thread, or an empty draft, and that check comes
  before `clear_base_view`. A tab already showing the thread comes forward (`activate_for`).
  Otherwise the panel `clear_base_view`s (it keeps the thread retained), and the tab goes in with
  `add_item_to_center`.
- `MoveThreadToPanel` runs `move_to_panel` on the active center item: `activate_retained_thread`
  when the panel still retains the thread, else `load_agent_thread` from its history; the tab
  closes, and the panel takes focus.
- The rail: `show_thread` first brings a tab showing the thread forward (`activate_for`), and the
  thread row's menu gains Open in Center (`open_listed_thread(key, center)`), which opens the
  thread as a click does, then runs `open_thread_in_center` in that workspace.
- The tab comes back after a restart (#736, above).

## The Rusty home page (`src/rusty/home_tab.rs`, #679)

Since #782 this is `RustyCards`, a view `MarleyHome` holds while Rusty is on, not an `Item`: it
takes Home's focus handle (the forms dispatch from Home's node), draws one RUSTY card with the
reason while Rusty is not connected (`not_connected`: "Rusty is still connecting." while
starting, else `rusty::unavailable`), and reads again whenever Rusty connects again, not only
after a failed read. `OpenHome` runs `home_page::open_here` in the shown group with no readiness
toast; `open_here`, `home_of` and `rusty::open_home_here` went, and so did the status bar's Rusty
button (`HomeButton` draws Home alone, made in place with no observer). The `rusty` verb with no
screen opens Home after `rusty_unavailable`.

- `RustyHome` (an `Item`, title Rusty, `rusty::RUSTY_ICON`, a placeholder) is the Rusty group's
  first tab: `rusty::in_rusty_group` calls `home_tab::ensure` before every open there, which adds it
  at index 0 of the active pane when the workspace has none. `OpenHome` (`rusty: open home`) and the
  rail's one Rusty button (`Rail::render_rusty_button`, in place of #672's row, its fit and `…`)
  open it.
- Its cards: every `brain::Screen`'s `Button` (through `brain::open_screen`); the page picker's
  recents (`page_picker::recent_slugs`), opening in the Brain tab (`brain_tab::open_page_later`,
  which defers `show_page` past the workspace's update); `brain_due`'s decisions, opening their
  page there; every list's open tasks (`list_task_groups`, then `list_tasks` per list, as the
  Knowledge panel's project view reads them), opening Tasks on the list. It reads on open, on
  `Announced` and on connecting after a failed read, one read at a time.

## The Brain tab (`src/rusty/brain_tab.rs`, `src/rusty/brain.rs`, `src/rail.rs`, #678)

- `BrainTab` (an `Item`, one per window, opened in the Rusty group by `brain_tab::open_later`,
  which `ToggleBrainView` and the header's Brain button call) hosts the `BrainView` in a 280px
  column and one `PageView` on its right, made by the first `show_page` and `navigate`d after, so
  its back and forward walk what the tab showed. `act_as_type` hands out the page, and
  `page::page_in` (a Page tab's page, or the Brain tab's) is what the Knowledge panel, the Graph
  tab and the page picker read as the page in front; the tab passes the page's `UpdateTab` on so
  they follow. `open_today_later` opens it and runs `BrainView::today`.
- `BrainView` takes its host: `open` defers to `BrainTab::show_page`; Open in New Tab is the old
  `open_page` (a Page tab in the Rusty group); New Page sits after its search field.
- The rail keeps only `BrainSide::connected` for the header: the Brain view, `RailView`, the
  Projects/Brain switch, the header's New Page, the body swap and the focus hand-off went.
  `Screen::Brain` leads `Screen::ALL`, and `screens_that_fit` counts the screens alone.
- `groups::make` keeps a Rusty group's name Rusty in every window; other names stay apart (#600).

## Home takes what belongs to no project (`src/groups.rs`, #676)

- #675's routing is `groups::in_group(kind, asked_from, window, cx, open)` and
  `groups::with_group(kind, …)` for `GroupKind::Home` and `GroupKind::Rusty` alike (found by
  `group_of_kind`; `Groups::waiting` keys on window and kind). `rusty::in_rusty_group` calls it with
  Rusty. `agent_tab::open_later`, the `OpenSystemOneCalls` action and `Rail::open_harness` call it
  with Home, and `Rail::in_home` (the empty-space menu) makes Home through `with_group`, so every
  path shares the guard against two Home groups.

## `marley <command>` (`src/commands.rs`, Zed's `main.rs` and `open_listener.rs`, #749)

- The launch side: `args_if_invoked`, called in `main` after the browser relay's intercept and
  before `Args::parse`, reads an optional leading `--user-data-dir`, the verb and its arguments
  (`Command::read`: flags `--name=value` or `--name value`, fixed lists for `talk`, `rusty`,
  `panel` and `agent-cli`'s provider, folders canonicalized against the working directory,
  `browser` through `marley_browser::address`). It answers `Invoked::Exit { code, message }`
  (`help`, wrong arguments; `main` prints it) or `Invoked::Launch` (the binary, the data
  directory, else `MARLEY_USER_DATA_DIR`, and one `zed://marley/<verb>?folder=…&…` URL of at most
  1024 bytes), which `main` hands to `Args::parse_from`. The URL then rides #513's hand-off;
  `single_instance::hand_off` names the verb in its line.
- The app side: `OpenRequest::parse` makes `OpenRequestKind::MarleyCommand` from the prefix
  (`commands::command_in`); `handle_open_request` calls `commands::run`; a cold start's first
  request, when it is a command, restores the session first. `run` finds the group whose project
  has a visible root holding the folder (the deepest), else Default (`groups::with_group`) or, in
  Zed's layout, the shown workspace; shows it with `MultiWorkspace::activate`; waits up to 10 s for
  the Agent Panel or the named panel; then runs the verb through the existing openers
  (`thread_tab::start`, `assistant::talk_by_name`, `agents::start_cli_in`,
  `routing::open_center_terminal`, `threads_page::open`, `home_page::open_here`,
  `rusty::brain::open_screen`, `rusty::open_home_here`, `browser::open_url_tab`) or `open_panel`,
  which activates, opens and focuses the panel's dock for #747's observer. A refusal is a toast.
- The agent started last is kept in the key-value store (`marley-commands`): by
  `thread_tab::start`, `agents::start_cli_in`, `agents::start_cli_with_prompt` and the verbs.
- A running Marley gets the verb from the intercept itself (#775): `hand_to_running` reads the
  data directory without initializing `paths` (a custom one canonicalized, else
  `paths::data_dir()`) and `single_instance::hand_verb` sends the URL, answering `None` when no
  socket listens, so the launch prints only the hand-off's line; only then does it answer
  `Launch`. Both are Linux and FreeBSD only, `hand_verb` too since #786; on macOS the launch
  answers `Launch`.
- `commands::note_data_dir` (in `init`) notes `paths::data_dir()` when the launch named
  `--user-data-dir`; `marley_terminal::identity::agent_environment` writes it as
  `MARLEY_USER_DATA_DIR`, empty in any other terminal.

## The Debug button at the left (`src/status_buttons.rs`, `src/marley_workbench.rs`, #774)

- `apply_defaults` sets `debugger.button` with `terminal.button`: false in the Marley layout,
  Zed's own default (kept in `LayoutState.zed_debug_button`) in Zed's, so the bottom dock's
  `PanelButtons` drops Debug in the Marley layout only.
- `status_buttons::init`'s `observe_new` inserts a `DebugButton` after the first
  `PanelButtons` of the status bar, the left dock's, which `Workspace::new` adds before any other
  item. The item keeps the workspace's three docks and observes them and the settings.
- It finds the panel with `Dock::panel_for_persistent_name("DebugPanel")` and draws only in the
  Marley layout, while the panel's own `icon` is `None` (Zed's button hidden) and the user's own
  `debugger.button` is not false. Its look and click follow `PanelButtons`: toggled while the
  dock is open on the panel; a click focuses the dock and dispatches the dock's `toggle_action`
  then, else the panel's. `hide_setting` answers the panel's `hide_button_setting`, so the status
  bar draws Zed's Hide Button.

## A split opens Home (`src/pane_splits.rs`, `src/home_page.rs`, Zed's `pane.rs`, #785)

- `pane_splits::init` registers capture listeners at each workspace's root for
  `pane::SplitLeft/Right/Up/Down/Horizontal/Vertical`, beside #747's `move_split`.
  `split_to_home` acts only in the Marley layout, in `ClonePane` mode, on the center pane that
  holds the focus (`Workspace::panes`, `contains_focused`) when its front tab is a
  `TerminalView`: it stops the action, opens an empty pane (`split_pane`) and puts a new Home in
  it with the focus, in place of the shell `TerminalView::clone_on_split` starts. The pane is
  found by the focus, not `active_pane`, because the split button's menu focuses its own pane
  before the workspace's active pane follows. A dock's pane is never in `panes`, so the terminal
  dock splits its own way.
- `MarleyHome` answers `can_split` with true and `clone_on_split` with a new Home of the same
  workspace, built from the `fs` the page keeps: Zed calls `clone_on_split` from
  `handle_pane_event`, while the workspace is being updated, and reading it there panicked.
  `home_page::new_page` builds a page for `ensure`, `open_here` and the splits.
- The tab menu's hook, `workspace::pane::MarleyTabMenu`, is set to `tab_menu`, which adds
  Split Right, Left, Up and Down after the item's own actions, in the Marley layout and for a
  center pane only, last so End reaches them. `split_out` activates the tab and runs
  `split_and_move`; on a pane's only tab it opens the new pane on the opposite side
  (`split_pane`), as Zed's `MovePane` does, puts Home there and gives the tab back the focus.
- The split button's menu in Zed's `default_render_tab_bar_buttons` now has the pane's focus as
  its context: without one, a choice never reached the pane, so Split Right did nothing on a file
  or Home; a terminal had split only through the capture listener at the root.
- `script/e2e/785-a-split-opens-home.sh` splits a terminal from the button and the palette, Home
  and a file from the button, and moves a tab through its menu, checking that no second terminal
  starts.

## Panel tabs after a restart (`src/panel_tabs.rs`, Zed's `dock.rs` and `outline_panel.rs`, #773)

- `PanelTab` is a `SerializableItem` of kind `MarleyPanelTab`, registered in `panel_tabs::init`.
  Its row, in its own table `marley_panel_tabs` (`MarleyPanelTabsDb`), is the workspace id, the
  item id and the panel's persistent name; `init` reads every row into the `SavedPanelTabs` global,
  and a restore reads the global. Zed writes the row when the tab joins a pane;
  `should_serialize` answers false.
- `deserialize` refuses at once in Zed's layout, for an item with no row and for a name outside
  `MOVED`; otherwise its task looks every 100 ms, up to 10 s, for the panel of that name in the
  workspace's docks (`Dock::panel_for_persistent_name`, a Zed hunk), then refuses a disabled panel
  or one the group already shows in a tab. `cleanup` drops the rows of items no longer alive.
- `PanelTab::new` builds the tab for `show_tab` and for the restore alike.
- `dock_changed` re-tells both the dock's previous panel and the one just asked for that their tab
  is in front: `activate_panel` tells the previous one it is inactive.
- `outline_panel::marley_active_editor` takes the panel's entity id: while no dock shows the
  panel, any active item that is not an editor stands for the editor used last.

## More side panels as tabs (`src/panel_tabs.rs`, `src/fleet.rs`, `src/rusty/knowledge_panel.rs`, #748)

- `MOVED` adds `CollabPanel`, `MarleyFleetPanel`, `MarleyKnowledgePanel` and
  `MarleyContainersPanel`; the Marley layout's right dock opens only on the Agent Panel.
- `panel_tabs::init`'s settings observer runs `close_tabs` with "leaving the Marley layout, or the
  panel is not `enabled`", so Knowledge's tab closes while Rusty is off and Containers' while it
  is hidden. `shows_panel::<P>` answers whether an item is a tab of a `P`.
- `fleet::fleet_shows` counts a Fleet tab in front of a pane as a place Fleet shows, which keeps its
  polling loop going (`FleetPanel`'s render starts it).
- `KnowledgePanel::follow_active_item` keeps its page when the active item is its own tab;
  `set_active` acts only on a change, since the tab and the dock may both say so.

## Side panels as tabs (`src/panel_tabs.rs`, Zed's `outline_panel.rs`, #747)

- `MOVED` names the panels a tab draws, by persistent name (`Project Panel`, `GitPanel`,
  `Outline Panel`; #748 adds more). A moved panel stays registered in its dock, so Zed's code still
  finds it, and the dock never draws it.
- `PanelTab` is an `Item` over the panel's `Arc<dyn PanelHandle>`: the button's tooltip and icon
  name it; it keeps the panel's focus handle (a panel asks for the active item's focus handle
  inside its own update, so answering must not read the panel); it draws `panel.to_any()` and,
  over it, a transparent drop layer for a dragged tab that hands the drop to its pane's
  `handle_tab_drop` at the pane's `drag_split_direction` (the panels block the mouse over their
  empty space, which hides the pane's own drop target). Its first render in front defers
  `set_active(true)`; `deactivated` tells `false`. A tab dropped in another group closes.
- One observer per dock (`dock_changed`, installed from `observe_new`): in the Marley layout an
  open dock whose active panel is moved goes back to the panel it showed before (still open) or
  closes; when the panel holds the focus, as `focus_panel`, `ToggleFocus` and Reveal leave it,
  `show_tab` brings the panel's tab forward anywhere in the workspace or opens one in the active
  pane. `retell` then gives the panel back `set_active(true)` while its tab is in front, since the
  dock's `set_open` and `activate_panel` told it otherwise.
- The split keys (`pane::Split{Left,Right,Up,Down}`) are caught in the capture phase: on a panel
  tab, a `ClonePane` split becomes `MovePane`. Switching to Zed's layout closes every panel tab.
- `outline_panel::marley_active_editor` (a Zed hunk): an active item whose focus handle is the
  panel's own stands for the editor used last, at both of the panel's reads of the active editor
  (#773 widens it to any active item that is not an editor).

## Marley and Rusty in the title bar (`src/status_buttons.rs`, Zed's `title_bar.rs`, #746)

- `title_bar::MarleyTitleBarButtons` (a Zed hunk, its ledger row extended) holds a function from a
  title bar's workspace to an `AnyView`; `TitleBar::new` asks it once and draws the view after the
  `?`. `status_buttons::init` sets it to a `TitleButtons` view: Rusty then Marley, #739's rules,
  tooltips and handlers, observing the settings, `rusty::Rusty` and `assistant::Assistant`.
- The status bar keeps `HomeButton` alone, with no observer.
- #778 (Chad, 2026-10-12: the top right Rusty button opens the Rusty agent, the home page goes
  back to the bottom): the title bar's Rusty runs `assistant::talk_to_rusty` (Talk to Rusty, its
  tooltip naming the action) and shows while `assistant::rusty_present`, Rusty's entry, as Marley's
  shows on `marley_present`. `HomeButton::new` observes the settings and `rusty::Rusty`, and the
  view draws Home, then #739's Rusty button (on and connected, `rusty::open_home_here`).

## Threads in the rail (`src/rail.rs`, `src/threads_page.rs`, `src/home_page.rs`, `src/status_buttons.rs`, #745)

- `Rail::threads: ThreadsSide { shown, search, … }`, the view's state outside `Rail::new`'s body.
  `render_view_switch` draws PROJECTS and THREADS as `Button`s (XSmall, the shown one toggled);
  PROJECTS takes an accent `Indicator::dot()` while Threads shows and
  `marley_rail::has_attention` holds. `show_view` flips the view and moves the focus (the search
  field, or the rail); `toggle_thread_history` answers `sidebar::ToggleThreadHistory`, bound to
  `ctrl-g` in the Marley keymap's `MarleyRail && !Picker` block.
- `render` draws `render_threads_search` and `render_thread_list` in place of the filter and the
  rows (the inbox stays); the header hides Add Project and the empty-space menu. The list reads
  `threads_page::conversations(query, cx)` (unarchived, not drafts, newest first, matching title or
  folder; shared with Reopen Thread…) as `row_card`s, each click deferring
  `threads_page::open_thread` on the shown workspace (#772's path). The store's observer notifies
  the rail while Threads shows, since `refresh` notifies only for a changed snapshot.
- While Threads shows, `move_cursor`, `fold` and `confirm` return at once; `focus_filter` focuses
  the search field; `cancel` clears it, then gives the focus back to the rail.
- `home_page::render_agents` adds a Threads button (`threads_page::open`) after New Agent…;
  `status_buttons` lost its Threads button.

## Reopen Thread…, and threads in tabs (`src/thread_tab.rs`, `src/threads_page.rs`, `src/rail.rs`, #772)

- `thread_tab::show_here(workspace, thread_id, …) -> bool` is the Marley layout's one way to show a
  thread: its tab forward (`activate_for`); else, when the workspace's Agent Panel holds it, a kept
  thread made the panel's view with `activate_retained_thread(id, false, …)` and the view moved
  into a new `ThreadTab` after `clear_base_view`, as `open_in_center` moves the active one; else
  `false`. A draft stays in the panel.
- Its callers: `threads_page::open_thread`'s panel branch (Marley layout), `rail::show_thread`
  (then the store's record through `open_saved`, unarchived first, and only with no record the
  panel's load), `MarleyThreadHost.reveal` (`reveal_for`: `show_here` in the Marley layout,
  `activate_for` in Zed's). `Rail::open_manager_thread` starts the Manager with `thread_tab::start`;
  `agents::start_thread` went.
- `threads_page::ReopenThread` opens `ReopenThreadPicker` (`Picker::uniform_list`): the store's
  unarchived, non-draft records newest first (`conversations`, sharing `row_of` with the page),
  fuzzy over "title folder", two-line rows; confirm dismisses and defers `open_thread`. The rail's
  `+` (`Rail::reopen_thread`) and the pane's `+` (`agents::new_item_entries`) list it under New
  Agent….
- In the Marley layout `ThreadTab::tab_extra_context_menu_actions` is empty and
  `thread_tab::filter_palette` hides `MoveThreadToPanel` and `OpenThreadInCenter` (rechecked on
  every settings change); the rail's row menu has no Open in Center. `harness::manager_in_front`
  also counts a thread tab on the Manager entry (`thread_tab::shows_agent`).

## One New Agent… (`src/agents.rs`, `src/rail.rs`, `src/thread_tab.rs`, #744)

- `agents::places_for` puts `workspace.root_paths(cx).first()` first, noted "this project", and
  #735's `guess` after it unless it is the same folder; with no root the guess leads as before.
- `agents::init` (`&mut App`) registers capture listeners on every workspace's root for
  `agent::{ToggleFocus, Toggle, FocusAgent}` (`zed_actions::assistant`) and
  `agent_ui::{NewThread, ToggleNewThreadMenu}`, one generic `picker_in_place_of::<A>`: in the
  Marley layout it stops propagation and runs `show_picker`; in Zed's it does nothing, so the panel
  gets the action. The status bar's sparkle is Zed's `PanelButtons` entry, whose click dispatches
  `ToggleFocus` from the dock inside the workspace, so the capture sees it. A key from the rail
  reaches none of them: the rail is the `MultiWorkspace`'s sidebar, outside the workspace's element.
- #781: the same renderer captures `zed_actions::agent::OpenSettings` (a model picker's Configure,
  the panel onboarding's Configure Providers) with `providers_in_place_of_panel`: in the Marley
  layout it stops it and dispatches `llm_providers_page()` (`OpenSettingsAt { path: "llm_providers"
  }`), since Zed's handler focuses the Agent Panel in its dock first. `ThreadTab::render` puts
  `no_model_callout` above a Zed Agent thread while `LanguageModelRegistry::default_model()` is
  none (the panel's new-user card, which a tab lacks), redrawn by an observation of the registry.
- `agents::init` sets `MarleyNewItemMenu` (Zed's `pane.rs` hook, #699) to `new_item_entries`:
  New Agent… (with `marley::NewAgent`'s key shown; the entry's handler alone runs) and a separator
  ahead of Zed's entries while AI is on.
- `Rail::render_project_menu` lists New Terminal, New Browser Tab, New Agent…, New Agent in
  Worktree and the launch configs; `agent_menu`, `agent_cli_entries` and `new_agent_thread` went.
  The empty-space menu keeps its CLI entries. `agent_server_subscriptions` stays: thread rows take
  their icons from each project's agent-server store (`agents::thread_icon`).
- `thread_tab::init` hides `NewAgentThread` from the palette (`CommandPaletteFilter`) in each new
  workspace's setup; the action and its keys stay.

## No Rusty group; Home is Default (`src/groups.rs`, `src/rail.rs`, `src/rusty.rs`, `src/rusty/home_tab.rs`, #743)

The sections on the Rusty group (#675), its + menu (#699) and the home page's place in it (#679)
below describe what #743 retired.

- `GroupKind` is `{ Named, Home }`. The Home kind's display name is `DEFAULT` ("Default") in
  `make`, `claim` and `adopt`, which renames a record saved as "Home"; the kind, `is_home` and the
  record's `home` field keep their names, so #700's start rules and saved records carry over.
- `Group::rusty` and `SavedGroup::rusty` stay only to find an earlier build's Rusty group. Such a
  record is adopted and reopened as any group is (#601); `Rail::refresh` then calls
  `retire_rusty_groups`, which lists them (`groups::rusty_groups`), skips those already closing
  (`StartupGroups::retiring`), and defers. A hidden one closes at once; a shown one waits until the
  window holds Default or is making it (`groups::has_kind`), since a Default made while the
  restored one is still on its way would be a second, then shows Default through `with_group` and
  closes. `close_retired` runs `MultiWorkspace::remove(.., RemovalIntent::CloseProject, ..)`:
  closed, the group is forgotten; refused or failed, `groups::demote` keeps it as a named group
  called Rusty.
- `groups::in_shown(asked_from, window, cx, open)` defers, then runs `open` in the window's shown
  workspace in the Marley layout, else in `asked_from`. `rusty::in_shown_group` wraps it, and every
  Rusty opener that called `in_rusty_group` calls it, so a Rusty tab lands in the group on screen.
- `home_tab`: `fill`, `ensure` and `rusty_links` go, and `init` no longer sets
  `MarleyNewItemMenu`, which now has no user. `open_later` runs `open_here` through
  `in_shown_group`. The RUSTY card gains `render_forms`: Open Page…, Capture to Today…, Capture to
  Inbox… and Capture a URL…, each dispatched from the page's focus handle, deferred.
- `ensure_groups` makes Home alone; `rusty_changed` follows only the connection.

## The Rusty group (`src/groups.rs`, `src/rusty.rs`, `src/rail.rs`, #675)

- `groups::GroupKind { Named, Home, Rusty }` is what `make` makes; `Group::rusty` and
  `SavedGroup::rusty` (serde default) keep it across a restart. `rusty_group` finds a window's,
  `is_rusty` asks of a group id, and `with_rusty_group` runs an open in it, making it first; opens
  asked for while it is being made wait in `Groups::waiting_for_rusty` by window, so two quick
  opens make one group.
- `rusty::in_rusty_group(asked_from, window, cx, open)` defers, then in the Marley layout shows the
  window's Rusty group (`MultiWorkspace::activate`) and runs `open` in its workspace; in the Zed
  layout it runs `open` in `asked_from`. Every Rusty tab's `open_later` and `page::open_later` and
  `open_at_heading_later` go through it, and each `rusty: open …` action calls its `open_later`, so
  the header's buttons, the palette, the tree, the picker, the Knowledge panel and capture all land
  there.
- The rail: `header_icon` draws `RUSTY_GROUP_ICON` (a placeholder, `BookCopy`) for it; its menu
  is a single label; `rail_groups` leaves it out while Rusty is off, and `leave_rusty_group` moves a
  window showing it to its first open group when Rusty turns off.

## Every center tab in the rail (`src/rail.rs`, `marley_rail`, #674)

- **The model** (`marley_rail`). `TabKind { File, Other }` and `TabSnapshot { id, title, folder,
  kind, dirty, matched }` in `ProjectSnapshot::tabs`, with `files_open`; `Focus::tab`;
  `Selection::{Tab, Files}`; `Row::{Tab(TabRow), Files(FilesRow)}`. `walk` lists Other tabs after
  the Browser tabs, and the Files row (over any file that shows) and its files (while open, or
  under a filter) after the threads, before the ports. `selection` tries the focused tab, then
  its project's Files row, so a file folded away highlights Files. `parent` puts a file under
  Files and Files under its project; `cycle_row` reaches tabs; `run` gives them no drag.
- **The rail.** `member_tabs` reads each pane's items in the panes' order, skipping terminals and
  Browser tabs: a file is an item with a project path and a single buffer (`ItemBufferKind`), its
  folder the path's parent. `TabEntry` keeps the workspace, the item weakly and its `tab_icon`.
  `follow_panes` subscribes to every pane of every workspace, since a title or dirty change
  outside the active pane reaches no workspace event (`Workspace::handle_pane_event`).
  `active_rows` returns the active item's id when it is neither a terminal nor a Browser tab.
  `files_folded` (by header place, in memory) feeds `note_files_open`; `toggle_files`,
  `activate_tab` and `close_tab` (through the pane's `close_item_by_id`, which asks for a dirty
  file) serve the rows; `open_row` takes `&mut self` for the fold. `render_tab_row` indents a file
  a step further than an Other tab; `render_files_row` is a `row_frame` with a `Disclosure`.

## The rail's attention order (`src/rail.rs`, #542)

- `terminal_snapshot` fills `reporting` from the view's #519 seat: `Stale` when it works and
  `marley_fleet::is_stale` passes `no_update_after_minutes`, else `Events`; no seat is `Timer`.
- The rail's root `on_hover` stores `marley_rail::held_order` of the current snapshot when the
  pointer enters and clears it, then refreshes, when it leaves; `refresh` copies
  `marley.rail_order` and the hold into the snapshot before the change check, so a settings change
  or the pointer leaving redraws in the new order.
- `project_name` draws a collapsed project's summary in XSmall muted text after its name.
- Since #671 `marley.rail_order` defaults to `"window"` (`default.json`, both enums' `#[default]`):
  an agent's output reorders nothing unless the user picks `"attention"`. The summary still shows.

## Claude Code's hook events (`src/agent_events.rs`, #519)

- `AgentEvents`, a global made at the first frame, holds one `marley_fleet::FleetSnapshot`: a
  seat per terminal view whose Claude Code has sent an event, keyed by the view's entity id as
  `terminal_list` gives it. `seat(view)` gives the seat until its session ends, and
  `waiting(view)` says whether it waits on the user (#508).
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
  a third line is 3.5 rem tall; every other row keeps `h_11`. (Since #796 a terminal row is
  never shorter than 3.5 rem: it pads its lines to two.) A row without a seat keeps the
  quiet timer's reading.
- **The stop kind** (#566). After each fold, `after_fold` runs the use when its mode is not off:
  on a lead `Stop`, or the interrupt that ended a running turn, `ask_stop_kind` reads the seat's
  message and `turn` facts through `marley_agent::stop_kind::rules`. The project and its folders
  come from the view's workspace, which no update holds while a terminal's event reaches the view.
  A verdict the rules settle goes to `system_one::record` as a `rules` row; `Open` goes to
  `system_one::ask` with `STOP_KIND[parts]`, the prompt's parts in the state when the project
  sends text. In `suggest` or `act` the answer lands through `land_stop_kind`, an `Upsert` at the
  seat's own time, only while the seat is idle on the same `prompt_id` with no stop since, since an
  `Upsert` would bring back a forgotten seat. `shadow` lands nothing, as `fleet_snapshot`
  publishes every label to agents. Since #569 the landing is `land_labels(session, holds,
  labels)`, which the stall kind lands through too, each with its own `holds`.
- The global keeps each seat's last stop that read something, with its session, and the user's
  next prompt in that session logs an outcome through `system_one::outcome`: `next prompt after
  12 s, 48 characters`, and `within a minute of asks you` for a stop that asked or was blocked.
  A new session, `end` and `forget` drop it. `stop_kind_shown` gives the rail the mode as a
  `StopKindShown`, and the rail's settings observer refreshes the rows, so a new mode shows at once.

## The shared Claude Code plugin (`src/shared_plugin.rs`, `marley_terminal::identity`, #709)

- `claude_shared_plugin/` carries rustal-harness's plugin files unchanged:
  - at first, its TICKET-108's six (version 0.2.0, MIT OR Apache-2.0);
  - since #724, its TICKET-115's sixteen at `cda7c24` (version 0.3.0, with `skills/rustal-ste/`);
  - since #776, its TICKET-119's seventeen at `5333d15` (version 0.4.0): `register.js` hands a call
    that `MARLEY_APPROVE_TOOLS` names to `MARLEY_BIN approve` (#726's Plan tab), and
    `skills/plan-questions/` teaches #728's `:::question` blocks.

  `FILES` embeds them with `include_str!` in `rh`'s order. `digest()` hashes each path and text
  after its length as a little-endian u64; the folder is `b3b5ee99…e26a`, the harness's own.
- `identity::marley_plugins` gives terminals the shared plugin when it is set, else #725's skill
  folder, so the skill loads once (#724).
- `install_in(<data>/claude-code/shared)` gives the digest's folder. One already there is checked
  byte for byte (`check_in`); otherwise it is staged in `.staged-<uuid>` (folders 0700, files 0400,
  `create_new`, `sync_all`) and renamed into place.
- `reconcile` runs at `init`, on `SettingsStore` and on `agent_versions::observe`.
  - It wants the plugin while `marley.claude_code_shared_plugin` is on (`SharedPlugin::of`, from
    the merged settings) and `agent_versions::is_on(&CLAUDE_SHARED_PLUGIN)`.
  - Wanted, it installs off the main thread, then `set_shared_plugin(Some(folder))` if still wanted.
    Otherwise it calls `set_shared_plugin(None)`.
- `marley_terminal::identity::agent_environment` writes `CLAUDE_CODE_PLUGIN_DIRS` for a named
  terminal: the folder first, then Marley's inherited entries minus any under the same `shared`
  parent (`plugin_dirs`), only when that differs from the inherited value. No Zed path changes.
- `marley_agent::versions::CLAUDE_SHARED_PLUGIN` (2.1.287 and later) is the version row, chipped
  only while the setting is on.

## An agent's reports (`src/agent_reports.rs`, `bin/marley-agent`, #652)

- `mcp::start` calls `start(data_dir)`: the socket at
  `$XDG_RUNTIME_DIR/marley/<16 hex of the data directory's SHA-256>.sock` (Zed's temporary folder
  without one), checked with `marley_browser::service::socket_fits` and in a folder made 0700;
  then, off the main thread, `bin/marley-agent` written into `<data_dir>/mcp/`
  (`mcp::write_program_in`) with `agent-socket` beside it naming the socket, and only then the
  program's path handed to `marley_terminal::identity::set_agent_program`. Any failure logs a
  warning and terminals name no program; the hook frames go on.
- `marley-agent` (Python 3, standard library) takes `rh report`'s and `rh release`'s arguments,
  sends `{"verb", "terminal": $MARLEY_TERMINAL_ID, ...}` as one line, and exits 0, 1 with
  `<refusal>: <reason>` on stderr (`marley_not_running` when nothing answers within 4 s for a
  report, 0.9 s for a release), or 2 on arguments `rh` would refuse.
- On the connection's thread the handler reads the peer's chain of parents with their start times
  (`process_chain`, `report::stat_fields`, at most 32) and the working directory of its parent;
  the main thread takes the request (`take`). `terminal_of` matches the chain against each local
  terminal's shell (`pid_getter().fallback_pid()`), nearest first, checks the id the program sent
  against that terminal's, and names the reporter's parent the agent. `accept` refuses a stale
  `seq` per source and terminal, and a report from another source or another agent while the
  holder's process lives (its pid and start time); a holder whose process ended lapses.
- `Reports`, a global, keeps the holder per terminal view (`Held`: source, agent, terminal, last
  report). A report goes to `agent_events::apply_report` and, with a session id and the agent's
  folder, to `resume::on_report`; a moved state gets the banner and push (`announce`) through the
  view's window. A release ends the seat (`agent_events::end`) and `resume::on_release` drops the
  saved session.
- `agent_events::apply_report(view, report, kind, frame)` upserts the seat: the frames' labels,
  the report's over them (the progress pair cleared first), the agent label from the terminal's
  foreground agent, the reported state, except that a frame's `waiting` stands over a reported
  `working`, and a `QuestionRaised` for a reported question. `on_frame` runs it again after each
  fold (`frame` true, `held_report`), so a frame never moves the state the holder reported while
  its labels (the prompt, the tool in flight) still come through. `forget` and `end` drop the
  holder.

## A project's changed lines and pull request (`src/rail.rs`, `src/github.rs`, #531)

- `GroupEntry` carries `source` (`git_source`: the group's first workspace when its folder is a
  main checkout, with its branch, `HEAD` and `Repository`) and `git` (`ProjectGit`: the counts
  with their base, and the `github::PullRequest`), which `note_project_git` fills from the rail's
  `project_git` each refresh; `render_project_row` draws `ui::DiffStat` and
  `pull_request_chip` before the attention dot. The data stays out of `marley_rail`'s row model;
  a read's end calls `cx.notify()` itself.
- `follow_project_git`, from each refresh and, through the `GitStore` subscription, from each
  `StatusesChanged` (which marks `git_edited` and rebuilds nothing), schedules a
  `project_git_run` per trusted project whose branch or `HEAD` moved, whose tracked files
  changed, or whose pull request was asked about `PULL_REQUEST_EVERY` (two minutes) ago while the
  window is active; a run in flight keeps the edit mark for the next. The run waits a second,
  checks trust again, reads `origin`'s GitHub `owner/repo` (`github_repository`, Zed's
  `parse_git_remote_url` and the provider's name) and `default_branch(true)`, then
  `read_project_git` in the background: `github::pull_request` when due, and
  `worktree_git::changed_lines` against its base, else the kept pull request's, else the default.
  A read that was due to ask counts as asked, whatever it could ask.
- `github::pull_request` runs `gh pr list --repo=<owner/repo> --head=<branch> --state=all
  --limit=1 --json=number,state,isDraft,title,url,baseRefName` in the folder, no shell,
  `GH_PROMPT_DISABLED=1`; a failure is logged once per folder and shows no chip.
- `worktree_git::changed_lines(folder, base)` refuses a base that could read as an option or holds
  a space or a control character (a pull request's base came over the network), resolves
  `origin/<base>` then `<base>`, and sums `git diff --numstat --merge-base <base> --`, binary files
  counting nothing.

## Failed checks to an agent (`src/failed_checks.rs`, `src/github.rs`, `src/rail.rs`, #718)

- `github::pull_request` also asks for `headRefOid,statusCheckRollup`. `Rollup` reads the two
  shapes, tagged by `__typename`: a `CheckRun` is pending until `COMPLETED`, then failed for
  `FAILURE`, `TIMED_OUT`, `STARTUP_FAILURE` or `ACTION_REQUIRED`, cancelled for `CANCELLED` or
  `STALE`, else passed; a `StatusContext` is failed for `FAILURE` or `ERROR`, passed for
  `SUCCESS`, else pending. `Check::job` takes the job id from an Actions link.
  `github::failed_log` runs `gh run view --repo=… --job=… --log-failed` and keeps the last 64 KiB
  from a line's start (`last_bytes`).
- The chip draws `failed_checks::mark` after the number and adds `summary` to its tooltip.
  `HeaderMenu.failed_checks` (`has_failed_checks`) adds Send Failed Checks to Agent… to the project
  row's menu, which a right-click on the chip opens, since the chip has no handler of its own.
- `failed_checks::send` (the menu item and `SendFailedChecksToAgent`, registered by
  `send_selection::init`) reads the project through `rail::github_of` outside the workspace's
  update, asks `gh` again, and opens a `TargetPicker` of the agents whose folder is inside the
  project's, labelled with `review_notes::readiness`, and Copy Prompt. `prepare` reads the failed
  jobs' logs in the background, keeps 256 KiB in all, redacts them, and writes
  `<data dir>/checks/<owner>-<repo>-pr<n>-<head 8>.log` (`write_in`: made 0600, and set to it
  before the write); `deliver` pastes the one-line `prompt` with `send_text(…, submit: false, …)`
  or copies it.

## `show` from a terminal (`src/terminal_show.rs`, `bin/marley-agent`, #805)

- `marley-agent show [--as KIND] [--title TEXT] FILE|-` types the file by `--as`, else its
  extension; a missing file or a type it cannot name prints the reason and exits 1. With no
  `MARLEY_TERMINAL_ID` it prints its input and exits 0. Stdin (16 MB at most) is written mode 0600
  to `show-<pid>-<ns><ext>` in the socket's private runtime folder, its extension from `--as` or,
  for an image, its first bytes; the request names that copy with `stdin: true`. An `ok` exits 0;
  `show_invalid` prints its reason and exits 1; any other answer, or none in 4 s, deletes the copy
  and prints the input, exit 0.
- `agent_reports::take` hands a show to `terminal_show::arrived` once `terminal_of` finds its
  terminal, as it hands an approve to its review; `accept` refuses one that gets there.
- `arrived` reads the file through `smol::unblock` (`read_as`: a file, at most 16 MB; a table file
  Zed's preview reads stays a path, any other table goes through `tables::parse`, a component
  through `parse_str_with` and the user's templates, JSON a component else a table else the
  component's error, markdown read as text to check it), answers, deletes a stdin copy read into a
  table or a tree, and opens the tab through the window: `block_tables::open_file` (no block key),
  `block_tables::show_table` (a `BlockTable` of the same title reused), `component_tools::open_tab`,
  Zed's `MarkdownPreviewView::open_preview_in_pane` over an `Editor::for_buffer` no pane shows, or
  `open_abs_path` with an invisible worktree for an image.
- Marley's bash, zsh and fish integrations define `show` as `"$MARLEY_BIN" show` when `MARLEY_BIN`
  is set and the shell has no `show` (`type -t`, `whence`, `type -q`); fish's is replaced by a
  user's own function defined after it.

## Plan review (`src/plan_review.rs`, `src/agent_reports.rs`, `src/last_reply.rs`, #726 to #729)

- `plan_review::init` (from `agent_reports::start`) follows `marley.plan_review`: on, it hands
  `identity::set_approve_tools(Some("ExitPlanMode"))` to the terminals started from then on.
- `agent_reports::take` hands a `Request::Approve` from a known terminal to `plan_review::arrived`
  with the socket's `AgentRequest`, unanswered. A tool other than `ExitPlanMode`, the switch off,
  or an input with no `plan` is refused `agent_approve_tool` or `agent_report_shape`, which leaves
  Claude Code's dialog in charge.
- `arrived` numbers the request (a nonce), shows the plan in the terminal's `PlanReview` tab (made
  in the pane holding the terminal, else the active one; focused only when the terminal had the
  focus) and keeps the request in the `PendingPlans` global, which the rail observes. A second
  plan from the same terminal replaces the first, whose request is refused. A one-second watch
  drops plans whose `withdrawn` flag the socket raised and marks their tabs.
- `PlanReview`: a `Markdown` (default options, HTML not parsed) in an area whose focus is the
  tab's (`PlanReviewPlan`, where Marley's keymap binds `c` to `plan_review::Comment`); the
  right-click menu keeps the selected range when it opens, since a click on its entry clears the
  Markdown's selection first; notes are source ranges painted with `set_search_highlights`; a
  note's single-line editor takes `menu::Confirm` and `editor::Cancel`. Approve and Send Feedback
  call `answer`, which sends once to the request with the tab's nonce, unless it was withdrawn.
- The rail: `InboxKind::Plan` entries from `plan_review::entry_of`, ahead of the seat's own wait;
  `InboxTarget::Plan` opens the tab through `show_tab`, deferred out of the rail's update.
- Versions (#727): `PlanReview.versions` keeps each plan's text. From the second, `show` runs
  `changes_from`: `language::line_diff` counts the lines, and `acp_thread::Diff::finalized`
  ("plan.md", the version before as its base) feeds a read-only full-mode `Editor` (no gutter, all
  hunks expanded, soft wrap). The language registry comes from `open_tab`, which holds the
  workspace: `show` runs inside the workspace's update and cannot read it. The column's header
  draws "Version N", a `DiffStat` and the Changes / Plan toggle; while Changes is on, the left area
  is the diff editor in a container of its own, outside the plan's key context.
- Question cards (#728): `parts_of` turns `plan_questions::plan_parts` into `Part::Text` (a
  `Markdown` per run, with its start and length in the plan) and `Part::Card` (an index into
  `cards`). Zed's Markdown has no container block, so the plan is split before parsing, and a
  selection or a note stays inside one run: `selection` and `highlight` map a run's range to the
  plan's by its start. A `Card` holds its `Question`, its context as `Markdown` (drawn in the UI's
  small type, muted: the container style sets the size, the base style the color), a `picked` flag
  per choice, a single-line editor for Other… or the free-text answer (subscribed so that typing
  redraws the count), and `skipped` and `accepted`. `card_state` reads Settled (a `- [x]` choice),
  Skipped, Answered or Open. While `answers_pending`, the column's `answer_buttons` draw Send
  Answers alone; `send_answers` denies with `plan_questions::questions_feedback`. `c` propagates
  while a card's field contains the focus, so the field takes the letter; the area's own handle
  does not hold the focus after a drag, since the Markdown element takes it (#729's fix).
- Files and replies (#729): the tab's `Subject` is `Plan { view }`, `File { path, workspace }` or
  `Reply(ReplyOrigin::Terminal { view, project, workspace } | Thread { conversation, workspace })`;
  a file or a reply is `State::Open` (taking notes, no request) and one `Part::Text`, with no
  cards or versions. `marley::ReviewMarkdown` takes the active item's `act_as::<Editor>` (a
  Markdown editor, or a preview's source editor) when `MarkdownPreviewView::is_markdown_file`, its
  local path and its buffer's text. `marley::ReviewLastReply` takes the Agent Panel's thread while
  the panel has the focus, else the active thread tab, the active terminal, or the panel's thread.
  `review_terminal_reply` (also the rail's terminal menu) reads the transcript in
  `futures::future::lazy` on the background executor. `open_document` reuses a tab of the same
  subject and text. Send Notes: a file's notes become `ReviewLine`s (lines from the note's range,
  `quote`, the overall note with `first == 0`) for `review_notes::send_file_notes`, opened in a
  `window.defer` out of the tab's update; a reply's become `marley_agent::reply_notes` in the
  template, to `drafts::send_to_terminal` after `target_of` reads the seat ready, or to
  `drafts::send_to_thread`. A sent note is marked, the overall note kept as a sent note with an
  empty quote and its editor's buffer cleared. The overall editor's edits redraw the column.
- `last_reply.rs` (#729): `of_thread` gathers the `Message` chunks of the non-subagent assistant
  entries from the user message before the last one with text through it; `transcript_of` reads
  the seat's `transcript_path` label; `read_reply` seeks to the file's last 16 MiB and drops the
  partial first line; `reply_in` keeps entries that are neither `isSidechain` nor `isMeta`, finds
  the last assistant entry with text and the last prompt (a user entry with text and no
  `tool_result`) before it, and joins the assistant `text` blocks between.
- The rail (#729): Review Last Reply in `terminal_context_menu` calls `review_terminal_reply`; in
  the thread row's menu it opens the thread, then dispatches `ReviewLastReply` deferred.

## Guided review (`src/guide_tab.rs`, `src/review_tools.rs`, #733)

- `review_tools::guide_open` checks the guide (`guide_of`: a title, 1 to 30 sections, absolute
  paths, lines from 1, titles and summaries cut), finds the project holding the first file, else
  the caller's folder (`Caller::cwd`), is admitted as `review_notes_add` is, then activates that
  workspace and runs `guide_tab::open`.
- `guide_tab::open` replaces the sections of an open `GuideTab` with the same title (ticks cleared)
  or adds a new one to the center. Summaries are `Markdown::new` entities (HTML not parsed), drawn
  with `MarkdownStyle::themed(Preview)`. A section's Reviewed `Checkbox` folds it to its title and
  counts it; a chip runs `ProjectDiff::deploy_at_project_path` when the repository reports a status
  for the file (`in_project_diff`), else `open_abs_path` and `go_to_singleton_buffer_point`. Ticks
  live with the tab.

## Viewed marks in the project diff (`src/viewed_marks.rs`, Zed's `editor`, #731)

- `review_notes::init` calls `viewed_marks::init`, which follows `observe_new::<ProjectDiff>`:
  each new diff's right-hand editor (`ProjectDiff::editor(cx).rhs_editor()`) gets a `ViewedAddon`
  holding a `ViewedMarks` entity. `GitPanelAddon` is `pub(crate)` and sits on the staged and
  unstaged views too, so Marley does not key off it.
- `ViewedMarks` follows the editor's `BufferRangesUpdated` (a file arrives) and `BuffersRemoved`,
  and each file's `BufferDiff` (`MultiBuffer::diff_for`). Its `check` compares the file's
  fingerprint, a sha256 of `base_text_string` and the buffer's text, with the in-memory tick first,
  then with the key-value store's mark (`marley-viewed`, keyed by the absolute path): a match
  folds the file and ticks it; a tick that no longer matches goes, with its stored mark, and the
  file unfolds. `set` writes or deletes the mark off the main thread and folds or unfolds.
- Zed's side (`zed-touchpoints.md`): `Addon::render_buffer_header_trailing_controls`, a default
  method that draws nothing, and `render_buffer_header` drawing every addon's after Open File, in
  the header's trailing row. The tick stops its click, since the header folds on its own.

## Findings into the project diff (`src/review_tools.rs`, Zed's `editor`, #730)

- `mcp::answer` routes `review_*` to `review_tools::answer`. `review_notes_add` checks the notes
  (absolute paths, lines from 1, at most 50, comments cut at 4,000 characters), takes the source
  from the call or the caller's name, finds the first note's project (`editor_tools::project_holding`)
  and is admitted through `agent_control::admit(Area::Editors, Level::Act, …)`.
- `place` activates the project's workspace, runs `ProjectDiff::deploy_at_project_path` for the
  first file, and takes the diff's `rhs_editor()`. The diff loads its files a few at a time with no
  event at the end, so each note's range is tried every 100 ms for up to three seconds:
  `range_in_diff` finds the file's buffer among the multibuffer's, anchors the lines in the buffer,
  and maps them with `anchor_in_excerpt`, which is none outside the diff's excerpts. Each found
  range goes to `Editor::add_review_note`; the answer lists `placed` and `refused` with reasons.
- Zed's side (`zed-touchpoints.md`): `StoredReviewComment.{source, proposed}`, `ReviewNote.source`,
  the count and `unsent_review_notes` skipping proposals; `add_review_note` opens the block through
  `show_diff_review_overlay` and gives the focus back, then stores the note as a proposal and
  resizes the block; `keep_review_note`; `drop_review_note` runs the delete handler;
  `render_comment_row` takes the editor handle, so Keep and Drop reach the editor whatever has the
  focus. `review_notes::prompt` passes `source` into `ReviewLine`.

## Per-turn diffs (`src/turns.rs`, `src/turn_git.rs`, #509)

- `Turns`, a global made at the first write, holds a seat per terminal view whose Claude Code
  opened a turn: the session, the repository (weak) and its work directory, the open turn (its
  title, whether a harness started it, and its checkpoint as a shared task), the listed turns
  (`Turn { title, files, failed, injected, sha, repository }`, oldest first, each with the
  repository it was taken in), and the last record as a shared task; and the sessions that have
  pinned a turn. `Turns::of(view)` and `Turns::repository_of(view, sha)` serve the rail.
- `after_fold` calls `turns::on_event` for each lead event. A UserPromptSubmit that is not the
  compaction's continuation (`claude_events::prompt_origin`) closes the open turn and opens one in
  the innermost local repository holding the event's `cwd`, else the seat's `cwd` label, else the
  terminal's working directory, reusing the close's checkpoint when the repository is the same.
  `Stop`, `SessionEnd`, `SessionStart`, a manual `PostCompact`, an interrupt and a new session id
  close; `StopFailure` closes as failed. `agent_events::end` calls `on_end`, and `forget` calls
  `turns::forget`, which closes and drops the seat. A session id that is not ASCII letters,
  digits, `-` and `_` opens no turn, since it names the refs.
- A close takes `Repository::checkpoint` in the repository's job queue and spawns the record,
  which awaits the seat's record before it, so each takes the next number and lists in order: both
  checkpoints, `compare_checkpoints` (equal ends it), `turn_git::turn_refs_in` for the session's
  highest number, `turn_git::commit_tree_in` of `<end>^{tree}` on the start with the title and
  the `Marley-Session` and `Marley-Turn` trailers, `Repository::update_ref`, the file count from
  `diff_tree(Since)`, and on the session's first pin, `delete_ref` for each turn ref whose commit
  is older than 30 days. A detached waiter keeps a record running when its terminal closes.
- `turn_git.rs` is the adapter for the two programs `Repository` has no job for: `git
  commit-tree <tree> -p <parent> --no-gpg-sign -m …` with the author and committer `Marley
  <marley@localhost>`, and `git for-each-ref` over `refs/marley/turns/`, through `util::command`
  in the work directory, never a shell. #541's `process.rs` takes them over.
- The rail observes the global beside `AgentEvents`. `terminal_snapshot` fills
  `TerminalSnapshot.turns` newest first, and `note_turns_open` marks the rows in the rail's
  `turns_open` set (by terminal id, not saved, pruned to the live terminals). `render_turns` draws
  under the terminal's card a "Turns (N)" line with a `Disclosure` (a click on either toggles) and,
  while open, a row per turn: the title (truncated, whole in its tooltip), `· N files`, `· failed`
  in the error color and `· injected`. `open_turn` shows the terminal's project and calls
  `git_ui::commit_view::CommitView::open` with the turn's full sha and the repository the turn
  was taken in; the view diffs against the first parent, the turn's start. The keyboard walk does
  not step into the turns.

- **Codex's turns (#753).** `turns::on_codex` takes what the follow in `codex_server.rs` tells of
  the lead thread: `CodexTurn::Started` (a `turn/started`, the lead's `cwd` or the server's
  folder) closes the open turn and opens one with the thread's id as the session;
  `CodexTurn::Titled` (the turn's `userMessage` item, `user_message_started`) retitles it;
  `CodexTurn::Ended` (that turn's `turn/completed`, `failed` for a failed one) closes it. `Open`
  keeps the Codex turn's id, so another turn's messages change nothing. `Following::end_turn`
  closes the turn at `thread/closed` and at the server's stop, which leave a seat the rail's
  `end` passes over. The record, the refs and the rows are #509's.

- **Keep and Undo (`src/turn_undo.rs`, #714).** Each turn row sits in a `right_click_menu`;
  `turn_menu` offers Keep (unless kept) and Undo This Turn…, or a disabled "This turn was undone".
  `Turn.mark` (`marley_rail::TurnMark`, `Kept` or `Undone`, an undone turn never re-marked) goes
  through `Turns::mark`, which writes the global the rail observes; `turn_marks` draws `· kept`
  in the success color and `· undone` in the warning color.
  - `undo` checks for an agent at work (`agent_at_work`: each terminal whose `agent_in` has a
    `close_guard::working_status`, its folder the seat's `cwd` label or the terminal's working
    directory, and each Agent Panel thread `Generating` in its `work_dirs` or the workspace's
    roots; a hit is either path starting with the other, the turn's own agent included), takes
    one slot per work directory in the `Undoing` global, and asks the repository for a
    checkpoint.
  - `run`, on the window: `turn_git::undo_merge_in` merges with the turn's commit as base, the
    checkpoint as ours and the turn's parent as theirs (`git merge-tree --write-tree
    --merge-base`, git 2.40 or later); a conflict refuses with its paths. `changed_paths_in`
    lists what the merged tree changes against the checkpoint; none says there is nothing to
    undo. `unsaved_among` asks each project's `BufferStore` for a dirty buffer at each path. The
    question names each file, then the agent check runs again and `write_merged_in` writes the
    merge through a temporary index (`read-tree -m -u`), which refuses a file changed since the
    checkpoint. The mark follows a write.
  - The cross-window lookups run inside the window's update, where that window's handle reads as
    gone; `workspaces(window, cx)` reaches it through `window.root` and the rest through their
    handles, and `mcp::terminals_of` takes that list (PR-claude-596).
  - The user's index, `HEAD`, branches and the turn refs are never written; the undone turn's ref
    keeps its diff.

## Stalled or looping agents (`src/stall.rs`, #569)

- `on_frame` calls `stall::moved` after each fold and arms `stall::watch` while the seat works,
  and `after_fold` calls `stall::note_tool_end` at a working seat's tool end.
- `StallWatch`, a global no view observes, holds each working seat's last two CPU samples, its
  quiet episode (the event it is the quiet after, how many checks were asked, whether an ask is
  out, whether the banner went), its call waiting for an outcome, the boot time and whether the
  timer runs. Its ticks redraw nothing; only a landing writes `AgentEvents`.
- `watch` starts one timer for the app while the use is on and `stall_check_after_seconds` gives
  checks, armed by `on_frame` and by a settings change. It ticks every quarter of the first check,
  between 2 and 10 seconds, and ends itself when no seat works. A tick plans on the main thread
  (the working seats quiet half a check or more, each terminal's pid and its turn's start),
  samples `/proc` on the background executor (`futures::future::lazy`), and decides on the main
  thread. A sample counts beside another only when both are inside the quiet, and a seat at its
  next check with nothing burning CPU is asked once; a watch that starts late asks once for all
  the checks already passed.
- `ask` builds the state in `asking_for`: the project, the agent, the quiet in words, the tool in
  flight by name, whether the tools use the CPU, the subagents and the permission mode as facts;
  the prompt, the tool line and the terminal's last five lines as text, masked and cut, and left
  out for a metadata-only project. `answered` keeps the call for its outcome and, in `suggest` or
  `act`, lands `stalled:<kind>` for `waiting_for_input`, `stuck` or `frozen` through `Landing`
  (the seat working on the same session, prompt and stop count, with no event since), and posts
  one banner for the episode in `act`.
- `note_tool_end` flags a loop at once from the rule alone: a `rules` row through
  `system_one::record` with `repeating` held, whatever the provider, logged once for each loop,
  and in `suggest` or `act` the `looping` labels.
- `moved` logs the outcome of the seat's call waiting for one at its next event (`the next event
  12 seconds later: Stop`), or for a loop when the loop ends (`the loop ended … later: Stop`).
  `hold` settles a call the next check replaces (`still quiet 20 seconds later`), and a reading
  that came after the seat moved on logs `the agent moved on before the reading came back`.
  Refused and failed calls wait for nothing.
- Nothing here writes to the terminal, interrupts or stops the agent: a flag marks the row.
- The rail: `terminal_snapshot` gives `seat_line` the `FlagShown` that `flag_shown` reads from
  the mode, and `TerminalSnapshot.flag` the tooltip. `render_terminal_row` draws
  `IconName::Warning` in `Color::Warning` before the row's end, with an id of its own for the
  tooltip, so it stays while the pointer over the row shows the close button.

## The pause before a consequential click (`src/click_pause.rs`, #571)

- `Who::of(call)` sorts a browser call's caller when `browser_tools::answer` takes it: an
  outside client (#524) is unknown; a call from a Marley terminal (#520, `mcp::caller_terminal`)
  asks first unless its Claude Code seat's `permission_mode` is `bypassPermissions` or `dontAsk`,
  and one with no Claude Code there is unknown; with no terminal, a session whose client named
  itself `Zed` asks first unless Zed's `agent.tool_permissions` give `mcp:marley:browser_click`
  (or, unset, the default) `allow`, and any other caller is unknown. Unknown callers do not ask.
- `before_click(Click { page, tab, target, point, what }, who)` runs before `click` and before
  the click `type_text` makes for a ref (`browser_tools::paused_point`). With the use off, or a
  caller that asks first under `agents_without_prompts`, it does nothing. Else it reads the
  element (`Page::node_facts`, or `node_at` for a point), classifies it, and: a plain click goes;
  a rule's class is a `rules` row and a pause; an open one is asked (`system_one::ask`,
  `CLICK_CONSEQUENCE`), and in `act` a noul that holds pauses, in `suggest` a notice follows the
  click, in `shadow` the reading is only logged. A read that fails is open unless the rules
  already pause.
- `hold` parks the click on the tab's page state (`BrowserHub::pause_click`, a `PendingClick`
  with a one-shot answer), shows the toast, and waits for the answer, the page's going or 25
  seconds. Allow clicks only when `same_element` finds the same page and the same element, role,
  name and tag, under the ref or the point; the refusal the agent reads says the user refused,
  did not answer, or the page changed or went. The pause's end is the outcome of the call that
  decided it (`allowed after 3 s`), and the recorder keeps `paused:` and how it ended.
- While a click waits, `run` refuses the tab's other writes (`not_paused`: the `WRITES` set,
  `browser_navigate` and `browser_check_pick` on that tab), and a second pause in the tab is
  refused too.
- The card (`BrowserView::render_pause`, drawn by `pause_bar` under the toolbar) has its own
  focus handle and never takes the focus by itself: Refuse and Allow are buttons whose clicks go
  no further, a click on the card focuses it, and only then do Enter and Escape answer it
  (`MarleyBrowserPause`, `marley::AllowPausedClick`, `marley::RefusePausedClick`).
  `show_pause_toast` names the click with Show (`show_paused`: the tab in front, its workspace
  active, the focus on the card), and `dismiss_pause_toast` takes it away.
- The rail's inbox lists the held click (#508): `pause_click` and `end_pause` emit
  `PageStatusChanged`, the entry says what the card says (`pause_sentence`), and its Refuse and
  Allow call `answer_pause` as the card's buttons do. Since #568 the pause keeps its `Class`
  (`PendingClick.class`, from `hold`'s `Held { sentence, class, call }`), which
  `BrowserHub::pause_class` gives the inbox for the entry's chip.

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
  A terminal whose foreground arguments hold `--remote-control` (`marley_agent::remote_control`)
  pushes nothing, since the Claude app notifies for it (#757).
- `post` runs on a background task: the token from `token_file`, refused when group or others
  can read the file; `Title: Marley`, `Priority` 4 (needs input, failed) or 3 (finished), `Tags`
  `question`, `white_check_mark` or `x`, the line as the body. `report` logs a refusal, and a
  failure too, with one toast (`Pushes::failing`) until a post succeeds.
- **More sources and four kinds (#752).** `on_change` reads the event with `TurnEvent::of_seat`
  (an approval, a question, a finish, a failure) and names the seat's agent (`agent_name`, from
  `agent_events::seat_agent`); its callers are the frames, an agent's report
  (`agent_reports::announce`) and Codex's follow (`codex_server::push_change`, after
  `agent_events::fold_codex` gives the change, in the window `browser::window_of` finds). A Zed
  thread pushes through `on_thread`, which `thread_tab` sets as `MarleyThreadHost::attention`:
  Zed's `ConversationView::notify_with_sound` hands it the event's `MarleyAttention`, whether the
  thread is visible, its agent's id (named through `agents::thread_agent_name`) and its folders.
  `push` posts only a kind `PushSettings::kinds` holds (`marley.push.approval`, `input`,
  `finish`, `failure`, each on unless set false); an approval's tag is `lock`.

## System One (`src/system_one.rs`, `src/system_one_calls.rs`, #565, #659, #548)

- `SystemOneSettings`, in `MarleySettings`, is `marley.system_one` resolved: the switch, the
  provider, the endpoint, the model, the two project lists (with `~/` as the home directory), the
  day's budget in cents, a `compatible` provider's price in thousandths of a cent per million
  tokens (the `f32` of the settings goes through a rounded decimal, since the lint table refuses
  a lossy cast) and each use's mode.
- The `SystemOne` global holds the settings it applied, the key and its source, the gate, the
  recorded answers, this session's rows and the log's sender. `init` applies the settings and
  follows them, and registers the check and `OpenSystemOneCalls` on each workspace.
- **The key.** `Key` prints as `Key(***)` and leaves only as the `Authorization` header and as its
  own mask. `load_key` reads it only while the layer is on and its provider sends requests: the
  variable `MARLEY_SYSTEM_ONE_KEY`, else gpui's `read_credentials` at the endpoint's URL, never
  Zed's dev-channel credentials file; a newer load wins over an older read that finishes late. So
  a user who leaves the layer off never meets the keyring's unlock prompt. `store_key` and
  `forget_key` write and remove the keyring's item and load the key again. On Linux gpui's
  keychain is the Secret Service through `oo7`: items labelled `Marley` with `application =
  marley`, an item under the old `zed-github-account` label read and copied once, and a delete
  removing both (#761, `gpui_linux/src/linux/platform.rs`). #761's scenario runs a keyring of its
  own on a private bus; the others touch none.
- **`ask(spec, asking, cx)`** is the one way a use asks. With the layer or the use off it answers
  `Reading::Off` and does nothing else. Then `policy::may_send` decides the detail, and the state is
  built with `mcp::model_redactor`, the rules and patterns whatever agents' redaction says, plus the
  key's own mask. `rules` answers the use's verdict, and `replay` the recorded answers.
  `typesafe` and `compatible` go through `send`: the endpoint is checked (`https`, or `http` on this
  machine) before the key is used, then the key, then the gate.
- `post`, on the background executor, builds the request with the key's header and
  `HttpRequestExt::timeout`, and races it with a backstop timer (dropping the send cancels it). It
  tries once more on 429, 503 or 529 while the deadline leaves time. `read_posted` spends on an
  answer, counts a failure toward the breaker, and marks the key refused on a 401 until the key
  changes; an error kept is cut to 300 characters and masked with the key.
- `finish` fills in the row, keeps it for System One calls and sends it to the log task, which the
  first call starts and which appends each row off the main thread (`files::append_in` under
  `<data dir>/system_one/`).
- **The check** (`SystemOneCheck`) asks about `browser::last_terminal`, the terminal the focus
  entered last: the project's name, the block's index, the exit code and the program as facts, and
  the terminal's title and the last command as text, with the exit code as the use's own verdict.
  It runs inside the workspace's update, so a terminal of that workspace takes its folders from
  the `&mut Workspace` the action has; reading the workspace entity there would panic. The program
  is the command's first word after its `NAME=value` assignments, without its folder. A toast
  gives the reading, the provider, the tokens and the time.
- Since #567 `ask` and `record` take a `UseSpec` by value, which is `Copy` with `'static`
  references, so a use makes its own at call time, and `Asked.answers` carries the parsed answers,
  whose per-option probabilities a use can rank by.
- Since #566 the adapter has three more ways in for a use: `use_mode(name)` (off while the layer
  is off), `detail(asking)` (what the project may send, read before a use picks its set), `record`
  (a verdict the use's own rules settled, logged as a `rules` row whatever the provider, with the
  state the project may send) and `outcome(call, text)` (an `OutcomeRow`). `state_for` builds the
  masked state for `ask` and `record` alike.
- Since #569 `project_name(folders)` gives the name a state calls a project by, its first
  folder's or `a project`, which the check, the stop kind, `terminal_find` and the stall kind
  share.
- **System One calls** (`SystemOneCallsView`, a workspace item) reads today's file when it opens
  and follows the global for the calls made after, newest first; a click opens a row to the state
  as sent, the answers and the error. Its header gives the day's calls and spend against the
  budget, the provider and model, the key's source and an open breaker; Run Check dispatches the
  check, and Set Key shows a masked single-line editor whose Enter (`menu::Confirm`) writes the
  keyring and whose Escape (`editor::Cancel`, which the editor lets through) puts it away.
- **The rename** (#659). The tab was `DecisionsView` in `src/decisions.rs`, titled Decisions, until
  Rusty's Decisions tab took the name. The ids are `system-one-calls*`, the key context
  `SystemOneCalls`, and `marley::OpenSystemOneCalls` declares `marley::OpenDecisions` in
  `deprecated_aliases`, so a keymap binding the old id still opens the tab and the palette lists
  only the new name. The Marley settings page's link is System One Calls, dispatched by its name
  through `build_action`.
- **Cloudflare, and a provider per project** (#548).
  - `SystemOneSettings` holds `by_project` (folders with `~/` expanded, each with its provider),
    `cloudflare_account` and `cloudflare_api`. `provider_for(folders)` picks the deepest folder
    holding one of the project's, as `agents.rs` does for `agent_permissions_by_project`, else
    `provider`; `uses_provider` asks whether the default or any entry names one.
  - The keys are two `Slot`s, `Direct` (`typesafe`, `compatible`; `MARLEY_SYSTEM_ONE_KEY`) and
    `Cloudflare` (`MARLEY_CLOUDFLARE_API_TOKEN`), each a `Credential` with its key, source, refused
    flag and load counter. `load_key` loads both; a slot reads only while the layer is on and some
    provider in use sends with it, at the URL of `slot_provider` (the default when it sends with
    the slot, else the first project's entry). `store_key` and `forget_key` take the slot.
    `state_for` masks with both keys.
  - `ask` resolves the project's provider and the `Draft` carries it; `record` stays `rules`.
    `endpoint(settings, provider)` builds Cloudflare's `{api}/accounts/{account}/ai/run` (the id
    letters and digits only) through `guarded_url`, the `https` or loopback `http` check
    `compatible` shares. `send` takes the slot's key and builds `build_cloudflare` at Jev's
    price, and the row records `typesafe/jev`; `read_posted` parses the envelope, refuses the
    Cloudflare token on 401 or 403, and puts Cloudflare's words in the reading.
  - System One calls adds a Cloudflare line (account, token source) and Set Cloudflare Token and
    Forget Cloudflare Token while Cloudflare is in use; its key field knows its slot.

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
  - `terminal_read`: `Terminal::block_output`, redacted whole, then one page of it (#680):
    `page(text, before)` checks `before` (2 to the line count plus one, else `bad_argument`) and
    `page_ending` takes whole lines back from the line before it while they fit in `PAGE_BYTES`
    (12,000) and `MAX_READ_LINES`, or the end of a single longer line (`line_cut`). `Page::fill`
    puts `output`, `first_line`, `last_line`, `total_lines`, `previous` (the next `before`, null at
    line 1), `line_cut` and `truncated` into the structured answer; `Page::text_block` starts the
    text with a line naming the lines and the `before` when the page leaves output out.
    `terminal_drive::run_answer` gives `terminal_run`'s output as `newest_page`. The terminal
    tools refuse with codes: `terminal_with_id` answers `no_terminal` with up to 20 ids there are,
    `terminal_of` `bad_argument` for a call from no Marley terminal that names none,
    `block_argument` `bad_argument`, `no_block` the blocks' range and `output_gone` a block whose
    output left the scrollback; `check` and `check_run` refuse through `terminal_of` and keep their
    other refusals in words (`check_in`, `check_run_in`), which answer `refused`;
  - `ports_list` (#521): a scan made at once (`ports::list`), off the main thread, each
    listener with its project's rail name, the folder, the address, port, URL, pid, process name
    and working directory, by project and port. The command line stays out, since it can carry a
    token;
  - `terminal_find` (#567), on `ports_list`'s spawned route: `FindLines::of` takes the block by
    index from `terminal_of`'s terminal, masks its whole output with `model_redactor` before any
    cut, keeps `tail`'s end and of it the newest lines up to 96,000 bytes, with the first one's
    number and the terminal's workspace as the place; then `crate::find::find_items`. The
    candidates carry the lines as masked, whatever agents' redaction says, so the answer and the
    model read one text.
- **The tools turned on (#567).** `enabled_tools` reads which of `marley_mcp::CONDITIONAL_TOOLS`
  the settings turn on (System One on and the use's mode not `off`), and `push_enabled` sends the
  set at `start` and at each `SettingsStore` change to `enabler`, a background task that takes the
  server's lock (`transport::set_enabled`), as the fleet snapshot goes.
- **Redaction (#516).** `start` builds the `AgentRedaction` global from `MarleySettings` and
  rebuilds it on each `SettingsStore` change that alters `redact_secrets` or
  `redaction_patterns`, with an app notification naming a pattern that did not compile.
  `agent_redactor(cx)` answers its `Arc<Redactor>`, `None` while redaction is off, and the
  built-in rules alone before `start` ran, so nothing leaves unredacted by accident.
  `model_redactor(cx)` answers the same redactor whether or not agents' redaction is on, for
  what leaves the machine for a System One model (#565).
  `terminal_blocks` runs each command through it and `terminal_read` the command and the whole
  output, before a page is cut (a key whose BEGIN line fell before the cut would pass
  otherwise); both answer `redacted`, the count, over the whole output. `browser_tools`' `browser_console` runs each
  entry's text through it.
- Since #501, once the server runs, `offer_to_zeds_agents` writes the bridge
  (`claude_plugin::BRIDGE`) to `<data dir>/mcp/marley-mcp-bridge` off the main thread and adds
  `context_servers.marley` to Zed's default settings: a stdio server running it, with
  `MARLEY_MCP_ENDPOINT` naming the endpoint file. Zed's context server store then runs it for
  each local project, where the Zed Agent's Write profile (`enable_all_context_servers`) takes
  its tools from, and `mcp_servers_for_project` hands it to each external agent's `session/new`.
  No bearer goes into a setting; a user's own `context_servers.marley` replaces the default.
  Since #811 the file it names is `<data>/mcp/own-endpoint.json` (`OWN_ENDPOINT`, 0600, written
  beside the bridge and removed at quit), which holds `ServerHandle::own_bearer`: Zed's agents are
  Marley's own, and the shared `mcp-endpoint.json` opens the read tools alone without a proven
  terminal. `route` hands a `client_*` call to `clients::answer` before `answer`, since the
  pairing tools are open to every caller and act on nothing.
- **The Marley agent (#683, `src/assistant.rs`).** `init` follows `marley.assistant.enabled`:
  `sync` acts only when the switch moved, since the defaults it writes fire the settings observer
  again. On, it puts `marley.agent_session_meta.Marley` (`session_meta`: the `INSTRUCTIONS` as a
  system-prompt append, `DISALLOWED_TOOLS`, no bypass) in the in-memory defaults, adds a default
  `claude-acp` registry entry when the user's settings lack one, and `resolve`s that adapter's
  command through the first open project's `AgentServerStore` (`get_command`, which installs it),
  or again from the next workspace when none is open; `set_entry` then adds the custom `Marley`
  entry with that command, `CLAUDE_CODE_EXECUTABLE` (`claude_program`) and an empty
  `ANTHROPIC_API_KEY`. Off, it takes out all three. `MARLEY_ASSISTANT_ADAPTER` replaces the
  resolved command. `offer_later` runs `claude auth status` through `process::output` five seconds
  after start while the user's raw settings lack the key (`decided`), and `show_offer`'s Turn On
  and Not Now write it (`choose`). Since #684 `OpenMarleyAgentInTerminal` (`open_in_terminal`)
  writes the instructions to `<data dir>/assistant/instructions.md` and types `'<claude>'
  --append-system-prompt-file '<file>' --disallowedTools …` into a new agent terminal through
  `agents::start_in_terminal`; `filter_palette` hides the action while the switch is off.
  - **Since #687** `wanted` reads the switch with `marley.assistant.agent`
    (`MarleyAssistantAgent`).
  - `Assistant.applied` holds the agent whose defaults are in place. `sync` runs `take_out` on the
    old agent and `put_in` on the new one.
  - Claude Code keeps #683's defaults. Codex adds a `codex-acp` registry default and resolves it;
    `set_entry` gives it `INITIAL_AGENT_MODE=read-only` and `CODEX_CONFIG` (`codex_config`:
    `developer_instructions`, `sandbox_mode`). Zed's agent adds `agent.profiles.marley`
    (`profile`: no built-in tools, `enable_all_context_servers: false`, and `PROFILE_TOOLS` under
    `mcp::CONTEXT_SERVER`).
  - `resolve` tracks the agent it resolves for. A command that comes back after the agent changed
    is dropped.
  - `forget_default_profile` resets a user `agent.default_profile` of `marley` to `write` once the
    merged settings lack the profile. It runs on `take_out` and at the offer's moment.
  - `offer_later` tries `claude_signed_in`, then `codex_signed_in` (`codex login status`'s exit
    status), then `zed_model_ready` (`LanguageModelRegistry::default_model` with an authenticated
    provider). `show_offer` names the agent it found, and `choose` writes the switch with it.
  - `open_in_terminal` types the Codex line `'<codex>' --sandbox read-only -c
    'developer_instructions="…"'`. `filter_palette` lists the action only for an agent with an
    adapter.
  - **Since #696** there are two entries and the agent is on by default.
    - `Entry { Marley, Rusty }` gives each entry its name (the agent-server and meta key), its
      profile (`marley`, `rusty`) and its instructions (`INSTRUCTIONS`, `RUSTY_INSTRUCTIONS`).
    - `Entries { agent, marley, rusty: Option<RustyServer> }` is both `Assistant.applied` and what
      `wanted` computes. `Entries::of(entry)` compares one entry, so `sync` takes out and puts in
      only the entries that changed. The adapter changes only with the agent (`add_adapter`,
      `drop_adapter`), so a thread on one entry survives the other's move.
    - `MarleyAssistantAgent::Auto` resolves through `detect` and the `Detected { NotYet, Looking,
      Found }` state, which replace `offer_later`, `show_offer`, `choose` and `decided`.
      - It waits `DETECT_AFTER` (1 s) first, so Zed's tests, with their stopped clock, start no
        status check.
      - It then checks Claude Code, Codex, and after `ZED_MODEL_AFTER` Zed's model, logs
        `assistant: auto chose …` and syncs.
      - A detection the setting moved away from while it looked is dropped.
    - Rusty's entry follows `rusty::agent_server`. `rusty.rs` sets that to the
      `RustyServer::Stdio(path)` or `Http(url)` of the last connection that came up, keeps it
      while the link is down, and clears it when the source changes. `init` observes the `Rusty`
      global as well as the settings.
    - `session_meta(entry, rusty)` adds `mcpServers.rusty` and `mcp__marley` to the disallowed
      tools. `codex_config(entry, rusty)` adds `mcp_servers.rusty`.
    - **Since #698** the Marley entry's `session_meta` also disallows `marley_tools_kept_out()`:
      every name in `marley_mcp::registry()` outside `PROFILE_TOOLS`, as `mcp__marley__<name>`.
      Codex has no equivalent: codex-acp's `build_session_config` inserts each server ACP passes
      with `disabled_tools: None`, replacing a same-named entry of `CODEX_CONFIG`.
    - The `rusty` profile has no built-in tools and `enable_all_context_servers: true`. While it is
      applied, `rusty::offer_again` offers the `rusty` context server as `agent_tools` does
      (`wants_rusty_profile`).
    - `set_entries` writes every applied entry once the adapter's command is known.
  - **Since #813** the read-only limits are the Marley entry's alone; Rusty's entry is the agent
    it runs on, plus Rusty's instructions and server.
    - `session_meta` puts `DISALLOWED_TOOLS` and `marley_tools_kept_out()` in for `Entry::Marley`
      only; Rusty's `disallowedTools` is `mcp__marley` alone.
    - `codex_config` adds `sandbox_mode: read-only`, and `set_entry` `INITIAL_AGENT_MODE=read-only`,
      for the Marley entry only, so Rusty's Codex threads start in codex-acp 2.2.2's
      `DEFAULT_AGENT_MODE`, `agent` (Auto review).
    - `profile(entry, cx)` gives the `rusty` profile the `tools` of `agent.profiles.write` in
      `raw_default_settings` (`agent_settings::builtin_profiles::WRITE`), so Zed's Write list stays
      the source.
    - `RUSTY_INSTRUCTIONS` drops the no-commands paragraph and says Rusty's own data changes only
      through the rusty server's tools.
    - Zed's `AgentConnectionStore` keeps an agent's connection by agent id, so an entry whose
      settings change (the agent moved from Claude Code to Codex) keeps its old process until
      Marley restarts (#814).
- **Marley and Rusty in tabs (#738).** `TalkToMarley`, `TalkToRusty`, `NewMarleyConversation` and
  `NewRustyConversation` run `talk_to(entry, fresh)`: nothing unless `applied.of(entry)`; unless
  `fresh` or on Zed's agent, the entry's newest unarchived conversation (`latest_conversation`, by
  the agent id its threads carry) comes forward through `window.defer` +
  `threads_page::open_thread`; otherwise `thread_tab::start_thread` on `made(folder_of(entry))`
  (#780: `marley.assistant.folder` or `marley.rusty.agent_folder` through
  `system_one::folder_path`, else `paths::config_dir()` or `rusty::vault_here`, else, since #791,
  the home folder; a set folder is made when missing; it replaced #738's `assistant/marley|rusty`
  under the data folder), on Zed's agent with
  the entry's profile,
  which `build_view` sets on the native thread once it exists. `filter_talk` shows each pair in
  the palette only while its entry is there (`talk_shown`).
- **Docs, settings and actions (#681).** `answer` sends `docs_` calls to `docs_tools::answer` and
  `settings_`/`actions_` calls to `settings_tools::answer`.
  - `docs_tools.rs`: `DocsBundle`, a `util::fs_embed!` over `docs/src/**/*.md` and
    `docs/marley/guide.md` (embedded in a release build, read from the checkout in a dev one). The
    `DocsIndex` (a `OnceLock`, built on first use off the main thread) splits each page into
    sections at headings `#` to `####` outside code fences; pages are named `zed/<path>` and
    `marley/guide.md`. `search` ranks sections by how many of the query's words they hold, then
    hits (a heading hit counting `HEADING_WEIGHT`), with a `snippet`; `read` takes a page or a
    section by heading or slug, fills `{#kb …}` and `{#action …}` (`fill_placeholders`, from the
    keymap read on the main thread) and pages forward with `mcp::page_from`.
  - Since #723:
    - `DocsBundle` adds `docs/marley/walkthrough.md`, and `RootBundle` (`crate_relative = "../.."`,
      `include = ["CHANGELOG.md"]`) serves the changelog as `marley/CHANGELOG.md`.
    - `split_page(name, text, bullets)` starts a section at each top-level bullet when `bullets`
      is set (the changelog), headed by `bullet_heading`: the bold title, else the first eight
      words.
    - `search` counts each query word, else its `stem` (-ies, -es after s/x/z/ch/sh, -s, -ing,
      -ed, at least three letters left); `snippet` looks for either.
  - Since #815, the crate's `build.rs` names the two bundles' paths (`docs/src`,
    `docs/marley/guide.md`, `docs/marley/walkthrough.md`, `CHANGELOG.md`) to cargo in a build
    without `debug_assertions`, so a release build compiles the crate again when a page changes;
    rust-embed's compressed embed names no file to cargo. Its list has to follow the two
    `fs_embed!` blocks' `include`.
  - `settings_tools.rs`: `schema` builds `SettingsStore::json_schema` on the background executor
    with the actions' names and documentation, and `describe` resolves the key path (`resolve`
    follows `$ref` into `$defs` and drops an optional's null branch), with the default from
    `raw_default_settings`. `read` walks `get_all_files()`, each file's content serialized and read
    at the key path, names each file (`file_name`), and answers `effective` (the winning file's
    value, the merged one for an object) and `global` (`merged_settings`, which leaves project
    files out); `hidden` masks values under secret names, the key's own last name included, and
    `redact_strings` runs the agents' redaction over every string. `actions_list` matches the
    query's words in each action's name, palette name and documentation; `bound_keys` reads the
    keymap highest precedence first, and `palette_name` is Marley's own rule.
  - `mcp.rs`: `page_from` and `Page::{next, fill_forward, text_block_forward}`, #680's pager run
    forward for a document.
  - `settings_change.rs` (#682): `propose` checks the key with `settings_tools::find_setting` on
    the background executor, reads the user's settings file through `Fs` (an absent one as `{}`),
    edits it as text with `settings::update_value_in_json_text` over the old and new roots
    (`set_at` makes the objects on the way), and refuses a change whose `UserSettingsContent`
    parse fails where the old file's did not. It asks with `show_app_notification` (a
    `MessageNotification` with Apply and Decline, one `NotificationId` per question), races the
    answer against 25 seconds (under the server's 30-second wait for the app), rereads the file
    before `atomic_write`, and answers with the values before and after through
    `settings_tools::hidden`. Since #686 the question, the wait, the reread and the write are
    `ask_then_write` (`Question { headline, change, file }`), and `answer_keymap` uses them too:
    `check_binding` (the action among the app's, `Keystroke::parse`,
    `KeyBindingContextPredicate::parse`), then `propose_binding` runs
    `KeymapFile::update_keybinding` with `Add` on `paths::keymap_file()` (an absent one reads as
    Zed's initial keymap), the keystrokes mapped through the platform's keyboard mapper.

## Rusty (`src/rusty.rs`, #633, #643)

- `MarleySettings::rusty` is `RustySettings::from_content(marley)`: `Source::{Off, Embedded,
  Service(url)}`, off unless `marley.rusty.enabled` is on, and `agent_tools` only while it is.
- **Off leaves no trace** (#661). `follow_setting` ends with `filter_palette`, which on a change
  (kept in `PaletteShown`) hides or shows the `rusty` namespace and `ToggleBrainView`'s type
  through `command_palette_hooks::CommandPaletteFilter`, as `agent_ui` does for `disable_ai`.
  The settings page's half is in `settings_ui` (`marley_page::rusty_on`, the page built from
  `cx`, and a rebuild when the switch changes; see the ledger rows).
- `init` sets the `Rusty` global (the source, the `State`: `Off`, `Starting`, `Connected { server,
  via }`, `Down(reason)`, `Missing(reason)`, Rusty's settings as last read, a refused write, the
  server and the keeper task), makes `RustyServerView` and registers it in
  `settings_ui::MarleyPageViews` as `rusty`, and follows the settings store. `follow_setting`
  drops the keeper and the server when the source changes, which ends an embedded child (Zed's
  `StdioTransport` kills its process group on drop), and starts `keep` for a new one; every change
  goes through `update_global`, which the view observes.
- `keep` loops over `connected`, 1 s doubling to 60 s between attempts. `server_for` builds Zed's
  `ContextServer::stdio` for `rusty-mcp` (`find`: `MARLEY_RUSTY_MCP`, else the launcher's search
  path, off the main thread, the reason naming where it looked) or `ContextServer::http` for a
  loopback `http` URL (`loopback`). `connected` starts it within 5 s, says the server's name and
  version and how it is reached, reads Rusty's settings (`settings_list` into
  `marley_rusty::ServerSettings`), re-reads them on an embedded server's
  `notifications/resources/list_changed`, and checks the link with a `ping` every 5 s within 5 s.
  The `Ping` is a `Request` of its own, answered with a JSON value: Zed's typed one reads `()`,
  which no server's `{}` parses as. `set_provider` writes `embedding_provider` with `setting_set`
  and reads again (`set_setting` since #666, for any key); `call` gives a tool's text or its
  failure's first line, within 5 s. Since #666 `read_settings` also reads `brain_semantic_status`
  into the global's `semantic`, and the page's settings section (`settings_section`) draws the
  status, a row per key in `KNOWN` (the provider's dropdown for `embedding_provider`), the other
  stored keys and the add row. Each field is `window.use_keyed_state` under the key and a hash of
  Rusty's value, so a value read back makes a fresh field; Enter in a row's `RustySetting menu`
  context writes `value_to_write`'s value, and a masked key's field is emptied after its write, as
  Rusty answers the mask again. Since #663
  `call_within` and `call_tool_within` take a deadline of their own, given both to Zed's client
  (`request_with`, which otherwise stops at its own 60 s) and to Marley's timer, whose failure
  names the seconds.
- `offer` and `settle` (#633) put `context_servers.rusty` into Zed's defaults while
  `agent_tools` is on and `find` finds `rusty-mcp`, or take Marley's entry out; a user's own entry
  sits in the user's layer and wins.
- `RustyServerView` draws Zed's `ui::AiSettingItem` for `rusty-mcp` (a short word on the row, the
  path or the reason under it) and, while connected, the Embedding Provider row with a
  `ui::DropdownMenu` over a `ContextMenu` kept with `use_keyed_state`.
- `script/e2e.sh` writes `marley.rusty` off with a service URL that reaches nothing and drops
  `marley.rusty_tools` in each run's copy, and exports `MARLEY_RUSTY_MCP` naming no file; a
  scenario that turns Rusty on names `marley_rusty`'s stand-in (`MARLEY_RUSTY_MCP`,
  `RUSTY_STAND_IN_STATE`). Voice is off in the same copy (#642).

## The rail's Brain view (`src/rusty/brain.rs`, `src/rusty.rs`, `src/rail.rs`, #644)

- **The header's screens** (#672). `brain::Screen` lists Today, Graph, Tasks, Decisions, Memory,
  Skills and Secrets with their element ids, icons and labels; `brain::open_screen` opens one in
  the shown workspace (Today through the Brain view's `today` when the rail has one, so the page is
  revealed in the tree, else `capture::open_today`). The Brain view's own row of them is gone. The
  rail's header draws Projects, Brain and the screens in one `min_w_0`, `overflow_hidden` group;
  every header button is `IconButtonShape::Square` at `IconSize::Small`, whose side `render`
  computes once, so `Rail::screens_that_fit` sums the group exactly from `Rail::width` and the rem
  size: all seven when nine buttons fit, else those that fit beside a `…` `PopoverMenu` whose
  `ContextMenu` lists the rest. The header is the title bar's height plus the 1px line the window
  draws under the title bar, and the filter row and the Brain view's search row are
  `Tab::container_height`, so both lines meet the main column's.
- **The vault cache** (`rusty.rs`). `Vault { tree }` is a global the Brain views observe, written
  only when a read differs (L-572); `VaultReads { wanted, reading, again }`, apart from it, keeps
  one `brain_tree` read in flight and one more queued. `want_vault` starts the reads when the first
  Brain view is made; from then on `reread_vault` runs on each connection, on each embedded
  `list_changed` (beside `read_settings`), after each write and on Refresh. The answer parses off
  the main thread into `marley_rusty::vault::VaultNode`; a read begun before the switch moved is
  dropped, and the tree clears when the source changes. `call_tool` calls a tool on Marley's
  connection with Rusty's own message as a refusal's error; `is_connected`, `is_on` and
  `unavailable` (the toast's words) give the connection's state; `vault_folder` is
  `Rusty.vault_root`, the folder `brain_stats` names (#791, read by `read_settings` after the
  settings and the embedding status, cleared with them), else `brain_vault_path` from
  `settings_list`, else `$HOME/.rusty/brain`, Rusty's own default, and `vault_here` is that folder
  when it is a folder on this machine, none when Rusty runs on another.
- **`BrainView`** (`rusty/brain.rs`, a `pub` module of `rusty`, so its `pub(crate)` items satisfy
  both `unreachable_pub` and clippy's `redundant_pub_crate`). Its search `Editor`, the last query
  and its hits (`Found`), the tree as last read, the open folders, the rows from
  `marley_rusty::vault::rows`, and the lines the `uniform_list` draws (`Line::Row`, `Line::Draft`
  for a name being made, `Line::Hit`); the selected path, an `Editing` (`NewPage`, `NewFolder`,
  `Rename`, its editor and a blur subscription that commits), the scroll handle and a menu
  deployed by hand at the pointer. The key context is `MarleyBrain menu`, with Zed's list actions,
  `menu::Confirm` and `menu::Cancel`, inside the rail's `MarleyRail menu`, so the view handles
  them first. Rows are `ui::ListItem`s with `indent_level` and a 20 px step, a chevron and an
  icon in the start slot, a folder's page count at its end, under `ui::indent_guides`; each sits
  in a `div` keyed by its vault path that drags a `DraggedVaultEntry` and takes its drop
  (`vault::move_target`). The tree's empty space takes a drop to the top and its own menu.
  Writes go through `write`: `call_tool`, then `reread_vault` and the follow-up on success, or a
  toast with Rusty's message. Delete asks with `Window::prompt` first. Search calls
  `brain_search` on Enter only (the embedding provider embeds every query). `open_page` is the one
  open: `Workspace::project_path_for_path(.., visible: false)` and `open_path_preview`, a preview
  where `PreviewTabsSettings` lets the project panel open one; #645 points it at its Page tab.
- **The action.** `brain::init` registers `marley::ToggleBrainView` on every workspace, deferred
  out of the dispatch: no rail, Zed's AI gate closed or `Rail::brain_refusal` give a toast;
  otherwise the sidebar opens first (opening reads the rail) and `Rail::toggle_brain_view` flips.
  The Marley keymap binds `secondary-alt-v` in `Workspace`.
- **The rail** (`rail.rs`). `BrainSide { view, entity, connected, _rusty }`: the chosen
  `RailView`, the `BrainView` made on first show and dropped when Rusty turns off, and whether Rusty
  was connected at its last change, written by an observer of the `Rusty` global that notifies
  only when that changes (F-438-a). `render_header` draws Projects and Brain `IconButton`s while
  connected (Projects carries an `Indicator::dot` while Brain shows and `has_attention` or the
  inbox says so), PROJECTS otherwise; the end is Add Project or New Page. `render` draws the Brain
  view in place of the filter, the inbox and the rows. `Focusable` stays the rail's own handle,
  which Zed keeps as the window's sidebar focus; `follow_focus` forwards focus given to it into the
  Brain view while that shows. `focus_filter` focuses the Brain view's search field, and the
  rail's `cancel` passes on while Brain shows.

## A brain page in a tab (`src/rusty/page.rs`, `src/rusty/properties.rs`, #645)

- **The actions.** `rusty::OpenPage { slug, preview }` (`#[derive(Action)]` in the `rusty`
  namespace, which `crates/zed/src/zed.rs`'s namespace test lists), and `actions!(rusty,
  [PageBack, PageForward, TogglePageEdit])`. `page::init` registers `OpenPage` on every workspace;
  with Rusty unavailable it shows `rusty::unavailable`'s words in a toast and opens nothing.
- **The opener.** `open_later(workspace, slug, preview, focus, window, cx)` runs in `window.defer`
  (a link's click arrives inside the `Markdown` entity's update; the opener reads every Page tab).
  A `PageView` showing `slug` comes forward (`activate_item`), kept when not `preview`
  (`unpreview_item_if_preview`); otherwise a new one is added to the active pane, taking the
  preview's place through `Pane::replace_preview_item_id` and `Workspace::add_item` at the index
  it gives, the two steps of `Workspace::open_project_item`. `rusty::brain::open_page`, the Brain
  view's one door, calls it.
- **`PageView`.** Its `PageHistory`, a generation counter, `Shown` (`Loading`, `Page`, `Missing`,
  `Failed`), a `markdown::Markdown` entity (the project's language registry, heading slugs and HTML
  on), its `ScrollHandle`, `Mode` (`Read`, or `Edit` with an `Editor` and its events), whether Rusty
  was connected, and a problem line. `load` calls `brain_render` through `rusty::call_tool`, reads
  the answer and runs `page_markdown` off the main thread, and drops an answer whose generation
  moved; `show` replaces the Markdown source and asks it to scroll to the visit's heading
  (`util::markdown::generate_heading_slug`). The tab re-reads on `rusty::Announced`, a global
  `rusty::connected` bumps on each embedded `list_changed`, while it shows Read.
- **Drawing.** The header (Back and Forward `IconButton`s, the folder and name, an Edit or Read
  `Button`), the not-connected and problem lines, then in Read a scroll container tracked by the
  `ScrollHandle`, the title as a `Headline`, `properties::render` (Ely's `DescriptionList` layout,
  its MIT notice on the file; a list as `ui::Chip`s) and a `MarkdownElement` with Zed's Preview
  style, a `link_callback` that mutes `rusty:new/` links, the same `ScrollHandle` for heading
  scrolls, and an `on_url_click` that defers to `follow` (`PageLink::parse`: navigate, toast,
  scroll, or `cx.open_url`). In Edit, the editor fills the body.
- **Edit.** `Mode::Edit` carries an `EditSource`. With a local project and `rusty::vault_here`,
  `toggle_edit` opens `brain_render`'s `file` (Rusty's TICKET-042), else
  `page_file_in(vault, slug)`, with `Project::open_local_buffer` into `Editor::for_buffer`
  (`EditSource::File`). Otherwise (#791: the vault on another machine, or a remote project) it
  makes `Buffer::local(raw)`, Markdown from the project's registry once it loads, in
  `Editor::for_buffer(buffer, None)` (`EditSource::Rusty { slug, began }`); an empty `raw` is
  refused, since a save would write the page over. `enter_edit` subscribes both: `BufferEdited`
  becomes `ItemEvent::Edit` (a preview tab is kept), the dirty, saved and title events update the
  tab. `save_edit` is the one save: the file through the editor's `save`; Rusty's through
  `write_through_rusty`, which renders the page, writes nothing when `raw` is no longer `began`,
  else calls `brain_write_page`, `did_save`s the buffer at the version it read and sets `began`
  to the text written; its failure goes in the tab's line (`could_not_save`, one full stop, as
  Zed's "Failed to save" prompt adds one to the same error after Ctrl-S). `leave_edit_then` saves
  a dirty edit with `SaveOptions { format: false, autosave: true, .. }` before Read, Back,
  Forward or a link; a failed save stays in Edit and says why.
- **`Item`.** `tab_content` draws the title (the slug's last part until it loads) in italics while
  `TabContentParams::preview`; `FileMarkdown` icon; the slug as tooltip; while in Edit,
  `is_dirty`, `has_conflict`, `can_save`, `save`, `reload` and `for_each_project_item` are the
  editor's; `added_to_workspace` keeps the workspace (L-613). `Focusable` is the editor's handle in
  Edit. Marley's keymap binds Alt-Left and Alt-Right in `RustyPage`.

## A page's outline and edits in place (`src/rusty/page.rs`, `src/rusty/inline_edit.rs`, #656)

- **The outline.** In Read, `render_read` puts the scrolling body and, while `outline_shown` and
  the page has headings, a column of `ListItem`s beside it (the body `min_w_0` and clipped in x
  since #663, so a line that cannot wrap keeps the column in view): one per `RenderedPage.outline` entry,
  indented by its level less the shallowest, labelled by `outline_label`. A click runs
  `scroll_to_heading_line`: `line_offset` on the Markdown's source, past the heading's marks, then
  Zed's `Markdown::request_autoscroll_to_top` (the `markdown.rs` touchpoint), which puts that line
  three lines under the top of the tab's `ScrollHandle`. `rusty::TogglePageOutline` and the
  header's `IconButton` (`ListTree`) flip `outline_shown`; the button is disabled with "No
  headings" for a page with none, and in Edit. In Edit, `act_as_type` hands Zed the tab's
  `Editor`, so Zed's outline panel lists the file's headings; the `UpdateTab` each Read and Edit
  change emits is the `ActiveItemChanged` the panel looks again on.
- **The in-place editor.** `inline_edit.rs` is Ely GPUI Components' `InlineEdit`, under Ely's MIT
  notice, on Zed's single-line `Editor`: `shown` (the value with a hover background and a pencil
  on hover), `editor_for` (the text, all selected) and `field` (the editor and a red line for
  what the text needs). The tab holds one `InlineEditing` (its `EditTarget`: `Title`, `Name`,
  `Value`, `ListItem` or `NewKey`, the editor, an error, and a blur subscription). The field sits in
  a `key_context("menu")` div, so the editor's Enter and Escape, which the editor lets through,
  reach the tab as `menu::Confirm` and `menu::Cancel`. A blur keeps the text while the window is
  active. Nothing opens unless `can_edit`: connected, in Read, with a page shown.
- **The commits.** `commit_title` writes `title`; `commit_name` sends `Rename` with
  `vault::rename_target`; `commit_value` parses by `PropertyKind` and keeps the editor open with
  the parse's words on a refusal; `commit_item` and `remove_item` write the whole list;
  `toggle_checkbox` writes the other boolean; `remove_property` sends `Remove`; `commit_key`
  refuses a key the page has and writes the kind's `empty_value` (a date takes today). Each
  shows its value at once (`set_local`) and queues the write.
- **Properties.** `properties.rs` keeps #645's layout as `row` (key, value, the row's end, the
  hairline) and `value`; `render_properties` builds each row's value by kind (an editable text,
  a `Checkbox`, chips with remove buttons and an add button, or `properties::value` for an object
  or a mixed list), the row's remove button, and Add property's `PopoverMenu` over
  `PropertyKind::ADDABLE`, whose entry defers to the window before opening the key's editor.
- **The write queue.** `Writes` holds `(slug, Write)` pairs (`Set`, `Remove`, `Rename`) and sends
  them one at a time through `rusty::call_tool` (`pump`). A failure shows Rusty's message in a
  toast. With the queue empty, the tab reads the page again once. A change Rusty announces while
  a write is out waits for the last answer (`read_after`).
- **Renames.** The `PageViews` global lists every Page tab. A rename's answer (`RenameReport`)
  moves the tab's own `PageHistory` and toasts `renamed_words`; `follow_rename` then defers to the
  window and calls `renamed` on every Page tab, which moves its history and reads the page again
  when the one it shows moved. The Brain view's rename and move call `follow_rename` too.

## The Tasks tab (`src/rusty/tasks_tab.rs`, #658)

- **Opening.** `rusty::OpenTasks`, registered on every workspace by `tasks_tab::init`;
  `open_later(workspace, list, ..)` defers, then brings the workspace's `TasksView` forward or adds
  one, and with a list calls `show_list`. The Brain view's fixed row (Tasks after Graph, through
  the rail's multi-workspace) and the Knowledge panel's `tasks_header` (Open in Tasks on the first
  joined group) call it.
- **`TasksView`.** The lists, the chosen list, its tasks, the selection, Show archived, the add
  field, a row editor (`RowTarget`: new list, a list renamed, a task renamed), the write queue,
  the read in flight and `ReadDue` (no, after this one, when shown), `State` (reading, ready,
  failed), `Link` (off, down, up), a hand-deployed `ContextMenu`, and a `ScrollHandle`.
- **Reads.** `list_task_groups`, then `list_tasks` for `kept_list`'s choice with
  `include_archived`; the tasks parsed off the window's thread; a read for a list no longer
  chosen reads again. Triggers: opening, the link coming up (`rusty_changed` acts only on a change
  of `Link`, since the `Rusty` global notifies for more), `Announced` while `showing` (else
  `WhenShown`, read at the next draw), `on_focus_in` of the root from outside, window activation
  while showing, Refresh, after each write, and `deactivated` under the service connection.
- **Writes.** `write` queues a `TaskWrite` (refused with a toast while not connected); `pump` sends
  one at a time through `rusty::call_tool`, chooses a new list by its answered id, toasts a
  refusal's first line, and reads after each. A reorder is drawn before it is sent. The deletes ask
  with `window.prompt` at Warning, the name fenced by `launch::verbatim`.
- **Drawing and input.** A 240 px lists column (`ListItem`s, a menu each, + for a new list), the
  header (the name, Show archived, Refresh), the add field (read only without a list), the rows
  (`ListItem` with a filled `Checkbox`, the title struck and muted when done, archived faint with
  "Archived"), and the notice line. A row drags as `DraggedTask` with the rail's `drag_preview`
  and `drop_line` (`crate::rail::order`, `pub` since #658) and drops by `marley_rail::move_to`.
  The root is `RustyTasks menu` (Zed's list keys, `menu::Confirm` and `Cancel` by where the focus
  is); the list is `RustyTaskList`, with `not_editing` while no row editor is open, for the
  keymap's Space, F2, Delete, Backspace, Shift-Delete, Alt-Up and Alt-Down.

## The Decisions tab (`src/rusty/decisions_tab.rs`, #659)

- **Opening.** `rusty::OpenDecisions`, registered on every workspace by `decisions_tab::init`,
  toasts `rusty::unavailable`'s reason and opens nothing while Rusty is off; else it brings the
  workspace's `BrainDecisionsView` forward or adds one to the active pane. `open_later` defers that
  for the Brain view's fixed row (Decisions after Tasks, through the rail's multi-workspace).
- **Reads.** `brain_due { days: 0 }` through `rusty::call_tool`, parsed off the window's thread by
  `marley_rusty::decisions::parse_due`. Triggers: opening, the link coming up (`Link` as the Tasks
  tab keeps it, L-658), `Announced` while the tab is its pane's active item (else `ReadDue`'s
  `WhenShown`, read at the next draw), Read again, and `deactivated` under the service connection.
  A read while one runs queues one more (`AfterThis`). Rusty off drops the list and calls nothing.
- **Drawing.** The title and Rusty's line on the loop, then one state line (off, not connected
  with the reason, a failure's first line with Read again over the list kept, reading, or none
  yet), then `decisions::entries` in a scrolling column: Due and the count as `ListSubHeader`s,
  each in a `flex_none` box since the header carries `flex_1` and would grow in the column, and
  each decision a `ListItem` (the title cut, a status `Chip`, muted for superseded, the decided
  line, the follow-up line in the warning colour when Rusty flags it overdue, a tooltip with the
  title and slug) that opens the page through `page::open_later`. Nothing works out a date: the
  order, the horizon and `overdue` are Rusty's.
- **Since #660, the rows** (`render_end_slot`) add `followed_up_line` after the decided line, and
  on a superseded row with a successor "replaced by" and a `Button` with the successor's title
  (from the list, else its slug) that opens that page; a `Button`'s click stops there, so the row's
  own page stays shut. A Due row that is not superseded ends with a Follow Up `Button` while
  connected. A right-click deploys a `ContextMenu` by hand, as the Tasks tab's: Follow Up… (not on
  a superseded row, not while disconnected) and Open Page.

## The follow-up form (`src/rusty/follow_up.rs`, #660)

- **Opening.** The tab's `open_follow_up` hands `follow_up::open` the summary and every other
  decision as a `Candidate` (slug, title), deferred into the workspace's `toggle_modal`.
- **`FollowUpModal`** holds the status (`Option<FollowUpStatus>`), the successor's index, the
  outcome (`Editor::auto_height(3, 8)`), the day (`Editor::single_line`), the successor picker
  while it shows, the call in flight, Rusty's refusal and a refocus flag. Its `draft()` reads the
  editors into `marley_rusty::decisions::FollowUpDraft`; `missing()` gives Record's hint and
  whether it is enabled, `arguments(slug)` the call's.
- **Drawing.** The title and the decision's follow-up line; `ToggleButtonGroup::single_row`,
  `Outlined`, with an index past the three while none is chosen; Replaced by for Superseded (the
  embedded picker, or the choice and Change…); the outcome; Next follow-up for Revised; the
  refusal in the error colour; the hint and Record (Recording… during the call). The form is
  `FORM_WIDTH` (34 rem) wide, and the picker opens at that less the padding, since a `Picker`
  opens at Zed's modal width.
- **Keys.** The root's context is `RustyFollowUp menu`: Enter in either editor falls through to
  `menu::Confirm`, which records (the auto-height editor binds no Enter; Shift+Enter and
  Ctrl+Enter are its newlines), and `menu::Cancel` closes. The picker's own keys win while it
  shows; its Escape and its pick come back through `cx.defer`, and the next draw focuses the
  outcome. A status click focuses the outcome too. The form watches the outcome's
  `BufferEdited` only, not every notification of the editor.
- **Recording.** `rusty::call_tool(BRAIN_FOLLOW_UP, ..)`; on `Ok` the tab's `read_again` and
  `DismissEvent`; on `Err` the first line stays as the refusal. Closing the form during a call
  drops the task; Rusty may still record it, and the tab shows it at its next read.

## Open a page by name (`src/rusty/page_picker.rs`, #654)

- `OpenPage.slug` is an `Option` (its fields read through `OpenPageFields`, `slug` defaulted), so
  gpui builds the action from `{}` and the command palette lists `rusty: open page`. `page::init`'s
  handler checks `rusty::unavailable` first; a slug goes to `open_later`, none to
  `page_picker::toggle`. The Marley keymap binds `secondary-alt-u` to it in `Workspace`.
- `toggle` shows the shortcut note when a terminal had the focus (`shortcut_note::taken`), reads the
  active Page tab's slug, and opens `PagePicker` with `Workspace::toggle_modal` (a second press
  closes it): `Picker::uniform_list` over `PagePickerDelegate`, key context `RustyPagePicker`.
- The delegate reads `brain_list_pages { limit: 100000 }` once per open through
  `rusty::call_tool`, parses it off the main thread and refreshes the picker (`List::Reading`,
  `Read`, `Failed` with the first line). An empty query takes `switcher::empty_order`; any other
  runs two `fuzzy_nucleo::match_strings_async` calls, over titles and over slugs, merges them
  (`switcher::merge`, 100 rows) and appends `switcher::create_target`'s row. Rows draw
  `IconName::FileMarkdown`, the title and the slug as `HighlightedLabel`s.
- Confirming a page dismisses the picker and calls `open_later(workspace, slug, false, true, ..)`.
  The create row calls `page::create` (`brain_new_page` with `NewPage::arguments`, the slug parsed
  with `vault::slug_from_answer`), the row reading "Creating…" until Rusty answers: then the picker
  closes and the page opens kept, with a toast when Rusty named another slug; a refusal shows in
  a toast and the picker stays.
- `Recent`, a global, holds `RecentPages`, read at `rusty::init` from Zed's key-value store
  (`marley-rusty-recent-pages`, key `pages`); `opened(slug)` puts a slug first and writes the list
  in the background. `page::open` (every opener), and a Page tab's `navigate`, `back` and
  `forward`, call it.
- `PageView::follow`'s `PageLink::Missing` arm makes the page (`create_linked`: `NewPage::from_target`,
  `page::create`) and navigates to the slug Rusty returns; a refusal shows in the tab's toast.
- Since #662 the delegate keeps the file bookmarks' slugs (`favourites::list`) for
  `switcher::empty_order`, draws a separator after each index `Order::separators_after` names, and
  puts a filled star in a favourite row's end slot. `toggle` calls `favourites::ensure`.

## Favourites (`src/rusty/favourites.rs`, `src/rusty/brain.rs`, `src/rusty/page.rs`, #662)

- **The list.** `Bookmarks`, a global the Brain view and every Page tab observe, holds Rusty's list
  as last answered, or nothing while unread or Rusty is off. `BookmarkReads` (observed by nothing)
  holds whether a view has asked (`ensure`), the read's state (`Reading`: idle, running, running
  with one more queued) and the connection as last seen, so a read follows the connection coming
  up and not every notification of `Rusty` (L-658). `init` reads again on `Announced` and when the
  connection comes up, and drops the list when Rusty turns off. `keep` sets the global only when
  the list differs.
- **Writes.** `write(&BookmarkWrite, workspace, cx)` calls the tool and keeps the list it answers;
  a refusal's first line goes to the workspace's toast and the list is read again. `toggle_page`
  adds the page or removes its bookmark; `retitle` sends `bookmark_set` with
  `marley_rusty::bookmarks::retitled`. The Brain view's own writes (rename, move, delete) read the
  list again after Rusty answers, since the service connection announces nothing and Rusty
  carries or drops bookmarks on those.
- **The Brain view.** `render_favourites` draws nothing for an empty list, else an inset
  `ListSubHeader` (in a `flex_none` div, F-659) and a `ListItem` per bookmark with the kind's icon
  (`Folder`, `MagnifyingGlass`, `Hash`, `FileTextOutlined`) and the path, query or heading in its
  tooltip. `favourite_clicked`: a page opens as a tree row does; a folder clears a search, opens and
  `reveal`s; a search sets the field's text and runs `search_for`; a heading calls
  `page::open_at_heading_later`. The right-click menu has Rename… (`Edit::BookmarkTitle { key }`,
  the inline editor typed in the row; `start_edit` leaves the search and the tree's scroll alone
  for it) and Remove.
- **The Page tab.** `render_star_button` between the name and Edit: `Star` or `StarFilled` in the
  accent colour, each with its own element id so a click drops a tooltip built for the other
  state. `ToggleBookmark` (`rusty: toggle bookmark`, `ctrl-d` in `RustyPage`) does what the star
  does. `open` takes a `Visit`, so a tab already showing the page goes to the heading
  (`go_to_heading`); `show` lands a visit's heading through `outline_line` and
  `scroll_to_heading_line` when the outline holds it.

## Capture and import (`src/rusty/capture.rs`, `src/rusty/import.rs`, #663)

- **Commands.** `capture::init` registers `CaptureToToday`, `CaptureToInbox`, `CaptureUrl` and
  `OpenToday`, and `import::init` `ImportVault`, on every workspace; each first calls
  `capture::ready`, which shows `rusty::unavailable`'s reason in a toast. The actions carry
  `#[derive(Eq)]`, as `tasks_tab`'s do, for clippy's `derive_partial_eq_without_eq`.
- **`CaptureForm`** (a `ModalView` through `Workspace::toggle_modal`, key context `RustyCapture
  menu`): a `Kind` (a line for a `CaptureTarget`, or a URL), one `Editor::single_line`, the task in
  flight and Rusty's refusal. `menu::Confirm` sends what the field holds, as typed. A line goes by
  `call_tool(BRAIN_CAPTURE)`; its receipt rereads the vault, toasts "Captured to `<slug>`" with
  Open (`page::open_later`) and `autohide`, and dismisses. A URL goes by
  `call_tool_within(SOURCE_CAPTURE, CAPTURE_URL_DEADLINE)`; the page opens kept and focused,
  `SourcePage::failed` toasts the error without autohide, and the form dismisses. A refusal's first
  line stays in the form.
- **`open_today`** calls `brain_daily_note` from the workspace's own spawn and opens the slug with
  `page::open_later`; a failure toasts through the workspace it is updating.
- **The import.** `choose_folder` asks `Workspace::prompt_for_open_path` for one directory with
  `DirectoryLister::Local`, since Rusty reads its own machine's disk, then opens `ImportForm` on
  the path. Its `Stage` runs Reading (`brain_import_plan` within `IMPORT_PLAN_DEADLINE`), Plan,
  Importing (`brain_import` within `IMPORT_DEADLINE`), Done or Failed. The form draws the path
  truncated from its start, the stage's sentence (`ImportPlan::summary`, `ImportReport::summary`),
  the plan's `details` in a scrolled block, and the stage's buttons; `menu::Confirm` imports a plan
  that `brings_anything` and closes a report. `on_before_dismiss` answers `Dismiss(false)` while
  Importing, so neither Escape nor a click outside drops the import's answer. Done rereads the vault
  and the bookmarks (AD-662).

## The Memory tab (`src/rusty/memory_tab.rs`, #664)

- **Opening.** `OpenMemory` on every workspace, and the Brain view's Memory button (`Book`) through
  `open_later`, bring the workspace's one `BrainMemoryView` forward or add it to the active pane;
  with Rusty unavailable, a toast says why.
- **Reads.** The Decisions tab's `Link` and `ReadDue` (now `pub(super)` there): a read when it
  opens, when the link comes up, on `Announced` while it shows (else when it next shows, AD-609),
  after each write and on Read again; off drops the list. `list_memories` is drawn in Rusty's
  order; a filter whose category went away falls back to All.
- **Drawing.** The title and Rusty's line; while connected, the add row: the content editor
  (`flex_1`), the category editor (10 rem), and the importance `ToggleButtonGroup` in a 16 rem box,
  since the group fills its parent; then the state line, the count and the Category
  `DropdownMenu`, whose `ContextMenu` is keyed by the categories and the choice; then the rows
  (`ListItem`: the content `line_clamp(3)`, a `Chip` for the category, importance, source and the
  day in this machine's zone through `chrono::Local`).
- **Writes.** `menu::Confirm` in `RustyMemoryAdd menu` stores the content (a refusal toasts). A row
  opens `MemoryForm` through `window.defer` and `toggle_modal`: an auto-height content editor, the
  category, the importance group (none chosen for an older word), Delete through `window.prompt`,
  Save while the content holds text. A success has the tab read again and dismisses; a refusal's
  first line stays.

## The Skills tab (`src/rusty/skills_tab.rs`, #665)

- **Opening and reads.** `OpenSkills` and the Brain view's Skills button (`ToolHammer`) bring the
  workspace's one `BrainSkillsView` forward; the Decisions tab's `Link` and `ReadDue` drive the
  reads, `skill_list` and `script_list` (both with the staged ones) read together and kept as one
  answer, the skills `ordered`.
- **The choice.** `Chosen` is a skill by name or a script by path. Choosing fills the editors (the
  description `single_line`, the body `multi_line`, which fills the right side); a script's text
  comes from `script_view` on a task of its own, dropped if the choice moved on. A read that no
  longer lists the choice clears it; one that does leaves the editors as they are, so a read never
  overwrites an edit in progress.
- **Writes.** `send` calls the write's tool and `answered` sets the notice, the findings after
  `skill_update`, `blocked` after an Approve refused by `blocked_by_scan` (which shows Approve
  Anyway), and clears the choice after Delete or Reject; every write reads again. Delete asks
  through `window.prompt`. `NewSkillForm` hands the skill Rusty answers to the tab, which lists
  and chooses it before its read lands.
- **Run.** `run_script` checks the project is on this machine, then schedules a `TaskTemplate`:
  `bash` with the shell-quoted path, the script's folder as the directory, label `rusty <name>`,
  `RevealTarget::Center`, the summary shown and the command hidden (Zed's summary would leave out
  the path). Marley's terminal routing puts it in the center; no process starts in this crate.

## The Secrets tab (`src/rusty/secrets_tab.rs`, #667)

- **Logs.** `rusty::init` calls `context_server::client::log_messages_by_size` for `MARLEY_SERVER`
  (`marley-rusty`) and `CONTEXT_SERVER` (`rusty`), so Zed's MCP client logs their messages by size
  (the `context_server` touchpoints). The tab itself logs only `secret_lock`'s answer.
- **Reads.** The Decisions tab's `Link` and `ReadDue`: `secret_list` and `secret_pin_status` read
  together; off or a lost connection forgets the token.
- **The token.** `secret_unlock`'s token lives in `BrainSecretsView::token` alone, with an `expiry`
  task that locks after `expires_in_seconds`. `lock` drops the token, the revealed value, the
  replace target and the timer, and sends `secret_lock`; it runs on Lock, on expiry, on
  `observe_window_activation` reporting the window inactive, and from `on_release` when the tab
  closes. `can_write` is true with no PIN, or with the token.
- **Fields.** `Fields` holds the PIN, again, unlock PIN, key, value and replace editors, the PIN and
  value ones `set_masked`; each is emptied as its text is sent. Each row's `RustySecret menu`
  context turns Enter into its action.
- **Writes.** `send(&SecretWrite, done)` carries the token, sets the notice to `done` or Rusty's
  refusal, and reads again; Delete asks through `window.prompt`; Copy writes the clipboard; Reveal
  toggles `revealed`.

## The project view (`src/rusty/project.rs`, #655)

- `ProjectPages`, a global the panel and the Graph tab observe, holds every project page, or the
  read's failure, written only when it differs; `ProjectReads` keeps the read in flight, one
  queued, the delayed second look and the pages an older Rusty gave by read. `ensure` starts the
  first read (from a panel shown in its dock, or `rusty: open graph`); `Announced` re-lists now and
  6 s later. The list is `brain_list_pages { page_type: "project", limit: 1000, properties: [path,
  task_group, summary] }`; a summary without `properties` is read with `brain_read_page`, eight
  at a time, again only when its `updated_at` moved.
- `project_of(workspace)` is the project group's folders (`Workspace::project_group_key`) and
  whether its host is remote; `join` resolves it over the cache (`marley_rusty::project::resolve`,
  the home folder from `util::paths`), `project_page` gives the slug for the Graph tab.
- `link_page(workspace, slug)` adds the folders to the cached page's `path` with
  `path_value_with` and writes it with `brain_set_property`; `write` reads the pages again after,
  and a refusal goes to a toast. `rusty::LinkProjectPage` and `rusty::LinkTaskGroup` (each with
  `unavailable`'s toast, a "no folder" and a "link a page first" toast) open `LinkPicker`, Zed's
  `Picker` over choices matched with `fuzzy`, its pick a closure run after the picker's update.
  Since #660 `LinkDelegate` takes its pick and its close as callbacks, and whether a pick also
  closes, so the follow-up form embeds the same delegate for a decision's successor.
- In the Knowledge panel, the no-page place draws the project view from the panel's `join`:
  `render_project_page` (title, slug and how it matched, Open Page, the others that list the
  folder, the summary, Follow-ups due, Tasks · GROUP or Link a Task Group), `render_candidates`
  (Link per row), `render_unmatched` (`ui::Callout` with Link a Page). `refresh_join` runs on the
  cache's change, on the project's folder events (deferred), when no Page tab is in front and when
  the dock shows the panel (`Panel::set_active`, deferred: the dock calls it inside the
  workspace's update). `load_project` reads the page, its links, `brain_due { days: 0 }` and the
  task groups, then `list_tasks` for the groups it joins, coalesced as the page view's reads.
- The Graph tab keeps `project_page` beside `page`; `centre()` is the page last in front, else the
  project's; the header marks a project centre "(project)"; `open` starts Local when either
  exists and says "Open a page first, or link this project to its page in the Knowledge panel."
  with neither; `go_project` turns an open tab to it.

## The Knowledge panel (`src/rusty/knowledge_panel.rs`, #646)

- **The panel.** `KnowledgePanel`, a `workspace::Panel` in the right dock only (persistent name
  `MarleyKnowledgePanel`, `IconName::Book`, activation priority 21 after the Fleet panel's 20,
  320 px). `knowledge_panel::init` (from `rusty::init`) registers `rusty::ToggleKnowledgePanel` on
  every workspace and adds the panel to each one that has a window. It is always added and hides
  itself as the Agent Panel does: `enabled` and `icon` read `rusty::is_on`, so `PanelButtons`
  draws no button while Rusty is off; the toggle shows `rusty::unavailable`'s words in a toast
  instead. A panel that renders while off asks once (`closing`) for a `window.defer` that closes
  the right dock only when its `visible_panel` downcasts to this panel (PR-607; `close_panel`
  would close the dock whatever it shows).
- **Following.** A `subscribe_in` on the workspace's `ActiveItemChanged` (the outline panel's way)
  reads the active item; a `PageView` gives its slug, and a subscription to its
  `PageEvent::UpdateTab` follows the tab's own navigation. Any other item shows the no-page line.
  The first look waits for `cx.defer_in`, since the workspace is mid-update while panels are made.
- **The page view.** `load_page` joins `brain_get_links { slug }`, `brain_graph { around: slug,
  depth: 1 }` and `brain_tags` through `rusty::call_tool` and builds a `marley_rusty::knowledge`
  `PageKnowledge`; one read in flight, one more queued (`Reads`), an answer for a page no longer
  shown dropped. Tags are `ui::Chip`s inside a clickable `div` (a chip takes no click) that put
  `tag:<name>` in the field and search; backlinks and links are `ListItem`s, a backlink's line a
  `HighlightedLabel::from_ranges` over its mention; an unresolved link's Create button stops the
  row's click and calls `brain_new_page { path, folder, name }`, then opens the slug it answers.
  Opening goes through `page::open_later` as a preview, without moving the focus.
- **Search.** A single-line `Editor` and two `IconButton`s (`CaseSensitive`, `Regex`, square,
  `toggle_state`), as the search bar draws its options. The container's key context is
  `KnowledgePanel menu`, so the field's Up, Down, Enter and Escape arrive as `menu` actions (L-457).
  `confirm` sends a new query, or new options, as `brain_search { query, limit: 60,
  case_sensitive, regex }`; the same query again opens the selected hit, else the first. `Results`
  holds the query and options it was asked with, and an answer for another is dropped. A toggle
  gives the field its focus back and asks again. `cancel` clears the field and the results, or
  propagates when there is nothing to clear. Snippets are drawn with `knowledge::snippet`'s ranges.
- **Live and states.** `rusty::Announced` reads the page and the shown query again; the `Rusty`
  global re-reads a page whose read failed once the client connects; while not connected the body
  is the client's state line, and a failed read shows its error in place of the sections.

## The Graph tab (`src/rusty/graph_tab.rs`, #647)

- **Opening.** `actions!(rusty, [OpenGraph, OpenLocalGraph])`, registered on every workspace by
  `graph_tab::init` (from `rusty::init`); with `rusty::unavailable` they toast and open nothing.
  `open` brings the workspace's one `GraphView` forward (`items_of_type`, `activate_item`) or adds
  one to the active pane, Local when a Page tab is in front, else Vault; with `local` it turns the
  tab to the page in front or the page it last followed, or toasts "Open a page first".
  `open_later` defers it for the rail's Graph entry (`brain.rs`'s fixed row, after Today,
  `IconName::GitGraph`), which reaches its workspace through `multi_workspace`.
- **Following and reading.** A `subscribe_in` on the workspace's `ActiveItemChanged`: a Page tab in
  front sets `page` (and its `PageEvent::UpdateTab` follows it as it navigates), clearing the kept
  places of a local graph; the tab itself in front reads when the graph held is not the one wanted
  (`ReadKey { around, depth, unresolved }`), or always when a change came while it was hidden or the
  connection is the service one (`Rusty::source`). `rusty::Announced` reads at once while the tab
  is its pane's active item, else marks it stale. One read in flight, one more queued; the JSON is
  parsed on the background executor; an answer for a key no longer wanted is dropped; a failure
  shows in the header with Read again and keeps the last graph. `brain_page_types` is read once.
  The `Rusty` global: off drops the graph, its places, the run and the read (no call is made);
  connected reads.
- **The layout's run.** `rebuild` records each shown node's place by id, computes `graph::shown`,
  seeds a `Layout` from the places kept and starts the run: `start_run` takes the layout out of the
  view and lends it to `cx.background_spawn(futures::future::lazy(..))` for a batch of about 16
  million pair checks (eight steps at the cap; a batch waits on the window's thread for the frame
  under way, so small ones cost frames, not work), gets it back with its places, applies the pins
  and releases made meanwhile, redraws, and goes on until it settles, when it logs `graph layout:
  N nodes, E edges, S steps in B batches, W ms of work in T ms` and gives the layout back. A rebuild drops the task, so a replaced run stops at its next
  batch. A new tab, scope or centre fits the view once settled; the Fit button fits on the next
  frame (the canvas's bounds, kept in an `Rc<Cell<Bounds>>` its prepaint writes).
- **Drawing.** One `canvas`: edges batched into a `PathBuilder` per line kind and emphasis,
  clipped to the view (Liang and Barsky) and drawn and begun again before 8,000 segments or
  dashes, so no path passes gpui's `u16` index limit; the typed edges dashed (5 and 4 px) in the
  status colours info, warning and success; links in the border colour, lit in the accent text
  colour around the hovered node and faint elsewhere (Ely's rule). Nodes are round quads edged in
  the background colour (Ely's `ring`), the page types in `accents().color_for_index` by
  `type_order`, tags in the hint colour, unresolved hollow, the centre ringed in the accent;
  their radius Rusty's 3 + 1.6√degree, at least 2.5 px. Labels are shaped and painted in the
  canvas: past zoom 0.9, fading in to 1.3, the 200 most linked in view, and always the hovered
  node, its neighbours and the centre.
- **Input.** A press on a node holds it once the pointer moves past 3 px (a `Layout::pin`), else it
  is a click (`page::open_later(…, false, true, …)` for a page, the filter for a tag); a press
  elsewhere pans. The local graph's centre stays pinned where it is dropped. The wheel zooms about
  the pointer by 1.05 a line (gpui reports a notch as three lines), a pinch by its delta.
- **The panel.** Local and Vault buttons, Depth 1 to 4 (shown in Vault too, off, so the rows
  keep their places), a single-line `Editor` whose edits re-parse
  the `Query`, four `Checkbox`es (Tags, Unresolved links, Decision edges, Orphans), the legend (a
  dot, the type and its count, faint and toggled on a click; the edge kinds shown, a solid or dashed
  sample), and Restart layout, Fit and Hide panel. `Item`: "Graph" or "Local graph",
  `IconName::GitGraph`, no toolbar.

## The Graph tab's settings and restore (`graph_tab.rs`, `graph_store.rs`, `slider.rs`, #657)

- **The record.** `graph_store::GraphSettingsStore`, a global holding `marley_rusty`'s
  `GraphSettings`: read from Zed's key-value store (scope `marley-rusty-graph`, key `settings`)
  at `init`, before any window restores, each fallback logged; `update` applies a change, skips one
  that changes nothing, replaces the global (`set_global` notifies its observers) and a pending
  write that waits 300 ms; `on_app_quit` writes it once more. The four switches and the depth live
  here, so every Graph tab shares them, as Rusty's two graph tabs share one record.
- **The tab's row.** `marley_rusty_graph_tabs(workspace_id, item_id, state)`, keyed by the pair
  with no `UNIQUE(item_id)` (#576), `state` a JSON `SavedGraphTab` (local, page, filter, hidden
  types, panel open, `Sections`). The rows are read into the `SavedGraphTabs` global at `init`;
  `save_tab` updates it and writes the row; `cleanup` drops the unloaded items from both.
  `impl SerializableItem for GraphView` (`MarleyRustyGraph`): `deserialize` refuses while Rusty is
  off ("Rusty is off; the Graph tab is not restored", logged by Zed's loader), reads the memory
  copy and builds the tab with the same `GraphView::new` as `open`, a deferred `follow_project`
  finding the project's page; `serialize` on each `GraphEvent::UpdateTab`, which every change of
  the tab's own state emits. `added_to_workspace` re-points the workspace and its subscription.
  A restored tab reads once #647's `Rusty` observer sees the connection up.
- **The tab.** `GraphView` keeps a copy of the record; `settings_changed` (its observer) does only
  what changed: the group fields and `coloring`, a rebuild for a switch, a read for the depth or
  Unresolved links, `Change::Forces` through the run's `pending` for a force, a repaint for
  Display. The panel's controls write the store from their listeners. `node_colour` takes a
  group's terminal hue before the type's accent; radii and the hit test take Node size; every
  stroke takes Link thickness; labels fade by `label_alpha`; `paint_heads` fills the heads per
  line style, skipping an edge with a tag at either end and a head whose target is off the
  canvas. The run line ends with the forces. The panel `occlude`s the canvas under it, so a press
  in it never reaches the canvas, which would take the focus from a slider.
- **The panel's sections.** Under the switches, Groups, Display and Forces, each a header row
  with a `ui::Disclosure` (folded on a new tab), then the legend. Groups: a swatch (Next colour),
  a single-line `Editor` per group (its edits write the query; another tab's change sets an
  unfocused field's text when it differs), the count, Remove group, and New group (whose field
  takes the focus). Display: the Arrows `Checkbox` and three slider rows; Forces: four. A row is
  the name and the value over a `Slider`.
- **The slider** (`slider.rs`, Ely GPUI Components' `Slider`, its MIT notice on the file): one
  thumb, horizontal, `f32`. Keyed states hold the drag's owner, the thumb's focus handle
  (`tab_stop`) and the track's measured bounds. The rail is in `border` and the fill in
  `text_accent`; the thumb is `elevated_surface_background`, edged in `border` or
  `border_focused`, with `Role::Slider` and its aria values. A press on the track jumps there and
  takes the focus, a drag sets the value under the pointer (owner checked), and the keys step,
  jump ten and go to the ends. A change under half a step is none, and a slider whose range or step
  makes no sense does not move.

## The find tools (`src/find.rs`, #567)

- `find_items(name, subject, query, items, place, cx)` is what `browser_find` and
  `terminal_find` share. The query's words come first (`marley_mcp::find::local`): an item that
  alone holds them is the answer, `source: rules` and sure, with no call. Otherwise the use's mode
  decides (`system_one::use_mode`); `system_one::detail` is read before any ask, so an unlisted or
  metadata-only project answers by the words with a note and makes no call. The items asked about
  are the words' matches when there are several, else every item, in windows of `FIND_WINDOW`: a
  window's `Asking` is its `query` line and its items as `1: …` to `N: …`, and its `ask` a
  `UseSpec` of the tool's name, `find_set(N)` and 3 s. Shadow detaches the asks and answers by the
  words.
- `read_windows` joins the windows: found when a window's `present` reads yes, absent when every
  window's reads no, else unsure; the candidates are the top three by the parsed probabilities
  across windows; in `act` the item to act on is the model's confident choice when it found one.
  A refused, unavailable or `rules`-provider window counts as no answer, and with none the words
  answer, with the reason as the note.
- `browser_tools::find_query` checks `query` (200 characters), and `found_text` writes an answer
  as text: the item to act on or why there is none, the candidates, the note, and where to read.

## Projectless groups (`src/groups.rs`, #600)

- A group is a record in the global `Groups` (an id, a name, its folderless workspace, that
  workspace's project entity id, `expanded`, and whether it is the window's Home group), not a
  Zed project group: Zed makes none for a workspace with no folder, and every such workspace has
  the same empty key. `make` opens one with `Workspace::new_local(Vec::new(), …, OpenMode::Add)`
  in the window, records it once it exists, and runs its follow-up through `AnyWindowHandle` so
  the window's `MultiWorkspace` stays free for it. After every change `keep` sets
  `workspace::MarleyKeptWorkspaces`, which `MultiWorkspace::open_project` reads so it does not
  replace a shown group (a `workspace` touchpoint).
- The rail's `rail_groups` appends a `ProjectGroup` per group of the window (the empty key, its
  own workspace) after Zed's (then sorts every header by #602's saved order), so rows, focus and the attention order need nothing new;
  `GroupEntry.group` marks them for the header's icon (`header_icon`), its fold (the record's),
  its menu (`HeaderMenu`: Rename Group…, Remove Group) and the `+` (no New Agent Thread).
  `group_threads` returns nothing for a key with no folder, and `Ports::of` finds nothing for it.
- The empty space's menu is deployed by hand: a filler under the rows (`render_rows`) and the
  header's spacer take a right mouse-down, stop it and prevent its default (the rail's own focus
  would take the keyboard from the menu), and the rail draws the menu anchored and deferred.
  `in_home` finds or makes the window's Home group and runs the item there.
- `BrowserProject::of(&Entity<Project>)` keys a group's project by `service::group_key(id)`,
  found by `groups::group_of_project` from the recorded project id: the browser asks while the
  group's workspace is being updated, so the lookup reads no workspace. `live_projects` keeps a
  group's Chromium through `of_workspace`, and Remove Group lets `review_browsers` stop it.

- Since #601 groups survive a restart. `groups::init` reads every group's record (`SavedGroup`:
  its workspace's database id, its id, name, Home flag and fold) from the key-value scope
  `marley-groups` at startup into `pending`, and `save` writes them after every change. The
  rail's saved sidebar blob keeps its window's groups' workspace ids as `marley_groups`
  (`write_rail_groups`, `read_rail_groups`, beside `RailState`); `restore_serialized_state`
  defers `groups::reopen`, which opens each id the window does not hold through
  `workspace::open_workspace_by_id` (a failure drops the record). `refresh` calls `groups::adopt`,
  which makes a pending record live once a workspace of the window holds its id (Zed restores the
  shown one itself), and the `Groups` observer (`groups_changed`) also serializes the window. A
  Browser tab that deserializes before its group is adopted finds it through
  `groups::group_of_workspace_id` and `BrowserProject::of_projectless`.

## Dragging to reorder the rail (`src/rail_order.rs`, #602)

- `rail_order.rs` is a child module of `rail.rs`, as the switcher is. `SavedOrder` holds #601's
  projectless-group ids (`groups`) and the order the user left: `headers`, the headers' places,
  and `rows`, per header's place its rows' places. A header's place is `project:` and its folders
  (one per line) or `group:` and the group's id; a row's is `terminal:` and its
  `MARLEY_TERMINAL_ID`, `browser:` and its page's target, or `thread:` and its key, and a row
  with neither is `view:` and its entity id, which keeps its place for the session only. The
  sidebar blob keeps it as `marley_order` (`read_rail_order`, `write_rail_order`).
- `rail_groups` sorts the listed headers, projects and groups together, by `headers` with
  `marley_rail::place`, so project indices, focus and `GroupEntry` follow one order;
  `build_snapshot` notes each terminal's and Browser tab's place in `Snapshot::places`, and
  `arrange` sorts each project's terminals, Browser tabs and threads by `rows`. `marley_rail`'s
  attention sorts are stable, so the placed order breaks their ties. Zed's project-group order is
  left alone: it cannot hold a folderless workspace.
- `render_blocks` wraps each header and the rows under it in one block. The header's frame takes
  `on_drag` with a `DraggedRailHeader`; the block is its drop target. A terminal's, Browser tab's
  or thread's card takes `on_drag`, `can_drop`, `drag_over` and `on_drop` with a
  `DraggedRailRow` (`draggable_row`). Each carries its place or selection, its position in the
  rail and its `marley_rail::Run`; `can_drop` wants the same run and not itself, and `drag_over`
  draws a 2 px `drop_target_border` on the target's top when it sits above the dragged one and on
  its bottom otherwise. Each drag type renders its own preview, a raised card with the name.
- `drop_header` and `drop_row` rebuild the header list, or the group's rows (terminals, Browser
  tabs, threads), with `marley_rail::move_to` (before the target when it sat above), and
  `order_changed` keeps only listed headers' rows, serializes the window, lets the hold go and
  refreshes. `move_project` swaps a project with its neighbour in the same list.
- The hold (`Hold`: #542's held order and `dragging`). `on_drag`'s constructor calls
  `start_drag` through the rail's weak handle, which sets `dragging` and takes the hold if none
  is held; the root's `on_hover` ignores `false` while dragging (gpui reads no hover during a
  drag); `end_drag` runs on the root's left mouse-up and mouse-up-out, and a hover-true, which
  gpui sends only with no drag in flight, clears a stale flag.

## The guide (`src/guide.rs`, `guide/index.html`, #599)

- `guide/index.html` is the user's guide: one self-contained page (styles and a small filter
  script inline, nothing loaded from the network), an area per `<section id>` with its `<h2>`,
  a feature per `<article id>` with its `<h3>`, each a "What it is" box then "How to use it", and
  a contents `<nav>` listing every section and article by id. Edit the page itself; a feature
  that changes what a user sees updates its article in its ticket's Phase 4, and a new article
  needs its line in the `<nav>`.
- `PAGE` carries it with `include_str!`. `marley: open guide` (`OpenGuide`), which the title bar's
  `?` dispatches by name (a `title_bar` touchpoint), runs `open`: `write_page_in(data_dir)` on the
  background executor (`guide/index.html` under the data folder, written only when its bytes
  differ), then a Browser tab of the workspace's project when it is local with a visible worktree,
  else `cx.open_url` (the system browser). A failed write shows in the workspace.
- The tab is found again by `browser::show_tab_where` with the page's URL, fragment dropped, since
  the page's own contents links add one; only when no tab of the workspace shows it does
  `browser::open_url_tab` open one.

## The last session's terminal size (`src/terminal_size.rs`, #486)

- `init` reads the `TerminalBounds` kept in the key-value store (scope `marley-terminal-size`,
  key `last`, as JSON) and hands them to `terminal::marley_seed_last_bounds`, which fills #485's
  slot only while no view has laid out a size this launch, so a launch's first terminals open at
  the last session's size and their shells lay a wide prompt out for it. An `on_app_quit` future
  writes `terminal::marley_last_bounds()` back; a crash leaves the last clean quit's size.

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

## Agent sessions across a restart (`src/resume.rs`, #540, #750)

- `init`, after `terminal_ids::init`, reads `MarleyAgentSessionsDb`'s table,
  `marley_agent_sessions(terminal_id, session_id, folder)` (its own domain, keyed by the
  terminal's `MARLEY_TERMINAL_ID`, which a restored terminal keeps), into `Sessions`, and removes
  the rows of terminal ids no saved terminal holds (`terminal_ids::known_ids`). `on_app_quit`
  sets `quitting`.
- `agent_events::on_frame` hands the lead's events to `on_event`: a `SessionStart` (but a
  compaction's) writes the terminal's row, memory first; a `SessionEnd` with `prompt_input_exit`
  or `logout` removes it; any other end waits, since a quit reads as `other`. The rail's
  `agent_events::end`, for a Claude Code gone without a `SessionEnd`, hands the seats to `ended`,
  which removes their rows unless `quitting`. A remote terminal has no row.
- Each new local `TerminalView` is recorded by its seat (`views`) and goes to `resume_restored`:
  with `MarleySettings::resume_agents` on, no agent in the foreground, a row for its terminal's id
  and that session not claimed this launch, it claims the session and, in a task, runs the
  startup handshake (`agents::STARTUP_TIMEOUT`) and writes `marley_agent::resume_line` with the
  project's launch mode for Claude Code (`agents::launch_mode`); a terminal that took input first
  keeps its shell.
- Since #652 a terminal whose agent holds a report (`agent_reports::holds_terminal`) saves the
  session the agent reported, with the agent's own folder (`on_report`), and skips the hook's
  `SessionStart`; the holder's release removes the row (`on_release`) unless `quitting`.
- **Codex, and a cut-off turn (#750).** The table gains `agent` (`claude-code` or `codex`, by
  migration with Claude Code the default) and `turn_open`. `codex_server`'s `follow_lead` hands a
  lead thread and the folder its server started in to `on_codex_lead`, which saves it as the
  terminal's Codex session; the lead's close (`thread/closed`) or Codex leaving the foreground hands
  the terminal to `on_release`, which keeps the row at a quit. `resume_restored` types the saved
  agent's `marley_agent::resume_line`; for Codex it first starts a new App Server for the restored
  terminal (`codex_server::prepare`, `start`), so #650 follows the resumed thread, and types the
  plain `codex resume <id>` when none comes up.
- `note_turns` observes the `AgentEvents` global: for each terminal with a saved session it writes
  `turn_open` when the seat moves into or out of working or waiting, never while `quitting`. `init`
  reads the rows whose turn was open into `cut`; a session resumed from `cut` puts its seat in
  `offers`, which the rail reads (`offered`) to show **Interrupted · Continue** on a line of the
  row's own (`rail.rs`'s `interrupted_chip`, a `CardLine::Element`). `continue_turn` pastes
  `marley_agent::RESUME_PROMPT` and Enter (`terminal_drive::paste_then`) only while the seat is
  `Idle`, and otherwise returns the state for a toast; the offer goes after Continue, a turn the
  agent starts once idle, the session's end or the view's release.

## Terminal output across a restart (`src/scrollback.rs`, #751)

- `init`, after `terminal_ids::init` and `resume::init`, sets `terminal::MarleyTerminalHistory`
  over `<data dir>/terminal_history` and an `Arc<AtomicBool>` read from
  `MarleySettings::restore_scrollback`, starts the writer task, and sends it the start's `Retain`:
  the files of `terminal_ids::known_ids`, or none with the setting off.
- `recorder(id, nonce)`, called in each local interactive terminal's builder: a closure run on
  the terminal's IO thread, fed by the event loop with what the parser sees and each hook frame as
  it came. A `DcsScanner` splits the two: the bytes go through `marley_dcs::history::Allowlist`,
  and `mark_of` turns a `Preexec` (not Zed's startup check) or a `Precmd` into a
  `marley_terminal::history::Mark`, a command's verified against `nonce`, none while the alternate
  screen shows (#764). Both go to the writer while the setting is on.
  `restored(id)`, called in a restored terminal's builder before its recorder: the kept tail
  (`marley_terminal::history::read_in`) cut into pieces at its marks (`history::restore`), and
  `SESSION_END` sent to the writer, so the file holds the break the replay draws. The builder
  places the blocks (`marley_terminal::Restoring`, `AnchoredBlocks::restore`).
- A restored block is history (#764): `rerun_offered` refuses it; `send_block`'s chip, `english`'s
  hint, the rail's command line, System One's last command, `typed_line`'s last exit and
  `failures` read `AnchoredBlocks::live`; `block_headers` records no branch for it and shows
  `restored`; `terminal_blocks` marks it `restored`.
- The writer (`write`, `Pending`) appends a terminal's output at most `WRITE_AFTER` (a second)
  after it came, or at once past `MAX_BYTES`; compacts a file past twice `MAX_BYTES` or past
  `MAX_LINES` new lines; acts on `Retain`, `Remove` and `Flush` in order; and drops the output of
  a removed terminal.
- `on_app_quit` sets `quitting` and awaits a `Flush`. A `TerminalView` released while Marley is
  not quitting and its window is still open (`cx.windows()`, which gpui leaves a closing window out
  of) sends `Remove`: a terminal the user closed. The setting turning off sends `Retain(None)`.

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
- **`browser_find` (#567)** reads the page through `read_snapshot`, the walk `browser_snapshot`
  uses, and keeps its refs in the hub, so its answer is the same currency and `browser_click`
  takes it; the items are the refs as `RefTarget::describe` names them (`button “Sign in”`), and
  the place is `hub.project_of(tab)`'s `BrowserProject` (its `paths`, and `host` for a remote one).
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
- Since #756 `run` answers with a `Refusal`: the older tools' strings become `refused`, and the
  eight tools in `browser_actions.rs` refuse with codes. `acting` is generic over its error and
  races its action against the tab showing a dialog (`dialog_opening`, polling the hub every
  100 ms): Chromium holds the answer to the click or key whose handler opened `confirm()` until
  the dialog closes, so the tool answers once the dialog shows, its `did` saying so and its
  answer carrying `dialog` (kind, message masked). `browser_dialog` answers it through the hub's
  `answer_dialog`. `browser_tabs` and `browser_look` add `dialog` and `emulation` (preset, size,
  scheme) when a page has them. An upload's paths are made real in the background (a link inside
  a project can point anywhere), then checked with `editor_tools::project_holding` and
  `not_secret`.
- Since #767 `PageState` also keeps `chooser` (the session, the input's node, `multiple`),
  `downloads` (the newest 20: name, path, state) and `taken_over` with `last_agent` (who acted
  last, kept after the Agent chip's five seconds). `BrowserHub::start` calls `allow_downloads`
  (the folder: `$XDG_DOWNLOAD_DIR`, `dirs::download_dir`, `~/Downloads`); `follow` hands the
  session-less download events to `follow_download`; a finished download is renamed off the main
  thread (`unique_in`) and an app notification offers Open (`open_with_system`) and Show in Folder
  (`reveal_path`). `follow_navigation` hands `Page.fileChooserOpened` to `file_chooser_opened`,
  which emits `ChooserOpened`; the view opens `prompt_for_open_path` when its press is within
  `USER_PRESS` (#495's rule), else the chooser bar waits (Choose…, Cancel) for the person or
  `browser_upload` with no `ref` (`answer_chooser`, else `no_chooser`). The control bar (Take Over
  and a close button, or Hand Back) and `TakeOverBrowser` (Ctrl-I in `MarleyBrowser`, propagated
  when no agent acted) move `PageState::control` between `PageControl::Agents`, `Quiet` and
  `Taken` (#810): the close button (`close_control_bar`) and Hand Back leave `Quiet`, which draws
  no bar until the next agent action sets `last_agent`. `run` refuses `WRITES` while `Taken` with
  `taken_over`, and `not_paused` refuses `browser_navigate` and `browser_check_pick` with the same
  words.
- `browser.rs` keeps `emulated`, `scheme`, `tab_pixels` and the drag's one-shot in `PageState`;
  `resize` leaves an emulated page's layout alone and only refits its stream; the toolbar's
  emulation chip (`render_emulation`) has Reset, `reset_emulation`.

## Outside clients (`src/clients.rs`, #524, #811)

- **Pairing (#811).** `clients::answer` serves `client_pair` and `client_pair_status`.
  `begin_pairing` checks the name (`check_client_name`), refuses one in the registry
  (`already_paired`), one already waiting (`pairing_pending`) and a fourth waiting pairing
  (`too_many_pairings`, `MAX_PAIRINGS`), then mints a 32-hex id and a six-digit code (`rand`),
  shows the notification in every window (`show_pairing`, `show_app_notification`: the name, the
  code, "Asked from" the caller's `cwd`, and Read, Read and Write, Read Write and Run, Refuse) and
  keeps a `Pairing` in `ClientsState::pairings`. Its task waits on a one-shot slot or `PAIR_WAIT`
  (two minutes), dismisses the notification, and on a power runs `allow(name, false,
  Some(power))`; the state becomes `Trusted { power, file }`, `Refused` or `Expired`.
  `pairing_status` answers `waiting`, or the final state once (the trusted one with the endpoint
  file and `client_lines`' two commands) and drops the pairing; an unread final state is dropped
  at the next pairing once twice `PAIR_WAIT` old.
- `RegisteredClient::power` (`Option<Power>`, left out of the file when none) marks a paired
  client; `start` and `allow` pass it to `transport::allow_client`. `set_power` changes the
  registry on the main thread, then the server's table (`transport::set_client_power`) and the
  file off it.

- The registry is `<data>/mcp/clients.json` (0600, written through a file renamed into place):
  each client's name, grant (`write`) and when it was allowed, and no token. A file Marley cannot
  read is left as it is, logged, and Allow and Cut Off refuse with the reason. At start, once
  the server runs, `clients::start` removes the last run's endpoint files, reads the registry
  and keeps it on the main thread, then mints each client a token (`transport::allow_client`)
  and writes `<data>/mcp/clients/<name>.json` (0600, the folder 0700) in the shape of
  `mcp-endpoint.json`; the files go at quit. The main thread never takes the server's locks:
  the token and file work runs in background tasks.
- Outside Clients (`marley: outside clients`, and `marley: browser clients` since #524,
  `BrowserClientsModal`): a paired client with "may read", "may read and write" or "may read, write
  and run" and the Read, Write and Run buttons (`change_power`), a client allowed by name with
  "browser: reads" or "browser: reads and acts"; each with its last call (read off the main thread
  every five seconds) and Cut Off;
  a name field (Enter allows), "May act in pages" (`ui::Checkbox`) and Allow; after an Allow,
  the client's endpoint file, `client_lines`: the line that points Marley's bridge at it
  (`MARLEY_MCP_ENDPOINT=<file> <data>/mcp/marley-mcp-bridge`), and the line that runs that bridge
  here from another machine (#584): `ssh -T -o BatchMode=yes <user>@<host> 'env
  MARLEY_MCP_ENDPOINT=<file> <bridge>'`, the names from `whoami::fallible` (`this_machine`), a
  placeholder standing in for a name that cannot be read. Each has Copy. Escape closes it.
- `clients::allow` checks the name, mints, writes the file and then the registry, and takes the
  token back when a write fails. `clients::cut_off` changes the main thread's registry first, so
  `is_allowed` answers at once for a call still waiting and for the tab's mark, and clears the
  mark (`BrowserHub::forget_client`); then, off the main thread, `transport::cut_off_client`, the
  registry and then the file, so a bridge that finds the file gone finds the client unlisted
  too (#584).
- `mcp::answer` runs `marley_mcp::permits` before any app call, the second wall behind the
  server's. `browser_tools` takes the client's name from `AppCall::principal` and passes it to
  `acting`, `check_pick` and `annotate`; `still_allowed` refuses a cut-off client's call at the
  start of `run` and after `settled`. `BrowserHub::agent_started` and `agent_ended` take the
  client: the chip names it in place of "Agent", `PageState::driven_by` keeps its name and last
  action, and the toolbar draws "Driven by <name>" and Cut Off (`render_driven_by`) while the
  client is allowed and acted within `DRIVEN_MARK` (a minute); the minute's `Agent` entry carries
  `by`.
- The bridge (`claude_plugin/marley/bin/marley-mcp-bridge`) raises `Refused` on a 403 and answers
  a tool call with `REFUSED` ("Marley refused this endpoint file's token …") in place of "Marley
  is not running". Since #584 a bridge whose client file (`<data>/mcp/clients/<name>.json`) is
  gone while `clients.json` beside the folder no longer lists the client raises `NotAllowed` and
  answers `NOT_ALLOWED` ("Marley does not allow this client …"); a listed client with no file
  (Marley quit or is starting) still hears that Marley is not running. Each error class carries
  its call's text and its unknown method's phrase (`why`, `state`).

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
- Since #570 (version 1.4.0) an AskUserQuestion's summary carries `options`, its first
  question's option labels, at most eight, each cut to 40 characters; under the bound they go
  after the working directory and before the message, so a long list never pushes out what a row
  shows.
- Since #566 (version 1.3.0) a message over 300 characters keeps its end (`cut_ends`): its start,
  cut back to a word, then ` … ` and its last whole sentences up to 147 characters, where a final
  message's question or status sits; a last sentence longer than that leaves the plain cut. Marley
  redacts after the hook cuts, so the end starts at a sentence and the start stops at a word:
  neither cut parts a secret from the name or the `Bearer` that marks it.
- Since #547 `ClaudePlugin` keeps the installed version (`installed_version_in`, the user-scope
  entry of `marley@marley`), and `needs_update` compares it with the embedded manifest's
  (`shipped_version`, `semver`); an unparsable or newer one needs none. The chip then reads
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
  `project.json` is written. `start` connects through the project's relay socket when a relay
  answers there (#583); otherwise it checks the socket's path against `sun_path`, closes over its
  port a unit an earlier build started (a Chromium that answers on `DevToolsActivePort`'s port),
  and, unless the unit is up, removes a stale endpoint file, starts the unit with the relay's
  command (`service::relay_executable` and the browser binary, found off the main thread) and
  waits up to fifteen seconds, failing early when the unit stops. It then turns on target
  discovery and attaches the pages the
  browser lists, each once and each in a task of its own, with the page's observers on before
  the page is announced; a start opens no page (#494). The event loop routes each
  event to the page whose session it came from, or whose iframe's; attaches each `page` target
  discovery reports later (`targetCreated`, which carries `openerId` for a page a page opened);
  drops a page on `targetDestroyed` or `targetCrashed`, and one that went while it was being
  attached (`closing`); decodes each frame off the main thread, keeps it for its page and
  acknowledges it; follows each page's URL and title (asking for the title after
  DOMContentLoaded, load and same-document navigations, since no target event reports it, and
  taking a title the page's script sets later from the page's title watcher, #582:
  `title_reported` accepts it from the page's own session only and emits `PageInfoChanged` when
  it differs); and fails with "The browser closed its connection." when the socket ends.
  `Runtime.bindingCalled` goes by the binding's name: the recorder's to `action_reported`, the
  title watcher's to `title_reported`, the rest to `select_requested`. A generation number
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
  live: its pages go, their tabs with them, and `stop_chromium` sends `Browser.close` (through the
  relay, else over the port of a Chromium an earlier build or the pre-#507 profile started, #583),
  waits up to five seconds for the unit to stop, then runs `systemctl --user stop` for whatever
  is left.
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
- **Playwright scripts (#523, `src/playwright_scripts.rs`).** The toolbar's Scripts button,
  between the address bar and Pick so the buttons after it keep their places, or
  `marley::PlaywrightScripts`, toggles the tab's `ScriptsTray` (an `Option`, open while `Some`),
  drawn above the picks tray: a row per script (`library_in`: the project's `.mjs` files under
  `<config>/playwright/projects/<key>/`, then every project's under `…/global/`, each sorted), each
  with its scope, Run (disabled while the tab has no page) and Edit, then a name field with For
  This Project and For All Projects (`create_in`, from the embedded `template.mjs`, refusing a
  name that exists; Enter is `marley::NewPlaywrightScript`); a made script opens in an editor tab.
  Run focuses the tab, so its pane is active, and hands `playwright_scripts::run` a `RunTarget`:
  off the main thread it reads the endpoint of the tab's project's Chromium, writes the embedded
  runner and its `package.json` into `<data>/playwright`, and checks for `playwright-core`
  there; a second first run is refused while one installs (the `Installing` global). It opens a
  terminal with `add_center_terminal` and moves it from the tab's pane into the pane on its right
  (`move_active_item`), or a new split (`split_and_move`); after the startup handshake it types
  one command, `[npm install --prefix … --no-audit --no-fund &&] MARLEY_CDP_FILE=… MARLEY_TAB=…
  node run.mjs <script>` (each part quoted by `ShellKind::Posix`); since #583 it names the relay's
  `relay.json` and never the token, and `run.mjs` reads the file, attaches with
  `connectOverCDP(url, { headers })` and hands `MARLEY_CDP_URL` and `MARLEY_CDP_TOKEN` to what the
  script starts. It puts
  `Entry::Script { name, exit_code: None }` in the page's minute, whether or not a tab draws the
  page at that moment (#583: the terminal takes the tab's place before it moves beside it). `block_end` polls the terminal
  for the block whose command is that line (a new terminal's startup opens a block first) until
  it finishes: a code of 0 ends the run; another saves the page's minute through
  `BrowserHub::record` with the `Script` entry's end, and a toast names the recording; a block
  with no code says so.
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
- Pages see Chrome (#539). The hub reads the identity with the pages a start finds
  (`pages_and_identity`) and hands it to each attach. `child_attached` handles every target a
  page's auto-attach holds: a cross-site iframe gets the identity, and when its page is known the
  hub keeps it and observes it (`record_iframe`); then every held target runs on, handled or not.
  `create_page_task` opens every page at `about:blank`, with its URL in `pending_urls` until the
  attach has set the identity, and `attached` then sends the page there and notes it in
  `blank_entries`. When that URL commits, `refresh_history` resets the page's history, so Back
  does not lead to the blank page, and sends the tab's size and restarts its stream, which the
  first commit can leave at the window's size. `ProjectBrowser::starting` builds a starting
  browser's state.
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
  terminal used yet, says so in the tray. So does a Send while that terminal's Claude Code waits
  on the user (#508, `agent_events::waiting`), which types nothing and leaves the pick and its
  caption unsent: the line would land in the agent's prompt.
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

## The Fleet panel (`src/fleet.rs`, #607)

The first surface of D20 (`docs/marley/fleet-contract.md`): the agents a workflow store reports,
drawn from `marley_sdk`'s types.
- **The global.** `Fleet` holds each provider's last reading as a `Source` (its name, agents,
  hosts, `poll_s`, `stale_after_s`, the reading's time and a failure), the started `Pseudo`, and
  whether the reads run. One task reads every provider named in
  `MarleySettings::fleet_providers` (`marley.fleet.providers`), then waits the shortest `poll_s`
  (5 s when none names one). It writes the global only when a reading differs, reading it
  through `try_global` first, since `default_global` tells the observers.
- **When it reads.** The loop checks at each read that a Fleet panel is what an open right dock
  shows in some window's shown workspace (`panel_shows`: each `MultiWorkspace` window, its
  `workspace()`, `right_dock().visible_panel()`, downcast through `to_any()`), and stops when
  none is. A `FleetPanel` that draws while no reads run starts them with `cx.defer`.
  `Panel::set_active` is not the signal: Zed's docks call it for a panel activated in a closed
  dock too, and a layout switch moves panels between docks without the call that matches
  (F-claude-607-polling-followed-set-active-not-what-shows-001).
- **The panel.** `FleetPanel` is a `workspace::Panel`, right dock only (`position_is_valid`), 320 px,
  `IconName::Server`, tooltip "Fleet", toggled by `marley::ToggleFleet` (`marley: toggle fleet`),
  activation priority 20, which no other right-dock panel uses. `fleet::init` adds one to each
  workspace in an `observe_new`, and registers the toggle through `toggle_panel_focus`. It draws a
  "FLEET" header, then each source: its name, a failure in red, and its agents grouped under
  their hosts in the provider's order (agents whose host it did not describe last, under their
  `host_id` or "No host"). An agent's row: its runtime's icon (`runtime_icon`), its name, the work
  item's key and the phase as `name n/m`, a warning mark for a question or a red mark for a failed
  run, and a `Chip` whose word and color come from `state_chip`, reading `stale` when
  `marley_sdk::is_stale` says so against the reading's time.
- **Selection and the snapshot (#608).** A second global, `Wanted`, maps each panel's entity id to
  its `Selected { source, agent }`; the reads keep in each `Source` the `AgentDetail`s of wanted
  agents only (`details`), with its handshake's `capabilities`. A row's click focuses the panel
  and selects (`FleetPanel::select`), which registers the selection and reads at once through
  `cx.defer(read_now)`; the panels do not observe `Wanted`. The key context is `FleetPanel menu`,
  so Zed's `menu::SelectNext` and `SelectPrevious` step through `drawn_order`, which shares
  `Source::host_groups` with the list. The global's observer drops a selection whose agent left
  the list, and the panel's release removes its entry. With a selection the body splits: the
  list (`flex_1`), a 1 px handle whose 7 px grip drags a `DraggedFleetSplit`, and the snapshot at
  `relative(snapshot_ratio)` (half by default, clamped to 0.15 to 0.85 by the body's
  `on_drag_move`), after `git_ui`'s commit-view split. `render_snapshot` draws the header, then
  each section its capability allows: `work_items`, `runs` (`render_phase_strip`, a segment per
  phase coloured by `phase_color`), `hosts` (`render_resources`, `ui::ProgressBar` meters, or "no
  resources yet"), `usage` (tokens today) and `questions` (the options as chips, read-only).
- **Samples and the tabs (#609).** Each read appends a `Sample { at_ms, cpu, memory, network }`
  per host snapshot to `Fleet.samples`, keyed by `HostKey { source, host }`, and drops what is
  older than 30 minutes (`sampled`); memory is `memory_share`, network `rx_bps + tx_bps`. The
  reads run while `fleet_shows`: a Fleet panel an open right dock shows, or an `AgentView` that
  is a pane's active item. The panel holds its workspace's `WeakEntity` from `init`, and opens
  the selected agent's tab on a row's second click (`ClickEvent::click_count() == 2`), on
  `menu::Confirm`, and from an outlined Open button that `render_snapshot_header` takes as its
  first line's `end`.
- **Hosts (#610).** `Fleet.collected` holds what `fleet_hosts::collect` last found; the loop
  runs it every 5 s (`COLLECT_EVERY`) between reads, outside any update, while `fleet_shows`, and
  `keep_collected` logs a refused host once. `read_providers` calls `fleet_hosts::join`, which
  fills `Source.unreachable` and `Source.processes` and gives the Hosts source (`hosts_only`),
  appended after the stores. `host_groups` takes unreachable hosts (problems first in the Hosts
  source, which also shows hosts with no agents); a host header carries `host_line` (CPU, memory,
  disk, network) and an `unreachable` or `refused` chip whose tooltip is the reason;
  `agent_chip` gives `running` to a process row and `offline` to a store agent on an unreachable
  host; a joined agent's snapshot adds its process line. `rate` lives here now.
- **Stores and their states (#611).** `Source.state` (`SourceState`: `Ready`, `Failed`,
  `Connecting`, `Unreachable`, `Stale`, `Incompatible`) is drawn on each store's header as a chip
  (`source_chip`) with its reason (`source_reason`) and the reason's tooltip; an unreachable
  store's agents read `offline`. `Fleet.remotes` holds the `mcp` and `http` providers; the loop
  syncs them with the settings and polls them after the hosts (`keep_remotes`), and
  `read_providers` gives each entry its remote's source, or `Connecting` before its first poll.
- **Not set up.** With no provider, the panel says "The fleet is not set up." and names
  `marley.fleet.providers` and `{ "kind": "pseudo" }`.
- **Settings.** `settings_content::MarleyFleetSettingsContent { providers }` with
  `FleetProviderContent`, tagged by `kind`; `Pseudo` is its only variant until #611 adds the MCP
  and HTTP clients. `default.json` has `"fleet": { "providers": [] }`.

## The host collector (`src/fleet_hosts.rs`, `bin/marley-collect.sh`, #610)

Marley reads each host the settings list with its own script.
- **The script** is POSIX `sh`, shipped by `include_str!`. It reads `/proc/stat`, `meminfo`,
  `loadavg`, `uptime`, `net/dev` and `df -P -k /`, and each `/proc/<pid>` whose `comm` is
  `claude`, `codex` or a name in `MARLEY_COLLECT_NAMES` (its `cwd`, `stat`, `VmRSS` and
  `MARLEY_FLEET_SESSION`), and prints one `marley.host/v1` document. Rates come from the previous
  run's counters in a state file under `$XDG_RUNTIME_DIR`, else a 0700 folder of its own in
  `/tmp` it checks it owns; a first run samples twice, a second apart. The shellcheck gate lists
  it.
- **Running it.** `FleetHost { target: HostTarget::{Local, Ssh}, name, id }` comes from the
  settings (`MarleySettings::fleet_hosts`, through `fleet_settings`). `collect` runs each host in
  turn through `process::output`: `sh -c <script>` for this machine; for SSH, the destination
  through `marley_remote::parse_ssh_target` (a failure is `refused` and nothing runs), then
  `MARLEY_SSH` or `ssh` with `-T`, `BatchMode=yes`, `ConnectTimeout=5`, `ControlMaster=auto`,
  `ControlPath=$XDG_RUNTIME_DIR/marley-ssh-%C`, `ControlPersist=60`, `ssh_command`'s port and
  `--`, and `printf %s <b64> | base64 -d | env MARLEY_COLLECT_NAMES='…' sh`. Only names of
  `[A-Za-z0-9._-]`, at most 32 characters, reach the host's shell. A failure's reason is the last
  line of stderr.
- **The join** (`join`, `claim`): a reachable host's snapshot, named as the settings name it,
  replaces a store source's host of the same id in place; a process is claimed by the store agent
  whose id is its session, else by host, runtime and the agent's folder (where its detail is
  known), else by the one agent and one sessionless process of that runtime on the host. The
  rest become the Hosts source's agents (`process_agent`: `"<host id>:<pid>"`, named for their
  folder, with a detail carrying the snapshot). A host that did not answer is an `Unreachable`
  of the Hosts source and of each store source with agents on it.

## The stores over MCP and HTTP (`src/fleet_providers.rs`, #611)

A `Remote` per `mcp` or `http` provider, found again by its settings entry.
- **Clients.** `connect` starts `ContextServer::stdio` (a `ContextServerCommand` of the entry's
  `command` and `args`) or `ContextServer::http` (the bearer as an `Authorization` header), or
  keeps Zed's HTTP client and the base URL for plain HTTP. `web_url` allows `http` and `https`
  only. `bearer` reads the variable `bearer_env` names and logs only the variable's name.
- **A poll** (`poll_once`): the handshake when there is none (a `contract` other than
  `WORK_CONTRACT` is incompatible); `work_changes` from the cursor when the handshake offers
  `changes`, and the list only on a reset or a changed agent, else `work_agents` every time;
  `work_agent` for each wanted agent the list holds. `call` asks over MCP (`CallTool`, then
  `structured_content`, else the JSON of the text, `is_error` an error) or with `GET
  <base>/marley/v1/…` (`get`, a non-2xx an error), and gives up after `CALL_TIMEOUT` (5 s).
- **States** (`poll`): an answer is `Ready`; a timeout changes nothing, so the source reads
  stale after three polls without an answer (`is_stale` on the last answer); an error is
  `Unreachable` with its root cause, logged whole once, the MCP server stopped so the retry
  starts it again; an incompatible handshake keeps no agents. Both wait before the next try:
  `back_off` gives 1, 2, 4, 8, 16, then 30 seconds. `poll_all` polls every due remote together.
- **`sync`** keeps the remotes whose entries are still set, in the settings' order, and stops
  the MCP servers of the ones that went. `Remote`'s `Debug` leaves its client, and so its
  bearer, out. `source` turns a remote into the panel's `Source`, its hosts from its details.
- **The stub**, `script/e2e/fleet-stub-provider.py`, serves the fixtures as a store over HTTP
  or MCP on stdio, for scenarios.

## The Agent tab (`src/agent_tab.rs`, #609)

One agent of the fleet in full, in the center, read-only and not serialized.
- **Opening.** `open_later(workspace, selected, window, cx)` defers through `Window::defer` to
  `open`, which activates the `AgentView` among `Workspace::items_of_type` whose `Selected`
  matches, or adds a new one to the active pane (as `system_one_calls::open`). The defer is load-bearing:
  `open` reads every Agent tab, and a neighbour row's click comes from inside one's update.
- **Its data.** `AgentView::new` puts its entity id and `Selected` in `Wanted`, so the reads keep
  the agent's detail, and runs `fleet::read_now` deferred so the tab fills at once; its release
  removes the entry. It observes the `Fleet` global. A render with no reads running starts them.
  An agent the source no longer lists reads "… is no longer listed by …"; before the first
  detail, "Reading…".
- **Its parts** (`render_agent`): `fleet::render_snapshot_header` and the work item's line; PHASES
  (`render_timeline`: a track per phase over the run's start to its end or now, the bar placed with
  `share`, per-mille integers so nothing casts a float; a phase with a start and no end runs to now
  while active, else to the next phase's start; `render_gates` under it with a failed gate's
  detail); EVENTS (`render_events`: the run's and the agent's recent events merged, newest first,
  duplicates dropped, 50 at most, "N ago · kind · text"); RESOURCES (`render_history`: CPU and
  memory in percent and network as a share of the busiest sample, each a `sparkline`, a `canvas`
  stroking a `PathBuilder` line, with the value now and "N samples over …"); TOKENS
  (`token_line`, run and day, with cache reads); ON THIS HOST (`render_neighbours`, rows that open
  their own tabs). Each part checks the handshake's capability as the snapshot does.
- **Item.** The tab's title is the agent's name, its icon `IconName::Server`; `type Event = ()`.

## An agent's HTML page (`src/browser_pages.rs`, #802)

- `browser_tools::run` routes `browser_show_page` before the browser settles, as it does
  `browser_open_url`, and passes the caller's workspace (`Scope::home`); a caller in no project
  is declined.
- `show_page` writes the stylesheet first, then the page: with `html` and `name` (checked by
  `page_file_name`: letters, digits, `-`, `_`, `.`, no leading dot, 100 at most) into
  `<data_dir>/pages/<BrowserProject key>/<name>.html`, the stylesheet's `<link>` put into the
  head by `with_stylesheet` unless `theme: false` or the text names the stylesheet already; with
  `path`, `project_page` reads it as `address::local_page` does, `canonicalize`s it and each of
  the workspace's roots, and takes it only inside one, as a file. Nothing is written for `path`.
- The tab opens through `browser_tools::open_in_workspace`, the half of `open_in_tab` that
  shows the workspace and calls `open_url_tab`, which brings forward a tab already on the URL.
  The answer waits up to 5 s for the tab's page id (`BrowserView::target`) and leaves `tab` out
  after that.
- `watch` follows each page file for the rest of the session through its folder's `Fs::watch`
  (a write by rename replaces the file, which a watch on the file would lose), filtered to the
  file; each change calls `browser::reload_tabs_on(url)`, which reloads every live tab whose
  address is the file's URL. A watch that ended when no tab showed the file raced a call opening
  a new tab on it.
- The stylesheet (`<data_dir>/pages/marley-theme.css`): `stylesheet_text` writes
  `color-scheme` and eleven variables from `cx.theme()` (editor background, surface, text, muted
  text, border, accent text, and the success, warning and error status colors, as CSS `rgb()` in
  percentages) and `ThemeSettings`' UI and buffer font families, Zed's aliases resolved by
  `gpui::font_name_with_fallbacks` (`.ZedSans` is IBM Plex Sans, `.ZedMono` Lilex). `ShownPages`
  keeps the text last written; `restyle`, on every `GlobalTheme` and `SettingsStore` change,
  rewrites it only when the tool has written it before and the text differs, then reloads the
  tabs of every watched page.

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

The shared setups (`init_test`, `init_agent_test`) keep the machine's IO out, which the test
scheduler cannot drive (#634): an open rail's ports scan reads `Ports::proc_root`, a folder with no
sockets, and asks no container engine; `Launcher::passphrase_dialog` is `Skipped`, so an agent's
terminal opens without ssh's passphrase socket. Blocking reads go through gpui's background
executor (`futures::future::lazy`), never `smol::unblock`, so a test drives them.

The test binary sets its data folder before its first test (a `ctor` in
`marley_workbench_tests.rs` calling `terminal::marley_use_test_data_dir()`, #475): the routing
tests start the system shell, which installs Marley's scripts, and the workbench reads and writes
`paths::data_dir()` in many places, so a run writes `marley-test-data` in the build folder and
never the user's.

Since #483 no test is added (CONSTITUTION §7): the files above stay and keep building, and each
change is proven by an e2e scenario in `script/e2e/`. `480-voice-input.sh` drives the
microphone through a fake Voxtype whose `record toggle` moves its status on, and
`481-rich-input.sh` types into the rich input and reads what a stand-in agent prints;
`484-autosuggestions.sh` types prefixes at a bash with a history file of its own,
`637-history-suggestions-in-the-prompt-editor.sh` the same in the prompt editor, and
`783-up-and-down-in-the-prompt-editor.sh` walks that history with Up and Down and sends a
stand-in agent a line after both keys; `784-terminal-keys-from-the-prompt-editor.sh` presses the
terminal's keys in both editors at a bash that prints each one it gets and a stand-in agent that
prints each byte.

## Known limits

- `browser_show_page`'s scratch folders and watches are never pruned. The stylesheet names Zed's
  UI and monospace fonts, which Zed carries inside its binary; Chromium finds them only where
  they are installed, and otherwise uses `system-ui` and `monospace`.
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
- The inbox lists the center terminals' agents, as the rail does, so a Claude Code waiting in a
  docked Terminal Panel has no entry. A terminal agent's entry opens the terminal; answering it
  from the rail is the second slice in #508's notes.
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
