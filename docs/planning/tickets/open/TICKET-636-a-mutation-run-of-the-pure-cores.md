# TICKET-636 — A mutation run of the Marley crates' pure cores

- **Ticket:** LOCAL #636 (chore, wave 5: the test pass)
- **Owner:** claude-opus-5-5, 2026-10-01
- **Pipeline doc:** ../../pipeline/queued/636-a-mutation-run-of-the-pure-cores.spec.md
- **Source ticket:** wave 5 of `design-notes/remaining-work-2026-09-30.md`; AD-claude-the-workflow-is-four-phases-and-mutation-waits-for-the-end-001
- **Status:** open

## Summary
Mutation testing left the gate in 2026-09; the workflow keeps it for the end. This ticket runs
`cargo-mutants` over the Marley crates' pure cores (the crates with no gpui: `marley_terminal`,
`marley_fleet`, `marley_rail`, `marley_system_one`, `marley_sdk`, `marley_dcs`, and the pure modules
of the others, chosen at promotion), reports the survivors in a findings doc, and fixes any survivor
that is a real bug. Writing tests to kill survivors waits for Chad (an intake): the constitution
forbids new unit tests.

## Acceptance
A findings doc lists each crate's mutants, kills and survivors, each survivor read and marked
untested, equivalent or bug; every bug is fixed; the intake for killing survivors is written.
