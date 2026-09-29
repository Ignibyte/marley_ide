# TICKET-560 — Which files a worktree agent's branch would conflict on, before anyone merges

- **Ticket:** LOCAL #560 (feature, prong 2 with the rail: worktree agents, on #510's rows)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/560-conflict-chip.spec.md
- **Source ticket:** The Orca second pass of 2026-09-25 (`docs/planning/design-notes/orca-second-pass-2026-09-25.md`), finding 2; Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** closed

## Summary
Nothing in Marley says whether a worktree agent's branch still merges cleanly, or how far its
base has moved since the branch was made. Orca answers both from three read-only git calls
(`merge-base`, `rev-list --count`, `merge-tree --write-tree`), with no pull request involved.
Marley runs the same three against the local base branch #510 records, off the main thread and
only when the branch tip or the base tip moves, and draws the answer as a chip on the worktree's
rail row: `12 behind main`, and `conflicts in 3 files` in the warning color, with the files in the
tooltip. A chip that reads "conflicts" before anyone merges is the collision that grows with the
number of agents on one repository.

## Acceptance
A worktree row shows how many commits its branch is behind its base while it is behind; when a
commit on the base changes a line the branch also changed, the row's chip names the number of
conflicting files within a few seconds and its tooltip lists them; when the worktree merges the
base, the chip goes; nothing Marley runs moves a ref, touches an index or a working tree, or
fetches; a git without `merge-tree --write-tree` shows no chip and logs one line.
