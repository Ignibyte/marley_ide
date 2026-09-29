//! The Marley layout: a second window layout beside Zed's own, switched by `marley.layout`.
//!
//! In the Zed layout the fork behaves like upstream Zed. In the Marley layout the window's sidebar
//! is the [`Rail`] (each project with its terminals under it), the Agent Panel docks on the right,
//! and the bottom Terminal Panel's button is hidden. Everything the layout changes lives in this
//! crate and is undone when the setting goes back to `zed` (`docs/marley/workbench-shell.md`).

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

pub mod agent_bar;
pub mod agent_events;
pub mod agent_trust;
pub mod agents;
pub mod autosuggest;
pub mod block_filter;
pub mod blocks;
pub mod bookmarks;
pub mod browser;
pub mod browser_tools;
pub mod claude_plugin;
pub mod click_pause;
pub mod clients;
pub mod close_guard;
pub mod command_watch;
pub mod decisions;
pub mod english;
pub mod find;
pub mod github;
pub mod launch;
pub mod links;
pub mod markdown_commands;
#[cfg(test)]
pub mod marley_workbench_tests;
pub mod mcp;
pub mod notifications;
pub mod playwright_scripts;
pub mod ports;
pub mod push;
mod rail;
pub mod review_notes;
pub mod rich_input;
pub mod routing;
pub mod running_errors;
pub mod send_block;
pub mod send_selection;
#[cfg(unix)]
pub mod single_instance;
pub mod stall;
pub mod sticky_header;
pub mod system_one;
pub mod terminal_drive;
pub mod terminal_ids;
pub mod turn_git;
pub mod turns;
pub mod voice;
pub mod workflows;
pub mod worktree_agents;
pub mod worktree_git;
pub mod worktree_include;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use agent_ui::AgentPanel;
use anyhow::Context as _;
use fs::Fs;
use gpui::{
    AnyWindowHandle, App, AppContext as _, BorrowAppContext as _, Context, Entity, EntityId,
    Focusable as _, Global, InteractiveElement as _, ReadGlobal as _, UpdateGlobal as _,
    WeakEntity, Window, actions,
};
use settings::{
    ClaudeCodeWorktreeTrust, DockPosition, KeybindSource, KeymapFile, KeymapFileLoadResult,
    MarleyClickPauseAgents, MarleyLayout, MarleyTerminalLinks, RegisterSetting, Settings,
    SettingsContent, SettingsStore,
};
use title_bar::{UseAgenticLayout, UseClassicLayout};
use util::ResultExt as _;
use workspace::dock::{Dock, Panel as _};
use workspace::notifications::NotificationId;
use workspace::{MultiWorkspace, ProjectGroup, Sidebar as _, Toast, Workspace};

pub use rail::{KeptSidebar, Rail};

