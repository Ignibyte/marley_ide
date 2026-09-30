//! Dragging the rail's headers and rows to reorder them (#602), and the order the user leaves by
//! it, which the window's saved sidebar state keeps beside the rail's other fields.

use std::collections::{HashMap, HashSet};

use gpui::{
    AnyElement, App, Context, Div, Entity, Render, SharedString, Stateful, StyleRefinement, Window,
};
use marley_rail::{Run, Selection};
use project::ProjectGroupKey;
use terminal_view::TerminalView;
use ui::{Label, LabelSize, prelude::*};
use util::ResultExt as _;
use uuid::Uuid;
use workspace::{MultiWorkspace, WorkspaceId};

use super::{GroupEntry, Rail, Snapshot};
use crate::browser::BrowserView;

/// The order the window saved for the rail: its projectless groups' (#601), and the one the user
/// left by dragging (#602).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct SavedOrder {
    /// The projectless groups' workspace ids, in the order the window saved them (#601).
    pub(super) groups: Vec<WorkspaceId>,
    /// The headers' places, top first (see [`header_place`]).
    pub(super) headers: Vec<String>,
    /// Per header's place, its rows' places in the order the user left them.
    pub(super) rows: HashMap<String, Vec<String>>,
}

/// The order the rail holds while the pointer is over it (#542), and whether a header or a row
/// dragged from it is in flight, which holds the order however the pointer's hover reads.
#[derive(Debug, Default)]
pub(super) struct Hold {
    pub(super) order: Option<marley_rail::Held>,
    pub(super) dragging: bool,
}

/// The rail's order in a saved sidebar blob, beside #601's `marley_groups`.
#[derive(Default, serde::Deserialize)]
struct SavedPlaces {
    #[serde(default)]
    headers: Vec<String>,
    #[serde(default)]
    rows: HashMap<String, Vec<String>>,
}

#[derive(Default, serde::Deserialize)]
struct SavedBlob {
    #[serde(default)]
    marley_order: SavedPlaces,
}

/// The order a saved sidebar blob keeps: the headers' places and each header's rows'; none when
/// it cannot be read.
pub(super) fn read_rail_order(blob: &str) -> (Vec<String>, HashMap<String, Vec<String>>) {
    let saved = serde_json::from_str::<SavedBlob>(blob)
        .unwrap_or_default()
        .marley_order;
    (saved.headers, saved.rows)
}

/// `blob` with the rail's order written into it and every other field kept.
pub(super) fn write_rail_order(blob: &str, order: &SavedOrder) -> String {
    let mut fields: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(blob).unwrap_or_default();
    fields.insert(
        "marley_order".into(),
        serde_json::json!({ "headers": order.headers, "rows": order.rows }),
    );
    serde_json::Value::Object(fields).to_string()
}

/// A header's place, which survives a restart: a project by its folders, a projectless group by
/// its id (#601 brings the id back). Two projects on the same folders on different hosts share
/// one place, and sort together.
pub(super) fn header_place(key: &ProjectGroupKey, group: Option<Uuid>) -> String {
    group.map_or_else(
        || {
            let folders: Vec<String> = key
                .path_list()
                .paths()
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect();
            format!("project:{}", folders.join("\n"))
        },
        |group| format!("group:{group}"),
    )
}

/// The place of a listed group's header.
pub(super) fn group_place(group: &GroupEntry) -> String {
    header_place(&group.key, group.group)
}

/// A terminal row's place: its `MARLEY_TERMINAL_ID` (#575), or, for a task's or a remote
/// terminal, which has none, its view, which holds its place for the session only.
pub(super) fn terminal_place(view: &Entity<TerminalView>, cx: &App) -> String {
    view.read(cx)
        .terminal()
        .read(cx)
        .marley_terminal_id()
        .map_or_else(
            || format!("view:{}", view.entity_id().as_u64()),
            |id| format!("terminal:{id}"),
        )
}

/// A Browser tab row's place: the page it shows, which the tab saves and takes back (#494), or its
/// view while it has no page yet.
pub(super) fn browser_place(view: &Entity<BrowserView>, cx: &App) -> String {
    view.read(cx).target().map_or_else(
        || format!("view:{}", view.entity_id().as_u64()),
        |target| format!("browser:{target}"),
    )
}

/// A thread row's place: its thread's key.
fn thread_place(key: &str) -> String {
    format!("thread:{key}")
}

