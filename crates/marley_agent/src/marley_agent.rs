//! The agent-run model — the foundation of the agent cockpit.
//!
//! Marley launches and observes agent terminals (Claude, Codex, …). This crate is the pure, gpui-free
//! model for that: [`agent_kind_of`] recognizes an agent CLI from a command line, [`agent_status_from`]
//! projects a pane's flags into an [`AgentStatus`], and [`AgentRun`] ties a kind + label + status
//! together. The launch and observation live in the app shim (a later ticket); every decision here is
//! pure and unit-tested.

/// A recognized agent CLI. Extend the enum (and [`agent_kind_of`]) as more agents are supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    /// Anthropic's Claude Code CLI (`claude`).
    Claude,
    /// `OpenAI`'s Codex CLI (`codex`).
    Codex,
}

/// Classify a command line as an agent CLI by its leading program.
///
/// A directory path on the program is stripped and arguments are ignored, so `claude --resume` and
/// `/usr/bin/claude` both resolve to [`AgentKind::Claude`]. Returns `None` for a non-agent command
/// or an empty/whitespace-only one.
#[must_use]
pub fn agent_kind_of(command: &str) -> Option<AgentKind> {
    let token = command.split_whitespace().next()?;
    let program = token.rsplit('/').next().unwrap_or(token);
    match program {
        "claude" => Some(AgentKind::Claude),
        "codex" => Some(AgentKind::Codex),
        _ => None,
    }
}

/// The CLI program to spawn for an agent kind — the inverse of [`agent_kind_of`].
#[must_use]
pub const fn launch_command(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Claude => "claude",
        AgentKind::Codex => "codex",
    }
}

/// The PTY bytes to send a composed line to a running agent — the line plus a trailing `\r` (Enter
/// for the foreground program).
///
/// A running agent WANTS this raw write (unlike a bare cooked prompt, #59/#65).
#[must_use]
pub fn send_payload(line: &str) -> Vec<u8> {
    format!("{line}\r").into_bytes()
}

/// An agent run's live status, driving its cockpit indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// At rest — no active work.
    Idle,
    /// Actively producing output / running.
    Working,
    /// Running but quiet for a while — probably at its prompt waiting for you (#79).
    Waiting,
    /// The agent process has ended.
    Exited,
}

/// Pump ticks of silence (~16ms each, so ≈ 1s) before a running agent reads as [`AgentStatus::Waiting`]
/// rather than [`AgentStatus::Working`] (#79).
pub const WAITING_TICKS: u32 = 60;

/// Project a pane's flags into an [`AgentStatus`].
///
/// An EXITED agent is [`AgentStatus::Exited`] (checked FIRST, so a stale `active` never overrides
/// it); an inactive one is [`AgentStatus::Idle`]; an active one that has been quiet for
/// `quiet_ticks >= WAITING_TICKS` is [`AgentStatus::Waiting`] (probably at its prompt); otherwise
/// it is [`AgentStatus::Working`] (#79).
#[must_use]
pub const fn agent_status_from(exited: bool, active: bool, quiet_ticks: u32) -> AgentStatus {
    if exited {
        AgentStatus::Exited
    } else if !active {
        AgentStatus::Idle
    } else if quiet_ticks >= WAITING_TICKS {
        AgentStatus::Waiting
    } else {
        AgentStatus::Working
    }
}

/// A running (or finished) agent terminal: what it is, a display label, and its current status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRun {
    /// Which agent CLI this run is.
    pub kind: AgentKind,
    /// A human-facing label (e.g. the pane title or the launch command).
    pub label: String,
    /// The current status.
    pub status: AgentStatus,
    /// The agent's most-recent non-empty output line (#78) — refreshed each pump tick; shown in the Fleet.
    pub last_line: String,
    /// Pump ticks since the agent last produced output (#79) — drives the Waiting status.
    pub quiet_ticks: u32,
    /// The ticket this agent was last told to work (#80) — parsed from a sent/broadcast line ("#77").
    pub ticket: Option<u64>,
    /// Pump ticks the agent has been ALIVE (M12 #187) — +1 each tick while running, frozen at exit. The
    /// run duration is `run_ticks / ~62` (tick-count elapsed; `Date::now` is banned — same clock as #82).
    pub run_ticks: u32,
    /// The shell's exit code once the agent has ended (M12 #187): `None` while alive, `Some(code)` after
    /// `ChildExited` (a signal/unknown exit is stamped `-1` by the shim). Drives the ✓/✕ done glyph.
    pub exit_code: Option<i32>,
}

