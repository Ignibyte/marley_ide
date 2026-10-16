//! The TCP ports something on this machine listens on, and who listens.
//!
//! The ports come from Linux's `/proc/net/tcp` and `tcp6` (#503), and the processes of this
//! user's that listen on them from each process's `fd` links (#521). A terminal offers a dev
//! server's printed URL only while its port listens, and the rail lists each project's
//! listeners, each with the systemd service it runs in, when it runs in one (#603).

use std::collections::{BTreeSet, HashMap};
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use procfs_core::net::{TcpNetEntries, TcpNetEntry, TcpState};
use procfs_core::process::FDTarget;
use procfs_core::{ExplicitSystemInfo, FromReadSI as _};

/// The ports something listens on in `dir`'s `tcp` and `tcp6` tables, `/proc/net` on the machine.
///
/// Only the sockets a local URL reaches count: those bound to a loopback address, or to the
/// unspecified one of either family.
///
/// # Errors
///
/// When neither table could be read.
pub fn listening_ports_in(dir: &Path) -> Result<BTreeSet<u16>> {
    Ok(listening_entries_in(dir)
        .context("the listening ports")?
        .iter()
        .filter(|entry| reachable(entry.local_address))
        .map(|entry| entry.local_address.port())
        .collect())
}

/// The entries of `dir`'s `tcp` and `tcp6` tables that listen.
fn listening_entries_in(dir: &Path) -> Result<Vec<TcpNetEntry>> {
    // The TCP tables use none of the values but the byte order.
    let system = ExplicitSystemInfo {
        boot_time_secs: 0,
        ticks_per_second: 0,
        page_size: 0,
        is_little_endian: cfg!(target_endian = "little"),
    };
    let mut listening = Vec::new();
    let mut failure = None;
    let mut read = false;
    for table in ["tcp", "tcp6"] {
        let path = dir.join(table);
        match TcpNetEntries::from_file(&path, &system) {
            Ok(entries) => {
                read = true;
                listening.extend(
                    entries
                        .0
                        .into_iter()
                        .filter(|entry| entry.state == TcpState::Listen),
                );
            }
            // A machine without IPv6 has no `tcp6`.
            Err(error) => {
                failure =
                    Some(anyhow::Error::new(error).context(format!("reading {}", path.display())));
            }
        }
    }
    match failure {
        Some(failure) if !read => Err(failure),
        _ => Ok(listening),
    }
}

/// A TCP listener of one of this user's processes (#521).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listener {
    /// The address and port it is bound to.
    pub address: SocketAddr,
    /// The process that holds the socket.
    pub pid: u32,
    /// The process's name, as its `comm` gives it.
    pub name: String,
    /// Its command line, the arguments joined with spaces.
    pub command: String,
    /// Its working directory.
    pub cwd: PathBuf,
    /// The systemd service it runs in, when it runs in one (#603).
    pub service: Option<Service>,
}

/// A systemd service a process runs in (#603).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// The unit's name, as `app.service`.
    pub unit: String,
    /// Whether the user's own manager runs it, rather than the system's.
    pub user: bool,
}

/// The service a process runs in, from its `cgroup` file's text.
///
/// The unified hierarchy's line, `0::<path>`, names it when the path's last part is a `.service`
/// unit. A unit under the user's manager (`user@<uid>.service`) is the user's; any other the
/// system's. A scope, such as a desktop app's or a terminal's, is no service, and neither is a
/// process in a scope under a service's slice.
#[must_use]
pub fn service_of(cgroup: &str) -> Option<Service> {
    let path = cgroup.lines().find_map(|line| line.strip_prefix("0::"))?;
    let mut parts = path.trim().split('/').filter(|part| !part.is_empty());
    let unit = parts.next_back()?;
    if !unit.ends_with(".service") {
        return None;
    }
    let user = parts.any(|part| part.starts_with("user@") && part.ends_with(".service"));
    Some(Service {
        unit: unit.to_string(),
        user,
    })
}

/// The service the process reading `proc_root` runs in: `self`'s.
#[must_use]
pub fn own_service_in(proc_root: &Path) -> Option<Service> {
    std::fs::read_to_string(proc_root.join("self").join("cgroup"))
        .ok()
        .and_then(|cgroup| service_of(&cgroup))
}

/// How [`stop_in`] went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stopped {
    /// SIGTERM went to the process.
    Sent,
    /// The process ended before the signal reached it.
    Gone,
    /// The process no longer listens on the port, so no signal went.
    NotListening,
}

