---
pipeline_id: 5753cf0b-1be2-47c3-82ec-4c69a36c0f1f
ticket: docs/planning/tickets/open/TICKET-622-a-task-blocks-rerun-and-pill.md
status: Phase 4 — Complete PASS
title: "A task block's Rerun and its pill"
type: feature
slice: prong 1 T4
references: [docs/marley/three-prong-plan.md, docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md]
---

## Title
A task block's Rerun runs the task again through Zed's task machinery, and the block's pill takes
the place of Zed's appended summary line (plan T4, D6). After #621.

## Scope
### In
- The task block keeps its spawn spec (the task's id and its `SpawnInTerminal`).
- Its Rerun, from the hover actions (#474) and the block menu (#554), runs the task again as
  Zed's `task::Rerun` does for that task, in the same terminal under Zed's reuse rules, instead of
  typing the command.
- In the Marley layout, a task terminal's finish appends no summary line (`⏵ Task `…` finished`);
  the Zed layout keeps it.

### Out (explicitly deferred)
- Collapse on success (`HideStrategy::OnSuccess`).
- The diagnostics (#623).

## Reference (§20)
- **Upstream Zed:** `task::Rerun` and `TerminalPanel::spawn_task`'s reuse of a task's terminal;
  `register_task_finished`'s summary line, kept in the Zed layout.
- **Warp (behavior):** a block's rerun runs its command again
  (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6);
  `docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` §8 (per-block rerun, the pill in
  place of the summary line).

### Prior art
- **Behavior maps:** the notes above; plan D6.
- **Published material:** none needed.
- **Code we already ship:** #474's Rerun types a block's command at the prompt; Zed's
  `Rerun { task_id, .. }` action and the task inventory's last-scheduled task; the summary line in
  `register_task_finished`.

## UI proof
`script/e2e/622-a-task-blocks-rerun-and-pill.sh` (sway): #621's `fails` task, run once.
- `finished.png`: its block with the pill and no `⏵ Task` line under it;
- `rerun.png`: after the block's Rerun, the `fails` tab running the task again: Zed's rerun
  replaces the tab's terminal with a new one (`replace_terminal`), so the tab shows the new run's
  block.

## Locked-In Decisions
- D1 — Rerun goes through Zed's task path, so reuse and reveal rules hold (AD-claude-441).
- D2 — The summary line goes only in the Marley layout.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user chooses Rerun on a task block, Marley shall run that task again through Zed's task machinery, in its tab. | Shot `rerun.png`; the run log's second `building` |
| REQ-002 | WHILE the Marley layout is on, a task terminal's finish shall append no summary line. | Shot `finished.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the spec on the block, the rerun, the summary line; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
