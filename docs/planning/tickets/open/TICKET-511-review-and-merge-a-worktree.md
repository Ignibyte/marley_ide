# TICKET-511 — Review and merge a worktree agent's branch

- **Ticket:** LOCAL #511 (feature, prong 2, worktree agents, review and merge; slice 1 of 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/511-review-and-merge-a-worktree.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 5, second half, of the list after the browser waves). On merging, the same day, answering the Orca survey's open question 4 (`docs/orca_architecture/README.md`; report 02 §5, question 2): "the rustal workflow will probably be where this lives. whatever default you want to set is fine but it should not impede the harness / workflow." The review, merge and removal parts are report 02 §2.6, §2.8 and §3 items 2 and 3.
- **Status:** open

## Summary
Once a worktree agent is done, its branch has to be reviewed and merged by hand. A worktree's row in the rail (#510) gets a menu: Review opens Zed's branch diff of the worktree against the base its branch recorded (`branch.<branch>.base`), and Merge makes a merge commit of the branch in the main checkout, after checks that fail closed, and never pushes. Where the Rustal workflow manages the project, Marley shows the branch's state and leaves the merge to the workflow, so it cannot get in its way; Marley knows such a project by `git config marley.merge workflow` or by the `workflow.toml` that `rw init` writes. Removing the worktree, with branch cleanup that recognizes squash merges and a teardown hook, is the second slice, a ticket of its own.

## Acceptance
Review shows the worktree's diff against its recorded base; Merge makes a merge commit in the main checkout, refuses with the reason when a check fails, aborts a conflict and pushes nothing; a project the Rustal workflow manages shows the branch's state and offers no Merge.
