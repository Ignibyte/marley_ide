# A teardown hook before a worktree is removed — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-591-a-teardown-hook-before-a-worktree-is-removed.md
- **Pipeline spec:** 591-a-teardown-hook-before-a-worktree-is-removed.spec.md

## Phase 1 — Plan
- **Checklist** (no task tool): pre-flight ✓ (no active pipeline; #589's release install
  compiling, so no crate edits until it ends); recall ✓; the pair minted ✓; the prior-art sweep ✓;
  spec and design ✓; the ledger row ✓.
- **Request:** TICKET-591, split from #589 at its planning; Chad's standing word and 2026-09-29's
  (no tests).
- **Classification / tier:** feature, prong 2 (S). One Zed touch: a `TaskHook` variant.
- **Recall (§18.3):** #589's notes (Remove's order: the plan, the question, the workspace,
  `remove_root`); #585's L-block (a new worktree's tasks are its base's committed ones; for removal
  the worktree's own `.zed/tasks.json` is what it has); the brain (consultation
  07570809eb2f45fe8e93603e86f859f0): nothing on this seam.
- **Discovery:** `TaskHook` (`crates/task/src/task_template.rs:95-98`) has one variant and no
  exhaustive match anywhere; `run_create_worktree_tasks` (`crates/workspace/src/tasks.rs:235-338`)
  builds a `TaskContext` per folder (cwd, `WorktreeRoot`, `MainGitWorktree` from
  `git_store.original_repo_path_for_worktree`), resolves each template with an id base and awaits
  `spawn_in_terminal`'s `Option<Result<ExitStatus>>`; in the Marley layout the provider is
  `RoutedTerminals`, which adds Marley's paths.

### Design
- **`crates/task/src/task_template.rs`** (Zed crate, ledger row written): `RemoveWorktree` after
  `CreateWorktree`, a `// Marley:` comment.
- **`worktree_agents::tear_down(workspace, name, cx) -> anyhow::Result<bool>`** (Marley): the
  resolved `remove_worktree` tasks of the workspace's folders (id base `worktree_teardown_<id>`);
  each spawned and raced against `TEARDOWN_FOR` (`Duration::from_mins(2)`) with `select_biased!`;
  the first failure or timeout asks "The teardown of <name> did not finish" with the task's label
  and what happened, Remove Anyway or Cancel; returns whether Remove goes on.
- **`rail.rs`** (Marley): in `remove_worktree`, after the answer and when the worktree's workspace
  is open, `worktree_agents::tear_down(&member, &name, cx)`; `false` ends Remove.

### For the quality pass
- No tests (§7). A later scenario: a `remove_worktree` task that writes a file (it runs, Remove
  goes on), one that exits 1 (the question; Cancel keeps the worktree), one that sleeps past the
  deadline with the deadline lowered.

### Risks
- A teardown that hangs holds Remove for two minutes per task before the question.
- Global `remove_worktree` tasks run for every worktree Remove takes, as global `create_worktree`
  ones run for every worktree Zed makes.

## Phase 2 — Code
- **Checklist** (no task tool): the ledger row (at Plan) ✓; the variant ✓; `tear_down` and
  `teardown_tasks` ✓; the call in the rail ✓; the review ✓; the gate ✓.
- **Built as designed.** Clippy's four findings (a redundant closure, a `()` pattern, a
  `map_or_else`, `HashMap::default()`) were fixed before the gate.
- **The review**, against each criterion:
  - REQ-001: `TaskHook` is `rename_all = "snake_case"`, so the variant reads `remove_worktree`;
    no exhaustive match on `TaskHook` exists to update.
  - REQ-002: `tear_down` runs after the question's answer and before the workspace goes, one task
    at a time, each raced with the executor's two-minute timer.
  - REQ-003: a non-zero exit, a signal, an error or the deadline asks with the label and what
    happened; only answer 0, Remove Anyway, goes on.
  - REQ-004: no task, no terminal and no question; `spawn_in_terminal` answering `None` (no
    terminal provider) counts as done.
  - The tasks' terminals are in the worktree's workspace, which Remove then removes; on Cancel
    they stay. The Zed hunk is one variant with its `// Marley:` comment; its row is written.
- **The gate:** `just gate-diff` green: 16 passed, 0 failed, `GATE GREEN [diff]`, the receipt
  written.

---
## Phase 3 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md` (Added: a worktree's teardown task before Remove; #589's entry
  loses its "next"); `docs/marley_architecture/marley_workbench.md` (the teardown under Remove);
  `docs/marley/workbench-shell.md` (W4's record); `docs/marley/tutorial-outline.md` (591 shipped);
  `docs/marley/guide.md` (A teardown task). The row for `crates/task/src/task_template.rs`,
  written at Plan, describes the variant as shipped.
- **Knowledge:** AD-claude-591-a-remove-worktree-task-hook-runs-before-remove-with-a-two-minute-deadline-001.
- **Brain:** consultation 07570809eb2f45fe8e93603e86f859f0 closed with a decision (follow-up
  2026-10-29).
- **Closed:** TICKET-591 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **No tests** (§7): what a later scenario would show is in "For the quality pass".

---
## Phase 3 — Test (the visual check, after the fact)
- **Why after:** this ticket shipped on 2026-09-29 while the workflow had no visual check; Chad
  brought it back the same day (7e589cb0a1). #590, #591 and #589 were checked together.
- **The scenario:** `script/e2e/589-remove-a-worktree.sh`, `compositor sway`, on #510 and #585's
  fixture (a fake `claude` that logs its folder, arguments, `PORT` and `MARLEY_PORT_OFFSET`, the
  plugin's SessionStart; the `+` menu's New Agent in Worktree). A repository with a committed
  `.zed/tasks.json` holding a `remove_worktree` task that appends `torn down in
  $ZED_WORKTREE_ROOT` to a file. One run; every check passed.
- **What it showed:** #590: the first agent saw `PORT` 3010 and `MARLEY_PORT_OFFSET` 10, the second
  3020; the main checkout's terminal printed `main-port=[]` (`589-03-main-no-port`). #589: the
  first worktree's row menu reads Review, "Nothing to merge into main", a separator and Remove…
  (`589-04-menu`); Remove asks "Remove <name>?" naming the folder it deletes, with Remove and Cancel
  (`589-05-prompt`); after Remove the row is gone and the toast reads "Removed <name> and its branch
  agent/<name>" (`589-06-removed`), the folder is gone, the merged branch deleted, the other
  worktree kept. #591: the teardown task ran in the worktree before it went. #590 again: a third
  agent took the freed slot, `PORT` 3010.
- **Found:** the project row's `+0 −0` (#531) shows on a project with nothing changed; fixed in
  #531's own check.
