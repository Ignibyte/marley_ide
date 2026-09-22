# TICKET-001 — marley_text_offsets

- **Forge ticket:** #2 `b17cd9a8-1ca3-4f9d-95d3-39ae8fdb26b7` (feature, M0)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` (this session)
- **AAR:** `49f16b7b-26ec-4d47-910a-6e8f16e0cf59`
- **Pipeline doc:** `../../pipeline/active/marley-text-offsets.spec.md`
- **Authoritative spec:** [SPEC-text-offsets.spec.md](../../../specs/SPEC-text-offsets.spec.md)
- **Source ticket:** [M0-round-1.md → TICKET-001](../../../tickets/M0-round-1.md)
- **Branch:** `ticket-001-text-offsets` (stacked on 000)
- **Status:** closed — delivered (forge #2 → done); FULL gate green (cov 100 / MSI 100), receipt written

## Summary

The shared, mistake-proof text-position vocabulary: `CharOffset`/`ByteOffset`
newtypes (private `usize`, non-interchangeable) + the forward-only streaming
`CharCounter` byte→char converter, per SPEC-text-offsets.spec.md R1–R22. Sole
owner of the offset types (seam-contracts §1). The first real Marley crate — and
the one that first drives the FULL `scripts/gates.sh` to `GATE GREEN [full]`
(100% coverage + mutation MSI 100 + the receipt).

## Acceptance

All R1–R22 (the spec is the contract) + 100% line coverage on touched + mutation
MSI 100 (the spec's enumerated mutants) + the R2 `trybuild` compile-fail + FULL
gate green. Full criteria in the pipeline spec (AC-A…AC-G).
