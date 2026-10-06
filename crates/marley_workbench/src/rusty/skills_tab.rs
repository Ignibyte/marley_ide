//! Rusty's skills store in a center tab (#665): the screen Rusty's app draws as
//! `SkillsPage.qml`.
//!
//! On the left, the skills (staged ones first, then the active ones, each by name) and the scripts
//! beside them; on the right, the one chosen. A skill's description and body save together; a
//! staged skill is approved or rejected; Scan shows Rusty's findings, and an approval the scan
//! blocked can be made anyway. A script's text saves, and Run opens a terminal on it, as Rusty's
//! app does. Every write is Rusty's, which commits its store.

use std::borrow::Cow;
use std::path::Path;

use anyhow::Context as _;
use editor::{Editor, EditorEvent};
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, PromptLevel, Render,
    SharedString, Subscription, Task, WeakEntity, Window, actions,
};
use marley_rusty::skills::{
    SCRIPT_LIST, SCRIPT_VIEW, SKILL_LIST, SKILL_SCAN, Script, Skill, SkillWrite, blocked_by_scan,
    findings_from_answer, ordered, script_text_from_answer, scripts_from_answer,
    skills_from_answer, update_from_answer,
};
use project::TaskSourceKind;
use serde_json::json;
use task::{RevealTarget, TaskContext, TaskTemplate};
use ui::{Checkbox, Chip, ListItem, ListItemSpacing, ListSubHeader, ToggleState, prelude::*};
use util::ResultExt as _;
use util::shell::ShellKind;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};

use super::decisions_tab::{Link, ReadDue};

actions!(
    rusty,
    [
        /// Opens Skills: Rusty's skills and scripts, to review, edit, make and run.
        #[derive(Eq)]
        OpenSkills,
    ]
);

/// The task id's base for a script's run; Zed keys a task's terminal by it.
const RUN_ID_BASE: &str = "marley-rusty-script";

/// Registers `rusty: open skills` on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &OpenSkills, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Opens the Skills tab once the update in progress ends; the rail's Brain view calls it.
pub(super) fn open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    window.defer(cx, move |window, cx| {
        workspace
            .update(cx, |workspace, cx| open(workspace, window, cx))
            .log_err();
    });
}

/// Brings the workspace's Skills tab forward with the focus, or adds one to the active pane; with
/// Rusty off, says so and opens nothing.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if let Some(reason) = super::unavailable(cx) {
        workspace.show_toast(
            Toast::new(
                NotificationId::unique::<BrainSkillsView>(),
                reason.to_string(),
            ),
            cx,
        );
        return;
    }
    let open = workspace.items_of_type::<BrainSkillsView>(cx).next();
    if let Some(view) = open {
        workspace.activate_item(&view, true, true, window, cx);
        return;
    }
    let weak = cx.entity().downgrade();
    let view = cx.new(|cx| BrainSkillsView::new(weak, window, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}

/// Runs `script` in a terminal in the center, as Rusty runs it: `bash <path>`.
///
/// # Errors
///
/// When the project is not on this machine, where Rusty keeps the script, or the path cannot be
/// quoted for the shell.
fn run_script(
    workspace: &mut Workspace,
    script: &Script,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        workspace.project().read(cx).is_local(),
        "A script runs from a project on this machine, where Rusty keeps it"
    );
    // A task's arguments reach its shell unquoted (`ShellBuilder::build_no_quote`, as
    // `remote::open` notes).
    let path = ShellKind::system()
        .try_quote(&script.path)
        .map(Cow::into_owned)
        .with_context(|| format!("{:?} cannot be quoted for the shell", script.path))?;
    let template = TaskTemplate {
        label: format!("rusty {}", script.name),
        command: "bash".to_string(),
        args: vec![path],
        cwd: Path::new(&script.path)
            .parent()
            .map(|folder| folder.to_string_lossy().into_owned()),
        reveal_target: RevealTarget::Center,
        use_new_terminal: false,
        allow_concurrent_runs: false,
        show_summary: true,
        // Zed's summary would name the command without the script's path.
        show_command: false,
        ..TaskTemplate::default()
    };
    let resolved = template
        .resolve_task(RUN_ID_BASE, &TaskContext::default())
        .context("the script's task did not resolve")?;
    workspace.schedule_resolved_task(TaskSourceKind::UserInput, resolved, false, window, cx);
    Ok(())
}

