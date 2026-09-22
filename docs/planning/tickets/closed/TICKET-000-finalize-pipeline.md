# TICKET-000 — Finalize the Marley pipeline

- **Forge ticket:** #1 `773e5990-f7c4-40fa-8e16-a02846d27bcd` (chore, M0)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` (this session)
- **AAR:** `50fc22d0-9c2d-4c92-ac25-7563e0a9b079`
- **Pipeline doc:** `../../pipeline/active/finalize-marley-pipeline.spec.md`
- **Source ticket:** [M0-round-1.md → TICKET-000](../../../tickets/M0-round-1.md)
- **Status:** closed — delivered (forge #1 → done); pipeline green + 5/5 gate smokes

## Summary

Port the inherited Ignibyte-IDE pipeline apparatus (gate + hooks + CONSTITUTION +
supply-chain config) to Marley's gpui/crates workspace so every quality gate of
[quality-bar.spec.md](../../../specs/standards/quality-bar.spec.md) actually
enforces. Gate-is-test infra change (no `.rs`); verification = gate exit codes +
negative smokes. First of the M0 run (000 → 001).

## Acceptance

The full EARS criteria live in the pipeline spec (REQ-001…REQ-015). Headline:
`scripts/gates.sh --fast` prints `GATE GREEN [fast]`; prettier dropped; coverage
floor 100% whole-workspace; mutation whole-workspace MSI 100; gate-15 wired
conditional + fail-closed; gate-16 wired; deny allowlist permissive minus MPL-2.0
with crate `license.workspace = true`; hooks bite on `crates/`; CONSTITUTION
adapted to Marley with a clean-room provenance §. The FULL-gate green is proven by
TICKET-001 (first real testable code).
