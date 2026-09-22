# TICKET-411 — Remove marley_forge_client + the fleet-brain forge wiring (product-rip slice 2)

- **Ticket:** LOCAL #411 (chore, M30)
- **Tags:** M30, pivot, forge-client, fleet, product-rip, 384-lineage
- **Created:** 2026-08-09
- **Pipeline doc:** ../../pipeline/completed/411-forge-client-fleet-brain-rip.spec.md
- **Status:** closed (2026-08-09 — shipped; gate GREEN [diff]; the scrap-forge pivot is complete)

## Summary
Second slice of the scrap-forge product rip — runs AFTER TICKET-410 (the seam
map's dependency order: post-410, `marley_app`'s only remaining forge_client edges
are the sprint/fleet imports and the probe pair). Remove the forge product surface
and the crate, preserving the generic seams:

- **Re-home, don't delete:** `probe_web_origin` + `ProbeOutcome` +
  `PROBE_*_TIMEOUT_MS` (the #406 browser-lifecycle failure detector — zero forge
  semantics) → `marley_app/src/browser_probe.rs` per the seam map's fit analysis;
  its 8 unit + 12 livewire tests move with it.
- Delete the sprint UI: `forge_view.rs`, the sprint overlay, `refresh_forge`,
  `claim_forge_ticket`/`comment_focused_on`, the `toggle-forge` ⌘⇧F verb + keymap
  rows, `RightSection::Forge` + its settings key + codec arm, the status-bar
  sprint segment.
- Delete the fleet's forge transport: `FleetSubscription`/`ConnectionState`
  wiring, `endpoint_for_brain` bearer plumbing, `MisconfigArm::ForgeClientMissing`
  + `NO_FORGE_CLIENT_REASON`, the four dead write paths. The fleet RAIL stays
  (forge-agnostic by design, fleet_rail.rs:9): snapshot model + demo feed +
  quiet-Unconfigured until a future non-forge brain endpoint.
- Delete `crates/marley_forge_client` + the Cargo edge; prose pass over
  `marley_mcp`'s 6 doc-comment mentions.
- **Inherited from 410's promotion (Explore corrections, 2026-08-09):**
  - Delete `marley_app/src/mcp_config.rs` + its 9 unit tests — 410 could not (its
    boot call site app.rs:2476 feeds the sprint `forge_client` and
    `endpoint_for_brain`, both dying here).
  - The Browser section's cockpit-Forge vocabulary — ＋-menu/context-menu default
    `OpenCockpit(Forge)` rows (context_menu.rs:160-166, tabs.rs:111-115) — dies
    with `RightSection::Forge` (neither ticket had enumerated it).
- CONSTITUTION preamble product words (the documented 409 exceptions) retire in
  this ticket's amendment commit; §21 architecture docs
  (`marley_forge_client.md`, `fleet-control-plane.md`, crate-map, 00-overview)
  updated to the as-ripped shape.

Test blast radius (seam map): ~105 forge_client-crate tests retire (20 probe
tests move), ~45 marley_app forge tests + 14 headless rows retire/repoint;
`marley_fleet` and the visual harness untouched.

## Acceptance
`marley_forge_client` gone from the workspace; the fleet rail renders
Unconfigured/demo cleanly; the probe pair lives in `browser_probe.rs` at cov/MSI
100; zero forge references in live product code or the CONSTITUTION preamble.
Full EARS at promotion.
