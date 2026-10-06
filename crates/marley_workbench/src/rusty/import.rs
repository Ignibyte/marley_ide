//! Importing an Obsidian vault into Rusty's brain from the palette (#663).
//!
//! A folder on this machine, then Rusty's plan for it: what comes in, what is skipped as already
//! in the brain, the links that will not resolve and the bookmarks, each list in full. Import has
//! Rusty bring it in and the form gives its report. Rusty reads the vault and never writes it,
//! and an import that fails part way leaves nothing behind.

use std::path::Path;
use std::sync::Arc;

use gpui::{
    App, Context, DismissEvent, EventEmitter, FocusHandle, Focusable, PathPromptOptions, Render,
    ScrollHandle, SharedString, Task, WeakEntity, Window, actions,
};
use marley_rusty::capture::{
    BRAIN_IMPORT, BRAIN_IMPORT_PLAN, IMPORT_DEADLINE, IMPORT_PLAN_DEADLINE, ImportPlan,
    ImportReport, import_arguments, plan_from_answer, report_from_answer,
};
use project::DirectoryLister;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::{DismissDecision, ModalView, Workspace};

actions!(
    rusty,
    [
        /// Imports an Obsidian vault into Rusty's brain, after showing what it would do.
        #[derive(Eq)]
        ImportVault,
    ]
);

/// Registers [`ImportVault`] on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &ImportVault, window, cx| {
            if super::capture::ready(workspace, cx) {
                choose_folder(workspace, window, cx);
            }
        });
    })
    .detach();
}

/// Asks for the vault's folder, then opens the form on it.
fn choose_folder(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    // Rusty reads its own machine's disk, so the folder is this machine's, a remote project's
    // included.
    let lister = DirectoryLister::Local(
        workspace.project().clone(),
        Arc::clone(&workspace.app_state().fs),
    );
    let paths = workspace.prompt_for_open_path(
        PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(SharedString::new_static("Import")),
        },
        lister,
        window,
        cx,
    );
    cx.spawn_in(window, async move |workspace, cx| {
        let Some(folder) = paths
            .await
            .log_err()
            .flatten()
            .and_then(|paths| paths.into_iter().next())
        else {
            return;
        };
        workspace
            .update_in(cx, |workspace, window, cx| {
                let weak = cx.weak_entity();
                workspace.toggle_modal(window, cx, |window, cx| {
                    ImportForm::new(&folder, weak, window, cx)
                });
            })
            .log_err();
    })
    .detach();
}

/// Where the import stands.
enum Stage {
    Reading,
    Plan(ImportPlan),
    Importing(ImportPlan),
    Done(ImportReport),
    /// Rusty's words for a plan or an import it could not make.
    Failed(SharedString),
}

/// The form's width in rems: the lists' paths need more than a capture's field.
const FORM_WIDTH: f32 = 40.;

pub(super) struct ImportForm {
    workspace: WeakEntity<Workspace>,
    path: String,
    stage: Stage,
    task: Option<Task<()>>,
    focus_handle: FocusHandle,
    scroll: ScrollHandle,
}

