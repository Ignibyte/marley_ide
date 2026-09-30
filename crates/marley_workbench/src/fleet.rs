//! The fleet (#607, `docs/marley/fleet-contract.md`): the agents the workflow stores report, and
//! the Fleet panel in the right dock that lists them by host.
//!
//! One [`Fleet`] global holds what every provider last answered, for every window. It reads the
//! providers every `poll_s` while a Fleet panel is what an open right dock shows in some window,
//! and not at all while none is, and it changes, so telling its observers, only when a reading
//! differs. Each workspace has its own [`FleetPanel`], which draws from the global and starts the
//! reads when it draws and none run.
//!
//! A click on an agent selects it (#608): the panel then splits, the list above and the agent's
//! snapshot below. The reads keep the full detail only of the agents some panel has selected, or
//! some Agent tab shows. A double-click, Enter or the snapshot's Open opens the agent's tab
//! (#609, `crate::agent_tab`), and the reads keep a sample of each host's resources for its
//! graphs.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{
    Action, AnyElement, App, ClickEvent, Context, Div, DragMoveEvent, EntityId, EventEmitter,
    FocusHandle, Focusable, Global, Hsla, Pixels, Render, Subscription, Task, WeakEntity, Window,
    px, relative,
};
use marley_sdk::stale::DEFAULT_POLL_S;
use marley_sdk::{
    AgentDetail, AgentSummary, Attention, HostSnapshot, Memory, PhaseState, Pseudo, Run, State,
    is_stale,
};
use settings::{FleetProviderContent, Settings as _};
use ui::{
    Button, ButtonSize, ButtonStyle, Chip, Divider, Icon, IconName, IconSize, Label, LabelSize,
    ProgressBar, prelude::*,
};
use workspace::dock::{DockPosition, Panel, PanelEvent};
use workspace::{MultiWorkspace, Workspace};

use crate::agent_tab::{self, AgentView};
use crate::{MarleySettings, ToggleFleet};

/// The panel's place among the right dock's panels; no other panel takes it.
const ACTIVATION_PRIORITY: u32 = 20;

/// The snapshot's share of the panel's height until the user drags the split. A third hid the
/// question of an agent that asks one on a 1000 px tall window.
const SNAPSHOT_RATIO: f32 = 0.5;

/// How long a host's samples are kept for its graphs.
const SAMPLES_KEPT_MS: u64 = 30 * 60 * 1000;

/// What the fleet's providers last answered.
#[derive(Debug, Default)]
pub struct Fleet {
    pub(crate) sources: Vec<Source>,
    /// Each host's resources at each read, the last 30 minutes of them.
    pub(crate) samples: HashMap<HostKey, VecDeque<Sample>>,
    /// The pseudo provider, once a reading has started it.
    pseudo: Option<Pseudo>,
    /// Whether the reads run.
    pub(crate) polling: bool,
}

/// A host as one provider names it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct HostKey {
    pub(crate) source: String,
    pub(crate) host: String,
}

/// A host's resources at one read.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sample {
    pub(crate) at_ms: u64,
    /// The processor's use, in percent.
    pub(crate) cpu: Option<f32>,
    /// The memory used, in percent of the total.
    pub(crate) memory: Option<f32>,
    /// Bytes a second, in and out together.
    pub(crate) network: Option<u64>,
}

impl Global for Fleet {}

/// Each Fleet panel's selected agent and each Agent tab's agent, whose detail the reads keep. The
/// panels and tabs do not observe it, so a selection alone redraws nothing.
#[derive(Debug, Default)]
pub(crate) struct Wanted(pub(crate) HashMap<EntityId, Selected>);

impl Global for Wanted {}

/// An agent picked in a Fleet panel: its provider's name and its id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Selected {
    pub(crate) source: String,
    pub(crate) agent: String,
}

/// One provider, as the fleet last read it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Source {
    /// The provider's name, for its header.
    pub(crate) name: String,
    /// What its handshake offers, which decides the snapshot's sections.
    capabilities: Vec<String>,
    /// Its agents, in its order.
    pub(crate) agents: Vec<AgentSummary>,
    /// The full detail of its agents a panel has selected.
    pub(crate) details: Vec<AgentDetail>,
    /// Its hosts.
    pub(crate) hosts: Vec<HostSnapshot>,
    pub(crate) poll_s: Option<u64>,
    pub(crate) stale_after_s: Option<u64>,
    /// When it was read, which the stale rule measures from.
    pub(crate) read_ms: u64,
    /// Why it could not be read, when it could not.
    failure: Option<String>,
}

