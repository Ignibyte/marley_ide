//! The form that records how a decision went (#660), opened from the Decisions tab.
//!
//! It holds a `FollowUpDraft`: the status, the outcome, the next day when revised and the
//! successor when superseded, picked from the decisions the tab lists. Record sends one
//! `brain_follow_up`; Rusty's refusal stays in the form with every field, and a success closes it
//! and has the tab read again.

use std::rc::Rc;

use editor::{Editor, EditorEvent};
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, Render, SharedString,
    Subscription, Task, WeakEntity, Window,
};
use marley_rusty::decisions::{
    BRAIN_FOLLOW_UP, DecisionSummary, FollowUpDraft, FollowUpStatus, Missing,
};
use picker::Picker;
use ui::{ToggleButtonGroup, ToggleButtonGroupStyle, ToggleButtonSimple, prelude::*};
use util::ResultExt as _;
use workspace::{ModalView, Workspace};

use super::decisions_tab::BrainDecisionsView;
use super::project::{Choice, LinkDelegate, OnClose, OnPick};

/// A decision the successor can be: its slug and its title.
pub(super) struct Candidate {
    pub(super) slug: String,
    pub(super) title: SharedString,
}

/// Opens the form for `summary` in the workspace's modal layer; `candidates` are the decisions
/// the successor can be, without this one.
pub(super) fn open(
    workspace: &mut Workspace,
    tab: WeakEntity<BrainDecisionsView>,
    summary: &DecisionSummary,
    candidates: Vec<Candidate>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let slug = summary.slug.clone();
    let title = if summary.title.is_empty() {
        SharedString::from(summary.slug.clone())
    } else {
        SharedString::from(summary.title.clone())
    };
    let follow_up_line = summary.follow_up_line().map(SharedString::from);
    workspace.toggle_modal(window, cx, |window, cx| {
        FollowUpModal::new(tab, slug, title, follow_up_line, candidates, window, cx)
    });
}

pub(super) struct FollowUpModal {
    tab: WeakEntity<BrainDecisionsView>,
    slug: String,
    title: SharedString,
    follow_up_line: Option<SharedString>,
    candidates: Rc<Vec<Candidate>>,
    status: Option<FollowUpStatus>,
    successor: Option<usize>,
    outcome: Entity<Editor>,
    day: Entity<Editor>,
    /// The successor picker while it shows.
    choosing: Option<Entity<Picker<LinkDelegate>>>,
    recording: Option<Task<()>>,
    /// Rusty's words when it refused the last Record.
    refusal: Option<SharedString>,
    /// Back from the successor picker: the outcome takes the focus at the next draw.
    refocus: bool,
    _subscriptions: Vec<Subscription>,
}

