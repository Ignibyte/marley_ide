//! The agent CLIs Marley knows.
//!
//! Marley starts agent CLIs (Claude Code, Codex, …) in its terminals and recognizes them there.
//! This crate is the pure, gpui-free model for that: [`agent_kind_of`] recognizes an agent from
//! the command a terminal runs, [`launch_input`] is what starts one in a shell, and
//! [`agent_status`] judges from its terminal whether it is working or waiting on the user.
//! [`claude_events`] reads Claude Code's own hook events, which Marley's plugin sends, and folds
//! them into a fleet seat (#519), [`stop_kind`] says what a stopped turn needs (#566), and
//! [`stall`] what flags a looping or quiet working one (#569). The launching and the watching live
//! in `marley_workbench`.

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

use marley_fleet::State;

pub mod claude_events;
pub mod stall;
pub mod stop_kind;

/// An agent CLI Marley knows. [`AgentKind::ALL`], [`AgentKind::program`] and
/// [`AgentKind::display_name`] grow with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentKind {
    /// Anthropic's Claude Code (`claude`).
    Claude,
    /// `OpenAI`'s Codex CLI (`codex`).
    Codex,
    /// Google's Gemini CLI (`gemini`).
    Gemini,
    /// `OpenCode` (`opencode`).
    OpenCode,
}

impl AgentKind {
    /// Every known agent, in the order a menu lists them.
    pub const ALL: [Self; 4] = [Self::Claude, Self::Codex, Self::Gemini, Self::OpenCode];

    /// The program the agent runs as, which is also the command that starts it.
    #[must_use]
    pub const fn program(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
        }
    }

    /// The agent's short name, for a line that has little room, such as a push (#535).
    #[must_use]
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini",
            Self::OpenCode => "OpenCode",
        }
    }

    /// The agent's name as people call it.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini CLI",
            Self::OpenCode => "OpenCode",
        }
    }
}

/// Classify a command line as an agent CLI by its leading program.
///
/// A directory path on the program is stripped and arguments are ignored, so `claude --resume`
/// and `/usr/bin/claude` both resolve to [`AgentKind::Claude`]. Returns `None` for any other
/// command, and for an empty or whitespace-only one.
#[must_use]
pub fn agent_kind_of(command: &str) -> Option<AgentKind> {
    let token = command.split_whitespace().next()?;
    let program = token.rsplit('/').next().unwrap_or(token);
    AgentKind::ALL
        .into_iter()
        .find(|kind| kind.program() == program)
}

/// The PTY bytes that send a line to the foreground program: the line, then a carriage return
/// (Enter).
#[must_use]
pub fn send_payload(line: &str) -> Vec<u8> {
    format!("{line}\r").into_bytes()
}

/// What starts `kind` in a shell: its program name and Enter, and nothing else.
#[must_use]
pub fn launch_input(kind: AgentKind) -> Vec<u8> {
    send_payload(kind.program())
}

/// What an agent CLI is doing, as far as its terminal shows, or as its hook events say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// Output is arriving, or its events say a turn is under way: the agent is at work.
    Working,
    /// Output has stopped, the agent rang the bell, or it asked for a permission: it waits on the
    /// user.
    Waiting,
    /// Its events say the turn ended and it waits at its prompt (#519).
    Idle,
    /// Its events say the turn failed, with an error (#519).
    Failed,
}

impl AgentStatus {
    /// The word a row shows for the status.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Idle => "idle",
            Self::Failed => "failed",
        }
    }
}

/// What an agent's turn came to, for a line that names it outside Marley (#535).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnEvent {
    /// It asks for a permission or a choice.
    NeedsInput,
    /// Its turn ended.
    Finished,
    /// Its turn failed.
    Failed,
}

impl TurnEvent {
    /// The event a seat's change of state from `before` to `after` makes, if it makes one: a
    /// wait that starts, a turn that ends, a turn that fails.
    #[must_use]
    pub const fn of_change(before: State, after: State) -> Option<Self> {
        match (before, after) {
            (State::Waiting, State::Waiting) | (State::Error, State::Error) => None,
            (_, State::Waiting) => Some(Self::NeedsInput),
            (State::Working | State::Waiting, State::Idle) => Some(Self::Finished),
            (_, State::Error) => Some(Self::Failed),
            _ => None,
        }
    }

