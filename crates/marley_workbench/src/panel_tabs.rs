//! Side panels as center tabs (#747).
//!
//! Chad, 2026-10-11, wanted everything to open in the main pane as a tab, the panels behind the
//! status bar's right-hand buttons included, to split and drag like any tab: "everything opens in
//! the main pane now then the user can just split panel it out themselves".
//!
//! In the Marley layout a moved panel (Files, Git, Outline here; #748 adds more) stays registered
//! in its dock, so Zed's code still finds it, but the dock never draws it: a [`PanelTab`] in the
//! active pane draws the panel's own entity. Every way Zed shows the panel ends in the dock opening
//! on it (the status bar's button, its `ToggleFocus` and `Toggle`, `focus_panel`, Reveal in
//! Project Panel), so one observer per dock catches them all: it closes the dock again, gives it
//! back the panel it showed before, and, when the panel took the focus, opens or brings forward
//! its tab. Observers run in the effect flush of the update that opened the dock, before the next
//! frame, so the dock never shows the panel. The Zed layout keeps its docks.

use std::sync::Arc;

use gpui::{
    App, AppContext as _, Context, DefiniteLength, Entity, EntityId, EventEmitter, FocusHandle,
    Focusable, Render, SharedString, WeakEntity, Window,
};
use settings::SettingsStore;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::dock::{Dock, PanelHandle};
use workspace::item::{Item, ItemEvent};
use workspace::pane::{SplitDown, SplitLeft, SplitMode, SplitRight, SplitUp};
use workspace::{DraggedTab, MultiWorkspace, Pane, SaveIntent, SplitDirection, Workspace};

/// The panels a tab draws in the Marley layout, by their persistent names (#747's D3, #748's D1).
/// The Agent Panel stays in the right dock, and Debug in the bottom one.
const MOVED: [&str; 7] = [
    "Project Panel",
    "GitPanel",
    "Outline Panel",
    "CollabPanel",
    "MarleyFleetPanel",
    "MarleyKnowledgePanel",
    "MarleyContainersPanel",
];

/// Whether the panel named `name` opens as a tab in the Marley layout.
fn is_moved(name: &str) -> bool {
    MOVED.contains(&name)
}

/// Installs the dock observers and the split keys on every workspace, and closes every panel tab
/// when the Zed layout is chosen. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        let docks: Vec<Entity<Dock>> = workspace.all_docks().into_iter().cloned().collect();
        for dock in docks {
            // The dock's active panel and whether it was open, at its last change.
            let mut last: (Option<EntityId>, bool) = (None, false);
            cx.observe_in(&dock, window, move |workspace, dock, window, cx| {
                last = dock_changed(workspace, &dock, last, window, cx);
            })
            .detach();
        }
        workspace.register_action_renderer(|div, _, _, cx| {
            div.capture_action(cx.listener(|workspace, split: &SplitLeft, window, cx| {
                move_split(workspace, SplitDirection::Left, split.mode, window, cx);
            }))
            .capture_action(cx.listener(|workspace, split: &SplitRight, window, cx| {
                move_split(workspace, SplitDirection::Right, split.mode, window, cx);
            }))
            .capture_action(cx.listener(|workspace, split: &SplitUp, window, cx| {
                move_split(workspace, SplitDirection::Up, split.mode, window, cx);
            }))
            .capture_action(cx.listener(
                |workspace, split: &SplitDown, window, cx| {
                    move_split(workspace, SplitDirection::Down, split.mode, window, cx);
                },
            ))
        });
    })
    .detach();
    let mut marley_layout = crate::marley_layout(cx);
    cx.observe_global::<SettingsStore>(move |cx| {
        let now = crate::marley_layout(cx);
        // The Zed layout shows every panel in its dock (D10); a panel turned off (Knowledge while
        // Rusty is off, Containers while hidden) loses its tab, as Zed hides its button (#748).
        let leaving = marley_layout && !now;
        close_tabs(cx, |panel, cx| leaving || !panel.enabled(cx));
        marley_layout = now;
    })
    .detach();
}

/// Whether `item` is a panel tab drawing a panel of type `P`: Fleet counts its tab as a place it
/// shows (#748).
pub(crate) fn shows_panel<P: 'static>(item: &dyn workspace::item::ItemHandle, cx: &App) -> bool {
    item.downcast::<PanelTab>()
        .is_some_and(|tab| tab.read(cx).panel.to_any().downcast::<P>().is_ok())
}

