//! Forge around Marley (#843): the rail's FORGE section, Home's FORGE card and the bottom row's
//! Forge item.
//!
//! While a `forge` provider is set, `ForgeWatch` reads Forge every 30 seconds (its running agents,
//! the current sprint and its tickets, its bulletins) into one snapshot the three surfaces draw.
//! It writes only when the snapshot changed, keeps the last one through a failed read, and stops
//! when the provider goes. Each surface opens the sprint board (#842).

use std::collections::HashSet;
use std::time::Duration;

use gpui::{
    AnyElement, App, AsyncApp, Context, FontWeight, Global, IntoElement, Render, SharedString,
    Task, Window,
};
use marley_sdk::{AgentSummary, State};
use serde::Deserialize;
use serde_json::json;
use settings::{FleetProviderContent, SettingsStore};
use ui::{Divider, prelude::*};
use workspace::{ItemHandle, StatusItemView};

use crate::fleet_providers::{self, Client};
use crate::forge_board::{self, SprintAnswer, TICKET_LIMIT, Tickets};
use crate::{OpenForgeBoard, forge};

/// How often Forge is read.
const EVERY: Duration = Duration::from_secs(30);

/// One of Forge's bulletins, the fields Marley reads.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Bulletin {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    severity: String,
}

#[derive(Deserialize)]
struct Bulletins {
    #[serde(default)]
    bulletins: Vec<Bulletin>,
}

/// What the surfaces draw.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ForgeSnapshot {
    agents: Vec<AgentSummary>,
    /// "Sprint 14", and its days left.
    sprint: Option<(String, Option<String>)>,
    done: usize,
    total: usize,
    /// The sprint's tickets that failed QA, newest first: number and title.
    qa_failed: Vec<(i64, String)>,
    bulletins: Vec<Bulletin>,
    /// Why the last read failed; the snapshot is the one before it.
    unreachable: Option<String>,
}

/// The snapshot, the bulletins dismissed this session, and the loop that reads Forge.
#[derive(Default)]
pub(crate) struct ForgeWatch {
    snapshot: Option<ForgeSnapshot>,
    dismissed: HashSet<String>,
    entry: Option<FleetProviderContent>,
    keeper: Option<Task<()>>,
}

impl Global for ForgeWatch {}

/// Sets the global and follows the settings. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(ForgeWatch::default());
    follow_setting(cx);
    cx.observe_global::<SettingsStore>(follow_setting).detach();
}

/// Starts reading when a `forge` provider is set or changes, and stops when it goes.
fn follow_setting(cx: &mut App) {
    let entry = forge_board::forge_entry(cx);
    if cx.global::<ForgeWatch>().entry == entry {
        return;
    }
    let keeper = entry
        .clone()
        .map(|entry| cx.spawn(async move |cx| keep(entry, cx).await));
    cx.update_global::<ForgeWatch, _>(|watch, _| {
        watch.entry = entry;
        watch.keeper = keeper;
        watch.snapshot = None;
    });
}

async fn keep(entry: FleetProviderContent, cx: &AsyncApp) {
    let mut client: Option<Client> = None;
    loop {
        let read = async {
            let reached = match client.clone() {
                Some(reached) => reached,
                None => fleet_providers::connect(&entry, "forge-watch", cx).await?,
            };
            let snapshot = read(&reached, cx).await?;
            anyhow::Ok((reached, snapshot))
        };
        match read.await {
            Ok((reached, snapshot)) => {
                client = Some(reached);
                cx.update(|cx| store(snapshot, cx));
            }
            Err(error) => {
                client = None;
                let reason = format!("{error:#}");
                cx.update(|cx| {
                    let last = cx.global::<ForgeWatch>().snapshot.clone();
                    if let Some(mut snapshot) = last {
                        snapshot.unreachable = Some(reason);
                        store(snapshot, cx);
                    }
                });
            }
        }
        cx.background_executor().timer(EVERY).await;
    }
}

/// Keeps `snapshot` when it differs from the one held, which tells every surface.
fn store(snapshot: ForgeSnapshot, cx: &mut App) {
    if cx.global::<ForgeWatch>().snapshot.as_ref() != Some(&snapshot) {
        cx.update_global::<ForgeWatch, _>(|watch, _| watch.snapshot = Some(snapshot));
    }
}

