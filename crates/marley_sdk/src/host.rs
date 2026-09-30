//! `marley.host/v1`: what a host collector prints (`docs/marley/fleet-contract.md`).

use serde::{Deserialize, Serialize};

/// The `contract` name this version reads.
pub const HOST_CONTRACT: &str = "marley.host/v1";

/// One reading of a host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostSnapshot {
    /// The contract and its major version.
    #[serde(default)]
    pub contract: String,
    /// The host.
    pub host: HostInfo,
    /// When it was read.
    #[serde(default)]
    pub sampled_ms: u64,
    /// The processor's use over the last interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<Cpu>,
    /// The load averages over 1, 5 and 15 minutes.
    #[serde(default)]
    pub load: Vec<f64>,
    /// Its memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<Memory>,
    /// Its disks, by mount point.
    #[serde(default)]
    pub disks: Vec<Disk>,
    /// Its network over the last interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    /// The agent processes running there.
    #[serde(default)]
    pub agents: Vec<AgentProcess>,
}

/// A host, as a collector names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInfo {
    /// Its id, which a store's agents name as their `host_id`.
    pub id: String,
    /// The name shown.
    #[serde(default)]
    pub name: String,
    /// Its system, as `Linux 6.12`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// Its processor cores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cores: Option<u32>,
    /// How long it has run, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime_s: Option<u64>,
}

/// The processor's use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cpu {
    /// Its use, from 0 to 100.
    pub percent: f64,
}

/// Memory in use and in all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    /// Bytes in use.
    pub used_bytes: u64,
    /// Bytes in all.
    pub total_bytes: u64,
}

/// A disk, by where it is mounted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Disk {
    /// Its mount point.
    pub mount: String,
    /// Bytes in use.
    pub used_bytes: u64,
    /// Bytes in all.
    pub total_bytes: u64,
}

/// Network traffic over the last interval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    /// Bytes received per second.
    pub rx_bps: u64,
    /// Bytes sent per second.
    pub tx_bps: u64,
}

/// An agent process a collector recognized.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentProcess {
    /// Its process id.
    pub pid: u32,
    /// The runtime it is, as `claude-code`.
    #[serde(default)]
    pub runtime: String,
    /// The folder it works in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// When it started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_ms: Option<u64>,
    /// Its processor use, from 0 to 100 of one core.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f64>,
    /// Its resident memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rss_bytes: Option<u64>,
    /// An id the runtime exposes, which matches a store's agent id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
}
