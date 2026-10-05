//! Rusty's to-do lists in a center tab (#658): the screen Rusty's app draws as `TasksPage.qml`.
//!
//! The lists (Rusty's task groups) sit in a column on the left, the chosen list's tasks on the
//! right, both in Rusty's order. Every change is one of Rusty's task tools, sent one at a time in
//! the order made and read back; nothing is kept but what Rusty answers. The tab reads again on
//! Rusty's `list_changed` while it shows (else when it next shows), when it takes the focus from
//! outside, when Marley's window comes back to the front while it shows, and on Refresh: a task
//! another `rusty-mcp` writes, an agent's own, is announced to no other process.

use std::collections::VecDeque;

use editor::Editor;
use gpui::{
    Anchor, AnyElement, App, ClickEvent, Context, DismissEvent, Entity, EventEmitter, FocusHandle,
    Focusable, MouseDownEvent, Pixels, Point, PromptLevel, Render, ScrollHandle, SharedString,
    Subscription, Task, WeakEntity, Window, actions, anchored, deferred, px,
};
use marley_rusty::tasks::{
    LIST_TASK_GROUPS, LIST_TASKS, TaskGroup, TaskWrite, UserTask, groups_from_answer,
    id_from_answer, kept_list, kept_task, moved_one, tasks_from_answer, typed_name,
};
use serde_json::json;
use ui::{Checkbox, ContextMenu, ListItem, ListItemSpacing, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::launch::verbatim;
use crate::rail::order::{drag_preview, drop_line};

actions!(
    rusty,
    [
        /// Opens Rusty's to-do lists in a center tab, or brings forward the one open.
        #[derive(Eq)]
        OpenTasks,
        /// Checks the selected task done, or open again.
        #[derive(Eq)]
        ToggleTask,
        /// Renames the selected task in its row.
        #[derive(Eq)]
        RenameTask,
        /// Archives the selected task, or restores an archived one.
        #[derive(Eq)]
        ArchiveTask,
        /// Deletes the selected task for good, after asking.
        #[derive(Eq)]
        DeleteTask,
        /// Moves the selected task one place up.
        #[derive(Eq)]
        MoveTaskUp,
        /// Moves the selected task one place down.
        #[derive(Eq)]
        MoveTaskDown,
    ]
);

/// The lists column's width (Rusty's).
const LISTS_WIDTH: Pixels = px(240.);

/// Registers `rusty: open tasks` on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &OpenTasks, window, cx| {
            open(workspace, None, window, cx);
        });
    })
    .detach();
}

/// Opens the Tasks tab once the update in progress ends, on `list` when one is named (#655's
/// project view names its task group).
pub(crate) fn open_later(
    workspace: WeakEntity<Workspace>,
    list: Option<i64>,
    window: &Window,
    cx: &mut App,
) {
    window.defer(cx, move |window, cx| {
        workspace
            .update(cx, |workspace, cx| open(workspace, list, window, cx))
            .log_err();
    });
}

/// Brings the workspace's Tasks tab forward with the focus, or adds one to the active pane.
fn open(
    workspace: &mut Workspace,
    list: Option<i64>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if let Some(reason) = super::unavailable(cx) {
        workspace.show_toast(
            Toast::new(NotificationId::unique::<TasksView>(), reason.to_string()),
            cx,
        );
        return;
    }
    let open = workspace.items_of_type::<TasksView>(cx).next();
    let view = if let Some(view) = open {
        workspace.activate_item(&view, true, true, window, cx);
        view
    } else {
        let weak = cx.entity().downgrade();
        let view = cx.new(|cx| TasksView::new(weak, window, cx));
        workspace.add_item_to_active_pane(Box::new(view.clone()), None, true, window, cx);
        view
    };
    if let Some(list) = list {
        view.update(cx, |view, cx| view.show_list(list, cx));
    }
}

/// Where the tab is with Rusty's lists.
enum State {
    Reading,
    Ready,
    Failed(SharedString),
}

/// Marley's link to Rusty, as the tab last saw it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Link {
    Off,
    Down,
    Up,
}

impl Link {
    fn now(cx: &App) -> Self {
        if !super::is_on(cx) {
            Self::Off
        } else if super::is_connected(cx) {
            Self::Up
        } else {
            Self::Down
        }
    }
}