/// What the right side shows.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Chosen {
    /// A skill, by name.
    Skill(String),
    /// A script, by path, which two skills' scripts cannot share.
    Script(String),
}

/// The tab's word on the last thing it did: Rusty's refusal, or what went well.
#[derive(Clone, Debug)]
struct Notice {
    text: SharedString,
    color: Color,
}

impl Notice {
    fn refused(error: &str) -> Self {
        Self {
            text: error.lines().next().unwrap_or_default().to_string().into(),
            color: Color::Error,
        }
    }

    const fn done(text: &'static str) -> Self {
        Self {
            text: SharedString::new_static(text),
            color: Color::Muted,
        }
    }
}

/// The Skills tab.
pub(super) struct BrainSkillsView {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    /// The last answers drawn, skills in the tab's order; none before the first.
    listed: Option<(Vec<Skill>, Vec<Script>)>,
    /// A failed read's words, shown with Read again over the lists kept.
    failure: Option<SharedString>,
    reading: Option<Task<()>>,
    due: ReadDue,
    link: Link,
    chosen: Option<Chosen>,
    description: Entity<Editor>,
    /// A skill's body or a script's text.
    body: Entity<Editor>,
    /// The chosen skill's last scan, from Scan or a Save; none until one ran.
    findings: Option<Vec<String>>,
    /// Whether Approve was refused for the scan's findings, which Approve Anyway overrides.
    blocked: bool,
    notice: Option<Notice>,
    writing: Option<Task<()>>,
    /// The chosen script's text being read.
    viewing: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl BrainSkillsView {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![
            cx.observe_global::<super::Announced>(|this, cx| {
                if this.showing(cx) {
                    this.read(cx);
                } else if this.due == ReadDue::No {
                    this.due = ReadDue::WhenShown;
                }
            }),
            cx.observe_global::<super::Rusty>(Self::rusty_changed),
        ];
        let description = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("When an agent should use it", window, cx);
            editor
        });
        let body = cx.new(|cx| Editor::multi_line(window, cx));
        let mut view = Self {
            workspace,
            focus_handle: cx.focus_handle(),
            listed: None,
            failure: None,
            reading: None,
            due: ReadDue::No,
            link: Link::now(cx),
            chosen: None,
            description,
            body,
            findings: None,
            blocked: false,
            notice: None,
            writing: None,
            viewing: None,
            _subscriptions: subscriptions,
        };
        view.read(cx);
        view
    }

    /// Whether the tab is its pane's active item, asked each time (PR-607).
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

    /// Off drops the lists and calls nothing; the link coming up reads (L-658).
    fn rusty_changed(&mut self, cx: &mut Context<Self>) {
        let link = Link::now(cx);
        if link == self.link {
            return;
        }
        self.link = link;
        match link {
            Link::Off => {
                self.listed = None;
                self.failure = None;
                self.reading = None;
                self.due = ReadDue::No;
                self.chosen = None;
            }
            Link::Up => self.read(cx),
            Link::Down => {}
        }
        cx.notify();
    }

    /// Reads the skills and the scripts together; with one read running, one more after it.
    fn read(&mut self, cx: &Context<Self>) {
        if !super::is_on(cx) || !super::is_connected(cx) {
            return;
        }
        if self.reading.is_some() {
            self.due = ReadDue::AfterThis;
            return;
        }
        self.due = ReadDue::No;
        let skills = super::call_tool(SKILL_LIST, json!({ "include_pending": true }), cx);
        let scripts = super::call_tool(SCRIPT_LIST, json!({ "include_pending": true }), cx);
        self.reading = Some(cx.spawn(async move |this, cx| {
            let skills = skills.await.and_then(|text| {
                skills_from_answer(&text)
                    .map_err(|error| format!("{SKILL_LIST}'s answer did not parse: {error}"))
            });
            let scripts = scripts.await.and_then(|text| {
                scripts_from_answer(&text)
                    .map_err(|error| format!("{SCRIPT_LIST}'s answer did not parse: {error}"))
            });
            let read = skills.and_then(|skills| scripts.map(|scripts| (ordered(skills), scripts)));
            this.update(cx, |this, cx| this.take_read(read, cx))
                .log_err();
        }));
    }

    fn read_again(&mut self, cx: &mut Context<Self>) {
        self.read(cx);
        cx.notify();
    }

    fn take_read(
        &mut self,
        read: Result<(Vec<Skill>, Vec<Script>), String>,
        cx: &mut Context<Self>,
    ) {
        self.reading = None;
        match read {
            Ok(listed) => {
                // A choice that went away clears the right side; one still there keeps what its
                // editors hold.
                let kept = match &self.chosen {
                    Some(Chosen::Skill(name)) => listed.0.iter().any(|skill| &skill.name == name),
                    Some(Chosen::Script(path)) => {
                        listed.1.iter().any(|script| &script.path == path)
                    }
                    None => true,
                };
                if !kept {
                    self.chosen = None;
                }
                self.listed = Some(listed);
                self.failure = None;
            }
            Err(error) => {
                let line = error.lines().next().unwrap_or_default().to_string();
                self.failure = Some(line.into());
            }
        }
        if self.due == ReadDue::AfterThis {
            self.read(cx);
        }
        cx.notify();
    }

    fn skill(&self, name: &str) -> Option<&Skill> {
        self.listed
            .as_ref()
            .and_then(|(skills, _)| skills.iter().find(|skill| skill.name == name))
    }

    fn script(&self, path: &str) -> Option<&Script> {
        self.listed
            .as_ref()
            .and_then(|(_, scripts)| scripts.iter().find(|script| script.path == path))
    }

    /// Shows `skill` on the right, its description and body in the editors.
    fn show_skill(&mut self, skill: &Skill, window: &mut Window, cx: &mut Context<Self>) {
        self.chosen = Some(Chosen::Skill(skill.name.clone()));
        self.findings = None;
        self.blocked = false;
        self.notice = None;
        self.description.update(cx, |editor, cx| {
            editor.set_text(skill.description.clone(), window, cx);
        });
        self.body.update(cx, |editor, cx| {
            editor.set_text(skill.body.clone(), window, cx);
        });
        cx.notify();
    }

    /// Shows `script` on the right and reads its text into the editor.
    fn show_script(&mut self, script: &Script, window: &mut Window, cx: &mut Context<Self>) {
        let path = script.path.clone();
        self.chosen = Some(Chosen::Script(path.clone()));
        self.findings = None;
        self.blocked = false;
        self.notice = None;
        self.body
            .update(cx, |editor, cx| editor.set_text("", window, cx));
        let asking = super::call_tool(SCRIPT_VIEW, json!({ "name": script.qualified() }), cx);
        self.viewing = Some(cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await.and_then(|text| {
                script_text_from_answer(&text)
                    .map_err(|error| format!("{SCRIPT_VIEW}'s answer did not parse: {error}"))
            });
            this.update_in(cx, |this, window, cx| {
                this.viewing = None;
                if this.chosen != Some(Chosen::Script(path)) {
                    return;
                }
                match answer {
                    Ok(viewed) => this
                        .body
                        .update(cx, |editor, cx| editor.set_text(viewed.text, window, cx)),
                    Err(error) => this.notice = Some(Notice::refused(&error)),
                }
                cx.notify();
            })
            .log_err();
        }));
        cx.notify();
    }

    /// A skill the New Skill form made: listed and shown at once, then read again.
    fn created(&mut self, skill: &Skill, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((skills, _)) = &mut self.listed {
            skills.retain(|each| each.name != skill.name);
            skills.push(skill.clone());
            *skills = ordered(std::mem::take(skills));
        }
        self.show_skill(skill, window, cx);
        self.read_again(cx);
    }

    /// Sends `write` to Rusty; its answer reads the lists again and says how it went.
    fn send(&mut self, write: SkillWrite, window: &Window, cx: &mut Context<Self>) {
        if self.writing.is_some() {
            return;
        }
        self.notice = None;
        let asking = super::call_tool(write.tool(), write.arguments(), cx);
        self.writing = Some(cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await;
            this.update(cx, |this, cx| {
                this.writing = None;
                this.answered(&write, answer);
                this.read_again(cx);
            })
            .log_err();
        }));
        cx.notify();
    }

    fn answered(&mut self, write: &SkillWrite, answer: Result<String, String>) {
        let text = match answer {
            Ok(text) => text,
            Err(error) => {
                self.blocked = matches!(write, SkillWrite::Approve { force: false, .. })
                    && blocked_by_scan(&error);
                self.notice = Some(Notice::refused(&error));
                return;
            }
        };
        match write {
            SkillWrite::Update { .. } => {
                let findings = update_from_answer(&text)
                    .map(|answer| answer.findings)
                    .unwrap_or_default();
                self.notice = Some(Notice::done(if findings.is_empty() {
                    "Saved."
                } else {
                    "Saved; the scan has findings."
                }));
                self.findings = Some(findings);
            }
            SkillWrite::UpdateScript { .. } => self.notice = Some(Notice::done("Saved.")),
            SkillWrite::Approve { .. } => {
                self.blocked = false;
                self.findings = None;
                self.notice = Some(Notice::done("Approved: Claude Code can load it now."));
            }
            SkillWrite::Delete { .. } | SkillWrite::Reject { .. } => self.chosen = None,
            SkillWrite::Create { .. } => {}
        }
    }

    /// Runs Rusty's safety scan on the chosen skill.
    fn scan(&mut self, name: &str, window: &Window, cx: &mut Context<Self>) {
        if self.writing.is_some() {
            return;
        }
        let asking = super::call_tool(SKILL_SCAN, json!({ "name": name }), cx);
        self.notice = None;
        self.writing = Some(cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await.and_then(|text| {
                findings_from_answer(&text)
                    .map_err(|error| format!("{SKILL_SCAN}'s answer did not parse: {error}"))
            });
            this.update(cx, |this, cx| {
                this.writing = None;
                match answer {
                    Ok(findings) => {
                        if findings.is_empty() {
                            this.notice = Some(Notice::done("The scan is clean."));
                        }
                        this.findings = Some(findings);
                    }
                    Err(error) => this.notice = Some(Notice::refused(&error)),
                }
                cx.notify();
            })
            .log_err();
        }));
        cx.notify();
    }

    /// Asks first, then deletes the skill `name`.
    fn delete(name: String, window: &mut Window, cx: &mut Context<Self>) {
        let answer = window.prompt(
            PromptLevel::Warning,
            &format!("Delete the skill {name}?"),
            Some("Rusty removes its folder from the store and commits the change."),
            &["Delete", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if answer.await.ok() != Some(0) {
                return;
            }
            this.update_in(cx, |this, window, cx| {
                this.send(SkillWrite::Delete { name }, window, cx);
            })
            .log_err();
        })
        .detach();
    }

    fn save(&mut self, window: &Window, cx: &mut Context<Self>) {
        let body = self.body.read(cx).text(cx);
        let write = match &self.chosen {
            Some(Chosen::Skill(name)) => SkillWrite::Update {
                name: name.clone(),
                description: self.description.read(cx).text(cx),
                body,
            },
            Some(Chosen::Script(path)) => {
                let Some(script) = self.script(path) else {
                    return;
                };
                SkillWrite::UpdateScript {
                    name: script.qualified(),
                    body,
                }
            }
            None => return,
        };
        self.send(write, window, cx);
    }

    fn run(&self, script: Script, window: &Window, cx: &mut App) {
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        // The task opens a tab in the workspace's center, outside this tab's update.
        window.defer(cx, move |window, cx| {
            workspace.update(cx, |workspace, cx| {
                if let Err(error) = run_script(workspace, &script, window, cx) {
                    workspace.show_toast(
                        Toast::new(
                            NotificationId::unique::<Self>(),
                            format!("Could not run {}: {error}", script.name),
                        ),
                        cx,
                    );
                }
            });
        });
    }

    fn new_skill(&self, window: &Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let tab = cx.entity().downgrade();
        window.defer(cx, move |window, cx| {
            workspace.update(cx, |workspace, cx| {
                workspace.toggle_modal(window, cx, |window, cx| NewSkillForm::new(tab, window, cx));
            });
        });
    }
}

