# TICKET-475 — The shell tests install Marley's scripts in a scratch directory

- **Ticket:** LOCAL #475 (chore, prong 1: T0 test hygiene)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/475-shell-tests-scratch-data-dir.spec.md
- **Source ticket:** ../../pipeline/completed/474-block-hover-actions.notes.md (Phase 3)
- **Status:** closed

## Summary
The PTY tests of #463, #465 and #474 start their shells through `TerminalBuilder::new`, which
installs Marley's integration scripts in `paths::data_dir()/shell_integration`: on this box the
user's own `~/.local/share/marley`. Each run of `terminal`'s tests rewrites the scripts the
user's Marley terminals read, and a negative check that mutates a script leaves the mutant there
until the next run reinstalls it. The tests should install into a directory of their own, for
example through `paths::set_custom_data_dir` in each test process.

## Acceptance
A run of `terminal`'s tests leaves `~/.local/share/marley/shell_integration` as it was, and the
tests' shells still start with the scripts from the tree.

## Queued 2026-10-01
Wave 5 runs the tests again (#634), so this comes first.

## Deliberate since 2026-09-23
#483 took the tests out of the workflow: they stay in the tree and keep building, but no gate
runs them, so none of them writes to `~/.local/share/marley` in the normal flow. The fix waits
until the tests run again.
