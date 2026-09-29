//! Save as Workflow (#558): a block's command kept as a Zed task with `{{name}}` parameters.
//!
//! A hovered block's Save as Workflow button, or `marley::SaveAsWorkflow` on the selected or newest
//! block, opens an editor with the command's numbers, URLs, branch and existing paths already
//! turned into parameters (`marley_terminal::workflow::guess`). Save appends a task to the
//! project's `.zed/tasks.json`, or the global `tasks.json`, keeping the file's comments: `label`,
//! `command`, `cwd`, and `marley.parameters` with each default and description, which Zed ignores.
//! Zed's picker lists it at once. When any task whose command holds `{{name}}` runs, Marley's task
//! provider asks for the values first (`fill`), the session's last ones or the file's defaults
//! prefilled.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use collections::HashMap;
use editor::{Editor, EditorEvent};
use futures::channel::oneshot;
use gpui::{
    AnyElement, App, AppContext as _, AsyncWindowContext, Context, DismissEvent, Entity,
    EventEmitter, FocusHandle, Focusable, Global, Subscription, WeakEntity, Window,
};
use marley_terminal::BlockState;
use marley_terminal::workflow::{guess, is_name, params_of, substitute};
use serde_json::{Value, json};
use task::SpawnInTerminal;
use terminal::Terminal;
use terminal_view::{MarleyBlockExtras, TerminalView};
use ui::{
    Button, ButtonStyle, Checkbox, Headline, HeadlineSize, IconButton, IconName, Label, LabelSize,
    ToggleState, Tooltip, prelude::*,
};
use util::ResultExt as _;
use workspace::{ModalView, Workspace};

use crate::{RunWorkflow, SaveAsWorkflow, SaveWorkflow};

/// What a new `tasks.json` starts as.
const NEW_TASKS_FILE: &str = "// Tasks and Marley's workflows, in Zed's tasks.json format; Save as Workflow appends here.\n[\n]\n";

/// The values each parameterized task last ran with this session, by its full label.
#[derive(Default)]
struct LastValues(HashMap<String, BTreeMap<String, String>>);

impl Global for LastValues {}

/// Installs the block button, the action and the values' memory; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(LastValues::default());
    cx.set_global(MarleyBlockExtras(Arc::new(block_buttons)));
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action_renderer(|div, _, _, cx| {
            div.on_action(cx.listener(|workspace, _: &SaveAsWorkflow, window, cx| {
                let Some(view) = crate::blocks::focused_terminal(workspace, window, cx) else {
                    cx.propagate();
                    return;
                };
                let Some(block) = crate::block_filter::block_to_filter(&view, cx) else {
                    cx.propagate();
                    return;
                };
                open_editor(&view, block, window, cx);
            }))
        });
    })
    .detach();
}

/// The hover buttons for the block at `index`: Save as Workflow for a finished block whose
/// one-line command the shell reported.
fn block_buttons(
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    cx: &App,
) -> Vec<AnyElement> {
    let savable = terminal.read(cx).blocks().get(index).is_some_and(|block| {
        block.command_verified
            && block.state == BlockState::Finished
            && !block.command.trim().is_empty()
            && !block.command.contains('\n')
    });
    if !savable {
        return Vec::new();
    }
    let view = view.clone();
    vec![
        IconButton::new(("marley-block-workflow", index), IconName::Bookmark)
            .icon_size(IconSize::XSmall)
            .tooltip(Tooltip::text("Save as Workflow"))
            .on_click(move |_, window, cx| open_editor(&view, index, window, cx))
            .into_any_element(),
    ]
}

