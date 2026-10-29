//! Plan review in a tab (#726).
//!
//! While `marley.plan_review` is on, Marley's new local terminals name `ExitPlanMode` in
//! `MARLEY_APPROVE_TOOLS`, and the shared Claude Code plugin (0.4.0 since #776) hands that
//! call to `"$MARLEY_BIN" approve`, which waits on Marley's agent socket for the user. The plan opens
//! as Markdown (HTML not parsed) in a tab beside its terminal; the user selects a passage and
//! comments on it (the right-click menu's Comment…, or `c`), adds an overall note, and answers
//! Approve or Send Feedback, which goes back to the waiting call once. The plan waits in Needs you
//! meanwhile. A request whose caller stops waiting, or whose wait runs out, leaves the tab saying the
//! plan went back to the terminal's own dialog. A later plan from the same terminal replaces the
//! tab's plan and notes. Since #727 the tab keeps each plan as a version: from the second, its
//! header says which, with the lines added and removed since the one before, and Changes shows that
//! diff, drawn by Zed's own diff editor. Since #728 a plan's `:::question` blocks are cards in
//! their place, with choices, Accept recommended and Skip; the answers lead the feedback, and Send
//! Answers takes Approve's place while any is unsent.
//!
//! Since #729 the same tab reviews a Markdown file (`marley: review markdown`) or an agent's last
//! reply (`marley: review last reply`, or Review Last Reply in the rail's menus). Send Notes sends
//! the notes not sent yet: a file's through #522's picker, a reply's back to the agent that wrote
//! it, in its terminal while it is ready or in its thread. Sent notes stay, marked Sent.

use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::prompts::Role;
use agent_ui::{AgentPanel, ConversationView};
use editor::{Editor, EditorEvent, EditorMode};
use gpui::{
    Anchor, AnyWindowHandle, App, AppContext as _, Context, DismissEvent, Entity, EntityId,
    EventEmitter, FocusHandle, Focusable, FontWeight, Global, MouseButton, MouseDownEvent, Pixels,
    Point, SharedString, Stateful, Subscription, WeakEntity, Window, actions, anchored, deferred,
};
use language::LanguageRegistry;
use markdown::{Markdown, MarkdownElement, MarkdownFont, MarkdownStyle};
use markdown_preview::markdown_preview_view::MarkdownPreviewView;
use marley_agent::plan_questions::{self, Answer, PlanPart, Question, QuestionKind};
use marley_agent::report::{self, Refusal};
use marley_agent::{PlanNote, ReviewLine};
use marley_mcp::agent_socket::AgentRequest;
use marley_rail::{InboxEntry, InboxKind, TurnMark};
use serde_json::Value;
use settings::SettingsStore;
use terminal_view::TerminalView;
use ui::{Checkbox, ContextMenu, DiffStat, ToggleState, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

actions!(
    plan_review,
    [
        /// Comments on the plan's selected passage.
        #[derive(Eq)]
        Comment
    ]
);

actions!(
    marley,
    [
        /// Opens the active Markdown file, or the file of the active Markdown preview, in a review
        /// tab: notes on its passages go to an agent working on it.
        #[derive(Eq)]
        ReviewMarkdown,
        /// Opens the last reply of the active agent thread, or of Claude Code in the active
        /// terminal, in a review tab: notes on it go back to that agent.
        #[derive(Eq)]
        ReviewLastReply
    ]
);

/// The tool whose call Marley decides while plan review is on.
const PLAN_TOOL: &str = "ExitPlanMode";

/// What an approve request carries, as `agent_reports::take` hands it over.
pub(crate) struct Approve {
    pub(crate) source: String,
    pub(crate) call: String,
    pub(crate) tool: String,
    pub(crate) input: Value,
}

/// The plans waiting for the user, one per terminal.
#[derive(Default)]
pub(crate) struct PendingPlans {
    plans: Vec<Pending>,
    next_nonce: u64,
    watching: bool,
}

impl Global for PendingPlans {}

/// One plan waiting for the user's answer.
struct Pending {
    /// The terminal view the agent runs in.
    view: EntityId,
    /// The request's own number, so an answer reaches only the request its tab shows.
    nonce: u64,
    since: Instant,
    request: AgentRequest,
    withdrawn: Arc<AtomicBool>,
    tab: WeakEntity<PlanReview>,
    workspace: WeakEntity<Workspace>,
    /// The plan's title, for its Needs you entry (#925, D3).
    title: String,
}

/// A plan's title (#925): its first heading, else its first line, cut to sixty characters.
fn plan_title(plan: &str) -> String {
    let mut lines = plan.lines().map(str::trim).filter(|line| !line.is_empty());
    let heading = plan
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('#'))
        .map(|line| line.trim_start_matches('#').trim());
    let title = heading.or_else(|| lines.next()).unwrap_or_default();
    if title.chars().count() > 60 {
        let cut: String = title.chars().take(59).collect();
        format!("{}\u{2026}", cut.trim_end())
    } else {
        title.to_string()
    }
}

/// Follows the setting; `agent_reports::start` calls it once. Marley's keymap binds `c` to
/// [`Comment`] in the plan's area.
pub fn init(cx: &mut App) {
    cx.set_global(PendingPlans::default());
    reconcile(cx);
    cx.observe_global::<SettingsStore>(reconcile).detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &ReviewMarkdown, window, cx| {
            if let Err(message) = review_markdown(workspace, window, cx) {
                show_toast(workspace, message, cx);
            }
        });
        workspace.register_action(|workspace, _: &ReviewLastReply, window, cx| {
            if let Err(message) = review_last_reply(workspace, window, cx) {
                show_toast(workspace, message, cx);
            }
        });
    })
    .detach();
}

/// Whether `marley.plan_review` is on.
fn on(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.plan_review)
        .unwrap_or(false)
}

/// Names `ExitPlanMode` to the terminals started from now on while the setting is on.
fn reconcile(cx: &mut App) {
    marley_terminal::identity::set_approve_tools(on(cx).then(|| PLAN_TOOL.to_string()));
}

/// Takes an approve from the agent in `view`: the plan opens in its tab and waits for the user; a
/// tool Marley does not decide, or a request while the review is off, goes back to the agent's
/// own dialog.
pub(crate) fn arrived(
    request: AgentRequest,
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    approve: Approve,
    cx: &mut App,
) {
    let Approve {
        source,
        call,
        tool,
        input,
    } = approve;
    if tool != PLAN_TOOL || !on(cx) {
        request.answer(report::answer_refused(
            Refusal::ApproveTool,
            "Marley does not decide this call; the agent's own dialog does",
        ));
        return;
    }
    let Some(plan) = input
        .get("plan")
        .and_then(Value::as_str)
        .filter(|plan| !plan.trim().is_empty())
        .map(str::to_string)
    else {
        request.answer(report::answer_refused(
            Refusal::Shape,
            "the call's input holds no plan",
        ));
        return;
    };
    let Some(window) = crate::browser::window_of(workspace, cx) else {
        request.answer(report::answer_refused(
            Refusal::Unknown,
            "the terminal's window is gone",
        ));
        return;
    };
    log::info!("plan review: {source} asks for a decision on {tool} ({call})");
    let view_id = view.entity_id();
    let nonce = {
        let plans = cx.global_mut::<PendingPlans>();
        plans.next_nonce += 1;
        plans.next_nonce
    };
    let title = plan_title(&plan);
    let opened = window.update(cx, |_, window, cx| {
        open_tab(workspace, view, plan, nonce, window, cx)
    });
    let Ok(tab) = opened else {
        request.answer(report::answer_refused(
            Refusal::Unknown,
            "the terminal's window is gone",
        ));
        return;
    };
    let withdrawn = request.withdrawn();
    let earlier = cx.update_global::<PendingPlans, _>(|plans, _| {
        let earlier = plans
            .plans
            .iter()
            .position(|pending| pending.view == view_id)
            .map(|at| plans.plans.remove(at));
        plans.plans.push(Pending {
            view: view_id,
            nonce,
            since: Instant::now(),
            request,
            withdrawn,
            tab,
            workspace: workspace.downgrade(),
            title,
        });
        earlier
    });
    // A plan the agent replaced goes back to its dialog.
    if let Some(earlier) = earlier {
        earlier.request.answer(report::answer_refused(
            Refusal::ApproveTool,
            "a later plan replaced this one",
        ));
    }
    watch(cx);
}

