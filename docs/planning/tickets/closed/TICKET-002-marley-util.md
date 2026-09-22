# TICKET-002 — marley_util

- **Forge ticket:** #4 `0da40c8a-da93-430e-9c04-0c7c86bbbd60` (feature, M0)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `0b16e47e-09c2-49be-bb60-37ab2971b025`
- **Pipeline doc:** `../../pipeline/active/marley-util.spec.md`
- **Authoritative spec:** [SPEC-marley-util.spec.md](../../../specs/SPEC-marley-util.spec.md)
- **Branch:** `ticket-002-marley-util`
- **Status:** closed — delivered (forge #4 → done); GATE GREEN [full], cov 100 / MSI 100

## Summary

The value-type vocabulary 16 crates bind to (seam-contracts §8): `FileId`,
`ContentVersion`, `HostId`, `StandardizedPath` + `standardize_path`,
`LocalOrRemotePath`, per SPEC-marley-util R1–R16. Leaf crate. The data foundation
for 003 and the HostId/local-vs-remote-path the retained remote seam binds to.

## Acceptance

All R1–R16 + 100% coverage (both path flavors via the flavor-parameterized core) +
MSI 100 + trybuild R13 (`#[non_exhaustive]`) + FULL gate green + a §21 CHANGELOG
entry & architecture-doc note. Full criteria in the pipeline spec (AC-A…AC-G).