/// What a row's editor names or renames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RowTarget {
    NewList,
    RenameList(i64),
    RenameTask(i64),
}

/// The row editor that is open.
struct RowEditing {
    target: RowTarget,
    editor: Entity<Editor>,
    _blur: Subscription,
}

/// A task being dragged: what follows the pointer, and where it came from.
#[derive(Clone)]
struct DraggedTask {
    list: i64,
    id: i64,
    position: usize,
    title: SharedString,
}

impl Render for DraggedTask {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_preview(self.title.clone(), cx)
    }
}

/// A read the tab owes: none, one more after the one running, or one when the tab next shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadDue {
    No,
    AfterThis,
    WhenShown,
}

/// What a read brought: the lists, the list it read, and that list's tasks.
type Read = (Vec<TaskGroup>, Option<i64>, Vec<UserTask>);

/// The Tasks tab.
pub(crate) struct TasksView {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    /// The task list's own focus, under the tab's.
    list_focus: FocusHandle,
    lists: Vec<TaskGroup>,
    chosen: Option<i64>,
    tasks: Vec<UserTask>,
    selected: Option<usize>,
    show_archived: bool,
    add_field: Entity<Editor>,
    editing: Option<RowEditing>,
    writes: VecDeque<TaskWrite>,
    writing: Option<Task<()>>,
    reading: Option<Task<()>>,
    due: ReadDue,
    /// A new list was made: the add field takes the focus once it is read.
    focus_add: bool,
    /// Whether the add field was last set up for a chosen list.
    add_field_ready: bool,
    state: State,
    link: Link,
    menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ItemEvent> for TasksView {}

impl TasksView {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let add_field = cx.new(|cx| Editor::single_line(window, cx));
        let subscriptions = vec![
            cx.observe_global::<super::Announced>(|this, cx| {
                if this.showing(cx) {
                    this.read(cx);
                } else if this.due == ReadDue::No {
                    this.due = ReadDue::WhenShown;
                }
            }),
            cx.observe_global::<super::Rusty>(Self::rusty_changed),
            // Back from where an agent may have written: its own `rusty-mcp` tells no one.
            cx.on_focus_in(&focus_handle, window, |this, _, cx| this.read(cx)),
            cx.observe_window_activation(window, |this, window, cx| {
                if window.is_window_active() && this.showing(cx) {
                    this.read(cx);
                }
            }),
        ];
        let mut view = Self {
            workspace,
            focus_handle,
            list_focus: cx.focus_handle(),
            lists: Vec::new(),
            chosen: None,
            tasks: Vec::new(),
            selected: None,
            show_archived: false,
            add_field,
            editing: None,
            writes: VecDeque::new(),
            writing: None,
            reading: None,
            due: ReadDue::No,
            focus_add: false,
            add_field_ready: false,
            state: State::Reading,
            link: Link::now(cx),
            menu: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        view.sync_add_field(window, cx);
        view.read(cx);
        view
    }

    /// Chooses list `id` and reads it; #655's project view opens the tab through it.
    pub(crate) fn show_list(&mut self, id: i64, cx: &mut Context<Self>) {
        self.chosen = Some(id);
        self.selected = None;
        self.read(cx);
        cx.notify();
    }

    /// Whether the tab is its pane's active item in its workspace, asked each time (PR-607).
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

    /// Rusty off drops everything and calls nothing; Rusty connected again reads. The global
    /// changes for more than the link, so only a change of the link counts.
    fn rusty_changed(&mut self, cx: &mut Context<Self>) {
        let link = Link::now(cx);
        if link == self.link {
            return;
        }
        self.link = link;
        if link == Link::Off {
            self.lists.clear();
            self.tasks.clear();
            self.writes.clear();
            self.writing = None;
            self.reading = None;
            self.due = ReadDue::No;
            self.editing = None;
            self.state = State::Reading;
        } else if link == Link::Up {
            self.read(cx);
        }
        cx.notify();
    }

