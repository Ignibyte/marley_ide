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
use std::time::Duration;

use anyhow::Context as _;
use collections::HashMap;
use gpui::{
    App, AppContext as _, Context, DefiniteLength, Entity, EntityId, EventEmitter, FocusHandle,
    Focusable, Global, Render, SharedString, Task, WeakEntity, Window,
};
use project::Project;
use settings::{Settings as _, SettingsStore};
use ui::prelude::*;
use util::ResultExt as _;
use workspace::dock::{Dock, PanelHandle};
use workspace::item::{Item, ItemEvent, SerializableItem};
use workspace::pane::{SplitDown, SplitLeft, SplitMode, SplitRight, SplitUp};
use workspace::{
    DraggedTab, ItemId, MultiWorkspace, Pane, SaveIntent, SplitDirection, Workspace, WorkspaceId,
};

use persistence::MarleyPanelTabsDb;

use crate::{MarleySettings, SidePanels};

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

/// Installs the dock observers and the split keys on every workspace, closes every panel tab when
/// the Zed layout is chosen, and lets a panel tab come back after a restart (#773). [`crate::init`]
/// calls it once.
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
    let mut side_panels = MarleySettings::get_global(cx).side_panels;
    cx.observe_global::<SettingsStore>(move |cx| {
        let now = crate::marley_layout(cx);
        let side_now = MarleySettings::get_global(cx).side_panels;
        // The Zed layout shows every panel in its dock (D10); a panel turned off (Knowledge while
        // Rusty is off, Containers while hidden) loses its tab, as Zed hides its button (#748).
        // The Inspector, once chosen, holds Files, Changes and Outline in place of their tabs (#837).
        let leaving = marley_layout && !now;
        let inspecting = side_now == SidePanels::Inspector && side_panels != side_now;
        close_tabs(cx, |panel, cx| {
            leaving
                || !panel.enabled(cx)
                || (inspecting && crate::inspector::inspects(panel.persistent_name()))
        });
        if leaving || side_now == SidePanels::Tabs {
            crate::inspector::close_all(cx);
        }
        marley_layout = now;
        side_panels = side_now;
    })
    .detach();
    load_saved_tabs(cx);
    workspace::register_serializable_item::<PanelTab>(cx);
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
        if crate::inspector::takes(panel.persistent_name(), cx) {
            crate::inspector::show(workspace, &panel, window, cx);
        } else {
            show_tab(workspace, Arc::clone(&panel), window, cx);
        }
    }
    // The dock's calls told this panel, and the one it showed before, that they are inactive: the
    // Outline tab emptied when the Files button moved the dock off the outline panel (#773).
    retell(workspace, last.0, window, cx);
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
        .find(|tab| tab.read(cx).panel.panel_id() == id && tab.read(cx).front)
        .map(|tab| Arc::clone(&tab.read(cx).panel));
    // The Inspector in front showing the panel counts as its tab (#837's D4).
    if let Some(panel) = tab.or_else(|| crate::inspector::front_panel(workspace, id, cx)) {
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
    let tab = PanelTab::new(panel, workspace.weak_handle(), window, cx);
    let tab = cx.new(|_| tab);
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
    close_items(cx, |pane, cx| {
        pane.items_of_type::<PanelTab>()
            .filter(|tab| close(tab.read(cx).panel.as_ref(), cx))
            .map(|tab| tab.entity_id())
            .collect()
    });
}

