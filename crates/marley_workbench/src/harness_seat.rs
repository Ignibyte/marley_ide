//! A harness seat in one step (#691).
//!
//! `marley: new harness seat` asks for a name, the agent, a folder and a role, then runs the
//! harness's `seat add` and `seat start` (its TICKET-109) through the command Marley follows the
//! harness with, and opens the new session's tab.
//!
//! The command is `harness::Harness::seat_command`'s: a `marley.harness` command up to its `mcp`,
//! so a harness reached over SSH is set up over SSH, or the embedded runtime's `rh --state
//! <root>`. Each answers one JSON object on stdout, or `rh: CODE: reason` on stderr, which the form
//! shows. The palette lists the action only while the harness's write verbs are on.

use std::path::PathBuf;

use editor::Editor;
use gpui::{
    App, AppContext as _, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    SharedString, Task, WeakEntity, Window, actions,
};
use serde_json::Value;
use ui::{ButtonStyle, Headline, HeadlineSize, prelude::*};
use util::ResultExt as _;
use workspace::{ModalView, Workspace};

use crate::harness::{Harness, writes_on};

actions!(
    marley,
    [
        /// Adds and starts a harness seat: a Claude Code or Codex session from a new profile, with
        /// a role, opened in its tab.
        #[derive(Eq)]
        NewHarnessSeat
    ]
);

/// The agent a seat runs.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SeatAgent {
    Claude,
    Codex,
}

impl SeatAgent {
    /// The agent as `seat add --agent` takes it.
    const fn argument(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

/// Opens the form in `workspace`, with the active project's folder as the seat's.
pub(crate) fn open_form(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if !writes_on(cx) {
        return;
    }
    let folder = terminal_view::default_working_directory(workspace, cx);
    let handle = workspace.weak_handle();
    workspace.toggle_modal(window, cx, |window, cx| {
        NewSeatModal::new(handle, folder, window, cx)
    });
}

/// The form: the seat's name, agent, folder and role, and how the last run went.
struct NewSeatModal {
    workspace: WeakEntity<Workspace>,
    name: Entity<Editor>,
    folder: Entity<Editor>,
    role: Entity<Editor>,
    agent: SeatAgent,
    status: Option<(SharedString, Color)>,
    running: Option<Task<()>>,
}

impl NewSeatModal {
    fn new(
        workspace: WeakEntity<Workspace>,
        folder: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let field = |placeholder: &str, text: Option<String>, window: &mut Window, cx: &mut App| {
            cx.new(|cx| {
                let mut editor = Editor::single_line(window, cx);
                editor.set_placeholder_text(placeholder, window, cx);
                if let Some(text) = text {
                    editor.set_text(text, window, cx);
                }
                editor
            })
        };
        let name = field("A name, such as manager or builder-1", None, window, cx);
        let folder = field(
            "The folder it works in",
            folder.map(|folder| folder.display().to_string()),
            window,
            cx,
        );
        let role = field("manager, or any label (optional)", None, window, cx);
        Self {
            workspace,
            name,
            folder,
            role,
            agent: SeatAgent::Claude,
            status: None,
            running: None,
        }
    }

    /// Runs `seat add`, then `seat start`, and opens the session's tab; a refusal stays in the
    /// form.
    fn create(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.running.is_some() {
            return;
        }
        let name = self.name.read(cx).text(cx).trim().to_string();
        let folder = self.folder.read(cx).text(cx).trim().to_string();
        let role = self.role.read(cx).text(cx).trim().to_string();
        if name.is_empty() || folder.is_empty() {
            self.status = Some((
                SharedString::new_static("A seat needs a name and a folder"),
                Color::Error,
            ));
            cx.notify();
            return;
        }
        let Some((program, base)) = Harness::seat_command(cx) else {
            self.status = Some((
                SharedString::new_static("Marley follows no harness it can reach"),
                Color::Error,
            ));
            cx.notify();
            return;
        };
        let mut add = base.clone();
        add.extend([
            "seat".to_string(),
            "add".to_string(),
            name.clone(),
            "--agent".to_string(),
            self.agent.argument().to_string(),
            "--cwd".to_string(),
            folder,
        ]);
        if !role.is_empty() {
            add.extend(["--role".to_string(), role]);
        }
        let mut start = base;
        start.extend(["seat".to_string(), "start".to_string(), name]);
        let waiting = if self.agent == SeatAgent::Claude {
            "Starting the seat: Claude Code reports once it is up, within a minute"
        } else {
            "Starting the seat"
        };
        self.status = Some((SharedString::new_static(waiting), Color::Muted));
        cx.notify();
        let workspace = self.workspace.clone();
        self.running = Some(cx.spawn_in(window, async move |modal, cx| {
            let started = async {
                run_seat(&program, &add).await?;
                run_seat(&program, &start).await
            }
            .await;
            match started {
                Ok(value) => {
                    let id = value
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    if !id.is_empty() {
                        workspace
                            .update_in(cx, |workspace, window, cx| {
                                crate::harness::open_in_home(
                                    workspace.weak_handle(),
                                    id,
                                    window,
                                    cx,
                                );
                            })
                            .log_err();
                    }
                    modal.update(cx, |_, cx| cx.emit(DismissEvent)).log_err();
                }
                Err(reason) => {
                    modal
                        .update(cx, |modal, cx| {
                            modal.running = None;
                            modal.status = Some((reason, Color::Error));
                            cx.notify();
                        })
                        .log_err();
                }
            }
        }));
    }

    fn render_field(label: &'static str, editor: &Entity<Editor>, cx: &App) -> AnyElement {
        let colors = cx.theme().colors();
        v_flex()
            .gap_0p5()
            .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
            .child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.editor_background)
                    .child(editor.clone()),
            )
            .into_any_element()
    }
}