async fn read(client: &Client, cx: &AsyncApp) -> anyhow::Result<ForgeSnapshot> {
    let agents: forge::ForgeAgents =
        fleet_providers::call_tool(client, "agent-list", json!({}), cx).await?;
    let sprint: SprintAnswer =
        fleet_providers::call_tool(client, "sprint-current", json!({}), cx).await?;
    let mut arguments = json!({ "status": "all", "limit": TICKET_LIMIT });
    if let Some(sprint) = &sprint.sprint {
        arguments["sprint_id"] = json!(sprint.id);
    }
    let tickets: Tickets = fleet_providers::call_tool(client, "ticket-list", arguments, cx).await?;
    let bulletins: Bulletins =
        fleet_providers::call_tool(client, "bulletin-list", json!({}), cx).await?;
    let done = tickets
        .tickets
        .iter()
        .filter(|ticket| matches!(ticket.status.as_str(), "done" | "closed"))
        .count();
    let mut failed: Vec<_> = tickets
        .tickets
        .iter()
        .filter(|ticket| ticket.status == "qa-failure")
        .collect();
    failed.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(ForgeSnapshot {
        agents: forge::summaries(agents),
        sprint: sprint.sprint.as_ref().map(|sprint| {
            (
                format!("Sprint {}", sprint.number),
                forge_board::days_left(sprint),
            )
        }),
        done,
        total: tickets.tickets.len(),
        qa_failed: failed
            .into_iter()
            .map(|ticket| (ticket.number, ticket.title.clone()))
            .collect(),
        bulletins: bulletins.bulletins,
        unreachable: None,
    })
}

fn snapshot(cx: &App) -> Option<&ForgeSnapshot> {
    cx.try_global::<ForgeWatch>()?.snapshot.as_ref()
}

/// Hides a bulletin until Marley restarts (D3).
fn dismiss(id: String, cx: &mut App) {
    cx.update_global::<ForgeWatch, _>(|watch, _| {
        watch.dismissed.insert(id);
    });
}

/// The two letters on an agent's tile.
fn glyph(runtime: &str) -> &'static str {
    match runtime {
        "codex" => "CX",
        "gemini" => "GE",
        _ => "CC",
    }
}

/// An agent's state as the rail's line says it, and its colour.
fn state_of(agent: &AgentSummary) -> (String, Color) {
    match agent.state {
        State::Working => ("working".to_string(), Color::Warning),
        State::Waiting => {
            let silent = agent.state_since_ms.map_or_else(String::new, |since| {
                let now = u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or_default();
                format!(" {} m", now.saturating_sub(since) / 60_000)
            });
            (format!("silent{silent}"), Color::Error)
        }
        _ => ("idle".to_string(), Color::Muted),
    }
}

fn open_board(window: &mut Window, cx: &mut App) {
    window.dispatch_action(Box::new(OpenForgeBoard), cx);
}

/// One of the rail's Forge rows: a tile, the agent's name, and "host · ticket · state".
fn rail_row(index: usize, agent: &AgentSummary, cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();
    let (state, color) = state_of(agent);
    let mut line = Vec::new();
    line.extend(agent.host_id.clone());
    line.extend(
        agent
            .work_item
            .as_ref()
            .map(|item| item.id.clone())
            .filter(|id| !id.is_empty()),
    );
    line.push(state);
    h_flex()
        .id(("marley-rail-forge-agent", index))
        .w_full()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .hover(|style| style.bg(colors.ghost_element_hover))
        .on_click(|_, window, cx| open_board(window, cx))
        .child(
            div()
                .size_6()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .bg(colors.element_background)
                .child(
                    Label::new(glyph(&agent.runtime))
                        .size(LabelSize::XSmall)
                        .color(Color::Accent),
                ),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(
                    Label::new(agent.name.clone())
                        .size(LabelSize::Small)
                        .truncate(),
                )
                .child(
                    Label::new(line.join(" · "))
                        .size(LabelSize::XSmall)
                        .color(color)
                        .truncate(),
                ),
        )
}

