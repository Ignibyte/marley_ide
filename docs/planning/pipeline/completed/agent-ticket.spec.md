---
pipeline_id: 08bec086-8c90-49fb-95d3-153ce6074217
ticket: forge#80 (1b383541-41e3-44f6-9f56-956dafc8ffa1) · local docs/planning/tickets/open/TICKET-080-agent-ticket.md
aar_id: 0b0b6422-1e04-43a9-b3b9-65140962cf2d
status: Phase 5 — Complete PASS
title: agent ↔ ticket association (who's on what)
type: feature
milestone: M2.E — The Observing Cockpit
references:
  - crates/marley_app/src/agent_view.rs (PURE: extract_ticket_ref; AgentRow.ticket; agent_badge)
  - crates/marley_agent/src/lib.rs (AgentRun.ticket)
  - crates/marley_app/src/app.rs (SHIM: store on send/broadcast; Fleet render)
---

## Title
When you send/broadcast a line mentioning a ticket ("go work #77"), remember agent→#77 and show it in the
Fleet row + the pane badge — so you know who's on what.

## Scope
### In
- PURE `extract_ticket_ref(line) -> Option<u64>` (the first `#N` token); `AgentRun.ticket: Option<u64>`;
  `AgentRow.ticket` + agent_rows copies; `agent_badge` appends ` #{n}`.
- SHIM: send-to-agent (#72) + broadcast (#73) store the ref on the delivered agent(s); Fleet render shows ` · #{n}`.

### Out
- Verifying the ticket exists in the forge. Clearing the association. Auto-detecting from the agent's own
  output. Non-agent panes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `extract_ticket_ref`: `#` then a digit run (the FIRST such); a bare `#` / no digits / overflow → None.
- D2 — stored ONLY after a DELIVERED send/broadcast (mirror #72's clear-on-confirmed).
- D3 — shown as ` · #{n}` in the Fleet row + ` #{n}` in the badge.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `extract_ticket_ref(line)` runs, it shall return the first `#`-then-digits number, else None (bare #, no digits, overflow). | unit |
| REQ-002 | WHEN an agent has a ticket, its `agent_badge` + Fleet row shall show the `#N`. | unit + self-test |
| REQ-003 | WHEN a ticket-mentioning line is delivered to agent(s), their `AgentRun.ticket` shall be set. | self-test (env-blocked → wiring) |
| REQ-004 | gate GREEN, cov/MSI 100 on the pure surface; the shim masked. | gate |

## Phase Plan
- **P2** — extract_ticket_ref; AgentRun.ticket + AgentRow.ticket + agent_rows + agent_badge; the send/
  broadcast store + Fleet render; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: extract_ticket_ref MSI (first-#N, digit-run, empty/overflow); the badge/row render;
  the store-on-delivered at both sites.
- **P4** — the pure tests (cov/MSI 100) + gate GREEN + self-test (env-blocked → engine).
- **P5** — docs, AAR, archive, close #80.
