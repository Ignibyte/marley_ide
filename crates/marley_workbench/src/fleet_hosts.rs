//! The host collector (#610, `docs/marley/fleet-contract.md`): Marley's own script, run on each
//! host the settings list, over SSH or on this machine.
//!
//! The script travels in the SSH command, base64-encoded, as #526's bootstrap does, so nothing is
//! installed and nothing is written to a stdin. `collect` runs it on every listed host; `join`
//! puts what it found beside the workflow stores' agents: a store's host gets the collector's
//! snapshot, a process the store's agent claims is kept with that agent, and the rest become a
//! Hosts source of their own.

use base64::Engine as _;
use marley_sdk::{
    AgentDetail, AgentInfo, AgentProcess, AgentSummary, Attention, HostSnapshot, State,
};
use settings::FleetHostContent;

use crate::fleet::{Source, Unreachable};
use crate::process;

/// The collector script, as it ships.
const SCRIPT: &str = include_str!("../bin/marley-collect.sh");

/// The name of the source the collected hosts and their unclaimed processes show under.
pub(crate) const HOSTS_SOURCE: &str = "Hosts";

/// How long SSH waits for a host before the collection gives up on it.
const CONNECT_TIMEOUT_S: u32 = 5;

/// How long an idle SSH connection to a host is kept for the next collection.
const CONTROL_PERSIST_S: u32 = 60;

/// A host the settings list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetHost {
    /// Where the script runs.
    pub target: HostTarget,
    /// The name the panel shows.
    pub name: String,
    /// The id a store gives the host, when the settings name one.
    pub id: Option<String>,
}

/// Where a host's collector runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostTarget {
    /// This machine, under `sh`.
    Local,
    /// An SSH destination, as the settings wrote it; it is checked before anything runs.
    Ssh(String),
}

impl FleetHost {
    /// The host a settings entry names, or `None` for an entry with neither `ssh` nor `local`.
    #[must_use]
    pub fn from_settings(content: &FleetHostContent) -> Option<Self> {
        let target = match (&content.ssh, content.local) {
            (Some(destination), _) => HostTarget::Ssh(destination.clone()),
            (None, Some(true)) => HostTarget::Local,
            _ => return None,
        };
        let name = content.name.clone().unwrap_or_else(|| match &target {
            HostTarget::Local => "This machine".to_string(),
            HostTarget::Ssh(destination) => destination.clone(),
        });
        Some(Self {
            target,
            name,
            id: content.id.clone(),
        })
    }
}

/// What one collection found on a host.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Collected {
    pub(crate) name: String,
    /// The settings' id, else the host's own name, else the settings' name.
    pub(crate) id: String,
    pub(crate) snapshot: Option<HostSnapshot>,
    /// Why there is no snapshot: the host did not answer, or was refused.
    pub(crate) problem: Option<String>,
}

impl Collected {
    fn failed(host: &FleetHost, problem: String) -> Self {
        Self {
            name: host.name.clone(),
            id: host.id.clone().unwrap_or_else(|| host.name.clone()),
            snapshot: None,
            problem: Some(problem),
        }
    }

    /// Whether the host was refused before anything ran.
    pub(crate) fn refused(&self) -> bool {
        self.problem
            .as_deref()
            .is_some_and(|problem| problem.starts_with("refused"))
    }
}

/// Runs the collector on each host, one after another, and gives what each answered.
pub(crate) async fn collect(hosts: Vec<FleetHost>, names: &[String]) -> Vec<Collected> {
    // The names reach a shell on the host, so only plain ones go.
    let names = names
        .iter()
        .filter(|name| {
            !name.is_empty()
                && name.len() <= 32
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        })
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let mut collected = Vec::with_capacity(hosts.len());
    for host in hosts {
        collected.push(collect_one(&host, &names).await);
    }
    collected
}