actions!(
    marley,
    [
        /// Switches every window to the Marley layout: a rail of projects with their terminals.
        #[derive(Eq)]
        UseMarleyLayout,
        /// Switches every window back to Zed's own layout.
        #[derive(Eq)]
        UseZedLayout,
        /// Opens the Settings window on Marley's page.
        #[derive(Eq)]
        OpenSettings,
        /// Opens the New Agent picker: Zed's agents and the installed agent CLIs, started in
        /// this project.
        #[derive(Eq)]
        NewAgent,
        /// Selects the block before the focused terminal's selected one, or its newest block,
        /// and scrolls it into view.
        #[derive(Eq)]
        PreviousBlock,
        /// Selects the block after the focused terminal's selected one, and past the last ends
        /// the selection; with none selected, scrolls to the start of the next block.
        #[derive(Eq)]
        NextBlock,
        /// Ends the focused terminal's block selection.
        #[derive(Eq)]
        ClearBlockSelection,
        /// Types the selected block's command at the prompt, unrun.
        #[derive(Eq)]
        ReinputBlock,
        /// Sends the selected block to a CLI agent in another terminal.
        #[derive(Eq)]
        SendBlockToAgent,
        /// Filters the output of the focused terminal's selected block, or its newest block in
        /// view, in a panel over the terminal; again, closes it.
        #[derive(Eq)]
        FilterBlock,
        /// Closes the block filter.
        #[derive(Eq)]
        CloseBlockFilter,
        /// Saves the focused terminal's selected block, or its newest block in view, as a workflow:
        /// a task in tasks.json with its parameters.
        #[derive(Eq)]
        SaveAsWorkflow,
        /// Saves the workflow the editor holds.
        #[derive(Eq)]
        SaveWorkflow,
        /// Runs the workflow with the values the prompt holds.
        #[derive(Eq)]
        RunWorkflow,
        /// Bookmarks the focused terminal's selected block, or its newest block in view, for the
        /// session; again, removes the bookmark.
        #[derive(Eq)]
        ToggleBookmark,
        /// Scrolls the focused terminal to the bookmarked block before its top.
        #[derive(Eq)]
        PreviousBookmark,
        /// Scrolls the focused terminal to the bookmarked block after its top.
        #[derive(Eq)]
        NextBookmark,
        /// Searches the focused terminal's selected block, or its newest block in view, alone,
        /// until the search bar closes; again, searches the whole terminal.
        #[derive(Eq)]
        FindInBlock,
        /// Chooses files and types their paths into the focused terminal, as dropping them does.
        #[derive(Eq)]
        AttachFile,
        /// Starts or stops a dictation with Voxtype, which types the text where the focus is.
        #[derive(Eq)]
        ToggleDictation,
        /// Opens an editor for the prompt of the CLI agent in the focused terminal; without an
        /// agent, the key goes to the terminal's program.
        #[derive(Eq)]
        RichInput,
        /// Takes the focused terminal over from the agent typing into its program, or hands it
        /// back; without such an agent, the key goes to the terminal's program (#525).
        #[derive(Eq)]
        TakeOverTerminal,
        /// Runs the command an agent waits to run in the focused terminal; without one, the key
        /// goes to the terminal's program (#556).
        #[derive(Eq)]
        RunAgentCommand,
        /// Hands the line typed at the focused terminal's prompt to an agent instead of the shell;
        /// with nothing typed, the key goes to the terminal's program (#557).
        #[derive(Eq)]
        AskAgent,
        /// Refuses the command an agent waits to run in the focused terminal; without one, the
        /// key goes to the terminal's program (#556).
        #[derive(Eq)]
        RefuseAgentCommand,
        /// Sends the rich input's text to the agent as its prompt.
        #[derive(Eq)]
        SendRichInput,
        /// Closes the rich input, keeping its text for the next time it opens.
        #[derive(Eq)]
        CloseRichInput,
        /// Makes the worktree and starts its agent with the prompt typed (#510).
        #[derive(Eq)]
        StartWorktreeAgent,
        /// Types the autosuggestion shown after the cursor; without one, the key goes to the
        /// terminal's program.
        #[derive(Eq)]
        AcceptSuggestion,
        /// Opens the Browser tab: the page Marley's own Chromium shows, starting the project's
        /// Chromium the first time.
        #[derive(Eq)]
        OpenBrowser,
        /// Opens a blank page in a new Browser tab, with the focus in its address bar.
        #[derive(Eq)]
        NewBrowserTab,
        /// Clears this project's browser data, after asking: its Browser tabs close, and every
        /// site in it signs out.
        #[derive(Eq)]
        ClearProjectBrowserData,
        /// Opens or closes the Browser tab's Playwright scripts: the ones kept for this project
        /// and for every project, each run on the tab's page.
        #[derive(Eq)]
        PlaywrightScripts,
        /// Keeps a new Playwright script for this project, named in the Browser tab's scripts
        /// tray.
        #[derive(Eq)]
        NewPlaywrightScript,
        /// Opens Browser Clients: the programs outside Marley allowed to read or act in its
        /// Browser tabs, each with a token of its own, and a form to allow another.
        #[derive(Eq)]
        BrowserClients,
        /// Allows the client named in Browser Clients.
        #[derive(Eq)]
        AllowBrowserClient,
        /// Puts the focus in the Browser tab's address bar, with its text selected.
        #[derive(Eq)]
        FocusAddressBar,
        /// Goes to what the address bar holds: a URL, a host, or a search.
        #[derive(Eq)]
        GoToAddress,
        /// Puts the page's URL back in the address bar and gives the page the focus.
        #[derive(Eq)]
        RestoreAddress,
        /// Goes back in the Browser tab's history.
        #[derive(Eq)]
        BrowserBack,
        /// Goes forward in the Browser tab's history.
        #[derive(Eq)]
        BrowserForward,
        /// Loads the Browser tab's page again.
        #[derive(Eq)]
        BrowserReload,
        /// Answers the page's dialog with OK.
        #[derive(Eq)]
        AnswerDialog,
        /// Answers the page's dialog with Cancel.
        #[derive(Eq)]
        DismissDialog,
        /// Lets the agent's click the Browser tab holds go ahead.
        #[derive(Eq)]
        AllowPausedClick,
        /// Refuses the agent's click the Browser tab holds.
        #[derive(Eq)]
        RefusePausedClick,
        /// Turns pick mode on or off in the Browser tab: the next click in the page picks the
        /// element under the pointer, for the agent.
        #[derive(Eq)]
        PickElement,
        /// Sends the pick whose caption has the focus to the agent in the terminal used last.
        #[derive(Eq)]
        SendPick,
        /// Turns annotate mode on or off in the Browser tab: a drag in the page draws a box, with
        /// a note, over it.
        #[derive(Eq)]
        Annotate,
        /// Keeps the annotation being drawn, with the note typed for it.
        #[derive(Eq)]
        KeepAnnotation,
        /// Drops the annotation being drawn.
        #[derive(Eq)]
        DropAnnotation,
        /// Saves the Browser tab's last minute, which Marley keeps while the tab shows the page,
        /// as a recording agents can read.
        #[derive(Eq)]
        RecordThis,
        /// Brings back the working agent's terminal closed last, while Marley still holds it;
        /// with none held, the key reopens Zed's closed item.
        #[derive(Eq)]
        UndoCloseTerminal,
        /// Types a reference to the editor's selection at the prompt of the agent running in a
        /// terminal of this window; with several, a picker asks which.
        #[derive(Eq)]
        SendSelectionToAgent,
        /// Asks the System One layer whether the last command of the terminal you used last
        /// failed, and shows what came back.
        #[derive(Eq)]
        SystemOneCheck,
        /// Opens Decisions: the System One layer's calls today, the day's spend and where the
        /// key comes from.
        #[derive(Eq)]
        OpenDecisions,
    ]
);