/// A section's header. `ListSubHeader` grows in a column, so it sits in a box that does not.
fn header(label: impl Into<SharedString>) -> AnyElement {
    div()
        .flex_none()
        .child(ListSubHeader::new(label).inset(true))
        .into_any_element()
}

fn line(text: impl Into<SharedString>, color: Color) -> AnyElement {
    Label::new(text)
        .size(LabelSize::Small)
        .color(color)
        .into_any_element()
}

/// An editor in a box with a border.
fn field(editor: &Entity<Editor>, cx: &App) -> Div {
    div()
        .px_2()
        .py_1()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border)
        .child(editor.clone())
}

impl BrainSkillsView {
    fn render_state(&self, cx: &Context<Self>) -> Option<AnyElement> {
        if !super::is_on(cx) {
            return Some(line(
                "Rusty is off. Turn it on in the Rusty section of the Marley settings.",
                Color::Muted,
            ));
        }
        if !super::is_connected(cx) {
            let reason = super::unavailable(cx).unwrap_or_default();
            return Some(line(
                format!("Marley is not connected to Rusty: {reason}"),
                Color::Muted,
            ));
        }
        if let Some(failure) = &self.failure {
            return Some(
                h_flex()
                    .gap_2()
                    .child(
                        Label::new(failure.clone())
                            .size(LabelSize::Small)
                            .color(Color::Error),
                    )
                    .child(
                        Button::new("rusty-skills-read-again", "Read again")
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(|this, _, _, cx| this.read_again(cx))),
                    )
                    .into_any_element(),
            );
        }
        self.listed
            .is_none()
            .then(|| line("Reading Rusty's skills…", Color::Muted))
    }

    /// The left column: New Skill, the skills, then the scripts.
    fn render_lists(&self, cx: &Context<Self>) -> impl IntoElement {
        let (skills, scripts) = self
            .listed
            .as_ref()
            .map_or((&[][..], &[][..]), |(skills, scripts)| {
                (skills.as_slice(), scripts.as_slice())
            });
        let skill_rows = skills
            .iter()
            .enumerate()
            .map(|(index, skill)| self.render_skill_row(index, skill, cx));
        let script_rows = scripts
            .iter()
            .enumerate()
            .map(|(index, script)| self.render_script_row(index, script, cx));
        v_flex()
            .id("rusty-skills-lists")
            .flex_none()
            .w(rems(20.))
            .h_full()
            .overflow_y_scroll()
            .border_r_1()
            .border_color(cx.theme().colors().border_variant)
            .p_1()
            .child(
                h_flex().px_2().py_1().child(
                    Button::new("rusty-skills-new", "New Skill")
                        .start_icon(Icon::new(IconName::Plus).size(IconSize::Small))
                        .on_click(cx.listener(|this, _, window, cx| this.new_skill(window, cx))),
                ),
            )
            .child(header(format!("Skills ({})", skills.len())))
            .children(skill_rows)
            .child(header(format!("Scripts ({})", scripts.len())))
            .children(script_rows)
    }

    fn render_skill_row(&self, index: usize, skill: &Skill, cx: &Context<Self>) -> AnyElement {
        let chosen = self.chosen == Some(Chosen::Skill(skill.name.clone()));
        let row = skill.clone();
        ListItem::new(("rusty-skill", index))
            .spacing(ListItemSpacing::Sparse)
            .toggle_state(chosen)
            .child(
                v_flex()
                    .min_w_0()
                    .child(
                        h_flex()
                            .gap_1()
                            .child(Label::new(skill.name.clone()).truncate())
                            .when(skill.is_pending(), |name| name.child(Chip::new("pending")))
                            .when(skill.is_auto(), |name| {
                                name.child(
                                    Label::new("auto")
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                            }),
                    )
                    .child(
                        Label::new(skill.description.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                    ),
            )
            .on_click(cx.listener(move |this, _, window, cx| this.show_skill(&row, window, cx)))
            .into_any_element()
    }

    fn render_script_row(&self, index: usize, script: &Script, cx: &Context<Self>) -> AnyElement {
        let chosen = self.chosen == Some(Chosen::Script(script.path.clone()));
        let row = script.clone();
        ListItem::new(("rusty-script", index))
            .spacing(ListItemSpacing::Sparse)
            .toggle_state(chosen)
            .child(
                h_flex()
                    .min_w_0()
                    .gap_1()
                    .child(Label::new(format!("$ {}", script.name)).buffer_font(cx))
                    .child(
                        Label::new(script.skill.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                    )
                    .when(script.is_pending(), |name| name.child(Chip::new("pending"))),
            )
            .on_click(cx.listener(move |this, _, window, cx| this.show_script(&row, window, cx)))
            .into_any_element()
    }

    /// The buttons a skill offers: Approve, Approve Anyway once blocked, and Reject while it is
    /// staged; then Scan, Save and Delete.
    fn render_skill_buttons(&self, skill: &Skill, cx: &Context<Self>) -> impl IntoElement {
        let busy = self.writing.is_some();
        let name = skill.name.clone();
        h_flex()
            .gap_1()
            .when(skill.is_pending(), |buttons| {
                let approve = name.clone();
                let anyway = name.clone();
                let reject = name.clone();
                buttons
                    .child(
                        Button::new("rusty-skill-approve", "Approve")
                            .style(ButtonStyle::Filled)
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                let write = SkillWrite::Approve {
                                    name: approve.clone(),
                                    force: false,
                                };
                                this.send(write, window, cx);
                            })),
                    )
                    .when(self.blocked, |buttons| {
                        buttons.child(
                            Button::new("rusty-skill-approve-anyway", "Approve Anyway")
                                .disabled(busy)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    let write = SkillWrite::Approve {
                                        name: anyway.clone(),
                                        force: true,
                                    };
                                    this.send(write, window, cx);
                                })),
                        )
                    })
                    .child(
                        Button::new("rusty-skill-reject", "Reject")
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                let write = SkillWrite::Reject {
                                    name: reject.clone(),
                                };
                                this.send(write, window, cx);
                            })),
                    )
            })
            .child({
                let name = name.clone();
                Button::new("rusty-skill-scan", "Scan")
                    .disabled(busy)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.scan(&name, window, cx);
                    }))
            })
            .child(
                Button::new("rusty-skill-save", "Save")
                    .style(ButtonStyle::Filled)
                    .disabled(busy)
                    .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
            )
            .child(
                Button::new("rusty-skill-delete", "Delete")
                    .disabled(busy)
                    .on_click(cx.listener(move |_, _, window, cx| {
                        Self::delete(name.clone(), window, cx);
                    })),
            )
    }

    fn render_skill(&self, skill: &Skill, cx: &Context<Self>) -> AnyElement {
        let findings = self
            .findings
            .as_ref()
            .filter(|findings| !findings.is_empty());
        v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_2()
                            .min_w_0()
                            .child(Label::new(skill.name.clone()).size(LabelSize::Large))
                            .when(skill.is_pending(), |name| name.child(Chip::new("pending")))
                            .when(skill.is_auto(), |name| name.child(Chip::new("auto"))),
                    )
                    .child(self.render_skill_buttons(skill, cx)),
            )
            .child(line("Description", Color::Muted))
            .child(field(&self.description, cx))
            .child(line("Body", Color::Muted))
            .child(field(&self.body, cx).flex_1().min_h_0())
            .children(findings.map(|findings| {
                v_flex()
                    .gap_1()
                    .child(line("Scan findings", Color::Warning))
                    .children(
                        findings
                            .iter()
                            .map(|finding| line(format!("• {finding}"), Color::Default)),
                    )
            }))
            .children(self.render_notice())
            .into_any_element()
    }

    fn render_script(&self, script: &Script, cx: &Context<Self>) -> AnyElement {
        let busy = self.writing.is_some();
        let run = script.clone();
        v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_2()
                            .min_w_0()
                            .child(
                                Label::new(format!("$ rusty {}", script.name))
                                    .size(LabelSize::Large)
                                    .buffer_font(cx),
                            )
                            .child(Label::new(script.skill.clone()).color(Color::Muted))
                            .when(script.is_pending(), |name| name.child(Chip::new("pending"))),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("rusty-script-run", "Run")
                                    .start_icon(
                                        Icon::new(IconName::PlayFilled).size(IconSize::Small),
                                    )
                                    .disabled(script.is_pending())
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.run(run.clone(), window, cx);
                                    })),
                            )
                            .child(
                                Button::new("rusty-script-save", "Save")
                                    .style(ButtonStyle::Filled)
                                    .disabled(busy)
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.save(window, cx)),
                                    ),
                            ),
                    ),
            )
            .when(script.is_pending(), |side| {
                side.child(line(
                    "Its skill waits for approval, so the script cannot run yet.",
                    Color::Muted,
                ))
            })
            .child(field(&self.body, cx).flex_1().min_h_0())
            .children(self.render_notice())
            .into_any_element()
    }

    fn render_notice(&self) -> Option<AnyElement> {
        self.notice
            .as_ref()
            .map(|notice| line(notice.text.clone(), notice.color))
    }

    /// The right side: the chosen skill or script, or a word on choosing one.
    fn render_chosen(&self, cx: &Context<Self>) -> AnyElement {
        let shown = match &self.chosen {
            Some(Chosen::Skill(name)) => self.skill(name).map(|skill| self.render_skill(skill, cx)),
            Some(Chosen::Script(path)) => self
                .script(path)
                .map(|script| self.render_script(script, cx)),
            None => None,
        };
        shown.unwrap_or_else(|| {
            line(
                "Choose a skill or a script on the left, or make a skill with New Skill.",
                Color::Muted,
            )
        })
    }
}

