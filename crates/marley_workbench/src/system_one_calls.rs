//! The System One page (#937), the System One calls tab until then (#565; Decisions until #659).
//!
//! It shows what the layer is, its provider and the day's spend against the budget, each use with
//! its mode, and the layer's calls today, each with what it sent and what came back.
//!
//! A use's mode is set on the page as in the Settings window: both write
//! `marley.system_one.uses.<use>` in the user's settings, and both draw from them.
//!
//! The view reads today's file when it opens and follows the `SystemOne` global for the calls
//! made after, so it polls nothing. A row opens to the masked state as sent, the answers as they
//! came and any error. Set Key writes the keyring at the provider's URL; the key's value is
//! never shown.

use std::collections::HashSet;

use editor::Editor;
use fs::Fs;
use gpui::{App, Entity, EventEmitter, FocusHandle, Focusable, Subscription, Task};
use marley_system_one::files::{self, CallRow};
use menu::Confirm;
use settings::{SystemOneMode, SystemOneProvider};
use ui::{Button, Color, Label, LabelSize, prelude::*};
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::Item;

use crate::SystemOneCheck;
use crate::system_one::{self, KeySource, Slot, SystemOne};

/// A System One use as the page lists it (#937).
struct SystemOneUse {
    /// The use's key under `marley.system_one.uses`.
    key: &'static str,
    /// The use's name on the `SystemOne` board.
    name: &'static str,
}

/// The uses in the board's order (D5).
const USES: [SystemOneUse; 10] = [
    SystemOneUse {
        key: "check",
        name: "The check",
    },
    SystemOneUse {
        key: "stop_kind",
        name: "Stop kind",
    },
    SystemOneUse {
        key: "browser_find",
        name: "Browser find",
    },
    SystemOneUse {
        key: "terminal_find",
        name: "Terminal find",
    },
    SystemOneUse {
        key: "stall_kind",
        name: "Stall kind",
    },
    SystemOneUse {
        key: "click_consequence",
        name: "Click consequence",
    },
    SystemOneUse {
        key: "inbox",
        name: "Inbox risk",
    },
    SystemOneUse {
        key: "question_route",
        name: "Question route",
    },
    SystemOneUse {
        key: "running_error",
        name: "Running error",
    },
    SystemOneUse {
        key: "typed_line",
        name: "Typed line",
    },
];

/// A use's modes, in the order its track shows them.
const MODES: [SystemOneMode; 4] = [
    SystemOneMode::Off,
    SystemOneMode::Shadow,
    SystemOneMode::Suggest,
    SystemOneMode::Act,
];

const fn mode_label(mode: SystemOneMode) -> &'static str {
    match mode {
        SystemOneMode::Off => "Off",
        SystemOneMode::Shadow => "Shadow",
        SystemOneMode::Suggest => "Suggest",
        SystemOneMode::Act => "Act",
    }
}

/// Writes a use's mode to the user's settings, never a project's (D3). The `SystemOne` global
/// follows the settings, so the page and the Settings window both redraw from it (D2).
fn set_use_mode(key: &'static str, mode: SystemOneMode, cx: &App) {
    settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
        content
            .marley
            .get_or_insert_default()
            .system_one
            .get_or_insert_default()
            .uses
            .get_or_insert_default()
            .insert(key.to_owned(), mode);
    });
}

/// The System One page.
pub struct SystemOneCallsView {
    focus_handle: FocusHandle,
    /// Today's rows, newest first.
    rows: Vec<CallRow>,
    /// The ids of the rows shown, so a row that is in the file and in the session shows once.
    shown: HashSet<String>,
    /// The rows opened to their state and answers.
    expanded: HashSet<String>,
    /// The field a key is typed into, and which key, while it shows.
    key_field: Option<(Slot, Entity<Editor>)>,
    /// What the last Set Key or Forget Key, or their Cloudflare pair, came to.
    key_note: Option<SharedString>,
    _layer: Subscription,
    _loading: Task<()>,
}