/// The Marley keymap, bound after Zed's defaults by [`load_keymap`].
const KEYMAP: &str = include_str!("../keymap.json");

/// The resolved `marley` settings block.
#[derive(Debug, Clone, PartialEq, Eq, RegisterSetting)]
pub struct MarleySettings {
    /// Which layout the windows use.
    pub layout: MarleyLayout,
    /// Whether what Marley's tools give agents has its secrets hidden (#516).
    pub redact_secrets: bool,
    /// The user's own patterns to hide from agents (#516).
    pub redaction_patterns: Vec<String>,
    /// Minutes a working Claude Code may go without an event before its row says so; 0 is never
    /// (#547).
    pub no_update_after_minutes: u64,
    /// Seconds a working Claude Code may be quiet before the stall kind's first check; 0 is none
    /// (#569).
    pub stall_check_after_seconds: u64,
    /// Where agent events are pushed for the phone, when a server and a topic are set (#535).
    pub push: Option<PushSettings>,
    /// Whether a close or a quit asks first while an agent is working (#550).
    pub ask_before_ending_a_working_agent: bool,
    /// Seconds a working terminal closed from its tab is held for undo; 0 is none (#550).
    pub undo_close_seconds: u64,
    /// Where a URL Ctrl+clicked in a terminal opens (#503).
    pub terminal_links: MarleyTerminalLinks,
    /// Whether a scrolled-back block's command is pinned over the terminal's top row (#529).
    pub sticky_command_header: bool,
    /// How long a command runs before its end notifies; 0 is never (#551).
    pub long_command_seconds: u64,
    /// Whose consequential clicks in the Browser tab wait for Allow (#571).
    pub browser_click_pause_agents: MarleyClickPauseAgents,
    /// When an agent's writes into a running program ask the user (#525).
    pub agent_terminal_writes: settings::MarleyAgentTerminalWrites,
    /// The commands an agent's `terminal_run` types at once, as regular expressions (#556).
    pub agent_command_allowlist: Vec<String>,
    /// The commands an agent's `terminal_run` always asks about (#556).
    pub agent_command_denylist: Vec<String>,
    /// Whether a command neither list matches asks (#556).
    pub agent_commands_outside_lists: settings::MarleyAgentCommandsOutsideLists,
    /// Whether an agent's commands enter the shell's history and the suggestions (#553).
    pub agent_command_history: AgentCommandHistory,
    /// Whether a line typed at a prompt that reads as English gets a hint, and an exit-127 block
    /// the Ask chip (#557).
    pub english_hint: EnglishHint,
    /// What Marley starts Claude Code and Codex with (#532).
    pub agent_permissions: agents::AgentPermissions,
    /// Who answers Claude Code's trust question in a new worktree (#587).
    pub claude_code_worktree_trust: ClaudeCodeWorktreeTrust,
    /// The System One layer (#565).
    pub system_one: system_one::SystemOneSettings,
}