impl AgentRun {
    /// A fresh agent run — the given `kind` and `label`, starting [`AgentStatus::Idle`], no output yet.
    #[must_use]
    pub const fn new(kind: AgentKind, label: String) -> Self {
        Self {
            kind,
            label,
            status: AgentStatus::Idle,
            last_line: String::new(),
            quiet_ticks: 0,
            ticket: None,
            run_ticks: 0,
            exit_code: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_kind_of_recognizes_and_rejects() {
        assert_eq!(agent_kind_of("claude"), Some(AgentKind::Claude));
        assert_eq!(agent_kind_of("claude --resume"), Some(AgentKind::Claude)); // args ignored
        assert_eq!(agent_kind_of("/usr/bin/claude"), Some(AgentKind::Claude)); // path stripped
        assert_eq!(agent_kind_of("codex"), Some(AgentKind::Codex));
        assert_eq!(agent_kind_of("ls"), None); // not an agent
        assert_eq!(agent_kind_of(""), None); // empty → no leading token
        assert_eq!(agent_kind_of("   "), None); // whitespace-only
    }

    // REQ-001 (#72) — the send payload is the line plus a single CR (Enter for the running program).
    #[test]
    fn send_payload_appends_cr() {
        assert_eq!(send_payload("ls"), b"ls\r");
        assert_eq!(send_payload(""), b"\r"); // empty line → just Enter
        assert_eq!(send_payload("café"), "café\r".as_bytes()); // multibyte-clean
    }

    #[test]
    fn launch_command_is_the_inverse_of_agent_kind_of() {
        assert_eq!(launch_command(AgentKind::Claude), "claude");
        assert_eq!(launch_command(AgentKind::Codex), "codex");
        // Round-trip: the command we spawn is recognized back as that same agent kind.
        assert_eq!(
            agent_kind_of(launch_command(AgentKind::Claude)),
            Some(AgentKind::Claude)
        );
        assert_eq!(
            agent_kind_of(launch_command(AgentKind::Codex)),
            Some(AgentKind::Codex)
        );
    }

    #[test]
    fn agent_status_from_exited_first() {
        // Exited is checked FIRST — it wins even over a stale `active`.
        assert_eq!(agent_status_from(true, true, 0), AgentStatus::Exited);
        assert_eq!(agent_status_from(true, false, 999), AgentStatus::Exited); // exited wins even if quiet
        assert_eq!(agent_status_from(false, false, 999), AgentStatus::Idle); // !active → Idle regardless
        assert_eq!(agent_status_from(false, true, 0), AgentStatus::Working); // active + recent output
        // #79 the WAITING_TICKS boundary: 59 → Working, 60 → Waiting
        assert_eq!(
            agent_status_from(false, true, WAITING_TICKS - 1),
            AgentStatus::Working
        );
        assert_eq!(
            agent_status_from(false, true, WAITING_TICKS),
            AgentStatus::Waiting
        );
    }

    #[test]
    fn agent_run_new_starts_idle() {
        let run = AgentRun::new(AgentKind::Claude, "my agent".to_string());
        assert_eq!(run.kind, AgentKind::Claude);
        assert_eq!(run.label, "my agent");
        assert_eq!(run.status, AgentStatus::Idle);
        assert_eq!(run.run_ticks, 0); // #187 — no elapsed ticks yet
        assert_eq!(run.exit_code, None); // #187 — alive (not exited)
    }
}
