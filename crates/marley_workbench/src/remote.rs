//! Remote terminals that survive a dropped link (#543).
//!
//! `marley: open remote terminal` lists the SSH hosts saved in Zed's settings and opens a Zed task
//! terminal running `marley_remote::remote_terminal_command`: ssh into a tmux session of Marley's
//! own on the host. When the link drops, the shell and any agent in it keep running there; the
//! task's tab shows it ended, and Zed's Rerun runs the same argv in the same terminal, which
//! attaches the same session again. The frames Claude Code's hooks send from inside the session
//! reach the terminal through tmux's passthrough; since the terminal's foreground program is ssh,
//! `is_remote` is what lets them in.
//!
//! A link that dies is known as dead (#641). ssh's keepalive ends a silent link within 20 s, and
//! when a remote terminal's ssh ends with ssh's own error status, 255, the terminal turns down: it
//! keeps its last screen, dimmed under a line that names the host and when Marley checks next, and
//! its input is held and counted, never sent later. Marley checks the host with a `BatchMode` ssh
//! on a backoff from 1 s to two minutes, and once the host answers reruns the task in the same
//! terminal with `reveal: Never`, so focus and tabs stay where the user left them. Any other end
//! is the remote command's own, such as the user's `exit`, and the terminal ends as before.

use std::borrow::Cow;
use std::collections::HashMap;
use std::process::ExitStatus;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use fuzzy::{StringMatch, StringMatchCandidate};
use gpui::{
    AnyElement, App, AppContext as _, AsyncWindowContext, Context, DismissEvent, Entity, EntityId,
    EventEmitter, FocusHandle, Focusable, Global, Render, SharedString, Task, WeakEntity, Window,
};
use marley_remote::{
    LinkCheck, STABLE_LINK, SessionName, SshTarget, check_delay, link_check_command,
    read_link_check, remote_terminal_command, saved_target,
};
use picker::{Picker, PickerDelegate};
use project::TaskSourceKind;
use recent_projects::RemoteSettings;
use settings::Settings as _;
use task::{RevealStrategy, RevealTarget, SpawnInTerminal, TaskContext, TaskId, TaskTemplate};
use terminal::Terminal;
use terminal_view::MarleyFooterContext;
use terminal_view::terminal_panel::TerminalPanel;
use ui::{HighlightedLabel, IconName, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use util::shell::ShellKind;
use workspace::notifications::DetachAndPromptErr as _;
use workspace::{ModalView, Workspace};

use crate::OpenRemoteTerminal;

/// The id base of a remote terminal's task, which [`is_remote`] looks for.
const ID_BASE: &str = "marley-remote";

/// ssh's exit status for an error of its own: a remote terminal's ssh that ends with it lost its
/// link (#641).
const LINK_LOST: i32 = 255;

/// How often a down terminal's line is drawn again, for its countdown and its count.
const TICK: Duration = Duration::from_secs(1);

/// Marley's remote terminals and their links (#641).
#[derive(Default)]
struct Links {
    /// Each remote task's host, as `open` named it, by task id.
    hosts: HashMap<TaskId, Host>,
    /// Each remote task's backoff, by task id.
    backoff: HashMap<TaskId, Backoff>,
    /// The terminals whose link is down, by the ended terminal's entity.
    down: HashMap<EntityId, Down>,
}

impl Global for Links {}

/// How a remote task's checks back off: the checks that failed since its link was last stable,
/// and when its link last came up.
#[derive(Default)]
struct Backoff {
    failures: u32,
    up_since: Option<Instant>,
}

/// A remote terminal whose link is down.
struct Down {
    task: TaskId,
    host: SharedString,
    phase: Phase,
    /// What ssh said when the last check found the host still down.
    reason: Option<String>,
    /// The checks, ended with the entry.
    _checks: Task<()>,
    /// The line drawn again each second, for its countdown and its count, whatever the checks
    /// are doing: a check can take its whole connect timeout.
    _redraws: Task<()>,
}

/// Where a down terminal's checks are.
enum Phase {
    Waiting {
        until: Instant,
    },
    Checking,
    /// The checks stopped, with ssh's words; the user's Rerun logs in by hand.
    Stopped(String),
}

/// Registers `marley: open remote terminal` on every workspace.
pub fn init(cx: &mut App) {
    cx.set_global(Links::default());
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(open_remote_terminal);
    })
    .detach();
}

