---
pipeline_id: 08a56455-7c06-42fe-a54b-e718410f1c84
ticket: docs/planning/tickets/open/TICKET-892-browser-recordings-send-and-downloads.md
status: Phase 4 — Complete PASS
title: The Browser drawer's Recordings, and Send to the agent
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Browser drawer (#887) gains a Recordings tab, the recordings Record This saved, each with its
title, address, time, length and Send; and the Console's Send to the agent. Send types a line
into the terminal used last, as a pick's Send does, naming what the agent reads next
(`browser_console` on this tab, or `browser_recording` with the recording's id).

## Scope
### In
- `browser.rs`: `Drawer::Recordings`, the list read off the main thread when the tab opens,
  `console_line`, `recording_line`, and `send_line` (the terminal half of `send_pick`).

### Out (explicitly deferred)
- The download bar and a pick's "Goes with it" and "To": TICKET-896.

## Reference (§20)
N/A — Marley-specific: Marley's Browser (#499's recordings, #496's Send); the Browser and
BrowserTools boards.

### Prior art
- `send_pick` (the last terminal, the waiting-agent guard), `pick_line`;
  `recorder::list_in`, `recordings_dir`, `browser_tools::recordings`.
- The brain (consultation 8729c76b).

## UI proof
Scenario `script/e2e/892-drawer-recordings-send.sh` (sway): an offline Chromium; a page that
logs an error; the terminal used first.
- `892-01-recordings`: Record This, then the drawer's Recordings: the saved recording (REQ-001).
- `892-02-sent`: the Console's Send to the agent: the line in the terminal (REQ-002).
- `892-03-day`: Marley Day, Recordings (REQ-001).

## Locked-In Decisions
- D1 — Send refuses, saying why, when no terminal was used or its agent waits on the user, as a
  pick's Send does.
- D2 — The list is read each time the Recordings tab is chosen; never on a timer.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Browser drawer shall list the saved recordings on a Recordings tab, each with its title, address, time and length and a Send. | `892-01-recordings`, `892-03-day` |
| REQ-002 | WHEN the user clicks the drawer's Send to the agent, Marley shall type into the terminal used last a line naming what the agent should read. | `892-02-sent` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
