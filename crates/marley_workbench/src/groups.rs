//! Projectless groups (#600): named groups in the rail with no folder.
//!
//! Each is a folderless workspace Marley keeps in its window: its terminals and agent CLIs start
//! in the home folder, and its Browser tabs use a Chromium of the group's own. The rail lists them
//! after the window's projects; the window's Home group takes what the rail's empty space makes.
//!
//! Since #601 a group comes back after a restart. Every group's record is kept in Zed's key-value
//! store and read at startup, before Zed restores a window, so a group's Browser tabs find their
//! group while they deserialize; each window's rail keeps its groups' workspace ids in its saved
//! state and reopens them, and a record is adopted by the workspace that holds its id.

use std::sync::Arc;

use anyhow::Context as _;
use db::kvp::KeyValueStore;
use editor::Editor;
use gpui::{
    AnyWindowHandle, App, AppContext as _, Context, DismissEvent, Entity, EntityId, EventEmitter,
    FocusHandle, Focusable, Global, Render, TaskExt as _, WeakEntity, Window, WindowId,
};
use ui::{Headline, HeadlineSize, prelude::*};
use util::ResultExt as _;
use uuid::Uuid;
use workspace::{
    MarleyKeptWorkspaces, ModalView, MultiWorkspace, OpenMode, Workspace, WorkspaceId,
};

use crate::MakeGroup;

/// The name of the window's group for what the rail's empty space makes.
const HOME: &str = "Home";

/// The name of the window's group for Rusty's screens and pages (#675).
const RUSTY: &str = "Rusty";

/// The name an unnamed group takes, numbered from its second.
const UNNAMED: &str = "Group";

/// The store's scope and key for every group's record (#601).
const SCOPE: &str = "marley-groups";
const KEY: &str = "groups";

/// A projectless group.
#[derive(Clone)]
pub(crate) struct Group {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) workspace: WeakEntity<Workspace>,
    /// Its workspace's id in Zed's database, which a restart reopens it by (#601).
    pub(crate) database_id: Option<WorkspaceId>,
    /// Its workspace's project, kept so the browser can find the group while its workspace is
    /// being updated, when the workspace cannot be read.
    project: EntityId,
    pub(crate) expanded: bool,
    /// Whether it is its window's Home group.
    pub(crate) home: bool,
    /// Whether it is its window's Rusty group, where Rusty's screens and pages open (#675).
    pub(crate) rusty: bool,
}

/// What a group is made for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroupKind {
    /// One the user names.
    Named,
    /// The window's Home group, which takes what the rail's empty space makes.
    Home,
    /// The window's Rusty group (#675).
    Rusty,
}

/// What waits for a window's Rusty group while it is being made (#675).
type RustyOpen = Box<dyn FnOnce(Entity<Workspace>, &mut Window, &mut App)>;

/// What the store keeps of a group (#601).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SavedGroup {
    database_id: WorkspaceId,
    id: Uuid,
    name: String,
    home: bool,
    #[serde(default)]
    rusty: bool,
    expanded: bool,
}

/// Every window's groups, in the order they were made; the rail observes it.
#[derive(Default)]
pub(crate) struct Groups {
    live: Vec<Group>,
    /// The groups read from the store whose workspace no window holds yet (#601).
    pending: Vec<SavedGroup>,
    /// The opens waiting for their window's Rusty group while it is being made, by window, so two
    /// quick opens make one group (#675).
    waiting_for_rusty: Vec<(WindowId, RustyOpen)>,
}

impl Global for Groups {}

/// Reads the groups the store keeps, before Zed restores a window (#601).
pub(crate) fn init(cx: &mut App) {
    let pending = KeyValueStore::global(cx)
        .scoped(SCOPE)
        .read(KEY)
        .log_err()
        .flatten()
        .and_then(|text| serde_json::from_str::<Vec<SavedGroup>>(&text).log_err())
        .unwrap_or_default();
    cx.set_global(Groups {
        live: Vec::new(),
        pending,
        waiting_for_rusty: Vec::new(),
    });
}

/// Writes every group with a workspace id, open or still pending, to the store.
fn save(cx: &App) {
    let Some(groups) = cx.try_global::<Groups>() else {
        return;
    };
    let saved: Vec<SavedGroup> = groups
        .live
        .iter()
        .filter_map(|group| {
            Some(SavedGroup {
                database_id: group.database_id?,
                id: group.id,
                name: group.name.clone(),
                home: group.home,
                rusty: group.rusty,
                expanded: group.expanded,
            })
        })
        .chain(groups.pending.iter().cloned())
        .collect();
    let Some(text) = serde_json::to_string(&saved).log_err() else {
        return;
    };
    let store = KeyValueStore::global(cx);
    cx.background_spawn(async move { store.scoped(SCOPE).write(KEY.to_string(), text).await })
        .detach_and_log_err(cx);
}

