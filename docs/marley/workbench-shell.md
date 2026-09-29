# The workbench shell: a Marley layout on Zed

*Written 2026-09-22 in the fork. A plan, not a spec; each slice gets its own ticket when it
starts. It comes before the three prongs of [three-prong-plan.md](three-prong-plan.md)
because every prong renders into this shell: Blocks draw inside its terminals, the fleet
becomes a section of its rail, and the browser is one more thing that opens in its main
area.*

## What Chad asked for

> Zed is a good base editor because its fairly plain. However, warp.dev is much better
> interface. So in Zed the left you have projects but its weird because it only has agents
> under it. I cant figure out even how to start a new agent. And also the terminals are at the
> bottom but can be brought up into the horizontal tabs.
>
> this is a bit different from warp which i think is cleaner. Which is the terminal is the main
> thing and its on the left under the project.
>
> So where i'd like to do first is we need to figure out how to do this. Our crates should
> attempt to not go into zed as much as possible and if we do then we need to record it for
> future code changes

And, answering the first round of questions the same day:

> wonder if we can think about this holistically and we have a Marley layout which basically
> makes it where zed can be used as default but Marley basically is its own layout/addition?

The target is the layout the gpui-era app reached at Warp parity: a left rail with each
project and its sessions beneath it, the selected session filling the main area, no separate
terminal dock, and exactly one highlighted row
([session-tabs-vs-sidebar](../planning/design-notes/session-tabs-vs-sidebar.md),
[simple-rail-shelf](../planning/design-notes/simple-rail-shelf.md),
`PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`). It arrives as
a layout of its own, beside Zed's, so the fork stays usable as plain Zed.

## Decisions so far

Chad, 2026-09-22:

1. **A Marley layout beside Zed's.** Zed keeps working as Zed; the Marley layout is an
   addition you switch to. Design in D0.
2. **Rail rows:** each project lists its terminals and its Zed agent threads.
3. **Bottom panel:** hidden in the Marley layout, with every terminal routed to the center.
4. **Own identity now:** the fork becomes Marley, with its own name and data directories
   (W1).

## How Zed's window works today

A Zed window is a `MultiWorkspace` (`crates/workspace/src/multi_workspace.rs:305`). It holds
one `Workspace` per open project, shows one of them, and draws an optional sidebar beside it
(`render`, `:1996`). Each `Workspace` has its own title bar, left, right and bottom docks, a
center pane group and a status bar.

What Chad saw follows from Zed's defaults, which Zed calls the Agentic layout:

- The left column is the Threads Sidebar (`crates/sidebar`, 8.3k lines): project groups with
  agent threads under each. It does list terminals ("Terminal Threads"), but those live inside
  the Agent Panel, open only from the agent menu, and disappear when AI is off.
- The Agent Panel docks left beside it; the project and git panels dock right
  (`assets/settings/default.json:868`, `:1146-1157`; the presets are `PanelLayout::AGENT` and
  `EDITOR`, `crates/agent_settings/src/agent_settings.rs:41-65`).
- A new thread starts from a `+` that appears only while the pointer is over a project
  header, from the Agent Panel's new-thread menu, or from `ctrl-n` while the sidebar has focus
  (`crates/sidebar/src/sidebar.rs:2527-2751`, `assets/keymaps/default-linux.json:748`).
- Terminals default to the Terminal Panel, a bottom dock (`default.json:1999`). A terminal
  can also open as a center tab (`workspace::NewCenterTerminal`), which is the horizontal-tabs
  path Chad found.

## The seam

The sidebar is a trait. `workspace::Sidebar` (`multi_workspace.rs:121-160`) asks for a width,
a side, a notification flag, focus handling, project and thread cycling, and an opaque
serialized blob. `MultiWorkspace::register_sidebar` (`:387`) accepts any implementation and
replaces whatever was registered before. The app registers Zed's in one place, a deferred
callback in `crates/zed/src/zed.rs:536-546`. The `MultiWorkspace` keeps everything around the
sidebar: the resize handle, open and closed state and its persistence, the toggle and focus
actions, project order and collapse state, and which workspace is on screen. A crate that
implements the trait inherits all of it. `crates/agent_ui/src/test_support.rs:159-225` is a
forty-line implementation Zed's own tests use.

Everything else the shell needs is public:

