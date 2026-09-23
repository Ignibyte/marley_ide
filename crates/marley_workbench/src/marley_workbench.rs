//! The Marley layout: a second window layout beside Zed's own, switched by `marley.layout`.
//!
//! In the Zed layout the fork behaves like upstream Zed. In the Marley layout the window's sidebar
//! is the [`Rail`] (each project with its terminals under it), the Agent Panel docks on the right,
//! and the bottom Terminal Panel's button is hidden. Everything the layout changes lives in this
//! crate and is undone when the setting goes back to `zed` (`docs/marley/workbench-shell.md`).

#[cfg(test)]
pub mod marley_workbench_tests;
mod rail;
pub mod routing;

use fs::Fs;
use gpui::{
    App, AppContext as _, BorrowAppContext as _, Entity, Focusable as _, Global, ReadGlobal as _,
    UpdateGlobal as _, Window, actions,
};
use settings::{
    DockPosition, MarleyLayout, RegisterSetting, Settings, SettingsContent, SettingsStore,
};
use util::ResultExt as _;
use workspace::{MultiWorkspace, Sidebar as _};

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
    ]
);

/// The resolved `marley` settings block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RegisterSetting)]
pub struct MarleySettings {
    /// Which layout the windows use.
    pub layout: MarleyLayout,
}

impl Settings for MarleySettings {
    // `default.json` carries no `marley` block, so a missing key means Zed's layout.
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
    };
    let layout = state.applied;
    cx.set_global(state);
    apply_defaults(layout, cx);
    routing::init(cx);
    cx.on_action(|_: &UseMarleyLayout, cx: &mut App| write_layout(MarleyLayout::Marley, cx))
        .on_action(|_: &UseZedLayout, cx: &mut App| write_layout(MarleyLayout::Zed, cx))
        .observe_global::<SettingsStore>(layout_setting_changed)
        .detach();
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