/// The ntfy server agent events are pushed to (#535), as the user set it; [`push`] checks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushSettings {
    /// The server's URL.
    pub url: String,
    /// The topic on the server.
    pub topic: String,
    /// The file holding the access token, when there is one.
    pub token_file: Option<String>,
}

/// Whether an agent's `terminal_run` commands enter the shell's history and Marley's suggestions,
/// from `marley.agent_commands_in_history` (#553).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCommandHistory {
    /// They enter both, as the user's own commands do.
    Entered,
    /// They are typed with a leading space the shell keeps out, in terminals started since.
    KeptOut,
}

/// Whether English typed at a prompt is answered with a hint and an exit-127 block with the Ask
/// chip, from `marley.english_hint` (#557).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnglishHint {
    /// The hint and the chip show.
    Shown,
    /// Neither shows; Ctrl+Shift+Enter still asks.
    Hidden,
}

/// `patterns` as owned strings, for a list setting's fallback.
fn owned(patterns: &[&str]) -> Vec<String> {
    patterns.iter().map(ToString::to_string).collect()
}

impl Settings for MarleySettings {
    // `default.json`'s `marley` block names no layout, so a missing key means the Marley layout,
    // the enum's default; redaction is on unless turned off.
    fn from_settings(content: &SettingsContent) -> Self {
        let marley = content.marley.as_ref();
        Self {
            layout: marley.and_then(|marley| marley.layout).unwrap_or_default(),
            redact_secrets: marley
                .and_then(|marley| marley.redact_secrets_for_agents)
                .unwrap_or(true),
            redaction_patterns: marley
                .and_then(|marley| marley.redaction_patterns.clone())
                .unwrap_or_default(),
            no_update_after_minutes: marley
                .and_then(|marley| marley.no_update_after_minutes)
                .unwrap_or(30),
            stall_check_after_seconds: marley
                .and_then(|marley| marley.stall_check_after_seconds)
                .unwrap_or(60),
            push: marley
                .and_then(|marley| marley.push.as_ref())
                .and_then(|push| {
                    let set = |value: &Option<String>| {
                        value
                            .as_deref()
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(str::to_string)
                    };
                    Some(PushSettings {
                        url: set(&push.url)?,
                        topic: set(&push.topic)?,
                        token_file: set(&push.token_file),
                    })
                }),
            ask_before_ending_a_working_agent: marley
                .and_then(|marley| marley.ask_before_ending_a_working_agent)
                .unwrap_or(true),
            undo_close_seconds: marley
                .and_then(|marley| marley.undo_close_seconds)
                .unwrap_or(60),
            terminal_links: marley
                .and_then(|marley| marley.terminal_links)
                .unwrap_or_default(),
            sticky_command_header: marley
                .and_then(|marley| marley.sticky_command_header)
                .unwrap_or(true),
            long_command_seconds: marley
                .and_then(|marley| marley.long_command_seconds)
                .unwrap_or(30),
            browser_click_pause_agents: marley
                .and_then(|marley| marley.browser_click_pause_agents)
                .unwrap_or_default(),
            agent_terminal_writes: marley
                .and_then(|marley| marley.agent_terminal_writes)
                .unwrap_or_default(),
            agent_command_allowlist: marley
                .and_then(|marley| marley.agent_command_allowlist.clone())
                .unwrap_or_else(|| owned(marley_terminal::agent_commands::WARP_ALLOWLIST)),
            agent_command_denylist: marley
                .and_then(|marley| marley.agent_command_denylist.clone())
                .unwrap_or_else(|| owned(marley_terminal::agent_commands::WARP_DENYLIST)),
            agent_commands_outside_lists: marley
                .and_then(|marley| marley.agent_commands_outside_lists)
                .unwrap_or_default(),
            english_hint: if marley
                .and_then(|marley| marley.english_hint)
                .unwrap_or(true)
            {
                EnglishHint::Shown
            } else {
                EnglishHint::Hidden
            },
            agent_command_history: if marley
                .and_then(|marley| marley.agent_commands_in_history)
                .unwrap_or(true)
            {
                AgentCommandHistory::Entered
            } else {
                AgentCommandHistory::KeptOut
            },
            agent_permissions: agents::AgentPermissions::from_content(marley),
            claude_code_worktree_trust: marley
                .and_then(|marley| marley.claude_code_worktree_trust)
                .unwrap_or_default(),
            system_one: system_one::SystemOneSettings::from_content(
                marley.and_then(|marley| marley.system_one.as_ref()),
            ),
        }
    }
}