/// Opens the workflow editor for the block at `index` of `view`'s terminal, after this update,
/// since the action runs while the workspace is being updated.
fn open_editor(view: &Entity<TerminalView>, index: usize, window: &Window, cx: &mut App) {
    let view = view.clone();
    window.defer(cx, move |window, cx| {
        let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
            return;
        };
        let terminal = view.read(cx).terminal().read(cx);
        let Some(block) = terminal.blocks().get(index) else {
            return;
        };
        let command = block.command.trim().to_string();
        let cwd = block.prompt.pwd.clone();
        let exists = |token: &str| {
            cwd.as_deref()
                .is_some_and(|cwd| Path::new(cwd).join(token).exists())
        };
        let guessed = guess(&command, block.prompt.git_branch.as_deref(), exists);
        let name: String = command
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        let root = workspace
            .read(cx)
            .project()
            .read(cx)
            .visible_worktrees(cx)
            .next()
            .map(|worktree| worktree.read(cx).abs_path().to_path_buf());
        let defaults: BTreeMap<String, String> = guessed
            .parameters
            .into_iter()
            .map(|parameter| (parameter.name, parameter.default))
            .collect();
        let draft = Draft {
            workspace: workspace.downgrade(),
            name,
            command: guessed.command,
            defaults,
            cwd,
            root,
        };
        workspace.update(cx, |workspace, cx| {
            workspace.toggle_modal(window, cx, |window, cx| {
                WorkflowEditor::new(draft, window, cx)
            });
        });
    });
}

/// What the editor opens with.
struct Draft {
    workspace: WeakEntity<Workspace>,
    name: String,
    command: String,
    defaults: BTreeMap<String, String>,
    cwd: Option<String>,
    root: Option<PathBuf>,
}

/// One parameter's row in the editor: its default and its description.
struct ParameterRow {
    name: String,
    default: Entity<Editor>,
    description: Entity<Editor>,
}

/// The Save as Workflow editor.
struct WorkflowEditor {
    workspace: WeakEntity<Workspace>,
    name: Entity<Editor>,
    command: Entity<Editor>,
    rows: Vec<ParameterRow>,
    guessed: BTreeMap<String, String>,
    global: bool,
    cwd: Option<String>,
    root: Option<PathBuf>,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

/// A single-line editor holding `text`.
fn field(text: &str, window: &mut Window, cx: &mut App) -> Entity<Editor> {
    cx.new(|cx| {
        let mut editor = Editor::single_line(window, cx);
        editor.set_text(text, window, cx);
        editor
    })
}

impl WorkflowEditor {
    fn new(draft: Draft, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = field(&draft.name, window, cx);
        let command = field(&draft.command, window, cx);
        let subscriptions = vec![cx.subscribe_in(
            &command,
            window,
            |this, _, event: &EditorEvent, window, cx| {
                if matches!(event, EditorEvent::BufferEdited) {
                    this.refresh_rows(window, cx);
                }
            },
        )];
        let mut editor = Self {
            workspace: draft.workspace,
            name,
            command,
            rows: Vec::new(),
            guessed: draft.defaults,
            global: false,
            cwd: draft.cwd,
            root: draft.root,
            error: None,
            _subscriptions: subscriptions,
        };
        editor.refresh_rows(window, cx);
        editor
    }

    /// A row for each parameter the command names, the rows kept for names it still names.
    fn refresh_rows(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let names = params_of(&self.command.read(cx).text(cx));
        let mut old: Vec<ParameterRow> = std::mem::take(&mut self.rows);
        for name in names {
            let row = if let Some(place) = old.iter().position(|row| row.name == name) {
                old.remove(place)
            } else {
                let default = self.guessed.get(&name).cloned().unwrap_or_default();
                ParameterRow {
                    default: field(&default, window, cx),
                    description: field("", window, cx),
                    name,
                }
            };
            self.rows.push(row);
        }
        cx.notify();
    }

    /// The file the workflow goes into.
    fn file(&self) -> Option<PathBuf> {
        if self.global {
            return Some(paths::tasks_file().clone());
        }
        let root = self.root.as_ref()?;
        Some(root.join(paths::local_tasks_file_relative_path().as_std_path()))
    }

