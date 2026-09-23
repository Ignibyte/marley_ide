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
pub mod agents;
pub mod blocks;
#[cfg(test)]
pub mod marley_workbench_tests;
mod rail;
pub mod routing;

use std::collections::{HashMap, HashSet};

use agent_ui::AgentPanel;
use fs::Fs;
use gpui::{
    AnyWindowHandle, App, AppContext as _, BorrowAppContext as _, Context, Entity, EntityId,
    Focusable as _, Global, InteractiveElement as _, ReadGlobal as _, UpdateGlobal as _,
    WeakEntity, Window, actions,
};
use settings::{
    DockPosition, KeybindSource, KeymapFile, KeymapFileLoadResult, MarleyLayout, RegisterSetting,
    Settings, SettingsContent, SettingsStore,
};
use title_bar::{UseAgenticLayout, UseClassicLayout};
use util::ResultExt as _;
use workspace::dock::{Dock, Panel as _};
use workspace::notifications::NotificationId;
use workspace::{MultiWorkspace, Sidebar as _, Toast, Workspace};

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
        /// Opens the New Agent picker: Zed's agents and the installed agent CLIs, started in
        /// this project.
        #[derive(Eq)]
        NewAgent,
        /// Scrolls the focused terminal to the start of the block before the one at its top.
        #[derive(Eq)]
        PreviousBlock,
        /// Scrolls the focused terminal to the start of the next block, or to its live screen.
        #[derive(Eq)]
        NextBlock,
    ]
);

/// The Marley keymap, bound after Zed's defaults by [`load_keymap`].
const KEYMAP: &str = include_str!("../keymap.json");

/// The resolved `marley` settings block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RegisterSetting)]
pub struct MarleySettings {
    /// Which layout the windows use.
    pub layout: MarleyLayout,
}

impl Settings for MarleySettings {
    // `default.json` carries no `marley` block, so a missing key means the Marley layout, the
    // enum's default.
    fn from_settings(content: &SettingsContent) -> Self {
        Self {
            layout: content
                .marley
                .as_ref()
                .and_then(|marley| marley.layout)
                .unwrap_or_default(),
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
    agent_bar::init(cx);
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action_renderer(|div, _, _, cx| {
            div.capture_action(cx.listener(layout_preset::<UseClassicLayout>))
                .capture_action(cx.listener(layout_preset::<UseAgenticLayout>))
        });
    })
    .detach();
    cx.on_action(|_: &UseMarleyLayout, cx: &mut App| write_layout(MarleyLayout::Marley, cx))
        .on_action(|_: &UseZedLayout, cx: &mut App| write_layout(MarleyLayout::Zed, cx))
        .observe_global::<SettingsStore>(layout_setting_changed)
        .detach();
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