/// The layout the windows were last built for, and Zed's own values for the two defaults the
/// Marley layout replaces, read before anything patched them. The store keeps no copy of the
/// originals once they are patched.
struct LayoutState {
    applied: MarleyLayout,
    zed_terminal_button: Option<bool>,
    zed_agent_dock: Option<DockPosition>,
    /// Per workspace, the panel the Agent Panel took over in each dock, in `Workspace::all_docks`
    /// order, when a switch moved it there; the switch that moves it away gives the panel back.
    displaced: HashMap<EntityId, [Option<DockShown>; 3]>,
}

/// A dock as a layout switch found it: open or not, and its active panel's persistent name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DockShown {
    open: bool,
    panel: Option<&'static str>,
}

impl DockShown {
    fn of(dock: &Dock) -> Self {
        Self {
            open: dock.is_open(),
            panel: dock.active_panel().map(|panel| panel.persistent_name()),
        }
    }
}

/// A workspace's docks in `Workspace::all_docks` order, before a layout switch moved the Agent
/// Panel, and the dock that held it.
struct DocksBefore {
    window: AnyWindowHandle,
    workspace: WeakEntity<Workspace>,
    docks: [DockShown; 3],
    agent: Option<usize>,
}

impl Global for LayoutState {}

/// Runs Marley's browser relay when this process was started as one (#583).
///
/// It answers the exit code, or `None` for any other start. `main` asks before it parses its
/// arguments, since the relay's command ends with Chromium's.
#[must_use]
pub fn run_browser_relay_if_invoked() -> Option<i32> {
    marley_browser::relay::run_if_invoked()
}

/// Installs the layout switch. Call once at startup, after `settings::init` and before any
/// window opens.
pub fn init(cx: &mut App) {
    // A second call would record the patched values as Zed's own.
    if cx.has_global::<LayoutState>() {
        return;
    }
    let defaults = SettingsStore::global(cx).raw_default_settings();
    let state = LayoutState {
        applied: MarleySettings::get_global(cx).layout,
        zed_terminal_button: defaults
            .terminal
            .as_ref()
            .and_then(|terminal| terminal.button),
        zed_agent_dock: defaults.agent.as_ref().and_then(|agent| agent.dock),
        displaced: HashMap::default(),
    };
    let layout = state.applied;
    cx.set_global(state);
    apply_defaults(layout, cx);
    routing::init(cx);
    agents::init(cx);
    blocks::init(cx);
    block_filter::init(cx);
    workflows::init(cx);
    bookmarks::init(cx);
    sticky_header::init(cx);
    markdown_commands::init(cx);
    command_watch::init(cx);
    running_errors::init(cx);
    english::init(cx);
    agent_bar::init(cx);
    claude_plugin::init(cx);
    notifications::init(cx);
    close_guard::init(cx);
    terminal_ids::init(cx);
    voice::init(cx);
    rich_input::init(cx);
    send_selection::init(cx);
    review_notes::init(cx);
    launch::init(cx);
    terminal_drive::init(cx);
    autosuggest::init(cx);
    browser::init(cx);
    clients::init(cx);
    links::init(cx);
    system_one::init(cx);
    stall::init(cx);
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        worktree_agents::give_slot_reader(std::sync::Arc::clone(&workspace.app_state().fs));
        workspace.register_action_renderer(|div, _, _, cx| {
            div.capture_action(cx.listener(layout_preset::<UseClassicLayout>))
                .capture_action(cx.listener(layout_preset::<UseAgenticLayout>))
        });
        // Through the window, as the Agent Panel opens its page: an app-level dispatch from inside
        // the palette's own dispatch finds no window.
        workspace.register_action(|_, _: &OpenSettings, window, cx| {
            window.dispatch_action(
                Box::new(zed_actions::OpenSettingsPage {
                    page: "Marley".into(),
                    target: None,
                }),
                cx,
            );
        });
    })
    .detach();
    cx.on_action(|_: &UseMarleyLayout, cx: &mut App| write_layout(MarleyLayout::Marley, cx))
        .on_action(|_: &UseZedLayout, cx: &mut App| write_layout(MarleyLayout::Zed, cx))
        .observe_global::<SettingsStore>(layout_setting_changed)
        .detach();
}