    /// The task as `tasks.json` holds it.
    fn task(&self, label: &str, cx: &App) -> Value {
        let parameters: serde_json::Map<String, Value> = self
            .rows
            .iter()
            .map(|row| {
                let mut parameter = serde_json::Map::new();
                parameter.insert("default".into(), json!(row.default.read(cx).text(cx)));
                let description = row.description.read(cx).text(cx);
                if !description.trim().is_empty() {
                    parameter.insert("description".into(), json!(description.trim()));
                }
                (row.name.clone(), Value::Object(parameter))
            })
            .collect();
        let mut task = json!({
            "label": label,
            "command": self.command.read(cx).text(cx).trim(),
        });
        let at_root = self
            .cwd
            .as_deref()
            .zip(self.root.as_deref())
            .is_some_and(|(cwd, root)| Path::new(cwd) == root);
        let cwd = if at_root {
            Some("$ZED_WORKTREE_ROOT".to_string())
        } else {
            self.cwd.clone()
        };
        if let (Some(object), Some(cwd)) = (task.as_object_mut(), cwd) {
            object.insert("cwd".into(), json!(cwd));
        }
        if !parameters.is_empty()
            && let Some(object) = task.as_object_mut()
        {
            object.insert("marley".into(), json!({ "parameters": parameters }));
        }
        task
    }

    /// Checks the editor and writes the task off the main thread; the toast names the file.
    fn save(&mut self, _: &SaveWorkflow, _: &mut Window, cx: &mut Context<Self>) {
        let label = self.name.read(cx).text(cx).trim().to_string();
        let problem = if label.is_empty() {
            Some("Give the workflow a name.".to_string())
        } else {
            self.rows.iter().find(|row| !is_name(&row.name)).map(|row| {
                format!(
                    "{{{{{}}}}} is not a parameter name: letters, digits, - and _.",
                    row.name
                )
            })
        };
        if let Some(problem) = problem {
            self.error = Some(problem);
            cx.notify();
            return;
        }
        let Some(file) = self.file() else {
            self.error = Some("This window has no project folder: save it globally.".to_string());
            cx.notify();
            return;
        };
        let task = self.task(&label, cx);
        let writing = cx.background_spawn(futures::future::lazy(move |_| {
            write_workflow_in(&file, &label, &task).map(|()| (file, label))
        }));
        cx.spawn(async move |this, cx| {
            let written = writing.await;
            this.update(cx, |this, cx| match written {
                Ok((file, label)) => {
                    cx.emit(DismissEvent);
                    let message = format!("Saved {label:?} to {}", file.display());
                    this.workspace
                        .update(cx, |workspace, cx| {
                            crate::send_selection::show_toast(workspace, message, cx);
                        })
                        .log_err();
                }
                Err(problem) => {
                    this.error = Some(problem);
                    cx.notify();
                }
            })
            .log_err();
        })
        .detach();
    }

    fn header(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let boxed = |editor: Entity<Editor>| {
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(colors.border)
                .bg(colors.editor_background)
                .child(editor)
        };
        v_flex()
            .gap_1()
            .child(
                Label::new("Name")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(boxed(self.name.clone()))
            .child(
                Label::new("Command")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(boxed(self.command.clone()))
    }

    fn parameters(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let boxed = |editor: Entity<Editor>| {
            div()
                .flex_1()
                .px_2()
                .py_0p5()
                .rounded_md()
                .border_1()
                .border_color(colors.border)
                .bg(colors.editor_background)
                .child(editor)
        };
        v_flex()
            .gap_1()
            .when(!self.rows.is_empty(), |list| {
                list.child(
                    Label::new("Parameters: default and description")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
            .children(self.rows.iter().map(|row| {
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .w(rems(6.))
                            .child(Label::new(row.name.clone()).size(LabelSize::Small)),
                    )
                    .child(boxed(row.default.clone()))
                    .child(boxed(row.description.clone()))
            }))
    }
}

impl ModalView for WorkflowEditor {}

impl EventEmitter<DismissEvent> for WorkflowEditor {}

impl Focusable for WorkflowEditor {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.name.focus_handle(cx)
    }
}

impl Render for WorkflowEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let place = self.file().map_or_else(
            || "no project folder".to_string(),
            |file| file.display().to_string(),
        );
        v_flex()
            .key_context("MarleyWorkflowEditor")
            .on_action(cx.listener(Self::save))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .w(rems(40.))
            .elevation_3(cx)
            .p_3()
            .gap_2()
            .child(Headline::new("Save as Workflow").size(HeadlineSize::Small))
            .child(self.header(cx))
            .child(self.parameters(cx))
            .child(
                Checkbox::new("marley-workflow-global", ToggleState::from(self.global))
                    .label("In every project (the global tasks.json)")
                    .label_size(LabelSize::Small)
                    .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                        this.global = state.selected();
                        cx.notify();
                    })),
            )
            .child(Label::new(place).size(LabelSize::Small).color(Color::Muted))
            .children(
                self.error
                    .clone()
                    .map(|error| Label::new(error).size(LabelSize::Small).color(Color::Error)),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_1()
                    .child(
                        Button::new("marley-workflow-cancel", "Cancel")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent))),
                    )
                    .child(
                        Button::new("marley-workflow-save", "Save")
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.save(&SaveWorkflow, window, cx);
                            })),
                    ),
            )
    }
}

