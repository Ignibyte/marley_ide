//! What a row, a tab or a switcher row stands for, its state, and their colours (#905).
//!
//! The rail, the center tabs and the Ctrl Tab switcher draw from here, so an agent, a tmux session
//! or a waiting thread looks the same wherever it is listed.

use std::collections::HashMap;

use gpui::{App, EntityId, Global, Hsla};
use marley_agent::AgentKind;
use marley_rail::{Attention, ThreadStatus};
use project::AgentId;
use ui::{Color, IconName, prelude::*};

/// An agent, as its tile's glyph and colour tell it apart (#905, D2). Agent Activity's day files
/// keep it (#933).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentMark {
    Claude,
    Codex,
    Gemini,
    OpenCode,
    /// Marley's own agent (#683).
    Marley,
    /// Rusty's agent (#696).
    Rusty,
    /// Any other agent, by the first letter of its id.
    Other(char),
}

impl AgentMark {
    /// The agent CLI `kind`.
    pub(crate) const fn from_cli(kind: AgentKind) -> Self {
        match kind {
            AgentKind::Claude => Self::Claude,
            AgentKind::Codex => Self::Codex,
            AgentKind::Gemini => Self::Gemini,
            AgentKind::OpenCode => Self::OpenCode,
        }
    }

    /// A thread's agent, by the id its threads carry: the ACP adapters Marley starts for Claude
    /// Code and Codex, Marley's and Rusty's entries, else the id's first letter.
    pub(crate) fn from_agent_id(agent: &AgentId) -> Self {
        match crate::assistant::assistant_icon(agent) {
            Some(IconName::MarleyCat) => return Self::Marley,
            Some(IconName::RustyButler) => return Self::Rusty,
            _ => {}
        }
        match agent.0.as_ref() {
            crate::agents::CLAUDE_AGENT => Self::Claude,
            crate::agents::CODEX_AGENT => Self::Codex,
            id => Self::Other(
                id.chars()
                    .next()
                    .map_or('?', |first| first.to_ascii_uppercase()),
            ),
        }
    }

    /// An MCP client, by the name it gave at `initialize` (#933): Claude Code's and Codex's, else
    /// the name's first letter, as Zed's `Z`.
    pub(crate) fn from_client(name: &str) -> Self {
        match name {
            "claude-code" => Self::Claude,
            codex if codex.starts_with("codex") => Self::Codex,
            other => Self::Other(
                other
                    .chars()
                    .next()
                    .map_or('?', |first| first.to_ascii_uppercase()),
            ),
        }
    }

    /// The letters on the agent's tile.
    pub(crate) fn glyph(self) -> SharedString {
        match self {
            Self::Claude => SharedString::new_static("CC"),
            Self::Codex => SharedString::new_static("CX"),
            Self::Gemini => SharedString::new_static("G"),
            Self::OpenCode => SharedString::new_static("OC"),
            Self::Marley => SharedString::new_static("M"),
            Self::Rusty => SharedString::new_static("R"),
            Self::Other(first) => first.to_string().into(),
        }
    }

    /// The agent's colour: Claude Code amber, Codex violet, Marley the coat's light, Rusty teal,
    /// any other agent amber.
    pub(crate) fn ink(self, cx: &App) -> Hsla {
        let colors = cx.theme().colors();
        match self {
            Self::Codex => colors.terminal_ansi_magenta,
            Self::Marley => colors.text_accent,
            Self::Rusty => colors.terminal_ansi_cyan,
            Self::Claude | Self::Gemini | Self::OpenCode | Self::Other(_) => {
                cx.theme().status().warning
            }
        }
    }
}

/// What a row, a tab or a switcher row stands for (#827, #905).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemKind {
    /// An agent thread, by its agent.
    Thread(AgentMark),
    /// A terminal running an agent CLI, by its agent.
    AgentTerminal(AgentMark),
    Shell,
    /// A terminal attached to a tmux session.
    Tmux,
    Browser,
    File,
    /// A port's row (#890), whose tile holds the port and so takes the port's width.
    Port,
    /// Any other tab: Home, the Threads page, a settings page.
    Page,
}

impl ItemKind {
    /// The colour of the kind's glyph (D2).
    pub(crate) fn ink(self, cx: &App) -> Hsla {
        let colors = cx.theme().colors();
        match self {
            Self::Thread(agent) | Self::AgentTerminal(agent) => agent.ink(cx),
            Self::Shell | Self::Page => colors.text_muted,
            // The boards' teal.
            Self::Tmux | Self::Port => colors.terminal_ansi_cyan,
            Self::Browser | Self::File => colors.text_accent,
        }
    }

