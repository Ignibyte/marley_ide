//! The Agent tab (#609): one agent of the fleet in full, in the center.
//!
//! Its run's phases and gates in time, its events, its host's resources over the samples the
//! fleet kept, its tokens, and the other agents on its host. It is read-only and not kept across
//! a restart.
//!
//! The tab names its agent in the fleet's `Wanted`, so the reads keep its detail, and the reads
//! run while it is its pane's active item.

use gpui::{
    AnyElement, App, ClickEvent, Context, Div, EventEmitter, FocusHandle, Focusable, Hsla,
    PathBuilder, Render, SharedString, Subscription, WeakEntity, Window, canvas, point, px,
    relative,
};
use marley_sdk::{
    AgentDetail, AgentSummary, Event, Gate, GateState, HostSnapshot, PhaseRun, PhaseState, Run,
    TokenUse,
};
use ui::{Chip, Icon, IconName, IconSize, Label, LabelSize, prelude::*};
use workspace::Workspace;
use workspace::item::Item;

use crate::fleet::{self, Fleet, HostKey, Sample, Selected, Source, Wanted};

/// How many events the log shows, newest first.
const EVENTS_SHOWN: usize = 50;

/// Opens `selected`'s tab in the window's Home group once the current update is over, or shows the
/// one open there (#676); `workspace` is where it opens in the Zed layout. The opener reads every
/// Agent tab of the workspace, so it must not run inside one's update.
pub(crate) fn open_later(
    workspace: WeakEntity<Workspace>,
    selected: Selected,
    window: &Window,
    cx: &mut App,
) {
    crate::groups::in_group(
        crate::groups::GroupKind::Home,
        workspace,
        window,
        cx,
        move |workspace, window, cx| open(workspace, selected, window, cx),
    );
}

