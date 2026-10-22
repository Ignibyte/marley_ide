//! A guided review (#733).
//!
//! An agent splits a change into ordered sections and sends them with `review_guide_open`; the
//! Guide tab draws the guide's title and summary, "K of N reviewed", and each section with its
//! number, title, Markdown summary (HTML not parsed), a chip per file and a Reviewed box. A ticked
//! section folds to its title. A chip shows its file in the project diff when the file has changes,
//! else opens it at its first line. A guide with the same title in the same workspace replaces the
//! open tab's sections, its ticks cleared. Ticks live with the tab.

use std::path::{Path, PathBuf};

use editor::Editor;
use git_ui::project_diff::ProjectDiff;
use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, SharedString,
    WeakEntity, Window,
};
use language::Point;
use markdown::{Markdown, MarkdownElement, MarkdownFont, MarkdownStyle};
use project::ProjectPath;
use ui::{Checkbox, ToggleState, prelude::*};
use workspace::item::{Item, ItemEvent};
use workspace::{OpenOptions, OpenVisible, Workspace};

/// One file of a section, and the lines the guide points at.
#[derive(Clone, Debug)]
pub(crate) struct GuideFile {
    pub(crate) path: PathBuf,
    pub(crate) first_line: Option<u32>,
    pub(crate) last_line: Option<u32>,
}

/// One section as the call gave it.
#[derive(Clone, Debug)]
pub(crate) struct GuideSection {
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) files: Vec<GuideFile>,
}

/// A guide as the call gave it.
#[derive(Clone, Debug)]
pub(crate) struct Guide {
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) sections: Vec<GuideSection>,
}

/// A section as the tab draws it.
struct Section {
    title: SharedString,
    summary: Entity<Markdown>,
    files: Vec<(GuideFile, SharedString)>,
    reviewed: bool,
}

/// The Guide tab.
pub(crate) struct GuideTab {
    title: SharedString,
    summary: Option<Entity<Markdown>>,
    sections: Vec<Section>,
    workspace: WeakEntity<Workspace>,
    /// The project's folder, which the chips' paths are shown relative to.
    root: Option<PathBuf>,
    focus_handle: FocusHandle,
}