/// A dock of `workspace` changed. In the Marley layout, a dock open on a moved panel closes again,
/// or goes back to the panel it showed before; the panel's tab opens when the panel took the focus
/// (D2), and the panel hears again whether its tab is in front (D5). Answers the dock's state to
/// remember.
fn dock_changed(
    workspace: &mut Workspace,
    dock: &Entity<Dock>,
    last: (Option<EntityId>, bool),
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> (Option<EntityId>, bool) {
    let (open, active) = {
        let dock = dock.read(cx);
        (dock.is_open(), dock.active_panel().cloned())
    };
    let now = (active.as_ref().map(|panel| panel.panel_id()), open);
    if !crate::marley_layout(cx) {
        return now;
    }
    let Some(panel) = active.filter(|panel| open && is_moved(panel.persistent_name())) else {
        retell(workspace, last.0, window, cx);
        return now;
    };
    let asked = panel.activation_focus_handle(cx).is_focused(window)
        || panel.panel_focus_handle(cx).is_focused(window);
    // The panel the dock showed before, open, when that was another one: it shows again.
    let before = last
        .0
        .filter(|id| last.1 && *id != panel.panel_id())
        .and_then(|id| {
            let dock = dock.read(cx);
            let name = dock.panel_for_id(id)?.persistent_name();
            dock.panel_index_for_persistent_name(name, cx)
        });
    dock.update(cx, |dock, cx| match before {
        Some(index) => dock.activate_panel(index, window, cx),
        None => dock.set_open(false, window, cx),
    });
    if asked {
        show_tab(workspace, Arc::clone(&panel), window, cx);
    }
    retell(workspace, Some(panel.panel_id()), window, cx);
    let dock = dock.read(cx);
    (
        dock.active_panel().map(|panel| panel.panel_id()),
        dock.is_open(),
    )
}

/// Tells the panel `id` again that it is active, when its tab is in front: the dock's own calls
/// (`set_open`, `activate_panel`) told it otherwise (D5).
fn retell(
    workspace: &Workspace,
    id: Option<EntityId>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let Some(id) = id else {
        return;
    };
    let tab = workspace
        .items_of_type::<PanelTab>(cx)
        .find(|tab| tab.read(cx).panel.panel_id() == id && tab.read(cx).front);
    if let Some(tab) = tab {
        let panel = Arc::clone(&tab.read(cx).panel);
        panel.set_active(true, window, cx);
    }
}

/// Brings forward `panel`'s tab in any pane of `workspace`, focused, or opens one in the active
/// pane: one tab per panel per group (D4).
fn show_tab(
    workspace: &mut Workspace,
    panel: Arc<dyn PanelHandle>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open = workspace
        .items_of_type::<PanelTab>(cx)
        .find(|tab| tab.read(cx).panel.panel_id() == panel.panel_id());
    if let Some(tab) = open {
        workspace.activate_item(&tab, true, true, window, cx);
        return;
    }
    let name = panel
        .icon_tooltip(window, cx)
        .unwrap_or_else(|| panel.persistent_name());
    let icon = panel.icon(window, cx);
    let focus = panel.panel_focus_handle(cx);
    let workspace_handle = workspace.weak_handle();
    let tab = cx.new(|_| PanelTab {
        panel,
        workspace: workspace_handle,
        focus,
        name: SharedString::new_static(name),
        icon,
        front: false,
    });
    workspace.add_item_to_active_pane(Box::new(tab), None, true, window, cx);
}

/// The split keys on a panel tab: its default mode would clone the tab, which a panel tab cannot
/// be, and do nothing, so it moves into the new pane as a drag does (D4).
fn move_split(
    workspace: &Workspace,
    direction: SplitDirection,
    mode: SplitMode,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if mode != SplitMode::ClonePane || workspace.active_item_as::<PanelTab>(cx).is_none() {
        return;
    }
    cx.stop_propagation();
    workspace.active_pane().update(cx, |pane, cx| {
        pane.split(direction, SplitMode::MovePane, window, cx);
    });
}

/// Closes every panel tab of every window whose panel `close` names: all of them when the Zed
/// layout is chosen (D10), a disabled panel's otherwise (#748).
fn close_tabs(cx: &mut App, close: impl Fn(&dyn PanelHandle, &App) -> bool) {
    for window in cx.windows() {
        let Some(window) = window.downcast::<MultiWorkspace>() else {
            continue;
        };
        window
            .update(cx, |multi_workspace, window, cx| {
                let workspaces: Vec<_> = multi_workspace.workspaces().cloned().collect();
                for workspace in workspaces {
                    let panes = workspace.read(cx).panes().to_vec();
                    for pane in panes {
                        let tabs: Vec<EntityId> = pane
                            .read(cx)
                            .items_of_type::<PanelTab>()
                            .filter(|tab| close(tab.read(cx).panel.as_ref(), cx))
                            .map(|tab| tab.entity_id())
                            .collect();
                        for tab in tabs {
                            pane.update(cx, |pane, cx| {
                                pane.close_item_by_id(tab, SaveIntent::Skip, window, cx)
                                    .detach_and_log_err(cx);
                            });
                        }
                    }
                }
            })
            .log_err();
    }
}

/// A center tab drawing a dock panel's own entity (#747): Files, Git or Outline in the Marley
/// layout. The panel stays its dock's, which never draws it.
pub struct PanelTab {
    panel: Arc<dyn PanelHandle>,
    /// The workspace whose dock holds the panel, where the tab's pane is found.
    workspace: WeakEntity<Workspace>,
    /// The panel's focus handle, kept: the panel reads the active item's focus handle while it is
    /// being updated (the outline panel does, #747's hunk), so answering must not read the panel.
    focus: FocusHandle,
    /// The panel's button tooltip, which names it on its tab.
    name: SharedString,
    icon: Option<IconName>,
    /// Whether the tab is its pane's front item, as the panel was last told (D5).
    front: bool,
}

impl std::fmt::Debug for PanelTab {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PanelTab")
            .field("panel", &self.panel.persistent_name())
            .finish_non_exhaustive()
    }
}

