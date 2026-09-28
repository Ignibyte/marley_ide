//! Decisions (#565): the System One layer's calls today, each with what it sent and what came
//! back, the day's spend against the budget, and where the key comes from.
//!
//! The view reads today's file when it opens and follows the `SystemOne` global for the calls
//! made after, so it polls nothing. A row opens to the masked state as sent, the answers as they
//! came and any error. Set Key writes the keyring at the provider's URL; the key's value is
//! never shown.

use std::collections::HashSet;

use editor::Editor;
use gpui::{App, Entity, EventEmitter, FocusHandle, Focusable, Subscription, Task};
use marley_system_one::files::{self, CallRow};
use menu::Confirm;
use ui::{Button, Color, Label, LabelSize, prelude::*};
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::Item;

use crate::SystemOneCheck;
use crate::system_one::{self, KeySource, SystemOne};

/// The Decisions view.
pub struct DecisionsView {
    focus_handle: FocusHandle,
    /// Today's rows, newest first.
    rows: Vec<CallRow>,
    /// The ids of the rows shown, so a row that is in the file and in the session shows once.
    shown: HashSet<String>,
    /// The rows opened to their state and answers.
    expanded: HashSet<String>,
    /// The field the key is typed into, while it shows.
    key_field: Option<Entity<Editor>>,
    /// What the last Set Key or Forget Key came to.
    key_note: Option<SharedString>,
    _layer: Subscription,
    _loading: Task<()>,
}

impl std::fmt::Debug for DecisionsView {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DecisionsView")
            .field("rows", &self.rows.len())
            .field("expanded", &self.expanded.len())
            .field("key_field", &self.key_field.is_some())
            .finish_non_exhaustive()
    }
}