/// The groups `multi_workspace` holds, each with its workspace, in the order they were made.
pub(crate) fn groups_of(
    multi_workspace: &MultiWorkspace,
    cx: &App,
) -> Vec<(Group, Entity<Workspace>)> {
    let Some(groups) = cx.try_global::<Groups>() else {
        return Vec::new();
    };
    groups
        .live
        .iter()
        .filter_map(|group| {
            let workspace = group.workspace.upgrade()?;
            multi_workspace
                .workspaces()
                .any(|held| *held == workspace)
                .then(|| (group.clone(), workspace))
        })
        .collect()
}

/// Makes each pending group whose workspace `multi_workspace` holds live, with that workspace
/// (#601). It writes the registry only when there is a group to adopt, since a write notifies
/// the rail, which calls this again.
pub(crate) fn adopt(multi_workspace: &Entity<MultiWorkspace>, cx: &mut App) {
    let found: Vec<(WorkspaceId, Entity<Workspace>)> = cx
        .try_global::<Groups>()
        .map(|groups| {
            multi_workspace
                .read(cx)
                .workspaces()
                .filter_map(|workspace| {
                    let database_id = workspace.read(cx).database_id()?;
                    groups
                        .pending
                        .iter()
                        .any(|saved| saved.database_id == database_id)
                        .then(|| (database_id, workspace.clone()))
                })
                .collect()
        })
        .unwrap_or_default();
    if found.is_empty() {
        return;
    }
    for (database_id, workspace) in found {
        let project = workspace.read(cx).project().entity_id();
        let groups = cx.default_global::<Groups>();
        let Some(at) = groups
            .pending
            .iter()
            .position(|saved| saved.database_id == database_id)
        else {
            continue;
        };
        let saved = groups.pending.remove(at);
        groups.live.push(Group {
            id: saved.id,
            name: saved.name,
            workspace: workspace.downgrade(),
            database_id: Some(database_id),
            project,
            expanded: saved.expanded,
            home: saved.home,
            rusty: saved.rusty,
        });
    }
    keep(cx);
}

/// Reopens into `window` each group whose workspace id `saved` lists and the window does not
/// hold yet (#601); the rail adopts each one as its workspace arrives. A workspace Zed no longer
/// has drops its group.
pub(crate) fn reopen(
    multi_workspace: &Entity<MultiWorkspace>,
    saved: &[WorkspaceId],
    window: &Window,
    cx: &mut App,
) {
    let (app_state, held) = {
        let multi_workspace = multi_workspace.read(cx);
        let held: Vec<WorkspaceId> = multi_workspace
            .workspaces()
            .filter_map(|workspace| workspace.read(cx).database_id())
            .collect();
        (
            Arc::clone(multi_workspace.workspace().read(cx).app_state()),
            held,
        )
    };
    let handle = window.window_handle().downcast::<MultiWorkspace>();
    for &database_id in saved {
        let pending = cx.try_global::<Groups>().is_some_and(|groups| {
            groups
                .pending
                .iter()
                .any(|saved| saved.database_id == database_id)
        });
        if held.contains(&database_id) || !pending {
            continue;
        }
        let opened =
            workspace::open_workspace_by_id(database_id, Arc::clone(&app_state), handle, cx);
        cx.spawn(async move |cx| {
            if let Err(error) = opened.await {
                log::warn!(
                    "a projectless group's workspace {database_id:?} did not reopen, so the \
                     group is dropped: {error:#}"
                );
                cx.update(|cx| forget_pending(database_id, cx));
            }
        })
        .detach();
    }
}

/// Drops the pending group whose workspace `database_id` could not be reopened (#601).
pub(crate) fn forget_pending(database_id: WorkspaceId, cx: &mut App) {
    cx.default_global::<Groups>()
        .pending
        .retain(|saved| saved.database_id != database_id);
    save(cx);
}

/// The group whose workspace's project is `project`: its id and name, for its browser (#600).
/// It reads no workspace: the browser asks while the group's own workspace is being updated.
pub(crate) fn group_of_project(project: EntityId, cx: &App) -> Option<(Uuid, String)> {
    cx.try_global::<Groups>()?
        .live
        .iter()
        .find(|group| group.project == project)
        .map(|group| (group.id, group.name.clone()))
}

/// The group, open or still pending, whose workspace has the id `database_id`: its id and name,
/// for a Browser tab that deserializes before its workspace is adopted (#601).
pub(crate) fn group_of_workspace_id(database_id: WorkspaceId, cx: &App) -> Option<(Uuid, String)> {
    let groups = cx.try_global::<Groups>()?;
    groups
        .live
        .iter()
        .find(|group| group.database_id == Some(database_id))
        .map(|group| (group.id, group.name.clone()))
        .or_else(|| {
            groups
                .pending
                .iter()
                .find(|saved| saved.database_id == database_id)
                .map(|saved| (saved.id, saved.name.clone()))
        })
}

