---
pipeline_id: a8f018c0-eff6-4ee3-afb0-4fc1fb0a20ed
ticket: docs/planning/tickets/closed/TICKET-690-watch-a-harness-session-in-a-terminal.md
status: Phase 4 — Complete PASS
title: Watch a harness session in a terminal
type: feature
slice: phase 2 item 5 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/pipeline/completed/689-harness-writes-from-marley.spec.md
---

## Title
Each view in a harness session's tab gets Open. Marley runs the view's command, as the harness
gives it, in a new terminal of the tab's workspace.

## Scope
### In
- Open on each listed view, before Copy. It starts a new terminal in the tab's workspace through
  `agents::start_in_terminal` (#684's way), with the view's command line typed into its shell.
- `HarnessView` keeps its workspace, to open the terminal there.

### Out (explicitly deferred)
- A display-only observer fed by the harness's stream (`intake/harness-session-live-terminal.md`).
- A remote harness: its views name the remote `rh`, and running one here needs the command's
  SSH path. That waits for item 6's SSH handling.

## Reference (§20)
N/A — Marley-specific. The commands are the harness's (`docs/MCP.md`, "Surfacing a session").
The terminal is Marley's own, opened as #684 opens the Marley agent's.

### Prior art
- `agents::start_in_terminal` (#684), and `HarnessView`'s views (#689).
- The harness's views table (`docs/MCP.md`) and its TICKET-109, which gives agent seats views.

## UI proof
`script/e2e/690-watch-a-harness-session-in-a-terminal.sh`:
- `690-01-views`: the views with Open and Copy.
- `690-02-terminal`: the terminal running `rh attach`, showing the actor's pane.

## Locked-In Decisions
- **D1:** a typed line in a new shell terminal, as #684 does. Ending the view leaves the shell,
  and the user reads what it said.
- **D2:** local only. A remote harness is out (see Scope).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the views are listed, each shall have Open | shot `690-01-views` |
| REQ-002 | WHEN Open is clicked, the system shall start the view's command in a new terminal of the tab's workspace | shot `690-02-terminal` showing the actor's screen |

## Phase Plan
- **P1 Plan:** this spec.
- **P2 Code:** `harness.rs`, a review, and the gate.
- **P3 Test:** the scenario.
- **P4 Complete:** docs, ledger, close, archive, commit, push and install.
