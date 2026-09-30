//! `marley.work/v1`: what a central workflow store answers (`docs/marley/fleet-contract.md`).

use serde::{Deserialize, Serialize};

use crate::host::HostSnapshot;

/// The `contract` name this version reads.
pub const WORK_CONTRACT: &str = "marley.work/v1";

/// What a store answers first: its contract, who it is, and which parts it fills.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Handshake {
    /// The contract and its major version, `marley.work/v1` for this one.
    pub contract: String,
    /// Who answers.
    #[serde(default)]
    pub provider: Provider,
    /// The parts it fills; Marley hides the parts it leaves out.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// How often Marley should ask for changes, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_s: Option<u64>,
    /// How long a quiet agent is still normal, in seconds, when it is longer than three polls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_after_s: Option<u64>,
}

impl Handshake {
    /// Whether the store fills the part named `capability`.
    #[must_use]
    pub fn offers(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|offered| offered == capability)
    }
}

/// Who answers a contract.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provider {
    /// Its name, shown in the Fleet panel's header.
    #[serde(default)]
    pub name: String,
    /// Its version.
    #[serde(default)]
    pub version: String,
}

/// What an agent is doing: the states Marley's fleet already uses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// It is starting.
    Starting,
    /// It works.
    Working,
    /// It has nothing to do.
    #[default]
    Idle,
    /// It waits on a person.
    Waiting,
    /// It failed.
    Error,
    /// It finished.
    Done,
    /// A state this version does not name.
    #[serde(other)]
    Unknown,
}

/// Why an agent needs a person, if it does.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Attention {
    /// Nothing.
    #[default]
    None,
    /// It waits on a question.
    Question,
    /// A gate or its run failed.
    Failed,
    /// A reason this version does not name.
    #[serde(other)]
    Unknown,
}

/// The agents a store knows, for the Fleet panel's list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentList {
    /// Every agent, in the store's order.
    #[serde(default)]
    pub agents: Vec<AgentSummary>,
    /// Where the change feed continues from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// One agent, as the list shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSummary {
    /// Its id, opaque and stable.
    pub id: String,
    /// The name shown.
    #[serde(default)]
    pub name: String,
    /// What runs: `claude-code`, `codex`, `script`, or any name, shown as given.
    #[serde(default)]
    pub runtime: String,
    /// What it is doing.
    #[serde(default)]
    pub state: State,
    /// When it entered that state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_since_ms: Option<u64>,
    /// The last time it reported anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_ms: Option<u64>,
    /// The host it runs on, which joins the host collector's data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    /// What it works on, in short.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item: Option<WorkItem>,
    /// Its run's current phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<PhaseSummary>,
    /// Why it needs a person.
    #[serde(default)]
    pub attention: Attention,
    /// Its question, while it waits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<Question>,
}

/// Where a run stands, in short: its current phase, by name and place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseSummary {
    /// The phase's name, shown as given.
    pub name: String,
    /// Its place among the run's phases, from 0.
    pub index: usize,
    /// How many phases the run has.
    pub count: usize,
}

/// A work item: a ticket, an issue, whatever the store calls what an agent works on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItem {
    /// Its id.
    pub id: String,
    /// The short name people use, as `RB-142`.
    #[serde(default)]
    pub key: String,
    /// Its title.
    #[serde(default)]
    pub title: String,
    /// The store's own word for where it stands, shown as given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Where it can be read, opened in a Browser tab.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A line or two on what it asks for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// One agent in full: the snapshot and the Agent tab read this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDetail {
    /// The agent.
    pub agent: AgentInfo,
    /// What it works on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item: Option<WorkItem>,
    /// Its current run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<Run>,
    /// Its latest events, newest first.
    #[serde(default)]
    pub recent_events: Vec<Event>,
    /// The tokens its model used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// Its question, while it waits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<Question>,
    /// Its host's snapshot, when the store keeps one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<HostSnapshot>,
}

