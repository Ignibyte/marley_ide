//! Harnesses on other hosts (#740).
//!
//! `marley.harnesses` names each, and Marley follows each beside the harness it follows already
//! (`marley.harness`, or its own runtime), over its own SSH connection, as the harness settled
//! (its D164). Each is followed by `harness::follow` through its own `Slot`, and its sessions,
//! connection and fold live here, in the `Hosts` global.

use std::path::PathBuf;
use std::sync::Arc;

use context_server::{ContextServer, ContextServerCommand};
use gpui::{App, Global, SharedString, Task};
use marley_fleet::FleetSnapshot;
use marley_remote::SshTarget;
use settings::{MarleySettingsContent, SettingsStore};

use crate::harness::{Connection, Slot};

/// A harness `marley.harnesses` names: where it is and how its `rh` is reached.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HarnessHost {
    /// The name the rail shows it under.
    pub(crate) name: SharedString,
    /// Its host, or none for this machine.
    pub(crate) ssh: Option<SshTarget>,
    /// Its state root there.
    state: String,
    /// The `rh` to run there.
    rh: String,
}

impl HarnessHost {
    /// The entries with a name and a root, the first of each name. An `ssh` that names no
    /// destination, or one that starts with `-`, drops its entry with a log line.
    pub(crate) fn from_content(marley: Option<&MarleySettingsContent>) -> Vec<Self> {
        let content = marley.and_then(|marley| marley.harnesses.as_deref());
        let mut hosts: Vec<Self> = Vec::new();
        for entry in content.unwrap_or_default() {
            let (Some(name), Some(state)) = (&entry.name, &entry.state) else {
                continue;
            };
            let (name, state) = (name.trim(), state.trim());
            if name.is_empty() || state.is_empty() || hosts.iter().any(|host| host.name == name) {
                continue;
            }
            let ssh = match entry.ssh.as_deref().map(str::trim) {
                None | Some("") => None,
                Some(destination) => {
                    let Some(target) = marley_remote::parse_ssh_target(destination) else {
                        log::warn!("harness {name}: `{destination}` is no SSH destination");
                        continue;
                    };
                    Some(target)
                }
            };
            hosts.push(Self {
                name: SharedString::from(name.to_string()),
                ssh,
                state: state.to_string(),
                rh: entry
                    .rh
                    .as_deref()
                    .map(str::trim)
                    .filter(|rh| !rh.is_empty())
                    .unwrap_or("rh")
                    .to_string(),
            });
        }
        hosts
    }

    /// The program and the arguments that reach `rh` on the host, up to its subcommand: `rh
    /// --state ROOT` here, or `ssh … -- HOST rh --state ROOT` with the remote words quoted for
    /// the host's shell.
    pub(crate) fn base(&self) -> (PathBuf, Vec<String>) {
        self.ssh.as_ref().map_or_else(
            || {
                (
                    PathBuf::from(&self.rh),
                    vec!["--state".to_string(), self.state.clone()],
                )
            },
            |target| {
                let mut arguments = ssh_arguments(target, false);
                arguments.extend([
                    shell_word(&self.rh),
                    "--state".to_string(),
                    shell_word(&self.state),
                ]);
                (PathBuf::from("ssh"), arguments)
            },
        )
    }

    /// The command of the harness's MCP server, with the write grant when `writes` (#689).
    pub(crate) fn mcp_command(&self, writes: bool) -> ContextServerCommand {
        let (path, mut args) = self.base();
        args.push("mcp".to_string());
        if writes {
            args.extend(["--grant".to_string(), "write".to_string()]);
        }
        ContextServerCommand {
            path,
            args,
            env: None,
            timeout: None,
        }
    }
}

