//! The Containers panel (#673): this machine's container ports that no project holds.
//!
//! It sits in the right dock, opened from its button in the status bar. Until #673 these ports
//! were the rail's Containers list (#614, #669, #670); a container whose Compose folder is in a
//! project still shows under that project in the rail. A child module of the rail, so its rows
//! are drawn with the rail's own row helpers.

use gpui::{
    Action, App, ClickEvent, ClipboardItem, Context, EventEmitter, FocusHandle, Focusable, Pixels,
    Render, Subscription, WeakEntity, Window, px,
};
use marley_rail::PortSnapshot;
use settings::{Settings as _, SettingsStore};
use ui::{
    ContextMenu, Icon, IconButton, IconName, IconSize, Label, LabelSize, Tab, Tooltip, prelude::*,
    right_click_menu,
};
use util::ResultExt as _;
use workspace::Workspace;
use workspace::dock::{DockPosition, Panel, PanelEvent};

use super::{
    Cut, ROW_GROUP, RowLine, container_line, container_refused_toast, port_snapshot, row_card,
    row_label, stop_words, url_label,
};
use crate::ports::{self, ContainerStop, Ports, ProjectListener};
use crate::{MarleySettings, RailContainers, ToggleContainers, browser};

/// The panel's place among the right dock's buttons, after the Fleet and Knowledge panels'.
const ACTIVATION_PRIORITY: u32 = 22;

/// Registers the panel's toggle on every workspace, and gives each its Containers panel.
pub fn init(cx: &App) {
    cx.observe_new(
        |workspace: &mut Workspace, window: Option<&mut Window>, cx: &mut Context<Workspace>| {
            workspace.register_action(|workspace, _: &ToggleContainers, window, cx| {
                workspace.toggle_panel_focus::<ContainersPanel>(window, cx);
            });
            let Some(window) = window else {
                return;
            };
            let weak_workspace = cx.weak_entity();
            let panel = cx.new(|cx| ContainersPanel::new(weak_workspace, cx));
            workspace.add_panel(panel, window, cx);
        },
    )
    .detach();
}

/// Whether the panel and its button show: in the Marley layout, with `marley.rail_containers` on.
fn shown(cx: &App) -> bool {
    crate::marley_layout(cx)
        && MarleySettings::get_global(cx).rail_containers != RailContainers::Hidden
}

/// The machine's container ports no project holds, a row each.
pub struct ContainersPanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    /// Whether this panel keeps the port scan running, which it does while it is active.
    watching: bool,
    /// Whether it has asked its dock to close, which it does once each time it is hidden.
    closing: bool,
    _subscriptions: [Subscription; 2],
}

impl std::fmt::Debug for ContainersPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainersPanel")
            .field("watching", &self.watching)
            .finish_non_exhaustive()
    }
}

