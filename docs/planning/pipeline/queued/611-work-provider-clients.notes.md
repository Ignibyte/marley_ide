# The marley.work/v1 clients over MCP and HTTP — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-611-work-provider-clients.md
- **Pipeline spec:** 611-work-provider-clients.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - #534's spec plans the same ContextServer::stdio client for the harness.
  - No Marley crate depends on context_server yet; Zed's agent calls tools at context_server_registry.rs:362-395 but reads content, not structured_content.
  - system_one.rs is the http_client precedent (bearer header, timeout, backstop timer).
  - marley_mcp's rule: a bearer never enters a log, a prompt or a file other than the user's own config.
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.
