# TICKET-370 — marley_mcp: the expose-side MCP server — fleet read slice + session.surface_to_human

- **Forge ticket:** #370 (5b1e4eda-2024-49b4-b981-f7a4f38abbf5) (feature, M23)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/370-marley-mcp-server-l1.spec.md
- **Source ticket:** sprint "M23 — Fleet Control Plane: Layer 1" (orchestration-shell.md §12, Layer 1 item ④)
- **Status:** closed (shipped 2026-07-21; gate GREEN --diff 15/15; see pipeline/completed/370-marley-mcp-server-l1.notes.md)

## Summary
NEW crate `marley_mcp` — Marley's own MCP **server** (the expose direction of
`docs/marley_architecture/orchestration-shell.md` §4), so the manager seat (an MCP client,
location-independent) can read the fleet and surface sessions to the human. The L1 slice is
deliberately read-heavy: `fleet.snapshot` (read tool returning the current `FleetSnapshot` — the
`marley_fleet` types ARE the schema, one seam / three consumers), the snapshot as a **subscribable
MCP resource** (the same contract shape Forge's push uses — one contract, N consumers), and the ONE
gated write `session.surface_to_human(id)` — chad's ask verbatim: *"opening new sessions so the user
can view."* Tool FAMILIES (fleet / session / editor / browser) and permission tiers (read = loose;
writes = per-tool-class grants, deny-by-default) are first-class from day one, so growing the surface
never means re-architecting the server. `headless_drive.rs` already proves Marley's whole surface is
drivable without a window; `marley_mcp` turns that lane into a stable, permissioned, external
protocol.

Depends on the concurrent siblings: **#367** (`marley_fleet` — `FleetSnapshot` + verb
request/receipt types) and **#371** (`[[mcp.servers]]` + permission-grant settings). This ticket
writes against their contracts; exact signatures are Phase-2 business.

## Acceptance
`tools/list` exposes exactly the L1 set; `fleet.snapshot` returns the current snapshot;
`resources/subscribe` on the fleet resource delivers `notifications/resources/updated` when the
snapshot changes; `session.surface_to_human` on a known id returns a typed receipt and provably
focuses/opens that session's surface (headless drive); an unknown id gets a typed refusal, never a
panic; an ungranted write is refused deny-by-default while a granted one proceeds; adding a tool
family is additive. Full EARS criteria live in the pipeline spec
(../../pipeline/queued/370-marley-mcp-server-l1.spec.md).
