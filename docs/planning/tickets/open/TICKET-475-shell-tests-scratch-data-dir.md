# TICKET-475 — The shell tests install Marley's scripts in a scratch directory

- **Ticket:** LOCAL #475 (chore, prong 1: T0 test hygiene)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (found in #474's Test phase)
- **Source ticket:** ../../pipeline/completed/474-block-hover-actions.notes.md (Phase 3)
- **Status:** open

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
