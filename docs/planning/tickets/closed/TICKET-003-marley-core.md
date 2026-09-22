# TICKET-003 — marley_core

- **Forge ticket:** #5 `644bd932-eb18-4e13-9883-03983c2dbaae` (feature, M0)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `5bb453d9-7fc1-40d4-ad83-af3ffdf1aeaf`
- **Pipeline doc:** `../../pipeline/active/marley-core.spec.md`
- **Authoritative spec:** [SPEC-marley_core.spec.md](../../../specs/SPEC-marley_core.spec.md)
- **Branch:** `ticket-003-marley-core`
- **Status:** closed — delivered (forge #5 → done); GATE GREEN [full], cov 100 / MSI 100, 22 tests

## Summary

The offline-boot app/runtime core (R1–R18): `SessionId` (seam-contracts §2 owner),
the `~/.marley` path layout, a single lazily-built `Config` (no cloud/auth), and the
process feature-flag registry (3-layer resolution, RAII override guard, debug
init-guard). Composes the M0 leaves; `marley_terminal` binds `SessionId` downstream.

## Acceptance

All R1–R18 + 100% coverage on the testable surface + mutation MSI 100 on the runtime
surface (R9/R18 are compile-time structural, no viable mutant) + FULL gate green + a
§21 CHANGELOG entry & `docs/marley_architecture/marley_core.md`. The hard part:
global flag-state tests deterministic under the threaded mutation runner (serial +
reset). Full criteria in the pipeline spec (AC-A…AC-G).