impl FollowUpModal {
    fn new(
        tab: WeakEntity<BrainDecisionsView>,
        slug: String,
        title: SharedString,
        follow_up_line: Option<SharedString>,
        candidates: Vec<Candidate>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let outcome = cx.new(|cx| {
            let mut editor = Editor::auto_height(3, 8, window, cx);
            editor.set_placeholder_text("How it went", window, cx);
            editor
        });
        let day = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("YYYY-MM-DD, or empty for none", window, cx);
            editor
        });
        // Record's state and hint follow the outcome's words, not its caret's blink.
        let subscriptions = vec![cx.subscribe(&outcome, |_, _, event: &EditorEvent, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                cx.notify();
            }
        })];
        Self {
            tab,
            slug,
            title,
            follow_up_line,
            candidates: Rc::new(candidates),
            status: None,
            successor: None,
            outcome,
            day,
            choosing: None,
            recording: None,
            refusal: None,
            refocus: false,
            _subscriptions: subscriptions,
        }
    }

    fn draft(&self, cx: &App) -> FollowUpDraft {
        FollowUpDraft {
            status: self.status,
            outcome: self.outcome.read(cx).text(cx),
            day: self.day.read(cx).text(cx),
            successor: self
                .successor
                .and_then(|index| self.candidates.get(index))
                .map(|candidate| candidate.slug.clone()),
        }
    }

    fn choose(&mut self, status: FollowUpStatus, window: &mut Window, cx: &mut Context<Self>) {
        self.status = Some(status);
        if status == FollowUpStatus::Superseded && self.successor.is_none() {
            self.show_successor_picker(window, cx);
        } else {
            self.choosing = None;
            self.outcome.focus_handle(cx).focus(window, cx);
        }
        cx.notify();
    }

    fn show_successor_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let choices = self
            .candidates
            .iter()
            .map(|candidate| Choice {
                label: candidate.title.clone(),
                detail: Some(SharedString::from(candidate.slug.clone())),
            })
            .collect();
        let form = cx.entity().downgrade();
        let on_pick: OnPick = Rc::new({
            let form = form.clone();
            move |index, cx| {
                form.update(cx, |form, cx| form.successor_picked(index, cx))
                    .log_err();
            }
        });
        // Escape in the picker goes back to the form; the pick goes back after the picker's
        // update, so both wait for it to end.
        let on_close: OnClose = Rc::new(move |cx| {
            let form = form.clone();
            cx.defer(move |cx| {
                form.update(cx, Self::close_successor_picker).log_err();
            });
        });
        let delegate = LinkDelegate::new(
            choices,
            "The decision that replaced it…",
            on_pick,
            on_close,
            false,
        );
        // The form's width less its padding: a picker opens at the whole modal's width.
        let picker = cx.new(|cx| {
            Picker::uniform_list(delegate, window, cx).initial_width(rems(FORM_WIDTH - 1.5))
        });
        picker.focus_handle(cx).focus(window, cx);
        self.choosing = Some(picker);
    }

    fn successor_picked(&mut self, index: usize, cx: &mut Context<Self>) {
        self.successor = Some(index);
        self.close_successor_picker(cx);
    }

    fn close_successor_picker(&mut self, cx: &mut Context<Self>) {
        self.choosing = None;
        self.refocus = true;
        cx.notify();
    }

    fn record(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.recording.is_some() || self.choosing.is_some() {
            return;
        }
        let Some(arguments) = self.draft(cx).arguments(&self.slug) else {
            return;
        };
        self.refusal = None;
        let asking = super::call_tool(BRAIN_FOLLOW_UP, arguments, cx);
        self.recording = Some(cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await;
            this.update(cx, |this, cx| {
                this.recording = None;
                match answer {
                    Ok(_) => {
                        this.tab
                            .update(cx, BrainDecisionsView::read_again)
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

    fn render_status(&self, cx: &Context<Self>) -> impl IntoElement {
        let buttons = FollowUpStatus::ALL.map(|status| {
            ToggleButtonSimple::new(
                status.label(),
                cx.listener(move |this, _, window, cx| this.choose(status, window, cx)),
            )
        });
        let chosen = self
            .status
            .and_then(|status| FollowUpStatus::ALL.iter().position(|each| *each == status))
            // An index past the three lights none: the status starts unchosen (D3).
            .unwrap_or(FollowUpStatus::ALL.len());
        ToggleButtonGroup::single_row("rusty-follow-up-status", buttons)
            .style(ToggleButtonGroupStyle::Outlined)
            .selected_index(chosen)
    }

    fn render_successor(&self, cx: &Context<Self>) -> AnyElement {
        if let Some(picker) = &self.choosing {
            return v_flex()
                .gap_1()
                .child(field_label("Replaced by"))
                .child(picker.clone())
                .into_any_element();
        }
        let chosen = self
            .successor
            .and_then(|index| self.candidates.get(index))
            .map(|candidate| candidate.title.clone());
        h_flex()
            .gap_2()
            .child(field_label("Replaced by"))
            .child(
                Label::new(
                    chosen
                        .clone()
                        .unwrap_or_else(|| SharedString::new_static("none chosen")),
                )
                .size(LabelSize::Small)
                .color(if chosen.is_some() {
                    Color::Default
                } else {
                    Color::Muted
                }),
            )
            .child(
                Button::new(
                    "rusty-follow-up-choose-successor",
                    if chosen.is_some() {
                        "Change…"
                    } else {
                        "Choose…"
                    },
                )
                .label_size(LabelSize::Small)
                .on_click(cx.listener(|this, _, window, cx| {
                    this.show_successor_picker(window, cx);
                    cx.notify();
                })),
            )
            .into_any_element()
    }

    fn render_footer(&self, cx: &Context<Self>) -> impl IntoElement {
        let missing = self.draft(cx).missing();
        let recording = self.recording.is_some();
        h_flex()
            .gap_2()
            .justify_between()
            .child(
                Label::new(missing.map_or("", Missing::hint))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Button::new(
                    "rusty-follow-up-record",
                    if recording { "Recording…" } else { "Record" },
                )
                .style(ButtonStyle::Filled)
                .disabled(missing.is_some() || recording)
                .on_click(cx.listener(|this, _, window, cx| this.record(window, cx))),
            )
    }
}

/// The form's width in rems, a picker's own.
const FORM_WIDTH: f32 = 34.;

fn field_label(text: &'static str) -> Label {
    Label::new(text).size(LabelSize::Small).color(Color::Muted)
}

impl ModalView for FollowUpModal {}

impl EventEmitter<DismissEvent> for FollowUpModal {}

impl Focusable for FollowUpModal {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.choosing.as_ref().map_or_else(
            || self.outcome.focus_handle(cx),
            |picker| picker.focus_handle(cx),
        )
    }
}

impl Render for FollowUpModal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The picker closes inside its own update, without a window; the focus it held goes to
        // the outcome here.
        if self.refocus {
            self.refocus = false;
            self.outcome.focus_handle(cx).focus(window, cx);
        }
        let status = self.status;
        v_flex()
            .key_context("RustyFollowUp menu")
            .w(rems(FORM_WIDTH))
            .p_3()
            .gap_2()
            .elevation_3(cx)
            .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| this.record(window, cx)))
            .on_action(cx.listener(|this, _: &menu::Cancel, _, cx| {
                if this.choosing.is_some() {
                    this.close_successor_picker(cx);
                } else {
                    cx.emit(DismissEvent);
                }
            }))
            .child(Label::new(format!("Follow up: {}", self.title)))
            .children(
                self.follow_up_line
                    .clone()
                    .map(|line| Label::new(line).size(LabelSize::Small).color(Color::Muted)),
            )
            .child(self.render_status(cx))
            .when(status == Some(FollowUpStatus::Superseded), |this| {
                this.child(self.render_successor(cx))
            })
            .child(field_label("Outcome"))
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .child(self.outcome.clone()),
            )
            .when(status == Some(FollowUpStatus::Revised), |this| {
                this.child(field_label("Next follow-up")).child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .border_1()
                        .border_color(cx.theme().colors().border)
                        .child(self.day.clone()),
                )
            })
            .children(self.refusal.clone().map(|refusal| {
                Label::new(refusal)
                    .size(LabelSize::Small)
                    .color(Color::Error)
            }))
            .child(self.render_footer(cx))
    }
}
