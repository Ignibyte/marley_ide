//! `marley_sdk`: the contracts Marley reads to show agents, their work and their hosts
//! (`docs/marley/fleet-contract.md`, plan D20).
//!
//! A provider sends data in these shapes and Marley draws it; no provider sends UI.
//! - [`work`]: `marley.work/v1`, from a central workflow store: agents, work items, runs through
//!   ordered phases, gates, events, questions and token use.
//! - [`host`]: `marley.host/v1`, from a collector on a host: its resources and the agent
//!   processes running there.
//! - [`stale`]: when Marley reads an agent as stale.
//! - [`pseudo`]: a provider that serves the contract's own examples, moved by the clock it is
//!   given, for building and showing the fleet before a real store answers.
//!
//! Pure: serde types and functions of their inputs, with no clock, no IO and no gpui. Readers
//! ignore fields they do not know and take a missing optional field as absent, so a provider may
//! send more than this crate names.

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

pub mod host;
pub mod pseudo;
pub mod stale;
pub mod work;

pub use host::{AgentProcess, Cpu, Disk, HostInfo, HostSnapshot, Memory, Network};
pub use pseudo::{Pseudo, PseudoError, PseudoState};
pub use stale::is_stale;
pub use work::{
    AgentDetail, AgentInfo, AgentList, AgentSummary, Attention, Changed, Changes, Event, Gate,
    GateState, Handshake, Outcome, PhaseRun, PhaseState, PhaseSummary, Provider, Question, Run,
    State, TokenUse, Usage, WorkItem, WorkflowRef,
};