/// Whether `terminal` is a remote terminal Marley opened: its task was resolved with Marley's id
/// base.
pub(crate) fn is_remote(terminal: &Terminal) -> bool {
    terminal
        .task()
        .is_some_and(|task| task.spawned_task.id.0.starts_with(ID_BASE))
}

fn open_remote_terminal(
    workspace: &mut Workspace,
    _: &OpenRemoteTerminal,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let hosts = saved_hosts(cx);
    let handle = workspace.weak_handle();
    workspace.toggle_modal(window, cx, |window, cx| {
        RemotePicker::new(handle, hosts, window, cx)
    });
}

/// A saved host the picker lists: its nickname or host, and where ssh goes.
#[derive(Clone)]
struct Host {
    label: SharedString,
    target: SshTarget,
}

/// The SSH hosts saved in Zed's settings whose host, user and port pass `parse_ssh_target`'s
/// checks; an entry that fails them is left out, so nothing in it reaches ssh.
fn saved_hosts(cx: &App) -> Vec<Host> {
    RemoteSettings::get_global(cx)
        .ssh_connections()
        .filter_map(|connection| {
            let target = saved_target(
                &connection.host,
                connection.username.as_deref(),
                connection.port,
            )?;
            let label = connection.nickname.unwrap_or(connection.host);
            Some(Host {
                label: label.into(),
                target,
            })
        })
        .collect()
}

/// Opens a remote terminal on `host` in a session of its own, as a task Rerun can run again.
///
/// # Errors
///
/// When the project is not on this machine, where a task would run ssh on the project's host, or
/// a word cannot be quoted for the shell.
fn open(
    host: &Host,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        workspace.project().read(cx).is_local(),
        "A remote terminal opens from a project on this machine"
    );
    let session = SessionName::from_bits(rand::random());
    let mut words = remote_terminal_command(&host.target, &session).into_iter();
    let ssh = words.next().unwrap_or_else(|| "ssh".to_string());
    // A scenario's fake must not lose to the real ssh on a PATH the login shell set.
    let command = std::env::var("MARLEY_SSH").unwrap_or(ssh);
    // A task's command and arguments reach its shell as shell text, unquoted
    // (`ShellBuilder::build_no_quote`), so each word is quoted for that shell.
    let shell = ShellKind::system();
    let quote = |word: String| {
        shell
            .try_quote(&word)
            .map(Cow::into_owned)
            .with_context(|| format!("{word:?} cannot be quoted for the shell"))
    };
    let template = TaskTemplate {
        label: format!("{} · {}", host.label, session.as_str()),
        command: quote(command)?,
        args: words.map(quote).collect::<anyhow::Result<_>>()?,
        reveal_target: RevealTarget::Center,
        use_new_terminal: false,
        allow_concurrent_runs: false,
        show_summary: true,
        show_command: false,
        ..TaskTemplate::default()
    };
    let resolved = template
        .resolve_task(ID_BASE, &TaskContext::default())
        .context("the remote terminal's task did not resolve")?;
    // The host is checked by its target when the link drops (#641).
    cx.default_global::<Links>()
        .hosts
        .insert(resolved.id.clone(), host.clone());
    workspace.schedule_resolved_task(TaskSourceKind::UserInput, resolved, false, window, cx);
    Ok(())
}

/// Whether `task` is a remote terminal's.
pub(crate) fn is_remote_task(task: &SpawnInTerminal) -> bool {
    task.id.0.starts_with(ID_BASE)
}

/// A remote task starts, from the user or a reattach: its link counts as up from now, and the
/// checks of its terminals that are down end, since the run replaces them (#641).
pub(crate) fn starting(task: &TaskId, cx: &mut App) {
    let links = cx.default_global::<Links>();
    links.backoff.entry(task.clone()).or_default().up_since = Some(Instant::now());
    links.down.retain(|_, down| &down.task != task);
}

