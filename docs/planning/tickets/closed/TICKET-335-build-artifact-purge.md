# TICKET-335 — chore: purge build artifacts at pipeline end — and stop cargo-mutants leaking multi-GB temp trees

- **Forge ticket:** #335 fc1a613d-3273-47ed-ab0c-83713de67b8b (chore, M24)
- **Owner:** unassigned — claimed at /work promotion
- **AAR:** PENDING — aar-open runs at /work promotion
- **Pipeline doc:** ../../pipeline/queued/335-build-artifact-purge.spec.md (promotes to ../../pipeline/active/ for Phase 2)
- **Source ticket:** PRE-EXISTING forge ticket (filed 2026-07-16 with the disk measurements), adopted into sprint #35 "M24 — Fleet Layer 2"
- **Status:** closed

## Summary
Two-part build-artifact hygiene. **(1) Fix the leak at source:** `mutation_g()` in
`scripts/gates.sh` runs `cargo mutants` in a bare subshell with no trap, so an interrupted
mutation run strands cargo-mutants' copy-mode scratch trees in `$TMPDIR`
(`cargo-mutants-Marley-*.tmp` — 3 orphans totalling 8.6G measured 2026-07-16 with no live
process); add EXIT/INT/TERM cleanup scoped to that invocation's own trees, with a per-invocation
scratch dir under `target/` as the leading mechanism (cargo-mutants has no scratch-dir flag, but
its tempfile-crate placement honors `$TMPDIR`, and `target/` is never copied into the tree copy).
**(2) Reclaim at rest:** a new `scripts/purge-build-artifacts.sh` (with `--dry-run`, a
pgrep live-cargo-mutants guard, and per-path byte reporting) purges ONLY the verified-safe
classes — orphaned cargo-mutants trees, `target/llvm-cov-target`, `target/release`
(bundle-app.sh defaults debug), `target/tests/trybuild`, `target/debug/incremental`,
`target/doc`, `mutants.out*` — and must NEVER touch `target/debug/deps` (~27G / 464k files, the
live cache keeping `cargo check` at ~7s). The /commit skill's closeout invokes the purge AFTER a
successful commit, advisory + non-fatal (deliberately NOT a Stop hook, NOT pre-gate — a purge
failure never fails a commit). cargo-sweep for the deps class is explicitly a SEPARATE future
decision, out of scope.

## Acceptance
Headline: an interrupted mutation run leaves no orphaned cargo-mutants tree; the purge script's
`--dry-run` reports exactly what the real run then reclaims (idempotent), refuses cargo-mutants
trees while a cargo-mutants process is alive, and never touches `target/debug/deps`
(post-purge `cargo check --workspace` stays warm); /commit's closeout runs the purge after a
successful commit and a purge failure never fails the commit; the mutation gate's verdict
semantics stay byte-identical. Gate-is-test verification (CONSTITUTION §7): gate exit codes +
negative smokes. The full EARS criteria (REQ-001..007) live in the pipeline spec.