/// The listeners of this user's processes under `proc_root`, `/proc` on the machine.
///
/// Each is a listening socket of `net/tcp` and `net/tcp6` whose inode a readable `fd` link names,
/// with that process's name, command line and working directory, one per port and process, by
/// port.
///
/// A process whose folder cannot be read, another user's or one that ended during the walk, is
/// left out, and so is one whose working directory cannot be read, since it places the listener.
///
/// # Errors
///
/// When neither table, or `proc_root` itself, could be read.
pub fn listeners_in(proc_root: &Path) -> Result<Vec<Listener>> {
    let by_inode: HashMap<u64, SocketAddr> = listening_entries_in(&proc_root.join("net"))
        .context("the listening sockets")?
        .into_iter()
        .filter(|entry| entry.inode != 0)
        .map(|entry| (entry.inode, entry.local_address))
        .collect();
    let mut listeners = Vec::new();
    if by_inode.is_empty() {
        return Ok(listeners);
    }
    let processes =
        std::fs::read_dir(proc_root).with_context(|| format!("reading {}", proc_root.display()))?;
    for process in processes.flatten() {
        let Some(pid) = process
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        let folder = process.path();
        let Ok(descriptors) = std::fs::read_dir(folder.join("fd")) else {
            continue;
        };
        let mut addresses: Vec<SocketAddr> = descriptors
            .flatten()
            .filter_map(|descriptor| std::fs::read_link(descriptor.path()).ok())
            .filter_map(|target| match target.to_str()?.parse::<FDTarget>().ok()? {
                FDTarget::Socket(inode) => by_inode.get(&inode).copied(),
                _ => None,
            })
            .collect();
        if addresses.is_empty() {
            continue;
        }
        let Ok(cwd) = std::fs::read_link(folder.join("cwd")) else {
            continue;
        };
        let name = std::fs::read_to_string(folder.join("comm"))
            .map(|name| name.trim_end().to_string())
            .unwrap_or_default();
        let command = std::fs::read(folder.join("cmdline"))
            .map(|arguments| command_line(&arguments))
            .unwrap_or_default();
        let service = std::fs::read_to_string(folder.join("cgroup"))
            .ok()
            .and_then(|cgroup| service_of(&cgroup));
        // One listener per port: a server bound to two addresses of a port answers on both, and
        // the order keeps the first the same from scan to scan.
        addresses.sort_by_key(|address| (address.port(), *address));
        addresses.dedup_by_key(|address| address.port());
        listeners.extend(addresses.into_iter().map(|address| Listener {
            address,
            pid,
            name: name.clone(),
            command: command.clone(),
            cwd: cwd.clone(),
            service: service.clone(),
        }));
    }
    listeners.sort_by_key(|listener| (listener.address.port(), listener.pid));
    Ok(listeners)
}

/// A `cmdline` file's arguments, joined with spaces.
fn command_line(arguments: &[u8]) -> String {
    arguments
        .split(|byte| *byte == 0)
        .filter(|argument| !argument.is_empty())
        .map(String::from_utf8_lossy)
        .collect::<Vec<_>>()
        .join(" ")
}

/// The URL a listener on `address` answers (#521): an unspecified address opens as the loopback
/// of its family, any other as it is bound, and ports 443 and 8443 as `https`.
#[must_use]
pub fn url(address: SocketAddr) -> String {
    let ip = match address.ip() {
        IpAddr::V6(ip) => ip.to_ipv4_mapped().map_or(IpAddr::V6(ip), IpAddr::V4),
        ip @ IpAddr::V4(_) => ip,
    };
    let host = match ip {
        IpAddr::V4(ip) if ip.is_unspecified() => "127.0.0.1".to_string(),
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) if ip.is_unspecified() => "[::1]".to_string(),
        IpAddr::V6(ip) => format!("[{ip}]"),
    };
    let scheme = if matches!(address.port(), 443 | 8443) {
        "https"
    } else {
        "http"
    };
    format!("{scheme}://{host}:{}/", address.port())
}

/// Stops the process `pid` that listens on `port` under `proc_root` with SIGTERM (#521).
///
/// The signal goes only when a fresh scan finds the process listening there still, so a pid
/// given to another process since the last scan is left alone.
///
/// # Errors
///
/// When the scan fails, or the signal fails for a reason other than the process being gone.
pub fn stop_in(proc_root: &Path, port: u16, pid: u32) -> Result<Stopped> {
    stop_among(&listeners_in(proc_root)?, port, pid)
}

/// Stops `pid` with SIGTERM when `listeners`, a fresh scan, has it listening on `port`: the half
/// of [`stop_in`] that macOS shares, with its listeners from `lsof` (#788).
///
/// # Errors
///
/// When the signal fails for a reason other than the process being gone.
pub fn stop_among(listeners: &[Listener], port: u16, pid: u32) -> Result<Stopped> {
    let listening = listeners
        .iter()
        .any(|listener| listener.pid == pid && listener.address.port() == port);
    let process = i32::try_from(pid)
        .ok()
        .and_then(rustix::process::Pid::from_raw);
    let (true, Some(process)) = (listening, process) else {
        return Ok(Stopped::NotListening);
    };
    match rustix::process::kill_process(process, rustix::process::Signal::TERM) {
        Ok(()) => Ok(Stopped::Sent),
        Err(error) if error == rustix::io::Errno::SRCH => Ok(Stopped::Gone),
        Err(error) => Err(anyhow::Error::new(error).context(format!("stopping process {pid}"))),
    }
}

