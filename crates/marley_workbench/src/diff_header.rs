//! The diff review's header (#893).
//!
//! At the left of the toolbar over Zed's project diff: what the diff holds (the branch, the files,
//! the lines added and removed, how many files are viewed) and Copy Notes, beside Zed's own Send
//! Review to Agent at the right.

use editor::Editor;
use git_ui::project_diff::ProjectDiff;
use gpui::{Context, Entity, EventEmitter, Subscription, WeakEntity, Window};
use project::Project;
use project::git_store::Repository;
use ui::{Tooltip, prelude::*};
use workspace::item::ItemHandle;
use workspace::{ToolbarItemEvent, ToolbarItemLocation, ToolbarItemView, Workspace};

/// Adds Marley's items to a pane's toolbar; Zed's `initialize_pane` calls it.
pub fn add_toolbar_items(
    toolbar: &mut workspace::Toolbar,
    workspace: &Workspace,
    window: &mut Window,
    cx: &mut Context<workspace::Toolbar>,
) {
    let header = cx.new(|_| DiffHeader::new(workspace.project().downgrade()));
    toolbar.add_item(header, window, cx);
}

/// The header, while the pane's active item is a project diff.
pub(crate) struct DiffHeader {
    /// The workspace's project, read rather than the workspace: the toolbar hears of the active
    /// item from inside the workspace's update.
    project: WeakEntity<Project>,
    /// The project diff, while it is the active item.
    diff: Option<WeakEntity<ProjectDiff>>,
    observed: Vec<Subscription>,
}

/// What the header says of the diff.
struct Summary {
    branch: Option<SharedString>,
    files: usize,
    added: u32,
    deleted: u32,
    viewed: usize,
}

impl DiffHeader {
    const fn new(project: WeakEntity<Project>) -> Self {
        Self {
            project,
            diff: None,
            observed: Vec::new(),
        }
    }

    fn repository(&self, cx: &App) -> Option<Entity<Repository>> {
        self.project.upgrade()?.read(cx).active_repository(cx)
    }

    /// The diff's own counts, as Zed's toolbar at the right counts its lines.
    fn summary(&self, diff: &Entity<ProjectDiff>, editor: &Entity<Editor>, cx: &App) -> Summary {
        let branch = self.repository(cx).and_then(|repository| {
            repository
                .read(cx)
                .branch
                .as_ref()
                .map(|branch| SharedString::from(branch.name().to_string()))
        });
        let (added, deleted) = diff.read(cx).calculate_changed_lines(cx);
        Summary {
            branch,
            files: diff.read(cx).multibuffer(cx).read(cx).all_buffers().len(),
            added,
            deleted,
            viewed: crate::viewed_marks::viewed(editor.read(cx), cx),
        }
    }
}

impl EventEmitter<ToolbarItemEvent> for DiffHeader {}

impl ToolbarItemView for DiffHeader {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn ItemHandle>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> ToolbarItemLocation {
        let diff = active_pane_item.and_then(|item| item.act_as::<ProjectDiff>(cx));
        self.observed.clear();
        let Some(diff) = diff else {
            self.diff = None;
            return ToolbarItemLocation::Hidden;
        };
        let editor = diff.read(cx).editor(cx).read(cx).rhs_editor().clone();
        // The ticks and the notes redraw the editor; the diff's files and the branch redraw the
        // diff and the repository.
        self.observed
            .push(cx.observe(&editor, |_, _, cx| cx.notify()));
        self.observed
            .push(cx.observe(&diff, |_, _, cx| cx.notify()));
        if let Some(repository) = self.repository(cx) {
            self.observed
                .push(cx.observe(&repository, |_, _, cx| cx.notify()));
        }
        self.diff = Some(diff.downgrade());
        ToolbarItemLocation::PrimaryLeft
    }
}

impl Render for DiffHeader {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(diff) = self.diff.as_ref().and_then(WeakEntity::upgrade) else {
            return div().into_any_element();
        };
        let editor = diff.read(cx).editor(cx).read(cx).rhs_editor().clone();
        let Summary {
            branch,
            files,
            added,
            deleted,
            viewed,
        } = self.summary(&diff, &editor, cx);
        let unsent = editor.read(cx).unsent_review_notes(cx).len();
        let files_word = if files == 1 { "file" } else { "files" };
        h_flex()
            .gap_2()
            .child(Label::new("Uncommitted changes").size(LabelSize::Small))
            .children(branch.map(|branch| {
                Label::new(branch)
                    .size(LabelSize::Small)
                    .buffer_font(cx)
                    .color(Color::Muted)
            }))
            .child(
                Label::new(format!("· {files} {files_word} ·"))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Label::new(format!("+{added}"))
                    .size(LabelSize::Small)
                    .color(Color::Created),
            )
            .child(
                Label::new(format!("−{deleted}"))
                    .size(LabelSize::Small)
                    .color(Color::Deleted),
            )
            .child(
                Label::new(format!("· {viewed} of {files} viewed"))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Button::new("marley-diff-copy-notes", "Copy Notes")
                    .label_size(LabelSize::Small)
                    .disabled(unsent == 0)
                    .tooltip(Tooltip::text(if unsent == 0 {
                        "No note to copy yet"
                    } else {
                        "Copy the notes not sent yet, as the agent would get them"
                    }))
                    .on_click(move |_, _, cx| crate::review_notes::copy_notes(&editor, cx)),
            )
            .into_any_element()
    }
}
