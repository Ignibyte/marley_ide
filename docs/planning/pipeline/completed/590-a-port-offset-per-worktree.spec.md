---
pipeline_id: d17c541e-cd16-4f6d-a88b-37a97c19ff2d
ticket: docs/planning/tickets/open/TICKET-590-a-port-offset-per-worktree.md
status: Phase 3 — Complete PASS
title: A port offset for each worktree agent's worktree
type: feature
slice: prong 2, worktree agents, the worktree's environment, part 2 (after #585)
references: [TICKET-510, TICKET-585, TICKET-520, TICKET-561]
---

## Title
Each worktree New Agent in Worktree makes gets a slot, and the terminals and tasks of its project
get `MARLEY_PORT_OFFSET` (the slot times ten) and `PORT` (3000 plus the offset), so two worktree
agents that each start a dev server that reads `PORT` do not both ask for 3000.

## Scope
### In
- A slot per Marley-made worktree: the lowest, from 1, that no other live worktree of the
  repository holds, kept as `git config branch.<branch>.marleySlot` beside #510's
  `branch.<branch>.base`, written when the worktree is made and before its agent starts.
- Both of Zed's terminal builders in `crates/project/src/terminals.rs` add the two variables for a
  local project whose first folder is a linked worktree with a slot, read when the terminal starts
  through a reader the workbench registers in `marley_terminal`, so restored terminals, Zed's panel,
  tasks, the debugger's and the agents' terminals all get them.

### Out (explicitly deferred)
- Port discovery (Orca's `/proc` scan): #521 already finds the ports a project listens on.
- Showing the port on the worktree's row; a slot for a worktree Marley did not make.
- Unsetting a `PORT` Marley itself inherited (a Marley started from a worktree's terminal).
- Zed's own `create_worktree` hook tasks: they start inside Zed's create, before the slot is
  written, so they carry no port.
- Freeing the slot's config key when a worktree is removed (#589 may unset it; a gone worktree's
  slot is free anyway, since only live worktrees count).

## Reference (§20)
N/A — Marley-specific: Zed has no worktree agents and no per-worktree environment, and Warp none.
The behavior follows Conductor's published `CONDUCTOR_PORT` ("the first port in a range of 10
ports assigned to the workspace", conductor.build/docs/reference/environment-variables): a block of
ten per worktree, handed to programs in the environment. Marley names it `MARLEY_PORT_OFFSET`
and also sets the conventional `PORT`.

### Prior art
- The behavior maps: Orca allocates no ports and discovers them instead
  (`docs/orca_architecture/02-worktrees-and-review.md` §2.4 and §3 item 7, which names this offset
  and warns that many servers ignore `PORT`); discovery is already Marley's (#521).
- Published: Conductor's `CONDUCTOR_PORT`, ten ports per workspace (above).
- The code we ship: Zed's terminal builders take a directory environment, then `terminal.env`, then
  a task's own `env`; #520's `MARLEY_PROJECT` and #575's restored id are hunks in the shell builder,
  #561's `BROWSER` a process-wide opener in `marley_terminal::shell_integration` that the workbench
  sets. Zed's `git::repository::parse_worktrees_from_str` parses `git worktree list --porcelain`,
  taken as is. No crate owns a port slot.

## Locked-In Decisions
- D1 — The offset is the slot times ten, the port 3000 plus the offset: Conductor's block of ten,
  and 3000 is the default most JavaScript servers read `PORT` against.
- D2 — The slot is kept in the repository's git config per branch (`branch.<b>.marleySlot`), as the
  base is; the lowest slot no live linked worktree's branch holds is taken, so a removed
  worktree's slot is reused without a cleanup step.
- D3 — The terminal reads the slot when it starts, through an async reader in `marley_terminal`
  the workbench registers (a `.git` file, then `git symbolic-ref` and `git config`), not a cache
  filled later, so a terminal restored at launch gets it too. A folder whose `.git` is not a file
  (the main checkout, a plain folder) runs no git.
- D4 — The variables go in after the directory's environment and before `terminal.env` and a
  task's own `env`, so the user's settings and a task's values win over Marley's.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN New Agent in Worktree makes a worktree, Marley shall write, before the agent starts, `branch.<branch>.marleySlot` as the lowest number from 1 that no other live linked worktree's branch holds. | Review of `worktree_git::assign_slot` and its call in `create` |
| REQ-002 | WHEN a local terminal or task starts in a local project whose first folder is a linked worktree with a slot, its environment shall hold `MARLEY_PORT_OFFSET` as the slot times ten and `PORT` as 3000 plus that. | Review of both builder hunks and `marley_terminal::ports` |
| REQ-003 | WHEN a terminal or task starts in the main checkout, a folder with no slot, or a remote project, Marley shall add neither variable. | Review |
| REQ-004 | WHERE `terminal.env` or a task's own `env` names `PORT` or `MARLEY_PORT_OFFSET`, that value shall be kept. | Review of the hunks' order |
| REQ-005 | WHEN a worktree that held a slot is gone, the next worktree made shall be able to take that slot. | Review: only live worktrees' slots count |
| REQ-006 | WHEN Marley restores a terminal of a slotted worktree at launch, the terminal shall get the same two variables. | Review: the slot is read as the terminal starts |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the ledger row first; `marley_terminal::ports`; the two hunks in `terminals.rs`;
  `worktree_git::slot_of` and `assign_slot`; the reader registered in `init`; the call in `create`;
  a review of the diff; `just gate-diff` green (no tests, §7).
- **P3 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit, push, install.
