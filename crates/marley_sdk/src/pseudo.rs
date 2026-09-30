//! A provider with no store behind it (#607): the contract's own examples, moved by the clock it
//! is given, so the fleet can be built and shown before a real store answers.
//!
//! The fixtures under `fixtures/` are the examples of `docs/marley/fleet-contract.md`: three
//! agents on two hosts, one working, one waiting on a question and one whose gate failed. Each
//! reading moves every time in them to the reading's own moment, then moves the working agent's
//! run along, adds its events and tokens, lets the hosts' processor and network wander, and has
//! the third agent stop reporting a little after the start, so it reads stale.

use std::fmt;

use serde_json::Value;

use crate::host::HostSnapshot;
use crate::work::{
    AgentDetail, AgentList, AgentSummary, Attention, Event, GateState, Handshake, PhaseState,
    PhaseSummary, WorkItem,
};

const HANDSHAKE: &str = include_str!("../fixtures/handshake.json");
const DETAILS: &str = include_str!("../fixtures/details.json");
const HOSTS: &str = include_str!("../fixtures/hosts.json");

/// The moment the fixtures' times are written against.
pub const FIXTURE_NOW_MS: u64 = 1_759_243_100_000;

/// Seconds between the working agent's events.
const EVENT_EVERY_S: u64 = 8;

/// Seconds between its run's phases.
const PHASE_EVERY_S: u64 = 60;

/// Seconds after the start at which the third agent stops reporting.
const QUIET_AFTER_S: u64 = 12;

/// How many of the working agent's own events a reading keeps.
const EVENTS_KEPT: u64 = 6;

/// What the working agent's events say, in turn.
const EVENT_TEXTS: [(&str, &str); 5] = [
    ("tool", "edited crates/dispatch/src/lib.rs"),
    ("gate", "cargo check green"),
    ("tool", "ran the dispatch tests"),
    ("gate", "clippy: no warnings"),
    ("commit", "moved the queue into dispatch"),
];

/// A wobble in percent, a coarse sine, so the numbers move without a float cast.
const WOBBLE: [i32; 12] = [0, 31, 54, 62, 54, 31, 0, -31, -54, -62, -54, -31];

/// The fixtures did not parse, which is a fault in this crate's own files.
#[derive(Debug)]
pub struct PseudoError(serde_json::Error);

impl fmt::Display for PseudoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "the pseudo provider's fixtures did not parse: {}",
            self.0
        )
    }
}

impl std::error::Error for PseudoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<serde_json::Error> for PseudoError {
    fn from(error: serde_json::Error) -> Self {
        Self(error)
    }
}

/// The pseudo provider, started at a moment.
#[derive(Debug, Clone)]
pub struct Pseudo {
    started_ms: u64,
    handshake: Handshake,
    details: Value,
    hosts: Value,
}

/// One reading of the pseudo provider.
#[derive(Debug, Clone, PartialEq)]
pub struct PseudoState {
    /// The agents, for the list.
    pub list: AgentList,
    /// Each agent in full, in the list's order.
    pub details: Vec<AgentDetail>,
    /// The hosts.
    pub hosts: Vec<HostSnapshot>,
}

impl Pseudo {
    /// The provider, started at `started_ms`.
    ///
    /// # Errors
    ///
    /// When the fixtures do not parse.
    pub fn new(started_ms: u64) -> Result<Self, PseudoError> {
        Ok(Self {
            started_ms,
            handshake: serde_json::from_str(HANDSHAKE)?,
            details: serde_json::from_str(DETAILS)?,
            hosts: serde_json::from_str(HOSTS)?,
        })
    }

    /// What it answers first.
    #[must_use]
    pub const fn handshake(&self) -> &Handshake {
        &self.handshake
    }

    /// Its reading at `now_ms`.
    ///
    /// # Errors
    ///
    /// When the fixtures, moved to `now_ms`, do not parse.
    pub fn at(&self, now_ms: u64) -> Result<PseudoState, PseudoError> {
        let mut details = self.details.clone();
        rebase(&mut details, now_ms);
        let mut hosts = self.hosts.clone();
        rebase(&mut hosts, now_ms);
        let mut details: Vec<AgentDetail> = serde_json::from_value(details)?;
        let mut hosts: Vec<HostSnapshot> = serde_json::from_value(hosts)?;
        let elapsed_s = now_ms.saturating_sub(self.started_ms) / 1000;
        for (index, detail) in details.iter_mut().enumerate() {
            detail.agent.last_seen_ms = Some(match index {
                // The third agent stops reporting a little after the start.
                2 => now_ms.min(self.started_ms + QUIET_AFTER_S * 1000),
                _ => now_ms,
            });
        }
        if let Some(working) = details.first_mut() {
            advance(working, self.started_ms, now_ms);
        }
        for (index, host) in hosts.iter_mut().enumerate() {
            wander(host, index, elapsed_s);
        }
        let list = AgentList {
            agents: details.iter().map(summary).collect(),
            cursor: Some(format!("pseudo-{elapsed_s}")),
        };
        Ok(PseudoState {
            list,
            details,
            hosts,
        })
    }
}

