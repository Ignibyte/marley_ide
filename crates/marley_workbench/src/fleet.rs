//! The fleet (#607, `docs/marley/fleet-contract.md`): the agents the workflow stores report, and
//! the Fleet panel in the right dock that lists them by host.
//!
//! One [`Fleet`] global holds what every provider last answered, for every window. It reads the
//! providers every `poll_s` while a Fleet panel is what an open right dock shows in some window,
//! and not at all while none is, and it changes, so telling its observers, only when a reading
//! differs. Each workspace has its own [`FleetPanel`], which draws from the global and starts the
//! reads when it draws and none run.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{
    Action, AnyElement, App, Context, EventEmitter, FocusHandle, Focusable, Global, Pixels, Render,
    Subscription, Task, Window, px,
};
use marley_sdk::stale::DEFAULT_POLL_S;
use marley_sdk::{AgentSummary, Attention, HostSnapshot, Pseudo, State, is_stale};
use settings::{FleetProviderContent, Settings as _};
use ui::{Chip, Divider, Icon, IconName, IconSize, Label, LabelSize, prelude::*};
use workspace::dock::{DockPosition, Panel, PanelEvent};
use workspace::{MultiWorkspace, Workspace};

use crate::{MarleySettings, ToggleFleet};

/// The panel's place among the right dock's panels; no other panel takes it.
const ACTIVATION_PRIORITY: u32 = 20;

/// What the fleet's providers last answered.
#[derive(Debug, Default)]
pub struct Fleet {
    sources: Vec<Source>,
    /// The pseudo provider, once a reading has started it.
    pseudo: Option<Pseudo>,
    /// Whether the reads run.
    polling: bool,
}

impl Global for Fleet {}

/// One provider, as the fleet last read it.
#[derive(Debug, Clone, PartialEq)]
struct Source {
    /// The provider's name, for its header.
    name: String,
    /// Its agents, in its order.
    agents: Vec<AgentSummary>,
    /// Its hosts.
    hosts: Vec<HostSnapshot>,
    poll_s: Option<u64>,
    stale_after_s: Option<u64>,
    /// When it was read, which the stale rule measures from.
    read_ms: u64,
    /// Why it could not be read, when it could not.
    failure: Option<String>,
}

impl Source {
    /// A provider that could not be read, and why.
    fn failed(name: &str, failure: String, read_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            agents: Vec::new(),
            hosts: Vec::new(),
            poll_s: None,
            stale_after_s: None,
            read_ms,
            failure: Some(failure),
        }
    }
}

/// The time now, in milliseconds since the Unix epoch.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Registers the panel's toggle on every workspace, and gives each its Fleet panel.
pub fn init(cx: &App) {
    cx.observe_new(
        |workspace: &mut Workspace, window: Option<&mut Window>, cx: &mut Context<Workspace>| {
            workspace.register_action(|workspace, _: &ToggleFleet, window, cx| {
                workspace.toggle_panel_focus::<FleetPanel>(window, cx);
            });
            let Some(window) = window else {
                return;
            };
            let panel = cx.new(|cx| FleetPanel::new(window, cx));
            workspace.add_panel(panel, window, cx);
        },
    )
    .detach();
}

/// Starts the reads, unless they run.
fn start_polling(cx: &mut App) {
    let fleet = cx.default_global::<Fleet>();
    if !fleet.polling {
        fleet.polling = true;
        poll_while_shown(cx).detach();
    }
}

/// Whether a Fleet panel is what an open right dock shows, in any window's shown workspace.
/// Zed's docks call `Panel::set_active` also for a panel activated in a closed dock, and a layout
/// switch that moves panels between docks can leave one showing without the call, so the reads
/// ask the docks rather than count those calls.
fn panel_shows(cx: &App) -> bool {
    cx.windows()
        .iter()
        .filter_map(gpui::AnyWindowHandle::downcast::<MultiWorkspace>)
        .filter_map(|window| window.read(cx).ok())
        .any(|multi_workspace| {
            multi_workspace
                .workspace()
                .read(cx)
                .right_dock()
                .read(cx)
                .visible_panel()
                .is_some_and(|panel| panel.to_any().downcast::<FleetPanel>().is_ok())
        })
}

/// The polls' loop: read every provider, keep what changed, and wait the shortest `poll_s`,
/// until no panel shows.
fn poll_while_shown(cx: &App) -> Task<()> {
    cx.spawn(async move |cx| {
        while let Some(wait) = cx.update(read_providers) {
            cx.background_executor().timer(wait).await;
        }
    })
}

