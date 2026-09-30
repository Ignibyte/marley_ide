# TICKET-607 — The fleet contract's types, a pseudo provider, and the Fleet panel's list

- **Ticket:** LOCAL #607 (feature, prong 2 D20: the fleet, wave 1)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/607-fleet-panel-with-pseudo-agents.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Status:** closed

## Summary
The Rustal services are not ready, so the fleet starts from Marley's side: the `v1` contract types in a new pure crate `marley_sdk`, a pseudo provider that serves the contract's example data and moves it, and a Fleet panel in the right dock that lists agents by host with their state, work item, phase and attention.

## Acceptance
With the pseudo provider set, the Fleet panel lists three agents under two hosts, follows their changes, marks the quiet one stale, and says how to set the fleet up when no provider is set.
