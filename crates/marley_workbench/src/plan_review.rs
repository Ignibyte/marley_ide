//! Plan review in a tab (#726).
//!
//! While `marley.plan_review` is on, Marley's new local terminals name `ExitPlanMode` in
//! `MARLEY_APPROVE_TOOLS`, and the shared Claude Code plugin (at a revision that reads it) hands that
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

use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use editor::{Editor, EditorEvent, EditorMode};
use gpui::{
    Anchor, App, AppContext as _, Context, DismissEvent, Entity, EntityId, EventEmitter,
    FocusHandle, Focusable, FontWeight, Global, MouseButton, MouseDownEvent, Pixels, Point,
    SharedString, Subscription, WeakEntity, Window, actions, anchored, deferred,
};
use language::LanguageRegistry;
use markdown::{Markdown, MarkdownElement, MarkdownFont, MarkdownStyle};
use marley_agent::PlanNote;
use marley_agent::plan_questions::{self, Answer, PlanPart, Question, QuestionKind};
use marley_agent::report::{self, Refusal};
use marley_mcp::agent_socket::AgentRequest;
use marley_rail::{InboxEntry, InboxKind};
use serde_json::Value;
use settings::SettingsStore;
use terminal_view::TerminalView;
use ui::{Checkbox, ContextMenu, DiffStat, ToggleState, prelude::*};
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