/// Makes a group in `window` of `kind`, named `name` when it is a named one, and runs `then` with
/// its workspace in the window once the workspace exists.
pub(crate) fn make(
    multi_workspace: &Entity<MultiWorkspace>,
    name: &str,
    kind: GroupKind,
    then: impl FnOnce(Entity<Workspace>, &mut Window, &mut App) + 'static,
    window: &Window,
    cx: &mut App,
) {
    let app_state = Arc::clone(multi_workspace.read(cx).workspace().read(cx).app_state());
    let handle = window.window_handle().downcast::<MultiWorkspace>();
    let name = match kind {
        GroupKind::Home => HOME.to_string(),
        GroupKind::Rusty => RUSTY.to_string(),
        GroupKind::Named => name.trim().to_string(),
    };
    let opened = Workspace::new_local(Vec::new(), app_state, handle, None, None, OpenMode::Add, cx);
    cx.spawn(async move |cx| {
        let opened = opened.await.context("making a projectless group")?;
        let workspace = opened.workspace;
        cx.update(|cx| {
            let taken = taken_names(cx);
            let (project, database_id) = {
                let workspace = workspace.read(cx);
                (workspace.project().entity_id(), workspace.database_id())
            };
            cx.default_global::<Groups>().live.push(Group {
                id: Uuid::new_v4(),
                name: free_name(&name, &taken),
                workspace: workspace.downgrade(),
                database_id,
                project,
                expanded: true,
                home: kind == GroupKind::Home,
                rusty: kind == GroupKind::Rusty,
            });
            keep(cx);
            save(cx);
        });
        // Through the window alone: its `MultiWorkspace` stays free for `then`, which the rail's
        // Home path updates to show the group.
        AnyWindowHandle::from(opened.window)
            .update(cx, |_, window, cx| then(workspace, window, cx))
            .context("the window closed while its group was made")
    })
    .detach_and_log_err(cx);
}

/// The window's Rusty group's workspace, if the window has the group (#675).
pub(crate) fn rusty_group(multi_workspace: &MultiWorkspace, cx: &App) -> Option<Entity<Workspace>> {
    groups_of(multi_workspace, cx)
        .into_iter()
        .find_map(|(group, workspace)| group.rusty.then_some(workspace))
}

/// Whether the group `id` is a window's Rusty group (#675).
pub(crate) fn is_rusty(id: Uuid, cx: &App) -> bool {
    cx.try_global::<Groups>().is_some_and(|groups| {
        groups
            .live
            .iter()
            .any(|group| group.id == id && group.rusty)
    })
}

