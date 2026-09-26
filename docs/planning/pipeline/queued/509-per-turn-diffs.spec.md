---
pipeline_id: 4e39dc0b-b7a9-4912-b583-542ecc9461cc
ticket: docs/planning/tickets/open/TICKET-509-per-turn-diffs.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Per-turn diffs for Claude Code in a terminal"
type: feature
slice: prong 2 (review), on #519's events
references: [docs/planning/pipeline/queued/519-claude-code-events-in-the-rail.spec.md, docs/orca_architecture/README.md, docs/orca_architecture/02-worktrees-and-review.md, docs/orca_architecture/01-agents-and-sessions.md, docs/planning/design-notes/warp-once-over-2026-09-25.md]
---

## Title
Marley snapshots a terminal's repository when each Claude Code turn opens and closes, records a
turn that changed files as a commit whose parent is the turn's start, pins it under
`refs/marley/turns/`, and lists it under the terminal's row in the rail. A click opens the turn in
Zed's commit view: a diff multibuffer of that turn's changes and nothing else, the edits made
through the shell included. Chad reviews an agent's work turn by turn.

## Scope
### In
- **Turns**, per terminal seat, from #519's events. A turn opens at a UserPromptSubmit, the user's
  or a harness's, and not at the post-compaction continuation. It closes at Stop, StopFailure, a
  manual PostCompact, the next UserPromptSubmit or SessionEnd, or when Claude Code leaves the
  terminal.
- **Snapshots.** At the open and at the close, `Repository::checkpoint` of the innermost repository
  whose work directory holds the event's `cwd` (the agent bar's rule, `branch_for`). A close that
  a new prompt causes serves as the next turn's open, so one checkpoint does both.
- **The turn's commit.** When the close's tree differs from the open's, Marley writes a commit with
  the close's tree and the open's checkpoint as its one parent: `git commit-tree <tree> -p <open>
  --no-gpg-sign -F -`, the message the turn's prompt with the trailers `Marley-Session` and
  `Marley-Turn`, authored and committed as Marley. It pins the commit at
  `refs/marley/turns/<session_id>/<n>` with `Repository::update_ref`. A turn that changed nothing
  writes and pins nothing.
- **The rail.** Under an agent row with turns, a "Turns (N)" line with a disclosure; open, the
  session's turns newest first, each with its prompt on one line, the number of files it changed,
  a failed mark after StopFailure, and "injected" for a turn a harness's prompt started, titled by
  its tag (`task notification`) instead of its text.
- **The view.** A click on a turn opens Zed's `CommitView` on the turn's commit. Its diff is
  against the first parent, the turn's start, so it shows the turn alone.
- **Pruning.** When a session pins its first turn, Marley deletes the turn refs of this repository
  whose commits are older than 30 days.

### Out (explicitly deferred)
- Review notes back to the agent: Zed draws "Send Review to Agent" on its diffs and nothing in the
  tree handles the action (report 02, item 5; #522).
- Keep or reject per hunk, which Zed's `action_log` gives Agent Panel threads only.
- Attributing hunks to tool calls, and a warning when the turn ran in a checkout shared with the
  user or another agent: every change made in the tree during the turn lands in it, and #510's
  worktrees remove the overlap.
- Edits a subagent or a background task makes after the lead's Stop: they land in the next turn.
- Remote repositories: Zed's checkpoint works there, `git commit-tree` here does not.
- A past session's turns after a restart: the refs keep them for session resume (report 01,
  item 7; #540) to list.
- Restoring a turn, and #511 deleting a worktree's turns with the worktree.

## Reference (§20)
- **Warp:** Interactive Code Review gathers an agent's edits into a diff, browsed file by file,
  with line comments sent back to the agent in one batch; it works with Claude Code and Codex
  (docs.warp.dev/agents/local-agents/interactive-code-review/). Marley groups the diff by turn and
  opens it in Zed's commit view; batched comments come with review notes (report 02, item 5).
  No Warp code was read.
- **Upstream Zed:** `project`'s git checkpoints (`Repository::checkpoint`, a temporary index that
  touches neither the user's index nor a ref, made for the agent's own threads) and `git_ui`'s
  `CommitView` (a commit's diff multibuffer). Both kept as they are.
- **Orca:** turn boundaries and snapshots (report 02 item 4 and §2.13; report 01 item 3). Orca
  itself takes no git snapshots (report 02 §2.7).

### Prior art
- **Reports.** Report 02 item 4: checkpoints as the snapshots, a turn also closed by the next
  UserPromptSubmit, refs to pin them, the prompt as the title, a multibuffer to show them, and the
  warning that a checkpoint holds every change in the tree. §2.13 records the hook order in Orca's
  Claude Code 2.1.280 sessions (`src/shared/__fixtures__/claude-cancel-shell-hooks.jsonl`): an API
  error sends StopFailure and no Stop, a manual `/compact` ends without Stop, and an Escape sends
  nothing, the next event being the next prompt. Report 01 item 3: a harness-injected prompt gets
  a turn of its own, marked; Orca's rebuilt turn diffs
  (`src/renderer/src/components/native-chat/native-chat-turn-diffs.ts`) miss what an agent changes
  through shell commands. The README's open question 6 sets the default this ticket takes.
- **Published material.** git's `commit-tree` (`-p` names the parent; `--no-gpg-sign`
  countermands `commit.gpgSign`, which applies to `commit-tree` too), `update-ref` and
  `for-each-ref`. Warp's Interactive Code Review, above.
