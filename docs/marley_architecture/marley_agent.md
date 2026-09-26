# `marley_agent`

The agent CLIs Marley knows, for the rail's agent rows (W4, #440). Pure and gpui-free, MIT OR
Apache-2.0. Since #519 it depends on `marley_fleet`, `serde`, `serde_json` and `base64`, for
Claude Code's hook events. Refreshed for the fork on 2026-09-22; the gpui-era cockpit it once
served is history (below).

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

pub enum AgentStatus { Working, Waiting, Idle, Failed }  // label(): "working", "waiting", …
pub const WAITING_AFTER: Duration;                  // 2 s
pub fn agent_status(quiet_for: Duration, bell: bool) -> AgentStatus;
pub fn status_line(kind: AgentKind, status: AgentStatus) -> String;   // "Claude Code · waiting"
pub enum TurnEvent { NeedsInput, Finished, Failed }    // of_change(before, after), words()
pub fn event_line(project: &str, kind: AgentKind, event: TurnEvent) -> String;
                                                     // "marley_ide: Claude needs input" (#535)
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

## Claude Code's hook events (`src/claude_events.rs`, #519)

```rust
pub struct HookEvent { v, event, session_id, agent_id, prompt_id, permission_mode, cwd,
                       transcript_path, prompt, tool, preview, tool_use_id, message, error,
                       source, trigger, is_interrupt }
pub fn decode(body: &str) -> Result<HookEvent, DecodeError>;   // base64, ≤ 3,000 bytes, v 1
pub fn fold(seat: &str, previous: Option<&Session>, event: &HookEvent, now_ms: u64)
    -> Vec<SessionEvent>;
pub const fn seat_status(state: State) -> AgentStatus;
pub fn seat_line(seat: &Session, now_ms: u64, no_update_after_ms: u64) -> String;
                                                     // "working · 1 subagent · Add a README"
pub fn seat_activity(seat: &Session) -> Option<String>;   // "Bash: ls -la", the question, …
pub fn is_harness_injected(prompt: &str) -> bool;
pub fn is_compact_continuation(prompt: &str) -> bool;
```

- The plugin's `hooks/event.py` sends each hook event as an OSC 777 notify titled
  `marley-event` whose body is the base64 of a JSON summary; `decode` reads the body.
- `fold` moves a terminal's seat from the seat as it was, and returns the fleet events for
  `marley_fleet::apply`: an `Upsert` with the state and labels, then a `QuestionRaised` while
  the seat waits, or an `Ended` for `SessionEnd`. The rules are the spec's D6: prompts and tools
  make it working, a permission request (or `AskUserQuestion`) waiting until the tool it asked
  for finishes or the turn ends, `Stop` idle, `StopFailure` failed, a subagent's events move
  only a count, and a new session id starts the seat over.
- The labels carry the prompt, the lead's tool line, the last message, the error, the
  subagent count and the session's id, transcript, directory and permission mode, and, for the
  fold itself, the tools in flight by call id (`lead_tool:`, `subagent_tool:`) and the call a
  wait ends with (`waiting_on`), so `fold` needs nothing but the previous seat.
- `seat_line` gives a working seat whose last event is at least `no_update_after_ms` old
  `no update in N m` in place of `working` (#547, through `marley_fleet::is_stale`); 0 never.
- A prompt with one of the tags or openings Orca observed harnesses inject
  (`src/shared/harness-injected-user-turns.ts` in stablyai/orca, MIT) keeps the user's prompt on
  the seat; the continuation after a compaction changes nothing.

## Consumers

- `marley_rail`: a terminal row carries `TerminalAgent { kind, status }` when an agent runs in
  its foreground, and `activity` from `seat_activity` when its events gave one.
- `marley_workbench::agent_events`: `decode` and `fold` for each `marley-event` frame;
  `marley_workbench::rail`: `seat_status`, `seat_line` and `seat_activity` for a Claude Code
  row with a seat.
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
