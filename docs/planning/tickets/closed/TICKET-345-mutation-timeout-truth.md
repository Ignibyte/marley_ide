# TICKET-345 — gate:5 truth: mutants under nextest + a Timeout must prove it was a hang

- **Forge ticket:** #345 `8af264da-17ea-4974-8dd0-bceb389d6730` (chore, M22)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `7a4476ce-188c-41dc-ab1c-d72133650546`
- **Pipeline doc:** ../../pipeline/active/345-mutation-timeout-truth.spec.md
- **Source ticket:** the M22 integrity-five shelf — ../../design-notes/integrity-five-shelf.md
- **Status:** closed

## Summary

`scripts/gates.sh:258-259` counts a `Timeout` mutant as CAUGHT. That is sound when the timeout means
"the mutant caused a hang" (Infection/Stryker convention: a hang IS detection); it is NOT sound when the
timeout means "the suite was slow today" — a genuinely undetected mutant that happens to time out is
laundered into MSI 100, the one thing the floor exists to make impossible. #337's run had 3 Timeouts,
all killed DECISIVELY by failing asserts, mislabeled only because plain `cargo test` doesn't fail-fast
and the wall clock blew cargo-mutants' deadline — MSI 100 was honest by luck of inspection. This ticket
makes the classifier tell the truth two ways: (1) run mutants under **nextest** (`--test-tool=nextest` —
fail-fast, process-per-test) so a killed mutant is detected in seconds, not laundered; (2) after the run,
**audit** every remaining `Timeout`'s `log_path` for failing tests and report `N timeout(s); M mislabeled`
so a mislabel is VISIBLE instead of silently absorbed. The `Timeout∈caught` convention stays; no floor
is lowered (§0) — the classifier is made honest.

Gate-is-test change: `scripts/gates.sh` gate:5 only, no `.rs`; verified via exit codes + negative smokes.

## Acceptance

Gate:5 runs its mutants under nextest in both full and `--in-diff` modes; a run with any `Timeout` prints
`N timeout(s); M mislabeled` sourced from each outcome's `log_path`; a genuinely MISSED mutant still fails
the gate red under the new runner; `MUT_MSI_MIN` and the MSI arithmetic are byte-identical. Full EARS
criteria (REQ-001 … REQ-006) live in the pipeline spec.