| Need | Public API |
|---|---|
| Projects and their workspaces | `MultiWorkspace::project_groups`, `workspaces_for_project_group`, `last_active_workspace_for_group`, `find_or_create_workspace` (`multi_workspace.rs:849-1196`) |
| Switch project | `MultiWorkspace::activate` (`:1310`) |
| Add a project | `recent_projects::sidebar_recent_projects::SidebarRecentProjects::popover` (`crates/recent_projects/src/sidebar_recent_projects.rs:30-95`); `ui::ProjectEmptyState` for an empty window |
| A terminal in the center | `Project::create_terminal_shell` (`crates/project/src/terminals.rs:284`), `TerminalView::new` (`crates/terminal_view/src/terminal_view.rs:233`), `Workspace::add_item_to_active_pane` (`crates/workspace/src/workspace.rs:4939`); `TerminalPanel::add_center_terminal` (`crates/terminal_view/src/terminal_panel.rs:835`) is the same three steps |
| List and activate terminals | `Workspace::items_of_type::<TerminalView>` (center panes only, `workspace.rs:4296`), `Workspace::activate_item` (`:5559`) |
| Zed threads | `ThreadMetadataStore::entries_for_main_worktree_path` (`crates/agent_ui/src/thread_metadata_store.rs:643`); live status from `AgentPanel::conversation_views` (`crates/agent_ui/src/agent_panel.rs:4151`); open with `AgentPanel::load_agent_thread` (`:4452`) |
| Start Zed agents | `AgentServerStore::external_agents` (`crates/project/src/agent_server_store.rs:609`), `AgentPanel::new_thread` and `new_external_agent_thread` (`agent_panel.rs:1783`, `:1951`) |
| Route tasks to the center | `Workspace::set_terminal_provider` (`workspace.rs:3340`) |
| Zed's own sidebar, for the Zed layout | `sidebar::Sidebar::new` (`sidebar.rs:832`) |
| Window controls in the rail header | `platform_title_bar::render_left_window_controls`; the title bar stops drawing them when the sidebar is open on that side (`crates/platform_title_bar/src/platform_title_bar.rs:250-313`) |

## Design

### D0. The Marley layout is a mode

Zed already has layouts: the title bar's Panel Layout menu switches between Agentic and
Classic (`crates/title_bar/src/title_bar.rs:120-127`). Those presets only move panels by
rewriting dock positions in the user's settings (`agent_settings.rs:370-396`). The Marley
layout changes more than dock positions, so it is a mode with one switch, and everything the
mode does lives in `crates/marley_workbench` and stops when the switch goes back:

| | Zed layout | Marley layout |
|---|---|---|
| Left column | Zed's Threads Sidebar | The rail: projects, their terminals and their Zed threads |
| Terminals | Zed's defaults (bottom panel) | The center; tasks, Open in Terminal and the terminal keys follow; the bottom panel stays hidden |
| Agent Panel | Where Zed's layout puts it | Right dock, showing the thread picked in the rail |
| Opening a project | Zed's behavior | A terminal at the root when the project has none |
| Keys | Zed's | The Marley keymap; each Marley action falls back to the Zed action it replaces when the layout is Zed |

**The switch is a setting**, `"marley": { "layout": "zed" | "marley" }`. `SettingsContent` is
a closed struct and its schema rejects unknown keys (`crates/settings_content/src/settings_content.rs:174`,
`crates/settings/src/settings_store.rs:1299`), so the block costs one touchpoint set: a field
and a new `marley.rs` in `settings_content`, and one `marley: None` line in the VS Code
importer, which lists every field (`crates/settings/src/vscode_import.rs:183-245`). The same
block is where the three-prong plan's harness, Rusty, MCP grant and browser settings go
(three-prong plan D9), so the cost is paid once. The zero-touch alternative, a flag in Zed's
key-value store, would hide the choice from `settings.json` and from settings profiles.

**Switching is live.** On a change of `marley.layout` the crate:

- registers the other sidebar in every window with `register_sidebar`. The rail keeps Zed's
  sidebar alive while it stands in and hands it back, open or closed as it was, on the switch
  to Zed; a window that opened in the Marley layout gets a new one from Zed's public
  constructor, given the saved state the rail kept for it;
- applies the Marley default values or puts Zed's back (D6);
- does nothing else, because the terminal provider, the action capture and the Marley keys
  read the layout on every call.

