//! Marley's pages as buttons (#739, #746), and the Debug panel's button (#774).
//!
//! Home at the right of the status bar; Rusty and the Marley agent in the title bar beside the
//! guide's `?` (#746: Chad, 2026-10-11, "Rusty and Marley go to the top right next to (?) and
//! Avatar instead"). Each opens its tab in the group the window shows, in either layout. Chad,
//! 2026-10-10: "they just open in a new tab." Threads left for the rail's header and the Home page
//! (#745). In the Marley layout the Debug panel's button sits at the status bar's left (#774).

use std::sync::Arc;

use gpui::{
    App, AppContext as _, Context, Entity, Focusable as _, Render, Subscription, WeakEntity, Window,
};
use project::DisableAiSettings;
use settings::{Settings as _, SettingsStore};
use title_bar::MarleyTitleBarButtons;
use ui::{IconButtonShape, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::dock::{Dock, PanelButtons, PanelHandle};
use workspace::{HideStatusItem, ItemHandle, StatusItemView, Workspace};

/// The Debug panel's persistent name. The panel stays in its dock and is found by this name, so the
/// crate needs no `debugger_ui`.
const DEBUG_PANEL: &str = "DebugPanel";

/// Adds Home and Debug to every workspace's status bar, and hands the title bar the view that draws
/// Rusty and Marley. [`crate::init`] calls it once, before any window opens.
pub(crate) fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        let handle = workspace.weak_handle();
        let buttons = cx.new(|_| HomeButton { workspace: handle });
        let docks = workspace.all_docks().map(Entity::clone);
        let debug = cx.new(|cx| DebugButton::new(docks, cx));
        workspace.status_bar().update(cx, |status_bar, cx| {
            status_bar.add_right_item(buttons, window, cx);
            // Debug at the left end, after the left dock's buttons, which `Workspace::new` adds
            // before any other status item.
            match status_bar.position_of_item::<PanelButtons>() {
                Some(position) => status_bar.insert_item_after(position, debug, window, cx),
                None => status_bar.add_left_item(debug, window, cx),
            }
        });
    })
    .detach();
    cx.set_global(MarleyTitleBarButtons(Arc::new(|workspace, _, cx| {
        cx.new(|cx| TitleButtons::new(workspace, cx)).into()
    })));
}

/// The status bar's Home button, drawn only while its workspace is shown.
pub struct HomeButton {
    workspace: WeakEntity<Workspace>,
}

impl std::fmt::Debug for HomeButton {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("HomeButton").finish_non_exhaustive()
    }
}

impl Render for HomeButton {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // A plain closure over the weak workspace: opening a tab updates the workspace, which reads
        // its items (PR-claude-701).
        let workspace = self.workspace.clone();
        IconButton::new("marley-status-home", IconName::ListTree)
            .icon_size(IconSize::Small)
            .shape(IconButtonShape::Square)
            .tooltip(Tooltip::text("Home"))
            .on_click(move |_, window, cx| {
                workspace
                    .update(cx, |workspace, cx| {
                        crate::home_page::open_here(workspace, window, cx);
                    })
                    .log_err();
            })
    }
}

impl StatusItemView for HomeButton {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    /// Home is Marley's own page, always there.
    fn hide_setting(&self, _cx: &App) -> Option<HideStatusItem> {
        None
    }
}

/// The Debug panel's button at the status bar's left, in the Marley layout (#774).
///
/// Chad, 2026-10-11: "yes debug moves to bottom left and stays the exception". The panel stays in
/// its dock; the button is drawn and clicked as the dock's own `PanelButtons` draws it, which the
/// Marley layout's `debugger.button` default turns off.
pub struct DebugButton {
    docks: [Entity<Dock>; 3],
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for DebugButton {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DebugButton")
            .finish_non_exhaustive()
    }
}

impl DebugButton {
    fn new(docks: [Entity<Dock>; 3], cx: &mut Context<Self>) -> Self {
        // What the button's look rests on: the docks (open, and on which panel) and the settings
        // (the layout, `debugger.button`).
        let mut subscriptions: Vec<Subscription> = docks
            .iter()
            .map(|dock| cx.observe(dock, |_, _, cx| cx.notify()))
            .collect();
        subscriptions.push(cx.observe_global::<SettingsStore>(|_, cx| cx.notify()));
        Self {
            docks,
            _subscriptions: subscriptions,
        }
    }

