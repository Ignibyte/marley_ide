# A task block's Rerun and its pill — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-622-a-task-blocks-rerun-and-pill.md
- **Pipeline spec:** 622-a-task-blocks-rerun-and-pill.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - #474's Rerun types the block's command at the prompt, which a task terminal has none of.
  - AD-claude-441: tasks run through `TerminalPanel::spawn_task` so Zed's rerun and reuse rules hold.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted;** seams re-verified: Zed's terminal view reruns its task with `RerunTask`, which
  dispatches `zed_actions::Rerun { task_id, allow_concurrent_runs: Some(true), use_new_terminal:
  Some(false), reevaluate_context: false }` (`terminal_rerun_override`, `terminal_view.rs`); the
  tab's own Rerun button does the same; Zed's rerun replaces the tab's terminal. The element's
  block Rerun (#474) shows only for `command_verified` (`terminal_element.rs:2517`), which a task
  block is not (#621). `register_task_finished` appends `task_line` when `show_summary` and
  `command_line` when `show_command`. The terminal crate already reads a Marley global
  (`MarleyAgentHistory`) that the workbench sets.
- **Recall:** AD-claude-441 (tasks keep Zed's spawn and rerun path). The brain (`rusty-cli brain
  ask`, consultation abedbfd9640a46a8ac2b07dbf0039521): nothing on this seam.

### Design
- **`crates/terminal_view/src/terminal_element.rs`** (Zed crate, `// Marley:` hunk): beside the
  typed Rerun, a task terminal's newest block, once the task is not running, gets a Rerun Task
  button that dispatches `terminal_rerun_override(&task_id)`.
- **`crates/marley_workbench/src/blocks.rs`:** the block menu's Rerun Task for the same block,
  dispatching the same action (`zed_actions::Rerun`).
- **`crates/terminal/src/terminal.rs`** (Zed crate): `MarleyTaskSummaryHidden(bool)`, a global;
  `register_task_finished` leaves out `task_line` while it holds true.
- **`crates/marley_workbench/src/marley_workbench.rs`:** `apply_defaults` sets it to whether the
  Marley layout is on.
- **Ledger:** the `terminal.rs` and `terminal_element.rs` rows' clauses.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-002 | `fails` from `task: spawn`, the Marley layout | `finished.png`: the block, no `Task \`fails\` finished` line, the command line kept |
| REQ-001 | the pointer on the block, its Rerun Task button clicked | `hover.png`, `rerun.png` |

### Risks
- A user's own `show_summary` is honored in the Zed layout and overridden in the Marley layout;
  the block's pill says the same.

## Phase 2 — Code (2026-09-30)
- **Built:** the element's `task_rerun` button in `marley_block`; the block menu's Rerun Task in
  `blocks.rs`; `MarleyTaskSummaryHidden` in `terminal.rs` and its read in
  `register_task_finished`; `apply_defaults` setting it; the two ledger clauses.
- **Deviations:** none.
- **Review:** both paths dispatch `zed_actions::Rerun` with the override Zed's tab uses, so Zed's
  reuse and reveal rules hold (AD-claude-441); the dispatch goes through the window, outside any
  entity update. Only the newest block of a task terminal offers it, and only once the task is not
  running.
- **Clippy:** clean the first time.
- **Gate:** `just gate-diff` GREEN, 17 PASS, after the scenario's measuring passes.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/622-a-task-blocks-rerun-and-pill.sh` (sway); `run.sh` counts its runs
  in `.runs`. A first pass pointed at y=878, where a one-line block has no row; the block's row is
  897.
- **Run 2, the shots read** (code and scenario as gated):
  - `finished.png` (REQ-002): the `fails` tab: one block `building 1` with the `exit 3` pill, then
    only `⏵ Command: /usr/bin/bash -i -c 'sh run.sh'`; no `Task \`fails\` finished` line.
  - `hover.png`: the pointer on the block: Bookmark, Find, Filter, Copy and the Rerun Task button
    by the pill.
  - `rerun.png` (REQ-001): after its click, the tab's block reads `building 2`, the task run again
    in its tab; the run log counts 2 runs.
- **Not driven:** the menu's Rerun Task, which dispatches the same action.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide; `terminal_blocks.md`; the plan's T4 row; the ledger.
- **Knowledge:** none new (the row position is L-claude-621's).
- **Brain:** consultation abedbfd9640a46a8ac2b07dbf0039521 closed with `brain decide`.