/// Shows `plan` under `nonce` in the terminal's Plan tab, opening one beside the terminal when it
/// has none; the tab takes the focus only when the terminal has it.
fn open_tab(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    plan: String,
    nonce: u64,
    window: &mut Window,
    cx: &mut App,
) -> WeakEntity<PlanReview> {
    let view_id = view.entity_id();
    let title = SharedString::from(format!("Plan · {}", project_name(workspace.read(cx), cx)));
    let terminal_focused = view.focus_handle(cx).contains_focused(window, cx);
    workspace.update(cx, |workspace, cx| {
        let open = workspace
            .items_of_type::<PlanReview>(cx)
            .find(|tab| tab.read(cx).plan_view() == Some(view_id));
        let tab = if let Some(tab) = open {
            workspace.activate_item(&tab, false, terminal_focused, window, cx);
            tab
        } else {
            let subject = Subject::Plan { view: view_id };
            let tab = cx.new(|cx| PlanReview::new(subject, title, window, cx));
            // Beside the terminal when it sits in a center pane, else in the active one.
            let pane = workspace
                .panes()
                .iter()
                .find(|pane| pane.read(cx).items().any(|item| item.item_id() == view_id))
                .cloned()
                .unwrap_or_else(|| workspace.active_pane().clone());
            workspace.add_item(
                pane,
                Box::new(tab.clone()),
                None,
                terminal_focused,
                terminal_focused,
                window,
                cx,
            );
            tab
        };
        // The registry from the workspace in hand: `show` runs inside its update.
        let languages = Arc::clone(workspace.project().read(cx).languages());
        tab.update(cx, |tab, cx| tab.show(plan, nonce, languages, window, cx));
        tab.downgrade()
    })
}

/// The workspace's project name, as the rail shows it.
fn project_name(workspace: &Workspace, cx: &App) -> String {
    workspace
        .project()
        .read(cx)
        .visible_worktrees(cx)
        .next()
        .map(|worktree| worktree.read(cx).root_name_str().to_string())
        .unwrap_or_default()
}

/// Checks each second for plans whose caller stopped waiting, while any wait.
fn watch(cx: &mut App) {
    if std::mem::replace(&mut cx.global_mut::<PendingPlans>().watching, true) {
        return;
    }
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            if !cx.update(sweep) {
                cx.update(|cx| cx.global_mut::<PendingPlans>().watching = false);
                break;
            }
        }
    })
    .detach();
}

/// Drops the plans whose request was withdrawn and marks their tabs; whether any still wait.
fn sweep(cx: &mut App) -> bool {
    let withdrawn_any = cx
        .global::<PendingPlans>()
        .plans
        .iter()
        .any(|pending| pending.withdrawn.load(Ordering::SeqCst));
    if withdrawn_any {
        let gone: Vec<(WeakEntity<PlanReview>, u64)> =
            cx.update_global::<PendingPlans, _>(|plans, _| {
                let (gone, kept): (Vec<Pending>, Vec<Pending>) = plans
                    .plans
                    .drain(..)
                    .partition(|pending| pending.withdrawn.load(Ordering::SeqCst));
                plans.plans = kept;
                gone.into_iter()
                    .map(|pending| (pending.tab, pending.nonce))
                    .collect()
            });
        for (tab, nonce) in gone {
            tab.update(cx, |tab, cx| tab.withdraw(nonce, cx)).log_err();
        }
    }
    !cx.global::<PendingPlans>().plans.is_empty()
}

/// What the user decided.
enum Decision {
    Approve,
    Deny(String),
}

/// Sends `decision` to the request `nonce` of `view`, once; whether it was still waiting.
fn answer(view: EntityId, nonce: u64, decision: Decision, cx: &mut App) -> bool {
    let taken = cx.update_global::<PendingPlans, _>(|plans, _| {
        plans
            .plans
            .iter()
            .position(|pending| pending.view == view && pending.nonce == nonce)
            .map(|at| plans.plans.remove(at))
    });
    let Some(pending) = taken else {
        return false;
    };
    if pending.withdrawn.load(Ordering::SeqCst) {
        return false;
    }
    let reply = match decision {
        Decision::Approve => report::answer_decision("approve", None),
        Decision::Deny(reason) => report::answer_decision("deny", Some(&reason)),
    };
    pending.request.answer(reply);
    true
}

/// The Needs you entry of the plan the agent in `view` waits on, if one does.
pub(crate) fn entry_of(view: EntityId, project: &str, cx: &App) -> Option<InboxEntry> {
    let pending = cx
        .try_global::<PendingPlans>()?
        .plans
        .iter()
        .find(|pending| pending.view == view)?;
    Some(InboxEntry {
        key: format!("plan:{}:{}", view.as_u64(), pending.nonce),
        kind: InboxKind::Plan,
        agent: marley_agent::AgentKind::Claude.display_name().to_string(),
        project: project.to_string(),
        ask: format!("Plan to review: \u{201c}{}\u{201d}", pending.title),
        waited: marley_rail::waited_words(pending.since.elapsed().as_secs()),
        answers: Vec::new(),
        chips: Vec::new(),
        level: 0,
        route: None,
    })
}

/// Brings the tab of the plan the agent in `view` waits on to the front, after the caller's
/// update.
pub(crate) fn show_tab(view: EntityId, window: &Window, cx: &mut App) {
    let Some((tab, workspace)) = cx.try_global::<PendingPlans>().and_then(|plans| {
        plans
            .plans
            .iter()
            .find(|pending| pending.view == view)
            .map(|pending| (pending.tab.clone(), pending.workspace.clone()))
    }) else {
        return;
    };
    window.defer(cx, move |window, cx| {
        let (Some(tab), Some(workspace)) = (tab.upgrade(), workspace.upgrade()) else {
            return;
        };
        workspace.update(cx, |workspace, cx| {
            workspace.activate_item(&tab, true, true, window, cx);
        });
    });
}

/// The review tab's toasts' id.
struct ReviewToasts;

pub(crate) fn show_toast(
    workspace: &mut Workspace,
    message: impl Into<String>,
    cx: &mut Context<Workspace>,
) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<ReviewToasts>(), message.into()),
        cx,
    );
}

/// `marley: review markdown` (#729): the active Markdown file, or the file the active preview
/// shows, in a review tab; the message says why it did not open.
fn review_markdown(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), &'static str> {
    // A preview hands out its source editor as an `Editor`.
    let editor = workspace
        .active_item(cx)
        .and_then(|item| item.act_as::<Editor>(cx))
        .filter(|editor| MarkdownPreviewView::is_markdown_file(editor, cx))
        .ok_or("Review Markdown reviews the active Markdown file or its preview.")?;
    let buffer = editor
        .read(cx)
        .buffer()
        .read(cx)
        .as_singleton()
        .ok_or("Review Markdown reviews one file at a time.")?;
    let buffer = buffer.read(cx);
    let path = buffer
        .file()
        .and_then(|file| file.as_local())
        .map(|file| file.abs_path(cx))
        .ok_or("Review Markdown needs a file saved on this machine.")?;
    let text = buffer.text();
    let name = path.file_name().map_or_else(
        || path.to_string_lossy().into_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let subject = Subject::File {
        path,
        workspace: cx.entity().downgrade(),
    };
    open_document(
        workspace,
        subject,
        format!("Review · {name}"),
        text,
        window,
        cx,
    );
    Ok(())
}

/// `marley: review last reply` (#729): the Agent Panel's thread while the panel has the focus,
/// else the active thread tab, the active terminal's Claude Code, or the panel's thread.
fn review_last_reply(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), &'static str> {
    let panel = workspace.panel::<AgentPanel>(cx);
    let panel_thread = panel
        .as_ref()
        .and_then(|panel| panel.read(cx).active_conversation_view().cloned());
    let panel_focused = panel
        .as_ref()
        .is_some_and(|panel| panel.focus_handle(cx).contains_focused(window, cx));
    if panel_focused && let Some(conversation) = panel_thread.as_ref() {
        return review_thread_reply(workspace, conversation, window, cx);
    }
    if let Some(conversation) = crate::thread_tab::active_conversation(workspace, cx) {
        return review_thread_reply(workspace, &conversation, window, cx);
    }
    if let Some(view) = workspace.active_item_as::<TerminalView>(cx) {
        return review_terminal_reply(workspace, &view, window, cx);
    }
    let conversation = panel_thread.ok_or(
        "Review Last Reply reviews the active agent thread, or Claude Code in the active terminal.",
    )?;
    review_thread_reply(workspace, &conversation, window, cx)
}

