//! Rusty's vault as a graph in a center tab (#647).
//!
//! One Graph tab per workspace, scoped to the whole vault or to the neighbourhood of the page
//! last in front. Pages are dots sized by their links and coloured by page type; links are lines,
//! and a decision's typed edges are dashed, a colour each. A panel filters by query and type and
//! turns tags, unresolved links, decision edges and orphans on or off.
//!
//! The layout (`marley_rusty::graph_layout`) runs on the background executor a batch at a time, so
//! the window never waits on it; the tab only draws and finds the node under the pointer. The
//! drawing of nodes, the light on a node's neighbours and the hit test follow Ely GPUI Components'
//! `NetworkGraph` (`src/charts/network.rs` at `2f8b2f6`, MIT, its notice on
//! `marley_rusty::graph_layout`), drawn in one canvas with Zed's theme.
//!
//! Since #657 the panel has Rusty's Groups, Display and Forces (Obsidian's): colour groups by
//! query, arrows, the label fade, node size, link thickness and the four forces, on sliders ported
//! from Ely (`rusty::slider`). They are one record for every window (`rusty::graph_store`), and
//! the tab is saved with its workspace and restored at the next launch while Rusty is on.

use std::cell::Cell;
use std::collections::{BTreeSet, HashMap};
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use editor::{Editor, EditorEvent};
use gpui::{
    AnyElement, App, Bounds, ContentMask, Context, ElementId, Entity, EventEmitter, FocusHandle,
    Focusable, Font, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathBuilder,
    PinchEvent, Pixels, Point, ScrollDelta, ScrollWheelEvent, SharedString, Subscription, Task,
    TextAlign, TextRun, WeakEntity, Window, actions, canvas, fill, point, px, size,
};
use marley_rusty::graph::{
    BRAIN_PAGE_TYPES, CAP, EdgeKind, Filters, Graph, NodeKind, Query, Shown, ShownNode,
    page_types_from_answer, shown, type_order,
};
use marley_rusty::graph_layout::{Layout, Viewport, arrowhead, float, nearest};
use marley_rusty::graph_settings::{
    CENTER_FORCE, DEPTHS, Forces, GraphSettings, Group, GroupColor, GroupColoring, LINK_DISTANCE,
    LINK_FORCE, LINK_THICKNESS, NODE_SIZE, REPEL_FORCE, SliderRange, Switch, TEXT_FADE,
    group_colors,
};
use marley_rusty::knowledge::BRAIN_GRAPH;
use project::Project;
use serde_json::json;
use ui::{Checkbox, Disclosure, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent, SerializableItem};
use workspace::notifications::NotificationId;
use workspace::{ItemId, Toast, Workspace, WorkspaceId};

use super::graph_store::{self, SavedGraphTab, Sections};
use super::page::{PageEvent, PageView};
use super::slider::Slider;

actions!(
    rusty,
    [
        /// Opens Rusty's vault as a graph in a tab, or brings the Graph tab forward.
        #[derive(Eq)]
        OpenGraph,
        /// Opens the Graph tab on the neighbourhood of the brain page in front.
        #[derive(Eq)]
        OpenLocalGraph,
    ]
);

/// Pair checks one layout batch spends on the background executor: about 15 ms in an optimized
/// build, eight steps of the cap's 2,000 nodes. Each batch goes back to the window's thread to be
/// drawn and waits there for the frame under way, so a run of one-step batches at the cap took
/// four seconds in a debug build; fewer, larger ones settle sooner.
const BATCH_PAIRS: usize = 16_000_000;

/// View pixels a press may move and still be a click (Rusty's).
const CLICK_SLOP: f32 = 3.0;

/// View pixels around a node that still hit it (Rusty's).
const HIT_SLACK: f32 = 6.0;

/// The most labels a frame paints besides the hovered node's, by link count.
const LABELS: usize = 200;

/// The zoom one wheel line gives: gpui reports a notch as three lines, so a notch zooms by about
/// 1.16, as Rusty's does.
const WHEEL_STEP: f32 = 1.05;

/// The view's margin on a fit, in view pixels.
const FIT_MARGIN: f32 = 60.0;

/// The work one path takes before it is drawn and the next begun: gpui tessellates a path into
/// `u16` indices, four vertices a segment or a dash.
const PATH_UNITS: f32 = 8_000.0;

/// The dashes of a decision's typed edges, in view pixels.
const DASH: f32 = 5.0;
const GAP: f32 = 4.0;

/// Reads what the Graph tab keeps, makes it an item Zed saves with its workspace, and registers
/// the two actions on every workspace; `rusty::init` calls it once, whatever the switch says, so
/// Zed knows the item to refuse it while Rusty is off.
pub(super) fn init(cx: &mut App) {
    graph_store::init(cx);
    workspace::register_serializable_item::<GraphView>(cx);
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &OpenGraph, window, cx| {
            open(workspace, false, window, cx);
        });
        workspace.register_action(|workspace, _: &OpenLocalGraph, window, cx| {
            open(workspace, true, window, cx);
        });
    })
    .detach();
}

/// Opens the Graph tab once the update in progress ends, from a view that may be inside one.
pub(super) fn open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    window.defer(cx, move |window, cx| {
        workspace
            .update(cx, |workspace, cx| open(workspace, false, window, cx))
            .log_err();
    });
}

/// Brings the Graph tab forward, or adds one to the active pane: Local when a brain page is in
/// front, else the vault. With `local`, the tab turns to the page in front, or the page it last
/// followed, and says to open a page first when there is none.
fn open(workspace: &mut Workspace, local: bool, window: &mut Window, cx: &mut Context<Workspace>) {
    if let Some(reason) = super::unavailable(cx) {
        toast(workspace, reason.to_string(), cx);
        return;
    }
    let front = workspace
        .active_item(cx)
        .and_then(|item| item.downcast::<PageView>())
        .map(|page| page.read(cx).slug().to_string());
    // With no page in front, the workspace's project page is the centre (#655).
    super::project::ensure(cx);
    let project_page = super::project::project_page(workspace, cx);
    let open = workspace.items_of_type::<GraphView>(cx).next();
    if let Some(view) = open {
        if local {
            let page = front.or_else(|| view.read(cx).page.clone());
            match page {
                Some(page) => view.update(cx, |view, cx| view.go_local(page, cx)),
                None if project_page.is_some() => view.update(cx, GraphView::go_project),
                None => {
                    toast(workspace, NO_CENTRE.to_string(), cx);
                    return;
                }
            }
        }
        workspace.activate_item(&view, true, true, window, cx);
        return;
    }
    if local && front.is_none() && project_page.is_none() {
        toast(workspace, NO_CENTRE.to_string(), cx);
        return;
    }
    let workspace_entity = cx.entity();
    let view =
        cx.new(|cx| GraphView::new(&workspace_entity, None, front, project_page, window, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}

/// What `rusty: open local graph` says with no centre to take.
const NO_CENTRE: &str =
    "Open a page first, or link this project to its page in the Knowledge panel.";

fn toast(workspace: &mut Workspace, message: String, cx: &mut Context<Workspace>) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<GraphView>(), message),
        cx,
    );
}

/// What the tab shows: one page's neighbourhood, or the vault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    Local,
    Vault,
}

/// What a read asks `brain_graph` for; the graph held is for one.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReadKey {
    around: Option<String>,
    depth: usize,
    unresolved: bool,
}

/// `brain_graph`'s reads: the one in flight, whether one more is due when it lands, and whether
/// a change came while the tab was hidden.
#[derive(Default)]
struct Reads {
    in_flight: Option<ReadKey>,
    again: bool,
    stale: bool,
    task: Option<Task<()>>,
}

/// A pointer gesture under way: the view panned from its last point, or a node held, `moved`
/// once the pointer leaves the click's slop.
#[derive(Clone, Copy, Debug)]
enum Gesture {
    Pan {
        last: Point<Pixels>,
    },
    Node {
        at: usize,
        start: Point<Pixels>,
        moved: bool,
    },
}

/// A pin, a release or new forces made while a batch is out, applied when it comes back.
#[derive(Clone, Copy, Debug)]
enum Change {
    Pin(usize, (f32, f32)),
    Release(usize),
    Forces(Forces),
}

/// The layout's run: the task driving its batches, the changes waiting for the next, and a
/// count that tells a batch from a replaced layout's.
#[derive(Default)]
struct Run {
    task: Option<Task<()>>,
    pending: Vec<Change>,
    generation: u64,
}

/// A layout run's time: since it began, its batches, and the time they spent working.
struct RunClock {
    started: Instant,
    batches: usize,
    work: Duration,
}

/// The tab's view: where it looks, whether a fit is due, and whether the panel shows.
struct Look {
    viewport: Viewport,
    fit_wanted: bool,
    fitted: bool,
    panel_open: bool,
}