/// Runs the collector on one host.
async fn collect_one(host: &FleetHost, names: &str) -> Collected {
    let output = match &host.target {
        HostTarget::Local => {
            process::output(
                "sh",
                ["-c", SCRIPT],
                None,
                &[("MARLEY_COLLECT_NAMES", names)],
            )
            .await
        }
        HostTarget::Ssh(destination) => {
            let Some(target) = marley_remote::parse_ssh_target(destination) else {
                return Collected::failed(
                    host,
                    format!("refused: {destination:?} is not an SSH destination"),
                );
            };
            // A scenario's own `ssh` must not lose to the real one on the login shell's PATH.
            let ssh = std::env::var("MARLEY_SSH").unwrap_or_else(|_| "ssh".to_string());
            process::output(ssh, ssh_args(&target, names), None, &[]).await
        }
    };
    let output = match output {
        Ok(output) => output,
        Err(error) => return Collected::failed(host, format!("could not start: {error}")),
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = stderr
            .lines()
            .rev()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map_or_else(
                || format!("the collector ended with {}", output.status),
                str::to_string,
            );
        return Collected::failed(host, reason);
    }
    match serde_json::from_slice::<HostSnapshot>(&output.stdout) {
        Ok(snapshot) => Collected {
            name: host.name.clone(),
            id: host.id.clone().unwrap_or_else(|| snapshot.host.id.clone()),
            snapshot: Some(snapshot),
            problem: None,
        },
        Err(error) => Collected::failed(host, format!("its answer did not parse: {error}")),
    }
}