impl Focusable for BrainSkillsView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for BrainSkillsView {}

impl Render for BrainSkillsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // A change announced while the tab was hidden is read now that it draws (AD-609).
        if self.due == ReadDue::WhenShown && self.reading.is_none() {
            self.read(cx);
        }
        let state = self.render_state(cx);
        let listing = super::is_on(cx) && super::is_connected(cx) && self.listed.is_some();
        v_flex()
            .key_context("RustySkills")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                v_flex()
                    .px_4()
                    .pt_3()
                    .pb_2()
                    .gap_1()
                    .border_b_1()
                    .border_color(cx.theme().colors().border_variant)
                    .child(Label::new("Skills").size(LabelSize::Large))
                    .child(
                        Label::new(
                            "Rusty's skills store: staged skills wait for your approval; Claude \
                             Code loads the active ones.",
                        )
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    )
                    .children(state),
            )
            .when(listing, |tab| {
                tab.child(
                    h_flex()
                        .flex_1()
                        .min_h_0()
                        .items_start()
                        .child(self.render_lists(cx))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .h_full()
                                .p_4()
                                .child(self.render_chosen(cx)),
                        ),
                )
            })
    }
}

impl Item for BrainSkillsView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Skills")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ToolHammer))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static("Rusty's skills and scripts"))
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