impl Source {
    /// A provider that could not be read, and why.
    fn failed(name: &str, failure: String, read_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            capabilities: Vec::new(),
            agents: Vec::new(),
            details: Vec::new(),
            hosts: Vec::new(),
            poll_s: None,
            stale_after_s: None,
            read_ms,
            failure: Some(failure),
        }
    }

    pub(crate) fn offers(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|offered| offered == capability)
    }

    /// Its agents under their hosts: the hosts in its order, then agents whose host it did not
    /// describe, each group in its own order.
    fn host_groups(&self) -> Vec<(Option<&str>, Vec<&AgentSummary>)> {
        let mut host_ids: Vec<Option<&str>> = self
            .hosts
            .iter()
            .map(|host| Some(host.host.id.as_str()))
            .collect();
        for agent in &self.agents {
            let id = agent.host_id.as_deref();
            if !host_ids.contains(&id) {
                host_ids.push(id);
            }
        }
        host_ids
            .into_iter()
            .filter_map(|host_id| {
                let agents: Vec<&AgentSummary> = self
                    .agents
                    .iter()
                    .filter(|agent| agent.host_id.as_deref() == host_id)
                    .collect();
                (!agents.is_empty()).then_some((host_id, agents))
            })
            .collect()
    }

    /// The snapshot of the host `host_id` names, from the agent's detail or the provider's hosts.
    pub(crate) fn host<'a>(&'a self, detail: &'a AgentDetail) -> Option<&'a HostSnapshot> {
        detail.host.as_ref().or_else(|| {
            let id = detail.agent.host_id.as_deref()?;
            self.hosts.iter().find(|host| host.host.id == id)
        })
    }

    pub(crate) fn host_name(&self, host_id: Option<&str>) -> String {
        host_id.map_or_else(
            || "No host".to_string(),
            |id| {
                self.hosts
                    .iter()
                    .find(|host| host.host.id == id)
                    .map_or_else(|| id.to_string(), |host| host.host.name.clone())
            },
        )
    }
}

