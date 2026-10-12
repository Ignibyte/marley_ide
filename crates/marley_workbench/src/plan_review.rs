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
//! tab's plan and notes.

use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use editor::Editor;
use gpui::{
    Anchor, App, AppContext as _, Context, DismissEvent, Entity, EntityId, EventEmitter,
    FocusHandle, Focusable, Global, MouseButton, MouseDownEvent, Pixels, Point, SharedString,
    Subscription, WeakEntity, Window, actions, anchored, deferred,
};
use markdown::{Markdown, MarkdownElement, MarkdownFont, MarkdownStyle};
use marley_agent::PlanNote;
use marley_agent::report::{self, Refusal};
use marley_mcp::agent_socket::AgentRequest;
use marley_rail::{InboxEntry, InboxKind};
use serde_json::Value;
use settings::SettingsStore;
use terminal_view::TerminalView;
use ui::{ContextMenu, prelude::*};
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
            let handle = workspace.weak_handle();
            let tab = cx.new(|cx| PlanReview::new(view_id, project, handle, window, cx));
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
        tab.update(cx, |tab, cx| tab.show(plan, nonce, window, cx));
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
    markdown: Entity<Markdown>,
    notes: Vec<Note>,
    composing: Option<Composing>,
    overall: Entity<Editor>,
    state: State,
    /// The right-click menu, where it opened, and the passage selected when it did: a click on its
    /// entry clears the selection before the entry runs.
    menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,
    /// The plan's area, which the tab's focus is: `c` works there and not in the note editors.
    plan_focus: FocusHandle,
    _workspace: WeakEntity<Workspace>,
}

impl PlanReview {
    fn new(
        view: EntityId,
        project: SharedString,
        workspace: WeakEntity<Workspace>,
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
            markdown: cx.new(|cx| Markdown::new(SharedString::default(), None, None, cx)),
            notes: Vec::new(),
            composing: None,
            overall,
            state: State::Withdrawn,
            menu: None,
            plan_focus: cx.focus_handle(),
            _workspace: workspace,
        }
    }

    /// Shows a new plan waiting under `nonce`, its notes cleared.
    fn show(&mut self, plan: String, nonce: u64, window: &mut Window, cx: &mut Context<Self>) {
        self.markdown
            .update(cx, |markdown, cx| markdown.reset(plan.clone().into(), cx));
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

    /// Paints the noted passages.
    fn highlight(&self, cx: &mut Context<Self>) {
        let mut ranges: Vec<Range<usize>> =
            self.notes.iter().map(|note| note.range.clone()).collect();
        ranges.sort_by_key(|range| (range.start, range.end));
        self.markdown.update(cx, |markdown, cx| {
            markdown.set_search_highlights(ranges, None, cx);
        });
    }

    const fn waiting(&self) -> Option<u64> {
        match self.state {
            State::Waiting(nonce) => Some(nonce),
            _ => None,
        }
    }

    /// `c`: a note on the passage selected now.
    fn comment(&mut self, _: &Comment, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(range) = self.markdown.read(cx).selected_range() {
            self.start_note(range, window, cx);
        }
    }

    /// Opens the right-click menu, keeping the passage selected now for its Comment….
    fn open_menu(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let range = self.markdown.read(cx).selected_range();
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
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("plan-review-approve", "Approve")
                            .style(ButtonStyle::Filled)
                            .disabled(!waiting)
                            .on_click(cx.listener(|tab, _, _, cx| tab.approve(cx))),
                    )
                    .child(
                        Button::new("plan-review-feedback", "Send Feedback")
                            .disabled(!waiting || !has_feedback)
                            .on_click(cx.listener(|tab, _, _, cx| tab.send_feedback(cx))),
                    ),
            )
    }
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
        h_flex()
            .key_context("PlanReview")
            .size_full()
            .bg(background)
            .child(
                v_flex()
                    .id("plan-review-plan")
                    .key_context("PlanReviewPlan")
                    .track_focus(&self.plan_focus)
                    .on_action(cx.listener(Self::comment))
                    // Before the Markdown's own handling, which may stop the event.
                    .capture_any_mouse_down(cx.listener(|tab, _: &MouseDownEvent, window, cx| {
                        window.focus(&tab.plan_focus, cx);
                    }))
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
                    .child(MarkdownElement::new(self.markdown.clone(), style)),
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