/// What follows a remote terminal's run (#641). An ssh that ended with its own error status lost
/// its link: the terminal turns down and its host is checked until it answers. Any other end is
/// the remote command's own, and the backoff starts over.
pub(crate) fn supervise(
    task: SpawnInTerminal,
    terminal: &Entity<Terminal>,
    status: Option<ExitStatus>,
    panel: Entity<TerminalPanel>,
    window: &Window,
    cx: &mut App,
) {
    let links = cx.default_global::<Links>();
    if status.and_then(|status| status.code()) != Some(LINK_LOST) {
        links.backoff.remove(&task.id);
        return;
    }
    let Some(host) = links.hosts.get(&task.id).cloned() else {
        return;
    };
    let backoff = links.backoff.entry(task.id.clone()).or_default();
    if backoff
        .up_since
        .is_some_and(|up| up.elapsed() >= STABLE_LINK)
    {
        backoff.failures = 0;
    }
    let until = Instant::now() + check_delay(backoff.failures);
    let key = terminal.entity_id();
    let (task_id, label) = (task.id.clone(), host.label.clone());
    // The checks start by waiting on the clock, so the entry they read is in place first.
    let weak = terminal.downgrade();
    let checks = window.spawn(cx, async move |cx| {
        check_until_up(task, host, key, weak, panel, cx).await;
    });
    let weak = terminal.downgrade();
    let redraws = window.spawn(cx, async move |cx| {
        loop {
            cx.background_executor().timer(TICK).await;
            if weak.update(cx, |_, cx| cx.notify()).is_err() {
                return;
            }
        }
    });
    cx.default_global::<Links>().down.insert(
        key,
        Down {
            task: task_id,
            host: label,
            phase: Phase::Waiting { until },
            reason: None,
            _checks: checks,
            _redraws: redraws,
        },
    );
    terminal.update(cx, |terminal, cx| {
        terminal.marley_hold_input();
        cx.notify();
    });
    // A Rerun gives the view a new terminal and a closed tab drops it: either way the checks end.
    cx.observe_release(terminal, move |_, cx| {
        cx.default_global::<Links>().down.remove(&key);
    })
    .detach();
}

/// Checks `host` on the backoff while the terminal `key` is down, and reruns the task once the
/// host answers; a check that stops leaves the line saying why.
async fn check_until_up(
    task: SpawnInTerminal,
    host: Host,
    key: EntityId,
    terminal: WeakEntity<Terminal>,
    panel: Entity<TerminalPanel>,
    cx: &mut AsyncWindowContext,
) {
    let mut words = link_check_command(&host.target).into_iter();
    let ssh = words.next().unwrap_or_else(|| "ssh".to_string());
    // A scenario's wrapper, as `open` honors it.
    let program = std::env::var("MARLEY_SSH").unwrap_or(ssh);
    let args: Vec<String> = words.collect();
    loop {
        let waiting = cx.update(|_, cx| match phase_of(key, cx) {
            Some(Phase::Waiting { until }) => Some(*until),
            _ => None,
        });
        let Ok(Some(until)) = waiting else {
            return;
        };
        let now = Instant::now();
        if now < until {
            cx.background_executor().timer(until - now).await;
        }
        set_phase(key, Phase::Checking, &terminal, cx);
        let found = match crate::process::output(&program, &args, None, &[]).await {
            Ok(output) => read_link_check(
                output.status.code(),
                &String::from_utf8_lossy(&output.stderr),
            ),
            Err(error) => LinkCheck::Stopped(format!("ssh did not start: {error}")),
        };
        match found {
            LinkCheck::Answers => {
                reattach(task, key, panel, cx);
                return;
            }
            LinkCheck::Down(reason) => {
                let next = cx.update(|_, cx| {
                    let links = cx.default_global::<Links>();
                    let backoff = links.backoff.entry(task.id.clone()).or_default();
                    backoff.failures = backoff.failures.saturating_add(1);
                    let until = Instant::now() + check_delay(backoff.failures);
                    if let Some(down) = links.down.get_mut(&key) {
                        down.reason = Some(reason);
                    }
                    until
                });
                let Ok(until) = next else {
                    return;
                };
                set_phase(key, Phase::Waiting { until }, &terminal, cx);
            }
            LinkCheck::Stopped(reason) => {
                set_phase(key, Phase::Stopped(reason), &terminal, cx);
                return;
            }
        }
    }
}