/// New Skill's form: a name Rusty checks, a description, a body, and whether it waits for
/// approval.
pub(super) struct NewSkillForm {
    tab: WeakEntity<BrainSkillsView>,
    name: Entity<Editor>,
    description: Entity<Editor>,
    body: Entity<Editor>,
    staged: bool,
    sending: Option<Task<()>>,
    /// Rusty's words when it refused the last Create.
    refusal: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

/// The form's width in rems, #660's form's.
const FORM_WIDTH: f32 = 34.;

impl NewSkillForm {
    fn new(tab: WeakEntity<BrainSkillsView>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("tidy-commits", window, cx);
            editor
        });
        let description = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("When an agent should use it", window, cx);
            editor
        });
        let body = cx.new(|cx| {
            let mut editor = Editor::auto_height(4, 12, window, cx);
            editor.set_text("# Skill\n\nSteps the agent follows.\n", window, cx);
            editor
        });
        // Create's state follows the name's words, not its caret's blink.
        let subscriptions = vec![cx.subscribe(&name, |_, _, event: &EditorEvent, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                cx.notify();
            }
        })];
        Self {
            tab,
            name,
            description,
            body,
            staged: false,
            sending: None,
            refusal: None,
            _subscriptions: subscriptions,
        }
    }

    fn create(&mut self, window: &Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).text(cx);
        if self.sending.is_some() || name.trim().is_empty() {
            return;
        }
        let write = SkillWrite::Create {
            name,
            description: self.description.read(cx).text(cx),
            body: self.body.read(cx).text(cx),
            staged: self.staged,
        };
        self.refusal = None;
        let asking = super::call_tool(write.tool(), write.arguments(), cx);
        self.sending = Some(cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await.and_then(|text| {
                serde_json::from_str::<Skill>(&text)
                    .map_err(|error| format!("{}'s answer did not parse: {error}", write.tool()))
            });
            this.update_in(cx, |this, window, cx| {
                this.sending = None;
                match answer {
                    Ok(skill) => {
                        this.tab
                            .update(cx, |tab, cx| tab.created(&skill, window, cx))
                            .log_err();
                        cx.emit(DismissEvent);
                    }
                    Err(error) => {
                        let line = error.lines().next().unwrap_or_default().to_string();
                        this.refusal = Some(line.into());
                    }
                }
                cx.notify();
            })
            .log_err();
        }));
        cx.notify();
    }
}

