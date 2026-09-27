//! The TCP ports something on this machine listens on, and who listens.
//!
//! The ports come from Linux's `/proc/net/tcp` and `tcp6` (#503), and the processes of this
//! user's that listen on them from each process's `fd` links (#521). A terminal offers a dev
//! server's printed URL only while its port listens, and the rail lists each project's
//! listeners.

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
    let listening = listeners_in(proc_root)?
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
