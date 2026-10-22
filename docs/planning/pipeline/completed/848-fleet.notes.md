# The Fleet panel as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-848-fleet-and-harness.md
- **Pipeline spec:** 848-fleet.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #847; split while planning: the harness screens are
  TICKET-878.
- **Classification / tier:** feature, S–M. Marley crate only.
- **Recall (§18.3):** #841's Forge rows in the panel; #843's lettered tiles.
- **Discovery:** a probe run over the pseudo provider: the panel shows FLEET, the provider and its
  state, each host's name and resource line, each agent's runtime icon, name, work and state chip.
- **Decisions:** D1 in the spec.

### Design
- `fleet.rs`: a summary line under the panel's heading (agents and hosts across the sources, the
  providers' names); `render_source`'s host groups wrapped in bordered cards, the resource line at
  the header's right; `render_agent`'s icon replaced by a tile with `glyph(runtime)`.
- **File manifest:** `fleet.rs`; the scenario.

## Phase 2 — Code
- **Built:** `fleet.rs`: `summary(sources)` ("N agents on M hosts · the providers", none while no
  provider lists an agent) beside the panel's FLEET; each host group in `render_source` a bordered
  card on the editor background, its resource line at the header's right; `render_agent`'s
  runtime icon replaced by a tile with `runtime_glyph` (D1).
- **Gate:** green on the first run, 17 of 17.
- **Review:** selection, Open and the snapshot pane are unchanged.


## Phase 3 — Test
- **Scenario:** `script/e2e/848-fleet.sh` (sway): the pseudo provider.
- `848-01-fleet`: "FLEET 3 agents on 2 hosts · Pseudo provider"; build-1's card (CPU, memory,
  disk, network at the right) with build-1 (CC, RB-142 · code 2/4, working) and review-1 (CX,
  waiting); vps-2's card with docs-1 (CC, error). REQ-001 met.
- `848-02-day`: Marley Day, the same. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation be835a67.
- Ticket closed (its harness screens are TICKET-878); pipeline archived.
