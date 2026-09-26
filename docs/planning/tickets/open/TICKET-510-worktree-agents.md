# TICKET-510 — Worktree agents: an agent on its own branch and worktree

- **Ticket:** LOCAL #510 (feature, prong 2, worktree agents, slice 1 of 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/510-worktree-agents.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 5, first half, of the list after the browser waves). Revised the same day, when he answered the Orca survey: worktree agents are rows nested under their project in the rail (open question 3 of `docs/orca_architecture/README.md`; report 02 §5, question 1; the create path is report 02 §2.1 to §2.5 and §3 items 1, 6 and 7, report 01 §3 item 6, report 06 §3 item 4).
- **Status:** open

## Summary
Two agents in one checkout overwrite each other's files. A project's + in the rail gains New Agent in Worktree: Marley asks for the agent's first prompt, has Zed's own worktree service make a git worktree on a new branch `agent/<name>` from the main checkout's branch, with no upstream, writes that base into the repository's git config as `branch.<branch>.base` for #511, and starts the agent in the worktree with the prompt on its command line, where no trust dialog can eat it. The rail shows each of the project's linked worktrees as a row nested under the project, with its branch, and the terminals of an open worktree under that row; Claude Code's own scratch worktrees under `.claude/worktrees/` get no row. The worktree's environment (the `.worktreeinclude` copy and a port offset) is the second slice, a ticket of its own.

## Acceptance
New Agent in Worktree creates a worktree on `agent/<name>` with no upstream and its base in `branch.<branch>.base`, starts the chosen agent there with the first prompt as one argument, and the rail shows the worktree nested under its project with the agent's row under it; a worktree under `.claude/worktrees/` has no row.