/// Closes, in every pane of every window, the items `pick` names.
pub(crate) fn close_items(cx: &mut App, pick: impl Fn(&Pane, &App) -> Vec<EntityId>) {
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
                        let items = pick(pane.read(cx), cx);
                        for item in items {
                            pane.update(cx, |pane, cx| {
                                pane.close_item_by_id(item, SaveIntent::Skip, window, cx)
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
            .child(drop_layer(split).on_drop(cx.listener(
                |tab, dragged: &DraggedTab, window, cx| {
                    let Some(pane) = tab.pane(cx) else {
                        return;
                    };
                    hand_drop(pane, dragged, window, cx);
                },
            )))
    }
}

/// A tab dragged over a panel: the panels block the mouse over their empty space
/// (`block_mouse_except_scroll`), which hides the pane's own drop target, so the item hosting the
/// panel draws one over it and hands the drop to its pane (#747). The pane follows the drag in the
/// capture phase, so its split direction is right. Always there and transparent, since a hidden
/// element takes no drop; a hitbox that does not block leaves the panel under it working as before.
pub(crate) fn drop_layer(split: Option<SplitDirection>) -> Div {
    div()
        .absolute()
        .drag_over::<DraggedTab>(|style, _, _, cx| {
            style.bg(cx.theme().colors().drop_target_background)
        })
        .map(|target| {
            let half = DefiniteLength::Fraction(0.5);
            match split {
                None => target.top_0().right_0().bottom_0().left_0(),
                Some(SplitDirection::Up) => target.top_0().left_0().right_0().h(half),
                Some(SplitDirection::Down) => target.left_0().bottom_0().right_0().h(half),
                Some(SplitDirection::Left) => target.top_0().left_0().bottom_0().w(half),
                Some(SplitDirection::Right) => target.top_0().bottom_0().right_0().w(half),
            }
        })
}

/// A tab dropped over a hosted panel, as `pane`'s own drop target takes it.
pub(crate) fn hand_drop(pane: Entity<Pane>, dragged: &DraggedTab, window: &Window, cx: &mut App) {
    let dragged = dragged.clone();
    // The drop moves tabs between panes, the host's among them, so it runs after this update.
    window.defer(cx, move |window, cx| {
        pane.update(cx, |pane, cx| {
            let index = pane.active_item_index();
            pane.handle_tab_drop(&dragged, index, true, window, cx);
        });
    });
}

/// The pane of `workspace` holding the item `id`.
pub(crate) fn pane_of(
    workspace: &WeakEntity<Workspace>,
    id: EntityId,
    cx: &App,
) -> Option<Entity<Pane>> {
    workspace
        .upgrade()?
        .read(cx)
        .panes()
        .iter()
        .find(|pane| pane.read(cx).items().any(|item| item.item_id() == id))
        .cloned()
}

impl PanelTab {
    /// The tab for `panel`, named and drawn as its status-bar button. A click and a restart build
    /// the same tab (#773's D5).
    fn new(
        panel: Arc<dyn PanelHandle>,
        workspace: WeakEntity<Workspace>,
        window: &Window,
        cx: &App,
    ) -> Self {
        let name = panel
            .icon_tooltip(window, cx)
            .unwrap_or_else(|| panel.persistent_name());
        let icon = panel.icon(window, cx);
        let focus = panel.panel_focus_handle(cx);
        Self {
            panel,
            workspace,
            focus,
            name: SharedString::new_static(name),
            icon,
            front: false,
        }
    }

    /// The pane holding this tab.
    fn pane(&self, cx: &Context<Self>) -> Option<Entity<Pane>> {
        pane_of(&self.workspace, cx.entity_id(), cx)
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

/// A panel tab is saved with its workspace (#773): its row names the panel, and the restore draws
/// the workspace's panel of that name, as a click on its button would.
impl SerializableItem for PanelTab {
    fn serialized_item_kind() -> &'static str {
        "MarleyPanelTab"
    }

    fn cleanup(
        workspace_id: WorkspaceId,
        alive_items: Vec<ItemId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<()>> {
        cx.default_global::<SavedPanelTabs>()
            .0
            .retain(|(workspace, item), _| {
                *workspace != workspace_id || alive_items.contains(item)
            });
        workspace::delete_unloaded_items(
            alive_items,
            workspace_id,
            "marley_panel_tabs",
            &MarleyPanelTabsDb::global(cx),
            cx,
        )
    }

    fn deserialize(
        _project: Entity<Project>,
        workspace: WeakEntity<Workspace>,
        workspace_id: WorkspaceId,
        item_id: ItemId,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        // Each refusal leaves no tab: Zed logs it, and the cleanup drops the row (D4). Zed's layout
        // has no panel tabs (#747's D10).
        if !crate::marley_layout(cx) {
            return Task::ready(Err(anyhow::anyhow!(
                "panel tabs are not restored in Zed's layout"
            )));
        }
        let Some(name) = saved_tab(workspace_id, item_id, cx) else {
            return Task::ready(Err(anyhow::anyhow!("no panel tab was saved for the item")));
        };
        if !is_moved(&name) {
            return Task::ready(Err(anyhow::anyhow!("{name} does not open as a tab")));
        }
        window.spawn(cx, async move |cx| {
            // Zed adds the workspace's panels while it still restores items
            // (L-claude-736-a-restored-item-waits-for-its-workspaces-panels-001).
            let mut panel = None;
            for _ in 0..PANEL_WAITS {
                panel = workspace
                    .read_with(cx, |workspace, cx| workspace_panel(workspace, &name, cx))?;
                if panel.is_some() {
                    break;
                }
                cx.background_executor().timer(PANEL_WAIT).await;
            }
            let panel =
                panel.with_context(|| format!("the workspace has no {name} for its tab"))?;
            cx.update(|window, cx| {
                anyhow::ensure!(panel.enabled(cx), "{name} is turned off");
                let shown = workspace
                    .upgrade()
                    .context("the workspace closed before its panel tab was restored")?
                    .read(cx)
                    .items_of_type::<Self>(cx)
                    .any(|tab| tab.read(cx).panel.panel_id() == panel.panel_id());
                anyhow::ensure!(!shown, "the group already shows {name} in a tab");
                let tab = Self::new(panel, workspace, window, cx);
                Ok(cx.new(|_| tab))
            })?
        })
    }

    fn serialize(
        &mut self,
        workspace: &mut Workspace,
        item_id: ItemId,
        _closing: bool,
        cx: &mut Context<Self>,
    ) -> Option<Task<anyhow::Result<()>>> {
        let workspace_id = workspace.database_id()?;
        Some(save_tab(
            workspace_id,
            item_id,
            self.panel.persistent_name(),
            cx,
        ))
    }

    /// Nothing about the tab changes: Zed writes its row once, when the tab joins a pane.
    fn should_serialize(&self, _event: &ItemEvent) -> bool {
        false
    }
}

/// How often, and how many times, a restored panel tab looks for its panel.
const PANEL_WAITS: usize = 100;
const PANEL_WAIT: Duration = Duration::from_millis(100);

/// The panel named `name` in any of `workspace`'s docks.
pub(crate) fn workspace_panel(
    workspace: &Workspace,
    name: &str,
    cx: &App,
) -> Option<Arc<dyn PanelHandle>> {
    workspace
        .all_docks()
        .into_iter()
        .find_map(|dock| dock.read(cx).panel_for_persistent_name(name).cloned())
}

/// The saved tabs' panel names, read once at start, so `deserialize` finds one at once and a
/// cleanup between two restores cannot take it
/// (PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001).
#[derive(Default)]
struct SavedPanelTabs(HashMap<(WorkspaceId, ItemId), String>);

impl Global for SavedPanelTabs {}

fn load_saved_tabs(cx: &mut App) {
    let saved = MarleyPanelTabsDb::global(cx)
        .all_tabs()
        .log_err()
        .unwrap_or_default()
        .into_iter()
        .map(|(workspace_id, item_id, panel)| ((workspace_id, item_id), panel))
        .collect();
    cx.set_global(SavedPanelTabs(saved));
}

fn saved_tab(workspace_id: WorkspaceId, item_id: ItemId, cx: &App) -> Option<String> {
    cx.try_global::<SavedPanelTabs>()?
        .0
        .get(&(workspace_id, item_id))
        .cloned()
}

/// Keeps the tab's panel for the item, in memory and in the table.
fn save_tab(
    workspace_id: WorkspaceId,
    item_id: ItemId,
    panel: &str,
    cx: &mut App,
) -> Task<anyhow::Result<()>> {
    let panel = panel.to_string();
    let _previous = cx
        .default_global::<SavedPanelTabs>()
        .0
        .insert((workspace_id, item_id), panel.clone());
    let db = MarleyPanelTabsDb::global(cx);
    cx.background_spawn(async move { db.save_tab(item_id, workspace_id, panel).await })
}

mod persistence {
    use db::query;
    use db::sqlez::domain::Domain;
    use db::sqlez::thread_safe_connection::ThreadSafeConnection;
    use db::sqlez_macros::sql;
    use workspace::{ItemId, WorkspaceDb, WorkspaceId};

    /// The panel tabs' own table (#773): each tab's panel by its persistent name. Keyed by
    /// workspace and item, with no `UNIQUE(item_id)`: ids repeat across launches (#576).
    pub(super) struct MarleyPanelTabsDb(ThreadSafeConnection);

    impl Domain for MarleyPanelTabsDb {
        const NAME: &str = stringify!(MarleyPanelTabsDb);

        const MIGRATIONS: &[&str] = &[sql!(
            CREATE TABLE marley_panel_tabs (
                workspace_id INTEGER,
                item_id INTEGER,
                panel TEXT NOT NULL,

                PRIMARY KEY(workspace_id, item_id),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                ON DELETE CASCADE
            ) STRICT;
        )];
    }

    db::static_connection!(MarleyPanelTabsDb, [WorkspaceDb]);

    impl MarleyPanelTabsDb {
        query! {
            pub(super) async fn save_tab(
                item_id: ItemId,
                workspace_id: WorkspaceId,
                panel: String
            ) -> Result<()> {
                INSERT OR REPLACE INTO marley_panel_tabs(item_id, workspace_id, panel)
                VALUES (?, ?, ?)
            }
        }

        query! {
            pub(super) fn all_tabs() -> Result<Vec<(WorkspaceId, ItemId, String)>> {
                SELECT workspace_id, item_id, panel
                FROM marley_panel_tabs
            }
        }
    }
}
