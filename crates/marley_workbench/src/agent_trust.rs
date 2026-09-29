//! Claude Code's question whether to trust a folder, in a worktree New Agent in Worktree made
//! (#587), brought to the user.
//!
//! Claude Code asks it before it loads a folder's configuration, for a repository it has not
//! trusted yet, and a worktree agent's workspace opens in the background (#510), so the question
//! would wait in a terminal nobody sees. For a minute after the launch, Marley reads the last
//! lines of the agent's screen as it draws; while the question shows, a notification in every
//! workspace names it, with Trust Folder, which picks the trust option by name, and Show
//! Terminal. Under `claude_code_worktree_trust: "follow_zed"` Marley answers it itself when Zed
//! trusts the worktree's folder. Marley never writes Claude Code's own record of trusted folders:
//! the question stays Claude Code's, and Marley types only an answer.

use std::collections::HashMap;
use std::time::Duration;

use gpui::{
    App, AppContext as _, Context, Entity, EntityId, Global, Subscription, Task, WeakEntity,
};
use marley_agent::trust::{self, TrustQuestion};
use project::trusted_worktrees::TrustedWorktrees;
use settings::{ClaudeCodeWorktreeTrust, Settings as _};
use terminal::{RenderableCells, Terminal};
use terminal_view::TerminalView;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, dismiss_app_notification, show_app_notification};
use workspace::{MultiWorkspace, Toast, Workspace};

use crate::MarleySettings;

/// How long after the launch the screen is read for the question: Claude Code draws it within
/// seconds, before anything else.
const WATCH_FOR: Duration = Duration::from_secs(60);

/// The least time between two reads of the screen, as the served-URL scan waits.
const READ_EVERY: Duration = Duration::from_millis(500);

/// How long after Marley's keys the question must be gone.
const ANSWER_TAKES: Duration = Duration::from_millis(1500);

/// How many of the screen's last lines are read: the whole question, with its warnings.
const SCREEN_LINES: usize = 24;

/// The terminals being watched, by their terminal's id.
#[derive(Default)]
struct TrustWatches(HashMap<EntityId, Entity<TrustWatch>>);

impl Global for TrustWatches {}

/// One Claude Code terminal New Agent in Worktree started, read until its question has come and
/// gone, the minute is over, or it closes.
struct TrustWatch {
    id: EntityId,
    terminal: WeakEntity<Terminal>,
    /// The worktree's workspace, where the terminal is.
    worktree: WeakEntity<Workspace>,
    /// The workspace that started the agent, where the note of an answer goes.
    source: WeakEntity<Workspace>,
    /// The worktree's name, as the rail shows it.
    name: String,
    state: State,
    read: Option<Task<()>>,
    check: Option<Task<()>>,
    _deadline: Task<()>,
    _output: Subscription,
    _closed: Subscription,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// The question has not shown.
    Watching,
    /// It shows, and Marley has sent its keys, once, or not yet.
    Asking { answered: bool },
}

/// Watches the terminal of a Claude Code that New Agent in Worktree started in the worktree
/// `name`, whose workspace is `worktree`, from `source`.
pub(crate) fn watch(
    terminal: &WeakEntity<Terminal>,
    worktree: WeakEntity<Workspace>,
    source: WeakEntity<Workspace>,
    name: String,
    cx: &mut App,
) {
    let Some(terminal) = terminal.upgrade() else {
        return;
    };
    let id = terminal.entity_id();
    let watch = cx.new(|cx: &mut Context<TrustWatch>| {
        let output = cx.subscribe(&terminal, |watch, _, event: &terminal::Event, cx| {
            if matches!(event, terminal::Event::Wakeup) {
                watch.read_soon(cx);
            }
        });
        let closed = cx.observe_release(&terminal, move |_, _, cx| finish(id, cx));
        let deadline = cx.spawn(async move |watch: WeakEntity<TrustWatch>, cx| {
            cx.background_executor().timer(WATCH_FOR).await;
            watch
                .update(cx, |watch, cx| {
                    if watch.state == State::Watching {
                        finish(watch.id, cx);
                    }
                })
                .log_err();
        });
        let mut watch = TrustWatch {
            id,
            terminal: terminal.downgrade(),
            worktree,
            source,
            name,
            state: State::Watching,
            read: None,
            check: None,
            _deadline: deadline,
            _output: output,
            _closed: closed,
        };
        // The question may be on the screen already.
        watch.read_soon(cx);
        watch
    });
    cx.default_global::<TrustWatches>().0.insert(id, watch);
}

