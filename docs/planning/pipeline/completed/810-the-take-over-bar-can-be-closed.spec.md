---
pipeline_id: 5d44c85f-6a3b-4f90-869d-dcfa505876a7
ticket: docs/planning/tickets/open/TICKET-810-the-take-over-bar-can-be-closed.md
status: Phase 4 — Complete PASS
title: The Take Over bar can be closed
type: bug
slice: agents in terminals (#525) and the Browser tab (#767)
references:
  - docs/planning/pipeline/completed/767-file-choosers-downloads-and-take-over.spec.md
---

## Title
The bar an agent's writes leave under a terminal, and over a page, gets a close button, and Hand
Back closes it; the agent's next write or action shows it again.

## Scope
### In
- The terminal's bar (`terminal_drive::footer`): a close button while agents may write; Hand
  Back closes the bar; a write or a run opens it again. While the user has control the bar stays,
  with Hand Back and no close button, since the agents' writes are refused then.
- The page's bar (`BrowserView::control_bar`), the same three rules.
- Each bar's two flags become one state: agents in control with the bar shown, agents in control
  with the bar closed, or the user in control.

### Out (explicitly deferred)
- Who may type at all: #811's trusted clients.

## Reference (§20)
N/A — Marley-specific: Marley's own bars (#525, #767); Zed has neither.

### Prior art
- The code we ship: `Drive` and `toggle_control` (`terminal_drive.rs`), `PageState`,
  `BrowserHub::toggle_control` and `control_bar` (`browser.rs`).

## UI proof
Scenario `script/e2e/810-the-take-over-bar-can-be-closed.sh` (shots 810-01 to 810-09).

## Locked-In Decisions
- **D1 — No close while the user holds control**: a hidden "You have control" would hide that the
  agents' writes are refused.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the close button of a terminal's or a page's bar is clicked, the system shall hide the bar | shots `810-02-closed`, `810-07-page-closed` |
| REQ-002 | WHEN the user hands control back, the system shall hide the bar | shots `810-05-handed-back`, `810-09-page-handed-back` |
| REQ-003 | WHEN an agent writes, runs or acts again, the system shall show the bar again | shots `810-03-back`, `810-08-page-back` |

## Phase Plan
- P1 this spec; P2 the state and the buttons, gate; P3 the scenario; P4 CHANGELOG, guide, close.