    /// The words a line gives the event.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::NeedsInput => "needs input",
            Self::Finished => "finished",
            Self::Failed => "failed",
        }
    }
}

/// A line that names an agent's event outside Marley, such as `marley_ide: Claude needs input`
/// (#535). The project's name loses its control characters; nothing the agent wrote goes in.
#[must_use]
pub fn event_line(project: &str, kind: AgentKind, event: TurnEvent) -> String {
    let project: String = project
        .chars()
        .filter(|character| !character.is_control())
        .collect();
    format!("{project}: {} {}", kind.short_name(), event.words())
}

/// How long an agent's terminal stays quiet before the agent reads as waiting. Claude Code and
/// Codex redraw a spinner while they work, so a spell this long without output means they have
/// stopped.
pub const WAITING_AFTER: Duration = Duration::from_secs(2);

/// An agent's status from how long its terminal has been quiet and whether it rang the bell.
#[must_use]
pub fn agent_status(quiet_for: Duration, bell: bool) -> AgentStatus {
    if bell || quiet_for >= WAITING_AFTER {
        AgentStatus::Waiting
    } else {
        AgentStatus::Working
    }
}

/// A row's second line for an agent, such as "Claude Code · waiting".
#[must_use]
pub fn status_line(kind: AgentKind, status: AgentStatus) -> String {
    format!("{} · {}", kind.display_name(), status.label())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_agent_has_its_program_and_name() {
        let expected = [
            (AgentKind::Claude, "claude", "Claude Code"),
            (AgentKind::Codex, "codex", "Codex"),
            (AgentKind::Gemini, "gemini", "Gemini CLI"),
            (AgentKind::OpenCode, "opencode", "OpenCode"),
        ];
        assert_eq!(
            AgentKind::ALL,
            expected.map(|(kind, _, _)| kind),
            "menus list them in this order"
        );
        for (kind, program, name) in expected {
            assert_eq!(kind.program(), program);
            assert_eq!(kind.display_name(), name);
        }
    }

    #[test]
    fn agent_kind_of_recognizes_each_program_and_nothing_else() {
        for kind in AgentKind::ALL {
            let program = kind.program();
            assert_eq!(agent_kind_of(program), Some(kind));
            // Arguments are ignored and a directory path is stripped.
            assert_eq!(agent_kind_of(&format!("{program} --resume")), Some(kind));
            assert_eq!(
                agent_kind_of(&format!("/usr/local/bin/{program}")),
                Some(kind)
            );
            // The command the launch writes is recognized back as the same agent.
            let launched = String::from_utf8(launch_input(kind)).expect("the launch is text");
            assert_eq!(agent_kind_of(&launched), Some(kind));
        }
        assert_eq!(agent_kind_of("ls"), None);
        assert_eq!(agent_kind_of("claude-code"), None, "a different program");
        assert_eq!(
            agent_kind_of("Claude"),
            None,
            "program names are case-sensitive"
        );
        assert_eq!(agent_kind_of(""), None);
        assert_eq!(agent_kind_of("   "), None);
    }

    #[test]
    fn the_launch_is_the_program_and_enter() {
        assert_eq!(launch_input(AgentKind::Claude), b"claude\r");
        assert_eq!(launch_input(AgentKind::OpenCode), b"opencode\r");
        assert_eq!(send_payload("ls"), b"ls\r");
        assert_eq!(send_payload(""), b"\r");
        assert_eq!(send_payload("café"), "café\r".as_bytes());
    }

    #[test]
    fn an_agent_waits_after_a_quiet_spell_or_a_bell() {
        let just_under = WAITING_AFTER.saturating_sub(Duration::from_millis(1));
        assert_eq!(agent_status(Duration::ZERO, false), AgentStatus::Working);
        assert_eq!(agent_status(just_under, false), AgentStatus::Working);
        assert_eq!(agent_status(WAITING_AFTER, false), AgentStatus::Waiting);
        assert_eq!(agent_status(Duration::ZERO, true), AgentStatus::Waiting);
        assert_eq!(agent_status(just_under, true), AgentStatus::Waiting);
    }

    #[test]
    fn the_status_line_names_the_agent_and_its_status() {
        assert_eq!(
            status_line(AgentKind::Claude, AgentStatus::Working),
            "Claude Code · working"
        );
        assert_eq!(
            status_line(AgentKind::Gemini, AgentStatus::Waiting),
            "Gemini CLI · waiting"
        );
    }
}
