# Per-turn diffs for Claude Code in a terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-509-per-turn-diffs.md
- **Pipeline spec:** 509-per-turn-diffs.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 7 of the list after the browser
  waves: Claude Code's turns in a terminal as snapshots, each turn's diff in a review view. The
  Orca survey the same day settled the snapshots (Zed's checkpoints), the turn boundaries and the
  refs (`docs/orca_architecture/README.md`, #509 and open question 6). The ticket doc is revised to
  match.
- **Classification / tier:** feature, prong 2 (review). Marley crates only. Depends on #519.
  Size M.
- **Recall (§18.3):**
  - AD-claude-477: the agent bar finds a terminal's branch from the innermost repository whose
    work directory holds the folder; the turn's repository is found the same way.
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001: blocking work off the
    main thread is `background_spawn(futures::future::lazy(…))`; the `git commit-tree` and
    `for-each-ref` runs follow it, or go through `util::command`'s async process.
  - PR-claude-a-marley-crate-writes-from-the-contract-not-the-gpl-body-001: the turn code calls
    `Repository`'s public methods and `CommitView::open`; nothing of their bodies is carried over.
  - L-claude-480: the fake acts the program out; here it also makes real edits in the repository.
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery:** each seam opened and checked on 2026-09-25, at commit `520a6e22a7`.
  - `crates/project/src/git_store.rs:488` (`GitStoreCheckpoint`, its per-repository map
    private), `:2133` (`GitStore::checkpoint` over every repository), `:9853`
    (`Repository::checkpoint`, a job returning `GitRepositoryCheckpoint`), `:10027`
    (`Repository::diff_checkpoints`, the patch as a `String`), `:9365` (`update_ref`), `:9373`
    (`delete_ref`), `:7180` (`load_commit_diff`).
  - `crates/git/src/repository.rs:1333` (`GitRepositoryCheckpoint { pub commit_sha: Oid }`),
    `:3063` (the checkpoint: a temporary index, `add --update`, untracked files, `write-tree`,
    `commit-tree -p HEAD -m Checkpoint`), `:3774` (untracked files over 2 MB left out, with an
    excludes file of its own), `:4278` (the author, "Zed"), `:1448` (`load_commit`:
    `git show --first-parent --raw`), `:1135` and `:2797` (the trait's `update_ref`).
  - `crates/git_ui/src/git_ui.rs:41` (`pub mod commit_view`), `crates/git_ui/src/commit_view.rs:183`
    (`CommitView::open(commit_sha, repo, workspace, stash, file_filter, window, cx)`, which loads
    the diff and the commit's details, message included), and its callers in
    `blame_ui.rs:249` and `commit_context_menu.rs:69`.
  - `crates/git_ui/src/multi_diff_view.rs:148` (`MultiDiffView::open` takes pairs of paths on
    disk), `crates/git_ui/src/branch_diff.rs:48` (`BranchDiff`, a branch against the tree): neither
    shows two commits.
  - `crates/marley_workbench/src/agent_bar.rs:93-126` (`contents` gathers the repositories'
    work directories), `:138` (`branch_for`, the innermost one).
  - `crates/marley_workbench/Cargo.toml`: no `git` or `git_ui` dependency yet; `project` is one.
  - #519's spec: the seat's labels carry `prompt`, `session_id` and `cwd`, and the fold knows a
    harness-injected prompt and the post-compaction continuation.
- **Decisions:** D1 to D8 in the spec.

### Design
- **Approach.**
  1. `marley_agent::claude_events` (#519's module) gains `turn_boundary(event) ->
     Option<Boundary>`: `Open { prompt, injected_by }`, `Close { failed }`, or `CloseAndOpen` for a
     UserPromptSubmit while a turn is open; the continuation gives none.
  2. `marley_workbench::turns` (new): per seat, the open turn (its checkpoint, prompt, number, the
     repository's weak handle) and the session's listed turns. On `Open`, find the repository
     (`branch_for`'s rule over `project.git_store().read(cx).repositories()`, against the event's
     `cwd`), `Repository::checkpoint`. On `Close`, `Repository::checkpoint` again; when its tree
     differs from the open's (`git rev-parse <sha>^{tree}`, or `compare_checkpoints`), run
     `git commit-tree <close>^{tree} -p <open> --no-gpg-sign -F -` in the work directory with the
     author and committer set to Marley, the prompt and trailers on stdin, then
     `Repository::update_ref("refs/marley/turns/<session>/<n>", sha)`, then count the files with
     `git diff --name-only <open> <sha>`. `CloseAndOpen` reuses the close's checkpoint as the next
     open. A session's first pinned turn runs the prune: `git for-each-ref
     --format='%(refname) %(committerdate:unix)' refs/marley/turns/` and `Repository::delete_ref`
     for each older than 30 days.
  3. `agent_events` (#519) calls `turns::on_event` with each decoded event after its fold, in
     arrival order, on the main thread; the git work runs in the repository's job queue and in
     background processes, and a turn waits for its open checkpoint before its close is taken.
  4. `marley_rail` (pure): `TurnRow { project, terminal, index, title, files, failed, injected }`;
     `TerminalSnapshot` gains `turns` and an expanded flag the rail keeps per terminal;
     `rail_rows` puts a "Turns (N)" row under the agent row, and the turn rows under it when open.
  5. `rail.rs`: the disclosure toggles the flag; a turn row's click calls
     `git_ui::commit_view::CommitView::open(sha, repository, workspace, None, None, window, cx)`.
- **File manifest.** Marley only: `crates/marley_agent/src/claude_events.rs`,
  `crates/marley_workbench/src/turns.rs` (new), `crates/marley_workbench/src/agent_events.rs`,
  `crates/marley_workbench/src/marley_workbench.rs` (the module),
  `crates/marley_workbench/Cargo.toml` (`git_ui`, `git`), `crates/marley_rail/src/marley_rail.rs`,
  `crates/marley_workbench/src/rail.rs`, `script/e2e/509-per-turn-diffs.sh` (Test).
- **Ledger rows.** None: no Zed path changes. `Cargo.lock` regenerates.

### E2E plan
Fixtures: `git init` of a scratch repository with `src/lib.txt` and `src/util.txt` committed and
one staged change of the user's to `src/util.txt`; #519's fake `claude` with the five-turn session
of the spec, whose steps edit files with Python and `sed -i` between events; the run log records
`git status --porcelain`, `git diff --cached --stat` and `git rev-parse HEAD` before the session.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-004, REQ-005, REQ-006 | run the five turns (Enter between events); click the row's "Turns" disclosure | `509-01-turns` |
| REQ-002 | click "Add a README" | `509-02-first-turn` |
| REQ-003 | back to the terminal; click "Fix the typo in util" | `509-03-closed-by-a-prompt` |
| REQ-007 | after the session: `git for-each-ref refs/marley/turns/`; for each ref, its parent's message (`git log -1 --format=%s <ref>^`); the three before-commands again | the run log: four refs, each parent a Zed checkpoint (message `Checkpoint`), the index, the status and `HEAD` unchanged |

The prune is not reached by a scenario in real time; setup can plant a ref whose commit is dated
40 days back (`GIT_COMMITTER_DATE`) and the run log shows it gone after the first turn.

### Risks
- A checkpoint of a large tree: `add --update` and `ls-files --others` over the marley_ide tree
  may take seconds. Measured at promotion; if it is slow, the close's checkpoint still runs off the
  main thread, and the rail shows the turn when its commit exists.
- The open checkpoint races the agent's first edit: the frame reaches Marley a few milliseconds
  after the prompt, and the model's first reply takes seconds, so it normally wins; an edit that
  beats it lands in the previous turn.
- A shared checkout: the user's edits and another agent's during a turn land in it (Out, with
  the reason); the rail says nothing of it in this slice.
- Refs accumulate between prunes; each is one ref, and the objects are shared with the checkouts'
  history.
- `git` must be on the PATH Marley runs with; Zed's own git work uses the same binary.