- **Code we already ship.** Zed owns the snapshot and the view; the one piece missing is a commit
  whose parent is the turn's start, which one `git commit-tree` makes. Zed's own agent threads
  take the same checkpoints and restore them (`crates/acp_thread/src/acp_thread.rs:4003`,
  `:4345`). `GitStore::checkpoint` (`crates/project/src/git_store.rs:2133`) spans every
  repository and keeps its map private (`:488`), so the ticket takes `Repository::checkpoint`
  (`:9853`) per repository, with `Repository::update_ref` (`:9365`) and `delete_ref` (`:9373`).
  The checkpoint adds tracked changes and untracked files under 2 MB to a temporary index, and
  writes the tree and a commit on `HEAD` (`crates/git/src/repository.rs:3063`, `:3774`).
  `CommitView::open` (`crates/git_ui/src/commit_view.rs:183`) loads its diff through
  `load_commit_diff` (`git_store.rs:7180`), which runs `git show --first-parent`
  (`repository.rs:1448`). Rejected:
  `MultiDiffView` compares files on disk and `BranchDiff` a branch against the tree, neither two
  commits; `diff_checkpoints` (`git_store.rs:10027`) returns a patch as text, not a multibuffer.

## UI proof
UI-AFFECTING: turn rows in the rail and the diff they open.
`script/e2e/509-per-turn-diffs.sh` (`compositor sway`: the disclosure and the turns are clicked).
A scratch repository with two committed files, `src/lib.txt` and `src/util.txt`, and one staged
change of the user's; #519's fake `claude`, whose recorded session makes real edits between its
events, in five turns: "Add a README" writes `README.md` and runs `sed -i` on `src/lib.txt`, then
Stop; "Fix the typo in util" edits `src/util.txt` and sends no Stop; "Explain the build" (which
closes the turn before it) changes nothing, then Stop; a `<task-notification>` prompt edits
`src/lib.txt`, then Stop; "Refactor the parser" edits `src/util.txt`, then StopFailure. Shots:
- `509-01-turns`: the row reads "Turns (4)" and, opened, lists newest first "Refactor the parser
  · 1 file · failed", "task notification · injected · 1 file", "Fix the typo in util · 1 file"
  and "Add a README · 2 files"; "Explain the build" is absent.
- `509-02-first-turn`: "Add a README" opened: `README.md` added and the `sed` change to
  `src/lib.txt`, nothing of the later turns, nothing of the user's staged change.
- `509-03-closed-by-a-prompt`: "Fix the typo in util" opened: its one change, which the next
  prompt closed with no Stop between.
The run log records `git for-each-ref refs/marley/turns/`, each turn commit's parent, and
`git status --porcelain` and `git diff --cached --stat` before and after the session, which match.

## Locked-In Decisions
- D1: Snapshots are Zed's git checkpoints, not edits rebuilt from tool calls. They catch every
  change, shell commands included, and touch neither the user's index nor a ref.
- D2: A turn opens at a UserPromptSubmit and closes at Stop, StopFailure, a manual PostCompact, the
  next UserPromptSubmit or SessionEnd. An API error sends StopFailure without Stop, a manual
  `/compact` ends without Stop, and a cancelled turn sends nothing until the next prompt (report
  02 §2.13).
- D3: The view is Zed's commit view on a turn commit whose parent is the turn's opening
  checkpoint. `load_commit` diffs against the first parent, so the view shows the turn alone and
  Marley adds no diff view.
- D4: Turn commits are pinned under `refs/marley/turns/<session_id>/<n>` (the survey's default),
  so neither `git gc` nor a restart loses them. Refs older than 30 days go when a new session pins
  its first turn; #511 removes a worktree's with it.
- D5: A turn that changed nothing is not listed and pins nothing.
- D6: A harness-injected prompt starts a turn of its own, marked and titled by what injected it;
  the post-compaction continuation starts none (report 01, item 3).
- D7: The turn commit is Marley's: author and committer "Marley", `--no-gpg-sign` so a signing
  config never prompts, the prompt passed on stdin (`-F -`), and `git` run through
  `util::command` in the repository's work directory, never through a shell.
- D8: Local repositories only.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Claude Code turn in a Marley terminal ends having changed files, the terminal's rail row shall list the turn with its prompt and the number of files it changed. | Shot `509-01-turns` |
| REQ-002 | WHEN a listed turn is clicked, Marley shall open a diff of that turn's changes alone, edits made through the shell included. | Shot `509-02-first-turn` |
| REQ-003 | WHEN a new prompt arrives before the open turn's Stop, the turn shall close there with its changes. | Shot `509-03-closed-by-a-prompt` |
| REQ-004 | WHEN a turn ends in StopFailure, its row shall be marked failed. | Shot `509-01-turns` |
| REQ-005 | WHEN a turn changes nothing, Marley shall list no turn for it and pin nothing. | Shot `509-01-turns`; the run log's refs |
| REQ-006 | WHEN a harness-injected prompt starts a turn that changes files, the turn shall be listed marked injected and titled by what injected it. | Shot `509-01-turns` |
| REQ-007 | WHEN a turn is listed, its commit shall be pinned under `refs/marley/turns/<session>/<n>`, with the turn's start as its parent, and the user's index and branch shall be as they were. | The run log |
| REQ-008 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, check #519 has shipped, time a checkpoint of the
  marley_ide repository itself (the box's largest tree), and re-verify the git seams.
- **P2 Code:** turn tracking on #519's events, the checkpoints, the turn commit and its ref, the
  prune; the rail's disclosure and rows; the click to `CommitView`. fmt and clippy clean; a review
  of the diff.
- **P3 Test:** write and run the scenario, read every shot, `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
