# The fleet contract's types, a pseudo provider, and the Fleet panel's list — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-607-fleet-panel-with-pseudo-agents.md
- **Pipeline spec:** 607-fleet-panel-with-pseudo-agents.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-456 and F-claude-456: the Marley layout moves the Agent Panel between docks and restores what each dock showed; a panel sharing the right dock is subject to it.
  - Dock activation priorities must be unique per dock (dock.rs:784-796); 0 to 7 are in use.
  - marley_fleet: the same six states, serde conventions to copy, no run/phase/gate/event/usage/host types.
  - Brain: the contract decisions are D20 and docs/marley/fleet-contract.md's Settled list.
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.