/// `ssh`'s arguments for one collection on `target`: no terminal, no password prompt, a short
/// connect timeout, one connection kept for the next collection, and the script in the command.
fn ssh_args(target: &marley_remote::SshTarget, names: &str) -> Vec<String> {
    let mut args: Vec<String> = [
        "-T",
        "-o",
        "BatchMode=yes",
        "-o",
        &format!("ConnectTimeout={CONNECT_TIMEOUT_S}"),
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    // The runtime folder is the user's own, mode 0700, so the control socket needs no folder of
    // its own; without it the collection logs in each time.
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR")
        && !runtime.is_empty()
    {
        args.extend([
            "-o".to_string(),
            "ControlMaster=auto".to_string(),
            "-o".to_string(),
            format!("ControlPath={runtime}/marley-ssh-%C"),
            "-o".to_string(),
            format!("ControlPersist={CONTROL_PERSIST_S}"),
        ]);
    }
    // `ssh_command` puts `--` before the destination, and the port before that.
    args.extend(marley_remote::ssh_command(target).into_iter().skip(1));
    let script = base64::engine::general_purpose::STANDARD.encode(SCRIPT);
    // `env` sets the variable in any login shell, csh's included (#584).
    args.push(format!(
        "printf %s {script} | base64 -d | env MARLEY_COLLECT_NAMES='{names}' sh"
    ));
    args
}

/// Puts `collected` beside the stores' `sources`, and gives the Hosts source for what no store
/// claims, or `None` when no host is listed.
///
/// A store's host of the same id takes the collector's snapshot; a host that did not answer marks
/// that store's agents on it offline. A process is claimed by the store agent whose id is its
/// `session`, else by one on its host with its runtime and folder (when the agent's detail is
/// known), else by the one agent and the one process on the host with that runtime.
pub(crate) fn join(sources: &mut [Source], collected: &[Collected], now: u64) -> Option<Source> {
    if collected.is_empty() {
        return None;
    }
    let mut hosts = Source::for_hosts(HOSTS_SOURCE, now);
    for host in collected {
        let Some(snapshot) = &host.snapshot else {
            let unreachable = Unreachable {
                id: host.id.clone(),
                name: host.name.clone(),
                reason: host.problem.clone().unwrap_or_default(),
            };
            for source in sources.iter_mut() {
                if source
                    .agents
                    .iter()
                    .any(|agent| agent.host_id.as_deref() == Some(host.id.as_str()))
                {
                    source.unreachable.push(unreachable.clone());
                }
            }
            hosts.unreachable.push(unreachable);
            continue;
        };
        let mut snapshot = snapshot.clone();
        // The panel names the host as the settings do, and knows it by the store's id.
        snapshot.host.id.clone_from(&host.id);
        snapshot.host.name.clone_from(&host.name);
        for source in sources.iter_mut() {
            if source
                .agents
                .iter()
                .any(|agent| agent.host_id.as_deref() == Some(host.id.as_str()))
            {
                match source
                    .hosts
                    .iter_mut()
                    .find(|known| known.host.id == host.id)
                {
                    Some(known) => known.clone_from(&snapshot),
                    None => source.hosts.push(snapshot.clone()),
                }
            }
        }
        let mut unclaimed = Vec::new();
        for process in &snapshot.agents {
            if !claim(sources, &host.id, process, &snapshot.agents) {
                unclaimed.push(process.clone());
            }
        }
        for process in unclaimed {
            let (summary, detail) = process_agent(&process, &snapshot);
            hosts.processes.push((summary.id.clone(), process));
            hosts.agents.push(summary);
            hosts.details.push(detail);
        }
        hosts.hosts.push(snapshot);
    }
    Some(hosts)
}

/// Gives `process` to the store agent that claims it, if one does, and says whether one did.
fn claim(
    sources: &mut [Source],
    host_id: &str,
    process: &AgentProcess,
    on_host: &[AgentProcess],
) -> bool {
    let by_session = |source: &Source| {
        process.session.as_ref().and_then(|session| {
            source
                .agents
                .iter()
                .find(|agent| &agent.id == session)
                .map(|agent| agent.id.clone())
        })
    };
    let on_this_host = |agent: &&AgentSummary| {
        agent.host_id.as_deref() == Some(host_id) && agent.runtime == process.runtime
    };
    let by_folder = |source: &Source| {
        let folder = process.cwd.as_deref()?;
        source
            .agents
            .iter()
            .filter(on_this_host)
            .find(|agent| {
                source.details.iter().any(|detail| {
                    detail.agent.id == agent.id && detail.agent.cwd.as_deref() == Some(folder)
                })
            })
            .map(|agent| agent.id.clone())
    };
    let by_pair = |source: &Source| {
        let mut agents = source.agents.iter().filter(on_this_host);
        let agent = agents.next()?;
        let alike = on_host
            .iter()
            .filter(|other| other.runtime == process.runtime && other.session.is_none())
            .count();
        (agents.next().is_none() && alike == 1).then(|| agent.id.clone())
    };
    for source in sources.iter_mut() {
        let claimed = by_session(source)
            .or_else(|| by_folder(source))
            .or_else(|| by_pair(source));
        if let Some(agent) = claimed {
            source.processes.retain(|(id, _)| id != &agent);
            source.processes.push((agent, process.clone()));
            return true;
        }
    }
    false
}

/// An unclaimed process as an agent of the Hosts source: its folder's name, no work records.
fn process_agent(process: &AgentProcess, snapshot: &HostSnapshot) -> (AgentSummary, AgentDetail) {
    let id = format!("{}:{}", snapshot.host.id, process.pid);
    let name = process
        .cwd
        .as_deref()
        .and_then(|cwd| cwd.rsplit('/').find(|part| !part.is_empty()))
        .map_or_else(|| format!("pid {}", process.pid), str::to_string);
    let summary = AgentSummary {
        id: id.clone(),
        name: name.clone(),
        runtime: process.runtime.clone(),
        state: State::Working,
        state_since_ms: process.started_ms,
        last_seen_ms: Some(snapshot.sampled_ms),
        host_id: Some(snapshot.host.id.clone()),
        work_item: None,
        phase: None,
        attention: Attention::None,
        question: None,
    };
    let detail = AgentDetail {
        agent: AgentInfo {
            id,
            name,
            runtime: process.runtime.clone(),
            state: State::Working,
            state_since_ms: process.started_ms,
            last_seen_ms: Some(snapshot.sampled_ms),
            host_id: Some(snapshot.host.id.clone()),
            cwd: process.cwd.clone(),
            model: None,
        },
        work_item: None,
        run: None,
        recent_events: Vec::new(),
        usage: None,
        question: None,
        host: Some(snapshot.clone()),
    };
    (summary, detail)
}

/// A process's use in a few words: "pid 4121 · 18.5 % · 420 MB".
pub(crate) fn process_line(process: &AgentProcess) -> String {
    let mut parts = vec![format!("pid {}", process.pid)];
    if let Some(cpu) = process.cpu_percent {
        parts.push(format!("{cpu:.1} %"));
    }
    if let Some(rss) = process.rss_bytes {
        parts.push(format!("{} MB", rss / 1_000_000));
    }
    parts.join(" · ")
}