/// Reads every provider once: `None` once no panel shows, else how long to wait.
fn read_providers(cx: &mut App) -> Option<Duration> {
    if !panel_shows(cx) {
        cx.default_global::<Fleet>().polling = false;
        return None;
    }
    // Read through `try_global`: `default_global` tells the observers, so it waits for a change.
    let known = cx.try_global::<Fleet>();
    let now = now_ms();
    let providers = MarleySettings::get_global(cx).fleet_providers.clone();
    let mut pseudo = known.and_then(|fleet| fleet.pseudo.clone());
    let sources: Vec<Source> = providers
        .iter()
        .map(|provider| match provider {
            FleetProviderContent::Pseudo => read_pseudo(&mut pseudo, now),
        })
        .collect();
    let wait_s = sources
        .iter()
        .filter_map(|source| source.poll_s)
        .min()
        .unwrap_or(DEFAULT_POLL_S)
        .max(1);
    let started = known.is_some_and(|fleet| fleet.pseudo.is_some());
    if known.is_none_or(|fleet| fleet.sources != sources) || started != pseudo.is_some() {
        let fleet = cx.default_global::<Fleet>();
        fleet.sources = sources;
        fleet.pseudo = pseudo;
    }
    Some(Duration::from_secs(wait_s))
}

/// The pseudo provider's reading at `now`, starting it on the first.
fn read_pseudo(pseudo: &mut Option<Pseudo>, now: u64) -> Source {
    const NAME: &str = "Pseudo provider";
    if pseudo.is_none() {
        match Pseudo::new(now) {
            Ok(started) => *pseudo = Some(started),
            Err(error) => return Source::failed(NAME, error.to_string(), now),
        }
    }
    let Some(started) = pseudo.as_ref() else {
        return Source::failed(NAME, "the pseudo provider did not start".to_string(), now);
    };
    match started.at(now) {
        Ok(reading) => Source {
            name: NAME.to_string(),
            agents: reading.list.agents,
            hosts: reading.hosts,
            poll_s: started.handshake().poll_s,
            stale_after_s: started.handshake().stale_after_s,
            read_ms: now,
            failure: None,
        },
        Err(error) => Source::failed(NAME, error.to_string(), now),
    }
}

/// The Fleet panel: every provider's agents, grouped by host.
pub struct FleetPanel {
    focus_handle: FocusHandle,
    _fleet: Subscription,
}

impl std::fmt::Debug for FleetPanel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("FleetPanel").finish_non_exhaustive()
    }
}