/// The rail's FORGE section, between Needs you and the projects; none without Forge.
pub(crate) fn render_rail_section(cx: &App) -> Option<AnyElement> {
    let snapshot = snapshot(cx)?;
    let rows = snapshot
        .agents
        .iter()
        .enumerate()
        .map(|(index, agent)| rail_row(index, agent, cx));
    Some(
        v_flex()
            .id("marley-rail-forge")
            .debug_selector(|| "marley-rail-forge".into())
            .px_2()
            .pt_1p5()
            .gap_0p5()
            .child(
                h_flex()
                    .px_2()
                    .gap_1()
                    .child(
                        Label::new("FORGE")
                            .size(LabelSize::XSmall)
                            .color(Color::Accent),
                    )
                    .child(
                        Label::new(format!("{} online", snapshot.agents.len()))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .when(snapshot.unreachable.is_some(), |heading| {
                        heading.child(
                            Label::new("unreachable")
                                .size(LabelSize::XSmall)
                                .color(Color::Error),
                        )
                    }),
            )
            .children(rows)
            .child(
                h_flex()
                    .id("marley-rail-forge-board")
                    .px_2()
                    .py_0p5()
                    .cursor_pointer()
                    .on_click(|_, window, cx| open_board(window, cx))
                    .child(
                        Label::new("Open the sprint board")
                            .size(LabelSize::XSmall)
                            .color(Color::Accent),
                    ),
            )
            .child(div().pt_1p5().child(Divider::horizontal()))
            .into_any_element(),
    )
}

/// Home's FORGE card's body; none without Forge.
pub(crate) fn render_home_card(cx: &App) -> Option<AnyElement> {
    let snapshot = snapshot(cx)?;
    let dismissed = &cx.global::<ForgeWatch>().dismissed;
    let working = snapshot
        .agents
        .iter()
        .filter(|agent| agent.state == State::Working)
        .count();
    let waiting = snapshot
        .agents
        .iter()
        .filter(|agent| agent.state == State::Waiting)
        .count();
    let mut line = vec![format!("{} of {} done", snapshot.done, snapshot.total)];
    if let Some((sprint, left)) = &snapshot.sprint {
        line.insert(0, sprint.clone());
        line.extend(left.clone());
    }
    let chip = |text: String, color: Color| Label::new(text).size(LabelSize::Small).color(color);
    let bulletins = snapshot
        .bulletins
        .iter()
        .filter(|bulletin| !dismissed.contains(&bulletin.id))
        .map(|bulletin| {
            let id = bulletin.id.clone();
            let color = if bulletin.severity == "critical" {
                Color::Error
            } else {
                Color::Warning
            };
            h_flex()
                .gap_2()
                .items_start()
                .child(
                    Label::new(bulletin.severity.clone())
                        .size(LabelSize::XSmall)
                        .color(color),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(Label::new(bulletin.title.clone()).size(LabelSize::Small))
                        .when(!bulletin.body.is_empty(), |text| {
                            text.child(
                                Label::new(bulletin.body.clone())
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            )
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("forge-dismiss-{id}")), "Dismiss")
                        .style(ButtonStyle::Subtle)
                        .label_size(LabelSize::XSmall)
                        .on_click(move |_, _, cx| dismiss(id.clone(), cx)),
                )
        });
    Some(
        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(Label::new(line.join(" · ")).size(LabelSize::Small))
                    .child(div().flex_1())
                    .child(
                        Button::new("marley-home-forge-board", "Board")
                            .style(ButtonStyle::Subtle)
                            .label_size(LabelSize::Small)
                            .on_click(|_, window, cx| open_board(window, cx)),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .child(chip(format!("{working} working"), Color::Warning))
                    .child(chip(format!("{waiting} needs you"), Color::Error))
                    .child(chip(
                        format!("{} QA failed", snapshot.qa_failed.len()),
                        Color::Error,
                    )),
            )
            .children(bulletins)
            .into_any_element(),
    )
}

/// The bottom row's Forge item: online agents and the newest QA failure; nothing without Forge.
pub(crate) struct ForgeItem {
    _watch: gpui::Subscription,
}

impl ForgeItem {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            _watch: cx.observe_global::<ForgeWatch>(|_, cx| cx.notify()),
        }
    }
}

impl Render for ForgeItem {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(snapshot) = snapshot(cx) else {
            return div().into_any_element();
        };
        h_flex()
            .id("marley-forge-item")
            .debug_selector(|| "marley-forge-item".into())
            .flex_none()
            .gap_1()
            .px_1p5()
            .rounded_sm()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().colors().ghost_element_hover))
            .on_click(|_, window, cx| open_board(window, cx))
            .child(
                Label::new("Forge")
                    .size(LabelSize::Small)
                    .weight(FontWeight::SEMIBOLD),
            )
            .child(
                Label::new(format!("· {} online", snapshot.agents.len()))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .children(snapshot.qa_failed.first().map(|(number, _)| {
                Label::new(format!("· #{number} QA failed"))
                    .size(LabelSize::Small)
                    .color(Color::Error)
            }))
            .into_any_element()
    }
}

impl StatusItemView for ForgeItem {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    // The item shows only while a `forge` provider is set, the user's own choice.
    fn hide_setting(&self, _cx: &App) -> Option<workspace::HideStatusItem> {
        None
    }
}