impl std::fmt::Debug for SystemOneCallsView {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SystemOneCallsView")
            .field("rows", &self.rows.len())
            .field("expanded", &self.expanded.len())
            .field("key_field", &self.key_field.is_some())
            .finish_non_exhaustive()
    }
}

/// Opens the System One page in the workspace, or brings forward the one open there.
pub fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let open = workspace.items_of_type::<SystemOneCallsView>(cx).next();
    if let Some(open) = open {
        workspace.activate_item(&open, true, true, window, cx);
        return;
    }
    let view = cx.new(|cx| SystemOneCallsView::new(window, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}

impl SystemOneCallsView {
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
                    Err(error) => log::warn!("system one calls: reading today's calls: {error:#}"),
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

    fn show_key_field(&mut self, slot: Slot, window: &mut Window, cx: &mut Context<Self>) {
        let field = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_masked(true, cx);
            editor.set_placeholder_text(
                &format!("Paste the {}, then press Enter", slot.noun()),
                window,
                cx,
            );
            editor
        });
        field.focus_handle(cx).focus(window, cx);
        self.key_field = Some((slot, field));
        self.key_note = None;
        cx.notify();
    }

    fn save_key(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let Some((slot, field)) = self.key_field.take() else {
            return;
        };
        let key = field.read(cx).text(cx);
        if key.trim().is_empty() {
            self.key_field = Some((slot, field));
            return;
        }
        self.focus_handle.focus(window, cx);
        let write = system_one::store_key(slot, &key, cx);
        cx.spawn(async move |view, cx| {
            let note: SharedString = match write.await {
                Ok(()) => format!("The {} is in the keyring.", slot.noun()).into(),
                Err(error) => format!("The {} was not written: {error:#}", slot.noun()).into(),
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

    fn forget_key(slot: Slot, cx: &mut Context<Self>) {
        let delete = system_one::forget_key(slot, cx);
        cx.spawn(async move |view, cx| {
            let note: SharedString = match delete.await {
                Ok(()) => format!("The {} is gone from the keyring.", slot.noun()).into(),
                Err(error) => format!("The {} was not removed: {error:#}", slot.noun()).into(),
            };
            view.update(cx, |view, cx| {
                view.key_note = Some(note);
                cx.notify();
            })
            .log_err();
        })
        .detach();
    }

    /// The Provider and Today cards, as the `SystemOne` board draws them (#847); one line while
    /// System One is off.
    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let Some(layer) = cx.try_global::<SystemOne>() else {
            return v_flex()
                .child(
                    Label::new("System One is off.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .into_any_element();
        };
        let settings = layer.settings();
        let provider = stat_card("Provider", cx)
            .child(Label::new(format!(
                "{} · {}",
                system_one::provider_name(settings.provider),
                settings.model
            )))
            .child(
                Label::new(format!("Key: {}", layer.key_source(Slot::Direct)))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .when(
                settings.enabled && settings.uses_provider(SystemOneProvider::Cloudflare),
                |card| {
                    card.child(
                        Label::new(format!(
                            "Cloudflare: account {} · token {}",
                            settings.cloudflare_account.as_deref().unwrap_or("not set"),
                            layer.key_source(Slot::Cloudflare)
                        ))
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    )
                },
            );
        let today = self.render_today(layer, cx);
        v_flex()
            .gap_2()
            .when(!settings.enabled, |header| {
                header.child(
                    Label::new("System One is off: turn it on in the Marley settings.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
            .child(h_flex().gap_3().items_start().child(provider).child(today))
            .when(layer.breaker_open(), |header| {
                header.child(
                    Label::new("The breaker is open: calls wait two minutes after five failures.")
                        .size(LabelSize::Small)
                        .color(Color::Error),
                )
            })
            .into_any_element()
    }

    /// The Today card: the spend of the daily budget, a bar of the share, the calls count.
    fn render_today(&self, layer: &SystemOne, cx: &Context<Self>) -> Div {
        let settings = layer.settings();
        let colors = cx.theme().colors();
        let spent = layer.spent_today();
        // `spent_today` counts billionths of a cent.
        let budget = settings.daily_budget_cents.saturating_mul(1_000_000_000);
        // The share of the budget spent, in thousandths, for the bar.
        let thousandths = spent
            .saturating_mul(1000)
            .checked_div(budget)
            .unwrap_or(0)
            .min(1000);
        let share = f32::from(u16::try_from(thousandths).unwrap_or(1000)) / 1000.;
        let calls = match self.rows.len() {
            1 => "1 call".to_string(),
            count => format!("{count} calls"),
        };
        stat_card("Today", cx)
            .child(Label::new(format!(
                "{} of {}¢",
                system_one::cents(spent),
                settings.daily_budget_cents
            )))
            .child(
                div()
                    .h_1()
                    .w_full()
                    .rounded_full()
                    .bg(colors.element_background)
                    .child(
                        div()
                            .h_1()
                            .w(relative(share))
                            .rounded_full()
                            .bg(colors.text_accent),
                    ),
            )
            .child(Label::new(calls).size(LabelSize::Small).color(Color::Muted))
    }

    /// Each use with its mode (#937): a track of Off / Shadow / Suggest / Act, the set one filled,
    /// as #844's Marley rows draw a choice in the Settings window. The modes show as set while
    /// System One is off (D4); the header's line says why nothing asks.
    fn render_uses(cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let modes = cx
            .try_global::<SystemOne>()
            .map(|layer| &layer.settings().uses);
        let rows = USES.iter().enumerate().map(|(use_index, system_one_use)| {
            let set = modes
                .and_then(|modes| modes.get(system_one_use.key))
                .copied()
                .unwrap_or(SystemOneMode::Off);
            let key = system_one_use.key;
            let buttons = MODES.into_iter().enumerate().map(|(mode_index, mode)| {
                let picked = mode == set;
                Button::new(
                    ("system-one-use-mode", use_index * MODES.len() + mode_index),
                    mode_label(mode),
                )
                .style(if picked {
                    ButtonStyle::Filled
                } else {
                    ButtonStyle::Subtle
                })
                .color(if picked { Color::Default } else { Color::Muted })
                .on_click(move |_, _, cx| {
                    if !picked {
                        set_use_mode(key, mode, cx);
                    }
                })
            });
            h_flex()
                .justify_between()
                .gap_4()
                .child(Label::new(system_one_use.name).size(LabelSize::Small))
                .child(
                    h_flex()
                        .p_0p5()
                        .gap_0p5()
                        .rounded_md()
                        .border_1()
                        .border_color(colors.border)
                        .bg(colors.editor_background)
                        .children(buttons),
                )
        });
        v_flex()
            .debug_selector(|| "system-one-uses".into())
            .max_w(px(720.))
            .gap_1()
            .child(Headline::new("Each use and its mode").size(HeadlineSize::XSmall))
            .children(rows)
    }

    fn render_buttons(cx: &Context<Self>) -> impl IntoElement {
        let from_environment = |slot: Slot| {
            cx.try_global::<SystemOne>().is_some_and(|layer| {
                layer.key_source(slot) == KeySource::Environment(slot.variable())
            })
        };
        let cloudflare = cx.try_global::<SystemOne>().is_some_and(|layer| {
            let settings = layer.settings();
            settings.enabled && settings.uses_provider(SystemOneProvider::Cloudflare)
        });
        let notes = [Slot::Direct, Slot::Cloudflare]
            .into_iter()
            .filter(|slot| from_environment(*slot))
            .map(|slot| {
                Label::new(format!(
                    "{} is set, and it comes before the keyring.",
                    slot.variable()
                ))
                .size(LabelSize::Small)
                .color(Color::Muted)
            });
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("system-one-calls-run-check", "Run Check").on_click(
                            cx.listener(|_, _, window, cx| {
                                window.dispatch_action(Box::new(SystemOneCheck), cx);
                            }),
                        ),
                    )
                    .child(Button::new("system-one-calls-set-key", "Set Key").on_click(
                        cx.listener(|view, _, window, cx| {
                            view.show_key_field(Slot::Direct, window, cx);
                        }),
                    ))
                    .child(
                        Button::new("system-one-calls-forget-key", "Forget Key").on_click(
                            cx.listener(|_, _, _, cx| Self::forget_key(Slot::Direct, cx)),
                        ),
                    )
                    .when(cloudflare, |this| {
                        this.child(
                            Button::new(
                                "system-one-calls-set-cloudflare-token",
                                "Set Cloudflare Token",
                            )
                            .on_click(cx.listener(
                                |view, _, window, cx| {
                                    view.show_key_field(Slot::Cloudflare, window, cx);
                                },
                            )),
                        )
                        .child(
                            Button::new(
                                "system-one-calls-forget-cloudflare-token",
                                "Forget Cloudflare Token",
                            )
                            .on_click(
                                cx.listener(|_, _, _, cx| Self::forget_key(Slot::Cloudflare, cx)),
                            ),
                        )
                    }),
            )
            .children(notes)
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
            .id(SharedString::from(format!("system-one-call-{}", row.id)))
            .w_full()
            .px_2()
            .py_1()
            .gap_1()
            .rounded_sm()
            .cursor_pointer()
            .when(row.failed, |this| {
                this.bg(cx.theme().status().error_background)
            })
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

impl Render for SystemOneCallsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: Vec<_> = self
            .rows
            .iter()
            .map(|row| self.render_row(row, cx).into_any_element())
            .collect();
        v_flex()
            .id("system-one-calls")
            .key_context("SystemOneCalls")
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
            .child(
                v_flex()
                    .gap_1()
                    .child(Label::new("System One").size(LabelSize::Large))
                    .child(
                        Label::new(
                            "Small typed questions to a model about what Marley already knows. \
                             Off, it asks nothing and reads no key.",
                        )
                        .color(Color::Muted),
                    ),
            )
            .child(self.render_header(cx))
            .child(Self::render_uses(cx))
            .child(Self::render_buttons(cx))
            .when_some(self.key_field.clone(), |this, (slot, field)| {
                this.child(
                    h_flex()
                        .gap_2()
                        .child(
                            Label::new(match slot {
                                Slot::Direct => "Key",
                                Slot::Cloudflare => "Cloudflare token",
                            })
                            .size(LabelSize::Small),
                        )
                        .child(div().w_96().child(field)),
                )
            })
            .when_some(self.key_note.clone(), |this, note| {
                this.child(Label::new(note).size(LabelSize::Small).color(Color::Muted))
            })
            .child(
                h_flex()
                    .debug_selector(|| "system-one-calls-heading".into())
                    .gap_2()
                    .items_baseline()
                    .child(Headline::new("System One calls").size(HeadlineSize::XSmall))
                    .child(
                        Label::new("newest first · what left the machine is shown whole")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .when(rows.is_empty(), |this| {
                this.child(
                    Label::new("No calls today.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
            .children(rows)
            .child(
                div().max_w(px(720.)).child(
                    Label::new(
                        "What leaves the machine: only from the folders you list; folders marked \
                     facts-only send the numbers Marley computes and none of your text. The key \
                     is read from MARLEY_SYSTEM_ONE_KEY or the keyring, never from settings.",
                    )
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
                ),
            )
    }
}

/// A card of the calls tab's header (#847): a muted title over its lines.
fn stat_card(title: &'static str, cx: &App) -> Div {
    let colors = cx.theme().colors();
    v_flex()
        .gap_1()
        .p_3()
        .min_w(px(240.))
        .rounded_md()
        .border_1()
        .border_color(colors.border)
        .bg(colors.panel_background)
        .child(
            Label::new(title)
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
}

impl Focusable for SystemOneCallsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for SystemOneCallsView {}

impl Item for SystemOneCallsView {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("System One")
    }
}
