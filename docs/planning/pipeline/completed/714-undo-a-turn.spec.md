---
pipeline_id: 5c94bac9-e515-4351-b298-3e173c1abfa8
ticket: docs/planning/tickets/open/TICKET-714-undo-a-turn.md
status: Phase 4 — Complete PASS
title: Undo a turn
type: feature
slice: the finish line (docs/planning/design-notes/finish-line-2026-10-11.md row 5), prong 2 (review), on #509's turns; builds after #753
references:
  - docs/planning/pipeline/completed/509-per-turn-diffs.spec.md
  - docs/planning/pipeline/completed/541-spawn-in-adapters-ratchet.spec.md
  - docs/planning/pipeline/completed/550-ask-before-ending-a-working-agent.spec.md
  - docs/t3code_architecture/03-git-worktrees-and-prs.md
  - docs/t3code_architecture/01-agents-and-providers.md
  - docs/planning/intake/monocode-findings.md
---

## Title
Keep and Undo This Turn… on each turn row in the rail. Undo takes one turn's changes out of the
working tree with a three-way merge, so later turns and the user's own edits stay, and refuses with
the reason when it cannot do that safely. Keep marks the turn reviewed. Chad, 2026-10-11: "meh not
sure. convince me of this or not", then, after the case for and against, "ok we can build it".

