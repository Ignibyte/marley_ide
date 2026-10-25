//! Problems in the drawer as a list (#922): one row per primary error or warning of the project,
//! filtered by a field, each opening its place on a click or asking an agent to fix it.
//!
//! The rows are read as Zed's diagnostics editor reads them: the project's diagnostic summaries
//! name the files, and each file's buffer gives its diagnostics. They are read again when the
//! project's diagnostics change. Which severities count follows `diagnostics.include_warnings`, so
//! the list agrees with the Problems pill. The panel's place is `debugger.dock`, so Problems and
//! Debug move as one drawer (D3).

use std::path::{Path, PathBuf};

use ::fs::Fs;
use dap::debugger_settings::DebuggerSettings;
use editor::{Editor, EditorEvent};
use gpui::{
    Action, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, Pixels,
    Render, Subscription, Task, WeakEntity, Window, px,
};
use language::Point;
use project::project_settings::ProjectSettings;
use project::{Project, ProjectPath};
use settings::Settings as _;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::dock::{DockPosition, Panel, PanelEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, ToggleZoom, Workspace};

use crate::ToggleProblems;

const ACTIVATION_PRIORITY: u32 = 23;

/// One problem: where it is and what it says.
#[derive(Clone)]
struct Problem {
    path: ProjectPath,
    abs_path: Option<PathBuf>,
    file_name: SharedString,
    /// From 1.
    line: u32,
    /// From 1.
    column: u32,
    /// The message's first line.
    message: SharedString,
    error: bool,
}

/// The project's problems as rows, under a filter field (#922).
pub struct ProblemsPanel {
    project: Entity<Project>,
    workspace: WeakEntity<Workspace>,
    filter: Entity<Editor>,
    problems: Vec<Problem>,
    refresh: Task<()>,
    zoomed: bool,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for ProblemsPanel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProblemsPanel")
            .finish_non_exhaustive()
    }
}

