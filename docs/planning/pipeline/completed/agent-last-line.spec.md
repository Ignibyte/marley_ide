---
pipeline_id: 75d8d96e-ec95-4318-836e-d34dd25977e4
ticket: forge#78 (ee4f5718-938f-49c1-a839-6d747f2508e1) · local docs/planning/tickets/open/TICKET-078-agent-last-line.md
aar_id: 039affa1-d314-4bc6-baad-329492b7c07f
status: Phase 5 — Complete PASS
title: an agent's last output line in the Fleet overlay
type: feature
milestone: M2.E — The Observing Cockpit
references:
  - crates/marley_app/src/agent_view.rs (PURE: agent_last_line; AgentRow.last_line)
  - crates/marley_agent/src/lib.rs (AgentRun.last_line field)
  - crates/marley_app/src/app.rs (SHIM: the pump reads the pane output; the Fleet render)
---

## Title
Show each agent's most-recent output line in the ⌘⇧E Fleet overlay, so you see what every agent is doing
at a glance without focusing its pane. The core OBSERVE step of M2.E.

## Scope
### In
- PURE `agent_last_line(output, max)` (marley_app/agent_view.rs, cov/MSI 100) — the last non-empty line,
  trimmed + char-truncated to `max` (+ '…' when cut); empty/all-blank → "".
- `AgentRun.last_line: String` (marley_agent, data field).
- SHIM (app.rs, masked): the pump reads each agent pane's output → `agent_last_line` → `run.last_line`;
  `AgentRow.last_line` + `agent_rows` copies it; the Fleet row renders it (muted).

### Out
- Parsing claude's TUI semantics (best-effort last non-empty line). A scrolling transcript. Live per-char
  streaming (per-pump-tick refresh is enough). Non-agent panes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `agent_last_line` is the pure surface (cov/MSI 100): rev-scan for the last non-empty line, trim,
  CHAR-truncate (multibyte-safe) with '…' only when actually cut; empty/all-whitespace → "".
- D2 — the last line lives on `AgentRun.last_line` (a data field), refreshed each pump tick (alongside
  #67's status refresh) from the pane's `content_row_texts` — so `agent_rows` (pure) just copies it.
- D3 — max ≈ 60 chars (claude's output is noisy → truncate hard).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_last_line(output, max)` runs, it shall return the last non-empty line, trimmed, char-truncated to `max` (+ '…' iff cut). | unit |
| REQ-002 | WHEN the output is empty or all-whitespace, `agent_last_line` shall return "". | unit |
| REQ-003 (visual) | WHEN an agent produces output, its Fleet row shall show its last line. | self-test (env-blocked → engine) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on agent_last_line; the shim masked. | gate |

## Phase Plan
- **P2** — agent_last_line signature/edges; AgentRun.last_line; the pump wiring + AgentRow + render; tests.
- **P3** — implement (agent_view.rs + marley_agent + app.rs).
- **P3.5** — 1 critic: agent_last_line MSI (rev-scan, char-truncation multibyte + exactly-max, empty); the
  field + pump wiring.
- **P4** — agent_last_line tests (cov/MSI 100) + gate GREEN + self-test (env-blocked → engine).
- **P5** — docs, AAR, archive, close #78.
