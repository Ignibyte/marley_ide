# TICKET-590 — A port offset for each worktree agent's worktree

- **Ticket:** LOCAL #590 (feature, prong 2, worktree agents, the worktree's environment, part 2; after #585)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/590-a-port-offset-per-worktree.spec.md
- **Source ticket:** TICKET-585, split at its planning (2026-09-29): #510's slice 2, "the worktree's environment"; Orca report 02 §2.4 and §3 item 7.
- **Status:** closed

## Summary
Two worktree agents that each start a dev server want the same port, and the second fails or moves. Each worktree New Agent in Worktree makes gets a slot, the lowest free one among the repository's Marley-made worktrees, kept with the worktree (for example `git config branch.<b>.marleySlot`), and its terminals and tasks get `MARLEY_PORT_OFFSET` (the slot times ten, say) and `PORT` (3000 plus the offset). The seam is both of Zed's terminal builders in `crates/project/src/terminals.rs`, `create_terminal_shell_internal` beside `MARLEY_PROJECT` (#520) and `create_terminal_task`, each reading a registry keyed by the project's first folder that the workbench fills, as `BROWSER_OPENER` (#561) is filled: a small additive Zed touch with its ledger row. That reaches restored terminals, Zed's own panel, the Zed Agent's terminal tool and the debugger. Orca allocates no ports and discovers them instead (report 02 §2.4), which #521 already does; many dev servers ignore `PORT` or move on their own, so the offset helps those that read it. The slot is dropped with the worktree (#589's Remove).

## Acceptance
A worktree agent's worktree's terminals and tasks carry `MARLEY_PORT_OFFSET` and `PORT` of their own slot, the main checkout's carry none, two worktrees never share a slot, and a slot freed by a removed worktree is reused.