/// Appends `task` to the `tasks.json` at `file`, made with its folder when missing, keeping the
/// file's other entries and comments; a task already labelled `label` there is refused.
fn write_workflow_in(file: &Path, label: &str, task: &Value) -> Result<(), String> {
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if let Some(folder) = file.parent() {
                std::fs::create_dir_all(folder)
                    .map_err(|error| format!("Could not make {}: {error}", folder.display()))?;
            }
            NEW_TASKS_FILE.to_string()
        }
        Err(error) => return Err(format!("Could not read {}: {error}", file.display())),
    };
    let tasks: Vec<Value> = settings::parse_json_with_comments(&text)
        .map_err(|error| format!("{} does not parse: {error}", file.display()))?;
    if tasks
        .iter()
        .any(|task| task.get("label").and_then(Value::as_str) == Some(label))
    {
        return Err(format!(
            "A task named {label:?} is already in {}.",
            file.display()
        ));
    }
    let (range, value) = settings_json::append_top_level_array_value_in_json_text(&text, task, 2);
    let mut written = text;
    written.replace_range(range, &value);
    std::fs::write(file, written)
        .map_err(|error| format!("Could not write {}: {error}", file.display()))
}

/// Asks for the values of `task`'s `{{name}}` parameters and fills them in, or gives it back as it
/// is when it names none; none when the user cancels. The prompt starts from the values the task
/// last ran with this session, else the defaults in the `tasks.json` files of `workspace`'s
/// project and the global one.
pub(crate) async fn fill(
    workspace: &WeakEntity<Workspace>,
    mut task: SpawnInTerminal,
    cx: &mut AsyncWindowContext,
) -> Option<SpawnInTerminal> {
    let mut names: Vec<String> = Vec::new();
    for part in task.command.iter().chain(&task.args) {
        for name in params_of(part) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    if names.is_empty() {
        return Some(task);
    }
    let last = cx
        .update(|_, cx| {
            cx.try_global::<LastValues>()
                .and_then(|last| last.0.get(&task.full_label).cloned())
        })
        .ok()
        .flatten();
    let defaults = if let Some(last) = last {
        last
    } else {
        let files = workspace.read_with(cx, task_files).ok()?;
        let label = task.label.clone();
        cx.background_spawn(futures::future::lazy(move |_| {
            file_defaults(&files, &label)
        }))
        .await
    };
    let (sender, receiver) = oneshot::channel();
    let title = task.label.clone();
    workspace
        .update_in(cx, |workspace, window, cx| {
            workspace.toggle_modal(window, cx, |window, cx| {
                ParameterPrompt::new(title, &names, &defaults, sender, window, cx)
            });
        })
        .ok()?;
    let values = receiver.await.ok()?;
    let fill_in = |text: &str| substitute(text, &values).ok();
    task.command = match task.command.as_deref() {
        Some(command) => Some(fill_in(command)?),
        None => None,
    };
    task.args = task
        .args
        .iter()
        .map(|argument| fill_in(argument))
        .collect::<Option<Vec<_>>>()?;
    task.command_label = fill_in(&task.command_label)?;
    let label = task.full_label.clone();
    cx.update(|_, cx| {
        cx.default_global::<LastValues>().0.insert(label, values);
    })
    .ok()?;
    Some(task)
}

/// The `tasks.json` files a workflow can be in: each of the project's folders', then the global one.
fn task_files(workspace: &Workspace, cx: &App) -> Vec<PathBuf> {
    let project = workspace.project().read(cx);
    project
        .visible_worktrees(cx)
        .map(|worktree| {
            worktree
                .read(cx)
                .abs_path()
                .join(paths::local_tasks_file_relative_path().as_std_path())
        })
        .chain(std::iter::once(paths::tasks_file().clone()))
        .collect()
}

/// The defaults `marley.parameters` gives the task labelled `label` in the first of `files` that
/// has it; none when no file does.
fn file_defaults(files: &[PathBuf], label: &str) -> BTreeMap<String, String> {
    files
        .iter()
        .filter_map(|file| std::fs::read_to_string(file).ok())
        .filter_map(|text| settings::parse_json_with_comments::<Vec<Value>>(&text).ok())
        .flatten()
        .find(|task| task.get("label").and_then(Value::as_str) == Some(label))
        .and_then(|task| task.pointer("/marley/parameters").cloned())
        .and_then(|parameters| parameters.as_object().cloned())
        .map(|parameters| {
            parameters
                .into_iter()
                .filter_map(|(name, parameter)| {
                    let default = parameter.get("default")?.as_str()?.to_string();
                    Some((name, default))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The prompt for a workflow's parameters as it runs.
struct ParameterPrompt {
    title: String,
    fields: Vec<(String, Entity<Editor>)>,
    answer: Option<oneshot::Sender<BTreeMap<String, String>>>,
}

impl ParameterPrompt {
    fn new(
        title: String,
        names: &[String],
        defaults: &BTreeMap<String, String>,
        answer: oneshot::Sender<BTreeMap<String, String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let fields = names
            .iter()
            .map(|name| {
                let value = defaults.get(name).cloned().unwrap_or_default();
                (name.clone(), field(&value, window, cx))
            })
            .collect();
        Self {
            title,
            fields,
            answer: Some(answer),
        }
    }

    fn run(&mut self, _: &RunWorkflow, _: &mut Window, cx: &mut Context<Self>) {
        let values: BTreeMap<String, String> = self
            .fields
            .iter()
            .map(|(name, editor)| (name.clone(), editor.read(cx).text(cx)))
            .collect();
        if let Some(answer) = self.answer.take() {
            answer.send(values).ok();
        }
        cx.emit(DismissEvent);
    }
}

impl ModalView for ParameterPrompt {}

impl EventEmitter<DismissEvent> for ParameterPrompt {}

impl Focusable for ParameterPrompt {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.fields
            .first()
            .map_or_else(|| cx.focus_handle(), |(_, editor)| editor.focus_handle(cx))
    }
}

impl Render for ParameterPrompt {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .key_context("MarleyWorkflowPrompt")
            .on_action(cx.listener(Self::run))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .w(rems(30.))
            .elevation_3(cx)
            .p_3()
            .gap_2()
            .child(Headline::new(format!("Run {}", self.title)).size(HeadlineSize::Small))
            .children(self.fields.iter().map(|(name, editor)| {
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .w(rems(6.))
                            .child(Label::new(name.clone()).size(LabelSize::Small)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .border_1()
                            .border_color(colors.border)
                            .bg(colors.editor_background)
                            .child(editor.clone()),
                    )
            }))
            .child(
                Label::new("Enter runs it; Escape runs nothing.")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
    }
}
