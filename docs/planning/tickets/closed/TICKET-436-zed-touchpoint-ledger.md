# TICKET-436 — The Zed touchpoint ledger, enforced

- **Ticket:** LOCAL #436 (chore, workbench shell W0)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/436-zed-touchpoint-ledger.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
Chad's rule for the fork: stay out of Zed's crates, and record every change outside Marley-owned paths so upstream merges stay cheap. docs/marley/zed-touchpoints.md holds the record; this ticket makes it mechanical. gate:16 in script/gates.sh fails when a path that differs from the upstream base has no ledger row, or a row names a path that no longer differs. A Write/Edit hook blocks a change to a Zed path until its row exists, and the constitution names the ledger in §0, §14 and §21.

## Acceptance
A Zed path changed without a ledger row turns gate:16 red and is blocked at write time by the hook; with the row in place both pass. Full EARS in the pipeline spec.
