---
pipeline_id: 807349f4-6171-4a1e-89b0-7282ca1f7787
ticket: docs/planning/tickets/open/TICKET-867-home-prompt-picks-its-project.md
status: Phase 4 — Complete PASS
title: Home's prompt picks its project
type: feature
slice: The Marley look, W2.2 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
Home's prompt box carries a project chip beside the agent chips: Home first, then the window's
open projects, then the recent ones. Start runs the picked agent with the task in the picked
project, opening a recent project first.

## Scope
### In
- `home_page.rs`: the chip (a menu of the projects), the picked project, and Start in it.

### Out (explicitly deferred)
- Remote projects among the recent ones: Home lists the local ones (#701).

## Reference (§20)
N/A — Marley-specific: Home's page (#701, #835).

### Prior art
- `render_prompt` and `start_prompt` (#835); Home's recent projects (`Recent`, #701), opened with
  `Workspace::open_workspace_for_paths`, which activates a project already open.
- `MultiWorkspace::workspaces` lists the window's projects.
- The brain (consultation 4db6bbab): nothing on this seam.

## UI proof
Scenario `script/e2e/867-home-prompt-picks-its-project.sh` (sway): two scratch projects, one
open beside Home and one recent; a stand-in agent CLI that prints its folder and prompt.
- `867-01-menu`: the chip's menu: Home, the open project, the recent one (REQ-001).
- `867-02-started`: the recent project picked, a task typed, Start: the project opened and the
  agent's terminal in it printing its folder and the task (REQ-002).
- `867-03-day`: Marley Day, the prompt with its chip (REQ-001).

## Locked-In Decisions
- D1 — Home's own group stays the first choice and the default; a pick holds until changed.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Home's prompt shall carry a project chip whose menu lists Home, the window's open projects and the recent ones, in Marley Night and Marley Day. | `867-01-menu`, `867-03-day` |
| REQ-002 | WHEN the user starts with a project picked, Marley shall start the agent with the task in that project's folder and group, opening a recent project first. | `867-02-started` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
