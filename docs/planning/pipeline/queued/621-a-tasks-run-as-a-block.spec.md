---
pipeline_id: 3ba6657c-e031-4f8c-9606-0de86e34fe73
ticket: docs/planning/tickets/open/TICKET-621-a-tasks-run-as-a-block.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A task's run as a block"
type: feature
slice: prong 1 T4
references: [docs/marley/three-prong-plan.md, docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md]
---

## Title
A task or runnable's run is a block in its terminal: opened when the task's terminal spawns,
finished with the task's exit code (plan T4, D6).

## Scope
### In
- `crates/terminal/src/terminal.rs` (Zed crate, additive `// Marley:` hunks): a task terminal
  (`TerminalModeKind::Task`) opens a block in its `AnchoredBlocks` at spawn, the task's
  `command_label` as its command and its `cwd` as its folder, at the grid's first line; and
  `register_task_finished` finishes it with the exit code. Shell hooks stay off for tasks, as
  upstream keeps them.
- `crates/marley_terminal`: `AnchoredBlocks` takes a block opened and finished from outside the
  hook stream (the same states as a hooked block).
- The block draws, navigates and has its menu as any block does.

### Out (explicitly deferred)
- Rerun from the block, and the summary line (T4b, #622).
- Collapse on success (`HideStrategy::OnSuccess`), and the diagnostics (T4c, #623).
- Tasks in remote terminals.

## Reference (§20)
- **Upstream Zed:** `TerminalPanel::spawn_task` and `TerminalModeKind::Task` (`TaskState`,
  `register_task_finished`); kept as they are, with the block added beside them.
- **Warp (behavior):** a command's run is a block with its status in its header
  (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6); Zed's runnable-as-block
  plan (`docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` §8).

### Prior art
- **Behavior maps:** the two notes above; plan D6; AD-claude-441 (the Marley layout's tasks still
  run through `TerminalPanel::spawn_task`, reveal target the center).
- **Published material:** none needed.
- **Code we already ship:** `terminal.rs` loads the shell hooks only when `task.is_none()`;
  `TaskState` and `register_task_finished` append the summary line and apply `HideStrategy`;
  `AnchoredBlocks::apply` builds blocks from hook events; the gutter, pill and block menu
  (#470, #474, #554) draw any `AnchoredBlock`. No crate owns a task's block.

## UI proof
`script/e2e/621-a-tasks-run-as-a-block.sh` (sway): `repo/.zed/tasks.json` with `fails`
(`echo building; exit 3`) and `passes` (`echo ok`); each run from `task: spawn`.
- `failed.png`: the `fails` terminal with one block, its label `fails`, its output `building`, the
  failed pill with 3;
- `passed.png`: `passes` with its block and the finished pill;
- `menu.png`: the task block's menu (#554).

## Locked-In Decisions
- D1 — The block is synthesized from Zed's task state, not from shell hooks: the task's shell
  may not be one Marley integrates, and upstream keeps hooks out of tasks.
- D2 — The block's command is the task's `command_label`, its folder the task's `cwd`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a task finishes in its terminal, Marley shall show its run as one block with the task's label and output. | Shots `failed.png`, `passed.png` |
| REQ-002 | WHEN the task exits non-zero, its block shall carry the failed pill with the exit code. | Shot `failed.png` |
| REQ-003 | WHILE a task block is selected, its block menu shall open as a shell block's does. | Shot `menu.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the open and the finish, the hunks and their rows; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