    /// Reads the lists and the chosen list's tasks; with one read running, one more after it.
    fn read(&mut self, cx: &Context<Self>) {
        if !super::is_on(cx) || !super::is_connected(cx) {
            return;
        }
        if self.reading.is_some() {
            self.due = ReadDue::AfterThis;
            return;
        }
        self.due = ReadDue::No;
        let groups = super::call_tool(LIST_TASK_GROUPS, json!({}), cx);
        let (chosen, include_archived) = (self.chosen, self.show_archived);
        self.reading = Some(cx.spawn(async move |this, cx| {
            let read = async {
                let lists = groups_from_answer(&groups.await?).map_err(|error| {
                    format!("{LIST_TASK_GROUPS}'s answer did not parse: {error}")
                })?;
                let chosen = kept_list(&lists, chosen);
                let Some(list) = chosen else {
                    return Ok((lists, None, Vec::new()));
                };
                let arguments = json!({ "group_id": list, "include_archived": include_archived });
                let asking = this
                    .update(cx, |_, cx| super::call_tool(LIST_TASKS, arguments, cx))
                    .map_err(|error| error.to_string())?;
                let text = asking.await?;
                let tasks = cx
                    .background_spawn(futures::future::lazy(move |_| tasks_from_answer(&text)))
                    .await
                    .map_err(|error| format!("{LIST_TASKS}'s answer did not parse: {error}"))?;
                Ok((lists, chosen, tasks))
            };
            let read: Result<Read, String> = read.await;
            this.update(cx, |this, cx| this.take_read(read, cx))
                .log_err();
        }));
    }

    fn take_read(&mut self, read: Result<Read, String>, cx: &mut Context<Self>) {
        self.reading = None;
        match read {
            Ok((lists, read_list, tasks)) => {
                self.lists = lists;
                let chosen = kept_list(&self.lists, self.chosen);
                if chosen == read_list {
                    let selected = self
                        .selected
                        .and_then(|at| self.tasks.get(at))
                        .map(|task| task.id);
                    self.selected = kept_task(&tasks, selected, self.selected);
                    self.tasks = tasks;
                } else {
                    // The list was changed while it was read: read the one chosen now.
                    self.due = ReadDue::AfterThis;
                }
                self.chosen = chosen;
                self.state = State::Ready;
            }
            Err(error) => {
                self.state = State::Failed(first_line(&error).into());
            }
        }
        if self.due == ReadDue::AfterThis {
            self.read(cx);
        }
        cx.notify();
    }

    /// Queues a change; the next one goes when the last has answered (D10).
    fn write(&mut self, write: TaskWrite, cx: &mut Context<Self>) {
        if !super::is_connected(cx) {
            self.toast("Marley is not connected to Rusty.".to_string(), cx);
            return;
        }
        self.writes.push_back(write);
        self.pump(cx);
    }

    fn pump(&mut self, cx: &Context<Self>) {
        if self.writing.is_some() {
            return;
        }
        let Some(write) = self.writes.pop_front() else {
            return;
        };
        let asking = super::call_tool(write.tool(), write.arguments(), cx);
        self.writing = Some(cx.spawn(async move |this, cx| {
            let answer = asking.await;
            this.update(cx, |this, cx| {
                this.writing = None;
                match answer {
                    Ok(text) => {
                        if matches!(write, TaskWrite::NewList { .. })
                            && let Ok(id) = id_from_answer(&text)
                        {
                            this.chosen = Some(id);
                            this.selected = None;
                            this.focus_add = true;
                        }
                    }
                    Err(error) => this.toast(first_line(&error).to_string(), cx),
                }
                this.read(cx);
                this.pump(cx);
            })
            .log_err();
        }));
    }

    fn toast(&self, message: String, cx: &mut App) {
        self.workspace
            .update(cx, |workspace, cx| {
                workspace.show_toast(Toast::new(NotificationId::unique::<Self>(), message), cx);
            })
            .log_err();
    }

    /// Whether edits are taken now: connected, and not waiting on a first read.
    fn editable(&self, cx: &App) -> bool {
        super::is_connected(cx) && matches!(self.state, State::Ready)
    }

    fn chosen_list(&self) -> Option<&TaskGroup> {
        let chosen = self.chosen?;
        self.lists.iter().find(|list| list.id == chosen)
    }

    fn selected_task(&self) -> Option<&UserTask> {
        self.tasks.get(self.selected?)
    }

    /// The add field's words and whether it takes text: there is no list to add to without one.
    fn sync_add_field(&self, window: &mut Window, cx: &mut Context<Self>) {
        let ready = self.chosen.is_some();
        self.add_field.update(cx, |field, cx| {
            field.set_read_only(!ready);
            field.set_placeholder_text(
                if ready {
                    "Add a task and press Enter"
                } else {
                    "Create a list first"
                },
                window,
                cx,
            );
        });
    }

