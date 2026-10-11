//! Marley's pages at the right of the status bar (#739).
//!
//! Home, Rusty, Threads and the Marley agent, each opening its tab in the group the window shows,
//! in either layout. Chad, 2026-10-10: "they just open in a new tab." They took the place of the
//! rail header's Rusty button.

use gpui::{App, AppContext as _, Context, Render, WeakEntity, Window};
use project::DisableAiSettings;
use settings::{Settings as _, SettingsStore};
use ui::{IconButtonShape, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::{ItemHandle, StatusItemView, Workspace};

/// Adds the buttons to every workspace's status bar. [`crate::init`] calls it once.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        let handle = workspace.weak_handle();
        let buttons = cx.new(|cx| RustalButtons::new(handle, cx));
        workspace.status_bar().update(cx, |status_bar, cx| {
            status_bar.add_right_item(buttons, window, cx);
        });
    })
    .detach();
}

/// The buttons of one workspace's status bar, which is drawn only while its workspace is shown.
pub struct RustalButtons {
    workspace: WeakEntity<Workspace>,
}

impl std::fmt::Debug for RustalButtons {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RustalButtons")
            .finish_non_exhaustive()
    }
}

impl RustalButtons {
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

impl Render for RustalButtons {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ai = !DisableAiSettings::get_global(cx).disable_ai;
        let rusty = crate::rusty::is_on(cx) && crate::rusty::is_connected(cx);
        let marley = ai && crate::assistant::marley_present(cx);
        // Plain closures over the weak workspace: opening a tab updates the workspace, which reads
        // its items (PR-claude-701).
        let home_workspace = self.workspace.clone();
        let rusty_workspace = self.workspace.clone();
        let threads_workspace = self.workspace.clone();
        let marley_workspace = self.workspace.clone();
        h_flex()
            .gap_0p5()
            .child(
                IconButton::new("marley-status-home", IconName::ListTree)
                    .icon_size(IconSize::Small)
                    .shape(IconButtonShape::Square)
                    .tooltip(Tooltip::text("Home"))
                    .on_click(move |_, window, cx| {
                        home_workspace
                            .update(cx, |workspace, cx| {
                                crate::home_page::open_here(workspace, window, cx);
                            })
                            .log_err();
                    }),
            )
            .when(rusty, |buttons| {
                buttons.child(
                    IconButton::new("marley-status-rusty", crate::rusty::RUSTY_ICON)
                        .icon_size(IconSize::Small)
                        .shape(IconButtonShape::Square)
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
            .when(ai, |buttons| {
                buttons.child(
                    IconButton::new("marley-status-threads", IconName::Thread)
                        .icon_size(IconSize::Small)
                        .shape(IconButtonShape::Square)
                        .tooltip(|_, cx| {
                            Tooltip::for_action("Threads", &crate::threads_page::OpenThreads, cx)
                        })
                        .on_click(move |_, window, cx| {
                            threads_workspace
                                .update(cx, |workspace, cx| {
                                    crate::threads_page::open(workspace, window, cx);
                                })
                                .log_err();
                        }),
                )
            })
            .when(marley, |buttons| {
                buttons.child(
                    IconButton::new("marley-status-marley", IconName::Sparkle)
                        .icon_size(IconSize::Small)
                        .shape(IconButtonShape::Square)
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

impl StatusItemView for RustalButtons {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    /// The buttons follow their features' own switches (AI, Rusty, the Marley agent), and Home is
    /// Marley's own page.
    fn hide_setting(&self, _cx: &App) -> Option<workspace::HideStatusItem> {
        None
    }
}