    /// The kind's tile: its ink at a low opacity, so a light theme follows; a shell's and a
    /// page's a raised grey.
    pub(crate) fn tint(self, cx: &App) -> Hsla {
        match self {
            Self::Shell | Self::Page => cx.theme().colors().text.opacity(0.1),
            _ => self.ink(cx).opacity(0.15),
        }
    }
}

/// What state a row or a tab is in, as its tile's dot and its line's colour say it (#827, #905).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemState {
    Working,
    NeedsYou,
    Done,
    /// A shell's last command failed, or a thread's last run.
    Failed,
    Idle,
}

impl ItemState {
    /// The rail's own reading of how much a row needs the user; one that stopped reporting still
    /// reads as working.
    pub(crate) const fn from_attention(attention: Attention) -> Self {
        match attention {
            Attention::NeedsYou => Self::NeedsYou,
            Attention::DoneUnseen => Self::Done,
            Attention::Working | Attention::NotReporting => Self::Working,
            Attention::Idle => Self::Idle,
        }
    }

    /// A thread's state from its status alone, as a tab and a switcher row show it.
    pub(crate) const fn from_thread(status: ThreadStatus) -> Self {
        match status {
            ThreadStatus::Waiting => Self::NeedsYou,
            ThreadStatus::Error => Self::Failed,
            ThreadStatus::Running => Self::Working,
            ThreadStatus::Done => Self::Idle,
        }
    }

    /// The dot's colour (D3): working amber, needs you pink, done green, failed red, idle grey.
    pub(crate) fn dot(self, cx: &App) -> Hsla {
        let status = cx.theme().status();
        match self {
            Self::Working => status.warning,
            Self::NeedsYou => status.conflict,
            Self::Done => status.success,
            Self::Failed => status.error,
            Self::Idle => cx.theme().colors().text_muted.opacity(0.5),
        }
    }

    /// The colour of a row's first line in this state.
    pub(crate) fn line(self, cx: &App) -> Color {
        match self {
            Self::Working => Color::Warning,
            Self::NeedsYou => Color::Conflict,
            Self::Done => Color::Custom(cx.theme().colors().text_accent),
            Self::Failed => Color::Error,
            Self::Idle => Color::Muted,
        }
    }

    /// The state in words, as a switcher row's line says it.
    pub(crate) const fn words(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::NeedsYou => "needs you",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Idle => "idle",
        }
    }
}

/// `glyph` on a tile `width` wide, in `kind`'s tint and ink: a tab's, a switcher row's and the
/// bottom row's agent tile (#905, #907).
pub(crate) fn tile(kind: ItemKind, glyph: SharedString, width: Pixels, cx: &App) -> Div {
    div()
        .flex_none()
        .w(width)
        .py_0p5()
        .flex()
        .justify_center()
        .rounded_sm()
        .bg(kind.tint(cx))
        .child(
            Label::new(glyph)
                .size(LabelSize::XSmall)
                .buffer_font(cx)
                .color(Color::Custom(kind.ink(cx))),
        )
}

/// A square tile of `size` with the kind's glyph, for a card's head (#933).
pub(crate) fn square_tile(kind: ItemKind, glyph: SharedString, size: Pixels, cx: &App) -> Div {
    div()
        .flex_none()
        .size(size)
        .rounded(size / 4.)
        .flex()
        .items_center()
        .justify_center()
        .bg(kind.tint(cx))
        .child(
            Label::new(glyph)
                .size(LabelSize::Small)
                .buffer_font(cx)
                .color(Color::Custom(kind.ink(cx))),
        )
}

/// Each terminal's state as its rail row shows it, by the terminal view's entity id (#905): the
/// rail reads an agent CLI's quiet screen as waiting, which a tab cannot see, so a tab and a
/// switcher row take the rail's word. A terminal the rail is not showing has none.
#[derive(Default, PartialEq, Eq)]
pub(crate) struct TerminalStates(pub(crate) HashMap<EntityId, ItemState>);

impl Global for TerminalStates {}

/// The state the rail last showed for the terminal view `view`.
pub(crate) fn terminal_state(view: EntityId, cx: &App) -> Option<ItemState> {
    cx.try_global::<TerminalStates>()
        .and_then(|states| states.0.get(&view).copied())
}
