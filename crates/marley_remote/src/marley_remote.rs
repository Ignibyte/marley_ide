//! PURE — the remote (ssh) seam: parse a `[user@]host[:port]` target and build the `ssh` argv from it.
//!
//! Marley opens a remote pane by spawning the USER'S `ssh` client as a terminal (like Terminal.app) —
//! `ssh` owns ALL security (keys, `known_hosts`, auth, passwords). This crate handles NO secrets; it only
//! parses a typed target and produces an argv. The command is a `Vec<String>` (never a shell string), so
//! nothing can inject a shell command; and a leading-dash host/user is rejected (and the argv carries a
//! `--` before the destination) so it can never be re-parsed as an `ssh` OPTION (option-smuggling).
//!
//! Every link Marley starts asks ssh's own keepalive to check it, so a silent link ends within
//! 20 s, and a remote terminal whose link ended is checked on a backoff before it reattaches
//! (#641): [`keepalive_options`], [`link_check_command`], [`read_link_check`], [`check_delay`].

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

use std::time::Duration;

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

/// How often, in seconds, ssh asks the server whether it is there after hearing nothing (#641).
pub const SERVER_ALIVE_INTERVAL_S: u32 = 5;

/// How many such asks may go unanswered before ssh ends the link, so a silent link ends within
/// 20 s instead of when TCP gives up (#641).
pub const SERVER_ALIVE_COUNT_MAX: u32 = 3;

/// How long, in seconds, a remote terminal's ssh and a link check wait to connect and finish the
/// handshake (#641).
pub const CONNECT_TIMEOUT_S: u32 = 10;

/// The longest wait between two checks of a link that is down (#641).
pub const MAX_CHECK_DELAY: Duration = Duration::from_secs(120);

/// How long a link stays up before its next drop starts the checks over from one second (#641).
pub const STABLE_LINK: Duration = Duration::from_secs(60);

/// ssh's exit status for an error of its own, as opposed to the remote command's status.
const SSH_ERROR: i32 = 255;

/// What ssh prints when the link failed rather than the login: a check that ends with one of
/// these is checked again (#641).
const LINK_FAILURES: [&str; 11] = [
    "connect to host",
    "timed out",
    "Could not resolve hostname",
    "Connection closed",
    "Connection reset",
    "not responding",
    "banner exchange",
    "kex_exchange_identification",
    "Network is unreachable",
    "No route to host",
    "Broken pipe",
];

/// ssh's own keepalive, for every ssh Marley starts (#641).
///
/// Asked through the encrypted channel every [`SERVER_ALIVE_INTERVAL_S`] seconds, the link ends
/// after [`SERVER_ALIVE_COUNT_MAX`] unanswered asks. Given on the command line, they win over the
/// user's `~/.ssh/config`.
#[must_use]
pub fn keepalive_options() -> Vec<String> {
    vec![
        "-o".to_string(),
        format!("ServerAliveInterval={SERVER_ALIVE_INTERVAL_S}"),
        "-o".to_string(),
        format!("ServerAliveCountMax={SERVER_ALIVE_COUNT_MAX}"),
    ]
}

/// The argv that checks whether `target` answers (#641).
///
/// `ssh -T -o BatchMode=yes`, the keepalive and the connect timeout, then [`ssh_command`]'s
/// destination and `true`. `BatchMode` asks for no password, so a check never prompts behind a
/// dimmed terminal.
#[must_use]
pub fn link_check_command(target: &SshTarget) -> Vec<String> {
    let mut argv: Vec<String> = ["ssh", "-T", "-o", "BatchMode=yes"]
        .iter()
        .map(|word| (*word).to_string())
        .collect();
    argv.extend(keepalive_options());
    argv.extend([
        "-o".to_string(),
        format!("ConnectTimeout={CONNECT_TIMEOUT_S}"),
    ]);
    // `ssh_command` puts the port, `--` and the destination after its `ssh`.
    argv.extend(ssh_command(target).into_iter().skip(1));
    argv.push("true".to_string());
    argv
}

/// What a link check found (#641).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkCheck {
    /// The host answered: reattach.
    Answers,
    /// The link is still down, with ssh's words: check again later.
    Down(String),
    /// Anything else, such as a refused login or a changed host key, with ssh's words: stop
    /// checking, since another login could lock the user out.
    Stopped(String),
}