## Scope
### In
- **A menu on a turn row.** Right-click on a row under "Turns (N)" (#509) opens a menu with
  **Keep** and **Undo This Turn…**. A plain click still opens the turn's diff.
- **Undo This Turn…**, in this order:
  1. **The agent check.** Refuse while any agent works in the turn's checkout, in a folder inside
     it, or in a folder around it, the turn's own agent included: a terminal agent that is
     starting, working or waiting (#550's rule), or an Agent Panel or tab thread that is
     generating. The refusal names the agent and its folder.
  2. **The merge.** A fresh checkpoint of the turn's repository (`Repository::checkpoint`), then a
     three-way merge of trees: base the turn's commit, ours the fresh checkpoint, theirs the
     turn's start (the commit's parent). Nothing on disk changes. A conflict refuses and names the
     files. A merge that changes nothing says there is nothing to undo.
  3. **The unsaved check.** Refuse when a file the merge would change is open with unsaved edits
     in any Marley window, naming it.
  4. **The question.** "Undo the turn “<title>”?", listing each file it will restore or remove,
     and saying that the agent's conversation is not changed, so tell the agent. Buttons Undo Turn
     and Cancel.
  5. **The write.** The agent check again, then the merged tree written to the working tree
     through a temporary index: only the paths the merge changed, files the turn created and
     nobody changed since removed. When any of those paths changed since the checkpoint of step 2,
     git refuses the whole write before it writes a file, and Marley says so.
  6. **The row.** `· undone` on the row; its menu offers neither Keep nor Undo again.
- **Keep** marks the row `· kept`. Undo stays on a kept row.
- The user's index, `HEAD`, branches and the turn refs are never touched. The undone turn's ref
  stays, so its diff still opens.
- One undo at a time per repository.
- Every git run goes through `turn_git.rs` over `process.rs` (#541), argv only.
- Codex: the undo reads a row's commit and its parent and nothing else, so it covers every row in
  the turns store. #753 is to record Codex's turns there (D11); this ticket adds nothing for
  Codex.
- `docs/marley/guide.md` ("Per-turn diffs"): Keep, Undo and its refusals, what stays, and what a
  turn cannot hold.

### Out (explicitly deferred)
- Rewinding the agent's conversation: Claude Code's `/rewind` owns it (T3 report 03, §4).
- Restoring the tree to a turn's start and dropping everything after it (T3 Code's Revert files
  too): Undo takes out one turn.
- Several turns at once, and per-hunk or per-file undo.
- Redo. The turn's ref keeps the commit, so the change can be applied again by hand.
- Marks that outlive the rows: the rows end with their terminal (#509 D10).
- Per-file Viewed marks in the project diff: #731.
- Undo from the commit view's toolbar: a change to `git_ui`.
- Remote repositories: a remote project has no turns (#509 D8).
- Harness seats with no local folder: a harness seat in a local terminal is checked as a terminal.
- The keyboard walk into turn rows (#509 left them to clicks).

## Reference (§20)
- **Upstream Zed:** the Agent Panel's Restore Checkpoint on a user message
  (`AcpThread::restore_checkpoint`, `crates/acp_thread/src/acp_thread.rs:6608`) cancels the
  thread, rewinds it and restores the whole tree to the message's checkpoint through
  `Repository::restore_checkpoint`. Kept as it is for Zed's threads. Marley's Undo differs on
  purpose: one turn out, later work kept, a terminal agent's conversation left alone.
- **T3 Code** (survey, research only): Edit from here with Revert files too, refused while another
  thread or live session uses the folder, a folder inside it or one around it
  (`docs/t3code_architecture/03-git-worktrees-and-prs.md` §2.3, citing
  `CheckpointRestoreSafety.ts:13-88`). Marley takes the containment check and the two-time check.
- **MonoCode:** Keep and Undo per turn (`docs/planning/intake/monocode-findings.md`, item 1).
- No Warp behavior applies; no Warp code was read.

### Prior art
- **Behavior maps and surveys.**
  - T3 report 03, item 1: Undo This Turn… on a turn row, a three-way reverse apply that keeps later
    turns, a conflict that aborts and names the files, the shared-checkout refusal, and the hard
    part: delete a file the turn created only while it still matches the turn. Open question 2's
    default, one turn three-way, is the shape Chad accepted.
  - T3 report 01, §2.6 and item 7: file restore needs the isolation check, run twice; Claude Code's
    `/rewind` misses shell edits and Codex restores no files. Open question 6 (main checkout
    allowed?) is answered by Chad's acceptance: refused only while another agent works there.
  - `docs/zed_architecture/`: no subsystem note covers checkpoints or restore (grep).
- **Published material.** git's `merge-tree --write-tree --merge-base` (git 2.40 and later): a
  three-way merge of trees that writes objects only and exits 1 on a conflict, listing the
  conflicted files (`--name-only -z`). `read-tree -m -u <old> <new>`, the two-tree merge, which
  updates only the paths that differ and refuses an entry that is not up to date before it writes
  anything. `GIT_INDEX_FILE` for a temporary index. `apply --3way`, which implies `--index` and
  leaves conflict markers in the files.
- **The code we already ship.**
  - Zed's `git` crate (`crates/git/src/repository.rs`) and `project`'s `Repository` jobs
    (`crates/project/src/git_store.rs`):
    - `checkpoint` (3080; job 9975): a temporary index, `add --update`, the untracked files it lets
      through, `write-tree`, `commit-tree -p HEAD`. Taken: the "now" side of the merge.
    - `restore_checkpoint` (3108; job 9998): `git restore --source <sha> --worktree .`. It puts the
      whole tree back to that point, dropping later turns and the user's edits, and its `clean`
      is commented out (3121-3130), so files added since stay. Rejected for one turn.
    - `diff_checkpoints` (3254; job 10149): the patch as text. Nothing in Zed applies a patch.
    - `compare_checkpoints` (3220; job 10123).
    - `create_archive_checkpoint` and `restore_archive_checkpoint` (3138, 3191):
      `read-tree --reset -u` on the user's index. Rejected.
    - `stash_apply` (2712) is the only apply, for stashes. No `git apply`, no `merge-tree` and no
      three-way merge in `git`, `project`, `git_ui`, `acp_thread` or `action_log` (grep).
    - `GitBinary` (3894) and `with_temp_index` (3928) are `pub(crate)`; Zed names its temporary
      index `index-<uuid>.tmp` in the git directory (`path_for_index_id`, 3963). Marley's adapter
      sets `GIT_INDEX_FILE` itself, the same way.
    - `Cargo.lock` holds neither `git2` nor `gix`.
  - Marley (#509, #541, #550), lines at `19dda499a2`; #743 is changing `rail.rs` now, so
    promotion reads them again:
    - `turns.rs`: `Turn` (35) with its full `sha` and `repository` (47); `Turns::of` (90) and
      `repository_of` (98); `Seat.open` (70), private.
    - `turn_git.rs`: `commit_tree_in` (23) and `turn_refs_in` (70) over `process::output`
      (`process.rs:21`), the adapter #541 made gate:22's.
    - `rail.rs`: `render_turns` (5727), `open_turn` (4368), and the right-click pattern of a
      terminal row (`right_click_menu`, 5572; `terminal_context_menu`, 5595).
    - `close_guard.rs`: `working_agents` (155) and `working_status` (184), #550's "is this agent
      at work" (a seat starting, working or waiting, else the quiet timer), private.
    - `agent_bar::agent_in` (141); `send_selection::terminal_views` (304, a workspace's center and
      docked terminals); `thread_tab::conversations_of` (955); `rail.rs` `live_statuses` (7611),
      reading `AcpThread::status` (`acp_thread.rs:3947`) and `work_dirs` (3938).
    - `launch::verbatim` (`launch.rs:308`) for paths in a prompt.
  - Zed's buffers: `BufferStore::get_by_path` (`buffer_store.rs:1104`), `Buffer::is_dirty`
    (`buffer.rs:2525`), `Project::find_project_path` (`project.rs:5419`) for the unsaved check.
  - Proven in a scratch repository with git 2.55 (the notes): the merge and the write keep a later
    turn and the user's edit in the same file, remove the file the turn created, and leave the
    index as it was; a conflict exits 1 and names the files; a file changed after the checkpoint
    makes `read-tree` refuse with nothing written.

## UI proof
`script/e2e/714-undo-a-turn.sh`, under `compositor sway` (rows are right-clicked and clicked).
Fixtures: a scratch repository under a git config of the scenario's own (`GIT_CONFIG_GLOBAL`),
committed with `src/lib.txt` (nine lines, `alpha` to `iota`) and `src/util.txt`, and one staged
change of the user's to `src/util.txt`. #509's stand-in `claude`, case `turns`, runs four turns in
one terminal, each by an Enter and each with a Stop: "Add a README" writes `README.md` and changes
line 1; "Shout iota" changes line 9; "Fix the typo in util" changes `src/util.txt`; "Shout gamma"
changes line 3. Then the scenario, as the user, writes `notes.txt`, changes line 5, and changes
line 3 again. Case `busy`: a prompt, then a long wait before its Stop. Each edit sits an unchanged
line away from the others, since git merges adjacent lines as one change. The run log records,
before and after each step, `HEAD`, `git diff --cached`, the turn refs, `git status --porcelain`
and a hash of every file.

Shots:
- `714-01-menu`: "Add a README" right-clicked: Keep and Undo This Turn….
- `714-02-confirm`: Undo This Turn… chosen: the question, `README.md` to be removed and
  `src/lib.txt` restored, and the line about the agent's conversation.
- `714-03-undone`: confirmed: the row "Add a README · 2 files · undone". The run log: `README.md`
  gone, line 1 `alpha`, lines 3, 5 and 9 as the user and the later turns left them, `notes.txt`
  and `src/util.txt` unchanged; `HEAD`, the index and the four refs as before.
- `714-04-conflict`: Undo on "Shout gamma", whose line the user changed since: refused, naming
  `src/lib.txt`. The run log: every file as before.
- `714-05-kept`: Keep on "Fix the typo in util": the row ends `· kept`.
- `714-06-unsaved`: `src/util.txt` opened and typed into, then Undo on "Fix the typo in util":
  refused, naming `src/util.txt`. The edit is undone in the editor afterwards.
- `714-07-another-agent`: a second terminal in the repository runs `claude busy` and its prompt;
  Undo on "Shout iota": refused, naming that terminal's Claude Code and the repository's folder.
  The run log: every file as before.
- `714-08-changed-meanwhile`: the busy turn stopped; Undo on "Shout iota", and while the question
  is open the scenario changes line 9; confirmed: refused, saying the file changed. The run log:
  line 9 as the scenario left it, nothing else changed.

## Locked-In Decisions
- **D1: one turn out, three-way, later work kept.** T3 report 03's open question 2 default and the
  case Chad accepted. Rejected: Zed's `restore_checkpoint` to the turn's start (drops later turns
  and the user's edits, leaves files added since); `git apply -R --3way` (implies `--index`, so it
  stages into the user's index, wants the index to match the tree, and leaves conflict markers in
  files on a conflict); `git apply -R` alone (refuses wherever a later change sits in the patch's
  context, though a merge would be clean); Claude Code's `/rewind` (misses shell edits; Codex has
  none).
- **D2: the merge touches nothing on disk.** `git merge-tree --write-tree --name-only -z
  --merge-base=<turn> <now> <turn>^`, where `<now>` is a fresh `Repository::checkpoint` under the
  same rules as the turn's. Exit 1 is a conflict and its file list; nothing is written. It needs git
  2.40 or newer; an older git refuses with its version named. The dev box has 2.55.
- **D3: the write goes through a temporary index.** In the repository's git directory, as Zed
  names its own: `read-tree <now>`, `update-index -q --refresh`, `read-tree -m -u <now> <merged>`,
  then the file removed. Only the paths that differ are written; a file the turn created and nobody
  changed is removed; a path changed since `<now>` makes git refuse before it writes any file. The
  user's index, `HEAD`, branches and refs stay. Rejected: `git restore --source <merged>
  --worktree` (cannot remove an untracked file the turn created, so Marley would delete files
  itself); the `read-tree --reset -u` of Zed's archive checkpoints (the user's index).
- **D4: the adapter, and the git-write set extended on purpose.** The spawns live in
  `turn_git.rs` over `process.rs` (#541, gate:22), argv only, never a shell. They add to Marley's
  git writes (AD-claude-git-write-confined-to-add-restore-commit-001) objects, a temporary index
  and the merge's own paths in the tree; no reset, checkout, clean or stash. An AD at Complete
  records it.
- **D5: the agent check.** Refused while any agent works in the turn's checkout, a folder inside
  it, or a folder around it, compared both ways on canonical paths (T3's
  `CheckpointRestoreSafety.ts`). "Works" is #550's: a terminal agent whose seat is starting,
  working or waiting, or with no seat the quiet timer; a thread that is generating. The folder is a
  terminal's `cwd` label, else its working directory; a thread's `work_dirs`, else its project's
  roots. The turn's own agent counts: its open turn would take the undo in as its own change. Idle
  agents do not block (Chad: "refuses while another agent works in the same checkout"), so the main
  checkout is allowed when nobody works there. Checked before the question and again before the
  write. Rejected: T3's stricter rule, any live session (it would block an undo whenever Claude
  Code sits idle in the project's terminal, the common case); worktrees only (T3 report 01's
  default, overruled by the same words).
- **D6: unsaved edits refuse.** A file the merge would change, open with unsaved edits in any
  Marley window, refuses the undo: the change would land under an edit no checkpoint saw.
- **D7: ask first, refuse plainly.** The question lists each path as restored or removed in a code
  block (PR-claude-a-repositorys-text-in-a-zed-prompt-goes-in-a-code-block-001) and says the
  agent's conversation is not changed. A refusal is a prompt with the reason and one button.
- **D8: the write uses what was shown.** The write merges nothing again: it writes the tree the
  question was computed from, and a file changed while the question was open refuses it (D3). The
  next Undo computes afresh.
- **D9: marks live with the rows.** `kept` and `undone` sit on the `Turn` in memory, as the rows
  do (#509 D10). An undone row offers neither action again. Rejected: a git note or a ref per mark,
  for rows that end with the terminal.
- **D10: a right-click menu.** As every rail row's actions are (`right_click_menu`). Rejected:
  buttons on the row; the rail's width holds the title and its marks.
- **D11: Codex through #753.** The undo reads only a row's commit and its parent, so any row in
  the turns store undoes the same way. This ticket builds after #753 and adds nothing for Codex.
  Promotion checks that #753's rows are `turns::Turn` rows; if they live elsewhere, the menu reads
  that store the same way.
- **D12: one undo at a time per repository**, chained as #509 chains its records
  (PR-claude-a-number-taken-across-awaits-is-taken-by-one-task-at-a-time-001).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a turn row is right-clicked, the rail shall show a menu with Keep and Undo This Turn…. | Shot `714-01-menu` |
| REQ-002 | WHEN Undo This Turn… is chosen and the turn's changes come out cleanly, the system shall ask first, naming each file it will restore or remove and saying the agent's conversation is not changed. | Shot `714-02-confirm` |
| REQ-003 | WHEN the undo is confirmed, the system shall put the turn's files back as they were before the turn and remove the files the turn created. | Shot `714-03-undone`; the run log |
| REQ-004 | WHEN the undo is confirmed, later turns' changes and the user's own edits, in the same files and elsewhere, shall stay. | The run log |
| REQ-005 | WHEN a turn is undone, the user's index, `HEAD` and the turn refs shall be as they were. | The run log |
| REQ-006 | WHEN a turn is undone, its row shall show `· undone`. | Shot `714-03-undone` |
| REQ-007 | WHEN an undone turn's row is right-clicked, the menu shall offer neither Keep nor Undo This Turn…. | Review |
| REQ-008 | WHEN taking a turn's changes out conflicts with a later change, the system shall refuse, name the conflicting files, and change no file. | Shot `714-04-conflict`; the run log |
| REQ-009 | WHEN Keep is chosen, the turn's row shall show `· kept`. | Shot `714-05-kept` |
| REQ-010 | WHEN a file the undo would change has unsaved edits in Marley, the system shall refuse, naming the file, and change no file. | Shot `714-06-unsaved` |
| REQ-011 | WHILE another agent works in the turn's checkout, a folder inside it or a folder around it, the system shall refuse Undo This Turn, naming the agent and its folder, and change no file. | Shot `714-07-another-agent`; the run log |
| REQ-012 | WHILE the turn's own agent works, the system shall refuse Undo This Turn and change no file. | Review (the same check, the own terminal not excluded) |
| REQ-013 | WHEN a file the undo would change is changed while the question is open, the system shall refuse at the write and change no file. | Shot `714-08-changed-meanwhile`; the run log |
| REQ-014 | WHEN the files already match the turn's start, the system shall say there is nothing to undo and change no file. | Review |
| REQ-015 | WHEN Marley's git is older than 2.40, the system shall refuse, naming the version, and change no file. | Review |
| REQ-016 | The undo shall read only a row's commit and its parent, so a row #753 records for Codex undoes the same way. | Review |
| REQ-017 | Every git run of the undo shall go through `turn_git.rs` over `process.rs` as argv, and none shall name the user's index. | Review; gate:22 |
| REQ-018 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec; at promotion, #753's rows checked (D11), the seams re-verified, the
  design in the notes confirmed, the brain asked.
- **P2 Code** — `turn_git.rs` (the merge and the write), a new `turn_undo.rs` in
  `marley_workbench` (the checks, the question, the flow), `turns.rs` (the marks),
  `close_guard.rs` (`working_status` for the crate), `rail.rs` (the menu and the marks),
  `marley_rail.rs` (`TurnSnapshot`'s marks), the guide; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), the AD for D4,
  close the ticket, archive, commit.