/// Moves every time in `value`, each number under a key ending `_ms`, so its distance from
/// [`FIXTURE_NOW_MS`] becomes its distance from `now_ms`.
fn rebase(value: &mut Value, now_ms: u64) {
    match value {
        Value::Object(fields) => {
            for (key, field) in fields.iter_mut() {
                match field.as_u64() {
                    Some(at) if key.ends_with("_ms") => {
                        *field =
                            Value::from(now_ms.saturating_sub(FIXTURE_NOW_MS.saturating_sub(at)));
                    }
                    _ => rebase(field, now_ms),
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                rebase(item, now_ms);
            }
        }
        _ => {}
    }
}

/// The working agent's run, moved on: a phase a minute through `code`, `test` and `complete`,
/// an event every few seconds, and its tokens.
fn advance(detail: &mut AgentDetail, started_ms: u64, now_ms: u64) {
    let elapsed_s = now_ms.saturating_sub(started_ms) / 1000;
    let events: Vec<Event> = (1..=elapsed_s / EVENT_EVERY_S)
        .rev()
        .take(usize::try_from(EVENTS_KEPT).unwrap_or(usize::MAX))
        .map(|tick| {
            let (kind, text) = EVENT_TEXTS[index_of(tick, EVENT_TEXTS.len())];
            Event {
                at_ms: started_ms + tick * EVENT_EVERY_S * 1000,
                kind: kind.to_string(),
                text: text.to_string(),
            }
        })
        .collect();
    let recent = std::mem::take(&mut detail.recent_events);
    detail.recent_events = events.iter().cloned().chain(recent).collect();
    if let Some(run) = &mut detail.run {
        let count = run.phases.len();
        // Phase 0 has passed; the active one steps through the rest, one a minute.
        let active = 1 + index_of(elapsed_s / PHASE_EVERY_S, count.saturating_sub(1).max(1));
        for (index, phase) in run.phases.iter_mut().enumerate() {
            phase.state = match index.cmp(&active) {
                std::cmp::Ordering::Less => PhaseState::Passed,
                std::cmp::Ordering::Equal => PhaseState::Active,
                std::cmp::Ordering::Greater => PhaseState::Pending,
            };
            if index == active {
                phase.started_ms = Some(now_ms.saturating_sub((elapsed_s % PHASE_EVERY_S) * 1000));
                phase.ended_ms = None;
            }
            if index < active {
                for gate in &mut phase.gates {
                    gate.state = GateState::Pass;
                }
            }
        }
        let log = std::mem::take(&mut run.events);
        run.events = events.into_iter().chain(log).collect();
    }
    if let Some(usage) = &mut detail.usage {
        for tokens in [&mut usage.run, &mut usage.today].into_iter().flatten() {
            tokens.input += elapsed_s * 420;
            tokens.output += elapsed_s * 35;
        }
    }
}

/// A host's processor and network, moved by a wobble.
fn wander(host: &mut HostSnapshot, index: usize, elapsed_s: u64) {
    let offset = u64::try_from(index).unwrap_or(0) * 3;
    let step = index_of(elapsed_s / 2 + offset, WOBBLE.len());
    let wobble = WOBBLE[step];
    if let Some(cpu) = &mut host.cpu {
        cpu.percent = (cpu.percent + f64::from(wobble) / 3.0).clamp(1.0, 99.0);
    }
    if let Some(network) = &mut host.network {
        let factor = u64::try_from(100 + wobble).unwrap_or(100);
        network.rx_bps = network.rx_bps * factor / 100;
        network.tx_bps = network.tx_bps * factor / 100;
    }
}

/// `value` wrapped into `0..len`, as an index.
fn index_of(value: u64, len: usize) -> usize {
    let len = u64::try_from(len).unwrap_or(1).max(1);
    usize::try_from(value % len).unwrap_or(0)
}

/// An agent's detail, cut to what the list shows.
fn summary(detail: &AgentDetail) -> AgentSummary {
    let phase = detail.run.as_ref().and_then(|run| {
        let found = run.active_phase().or_else(|| {
            run.phases
                .iter()
                .enumerate()
                .find(|(_, phase)| phase.state == PhaseState::Failed)
        });
        found.map(|(index, phase)| PhaseSummary {
            name: phase.name.clone(),
            index,
            count: run.phases.len(),
        })
    });
    let attention = if detail.question.is_some() {
        Attention::Question
    } else if detail.run.as_ref().is_some_and(crate::work::Run::failed) {
        Attention::Failed
    } else {
        Attention::None
    };
    AgentSummary {
        id: detail.agent.id.clone(),
        name: detail.agent.name.clone(),
        runtime: detail.agent.runtime.clone(),
        state: detail.agent.state,
        state_since_ms: detail.agent.state_since_ms,
        last_seen_ms: detail.agent.last_seen_ms,
        host_id: detail.agent.host_id.clone(),
        work_item: detail.work_item.as_ref().map(|item| WorkItem {
            id: item.id.clone(),
            key: item.key.clone(),
            title: item.title.clone(),
            status: None,
            url: None,
            summary: None,
        }),
        phase,
        attention,
        question: detail.question.clone(),
    }
}
