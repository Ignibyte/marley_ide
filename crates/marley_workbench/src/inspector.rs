//! Changes and Outline in one column (#837; Files left it for the left dock in #868).
//!
//! With `marley.side_inspector` on, in the Marley layout, the panels #747 opens as a tab each open
//! in one [`Inspector`] instead: a center item in a pane split to the right of the active one,
//! narrow, with a switcher at its top. As a panel tab does, it draws the dock's own panel, which
//! the dock never shows; `panel_tabs`' dock observer sends the three panels here.

use std::sync::Arc;

use gpui::{
    App, Axis, Context, EntityId, EventEmitter, FocusHandle, Focusable, Render, SharedString,
    WeakEntity, Window, px,
};
use settings::Settings as _;
use ui::{ButtonStyle, prelude::*};
use workspace::dock::PanelHandle;
use workspace::item::{Item, ItemEvent};
use workspace::{DraggedTab, SplitDirection, Workspace};

use crate::panel_tabs;
use crate::{MarleySettings, SidePanels};

/// The panels the Inspector holds, by their persistent names, with their switcher labels.
const INSPECTED: [(&str, &str); 2] = [("GitPanel", "Changes"), ("Outline Panel", "Outline")];

/// The column's width when it opens, as the board draws it.
const WIDTH: f32 = 320.;

/// Whether the panel named `name` is one the Inspector holds.
pub(crate) fn inspects(name: &str) -> bool {
    INSPECTED.iter().any(|(inspected, _)| *inspected == name)
}

/// Whether the panel named `name` opens in the Inspector now: the setting is on and it is one of
/// the three. `panel_tabs` asks only in the Marley layout.
pub(crate) fn takes(name: &str, cx: &App) -> bool {
    MarleySettings::get_global(cx).side_panels == SidePanels::Inspector && inspects(name)
}

/// Brings forward the workspace's Inspector on `panel`, or opens one in a pane split to the right
/// of the active pane (D2).
pub(crate) fn show(
    workspace: &mut Workspace,
    panel: &Arc<dyn PanelHandle>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open = workspace.items_of_type::<Inspector>(cx).next();
    if let Some(inspector) = open {
        inspector.update(cx, |inspector, cx| {
            inspector.pick(panel.panel_id(), window, cx);
        });
        workspace.activate_item(&inspector, true, true, window, cx);
        return;
    }
    let Some(inspector) = Inspector::new(workspace, panel.panel_id(), cx) else {
        return;
    };
    let inspector = cx.new(|_| inspector);
    let pane = workspace.active_pane().clone();
    let right = workspace.split_pane(pane, SplitDirection::Right, window, cx);
    right.update(cx, |pane, cx| {
        pane.add_item(Box::new(inspector), true, true, None, window, cx);
    });
}

/// The panel `id`, when the workspace's Inspector is in front showing it: `panel_tabs`' `retell`
/// tells it again that it is active (PR-claude-773).
pub(crate) fn front_panel(
    workspace: &Workspace,
    id: EntityId,
    cx: &App,
) -> Option<Arc<dyn PanelHandle>> {
    workspace
        .items_of_type::<Inspector>(cx)
        .find_map(|inspector| {
            let inspector = inspector.read(cx);
            let shown = inspector.panels.get(inspector.current)?;
            (inspector.front && shown.panel.panel_id() == id).then(|| Arc::clone(&shown.panel))
        })
}

/// Closes every Inspector: the setting turned off, or Zed's layout chosen.
pub(crate) fn close_all(cx: &mut App) {
    panel_tabs::close_items(cx, |pane, _| {
        pane.items_of_type::<Inspector>()
            .map(|inspector| inspector.entity_id())
            .collect()
    });
}

/// One of the Inspector's panels, with what the pane may ask of it kept (PR-claude-747).
struct Shown {
    panel: Arc<dyn PanelHandle>,
    focus: FocusHandle,
    label: &'static str,
}

/// The column holding Changes and Outline, one at a time under a switcher.
pub struct Inspector {
    panels: Vec<Shown>,
    current: usize,
    workspace: WeakEntity<Workspace>,
    /// Whether the Inspector is its pane's front item, as its panel was last told.
    front: bool,
    /// Whether the column has been narrowed since it opened.
    sized: bool,
}

impl std::fmt::Debug for Inspector {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Inspector")
            .field("current", &self.current)
            .finish_non_exhaustive()
    }
}

