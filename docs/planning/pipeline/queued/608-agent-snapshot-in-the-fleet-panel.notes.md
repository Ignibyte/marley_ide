# The selected agent's snapshot under the Fleet panel's list — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-608-agent-snapshot-in-the-fleet-panel.md
- **Pipeline spec:** 608-agent-snapshot-in-the-fleet-panel.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-604: a row that is a place to look marks on one click and opens on two.
  - ui::ProgressBar has a fixed h_2 height and an over-colour past its max.
  - Sections a provider cannot fill are left out, never drawn empty (the contract's capabilities).
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.