**Default:** the fork starts in the Marley layout since #460, and the user's settings can
choose Zed. Chad on 2026-09-23: "when the program is installed the user shouldnt have to
choose Marley at first it should swap it over." Until then it started in the Zed layout, the
first reading of "zed can be used as default" in the decisions above, and the flip was the
same design with the default turned. In the Zed layout the fork behaves like upstream, which
also answers "is this bug ours?" in one switch.

**Where to switch:** two command-palette actions, "marley: use Marley layout" and "marley: use
Zed layout", and the rail's header menu. A Marley entry in Zed's own Panel Layout menu would
cost a touchpoint in `title_bar.rs`, so it waits until Chad wants it there.

### D1. The rail is a Marley crate

`crates/marley_workbench` implements `workspace::Sidebar` as `Rail`. The hunk in
`crates/zed/src/zed.rs:536-546` hands sidebar construction to the crate, which builds the
rail or Zed's sidebar according to the layout. The hunk has to live there: restore applies
the saved sidebar state only when a sidebar is already registered
(`workspace.rs:10344-10352`), and a registration from the crate's own
`observe_new::<MultiWorkspace>` would run before Zed's and be overwritten, since observers fire
in registration order.

Zed's `sidebar` crate stays linked. Its actions (`agents_sidebar::NewThreadInGroup`,
`agents_sidebar::ToggleThreadHistory`, `dev::DumpWorkspaceInfo`) are bound in the default
keymaps, and `load_default_keymap` unwraps the keymap load, so unlinking the crate panics at
startup (`zed.rs:2344-2376`, `crates/settings/src/keymap_file.rs:180-202`). The Zed layout
needs it anyway.

The crate is `MIT OR Apache-2.0` like the other Marley crates (CONSTITUTION §20), so it is
written fresh against the public APIs above. Reading `crates/sidebar` for behavior is fine;
pasting from it is not, because that code is GPL.

### D2. Terminals live in the center

In the Marley layout a terminal is an ordinary `TerminalView` in the workspace's center pane
group. The rail builds it with the three public calls in the table, so it keeps the view it
made. Splits, dragging between panes, zoom and restore come from Zed unchanged: a center
terminal comes back after a restart in its last directory with its custom title, though not
its scrollback or the program that was running (`terminal_view.rs:1871-1964`).

The Terminal Panel stays loaded, out of sight. It is the workspace's `TerminalProvider`, so
tasks, agent logins and two terminal context-menu entries need it, and a missing panel makes
tasks fail silently (`terminal_panel.rs:237-275`, `crates/workspace/src/tasks.rs:127`). In the
Marley layout the crate keeps anything from opening it:

- **Tasks:** the crate installs its own `TerminalProvider` after the panel's (on
  `workspace::Event::PanelAdded`). It sets `reveal_target` to `Center` and hands off to
  `TerminalPanel::spawn_task`, so the task modal, runnables, code lenses and the git commit
  menu all land in the center with Zed's rerun and reuse rules intact
  (`terminal_panel.rs:632-733`). A task reruns in its last terminal, so the provider first
  moves the task's terminals out of the panel (`workspace::move_item`); one that last ran in
  the Zed layout reruns in the center. In the Zed layout it hands the task over untouched.
- **New Terminal and Open in Terminal:** `workspace::NewTerminal` goes to the panel unless a
  center terminal already has focus, and `workspace::OpenTerminal`, which every "Open in
  Terminal" menu dispatches, always does (`terminal_panel.rs:609-630`, `:736-778`). The crate
  catches both in the capture phase and opens a center terminal instead
  (`register_action_renderer`, `workspace.rs:8492-8498`; capture runs before bubble handlers,
  `crates/gpui/src/window.rs:6309-6330`).