    /// The dock holding the Debug panel, and the panel.
    fn debug_panel(&self, cx: &App) -> Option<(Entity<Dock>, Arc<dyn PanelHandle>)> {
        self.docks.iter().find_map(|dock| {
            let panel = Arc::clone(dock.read(cx).panel_for_persistent_name(DEBUG_PANEL)?);
            Some((dock.clone(), panel))
        })
    }
}

/// Whether the user's own settings turn the Debug button off, as Hide Button does.
fn debug_button_hidden_by_user(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .raw_user_settings()
        .and_then(|user| user.content.debugger.as_ref())
        .and_then(|debugger| debugger.button)
        == Some(false)
}

impl Render for DebugButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some((dock, panel)) = self.debug_panel(cx) else {
            return div().into_any_element();
        };
        // Zed's own button draws while `debugger.button` is on: a user who turns it on gets that
        // one among the dock's buttons, not a second here.
        if !crate::marley_layout(cx)
            || panel.icon(window, cx).is_some()
            || debug_button_hidden_by_user(cx)
        {
            return div().into_any_element();
        }
        let (shown, focus_handle, close) = {
            let dock = dock.read(cx);
            let shown = dock.is_open()
                && dock
                    .active_panel()
                    .is_some_and(|active| active.panel_id() == panel.panel_id());
            (shown, dock.focus_handle(cx), dock.toggle_action())
        };
        let action = if shown {
            close
        } else {
            panel.toggle_action(window, cx)
        };
        IconButton::new(("marley-status-debug", u64::from(shown)), IconName::Debug)
            .icon_size(IconSize::Small)
            .toggle_state(shown)
            .on_click({
                let action = action.boxed_clone();
                move |_, window, cx| {
                    window.focus(&focus_handle, cx);
                    window.dispatch_action(action.boxed_clone(), cx);
                }
            })
            .when(!shown, |button| {
                button.tooltip(move |_, cx| Tooltip::for_action("Debug Panel", &*action, cx))
            })
            .into_any_element()
    }
}

impl StatusItemView for DebugButton {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    /// The Debug panel's own Hide Button, which writes `debugger.button: false`.
    fn hide_setting(&self, cx: &App) -> Option<HideStatusItem> {
        let (_, panel) = self.debug_panel(cx)?;
        panel.hide_button_setting(cx)
    }
}

/// Rusty and the Marley agent in one workspace's title bar (#746).
pub struct TitleButtons {
    workspace: WeakEntity<Workspace>,
}

impl std::fmt::Debug for TitleButtons {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TitleButtons")
            .finish_non_exhaustive()
    }
}

impl TitleButtons {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        // What each button's presence rests on: the settings (AI, Rusty's switch), Rusty's
        // connection, and the Marley entry.
        cx.observe_global::<SettingsStore>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<crate::rusty::Rusty>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<crate::assistant::Assistant>(|_, cx| cx.notify())
            .detach();
        Self { workspace }
    }
}

impl Render for TitleButtons {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ai = !DisableAiSettings::get_global(cx).disable_ai;
        let rusty = crate::rusty::is_on(cx) && crate::rusty::is_connected(cx);
        let marley = ai && crate::assistant::marley_present(cx);
        let rusty_workspace = self.workspace.clone();
        let marley_workspace = self.workspace.clone();
        h_flex()
            .gap_1()
            .when(rusty, |buttons| {
                buttons.child(
                    IconButton::new("marley-title-rusty", crate::rusty::RUSTY_ICON)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Rusty"))
                        .on_click(move |_, window, cx| {
                            rusty_workspace
                                .update(cx, |workspace, cx| {
                                    crate::rusty::open_home_here(workspace, window, cx);
                                })
                                .log_err();
                        }),
                )
            })
            .when(marley, |buttons| {
                buttons.child(
                    IconButton::new("marley-title-marley", IconName::Sparkle)
                        .icon_size(IconSize::Small)
                        .tooltip(|_, cx| {
                            Tooltip::for_action(
                                "Talk to Marley",
                                &crate::assistant::TalkToMarley,
                                cx,
                            )
                        })
                        .on_click(move |_, window, cx| {
                            marley_workspace
                                .update(cx, |workspace, cx| {
                                    crate::assistant::talk_to_marley(workspace, window, cx);
                                })
                                .log_err();
                        }),
                )
            })
    }
}
