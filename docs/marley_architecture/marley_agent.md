# `marley_agent`

> Per-crate architecture note — **round 4 refresh · 2026-07-12 · current to M15.**
> Provenance: **`[Marley-original]`** (INVENT, std-only; no Warp lineage). **This crate is the SEED of the
> sellable, proprietary BRAIN** — the one layer Marley intends to *sell*, so it must stay **clean-room.**
> See [`warp_architecture/subsystems/04-agent-ai-mcp.md`](../warp_architecture/subsystems/04-agent-ai-mcp.md)
> for the AGPL-3.0 **§13 network-use** boundary and the **`EditOrigin::Agent`** firewall
> (`crates/editor/src/types.rs`) that keeps the open editor and the proprietary brain talking only through
> Marley-original typed values.

The agent-run model — the pure, gpui-free foundation of the **agent cockpit** (Marley launches +
**observes** agent terminals: Claude, Codex, …). Every decision here is pure and unit-tested; the launch +
observation live in the `marley_app` shim.

## Provenance & the clean-room brain (read this first)

`marley_agent` is **early** and **small** — today it is a CLI recognizer + a status projection + a run
struct. But it is the *toehold* of the one subsystem Marley must **not** port from Warp. Terminal + editor
are the AGPL/open half; **the brain is the proprietary, sellable half.** The rule (from the linked Warp
subsystem doc — the sharpest provenance call in the map):

- **Build clean-room from concepts** (tool-calling, MCP, context assembly, streaming) — **never** from
  Warp's AGPL agent source (`crates/ai`, `app/src/ai/**`, `warp_multi_agent_client`). Those are the
  *reference*, not the source; **none is ported.**