/// Opens `selected`'s tab, or brings forward the one open in the workspace.
fn open(
    workspace: &mut Workspace,
    selected: Selected,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open = workspace
        .items_of_type::<AgentView>(cx)
        .find(|view| view.read(cx).selected == selected);
    if let Some(open) = open {
        workspace.activate_item(&open, true, true, window, cx);
        return;
    }
    let weak_workspace = cx.weak_entity();
    let view = cx.new(|cx| AgentView::new(selected, weak_workspace, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}

/// One agent's tab.
pub struct AgentView {
    selected: Selected,
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    _fleet: Subscription,
}

impl std::fmt::Debug for AgentView {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AgentView")
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl AgentView {
    fn new(selected: Selected, workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        let id = cx.entity_id();
        cx.default_global::<Wanted>().0.insert(id, selected.clone());
        cx.on_release(move |_, cx| {
            if cx.has_global::<Wanted>() {
                cx.global_mut::<Wanted>().0.remove(&id);
            }
        })
        .detach();
        // The detail comes with the next read; this one fills the tab without waiting a poll.
        cx.defer(fleet::read_now);
        Self {
            selected,
            workspace,
            focus_handle: cx.focus_handle(),
            _fleet: cx.observe_global::<Fleet>(|_, cx| cx.notify()),
        }
    }

    /// The agent in full, part by part.
    fn render_agent(
        detail: &AgentDetail,
        source: &Source,
        samples: &[Sample],
        cx: &Context<Self>,
    ) -> AnyElement {
        let work_item = detail.work_item.as_ref().map(|item| {
            let mut line = vec![item.key.clone(), item.title.clone()];
            line.extend(item.status.clone());
            Label::new(line.join(" · ")).size(LabelSize::Small)
        });
        let phases = detail
            .run
            .as_ref()
            .filter(|_| source.offers("runs"))
            .map(|run| fleet::section("PHASES", render_timeline(run, source.read_ms, cx)));
        let events = source
            .offers("events")
            .then(|| fleet::section("EVENTS", render_events(detail, source.read_ms)));
        let resources = source.offers("hosts").then(|| {
            fleet::section(
                "RESOURCES",
                render_history(source.host(detail), samples, cx),
            )
        });
        let tokens = detail
            .usage
            .as_ref()
            .filter(|_| source.offers("usage"))
            .map(|usage| {
                fleet::section(
                    "TOKENS",
                    v_flex()
                        .gap_0p5()
                        .children(usage.run.as_ref().map(|run| token_line("This run", run)))
                        .children(usage.today.as_ref().map(|today| token_line("Today", today))),
                )
            });
        v_flex()
            .max_w(px(960.))
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(fleet::render_snapshot_header(detail, source, None))
                    .children(work_item),
            )
            .children(phases)
            .children(events)
            .children(resources)
            .children(tokens)
            .child(fleet::section(
                "ON THIS HOST",
                Self::render_neighbours(detail, source, cx),
            ))
            .into_any_element()
    }

    /// The other agents the list puts on this agent's host, each opening its own tab.
    fn render_neighbours(detail: &AgentDetail, source: &Source, cx: &Context<Self>) -> AnyElement {
        let host_id = detail.agent.host_id.as_deref();
        let others: Vec<&AgentSummary> = source
            .agents
            .iter()
            .filter(|agent| {
                host_id.is_some()
                    && agent.host_id.as_deref() == host_id
                    && agent.id != detail.agent.id
            })
            .collect();
        if others.is_empty() {
            return Label::new("No other agents on this host.")
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .into_any_element();
        }
        let hover = cx.theme().colors().ghost_element_hover;
        v_flex()
            .gap_0p5()
            .children(others.into_iter().enumerate().map(|(index, agent)| {
                let (word, color) = fleet::agent_chip(
                    agent.state,
                    agent.last_seen_ms,
                    agent.host_id.as_deref(),
                    source,
                );
                let mut line = Vec::new();
                if let Some(item) = &agent.work_item {
                    line.push(item.key.clone());
                }
                if let Some(phase) = &agent.phase {
                    line.push(format!(
                        "{} {}/{}",
                        phase.name,
                        phase.index + 1,
                        phase.count
                    ));
                }
                let selected = Selected {
                    source: source.name.clone(),
                    agent: agent.id.clone(),
                };
                h_flex()
                    .id(("marley-agent-tab-neighbour", index))
                    .max_w(px(480.))
                    .px_2()
                    .py_1()
                    .gap_2()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|row| row.bg(hover))
                    .on_click(cx.listener(move |view, _: &ClickEvent, window, cx| {
                        open_later(view.workspace.clone(), selected.clone(), window, cx);
                    }))
                    .child(
                        Icon::new(fleet::runtime_icon(&agent.runtime))
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(Label::new(agent.name.clone()).size(LabelSize::Small))
                    .child(
                        Label::new(line.join(" · "))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(div().flex_1())
                    .children(fleet::attention_mark(agent.attention))
                    .child(
                        Chip::new(word)
                            .label_color(color)
                            .label_size(LabelSize::XSmall),
                    )
            }))
            .into_any_element()
    }
}

/// `at`'s place between `start` and `start + span`, from 0 to 1, in thousandths.
fn share(at: u64, start: u64, span: u64) -> f32 {
    let per_mille = at
        .saturating_sub(start)
        .saturating_mul(1000)
        .checked_div(span)
        .unwrap_or(0)
        .min(1000);
    f32::from(u16::try_from(per_mille).unwrap_or(1000)) / 1000.
}

/// The run's phases in time: each a bar on a track from the run's start to its end, or to now,
/// with its gates under it.
fn render_timeline(run: &Run, now_ms: u64, cx: &App) -> Div {
    let start = run
        .started_ms
        .or_else(|| run.phases.iter().filter_map(|phase| phase.started_ms).min())
        .unwrap_or(now_ms);
    let end = run.ended_ms.unwrap_or(now_ms).max(start.saturating_add(1));
    let span = end - start;
    let track = cx.theme().colors().element_background;
    v_flex()
        .gap_2()
        .children(run.phases.iter().enumerate().map(|(index, phase)| {
            // A phase with a start and no end runs to now while active, else to where the next
            // phase that started began: a store may mark a phase passed without its end.
            let reach = phase.started_ms.map(|from| {
                let to = phase.ended_ms.unwrap_or_else(|| {
                    if phase.state == PhaseState::Active {
                        now_ms
                    } else {
                        run.phases
                            .iter()
                            .skip(index + 1)
                            .find_map(|next| next.started_ms)
                            .unwrap_or(now_ms)
                            .max(from)
                    }
                });
                (from, to)
            });
            let bar = reach.map(|(from, to)| {
                let left = share(from, start, span);
                let width = (share(to, start, span) - left).max(0.01);
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(relative(left))
                    .w(relative(width))
                    .rounded_sm()
                    .bg(fleet::phase_color(phase.state, cx))
            });
            let length = reach.map_or_else(
                || phase_word(phase.state).to_string(),
                |(from, to)| fleet::how_long(from, to),
            );
            v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .w(px(96.))
                                .flex_none()
                                .child(Label::new(phase.name.clone()).size(LabelSize::Small)),
                        )
                        .child(
                            div()
                                .relative()
                                .flex_1()
                                .h(px(10.))
                                .rounded_sm()
                                .bg(track)
                                .children(bar),
                        )
                        .child(
                            div().w(px(72.)).flex_none().child(
                                Label::new(length)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                        ),
                )
                .children(render_gates(phase))
        }))
}

/// A phase with no start: what the store says of it.
const fn phase_word(state: PhaseState) -> &'static str {
    match state {
        PhaseState::Pending => "pending",
        PhaseState::Active => "active",
        PhaseState::Passed => "passed",
        PhaseState::Failed => "failed",
        PhaseState::Skipped => "skipped",
        PhaseState::Unknown => "unknown",
    }
}

/// A phase's gates under its bar, and a failed gate's detail.
fn render_gates(phase: &PhaseRun) -> Option<Div> {
    if phase.gates.is_empty() {
        return None;
    }
    Some(
        v_flex()
            .pl(px(104.))
            .gap_0p5()
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_3()
                    .children(phase.gates.iter().map(render_gate)),
            )
            .children(
                phase
                    .gates
                    .iter()
                    .filter(|gate| gate.state == GateState::Fail)
                    .filter_map(|gate| {
                        let detail = gate.detail.as_ref()?;
                        Some(
                            Label::new(format!("{}: {detail}", gate.name))
                                .size(LabelSize::XSmall)
                                .color(Color::Error),
                        )
                    }),
            ),
    )
}

