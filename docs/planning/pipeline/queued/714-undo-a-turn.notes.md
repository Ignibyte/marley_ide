# Undo a turn — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-714-undo-a-turn.md
- **Pipeline spec:** 714-undo-a-turn.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** the finish-line list (`docs/planning/design-notes/finish-line-2026-10-11.md`, row
  5). Chad, 2026-10-11: "meh not sure. convince me of this or not". The case put to him: Marley
  already keeps each Claude Code turn as a commit under `refs/marley/turns/<session>/<n>`, shell
  edits included (#509); Undo This Turn takes out only that turn's changes with a three-way reverse
  apply that keeps later turns and the user's own edits, refuses with the reason on a conflict, and
  refuses while another agent works in the same checkout (T3's `CheckpointRestoreSafety.ts`); Keep
  marks a turn reviewed (MonoCode). His answer: "ok we can build it". It builds after #753
  (Codex's turn checkpoints), so one Undo covers both agents.
- **Classification:** feature, size M, one slice. Marley crates only; no Zed path changes.
  Depends on #509 (shipped), #541 (shipped), #550 (shipped) and #753 (queued, for Codex rows).
- **Recall (§18.3):**
  - AD-claude-509-a-turn-is-a-commit-of-its-end-on-its-start-under-marleys-refs-001: the turn's
    commit has the start checkpoint as its one parent, so `<turn>^` is the start and `<turn>` the
    end; it rejected `diff_checkpoints` as text for viewing, which this ticket confirms for
    applying (nothing in Zed applies a patch).
  - L-claude-509-a-zed-checkpoint-is-a-commit-on-head-of-the-whole-tree-001: the fresh "now"
    checkpoint follows the same rules as the turn's, so the three trees are comparable; ignored
    files, untracked files of 2 MB or more and binaries are in none of them.
  - L-claude-509-a-scenario-agent-edits-after-its-prompts-checkpoint-001: the stand-in waits two
    seconds after each prompt before its edits.
  - F-claude-509-a-turn-would-have-opened-in-the-repository-its-terminal-moved-to-001: the undo
    uses the row's own `Turn.repository`, not the seat's current one.
  - PR-claude-a-number-taken-across-awaits-is-taken-by-one-task-at-a-time-001: two undos in one
    repository are chained (D12).
  - AD-claude-git-write-confined-to-add-restore-commit-001 (gpui era, its rule still stands): new
    git writes extend the adapter on purpose. D4 does, and Complete records an AD.
  - PR-claude-a-repositorys-text-in-a-zed-prompt-goes-in-a-code-block-001: paths in the question
    and the refusals go in a fenced block (`launch::verbatim`).
  - PR-claude-column-significant-git-output-is-read-untrimmed-001: `merge-tree -z` and
    `diff-tree -z` output is split on NUL and read as it came.
  - PR-claude-inspect-critic-must-not-git-checkout-the-working-tree-001: the Code phase's review
    tries the git sequence in a scratch repository, never in the live tree.
  - #550 (`close_guard.rs`): the working rule the agent check reuses.
  - #731 (queued): per-file Viewed marks in the project diff; Keep is a per-turn mark in the rail,
    a different thing.
  - Brain: no `rusty` MCP server in this repository's sessions; promotion asks.
- **Discovery** (at `19dda499a2`; #743, active, is changing `rail.rs`, so its lines move):
  - Zed, `crates/git/src/repository.rs`: `checkpoint` 3080, `restore_checkpoint` 3108 (its
    `clean` commented out, 3121-3130), `create_archive_checkpoint` 3138, `restore_archive_checkpoint`
    3191, `compare_checkpoints` 3220, `diff_checkpoints` 3254, `stash_apply` 2712, `GitBinary`
    3894 (`pub(crate)`), `with_temp_index` 3928, `path_for_index_id` 3963 (`index-<uuid>.tmp` in
    the git directory). No `apply`, `merge-tree` or three-way merge in `git`, `project`, `git_ui`,
    `acp_thread` or `action_log`.
  - Zed, `crates/project/src/git_store.rs`: the jobs `checkpoint` 9975, `restore_checkpoint`
    9998, `compare_checkpoints` 10123, `diff_checkpoints` 10149, `update_ref` 9487, `delete_ref`
    9495, `diff_tree` 9713.
  - Zed, `crates/acp_thread/src/acp_thread.rs`: `restore_checkpoint` 6608 (cancel, rewind, then
    the whole tree), `ThreadStatus` 3605, `work_dirs` 3938, `status` 3947.
  - Zed, buffers: `BufferStore::get_by_path` (`crates/project/src/buffer_store.rs:1104`),
    `Buffer::is_dirty` (`crates/language/src/buffer.rs:2525`), `Project::find_project_path`
    (`crates/project/src/project.rs:5419`).
  - Marley, `crates/marley_workbench/src/turns.rs`: `Turn` 35 (`sha`, `repository`), `Seat` 66
    (`open` 70, private), `Turns` 79, `of` 90, `repository_of` 98, `on_event` 108, `record` 343.
  - Marley, `turn_git.rs`: `TURN_REFS` 12, `commit_tree_in` 23, `turn_refs_in` 70, both over
    `process::output` (`process.rs:21`); `.config/spawn-sites.txt` lists `process.rs` for git.
  - Marley, `rail.rs`: `toggle_turns` 4270, `open_turn` 4368, `right_click_menu` for a terminal
    row 5572 and `terminal_context_menu` 5595, `render_turns` 5727, the snapshot's turns 7851;
    `window.prompt` patterns at 2934 (Delete thread) and 3361 (Remove worktree).
  - Marley, `crates/marley_rail/src/marley_rail.rs`: `TurnSnapshot` 233.
  - Marley, `close_guard.rs`: `working_agents` 155, `working_status` 184 (a seat Starting, Working
    or Waiting; else output in the last two seconds and no bell), both private.
  - Marley, agents and folders (an Explore report): `agent_bar::agent_in` 141 (the foreground
    program is a known agent); `agent_events::seat_of` 123; `claude_events::CWD_LABEL` (Claude
    Code seats only; Codex seats carry none, `codex_events.rs:469-520`); `Terminal::working_directory`
    (`crates/terminal/src/terminal.rs:3531`, `None` for a remote terminal);
    `send_selection::terminal_views` 304; `thread_tab::conversations_of` 955; `rail.rs`
    `live_statuses` 7611; every window's workspaces as `marley_workbench.rs:1127` walks them.
    Harness seats expose no local folder (`harness_seat.rs:403-406`, private and partial). No
    helper answers "which agents work in or around this folder" yet.
- **Proven in a scratch repository** (git 2.55, the scratchpad, never the live tree), with
  checkpoints made as Zed makes them:
  - Turn 1 (README added, line 1 changed), turn 2 (line 5 and `util.txt`), then the user's own
    edit (line 3, a new `notes.txt`) and a staged change of the user's. `merge-tree --write-tree
    --merge-base=<turn 1> <now> <turn 1>^` exited 0 ("Auto-merging src/lib.txt"); the temporary
    index's `read-tree -m -u <now> <merged>` removed `README.md`, put line 1 back, kept line 3 and
    line 5, left `notes.txt`, and `git diff --cached` was as before.
  - A turn whose line the user changed since, and whose new file the user edited: exit 1, the
    conflicted files listed (`new.txt` modify/delete, `src/lib.txt` content), nothing on disk
    changed.
  - A file changed after the checkpoint: `read-tree -m -u` refused ("Entry 'src/lib.txt' not
    uptodate. Cannot merge.", exit 128) and wrote nothing, the turn's new file still there.
- **Decisions:** D1 to D12 in the spec.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again. The drafted design follows.

### Design
- **Approach.**
  1. `turn_git.rs` (adapter): `undo_merge_in(work_dir, turn, now) -> Merge` runs `git merge-tree
     --write-tree --name-only -z --merge-base=<turn> <now> <turn>^`; exit 0 gives the merged tree,
     exit 1 `Conflict(paths)`, anything else an error (an unknown `--merge-base` reads as git
     older than 2.40, with `git --version` named). The changed paths: `Repository::diff_tree`
     (`DiffTreeType::Since`) if it takes a tree, else `git diff-tree -r -z --name-status <now>
     <merged>` here; each path restored or removed. `write_merged_in(work_dir, now, merged)`: a
     temporary index in the repository's git directory (`git rev-parse --git-dir`,
     `index-marley-undo-<uuid>.tmp`), `read-tree <now>`, `update-index -q --refresh`,
     `read-tree -m -u <now> <merged>`, the file removed whatever happens; a "not uptodate" exit
     gives `Changed(paths)`.
  2. `turn_undo.rs` (new): `undo(view, sha, window, cx)` from the menu. The row's `Turn` and its
     repository (`Turns::repository_of`); the agent check; `Repository::checkpoint`; the merge;
     nothing to undo when the merged tree equals `<now>`'s; the unsaved check over every
     project's `BufferStore` in every window; the question; the agent check again; the write; the
     mark. One undo at a time per repository (a shared task per work directory, chained). Every
     refusal is a `window.prompt` with one button and the paths through `launch::verbatim`.
  3. The agent check, `agents_in(checkout, cx) -> Vec<(name, folder)>`: every window's
     workspaces; each terminal view (`terminal_views`) whose `agent_in` is `Some` and whose
     `close_guard::working_status` is `Some`, its folder the seat's `cwd` label, else
     `working_directory()`; each conversation (`conversations_of`) whose thread is `Generating`,
     its folders `work_dirs()`, else the project's visible worktree roots. Paths canonicalized;
     a hit is either path starting with the other.
  4. `turns.rs`: `Turn` gains `kept` and `undone`; `Turns::mark(view, sha, mark)` sets one and
     notifies the rail through the global.
  5. `close_guard.rs`: `working_status` becomes `pub(crate)`.
  6. `rail.rs`: each turn row wrapped in `right_click_menu(("marley-rail-turn-menu", …))` with
     Keep (unless kept or undone) and Undo This Turn… (unless undone); the marks `· kept` and
     `· undone` after `· failed` and `· injected`; the snapshot copies them.
  7. `marley_rail.rs`: `TurnSnapshot` gains `kept` and `undone`; the tests' literals gain them.
  8. The guide's "Per-turn diffs": Keep, Undo, the refusals, the index left alone, and what a
     checkpoint leaves out.
- **File manifest** (Marley crates only): `crates/marley_workbench/src/turn_git.rs`,
  `crates/marley_workbench/src/turn_undo.rs` (new), `crates/marley_workbench/src/turns.rs`,
  `crates/marley_workbench/src/close_guard.rs`, `crates/marley_workbench/src/rail.rs`,
  `crates/marley_workbench/src/marley_workbench.rs` (the module),
  `crates/marley_rail/src/marley_rail.rs`; `docs/marley/guide.md`;
  `script/e2e/714-undo-a-turn.sh` (Test).
- **Ledger rows:** none. No path outside `crates/marley_*` changes; `.config/spawn-sites.txt`
  already lists `process.rs` for git.

### Visual check plan
| REQ | The scenario does | The shot or log |
|---|---|---|
| 001 | Four turns in terminal 1; the user's edits; Turns opened; "Add a README" right-clicked | `714-01-menu`: Keep and Undo This Turn… |
| 002 | Undo This Turn… chosen | `714-02-confirm`: `README.md` removed, `src/lib.txt` restored, the conversation line |
| 003, 004, 005, 006 | Undo Turn | `714-03-undone`: the row `· undone`; the log: `README.md` gone, line 1 back, lines 3, 5, 9 and `notes.txt` kept, `HEAD`, index and refs as before |
| 008 | Undo on "Shout gamma" | `714-04-conflict`: `src/lib.txt` named; the log: no file changed |
| 009 | Keep on "Fix the typo in util" | `714-05-kept` |
| 010 | `src/util.txt` opened and typed into; Undo on "Fix the typo in util"; the edit undone after | `714-06-unsaved`: `src/util.txt` named |
| 011 | Terminal 2 runs `claude busy` and its prompt; Undo on "Shout iota" | `714-07-another-agent`: the agent and the folder named; the log: no file changed |
| 013 | The busy turn stopped; Undo on "Shout iota"; line 9 changed while the question is open; Undo Turn | `714-08-changed-meanwhile`; the log: line 9 as the scenario left it, nothing else changed |
| 007, 012, 014 to 018 | Not driven | Review of the diff; the gate |

Points come from a first run's shots, as #509's did (the turn rows sit under a terminal card).
The scenario's user edits are made from the script while no turn is open, so they land in no
turn.

### Risks
- git merges changes on adjacent lines as one change, so an undo beside a later edit refuses
  though the lines differ. The refusal is honest; the fixture keeps an unchanged line between.
- What a checkpoint leaves out (ignored files, untracked files of 2 MB or more, binaries, archives,
  media) is in no turn, so Undo cannot take it out. The guide says so.
- A change the user staged after the turn stays staged: the undo writes the tree only, so `git
  status` shows the staged change and the tree's undo side by side. The guide says so.
- The agent's conversation still believes its changes are there; the question says to tell it.
- A clean buffer of a file the undo changes reloads from disk, as Zed does for any outside write.
- A git older than 2.40 refuses Undo (D2). The dev box has 2.55.
- The undo's spawns run outside the repository's job queue, as #509's commit does. The temporary
  index keeps the user's index out of it, and the write's up-to-date check guards the tree.
- #753's rows: if Codex's turns live outside `turns::Turn`, the menu reads that store too (D11);
  promotion checks.
- The rail's rows move when a terminal's turns are open; the scenario takes its points from a first
  run, as #509's did.
