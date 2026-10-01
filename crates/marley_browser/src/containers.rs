//! Container ports (#614): the ports a Docker or Podman container publishes on this machine.
//!
//! Docker's published ports belong to root's `docker-proxy`, whose sockets a user's scan cannot
//! tie to a process, but whose command line anyone can read: it names the host's port and the
//! container's address and port. Rootless Podman's helpers run as the user, so the ports scan
//! finds their sockets; their names mark them as a container's. Which container a port belongs to,
//! and its Compose folder, only the engine can say: [`parse_docker_ps`] and [`parse_podman_ps`]
//! read what `docker ps` and `podman ps` print.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The processes that hold a container's published port, not a server of their own: Docker's
/// proxy and rootless Podman's port forwarders.
pub const HELPERS: [&str; 6] = [
    "docker-proxy",
    "rootlessport",
    "rootlessport-child",
    "pasta",
    "pasta.avx2",
    "slirp4netns",
];

/// The container engines' own systemd units: stopping one would stop every container, so a port
/// is never stopped through them.
pub const ENGINE_UNITS: [&str; 3] = ["docker.service", "podman.service", "containerd.service"];

/// The Compose label that names the folder a container's project was started in.
const COMPOSE_FOLDER: &str = "com.docker.compose.project.working_dir";

/// A container engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Engine {
    /// Docker, whose published ports root's `docker-proxy` holds.
    Docker,
    /// Podman, whose rootless port forwarders run as the user.
    Podman,
}

impl Engine {
    /// The engine's command.
    #[must_use]
    pub const fn command(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
        }
    }

    /// The engine that runs a helper process named `name`.
    #[must_use]
    pub fn of_helper(name: &str) -> Option<Self> {
        match name {
            "docker-proxy" => Some(Self::Docker),
            name if HELPERS.contains(&name) => Some(Self::Podman),
            _ => None,
        }
    }
}

/// A port `docker-proxy` publishes, from its command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxiedPort {
    /// The proxy's process.
    pub pid: u32,
    /// The host's address and port.
    pub address: SocketAddr,
    /// The container's address and port, as `172.17.0.2:80`.
    pub target: String,
}

/// The ports every `docker-proxy` under `proc_root` publishes over TCP.
///
/// A process whose command line cannot be read, or is not a proxy's, is passed by.
#[must_use]
pub fn proxied_ports_in(proc_root: &Path) -> Vec<ProxiedPort> {
    let Ok(entries) = std::fs::read_dir(proc_root) else {
        return Vec::new();
    };
    let mut proxied: Vec<ProxiedPort> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let pid: u32 = entry.file_name().to_str()?.parse().ok()?;
            let command = std::fs::read(entry.path().join("cmdline")).ok()?;
            proxied_port(pid, &String::from_utf8_lossy(&command))
        })
        .collect();
    proxied.sort_by_key(|port| (port.address.port(), port.pid));
    proxied
}

/// The port a `docker-proxy` command line publishes, its words split on NULs (as the kernel keeps
/// them) or spaces (as a process that rewrote its own name leaves them).
fn proxied_port(pid: u32, command: &str) -> Option<ProxiedPort> {
    let words: Vec<&str> = command
        .split(['\0', ' '])
        .filter(|word| !word.is_empty())
        .collect();
    let program = words.first()?.rsplit('/').next()?;
    if program != "docker-proxy" {
        return None;
    }
    let value = |flag: &str| {
        words
            .iter()
            .position(|word| *word == flag)
            .and_then(|at| words.get(at + 1))
            .copied()
    };
    if value("-proto").is_some_and(|proto| proto != "tcp") {
        return None;
    }
    let port: u16 = value("-host-port")?.parse().ok()?;
    let ip: IpAddr = value("-host-ip")
        .and_then(|ip| ip.parse().ok())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let target = format!(
        "{}:{}",
        value("-container-ip").unwrap_or("?"),
        value("-container-port").unwrap_or("?")
    );
    Some(ProxiedPort {
        pid,
        address: SocketAddr::new(ip, port),
        target,
    })
}

/// A container, as its engine lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// Its id.
    pub id: String,
    /// Its name.
    pub name: String,
    /// The engine that runs it.
    pub engine: Engine,
    /// The host's ports it publishes.
    pub host_ports: Vec<u16>,
    /// The folder its Compose project was started in, when Compose started it.
    pub working_dir: Option<PathBuf>,
}

/// One line of `docker ps --format '{{json .}}'`.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DockerLine {
    #[serde(rename = "ID")]
    id: String,
    names: String,
    #[serde(default)]
    ports: String,
    #[serde(default)]
    labels: String,
}

/// The containers in `docker ps --format '{{json .}}'`'s output, one JSON object a line; a line
/// that does not parse is passed by.
#[must_use]
pub fn parse_docker_ps(output: &str) -> Vec<Container> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<DockerLine>(line).ok())
        .map(|line| {
            let labels: HashMap<&str, &str> = line
                .labels
                .split(',')
                .filter_map(|label| label.split_once('='))
                .collect();
            Container {
                id: line.id,
                name: line.names.split(',').next().unwrap_or_default().to_string(),
                engine: Engine::Docker,
                host_ports: docker_host_ports(&line.ports),
                working_dir: labels.get(COMPOSE_FOLDER).map(PathBuf::from),
            }
        })
        .collect()
}

/// The host's ports in Docker's `Ports` column, as `0.0.0.0:8081->80/tcp, [::]:8081->80/tcp`.
fn docker_host_ports(ports: &str) -> Vec<u16> {
    let mut found: Vec<u16> = ports
        .split(',')
        .filter_map(|mapping| {
            let (host, _) = mapping.trim().split_once("->")?;
            host.rsplit(':').next()?.parse().ok()
        })
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// One container of `podman ps --format json`.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PodmanContainer {
    id: String,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    ports: Option<Vec<PodmanPort>>,
    #[serde(default)]
    labels: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct PodmanPort {
    host_port: u16,
}

/// The containers in `podman ps --format json`'s output, a JSON array; output that does not parse
/// lists none.
#[must_use]
pub fn parse_podman_ps(output: &str) -> Vec<Container> {
    serde_json::from_str::<Vec<PodmanContainer>>(output)
        .unwrap_or_default()
        .into_iter()
        .map(|container| {
            let mut host_ports: Vec<u16> = container
                .ports
                .unwrap_or_default()
                .iter()
                .map(|port| port.host_port)
                .collect();
            host_ports.sort_unstable();
            host_ports.dedup();
            Container {
                name: container.names.first().cloned().unwrap_or_default(),
                id: container.id,
                engine: Engine::Podman,
                host_ports,
                working_dir: container
                    .labels
                    .and_then(|labels| labels.get(COMPOSE_FOLDER).map(PathBuf::from)),
            }
        })
        .collect()
}