/// Puts each project's terminals, Browser tabs and threads in the order the user left them; rows
/// with no place keep their order after the placed ones.
pub(super) fn arrange(snapshot: &mut Snapshot, order: &SavedOrder) {
    let Snapshot {
        rail,
        groups,
        places,
        ..
    } = snapshot;
    for (project, group) in rail.projects.iter_mut().zip(groups.iter()) {
        let Some(placed) = order.rows.get(&group_place(group)) else {
            continue;
        };
        marley_rail::place(&mut project.terminals, placed, |terminal| {
            places.get(&Selection::Terminal(terminal.id)).cloned()
        });
        marley_rail::place(&mut project.browsers, placed, |browser| {
            places.get(&Selection::Browser(browser.id)).cloned()
        });
        marley_rail::place(&mut project.threads, placed, |thread| {
            Some(thread_place(&thread.key))
        });
    }
}

/// A header being dragged, which is also what each header's block takes a drop as.
#[derive(Debug, Clone)]
pub(super) struct DraggedRailHeader {
    place: String,
    /// Where the header sat in the rail when it was drawn, which decides the side of a target's
    /// line.
    position: usize,
    run: Run,
    name: SharedString,
}

impl DraggedRailHeader {
    pub(super) fn new(group: &GroupEntry, position: usize, run: Run, name: &str) -> Self {
        Self {
            place: group_place(group),
            position,
            run,
            name: SharedString::from(name.to_string()),
        }
    }

    /// Whether `dragged` may land in this header's place.
    fn takes(&self, dragged: &Self) -> bool {
        dragged.place != self.place && dragged.run == self.run
    }
}

impl Render for DraggedRailHeader {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_preview(self.name.clone(), cx)
    }
}

/// A terminal, Browser tab or thread row being dragged, which is also what each such row takes a
/// drop as.
#[derive(Debug, Clone)]
pub(super) struct DraggedRailRow {
    selection: Selection,
    /// The project's index in the snapshot.
    project: usize,
    /// Where the row sat in the rail when it was drawn.
    position: usize,
    run: Run,
    title: SharedString,
}

impl DraggedRailRow {
    pub(super) fn new(
        selection: Selection,
        project: usize,
        position: usize,
        run: Run,
        title: &str,
    ) -> Self {
        Self {
            selection,
            project,
            position,
            run,
            title: SharedString::from(title.to_string()),
        }
    }

    /// Whether `dragged` may land in this row's place: a row of its kind, section and class.
    fn takes(&self, dragged: &Self) -> bool {
        dragged.selection != self.selection && dragged.run == self.run
    }
}

impl Render for DraggedRailRow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_preview(self.title.clone(), cx)
    }
}

/// What follows the pointer while a header or a row is dragged: its name on a raised card, as a
/// pane's dragged tab shows its own.
fn drag_preview(title: SharedString, cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();
    div()
        .max_w_64()
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(colors.border)
        .bg(colors.elevated_surface_background)
        .shadow_md()
        .child(Label::new(title).size(LabelSize::Small).truncate())
}

/// The line on the edge a drop lands on: the top when the target sits above what is dragged, the
/// bottom otherwise, as a pane's tab bar draws it on the side the tab lands.
fn drop_line(style: StyleRefinement, above: bool, cx: &App) -> StyleRefinement {
    let style = style
        .border_0()
        .border_color(cx.theme().colors().drop_target_border);
    if above {
        style.border_t_2()
    } else {
        style.border_b_2()
    }
}

impl Rail {
    /// Makes a header draggable; its block takes the drop.
    pub(super) fn draggable_header(
        header: Stateful<Div>,
        this: Option<DraggedRailHeader>,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let Some(this) = this else {
            return header;
        };
        let rail = cx.entity().downgrade();
        header.on_drag(this, move |dragged, _, _, cx| {
            rail.update(cx, |rail, _| rail.start_drag()).log_err();
            cx.new(|_| dragged.clone())
        })
    }

    /// A project's header and the rows under it, as one block a dragged header lands on.
    pub(super) fn header_block(
        header: AnyElement,
        rows: Vec<AnyElement>,
        this: Option<DraggedRailHeader>,
        cx: &Context<Self>,
    ) -> AnyElement {
        v_flex()
            .gap_0p5()
            .child(header)
            .children(rows)
            .when_some(this, |block, this| {
                let (allowed, over) = (this.clone(), this.clone());
                block
                    .can_drop(move |dragged, _, _| {
                        dragged
                            .downcast_ref::<DraggedRailHeader>()
                            .is_some_and(|dragged| allowed.takes(dragged))
                    })
                    .drag_over::<DraggedRailHeader>(move |style, dragged, _, cx| {
                        drop_line(style, over.position < dragged.position, cx)
                    })
                    .on_drop(
                        cx.listener(move |rail, dragged: &DraggedRailHeader, window, cx| {
                            rail.drop_header(dragged, &this, window, cx);
                        }),
                    )
            })
            .into_any_element()
    }

