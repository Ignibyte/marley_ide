---
pipeline_id: cee178e8-942b-405a-ad8b-65f02a8d42f0
ticket: docs/planning/tickets/open/TICKET-778-rustys-agent-in-the-title-bar.md
status: Phase 4 — Complete PASS
title: Rusty's button in the title bar opens Rusty's agent
type: feature
slice: the Marley layout (Marley and Rusty in the title bar, #746)
references: [738, 739, 746]
---

## Title
The title bar's Rusty button opens Rusty's agent conversation, as the Marley button beside it
opens Marley's, and Rusty's home page gets its button back in the status bar beside Home (Chad,
2026-10-12: "The top right rusty button is suppose to open the rusty agent not the rusty home page
that goes back to the bottom").

## Scope
### In
- The title bar's Rusty button runs Talk to Rusty: Rusty's latest conversation that is not
  archived comes forward, or a new one starts in a tab of the shown group. It shows while Rusty's
  agent entry is there and AI is on; its tooltip names the action and its key.
- A Rusty button at the status bar's right, beside Home, opens Rusty's home page in a tab of the
  shown group, as #739's did. It shows while Rusty is on and connected.
- The in-app guide page's sentence about these buttons.

### Out (explicitly deferred)
- Which agent Rusty runs on and the folder it starts in (#780).
- Permission modes for Rusty's conversations (#779).

## Reference (§20)
N/A — Marley-specific: the buttons open Marley's own entries and pages. The title bar seam is
#746's `MarleyTitleBarButtons` hook in Zed's `title_bar` crate, which this ticket does not change;
the status bar item follows Zed's `StatusItemView`, as Home and Debug do.

### Prior art
- Behavior maps: none apply (`docs/zed_architecture/` describes the title bar and status bar
  seams #739 and #746 already use).
- Published material: none needed.
- The code we ship: #739's status-bar Rusty button (`c744b8d0fc`, `rusty::open_home_here`), #746's
  `TitleButtons` and `HomeButton` (`status_buttons.rs`), `assistant::talk_to(Entry::Rusty, …)` and
  `TalkToRusty`, `assistant::marley_present` as the model for the presence check. Nothing new is
  needed: every piece exists.

## UI proof
`script/e2e/778-rustys-agent-in-the-title-bar.sh` (`compositor sway`, #746's set-up: the Marley
entry on a scripted agent, Rusty on through its stand-in `rusty-mcp`, the copy's thread list
emptied):
- `778-01-bars`: the title bar holds Rusty and Marley beside the `?`; the status bar's right end
  holds Home and Rusty (REQ-001, REQ-003).
- `778-02-title-rusty`: the title bar's Rusty opened a Rusty conversation in a tab, "Message
  Rusty" in its box (REQ-001).
- `778-03-status-rusty`: the status bar's Rusty opened Rusty's home page in a tab (REQ-002).
- `778-04-rusty-off`: Rusty turned off: neither Rusty button, Marley and Home stay (REQ-004).

## Locked-In Decisions
- D1 — The title bar's Rusty runs the same `talk_to(Entry::Rusty, false, …)` as `rusty: talk to
  rusty`, through a `talk_to_rusty` beside `talk_to_marley`; its presence is the Rusty entry's
  (`rusty_present`, as `marley_present`), so it never shows a button that does nothing.
- D2 — Rusty's home page button is drawn by the status bar's `HomeButton` view, after Home, as
  #739 placed it: one status item, not a second one to order.
- D3 — The status bar's Rusty keeps #739's rule (Rusty on and connected) and tooltip ("Rusty");
  the title bar's tooltip is "Talk to Rusty" with its key, as Marley's reads "Talk to Marley".

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks the title bar's Rusty button, the system shall bring forward Rusty's latest conversation or start one in a tab of the shown group. | shots `778-01-bars`, `778-02-title-rusty` |
| REQ-002 | WHEN the user clicks the status bar's Rusty button, the system shall open Rusty's home page in a tab of the shown group. | shot `778-03-status-rusty` |
| REQ-003 | WHILE Rusty is on and connected, the system shall show a Rusty button at the status bar's right, beside Home. | shot `778-01-bars` |
| REQ-004 | WHILE Rusty is off, the system shall show neither Rusty button. | shot `778-04-rusty-off` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `status_buttons.rs`, `assistant.rs`, the in-app guide page; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