    fn choose_list(&mut self, id: i64, cx: &mut Context<Self>) {
        if self.chosen != Some(id) {
            self.show_list(id, cx);
        }
    }

    fn select(&mut self, at: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = Some(at);
        self.scroll.scroll_to_item(at);
        window.focus(&self.list_focus, cx);
        cx.notify();
    }

    /// Adds the add field's title to the chosen list, empties the field and keeps the focus.
    fn add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(list) = self.chosen else {
            return;
        };
        let typed = self.add_field.read(cx).text(cx);
        self.add_field
            .update(cx, |field, cx| field.set_text("", window, cx));
        if let Some(title) = typed_name(&typed, None) {
            self.write(TaskWrite::NewTask { list, title }, cx);
        }
    }

    fn toggle(&mut self, id: i64, cx: &mut Context<Self>) {
        if self.editable(cx) {
            self.write(TaskWrite::Toggle { id }, cx);
        }
    }

    /// Archives the task, or restores it when archived.
    fn archive(&mut self, id: i64, cx: &mut Context<Self>) {
        let Some(task) = self.tasks.iter().find(|task| task.id == id) else {
            return;
        };
        let write = if task.archived {
            TaskWrite::Restore { id }
        } else {
            TaskWrite::Archive { id }
        };
        self.write(write, cx);
    }

    /// Asks before a task is deleted for good, naming it.
    fn delete_task(&self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        let Some(task) = self.tasks.iter().find(|task| task.id == id) else {
            return;
        };
        let detail = format!(
            "{}\n\nArchive hides a task and keeps it.",
            verbatim(&task.title)
        );
        Self::ask(
            "Delete this task for good?",
            &detail,
            TaskWrite::Delete { id },
            window,
            cx,
        );
    }

    /// Asks before a list is deleted, naming it and saying its tasks go too.
    fn delete_list(&self, id: i64, window: &mut Window, cx: &mut Context<Self>) {
        let Some(list) = self.lists.iter().find(|list| list.id == id) else {
            return;
        };
        let detail = format!(
            "{}\n\nIts tasks, archived ones too, go with it.",
            verbatim(&list.name)
        );
        Self::ask(
            "Delete this list and every task in it?",
            &detail,
            TaskWrite::DeleteList { id },
            window,
            cx,
        );
    }

