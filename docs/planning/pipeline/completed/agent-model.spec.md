---
pipeline_id: f455b448-691d-43a7-a4db-0f6a3b7f3108
ticket: forge#61 (57ebab5f-adab-4d76-a202-0ab33ab3e56b) · local docs/planning/tickets/open/TICKET-061-agent-model.md
aar_id: 0e27da39-7932-47a1-b222-f98ef51c28d6
status: Phase 5 — Complete PASS
title: marley_agent — the agent-run model
type: feature
milestone: M2.B
references:
  - crates/marley_agent/ (NEW crate — AgentKind/agent_kind_of/AgentStatus/agent_status_from/AgentRun)
---

## Title
The FOUNDATION of the agent cockpit: a pure `marley_agent` crate that recognizes an agent CLI and
represents a running agent (kind + label + status). #62 launches + tags a pane with an `AgentRun`; a
later ticket observes.

## Scope
### In (all in `crates/marley_agent/src/lib.rs`, gpui-free, std-only, cov/MSI 100)
- `pub enum AgentKind { Claude, Codex }` (Debug/Clone/Copy/Eq).
- `pub fn agent_kind_of(command: &str) -> Option<AgentKind>` — leading token → basename → match
  `claude`/`codex`; `None` for a non-agent or empty command; args ignored (`claude --resume` → Claude).
- `pub enum AgentStatus { Idle, Working, Exited }` (Debug/Clone/Copy/Eq).
- `pub fn agent_status_from(exited: bool, active: bool) -> AgentStatus` — `exited` FIRST → Exited; else
  `active` → Working; else Idle.
- `pub struct AgentRun { pub kind: AgentKind, pub label: String, pub status: AgentStatus }`
  (Debug/Clone/Eq, NO Default) + `AgentRun::new(kind, label)` (starts `Idle`).

### Out
- `WaitingInput` status (needs prompt/idle heuristics) — later. Agent CONTROL (sending input),
  observation/session-delta, the launch shim — #62+. More agent kinds — extend the enum + match later.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — recognize `claude` + `codex` (chad's default) — an extensible match; unknown commands → `None`.
- D2 — `agent_status_from` checks `exited` FIRST (state-first, mirroring `exit_status_kind` #36) — an
  exited agent is Exited even if a stale `active` lingers.
- D3 — NO `Default` on `AgentRun` (keeps the whole-body `Default::default()` mutant unviable, per #54/#55).
- D4 — a NEW crate `marley_agent` (the cockpit domain) — gpui-free, std-only, no deps.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_kind_of(cmd)` sees a leading program (path-stripped) of `claude`/`codex`, it shall return the matching `AgentKind`; any other leading program or an empty command → `None`. | unit (`claude`, `claude --resume`, `/usr/bin/claude`, `codex`, `ls`, ``) |
| REQ-002 | WHEN `agent_status_from(exited, active)` is called, it shall be `Exited` when `exited` (regardless of `active`), else `Working` when `active`, else `Idle`. | unit ((t,t)→Exited; (f,t)→Working; (f,f)→Idle) |
| REQ-003 | WHEN `AgentRun::new(kind, label)` is called, it shall hold that kind + label with `status = Idle`. | unit |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on marley_agent. | gate |

## Phase Plan
- **P2** — the crate skeleton + the three fns/types, mutation targets, the unit test plan.
- **P3** — the crate (`Cargo.toml` + `lib.rs`).
- **P3.5** — critic: the leading-token/basename edge cases, exited-first, the match arms, mutants.
- **P4** — the unit tests (cov/MSI 100) + gate GREEN. (Pure lib, no UI → no self-test.)
- **P5** — docs (crate-map + CHANGELOG + arch), AAR, archive, close #61.
