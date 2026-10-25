---
pipeline_id: 493ee9c6-3f92-4b49-805d-b40414a7a426
ticket: docs/planning/tickets/open/TICKET-926-worktree-prompts-as-the-board.md
status: Phase 4 — Complete PASS
title: The worktree merge and remove prompts as the board words them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/C-threads.md, docs/planning/pipeline/completed/870-prompts.spec.md]
---

## Title
The Merge prompt lists the checks it ran, each ticked; the Remove prompt says whether this
worktree's branch stays, as the Worktrees board words them.

## Scope
### In
- **The merge checks as a list** (`worktree_git.rs`, `merge_checks`; Marley crate): the checks
  keep their order and their refusals, and gain a fourth, read-only one: `git merge-tree
  --write-tree <base> <branch>` must succeed, so no file would stop the merge; a conflict refuses
  with the files git names. On success the function gives the checks it passed, as their board
  words: "The main checkout is on <base>", "Nothing uncommitted in the main checkout", "Nothing
  uncommitted in the worktree", "No file would stop the merge".
- **The Merge prompt** (`rail.rs`, `merge_worktree`; Marley crate): the question stays "Merge
  <N commits> into <base>?"; the detail is "<branch> goes into <base> in <main checkout> with a
  merge commit. Nothing is pushed." followed by the checks, one per line, each led by "✓ ".
- **Ticked lines in a prompt** (`prompts.rs`, `card`; Marley crate): the Marley prompt card draws
  a detail line that starts with "✓ " as a check row: a small round tile in the success colours
  with a check icon, then the line's words. Other lines stay as today.
- **The Remove prompt** (`rail.rs`, `remove_worktree`; `worktree_git.rs`; Marley crate): a new
  read-only `worktree_git::branch_fate(main, branch, base)` runs `end_branch`'s proof (`merged_into`
  against the base, `origin/HEAD` and `HEAD`) without deleting. The question is "Remove
  <name>?"; the detail says, in order: "It has N changes not committed. They go with the folder."
  when it has any; then "The branch goes: <target> has its commits." or "The branch stays: <base>
  does not have its commits."; a worktree on no branch says nothing of a branch. The confirm
  stays Remove, or Remove Anyway with changes.

### Out (explicitly deferred)
- The port slot among the New Agent in Worktree prompt's facts: #851 Out (the slot is assigned
  when the worktree is made).
- A merge prompt drawn as a view of its own (a custom modal): the prompt card of #870 carries the
  list.

## Reference (§20)
Upstream Zed (`ui_prompt`, `workspace`): the prompt is Zed's `window.prompt` with its scrim, keys
and answer, kept; Marley draws the card (#870). The merge and the branch's end are
Marley-specific (#511, #589), over git's own `merge-tree`, `merge-base` and `cherry`.

### Prior art
- `rail.rs:4115` `merge_worktree`: `ready_to_merge` in the background, then
  `window.prompt(Info, "Merge N commits into base?", detail, ["Merge", "Cancel"])`, the checks
  again, and the merge.
- `worktree_git.rs:245` `merge_checks`: on the base, nothing uncommitted in either checkout,
  commits ahead; each failure refuses with a sentence. No conflict check: `merge` (302) aborts a
  merge that stops.
- `worktree_git.rs:618` `merged_into` already runs `git merge-tree --write-tree` read-only;
  `:569` `end_branch` holds the branch proof Remove runs after the prompt.
- `rail.rs:3860` `remove_worktree`: `worktree_git::uncommitted` and the generic sentence;
  `RemoveTarget` (835) has `main` and `branch`; `recorded_base` (68) gives the base.
- `prompts.rs:44` `card`: the detail is one muted `Label`.
- AD-claude-511 and AD-claude-589 (merge locally; the branch goes only when merged).
- The brain: not consulted at queueing; the promoting session asks.

## UI proof
Scenario `script/e2e/926-worktree-prompts-as-the-board.sh` (sway): a scratch repository on
`main` with two worktrees made as #510 makes them (each branch with its base recorded): one with
two commits ahead and clean, one with a commit not in `main` and three uncommitted changes;
`marley.tmux_hosts` empty.
- `926-01-merge`: the first worktree's row menu, Merge 2 commits into main…: the prompt with its
  four ticked checks (REQ-001).
- `926-02-remove-stays`: the second worktree's Remove…: "It has 3 changes not committed. They go
  with the folder. The branch stays: main does not have its commits." and Remove Anyway (REQ-002).
- `926-03-remove-goes`: after the merge, the first worktree's Remove…: "The branch goes: main has
  its commits." (REQ-002).
- `926-04-day`: the Merge prompt in Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The prompt lists only checks that ran and passed; a failed check still refuses with its
  sentence, as now.
- D2 — The conflict check is read-only (`merge-tree --write-tree`), so asking changes nothing in
  either checkout.
- D3 — Remove's words come from the same proof that ends the branch after the prompt, so the
  prompt never says "stays" for a branch that then goes.
- D4 — A "✓ " line in any prompt's detail draws as a check row; only the Merge prompt writes one.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user chooses Merge on a worktree whose checks pass, the prompt shall say where the branch goes and list the four checks it ran, each with a tick. | `926-01-merge`, `926-04-day` |
| REQ-002 | WHEN the user chooses Remove on a worktree, the prompt shall say how many changes go with the folder, if any, and whether the branch stays or goes and why. | `926-02-remove-stays`, `926-03-remove-goes` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `worktree_git.rs`, `rail.rs`, `prompts.rs`; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
