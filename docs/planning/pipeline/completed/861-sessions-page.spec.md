---
pipeline_id: 1cc6a550-609c-4276-8825-84393d72cfca
ticket: docs/planning/tickets/closed/TICKET-861-a-sessions-page.md
status: Phase 4 — Complete PASS
title: The Sessions page
type: feature
slice: The Marley look, W1.3 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
`marley: open sessions page` opens a Sessions tab: a header ("N on M hosts · A attached · D
detached", New Session), a table of every tmux session (session, host, windows, folder, state,
Attach, Close…), the line on how a session is placed in the rail, and the hosts Marley looks on
with their state.

## Scope
### In
- `sessions_page.rs` (new): the action, the tab; it follows the `TmuxHosts` global.
- `rail_sessions.rs`: Open as a Page under the rail's Sessions view.

### Out (explicitly deferred)
- The project a session is placed under, by name, in its row: the rail places it; the page names
  its folder.

## Reference (§20)
N/A — Marley-specific: Marley's tmux sessions (#819, #821, #832).

### Prior art
- `tmux_sessions`: `TmuxHosts`, `Listing`, `attach`, `kill`, `new_session_in`,
  `host_choices`, `error_words`, `report`, `list_soon`.
- The brain (consultation 4e3fb05a).

## UI proof
Scenario `script/e2e/861-sessions-page.sh` (sway): the run's own tmux server with two sessions,
both detached.
- `861-00-rail`: Open as a Page under the rail's Sessions view (REQ-003).
- `861-01-page`, opened from that link: the header's counts, the table, the placing line, the hosts (REQ-001).
- `861-02-closed`: Close… on a session, confirmed: its row gone (REQ-002).
- `861-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — Close asks first (tmux ends the session and every program in it).
- D2 — New Session starts in the window's first project folder, as the rail's + does.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Sessions page shall show how many sessions run on how many hosts and how many are attached, a row per session with its host, windows, folder, state, Attach and Close, and the hosts with their state, in Marley Night and Marley Day. | `861-01-page`, `861-03-day` |
| REQ-002 | WHEN the user confirms Close on a session, Marley shall end it and the page shall drop its row. | `861-02-closed` |
| REQ-003 | The rail's Sessions view shall offer Open as a Page, which opens the Sessions page. | `861-00-rail`, `861-01-page` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