- **The toggles:** `terminal.button` is off (D6), and the crate catches
  `terminal_panel::Toggle`, `ToggleFocus` and a `workspace::ToggleBottomDock` that would show
  the panel, as it catches New Terminal, and switches between the code and the center
  terminals instead (#449, D7).

Vim's `:!` still hard-codes the dock (`crates/vim/src/command.rs:2464-2486`). That stays.

### D3. The row model

Per project group, the rail shows a header row and, under it, the project's terminals and
then its Zed agent threads:

- **Project header:** `ProjectGroupKey::display_name`, the branch from the workspace's git
  store, a collapse chevron (the expanded flag already lives in the `MultiWorkspace` group
  state and persists with it), and a `+` that is always drawn.
- **Terminal row:** a title, a subtitle (the cwd relative to the project, or the branch), an
  agent glyph and status when an agent CLI is running in it (D5), and one dot for attention
  (a bell while unfocused) or exit.
- **Thread row:** the thread's title, its agent's icon, and running, waiting or done from the
  live conversation. Clicking it opens the thread in the Agent Panel on the right. Archive,
  history and worktree restore stay in the Agent Panel's own views; Zed keeps that
  orchestration private to its sidebar (`sidebar.rs:5338-5712`).

Rows come from a pure, gpui-free function over a snapshot of the window, the Marley pure/shim
split: the snapshot collector and the render stay thin and the function carries the tests.
Exactly one row is selected, derived in one place from where Chad is looking: the thread in
the Agent Panel if the panel has focus, else the displayed workspace's active terminal, else
that workspace's project header.

The rail rebuilds on `MultiWorkspaceEvent`s; on `workspace::Event::{ItemAdded, ItemRemoved,
ActiveItemChanged, PaneAdded}` in every held workspace (`workspace.rs:1498-1525`); on
`terminal::Event::{TitleChanged, BreadcrumbsChanged, Bell, CloseTerminal}` from each terminal
(`crates/terminal/src/terminal.rs:667-679`), the same subscription the Agent Panel uses for
its terminals (`agent_panel.rs:2242-2272`); and on changes to `ThreadMetadataStore` and the
Agent Panel's events, as Zed's sidebar does (`sidebar.rs:900-911`, `:1137-1168`). A tab moved
between panes arrives as a removal and an add, so rows reconcile by item id.

### D4. One visible way to start things

Every project header carries a `+` that opens:

- **New Terminal:** a shell at the project root, in the center.
- **New Agent:** the agent CLIs found on `PATH` (on this box: `claude`, `codex`, `gemini`,
  `opencode`), each started in a new center terminal at the root (D5).
- **New Agent Thread:** Zed's own agents in the Agent Panel: the Zed Agent plus every ACP
  agent configured in `agent_servers` (Chad's settings already carry `claude-acp`). The
  action's `agent` field is private, so the rail builds it by name,
  `cx.build_action("agent::NewExternalAgentThread", Some(json!({"agent": id})))`. The new
  thread shows up as a row under the project.

The rail header carries Add Project (the recent-projects popover). An empty window shows
`ProjectEmptyState`. From the keyboard, `ctrl-alt-n` opens the same agents and CLIs in a picker
for the active project, in either layout (#450).

### D5. Agents in terminals

A Warp-style agent is a CLI in a terminal. The rail starts one the way Zed's Terminal Threads
run `agent.terminal_init_command`: an interactive shell at the root, then the command written
once the shell finishes starting (`start_init_command_startup_handshake` and
`write_init_command_after_startup`, `terminal.rs:2142-2225`). The shell outlives the CLI, the
terminal restores and renames like any other, and an agent Chad starts by hand in any
terminal is found the same way.

Recognition reads argv, not the process name. Claude Code here is a versioned binary
(`~/.local/share/claude/versions/2.1.280`), so the name Zed puts in its default title is the
version number. `Terminal::foreground_process_command_name` (`terminal.rs:2890`) reads argv[0],
which is `claude`, and `marley_agent::agent_kind_of` classifies it. The row label is the title
the CLI sets over OSC (`breadcrumb_text`, sent as `BreadcrumbsChanged`), falling back to the
agent's name. Status: working while output arrives, waiting after a quiet spell or a bell
(Claude Code rings when `preferredNotifChannel` is `terminal_bell`), exited when the shell is
back in the foreground.

Prong 2 feeds the same rows later: a harness seat is a terminal the rail did not spawn, with
state from the harness instead of output timing (three-prong plan D10). A display-only
terminal in a center pane restores as a local shell (`workspace.rs:10587-10604`), so seats
need an `Item` wrapper of the crate's own that is not serializable.

### D6. The Marley layout's defaults

Entering the Marley layout patches Zed's defaults in memory with
`SettingsStore::update_default_settings` (`settings_store.rs:919-927`); leaving it writes
back Zed's values, which `marley_workbench::init` reads from the default settings before
anything patches them. The file itself stays upstream's, and the
patch sits below the user's settings, so anything Chad sets still wins. The values:
`terminal.button: false`, and `agent.dock: "right"` so the left column holds only the rail.
The project and git panels stay on the right, where Zed already puts them.

The default layer cannot open the sidebar at startup. `MultiWorkspace::new` starts it closed
(`multi_workspace.rs:378`), and the saved window state can reopen it but never closes it
(`workspace.rs:10335-10341`). In the Marley layout the crate opens the rail when a window is
created. Keeping a rail Chad closed shut across restarts means the rail records that in its
own serialized state and closes itself after restore, deferred, because restore runs inside
a `MultiWorkspace` update.

The one-time layout backfill in `agent_ui::init` (`maybe_backfill_editor_layout`,
`crates/agent_ui/src/agent_ui.rs:768-788`) writes Zed's editor layout into the user's
settings on installs that are not new. After the W1 rename Marley starts from a fresh data
directory, so it counts as a new install and the backfill never runs.

### D7. Keys

`reload_keymaps` clears every binding and reloads the defaults on each keymap change, and the
first reload runs right after startup (`zed.rs:2128-2248`, `:2325-2330`), so bindings a crate
adds at init are gone within a second. The Marley keymap is a JSON file in the crate, parsed
with the public `KeymapFile::load` (`keymap_file.rs:258`), tagged as a default source, and
bound from one line at the end of `load_default_keymap`. It beats Zed's defaults at equal
context depth, loses to Chad's own keymap, and survives every reload.

The terminal keys need none of it (#449). Zed's defaults bind `` ctrl-` `` to
`terminal_panel::Toggle`, `ctrl-~` to `workspace::NewTerminal` and `ctrl-j` to
`workspace::ToggleBottomDock`, all in the Workspace context, and the crate catches those
actions in the capture phase, as it catches New Terminal. That also routes the palette, the
menus and a user's own bindings to them, which new bindings would miss. The Marley keymap is for
a key with no Zed action behind it. Its first binding, since #450, is `secondary-alt-n` to
`marley::NewAgent` in the Workspace context, a chord no Zed default uses in any context. A
binding in a Terminal context on an unmodified key yields to the PTY
(`PR-claude-unmodified-terminal-chords-yield-to-the-pty-001`).

### D8. Trait details that bite

- `is_threads_list_view_active` returns `true` only while the rail shows threads. While it is
  true and the sidebar is open, Zed silences agent notifications and terminal-bell popups on
  the assumption that the sidebar shows them (`crates/agent_ui/src/conversation_view.rs:2863-2915`,
  `agent_panel.rs:2913-2936`), so the rail has to draw the attention dots for threads it
  lists.
- `cycle_project` and `cycle_thread` walk projects and rows. The `MultiWorkspace` forwards
  `NextProject`, `PreviousProject`, `NextThread` and `PreviousThread` to the sidebar even while
  it is closed (`multi_workspace.rs:2105-2140`); the rail's pair shipped in #459.
  `toggle_thread_switcher` is the rail's switcher over terminals and threads since #454.
- The rail header is the title bar's height and draws the window controls itself.
- The width follows a default until Chad drags it. While the rail stands in, the window's
  saved sidebar state stays Zed's: the rail answers `serialized_state` with the state of the
  Zed sidebar it keeps, or with the blob a window restored into it, unread (#438). The rail's
  own width is not saved yet; #442 adds it to that blob under Zed's field names, so one
  width holds across both layouts.

## Zed touchpoints this plan adds

Each lands as a row in [zed-touchpoints.md](zed-touchpoints.md) in the same change.

| Slice | Path | Change |
|---|---|---|
| W1 | `crates/paths/src/paths.rs:18` | `APP_NAME` becomes `Marley`, which moves the config, data and state directories |
| W1 | `crates/zed/Cargo.toml:9`, `:57` | `default-run` and the binary name become `marley`; `main.rs:10-16` asserts they match `APP_NAME` |
| W2 | `crates/settings_content/src/settings_content.rs`, new `marley.rs` beside it | the `marley` settings block (D0) |
| W2 | `crates/settings/src/vscode_import.rs:183-245` | `marley: None` in the literal that lists every field |
| W2 | `Cargo.toml` | the `crates/marley_rail` and `crates/marley_workbench` members |
| W2 | `crates/zed/Cargo.toml` | depend on `marley_workbench` |
| W2 | `crates/zed/src/zed.rs`, `initialize_workspace` | `marley_workbench::init(cx)` as its first line, before any window opens (a hunk inside `fn main` would give the DIFF gate a mutant no test reaches) |
| W2 | `crates/zed/src/zed.rs:536-546` | sidebar construction handed to `marley_workbench` (D1) |
| W2 | `crates/zed/src/zed.rs`, `test_action_namespaces` | the crate's action namespace in the expected list (`:5889-5984`); the test fails once a crate with new actions is linked |
| W5c | `crates/zed/src/zed.rs`, `load_default_keymap` | bind the Marley keymap after `specific-overrides` (`:2378-2380`), with a test beside Zed's keymap tests |

Defaults, task routing and terminal routing need none (D2, D6). Deferred until Chad wants
them: a Marley entry in the title bar's Panel Layout menu (`title_bar.rs`), the AI gate
(`multi_workspace_enabled`, `multi_workspace.rs:426-428`, hides any sidebar when AI is off),
and the status bar toggle's "Open Threads Sidebar" label (`crates/workspace/src/status_bar.rs:269`).
Scripts that hard-code `target/*/zed` (`script/zed-local`, `script/debug-cli`) follow the
rename only if Marley uses them.

## Recording Zed changes

[zed-touchpoints.md](zed-touchpoints.md) lists every path where the fork differs from
upstream outside Marley-owned paths: what changed, why, and how to put it back after a merge.
Code hunks carry a `// Marley:` comment so `rg "Marley:"` finds them after a conflict. W0
makes the ledger mechanical: gate:16 fails when a changed path has no row or a row names a
path that no longer differs, and a PreToolUse hook blocks a write to a Zed path until its row
exists.

## Slices

**Status (2026-09-22):** the baseline landed with #443, which put the port on
`marley/workbench-shell` behind a green `--diff` gate. W0 shipped as #436: the ledger is
enforced at the write, in gate:16 and at every commit. W1 shipped as #437: the fork runs as
`marley` with its own directories (a debug build must start inside the checkout; see
TICKET-445). W2 shipped as #438: `marley.layout` switches every window between Zed's sidebar
and the rail, live, and the rail lists each project with its center terminals. W3 shipped as
#439: each project lists its Zed agent threads with live status and attention dots, a thread
row opens the thread in the right-hand Agent Panel, and the project's `+` starts one for any
configured agent. W4 shipped as #440: a terminal running Claude Code, Codex, Gemini CLI or
OpenCode shows as an agent row with the CLI's title and a working or waiting status, and the
project's `+` starts any installed CLI in one click; #532 adds permission modes to it: a
setting, off by default and per project in the user's own settings, starts Claude Code with its
bypass or Codex with its full access, and an agent row carries a chip whenever its agent runs
without its prompts, however it was started; #510 adds worktree agents: New Agent in Worktree
makes a git worktree on a branch of its own through Zed's worktree service and starts the agent
there with its first prompt, and the rail lists each linked worktree of a project under it with
its own terminals; #560 adds a chip on each worktree row with how far its branch is behind its
base and, when a merge would stop, how many files it would stop on, read from git against the
local base; #587 brings Claude Code's trust question in a new worktree to the user, in a
notification whose Trust Folder answers it; #511 adds Review and Merge to a worktree row's menu:
Zed's branch diff against the recorded base, and a merge commit in the main checkout after checks
that fail closed, never a push, left to the Rustal workflow where it manages the repository. W5
shipped as #441: in the Marley layout
tasks, New Terminal and Open in Terminal open center terminals, and nothing opens the bottom
panel. W5b shipped as #449: `` ctrl-` ``, `ctrl-~` and `ctrl-j` work on the center terminals,
by catching Zed's actions rather than rebinding keys. W5c shipped as #450: `ctrl-alt-n` opens
a New Agent picker over Zed's agents and the installed CLIs, bound through the Marley keymap.
W6 was split at its promotion. W6a shipped as #442: a closed rail stays closed across
restarts, and one width holds in both layouts. W6b shipped as #451: in the Marley layout Zed's
Panel Layout presets explain themselves instead of rewriting its docks. W6c shipped as #452: a
terminal row renames and closes from its own menu, a double-click and a hover button, through
Zed's tab rename and close. W6d shipped as #453: the rail walks and opens its rows with Zed's
own list keys, and a project header's menu moves the project up or down. W6h, split from it,
shipped as #457: a filter under the rail's header narrows it as you type, with Zed's Threads
Sidebar's matcher, and `ctrl-f` reaches it. W6e shipped as #454: `ctrl-tab` in the rail and the
Agent Panel opens a switcher over the window's terminals and threads, most recently worked in
first, through the rail's `toggle_thread_switcher`; the center panes keep Zed's tab switcher.
W6f shipped as #455: a folder opened fresh in the Marley layout starts with a terminal at its
root, told apart from a restored project by one Zed touchpoint in `new_local`. W6g shipped as
#456: a layout round trip gives each dock back the panel the Agent Panel took over. #458
fixed the rail to follow a project's folders, so a project whose last folder goes leaves it at
once. #459, split from W6e, gave Next and Previous Project and Thread the rail's meaning: they go
round its shown projects, and its shown terminals and threads, from the row it highlights. The
rail's smaller internals wait in `docs/planning/intake/rail-internals.md`.

| Slice | Delivers | Size |
|---|---|---|
| W0 | The ledger enforced: gate:16 in `script/gates.sh` (reusing its `upstream_base`), a Write/Edit hook on Zed paths, CONSTITUTION §0, §14 and §21 naming the ledger; negative smokes | S |
| W1 | Marley's own identity: the `APP_NAME` and binary rename, then Chad's `~/.config/zed/settings.json` copied once into Marley's config directory. Extensions and the `claude-acp` agent download again on first use | S |
| W2 | The switch and a first rail: the `marley` settings block, the two layout actions, live sidebar swapping, the Marley defaults, and a rail with project headers, center terminals, click to switch, New Terminal, single selection and window controls; driven gpui tests and a live drive | L |
| W3 | Zed threads in the rail: thread rows, status and attention dots, New Agent Thread, opening a thread in the right-hand Agent Panel | M |
| W4 | Agents in terminals: New Agent, agent recognition and status | M |
| W5 | Terminal routing in the Marley layout: the task provider and the action capture (#441), then the terminal keys (#449, W5b) and New Agent from the keyboard through the Marley keymap (#450, W5c); a terminal on project open moved to W6 | M |
| W6 | Persistence and polish, split at promotion: closed-rail memory and one width (#442, W6a); Zed's layout presets (#451, W6b); rename and close (#452, W6c); keyboard navigation and reorder (#453, W6d); the switcher (#454, W6e); a first terminal (#455, W6f); the docks across a layout round trip (#456, W6g); the filter (#457, W6h); Next and Previous Project and Thread (#459, from W6e) | M |

Then the prongs continue in this shell: T0 and T1 draw Blocks inside the rail's terminals,
and C1's fleet rows become a rail section.

## Risks

- Restoring center terminals may lose rows. The workspace and the Terminal Panel each clean
  the shared `terminals` table with only their own item ids (`workspace.rs:7961-7982`,
  `terminal_panel.rs:359-378`), so each can delete the other's saved directories. Rows come
  back when a terminal's cwd changes, so it may only bite on a quick quit. #442 tests a
  restart with both kinds open before trusting it. If it is real, the fix is small and belongs
  upstream too.
- Swapping sidebars at runtime is new ground: `register_sidebar` replaces the handle but keeps
  the old sidebar's two subscriptions in `_subscriptions` (`multi_workspace.rs:387-399`). A
  dropped rail's pair goes quiet, but Zed's sidebar stays alive while the rail stands in, so
  each round trip adds a live duplicate pair on it (#442). W2's driven tests switch back and
  forth and check that focus, width, open state and notifications survive.
- `crates/zed/src/zed.rs` saw 42 upstream commits in the 90 days before the fork and
  `default.json` 52. Every touch there stays one anchored hunk.
- The rail is a gpui crate under the Marley coverage floor (100% of lines). Its test binary
  links `workspace`, `terminal_view` and `agent_ui`, so it builds slowly, and a render path is
  covered only by a driven test that finds a row by debug selector and clicks it. Keeping the
  logic in the pure row crate keeps those tests few.
- An agent terminal comes back after a restart as a shell in its old directory, without the
  CLI. Warp behaves the same. Harness-backed seats fix it in prong 2.
- If Chad sets `max_tabs`, idle center shells can be closed to make room
  (`workspace.rs:1910`). Terminals are not dirty while idle.

## Open decisions for Chad

1. The default layout: Zed until chosen (the current reading), or Marley with Zed one command
   away.
2. The AI gate: accept that any sidebar hides when AI is disabled, or spend one touchpoint to
   decouple the rail from it.