/// Reruns `task` in its terminal now that its host answers, without moving the focus, and
/// follows the new run. The follow is a task of its own, since the old terminal's release, when
/// the rerun replaces it, ends the checks that start it.
fn reattach(
    task: SpawnInTerminal,
    key: EntityId,
    panel: Entity<TerminalPanel>,
    cx: &mut AsyncWindowContext,
) {
    let rerun = SpawnInTerminal {
        reveal: RevealStrategy::Never,
        ..task.clone()
    };
    let spawned = cx.update(|window, cx| {
        let links = cx.default_global::<Links>();
        links.backoff.entry(task.id.clone()).or_default().up_since = Some(Instant::now());
        panel.update(cx, |panel, cx| panel.spawn_task(&rerun, window, cx))
    });
    let Ok(spawned) = spawned else {
        return;
    };
    cx.spawn(async move |cx| {
        let terminal = match spawned.await {
            Ok(terminal) => terminal,
            Err(error) => {
                let reason = format!("Marley could not rerun the terminal: {error}");
                cx.update(|_, cx| {
                    if let Some(down) = cx.default_global::<Links>().down.get_mut(&key) {
                        down.phase = Phase::Stopped(reason);
                    }
                })
                .log_err();
                return;
            }
        };
        cx.update(|_, cx| cx.default_global::<Links>().down.remove(&key))
            .log_err();
        let Ok(completed) = terminal.read_with(cx, Terminal::wait_for_completed_task) else {
            return;
        };
        let status = completed.await;
        let Some(terminal) = terminal.upgrade() else {
            return;
        };
        cx.update(|window, cx| supervise(task, &terminal, status, panel, window, cx))
            .log_err();
    })
    .detach();
}

/// The phase of the down terminal `key`, if it is down.
fn phase_of(key: EntityId, cx: &App) -> Option<&Phase> {
    cx.try_global::<Links>()?
        .down
        .get(&key)
        .map(|down| &down.phase)
}

/// Moves the down terminal `key` to `phase` and draws its line again.
fn set_phase(
    key: EntityId,
    phase: Phase,
    terminal: &WeakEntity<Terminal>,
    cx: &mut AsyncWindowContext,
) {
    cx.update(|_, cx| {
        if let Some(down) = cx.default_global::<Links>().down.get_mut(&key) {
            down.phase = phase;
        }
    })
    .log_err();
    terminal.update(cx, |_, cx| cx.notify()).log_err();
}

/// The line over a down terminal, dimming its last screen and catching the mouse, for the overlay
/// hook (#641): the host, why its link is down, when Marley checks next, and what the terminal
/// refused.
pub(crate) fn link_overlay(context: &MarleyFooterContext, cx: &App) -> Option<AnyElement> {
    let down = cx
        .try_global::<Links>()?
        .down
        .get(&context.terminal.entity_id())?;
    let lost = down.reason.as_ref().map_or_else(
        || format!("Link to {} lost", down.host),
        |reason| format!("Link to {} lost: {reason}", down.host),
    );
    let line = match &down.phase {
        Phase::Waiting { until } => {
            let seconds = until
                .saturating_duration_since(Instant::now())
                .as_millis()
                .div_ceil(1000);
            format!("{lost} · next check in {seconds} s · Rerun to try now")
        }
        Phase::Checking => format!("{lost} · checking now · Rerun to try now"),
        Phase::Stopped(reason) => {
            // ssh ends its own sentences.
            let reason = reason.trim_end_matches('.');
            format!(
                "Not reconnecting to {}: {reason}. Rerun to log in.",
                down.host
            )
        }
    };
    let refused = context
        .terminal
        .read(cx)
        .marley_inputs_refused()
        .unwrap_or(0);
    let line = if refused > 0 {
        format!("{line} · {refused} not sent")
    } else {
        line
    };
    let colors = cx.theme().colors();
    Some(
        div()
            .id("marley-remote-link-down")
            .absolute()
            .inset_0()
            .occlude()
            .bg(colors.editor_background.opacity(0.6))
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .bg(colors.elevated_surface_background)
                    .border_b_1()
                    .border_color(colors.border)
                    .child(
                        Icon::new(IconName::Warning)
                            .size(IconSize::Small)
                            .color(Color::Warning),
                    )
                    .child(Label::new(line).size(LabelSize::Small)),
            )
            .into_any_element(),
    )
}

