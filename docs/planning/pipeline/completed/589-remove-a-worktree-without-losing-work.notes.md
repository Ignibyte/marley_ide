# Remove a worktree without losing work — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-589-remove-a-worktree-without-losing-work.md
- **Pipeline spec:** 589-remove-a-worktree-without-losing-work.spec.md

## Phase 1 — Plan
- **Checklist** (no task tool): pre-flight ✓ (no active pipeline; the README marker present;
  #590's release install compiling, so no crate edits until it ends); recall ✓; the pair minted ✓;
  the prior-art sweep ✓; spec and design ✓; TICKET-591 filed ✓.
- **Request:** TICKET-589, #511's slice 2; Chad's standing word (build every remaining finding)
  and 2026-09-29's (no tests; the static gate only).
- **Classification / tier:** feature, prong 2 (M). No Zed touch. The teardown hook, a Zed touch
  of its own with an awaited task, split to TICKET-591 to keep this slice to the removal.
- **Recall (§18.3):**
  - #511's notes, "The split": Remove's outline (the checks, `build_root_plan` and `remove_root`,
    `branch -d` and Orca's proof, the teardown hook, the workflow rule).
  - Orca report 02 §2.6: its removal steps and the merged proof; "Marley today" notes Zed's
    `thread_worktree_archive`.
  - #590: the branch's `marleySlot` goes with its config section; a removed worktree's slot is
    free whether or not the key stays.
  - The brain (consultation c2fe71a1b1f647978dbbc0b7fa7e7480): nothing on this seam.
- **Discovery** (an Explore agent over bb9f0d060b, then read here):
  - The row menu (`rail.rs:2977-3008`): Review, then Merge or a label from `merge_line`
    (`:2027-2075`); `merge_worktree` (`:2203-2261`): `spawn_in`, checks in the background,
    `window.prompt`, checks again, act, toast, `detach_and_prompt_err`. `WorktreeEntry`
    (`:264-278`): `member` (the worktree's own workspace if open), `project`, `path`, `name`,
    `branch`, `main`, `repository`. `DriftState` in `self.drift` carries `owner` and the drift's
    `ahead`.
  - `thread_worktree_archive` (`crates/agent_ui`, `pub mod`): `build_root_plan(path, None,
    &workspaces, cx) -> Option<RootPlan>` is `None` when no open project has the path as a visible
    worktree, no linked repository matches, the path is outside `git.worktree_directory`, or Zed
    kept no creation record; `remove_root(plan, &mut AsyncApp)` verifies the record, releases the
    worktree from each affected project, runs `repo.remove_worktree(path, force: true)` (the folder
    deleted first), rolls back on failure. It closes no workspace and deletes no branch. The
    sidebar builds the plan, removes the workspaces, then calls `remove_root`.
  - `MultiWorkspace::remove(workspaces, RemovalIntent::KeepProject, window, cx) ->
    Task<Result<bool>>` asks to save first; `false` means nothing was removed. Dropping a
    workspace drops its terminals, and `Terminal`'s `Drop` kills the process.
  - `worktree_git::git` gives the raw exit status; `merge_checks`'s porcelain loop (untrimmed) is
    the model for the uncommitted count.

### Design
- **`worktree_git`** (Marley): `uncommitted(worktree) -> usize` (`status --porcelain
  --untracked-files=normal`, non-empty lines); `enum BranchEnd { Deleted, Kept(String) }` and
  `end_branch(main, branch, base: Option<&str>) -> BranchEnd`: `branch -d`; on a refusal, the oid
  (`rev-parse refs/heads/<b>`), the targets (the base, `origin/HEAD` when it resolves, `HEAD`),
  `merged_into(main, oid, target)` (`merge-base --is-ancestor`; `merge-tree --write-tree` equal to
  `rev-parse <target>^{tree}`; `cherry <target> <oid>` all `-`), then `update-ref -d refs/heads/<b>
  <oid>`; after either delete, `config --remove-section branch.<b>` (exit 128 when there is none is
  fine).
- **`rail.rs`** (Marley): `RemoveLine { Remove, Note(String) }` from `remove_line(path)` (a linked
  worktree with a branch; in a workflow repository only when `ahead` is 0); the menu entry;
  `remove_worktree(path, window, cx)`: on the UI thread, the entry's `member`, `main`, `branch`,
  `name` and the plan from `build_root_plan` over the window's workspaces (none: a prompt error
  saying why); in the background `uncommitted`; the prompt; `MultiWorkspace::remove([member],
  KeepProject)` (false: stop); `remove_root(plan)`; `recorded_base`; `end_branch`; the toast; the
  whole under `detach_and_prompt_err("Could not remove <name>")`.
- **Manifest:** `crates/marley_workbench/src/worktree_git.rs`, `crates/marley_workbench/src/rail.rs`.
  No Zed path.

### For the quality pass
- No tests (§7). A later scenario: Remove on a clean merged worktree (row gone, branch gone),
  on a dirty one (the count in the prompt, Cancel keeps it), on an unmerged branch (the branch
  kept, the toast), and in a workflow repository (the note).

### Risks
- `remove_root` deletes the folder even with untracked work; the prompt is the only guard, so it
  always shows.
- A worktree open only as a member of a project group with other folders: `KeepProject` removes
  the member workspace; the plan releases the folder from the others.

## Phase 2 — Code
- **Checklist** (no task tool): `worktree_git` (`uncommitted`, `BranchEnd`, `end_branch`,
  `merged_into`) ✓; the rail (`RemoveTarget`, `RemoveLine`, `remove_line`, the menu entry,
  `remove_worktree`, `remove_target`) ✓; the review ✓; the gate ✓.
- **Built as designed.** Remove sits after a separator below Review and Merge, as "Remove…"; the
  prompt is a warning with Remove, or Remove Anyway when git counts changes not committed.
- **The review**, against each criterion:
  - REQ-001: the main checkout gets no worktree row (`worktree_rows` skips it), so Remove shows
    only on linked worktrees; in a workflow repository it shows once the drift reads 0 ahead, else
    the note; before the drift is read it shows, as Merge's checks would.
  - REQ-002: the count is `git status --porcelain --untracked-files=normal`, read before the
    prompt; Cancel returns before anything changes.
  - REQ-003: the plan is built before the worktree's workspace goes (as Zed's sidebar does, since
    the plan needs a project holding it); `MultiWorkspace::remove(.., KeepProject)` asks about
    unsaved files, `false` stops Remove; `remove_root` then verifies, releases and removes.
  - REQ-004: `build_root_plan` answers `None` for a worktree no open project holds or Zed has no
    record of; the prompt error says so. `remove_root`'s own check stops a mismatch.
  - REQ-005 and REQ-006: `end_branch` as designed; the oid is read before the proof and
    `update-ref -d` takes it as the old value; the config section goes after either delete.
  - **Changed in review, as a precaution:** the workspace removal first ran inside the rail's own
    update; since the window's sidebar is the rail, it now runs through the `MultiWorkspace`
    handle outside it. `MultiWorkspace::remove` only spawns before its first await, so the first
    form would not have panicked; no F-block.
  - A failure in `remove_root` rolls the worktree back into its projects; its own workspace,
    already removed, does not come back (the worktree's row reopens it).
  - Nothing from Warp or a GPL body: the branch proof is written from Orca's report's description;
    Zed's functions are called, not copied.
- **The gate:** `just gate-diff` green: 16 passed, 0 failed, `GATE GREEN [diff]`, the receipt
  written.

---
## Phase 3 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md` (Added: Remove on a worktree's row);
  `docs/marley_architecture/marley_workbench.md` (Remove in "Review and merge from a worktree's
  row"); `docs/marley/workbench-shell.md` (W4's record gains #589); `docs/marley/tutorial-outline.md`
  (589 shipped, 591 added); `docs/marley/guide.md` (Remove… under the worktree row's menu). No Zed
  path changed.
- **Knowledge:** AD-claude-589-remove-goes-through-zeds-archive-code-and-the-branch-goes-only-when-merged-001.
  No F-block: the review's one change was a precaution (Phase 2).
- **Brain:** consultation c2fe71a1b1f647978dbbc0b7fa7e7480 closed with a decision (follow-up
  2026-10-29).
- **Filed:** TICKET-591 (the teardown hook), at planning, its row after this one's.
- **Closed:** TICKET-589 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **No tests** (§7): what a later scenario would show is in "For the quality pass".
