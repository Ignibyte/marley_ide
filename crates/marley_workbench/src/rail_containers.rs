//! The Containers panel (#673): this machine's container ports that no project holds.
//!
//! It sits in the right dock, opened from its button in the status bar. Until #673 these ports
//! were the rail's Containers list (#614, #669, #670); a container whose Compose folder is in a
//! project still shows under that project in the rail. Since #890 a row is a container with its
//! ports, as the Splits board draws it. A child module of the rail, whose Stop words it shares.

use gpui::{
    Action, App, ClickEvent, ClipboardItem, Context, EventEmitter, FocusHandle, Focusable, Pixels,
    Render, Subscription, WeakEntity, Window, px,
};
use marley_rail::PortContainer;
use settings::{Settings as _, SettingsStore};
use ui::{
    Button, ContextMenu, IconName, Label, LabelSize, Tab, Tooltip, prelude::*, right_click_menu,
};
use util::ResultExt as _;
use workspace::Workspace;
use workspace::dock::{DockPosition, Panel, PanelEvent};

use super::{container_refused_toast, stop_words};
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

    /// A container's row (#890): a dot, its name over its ports, each a link that opens it in a
    /// Browser tab, and Stop; its menu opens or copies each port and stops it. A double-click
    /// opens its first port.
    fn render_container(index: usize, container: Boxed, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let Boxed { name, ports } = container;
        let first = ports.first().cloned();
        let links = ports.iter().map(|(port, url)| {
            let url = url.clone();
            let tooltip = format!("Open {url} in a Browser tab");
            div()
                .id(("marley-containers-port-link", u32::from(*port)))
                .debug_selector({
                    let port = *port;
                    move || format!("marley-containers-port-{port}")
                })
                .cursor_pointer()
                .child(
                    Label::new(format!(":{port}"))
                        .size(LabelSize::XSmall)
                        .buffer_font(cx)
                        .color(Color::Accent),
                )
                .tooltip(Tooltip::text(tooltip))
                .on_click(cx.listener(move |panel, _, window, cx| {
                    // The row under the link would take the click too.
                    cx.stop_propagation();
                    panel.open(url.clone(), window, cx);
                }))
        });
        let stop_port = first.as_ref().map(|(port, _)| *port);
        let (stop_title, stop_meta) = first
            .as_ref()
            .and_then(|(port, _)| ports::container_at(*port, cx))
            .map_or(("Stop the Container", String::new()), |found| {
                stop_words(0, None, Some(&rail_container(&found)))
            });
        let stop = Button::new(("marley-containers-stop", index), "Stop")
            .label_size(LabelSize::Small)
            .color(Color::Error)
            .tooltip(move |_, cx| Tooltip::with_meta(stop_title, None, stop_meta.clone(), cx))
            .on_click(cx.listener(move |panel, _, window, cx| {
                cx.stop_propagation();
                if let Some(port) = stop_port {
                    panel.stop(port, window, cx);
                }
            }));
        let open_first = first;
        let item = h_flex()
            .id(("marley-containers-box", index))
            .mx_2()
            .px_2()
            .py_1p5()
            .gap_2p5()
            .rounded_md()
            .bg(colors.element_background)
            .child(
                div()
                    .flex_none()
                    .size_2()
                    .rounded_full()
                    .bg(cx.theme().status().success),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(Label::new(name).size(LabelSize::Small).truncate())
                    .child(h_flex().gap_1p5().flex_wrap().children(links)),
            )
            .child(stop)
            .on_click(cx.listener(move |panel, event: &ClickEvent, window, cx| {
                if event.click_count() >= 2
                    && let Some((_, url)) = open_first.clone()
                {
                    panel.open(url, window, cx);
                }
            }));
        let panel = cx.entity().downgrade();
        right_click_menu(("marley-containers-box-menu", index))
            .trigger(move |_, _, _| div().child(item))
            .menu(move |window, cx| container_menu(&panel, &ports, stop_port, window, cx))
    }
}

/// A container row's menu: Open and Copy for each of its ports, then Stop the Container.
fn container_menu(
    panel: &WeakEntity<ContainersPanel>,
    ports: &[(u16, String)],
    stop_port: Option<u16>,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<ContextMenu> {
    let (panel, ports) = (panel.clone(), ports.to_vec());
    ContextMenu::build(window, cx, move |menu, _, _| {
        let menu = ports.iter().fold(menu, |menu, (port, url)| {
            let (open_panel, open_url, copy_url) = (panel.clone(), url.clone(), url.clone());
            menu.entry(
                format!("Open :{port} in a Browser Tab"),
                None,
                move |window, cx| {
                    open_panel
                        .update(cx, |panel, cx| panel.open(open_url.clone(), window, cx))
                        .log_err();
                },
            )
            .entry(format!("Copy :{port}'s URL"), None, move |_, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(copy_url.clone()));
            })
        });
        let stop_panel = panel.clone();
        menu.separator()
            .entry("Stop the Container", None, move |window, cx| {
                if let Some(port) = stop_port {
                    stop_panel
                        .update(cx, |panel, cx| panel.stop(port, window, cx))
                        .log_err();
                }
            })
    })
}

/// One container's ports, as the panel lists it (#890).
struct Boxed {
    name: String,
    /// Each port, with the URL that reaches it, in order.
    ports: Vec<(u16, String)>,
}

/// `found`'s listeners a row per container: by its engine and name, else by the address its
/// proxies send to, else a port alone.
fn by_container(found: Vec<ProjectListener>) -> Vec<Boxed> {
    let mut boxes: Vec<(String, Boxed)> = Vec::new();
    for listener in found {
        let port = listener.listener.address.port();
        let url = marley_browser::ports::url(listener.listener.address);
        let (key, name) = match &listener.container {
            Some(container) => {
                let engine = container.engine.command();
                match (&container.name, &container.target) {
                    (Some(name), _) => (format!("{engine} {name}"), name.clone()),
                    (None, Some(target)) => {
                        let host = target
                            .rsplit_once(':')
                            .map_or(target.as_str(), |(host, _)| host);
                        (format!("{engine} → {host}"), format!("→ {host}"))
                    }
                    (None, None) => (format!(":{port}"), format!("a {engine} container")),
                }
            }
            None => (format!(":{port}"), listener.listener.name.clone()),
        };
        match boxes.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, existing)) => existing.ports.push((port, url)),
            None => boxes.push((
                key,
                Boxed {
                    name,
                    ports: vec![(port, url)],
                },
            )),
        }
    }
    boxes
        .into_iter()
        .map(|(_, mut boxed)| {
            boxed.ports.sort_unstable();
            boxed
        })
        .collect()
}

/// The rail's shape of a container, for Stop's words.
fn rail_container(container: &ports::ContainerRef) -> PortContainer {
    PortContainer {
        engine: container.engine.command().to_string(),
        name: container.name.clone(),
        target: container.target.clone(),
    }
}

impl Render for ContainersPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !shown(cx) {
            self.close_while_hidden(window, cx);
            return div().into_any_element();
        }
        let found = by_container(Ports::containers(cx));
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
                    .py_2()
                    .gap_1p5()
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
                            list.children(found.into_iter().enumerate().map(
                                |(index, container)| Self::render_container(index, container, cx),
                            ))
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
