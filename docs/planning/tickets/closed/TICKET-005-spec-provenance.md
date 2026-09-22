# TICKET-005 — marley_spec_provenance

- **Forge ticket:** #8 `e1d6c657-efed-4aa8-b793-715c212b0c8c` (feature, M0)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `e247239e-99c3-41a4-af2e-a9951b3ac8d0`
- **Pipeline doc:** `../../pipeline/active/marley-spec-provenance.spec.md`
- **Authoritative spec:** [SPEC-gate.spec.md](../../../specs/SPEC-gate.spec.md)
- **Branch:** `ticket-005-spec-provenance`
- **Status:** in-progress (Phase 1 PASS → Design)

## Summary

The mutation-tested Rust replacement for the interim `scripts/spec-provenance.sh`
(quality-bar gate-16, spec-layer clean-room wall): scans `docs/specs/SPEC-*.spec.md`,
isolates each Public-surface section + `spec_source`, and reports fork-private
identifier reuse / transcription-level sources — fail-closed. Per SPEC-gate R1–R15.

## Acceptance

All R1–R15 + 100% coverage + mutation MSI 100 + **the golden self-test (live
`docs/specs/` → 0 violations)** + gate-16 rewired to the binary + FULL gate green + a
§21 CHANGELOG entry & `docs/marley_architecture/marley_spec_provenance.md`. The hard
part: the R5 warp-docs harvest tuned (conservative + allowlist) so the self-test stays
green. Full criteria in the pipeline spec (AC-A…AC-H).