    fn ask(
        question: &str,
        detail: &str,
        write: TaskWrite,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let answer = window.prompt(
            PromptLevel::Warning,
            question,
            Some(detail),
            &["Delete", "Cancel"],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await == Ok(0) {
                this.update(cx, |this, cx| this.write(write, cx)).log_err();
            }
        })
        .detach();
    }

    /// Moves the selected task one place, and calls nothing at either end.
    fn move_selected(&mut self, up: bool, cx: &mut Context<Self>) {
        let (Some(list), Some(task)) = (self.chosen, self.selected_task()) else {
            return;
        };
        let ids: Vec<i64> = self.tasks.iter().map(|task| task.id).collect();
        let Some(order) = moved_one(&ids, task.id, up) else {
            return;
        };
        let id = task.id;
        self.reorder(list, order, cx);
        self.selected = self.tasks.iter().position(|task| task.id == id);
        cx.notify();
    }

    /// Draws the new order at once, so the dropped row does not jump back while Rusty answers,
    /// and sends it; the read after it confirms or undoes it.
    fn reorder(&mut self, list: i64, ids: Vec<i64>, cx: &mut Context<Self>) {
        if !self.editable(cx) {
            return;
        }
        let mut tasks = std::mem::take(&mut self.tasks);
        self.tasks = ids
            .iter()
            .filter_map(|id| {
                let at = tasks.iter().position(|task| task.id == *id)?;
                Some(tasks.remove(at))
            })
            .collect();
        self.tasks.append(&mut tasks);
        self.write(TaskWrite::Reorder { list, ids }, cx);
    }

    /// Puts the dragged task in the target's place: before it when the target sat above.
    fn dropped(&mut self, dragged: &DraggedTask, target: (usize, i64), cx: &mut Context<Self>) {
        let ids: Vec<i64> = self.tasks.iter().map(|task| task.id).collect();
        let order = marley_rail::move_to(&ids, &dragged.id, &target.1, target.0 < dragged.position);
        if order != ids {
            let id = dragged.id;
            self.reorder(dragged.list, order, cx);
            self.selected = self.tasks.iter().position(|task| task.id == id);
        }
        cx.notify();
    }

    /// Opens a row's editor on `target`, holding `text`, all of it selected.
    fn start_editing(
        &mut self,
        target: RowTarget,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.editable(cx) {
            return;
        }
        self.commit_editing(window, cx);
        let editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_text(text, window, cx);
            editor.select_all(&editor::actions::SelectAll, window, cx);
            editor
        });
        window.focus(&editor.focus_handle(cx), cx);
        // Leaving the editor keeps its text, as the project panel's rename does.
        let blur = cx.on_blur(&editor.focus_handle(cx), window, |this, window, cx| {
            if window.is_window_active() {
                this.commit_editing(window, cx);
            }
        });
        self.editing = Some(RowEditing {
            target,
            editor,
            _blur: blur,
        });
        cx.notify();
    }

    /// Sends what the row editor holds, when it names something new.
    fn commit_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editing) = self.editing.take() else {
            return;
        };
        let typed = editing.editor.read(cx).text(cx);
        let write = match editing.target {
            RowTarget::NewList => typed_name(&typed, None).map(|name| TaskWrite::NewList { name }),
            RowTarget::RenameList(id) => {
                let current = self.lists.iter().find(|list| list.id == id);
                typed_name(&typed, current.map(|list| list.name.as_str()))
                    .map(|name| TaskWrite::RenameList { id, name })
            }
            RowTarget::RenameTask(id) => {
                let current = self.tasks.iter().find(|task| task.id == id);
                typed_name(&typed, current.map(|task| task.title.as_str()))
                    .map(|title| TaskWrite::Retitle { id, title })
            }
        };
        if let Some(write) = write {
            self.write(write, cx);
        }
        if editing.editor.focus_handle(cx).is_focused(window) {
            window.focus(&self.list_focus, cx);
        }
        cx.notify();
    }

    fn cancel_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.take().is_some() {
            window.focus(&self.list_focus, cx);
            cx.notify();
        }
    }

    fn select_next(&mut self, _: &menu::SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        if self.tasks.is_empty() {
            return;
        }
        let next = if self.add_field.focus_handle(cx).is_focused(window) {
            0
        } else {
            self.selected
                .map_or(0, |at| (at + 1).min(self.tasks.len() - 1))
        };
        self.select(next, window, cx);
    }

    fn select_previous(
        &mut self,
        _: &menu::SelectPrevious,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.tasks.is_empty() {
            return;
        }
        let previous = self.selected.map_or(0, |at| at.saturating_sub(1));
        self.select(previous, window, cx);
    }

    fn select_first(&mut self, _: &menu::SelectFirst, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tasks.is_empty() {
            self.select(0, window, cx);
        }
    }

    fn select_last(&mut self, _: &menu::SelectLast, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(last) = self.tasks.len().checked_sub(1) {
            self.select(last, window, cx);
        }
    }

    /// Enter: commits a row editor, adds from the add field, or renames the selected task.
    fn confirm(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            self.commit_editing(window, cx);
        } else if self.add_field.focus_handle(cx).is_focused(window) {
            self.add(window, cx);
        } else {
            self.rename_task(&RenameTask, window, cx);
        }
    }

    /// Escape: drops a row editor, goes from the list to the add field, or empties the add field
    /// and then lets Escape go on.
    fn cancel(&mut self, _: &menu::Cancel, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            self.cancel_editing(window, cx);
        } else if self.list_focus.is_focused(window) {
            window.focus(&self.add_field.focus_handle(cx), cx);
        } else if self.add_field.focus_handle(cx).is_focused(window)
            && !self.add_field.read(cx).text(cx).is_empty()
        {
            self.add_field
                .update(cx, |field, cx| field.set_text("", window, cx));
        } else {
            cx.propagate();
        }
    }

    fn toggle_task(&mut self, _: &ToggleTask, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_task().map(|task| task.id) {
            self.toggle(id, cx);
        }
    }

    fn rename_task(&mut self, _: &RenameTask, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(task) = self.selected_task() {
            let (id, title) = (task.id, task.title.clone());
            self.start_editing(RowTarget::RenameTask(id), &title, window, cx);
        }
    }

    fn archive_task(&mut self, _: &ArchiveTask, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_task().map(|task| task.id)
            && self.editable(cx)
        {
            self.archive(id, cx);
        }
    }

    fn delete_selected(&mut self, _: &DeleteTask, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_task().map(|task| task.id)
            && self.editable(cx)
        {
            self.delete_task(id, window, cx);
        }
    }

    fn move_up(&mut self, _: &MoveTaskUp, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selected(true, cx);
    }

    fn move_down(&mut self, _: &MoveTaskDown, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selected(false, cx);
    }

    /// A right-click menu, deployed by hand at the pointer.
    fn deploy_menu(
        &mut self,
        position: Point<Pixels>,
        entries: Vec<(&'static str, MenuChoice)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.editable(cx) {
            return;
        }
        let view = cx.entity().downgrade();
        let menu = ContextMenu::build(window, cx, move |mut menu, _, _| {
            for (label, choice) in entries {
                let view = view.clone();
                menu = menu.entry(label, None, move |window, cx| {
                    view.update(cx, |this, cx| this.chose(choice, window, cx))
                        .log_err();
                });
            }
            menu
        });
        let subscription = cx.subscribe_in(&menu, window, |this, _, _: &DismissEvent, _, cx| {
            this.menu = None;
            cx.notify();
        });
        window.focus(&menu.focus_handle(cx), cx);
        self.menu = Some((menu, position, subscription));
        cx.notify();
    }

    fn chose(&mut self, choice: MenuChoice, window: &mut Window, cx: &mut Context<Self>) {
        match choice {
            MenuChoice::RenameTask(id) => {
                if let Some(title) = self.tasks.iter().find(|task| task.id == id) {
                    let title = title.title.clone();
                    self.start_editing(RowTarget::RenameTask(id), &title, window, cx);
                }
            }
            MenuChoice::Archive(id) => self.archive(id, cx),
            MenuChoice::DeleteTask(id) => self.delete_task(id, window, cx),
            MenuChoice::RenameList(id) => {
                if let Some(list) = self.lists.iter().find(|list| list.id == id) {
                    let name = list.name.clone();
                    self.start_editing(RowTarget::RenameList(id), &name, window, cx);
                }
            }
            MenuChoice::DeleteList(id) => self.delete_list(id, window, cx),
        }
    }
}

