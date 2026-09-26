//! The TCP ports something on this machine listens on, read from Linux's `/proc/net/tcp` and
//! `tcp6` (#503). A terminal offers a dev server's printed URL only while its port listens.

use std::collections::BTreeSet;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;

use anyhow::{Context as _, Result};
use procfs_core::net::{TcpNetEntries, TcpState};
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
    // The TCP tables use none of the values but the byte order.
    let system = ExplicitSystemInfo {
        boot_time_secs: 0,
        ticks_per_second: 0,
        page_size: 0,
        is_little_endian: cfg!(target_endian = "little"),
    };
    let mut ports = BTreeSet::new();
    let mut failure = None;
    let mut read = false;
    for table in ["tcp", "tcp6"] {
        let path = dir.join(table);
        match TcpNetEntries::from_file(&path, &system) {
            Ok(entries) => {
                read = true;
                ports.extend(
                    entries
                        .0
                        .iter()
                        .filter(|entry| {
                            entry.state == TcpState::Listen && reachable(entry.local_address)
                        })
                        .map(|entry| entry.local_address.port()),
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
        _ => Ok(ports),
    }
    .context("the listening ports")
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