/// The Graph tab.
pub(crate) struct GraphView {
    workspace: WeakEntity<Workspace>,
    /// The workspace's events, followed again when the tab moves to another (L-613).
    workspace_events: Subscription,
    focus_handle: FocusHandle,
    scope: Scope,
    depth: usize,
    /// The brain page last in front, the local graph's centre.
    page: Option<String>,
    /// The workspace's project page, the centre while no page has been in front (#655).
    project_page: Option<String>,
    page_tab: Option<(WeakEntity<PageView>, Subscription)>,
    filter_field: Entity<Editor>,
    filters: Filters,
    unresolved: bool,
    /// The graph read, the key it was read for, and Rusty's page types.
    graph: Option<(Arc<Graph>, ReadKey)>,
    page_types: Vec<String>,
    notice: Option<SharedString>,
    reads: Reads,
    shown: Shown,
    type_order: Vec<String>,
    /// Shown nodes by link count, most first, for the labels.
    by_links: Vec<usize>,
    layout: Option<Layout>,
    places: Vec<(f32, f32)>,
    /// Places by node id, kept across reads and filters.
    kept: HashMap<String, (f32, f32)>,
    run: Run,
    look: Look,
    /// The canvas's bounds as last painted, for the pointer and the fit.
    canvas: Rc<Cell<Bounds<Pixels>>>,
    hover: Option<usize>,
    gesture: Option<Gesture>,
    /// The graph settings as this tab last took them; the switches and the depth are copied into
    /// `filters`, `unresolved` and `depth`, which the reads and the filter use (#657).
    settings: GraphSettings,
    /// Which group colours each shown node.
    coloring: GroupColoring,
    /// Each group's query field, by the group's place.
    group_fields: Vec<(Entity<Editor>, Subscription)>,
    /// New group was pressed: its field takes the focus once it is made.
    focus_new_group: bool,
    sections: Sections,
    _subscriptions: Vec<Subscription>,
}

/// The tab's title and tooltip change with its scope and centre, and the tab's own state with
/// them, which Zed saves on this event.
pub(crate) enum GraphEvent {
    UpdateTab,
}

impl EventEmitter<GraphEvent> for GraphView {}