impl ProblemsPanel {
    pub(crate) fn new(
        project: Entity<Project>,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let filter = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Filter problems", window, cx);
            editor
        });
        let subscriptions = vec![
            cx.subscribe(&filter, |_, _, event: &EditorEvent, cx| {
                if matches!(event, EditorEvent::BufferEdited) {
                    cx.notify();
                }
            }),
            cx.subscribe(&project, |panel, _, event, cx| {
                if matches!(event, project::Event::DiagnosticsUpdated { .. }) {
                    panel.refresh(cx);
                }
            }),
        ];
        let mut panel = Self {
            project,
            workspace,
            filter,
            problems: Vec::new(),
            refresh: Task::ready(()),
            zoomed: false,
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        panel.refresh(cx);
        panel
    }

    /// The panel's project.
    pub(crate) const fn project(&self) -> &Entity<Project> {
        &self.project
    }

    /// The project's errors and warnings, the Problems pill's count, and whether any is an error.
    pub(crate) fn count(&self, cx: &App) -> (usize, bool) {
        let summary = self.project.read(cx).diagnostic_summary(false, cx);
        (
            summary.error_count + summary.warning_count,
            summary.error_count > 0,
        )
    }

    /// Reads the rows again from the files the summaries name.
    fn refresh(&mut self, cx: &Context<Self>) {
        let include_warnings = ProjectSettings::get_global(cx).diagnostics.include_warnings;
        let mut paths: Vec<ProjectPath> = Vec::new();
        for (path, _, summary) in self.project.read(cx).diagnostic_summaries(false, cx) {
            let counts = summary.error_count > 0 || (include_warnings && summary.warning_count > 0);
            if counts && !paths.contains(&path) {
                paths.push(path);
            }
        }
        let project = self.project.clone();
        self.refresh = cx.spawn(async move |panel, cx| {
            let mut problems = Vec::new();
            for path in paths {
                let opening =
                    project.update(cx, |project, cx| project.open_buffer(path.clone(), cx));
                let Some(buffer) = opening.await.log_err() else {
                    continue;
                };
                let found = cx.update(|cx| {
                    let abs_path = project.read(cx).absolute_path(&path, cx);
                    problems_in(
                        &buffer.read(cx).snapshot(),
                        &path,
                        abs_path.as_deref(),
                        include_warnings,
                    )
                });
                problems.extend(found);
            }
            panel
                .update(cx, |panel, cx| {
                    panel.problems = problems;
                    cx.notify();
                })
                .log_err();
        });
    }

    /// The rows the filter keeps, by message or file name, case aside.
    fn shown(&self, cx: &App) -> Vec<Problem> {
        let filter = self.filter.read(cx).text(cx).trim().to_lowercase();
        self.problems
            .iter()
            .filter(|problem| {
                filter.is_empty()
                    || problem.message.to_lowercase().contains(&filter)
                    || problem.file_name.to_lowercase().contains(&filter)
            })
            .cloned()
            .collect()
    }

    fn toggle_zoom(&mut self, _: &ToggleZoom, window: &mut Window, cx: &mut Context<Self>) {
        if self.zoomed {
            cx.emit(PanelEvent::ZoomOut);
        } else {
            if !self.focus.contains_focused(window, cx) {
                window.focus(&self.focus, cx);
            }
            cx.emit(PanelEvent::ZoomIn);
        }
    }

    fn render_row(&self, index: usize, problem: Problem, cx: &App) -> AnyElement {
        let colors = cx.theme().colors();
        let status = cx.theme().status();
        let place = format!("{}:{}", problem.file_name, problem.line);
        // Plain closures over weak handles: opening or asking updates the workspace, which reads
        // its panels, this one among them (PR-claude-701).
        let (open_workspace, ask_workspace) = (self.workspace.clone(), self.workspace.clone());
        let (open_problem, ask_problem) = (problem.clone(), problem.clone());
        h_flex()
            .id(("marley-problem", index))
            .w_full()
            .px_3()
            .py_1()
            .gap_2()
            .cursor_pointer()
            .hover(|style| style.bg(colors.ghost_element_hover))
            .child(
                div()
                    .flex_none()
                    .size(px(8.))
                    .rounded_full()
                    .bg(if problem.error {
                        status.error
                    } else {
                        status.warning
                    }),
            )
            .child(
                div().flex_1().min_w_0().child(
                    Label::new(problem.message)
                        .size(LabelSize::Small)
                        .truncate(),
                ),
            )
            .child(
                Label::new(place)
                    .size(LabelSize::XSmall)
                    .buffer_font(cx)
                    .color(Color::Muted),
            )
            .child(
                h_flex()
                    .id(("marley-problem-ask", index))
                    .flex_none()
                    .h(px(20.))
                    .px_2()
                    .rounded_full()
                    .border_1()
                    .border_color(colors.border)
                    .hover(|style| style.bg(colors.element_hover))
                    .child(
                        Label::new("Ask an agent to fix")
                            .size(LabelSize::XSmall)
                            .color(Color::Accent),
                    )
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        ask(&ask_workspace, &ask_problem, window, cx);
                    }),
            )
            .on_click(move |_, window, cx| open(&open_workspace, &open_problem, window, cx))
            .into_any_element()
    }
}

/// `snapshot`'s primary errors, and warnings while `include_warnings`, as rows.
fn problems_in(
    snapshot: &language::BufferSnapshot,
    path: &ProjectPath,
    abs_path: Option<&Path>,
    include_warnings: bool,
) -> Vec<Problem> {
    let worst = if include_warnings {
        lsp::DiagnosticSeverity::WARNING
    } else {
        lsp::DiagnosticSeverity::ERROR
    };
    let file_name: SharedString = path
        .path
        .file_name()
        .map_or_else(|| path.path.as_unix_str().to_string(), str::to_string)
        .into();
    snapshot
        .diagnostics_in_range::<_, Point>(Point::zero()..snapshot.max_point(), false)
        .filter(|entry| entry.diagnostic.is_primary && entry.diagnostic.severity <= worst)
        .map(|entry| Problem {
            path: path.clone(),
            abs_path: abs_path.map(Path::to_path_buf),
            file_name: file_name.clone(),
            line: entry.range.start.row + 1,
            column: entry.range.start.column + 1,
            message: entry
                .diagnostic
                .message
                .as_str()
                .lines()
                .next()
                .unwrap_or_default()
                .to_string()
                .into(),
            error: entry.diagnostic.severity == lsp::DiagnosticSeverity::ERROR,
        })
        .collect()
}

