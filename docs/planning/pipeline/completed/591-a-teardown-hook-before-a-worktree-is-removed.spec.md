---
pipeline_id: b6653bd4-efa5-4adf-ba63-665a9f5c97eb
ticket: docs/planning/tickets/open/TICKET-591-a-teardown-hook-before-a-worktree-is-removed.md
status: Phase 3 — Complete PASS
title: A teardown hook before a worktree is removed
type: feature
slice: prong 2, worktree agents, review and merge (after #589)
references: [TICKET-589, TICKET-585]
---

## Title
A task whose `hooks` hold `remove_worktree` is a worktree's teardown: Remove (#589) runs each one
in the worktree's workspace after its question and before the worktree goes, and waits for it up
to two minutes, so a dev database, container or server the worktree started is stopped first. A
teardown that fails or runs past the deadline stops Remove unless the user confirms.

## Scope
### In
- `TaskHook::RemoveWorktree` in Zed's `task` crate, serialized `remove_worktree`, beside
  `CreateWorktree`: one variant (a Zed touch with its ledger row).
- Remove, after its question and before the worktree's workspace goes: the `remove_worktree`
  tasks of each folder of the worktree's own workspace (its `.zed/tasks.json` and the user's
  global tasks, as Zed finds `create_worktree` ones), with `ZED_WORKTREE_ROOT` and
  `ZED_MAIN_GIT_WORKTREE` (so Marley's `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH` too), run one
  after another in terminals, each awaited up to two minutes.
- A failure (a non-zero exit, an error) or the deadline: a prompt naming the task and what
  happened, with Remove Anyway and Cancel; Cancel leaves the worktree and its terminals open.

### Out (explicitly deferred)
- A teardown for a worktree whose own workspace is not open (Remove needs it open anyway, #589).
- Killing a teardown that ran past the deadline: its terminal closes with the workspace on Remove
  Anyway, and stays on Cancel.

## Reference (§20)
Upstream Zed: `Workspace::run_create_worktree_tasks` (`crates/workspace/src/tasks.rs`) finds the
tasks a hook names through `Inventory::templates_with_hooks`, gives them the worktree's variables
and awaits each one's exit status from `spawn_in_terminal`. Marley's teardown does the same for
the new hook, with a deadline and a question on failure. Orca's `scripts.archive` runs before its
removal and a failure stops it (`docs/orca_architecture/02-worktrees-and-review.md` §2.6 step 2,
two minutes in its hooks).

### Prior art
- The code we ship: `TaskHook`, `templates_with_hooks`, `resolve_task`, `spawn_in_terminal`
  returning `Task<Option<Result<ExitStatus>>>`, and `run_create_worktree_tasks` as the model; the
  executor's timer for the deadline.
- The behavior maps: Orca's archive script (§2.6).
- No crate owns a removal hook; Zed's thread archive runs none.

## Locked-In Decisions
- D1 — The hook is a `TaskHook` variant, so tasks declare it in `.zed/tasks.json` the way
  `create_worktree` ones do, and the task inventory finds both.
- D2 — Two minutes per task, Orca's deadline; a failure or the deadline asks rather than refuses,
  since the user may know the teardown does not matter.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE a task's `hooks` hold `remove_worktree`, the task inventory shall read it as `TaskHook::RemoveWorktree`. | Review; the gate (the schema derives) |
| REQ-002 | WHEN the user confirms Remove, Marley shall run the worktree's `remove_worktree` tasks one after another, each awaited up to two minutes, before its workspace is removed. | Review |
| REQ-003 | IF a teardown task fails or runs past two minutes, THEN Marley shall ask, naming the task, and shall remove the worktree only on Remove Anyway. | Review |
| REQ-004 | WHEN the worktree has no `remove_worktree` task, Remove shall go on as before. | Review |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the ledger row first; the variant; `worktree_agents::tear_down`; its call in the
  rail's `remove_worktree`; a review; `script/gates.sh --diff` green (no tests, §7).
- **P3 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit, push, install.