impl ContainersPanel {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        cx.on_release(|panel, cx| {
            if panel.watching {
                ports::unwatch(cx);
            }
        })
        .detach();
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
            watching: false,
            closing: false,
            _subscriptions: [
                cx.observe_global::<Ports>(|_, cx| cx.notify()),
                // The layout or the setting: a panel showing while it turns off closes its dock.
                cx.observe_global::<SettingsStore>(|this, cx| {
                    if shown(cx) {
                        this.closing = false;
                    }
                    cx.notify();
                }),
            ],
        }
    }

    /// While hidden the panel draws nothing and closes its dock, when the dock is showing it:
    /// once, after this update, since closing reads the panel (as the Knowledge panel does).
    fn close_while_hidden(&mut self, window: &Window, cx: &mut Context<Self>) {
        if std::mem::replace(&mut self.closing, true) {
            return;
        }
        let workspace = self.workspace.clone();
        window.defer(cx, move |window, cx| {
            workspace
                .update(cx, |workspace, cx| {
                    let shows_this = workspace
                        .right_dock()
                        .read(cx)
                        .visible_panel()
                        .is_some_and(|panel| panel.to_any().downcast::<Self>().is_ok());
                    if shows_this {
                        workspace.close_panel::<Self>(window, cx);
                    }
                })
                .log_err();
        });
    }

    /// Opens `url` in a Browser tab of the panel's workspace.
    fn open(&self, url: String, window: &mut Window, cx: &mut Context<Self>) {
        self.workspace
            .update(cx, |workspace, cx| {
                browser::open_url_tab(workspace, url, window, cx);
            })
            .log_err();
    }

    /// Stops the container that publishes `port`, or says why it could not, as the rail's row did.
    fn stop(&self, port: u16, window: &Window, cx: &Context<Self>) {
        let Some(container) = ports::container_at(port, cx) else {
            return;
        };
        let stop = ports::stop_container(&container, port, cx);
        let workspace = self.workspace.clone();
        cx.spawn_in(window, async move |_, cx| {
            let ContainerStop::Refused { command, reason } = stop.await else {
                return;
            };
            let toast = container_refused_toast(command, &reason);
            workspace
                .update(cx, |workspace, cx| workspace.show_toast(toast, cx))
                .log_err();
        })
        .detach();
    }

    /// A port's row, drawn as the rail drew it: the port and its process, the URL, the container;
    /// Open, Copy URL and Stop on hover and in its menu; a double-click opens it.
    fn render_row(listener: ProjectListener, cx: &Context<Self>) -> impl IntoElement {
        let snapshot = port_snapshot(listener, "", None);
        let port = snapshot.port;
        let key = (u64::from(port) << 32) | u64::from(snapshot.pid);
        let tooltip = format!(
            "{}\n{}\nDouble-click to open in a Browser tab.",
            snapshot.url, snapshot.tooltip
        );
        let lines: Vec<RowLine> = std::iter::once(RowLine {
            cut: Cut::Middle,
            ..RowLine::muted(url_label(&snapshot.url))
        })
        .chain(
            snapshot
                .container
                .as_ref()
                .map(|container| RowLine::muted(container_line(container))),
        )
        .collect();
        let url = snapshot.url.clone();
        let buttons = Self::render_buttons(&snapshot, key, cx);
        let item = row_card(
            ("marley-containers-port", key),
            format!("marley-containers-port-icon-{port}"),
            false,
            Icon::new(IconName::Server)
                .size(IconSize::Small)
                .color(Color::Muted)
                .into_any_element(),
            row_label(snapshot.title.clone(), Vec::new(), Color::Default),
            lines,
            cx,
        )
        .relative()
        .child(buttons)
        .on_click(cx.listener(move |panel, event: &ClickEvent, window, cx| {
            if event.click_count() >= 2 {
                panel.open(url.clone(), window, cx);
            }
        }));
        let panel = cx.entity().downgrade();
        let menu_url = snapshot.url;
        right_click_menu(("marley-containers-port-menu", key))
            // The row's tooltip is built only while its menu is closed, as the rail's rows do.
            .trigger(move |menu_open, _, _| {
                div()
                    .debug_selector(move || format!("marley-containers-port-{port}"))
                    .px_1()
                    .child(item.when(!menu_open, |item| item.tooltip(Tooltip::text(tooltip))))
            })
            .menu(move |window, cx| {
                let (open_panel, open_url) = (panel.clone(), menu_url.clone());
                let stop_panel = panel.clone();
                let copy_url = menu_url.clone();
                ContextMenu::build(window, cx, move |menu, _, _| {
                    menu.entry("Open in a Browser Tab", None, move |window, cx| {
                        open_panel
                            .update(cx, |panel, cx| panel.open(open_url.clone(), window, cx))
                            .log_err();
                    })
                    .entry("Copy URL", None, move |_, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(copy_url.clone()));
                    })
                    .separator()
                    .entry("Stop the Container", None, move |window, cx| {
                        stop_panel
                            .update(cx, |panel, cx| panel.stop(port, window, cx))
                            .log_err();
                    })
                })
            })
    }

    /// A row's Open, Copy URL and Stop, shown while the pointer is over it.
    fn render_buttons(snapshot: &PortSnapshot, key: u64, cx: &Context<Self>) -> Div {
        let port = snapshot.port;
        let colors = cx.theme().colors();
        let shade = colors.panel_background.blend(colors.ghost_element_hover);
        let button = |id: &'static str, icon: IconName| {
            IconButton::new((id, key), icon)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
        };
        let open_url = snapshot.url.clone();
        let open = button("marley-containers-port-open", IconName::ToolWeb)
            .tooltip(Tooltip::text("Open in a Browser Tab"))
            .on_click(cx.listener(move |panel, _, window, cx| {
                // The row under the button would take the click too.
                cx.stop_propagation();
                panel.open(open_url.clone(), window, cx);
            }));
        let copy_url = snapshot.url.clone();
        let copy = button("marley-containers-port-copy", IconName::Copy)
            .tooltip(Tooltip::text("Copy URL"))
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                cx.write_to_clipboard(ClipboardItem::new_string(copy_url.clone()));
            });
        let (stop_title, stop_meta) = stop_words(snapshot.pid, None, snapshot.container.as_ref());
        let stop = button("marley-containers-port-stop", IconName::Stop)
            .tooltip(move |_, cx| Tooltip::with_meta(stop_title, None, stop_meta.clone(), cx))
            .on_click(cx.listener(move |panel, _, window, cx| {
                cx.stop_propagation();
                panel.stop(port, window, cx);
            }));
        h_flex()
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .pl_1()
            .pr_1p5()
            .gap_0p5()
            .rounded_r_md()
            .bg(shade)
            .visible_on_hover(ROW_GROUP)
            .child(open)
            .child(copy)
            .child(stop)
    }
}

