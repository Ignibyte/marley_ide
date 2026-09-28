---
pipeline_id: 4e39dc0b-b7a9-4912-b583-542ecc9461cc
ticket: docs/planning/tickets/closed/TICKET-509-per-turn-diffs.md
status: Phase 4 — Complete PASS
title: "Per-turn diffs for Claude Code in a terminal"
type: feature
slice: prong 2 (review), on #519's events
references: [docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/queued/541-spawn-in-adapters-ratchet.spec.md, docs/orca_architecture/README.md, docs/orca_architecture/02-worktrees-and-review.md, docs/orca_architecture/01-agents-and-sessions.md, docs/planning/design-notes/warp-once-over-2026-09-25.md]
---

## Title
Marley snapshots a terminal's repository when each Claude Code turn opens and closes, records a
turn that changed files as a commit whose parent is the turn's start, pins it under
`refs/marley/turns/`, and lists it under the terminal's row in the rail. A click opens the turn in
Zed's commit view: a diff multibuffer of that turn's changes and nothing else, the edits made
through the shell included. Chad reviews an agent's work turn by turn.

## Scope
### In
- **Turns**, per terminal seat, from #519's fold as it stands (`after_fold`). A turn opens at a
  lead UserPromptSubmit, the user's, a slash command's or a harness's, and not at the
  post-compaction continuation. It closes at Stop, StopFailure, a manual PostCompact, an
  interrupt (a lead PostToolUseFailure with `is_interrupt`), SessionStart, SessionEnd, a new
  session in the terminal, the next UserPromptSubmit, or when Claude Code leaves the terminal or
  the terminal closes (`agent_events::end` and `forget`).
- **Snapshots.** At the open and at the close, `Repository::checkpoint` of the innermost local
  repository whose work directory holds the event's `cwd`, else the seat's `cwd` label, else the
  terminal's working directory (the agent bar's rule, `branch_for`). A close that a new prompt
  causes serves as the next turn's open, so one checkpoint does both. The repository's job queue
  runs them in order.
- **The turn's commit.** When `compare_checkpoints` finds the close's tree different from the
  open's, Marley writes a commit with the close's tree and the open's checkpoint as its one
  parent: `git commit-tree <close>^{tree} -p <open> -F -` in the repository's work directory,
  through an adapter module (`turn_git.rs`, #541's pattern), the message the turn's title with
  the trailers `Marley-Session` and `Marley-Turn`, author and committer "Marley". It pins the
  commit at `refs/marley/turns/<session_id>/<n>` with `Repository::update_ref`, `<n>` counting on
  from the refs the session already has, so a resumed session never overwrites its turns. The
  files it changed come from `Repository::diff_tree`. A turn that changed nothing writes and pins
  nothing.
- **The rail.** Under a terminal row with turns, a "Turns (N)" line with a disclosure; open, the
  session's turns newest first, each with its title on one line, the number of files it changed,
  `failed` after StopFailure, and `injected` for a turn a harness's prompt started, titled by
  what injected it (`task notification`) instead of its text; a slash command's turn is titled by
  the command. The turns stay listed after Claude Code leaves the terminal.
- **The view.** A click on a turn opens Zed's `CommitView` on the turn's commit (its full sha).
  Its diff is against the first parent, the turn's start, so it shows the turn alone.
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
- The rail's keyboard walk into the turn rows: they take clicks in this slice.
- The CommitView toolbar's "View on …", "Show in Git Graph" and review buttons, which do nothing
  useful for a turn commit on no branch.

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
- **Read again at promotion** (2026-09-28, an Explore report over `8d136c2bc0`): every Zed seam
  above holds, unchanged since the draft's discovery. `Repository` has no `commit_tree`, no
  `write_tree`, no raw run and no ref listing (the trait, `crates/git/src/repository.rs:789-1143`;
  `GitBinary` is `pub(crate)`), and `Cargo.lock` holds neither `git2` nor `gix`, so the commit
  and the prune's `for-each-ref` take a spawn, in an adapter, as #541 plans. `compare_checkpoints`
  (`git diff-tree --quiet`) and `diff_tree` (`DiffTreeType::Since`) replace the draft's
  `rev-parse` and `diff --name-only`. A checkpoint leaves out ignored files, untracked ones of
  2 MB or more and binaries, archives and media (`checkpoint.gitignore`). `CommitView::open`
  opens in the active pane, dedupes by the full sha, and draws toolbar buttons a branchless
  commit cannot use. The prompt reaches Marley collapsed and cut to 300 characters.

## UI proof
UI-AFFECTING: turn rows under a terminal's rail row and the diff they open.
`script/e2e/509-per-turn-diffs.sh` (`compositor sway`: the disclosure and the turns are clicked).
A scratch repository committed with two files, `src/lib.txt` and `src/util.txt`, and one staged
change of the user's to `src/util.txt`, a git identity of the scenario's own through
`GIT_CONFIG_GLOBAL`, and a planted turn ref dated 40 days back; #566's stand-in `claude` with
pseudo-events that edit files between its hook events, in five turns: "Add a README" writes
`README.md` and runs `sed -i` on `src/lib.txt`, then Stop; "Fix the typo in util" edits
`src/util.txt` and sends no Stop; "Explain the build" (which closes the turn before it) changes
nothing, then Stop; a `<task-notification>` prompt edits `src/lib.txt`, then Stop; "Refactor the
parser" edits `src/util.txt`, then StopFailure. Shots:
- `509-01-turns`: the row's "Turns (4)", opened: newest first "Refactor the parser · 1 file ·
  failed", "task notification · injected · 1 file", "Fix the typo in util · 1 file" and "Add a
  README · 2 files"; "Explain the build" absent.
- `509-02-first-turn`: "Add a README" opened in the commit view: `README.md` added and the `sed`
  change to `src/lib.txt`, nothing of the later turns, nothing of the user's staged change.
- `509-03-closed-by-a-prompt`: "Fix the typo in util" opened: its one change, which the next
  prompt closed with no Stop between.
The run log records `git for-each-ref refs/marley/turns/`, each turn commit's parent (a Zed
checkpoint, message `Checkpoint`), the planted old ref gone, and `git status --porcelain`, `git
diff --cached --stat` and `git rev-parse HEAD` before and after the session, which match.