impl GraphView {
    /// The one constructor, behind `rusty: open graph` and the restore (PR restore-is-a-second-
    /// constructor): a restored tab takes `saved`, a new one its page and the project's.
    fn new(
        workspace: &Entity<Workspace>,
        saved: Option<SavedGraphTab>,
        page: Option<String>,
        project_page: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let restored = saved.is_some();
        let local = page.is_some() || project_page.is_some();
        let saved = saved.unwrap_or_else(|| SavedGraphTab {
            local,
            page,
            ..SavedGraphTab::default()
        });
        let filter_field = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Filter: words, tag:, path:, type:", window, cx);
            editor.set_text(saved.filter.clone(), window, cx);
            editor
        });
        let settings = graph_store::settings(cx).clone();
        let subscriptions = vec![
            cx.subscribe(&filter_field, |this, field, event: &EditorEvent, cx| {
                if matches!(event, EditorEvent::BufferEdited) {
                    let query = Query::parse(&field.read(cx).text(cx));
                    if query != this.filters.query {
                        this.filters.query = query;
                        cx.emit(GraphEvent::UpdateTab);
                        this.rebuild(cx);
                    }
                }
            }),
            cx.observe_global_in::<graph_store::GraphSettingsStore>(window, Self::settings_changed),
            cx.observe_global::<super::Announced>(|this, cx| {
                if this.showing(cx) {
                    this.read_if_needed(true, cx);
                } else {
                    this.reads.stale = true;
                }
            }),
            cx.observe_global::<super::Rusty>(|this, cx| {
                if !super::is_on(cx) {
                    this.drop_graph(cx);
                } else if super::is_connected(cx) {
                    this.read_if_needed(false, cx);
                }
                cx.notify();
            }),
            cx.observe_global::<super::project::ProjectPages>(Self::follow_project),
        ];
        let scope = if saved.local {
            Scope::Local
        } else {
            Scope::Vault
        };
        let filters = Filters {
            query: Query::parse(&saved.filter),
            hidden_types: saved.hidden_types.iter().cloned().collect(),
            tags: settings.is_on(Switch::Tags),
            decision_edges: settings.is_on(Switch::DecisionEdges),
            orphans: settings.is_on(Switch::Orphans),
        };
        let mut view = Self {
            workspace: workspace.downgrade(),
            workspace_events: Self::follow_workspace(workspace, window, cx),
            focus_handle: cx.focus_handle(),
            scope,
            depth: settings.depth,
            page: saved.page,
            project_page,
            page_tab: None,
            filter_field,
            filters,
            unresolved: settings.is_on(Switch::Unresolved),
            graph: None,
            page_types: Vec::new(),
            notice: None,
            reads: Reads::default(),
            shown: Shown::default(),
            type_order: Vec::new(),
            by_links: Vec::new(),
            layout: None,
            places: Vec::new(),
            kept: HashMap::new(),
            run: Run::default(),
            look: Look {
                viewport: Viewport::default(),
                fit_wanted: false,
                fitted: false,
                panel_open: saved.panel_open,
            },
            canvas: Rc::new(Cell::new(Bounds::default())),
            hover: None,
            gesture: None,
            settings,
            coloring: GroupColoring::default(),
            group_fields: Vec::new(),
            focus_new_group: false,
            sections: saved.sections,
            _subscriptions: subscriptions,
        };
        view.sync_group_fields(window, cx);
        if restored {
            // The project's page is found once the restore's update is over, since it reads the
            // workspace (#655).
            cx.defer_in(window, |this, _, cx| this.follow_project(cx));
        }
        view.read_if_needed(false, cx);
        view
    }

    /// Follows the workspace's active item, as the local graph's centre.
    fn follow_workspace(
        workspace: &Entity<Workspace>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe_in(
            workspace,
            window,
            |this, workspace, event: &workspace::Event, window, cx| {
                if matches!(event, workspace::Event::ActiveItemChanged) {
                    this.follow_active_item(workspace, window, cx);
                }
            },
        )
    }

    /// The graph settings changed, in this tab or another: only what changed is done again (D2).
    fn settings_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new = graph_store::settings(cx).clone();
        if new == self.settings {
            return;
        }
        let old = mem::replace(&mut self.settings, new);
        let new = &self.settings;
        let changed = |switch| old.is_on(switch) != new.is_on(switch);
        let switched = [Switch::Tags, Switch::DecisionEdges, Switch::Orphans]
            .into_iter()
            .any(changed);
        let reread = old.depth != new.depth || changed(Switch::Unresolved);
        let forces = (old.forces != new.forces).then_some(new.forces);
        let regrouped = old.groups != new.groups;
        self.filters.tags = new.is_on(Switch::Tags);
        self.filters.decision_edges = new.is_on(Switch::DecisionEdges);
        self.filters.orphans = new.is_on(Switch::Orphans);
        self.unresolved = new.is_on(Switch::Unresolved);
        self.depth = new.depth;
        if regrouped {
            self.sync_group_fields(window, cx);
            self.color_groups();
        }
        if reread {
            cx.emit(GraphEvent::UpdateTab);
            self.read_if_needed(false, cx);
        }
        if switched {
            self.rebuild(cx);
        } else if let Some(forces) = forces {
            self.change_layout(Change::Forces(forces), cx);
        }
        cx.notify();
    }

    /// A query field for each group: made again when a group comes or goes, and an unfocused
    /// field's text set when another tab changed its group's query. A text set to what the field
    /// holds is skipped, so no edit comes back from it.
    fn sync_group_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let groups = self.settings.groups.clone();
        if self.group_fields.len() != groups.len() {
            self.group_fields = groups
                .iter()
                .enumerate()
                .map(|(at, group)| Self::group_field(at, &group.query, window, cx))
                .collect();
            if mem::take(&mut self.focus_new_group)
                && let Some((field, _)) = self.group_fields.last()
            {
                window.focus(&field.focus_handle(cx), cx);
            }
            return;
        }
        for ((field, _), group) in self.group_fields.iter().zip(&groups) {
            let held = field.read(cx).text(cx);
            if held != group.query && !field.focus_handle(cx).is_focused(window) {
                field.update(cx, |field, cx| {
                    field.set_text(group.query.clone(), window, cx);
                });
            }
        }
    }

    fn group_field(
        at: usize,
        query: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (Entity<Editor>, Subscription) {
        let field = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("tag:x path:y type:z or text", window, cx);
            editor.set_text(query, window, cx);
            editor
        });
        let edits = cx.subscribe(&field, move |_, field, event: &EditorEvent, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                let query = field.read(cx).text(cx);
                graph_store::update(cx, |settings| {
                    if let Some(group) = settings.groups.get_mut(at) {
                        group.query = query;
                    }
                });
            }
        });
        (field, edits)
    }

    /// Which group colours each shown node, again.
    fn color_groups(&mut self) {
        self.coloring = match &self.graph {
            Some((graph, _)) => group_colors(graph, &self.shown, &self.settings.groups),
            None => GroupColoring::default(),
        };
    }

    /// The tab's own state, as it is saved.
    fn saved(&self, cx: &App) -> SavedGraphTab {
        SavedGraphTab {
            local: self.scope == Scope::Local,
            page: self.page.clone(),
            filter: self.filter_field.read(cx).text(cx),
            hidden_types: self.filters.hidden_types.iter().cloned().collect(),
            panel_open: self.look.panel_open,
            sections: self.sections,
        }
    }

    /// Whether this tab is its pane's active item.
    fn showing(&self, cx: &Context<Self>) -> bool {
        let Some(workspace) = self.workspace.upgrade() else {
            return false;
        };
        let id = cx.entity_id();
        workspace.read(cx).panes().iter().any(|pane| {
            pane.read(cx)
                .active_item()
                .is_some_and(|item| item.item_id() == id)
        })
    }

    /// The workspace's active item: this tab shown, or a Page tab whose page becomes the centre
    /// (followed as it navigates). Any other item leaves the centre as it was (D2).
    fn follow_active_item(
        &mut self,
        workspace: &Entity<Workspace>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let active = workspace.read(cx).active_item(cx);
        if active
            .as_ref()
            .is_some_and(|item| item.item_id() == cx.entity_id())
        {
            // The service connection hears of no change, so every showing reads.
            let service = matches!(
                cx.global::<super::Rusty>().source,
                super::Source::Service(_)
            );
            let force = mem::take(&mut self.reads.stale) || service;
            self.read_if_needed(force, cx);
            return;
        }
        let Some(page) = active.and_then(|item| item.downcast::<PageView>()) else {
            return;
        };
        let followed = self
            .page_tab
            .as_ref()
            .is_some_and(|(tab, _)| tab.entity_id() == page.entity_id());
        if !followed {
            let events = cx.subscribe_in(&page, window, |this, page, event: &PageEvent, _, cx| {
                if matches!(event, PageEvent::UpdateTab) {
                    let slug = page.read(cx).slug().to_string();
                    this.set_page(slug, cx);
                }
            });
            self.page_tab = Some((page.downgrade(), events));
        }
        let slug = page.read(cx).slug().to_string();
        self.set_page(slug, cx);
    }

    /// A new page in front: the local graph's centre, read when the tab next shows.
    fn set_page(&mut self, slug: String, cx: &mut Context<Self>) {
        if self.page.as_deref() == Some(slug.as_str()) {
            return;
        }
        self.page = Some(slug);
        // The page is the tab's saved state in either scope (#657).
        cx.emit(GraphEvent::UpdateTab);
        if self.scope == Scope::Local {
            self.kept.clear();
            self.look.fitted = false;
            cx.notify();
        }
    }

    /// The local graph's centre: the page last in front, else the project's page.
    fn centre(&self) -> Option<&str> {
        self.page.as_deref().or(self.project_page.as_deref())
    }

    /// Whether the centre is the project's page, which the header says.
    const fn project_centred(&self) -> bool {
        self.page.is_none() && self.project_page.is_some()
    }

    /// The workspace's project page again; a project centre follows it.
    fn follow_project(&mut self, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let page = super::project::project_page(workspace.read(cx), cx);
        if page == self.project_page {
            return;
        }
        self.project_page = page;
        if self.page.is_none() && self.scope == Scope::Local {
            self.kept.clear();
            self.look.fitted = false;
            cx.emit(GraphEvent::UpdateTab);
            self.read_if_needed(false, cx);
        }
        cx.notify();
    }

    /// `rusty: open local graph` with no page in front: the project page's neighbourhood.
    fn go_project(&mut self, cx: &mut Context<Self>) {
        if self.scope != Scope::Local {
            self.kept.clear();
            self.look.fitted = false;
        }
        self.scope = Scope::Local;
        cx.emit(GraphEvent::UpdateTab);
        self.read_if_needed(false, cx);
        cx.notify();
    }

    /// `rusty: open local graph`: the neighbourhood of `page`, read now.
    fn go_local(&mut self, page: String, cx: &mut Context<Self>) {
        if self.scope != Scope::Local || self.page.as_deref() != Some(page.as_str()) {
            // A new centre: laid out afresh and fitted once settled.
            self.kept.clear();
            self.look.fitted = false;
        }
        self.scope = Scope::Local;
        self.page = Some(page);
        cx.emit(GraphEvent::UpdateTab);
        self.read_if_needed(false, cx);
        cx.notify();
    }

    fn set_scope(&mut self, scope: Scope, cx: &mut Context<Self>) {
        if self.scope == scope {
            return;
        }
        self.scope = scope;
        self.kept.clear();
        self.look.fitted = false;
        cx.emit(GraphEvent::UpdateTab);
        if self.wanted().is_none() {
            // A local graph with no page yet: the vault's graph is not what it shows.
            self.graph = None;
            self.rebuild(cx);
        }
        self.read_if_needed(false, cx);
        cx.notify();
    }

    /// The depth is the graph settings' (#657): every Graph tab takes it.
    fn set_depth(depth: usize, cx: &mut App) {
        graph_store::update(cx, |settings| settings.depth = depth);
    }

    /// What a read now would ask for; none for a local graph with no page.
    fn wanted(&self) -> Option<ReadKey> {
        let around = match self.scope {
            Scope::Local => Some(self.centre()?.to_string()),
            Scope::Vault => None,
        };
        Some(ReadKey {
            depth: if around.is_some() { self.depth } else { 1 },
            around,
            unresolved: self.unresolved,
        })
    }

    /// Reads the graph when the one held is not the one wanted, or always with `force`; with a
    /// read in flight, one more runs when it lands.
    fn read_if_needed(&mut self, force: bool, cx: &Context<Self>) {
        if !super::is_on(cx) || !super::is_connected(cx) {
            return;
        }
        let Some(key) = self.wanted() else {
            return;
        };
        if let Some(flight) = &self.reads.in_flight {
            if *flight != key || force {
                self.reads.again = true;
            }
            return;
        }
        if !force && self.graph.as_ref().is_some_and(|(_, held)| *held == key) {
            return;
        }
        let mut arguments = json!({ "tags": false, "unresolved": key.unresolved });
        if let Some(around) = &key.around {
            arguments["around"] = json!(around);
            arguments["depth"] = json!(key.depth);
        }
        let graph = super::call_tool(BRAIN_GRAPH, arguments, cx);
        let types = self
            .page_types
            .is_empty()
            .then(|| super::call_tool(BRAIN_PAGE_TYPES, json!({}), cx));
        self.reads.in_flight = Some(key.clone());
        self.reads.task = Some(cx.spawn(async move |this, cx| {
            let answer = graph.await;
            let types = match types {
                Some(types) => Some(types.await),
                None => None,
            };
            let read = match answer {
                Ok(text) => {
                    cx.background_spawn(futures::future::lazy(move |_| {
                        Graph::from_answer(&text).map_err(|error| {
                            format!("{BRAIN_GRAPH}'s answer did not parse: {error}")
                        })
                    }))
                    .await
                }
                Err(error) => Err(error),
            };
            this.update(cx, |this, cx| this.take_read(key, read, types, cx))
                .log_err();
        }));
    }

    fn take_read(
        &mut self,
        key: ReadKey,
        read: Result<Graph, String>,
        types: Option<Result<String, String>>,
        cx: &mut Context<Self>,
    ) {
        self.reads.in_flight = None;
        self.reads.task = None;
        if let Some(types) = types {
            match types.and_then(|text| {
                page_types_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_PAGE_TYPES}'s answer did not parse: {error}"))
            }) {
                Ok(types) => self.page_types = types,
                Err(error) => log::warn!("graph: {error}"),
            }
        }
        match read {
            Ok(graph) if self.wanted().as_ref() == Some(&key) => {
                self.notice = None;
                self.graph = Some((Arc::new(graph), key));
                self.rebuild(cx);
            }
            Ok(_) => {}
            Err(error) => self.notice = Some(error.into()),
        }
        if mem::take(&mut self.reads.again) {
            self.read_if_needed(true, cx);
        }
        cx.notify();
    }

    /// Rusty off: the graph, its places and its work dropped, and no call made until it is on.
    fn drop_graph(&mut self, cx: &mut Context<Self>) {
        self.graph = None;
        self.notice = None;
        self.reads = Reads::default();
        self.shown = Shown::default();
        self.by_links.clear();
        self.layout = None;
        self.places.clear();
        self.kept.clear();
        self.run.task = None;
        self.run.pending.clear();
        self.hover = None;
        self.gesture = None;
        self.look.fitted = false;
        cx.notify();
    }

    /// Works out what shows from the graph held and the panel, and lays it out again from the
    /// places it had.
    fn rebuild(&mut self, cx: &mut Context<Self>) {
        for (node, place) in self.shown.nodes.iter().zip(&self.places) {
            let _previous = self.kept.insert(node.id.clone(), *place);
        }
        self.run.task = None;
        self.run.pending.clear();
        self.run.generation += 1;
        self.hover = None;
        self.gesture = None;
        let Some((graph, _)) = &self.graph else {
            self.shown = Shown::default();
            self.layout = None;
            self.places.clear();
            return;
        };
        let centre = match self.scope {
            Scope::Local => self.centre().map(str::to_string),
            Scope::Vault => None,
        };
        self.type_order = type_order(&self.page_types, graph);
        self.shown = shown(graph, &self.filters, centre.as_deref(), CAP);
        let seeds: Vec<Option<(f32, f32)>> = self
            .shown
            .nodes
            .iter()
            .map(|node| self.kept.get(&node.id).copied())
            .collect();
        let edges = self.shown.edges.iter().map(|(a, b, _)| (*a, *b)).collect();
        let layout = Layout::seeded(
            self.shown.nodes.len(),
            edges,
            &seeds,
            self.shown.centre,
            self.settings.forces,
        );
        self.places = layout.places().to_vec();
        self.layout = Some(layout);
        self.color_groups();
        let mut by_links: Vec<usize> = (0..self.shown.nodes.len()).collect();
        by_links.sort_by_key(|at| std::cmp::Reverse(self.shown.degree[*at]));
        self.by_links = by_links;
        self.start_run(cx);
        cx.notify();
    }

    /// Lends the layout to the background executor a batch at a time until it settles; the
    /// places drawn are each batch's.
    fn start_run(&mut self, cx: &Context<Self>) {
        if self.run.task.is_some() {
            return;
        }
        let Some(layout) = self.layout.take() else {
            return;
        };
        if layout.settled() {
            self.layout = Some(layout);
            return;
        }
        let generation = self.run.generation;
        let mut clock = RunClock {
            started: Instant::now(),
            batches: 0,
            work: Duration::ZERO,
        };
        self.run.task = Some(cx.spawn(async move |this, cx| {
            let mut layout = layout;
            loop {
                let (back, work) = cx
                    .background_spawn(futures::future::lazy(move |_| {
                        let begun = Instant::now();
                        let mut layout = layout;
                        let _steps = layout.run(BATCH_PAIRS);
                        (layout, begun.elapsed())
                    }))
                    .await;
                layout = back;
                clock.batches += 1;
                clock.work += work;
                let going = this.update(cx, |this, cx| {
                    this.take_batch(&mut layout, generation, &clock, cx)
                });
                if !matches!(going, Ok(true)) {
                    break;
                }
            }
        }));
    }

    /// A batch back: the changes made meanwhile applied, its places drawn; settled, the layout
    /// is put back and the first settle fits the view.
    fn take_batch(
        &mut self,
        layout: &mut Layout,
        generation: u64,
        clock: &RunClock,
        cx: &mut Context<Self>,
    ) -> bool {
        if generation != self.run.generation {
            return false;
        }
        for change in self.run.pending.drain(..) {
            apply(layout, change);
        }
        self.places = layout.places().to_vec();
        cx.notify();
        if !layout.settled() {
            return true;
        }
        log::info!(
            "graph layout: {} nodes, {} edges, {} steps in {} batches, {} ms of work in {} ms, \
             forces {}",
            self.shown.nodes.len(),
            self.shown.edges.len(),
            layout.steps(),
            clock.batches,
            clock.work.as_millis(),
            clock.started.elapsed().as_millis(),
            layout.forces().describe()
        );
        self.layout = Some(mem::take(layout));
        self.run.task = None;
        if !self.look.fitted {
            self.look.fitted = true;
            self.look.fit_wanted = true;
        }
        false
    }

    fn change_layout(&mut self, change: Change, cx: &Context<Self>) {
        if let Some(layout) = self.layout.as_mut() {
            apply(layout, change);
            self.places = layout.places().to_vec();
            self.start_run(cx);
        } else if self.run.task.is_some() {
            self.run.pending.push(change);
        }
    }

    fn restart_layout(&mut self, cx: &mut Context<Self>) {
        self.kept.clear();
        self.places.clear();
        self.shown = Shown::default();
        self.look.fitted = false;
        self.rebuild(cx);
    }

    fn view_size(&self) -> (f32, f32) {
        let bounds = self.canvas.get();
        (f32::from(bounds.size.width), f32::from(bounds.size.height))
    }

    /// A window position as a point of the canvas, in view pixels.
    fn in_view(&self, position: Point<Pixels>) -> (f32, f32) {
        let origin = self.canvas.get().origin;
        (
            f32::from(position.x - origin.x),
            f32::from(position.y - origin.y),
        )
    }

    /// The radius each shown node is drawn at, in view pixels.
    fn radii(&self) -> Vec<f32> {
        let zoom = self.look.viewport.zoom;
        self.shown
            .degree
            .iter()
            .map(|degree| radius(*degree, zoom, self.settings.display.node_size))
            .collect()
    }

    fn node_at(&self, position: Point<Pixels>) -> Option<usize> {
        let size = self.view_size();
        let viewport = self.look.viewport;
        let places: Vec<(f32, f32)> = self
            .places
            .iter()
            .map(|place| viewport.to_view(*place, size))
            .collect();
        nearest(&places, &self.radii(), self.in_view(position), HIT_SLACK)
    }

    fn mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        let start = event.position;
        self.gesture = Some(
            self.node_at(start)
                .map_or(Gesture::Pan { last: start }, |at| Gesture::Node {
                    at,
                    start,
                    moved: false,
                }),
        );
    }

    fn mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let position = event.position;
        let pressed = event.pressed_button == Some(MouseButton::Left);
        match self.gesture {
            Some(Gesture::Pan { last }) if pressed => {
                let delta = position - last;
                self.look.viewport = self
                    .look
                    .viewport
                    .panned((f32::from(delta.x), f32::from(delta.y)));
                self.gesture = Some(Gesture::Pan { last: position });
                cx.notify();
            }
            Some(Gesture::Node { at, start, moved }) if pressed => {
                let moved = moved || beyond_slop(start, position);
                self.gesture = Some(Gesture::Node { at, start, moved });
                if moved {
                    let place = self
                        .look
                        .viewport
                        .to_world(self.in_view(position), self.view_size());
                    if let Some(drawn) = self.places.get_mut(at) {
                        *drawn = place;
                    }
                    self.change_layout(Change::Pin(at, place), cx);
                    cx.notify();
                }
            }
            _ => {
                let hover = self.node_at(position);
                if hover != self.hover {
                    self.hover = hover;
                    cx.notify();
                }
            }
        }
    }

    fn mouse_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.gesture.take() {
            Some(Gesture::Node {
                at, moved: false, ..
            }) => self.click(at, window, cx),
            // The centre stays held where it was dropped.
            Some(Gesture::Node {
                at, moved: true, ..
            }) if self.shown.centre != Some(at) => {
                self.change_layout(Change::Release(at), cx);
            }
            _ => {}
        }
        cx.notify();
    }

    /// A click on a node: a page opens in a kept tab with the focus (#645's opener, deferred, so
    /// the workspace's walk over its tabs never meets this one mid-update); a tag becomes the
    /// filter; an unresolved target does nothing.
    fn click(&self, at: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(node) = self.shown.nodes.get(at) else {
            return;
        };
        match node.kind {
            NodeKind::Page => {
                super::page::open_later(
                    self.workspace.clone(),
                    node.id.clone(),
                    false,
                    true,
                    window,
                    cx,
                );
            }
            NodeKind::Tag => {
                let filter = node.id.clone();
                self.filter_field
                    .update(cx, |field, cx| field.set_text(filter, window, cx));
            }
            NodeKind::Unresolved | NodeKind::Other => {}
        }
    }

    fn zoom_by(&mut self, factor: f32, position: Point<Pixels>, cx: &mut Context<Self>) {
        self.look.viewport =
            self.look
                .viewport
                .zoomed(factor, self.in_view(position), self.view_size());
        cx.notify();
    }

    fn scroll(&mut self, event: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let lines = match event.delta {
            ScrollDelta::Lines(lines) => lines.y,
            ScrollDelta::Pixels(pixels) => f32::from(pixels.y) / 20.0,
        };
        self.zoom_by(WHEEL_STEP.powf(lines), event.position, cx);
    }

    fn pinch(&mut self, event: &PinchEvent, cx: &mut Context<Self>) {
        self.zoom_by(1.0 + event.delta, event.position, cx);
    }

    fn fit(&mut self) {
        let size = self.view_size();
        if size.0 < 1.0 || self.places.is_empty() {
            return;
        }
        let centre = self
            .shown
            .centre
            .and_then(|at| self.places.get(at).copied());
        self.look.viewport = Viewport::fitting(&self.places, size, FIT_MARGIN, centre);
        self.look.fit_wanted = false;
    }

    fn toggle_type(&mut self, page_type: &str, cx: &mut Context<Self>) {
        if !self.filters.hidden_types.remove(page_type) {
            let _added = self.filters.hidden_types.insert(page_type.to_string());
        }
        cx.emit(GraphEvent::UpdateTab);
        self.rebuild(cx);
    }

    /// The header's lines: what shows, how many, and the cap's words when it applies.
    fn header(&self) -> (String, Option<String>) {
        let count = format!("{} nodes", thousands(self.shown.nodes.len()));
        let mut first = match self.scope {
            Scope::Vault => format!("Graph · {count}"),
            Scope::Local => {
                let title = self
                    .shown
                    .centre
                    .and_then(|at| self.shown.nodes.get(at))
                    .map(|node| node.title.clone())
                    .or_else(|| self.centre().map(str::to_string))
                    .unwrap_or_default();
                let project = if self.project_centred() {
                    " (project)"
                } else {
                    ""
                };
                format!(
                    "Local graph · {title}{project} · depth {} · {count}",
                    self.depth
                )
            }
        };
        if self.run.task.is_some() {
            first.push_str(" · settling");
        }
        let capped = (self.shown.total > self.shown.nodes.len()).then(|| {
            format!(
                "{} of {} nodes, the most linked; filter or open a local graph",
                thousands(self.shown.nodes.len()),
                thousands(self.shown.total)
            )
        });
        (first, capped)
    }

    /// What shows in the canvas's place, if anything does.
    fn empty_state(&self, cx: &App) -> Option<SharedString> {
        if !super::is_on(cx) {
            return super::unavailable(cx);
        }
        if self.graph.is_none() {
            if self.scope == Scope::Local && self.centre().is_none() {
                return Some(SharedString::new_static(
                    "Open a page to see its local graph",
                ));
            }
            return super::unavailable(cx).or_else(|| {
                self.notice
                    .is_none()
                    .then(|| SharedString::new_static("Reading the graph…"))
            });
        }
        match self.scope {
            Scope::Vault if self.shown.nodes.is_empty() => {
                Some(SharedString::new_static("No pages yet"))
            }
            Scope::Local if self.shown.nodes.len() <= 1 && self.filters.query.is_empty() => {
                Some(SharedString::new_static("No links around this page yet"))
            }
            _ => None,
        }
    }

    /// Everything one frame draws, its colours resolved from the theme.
    /// A node's colour: its group's hue, else its page type's accent, else its kind's (#657).
    fn node_colour(&self, at: usize, node: &ShownNode, cx: &App) -> Hsla {
        let group = self
            .coloring
            .per_node
            .get(at)
            .copied()
            .flatten()
            .and_then(|group| self.settings.groups.get(group));
        let theme = cx.theme();
        match (group, node.kind) {
            (Some(group), _) => hue(group.color, cx),
            (None, NodeKind::Page) => {
                let place = self
                    .type_order
                    .iter()
                    .position(|page_type| *page_type == node.page_type)
                    .unwrap_or(0);
                theme
                    .accents()
                    .color_for_index(u32::try_from(place).unwrap_or(0))
            }
            (None, NodeKind::Tag) => theme.status().hint,
            (None, NodeKind::Unresolved | NodeKind::Other) => theme.colors().text_muted,
        }
    }

    fn scene(&self, window: &Window, cx: &App) -> Scene {
        let colors = cx.theme().colors();
        let zoom = self.look.viewport.zoom;
        let lit: Option<BTreeSet<usize>> = self.hover.map(|at| self.shown.neighbourhood(at));
        let is_lit = |at: usize| lit.as_ref().is_none_or(|lit| lit.contains(&at));
        let nodes = self
            .shown
            .nodes
            .iter()
            .enumerate()
            .map(|(at, node)| {
                let colour = self.node_colour(at, node, cx);
                let faint = if is_lit(at) { 1.0 } else { 0.25 };
                let centre = self.shown.centre == Some(at);
                NodeLook {
                    fill: if node.kind == NodeKind::Unresolved {
                        colors.editor_background
                    } else {
                        colour.opacity(faint)
                    },
                    edge: if centre {
                        colors.text_accent
                    } else if node.kind == NodeKind::Unresolved {
                        colour.opacity(faint)
                    } else {
                        colors.editor_background
                    },
                    edge_width: if centre || node.kind == NodeKind::Unresolved {
                        2.0
                    } else {
                        1.0
                    },
                    radius: radius(self.shown.degree[at], zoom, self.settings.display.node_size),
                }
            })
            .collect();
        let edges = self
            .shown
            .edges
            .iter()
            .map(|(from, to, kind)| {
                let emphasis = match self.hover {
                    None => Emphasis::Normal,
                    Some(at) if *from == at || *to == at => Emphasis::Lit,
                    Some(_) => Emphasis::Faint,
                };
                // A tag's edge says a page carries it and points nowhere (D8).
                let directed = [*from, *to].iter().all(|at| {
                    self.shown
                        .nodes
                        .get(*at)
                        .is_some_and(|node| node.kind != NodeKind::Tag)
                });
                (*from, *to, Line::of(kind), emphasis, directed)
            })
            .collect();
        let display = self.settings.display;
        let fade = display.label_alpha(zoom);
        let mut forced: BTreeSet<usize> = lit.clone().unwrap_or_default();
        forced.extend(self.shown.centre);
        Scene {
            viewport: self.look.viewport,
            places: self.places.clone(),
            nodes,
            edges,
            arrows: display.arrows,
            thickness: display.link_thickness,
            strokes: Strokes::of(cx),
            labels: Labels {
                titles: self
                    .shown
                    .nodes
                    .iter()
                    .map(|node| SharedString::from(node.title.clone()))
                    .collect(),
                by_links: self.by_links.clone(),
                forced,
                hover: self.hover,
                fade: if self.hover.is_some() {
                    fade * 0.15
                } else {
                    fade
                },
                colour: colors.text_muted,
                hover_colour: colors.text,
                font: window.text_style().font(),
            },
        }
    }

    fn render_canvas(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement {
        let scene = Rc::new(self.scene(window, cx));
        let measured = Rc::clone(&self.canvas);
        div()
            .id("rusty-graph-canvas")
            .absolute()
            .inset_0()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    this.mouse_down(event, window, cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                this.mouse_move(event, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, window, cx| this.mouse_up(window, cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, window, cx| this.mouse_up(window, cx)),
            )
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                this.scroll(event, cx);
            }))
            .on_pinch(cx.listener(|this, event: &PinchEvent, _, cx| this.pinch(event, cx)))
            .on_hover(cx.listener(|this, inside: &bool, _, cx| {
                if !*inside && this.hover.take().is_some() {
                    cx.notify();
                }
            }))
            .child(
                canvas(
                    move |bounds, _, _| measured.set(bounds),
                    move |bounds, (), window, cx| paint(&scene, bounds, window, cx),
                )
                .size_full(),
            )
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let (first, capped) = self.header();
        v_flex()
            .absolute()
            .top_2()
            .left_2()
            .gap_0p5()
            .px_1()
            .rounded_sm()
            .bg(cx.theme().colors().editor_background.opacity(0.85))
            .child(Label::new(first).size(LabelSize::Small).color(Color::Muted))
            .children(capped.map(|capped| {
                Label::new(capped)
                    .size(LabelSize::Small)
                    .color(Color::Warning)
            }))
            .children(self.notice.clone().map(|notice| {
                h_flex()
                    .gap_2()
                    .child(
                        Label::new(notice)
                            .size(LabelSize::Small)
                            .color(Color::Error),
                    )
                    .child(
                        Button::new("rusty-graph-read-again", "Read again")
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.notice = None;
                                this.read_if_needed(true, cx);
                                cx.notify();
                            })),
                    )
            }))
    }

    fn render_scope(&self, cx: &Context<Self>) -> impl IntoElement {
        let scope_button = |id: &'static str, label: &'static str, scope: Scope| {
            Button::new(id, label)
                .label_size(LabelSize::Small)
                .toggle_state(self.scope == scope)
                .on_click(cx.listener(move |this, _, _, cx| this.set_scope(scope, cx)))
        };
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .gap_1()
                    .child(scope_button("rusty-graph-local", "Local", Scope::Local))
                    .child(scope_button("rusty-graph-vault", "Vault", Scope::Vault)),
            )
            // Shown in Vault too, off, so the panel's rows keep their places across scopes.
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Label::new("Depth")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .children(DEPTHS.into_iter().map(|depth| {
                        Button::new(("rusty-graph-depth", depth), depth.to_string())
                            .label_size(LabelSize::Small)
                            .disabled(self.scope == Scope::Vault)
                            .toggle_state(self.scope == Scope::Local && self.depth == depth)
                            .on_click(move |_, _, cx| Self::set_depth(depth, cx))
                    })),
            )
    }

    fn render_switches(&self) -> impl IntoElement {
        let switch = |id: &'static str, label: &'static str, on: bool| {
            Checkbox::new(id, ToggleState::from(on))
                .label(label)
                .label_size(LabelSize::Small)
        };
        v_flex()
            .gap_0p5()
            // The switches are the graph settings' (#657): every Graph tab takes them.
            .child(
                switch("rusty-graph-tags", "Tags", self.filters.tags).on_click(
                    |_: &ToggleState, _, cx| {
                        graph_store::update(cx, |settings| settings.toggle(Switch::Tags));
                    },
                ),
            )
            .child(
                switch(
                    "rusty-graph-unresolved",
                    "Unresolved links",
                    self.unresolved,
                )
                .on_click(|_: &ToggleState, _, cx| {
                    graph_store::update(cx, |settings| settings.toggle(Switch::Unresolved));
                }),
            )
            .child(
                switch(
                    "rusty-graph-decisions",
                    "Decision edges",
                    self.filters.decision_edges,
                )
                .on_click(|_: &ToggleState, _, cx| {
                    graph_store::update(cx, |settings| {
                        settings.toggle(Switch::DecisionEdges);
                    });
                }),
            )
            .child(
                switch("rusty-graph-orphans", "Orphans", self.filters.orphans).on_click(
                    |_: &ToggleState, _, cx| {
                        graph_store::update(cx, |settings| settings.toggle(Switch::Orphans));
                    },
                ),
            )
    }

    /// The page types in the graph with their counts, each a toggle, then the edge kinds shown.
    fn render_legend(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let counts = self
            .graph
            .as_ref()
            .map(|(graph, _)| graph.type_counts())
            .unwrap_or_default();
        let types = self
            .type_order
            .iter()
            .enumerate()
            .filter_map(|(place, page_type)| {
                let count = *counts.get(page_type)?;
                let hidden = self.filters.hidden_types.contains(page_type);
                let colour = theme
                    .accents()
                    .color_for_index(u32::try_from(place).unwrap_or(0));
                let toggled = page_type.clone();
                Some(
                    h_flex()
                        .id(("rusty-graph-type", place))
                        .gap_2()
                        .cursor_pointer()
                        .when(hidden, |entry| entry.opacity(0.4))
                        .child(div().size_2().rounded_full().bg(colour))
                        .child(Label::new(page_type.clone()).size(LabelSize::Small))
                        .child(
                            Label::new(count.to_string())
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.toggle_type(&toggled, cx);
                        })),
                )
            });
        let kinds: BTreeSet<Line> = self
            .shown
            .edges
            .iter()
            .map(|(_, _, kind)| Line::of(kind))
            .collect();
        let strokes = Strokes::of(cx);
        v_flex()
            .gap_0p5()
            .children(types)
            .children(kinds.into_iter().map(|line| {
                let (colour, dashed) = strokes.of_line(line);
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .w(px(16.))
                            .h_0()
                            .border_t_2()
                            .border_color(colour)
                            .when(dashed, Styled::border_dashed),
                    )
                    .child(
                        Label::new(line.name())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
            }))
    }

    fn render_panel(&self, cx: &Context<Self>) -> AnyElement {
        if !self.look.panel_open {
            return div()
                .absolute()
                .top_2()
                .right_2()
                .child(
                    IconButton::new("rusty-graph-show-panel", IconName::Settings)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Show Panel"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.look.panel_open = true;
                            cx.emit(GraphEvent::UpdateTab);
                            cx.notify();
                        })),
                )
                .into_any_element();
        }
        let colors = cx.theme().colors();
        v_flex()
            .id("rusty-graph-panel")
            // The canvas under the panel takes no press, wheel or focus from it (#657): a press on
            // a slider keeps the slider's focus for its keys.
            .occlude()
            .absolute()
            .top_2()
            .right_2()
            .w(px(240.))
            .max_h_full()
            .overflow_y_scroll()
            .p_2()
            .gap_2()
            .rounded_md()
            .border_1()
            .border_color(colors.border)
            .bg(colors.elevated_surface_background)
            .child(Self::render_panel_header(cx))
            .child(self.render_scope(cx))
            .child(
                div()
                    .px_1()
                    .py_0p5()
                    .rounded_sm()
                    .border_1()
                    .border_color(colors.border)
                    .child(self.filter_field.clone()),
            )
            .child(self.render_switches())
            .child(Self::section_header(
                "rusty-graph-groups",
                "Groups",
                self.sections.groups,
                |sections| sections.groups = !sections.groups,
                cx,
            ))
            .when(self.sections.groups, |panel| {
                panel.child(self.render_groups(cx))
            })
            .child(Self::section_header(
                "rusty-graph-display",
                "Display",
                self.sections.display,
                |sections| sections.display = !sections.display,
                cx,
            ))
            .when(self.sections.display, |panel| {
                panel.child(self.render_display())
            })
            .child(Self::section_header(
                "rusty-graph-forces",
                "Forces",
                self.sections.forces,
                |sections| sections.forces = !sections.forces,
                cx,
            ))
            .when(self.sections.forces, |panel| {
                panel.child(self.render_forces())
            })
            .child(self.render_legend(cx))
            .into_any_element()
    }

    /// The panel's first row: its name, Restart Layout, Fit and Hide Panel.
    fn render_panel_header(cx: &Context<Self>) -> impl IntoElement {
        let button = |id: &'static str, icon: IconName, tip: &'static str| {
            IconButton::new(id, icon)
                .icon_size(IconSize::Small)
                .tooltip(Tooltip::text(tip))
        };
        h_flex()
            .justify_between()
            .child(
                Label::new("GRAPH")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .child(
                h_flex()
                    .child(
                        button("rusty-graph-restart", IconName::RotateCw, "Restart Layout")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.restart_layout(cx);
                            })),
                    )
                    .child(
                        button("rusty-graph-fit", IconName::Maximize, "Fit").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.look.fit_wanted = true;
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        button("rusty-graph-hide-panel", IconName::Close, "Hide Panel").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.look.panel_open = false;
                                cx.emit(GraphEvent::UpdateTab);
                                cx.notify();
                            }),
                        ),
                    ),
            )
    }

    /// A section's header row: a click opens or folds it, which the tab keeps (#657).
    fn section_header(
        id: &'static str,
        name: &'static str,
        open: bool,
        toggle: fn(&mut Sections),
        cx: &Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .id(id)
            .gap_1()
            .cursor_pointer()
            .child(Disclosure::new(
                ElementId::from((ElementId::from(id), "disclosure")),
                open,
            ))
            .child(Label::new(name).size(LabelSize::Small))
            .on_click(cx.listener(move |this, _, _, cx| {
                toggle(&mut this.sections);
                cx.emit(GraphEvent::UpdateTab);
                cx.notify();
            }))
    }

    /// Groups: a row per group (its swatch, its query, how many nodes it colours, remove), then New
    /// group (Rusty's, `GraphView.qml:503-537`).
    fn render_groups(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let rows = self
            .settings
            .groups
            .iter()
            .zip(&self.group_fields)
            .enumerate()
            .map(|(at, (group, (field, _)))| {
                let count = self.coloring.counts.get(at).copied().unwrap_or(0);
                h_flex()
                    .gap_1()
                    .child(
                        div()
                            .id(("rusty-graph-group-colour", at))
                            .flex_none()
                            .size_3()
                            .rounded_full()
                            .cursor_pointer()
                            .bg(hue(group.color, cx))
                            .tooltip(Tooltip::text("Next colour"))
                            .on_click(move |_, _, cx| {
                                graph_store::update(cx, |settings| {
                                    if let Some(group) = settings.groups.get_mut(at) {
                                        group.color = group.color.next();
                                    }
                                });
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .px_1()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .child(field.clone()),
                    )
                    .child(
                        Label::new(count.to_string())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        IconButton::new(("rusty-graph-group-remove", at), IconName::Close)
                            .icon_size(IconSize::XSmall)
                            .icon_color(Color::Muted)
                            .tooltip(Tooltip::text("Remove group"))
                            .on_click(move |_, _, cx| {
                                graph_store::update(cx, |settings| {
                                    if at < settings.groups.len() {
                                        let _removed = settings.groups.remove(at);
                                    }
                                });
                            }),
                    )
            });
        v_flex().gap_1().children(rows).child(
            Button::new("rusty-graph-new-group", "New group")
                .start_icon(Icon::new(IconName::Plus).size(IconSize::Small))
                .label_size(LabelSize::Small)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.focus_new_group = true;
                    graph_store::update(cx, |settings| {
                        let color = GroupColor::for_place(settings.groups.len());
                        settings.groups.push(Group {
                            query: String::new(),
                            color,
                        });
                    });
                })),
        )
    }

    /// Display: Arrows and three sliders (Rusty's, `GraphView.qml:538-548`).
    fn render_display(&self) -> impl IntoElement {
        let display = self.settings.display;
        v_flex()
            .gap_1()
            .child(
                Checkbox::new("rusty-graph-arrows", ToggleState::from(display.arrows))
                    .label("Arrows")
                    .label_size(LabelSize::Small)
                    .on_click(|_: &ToggleState, _, cx| {
                        graph_store::update(cx, |settings| {
                            settings.display.arrows = !settings.display.arrows;
                        });
                    }),
            )
            .child(slider_row(
                "rusty-graph-text-fade",
                "Text fade threshold",
                TEXT_FADE,
                display.text_fade,
                |settings, value| settings.display.text_fade = value,
            ))
            .child(slider_row(
                "rusty-graph-node-size",
                "Node size",
                NODE_SIZE,
                display.node_size,
                |settings, value| settings.display.node_size = value,
            ))
            .child(slider_row(
                "rusty-graph-link-thickness",
                "Link thickness",
                LINK_THICKNESS,
                display.link_thickness,
                |settings, value| settings.display.link_thickness = value,
            ))
    }

    /// Forces: four sliders (Rusty's, `GraphView.qml:549-559`).
    fn render_forces(&self) -> impl IntoElement {
        let forces = self.settings.forces;
        v_flex()
            .gap_1()
            .child(slider_row(
                "rusty-graph-center-force",
                "Center force",
                CENTER_FORCE,
                forces.center,
                |settings, value| settings.forces.center = value,
            ))
            .child(slider_row(
                "rusty-graph-repel-force",
                "Repel force",
                REPEL_FORCE,
                forces.repel,
                |settings, value| settings.forces.repel = value,
            ))
            .child(slider_row(
                "rusty-graph-link-force",
                "Link force",
                LINK_FORCE,
                forces.link,
                |settings, value| settings.forces.link = value,
            ))
            .child(slider_row(
                "rusty-graph-link-distance",
                "Link distance",
                LINK_DISTANCE,
                forces.distance,
                |settings, value| settings.forces.distance = value,
            ))
    }
}

