//! Marley's pages as buttons (#739, #746).
//!
//! Home at the right of the status bar; Rusty and the Marley agent in the title bar beside the
//! guide's `?` (#746: Chad, 2026-10-11, "Rusty and Marley go to the top right next to (?) and
//! Avatar instead"). Each opens its tab in the group the window shows, in either layout. Chad,
//! 2026-10-10: "they just open in a new tab." Threads left for the rail's header and the Home page
//! (#745).

use std::sync::Arc;

use gpui::{App, AppContext as _, Context, Render, WeakEntity, Window};
use project::DisableAiSettings;
use settings::{Settings as _, SettingsStore};
use title_bar::MarleyTitleBarButtons;
use ui::{IconButtonShape, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::{ItemHandle, StatusItemView, Workspace};

/// Adds Home to every workspace's status bar, and hands the title bar the view that draws Rusty
/// and Marley. [`crate::init`] calls it once, before any window opens.
pub(crate) fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        let handle = workspace.weak_handle();
        let buttons = cx.new(|_| HomeButton { workspace: handle });
        workspace.status_bar().update(cx, |status_bar, cx| {
            status_bar.add_right_item(buttons, window, cx);
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
    fn hide_setting(&self, _cx: &App) -> Option<workspace::HideStatusItem> {
        None
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