/// One gate: a mark for its state, its name and the state's word.
fn render_gate(gate: &Gate) -> Div {
    let (icon, color, word) = match gate.state {
        GateState::Pass => (IconName::Check, Color::Success, "pass"),
        GateState::Fail => (IconName::XCircle, Color::Error, "fail"),
        GateState::Pending => (IconName::Circle, Color::Muted, "pending"),
        GateState::Skipped => (IconName::Dash, Color::Muted, "skipped"),
        GateState::Unknown => (IconName::CircleHelp, Color::Muted, "unknown"),
    };
    h_flex()
        .gap_1()
        .child(Icon::new(icon).size(IconSize::XSmall).color(color))
        .child(Label::new(gate.name.clone()).size(LabelSize::XSmall))
        .child(Label::new(word).size(LabelSize::XSmall).color(Color::Muted))
}

/// The run's events and the agent's recent ones, together, newest first.
fn render_events(detail: &AgentDetail, now_ms: u64) -> AnyElement {
    let mut events: Vec<&Event> = detail
        .run
        .iter()
        .flat_map(|run| &run.events)
        .chain(&detail.recent_events)
        .collect();
    events.sort_by(|first, second| {
        second
            .at_ms
            .cmp(&first.at_ms)
            .then_with(|| first.text.cmp(&second.text))
    });
    events.dedup_by(|first, second| first.at_ms == second.at_ms && first.text == second.text);
    if events.is_empty() {
        return Label::new("No events yet.")
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .into_any_element();
    }
    v_flex()
        .gap_0p5()
        .children(events.into_iter().take(EVENTS_SHOWN).map(|event| {
            h_flex()
                .gap_2()
                .child(
                    div().w(px(72.)).flex_none().child(
                        Label::new(format!("{} ago", fleet::how_long(event.at_ms, now_ms)))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    ),
                )
                .child(
                    div().w(px(64.)).flex_none().child(
                        Label::new(event.kind.clone())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    ),
                )
                .child(
                    Label::new(event.text.clone())
                        .size(LabelSize::Small)
                        .truncate(),
                )
        }))
        .into_any_element()
}

/// The host's processor, memory and network over the samples kept, each a line with its value now.
fn render_history(host: Option<&HostSnapshot>, samples: &[Sample], cx: &App) -> AnyElement {
    let Some(host) = host else {
        return Label::new("no resources yet")
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .into_any_element();
    };
    let colors = cx.theme().colors();
    let status = cx.theme().status();
    let cpu: Vec<f32> = samples.iter().filter_map(|sample| sample.cpu).collect();
    let memory: Vec<f32> = samples.iter().filter_map(|sample| sample.memory).collect();
    let busiest = samples
        .iter()
        .filter_map(|sample| sample.network)
        .max()
        .unwrap_or(0)
        .max(1);
    let network: Vec<f32> = samples
        .iter()
        .filter_map(|sample| sample.network)
        .map(|rate| share(rate, 0, busiest) * 100.)
        .collect();
    let kept = match (samples.first(), samples.last()) {
        (Some(first), Some(last)) => format!(
            "{} samples over {}",
            samples.len(),
            fleet::how_long(first.at_ms, last.at_ms)
        ),
        _ => "no samples yet".to_string(),
    };
    v_flex()
        .gap_2()
        .child(history_row(
            "CPU",
            cpu,
            host.cpu
                .as_ref()
                .map_or_else(String::new, |cpu| format!("{:.0} %", cpu.percent)),
            colors.text_accent,
            cx,
        ))
        .child(history_row(
            "Memory",
            memory,
            host.memory.as_ref().map_or_else(String::new, |memory| {
                format!(
                    "{} / {} GB",
                    fleet::gigabytes(memory.used_bytes),
                    fleet::gigabytes(memory.total_bytes)
                )
            }),
            status.info,
            cx,
        ))
        .child(history_row(
            "Network",
            network,
            host.network.as_ref().map_or_else(String::new, |network| {
                format!(
                    "in {} · out {}",
                    fleet::rate(network.rx_bps),
                    fleet::rate(network.tx_bps)
                )
            }),
            status.success,
            cx,
        ))
        .child(Label::new(kept).size(LabelSize::XSmall).color(Color::Muted))
        .into_any_element()
}

