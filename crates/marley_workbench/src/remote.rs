//! Remote terminals that survive a dropped link (#543).
//!
//! `marley: open remote terminal` lists the SSH hosts saved in Zed's settings and opens a Zed task
//! terminal running `marley_remote::remote_terminal_command`: ssh into a tmux session of Marley's
//! own on the host. When the link drops, the shell and any agent in it keep running there; the
//! task's tab shows it ended, and Zed's Rerun runs the same argv in the same terminal, which
//! attaches the same session again. The frames Claude Code's hooks send from inside the session
//! reach the terminal through tmux's passthrough; since the terminal's foreground program is ssh,
//! `is_remote` is what lets them in.

use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use anyhow::Context as _;
use fuzzy::{StringMatch, StringMatchCandidate};
use gpui::{
    App, AppContext as _, Context, DismissEvent, EventEmitter, FocusHandle, Focusable, Render,
    SharedString, Task, WeakEntity, Window,
};
use marley_remote::{SessionName, SshTarget, remote_terminal_command, saved_target};
use picker::{Picker, PickerDelegate};
use project::TaskSourceKind;
use recent_projects::RemoteSettings;
use settings::Settings as _;
use task::{RevealTarget, TaskContext, TaskTemplate};
use terminal::Terminal;
use ui::{HighlightedLabel, IconName, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use util::shell::ShellKind;
use workspace::notifications::DetachAndPromptErr as _;
use workspace::{ModalView, Workspace};

use crate::OpenRemoteTerminal;

/// The id base of a remote terminal's task, which [`is_remote`] looks for.
const ID_BASE: &str = "marley-remote";

/// Registers `marley: open remote terminal` on every workspace.
pub fn init(cx: &App) {
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
    workspace.schedule_resolved_task(TaskSourceKind::UserInput, resolved, false, window, cx);
    Ok(())
}

/// The picker of saved hosts.
struct RemotePicker {
    picker: gpui::Entity<Picker<RemoteDelegate>>,
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
