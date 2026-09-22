# richer agent status — waiting-for-input — Notes

- **Forge ticket:** #79 `ece6e649-2dbd-4540-bc85-14f58bfff70d` · **AAR:** `f2380298-2fa3-428a-a819-532dc70d15ed`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-079-agent-waiting.md

## Phase 1 — Plan
- **Request:** forge #79 (M2.E 6/15) — a Waiting agent status (running but quiet).
- **Classification:** work pipeline, `feature`, PURE (marley_agent) + agent_view glyph/label + app.rs SHIM.
- **Pre-flight:** `AgentStatus{Idle,Working,Exited}`; `agent_status_from(exited, active)` (marley_agent:59);
  glyph/label in agent_view.rs (exhaustive matches). #78's last_line reused for change-detection.
- **Decisions:** D1 arm order Exited→Idle(!active)→Waiting(quiet)→Working; D2 WAITING_TICKS=60; D3
  quiet_ticks resets on last_line change.
- **AAR id:** `f2380298-2fa3-428a-a819-532dc70d15ed`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- marley_agent: `AgentStatus::Waiting` (add variant, after Working); `pub const WAITING_TICKS: u32 = 60`; `agent_status_from(exited, active, quiet_ticks) = if exited {Exited} else if !active {Idle} else if quiet_ticks >= WAITING_TICKS {Waiting} else {Working}`; `AgentRun.quiet_ticks: u32` (new()=0).
- agent_view.rs: glyph Waiting → "\u{25d4}" (◔), label Waiting → "waiting" (both exhaustive matches).
- app.rs refresh loop: `if last_line != run.last_line { run.quiet_ticks = 0 } else { run.quiet_ticks = run.quiet_ticks.saturating_add(1) }` then `run.status = agent_status_from(false, running, run.quiet_ticks)`. Update the #67 caller + any test callers to pass a 3rd arg.
- **Mutation targets:** the 4-arm order, the `>= WAITING_TICKS` boundary, the glyph/label arms.
- **Test plan:** `agent_status_from_cases` (exited→Exited [even quiet+active]; !active→Idle; active+59→Working; active+60→Waiting) + `agent_status_glyph/label` gains the Waiting arm. Unit, cov/MSI 100. Self-test env-blocked → engine.
- **Risks:** the quiet reset uses #78 last_line-change as the activity signal (heuristic); WAITING_TICKS pub for testing.

## Phase 3 — Implement
- **Built:** AgentStatus::Waiting + WAITING_TICKS=60 + agent_status_from(exited,active,quiet_ticks) [4-arm] + AgentRun.quiet_ticks (marley_agent); glyph ◔ + label "waiting" (agent_view); the app.rs refresh resets quiet_ticks on last_line change else saturating_add(1), then agent_status_from(false,running,quiet_ticks).
- **Fixtures:** the agent_status_from test rewritten (4 arms + the 59/60 boundary); the 5 AgentRun literals got quiet_ticks (perl targeting AgentStatus:: lines).
- **Verification:** fmt; check --all-targets 0 err; clippy OK. (glyph/label Waiting arms need test coverage → validate.)

## Phase 3.5 — Inspect
- **Method:** self-review of a clean 4-arm pure-fn extension (scaled); the Phase-4 gate cargo-mutants is the authoritative MSI check.
- **Lenses — no correctness findings:** agent_status_from arm order (Exited→!active-Idle→quiet-Waiting→Working) + the `>= WAITING_TICKS` boundary (tested 59→Working/60→Waiting); glyph/label Waiting arms (need test cover → validate); the quiet_ticks reset heuristic (resets on the OLD last_line ≠ new, else saturating_add — overflow-safe); all agent_status_from callers updated (the #67 refresh + the marley_agent tests).
- **Fix applied:** 2 stale doc comments updated (agent_status_from + agent_status_glyph now mention Waiting).

## Phase 4 — Validate
- **Tests:** agent_status_from (4 arms + 59/60 boundary) [marley_agent]; agent_status_glyph/label gained the Waiting arm [marley_app]. `cargo nextest -E test(agent_status)` → pass.
- **Self-test (quiet agent → ◔):** ENV-BLOCKED → engine-tested (cov/MSI 100).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #79 → done. **M2.E 2/5.** AgentStatus::Waiting + agent_status_from(quiet_ticks) + ◔ glyph (cov/MSI 100). Self-test env-blocked; engine-tested.