/// Runs `then` with the window's Rusty group's workspace, making the group first when the window
/// has none; opens asked for while it is being made wait for it, so there is one (#675).
pub(crate) fn with_rusty_group(
    multi_workspace: &Entity<MultiWorkspace>,
    then: impl FnOnce(Entity<Workspace>, &mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(workspace) = rusty_group(multi_workspace.read(cx), cx) {
        then(workspace, window, cx);
        return;
    }
    let window_id = window.window_handle().window_id();
    let groups = cx.default_global::<Groups>();
    let making = groups
        .waiting_for_rusty
        .iter()
        .any(|(id, _)| *id == window_id);
    groups.waiting_for_rusty.push((window_id, Box::new(then)));
    if making {
        return;
    }
    make(
        multi_workspace,
        "",
        GroupKind::Rusty,
        move |workspace, window, cx| {
            let waiting: Vec<RustyOpen> = {
                let groups = cx.default_global::<Groups>();
                let (mine, others): (Vec<_>, Vec<_>) =
                    std::mem::take(&mut groups.waiting_for_rusty)
                        .into_iter()
                        .partition(|(id, _)| *id == window_id);
                groups.waiting_for_rusty = others;
                mine.into_iter().map(|(_, open)| open).collect()
            };
            for open in waiting {
                open(workspace.clone(), window, cx);
            }
        },
        window,
        cx,
    );
}

/// Every group's name, open or pending, in every window.
fn taken_names(cx: &App) -> Vec<String> {
    cx.try_global::<Groups>()
        .map(|groups| {
            groups
                .live
                .iter()
                .map(|group| group.name.clone())
                .chain(groups.pending.iter().map(|saved| saved.name.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// `wanted`, or the unnamed group's name when it is empty, numbered from 2 while another group
/// has it; one more candidate than there are names taken always finds a free one.
fn free_name(wanted: &str, taken: &[String]) -> String {
    let base = if wanted.is_empty() { UNNAMED } else { wanted };
    (1..=taken.len() + 1)
        .map(|count| {
            if count == 1 {
                base.to_string()
            } else {
                format!("{base} {count}")
            }
        })
        .find(|name| !taken.contains(name))
        .unwrap_or_else(|| base.to_string())
}

/// Renames the group `id` to `name`, unless `name` is empty; a renamed Home group is Home no
/// more.
pub(crate) fn rename(id: Uuid, name: &str, cx: &mut App) {
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    let taken: Vec<String> = cx
        .try_global::<Groups>()
        .map(|groups| {
            groups
                .live
                .iter()
                .filter(|group| group.id != id)
                .map(|group| group.name.clone())
                .chain(groups.pending.iter().map(|saved| saved.name.clone()))
                .collect()
        })
        .unwrap_or_default();
    let name = free_name(name, &taken);
    if let Some(group) = cx
        .default_global::<Groups>()
        .live
        .iter_mut()
        .find(|group| group.id == id)
    {
        group.home = group.home && name == HOME;
        group.name = name;
    }
    save(cx);
}

/// Folds or unfolds the group `id`.
pub(crate) fn toggle_expanded(id: Uuid, cx: &mut App) {
    if let Some(group) = cx
        .default_global::<Groups>()
        .live
        .iter_mut()
        .find(|group| group.id == id)
    {
        group.expanded = !group.expanded;
    }
    save(cx);
}

/// Drops the group `id`, and any group whose workspace is gone.
pub(crate) fn forget(id: Uuid, cx: &mut App) {
    cx.default_global::<Groups>()
        .live
        .retain(|group| group.id != id && group.workspace.upgrade().is_some());
    keep(cx);
    save(cx);
}

/// Tells Zed which workspaces are groups, which opening a project must not replace.
fn keep(cx: &mut App) {
    let kept = cx
        .try_global::<Groups>()
        .map(|groups| {
            groups
                .live
                .iter()
                .filter_map(|group| group.workspace.upgrade())
                .map(|workspace| workspace.entity_id())
                .collect()
        })
        .unwrap_or_default();
    cx.set_global(MarleyKeptWorkspaces(kept));
}

/// Opens the prompt for a group's name over `workspace`: a new group's, or `rename`'s new one.
pub(crate) fn prompt(
    workspace: &Entity<Workspace>,
    multi_workspace: WeakEntity<MultiWorkspace>,
    rename: Option<(Uuid, String)>,
    window: &mut Window,
    cx: &mut App,
) {
    workspace.update(cx, |workspace, cx| {
        workspace.toggle_modal(window, cx, |window, cx| {
            GroupNamePrompt::new(multi_workspace, rename, window, cx)
        });
    });
}

/// A group's name, for New Group… or Rename Group….
pub(crate) struct GroupNamePrompt {
    editor: Entity<Editor>,
    multi_workspace: WeakEntity<MultiWorkspace>,
    /// The group renamed; `None` for a new group.
    renaming: Option<Uuid>,
}

impl GroupNamePrompt {
    fn new(
        multi_workspace: WeakEntity<MultiWorkspace>,
        rename: Option<(Uuid, String)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            match &rename {
                Some((_, name)) => {
                    editor.set_text(name.as_str(), window, cx);
                    editor.select_all(&editor::actions::SelectAll, window, cx);
                }
                None => editor.set_placeholder_text("A name for the group", window, cx),
            }
            editor
        });
        Self {
            editor,
            multi_workspace,
            renaming: rename.map(|(id, _)| id),
        }
    }

    fn confirm(&mut self, _: &MakeGroup, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.editor.read(cx).text(cx);
        match self.renaming {
            Some(id) => rename(id, &name, cx),
            None => {
                if let Some(multi_workspace) = self.multi_workspace.upgrade() {
                    make(
                        &multi_workspace,
                        &name,
                        GroupKind::Named,
                        |_, _, _| {},
                        window,
                        cx,
                    );
                }
            }
        }
        cx.emit(DismissEvent);
    }
}

impl ModalView for GroupNamePrompt {}

impl EventEmitter<DismissEvent> for GroupNamePrompt {}

impl Focusable for GroupNamePrompt {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.focus_handle(cx)
    }
}

impl Render for GroupNamePrompt {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let (title, note) = if self.renaming.is_some() {
            ("Rename Group", "Enter renames it.")
        } else {
            (
                "New Group",
                "A group with no folder: its terminals and agents start in your home folder.",
            )
        };
        v_flex()
            .key_context("MarleyGroupName")
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .w(rems(28.))
            .elevation_3(cx)
            .p_3()
            .gap_2()
            .child(Headline::new(title).size(HeadlineSize::Small))
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.editor_background)
                    .child(self.editor.clone()),
            )
            .child(Label::new(note).size(LabelSize::Small).color(Color::Muted))
    }
}