/// Opens `guide` in `workspace`, or replaces the sections of its open tab with the same title;
/// whether it replaced one.
pub(crate) fn open(
    workspace: &mut Workspace,
    guide: Guide,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> bool {
    let open = workspace
        .items_of_type::<GuideTab>(cx)
        .find(|tab| tab.read(cx).title.as_ref() == guide.title);
    if let Some(tab) = open {
        tab.update(cx, |tab, cx| tab.set_guide(guide, cx));
        workspace.activate_item(&tab, true, false, window, cx);
        return true;
    }
    let root = workspace
        .project()
        .read(cx)
        .visible_worktrees(cx)
        .next()
        .map(|worktree| worktree.read(cx).abs_path().to_path_buf());
    let handle = workspace.weak_handle();
    let tab = cx.new(|cx| {
        let mut tab = GuideTab {
            title: SharedString::from(guide.title.clone()),
            summary: None,
            sections: Vec::new(),
            workspace: handle,
            root,
            focus_handle: cx.focus_handle(),
        };
        tab.set_guide(guide, cx);
        tab
    });
    workspace.add_item_to_center(Box::new(tab), window, cx);
    false
}

impl GuideTab {
    /// Puts `guide`'s sections in the tab, every tick cleared.
    fn set_guide(&mut self, guide: Guide, cx: &mut Context<Self>) {
        self.title = SharedString::from(guide.title);
        self.summary = (!guide.summary.trim().is_empty())
            .then(|| cx.new(|cx| Markdown::new(guide.summary.into(), None, None, cx)));
        let root = self.root.clone();
        self.sections = guide
            .sections
            .into_iter()
            .map(|section| Section {
                title: SharedString::from(section.title),
                summary: cx.new(|cx| Markdown::new(section.summary.into(), None, None, cx)),
                files: section
                    .files
                    .into_iter()
                    .map(|file| {
                        let label = chip_label(&file, root.as_deref());
                        (file, label)
                    })
                    .collect(),
                reviewed: false,
            })
            .collect();
        cx.notify();
    }

    fn set_reviewed(&mut self, index: usize, reviewed: bool, cx: &mut Context<Self>) {
        if let Some(section) = self.sections.get_mut(index) {
            section.reviewed = reviewed;
            cx.notify();
        }
    }

    /// Shows the file in the project diff when it has changes there, else opens it at its first
    /// line, once this tab's update is over: adding an item makes the pane read every item, this tab
    /// among them (PR-claude-defer-a-pane-change-out-of-an-items-own-event-001).
    fn open_file(&self, file: GuideFile, window: &Window, cx: &mut Context<Self>) {
        let workspace = self.workspace.clone();
        window.defer(cx, move |window, cx| {
            open_file_in(&workspace, &file, window, cx);
        });
    }
}

/// `open_file`'s work, with no entity leased.
fn open_file_in(
    workspace: &WeakEntity<Workspace>,
    file: &GuideFile,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(workspace) = workspace.upgrade() else {
        return;
    };
    if let Some(project_path) = in_project_diff(&workspace, &file.path, cx) {
        workspace.update(cx, |workspace, cx| {
            ProjectDiff::deploy_at_project_path(workspace, project_path, window, cx);
        });
        return;
    }
    let line = file.first_line.unwrap_or(1).saturating_sub(1);
    let options = OpenOptions {
        visible: Some(OpenVisible::None),
        focus: Some(true),
        ..OpenOptions::default()
    };
    let opening = workspace.update(cx, |workspace, cx| {
        workspace.open_abs_path(file.path.clone(), options, window, cx)
    });
    window
        .spawn(cx, async move |cx| {
            let item = opening.await?;
            let editor = cx
                .update(|_, cx| item.act_as::<Editor>(cx))?
                .ok_or_else(|| anyhow::anyhow!("the file did not open in an editor"))?;
            editor.update_in(cx, |editor, window, cx| {
                editor.go_to_singleton_buffer_point(Point::new(line, 0), window, cx);
            })?;
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
}

/// The file's path in its project when its repository reports a change to it, which puts it in the
/// project diff.
fn in_project_diff(workspace: &Entity<Workspace>, path: &Path, cx: &App) -> Option<ProjectPath> {
    let project = workspace.read(cx).project().read(cx);
    let project_path = project.find_project_path(path, cx)?;
    let (repository, repo_path) = project
        .git_store()
        .read(cx)
        .repository_and_path_for_project_path(&project_path, cx)?;
    repository.read(cx).status_for_path(&repo_path)?;
    Some(project_path)
}

/// A chip's words: the path relative to the project's folder, and its lines.
fn chip_label(file: &GuideFile, root: Option<&Path>) -> SharedString {
    let shown = root
        .and_then(|root| file.path.strip_prefix(root).ok())
        .unwrap_or(&file.path)
        .display()
        .to_string();
    let lines = match (file.first_line, file.last_line) {
        (Some(first), Some(last)) if last > first => format!(":{first}-{last}"),
        (Some(first), _) => format!(":{first}"),
        _ => String::new(),
    };
    SharedString::from(format!("{shown}{lines}"))
}

impl Focusable for GuideTab {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for GuideTab {}

impl Render for GuideTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let style = MarkdownStyle::themed(MarkdownFont::Preview, window, cx);
        let reviewed = self
            .sections
            .iter()
            .filter(|section| section.reviewed)
            .count();
        v_flex()
            .id("marley-guide")
            .key_context("MarleyGuide")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .bg(colors.editor_background)
            // The board's guide header (#888): whose guide it is, its title, and how far the
            // review has come.
            .child(
                Label::new("Guide · from the agent")
                    .size(LabelSize::XSmall)
                    .color(Color::Accent),
            )
            .child(Headline::new(self.title.clone()).size(HeadlineSize::Medium))
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        Label::new(format!("{reviewed} of {} reviewed", self.sections.len()))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(progress_bar(reviewed, self.sections.len(), cx)),
            )
            .when_some(self.summary.clone(), |guide, summary| {
                guide.child(MarkdownElement::new(summary, style.clone()))
            })
            .children(self.sections.iter().enumerate().map(|(index, section)| {
                // The section's number and title, then Reviewed at its right (#888).
                let header = h_flex()
                    .gap_2()
                    .child(
                        div().flex_1().min_w_0().child(
                            Label::new(format!("{}. {}", index + 1, section.title))
                                .size(LabelSize::Large)
                                .color(if section.reviewed {
                                    Color::Muted
                                } else {
                                    Color::Default
                                }),
                        ),
                    )
                    .child(
                        Checkbox::new(
                            ("marley-guide-reviewed", index),
                            ToggleState::from(section.reviewed),
                        )
                        .label("Reviewed")
                        .label_size(LabelSize::Small)
                        .on_click(cx.listener(
                            move |guide, state: &ToggleState, _, cx| {
                                guide.set_reviewed(index, state.selected(), cx);
                            },
                        )),
                    );
                let chips =
                    h_flex()
                        .flex_wrap()
                        .gap_1()
                        .children(section.files.iter().enumerate().map(
                            |(file_index, (file, label))| {
                                let file = file.clone();
                                Button::new(
                                    ("marley-guide-file", index * 1000 + file_index),
                                    label.clone(),
                                )
                                .style(ButtonStyle::Filled)
                                .label_size(LabelSize::Small)
                                .start_icon(Icon::new(IconName::File).size(IconSize::Small))
                                .on_click(cx.listener(
                                    move |guide, _, window, cx| {
                                        guide.open_file(file.clone(), window, cx);
                                    },
                                ))
                            },
                        ));
                v_flex()
                    .gap_2()
                    .p_3()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.elevated_surface_background)
                    .child(header)
                    .when(!section.reviewed, |body| {
                        body.child(MarkdownElement::new(section.summary.clone(), style.clone()))
                            .child(chips)
                    })
            }))
    }
}

/// A 4 px bar whose fill is the share of `total` sections `reviewed` (#888).
fn progress_bar(reviewed: usize, total: usize, cx: &App) -> impl IntoElement {
    let share = match (u16::try_from(reviewed), u16::try_from(total)) {
        (Ok(reviewed), Ok(total)) if total > 0 => f32::from(reviewed) / f32::from(total),
        _ => 0.,
    };
    let colors = cx.theme().colors();
    div()
        .w(px(160.))
        .h(px(4.))
        .rounded_full()
        .bg(colors.element_background)
        .child(
            div()
                .h_full()
                .w(relative(share))
                .rounded_full()
                .bg(colors.text_accent),
        )
}

impl Item for GuideTab {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::from(format!("Guide · {}", self.title))
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ListTodo))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::from(format!(
            "A guided review: {}",
            self.title
        )))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