/// A right-click menu's entry, by what it does.
#[derive(Clone, Copy, Debug)]
enum MenuChoice {
    RenameTask(i64),
    Archive(i64),
    DeleteTask(i64),
    RenameList(i64),
    DeleteList(i64),
}

/// The first line of a message, for a toast or the notice line.
fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or(text)
}

impl TasksView {
    fn render_row_editor(&self, target: RowTarget, cx: &App) -> Option<AnyElement> {
        let editing = self
            .editing
            .as_ref()
            .filter(|editing| editing.target == target)?;
        Some(
            div()
                .w_full()
                .px_1()
                .rounded_sm()
                .border_1()
                .border_color(cx.theme().colors().border_focused)
                .bg(cx.theme().colors().editor_background)
                .child(editing.editor.clone())
                .into_any_element(),
        )
    }

    fn render_lists(&self, cx: &Context<Self>) -> impl IntoElement {
        let editable = self.editable(cx);
        let rows = self.lists.iter().map(|list| {
            let id = list.id;
            let label = self
                .render_row_editor(RowTarget::RenameList(id), cx)
                .unwrap_or_else(|| Label::new(list.name.clone()).into_any_element());
            ListItem::new(("rusty-task-list", u64::try_from(id).unwrap_or_default()))
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(self.chosen == Some(id))
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| this.choose_list(id, cx)))
                .on_secondary_mouse_down(cx.listener(
                    move |this, event: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        window.prevent_default();
                        let entries = vec![
                            ("Rename", MenuChoice::RenameList(id)),
                            ("Delete List…", MenuChoice::DeleteList(id)),
                        ];
                        this.deploy_menu(event.position, entries, window, cx);
                    },
                ))
        });
        v_flex()
            .flex_none()
            .w(LISTS_WIDTH)
            .h_full()
            .border_r_1()
            .border_color(cx.theme().colors().border_variant)
            .child(
                h_flex()
                    .justify_between()
                    .px_2()
                    .py_1()
                    .child(
                        Label::new("Lists")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        IconButton::new("rusty-tasks-new-list", IconName::Plus)
                            .icon_size(IconSize::Small)
                            .disabled(!editable)
                            .tooltip(Tooltip::text("New List"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.start_editing(RowTarget::NewList, "", window, cx);
                            })),
                    ),
            )
            .child(
                v_flex()
                    .id("rusty-task-lists")
                    .px_1()
                    .overflow_y_scroll()
                    .children(rows)
                    .children(
                        self.render_row_editor(RowTarget::NewList, cx)
                            .map(|editor| div().px_1().py_0p5().child(editor)),
                    ),
            )
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let name = self
            .chosen_list()
            .map_or_else(|| "Tasks".to_string(), |list| list.name.clone());
        h_flex()
            .gap_2()
            .px_3()
            .py_2()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Label::new(name).size(LabelSize::Large).truncate()),
            )
            .child(
                Checkbox::new(
                    "rusty-tasks-show-archived",
                    ToggleState::from(self.show_archived),
                )
                .label("Show archived")
                .label_size(LabelSize::Small)
                .on_click(cx.listener(|this, _: &ToggleState, _, cx| {
                    this.show_archived = !this.show_archived;
                    this.read(cx);
                    cx.notify();
                })),
            )
            .child(
                IconButton::new("rusty-tasks-refresh", IconName::RotateCw)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Refresh"))
                    .on_click(cx.listener(|this, _, _, cx| this.read(cx))),
            )
    }

    fn render_task(&self, at: usize, task: &UserTask, cx: &Context<Self>) -> impl IntoElement {
        let (id, list) = (task.id, task.header_id);
        let editable = self.editable(cx);
        let colors = cx.theme().colors();
        let title = self
            .render_row_editor(RowTarget::RenameTask(id), cx)
            .unwrap_or_else(|| {
                let label = Label::new(task.title.clone()).when(task.completed, |label| {
                    label.strikethrough().color(Color::Muted)
                });
                label.into_any_element()
            });
        let check = Checkbox::new(
            ("rusty-task-done", u64::try_from(id).unwrap_or_default()),
            ToggleState::from(task.completed),
        )
        // Filled, so the box shows on a selected row's background too.
        .fill()
        .disabled(!editable)
        .on_click(cx.listener(move |this, _: &ToggleState, _, cx| {
            cx.stop_propagation();
            this.toggle(id, cx);
        }));
        let archived = task.archived;
        let row = ListItem::new(("rusty-task-row", u64::try_from(id).unwrap_or_default()))
            .spacing(ListItemSpacing::Sparse)
            .toggle_state(self.selected == Some(at))
            .start_slot(check)
            .child(
                div()
                    .when(archived, |title| title.opacity(0.5))
                    .child(title),
            )
            .when(archived, |row| {
                row.end_slot(
                    Label::new("Archived")
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
            })
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.select(at, window, cx);
                if event.click_count() >= 2 {
                    this.rename_task(&RenameTask, window, cx);
                }
            }))
            .on_secondary_mouse_down(cx.listener(
                move |this, event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    window.prevent_default();
                    this.selected = Some(at);
                    let entries = vec![
                        ("Rename", MenuChoice::RenameTask(id)),
                        (
                            if archived { "Restore" } else { "Archive" },
                            MenuChoice::Archive(id),
                        ),
                        ("Delete for Good…", MenuChoice::DeleteTask(id)),
                    ];
                    this.deploy_menu(event.position, entries, window, cx);
                },
            ));
        let dragged = DraggedTask {
            list,
            id,
            position: at,
            title: task.title.clone().into(),
        };
        div()
            .id(("rusty-task", u64::try_from(id).unwrap_or_default()))
            .when(editable && self.editing.is_none(), |row| {
                row.on_drag(dragged, |dragged, _, _, cx| cx.new(|_| dragged.clone()))
                    .can_drop(move |dragged, _, _| {
                        dragged
                            .downcast_ref::<DraggedTask>()
                            .is_some_and(|dragged| dragged.list == list && dragged.id != id)
                    })
                    .drag_over::<DraggedTask>(move |style, dragged, _, cx| {
                        drop_line(style, at < dragged.position, cx)
                    })
                    .on_drop(cx.listener(move |this, dragged: &DraggedTask, _, cx| {
                        this.dropped(dragged, (at, id), cx);
                    }))
            })
            .border_color(colors.border_transparent)
            .child(row)
    }

    /// The line under the rows: what the tab is waiting for, or what it found.
    fn render_notice(&self, cx: &Context<Self>) -> Option<AnyElement> {
        if !super::is_on(cx) {
            return Some(notice(
                "Rusty is off. Turn it on in the Rusty section of the Marley settings.",
            ));
        }
        if !super::is_connected(cx) {
            let reason = super::unavailable(cx).unwrap_or_default();
            return Some(notice(format!(
                "Marley is not connected to Rusty: {reason}"
            )));
        }
        match &self.state {
            State::Reading => Some(notice("Reading Rusty's lists…")),
            State::Failed(error) => Some(
                h_flex()
                    .gap_2()
                    .px_3()
                    .child(
                        Label::new(error.clone())
                            .size(LabelSize::Small)
                            .color(Color::Error),
                    )
                    .child(
                        Button::new("rusty-tasks-read-again", "Read Again")
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(|this, _, _, cx| this.read(cx))),
                    )
                    .into_any_element(),
            ),
            State::Ready if self.lists.is_empty() => Some(notice("No list yet. Make one with +.")),
            State::Ready if self.tasks.is_empty() && self.show_archived => {
                Some(notice("Nothing here"))
            }
            State::Ready if self.tasks.is_empty() => {
                Some(notice("Nothing open. Type above to add a task."))
            }
            State::Ready => None,
        }
    }
}