- **AGPL §13 is existential, not cosmetic.** The brain is by design a hosted/network service; if it were
  *derived* from Warp's AGPL agent code, §13's network-use trigger forces publication of the brain's
  complete source to every network user — **the resale model dies.** Studying the architecture (these docs
  are Marley's own descriptions) or speaking an open protocol (MCP) does **not** cross the derivative line;
  lifting or adapting the source does.
- **The `EditOrigin::Agent` firewall is that boundary made physical.** `crates/editor/src/types.rs` carries
  `EditOrigin::{Human, Agent}` on every `EditResult`, so an agent-issued buffer write is distinguishable
  from a human keystroke *at the buffer edge* — the exact seam a supervised brain writes through. Brain and
  (open) editor communicate **only** through Marley-original typed values (an `EditOrigin`, an MCP tool
  call, a `session.read` read-back delta), never by sharing Warp-derived code. That keeps the copyleft in
  the open half, off the sold half — and makes the boundary *auditable* (a reviewer can check the brain
  imports no Warp-derived crate and touches the editor only through this value seam).

**What exists in Marley today** (vs. Warp's `crates/ai` / `app/src/ai/**` reference): NO orchestration
loop, NO direct-provider streaming client, NO agent action/result vocabulary, NO agent-session supervision
— those are intake, not code (`brain-agent-session-supervision` · `local_control` + a `session.read`
delta). What DOES exist: this pure run-model and the `EditOrigin::Agent` seam. (The scrap-forge pivot —
2026-08-09, #409/#411 — took the `forge` MCP sidecar + `marley_forge_client` that once proved the MCP
wiring here; the ledger keeps what they taught.) The **M15 build
target** is to stand the brain up as a *Marley-original* layer on exactly those pieces.

## Surface

```rust
pub enum AgentKind { Claude, Codex }                              // extensible
pub fn agent_kind_of(command: &str) -> Option<AgentKind>;         // recognize an agent CLI
pub fn launch_command(kind: AgentKind) -> &'static str;           // the inverse — the program to spawn
pub fn send_payload(line: &str) -> Vec<u8>;                       // line + a single `\r` (raw keystroke)

pub enum AgentStatus { Idle, Working, Waiting, Exited }           // #79 added Waiting
pub const WAITING_TICKS: u32 = 60;                                // ~1s of silence → Waiting
pub fn agent_status_from(exited: bool, active: bool, quiet_ticks: u32) -> AgentStatus;

pub struct AgentRun {
    pub kind: AgentKind, pub label: String, pub status: AgentStatus,
    pub last_line: String,      // #78 — most-recent non-empty output line (shown in the Fleet)
    pub quiet_ticks: u32,       // #79 — pump ticks since last output (drives Waiting)
    pub ticket: Option<u64>,    // #80 — the ticket this agent was last told to work
    pub run_ticks: u32,         // M12 #187 — pump ticks alive (elapsed = run_ticks / ~62; Date::now banned)
    pub exit_code: Option<i32>, // M12 #187 — None while alive, Some(code) after ChildExited (signal → -1)
}
impl AgentRun { pub fn new(kind: AgentKind, label: String) -> AgentRun }   // starts Idle, no output yet
```

- **`agent_kind_of`** — classify a command line by its leading program with any directory path stripped
  (`/usr/bin/claude` → `claude`) and arguments ignored (`claude --resume` → `Claude`); `None` for a
  non-agent or empty/whitespace-only command. Case-sensitive (Unix program names are).
- **`launch_command`** — the inverse of `agent_kind_of` (`Claude` → `"claude"`); round-trips (what we spawn
  is recognized back as the same kind).
- **`send_payload`** — the bytes to feed a composed line to a *running* agent: the line + a single `\r`
  (Enter for the foreground program). Note the `\r`, **not** `\r\n`: a running agent is a foreground
  program, so this is a raw keystroke stream to its stdin — the INVERSE of the #59/#65 cooked-buffer rule
  that governs a bare shell prompt where Marley owns the line. Multibyte-clean (`café\r`).
- **`agent_status_from`** — `exited` is checked **FIRST** (a stale `active` never overrides it, mirroring
  `block_status::exit_status_kind`); then `!active` → `Idle`; then `quiet_ticks >= WAITING_TICKS` →
  `Waiting` (probably at its prompt waiting for you, #79); else `Working`. `WAITING_TICKS = 60` pump ticks
  (~16 ms each ≈ 1 s).
- **`AgentRun`** — a running/finished agent terminal. Deliberately **no `Default`** (keeps a would-be
  whole-body mutant unviable). The tick-count fields (`quiet_ticks`, `run_ticks`) are Marley's clock: the
  `Date::now` ban (#82) means elapsed time is counted in pump ticks, not wall-clock.

## Surfaced by the cockpit (M2.C, SHIPPED)
`marley_app::agent_view` (pure) projects an `AgentRun` to its pane badge: `agent_status_glyph`
(● Working / ○ Idle / ✓ Exited — the ● chosen distinct from the ◐ of the since-retired Forge pane, #411) + `agent_badge(run)`. #66 renders it as a
corner pill on tagged panes; #67 drives the status LIVE (each pump tick recomputes `agent_status_from`, and
the pump's dead-pane close drops the agent tag); #68's ⌘⇧E **Fleet** overlay lists every agent via
`agent_rows(&agents)` (sorted by pane id) + `agent_status_label`. #78 surfaces `last_line`, #79 the Waiting
state, #80 the worked `ticket`, and M12 #187 the run duration (`run_ticks`) + done glyph (`exit_code`).

## Launch + observe (M2.B, SHIPPED)
The launch + observation are the `marley_app` shim. ⌘⇧A splits a pane, runs `launch_command(Claude)`
(`\r\n`-terminated, so it RUNS in the fresh shell + renders through the block path), and tags the new pane
in `RootView.agents: HashMap<PaneId, AgentRun>` (closing a pane drops its tag). The pump observes each
tagged session's delta to drive `agent_status_from`. (The Forge pane (#64) — RETIRED at #411 — once showed
the sprint's work alongside the agents working it; the sprint cockpit went with the scrap-forge rip.)

## Control (M2.D, SHIPPED) — observe → act
The shim tracks `RootView.last_agent: Option<PaneId>` (the last ⌘⇧A launch); **⌘⇧S** sends the FOCUSED
prompt's line to that agent's PTY via `write_bytes(send_payload(line))` and clears the prompt — but ONLY on
confirmed delivery (a closed target or a failed write preserves the input, never silently drops it). **⌘⇧G**
BROADCASTS the composed line to every running agent — `agent_pane_ids(&agents)` (pane ids sorted by id, in
`marley_app::agent_view`) gives the deterministic target set, and the shim `send_payload`s to each; the
prompt clears only once ≥1 agent received it (same confirmed-delivery rule).

## The road to the brain (M15+, intake — not built)
Extend the enum + `agent_kind_of`/`launch_command` as more CLIs are supported; then the *Marley-original*
brain grows on this seam (per the linked Warp subsystem's M15 target): (a) a **direct-provider streaming
client** (BYO-key Anthropic / OpenAI / custom endpoint — NOT Warp's `app.warp.dev` transport), (b) a
Marley-native action/result vocabulary reimplemented from the concept, (c) **MCP as the tool substrate**
(upstream `rmcp`; the `forge` sidecar this once meant to reuse went with the #409 scrap-forge pivot), and (d) the `EditOrigin::Agent` write path as the editor seam.
**Observe-first** is the cheapest first win — tap Marley's own typed event stream with no server
cooperation, exactly as the run-model above already does for a launched terminal.