/// Runs `program` with `args`, an error carrying what it printed when it fails. The agent bar's
/// adapters run `claude` and `voxtype` with it.
pub(crate) async fn run_program(program: &Path, args: &[&OsStr]) -> anyhow::Result<()> {
    let name = program
        .file_name()
        .unwrap_or(program.as_os_str())
        .to_string_lossy();
    let output = util::command::new_command(program)
        .args(args)
        .output()
        .await
        .context(format!("running `{name}`"))?;
    anyhow::ensure!(
        output.status.success(),
        "`{name} {}` failed: {}",
        args.iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

/// Binds the Marley keymap as a default source.
///
/// `crates/zed` calls this at the end of `load_default_keymap`, which every keymap reload runs
/// again before binding the user's keymap: the Marley bindings beat Zed's defaults at the same
/// context depth and lose to the user's.
pub fn load_keymap(cx: &mut App) {
    load_keymap_from(KEYMAP, cx).log_err();
}

/// Parses `keymap` and binds all of it, tagged as a default source, or none of it when any part
/// fails to load.
fn load_keymap_from(keymap: &str, cx: &mut App) -> anyhow::Result<()> {
    let mut bindings = match KeymapFile::load(keymap, cx) {
        KeymapFileLoadResult::Success { key_bindings } => key_bindings,
        KeymapFileLoadResult::SomeFailedToLoad { error_message, .. } => {
            anyhow::bail!("the Marley keymap did not load: {error_message}")
        }
        KeymapFileLoadResult::JsonParseFailure { error } => {
            return Err(error.context("the Marley keymap is not valid JSON"));
        }
    };
    for binding in &mut bindings {
        binding.set_meta(KeybindSource::Default.meta());
    }
    cx.bind_keys(bindings);
    Ok(())
}

/// Builds and registers the sidebar the current layout calls for.
///
/// `crates/zed` calls this where it used to build Zed's sidebar, inside the same deferred callback,
/// so window restore still finds a registered sidebar; the settings observer calls it for every
/// window when the layout changes.
pub fn register_sidebar(
    multi_workspace: &Entity<MultiWorkspace>,
    window: &mut Window,
    cx: &mut App,
) {
    let (current, had_focus) = {
        let multi_workspace = multi_workspace.read(cx);
        let sidebar = multi_workspace.sidebar();
        (
            sidebar.map(workspace::SidebarHandle::to_any),
            sidebar.is_some_and(|sidebar| sidebar.focus_handle(cx).contains_focused(window, cx)),
        )
    };
    let focus_handle = match MarleySettings::get_global(cx).layout {
        MarleyLayout::Zed => {
            // Back from the rail: the Zed sidebar it kept, as open as it was, or a fresh one given
            // the state the rail was holding for it.
            let (kept, state) = current
                .and_then(|view| view.downcast::<Rail>().ok())
                .map(|rail| rail.update(cx, |rail, _| rail.take_zed_sidebar()))
                .unwrap_or_default();
            let (sidebar, open) = kept.unwrap_or_else(|| {
                let sidebar =
                    cx.new(|cx| sidebar::Sidebar::new(multi_workspace.clone(), window, cx));
                if let Some(state) = state {
                    sidebar.update(cx, |sidebar, cx| {
                        sidebar.restore_serialized_state(&state, window, cx);
                    });
                }
                (sidebar, multi_workspace.read(cx).sidebar_open())
            });
            let focus_handle = sidebar.focus_handle(cx);
            multi_workspace.update(cx, |multi_workspace, cx| {
                multi_workspace.register_sidebar(sidebar, cx);
                if open {
                    // Re-opening points every workspace's sidebar focus handle at the new one.
                    multi_workspace.open_sidebar(cx);
                } else if multi_workspace.sidebar_open() {
                    // Zed's sidebar was closed before the rail opened the window's sidebar.
                    multi_workspace.close_sidebar(window, cx);
                }
                // Registering does not redraw; this does.
                multi_workspace.set_sidebar_overlay(None, cx);
            });
            focus_handle
        }
        MarleyLayout::Marley => {
            let kept = current
                .and_then(|view| view.downcast::<sidebar::Sidebar>().ok())
                .map(|sidebar| (sidebar, multi_workspace.read(cx).sidebar_open()));
            let rail = cx.new(|cx| Rail::new(multi_workspace, kept, window, cx));
            let focus_handle = rail.focus_handle(cx);
            multi_workspace.update(cx, |multi_workspace, cx| {
                multi_workspace.register_sidebar(rail, cx);
                // With AI off no sidebar is drawn, and opening one would pin a workspace that
                // single-workspace mode then never lets go of.
                if multi_workspace.multi_workspace_enabled(cx) {
                    multi_workspace.open_sidebar(cx);
                }
                // Zed's thread switcher is an overlay of Zed's sidebar; it must not outlive it.
                multi_workspace.set_sidebar_overlay(None, cx);
            });
            focus_handle
        }
    };
    // Dropping a focused sidebar would leave the window with nothing focused.
    if had_focus {
        focus_handle.focus(window, cx);
    }
}

/// The label for each group, as the rail shows it and the browser tools name a tab's project
/// (#574): its roots' last components, with parent components added where two groups' labels
/// would otherwise read the same, through the public naming functions Zed's own project groups
/// use (`compute_disambiguation_details`, `path_suffix`, `display_name`).
pub(crate) fn group_names(groups: &[ProjectGroup]) -> Vec<String> {
    let roots: BTreeSet<PathBuf> = groups
        .iter()
        .flat_map(|group| group.key.path_list().paths().to_vec())
        .collect();
    let roots: Vec<PathBuf> = roots.into_iter().collect();
    let depths = util::disambiguate::compute_disambiguation_details(&roots, |root, depth| {
        project::path_suffix(root, depth)
    });
    let depth_by_root: HashMap<PathBuf, usize> = roots.into_iter().zip(depths).collect();
    groups
        .iter()
        .map(|group| group.key.display_name(&depth_by_root).to_string())
        .collect()
}

/// Marks the toast [`layout_preset`] shows.
struct LayoutPresets;

/// `workspace::UseClassicLayout` and `workspace::UseAgenticLayout`: Zed's Panel Layout presets
/// rewrite the docks the Marley layout sets, `agent.dock` among them, so in that layout they say
/// so and offer Zed's layout instead of writing. In the Zed layout they go on to Zed.
fn layout_preset<A>(workspace: &mut Workspace, _: &A, _: &mut Window, cx: &mut Context<Workspace>) {
    if marley_layout(cx) {
        cx.stop_propagation();
        let toast = Toast::new(
            NotificationId::unique::<LayoutPresets>(),
            "Panel Layout presets belong to Zed's layout: the Marley layout places its own panels.",
        )
        .on_click("Use Zed's Layout", use_zed_layout);
        workspace.show_toast(toast, cx);
    }
}

/// The layout toast's button.
fn use_zed_layout(_: &mut Window, cx: &mut App) {
    write_layout(MarleyLayout::Zed, cx);
}

/// Whether the windows use the Marley layout, read where each routing decision is made.
fn marley_layout(cx: &App) -> bool {
    MarleySettings::get_global(cx).layout == MarleyLayout::Marley
}

fn write_layout(layout: MarleyLayout, cx: &App) {
    if MarleySettings::get_global(cx).layout == layout {
        return;
    }
    settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
        content.marley.get_or_insert_default().layout = Some(layout);
    });
}