impl ModalView for NewSkillForm {}

impl EventEmitter<DismissEvent> for NewSkillForm {}

impl Focusable for NewSkillForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.name.focus_handle(cx)
    }
}

impl Render for NewSkillForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sending = self.sending.is_some();
        let named = !self.name.read(cx).text(cx).trim().is_empty();
        v_flex()
            .key_context("RustyNewSkill menu")
            .w(rems(FORM_WIDTH))
            .p_3()
            .gap_2()
            .elevation_3(cx)
            .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| this.create(window, cx)))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .child(Label::new("New Skill"))
            .child(line(
                "Name: lowercase letters, digits and hyphens; it is how agents call it",
                Color::Muted,
            ))
            .child(field(&self.name, cx))
            .child(line("Description", Color::Muted))
            .child(field(&self.description, cx))
            .child(line("Body", Color::Muted))
            .child(field(&self.body, cx))
            .child(
                Checkbox::new("rusty-new-skill-staged", ToggleState::from(self.staged))
                    .label("Stage for approval instead of making it active")
                    .label_size(LabelSize::Small)
                    .on_click(cx.listener(|this, _: &ToggleState, _, cx| {
                        this.staged = !this.staged;
                        cx.notify();
                    })),
            )
            .children(self.refusal.clone().map(|refusal| {
                Label::new(refusal)
                    .size(LabelSize::Small)
                    .color(Color::Error)
            }))
            .child(
                h_flex().justify_end().child(
                    Button::new(
                        "rusty-new-skill-create",
                        if sending { "Creating…" } else { "Create" },
                    )
                    .style(ButtonStyle::Filled)
                    .disabled(sending || !named)
                    .on_click(cx.listener(|this, _, window, cx| this.create(window, cx))),
                ),
            )
    }
}