/// A slider's row: its name and value over the track; a change goes to the graph settings.
fn slider_row(
    id: &'static str,
    name: &'static str,
    range: SliderRange,
    value: f32,
    set: fn(&mut GraphSettings, f32),
) -> impl IntoElement {
    v_flex()
        .child(
            h_flex()
                .justify_between()
                .child(Label::new(name).size(LabelSize::Small).color(Color::Muted))
                .child(
                    Label::new(format!("{value:.decimals$}", decimals = range.decimals()))
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
        .child(
            Slider::new(id, value)
                .range(range.min, range.max)
                .step(range.step)
                .label(name)
                .on_change(move |value, _, cx| {
                    graph_store::update(cx, |settings| set(settings, range.clamp(value)));
                }),
        )
}

/// A node's radius in view pixels: Rusty's growth with the square root of its links, times the
/// panel's node size, never smaller than a dot.
fn radius(degree: usize, zoom: f32, node_size: f32) -> f32 {
    (float(degree).sqrt().mul_add(1.6, 3.0) * zoom * node_size).max(2.5)
}

/// A pin, a release or new forces, on the layout.
fn apply(layout: &mut Layout, change: Change) {
    match change {
        Change::Pin(at, place) => layout.pin(at, place),
        Change::Release(at) => layout.release(at),
        Change::Forces(forces) => layout.set_forces(forces),
    }
}

/// A group's hue in the theme's terminal colours.
fn hue(color: GroupColor, cx: &App) -> Hsla {
    let colors = cx.theme().colors();
    match color {
        GroupColor::Red => colors.terminal_ansi_red,
        GroupColor::Green => colors.terminal_ansi_green,
        GroupColor::Yellow => colors.terminal_ansi_yellow,
        GroupColor::Magenta => colors.terminal_ansi_magenta,
        GroupColor::Cyan => colors.terminal_ansi_cyan,
        GroupColor::Blue => colors.terminal_ansi_blue,
    }
}

fn beyond_slop(start: Point<Pixels>, now: Point<Pixels>) -> bool {
    let delta = now - start;
    f32::from(delta.x).hypot(f32::from(delta.y)) > CLICK_SLOP
}

/// `1234567` as `1,234,567`.
fn thousands(count: usize) -> String {
    let digits = count.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// The kinds of line the tab draws, in the legend's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Line {
    Link,
    Consulted,
    Supersedes,
    FollowsUp,
}

impl Line {
    const fn of(kind: &EdgeKind) -> Self {
        match kind {
            EdgeKind::Consulted => Self::Consulted,
            EdgeKind::Supersedes => Self::Supersedes,
            EdgeKind::FollowsUp => Self::FollowsUp,
            EdgeKind::Link | EdgeKind::Other(_) => Self::Link,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Link => "Links",
            Self::Consulted => "Consulted",
            Self::Supersedes => "Supersedes",
            Self::FollowsUp => "Follows up",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Link => 0,
            Self::Consulted => 1,
            Self::Supersedes => 2,
            Self::FollowsUp => 3,
        }
    }
}

/// How an edge stands against the hovered node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Emphasis {
    Normal,
    Lit,
    Faint,
}

impl Emphasis {
    const fn index(self) -> usize {
        match self {
            Self::Normal => 0,
            Self::Lit => 1,
            Self::Faint => 2,
        }
    }
}

/// The lines' colours: links in the border colour, lit in the accent text colour; a decision's
/// typed edges in three status colours, outside the accents the page types take.
#[derive(Clone, Copy, Debug)]
struct Strokes {
    link: Hsla,
    lit: Hsla,
    consulted: Hsla,
    supersedes: Hsla,
    follows_up: Hsla,
}

impl Strokes {
    fn of(cx: &App) -> Self {
        let theme = cx.theme();
        Self {
            link: theme.colors().border,
            lit: theme.colors().text_accent,
            consulted: theme.status().info,
            supersedes: theme.status().warning,
            follows_up: theme.status().success,
        }
    }

    /// A line's colour and whether it is dashed.
    const fn of_line(&self, line: Line) -> (Hsla, bool) {
        match line {
            Line::Link => (self.link, false),
            Line::Consulted => (self.consulted, true),
            Line::Supersedes => (self.supersedes, true),
            Line::FollowsUp => (self.follows_up, true),
        }
    }
}

/// A node's look in one frame.
#[derive(Clone, Copy, Debug)]
struct NodeLook {
    fill: Hsla,
    edge: Hsla,
    edge_width: f32,
    radius: f32,
}

/// The labels one frame may paint: by link count, those `forced` always, faded by the zoom.
struct Labels {
    titles: Vec<SharedString>,
    by_links: Vec<usize>,
    forced: BTreeSet<usize>,
    hover: Option<usize>,
    fade: f32,
    colour: Hsla,
    hover_colour: Hsla,
    font: Font,
}

/// Everything one frame of the canvas draws.
struct Scene {
    viewport: Viewport,
    places: Vec<(f32, f32)>,
    nodes: Vec<NodeLook>,
    /// Each edge's ends, its line, its emphasis and whether it has a direction.
    edges: Vec<(usize, usize, Line, Emphasis, bool)>,
    /// Whether heads are drawn at the edges' targets.
    arrows: bool,
    /// Every edge's width times this.
    thickness: f32,
    strokes: Strokes,
    labels: Labels,
}

/// Paints the edges batched into a few paths a style, then the nodes as round quads (Ely's
/// `ring`), then the labels, skipping what lies outside the canvas.
fn paint(scene: &Scene, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let size = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let origin = (f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let view: Vec<(f32, f32)> = scene
        .places
        .iter()
        .map(|place| scene.viewport.to_view(*place, size))
        .collect();
    window.with_content_mask(Some(ContentMask { bounds }), |window| {
        paint_edges(scene, &view, size, origin, window);
        if scene.arrows {
            paint_heads(scene, &view, size, origin, window);
        }
        for (at, look) in scene.nodes.iter().enumerate() {
            let Some((x, y)) = view.get(at).copied() else {
                continue;
            };
            let reach = look.radius + look.edge_width;
            if x < -reach || y < -reach || x > size.0 + reach || y > size.1 + reach {
                continue;
            }
            let radius = px(look.radius);
            window.paint_quad(
                fill(
                    Bounds::new(
                        point(px(origin.0 + x), px(origin.1 + y)) - point(radius, radius),
                        square(radius),
                    ),
                    look.fill,
                )
                .corner_radii(radius)
                .border_widths(px(look.edge_width))
                .border_color(look.edge),
            );
        }
        paint_labels(scene, &view, size, origin, window, cx);
    });
}

fn square(radius: Pixels) -> gpui::Size<Pixels> {
    size(radius * 2.0, radius * 2.0)
}

fn paint_edges(
    scene: &Scene,
    view: &[(f32, f32)],
    size: (f32, f32),
    origin: (f32, f32),
    window: &mut Window,
) {
    // One path per line kind and emphasis, drawn and begun again before gpui's index limit.
    let mut batches: Vec<Batch> = Vec::new();
    let mut slots: [Option<usize>; 12] = [None; 12];
    for (from, to, line, emphasis, _) in &scene.edges {
        let (Some(a), Some(b)) = (view.get(*from), view.get(*to)) else {
            continue;
        };
        let Some((a, b)) = clipped(*a, *b, size) else {
            continue;
        };
        let (colour, dashed) = scene.strokes.of_line(*line);
        let colour = edge_colour(colour, scene.strokes.lit, *line, *emphasis);
        let units = if dashed {
            ((a.0 - b.0).hypot(a.1 - b.1) / (DASH + GAP))
                .ceil()
                .max(1.0)
        } else {
            1.0
        };
        let at = *slots[line.index() * 3 + emphasis.index()].get_or_insert_with(|| {
            batches.push(Batch {
                path: builder(dashed, *emphasis, scene.thickness),
                units: 0.0,
                dashed,
                emphasis: *emphasis,
                colour,
            });
            batches.len() - 1
        });
        let batch = &mut batches[at];
        batch
            .path
            .move_to(point(px(origin.0 + a.0), px(origin.1 + a.1)));
        batch
            .path
            .line_to(point(px(origin.0 + b.0), px(origin.1 + b.1)));
        batch.units += units;
        if batch.units >= PATH_UNITS {
            let full = mem::replace(
                &mut batch.path,
                builder(batch.dashed, batch.emphasis, scene.thickness),
            );
            batch.units = 0.0;
            draw(full, batch.colour, window);
        }
    }
    for batch in batches {
        draw(batch.path, batch.colour, window);
    }
}

/// The heads of the edges that have a direction, filled in their edges' colours, one path a
/// colour and drawn again before gpui's index limit; a head whose target is out of view is
/// skipped (D8).
fn paint_heads(
    scene: &Scene,
    view: &[(f32, f32)],
    size: (f32, f32),
    origin: (f32, f32),
    window: &mut Window,
) {
    let mut heads: Vec<(PathBuilder, f32, Hsla)> = Vec::new();
    let mut slots: [Option<usize>; 12] = [None; 12];
    for (from, to, line, emphasis, directed) in &scene.edges {
        if !directed {
            continue;
        }
        let (Some(a), Some(b), Some(target)) =
            (view.get(*from), view.get(*to), scene.nodes.get(*to))
        else {
            continue;
        };
        if b.0 < 0.0 || b.1 < 0.0 || b.0 > size.0 || b.1 > size.1 {
            continue;
        }
        let Some(points) = arrowhead(*a, *b, target.radius + target.edge_width) else {
            continue;
        };
        let (colour, _) = scene.strokes.of_line(*line);
        let colour = edge_colour(colour, scene.strokes.lit, *line, *emphasis);
        let at = *slots[line.index() * 3 + emphasis.index()].get_or_insert_with(|| {
            heads.push((PathBuilder::fill(), 0.0, colour));
            heads.len() - 1
        });
        let (path, units, colour) = &mut heads[at];
        let corners = points.map(|(x, y)| point(px(origin.0 + x), px(origin.1 + y)));
        path.add_polygon(&corners, true);
        *units += 1.0;
        if *units >= PATH_UNITS {
            let full = mem::replace(path, PathBuilder::fill());
            *units = 0.0;
            draw(full, *colour, window);
        }
    }
    for (path, _, colour) in heads {
        draw(path, colour, window);
    }
}

/// One path of edges being built, and what it has taken so far.
struct Batch {
    path: PathBuilder,
    units: f32,
    dashed: bool,
    emphasis: Emphasis,
    colour: Hsla,
}

fn builder(dashed: bool, emphasis: Emphasis, thickness: f32) -> PathBuilder {
    let width = if dashed { 1.5 } else { 1.0 } + if emphasis == Emphasis::Lit { 0.5 } else { 0.0 };
    let path = PathBuilder::stroke(px(width * thickness));
    if dashed {
        path.dash_array(&[px(DASH), px(GAP)])
    } else {
        path
    }
}

fn edge_colour(colour: Hsla, lit: Hsla, line: Line, emphasis: Emphasis) -> Hsla {
    match (emphasis, line) {
        (Emphasis::Lit, Line::Link) => lit,
        (Emphasis::Faint, _) => colour.opacity(0.25),
        _ => colour,
    }
}

fn draw(path: PathBuilder, colour: Hsla, window: &mut Window) {
    match path.build() {
        Ok(path) => window.paint_path(path, colour),
        Err(error) => log::error!("graph: a path failed to build: {error:#}"),
    }
}

/// The part of the segment from `a` to `b` inside a view of `size`, a little beyond its edges
/// (Liang and Barsky's clip), so no line is longer than the view.
fn clipped(a: (f32, f32), b: (f32, f32), size: (f32, f32)) -> Option<((f32, f32), (f32, f32))> {
    let margin = 10.0;
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let mut low: f32 = 0.0;
    let mut high: f32 = 1.0;
    for (step, room) in [
        (-dx, a.0 + margin),
        (dx, size.0 + margin - a.0),
        (-dy, a.1 + margin),
        (dy, size.1 + margin - a.1),
    ] {
        if step.abs() < f32::EPSILON {
            if room < 0.0 {
                return None;
            }
            continue;
        }
        let ratio = room / step;
        if step < 0.0 {
            low = low.max(ratio);
        } else {
            high = high.min(ratio);
        }
        if low > high {
            return None;
        }
    }
    Some((
        (dx.mul_add(low, a.0), dy.mul_add(low, a.1)),
        (dx.mul_add(high, a.0), dy.mul_add(high, a.1)),
    ))
}

fn paint_labels(
    scene: &Scene,
    view: &[(f32, f32)],
    size: (f32, f32),
    origin: (f32, f32),
    window: &mut Window,
    cx: &mut App,
) {
    let labels = &scene.labels;
    let inside = |at: usize| {
        view.get(at)
            .is_some_and(|(x, y)| *x >= 0.0 && *y >= 0.0 && *x <= size.0 && *y <= size.1)
    };
    let mut chosen: Vec<usize> = labels
        .forced
        .iter()
        .copied()
        .filter(|at| inside(*at))
        .collect();
    if labels.fade > 0.02 {
        chosen.extend(
            labels
                .by_links
                .iter()
                .copied()
                .filter(|at| !labels.forced.contains(at) && inside(*at))
                .take(LABELS),
        );
    }
    let font_size = px(11.);
    let line_height = px(14.);
    for at in chosen {
        let (Some(title), Some((x, y)), Some(look)) = (
            labels.titles.get(at),
            view.get(at).copied(),
            scene.nodes.get(at),
        ) else {
            continue;
        };
        let colour = if labels.hover == Some(at) {
            labels.hover_colour
        } else if labels.forced.contains(&at) {
            labels.colour
        } else {
            labels.colour.opacity(labels.fade)
        };
        let run = TextRun {
            len: title.len(),
            font: labels.font.clone(),
            color: colour,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window
            .text_system()
            .shape_line(title.clone(), font_size, &[run], None);
        let place = point(
            px(origin.0 + x + look.radius + 3.0),
            px(origin.1 + y) - line_height / 2.0,
        );
        line.paint(place, line_height, TextAlign::Left, None, window, cx)
            .log_err();
    }
}

impl Focusable for GraphView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for GraphView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.look.fit_wanted {
            self.fit();
        }
        let empty = self.empty_state(cx);
        div()
            .id("rusty-graph")
            .key_context("RustyGraph")
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(cx.theme().colors().editor_background)
            .when(empty.is_none(), |root| {
                root.child(self.render_canvas(window, cx))
            })
            .when_some(empty, |root, empty| {
                root.child(
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(Label::new(empty).color(Color::Muted)),
                )
            })
            .when(super::is_on(cx), |root| {
                root.child(self.render_header(cx))
                    .child(self.render_panel(cx))
            })
    }
}

impl Item for GraphView {
    type Event = GraphEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        match self.scope {
            Scope::Vault => SharedString::new_static("Graph"),
            Scope::Local => SharedString::new_static("Local graph"),
        }
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::GitGraph))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(match (self.scope, self.centre()) {
            (Scope::Local, Some(page)) => {
                format!("Local graph of {page}, depth {}", self.depth).into()
            }
            _ => SharedString::new_static("Rusty's vault as a graph"),
        })
    }

    fn to_item_events(event: &GraphEvent, f: &mut dyn FnMut(ItemEvent)) {
        match event {
            GraphEvent::UpdateTab => f(ItemEvent::UpdateTab),
        }
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // L-613: a tab moved to another workspace follows that workspace's items.
        self.workspace = workspace.weak_handle();
        if let Some(workspace) = self.workspace.upgrade() {
            self.workspace_events = Self::follow_workspace(&workspace, window, cx);
        }
    }
}

