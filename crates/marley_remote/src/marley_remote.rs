//! PURE — the remote (ssh) seam: parse a `[user@]host[:port]` target and build the `ssh` argv from it.
//!
//! Marley opens a remote pane by spawning the USER'S `ssh` client as a terminal (like Terminal.app) —
//! `ssh` owns ALL security (keys, `known_hosts`, auth, passwords). This crate handles NO secrets; it only
//! parses a typed target and produces an argv. The command is a `Vec<String>` (never a shell string), so
//! nothing can inject a shell command; and a leading-dash host/user is rejected (and the argv carries a
//! `--` before the destination) so it can never be re-parsed as an `ssh` OPTION (option-smuggling).

use serde::{Deserialize, Serialize};

/// A parsed ssh destination: `[user@]host[:port]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTarget {
    /// The login user, if given (`user@host`).
    pub user: Option<String>,
    /// The host (a name, an IPv4, or an IPv6 that was bracketed as `[::1]`).
    pub host: String,
    /// The port, if given (`host:port`).
    pub port: Option<u16>,
}

/// Parse a `[user@]host[:port]` target (including bracketed IPv6 like `[::1]:22`).
///
/// Returns `None` when the input is empty, the user before `@` is empty, the port is empty /
/// non-numeric / doesn't fit a `u16`, the host is empty, or the host is a BARE (unbracketed) IPv6 —
/// bracket IPv6 hosts as `[::1]`.
#[must_use]
pub fn parse_ssh_target(s: &str) -> Option<SshTarget> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (user, rest) = match s.split_once('@') {
        Some(("", _)) => return None, // "@host" — an empty user
        Some((u, r)) => (Some(u.to_string()), r),
        None => (None, s),
    };
    let (host, port) = parse_host_port(rest)?;
    // Reject an empty host, and a host OR user that starts with `-`: `ssh` would re-parse a leading-dash
    // argument as an OPTION (e.g. `-oProxyCommand=…`), not a destination — an option-smuggling injection
    // (the CVE-2017-1000117 class). `ssh_command` also guards this with a `--`, but rejecting up front
    // gives a clean failure and never even builds a dangerous target.
    if host.is_empty() || host.starts_with('-') {
        return None;
    }
    if user.as_deref().is_some_and(|u| u.starts_with('-')) {
        return None;
    }
    Some(SshTarget {
        user,
        host: host.to_string(),
        port,
    })
}

/// Split `host[:port]`, honoring bracketed IPv6 (`[host]` / `[host]:port`). A bare unbracketed IPv6 (a
/// host that still contains a `:` after the split) is rejected — use brackets.
fn parse_host_port(s: &str) -> Option<(&str, Option<u16>)> {
    if let Some(rest) = s.strip_prefix('[') {
        let (host, after) = rest.split_once(']')?;
        if after.is_empty() {
            Some((host, None))
        } else {
            let port = after.strip_prefix(':')?;
            Some((host, Some(parse_port(port)?)))
        }
    } else if let Some((host, port)) = s.rsplit_once(':') {
        if host.contains(':') {
            return None; // a bare (unbracketed) IPv6 — bracket it as [::1]
        }
        Some((host, Some(parse_port(port)?)))
    } else {
        Some((s, None))
    }
}

/// A non-empty port string that fits in a `u16`.
fn parse_port(s: &str) -> Option<u16> {
    if s.is_empty() { None } else { s.parse().ok() }
}

/// Build the `ssh` argv for `target`: `["ssh", ("-p", port)?, "--", [user@]host]`.
///
/// An argv (never a shell string) so nothing can inject a shell command; the `--` stops `ssh`
/// option parsing so a leading-dash destination (even on a hand-built [`SshTarget`] that skipped
/// [`parse_ssh_target`]) can NEVER be re-parsed as an `ssh` option (option-smuggling — the
/// CVE-2017-1000117 class).
#[must_use]
pub fn ssh_command(target: &SshTarget) -> Vec<String> {
    let mut argv = vec!["ssh".to_string()];
    if let Some(port) = target.port {
        argv.push("-p".to_string());
        argv.push(port.to_string());
    }
    argv.push("--".to_string());
    argv.push(target.user.as_ref().map_or_else(
        || target.host.clone(),
        |user| format!("{user}@{}", target.host),
    ));
    argv
}

/// Whether a remote (ssh) pane is still connected (#86).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteStatus {
    /// The `ssh` process is running.
    Connected,
    /// The `ssh` process has exited.
    Disconnected,
}

