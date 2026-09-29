# TICKET-531 — Pull request state and diff counts on the rail's rows

- **Ticket:** LOCAL #531 (feature, workbench shell: the rail's rows; Warp once-over item 7)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/531-pr-state-on-rail-rows.spec.md
- **Source ticket:** Chad, 2026-09-25: "yes" to Warp's vertical tabs metadata (item 7 of `docs/planning/design-notes/warp-once-over-2026-09-25.md`: "Rows can show the branch's pull request, its status (through the GitHub CLI) and diff stats")
- **Status:** closed

## Summary
The rail shows which agents are working but not whose work is ready, and with worktree agents
coming (#510) that is the question Chad asks first. Each project row gets two things: the lines
added and removed on its branch against the branch's base, uncommitted edits to tracked files
included, counted by git through Zed's own git layer; and, when the branch has a pull request on
GitHub, a chip with the PR's number and state (open, draft, merged, closed), found through the
`gh` CLI. A project without a GitHub remote, or a box without `gh`, shows the counts alone.
Worktree rows take the same chip and counts when #510 draws them.

## Acceptance
A project row shows its branch's added and removed lines against the base and, when the branch
has a pull request, a chip with its number and state; both follow a branch switch and new edits,
and a missing `gh` shows no error.