/// A Graph tab is saved with its workspace (#657). The workspace's layout holds only the item;
/// the tab's own state goes in its row, and the graph settings are every tab's.
impl SerializableItem for GraphView {
    fn serialized_item_kind() -> &'static str {
        "MarleyRustyGraph"
    }

    fn cleanup(
        workspace_id: WorkspaceId,
        alive_items: Vec<ItemId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<()>> {
        graph_store::cleanup(workspace_id, alive_items, cx)
    }

    fn deserialize(
        _project: Entity<Project>,
        workspace: WeakEntity<Workspace>,
        workspace_id: WorkspaceId,
        item_id: ItemId,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        // Rusty off restores nothing; Zed logs the refusal and the cleanup drops the row (D4).
        if !super::is_on(cx) {
            return Task::ready(Err(anyhow::anyhow!(
                "Rusty is off; the Graph tab is not restored"
            )));
        }
        let Some(saved) = graph_store::saved_tab(workspace_id, item_id, cx) else {
            return Task::ready(Err(anyhow::anyhow!("no Graph tab was saved for the item")));
        };
        window.spawn(cx, async move |cx| {
            cx.update(|window, cx| {
                let workspace = workspace
                    .upgrade()
                    .context("the workspace closed before its Graph tab was restored")?;
                super::project::ensure(cx);
                Ok(cx.new(|cx| Self::new(&workspace, Some(saved), None, None, window, cx)))
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
        let tab = self.saved(cx);
        Some(graph_store::save_tab(workspace_id, item_id, tab, cx))
    }

    fn should_serialize(&self, event: &GraphEvent) -> bool {
        matches!(event, GraphEvent::UpdateTab)
    }
}