impl Inspector {
    /// The Inspector over the workspace's three panels, showing `shown`; none when the workspace
    /// holds none of them.
    fn new(workspace: &Workspace, showing: EntityId, cx: &App) -> Option<Self> {
        let panels: Vec<Shown> = INSPECTED
            .iter()
            .filter_map(|(name, label)| {
                let panel = panel_tabs::workspace_panel(workspace, name, cx)?;
                let focus = panel.panel_focus_handle(cx);
                Some(Shown {
                    panel,
                    focus,
                    label,
                })
            })
            .collect();
        let current = panels
            .iter()
            .position(|shown| shown.panel.panel_id() == showing)
            .unwrap_or_default();
        (!panels.is_empty()).then(|| Self {
            panels,
            current,
            workspace: workspace.weak_handle(),
            front: false,
            sized: false,
        })
    }

    /// Shows the panel `id`, telling the one it showed that it is inactive.
    fn pick(&mut self, id: EntityId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self
            .panels
            .iter()
            .position(|shown| shown.panel.panel_id() == id)
        else {
            return;
        };
        self.show_index(index, window, cx);
    }

    fn show_index(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index == self.current || index >= self.panels.len() {
            return;
        }
        let before = self
            .panels
            .get(self.current)
            .map(|shown| Arc::clone(&shown.panel));
        self.current = index;
        // A front Inspector's new panel hears it on its first frame, as `render` tells it.
        let was_front = std::mem::replace(&mut self.front, false);
        if was_front && let Some(before) = before {
            cx.defer_in(window, move |_, window, cx| {
                before.set_active(false, window, cx);
            });
        }
        if let Some(shown) = self.panels.get(self.current) {
            window.focus(&shown.focus, cx);
        }
        cx.notify();
    }

    /// Narrows the column to [`WIDTH`] once it has been laid out: the split gave it half the pane.
    fn narrow(&mut self, window: &Window, cx: &Context<Self>) {
        self.sized = true;
        let id = cx.entity_id();
        let workspace = self.workspace.clone();
        window.on_next_frame(move |window, cx| {
            workspace
                .update(cx, |workspace, cx| {
                    // Found here, not through `pane_of`: that reads the workspace, which this
                    // closure is updating.
                    let Some(pane) = workspace
                        .panes()
                        .iter()
                        .find(|pane| pane.read(cx).items().any(|item| item.item_id() == id))
                        .cloned()
                    else {
                        return;
                    };
                    // `resize_pane` resizes the active pane, which the Inspector's opening made it.
                    if workspace.active_pane() != &pane {
                        return;
                    }
                    let Some(bounds) = workspace.bounding_box_for_pane(&pane) else {
                        return;
                    };
                    let amount = px(WIDTH) - bounds.size.width;
                    workspace.resize_pane(Axis::Horizontal, amount, window, cx);
                })
                .ok();
        });
    }
}

impl EventEmitter<ItemEvent> for Inspector {}

impl Focusable for Inspector {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.panels
            .get(self.current)
            .map_or_else(|| cx.focus_handle(), |shown| shown.focus.clone())
    }
}

impl Render for Inspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(shown) = self.panels.get(self.current) else {
            return div().into_any_element();
        };
        let panel = Arc::clone(&shown.panel);
        // Being drawn is being in front; the panel hears it after this frame, which draws it.
        if !self.front {
            self.front = true;
            let panel = Arc::clone(&panel);
            cx.defer_in(window, move |_, window, cx| {
                panel.set_active(true, window, cx);
            });
        }
        if !self.sized {
            self.narrow(window, cx);
        }
        let colors = cx.theme().colors();
        let pane = panel_tabs::pane_of(&self.workspace, cx.entity_id(), cx);
        let split = pane
            .as_ref()
            .and_then(|pane| pane.read(cx).drag_split_direction());
        let switcher = h_flex()
            .p_1()
            .gap_1()
            .border_b_1()
            .border_color(colors.border)
            .children(self.panels.iter().enumerate().map(|(index, shown)| {
                Button::new(("marley-inspector-switch", index), shown.label)
                    .style(ButtonStyle::Subtle)
                    .toggle_state(index == self.current)
                    .on_click(cx.listener(move |inspector, _, window, cx| {
                        inspector.show_index(index, window, cx);
                    }))
            }));
        v_flex()
            .debug_selector(|| "marley-inspector".into())
            .size_full()
            .bg(colors.panel_background)
            .child(switcher)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .relative()
                    .child(panel.to_any())
                    .child(panel_tabs::drop_layer(split).on_drop(cx.listener(
                        move |_, dragged: &DraggedTab, window, cx| {
                            if let Some(pane) = pane.clone() {
                                panel_tabs::hand_drop(pane, dragged, window, cx);
                            }
                        },
                    ))),
            )
            .into_any_element()
    }
}

impl Item for Inspector {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Inspector")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ListTree))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn deactivated(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.front {
            self.front = false;
            if let Some(shown) = self.panels.get(self.current) {
                shown.panel.set_active(false, window, cx);
            }
        }
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