/// Opens `problem`'s file with the caret at its place (#922).
fn open(workspace: &WeakEntity<Workspace>, problem: &Problem, window: &mut Window, cx: &mut App) {
    let Some(workspace) = workspace.upgrade() else {
        return;
    };
    let point = Point::new(problem.line - 1, problem.column - 1);
    let opening = workspace.update(cx, |workspace, cx| {
        workspace.open_path(problem.path.clone(), None, true, window, cx)
    });
    window
        .spawn(cx, async move |cx| {
            let item = opening.await?;
            if let Some(editor) = cx.update(|_, cx| item.act_as::<Editor>(cx))? {
                editor.update_in(cx, |editor, window, cx| {
                    editor.go_to_singleton_buffer_point(point, window, cx);
                })?;
            }
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
}

/// Asks the window's agent to fix `problem`, as #899's key does; what stops it shows as a toast.
fn ask(workspace: &WeakEntity<Workspace>, problem: &Problem, window: &mut Window, cx: &mut App) {
    let Some(path) = problem.abs_path.clone() else {
        return;
    };
    let (line, message) = (problem.line, problem.message.to_string());
    workspace
        .update(cx, |workspace, cx| {
            if let Err(why) = crate::send_selection::ask_to_fix_at(
                workspace,
                path,
                line,
                Some(message),
                window,
                cx,
            ) {
                workspace.show_toast(
                    Toast::new(NotificationId::unique::<ProblemsPanel>(), why),
                    cx,
                );
            }
        })
        .log_err();
}

impl EventEmitter<PanelEvent> for ProblemsPanel {}

impl Focusable for ProblemsPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for ProblemsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let shown = self.shown(cx);
        let empty = shown.is_empty();
        v_flex()
            .debug_selector(|| "marley-problems".into())
            .key_context("MarleyProblems")
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::toggle_zoom))
            .size_full()
            // Zoomed, the panel draws outside its dock and its strip, so it carries its own
            // Restore (#922).
            .when(self.zoomed, |panel| {
                let focus = self.focus.clone();
                panel.child(
                    h_flex()
                        .flex_none()
                        .w_full()
                        .px_2()
                        .py_1()
                        .border_b_1()
                        .border_color(colors.border)
                        .child(Label::new("Problems").size(LabelSize::Small))
                        .child(div().flex_1())
                        .child(
                            Button::new("marley-problems-restore", "Restore")
                                .style(ButtonStyle::Subtle)
                                .label_size(LabelSize::Small)
                                .on_click(move |_, window, cx| {
                                    window.focus(&focus, cx);
                                    window.dispatch_action(Box::new(ToggleZoom), cx);
                                }),
                        ),
                )
            })
            .child(
                h_flex()
                    .flex_none()
                    .m_2()
                    .px_2()
                    .py_1()
                    .gap_2()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .child(
                        Icon::new(IconName::MagnifyingGlass)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(div().flex_1().min_w_0().child(self.filter.clone())),
            )
            .child(
                v_flex()
                    .id("marley-problems-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .when(empty, |list| {
                        list.child(
                            div().px_3().py_2().child(
                                Label::new("No problems")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            ),
                        )
                    })
                    .children(
                        shown
                            .into_iter()
                            .enumerate()
                            .map(|(index, problem)| self.render_row(index, problem, cx)),
                    ),
            )
    }
}

impl Panel for ProblemsPanel {
    fn persistent_name() -> &'static str {
        "MarleyProblemsPanel"
    }

    fn panel_key() -> &'static str {
        "MarleyProblemsPanel"
    }

    fn activation_focus_handle(&self, cx: &App) -> FocusHandle {
        self.filter.focus_handle(cx)
    }

    // The drawer's place is the Debug panel's, `debugger.dock` (D3).
    fn position(&self, _window: &Window, cx: &App) -> DockPosition {
        DebuggerSettings::get_global(cx).dock.into()
    }

    fn position_is_valid(&self, _position: DockPosition) -> bool {
        true
    }

    fn set_position(
        &mut self,
        position: DockPosition,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let position: settings::DockPosition = position.into();
        settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
            content.debugger.get_or_insert_default().dock = Some(position);
        });
    }

    fn is_zoomed(&self, _window: &Window, _cx: &App) -> bool {
        self.zoomed
    }

    fn set_zoomed(&mut self, zoomed: bool, _window: &mut Window, cx: &mut Context<Self>) {
        self.zoomed = zoomed;
        cx.notify();
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        px(300.)
    }

    // No button of its own: the bottom row's diagnostics count is the way in.
    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        None
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Problems")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleProblems)
    }

    fn activation_priority(&self) -> u32 {
        ACTIVATION_PRIORITY
    }

    fn enabled(&self, cx: &App) -> bool {
        crate::marley_layout(cx)
    }
}