impl ImportForm {
    fn new(
        folder: &Path,
        workspace: WeakEntity<Workspace>,
        window: &Window,
        cx: &Context<Self>,
    ) -> Self {
        let path = folder.to_string_lossy().into_owned();
        let asking = super::call_tool_within(
            BRAIN_IMPORT_PLAN,
            import_arguments(&path),
            IMPORT_PLAN_DEADLINE,
            cx,
        );
        let task = cx.spawn_in(window, async move |this, cx| {
            let plan = asking.await.and_then(|text| {
                plan_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_IMPORT_PLAN}'s answer did not parse: {error}"))
            });
            this.update(cx, |this, cx| {
                this.task = None;
                this.stage = match plan {
                    Ok(plan) => Stage::Plan(plan),
                    Err(error) => Stage::Failed(first_line(&error)),
                };
                cx.notify();
            })
            .log_err();
        });
        Self {
            workspace,
            path,
            stage: Stage::Reading,
            task: Some(task),
            focus_handle: cx.focus_handle(),
            scroll: ScrollHandle::new(),
        }
    }

    /// Runs the import the plan shows, when it brings anything in.
    fn import(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.task.is_some() {
            return;
        }
        let plan = match &self.stage {
            Stage::Plan(plan) if plan.brings_anything() => plan.clone(),
            _ => return,
        };
        let asking = super::call_tool_within(
            BRAIN_IMPORT,
            import_arguments(&self.path),
            IMPORT_DEADLINE,
            cx,
        );
        self.stage = Stage::Importing(plan);
        self.task = Some(cx.spawn_in(window, async move |this, cx| {
            let report = asking.await.and_then(|text| {
                report_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_IMPORT}'s answer did not parse: {error}"))
            });
            this.update(cx, |this, cx| {
                this.task = None;
                this.stage = match report {
                    Ok(report) => {
                        // The service connection announces nothing (AD-662).
                        super::reread_vault(cx);
                        super::favourites::read(cx);
                        Stage::Done(report)
                    }
                    Err(error) => Stage::Failed(first_line(&error)),
                };
                cx.notify();
            })
            .log_err();
        }));
        cx.notify();
    }

    /// Enter: Import on a plan, Close on a report.
    fn confirm(&mut self, window: &Window, cx: &mut Context<Self>) {
        match self.stage {
            Stage::Plan(_) => self.import(window, cx),
            Stage::Done(_) | Stage::Failed(_) => cx.emit(DismissEvent),
            Stage::Reading | Stage::Importing(_) => {}
        }
    }

    fn open_report(&self, window: &Window, cx: &mut Context<Self>) {
        if let Stage::Done(report) = &self.stage
            && !report.report_slug.is_empty()
        {
            super::page::open_later(
                self.workspace.clone(),
                report.report_slug.clone(),
                false,
                true,
                window,
                cx,
            );
            cx.emit(DismissEvent);
        }
    }

    fn render_sentence(&self) -> AnyElement {
        let (text, color) = match &self.stage {
            Stage::Reading => (SharedString::new_static("Reading the vault…"), Color::Muted),
            Stage::Plan(plan) | Stage::Importing(plan) => (plan.summary().into(), Color::Default),
            Stage::Done(report) => (report.summary().into(), Color::Default),
            Stage::Failed(error) => (error.clone(), Color::Error),
        };
        Label::new(text).color(color).into_any_element()
    }

    /// The plan's lists, each under its heading, in a block that scrolls.
    fn render_details(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let plan = match &self.stage {
            Stage::Plan(plan) | Stage::Importing(plan) => plan,
            Stage::Done(report) => &report.plan,
            Stage::Reading | Stage::Failed(_) => return None,
        };
        let sections = plan.details().into_iter().map(|(heading, lines)| {
            let lines = if lines.is_empty() {
                vec![SharedString::new_static("none")]
            } else {
                lines.into_iter().map(SharedString::from).collect()
            };
            v_flex()
                .child(
                    Label::new(heading)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .children(lines.into_iter().map(|line| {
                    div()
                        .ml_2()
                        .child(Label::new(line).size(LabelSize::Small).buffer_font(cx))
                }))
        });
        Some(
            v_flex()
                .id("rusty-import-details")
                .max_h(rems(18.))
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .gap_2()
                .p_2()
                .rounded_sm()
                .border_1()
                .border_color(cx.theme().colors().border)
                .children(sections)
                .into_any_element(),
        )
    }

    fn render_buttons(&self, cx: &Context<Self>) -> impl IntoElement {
        let close = |label: &'static str, style: ButtonStyle| {
            Button::new("rusty-import-close", label)
                .style(style)
                .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent)))
        };
        let buttons = h_flex().gap_2().justify_end();
        match &self.stage {
            Stage::Reading | Stage::Failed(_) => buttons.child(close("Close", ButtonStyle::Filled)),
            Stage::Plan(plan) => buttons.child(close("Cancel", ButtonStyle::Subtle)).child(
                Button::new("rusty-import-run", "Import")
                    .style(ButtonStyle::Filled)
                    .disabled(!plan.brings_anything())
                    .on_click(cx.listener(|this, _, window, cx| this.import(window, cx))),
            ),
            Stage::Importing(_) => buttons.child(
                Button::new("rusty-import-run", "Importing…")
                    .style(ButtonStyle::Filled)
                    .disabled(true),
            ),
            Stage::Done(report) => buttons
                .when(!report.report_slug.is_empty(), |buttons| {
                    buttons.child(
                        Button::new("rusty-import-report", "Open Report")
                            .style(ButtonStyle::Subtle)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.open_report(window, cx)),
                            ),
                    )
                })
                .child(close("Close", ButtonStyle::Filled)),
        }
    }
}

fn first_line(error: &str) -> SharedString {
    error.lines().next().unwrap_or_default().to_string().into()
}

impl ModalView for ImportForm {
    /// The form stays while Rusty imports, so its report has somewhere to show.
    fn on_before_dismiss(&mut self, _: &mut Window, _: &mut Context<Self>) -> DismissDecision {
        DismissDecision::Dismiss(!matches!(self.stage, Stage::Importing(_)))
    }
}

impl EventEmitter<DismissEvent> for ImportForm {}

impl Focusable for ImportForm {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ImportForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("RustyImport menu")
            .track_focus(&self.focus_handle)
            .w(rems(FORM_WIDTH))
            .p_3()
            .gap_2()
            .elevation_3(cx)
            .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| this.confirm(window, cx)))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .child(Label::new("Import an Obsidian Vault"))
            .child(
                // The folder's own name is at the path's end.
                Label::new(self.path.clone())
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .truncate_start(),
            )
            .child(self.render_sentence())
            .children(self.render_details(cx))
            .child(self.render_buttons(cx))
    }
}
