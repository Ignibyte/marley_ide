---
pipeline_id: d2b9afcc-fa59-4e96-b614-8759e21d336c
ticket: forge#68 (0690368e-6fbe-419d-aa20-13f51977a0f4) · local docs/planning/tickets/open/TICKET-068-fleet-overlay.md
aar_id: a577e1a8-9793-4265-8678-6c2f1aa0b05a
status: Phase 5 — Complete PASS
title: Fleet overlay (cmd-shift-e)
type: feature
milestone: M2.C
references:
  - crates/marley_app/src/agent_view.rs (PURE: agent_status_label + AgentRow + agent_rows)
  - crates/marley_app/src/keymap.rs (cmd-shift-e → toggle-fleet)
  - crates/marley_app/src/app.rs (SHIM: the Fleet overlay)
---

## Title
A Fleet overlay (⌘⇧E) listing every launched agent + its status — the agent counterpart of the ⌘⇧F
Forge pane. See your whole fleet at a glance.

## Scope
### In
- PURE (`agent_view.rs`, cov/MSI 100): `agent_status_label(AgentStatus) -> &'static str` (working/idle/
  exited); `AgentRow { label: String, glyph: &'static str, status: &'static str }`; `agent_rows(&HashMap<
  PaneId, AgentRun>) -> Vec<AgentRow>` — sorted by `id.0` (deterministic), one row per agent; empty→empty.
- PURE (`keymap.rs`, cov/MSI 100): cmd-shift-e → `toggle-fleet` + a keymap test.
- SHIM (`app.rs`, mutants::skip + cov-excluded): `RootView.fleet_open`; dispatch `toggle-fleet`; an overlay
  (mirrors the ⌘⇧F forge overlay) — a header "Agents (N)" + `agent_rows` rows, or "no agents running".

### Out
- Clicking a fleet row to focus/close that agent (later). Auto-refresh of the list (it reads the live
  `agents` map each render). Grouping/filtering. The forge↔agent join.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — cmd-shift-e (FREE — no keymap "e" binding, no hardcoded cmd-E; no shadow risk per
  PR-claude-new-chord-shadowed-by-hardcoded-key-001).
- D2 — sort by `id.0` (PaneId is `PaneId(pub u64)`) for a deterministic order (HashMap iteration is random).
- D3 — `agent_view` imports `crate::layout::PaneId` (a pure value type) — fine for a pure sibling module.
- D4 — reuse `agent_status_glyph` (#66) for the glyph; add `agent_status_label` for the text.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_status_label(s)` is called, it shall return "working"/"idle"/"exited" for Working/Idle/Exited. | unit |
| REQ-002 | WHEN `agent_rows(agents)` runs, it shall yield one AgentRow per agent (label/glyph/status) SORTED by pane id; an empty map → empty Vec. | unit (2 agents inserted out of id-order) |
| REQ-003 | WHEN the keymap is queried, cmd-shift-e shall map to `toggle-fleet`. | unit |
| REQ-004 (visual) | WHEN cmd-shift-e is pressed with agents running, an overlay shall list them with status. | self-test (launch agent(s), cmd-shift-e → capture) |
| REQ-005 | `scripts/gates.sh` GREEN, cov/MSI 100 on agent_view + keymap; app shim excluded. | gate |

## Phase Plan
- **P2** — `agent_status_label`/`AgentRow`/`agent_rows` + the keymap + the fleet-overlay shim, mutation
  targets, test plan.
- **P3** — agent_view additions + keymap + the app.rs fleet overlay.
- **P3.5** — critic: the label arms, the id-sort determinism, the field mapping, the keymap non-conflict
  (no cmd-E shadow), the overlay seam, mutants.
- **P4** — agent_status_label + agent_rows + keymap unit tests (cov/MSI 100) + the SELF-TEST (⌘⇧E lists the
  fleet) + gate GREEN.
- **P5** — docs, AAR, archive, close #68.