impl EventEmitter<ItemEvent> for PanelTab {}

impl Focusable for PanelTab {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for PanelTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // A pane draws only its front item, so being drawn is being in front. The panel hears it
        // after this frame: telling it updates the panel, which this frame is drawing.
        if !self.front {
            self.front = true;
            let panel = Arc::clone(&self.panel);
            cx.defer_in(window, move |_, window, cx| {
                panel.set_active(true, window, cx);
            });
        }
        let split = self
            .pane(cx)
            .and_then(|pane| pane.read(cx).drag_split_direction());
        div()
            .size_full()
            .relative()
            .bg(cx.theme().colors().panel_background)
            .child(self.panel.to_any())
            // A tab dragged over the panel: the panels block the mouse over their empty space
            // (`block_mouse_except_scroll`), which hides the pane's own drop target, so the tab
            // draws one over the panel and hands the drop to its pane (#747). The pane follows the
            // drag in the capture phase, so its split direction is right.
            .child(
                // Always there and transparent, since a hidden element takes no drop; a hitbox
                // that does not block leaves the panel under it working as before.
                div()
                    .absolute()
                    .drag_over::<DraggedTab>(|style, _, _, cx| {
                        style.bg(cx.theme().colors().drop_target_background)
                    })
                    .on_drop(cx.listener(|tab, dragged: &DraggedTab, window, cx| {
                        tab.drop_tab(dragged, window, cx);
                    }))
                    .map(|target| {
                        let half = DefiniteLength::Fraction(0.5);
                        match split {
                            None => target.top_0().right_0().bottom_0().left_0(),
                            Some(SplitDirection::Up) => target.top_0().left_0().right_0().h(half),
                            Some(SplitDirection::Down) => {
                                target.left_0().bottom_0().right_0().h(half)
                            }
                            Some(SplitDirection::Left) => {
                                target.top_0().left_0().bottom_0().w(half)
                            }
                            Some(SplitDirection::Right) => {
                                target.top_0().bottom_0().right_0().w(half)
                            }
                        }
                    }),
            )
    }
}

impl PanelTab {
    /// The pane holding this tab.
    fn pane(&self, cx: &Context<Self>) -> Option<Entity<Pane>> {
        let id = cx.entity_id();
        self.workspace
            .upgrade()?
            .read(cx)
            .panes()
            .iter()
            .find(|pane| pane.read(cx).items().any(|item| item.item_id() == id))
            .cloned()
    }

    /// A tab dropped over the panel, as the pane's own drop target takes it.
    fn drop_tab(&self, dragged: &DraggedTab, window: &Window, cx: &mut Context<Self>) {
        let Some(pane) = self.pane(cx) else {
            return;
        };
        let dragged = dragged.clone();
        // The drop moves tabs between panes, this one among them, so it runs after this update.
        window.defer(cx, move |window, cx| {
            pane.update(cx, |pane, cx| {
                let index = pane.active_item_index();
                pane.handle_tab_drop(&dragged, index, true, window, cx);
            });
        });
    }
}

impl Item for PanelTab {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        self.name.clone()
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        self.icon.map(Icon::new)
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn deactivated(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.front {
            self.front = false;
            self.panel.set_active(false, window, cx);
        }
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    /// A tab dropped into another group closes: the panel is its own workspace's (D4).
    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.panel.panel_id();
        let held = workspace
            .all_docks()
            .into_iter()
            .any(|dock| dock.read(cx).panel_for_id(id).is_some());
        if !held {
            cx.defer_in(window, |_, _, cx| cx.emit(ItemEvent::CloseItem));
        }
    }
}
