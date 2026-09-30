---
pipeline_id: dfc7a52c-eaf8-44cf-b34e-b813bf73a6e3
ticket: docs/planning/tickets/open/TICKET-608-agent-snapshot-in-the-fleet-panel.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The selected agent's snapshot under the Fleet panel's list"
type: feature
slice: prong 2, D20, wave 1; after #607
references: [docs/marley/fleet-contract.md, docs/planning/pipeline/queued/607-fleet-panel-with-pseudo-agents.spec.md, docs/planning/pipeline/completed/604-port-row-click-does-not-open.spec.md]
---

## Title
One click on an agent in the Fleet panel selects it and fills a snapshot below the list: what it
works on, where its run stands, what its host uses, the tokens it spent today, and its question.

## Scope
### In
- **Selection:** one click on an agent's row selects it (the row drawn selected), and Up and Down
  move the selection while the panel holds the focus. The selection is kept by agent id across
  refreshes, and drops when the agent leaves the list.
- **The snapshot**, below the list, from `work_agent` for the selected agent:
  - the header: name, runtime, state chip and how long it has been in that state
    (`state_since_ms`), host and folder;
  - the work item: key, title and the store's status word;
  - the phase strip: one segment per phase in the run's order, coloured by state (passed,
    active, failed, pending, skipped), with the active phase's name under it;
  - CPU and memory bars from the host snapshot (`ui::ProgressBar`), with the numbers;
  - tokens today, input and output;
  - the question with its options, shown read-only (answering is a later action).
- **Missing parts:** a section the provider's capabilities leave out is not drawn. A host with
  no snapshot yet shows the host's name and "no resources yet".
- **Resizable split:** the list above and the snapshot below, the snapshot at a third of the
  panel's height by default.

### Out (explicitly deferred)
- Opening the Agent tab: double-click, Enter and an Open button come with it (#609).
- Answering the question, or any other action.
- Resources from SSH (#610); here they come from the pseudo provider's host snapshots.

## Reference (§20)
N/A — Marley-specific: an inspector for Marley's own contract, with no Warp or Zed counterpart.
It follows Marley's own rule for rows that are places to look (#604: one click marks, two open),
and Orca's dashboard card (name, last messages, question, host badge,
`docs/orca_architecture/01-agents-and-sessions.md:266-272`).

### Prior art
- **Behavior maps:** #604's spec and AD-claude-604 (mark on one click, open on two); the Orca note
  above.
- **Published material:** none needed.
- **Code we already ship:**
  - `ui::ProgressBar` (`crates/ui/src/components/progress/progress_bar.rs:11-89`), with its fixed
    `h_2` height and its over-colour.
  - The rail's row style (`row_frame`, `row_card` in `crates/marley_workbench/src/rail.rs`).
  - `ui::Divider`, and the rail's section headers.
  - `marley_sdk`'s `AgentDetail` and `HostSnapshot` (#607).

## UI proof
The scenario `script/e2e/608-agent-snapshot-in-the-fleet-panel.sh` (sway), on the pseudo provider:
- clicks the working agent and shoots the snapshot (`working.png`: header, work item, phase strip
  with its active phase, CPU and memory bars, tokens today);
- clicks the agent that waits and shoots its question (`question.png`);
- clicks the agent with the failed gate (`failed.png`: the failed phase segment);
- presses Down and shoots the selection moved (`keys.png`).

## Locked-In Decisions
- D1 — The snapshot reads `work_agent` for the selected agent only, each poll while shown; the
  list keeps reading `work_agents`.
- D2 — A section the provider cannot fill is left out, never drawn empty.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks an agent's row, the Fleet panel shall select it and show its snapshot below the list. | Shot `working.png` |
| REQ-002 | WHILE an agent is selected, its snapshot shall show its state and for how long, its work item, its run's phase strip, its host's CPU and memory, and its tokens today. | Shot `working.png` |
| REQ-003 | WHILE the selected agent waits on a question, the snapshot shall show the question and its options. | Shot `question.png` |
| REQ-004 | WHEN a phase of the selected agent's run failed, the phase strip shall mark that phase as failed. | Shot `failed.png` |
| REQ-005 | WHILE the panel holds the focus, Up and Down shall move the selection. | Shot `keys.png` |
| REQ-006 | WHERE the provider's capabilities leave a part out, the snapshot shall not draw that section. | Review of the diff |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the split, the phase strip's drawing).
- **P2 Code** — the selection, the snapshot's sections; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