/// The agent part of a detail: the list's fields and where it works.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Its id.
    pub id: String,
    /// The name shown.
    #[serde(default)]
    pub name: String,
    /// What runs.
    #[serde(default)]
    pub runtime: String,
    /// What it is doing.
    #[serde(default)]
    pub state: State,
    /// When it entered that state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_since_ms: Option<u64>,
    /// The last time it reported anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_ms: Option<u64>,
    /// The host it runs on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    /// The folder it works in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// The model it runs, shown as given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// One pass of a work item through its workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    /// Its id.
    pub id: String,
    /// The work item it serves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    /// The workflow it follows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowRef>,
    /// The agent that runs it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// When it began.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_ms: Option<u64>,
    /// When it ended, once it has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_ms: Option<u64>,
    /// How it ended, once it has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Outcome>,
    /// Its phases, in the workflow's order.
    #[serde(default)]
    pub phases: Vec<PhaseRun>,
    /// Its whole log, newest first.
    #[serde(default)]
    pub events: Vec<Event>,
}

impl Run {
    /// The phase under way, with its place: the one that is active, else none.
    #[must_use]
    pub fn active_phase(&self) -> Option<(usize, &PhaseRun)> {
        self.phases
            .iter()
            .enumerate()
            .find(|(_, phase)| phase.state == PhaseState::Active)
    }

    /// Whether a phase or a gate of the run failed.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.outcome == Some(Outcome::Failed)
            || self.phases.iter().any(|phase| {
                phase.state == PhaseState::Failed
                    || phase.gates.iter().any(|gate| gate.state == GateState::Fail)
            })
    }
}

/// A workflow, by id and name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowRef {
    /// Its id.
    pub id: String,
    /// Its name, shown as given.
    #[serde(default)]
    pub name: String,
}

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// Every phase passed.
    Passed,
    /// A phase failed.
    Failed,
    /// It was left.
    Abandoned,
    /// An outcome this version does not name.
    #[serde(other)]
    Unknown,
}

/// One phase of a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseRun {
    /// Its name, shown as given; Marley reads no meaning into it.
    pub name: String,
    /// Where it stands.
    #[serde(default)]
    pub state: PhaseState,
    /// When it began.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_ms: Option<u64>,
    /// When it ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_ms: Option<u64>,
    /// A line on what it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Its gates.
    #[serde(default)]
    pub gates: Vec<Gate>,
}

/// Where a phase stands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhaseState {
    /// Not begun.
    #[default]
    Pending,
    /// Under way.
    Active,
    /// Passed.
    Passed,
    /// Failed.
    Failed,
    /// Left out.
    Skipped,
    /// A state this version does not name.
    #[serde(other)]
    Unknown,
}

/// A check a phase must pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gate {
    /// Its name, shown as given.
    pub name: String,
    /// Its result.
    #[serde(default)]
    pub state: GateState,
    /// A short line on a failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A gate's result.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateState {
    /// Not run yet.
    #[default]
    Pending,
    /// Passed.
    Pass,
    /// Failed.
    Fail,
    /// Left out.
    Skipped,
    /// A result this version does not name.
    #[serde(other)]
    Unknown,
}

/// Something that happened, for the event log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// When.
    pub at_ms: u64,
    /// A short label: `phase`, `gate`, `commit`, `message`, `tool`, or any other.
    #[serde(default)]
    pub kind: String,
    /// What happened.
    #[serde(default)]
    pub text: String,
}

/// The tokens an agent's model used.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// For the current run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<TokenUse>,
    /// For the day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub today: Option<TokenUse>,
}

/// A count of tokens. The fields keep the contract's names on the wire (`input_tokens`,
/// `output_tokens`, `cache_read_tokens`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUse {
    /// Tokens sent to the model.
    #[serde(default, rename = "input_tokens")]
    pub input: u64,
    /// Tokens the model wrote.
    #[serde(default, rename = "output_tokens")]
    pub output: u64,
    /// Tokens read from the model's cache.
    #[serde(
        default,
        rename = "cache_read_tokens",
        skip_serializing_if = "Option::is_none"
    )]
    pub cache_read: Option<u64>,
}

/// A question an agent waits on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    /// Its id.
    #[serde(default)]
    pub id: String,
    /// What it asks.
    pub prompt: String,
    /// The answers it offers, if it offers any.
    #[serde(default)]
    pub options: Vec<String>,
    /// When it was asked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked_ms: Option<u64>,
}

/// What changed since a cursor.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changes {
    /// Where the feed continues from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Whether the store lost the cursor, so the list is read again.
    #[serde(default)]
    pub reset: bool,
    /// What changed.
    #[serde(default)]
    pub changed: Vec<Changed>,
}

/// One changed record, by kind and id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changed {
    /// `agent` or `run`.
    pub kind: String,
    /// Its id.
    pub id: String,
}
