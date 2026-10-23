# The worktree merge and remove prompts as the board words them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-926-worktree-prompts-as-the-board.md
- **Pipeline spec:** 926-worktree-prompts-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11 (Worktrees).
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):**
  - AD-claude-511-marley-merges-a-worktree-branch-locally-and-the-workflow-owns-it-where-it-runs-001:
    the merge is local, never pushed, and refused where the workflow owns merging; the prompt's
    words keep "Nothing is pushed".
  - AD-claude-589-remove-goes-through-zeds-archive-code-and-the-branch-goes-only-when-merged-001:
    the branch goes only when its commits are in a target; `branch_fate` reads the same proof.
  - F-claude-511-the-merge-checks-trimmed-porcelains-leading-space-001: porcelain lines are read
    untrimmed; the new check does not touch that parsing.
  - F-claude-880-a-scenario-opened-a-live-tmux-session-001: the scenario sets
    `marley.tmux_hosts` to `[]` (L-claude-831).
  - L-claude-832-a-right-click-menu-opens-at-the-pointer-so-click-its-entries-inside-001: the row
    menu's entries are clicked inside the opened menu.
- **Discovery:**
  - `crates/marley_workbench/src/worktree_git.rs`: `merge_checks` (245), `merged_into` (618),
    `end_branch` (569), `recorded_base` (68), `uncommitted` (535).
  - `crates/marley_workbench/src/rail.rs`: `merge_worktree` (4115), `remove_worktree` (3860),
    `RemoveTarget` (835).
  - `crates/marley_workbench/src/prompts.rs`: `card` (44).
- **Decisions:** D1–D4 in the spec.

### Design
- `merge_checks` returns `MergeChecks { ahead: u32, passed: Vec<String> }` (a struct, not a
  tuple); `ready_to_merge` passes it on in place of the bare count. The conflict check runs after
  the uncommitted checks and before the ahead count: `git merge-tree --write-tree --name-only
  refs/heads/<base> refs/heads/<branch>`; exit 1 means conflicts, and the refusal lists the
  conflicted paths from its output.
- `merge_worktree` builds the detail as the sentence, a blank line, then `✓ <check>` per line;
  the second `ready_to_merge` after the prompt still compares base and count.
- `prompts.rs`: `card` splits the detail on newlines; a line starting with "✓ " is an `h_flex`
  of a 16 px round tile (`status().success_background`, a `Check` icon in `Color::Success`) and a
  small label; blank lines become a small gap; other lines stay muted labels.
- `branch_fate(main, branch, base) -> anyhow::Result<BranchFate>` with
  `enum BranchFate { Goes { target: String }, Stays { base: String } }`: resolves the branch's
  commit, then tries `merged_into` against the base, `origin/HEAD` and `HEAD` as `end_branch`
  does. `remove_worktree` runs it beside `uncommitted` when the target has a branch and a main
  checkout. `end_branch` itself is unchanged.
- File manifest (all Marley crate): `worktree_git.rs`, `rail.rs`, `prompts.rs`.
- Ledger: none (no Zed path).

### Visual check plan
- REQ-001: the scenario makes worktree `agent/one` from `main` with two commits; its row menu's
  Merge 2 commits into main… opens the prompt; `926-01-merge` shows the sentence and four ticked
  lines. Marley Day: `926-04-day`.
- REQ-002: worktree `agent/two` has a commit and three edited tracked files; Remove… shows
  `926-02-remove-stays`; Cancel. After Merge on `agent/one`, its Remove… shows
  `926-03-remove-goes`; Cancel (nothing is removed in the run).

### Risks
- `git merge-tree --write-tree` needs git 2.38 or later; the box has it, and `merged_into`
  already depends on it. An older git makes the check fail with git's own message.
- The proof runs before the prompt and again inside `end_branch` after it; a branch that moves in
  between gets the after-prompt result, as today.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
