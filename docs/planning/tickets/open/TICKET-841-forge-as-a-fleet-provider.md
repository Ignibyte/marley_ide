# TICKET-841 — Forge as a fleet provider

- **Ticket:** LOCAL #841 (feature, the Marley look, W4.1)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [841-forge-provider.spec.md](../../pipeline/queued/841-forge-provider.spec.md)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W4.1. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** ForgeAgents
- **Size:** L
- **Status:** in-progress

## Summary

A `forge` provider in `marley_fleet` that calls Ignibyte Forge's MCP tools (`agent-list`, `ticket-list`, `sprint-current`, `bulletin-list`) and answers in the fleet contract's shape, so the Fleet panel shows forge's agents by host. Settings (proposed): `marley.forge.enabled`, `endpoint`, `projects`, `poll_seconds`; the bearer token from the system keyring, never `settings.json`. Running and silent past 3 minutes reads needs attention, forge's own rule.

## Acceptance

With forge set up, the Fleet panel lists forge's agents by host with their ticket and state; with it off, nothing reaches forge. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.

## Decision

Decision taken by default: Marley adapts forge's tools to the fleet contract (open decision 3). Chad can change it before the ticket is planned.