impl Render for ContainersPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !shown(cx) {
            self.close_while_hidden(window, cx);
            return div().into_any_element();
        }
        let found = Ports::containers(cx);
        let count = found.len();
        let colors = cx.theme().colors();
        v_flex()
            .key_context("MarleyContainers")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(colors.panel_background)
            .child(
                h_flex()
                    .debug_selector(|| "marley-containers-header".into())
                    .flex_none()
                    .h(Tab::container_height(cx))
                    .px_3()
                    .gap_1()
                    .border_b_1()
                    .border_color(colors.border)
                    .child(
                        Label::new("CONTAINERS")
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new(count.to_string())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    ),
            )
            .child(
                v_flex()
                    .id("marley-containers-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .py_1()
                    .gap_0p5()
                    .map(|list| {
                        if found.is_empty() {
                            list.child(
                                div().px_3().py_2().child(
                                    Label::new(
                                        "No container outside your projects publishes a port.",
                                    )
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                                ),
                            )
                        } else {
                            list.children(
                                found
                                    .into_iter()
                                    .map(|listener| Self::render_row(listener, cx)),
                            )
                        }
                    }),
            )
            .into_any_element()
    }
}

impl Focusable for ContainersPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for ContainersPanel {}

impl Panel for ContainersPanel {
    fn persistent_name() -> &'static str {
        "MarleyContainersPanel"
    }

    fn panel_key() -> &'static str {
        "MarleyContainersPanel"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        DockPosition::Right
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Right)
    }

    fn set_position(
        &mut self,
        _position: DockPosition,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        px(320.)
    }

    fn icon(&self, _window: &Window, cx: &App) -> Option<IconName> {
        shown(cx).then_some(IconName::Box)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Containers")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleContainers)
    }

    fn activation_priority(&self) -> u32 {
        ACTIVATION_PRIORITY
    }

    fn enabled(&self, cx: &App) -> bool {
        shown(cx)
    }

    // The scan runs while a rail shows; the panel keeps it running too while it is open.
    fn set_active(&mut self, active: bool, _window: &mut Window, cx: &mut Context<Self>) {
        if active == self.watching {
            return;
        }
        self.watching = active;
        if active {
            ports::watch(cx);
        } else {
            ports::unwatch(cx);
        }
    }
}