impl TrustWatch {
    /// Reads the screen a moment from now, unless a read is due already.
    fn read_soon(&mut self, cx: &Context<Self>) {
        if self.read.is_some() {
            return;
        }
        self.read = Some(cx.spawn(async move |watch, cx| {
            cx.background_executor().timer(READ_EVERY).await;
            watch
                .update(cx, |watch, cx| {
                    watch.read = None;
                    watch.read_screen(cx);
                })
                .log_err();
        }));
    }

    /// Asks once the question shows, and ends the watch once it has gone.
    fn read_screen(&mut self, cx: &mut Context<Self>) {
        match (self.state, self.question(cx)) {
            (State::Watching, Some(question)) => {
                self.state = State::Asking { answered: false };
                self.ask(&question, cx);
            }
            (State::Asking { .. }, None) => finish(self.id, cx),
            _ => {}
        }
    }

    /// The question the terminal's screen shows, if it shows it. The question's lines come with
    /// their wrapped rows joined; its footer must be on a row of the screen itself.
    fn question(&self, cx: &App) -> Option<TrustQuestion> {
        let terminal = self.terminal.upgrade()?;
        let terminal = terminal.read(cx);
        let rows = terminal.with_renderable_cells(screen_rows);
        if !trust::footer_on_screen(&rows) {
            return None;
        }
        trust::read(&terminal.last_n_non_empty_lines(SCREEN_LINES))
    }

    /// Answers the question itself when the user's setting says to and Zed trusts the worktree's
    /// folder, and asks the user otherwise.
    fn ask(&mut self, question: &TrustQuestion, cx: &mut Context<Self>) {
        let follow = MarleySettings::get_global(cx).claude_code_worktree_trust
            == ClaudeCodeWorktreeTrust::FollowZed;
        if follow && self.zed_trusts(cx) && self.answer(cx) {
            let message = format!(
                "Claude Code in {} asked whether to trust {}; Marley answered yes, since Zed \
                 trusts it.",
                self.name,
                self.folder(question)
            );
            // An id of its own: Zed shows a toast as a notification under its id, and the
            // question's goes when the question does. The note stays until it is closed: Marley
            // trusted a folder for the user.
            self.source
                .update(cx, |workspace, cx| {
                    workspace.show_toast(Toast::new(answered_id(self.id), message), cx);
                })
                .log_err();
            return;
        }
        self.notify(question, cx);
    }

    /// Whether Zed trusts the worktree's folder, which its worktree service carried over from
    /// the main checkout when it made the worktree.
    fn zed_trusts(&self, cx: &mut App) -> bool {
        let Some(workspace) = self.worktree.upgrade() else {
            return false;
        };
        let project = workspace.read(cx).project().clone();
        let (store, worktree) = {
            let project = project.read(cx);
            let worktree = project
                .visible_worktrees(cx)
                .next()
                .map(|worktree| worktree.read(cx).id());
            (project.worktree_store(), worktree)
        };
        let (Some(worktree), Some(trusted)) = (worktree, TrustedWorktrees::try_get_global(cx))
        else {
            return false;
        };
        trusted.update(cx, |trusted, cx| trusted.can_trust(&store, worktree, cx))
    }

    /// Sends the keys that pick the trust option, once, while the screen still shows the
    /// question, and checks a moment later that it has gone. Whether it sent them.
    fn answer(&mut self, cx: &mut Context<Self>) -> bool {
        if self.state != (State::Asking { answered: false }) {
            return false;
        }
        let Some(keys) = self
            .question(cx)
            .and_then(|question| question.answer_keys())
        else {
            return false;
        };
        let Some(terminal) = self.terminal.upgrade() else {
            return false;
        };
        terminal.update(cx, |terminal, _| terminal.input(keys));
        self.state = State::Asking { answered: true };
        self.check = Some(cx.spawn(async move |watch, cx| {
            cx.background_executor().timer(ANSWER_TAKES).await;
            watch
                .update(cx, |watch, cx| {
                    if watch.question(cx).is_some() {
                        watch.notify_unanswered(cx);
                    } else {
                        finish(watch.id, cx);
                    }
                })
                .log_err();
        }));
        true
    }

    /// The folder the question names, or the worktree's.
    fn folder(&self, question: &TrustQuestion) -> String {
        question
            .folder
            .clone()
            .unwrap_or_else(|| format!("the worktree {}", self.name))
    }