/// Opens Decisions in the workspace, or brings forward the one open there.
pub fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let open = workspace.items_of_type::<DecisionsView>(cx).next();
    if let Some(open) = open {
        workspace.activate_item(&open, true, true, window, cx);
        return;
    }
    let view = cx.new(|cx| DecisionsView::new(window, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}

impl DecisionsView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let layer = cx.observe_global_in::<SystemOne>(window, |view, _, cx| {
            view.take_session_rows(cx);
            cx.notify();
        });
        let dir = system_one::folder();
        let day = system_one::today();
        let loading = cx.spawn_in(window, async move |view, cx| {
            let read = cx
                .background_spawn(futures::future::lazy(move |_| {
                    files::read_day_in(&dir, &day)
                }))
                .await;
            view.update(cx, |view, cx| {
                match read {
                    Ok(rows) => view.add_rows(rows),
                    Err(error) => log::warn!("decisions: reading today's calls: {error:#}"),
                }
                cx.notify();
            })
            .log_err();
        });
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            rows: Vec::new(),
            shown: HashSet::new(),
            expanded: HashSet::new(),
            key_field: None,
            key_note: None,
            _layer: layer,
            _loading: loading,
        };
        view.take_session_rows(cx);
        view
    }

    /// Adds the rows not shown yet, keeping the newest first.
    fn add_rows(&mut self, rows: impl IntoIterator<Item = CallRow>) {
        let mut added = false;
        for row in rows {
            if self.shown.insert(row.id.clone()) {
                self.rows.push(row);
                added = true;
            }
        }
        if added {
            self.rows.sort_by(|left, right| right.time.cmp(&left.time));
        }
    }

    /// Adds this session's rows from the layer.
    fn take_session_rows(&mut self, cx: &App) {
        let rows: Vec<CallRow> = cx
            .try_global::<SystemOne>()
            .map(|layer| layer.rows().to_vec())
            .unwrap_or_default();
        self.add_rows(rows);
    }

    fn show_key_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let field = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_masked(true, cx);
            editor.set_placeholder_text("Paste the key, then press Enter", window, cx);
            editor
        });
        field.focus_handle(cx).focus(window, cx);
        self.key_field = Some(field);
        self.key_note = None;
        cx.notify();
    }

    fn save_key(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let Some(field) = self.key_field.take() else {
            return;
        };
        let key = field.read(cx).text(cx);
        if key.trim().is_empty() {
            self.key_field = Some(field);
            return;
        }
        self.focus_handle.focus(window, cx);
        let write = system_one::store_key(&key, cx);
        cx.spawn(async move |view, cx| {
            let note: SharedString = match write.await {
                Ok(()) => SharedString::new_static("The key is in the keyring."),
                Err(error) => format!("The key was not written: {error:#}").into(),
            };
            view.update(cx, |view, cx| {
                view.key_note = Some(note);
                cx.notify();
            })
            .log_err();
        })
        .detach();
        cx.notify();
    }

    /// Escape in the key field: the single-line editor lets `editor::Cancel` through, and the
    /// view's own Escape is `menu::Cancel`.
    fn cancel_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.key_field.take().is_some() {
            self.focus_handle.focus(window, cx);
            cx.notify();
        }
    }

    fn forget_key(cx: &mut Context<Self>) {
        let delete = system_one::forget_key(cx);
        cx.spawn(async move |view, cx| {
            let note: SharedString = match delete.await {
                Ok(()) => SharedString::new_static("The key is gone from the keyring."),
                Err(error) => format!("The key was not removed: {error:#}").into(),
            };
            view.update(cx, |view, cx| {
                view.key_note = Some(note);
                cx.notify();
            })
            .log_err();
        })
        .detach();
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let (lines, breaker_open) = cx.try_global::<SystemOne>().map_or_else(
            || (vec![SharedString::new_static("System One is off.")], false),
            |layer| {
                let settings = layer.settings();
                let mut lines = Vec::new();
                if settings.enabled {
                    let calls = match self.rows.len() {
                        1 => "1 call".to_string(),
                        count => format!("{count} calls"),
                    };
                    lines.push(SharedString::from(format!(
                        "{calls} today · {} spent of the {}¢ budget",
                        system_one::cents(layer.spent_today()),
                        settings.daily_budget_cents
                    )));
                } else {
                    lines.push(SharedString::new_static(
                        "System One is off: turn it on in the Marley settings.",
                    ));
                }
                lines.push(SharedString::from(format!(
                    "Provider: {} · {}",
                    system_one::provider_name(settings.provider),
                    settings.model
                )));
                lines.push(SharedString::from(format!("Key: {}", layer.key_source())));
                (lines, layer.breaker_open())
            },
        );
        v_flex()
            .gap_1()
            .children(
                lines
                    .into_iter()
                    .map(|line| Label::new(line).size(LabelSize::Small)),
            )
            .when(breaker_open, |this| {
                this.child(
                    Label::new("The breaker is open: calls wait two minutes after five failures.")
                        .size(LabelSize::Small)
                        .color(Color::Error),
                )
            })
    }

    fn render_buttons(cx: &Context<Self>) -> impl IntoElement {
        let from_environment = cx
            .try_global::<SystemOne>()
            .is_some_and(|layer| layer.key_source() == KeySource::Environment);
        h_flex()
            .gap_2()
            .child(
                Button::new("decisions-run-check", "Run Check").on_click(cx.listener(
                    |_, _, window, cx| {
                        window.dispatch_action(Box::new(SystemOneCheck), cx);
                    },
                )),
            )
            .child(
                Button::new("decisions-set-key", "Set Key")
                    .on_click(cx.listener(|view, _, window, cx| view.show_key_field(window, cx))),
            )
            .child(
                Button::new("decisions-forget-key", "Forget Key")
                    .on_click(cx.listener(|_, _, _, cx| Self::forget_key(cx))),
            )
            .when(from_environment, |this| {
                this.child(
                    Label::new("MARLEY_SYSTEM_ONE_KEY is set, and it comes before the keyring.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
    }

    fn render_row(&self, row: &CallRow, cx: &Context<Self>) -> impl IntoElement {
        let open = self.expanded.contains(&row.id);
        let id = row.id.clone();
        let time = row
            .time
            .get(11..19)
            .map_or_else(|| row.time.clone(), str::to_string);
        let reading = if row.mode == "shadow" && !row.failed {
            format!("would show: {}", row.reading)
        } else {
            row.reading.clone()
        };
        let mut numbers = Vec::new();
        if let Some(latency) = row.latency_ms {
            numbers.push(format!("{latency} ms"));
        }
        if let Some(tokens) = row.input_tokens.filter(|tokens| *tokens > 0) {
            numbers.push(format!("{} tokens", system_one::thousands(tokens)));
        }
        if let Some(cost) = row.cost.filter(|cost| *cost > 0) {
            numbers.push(system_one::cents(cost));
        }
        let who = format!(
            "{} · {} · {} · {}",
            row.use_name,
            row.project.as_deref().unwrap_or("no project"),
            row.provider,
            row.model
        );
        v_flex()
            .id(SharedString::from(format!("decision-{}", row.id)))
            .w_full()
            .px_2()
            .py_1()
            .gap_1()
            .rounded_sm()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().colors().element_hover))
            .on_click(cx.listener(move |view, _, _, cx| {
                if !view.expanded.remove(&id) {
                    let _opened = view.expanded.insert(id.clone());
                }
                cx.notify();
            }))
            .child(
                h_flex()
                    .gap_3()
                    .child(Label::new(time).size(LabelSize::Small).color(Color::Muted))
                    .child(Label::new(who).size(LabelSize::Small))
                    .child(
                        Label::new(reading)
                            .size(LabelSize::Small)
                            .color(if row.failed {
                                Color::Error
                            } else {
                                Color::Default
                            }),
                    )
                    .child(
                        Label::new(numbers.join(" · "))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .when(open, |this| {
                this.child(detail("State as sent", row.state.clone(), cx))
                    .child(detail(
                        "Answers",
                        row.answers.as_ref().map(|answers| {
                            serde_json::to_string_pretty(answers).unwrap_or_default()
                        }),
                        cx,
                    ))
                    .child(detail("Error", row.error.clone(), cx))
            })
    }
}

/// A labeled block of a row opened up, or nothing when there is nothing to show.
fn detail(title: &'static str, text: Option<String>, cx: &App) -> impl IntoElement {
    div().when_some(text.filter(|text| !text.is_empty()), |this, text| {
        this.child(
            v_flex()
                .pl_4()
                .child(
                    Label::new(title)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
                .child(Label::new(text).size(LabelSize::Small).buffer_font(cx)),
        )
    })
}

impl Render for DecisionsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: Vec<_> = self
            .rows
            .iter()
            .map(|row| self.render_row(row, cx).into_any_element())
            .collect();
        v_flex()
            .id("decisions")
            .key_context("Decisions")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::save_key))
            .on_action(cx.listener(|view, _: &menu::Cancel, window, cx| {
                view.cancel_key(window, cx);
            }))
            .on_action(
                cx.listener(|view, _: &editor::actions::Cancel, window, cx| {
                    view.cancel_key(window, cx);
                }),
            )
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .bg(cx.theme().colors().editor_background)
            .child(Label::new("Decisions").size(LabelSize::Large))
            .child(self.render_header(cx))
            .child(Self::render_buttons(cx))
            .when_some(self.key_field.clone(), |this, field| {
                this.child(
                    h_flex()
                        .gap_2()
                        .child(Label::new("Key").size(LabelSize::Small))
                        .child(div().w_96().child(field)),
                )
            })
            .when_some(self.key_note.clone(), |this, note| {
                this.child(Label::new(note).size(LabelSize::Small).color(Color::Muted))
            })
            .when(rows.is_empty(), |this| {
                this.child(
                    Label::new("No calls today.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
            .children(rows)
    }
}

impl Focusable for DecisionsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for DecisionsView {}

impl Item for DecisionsView {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Decisions")
    }
}