/// The agents in the order the panel draws them, for the keys that walk them.
fn drawn_order(sources: &[Source]) -> Vec<Selected> {
    sources
        .iter()
        .flat_map(|source| {
            source
                .host_groups()
                .into_iter()
                .flat_map(|(_, agents)| agents)
                .map(|agent| Selected {
                    source: source.name.clone(),
                    agent: agent.id.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
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
            let weak_workspace = cx.weak_entity();
            let panel = cx.new(|cx| FleetPanel::new(weak_workspace, window, cx));
            workspace.add_panel(panel, window, cx);
        },
    )
    .detach();
}

/// Starts the reads, unless they run.
pub(crate) fn start_polling(cx: &mut App) {
    let fleet = cx.default_global::<Fleet>();
    if !fleet.polling {
        fleet.polling = true;
        poll_while_shown(cx).detach();
    }
}

/// Whether a Fleet surface shows in any window's shown workspace: a Fleet panel an open right
/// dock shows, or an Agent tab that is some pane's active item. Zed's docks call
/// `Panel::set_active` also for a panel activated in a closed dock, and a layout switch that moves
/// panels between docks can leave one showing without the call, so the reads ask the windows
/// rather than count those calls.
fn fleet_shows(cx: &App) -> bool {
    cx.windows()
        .iter()
        .filter_map(gpui::AnyWindowHandle::downcast::<MultiWorkspace>)
        .filter_map(|window| window.read(cx).ok())
        .any(|multi_workspace| {
            let workspace = multi_workspace.workspace().read(cx);
            let panel = workspace
                .right_dock()
                .read(cx)
                .visible_panel()
                .is_some_and(|panel| panel.to_any().downcast::<FleetPanel>().is_ok());
            panel
                || workspace.panes().iter().any(|pane| {
                    pane.read(cx)
                        .active_item()
                        .is_some_and(|item| item.downcast::<AgentView>().is_some())
                })
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

/// Reads every provider now, for a selection or a tab that should not wait for the next poll.
pub(crate) fn read_now(cx: &mut App) {
    // The wait it returns is the loop's to keep; this read only fills the snapshot sooner.
    if read_providers(cx).is_none() {
        log::debug!("fleet: a selection's read found no Fleet panel showing");
    }
}

/// Reads every provider once: `None` once no Fleet surface shows, else how long to wait.
fn read_providers(cx: &mut App) -> Option<Duration> {
    if !fleet_shows(cx) {
        cx.default_global::<Fleet>().polling = false;
        return None;
    }
    // Read through `try_global`: `default_global` tells the observers, so it waits for a change.
    let known = cx.try_global::<Fleet>();
    let wanted: Vec<Selected> = cx
        .try_global::<Wanted>()
        .map(|wanted| wanted.0.values().cloned().collect())
        .unwrap_or_default();
    let now = now_ms();
    let providers = MarleySettings::get_global(cx).fleet_providers.clone();
    let mut pseudo = known.and_then(|fleet| fleet.pseudo.clone());
    let sources: Vec<Source> = providers
        .iter()
        .map(|provider| match provider {
            FleetProviderContent::Pseudo => read_pseudo(&mut pseudo, now, &wanted),
        })
        .collect();
    let wait_s = sources
        .iter()
        .filter_map(|source| source.poll_s)
        .min()
        .unwrap_or(DEFAULT_POLL_S)
        .max(1);
    let started = known.is_some_and(|fleet| fleet.pseudo.is_some());
    let samples = sampled(
        known.map(|fleet| fleet.samples.clone()).unwrap_or_default(),
        &sources,
        now,
    );
    if known.is_none_or(|fleet| fleet.sources != sources || fleet.samples != samples)
        || started != pseudo.is_some()
    {
        let fleet = cx.default_global::<Fleet>();
        fleet.sources = sources;
        fleet.samples = samples;
        fleet.pseudo = pseudo;
    }
    Some(Duration::from_secs(wait_s))
}

/// `samples` with one more for each host `sources` describe, taken at `now`, and without those
/// older than 30 minutes.
fn sampled(
    mut samples: HashMap<HostKey, VecDeque<Sample>>,
    sources: &[Source],
    now: u64,
) -> HashMap<HostKey, VecDeque<Sample>> {
    for source in sources {
        for host in &source.hosts {
            let key = HostKey {
                source: source.name.clone(),
                host: host.host.id.clone(),
            };
            samples.entry(key).or_default().push_back(Sample {
                at_ms: now,
                cpu: host.cpu.as_ref().map(|cpu| cpu.percent),
                memory: host.memory.as_ref().map(memory_share),
                network: host
                    .network
                    .as_ref()
                    .map(|network| network.rx_bps.saturating_add(network.tx_bps)),
            });
        }
    }
    let oldest = now.saturating_sub(SAMPLES_KEPT_MS);
    samples.retain(|_, kept| {
        while kept.front().is_some_and(|sample| sample.at_ms < oldest) {
            kept.pop_front();
        }
        !kept.is_empty()
    });
    samples
}

/// The memory used, in whole percent of the total.
pub(crate) fn memory_share(memory: &Memory) -> f32 {
    let share = memory
        .used_bytes
        .saturating_mul(100)
        .checked_div(memory.total_bytes)
        .unwrap_or(0)
        .min(100);
    f32::from(u16::try_from(share).unwrap_or(100))
}

/// The pseudo provider's reading at `now`, starting it on the first, with the detail of the
/// agents `wanted` names.
fn read_pseudo(pseudo: &mut Option<Pseudo>, now: u64, wanted: &[Selected]) -> Source {
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
            capabilities: started.handshake().capabilities.clone(),
            agents: reading.list.agents,
            details: reading
                .details
                .into_iter()
                .filter(|detail| {
                    wanted.iter().any(|selected| {
                        selected.source == NAME && selected.agent == detail.agent.id
                    })
                })
                .collect(),
            hosts: reading.hosts,
            poll_s: started.handshake().poll_s,
            stale_after_s: started.handshake().stale_after_s,
            read_ms: now,
            failure: None,
        },
        Err(error) => Source::failed(NAME, error.to_string(), now),
    }
}

/// What the split's handle carries while it is dragged.
#[derive(Debug)]
struct DraggedFleetSplit;

/// The Fleet panel: every provider's agents, grouped by host, and the selected one's snapshot.
pub struct FleetPanel {
    focus_handle: FocusHandle,
    workspace: WeakEntity<Workspace>,
    selected: Option<Selected>,
    /// The snapshot's share of the height below the header.
    snapshot_ratio: f32,
    _fleet: Subscription,
}

impl std::fmt::Debug for FleetPanel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FleetPanel")
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl FleetPanel {
    fn new(workspace: WeakEntity<Workspace>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let id = cx.entity_id();
        cx.on_release(move |_, cx| {
            if cx.has_global::<Wanted>() {
                cx.global_mut::<Wanted>().0.remove(&id);
            }
        })
        .detach();
        Self {
            focus_handle: cx.focus_handle(),
            workspace,
            selected: None,
            snapshot_ratio: SNAPSHOT_RATIO,
            _fleet: cx.observe_global::<Fleet>(|panel, cx| {
                panel.drop_gone_selection(cx);
                cx.notify();
            }),
        }
    }

    /// Selects an agent, and reads at once so its snapshot fills without waiting for a poll.
    fn select(&mut self, selected: Selected, cx: &mut Context<Self>) {
        if self.selected.as_ref() == Some(&selected) {
            return;
        }
        let id = cx.entity_id();
        cx.default_global::<Wanted>().0.insert(id, selected.clone());
        self.selected = Some(selected);
        cx.defer(read_now);
        cx.notify();
    }

    /// Opens the selected agent's tab, or shows the one open. Deferred, since the opener reads
    /// the Agent tabs, and a tab's own click would have it read the tab being updated.
    fn open_selected(&self, window: &Window, cx: &mut Context<Self>) {
        let Some(selected) = self.selected.clone() else {
            return;
        };
        agent_tab::open_later(self.workspace.clone(), selected, window, cx);
    }

    /// Drops the selection once the list no longer holds its agent.
    fn drop_gone_selection(&mut self, cx: &mut Context<Self>) {
        let Some(selected) = &self.selected else {
            return;
        };
        let listed = cx.try_global::<Fleet>().is_some_and(|fleet| {
            fleet.sources.iter().any(|source| {
                source.name == selected.source
                    && source.agents.iter().any(|agent| agent.id == selected.agent)
            })
        });
        if !listed {
            self.selected = None;
            let id = cx.entity_id();
            if cx.has_global::<Wanted>() {
                cx.global_mut::<Wanted>().0.remove(&id);
            }
        }
    }

    /// Moves the selection one agent down, or up, in the order drawn.
    fn step(&mut self, down: bool, cx: &mut Context<Self>) {
        let order = cx
            .try_global::<Fleet>()
            .map(|fleet| drawn_order(&fleet.sources))
            .unwrap_or_default();
        let at = self
            .selected
            .as_ref()
            .and_then(|selected| order.iter().position(|agent| agent == selected));
        let next = match (at, down) {
            (None, true) => order.first(),
            (None, false) => order.last(),
            (Some(at), true) => order.get(at + 1).or_else(|| order.get(at)),
            (Some(at), false) => order.get(at.saturating_sub(1)),
        };
        if let Some(next) = next.cloned() {
            self.select(next, cx);
        }
    }

    fn render_source(&self, source: &Source, index: &mut usize, cx: &Context<Self>) -> AnyElement {
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
        let mut groups = Vec::new();
        for (host_id, agents) in source.host_groups() {
            let mut rows = Vec::new();
            for agent in agents {
                rows.push(self.render_agent(agent, source, *index, cx));
                *index += 1;
            }
            groups.push(
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
                            .child(Label::new(source.host_name(host_id)).size(LabelSize::Small)),
                    )
                    .children(rows),
            );
        }
        v_flex().child(header).children(groups).into_any_element()
    }

    fn render_agent(
        &self,
        agent: &AgentSummary,
        source: &Source,
        index: usize,
        cx: &Context<Self>,
    ) -> AnyElement {
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
        let selected = Selected {
            source: source.name.clone(),
            agent: agent.id.clone(),
        };
        let is_selected = self.selected.as_ref() == Some(&selected);
        let colors = cx.theme().colors();
        let (fill, border, hover) = (
            colors.ghost_element_selected,
            colors.border,
            colors.ghost_element_hover,
        );
        h_flex()
            .id(("marley-fleet-agent", index))
            .mx_2()
            .px_2()
            .py_1()
            .gap_2()
            .rounded_md()
            .border_1()
            .cursor_pointer()
            .map(|row| {
                if is_selected {
                    row.border_color(border).bg(fill)
                } else {
                    row.border_color(gpui::transparent_black())
                        .hover(|style| style.bg(hover))
                }
            })
            .on_click(cx.listener(move |panel, event: &ClickEvent, window, cx| {
                window.focus(&panel.focus_handle, cx);
                panel.select(selected.clone(), cx);
                if event.click_count() == 2 {
                    panel.open_selected(window, cx);
                }
            }))
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
            .children(attention_mark(agent.attention))
            .child(
                Chip::new(word)
                    .label_color(color)
                    .label_size(LabelSize::XSmall),
            )
            .into_any_element()
    }

    /// The split's handle: a line with a taller grip to drag.
    fn render_handle(cx: &App) -> AnyElement {
        div()
            .id("marley-fleet-split")
            .relative()
            .w_full()
            .h(px(1.))
            .flex_none()
            .bg(cx.theme().colors().border_variant)
            .child(
                div()
                    .id("marley-fleet-split-grip")
                    .absolute()
                    .top(px(-3.))
                    .w_full()
                    .h(px(7.))
                    .cursor_row_resize()
                    .block_mouse_except_scroll()
                    .on_drag(DraggedFleetSplit, |_, _, _, cx| cx.new(|_| gpui::Empty)),
            )
            .into_any_element()
    }
}

/// The mark an agent's attention puts on its row.
pub(crate) fn attention_mark(attention: Attention) -> Option<Icon> {
    match attention {
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
    }
}

/// A snapshot section: its title over its body.
pub(crate) fn section(title: &'static str, body: impl IntoElement) -> Div {
    v_flex()
        .gap_1()
        .child(
            Label::new(title)
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
        .child(body)
}

/// The selected agent's snapshot: what it is, what it works on, where its run stands, what its
/// host uses, its tokens today and its question. A section the provider does not offer is left
/// out.
fn render_snapshot(
    detail: &AgentDetail,
    source: &Source,
    open: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let work_item = detail
        .work_item
        .as_ref()
        .filter(|_| source.offers("work_items"))
        .map(|item| {
            section(
                "WORK ITEM",
                v_flex()
                    .child(
                        Label::new(format!("{} · {}", item.key, item.title)).size(LabelSize::Small),
                    )
                    .children(item.status.as_ref().map(|status| {
                        Label::new(status.clone())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                    })),
            )
        });
    let run = detail
        .run
        .as_ref()
        .filter(|_| source.offers("runs"))
        .map(|run| section("RUN", render_phase_strip(run, cx)));
    let resources = source
        .offers("hosts")
        .then(|| section("RESOURCES", render_resources(source.host(detail), cx)));
    let tokens = detail
        .usage
        .as_ref()
        .and_then(|usage| usage.today.as_ref())
        .filter(|_| source.offers("usage"))
        .map(|today| {
            section(
                "TOKENS TODAY",
                Label::new(format!(
                    "in {} · out {}",
                    compact(today.input),
                    compact(today.output)
                ))
                .size(LabelSize::Small),
            )
        });
    let question = detail
        .question
        .as_ref()
        .filter(|_| source.offers("questions"))
        .map(|question| {
            section(
                "QUESTION",
                v_flex()
                    .gap_1()
                    .child(Label::new(question.prompt.clone()).size(LabelSize::Small))
                    .child(
                        h_flex().flex_wrap().gap_1().children(
                            question
                                .options
                                .iter()
                                .map(|option| Chip::new(option.clone())),
                        ),
                    ),
            )
        });
    v_flex()
        .p_2()
        .gap_2()
        .child(render_snapshot_header(detail, source, open))
        .children(work_item)
        .children(run)
        .children(resources)
        .children(tokens)
        .children(question)
        .into_any_element()
}

/// The snapshot's first lines: the agent's name and state, what runs it and for how long it has
/// been in that state, and where it runs; `end` closes the first line.
pub(crate) fn render_snapshot_header(
    detail: &AgentDetail,
    source: &Source,
    end: Option<AnyElement>,
) -> Div {
    let agent = &detail.agent;
    let stale = is_stale(
        agent.last_seen_ms,
        source.read_ms,
        source.poll_s,
        source.stale_after_s,
    );
    let (word, color) = state_chip(agent.state, stale);
    let mut about = vec![agent.runtime.clone()];
    about.extend(agent.model.clone());
    if let Some(since) = agent.state_since_ms {
        about.push(format!("for {}", how_long(since, source.read_ms)));
    }
    let mut place = vec![source.host_name(agent.host_id.as_deref())];
    place.extend(agent.cwd.clone());
    v_flex()
        .gap_0p5()
        .child(
            h_flex()
                .gap_2()
                .child(
                    Icon::new(runtime_icon(&agent.runtime))
                        .size(IconSize::Small)
                        .color(Color::Muted),
                )
                .child(
                    Label::new(agent.name.clone())
                        .size(LabelSize::Default)
                        .truncate(),
                )
                .child(div().flex_1())
                .child(
                    Chip::new(word)
                        .label_color(color)
                        .label_size(LabelSize::XSmall),
                )
                .children(end),
        )
        .child(
            Label::new(about.join(" · "))
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .child(
            Label::new(place.join(" · "))
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
}

/// A run's phases as one strip, a segment each in the run's order, with the active or failed
/// phase named under it.
fn render_phase_strip(run: &Run, cx: &App) -> Div {
    let count = run.phases.len();
    let named = run
        .active_phase()
        .map(|(index, phase)| format!("phase {}/{count}: {}", index + 1, phase.name))
        .or_else(|| {
            run.phases
                .iter()
                .enumerate()
                .find(|(_, phase)| phase.state == PhaseState::Failed)
                .map(|(index, phase)| format!("failed at {} ({}/{count})", phase.name, index + 1))
        })
        .unwrap_or_else(|| format!("{count} phases"));
    let failed = run.failed() && run.active_phase().is_none();
    v_flex()
        .gap_1()
        .child(h_flex().gap_0p5().children(run.phases.iter().map(|phase| {
            div()
                .flex_1()
                .h(px(6.))
                .rounded_sm()
                .bg(phase_color(phase.state, cx))
        })))
        .child(Label::new(named).size(LabelSize::XSmall).color(if failed {
            Color::Error
        } else {
            Color::Muted
        }))
}

/// A phase segment's colour.
pub(crate) fn phase_color(state: PhaseState, cx: &App) -> Hsla {
    let status = cx.theme().status();
    let colors = cx.theme().colors();
    match state {
        PhaseState::Passed => status.success,
        PhaseState::Active => colors.text_accent,
        PhaseState::Failed => status.error,
        PhaseState::Skipped => colors.border_variant,
        PhaseState::Pending | PhaseState::Unknown => colors.border,
    }
}

/// The host's processor and memory as bars, or a line saying none came yet.
pub(crate) fn render_resources(host: Option<&HostSnapshot>, cx: &App) -> AnyElement {
    let Some(host) = host else {
        return Label::new("no resources yet")
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .into_any_element();
    };
    let mut bars = Vec::new();
    if let Some(cpu) = &host.cpu {
        bars.push(meter(
            "CPU",
            "marley-fleet-cpu",
            cpu.percent,
            format!("{:.0} %", cpu.percent),
            cx,
        ));
    }
    if let Some(memory) = &host.memory {
        bars.push(meter(
            "Memory",
            "marley-fleet-memory",
            memory_share(memory),
            format!(
                "{} / {} GB",
                gigabytes(memory.used_bytes),
                gigabytes(memory.total_bytes)
            ),
            cx,
        ));
    }
    if bars.is_empty() {
        return Label::new("no resources yet")
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .into_any_element();
    }
    v_flex().gap_1().children(bars).into_any_element()
}

/// One resource: its name, a bar out of 100 and its number.
fn meter(
    name: &'static str,
    id: &'static str,
    percent: f32,
    value: String,
    cx: &App,
) -> AnyElement {
    h_flex()
        .gap_2()
        .child(
            div()
                .w(px(52.))
                .flex_none()
                .child(Label::new(name).size(LabelSize::XSmall).color(Color::Muted)),
        )
        .child(
            div()
                .flex_1()
                .child(ProgressBar::new(id, percent, 100., cx)),
        )
        .child(
            div()
                .w(px(84.))
                .flex_none()
                .child(Label::new(value).size(LabelSize::XSmall)),
        )
        .into_any_element()
}

/// How long ago `since_ms` was at `now_ms`, in its largest whole unit.
pub(crate) fn how_long(since_ms: u64, now_ms: u64) -> String {
    let seconds = now_ms.saturating_sub(since_ms) / 1000;
    match seconds {
        0..60 => format!("{seconds} s"),
        60..3600 => format!("{} m", seconds / 60),
        3600..86_400 => format!("{} h", seconds / 3600),
        _ => format!("{} d", seconds / 86_400),
    }
}

/// A token count in a few characters: 3.9M, 310k, 950.
pub(crate) fn compact(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{}.{}M", count / 1_000_000, count % 1_000_000 / 100_000)
    } else if count >= 1_000 {
        format!("{}k", count / 1_000)
    } else {
        count.to_string()
    }
}

/// Bytes as gigabytes with one decimal.
pub(crate) fn gigabytes(bytes: u64) -> String {
    let tenths = bytes / 100_000_000;
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// The word and colour of an agent's state chip; a stale agent reads stale, whatever it said.
pub(crate) const fn state_chip(state: State, quiet: bool) -> (&'static str, Color) {
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
pub(crate) fn runtime_icon(runtime: &str) -> IconName {
    match runtime {
        "claude-code" | "claude" => IconName::AiClaude,
        "codex" => IconName::AiOpenAi,
        "gemini" => IconName::AiGemini,
        _ => IconName::Terminal,
    }
}

impl FleetPanel {
    /// Every source's agents, or the line that says the fleet is not set up.
    fn render_list(&self, sources: &[Source], cx: &Context<Self>) -> AnyElement {
        let set_up = !MarleySettings::get_global(cx).fleet_providers.is_empty();
        let mut index = 0;
        let mut listed = Vec::new();
        for source in sources {
            listed.push(self.render_source(source, &mut index, cx));
        }
        v_flex()
            .id("marley-fleet-list")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pb_2()
            .when(!set_up, |list| {
                list.child(
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
            .children(listed)
            .into_any_element()
    }

    /// The selected agent's snapshot, or a line while its first detail is read; `None` with no
    /// selection.
    fn render_selected(&self, sources: &[Source], cx: &Context<Self>) -> Option<AnyElement> {
        let selected = self.selected.as_ref()?;
        let source = sources
            .iter()
            .find(|source| source.name == selected.source)?;
        let snapshot = source
            .details
            .iter()
            .find(|detail| detail.agent.id == selected.agent)
            .map_or_else(
                || {
                    div()
                        .p_2()
                        .child(
                            Label::new("Reading…")
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        )
                        .into_any_element()
                },
                |detail| {
                    let open = Button::new("marley-fleet-open", "Open")
                        .style(ButtonStyle::Outlined)
                        .size(ButtonSize::Compact)
                        .on_click(cx.listener(|panel, _: &ClickEvent, window, cx| {
                            panel.open_selected(window, cx);
                        }))
                        .into_any_element();
                    render_snapshot(detail, source, Some(open), cx)
                },
            );
        Some(snapshot)
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
        let list = self.render_list(&sources, cx);
        let snapshot = self.render_selected(&sources, cx);
        let ratio = self.snapshot_ratio;
        v_flex()
            .id("marley-fleet-panel")
            .key_context("FleetPanel menu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|panel, _: &menu::SelectNext, _, cx| panel.step(true, cx)))
            .on_action(cx.listener(|panel, _: &menu::SelectPrevious, _, cx| {
                panel.step(false, cx);
            }))
            .on_action(cx.listener(|panel, _: &menu::Confirm, window, cx| {
                panel.open_selected(window, cx);
            }))
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(
                h_flex().px_2().py_1p5().child(
                    Label::new("FLEET")
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
            )
            .child(Divider::horizontal())
            .child(
                v_flex()
                    .id("marley-fleet-body")
                    .flex_1()
                    .min_h_0()
                    .on_drag_move::<DraggedFleetSplit>(cx.listener(
                        |panel, event: &DragMoveEvent<DraggedFleetSplit>, _, cx| {
                            let bounds = event.bounds;
                            let height = bounds.size.height;
                            if height > px(0.) {
                                let below = bounds.bottom() - event.event.position.y;
                                panel.snapshot_ratio = (below / height).clamp(0.15, 0.85);
                                cx.notify();
                            }
                        },
                    ))
                    .child(list)
                    .when_some(snapshot, |body, snapshot| {
                        body.child(Self::render_handle(cx)).child(
                            div()
                                .id("marley-fleet-snapshot")
                                .flex_none()
                                .h(relative(ratio))
                                .overflow_y_scroll()
                                .child(snapshot),
                        )
                    }),
            )
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
