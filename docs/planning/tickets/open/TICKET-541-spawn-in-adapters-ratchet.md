# TICKET-541 — Process spawns held to adapter modules by a ratchet

- **Ticket:** LOCAL #541 (chore, cross-cutting: the gate, CONSTITUTION §0 and §14)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/541-spawn-in-adapters-ratchet.spec.md
- **Source ticket:** Chad, 2026-09-25: specced at his request with every item decided that day (the brief quotes no words for this one). From the Orca survey: `docs/orca_architecture/07-engineering-and-changelog.md` §A7 and §3 item 3, and the README's item 10
- **Status:** open

## Summary
CONSTITUTION §0 and §14 say process spawns stay in adapter modules, and nothing checks it. Of the
five spawn calls in the Marley crates' library code today, two sit outside any adapter: the
workbench crate root's `run_program` and Voice's status follower. A new gate step, gate:22, finds
every spawn call in the Marley crates with a semgrep rule and holds them with the five devices of
Orca's ratchet tests: a list of the adapter modules kept as a data file, a failure when a listed
file no longer spawns, a pinned count that must equal the real one, a floor on the number of
files scanned, and a planted file of every spawn form that the check must catch. The two strays
move into a workbench adapter module first, so the list names adapters only.

## Acceptance
gate:22 passes on the tree, and fails on each of these, one at a time: a spawn outside the listed
modules, a stale line in the list, a count that differs from the pin, a scan that sees too few
files, and a rule that misses a planted form.