    /// Makes a terminal's, Browser tab's or thread's row draggable, and a place its kind drops on.
    pub(super) fn draggable_row(
        card: Stateful<Div>,
        this: Option<DraggedRailRow>,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let Some(this) = this else {
            return card;
        };
        let rail = cx.entity().downgrade();
        let (allowed, over, dropped) = (this.clone(), this.clone(), this.clone());
        card.on_drag(this, move |dragged, _, _, cx| {
            rail.update(cx, |rail, _| rail.start_drag()).log_err();
            cx.new(|_| dragged.clone())
        })
        .can_drop(move |dragged, _, _| {
            dragged
                .downcast_ref::<DraggedRailRow>()
                .is_some_and(|dragged| allowed.takes(dragged))
        })
        .drag_over::<DraggedRailRow>(move |style, dragged, _, cx| {
            drop_line(style, over.position < dragged.position, cx)
        })
        .on_drop(
            cx.listener(move |rail, dragged: &DraggedRailRow, window, cx| {
                rail.drop_row(dragged, &dropped, window, cx);
            }),
        )
    }

    /// Puts the dragged header in `target`'s place: before it when the target sat above, after it
    /// otherwise.
    fn drop_header(
        &mut self,
        dragged: &DraggedRailHeader,
        target: &DraggedRailHeader,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let headers: Vec<String> = self.snapshot.groups.iter().map(group_place).collect();
        self.saved_order.headers = marley_rail::move_to(
            &headers,
            &dragged.place,
            &target.place,
            target.position < dragged.position,
        );
        // The keyboard's header is kept by index, which now names another project.
        if matches!(self.cursor, Some(Selection::Project(_))) {
            self.cursor = None;
        }
        self.order_changed(window, cx);
    }

    /// Puts the dragged row in `target`'s place among its group's rows.
    fn drop_row(
        &mut self,
        dragged: &DraggedRailRow,
        target: &DraggedRailRow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (Some(project), Some(group)) = (
            self.snapshot.rail.projects.get(target.project),
            self.snapshot.groups.get(target.project),
        ) else {
            self.end_drag(window, cx);
            return;
        };
        let rows: Vec<Selection> = project
            .terminals
            .iter()
            .map(|terminal| Selection::Terminal(terminal.id))
            .chain(
                project
                    .browsers
                    .iter()
                    .map(|browser| Selection::Browser(browser.id)),
            )
            .chain(
                project
                    .threads
                    .iter()
                    .map(|thread| Selection::Thread(thread.key.clone())),
            )
            .collect();
        let rows = marley_rail::move_to(
            &rows,
            &dragged.selection,
            &target.selection,
            target.position < dragged.position,
        );
        let known = &self.snapshot.places;
        let placed = rows
            .iter()
            .filter_map(|row| match row {
                Selection::Thread(key) => Some(thread_place(key)),
                row => known.get(row).cloned(),
            })
            .collect();
        self.saved_order.rows.insert(group_place(group), placed);
        self.order_changed(window, cx);
    }

    /// The order changed: the window's saved state keeps it, and the order the rail held is let
    /// go, since its project indices may now name other projects; the next time the pointer comes
    /// over the rail it holds the new order.
    pub(super) fn order_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let listed: HashSet<String> = self.snapshot.groups.iter().map(group_place).collect();
        self.saved_order
            .rows
            .retain(|place, _| listed.contains(place));
        self.multi_workspace
            .update(cx, MultiWorkspace::serialize)
            .log_err();
        self.hold = Hold::default();
        self.refresh(window, cx);
    }

    /// A header or a row began to move: the rail's order holds until the drag ends, taken now if
    /// the pointer's hover had not taken it.
    fn start_drag(&mut self) {
        self.hold.dragging = true;
        if self.hold.order.is_none() {
            self.hold.order = Some(marley_rail::held_order(&self.snapshot.rail));
        }
    }

    /// A drag that began in the rail ended without landing: the order it held is let go.
    pub(super) fn end_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.hold.dragging {
            return;
        }
        self.hold = Hold::default();
        self.refresh(window, cx);
    }
}