impl FleetPanel {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            _fleet: cx.observe_global::<Fleet>(|_, cx| cx.notify()),
        }
    }

    fn render_source(source: &Source, cx: &App) -> AnyElement {
        let header = h_flex()
            .px_2()
            .py_1()
            .gap_1p5()
            .child(
                Label::new(source.name.clone())
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .children(source.failure.as_ref().map(|failure| {
                Label::new(format!("could not be read: {failure}"))
                    .size(LabelSize::Small)
                    .color(Color::Error)
            }));
        // Hosts in the provider's order, then agents whose host it did not describe.
        let mut host_ids: Vec<Option<&str>> = source
            .hosts
            .iter()
            .map(|host| Some(host.host.id.as_str()))
            .collect();
        for agent in &source.agents {
            let id = agent.host_id.as_deref();
            if !host_ids.contains(&id) {
                host_ids.push(id);
            }
        }
        let groups = host_ids.into_iter().filter_map(|host_id| {
            let agents: Vec<&AgentSummary> = source
                .agents
                .iter()
                .filter(|agent| agent.host_id.as_deref() == host_id)
                .collect();
            if agents.is_empty() {
                return None;
            }
            let name = host_id.map_or_else(
                || "No host".to_string(),
                |id| {
                    source
                        .hosts
                        .iter()
                        .find(|host| host.host.id == id)
                        .map_or_else(|| id.to_string(), |host| host.host.name.clone())
                },
            );
            Some(
                v_flex()
                    .gap_0p5()
                    .child(
                        h_flex()
                            .px_2()
                            .pt_2()
                            .gap_1()
                            .child(
                                Icon::new(IconName::Server)
                                    .size(IconSize::Small)
                                    .color(Color::Muted),
                            )
                            .child(Label::new(name).size(LabelSize::Small)),
                    )
                    .children(
                        agents
                            .into_iter()
                            .map(|agent| Self::render_agent(agent, source, cx)),
                    ),
            )
        });
        v_flex().child(header).children(groups).into_any_element()
    }

    fn render_agent(agent: &AgentSummary, source: &Source, cx: &App) -> AnyElement {
        let stale = is_stale(
            agent.last_seen_ms,
            source.read_ms,
            source.poll_s,
            source.stale_after_s,
        );
        let (word, color) = state_chip(agent.state, stale);
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
        let mark = match agent.attention {
            Attention::Question => Some(
                Icon::new(IconName::Warning)
                    .size(IconSize::Small)
                    .color(Color::Warning),
            ),
            Attention::Failed => Some(
                Icon::new(IconName::XCircle)
                    .size(IconSize::Small)
                    .color(Color::Error),
            ),
            Attention::None | Attention::Unknown => None,
        };
        h_flex()
            .mx_2()
            .px_2()
            .py_1()
            .gap_2()
            .rounded_md()
            .hover(|row| row.bg(cx.theme().colors().ghost_element_hover))
            .child(
                Icon::new(runtime_icon(&agent.runtime))
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(
                v_flex()
                    .min_w_0()
                    .flex_1()
                    .child(
                        Label::new(agent.name.clone())
                            .size(LabelSize::Small)
                            .truncate(),
                    )
                    .when(!line.is_empty(), |column| {
                        column.child(
                            Label::new(line.join(" · "))
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .truncate(),
                        )
                    }),
            )
            .children(mark)
            .child(
                Chip::new(word)
                    .label_color(color)
                    .label_size(LabelSize::XSmall),
            )
            .into_any_element()
    }
}

/// The word and colour of an agent's state chip; a stale agent reads stale, whatever it said.
const fn state_chip(state: State, quiet: bool) -> (&'static str, Color) {
    if quiet {
        return ("stale", Color::Muted);
    }
    match state {
        State::Starting => ("starting", Color::Muted),
        State::Working => ("working", Color::Accent),
        State::Idle => ("idle", Color::Muted),
        State::Waiting => ("waiting", Color::Warning),
        State::Error => ("error", Color::Error),
        State::Done => ("done", Color::Success),
        State::Unknown => ("unknown", Color::Muted),
    }
}

/// The icon of a runtime the fleet names: an agent's own where Marley knows it.
fn runtime_icon(runtime: &str) -> IconName {
    match runtime {
        "claude-code" | "claude" => IconName::AiClaude,
        "codex" => IconName::AiOpenAi,
        "gemini" => IconName::AiGemini,
        _ => IconName::Terminal,
    }
}

impl Render for FleetPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let fleet = cx.try_global::<Fleet>();
        let sources = fleet.map(|fleet| fleet.sources.clone()).unwrap_or_default();
        // A panel draws only while it shows: it starts the reads when none run.
        if fleet.is_none_or(|fleet| !fleet.polling) {
            cx.defer(start_polling);
        }
        let set_up = !MarleySettings::get_global(cx).fleet_providers.is_empty();
        v_flex()
            .id("marley-fleet-panel")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .bg(cx.theme().colors().panel_background)
            .child(
                h_flex().px_2().py_1p5().child(
                    Label::new("FLEET")
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
            )
            .child(Divider::horizontal())
            .when(!set_up, |panel| {
                panel.child(
                    v_flex()
                        .p_2()
                        .gap_1()
                        .child(Label::new("The fleet is not set up.").size(LabelSize::Small))
                        .child(
                            Label::new(
                                "Add a workflow store under marley.fleet.providers in your \
                                 settings, or { \"kind\": \"pseudo\" } for Marley's own example \
                                 data.",
                            )
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                        ),
                )
            })
            .children(sources.iter().map(|source| Self::render_source(source, cx)))
    }
}

impl Focusable for FleetPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for FleetPanel {}

impl Panel for FleetPanel {
    fn persistent_name() -> &'static str {
        "MarleyFleetPanel"
    }

    fn panel_key() -> &'static str {
        "MarleyFleetPanel"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        DockPosition::Right
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Right)
    }

    fn set_position(
        &mut self,
        _position: DockPosition,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        px(320.)
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        Some(IconName::Server)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Fleet")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleFleet)
    }

    fn activation_priority(&self) -> u32 {
        ACTIVATION_PRIORITY
    }
}