## Locked-In Decisions
- D1: Snapshots are Zed's git checkpoints, not edits rebuilt from tool calls. They catch the
  changes in the tree, shell commands included, and touch neither the user's index nor a ref.
  Changed at promotion: a checkpoint leaves out ignored files, untracked files of 2 MB or more,
  and untracked binaries, archives and media, so a turn does too.
- D2: A turn opens at a UserPromptSubmit and closes at Stop, StopFailure, a manual PostCompact, the
  next UserPromptSubmit or SessionEnd. An API error sends StopFailure without Stop, a manual
  `/compact` ends without Stop, and a cancelled turn sends nothing until the next prompt (report
  02 §2.13). Changed at promotion: it also closes where #519's fold ends a turn, at an interrupt,
  a SessionStart and a new session in the terminal, and when Claude Code leaves or the terminal
  closes, so no turn outlives its session.
- D3: The view is Zed's commit view on a turn commit whose parent is the turn's opening
  checkpoint. `load_commit` diffs against the first parent, so the view shows the turn alone and
  Marley adds no diff view.
- D4: Turn commits are pinned under `refs/marley/turns/<session_id>/<n>` (the survey's default),
  so neither `git gc` nor a restart loses them. Refs older than 30 days go when a new session pins
  its first turn; #511 removes a worktree's with it.
- D5: A turn that changed nothing is not listed and pins nothing.
- D6: A harness-injected prompt starts a turn of its own, marked and titled by what injected it;
  the post-compaction continuation starts none (report 01, item 3). Changed at promotion: a slash
  command's envelope (`command-name`, `command-message`, `command-args`) is the user's, titled by
  the command and not marked; a prompt injected by a prefix with no tag is titled "injected
  prompt".
- D7: The turn commit is Marley's: author and committer "Marley" with the address
  `marley@localhost`, `--no-gpg-sign` so a signing config never prompts, the message passed on
  stdin (`-F -`), and `git` run through `util::command` in the repository's work directory, never
  through a shell, from one adapter module (`turn_git.rs`), which #541's `process.rs` takes over
  when it lands.
- D9 (at promotion): turn numbers count on from the refs a session already has, so a resumed
  session, in another terminal or after a restart, adds turns rather than overwriting them.
- D10 (at promotion): the turns belong to the terminal for the Marley session, and stay listed
  after Claude Code leaves it; the refs keep them past a restart (#540 lists them later).
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
| REQ-008 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |
| REQ-009 | WHEN a session pins its first turn, Marley shall delete this repository's turn refs whose commits are older than 30 days. | The run log: the planted ref gone |

## Phase Plan
- **P1 Plan:** this spec; at promotion (2026-09-28): #519 has shipped; the git seams re-verified;
  the scans a checkpoint makes of the marley_ide tree timed (the notes' "Promotion").
- **P2 Code:** `claude_events::prompt_origin`; `turns.rs` (the store, the boundaries, the
  checkpoints, the commit, the ref, the prune) and `turn_git.rs` (the two spawns); the calls
  from `after_fold`, `end` and `forget`; the rail's turns under a terminal row and the click to
  `CommitView`; `git` and `git_ui` in the workbench's dependencies. fmt and clippy clean; a review
  of the diff.
- **P3 Test:** write and run the scenario, read every shot, `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
