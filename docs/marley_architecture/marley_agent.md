# `marley_agent`

The agent CLIs Marley knows, for the rail's agent rows (W4, #440). Pure and gpui-free, MIT OR
Apache-2.0, with no dependencies. Refreshed for the fork on 2026-09-22; the gpui-era cockpit
it once served is history (below).

## Provenance

Marley-original, built from observed behavior only. It never borrows from Warp's agent source
(`crates/ai`, `app/src/ai/**`): those are AGPL, and the agent layer is where that licence would
matter most (`docs/warp_architecture/subsystems/04-agent-ai-mcp.md`).

## Surface

```rust
pub enum AgentKind { Claude, Codex, Gemini, OpenCode }
impl AgentKind {
    pub const ALL: [AgentKind; 4];                 // the order menus list them
    pub const fn program(self) -> &'static str;    // "claude", "codex", "gemini", "opencode"
    pub const fn display_name(self) -> &'static str;
}
pub fn agent_kind_of(command: &str) -> Option<AgentKind>;
pub fn send_payload(line: &str) -> Vec<u8>;        // the line and a carriage return
pub fn launch_input(kind: AgentKind) -> Vec<u8>;   // the program's name and a carriage return

pub enum AgentStatus { Working, Waiting }           // label(): "working", "waiting"
pub const WAITING_AFTER: Duration;                  // 2 s
pub fn agent_status(quiet_for: Duration, bell: bool) -> AgentStatus;
pub fn status_line(kind: AgentKind, status: AgentStatus) -> String;   // "Claude Code · waiting"
```

- **`agent_kind_of`** reads a command line's leading program, with a directory path stripped
  and arguments ignored, so `/home/me/.local/bin/claude --resume` is Claude Code. Program names
  are case-sensitive. The rail feeds it a terminal's foreground argv, never a process name:
  Claude Code's binary on the dev box is named after its version.
- **`launch_input`** is everything the rail writes to start an agent: a program name from the
  fixed list and Enter. No text from a user or a file reaches the shell this way.
- **`agent_status`** reads waiting on a bell, or once the terminal has been quiet for
  `WAITING_AFTER`; otherwise working. Claude Code and Codex redraw a spinner while they work,
  so output keeps flowing until they stop. An agent that thinks without printing reads as
  waiting until it prints again.

## Consumers

- `marley_rail`: a terminal row carries `TerminalAgent { kind, status }` when an agent runs in
  its foreground.
- `marley_workbench::rail`:
  - recognition through `agent_kind_of`;
  - the `+` menu's Agent CLIs section (`AgentKind::ALL`, filtered to what the search path
    holds);
  - the launch (`launch_input`, written after the shell's startup handshake);
  - the quiet timers (`WAITING_AFTER` on gpui's executor clock).

## Tests

`src/marley_agent.rs`, five unit tests: names and order, recognition of all four and rejection
of others, the launch bytes, status around `WAITING_AFTER` and with the bell, and status lines.

## History

In the gpui-era app (M2, #66 to #80, #187) this crate modelled a pump-driven cockpit: an
`AgentRun` with tick counters, `agent_status_from` over `WAITING_TICKS`, and a `marley_app`
shim that launched and watched agents. The fork has no 16 ms pump and nothing used that model,
so #440 removed it. The agent brain that note once planned now belongs to the three-prong
plan's control plane (`docs/marley/three-prong-plan.md`).