fn notice(text: impl Into<SharedString>) -> AnyElement {
    div()
        .px_3()
        .py_2()
        .child(Label::new(text).size(LabelSize::Small).color(Color::Muted))
        .into_any_element()
}

impl Focusable for TasksView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TasksView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // A change announced while the tab was hidden is read now that it draws (AD-609).
        if self.due == ReadDue::WhenShown && self.reading.is_none() {
            self.read(cx);
        }
        // The add field takes text once a list is chosen, which the first read does.
        if self.chosen.is_some() != self.add_field_ready {
            self.add_field_ready = self.chosen.is_some();
            self.sync_add_field(window, cx);
        }
        if std::mem::take(&mut self.focus_add) {
            window.focus(&self.add_field.focus_handle(cx), cx);
        }
        let list_context = if self.editing.is_none() {
            "RustyTaskList not_editing"
        } else {
            "RustyTaskList"
        };
        let rows: Vec<AnyElement> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(at, task)| self.render_task(at, task, cx).into_any_element())
            .collect();
        let colors = cx.theme().colors();
        h_flex()
            .key_context("RustyTasks menu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .size_full()
            .bg(colors.editor_background)
            .child(self.render_lists(cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.render_header(cx))
                    .child(
                        div().px_3().pb_2().child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .border_1()
                                .border_color(colors.border)
                                .child(self.add_field.clone()),
                        ),
                    )
                    .child(
                        v_flex()
                            .id("rusty-tasks")
                            .key_context(list_context)
                            .track_focus(&self.list_focus)
                            .on_action(cx.listener(Self::toggle_task))
                            .on_action(cx.listener(Self::rename_task))
                            .on_action(cx.listener(Self::archive_task))
                            .on_action(cx.listener(Self::delete_selected))
                            .on_action(cx.listener(Self::move_up))
                            .on_action(cx.listener(Self::move_down))
                            .flex_1()
                            .min_h_0()
                            .px_2()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .children(rows)
                            .children(self.render_notice(cx)),
                    ),
            )
            .children(self.menu.as_ref().map(|(menu, position, _)| {
                deferred(
                    anchored()
                        .position(*position)
                        .anchor(Anchor::TopLeft)
                        .child(menu.clone()),
                )
                .with_priority(1)
            }))
    }
}

impl Item for TasksView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Tasks")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ListTodo))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static("Rusty's to-do lists"))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    /// With the service connection no change is announced, so each showing reads (#647's D9).
    fn deactivated(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(
            cx.global::<super::Rusty>().source,
            super::Source::Service(_)
        ) {
            self.due = ReadDue::WhenShown;
        }
    }

    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        // L-613: an item moved to another workspace follows it.
        self.workspace = workspace.weak_handle();
    }
}