/// One listening socket in `lsof -nP -iTCP -sTCP:LISTEN -F pctn`'s output: the process and the
/// address it is bound to (#788).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LsofListening {
    /// The process that holds the socket.
    pub pid: u32,
    /// The process's name, as `lsof` gives it.
    pub name: String,
    /// The address and port it is bound to.
    pub address: SocketAddr,
}

/// The listening sockets in `lsof -nP -iTCP -sTCP:LISTEN -F pctn`'s output (#788).
///
/// It is macOS's stand-in for `/proc/net/tcp`. Each `p` line starts a process, `c` names it, and
/// each descriptor's `t` gives its family and `n` its `address:port`, `*` for any address of that
/// family. A line that does not parse is skipped.
#[must_use]
pub fn lsof_listening(text: &str) -> Vec<LsofListening> {
    let mut sockets = Vec::new();
    let mut pid = None;
    let mut name = String::new();
    let mut ipv6 = false;
    for line in text.lines() {
        let Some(field) = line.chars().next() else {
            continue;
        };
        let value = &line[field.len_utf8()..];
        match field {
            'p' => {
                pid = value.parse::<u32>().ok();
                name.clear();
            }
            'c' => name = value.to_string(),
            'f' => ipv6 = false,
            't' => ipv6 = value == "IPv6",
            'n' => {
                let Some(pid) = pid else { continue };
                if let Some(address) = lsof_address(value, ipv6) {
                    sockets.push(LsofListening {
                        pid,
                        name: name.clone(),
                        address,
                    });
                }
            }
            _ => {}
        }
    }
    sockets
}

/// An `lsof` socket name, `127.0.0.1:8080`, `[::1]:8080` or `*:8080`, as an address.
fn lsof_address(name: &str, ipv6: bool) -> Option<SocketAddr> {
    if let Some(port) = name.strip_prefix("*:") {
        let ip = if ipv6 {
            IpAddr::from(std::net::Ipv6Addr::UNSPECIFIED)
        } else {
            IpAddr::from(std::net::Ipv4Addr::UNSPECIFIED)
        };
        return Some(SocketAddr::new(ip, port.parse().ok()?));
    }
    name.parse().ok()
}

/// Each process's working directory in `lsof -nP -a -d cwd -p <pids> -F np`'s output (#788).
#[must_use]
pub fn lsof_folders(text: &str) -> HashMap<u32, PathBuf> {
    let mut folders = HashMap::new();
    let mut pid = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix('p') {
            pid = value.parse::<u32>().ok();
        } else if let (Some(value), Some(pid)) = (line.strip_prefix('n'), pid) {
            folders.insert(pid, PathBuf::from(value));
        }
    }
    folders
}

/// Each process's command line in `ps -ww -o pid=,command= -p <pids>`'s output (#788).
#[must_use]
pub fn ps_commands(text: &str) -> HashMap<u32, String> {
    text.lines()
        .filter_map(|line| {
            let (pid, command) = line.trim_start().split_once(char::is_whitespace)?;
            Some((pid.parse::<u32>().ok()?, command.trim().to_string()))
        })
        .collect()
}

/// The listeners `lsof` and `ps` describe, as [`listeners_in`] gives them from `/proc` (#788).
///
/// One per port of each process, sorted by port and pid. A process whose working directory `lsof`
/// could not read is left out, as `listeners_in` leaves out one whose `cwd` link it cannot read;
/// macOS has no systemd, so none runs in a service.
#[must_use]
pub fn listeners_from<S: std::hash::BuildHasher>(
    listening: &[LsofListening],
    folders: &HashMap<u32, PathBuf, S>,
    commands: &HashMap<u32, String, S>,
) -> Vec<Listener> {
    let mut sockets: Vec<&LsofListening> = listening.iter().collect();
    sockets.sort_by_key(|socket| (socket.pid, socket.address.port(), socket.address));
    sockets.dedup_by_key(|socket| (socket.pid, socket.address.port()));
    let mut listeners: Vec<Listener> = sockets
        .into_iter()
        .filter_map(|socket| {
            Some(Listener {
                address: socket.address,
                pid: socket.pid,
                name: socket.name.clone(),
                command: commands.get(&socket.pid).cloned().unwrap_or_default(),
                cwd: folders.get(&socket.pid)?.clone(),
                service: None,
            })
        })
        .collect();
    listeners.sort_by_key(|listener| (listener.address.port(), listener.pid));
    listeners
}

/// The ports in `listening` that a local URL reaches, as [`listening_ports_in`] counts them.
#[must_use]
pub fn reachable_ports(listening: &[LsofListening]) -> BTreeSet<u16> {
    listening
        .iter()
        .filter(|socket| reachable(socket.address))
        .map(|socket| socket.address.port())
        .collect()
}

/// Whether a socket bound to `address` answers a connection to this machine's loopback.
fn reachable(address: SocketAddr) -> bool {
    match address.ip() {
        IpAddr::V4(address) => address.is_loopback() || address.is_unspecified(),
        IpAddr::V6(address) => {
            address.is_loopback()
                || address.is_unspecified()
                || address
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| mapped.is_loopback() || mapped.is_unspecified())
        }
    }
}