/// Reads a link check's exit status and what it printed on stderr (#641).
///
/// 0 is [`LinkCheck::Answers`]; ssh's own error status with a link failure in its last line is
/// [`LinkCheck::Down`]; anything else is [`LinkCheck::Stopped`]. The reason is the last line ssh
/// printed.
#[must_use]
pub fn read_link_check(code: Option<i32>, stderr: &str) -> LinkCheck {
    if code == Some(0) {
        return LinkCheck::Answers;
    }
    let reason = stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or("ssh ended without saying why")
        .to_string();
    if code == Some(SSH_ERROR) && LINK_FAILURES.iter().any(|failure| reason.contains(failure)) {
        LinkCheck::Down(reason)
    } else {
        LinkCheck::Stopped(reason)
    }
}

/// The wait before a down link's next check, after `failures` checks failed since it was last
/// stable: 1 s, then twice as long each time, never more than [`MAX_CHECK_DELAY`] (#641).
#[must_use]
pub fn check_delay(failures: u32) -> Duration {
    1_u64
        .checked_shl(failures)
        .map_or(MAX_CHECK_DELAY, |seconds| {
            Duration::from_secs(seconds).min(MAX_CHECK_DELAY)
        })
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

/// The saved host `host`, with its user and port, as a target, through [`parse_ssh_target`]'s
/// checks; `None` for one that fails them. A host with a `:` is taken as an IPv6 address.
#[must_use]
pub fn saved_target(host: &str, user: Option<&str>, port: Option<u16>) -> Option<SshTarget> {
    let host = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    let written = user.map_or_else(|| host.clone(), |user| format!("{user}@{host}"));
    let target = parse_ssh_target(&written)?;
    // A port inside the saved host would be a second one; the entry's own field is the port.
    if target.port.is_some() {
        return None;
    }
    Some(SshTarget { port, ..target })
}

/// The name of a remote terminal's tmux session (#543): `marley-` and eight lowercase hex digits,
/// or a name of `[a-z0-9-]` only, so it is plain text to the remote shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionName(String);

impl SessionName {
    /// The name `bits` makes: `marley-` and its eight hex digits.
    #[must_use]
    pub fn from_bits(bits: u32) -> Self {
        Self(format!("marley-{bits:08x}"))
    }

    /// `name` as a session name; `None` when it is empty or holds anything outside `[a-z0-9-]`.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        let plain = !name.is_empty()
            && !name.starts_with('-')
            && name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        plain.then(|| Self(name.to_string()))
    }

    /// The name as tmux takes it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The tmux server options a remote terminal's session runs with, each a tmux command: no status
/// line, no prefix key (so Ctrl+B reaches the program), the mouse for tmux's history, escape
/// sequences passed through to Marley's terminal, and 24-bit colour.
const SERVER_OPTIONS: [&[&str]; 5] = [
    &["set-option", "-g", "status", "off"],
    &["set-option", "-g", "prefix", "None"],
    &["set-option", "-g", "mouse", "on"],
    &["set-option", "-g", "allow-passthrough", "on"],
    &[
        "set-option",
        "-as",
        "terminal-features",
        ",xterm-256color:RGB",
    ],
];

/// The argv of a remote terminal (#543): ssh into the tmux session `session` on the host.
///
/// It is [`ssh_command`] with `-t`, the keepalive and the connect timeout (#641), then the remote
/// command that attaches the session on a tmux server of Marley's own (`-L marley`, no config
/// file), making it if it is not there, with `MARLEY_REMOTE=1` in its shells.
///
/// ssh joins the words after the destination with spaces for the remote shell, so each word is
/// plain text there: fixed words, the session name (`[a-z0-9-]`), and `\;`, which the remote shell
/// turns into the `;` between tmux's commands.
#[must_use]
pub fn remote_terminal_command(target: &SshTarget, session: &SessionName) -> Vec<String> {
    let mut argv = vec!["ssh".to_string(), "-t".to_string()];
    argv.extend(keepalive_options());
    argv.extend([
        "-o".to_string(),
        format!("ConnectTimeout={CONNECT_TIMEOUT_S}"),
    ]);
    argv.extend(ssh_command(target).into_iter().skip(1));
    let attach = [
        "tmux",
        "-L",
        "marley",
        "-f",
        "/dev/null",
        "new-session",
        "-A",
        "-s",
        session.as_str(),
        "-e",
        "MARLEY_REMOTE=1",
    ];
    argv.extend(attach.iter().map(|word| (*word).to_string()));
    for option in SERVER_OPTIONS {
        argv.push("\\;".to_string());
        argv.extend(option.iter().map(|word| (*word).to_string()));
    }
    argv
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