/// Project a remote pane's status from whether its `ssh` process has exited (#86) — an exited ssh
/// (network drop, remote logout, or a clean `exit`) reads as `Disconnected`.
#[must_use]
pub const fn remote_status_from(exited: bool) -> RemoteStatus {
    if exited {
        RemoteStatus::Disconnected
    } else {
        RemoteStatus::Connected
    }
}

/// The status glyph for a remote pane: `⇄` connected, `✗` disconnected (#86).
#[must_use]
pub const fn remote_status_glyph(status: RemoteStatus) -> &'static str {
    match status {
        RemoteStatus::Connected => "\u{21c4}",    // ⇄
        RemoteStatus::Disconnected => "\u{2717}", // ✗
    }
}

/// The pane badge for a remote (ssh) session (#85, #86): `{status-glyph} {host}` — `⇄ localhost`
/// while connected, `✗ localhost` once the ssh has exited.
///
/// Marks a pane as remote + names the host + shows the connection state, mirroring the agent badge.
#[must_use]
pub fn remote_badge(host: &str, status: RemoteStatus) -> String {
    format!("{} {host}", remote_status_glyph(status))
}

/// A named, saved ssh host (#87) — one entry of the `[[remote.hosts]]` TOML setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteHost {
    /// A human label for the host (shown in the palette).
    pub name: String,
    /// The ssh target string (`[user@]host[:port]`), validated by [`parse_ssh_target`] when used.
    pub target: String,
}

/// A command-palette "connect: …" action for a saved host (#87): the display label + the parsed target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAction {
    /// The palette entry label, e.g. `connect: prod → deploy@prod:22`.
    pub label: String,
    /// The parsed ssh target the action opens.
    pub target: SshTarget,
}

