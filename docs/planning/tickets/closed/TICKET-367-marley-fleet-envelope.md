# TICKET-367 — marley_fleet: the generic Session envelope + FleetSnapshot reducer (pure crate)

- **Forge ticket:** #367 d8e06631-eb00-4f8e-be7d-f2ce484be23b (feature, M23)
- **Owner:** ca2d1534-e512-4d84-92e8-b2f73b4955c9 (M23 train session)
- **AAR:** f1f217d1-e5e4-4f25-ae72-fb8c51a2e0f1
- **Pipeline doc:** ../../pipeline/active/367-marley-fleet-envelope.spec.md
- **Source ticket:** sprint "M23 — Fleet Control Plane: Layer 1" — ticket ① of the Layer-1 train
  ([orchestration-shell.md §12](../../../marley_architecture/orchestration-shell.md))
- **Status:** closed

## Summary
NEW pure crate `marley_fleet` — the one schema the fleet control plane's project-agnosticism lives or
dies on ([orchestration-shell.md §2–3](../../../marley_architecture/orchestration-shell.md)). It carries
the generic `Session` envelope (stable string id, closed state vocabulary with Error ≠ Idle first-class,
structured `question` iff Waiting, opaque ordered `labels`, `last_event` timestamp, transport render
hint), the `FleetSnapshot` (ordered seats + read-time attention derivation), the deterministic
replay-safe reducer (`SessionEvent` stream → snapshot), the Deposited→Claimed→Started delivery-state
machine types, and the session-verb request/receipt types as data. No gpui, no transport, no Forge
vocabulary — a Forge-specific string in this crate is a defect (it belongs in `marley_forge_client`).
One seam, three consumers: the UI model, the test surface, and the MCP tool schema. This ticket forces
the envelope v1 field-level decisions while they are cheap; #368 (adapter projection), #369 (fleet
rail), and #370 (MCP read-slice) consume these types.

## Acceptance
The crate compiles standalone at cov/MSI 100 with deps = serde only; the reducer is deterministic and
idempotent under cursor'd overlap replay; question-present ⇒ state==Waiting holds through every event;
staleness/attention derive purely from an injected `now` (no clock in the crate); delivery states are
monotone-join; the envelope + verb types serde round-trip. Full EARS criteria live in the pipeline spec.
