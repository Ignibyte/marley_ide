# TICKET-585 — The worktree's environment: its gitignored files, a setup command and its hook paths

- **Ticket:** LOCAL #585 (feature, prong 2: worktree agents, slice 2 of 2, part 1; after #510; the port offset is #590)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/585-worktree-environment.spec.md
- **Source ticket:** #510's split (`../../pipeline/completed/510-worktree-agents.notes.md`, "The split" and "Folded in from the Orca second pass"); Orca report 02 §2.2 to §2.4 and §3 items 6 and 7.
- **Status:** closed

## Summary
A worktree #510 makes is a clean checkout: none of the main checkout's gitignored files (`.env`,
local config), every dev server on the same port as the main checkout's, and no dependencies
installed. This slice gives it its environment. A `.worktreeinclude` at the main checkout's root,
read in `.gitignore` syntax as Claude Code reads it (through the `ignore` crate, already in
`Cargo.lock`), names gitignored files Marley copies into the new worktree before the agent starts,
with a size cap and the skipped entries named in a toast. Each worktree Marley made gets a slot,
and its terminals and tasks get `MARLEY_PORT_OFFSET` and `PORT` (3000 plus the offset), so its
dev server does not take the main checkout's port; `create_worktree` hook tasks also get
`MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH` beside Zed's `ZED_MAIN_GIT_WORKTREE`. When the
repository has no `create_worktree` task and one lockfile of one package manager, the prompt
offers that manager's install command, off by default, remembered per repository in `git config
marley.worktreeSetup`; checked, it runs in the worktree's terminal before the agent's command.

## Acceptance
A worktree agent's worktree has the `.worktreeinclude`d gitignored files of the main checkout,
its terminals and tasks carry a port offset of their own, the hook tasks get Marley's two paths,
and the setup command a lockfile suggests runs before the agent when the user checks it.

**Split at planning (2026-09-29).** The port offset (`MARLEY_PORT_OFFSET`, `PORT`) needs a Marley
hunk in both of Zed's terminal builders, a slot registry and its persistence, so it is TICKET-590.
This ticket keeps `.worktreeinclude`, the setup command and `MARLEY_ROOT_PATH` and
`MARLEY_WORKTREE_PATH` for tasks in a linked worktree, `create_worktree` hooks among them.