/// Runs one harness command: its JSON answer, or the harness's `CODE: reason`.
async fn run_seat(program: &PathBuf, arguments: &[String]) -> Result<Value, SharedString> {
    let output = crate::process::output(program, arguments, None, &[])
        .await
        .map_err(|error| {
            SharedString::from(format!("{} did not start: {error}", program.display()))
        })?;
    if output.status.success() {
        return serde_json::from_slice::<Value>(&output.stdout).map_err(|error| {
            SharedString::from(format!("the harness's answer did not parse: {error}"))
        });
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let said = stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("the harness refused, and said nothing");
    Err(said.strip_prefix("rh: ").unwrap_or(said).to_string().into())
}

impl ModalView for NewSeatModal {}

impl EventEmitter<DismissEvent> for NewSeatModal {}

impl Focusable for NewSeatModal {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.name.focus_handle(cx)
    }
}

impl Render for NewSeatModal {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let agent_button = |id: &'static str, label: &'static str, agent: SeatAgent| {
            Button::new(id, label)
                .style(ButtonStyle::Filled)
                .toggle_state(self.agent == agent)
                .on_click(cx.listener(move |modal, _, _, cx| {
                    modal.agent = agent;
                    cx.notify();
                }))
        };
        v_flex()
            .key_context("NewHarnessSeat")
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .on_action(cx.listener(|modal, _: &menu::Confirm, window, cx| modal.create(window, cx)))
            .w(rems(34.))
            .elevation_3(cx)
            .p_3()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(Headline::new("New Harness Seat").size(HeadlineSize::Small))
                    .child(
                        Label::new(
                            "The harness adds a profile for the seat and starts it under its \
                             supervision. A seat whose role is manager becomes the root's manager.",
                        )
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    ),
            )
            .child(Self::render_field("Name", &self.name, cx))
            .child(
                v_flex()
                    .gap_0p5()
                    .child(
                        Label::new("Agent")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(agent_button(
                                "harness-seat-claude",
                                "Claude Code",
                                SeatAgent::Claude,
                            ))
                            .child(agent_button(
                                "harness-seat-codex",
                                "Codex",
                                SeatAgent::Codex,
                            )),
                    ),
            )
            .child(Self::render_field("Folder", &self.folder, cx))
            .child(Self::render_field("Role", &self.role, cx))
            .when_some(self.status.clone(), |this, (status, color)| {
                this.child(Label::new(status).size(LabelSize::Small).color(color))
            })
            .child(
                h_flex().justify_end().child(
                    Button::new("harness-seat-create", "Create")
                        .style(ButtonStyle::Filled)
                        .disabled(self.running.is_some())
                        .on_click(cx.listener(|modal, _, window, cx| modal.create(window, cx))),
                ),
            )
    }
}