/// Build the command-palette connect actions from the saved `hosts` (#87).
///
/// Each host with a VALID target yields a `connect: {name} → {target}` action; a host whose target
/// fails [`parse_ssh_target`] (empty, leading-dash, bare IPv6, bad port, …) is DROPPED — a bad
/// config entry is skipped, never a panic.
#[must_use]
pub fn remote_palette_actions(hosts: &[RemoteHost]) -> Vec<RemoteAction> {
    hosts
        .iter()
        .filter_map(|host| {
            parse_ssh_target(&host.target).map(|target| RemoteAction {
                label: format!("connect: {} → {}", host.name, host.target),
                target,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(user: Option<&str>, host: &str, port: Option<u16>) -> SshTarget {
        SshTarget {
            user: user.map(str::to_string),
            host: host.to_string(),
            port,
        }
    }

    // REQ-001 — the happy split cases (outer whitespace trimmed)
    #[test]
    fn parse_targets_ok() {
        assert_eq!(parse_ssh_target("h"), Some(target(None, "h", None)));
        assert_eq!(parse_ssh_target("u@h"), Some(target(Some("u"), "h", None)));
        assert_eq!(parse_ssh_target("h:22"), Some(target(None, "h", Some(22))));
        assert_eq!(
            parse_ssh_target("u@h:2222"),
            Some(target(Some("u"), "h", Some(2222)))
        );
        assert_eq!(
            parse_ssh_target("  u@h  "),
            Some(target(Some("u"), "h", None)) // trimmed
        );
        assert_eq!(
            parse_ssh_target("1.2.3.4:22"),
            Some(target(None, "1.2.3.4", Some(22)))
        );
        assert_eq!(
            parse_ssh_target("u@h:0"),
            Some(target(Some("u"), "h", Some(0))) // port 0 is a valid u16 (ssh rejects it, not us)
        );
    }

    // REQ-002 — rejects → None
    #[test]
    fn parse_targets_reject() {
        for bad in [
            "", "   ", "u@", "@h", "@", "h:", "h:x", "h:99999", "h:-1", "::1", "u@::1",
        ] {
            assert_eq!(parse_ssh_target(bad), None, "expected None for {bad:?}");
        }
    }

    // REQ-003 — bracketed IPv6 in; malformed brackets out (incl. the S2 numeric-junk gap)
    #[test]
    fn parse_bracketed_ipv6() {
        assert_eq!(parse_ssh_target("[::1]"), Some(target(None, "::1", None)));
        assert_eq!(
            parse_ssh_target("[::1]:22"),
            Some(target(None, "::1", Some(22)))
        );
        assert_eq!(
            parse_ssh_target("u@[::1]:22"),
            Some(target(Some("u"), "::1", Some(22)))
        );
        assert_eq!(parse_ssh_target("[::1]x"), None); // trailing junk, no :port
        assert_eq!(parse_ssh_target("[::1]22"), None); // numeric junk, no ':' separator (S2)
        assert_eq!(parse_ssh_target("[]"), None); // empty host
        assert_eq!(parse_ssh_target("[::1]:"), None); // empty port
    }

    // S1 (security) — a leading-`-` host or user is rejected (ssh option-smuggling guard)
    #[test]
    fn parse_rejects_leading_dash() {
        assert_eq!(parse_ssh_target("-oProxyCommand=evil"), None); // dash host
        assert_eq!(parse_ssh_target("-oProxyCommand=evil:22"), None); // dash host + port
        assert_eq!(parse_ssh_target("-bad@host"), None); // dash user
    }

    // #86 — status projection + glyphs
    #[test]
    fn remote_status_from_maps() {
        assert_eq!(remote_status_from(true), RemoteStatus::Disconnected);
        assert_eq!(remote_status_from(false), RemoteStatus::Connected);
    }

    #[test]
    fn remote_status_glyph_maps() {
        assert_eq!(remote_status_glyph(RemoteStatus::Connected), "\u{21c4}"); // ⇄
        assert_eq!(remote_status_glyph(RemoteStatus::Disconnected), "\u{2717}");
        // ✗
    }

    // #85/#86 — the remote-pane badge (glyph reflects status)
    #[test]
    fn remote_badge_formats() {
        assert_eq!(
            remote_badge("localhost", RemoteStatus::Connected),
            "\u{21c4} localhost"
        );
        assert_eq!(
            remote_badge("localhost", RemoteStatus::Disconnected),
            "\u{2717} localhost"
        );
        assert_eq!(
            remote_badge("deploy@prod", RemoteStatus::Connected),
            "\u{21c4} deploy@prod"
        );
        assert_eq!(remote_badge("", RemoteStatus::Connected), "\u{21c4} ");
    }

    // #87 — the palette connect actions: label + parsed target per valid host
    #[test]
    fn remote_palette_actions_labels() {
        let hosts = vec![
            RemoteHost {
                name: "prod".to_string(),
                target: "deploy@prod:22".to_string(),
            },
            RemoteHost {
                name: "box".to_string(),
                target: "localhost".to_string(),
            },
        ];
        let actions = remote_palette_actions(&hosts);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].label, "connect: prod → deploy@prod:22");
        assert_eq!(actions[0].target, target(Some("deploy"), "prod", Some(22)));
        assert_eq!(actions[1].label, "connect: box → localhost");
        assert_eq!(actions[1].target, target(None, "localhost", None));
    }

    // #87 — a host with an invalid target is DROPPED (no panic, no placeholder)
    #[test]
    fn remote_palette_actions_drops_invalid() {
        assert!(remote_palette_actions(&[]).is_empty()); // empty → empty
        for bad in ["", "-x", "::1", "h:bad"] {
            let hosts = vec![RemoteHost {
                name: "n".to_string(),
                target: bad.to_string(),
            }];
            assert!(
                remote_palette_actions(&hosts).is_empty(),
                "expected drop for {bad:?}"
            );
        }
        // A mix keeps ONLY the valid entries, in order.
        let hosts = vec![
            RemoteHost {
                name: "ok1".to_string(),
                target: "h1".to_string(),
            },
            RemoteHost {
                name: "bad".to_string(),
                target: "-x".to_string(),
            },
            RemoteHost {
                name: "ok2".to_string(),
                target: "u@h2:2222".to_string(),
            },
        ];
        let actions = remote_palette_actions(&hosts);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].label, "connect: ok1 → h1");
        assert_eq!(actions[1].label, "connect: ok2 → u@h2:2222");
    }

    // REQ-004 — the argv, with the `--` end-of-options guard
    #[test]
    fn ssh_command_argv() {
        assert_eq!(
            ssh_command(&target(Some("u"), "h", None)),
            vec!["ssh", "--", "u@h"]
        );
        assert_eq!(
            ssh_command(&target(Some("u"), "h", Some(22))),
            vec!["ssh", "-p", "22", "--", "u@h"]
        );
        assert_eq!(
            ssh_command(&target(None, "h", None)),
            vec!["ssh", "--", "h"]
        );
        // even a hand-built dangerous target is neutralized by the `--`
        assert_eq!(
            ssh_command(&target(None, "-x", None)),
            vec!["ssh", "--", "-x"]
        );
    }
}
