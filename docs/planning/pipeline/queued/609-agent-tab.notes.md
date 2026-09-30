# The Agent tab: an agent's full detail in the center — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-609-agent-tab.md
- **Pipeline spec:** 609-agent-tab.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - DecisionsView is the smallest read-only center item, found again rather than opened twice.
  - No sparkline exists in a Marley crate; git_graph.rs and circular_progress.rs draw with canvas and PathBuilder.
  - Ely-GPUI-Components (MIT OR Apache-2.0) charts/ may be read for scale and path code, rewritten against Zed's theme (evaluation of 2026-09-30).
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.