fn layout_setting_changed(cx: &mut App) {
    let layout = MarleySettings::get_global(cx).layout;
    // Patching the defaults below notifies this observer again; the layout it last applied is
    // what keeps that from looping.
    let changed = cx.update_global::<LayoutState, _>(|state, _| {
        std::mem::replace(&mut state.applied, layout) != layout
    });
    if !changed {
        return;
    }
    // Zed's docks move the Agent Panel in their own observers, which run after this one, so the
    // docks are noted before they move and settled once they have.
    let before = docks_before(cx);
    apply_defaults(layout, cx);
    for window in cx.windows() {
        window
            .update(cx, |_, window, cx| {
                if let Some(Some(multi_workspace)) = window.root::<MultiWorkspace>() {
                    register_sidebar(&multi_workspace, window, cx);
                }
            })
            .log_err();
    }
    cx.defer(move |cx| settle_docks(before, cx));
}

/// Every workspace's docks, before a layout switch moves the Agent Panel.
fn docks_before(cx: &App) -> Vec<DocksBefore> {
    cx.windows()
        .into_iter()
        .filter_map(|window| Some((window, window.downcast::<MultiWorkspace>()?.read(cx).ok()?)))
        .flat_map(|(window, multi_workspace)| {
            multi_workspace.workspaces().map(move |workspace| {
                let docks = workspace.read(cx).all_docks();
                DocksBefore {
                    window,
                    workspace: workspace.downgrade(),
                    docks: docks.map(|dock| DockShown::of(dock.read(cx))),
                    agent: docks
                        .iter()
                        .position(|dock| dock.read(cx).has_agent_panel(cx)),
                }
            })
        })
        .collect()
}

