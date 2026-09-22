---
pipeline_id: 0952948e-a75d-40a7-9a56-3b0ca1c4e993
ticket: forge#136 (a5c4b8e1-548f-4df0-8ef4-34fd6126c2d4) · local docs/planning/tickets/open/TICKET-136-agent-launch.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: launch an agent from the new-session affordance [M7]
type: feature
milestone: M7 — The Warp Top Bar & Sessions
references:
  - crates/marley_app/src/app.rs (SHIM: a 🧠 agent-launch icon → dispatch_action("new-agent"))
---

## Title
Launch an agent from the top bar — a 🧠 icon next to the "+" starts an agent session (not just a terminal).

## Scope
### In
- SHIM: a 🧠 agent-launch icon in the top bar next to the "+" → `dispatch_action("new-agent")` (the existing
  M2.D launch: split a pane, run the agent CLI, tag it as an AgentRun).

### Out
- New agent logic (reuse). A dropdown/menu (env-unverifiable). Agent-kind selection (Claude default).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — reuse the tested `"new-agent"` dispatch verbatim; the icon just calls `dispatch_action("new-agent")`.
- D2 — glyph 🧠 (distinct from the cockpit Agents 🤖 icon), placed after the #135 "+" in the top-bar left.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN the app renders, a 🧠 agent-launch icon shall show in the top bar next to the "+". | live capture |
| REQ-002 | WHEN the icon is clicked, it shall launch an agent session (via the tested `"new-agent"` dispatch). | code-review (click env-blocked) + existing path |
| REQ-003 | FULL gate GREEN (shim-only). | gate |

## Phase Plan
- **P2** — the icon render → dispatch_action("new-agent"); note shim-only.
- **P3** — implement (app.rs).
- **P3.5** — 1 self-review: the icon wires to the new-agent path; placement.
- **P4** — a LIVE capture (🧠 next to +) + gate GREEN.
- **P5** — docs, AAR, archive, close #136; **close M7 sprint #18**.
