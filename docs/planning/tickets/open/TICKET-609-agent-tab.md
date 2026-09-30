# TICKET-609 — The Agent tab: an agent's full detail in the center

- **Ticket:** LOCAL #609 (feature, prong 2 D20: the fleet, wave 1)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/609-agent-tab.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Status:** open

## Summary
Opening an agent gives it a center tab: its run's phases as a timeline with gates, the event log, its host's CPU, memory and network over time from samples Marley keeps, tokens for the run and the day, and the other agents on its host.

## Acceptance
Double-click, Enter or Open shows one tab per agent with the timeline, gates, events, three graphs and tokens; a second open shows the same tab.