/// Once a layout switch has moved the Agent Panel, each workspace's docks as they were: see
/// [`settle_workspace`].
fn settle_docks(before: Vec<DocksBefore>, cx: &mut App) {
    let live: Vec<(Entity<Workspace>, DocksBefore)> = before
        .into_iter()
        .filter_map(|docks| Some((docks.workspace.upgrade()?, docks)))
        .collect();
    let workspaces: HashSet<EntityId> = live
        .iter()
        .map(|(workspace, _)| workspace.entity_id())
        .collect();
    for (workspace, docks) in &live {
        docks
            .window
            .update(cx, |_, window, cx| {
                settle_workspace(workspace, docks, window, cx);
            })
            .log_err();
    }
    cx.update_global::<LayoutState, _>(|state, _| {
        state.displaced.retain(|workspace, slots| {
            workspaces.contains(workspace) && slots.iter().any(Option::is_some)
        });
    });
}

/// Zed's move opens the dock the Agent Panel enters on it, when it was visible, and forgets the
/// panel that dock showed; it closes the dock it leaves. So the dock it entered remembers the
/// panel it showed, and the dock it left gets back what it remembers while the Agent Panel was
/// still what it showed: a panel the user chose there in between stands, and so does a close.
fn settle_workspace(
    workspace: &Entity<Workspace>,
    before: &DocksBefore,
    window: &mut Window,
    cx: &mut App,
) {
    let docks = workspace.read(cx).all_docks().map(Entity::clone);
    let after = docks
        .iter()
        .position(|dock| dock.read(cx).has_agent_panel(cx));
    let Some((from, to)) = before.agent.zip(after).filter(|(from, to)| from != to) else {
        return;
    };
    let agent = Some(AgentPanel::persistent_name());
    let entered = before.docks[to];
    let key = workspace.entity_id();
    let memory = cx.update_global::<LayoutState, _>(|state, _| {
        let slots = state.displaced.entry(key).or_default();
        if entered.panel.is_some_and(|panel| Some(panel) != agent) {
            slots[to] = Some(entered);
        }
        slots[from].take()
    });
    if let Some(memory) = memory
        && before.docks[from].panel == agent
    {
        let open = memory.open && before.docks[from].open;
        docks[from].update(cx, |dock, cx| {
            if let Some(index) = memory
                .panel
                .and_then(|panel| dock.panel_index_for_persistent_name(panel, cx))
            {
                dock.activate_panel(index, window, cx);
            }
            dock.set_open(open, window, cx);
        });
    }
}

fn apply_defaults(layout: MarleyLayout, cx: &mut App) {
    let state = cx.global::<LayoutState>();
    // Never `None`: both settings' readers unwrap their default.
    let (terminal_button, agent_dock) = match layout {
        MarleyLayout::Marley => (Some(false), Some(DockPosition::Right)),
        MarleyLayout::Zed => (state.zed_terminal_button, state.zed_agent_dock),
    };
    SettingsStore::update_global(cx, |store, cx| {
        store.update_default_settings(cx, |defaults| {
            defaults.terminal.get_or_insert_default().button = terminal_button;
            defaults.agent.get_or_insert_default().dock = agent_dock;
        });
    });
}
