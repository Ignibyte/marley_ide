# TICKET-469 — gate:4 reads other crates' stale test executables

- **Ticket:** LOCAL #469 (chore, the gate)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/469-coverage-reads-only-this-runs-executables.spec.md
- **Source ticket:** ../../pipeline/completed/465-zsh-shell-integration.notes.md (Phase 3)
- **Status:** closed

## Summary
gate:4 runs `cargo llvm-cov nextest` on the touched Marley packages. cargo-llvm-cov cleans those
packages' artifacts first, but it reads every workspace test executable left in
`llvm-cov-target/debug/deps`, other packages' included. In #465's run, `marley_workbench`'s test
executable from #468's run still linked the `marley_terminal` of that time.
Its line map for `shell_integration.rs` belonged to the old file, so gate:4 reported 45 missed
lines in `marley_terminal`, on doc comments and a derive. Removing that one executable made the
report match the tree. The gate must not depend on what an earlier run left behind.

## Acceptance
A coverage run reports only what this run's executables built from this tree: an executable
another package's earlier run left in the coverage target cannot add or hide a missed line. A
negative smoke plants such a stale executable and the gate stays true to the tree.