actions!(
    plan_review,
    [
        /// Comments on the plan's selected passage.
        #[derive(Eq)]
        Comment
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
}

/// Follows the setting; `agent_reports::start` calls it once. Marley's keymap binds `c` to
/// [`Comment`] in the plan's area.
pub fn init(cx: &mut App) {
    cx.set_global(PendingPlans::default());
    reconcile(cx);
    cx.observe_global::<SettingsStore>(reconcile).detach();
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
    let project = SharedString::from(project_name(workspace, cx));
    let terminal_focused = view.focus_handle(cx).contains_focused(window, cx);
    workspace.update(cx, |workspace, cx| {
        let open = workspace
            .items_of_type::<PlanReview>(cx)
            .find(|tab| tab.read(cx).view == view_id);
        let tab = if let Some(tab) = open {
            workspace.activate_item(&tab, false, terminal_focused, window, cx);
            tab
        } else {
            let tab = cx.new(|cx| PlanReview::new(view_id, project, window, cx));
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
fn project_name(workspace: &Entity<Workspace>, cx: &App) -> String {
    workspace
        .read(cx)
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
        ask: "Plan to review".to_string(),
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

/// A note on a passage of the plan.
struct Note {
    range: Range<usize>,
    quote: String,
    comment: String,
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
    Approved,
    FeedbackSent,
    /// The caller stopped waiting, or its wait ran out.
    Withdrawn,
}

/// The Plan tab.
pub(crate) struct PlanReview {
    view: EntityId,
    project: SharedString,
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
        view: EntityId,
        project: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let overall = cx.new(|cx| {
            let mut editor = Editor::auto_height(1, 4, window, cx);
            editor.set_soft_wrap_mode(settings::SoftWrap::EditorWidth, cx);
            editor.set_placeholder_text("A note on the whole plan", window, cx);
            editor
        });
        Self {
            view,
            project,
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
        // A card's answer field sits in the plan's area: the letter is its own there.
        if !self.plan_focus.is_focused(window) {
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
        if self.waiting().is_none() {
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
            });
            self.highlight(cx);
        }
        window.focus(&self.plan_focus, cx);
        cx.notify();
    }

    fn remove_note(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.notes.len() {
            self.notes.remove(index);
            self.highlight(cx);
            cx.notify();
        }
    }

    fn overall_note(&self, cx: &App) -> String {
        self.overall.read(cx).text(cx).trim().to_string()
    }

    fn approve(&mut self, cx: &mut Context<Self>) {
        let Some(nonce) = self.waiting() else {
            return;
        };
        self.state = if answer(self.view, nonce, Decision::Approve, cx) {
            State::Approved
        } else {
            State::Withdrawn
        };
        self.composing = None;
        cx.notify();
    }

    fn send_feedback(&mut self, cx: &mut Context<Self>) {
        let Some(nonce) = self.waiting() else {
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
        self.state = if answer(self.view, nonce, Decision::Deny(reason), cx) {
            State::FeedbackSent
        } else {
            State::Withdrawn
        };
        self.composing = None;
        cx.notify();
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
        let Some(nonce) = self.waiting() else {
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
        self.state = if answer(self.view, nonce, Decision::Deny(reason), cx) {
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
                h_flex()
                    .justify_between()
                    .child(
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
                    )
                    .when(waiting && !settled, |header| {
                        header.child(Self::card_buttons(
                            index,
                            recommended.is_some() && state == CardState::Open,
                            card.skipped,
                            cx,
                        ))
                    }),
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
                        let label = choice_label(item, recommended.as_deref());
                        Checkbox::new(
                            ("plan-review-choice", index * 1000 + choice),
                            ToggleState::from(card.picked.get(choice).copied().unwrap_or(false)),
                        )
                        .label(label)
                        .label_size(LabelSize::Small)
                        .disabled(settled || !waiting)
                        .on_click(cx.listener(
                            move |tab, state: &ToggleState, _, cx| {
                                tab.pick(index, choice, state.selected(), cx);
                            },
                        ))
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
            .into_any_element()
    }

    fn render_notes(&self, cx: &Context<Self>) -> impl IntoElement {
        let surface = cx.theme().colors().surface_background;
        let focused_border = cx.theme().colors().border_focused;
        let waiting = self.waiting().is_some();
        v_flex()
            .gap_2()
            .children(self.notes.iter().enumerate().map(|(index, note)| {
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
                                Label::new(format!("“{}”", note.quote.trim()))
                                    .size(LabelSize::Small)
                                    .color(Color::Muted)
                                    .italic()
                                    .line_clamp(2),
                            )
                            .when(waiting, |row| {
                                row.child(
                                    IconButton::new(
                                        ("plan-review-remove-note", index),
                                        IconName::Close,
                                    )
                                    .icon_size(IconSize::Small)
                                    .tooltip(ui::Tooltip::text("Remove the note"))
                                    .on_click(cx.listener(
                                        move |tab, _, _, cx| {
                                            tab.remove_note(index, cx);
                                        },
                                    )),
                                )
                            }),
                    )
                    .child(Label::new(note.comment.clone()).size(LabelSize::Small))
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
        let waiting = self.waiting().is_some();
        let has_feedback = !self.notes.is_empty() || !self.overall_note(cx).is_empty();
        let answers_pending = self.answers_pending(cx);
        let (answered, questions) = self.answered(cx);
        let status = match self.state {
            State::Waiting(_) => "Claude Code waits for your answer.",
            State::Approved => "Approved.",
            State::FeedbackSent => "Feedback sent.",
            State::Withdrawn => "This plan went back to the terminal's own dialog.",
        };
        v_flex()
            .id("plan-review-notes")
            .w(rems(24.))
            .h_full()
            .overflow_y_scroll()
            .border_l_1()
            .border_color(border)
            .p_3()
            .gap_3()
            .child(Label::new(format!("Plan · {}", self.project)).size(LabelSize::Large))
            .when_some(self.changes.as_ref(), |column, changes| {
                let showing_changes = self.showing_changes;
                column.child(
                    h_flex()
                        .gap_2()
                        .child(
                            Label::new(format!("Version {}", self.versions.len()))
                                .size(LabelSize::Small),
                        )
                        .child(DiffStat::new(
                            "plan-review-version-stat",
                            changes.added,
                            changes.removed,
                        ))
                        .child(
                            Button::new(
                                "plan-review-changes",
                                if showing_changes { "Plan" } else { "Changes" },
                            )
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(|tab, _, _, cx| {
                                tab.showing_changes = !tab.showing_changes;
                                cx.notify();
                            })),
                        ),
                )
            })
            .when(questions > 0, |column| {
                column.child(
                    Label::new(format!("{answered} of {questions} answered"))
                        .size(LabelSize::Small),
                )
            })
            .child(Label::new(status).size(LabelSize::Small).color(if waiting {
                Color::Accent
            } else {
                Color::Muted
            }))
            .when(waiting, |column| {
                column.child(
                    Label::new("Select a passage, then right-click Comment… or press c.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
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
            .child(Self::answer_buttons(
                waiting,
                answers_pending,
                has_feedback,
                cx,
            ))
    }

    /// A card's Accept recommended and Skip (#728).
    fn card_buttons(
        index: usize,
        offer_recommended: bool,
        skipped: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .gap_2()
            .when(offer_recommended, |row| {
                row.child(
                    Button::new(("plan-review-accept", index), "Accept recommended")
                        .label_size(LabelSize::Small)
                        .on_click(cx.listener(move |tab, _, _, cx| {
                            tab.accept_recommended(index, cx);
                        })),
                )
            })
            .child(
                Button::new(
                    ("plan-review-skip", index),
                    if skipped { "Unskip" } else { "Skip" },
                )
                .label_size(LabelSize::Small)
                .on_click(cx.listener(move |tab, _, _, cx| tab.skip(index, cx))),
            )
    }

    /// Approve and Send Feedback, or Send Answers while answers wait to go.
    fn answer_buttons(
        waiting: bool,
        answers_pending: bool,
        has_feedback: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .gap_2()
            // An approval carries no text, so unsent answers take Approve's place (#728).
            .when(answers_pending, |row| {
                row.child(
                    Button::new("plan-review-answers", "Send Answers")
                        .style(ButtonStyle::Filled)
                        .disabled(!waiting)
                        .on_click(cx.listener(|tab, _, _, cx| tab.send_answers(cx))),
                )
            })
            .when(!answers_pending, |row| {
                row.child(
                    Button::new("plan-review-approve", "Approve")
                        .style(ButtonStyle::Filled)
                        .disabled(!waiting)
                        .on_click(cx.listener(|tab, _, _, cx| tab.approve(cx))),
                )
                .child(
                    Button::new("plan-review-feedback", "Send Feedback")
                        .disabled(!waiting || !has_feedback)
                        .on_click(cx.listener(|tab, _, _, cx| tab.send_feedback(cx))),
                )
            })
    }
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
    let diff = cx.new(|cx| {
        acp_thread::Diff::finalized("plan.md".to_string(), Some(before), after, languages, cx)
    });
    let multibuffer = diff.read(cx).multibuffer().clone();
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

/// A choice as its card shows it: its label, its reason, and whether the agent recommends it.
fn choice_label(choice: &plan_questions::Choice, recommended: Option<&str>) -> String {
    let label = choice.reason.as_ref().map_or_else(
        || choice.label.clone(),
        |reason| format!("{} — {reason}", choice.label),
    );
    if recommended == Some(choice.label.as_str()) {
        format!("{label} (recommended)")
    } else {
        label
    }
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
        h_flex()
            .key_context("PlanReview")
            .size_full()
            .bg(background)
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
                        .h_full()
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
                        .h_full()
                        .overflow_y_scroll()
                        .p_4()
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
        SharedString::from(format!("Plan · {}", self.project))
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::FileDoc))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "Claude Code's plan, for your review",
        ))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