    /// The notification in every workspace: the question, its warnings, Trust Folder when the
    /// screen gives both options, and Show Terminal.
    fn notify(&self, question: &TrustQuestion, cx: &mut App) {
        let id = self.id;
        let headline = SharedString::from(format!(
            "Claude Code in {} is asking whether to trust {}.",
            self.name,
            self.folder(question)
        ));
        let warnings: Vec<SharedString> = question
            .warnings
            .iter()
            .map(|warning| SharedString::from(warning.clone()))
            .collect();
        let answerable = question.answer_keys().is_some();
        show_app_notification(notification_id(id), cx, move |cx| {
            let headline = headline.clone();
            let warnings = warnings.clone();
            cx.new(|cx| {
                MessageNotification::new_from_builder(cx, move |_, _| {
                    v_flex()
                        .gap_1()
                        .child(Label::new(headline.clone()))
                        .children(warnings.iter().map(|warning| {
                            Label::new(warning.clone())
                                .size(LabelSize::Small)
                                .color(Color::Warning)
                        }))
                        .into_any_element()
                })
                .when(answerable, |notification| {
                    notification
                        .primary_message("Trust Folder")
                        .primary_on_click(move |_, cx| trust_clicked(id, cx))
                })
                .secondary_message("Show Terminal")
                .secondary_on_click(move |window, cx| show_terminal(id, window, cx))
            })
        });
    }

    /// The notification once Marley cannot answer: the question stayed after its keys, or the
    /// screen does not give both options. It sends no more keys.
    fn notify_unanswered(&self, cx: &mut App) {
        let id = self.id;
        let headline = SharedString::from(format!(
            "Claude Code in {} is still asking whether to trust its folder: answer it in its \
             terminal.",
            self.name
        ));
        // An id of its own: the notification whose click led here dismisses its own id after
        // the click, which would take this one with it.
        show_app_notification(unanswered_id(id), cx, move |cx| {
            let headline = headline.clone();
            cx.new(|cx| {
                MessageNotification::new(headline, cx)
                    .secondary_message("Show Terminal")
                    .secondary_on_click(move |window, cx| show_terminal(id, window, cx))
            })
        });
    }
}

/// Trust Folder: the answer, while the question still shows; a note to answer it in the
/// terminal when Marley cannot.
fn trust_clicked(id: EntityId, cx: &mut App) {
    let Some(watch) = watch_of(id, cx) else {
        return;
    };
    watch.update(cx, |watch, cx| match watch.question(cx) {
        None => finish(watch.id, cx),
        Some(_) => {
            if !watch.answer(cx) {
                watch.notify_unanswered(cx);
            }
        }
    });
}

/// Show Terminal: the worktree's workspace, with the agent's terminal focused.
fn show_terminal(id: EntityId, window: &mut Window, cx: &mut App) {
    let Some(watch) = watch_of(id, cx) else {
        return;
    };
    let (worktree, terminal) = {
        let watch = watch.read(cx);
        (watch.worktree.upgrade(), watch.terminal.upgrade())
    };
    let (Some(worktree), Some(terminal)) = (worktree, terminal) else {
        return;
    };
    if let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() {
        multi_workspace.update(cx, |multi_workspace, cx| {
            multi_workspace.activate(worktree.clone(), None, window, cx);
        });
    }
    worktree.update(cx, |workspace, cx| {
        let view = workspace
            .items_of_type::<TerminalView>(cx)
            .find(|view| view.read(cx).terminal().entity_id() == terminal.entity_id());
        if let Some(view) = view {
            workspace.activate_item(&view, true, true, window, cx);
        }
    });
}

/// The text of each row the terminal's screen shows, top to bottom.
fn screen_rows(cells: RenderableCells<'_>) -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    let mut line = None;
    for cell in cells {
        if line != Some(cell.point.line) {
            line = Some(cell.point.line);
            rows.push(String::new());
        }
        if !cell.is_wide_char_spacer()
            && let Some(row) = rows.last_mut()
        {
            row.push(cell.character());
        }
    }
    rows
}

fn watch_of(id: EntityId, cx: &App) -> Option<Entity<TrustWatch>> {
    cx.try_global::<TrustWatches>()
        .and_then(|watches| watches.0.get(&id).cloned())
}

/// Ends the watch of the terminal `id`: its notification goes, and the watch after this effect.
fn finish(id: EntityId, cx: &mut App) {
    dismiss_app_notification(&notification_id(id), cx);
    dismiss_app_notification(&unanswered_id(id), cx);
    cx.defer(move |cx| {
        cx.default_global::<TrustWatches>().0.remove(&id);
    });
}

fn notification_id(id: EntityId) -> NotificationId {
    NotificationId::composite::<TrustWatch>(("claude-code-trust", id.as_u64()))
}

fn unanswered_id(id: EntityId) -> NotificationId {
    NotificationId::composite::<TrustWatch>(("claude-code-trust-unanswered", id.as_u64()))
}

fn answered_id(id: EntityId) -> NotificationId {
    NotificationId::composite::<TrustWatch>(("claude-code-trust-answered", id.as_u64()))
}