fn review_thread_reply(
    workspace: &mut Workspace,
    conversation: &Entity<ConversationView>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), &'static str> {
    let text = crate::last_reply::of_thread(conversation, cx)
        .ok_or("This thread has no reply to review yet.")?;
    let title = conversation
        .read(cx)
        .root_thread_view()
        .and_then(|view| view.read(cx).thread.read(cx).title())
        .map_or_else(
            || "Reply · Thread".to_string(),
            |title| format!("Reply · {title}"),
        );
    let subject = Subject::Reply(ReplyOrigin::Thread {
        conversation: conversation.downgrade(),
        workspace: cx.entity().downgrade(),
    });
    open_document(workspace, subject, title, text, window, cx);
    Ok(())
}

/// The last reply of Claude Code in `view`, read from its transcript off the main thread, in a
/// review tab (#729); the rail's terminal menu calls it too. A failure to read it is a toast.
pub(crate) fn review_terminal_reply(
    workspace: &Workspace,
    view: &Entity<TerminalView>,
    window: &Window,
    cx: &Context<Workspace>,
) -> Result<(), &'static str> {
    let path = crate::last_reply::transcript_of(view.entity_id(), cx).ok_or(
        "Marley knows no transcript for this terminal: Claude Code names it to Marley's plugin.",
    )?;
    let project = project_name(workspace, cx);
    let title = format!("Reply · Claude Code · {project}");
    let origin = ReplyOrigin::Terminal {
        view: view.downgrade(),
        project,
        workspace: cx.entity().downgrade(),
    };
    let read = cx.background_spawn(futures::future::lazy(move |_| {
        crate::last_reply::read_reply(&path)
    }));
    cx.spawn_in(window, async move |workspace, cx| {
        let reply = read.await;
        workspace.update_in(cx, |workspace, window, cx| match reply {
            Ok(text) => open_document(workspace, Subject::Reply(origin), title, text, window, cx),
            Err(message) => show_toast(workspace, message, cx),
        })
    })
    .detach_and_log_err(cx);
    Ok(())
}

/// Shows `text` in a review tab of `subject` in the active pane, or brings back the tab already
/// showing the same text of it.
fn open_document(
    workspace: &mut Workspace,
    subject: Subject,
    title: String,
    text: String,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open = workspace.items_of_type::<PlanReview>(cx).find(|tab| {
        let tab = tab.read(cx);
        tab.subject.same_as(&subject) && tab.plan == text
    });
    if let Some(tab) = open {
        workspace.activate_item(&tab, true, true, window, cx);
        return;
    }
    let tab = cx.new(|cx| {
        let mut tab = PlanReview::new(subject, SharedString::from(title), window, cx);
        tab.show_document(text, window, cx);
        tab
    });
    workspace.add_item_to_active_pane(Box::new(tab), None, true, window, cx);
}

/// Into the terminal's agent, only while it is ready: an agent at work would take the notes as its
/// next prompt.
fn send_to_reply_terminal(
    view: &WeakEntity<TerminalView>,
    project: &str,
    text: String,
    here: AnyWindowHandle,
    cx: &mut App,
) -> Result<(), SharedString> {
    let target = view
        .upgrade()
        .and_then(|view| crate::send_selection::target_of(project.to_string(), view, cx))
        .ok_or_else(|| {
            SharedString::new_static("No agent CLI runs in that terminal now; nothing was sent.")
        })?;
    if !target.ready {
        return Err(SharedString::from(format!(
            "{} in {} is {}; the notes were not sent.",
            target.kind.display_name(),
            target.project,
            crate::review_notes::readiness(&target)
        )));
    }
    crate::drafts::send_to_terminal(view, project, text, here, cx)
}

/// The lines, from 1, that `range` of `text` covers; a passage that ends with its line break ends
/// on that line.
fn lines_of(text: &str, range: &Range<usize>) -> (u32, u32) {
    let line_at = |offset: usize| {
        text.get(..offset)
            .map_or(0, |before| before.matches('\n').count())
            + 1
    };
    let ends_with_break = text
        .get(range.clone())
        .is_some_and(|passage| passage.len() > 1 && passage.ends_with('\n'));
    let first = line_at(range.start);
    let last = line_at(range.end) - usize::from(ends_with_break);
    (
        u32::try_from(first).unwrap_or(u32::MAX),
        u32::try_from(last.max(first)).unwrap_or(u32::MAX),
    )
}

/// A note on a passage of the plan; an empty quote is a sent overall note (#729).
struct Note {
    range: Range<usize>,
    quote: String,
    comment: String,
    /// Whether it went to the agent (#729); a plan's notes go with the plan's answer instead.
    sent: bool,
}

/// What a review tab reviews.
enum Subject {
    /// Claude Code's plan, from the agent in this terminal view (#726).
    Plan { view: EntityId },
    /// A Markdown file, by its absolute path, from this workspace (#729).
    File {
        path: PathBuf,
        workspace: WeakEntity<Workspace>,
    },
    /// An agent's last reply (#729).
    Reply(ReplyOrigin),
}

/// Where a reply came from, which is where its notes go.
enum ReplyOrigin {
    /// Claude Code in a terminal of `project`.
    Terminal {
        view: WeakEntity<TerminalView>,
        project: String,
        workspace: WeakEntity<Workspace>,
    },
    /// A Zed agent thread.
    Thread {
        conversation: WeakEntity<ConversationView>,
        workspace: WeakEntity<Workspace>,
    },
}

impl Subject {
    /// Whether `other` reviews the same thing.
    fn same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plan { view }, Self::Plan { view: other }) => view == other,
            (Self::File { path, .. }, Self::File { path: other, .. }) => path == other,
            (
                Self::Reply(ReplyOrigin::Terminal { view, .. }),
                Self::Reply(ReplyOrigin::Terminal { view: other, .. }),
            ) => view.entity_id() == other.entity_id(),
            (
                Self::Reply(ReplyOrigin::Thread { conversation, .. }),
                Self::Reply(ReplyOrigin::Thread {
                    conversation: other,
                    ..
                }),
            ) => conversation.entity_id() == other.entity_id(),
            _ => false,
        }
    }

    const fn workspace(&self) -> Option<&WeakEntity<Workspace>> {
        match self {
            Self::Plan { .. } => None,
            Self::File { workspace, .. }
            | Self::Reply(
                ReplyOrigin::Terminal { workspace, .. } | ReplyOrigin::Thread { workspace, .. },
            ) => Some(workspace),
        }
    }
}

/// A note being written.
struct Composing {
    range: Range<usize>,
    quote: String,
    editor: Entity<Editor>,
}

/// Where the tab's plan stands.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// Waiting for the user, under the request's nonce.
    Waiting(u64),
    /// A file or a reply, taking notes with no request waiting (#729).
    Open,
    Approved,
    FeedbackSent,
    /// The caller stopped waiting, or its wait ran out.
    Withdrawn,
}

/// The Plan tab, and since #729 the review tab of a file or a reply.
pub(crate) struct PlanReview {
    subject: Subject,
    /// The tab's title: "Plan · project", "Review · file", "Reply · agent".
    title: SharedString,
    /// The text under review: the plan, the file or the reply.
    plan: String,
    /// The plan's runs of Markdown and its question cards, in order (#728).
    parts: Vec<Part>,
    cards: Vec<Card>,
    notes: Vec<Note>,
    composing: Option<Composing>,
    overall: Entity<Editor>,
    state: State,
    /// The right-click menu, where it opened, and the passage selected when it did: a click on its
    /// entry clears the selection before the entry runs.
    menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,
    /// The plan's area, which the tab's focus is: `c` works there and not in the note editors.
    plan_focus: FocusHandle,
    /// Each plan the tab showed, version 1 first (#727).
    versions: Vec<String>,
    /// The diff of the shown version against the one before, and its counts (#727).
    changes: Option<Changes>,
    /// Whether the left area shows the changes rather than the plan.
    showing_changes: bool,
    _overall_edited: Subscription,
    /// Redraws the tab as its terminal's turns come and are kept or undone (#893).
    _turns: Subscription,
}

/// A run of the plan as the tab draws it (#728).
enum Part {
    /// Markdown from byte `start` of the plan, `len` bytes long.
    Text {
        start: usize,
        len: usize,
        markdown: Entity<Markdown>,
    },
    /// A question card, by its place in `cards`.
    Card(usize),
}

/// A question of the plan and the user's answer so far (#728).
struct Card {
    question: Question,
    context: Option<Entity<Markdown>>,
    /// Which choices are picked, in the question's order.
    picked: Vec<bool>,
    /// The user's own answer: Other… for a question with choices, the answer for free text.
    words: Entity<Editor>,
    skipped: bool,
    /// Whether the pick is the recommendation, accepted.
    accepted: bool,
    /// Redraws the tab as the words change: the card's state and the count read them.
    _edited: Subscription,
}

