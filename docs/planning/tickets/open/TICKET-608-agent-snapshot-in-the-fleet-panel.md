# TICKET-608 — The selected agent's snapshot under the Fleet panel's list

- **Ticket:** LOCAL #608 (feature, prong 2 D20: the fleet, wave 1)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/608-agent-snapshot-in-the-fleet-panel.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Status:** open

## Summary
One click on an agent in the Fleet panel selects it, and the panel shows its snapshot below the list: state and for how long, work item, the run's phase strip, CPU and memory, tokens today and its question.

## Acceptance
Clicking an agent fills the snapshot; the waiting agent shows its question, the failed one its failed phase, and Up and Down move the selection.