/// One resource over time: its name, its line and its value now.
fn history_row(name: &'static str, values: Vec<f32>, now: String, color: Hsla, cx: &App) -> Div {
    h_flex()
        .gap_2()
        .child(
            div()
                .w(px(72.))
                .flex_none()
                .child(Label::new(name).size(LabelSize::XSmall).color(Color::Muted)),
        )
        .child(
            div()
                .flex_1()
                .h(px(40.))
                .rounded_sm()
                .bg(cx.theme().colors().element_background)
                .child(sparkline(values, color)),
        )
        .child(
            div()
                .w(px(160.))
                .flex_none()
                .child(Label::new(now).size(LabelSize::XSmall)),
        )
}

/// A line through `values`, each from 0 to 100, spread evenly across the element.
fn sparkline(values: Vec<f32>, color: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, (), window, _| {
            let Some(last) = values.len().checked_sub(1).filter(|last| *last > 0) else {
                return;
            };
            let steps = f32::from(u16::try_from(last).unwrap_or(u16::MAX));
            let mut builder = PathBuilder::stroke(px(1.5));
            for (index, value) in values.iter().enumerate() {
                let along = f32::from(u16::try_from(index).unwrap_or(u16::MAX)) / steps;
                let x = bounds.origin.x + bounds.size.width * along;
                let y = bounds.bottom() - bounds.size.height * (value / 100.).clamp(0., 1.);
                if index == 0 {
                    builder.move_to(point(x, y));
                } else {
                    builder.line_to(point(x, y));
                }
            }
            if let Ok(path) = builder.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size_full()
}

/// A line of token use: input, output and cache reads.
fn token_line(label: &'static str, tokens: &TokenUse) -> Div {
    let mut parts = vec![
        format!("in {}", fleet::compact(tokens.input)),
        format!("out {}", fleet::compact(tokens.output)),
    ];
    if let Some(cache) = tokens.cache_read {
        parts.push(format!("cache {}", fleet::compact(cache)));
    }
    h_flex()
        .gap_2()
        .child(
            div().w(px(72.)).flex_none().child(
                Label::new(label)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            ),
        )
        .child(Label::new(parts.join(" · ")).size(LabelSize::Small))
}

impl Render for AgentView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // A tab draws only while it shows: it starts the reads when none run.
        if cx.try_global::<Fleet>().is_none_or(|fleet| !fleet.polling) {
            cx.defer(fleet::start_polling);
        }
        let fleet = cx.try_global::<Fleet>();
        let source = fleet
            .and_then(|fleet| {
                fleet
                    .sources
                    .iter()
                    .find(|source| source.name == self.selected.source)
            })
            .cloned();
        let listed = source.as_ref().is_some_and(|source| {
            source
                .agents
                .iter()
                .any(|agent| agent.id == self.selected.agent)
        });
        let detail = source.as_ref().and_then(|source| {
            source
                .details
                .iter()
                .find(|detail| detail.agent.id == self.selected.agent)
        });
        let samples: Vec<Sample> = match (fleet, &source, detail) {
            (Some(fleet), Some(source), Some(detail)) => source
                .host(detail)
                .map(|host| HostKey {
                    source: source.name.clone(),
                    host: host.host.id.clone(),
                })
                .and_then(|key| fleet.samples.get(&key))
                .map(|kept| kept.iter().cloned().collect())
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let body = match (&source, detail) {
            (Some(source), Some(detail)) if listed => {
                Self::render_agent(detail, source, &samples, cx)
            }
            _ if !listed => Label::new(format!(
                "{} is no longer listed by {}.",
                self.selected.agent, self.selected.source
            ))
            .size(LabelSize::Small)
            .color(Color::Muted)
            .into_any_element(),
            _ => Label::new("Reading…")
                .size(LabelSize::Small)
                .color(Color::Muted)
                .into_any_element(),
        };
        v_flex()
            .id("marley-agent-tab")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .bg(cx.theme().colors().editor_background)
            .child(body)
    }
}

impl Focusable for AgentView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for AgentView {}

impl Item for AgentView {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        let name = cx
            .try_global::<Fleet>()
            .and_then(|fleet| {
                fleet
                    .sources
                    .iter()
                    .find(|source| source.name == self.selected.source)
            })
            .and_then(|source| {
                source
                    .agents
                    .iter()
                    .find(|agent| agent.id == self.selected.agent)
            })
            .map_or_else(|| self.selected.agent.clone(), |agent| agent.name.clone());
        SharedString::from(name)
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Server))
    }
}
