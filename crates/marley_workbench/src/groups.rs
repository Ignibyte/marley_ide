//! Projectless groups (#600): named groups in the rail with no folder.
//!
//! Each is a folderless workspace Marley keeps in its window: its terminals and agent CLIs start
//! in the home folder, and its Browser tabs use a Chromium of the group's own. The rail lists them
//! after the window's projects; the window's Home group takes what the rail's empty space makes.

use std::sync::Arc;

use anyhow::Context as _;
use editor::Editor;
use gpui::{
    AnyWindowHandle, App, AppContext as _, Context, DismissEvent, Entity, EntityId, EventEmitter,
    FocusHandle, Focusable, Global, Render, WeakEntity, Window,
};
use ui::{Headline, HeadlineSize, prelude::*};
use uuid::Uuid;
use workspace::{MarleyKeptWorkspaces, ModalView, MultiWorkspace, OpenMode, Workspace};

use crate::MakeGroup;

/// The name of the window's group for what the rail's empty space makes.
const HOME: &str = "Home";

/// The name an unnamed group takes, numbered from its second.
const UNNAMED: &str = "Group";

/// A projectless group.
#[derive(Clone)]
pub(crate) struct Group {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) workspace: WeakEntity<Workspace>,
    /// Its workspace's project, kept so the browser can find the group while its workspace is
    /// being updated, when the workspace cannot be read.
    project: EntityId,
    pub(crate) expanded: bool,
    /// Whether it is its window's Home group.
    pub(crate) home: bool,
}

/// Every window's groups, in the order they were made; the rail observes it.
#[derive(Default)]
pub(crate) struct Groups(Vec<Group>);

impl Global for Groups {}

/// The groups `multi_workspace` holds, each with its workspace, in the order they were made.
pub(crate) fn groups_of(
    multi_workspace: &MultiWorkspace,
    cx: &App,
) -> Vec<(Group, Entity<Workspace>)> {
    let Some(groups) = cx.try_global::<Groups>() else {
        return Vec::new();
    };
    groups
        .0
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

/// The group whose workspace's project is `project`: its id and name, for its browser (#600).
/// It reads no workspace: the browser asks while the group's own workspace is being updated.
pub(crate) fn group_of_project(project: EntityId, cx: &App) -> Option<(Uuid, String)> {
    cx.try_global::<Groups>()?
        .0
        .iter()
        .find(|group| group.project == project)
        .map(|group| (group.id, group.name.clone()))
}

/// Makes a group in `window`, named `name` (the Home group when `home`), and runs `then` with
/// its workspace in the window once the workspace exists.
pub(crate) fn make(
    multi_workspace: &Entity<MultiWorkspace>,
    name: &str,
    home: bool,
    then: impl FnOnce(Entity<Workspace>, &mut Window, &mut App) + 'static,
    window: &Window,
    cx: &mut App,
) {
    let app_state = Arc::clone(multi_workspace.read(cx).workspace().read(cx).app_state());
    let handle = window.window_handle().downcast::<MultiWorkspace>();
    let name = if home {
        HOME.to_string()
    } else {
        name.trim().to_string()
    };
    let opened = Workspace::new_local(Vec::new(), app_state, handle, None, None, OpenMode::Add, cx);
    cx.spawn(async move |cx| {
        let opened = opened.await.context("making a projectless group")?;
        let workspace = opened.workspace;
        cx.update(|cx| {
            let taken = taken_names(cx);
            let project = workspace.read(cx).project().entity_id();
            cx.default_global::<Groups>().0.push(Group {
                id: Uuid::new_v4(),
                name: free_name(&name, &taken),
                workspace: workspace.downgrade(),
                project,
                expanded: true,
                home,
            });
            keep(cx);
        });
        // Through the window alone: its `MultiWorkspace` stays free for `then`, which the rail's
        // Home path updates to show the group.
        AnyWindowHandle::from(opened.window)
            .update(cx, |_, window, cx| then(workspace, window, cx))
            .context("the window closed while its group was made")
    })
    .detach_and_log_err(cx);
}

/// Every group's name, in every window.
fn taken_names(cx: &App) -> Vec<String> {
    cx.try_global::<Groups>()
        .map(|groups| groups.0.iter().map(|group| group.name.clone()).collect())
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
                .0
                .iter()
                .filter(|group| group.id != id)
                .map(|group| group.name.clone())
                .collect()
        })
        .unwrap_or_default();
    let name = free_name(name, &taken);
    if let Some(group) = cx
        .default_global::<Groups>()
        .0
        .iter_mut()
        .find(|group| group.id == id)
    {
        group.home = group.home && name == HOME;
        group.name = name;
    }
}

/// Folds or unfolds the group `id`.
pub(crate) fn toggle_expanded(id: Uuid, cx: &mut App) {
    if let Some(group) = cx
        .default_global::<Groups>()
        .0
        .iter_mut()
        .find(|group| group.id == id)
    {
        group.expanded = !group.expanded;
    }
}

/// Drops the group `id`, and any group whose workspace is gone.
pub(crate) fn forget(id: Uuid, cx: &mut App) {
    cx.default_global::<Groups>()
        .0
        .retain(|group| group.id != id && group.workspace.upgrade().is_some());
    keep(cx);
}

/// Tells Zed which workspaces are groups, which opening a project must not replace.
fn keep(cx: &mut App) {
    let kept = cx
        .try_global::<Groups>()
        .map(|groups| {
            groups
                .0
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
                    make(&multi_workspace, &name, false, |_, _, _| {}, window, cx);
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