/// `ssh`'s arguments up to and including the destination: a terminal only when `tty`, never a
/// password prompt (`BatchMode`), Marley's keepalive (#641), the port, and `--` before the
/// destination so it can never be read as an option.
pub(crate) fn ssh_arguments(target: &SshTarget, tty: bool) -> Vec<String> {
    let mut arguments = vec![
        if tty { "-t" } else { "-T" }.to_string(),
        "-o".to_string(),
        "BatchMode=yes".to_string(),
    ];
    arguments.extend(marley_remote::keepalive_options());
    // `ssh_command` gives `ssh`, the port, `--` and the destination.
    arguments.extend(marley_remote::ssh_command(target).into_iter().skip(1));
    arguments
}

/// `word` as the remote shell reads it back: as it is when it holds only plain characters, else
/// in single quotes. ssh joins the words after the destination into one line for that shell.
pub(crate) fn shell_word(word: &str) -> String {
    let plain = !word.is_empty()
        && word
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_-./:@%+=,".contains(character));
    if plain {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

/// The harnesses on other hosts Marley follows, in the settings' order.
#[derive(Default)]
pub(crate) struct Hosts {
    pub(crate) followed: Vec<Followed>,
}

impl Global for Hosts {}

/// One harness followed: its sessions and connection, as the primary harness keeps its own.
pub(crate) struct Followed {
    pub(crate) host: HarnessHost,
    /// Whether it is followed with the write grant.
    writes: bool,
    pub(crate) connection: Connection,
    pub(crate) seats: FleetSnapshot,
    pub(crate) server: Option<Arc<ContextServer>>,
    /// Whether the rail folds its rows.
    pub(crate) folded: bool,
    /// Bumped each minute while a session works, so its `no update in N m` is drawn again.
    pub(crate) minute: u64,
    _run: Task<()>,
}

impl Hosts {
    /// The followed harness named `name`.
    pub(crate) fn get<'a>(name: &str, cx: &'a App) -> Option<&'a Followed> {
        cx.try_global::<Self>()?
            .followed
            .iter()
            .find(|followed| followed.host.name == name)
    }

    /// The followed harness named `name`, to change.
    pub(crate) fn get_mut<'a>(name: &str, cx: &'a mut App) -> Option<&'a mut Followed> {
        if !cx.has_global::<Self>() {
            return None;
        }
        cx.global_mut::<Self>()
            .followed
            .iter_mut()
            .find(|followed| followed.host.name == name)
    }

    /// The names of the harnesses followed, in order.
    pub(crate) fn names(cx: &App) -> Vec<SharedString> {
        cx.try_global::<Self>().map_or_else(Vec::new, |hosts| {
            hosts
                .followed
                .iter()
                .map(|followed| followed.host.name.clone())
                .collect()
        })
    }
}

/// Follows what `marley.harnesses` names: a harness whose entry and grant are unchanged keeps its
/// connection, a new or changed one starts again, and one no longer named is dropped with its
/// task. [`crate::harness`] calls it on every settings change.
pub(crate) fn follow_hosts(cx: &mut App) {
    // Read as `marley.harness_writes` is, from the merged settings.
    let wanted = HarnessHost::from_content(
        cx.global::<SettingsStore>()
            .merged_settings()
            .marley
            .as_ref(),
    );
    let writes = crate::harness::harness_writes(cx);
    let mut kept = std::mem::take(&mut cx.default_global::<Hosts>().followed);
    let mut followed = Vec::with_capacity(wanted.len());
    for host in wanted {
        if let Some(index) = kept
            .iter()
            .position(|old| old.host == host && old.writes == writes)
        {
            followed.push(kept.remove(index));
            continue;
        }
        let slot = Slot::Host(host.name.clone());
        let command = host.mcp_command(writes);
        let run = cx.spawn(async move |cx| crate::harness::follow(slot, command, cx).await);
        followed.push(Followed {
            host,
            writes,
            connection: Connection::Connecting,
            seats: FleetSnapshot::default(),
            server: None,
            folded: false,
            minute: 0,
            _run: run,
        });
    }
    cx.global_mut::<Hosts>().followed = followed;
    crate::harness::filter_palette(cx);
}