/// The picker of saved hosts.
struct RemotePicker {
    picker: Entity<Picker<RemoteDelegate>>,
}

impl RemotePicker {
    fn new(
        workspace: WeakEntity<Workspace>,
        hosts: Vec<Host>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let delegate = RemoteDelegate {
            modal: cx.entity().downgrade(),
            workspace,
            hosts,
            matches: Vec::new(),
            selected_index: 0,
        };
        Self {
            picker: cx.new(|cx| Picker::uniform_list(delegate, window, cx)),
        }
    }
}

impl ModalView for RemotePicker {}

impl EventEmitter<DismissEvent> for RemotePicker {}

impl Focusable for RemotePicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl Render for RemotePicker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("RemoteTerminalPicker")
            .w(rems(34.))
            .child(self.picker.clone())
    }
}

struct RemoteDelegate {
    modal: WeakEntity<RemotePicker>,
    workspace: WeakEntity<Workspace>,
    hosts: Vec<Host>,
    /// The hosts the query matches, in order, each naming its host by index.
    matches: Vec<StringMatch>,
    selected_index: usize,
}

impl PickerDelegate for RemoteDelegate {
    type ListItem = ListItem;

    fn name() -> &'static str {
        "marley remote terminal"
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: usize, _: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected_index = index;
    }

    fn placeholder_text(&self, _: &mut Window, _: &mut App) -> Arc<str> {
        "Open a terminal on a saved SSH host…".into()
    }

    fn no_matches_text(&self, _: &mut Window, _: &mut App) -> Option<SharedString> {
        Some(if self.hosts.is_empty() {
            SharedString::new_static("No SSH hosts saved in settings (`ssh_connections`)")
        } else {
            SharedString::new_static("No saved host matches")
        })
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let candidates: Vec<StringMatchCandidate> = self
            .hosts
            .iter()
            .enumerate()
            .map(|(index, host)| StringMatchCandidate::new(index, &host.label))
            .collect();
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |picker, cx| {
            // An empty query keeps the hosts in the settings' order.
            let matches = if query.is_empty() {
                candidates
                    .into_iter()
                    .map(|candidate| StringMatch {
                        candidate_id: candidate.id,
                        score: 0.,
                        positions: Vec::new(),
                        string: candidate.string,
                    })
                    .collect()
            } else {
                let cancelled = AtomicBool::new(false);
                fuzzy::match_strings(&candidates, &query, false, true, 100, &cancelled, executor)
                    .await
            };
            let update = |picker: &mut Picker<Self>, cx: &mut Context<Picker<Self>>| {
                picker.delegate.matches = matches;
                picker.delegate.selected_index = 0;
                cx.notify();
            };
            picker.update(cx, update).log_err();
        })
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        let host = self
            .matches
            .get(self.selected_index)
            .and_then(|found| self.hosts.get(found.candidate_id));
        if let Some(host) = host {
            let opened = self
                .workspace
                .update(cx, |workspace, cx| open(host, workspace, window, cx))
                .and_then(|opened| opened);
            Task::ready(opened).detach_and_prompt_err(
                "Could not open the remote terminal",
                window,
                cx,
                |_, _, _| None,
            );
        }
        self.dismissed(window, cx);
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        self.modal
            .update(cx, |_, cx| cx.emit(DismissEvent))
            .log_err();
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _: &mut Window,
        _: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let found = self.matches.get(index)?;
        let host = self.hosts.get(found.candidate_id)?;
        let destination = host.target.user.as_ref().map_or_else(
            || host.target.host.clone(),
            |user| format!("{user}@{}", host.target.host),
        );
        Some(
            ListItem::new(index)
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .start_slot(Icon::new(IconName::Server).color(Color::Muted))
                .child(HighlightedLabel::new(
                    host.label.clone(),
                    found.positions.clone(),
                ))
                .end_slot(
                    Label::new(destination)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
    }
}