/// Where a card stands.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CardState {
    Open,
    Answered,
    Skipped,
    /// The plan marked a choice already.
    Settled,
}

/// A version's changes from the one before (#727).
struct Changes {
    added: usize,
    removed: usize,
    /// Keeps the diff's buffers alive for the editor.
    _diff: Entity<acp_thread::Diff>,
    editor: Entity<Editor>,
}

impl PlanReview {
    fn new(
        subject: Subject,
        title: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let placeholder = match subject {
            Subject::Plan { .. } => "Anything about the whole plan",
            Subject::File { .. } => "A note on the whole file",
            Subject::Reply(_) => "A note on the whole reply",
        };
        let overall = cx.new(|cx| {
            let mut editor = Editor::auto_height(1, 4, window, cx);
            editor.set_soft_wrap_mode(settings::SoftWrap::EditorWidth, cx);
            editor.set_placeholder_text(placeholder, window, cx);
            editor
        });
        // Send Notes counts the overall note, and Send Feedback waits for one.
        let overall_edited = cx.subscribe(&overall, |_, _, event: &EditorEvent, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                cx.notify();
            }
        });
        Self {
            subject,
            title,
            plan: String::new(),
            parts: Vec::new(),
            cards: Vec::new(),
            notes: Vec::new(),
            composing: None,
            overall,
            state: State::Withdrawn,
            menu: None,
            plan_focus: cx.focus_handle(),
            versions: Vec::new(),
            changes: None,
            showing_changes: false,
            _overall_edited: overall_edited,
            _turns: cx.observe_global::<crate::turns::Turns>(|_, cx| cx.notify()),
        }
    }

    /// Shows a new plan waiting under `nonce`, its notes cleared.
    fn show(
        &mut self,
        plan: String,
        nonce: u64,
        languages: Arc<LanguageRegistry>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (parts, cards) = parts_of(&plan, window, cx);
        self.parts = parts;
        self.cards = cards;
        self.changes = self
            .versions
            .last()
            .map(|before| changes_from(before.clone(), plan.clone(), languages, window, cx));
        self.showing_changes = false;
        self.versions.push(plan.clone());
        self.plan = plan;
        self.notes.clear();
        self.composing = None;
        self.overall
            .update(cx, |editor, cx| editor.set_text("", window, cx));
        self.state = State::Waiting(nonce);
        self.highlight(cx);
        cx.notify();
    }

    /// Shows a file's or a reply's text, taking notes (#729). Question blocks stay text there.
    fn show_document(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        let markdown = cx.new(|cx| Markdown::new(text.clone().into(), None, None, cx));
        self.parts = vec![Part::Text {
            start: 0,
            len: text.len(),
            markdown,
        }];
        self.cards.clear();
        self.plan = text;
        self.notes.clear();
        self.composing = None;
        self.overall
            .update(cx, |editor, cx| editor.set_text("", window, cx));
        self.state = State::Open;
        self.highlight(cx);
        cx.notify();
    }

    /// The terminal view whose plan the tab shows, for a plan.
    const fn plan_view(&self) -> Option<EntityId> {
        match self.subject {
            Subject::Plan { view } => Some(view),
            Subject::File { .. } | Subject::Reply(_) => None,
        }
    }

    /// Whether the tab takes notes now: a plan while it waits, a file or a reply always.
    const fn taking_notes(&self) -> bool {
        matches!(self.state, State::Waiting(_) | State::Open)
    }

    fn withdraw(&mut self, nonce: u64, cx: &mut Context<Self>) {
        if self.state == State::Waiting(nonce) {
            self.state = State::Withdrawn;
            self.composing = None;
            cx.notify();
        }
    }

    /// Paints the noted passages, each run of the plan its own part of them.
    fn highlight(&self, cx: &mut Context<Self>) {
        for part in &self.parts {
            let Part::Text {
                start,
                len,
                markdown,
            } = part
            else {
                continue;
            };
            let end = start + len;
            let mut ranges: Vec<Range<usize>> = self
                .notes
                .iter()
                .filter(|note| note.range.start < end && note.range.end > *start)
                .map(|note| note.range.start.max(*start) - start..note.range.end.min(end) - start)
                .collect();
            ranges.sort_by_key(|range| (range.start, range.end));
            markdown.update(cx, |markdown, cx| {
                markdown.set_search_highlights(ranges, None, cx);
            });
        }
    }

    /// The passage selected in any run, in the plan's bytes.
    fn selection(&self, cx: &App) -> Option<Range<usize>> {
        self.parts.iter().find_map(|part| match part {
            Part::Text {
                start, markdown, ..
            } => markdown
                .read(cx)
                .selected_range()
                .map(|range| range.start + start..range.end + start),
            Part::Card(_) => None,
        })
    }

    const fn waiting(&self) -> Option<u64> {
        match self.state {
            State::Waiting(nonce) => Some(nonce),
            _ => None,
        }
    }

    /// `c`: a note on the passage selected now.
    fn comment(&mut self, _: &Comment, window: &mut Window, cx: &mut Context<Self>) {
        // A card's answer field sits in the plan's area: the letter is its own there. The focus
        // is not the area's own after a drag, since the Markdown element takes it for its copy.
        if self
            .cards
            .iter()
            .any(|card| card.words.focus_handle(cx).contains_focused(window, cx))
        {
            cx.propagate();
            return;
        }
        if let Some(range) = self.selection(cx) {
            self.start_note(range, window, cx);
        }
    }

    /// Opens the right-click menu, keeping the passage selected now for its Comment….
    fn open_menu(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let range = self.selection(cx);
        let tab = cx.entity().downgrade();
        let menu = ContextMenu::build(window, cx, move |menu, _, _| {
            let Some(range) = range.clone() else {
                return menu.item(
                    ui::ContextMenuEntry::new("Select a passage to comment on").disabled(true),
                );
            };
            menu.entry("Comment…", None, move |window, cx| {
                tab.update(cx, |tab, cx| tab.start_note(range.clone(), window, cx))
                    .log_err();
            })
        });
        let subscription = cx.subscribe_in(&menu, window, |tab, _, _: &DismissEvent, _, cx| {
            tab.menu = None;
            cx.notify();
        });
        window.focus(&menu.focus_handle(cx), cx);
        self.menu = Some((menu, position, subscription));
        cx.notify();
    }

    /// Starts a note on `range`: an editor for its words, focused.
    fn start_note(&mut self, range: Range<usize>, window: &mut Window, cx: &mut Context<Self>) {
        if !self.taking_notes() {
            return;
        }
        let Some(quote) = self.plan.get(range.clone()).map(str::to_string) else {
            return;
        };
        let tab = cx.entity().downgrade();
        let editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Your comment on the passage", window, cx);
            editor
        });
        editor.update(cx, |editor, _| {
            let add = tab.clone();
            editor
                .register_action(move |_: &menu::Confirm, window, cx| {
                    add.update(cx, |tab, cx| tab.add_note(window, cx)).log_err();
                })
                .detach();
            let drop = tab;
            editor
                .register_action(move |_: &editor::actions::Cancel, window, cx| {
                    drop.update(cx, |tab, cx| {
                        tab.composing = None;
                        window.focus(&tab.plan_focus, cx);
                        cx.notify();
                    })
                    .log_err();
                })
                .detach();
        });
        window.focus(&editor.focus_handle(cx), cx);
        self.composing = Some(Composing {
            range,
            quote,
            editor,
        });
        cx.notify();
    }

    /// Adds the note being written, unless its words are empty.
    fn add_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(composing) = self.composing.take() else {
            return;
        };
        let comment = composing.editor.read(cx).text(cx).trim().to_string();
        if !comment.is_empty() {
            self.notes.push(Note {
                range: composing.range,
                quote: composing.quote,
                comment,
                sent: false,
            });
            self.highlight(cx);
        }
        window.focus(&self.plan_focus, cx);
        cx.notify();
    }

    fn remove_note(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.notes.get(index).is_some_and(|note| !note.sent) {
            self.notes.remove(index);
            self.highlight(cx);
            cx.notify();
        }
    }

    fn overall_note(&self, cx: &App) -> String {
        self.overall.read(cx).text(cx).trim().to_string()
    }

    fn approve(&mut self, cx: &mut Context<Self>) {
        let (Some(nonce), Some(view)) = (self.waiting(), self.plan_view()) else {
            return;
        };
        self.state = if answer(view, nonce, Decision::Approve, cx) {
            State::Approved
        } else {
            State::Withdrawn
        };
        self.composing = None;
        cx.notify();
    }

    fn send_feedback(&mut self, cx: &mut Context<Self>) {
        let (Some(nonce), Some(view)) = (self.waiting(), self.plan_view()) else {
            return;
        };
        let notes: Vec<PlanNote> = self
            .notes
            .iter()
            .map(|note| PlanNote {
                quote: note.quote.clone(),
                comment: note.comment.clone(),
            })
            .collect();
        let reason = marley_agent::plan_feedback(&notes, &self.overall_note(cx));
        self.state = if answer(view, nonce, Decision::Deny(reason), cx) {
            State::FeedbackSent
        } else {
            State::Withdrawn
        };
        self.composing = None;
        cx.notify();
    }

    /// The notes not sent yet, by their place, and the overall note (#729).
    fn unsent(&self, cx: &App) -> (Vec<usize>, String) {
        let indices = self
            .notes
            .iter()
            .enumerate()
            .filter(|(_, note)| !note.sent)
            .map(|(index, _)| index)
            .collect();
        (indices, self.overall_note(cx))
    }

    /// Sends the notes not sent yet and the overall note where the subject's notes go (#729): a
    /// file's through #522's picker, a reply's back to its agent. A refusal says why in a toast.
    fn send_notes(&mut self, window: &Window, cx: &mut Context<Self>) {
        let (indices, overall) = self.unsent(cx);
        if indices.is_empty() && overall.is_empty() {
            return;
        }
        let notes: Vec<&Note> = indices
            .iter()
            .filter_map(|index| self.notes.get(*index))
            .collect();
        let refusal = match &self.subject {
            Subject::Plan { .. } => None,
            Subject::File { path, workspace } => {
                let file = path.to_string_lossy().into_owned();
                let mut lines: Vec<ReviewLine> = notes
                    .iter()
                    .map(|note| {
                        let (first, last) = lines_of(&self.plan, &note.range);
                        ReviewLine {
                            path: file.clone(),
                            first,
                            last,
                            comment: note.comment.clone(),
                            source: None,
                            quote: Some(note.quote.clone()),
                        }
                    })
                    .collect();
                if !overall.is_empty() {
                    lines.push(ReviewLine {
                        path: file,
                        first: 0,
                        last: 0,
                        comment: overall.clone(),
                        source: None,
                        quote: None,
                    });
                }
                let tab = cx.entity().downgrade();
                let on_sent: Rc<dyn Fn(&mut App)> = Rc::new(move |cx| {
                    tab.update(cx, |tab, cx| tab.mark_sent(&indices, &overall, cx))
                        .log_err();
                });
                let (path, workspace) = (path.clone(), workspace.clone());
                // Out of the tab's update: the picker opens in the workspace.
                window.defer(cx, move |window, cx| {
                    workspace
                        .update(cx, |workspace, cx| {
                            crate::review_notes::send_file_notes(
                                workspace, &path, lines, on_sent, window, cx,
                            );
                        })
                        .log_err();
                });
                return;
            }
            Subject::Reply(origin) => {
                let notes: Vec<PlanNote> = notes
                    .iter()
                    .map(|note| PlanNote {
                        quote: note.quote.clone(),
                        comment: note.comment.clone(),
                    })
                    .collect();
                let text = crate::review_notes::reply_message(
                    &marley_agent::reply_notes(&notes, &overall),
                    cx,
                );
                let here = window.window_handle();
                match origin {
                    ReplyOrigin::Terminal { view, project, .. } => {
                        send_to_reply_terminal(view, project, text, here, cx)
                    }
                    ReplyOrigin::Thread {
                        conversation,
                        workspace,
                    } => crate::drafts::send_to_thread(conversation, workspace, &text, here, cx),
                }
                .err()
            }
        };
        if let Some(message) = refusal {
            self.toast(message, cx);
        } else {
            self.mark_sent(&indices, &overall, cx);
        }
    }

    /// Marks the notes at `indices` sent, and keeps a sent overall note as a note of its own.
    fn mark_sent(&mut self, indices: &[usize], overall: &str, cx: &mut Context<Self>) {
        for index in indices {
            if let Some(note) = self.notes.get_mut(*index) {
                note.sent = true;
            }
        }
        if !overall.is_empty() {
            self.notes.push(Note {
                range: 0..0,
                quote: String::new(),
                comment: overall.to_string(),
                sent: true,
            });
            if let Some(buffer) = self.overall.read(cx).buffer().read(cx).as_singleton() {
                buffer.update(cx, |buffer, cx| {
                    buffer.set_text("", cx);
                });
            }
        }
        cx.notify();
    }

    fn toast(&self, message: impl Into<String>, cx: &mut App) {
        if let Some(workspace) = self.subject.workspace().and_then(WeakEntity::upgrade) {
            workspace.update(cx, |workspace, cx| show_toast(workspace, message, cx));
        }
    }

    /// The column's line on where the review stands.
    const fn status(&self) -> &'static str {
        match (&self.subject, self.state) {
            (Subject::Plan { .. }, State::Waiting(_) | State::Open) => {
                "Claude Code waits for your answer."
            }
            (Subject::Plan { .. }, State::Approved) => "Approved.",
            (Subject::Plan { .. }, State::FeedbackSent) => "Feedback sent.",
            (Subject::Plan { .. }, State::Withdrawn) => {
                "This plan went back to the terminal's own dialog."
            }
            (Subject::File { .. }, _) => "Notes go to an agent working on this file.",
            (Subject::Reply(ReplyOrigin::Terminal { .. }), _) => {
                "Notes go back to Claude Code in its terminal, once it is ready."
            }
            (Subject::Reply(ReplyOrigin::Thread { .. }), _) => "Notes go back to the thread.",
        }
    }

    fn card_state(card: &Card, cx: &App) -> CardState {
        if card.question.choices.iter().any(|choice| choice.settled) {
            CardState::Settled
        } else if card.skipped {
            CardState::Skipped
        } else if card.picked.iter().any(|picked| *picked)
            || !card.words.read(cx).text(cx).trim().is_empty()
        {
            CardState::Answered
        } else {
            CardState::Open
        }
    }

    /// A card's answer for the agent; none while it is open or settled.
    fn card_answer(card: &Card, cx: &App) -> Option<Answer> {
        let words = card.words.read(cx).text(cx).trim().to_string();
        match Self::card_state(card, cx) {
            CardState::Open | CardState::Settled => None,
            CardState::Skipped => Some(Answer::Skipped),
            CardState::Answered => {
                let picked: Vec<String> = card
                    .question
                    .choices
                    .iter()
                    .zip(&card.picked)
                    .filter(|(_, picked)| **picked)
                    .map(|(choice, _)| choice.label.clone())
                    .collect();
                Some(if !picked.is_empty() {
                    Answer::Choices {
                        picked,
                        recommended: card.accepted,
                    }
                } else if card.question.kind == QuestionKind::Text {
                    Answer::Text(words)
                } else {
                    Answer::Other(words)
                })
            }
        }
    }

    /// The questions that take an answer, settled ones left out, and how many have one.
    fn answered(&self, cx: &App) -> (usize, usize) {
        let open: Vec<&Card> = self
            .cards
            .iter()
            .filter(|card| Self::card_state(card, cx) != CardState::Settled)
            .collect();
        let answered = open
            .iter()
            .filter(|card| Self::card_state(card, cx) == CardState::Answered)
            .count();
        (answered, open.len())
    }

    /// Whether an answer or a skip waits to be sent.
    fn answers_pending(&self, cx: &App) -> bool {
        self.cards.iter().any(|card| {
            matches!(
                Self::card_state(card, cx),
                CardState::Answered | CardState::Skipped
            )
        })
    }

    fn pick(&mut self, card: usize, choice: usize, picked: bool, cx: &mut Context<Self>) {
        let Some(card) = self.cards.get_mut(card) else {
            return;
        };
        if card.question.kind == QuestionKind::One {
            card.picked.iter_mut().for_each(|picked| *picked = false);
        }
        if let Some(slot) = card.picked.get_mut(choice) {
            *slot = picked;
        }
        card.skipped = false;
        card.accepted = false;
        cx.notify();
    }

    fn accept_recommended(&mut self, card: usize, cx: &mut Context<Self>) {
        let Some(at) = self.cards.get(card).and_then(|card| {
            let pick = card.question.recommended.as_deref()?;
            card.question
                .choices
                .iter()
                .position(|choice| choice.label == pick)
        }) else {
            return;
        };
        self.pick(card, at, true, cx);
        if let Some(card) = self.cards.get_mut(card) {
            card.accepted = true;
        }
    }

    fn skip(&mut self, card: usize, cx: &mut Context<Self>) {
        if let Some(card) = self.cards.get_mut(card) {
            card.skipped = !card.skipped;
            card.picked.iter_mut().for_each(|picked| *picked = false);
            card.accepted = false;
            cx.notify();
        }
    }

    /// Sends the answers, then any notes, as the deny's reason (#728).
    fn send_answers(&mut self, cx: &mut Context<Self>) {
        let (Some(nonce), Some(view)) = (self.waiting(), self.plan_view()) else {
            return;
        };
        let questions: Vec<(String, Option<Answer>)> = self
            .cards
            .iter()
            .filter(|card| Self::card_state(card, cx) != CardState::Settled)
            .map(|card| (card.question.prompt.clone(), Self::card_answer(card, cx)))
            .collect();
        let answers = plan_questions::plan_answers(&questions);
        let notes: Vec<PlanNote> = self
            .notes
            .iter()
            .map(|note| PlanNote {
                quote: note.quote.clone(),
                comment: note.comment.clone(),
            })
            .collect();
        let reason = plan_questions::questions_feedback(&answers, &notes, &self.overall_note(cx));
        self.state = if answer(view, nonce, Decision::Deny(reason), cx) {
            State::FeedbackSent
        } else {
            State::Withdrawn
        };
        self.composing = None;
        cx.notify();
    }

    /// A question as a card in the plan (#728).
    fn render_card(
        &self,
        index: usize,
        style: &MarkdownStyle,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let border = cx.theme().colors().border;
        let surface = cx.theme().colors().surface_background;
        let total = self.cards.len();
        let waiting = self.waiting().is_some();
        let Some(card) = self.cards.get(index) else {
            return div().into_any_element();
        };
        let state = Self::card_state(card, cx);
        let (state_words, state_color) = match state {
            CardState::Open => ("Open", Color::Accent),
            CardState::Answered => ("Answered", Color::Success),
            CardState::Skipped => ("Skipped", Color::Muted),
            CardState::Settled => ("Settled", Color::Muted),
        };
        let settled = state == CardState::Settled;
        let recommended = card.question.recommended.clone();
        v_flex()
            .my_2()
            .p_3()
            .gap_2()
            .rounded_md()
            .border_1()
            .border_color(border)
            .bg(surface)
            .child(
                h_flex().justify_between().child(
                    h_flex()
                        .gap_2()
                        .child(
                            Label::new(format!("Question {} of {total}", index + 1))
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                        .child(
                            Label::new(state_words)
                                .size(LabelSize::Small)
                                .color(state_color),
                        ),
                ),
            )
            .child(Label::new(card.question.prompt.clone()).weight(FontWeight::SEMIBOLD))
            .when_some(card.context.clone(), |card_view, context| {
                card_view.child(MarkdownElement::new(context, style.clone()))
            })
            .children(
                card.question
                    .choices
                    .iter()
                    .enumerate()
                    .map(|(choice, item)| {
                        let is_recommended = recommended.as_deref() == Some(item.label.as_str());
                        h_flex()
                            .gap_2()
                            .child(
                                Checkbox::new(
                                    ("plan-review-choice", index * 1000 + choice),
                                    ToggleState::from(
                                        card.picked.get(choice).copied().unwrap_or(false),
                                    ),
                                )
                                .label(choice_label(item))
                                .label_size(LabelSize::Small)
                                .disabled(settled || !waiting)
                                .on_click(cx.listener(
                                    move |tab, state: &ToggleState, _, cx| {
                                        tab.pick(index, choice, state.selected(), cx);
                                    },
                                )),
                            )
                            .when(is_recommended, |row| row.child(recommended_chip(cx)))
                    }),
            )
            .when(!settled, |card_view| {
                card_view.child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .border_1()
                        .border_color(border)
                        .child(card.words.clone()),
                )
            })
            // Accept recommended and Skip at the card's foot, as the Review board draws them (#881).
            .when(waiting && !settled, |card_view| {
                card_view.child(Self::card_buttons(
                    index,
                    recommended.is_some() && state == CardState::Open,
                    card.skipped,
                    cx,
                ))
            })
            .into_any_element()
    }

    fn render_notes(&self, cx: &Context<Self>) -> impl IntoElement {
        let surface = cx.theme().colors().surface_background;
        let accent = cx.theme().colors().text_accent;
        let focused_border = cx.theme().colors().border_focused;
        let taking_notes = self.taking_notes();
        v_flex()
            .gap_2()
            .children(self.notes.iter().enumerate().map(|(index, note)| {
                let quote = if note.quote.is_empty() {
                    "Overall".to_string()
                } else {
                    format!("“{}”", note.quote.trim())
                };
                // The quote as a mono chip on an accent rule, Remove at the note's foot (#924).
                v_flex()
                    .gap_1()
                    .p_2()
                    .rounded_md()
                    .bg(surface)
                    .child(
                        h_flex()
                            .justify_between()
                            .gap_1()
                            .child(
                                div()
                                    .min_w_0()
                                    .pl_2()
                                    .border_l_2()
                                    .border_color(accent)
                                    .child(
                                        Label::new(quote)
                                            .size(LabelSize::XSmall)
                                            .buffer_font(cx)
                                            .color(Color::Muted)
                                            .line_clamp(2),
                                    ),
                            )
                            .when(note.sent, |row| {
                                row.child(
                                    Label::new("Sent")
                                        .size(LabelSize::XSmall)
                                        .color(Color::Success),
                                )
                            }),
                    )
                    .child(Label::new(note.comment.clone()).size(LabelSize::Small))
                    .when(taking_notes && !note.sent, |note_card| {
                        note_card.child(
                            h_flex().child(
                                Button::new(("plan-review-remove-note", index), "Remove")
                                    .style(ButtonStyle::Subtle)
                                    .label_size(LabelSize::XSmall)
                                    .color(Color::Muted)
                                    .on_click(cx.listener(move |tab, _, _, cx| {
                                        tab.remove_note(index, cx);
                                    })),
                            ),
                        )
                    })
            }))
            .when_some(self.composing.as_ref(), |column, composing| {
                column.child(
                    v_flex()
                        .gap_1()
                        .p_2()
                        .rounded_md()
                        .border_1()
                        .border_color(focused_border)
                        .child(
                            Label::new(format!("On “{}”", composing.quote.trim()))
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .line_clamp(2),
                        )
                        .child(composing.editor.clone())
                        .child(
                            Label::new("Enter adds it, Escape drops it.")
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                )
            })
    }

    /// The column at the right: the tab's title and status, the notes, the overall note, and the
    /// answers.
    fn render_column(&self, border: gpui::Hsla, cx: &Context<Self>) -> impl IntoElement {
        let (answered, questions) = self.answered(cx);
        let taking_notes = self.taking_notes();
        let is_plan = self.plan_view().is_some();
        let (unsent, overall) = self.unsent(cx);
        let unsent = unsent.len() + usize::from(!overall.is_empty());
        v_flex()
            .id("plan-review-notes")
            .w(rems(24.))
            .h_full()
            .overflow_y_scroll()
            .border_l_1()
            .border_color(border)
            .p_3()
            .gap_3()
            // With an earlier version the tabs' line carries the count (#893).
            .when(questions > 0 && self.changes.is_none(), |column| {
                column.child(
                    Label::new(format!("{answered} of {questions} answered"))
                        .size(LabelSize::Small),
                )
            })
            .child(
                Label::new(self.status())
                    .size(LabelSize::Small)
                    .color(if taking_notes {
                        Color::Accent
                    } else {
                        Color::Muted
                    }),
            )
            .child(
                Label::new("YOUR NOTES")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .child(self.render_notes(cx))
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        Label::new("Overall")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .border_1()
                            .border_color(border)
                            .child(self.overall.clone()),
                    ),
            )
            .children(self.render_turns(cx))
            // A file's or a reply's send at the column's foot; a plan's under the plan (#924).
            .when(!is_plan, |column| {
                column
                    .child(div().flex_1())
                    .child(Self::send_notes_button(unsent, cx))
            })
    }

    /// Plan and Changes as tabs over the left area, and the version's line (#893), once the plan
    /// has an earlier version.
    fn render_tabs(&self, cx: &Context<Self>) -> Option<impl IntoElement + use<>> {
        let changes = self.changes.as_ref()?;
        let colors = cx.theme().colors();
        let (answered, questions) = self.answered(cx);
        let tab = |id: &'static str, label: &'static str, chosen: bool, changes: bool| {
            h_flex()
                .id(id)
                .gap_1p5()
                .px_3()
                .py_1p5()
                .border_b_2()
                .border_color(if chosen {
                    colors.text_accent
                } else {
                    gpui::transparent_black()
                })
                .cursor_pointer()
                .child(Label::new(label).size(LabelSize::Small).color(if chosen {
                    Color::Default
                } else {
                    Color::Muted
                }))
                .on_click(cx.listener(move |tab, _, _, cx| {
                    tab.showing_changes = changes;
                    cx.notify();
                }))
        };
        let version = self.versions.len();
        let line = if questions > 0 {
            format!("Version {version} · {answered} of {questions} answered")
        } else {
            format!("Version {version}")
        };
        Some(
            v_flex()
                .flex_none()
                .border_b_1()
                .border_color(colors.border)
                .child(
                    h_flex()
                        .px_2()
                        .child(tab(
                            "plan-review-tab-plan",
                            "Plan",
                            !self.showing_changes,
                            false,
                        ))
                        .child(
                            tab(
                                "plan-review-tab-changes",
                                "Changes",
                                self.showing_changes,
                                true,
                            )
                            .child(DiffStat::new(
                                "plan-review-version-stat",
                                changes.added,
                                changes.removed,
                            )),
                        ),
                )
                .child(
                    div()
                        .px_4()
                        .py_1()
                        .child(Label::new(line).size(LabelSize::XSmall).color(Color::Muted)),
                ),
        )
    }

    /// The terminal the reviewed text came from, by its view's id, when it came from one.
    fn terminal_id(&self) -> Option<u64> {
        match &self.subject {
            Subject::Plan { view } => Some(view.as_u64()),
            Subject::Reply(ReplyOrigin::Terminal { view, .. }) => Some(view.entity_id().as_u64()),
            Subject::File { .. } | Subject::Reply(ReplyOrigin::Thread { .. }) => None,
        }
    }

    /// This terminal's turns (#893, D3), newest first, as the Review board draws them: each with
    /// its files and what was done with it, Show Diff, Keep and Undo This Turn….
    fn render_turns(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let id = self.terminal_id()?;
        let turns = crate::turns::Turns::of(id, cx);
        if turns.is_empty() {
            return None;
        }
        let colors = cx.theme().colors();
        let rows = turns.iter().enumerate().rev().map(|(index, turn)| {
            let (state, state_color) = turn_state(turn);
            let files = turn_files(turn);
            let tint = state_color.color(cx);
            let undone = turn.mark == Some(TurnMark::Undone);
            let (show, keep, undo) = (turn.sha.clone(), turn.sha.clone(), turn.sha.clone());
            v_flex()
                .gap_1()
                .p_2()
                .rounded_md()
                .border_1()
                .border_color(colors.border)
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            div().flex_1().min_w_0().child(
                                Label::new(format!("Turn {} \u{b7} {}", index + 1, turn.title))
                                    .size(LabelSize::Small)
                                    .truncate(),
                            ),
                        )
                        .child(
                            div()
                                .flex_none()
                                .px_1p5()
                                .rounded_full()
                                .bg(tint.opacity(0.15))
                                .child(
                                    Label::new(state).size(LabelSize::XSmall).color(state_color),
                                ),
                        ),
                )
                .child(
                    Label::new(files)
                        .size(LabelSize::XSmall)
                        .buffer_font(cx)
                        .color(Color::Muted)
                        .truncate(),
                )
                .child(
                    h_flex()
                        .gap_1()
                        .child(
                            action_button(100 + index * 3, "Show Diff", Role::Quiet, true, cx)
                                .on_click(cx.listener(move |_, _, window, cx| {
                                    Self::show_turn(id, show.clone(), window, cx);
                                })),
                        )
                        .when(!undone, |row| {
                            row.child(
                                action_button(
                                    101 + index * 3,
                                    "Keep",
                                    Role::Plain,
                                    turn.mark.is_none(),
                                    cx,
                                )
                                .on_click(move |_, _, cx| crate::turn_undo::keep(id, &keep, cx)),
                            )
                            .child(
                                action_button(
                                    102 + index * 3,
                                    "Undo This Turn…",
                                    Role::Plain,
                                    true,
                                    cx,
                                )
                                .on_click(move |_, window, cx| {
                                    crate::turn_undo::undo(id, &undo, window, cx);
                                }),
                            )
                        }),
                )
        });
        Some(
            v_flex()
                .gap_1p5()
                .child(
                    Label::new("This terminal's turns")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .children(rows)
                .into_any_element(),
        )
    }

    /// Opens the turn `sha` of the terminal `id` in Zed's commit view, in the workspace that holds
    /// this tab, as the rail's turn row does (#509).
    fn show_turn(id: u64, sha: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(repository) = crate::turns::Turns::repository_of(id, &sha, cx) else {
            return;
        };
        let me = cx.entity_id();
        let workspace = crate::mcp::window_workspaces(window, cx)
            .into_iter()
            .find(|workspace| {
                workspace
                    .read(cx)
                    .items_of_type::<Self>(cx)
                    .any(|tab| tab.entity_id() == me)
            });
        let Some(workspace) = workspace else {
            return;
        };
        git_ui::commit_view::CommitView::open(
            sha,
            repository,
            workspace.downgrade(),
            None,
            None,
            window,
            cx,
        );
    }

    /// A file's or a reply's Send Notes, with how many notes it sends (#729).
    fn send_notes_button(unsent: usize, cx: &Context<Self>) -> impl IntoElement {
        let label = if unsent > 0 {
            format!("Send Review to Agent ({unsent})")
        } else {
            "Send Review to Agent".to_string()
        };
        h_flex().child(
            action_button(13, &label, Role::Primary, unsent > 0, cx).when(unsent > 0, |button| {
                button.on_click(cx.listener(|tab, _, window, cx| tab.send_notes(window, cx)))
            }),
        )
    }

    /// A card's Accept recommended and Skip (#728).
    fn card_buttons(
        index: usize,
        offer_recommended: bool,
        skipped: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .id(("plan-review-card-buttons", index))
            .gap_2()
            .when(offer_recommended, |row| {
                row.child(
                    action_button(0, "Accept recommended", Role::Primary, true, cx).on_click(
                        cx.listener(move |tab, _, _, cx| {
                            tab.accept_recommended(index, cx);
                        }),
                    ),
                )
            })
            .child(
                action_button(
                    1,
                    if skipped { "Unskip" } else { "Skip" },
                    Role::Quiet,
                    true,
                    cx,
                )
                .on_click(cx.listener(move |tab, _, _, cx| tab.skip(index, cx))),
            )
    }

    /// The foot under a plan (#924): Send Answers while answers wait, else Approve, then Send
    /// Feedback (N) with how many notes it sends, and the hint at the row's right.
    fn render_foot(&self, cx: &Context<Self>) -> impl IntoElement {
        let waiting = self.waiting().is_some();
        let has_feedback = !self.notes.is_empty() || !self.overall_note(cx).is_empty();
        let answers_pending = self.answers_pending(cx);
        let (unsent, overall) = self.unsent(cx);
        let unsent = unsent.len() + usize::from(!overall.is_empty());
        let feedback = if unsent > 0 {
            format!("Send Feedback ({unsent})")
        } else {
            "Send Feedback".to_string()
        };
        h_flex()
            .flex_none()
            .px_4()
            .py_2()
            .gap_2()
            .border_t_1()
            .border_color(cx.theme().colors().border)
            // An approval carries no text, so unsent answers take Approve's place (#728).
            .when(answers_pending, |row| {
                row.child(
                    action_button(10, "Send Answers", Role::Primary, waiting, cx)
                        .when(waiting, |button| {
                            button.on_click(cx.listener(|tab, _, _, cx| tab.send_answers(cx)))
                        }),
                )
            })
            .when(!answers_pending, |row| {
                row.child(
                    action_button(11, "Approve", Role::Primary, waiting, cx)
                        .when(waiting, |button| {
                            button.on_click(cx.listener(|tab, _, _, cx| tab.approve(cx)))
                        }),
                )
            })
            .child({
                let enabled = waiting && has_feedback;
                action_button(12, &feedback, Role::Plain, enabled, cx).when(enabled, |button| {
                    button.on_click(cx.listener(|tab, _, _, cx| tab.send_feedback(cx)))
                })
            })
            .child(div().flex_1())
            .when(self.taking_notes(), |row| {
                row.child(
                    Label::new("Select a passage and press c to comment")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
    }
}

/// A turn's state as the board's pill and its colour (#924).
const fn turn_state(turn: &crate::turns::Turn) -> (&'static str, Color) {
    match turn.mark {
        Some(TurnMark::Kept) => ("kept", Color::Success),
        Some(TurnMark::Undone) => ("undone", Color::Muted),
        None if turn.failed => ("failed", Color::Error),
        None if turn.injected => ("injected", Color::Warning),
        None => ("new", Color::Accent),
    }
}

/// A turn's files with their lines, else how many, for a turn recorded before #924 (D4).
fn turn_files(turn: &crate::turns::Turn) -> String {
    if turn.file_counts.is_empty() {
        return match turn.files {
            1 => "1 file".to_string(),
            count => format!("{count} files"),
        };
    }
    turn.file_counts
        .iter()
        .map(|file| format!("{} +{} \u{2212}{}", file.name, file.added, file.removed))
        .collect::<Vec<_>>()
        .join(" \u{b7} ")
}

/// The lines added and removed from `before` to `after`, and their diff in a read-only editor as
/// Zed draws a diff (#727).
fn changes_from(
    before: String,
    after: String,
    languages: Arc<LanguageRegistry>,
    window: &mut Window,
    cx: &mut Context<PlanReview>,
) -> Changes {
    let (added, removed) = language::line_diff(&before, &after)
        .iter()
        .fold((0, 0), |(added, removed), (old, new)| {
            (added + new.len(), removed + old.len())
        });
    let diff = cx.new(|_| {
        acp_thread::Diff::finalized("plan.md".to_string(), Some(before), after, languages)
    });
    let multibuffer = diff.update(cx, acp_thread::Diff::multibuffer);
    let editor = cx.new(|cx| {
        let mut editor = Editor::new(EditorMode::full(), multibuffer, None, window, cx);
        editor.set_show_gutter(false, cx);
        editor.set_read_only(true);
        editor.set_show_git_diff_gutter(false, cx);
        editor.set_expand_all_diff_hunks(cx);
        editor.set_soft_wrap_mode(settings::SoftWrap::EditorWidth, cx);
        editor
    });
    Changes {
        added,
        removed,
        _diff: diff,
        editor,
    }
}

/// One of the tab's actions as #870 draws a button (#881); a disabled one is dimmed and the caller
/// gives it no click (D1).
fn action_button(index: usize, label: &str, role: Role, enabled: bool, cx: &App) -> Stateful<Div> {
    crate::prompts::render_button(index, label, role, false, cx)
        .when(enabled, Styled::cursor_pointer)
        .when(!enabled, |button| button.opacity(0.5))
}

/// The board's chip on the choice the agent recommends, in place of "(recommended)" in its text
/// (#881).
fn recommended_chip(cx: &App) -> impl IntoElement {
    div()
        .px_1p5()
        .rounded_sm()
        .bg(cx.theme().colors().text_accent.opacity(0.16))
        .child(
            Label::new("Recommended")
                .size(LabelSize::XSmall)
                .color(Color::Accent),
        )
}

/// A choice as its card shows it: its label and its reason.
fn choice_label(choice: &plan_questions::Choice) -> String {
    choice.reason.as_ref().map_or_else(
        || choice.label.clone(),
        |reason| format!("{} — {reason}", choice.label),
    )
}

/// The plan's runs of Markdown, each its own entity, and its question cards (#728).
fn parts_of(
    plan: &str,
    window: &mut Window,
    cx: &mut Context<PlanReview>,
) -> (Vec<Part>, Vec<Card>) {
    let mut parts = Vec::new();
    let mut cards = Vec::new();
    for part in plan_questions::plan_parts(plan) {
        match part {
            PlanPart::Text { start, text } => {
                let len = text.len();
                let markdown = cx.new(|cx| Markdown::new(text.into(), None, None, cx));
                parts.push(Part::Text {
                    start,
                    len,
                    markdown,
                });
            }
            PlanPart::Question(question) => {
                let context = (!question.context.is_empty()).then(|| {
                    let context = question.context.clone();
                    cx.new(|cx| Markdown::new(context.into(), None, None, cx))
                });
                let placeholder = if question.kind == QuestionKind::Text {
                    "Your answer"
                } else {
                    "Other…"
                };
                let words = cx.new(|cx| {
                    let mut editor = Editor::single_line(window, cx);
                    editor.set_placeholder_text(placeholder, window, cx);
                    editor
                });
                let edited = cx.subscribe(&words, |_, _, event: &EditorEvent, cx| {
                    if matches!(event, EditorEvent::BufferEdited) {
                        cx.notify();
                    }
                });
                let picked = question
                    .choices
                    .iter()
                    .map(|choice| choice.settled)
                    .collect();
                parts.push(Part::Card(cards.len()));
                cards.push(Card {
                    question,
                    context,
                    picked,
                    words,
                    skipped: false,
                    accepted: false,
                    _edited: edited,
                });
            }
        }
    }
    (parts, cards)
}

impl Focusable for PlanReview {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.plan_focus.clone()
    }
}

impl EventEmitter<ItemEvent> for PlanReview {}

impl Render for PlanReview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let background = cx.theme().colors().editor_background;
        let border = cx.theme().colors().border;
        let style = MarkdownStyle::themed(MarkdownFont::Preview, window, cx);
        // A card's context sits under its question, in the card's smaller, muted type.
        let mut context_style = MarkdownStyle::themed(MarkdownFont::Editor, window, cx);
        let small = TextSize::Small.rems(cx);
        // The runs take their color from the base style and their size from the container's.
        context_style.base_text_style.font_size = small.into();
        context_style.container_style.text.font_size = Some(small.into());
        context_style.paragraph_line_height = relative(1.5);
        context_style.base_text_style.color = cx.theme().colors().text_muted;
        let tabs = self.render_tabs(cx);
        let is_plan = self.plan_view().is_some();
        // A plan's own heading opens it; a file or a reply opens with the tab's title (#924).
        let headline = (!is_plan).then(|| {
            div()
                .pb_2()
                .child(Headline::new(self.title.clone()).size(HeadlineSize::Small))
        });
        let foot = is_plan.then(|| self.render_foot(cx));
        h_flex()
            .key_context("PlanReview")
            .size_full()
            .bg(background)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .children(tabs)
                    .child(
                        if let Some(editor) = self
                            .changes
                            .as_ref()
                            .filter(|_| self.showing_changes)
                            .map(|changes| changes.editor.clone())
                        {
                            // The diff takes no notes: its own area, out of the plan's key context.
                            div()
                                .id("plan-review-changes")
                                .flex_1()
                                .min_h_0()
                                .p_2()
                                .child(editor)
                                .into_any_element()
                        } else {
                            v_flex()
                                .id("plan-review-plan")
                                .key_context("PlanReviewPlan")
                                .track_focus(&self.plan_focus)
                                .on_action(cx.listener(Self::comment))
                                // Before the Markdown's own handling, which may stop the event.
                                .capture_any_mouse_down(cx.listener(
                                    |tab, _: &MouseDownEvent, window, cx| {
                                        window.focus(&tab.plan_focus, cx);
                                    },
                                ))
                                .on_mouse_down(
                                    MouseButton::Right,
                                    cx.listener(|tab, event: &MouseDownEvent, window, cx| {
                                        tab.open_menu(event.position, window, cx);
                                    }),
                                )
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .p_4()
                                .children(headline)
                                .children(self.parts.iter().map(|part| {
                                    match part {
                                        Part::Text { markdown, .. } => {
                                            MarkdownElement::new(markdown.clone(), style.clone())
                                                .into_any_element()
                                        }
                                        Part::Card(index) => self
                                            .render_card(*index, &context_style, cx)
                                            .into_any_element(),
                                    }
                                }))
                                .into_any_element()
                        },
                    )
                    .children(foot),
            )
            .child(self.render_column(border, cx))
            .children(self.menu.as_ref().map(|(menu, position, _)| {
                deferred(
                    anchored()
                        .position(*position)
                        .anchor(Anchor::TopLeft)
                        .child(menu.clone()),
                )
                .with_priority(1)
            }))
    }
}

impl Item for PlanReview {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        self.title.clone()
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(match self.subject {
            Subject::Plan { .. } => IconName::FileDoc,
            Subject::File { .. } => IconName::FileMarkdown,
            Subject::Reply(_) => IconName::Chat,
        }))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(match &self.subject {
            Subject::Plan { .. } => SharedString::new_static("Claude Code's plan, for your review"),
            Subject::File { path, .. } => SharedString::from(path.to_string_lossy().into_owned()),
            Subject::Reply(_) => SharedString::new_static("An agent's last reply, for your notes"),
        })
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
