# Merge upstream Zed through 2026-10-29 — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-950-merge-upstream-zed.md
- **Pipeline spec:** 950-merge-upstream-zed.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-29: "one last thing i forgot. lets pull down latest zed release
  changes so we can stay up to date."
- **Classification:** chore, upstream sync, after #695 and #797.
- **Trial merge (2026-10-29):** `git fetch upstream --tags` gives `0373366f43` (2026-10-29 16:25
  UTC). The merge base is `56934ec056` (#797's), 190 commits and 527 files (52,335 lines in,
  20,676 out). New tags: v1.25.2, v1.25.3, v1.26.0-pre to v1.26.2, v1.27.0-pre, on release
  branches. `git merge-tree --write-tree HEAD upstream/main` conflicts in ten files; 59 files
  changed on both sides (list in `scratchpad/both-sides.txt` of this session).
- **The conflicts, one hunk each unless noted:**
  - `Cargo.lock`: PR-claude-695, Marley's kept.
  - `agent/src/tools/terminal_tool.rs`: #797's nightly fix (`as_os_str().is_empty()`) against
    upstream's `PathStyle` rework (`a4a3a968d1`, `9521840152`). Take upstream's; keep the
    one-line fix only if `subpath` is still a `&Path` the lint nightly cannot call `is_empty` on,
    else drop the hunk and its row (rule 4).
  - `agent_servers/src/acp/v1.rs` (2): upstream sends load and resume through `ordered_response`
    (ACP 3.6.0, `5e8998a2a7`). Keep `ordered_response`, with `.meta(meta)` on the request it
    takes (#683).
  - `agent_ui/src/config_options.rs`: upstream's `is_read_only` beside Marley's `value_name_of`,
    `value_ids_of`, `value_of` and `set_value_of` (#907, #915): keep both.
  - `agent_ui/src/conversation_view/thread_view.rs`: upstream removed the Restore Checkpoint
    block (`eee7501970`); keep Marley's `.when(marley_bubble, …)` (#825), drop the block.
  - `agent_ui/src/thread_metadata_store.rs`: `remote_connection_options` now returns a
    reference (`.cloned()`); keep Marley's own-folders match (#734) with upstream's `.cloned()`.
  - `gpui_linux/src/linux/wayland/window.rs`: upstream's `activate` now passes a recent press's
    serial (`4d847859f5`) and dropped the old comment. Keep Marley's handed-over-token branch
    first (#545), then upstream's request.
  - `markdown/src/markdown.rs`: upstream waits for a pending render before taking the autoscroll
    request (`f41ac64812`, lazy parse). Keep upstream's guard and take the request with Marley's
    `marley_top_index` check inside it (#656).
  - `terminal/src/terminal.rs`: `secondary()` takes `cx` (`389c22e1c7`). Keep Marley's web link
    through `Event::Open` (#503), upstream's `secondary(cx)`.
  - `workspace/src/dock.rs`: `dispatch_context(cx)` and `_window`; take upstream's signature, then
    check whether Marley's dock hunks still use `window` in `render`.
- **Recall (§18.3):**
  - L-claude-695-merging-upstream-zed-001 and L-claude-797-merging-upstream-zed-001: a merge
    commit on a branch; `upstream_base` and gates 12 and 13 read `MERGE_HEAD`; count `Marley:`
    markers per file before and after; expect API breaks (`cargo check -p zed` finds them); the
    golden set compared like with like.
  - PR-claude-695-an-upstream-merge-keeps-marleys-lockfile-001: `git checkout HEAD -- Cargo.lock`,
    then build, then `cargo audit`.
  - PR-claude-797-a-lint-nightly-gap-is-fixed-at-its-call-001 and
    PR-claude-797-a-gate-check-with-a-time-limit-allows-a-loaded-box-001: in force.
  - The brain (`rusty-cli brain ask`, consultation `423ecb53ac9c4360b9e3b3589ec753d9`): nothing on
    upstream merges; only decisions due for follow-up.
- **Review (Chad, 2026-10-29):** "lets fix the conflicts in terms of assuming zed is right and
  working around it". Locked as D5: each conflict starts from upstream's side; a Marley hunk
  comes back only as an addition beside upstream's lines, never as an edit of them. For
  `terminal_tool.rs` that means upstream's `subpath.is_empty()` stays as upstream wrote it; if
  the lint nightly cannot build it, the workaround goes in the gate's lint step, not in the
  call (PR-claude-797-a-lint-nightly-gap-is-fixed-at-its-call-001 yields to D5 here, and is
  revisited at Complete).
- **Checklist (no TaskCreate in this harness):** pick ✓, pre-flight ✓ (cargo idle, no active
  pipeline, README marker present, #946's queued pair left queued), recall ✓, mint ✓ (#950;
  no BACKLOG row, picked by name), prior art ✓, spec ✓, design ✓.

### Prior art
- The checkpoint seam: `eee7501970` removed `GitRepository::checkpoint`, `restore_checkpoint`,
  `compare_checkpoints`, `diff_checkpoints`, `GitRepositoryCheckpoint`, their `Repository` jobs
  and proto messages. It kept `create_archive_checkpoint` (`git_store.rs:9491` upstream), a job
  in the repository's queue that returns `(staged_sha, unstaged_sha)`: a commit of the index on
  `HEAD`, then a commit of `git add --all` through a temporary index on that. The second commit's
  tree is the working state a checkpoint held, so it serves Marley's turns. Recorded in the spec.

### Design
- **Branch:** `marley/upstream-2026-10-29` from `marley/workbench-shell` at `31266f5797`; `git
  merge --no-ff --no-commit upstream/main` at `0373366f43`; at Complete, `marley/workbench-shell`
  fast-forwards to the merge commit.
- **Before resolving:** count `Marley:` per Zed file on `HEAD`, for the check after.
- **Conflicts:** as listed above, each against its touchpoints row.
- **Turn checkpoints (Marley crate only, `crates/marley_workbench`):**
  - `turns.rs`: `Checkpoint` becomes `Shared<Task<Option<TurnCheckpoint>>>`, where
    `TurnCheckpoint` is a newtype over the full-state commit's sha (a `String`; the two shas are
    not interchangeable, so the staged one is dropped at the call). `take_checkpoint` calls
    `repository.create_archive_checkpoint()` and keeps `unstaged_sha`. `record` compares the
    two through `turn_git::same_tree_in(work_dir, start, end)` in place of
    `compare_checkpoints`.
  - `turn_git.rs`: `same_tree_in`, `git diff-tree --quiet <a> <b>`: exit 0 the same, exit 1
    different, anything else an error, as the removed `compare_checkpoints` read it.
  - `turn_undo.rs`: the receiver's type and the sha read from `unstaged_sha`.
  - `codex_server.rs` (#753) and any other caller `cargo check` finds: the same type.
- **After the merge:** `rg "Marley:" crates assets` against every touchpoints row and the counts
  taken before; each of the 59 both-sides files read for a Marley hunk a clean merge displaced;
  `cargo check -p zed`, then the API breaks fixed in the Marley crates; the "At an upstream
  merge" steps (the base in `lib-hook-helpers.sh` and the touchpoints doc, `.cargo/audit.toml`,
  `deny.toml` — upstream's wgpu is back on crates.io, so `gfx-rs` in the allowed organizations may
  go — the workbench traps D1/D7, `test_action_namespaces`, `register_sidebar(` callers, the
  `workspace::Sidebar` trait); `script/gates.sh --diff`.
- **File manifest:** the ten conflicted Zed files (resolution only); `Cargo.lock`;
  `crates/marley_workbench/src/{turns,turn_git,turn_undo}.rs` and whatever else the build
  names in `crates/marley_*`; `docs/marley/zed-touchpoints.md`;
  `.claude/hooks/lib-hook-helpers.sh`; maybe `deny.toml`; `CHANGELOG.md` at Complete.

### Visual check plan
- REQ-001 and REQ-002: the review and the gate, not a shot.
- REQ-003: `509-per-turn-diffs.sh` (sway) on a release build of the merged tree: the turns row
  and its four turns with their files.
- REQ-004: `714-undo-a-turn.sh` (sway): the undone, conflict, kept, unsaved and changed-meanwhile
  shots.
- REQ-005: `753-codex-turn-checkpoints.sh`.
- Each of the three also runs on the installed pre-merge build (`31266f5797`); the scenarios
  have drifted since they were written (#797 found 29 of 53 golden failures), so a failure counts
  only when it passes before and fails after, at the same check.
- REQ-006: `just shot 950-merged`.
- Not reached by a scenario: the conflict hunks' own behaviors (#503 web links, #545 activation,
  #656 autoscroll, #825 bubbles, #907/#915 config options); they are checked by the review
  against their rows, as in #797.

### Risks
- **The archive checkpoint is wider than the old one.** The removed `checkpoint` left out
  untracked files over 2 MB and the patterns of `checkpoint.gitignore`; `git add --all` takes
  every untracked file `.gitignore` allows. A repository with large untracked files writes larger
  objects per turn. Accepted: the turn's tree is truer, and an ignored build folder stays out.
  Reversible: `turn_git.rs` can write its own checkpoint later if it shows up.
- **No checkpoint without a commit.** `create_archive_checkpoint` reads `HEAD` and fails in a
  repository with no commits, where the old one worked; turns are then not recorded there (a
  logged error, as today when git fails).
- **ACP 3.6.0 and the subagent work** may change types `marley_workbench` reads from
  `acp_thread`. Fixed at the source.
- **Clean merges can drop a Marley hunk** (`default.json`, `editor.rs`, `workspace.rs`,
  `terminal_view.rs`, `zed.rs` among the 59); each is read.

## Phase 2 — Code
- **Merge:** branch `marley/upstream-2026-10-29` from `31266f5797`; `git merge --no-ff --no-commit
  upstream/main` (`0373366f43`), conflicts in the ten files the trial named. `Marley:` markers
  counted per Zed file before (109 files).
- **Conflicts resolved under D5** (upstream's side first, Marley's lines added beside it):
  - `Cargo.lock`: `git checkout HEAD -- Cargo.lock` (PR-claude-695); cargo added upstream's.
  - `agent/src/tools/terminal_tool.rs`: upstream's file whole. #797's one-line hunk and its
    touchpoints row went (rule 4); the lint nightly's gap moved to gate:21 (below).
  - `agent_servers/src/acp/v1.rs`: upstream's `ordered_response` for load and resume, with
    `.meta(meta)` on the request it takes (#683). Seven markers before and after.
  - `agent_ui/src/config_options.rs`: upstream's `is_read_only`, then Marley's four accessors
    (#907, #915). `set_value_of` now does nothing on a read-only session, as upstream's own
    setters refuse a subagent's options; its row says so.
  - `agent_ui/src/conversation_view/thread_view.rs`: upstream's removal of the Restore Checkpoint
    block; Marley's `.when(marley_bubble, …)` line kept (#825).
  - `agent_ui/src/thread_metadata_store.rs`: upstream's two lines unchanged (`.cloned()` on the
    remote connection); Marley's own-folders choice added after them as a shadowing `let` (#734).
  - `gpui_linux/src/linux/wayland/window.rs`: upstream's new `activate` unchanged; Marley's
    handed-over token branch added before it (#545). Upstream's old comment, which Marley's side
    carried, dropped.
  - `markdown/src/markdown.rs`: upstream's pending-render guard unchanged; Marley's top-aligned
    check added after it as its own `update` that takes `marley_top_index` only when it names the
    request taken (#656).
  - `terminal/src/terminal.rs`: Marley's web-link branch (#503) kept with upstream's
    `secondary(cx)`. **Deviation from D5:** the branch cannot be a pure addition, since it decides
    between upstream's `cx.open_url` and Marley's `Event::Open`, and an early return would skip
    the resets at the function's end. Upstream's `open_url` line stays verbatim, in the `else`.
  - `workspace/src/dock.rs`: upstream's `dispatch_context(cx)`; the `window` parameter keeps its
    name (upstream made it `_window`) because Marley's header hook (#838) reads it.
- **Hunks kept:** marker counts match per file except `terminal_tool.rs` (retired on purpose). Of
  the lines Marley added to the 59 files both sides changed, every one is still in the merged
  file except those replaced above (`terminal_tool.rs`, `markdown.rs`'s old take,
  `thread_metadata_store.rs`'s old `None` arm, `config_options.rs`'s doc line).
- **Merge checklist:** the upstream base moved to `0373366f43` in the touchpoints doc and
  `UPSTREAM_BASE_FALLBACK`. No new `register_sidebar(` caller (the same five as at
  `56934ec056`); the `workspace::Sidebar` trait untouched; the keymaps only gain debugger hover
  and vim bindings, so D1 and D7 hold. `.cargo/audit.toml` needed nothing (gate:7 green).
- **Turn checkpoints (D4):** `turns.rs` takes `Repository::create_archive_checkpoint` and keeps
  its full-state commit as `CheckpointSha`; `checkpoint_of` is shared with `turn_undo.rs`;
  `turn_git::same_tree_in` (`git diff-tree --quiet`) replaces `compare_checkpoints`.
- **Upstream API changes fixed (`cargo check -p zed`, four runs):**
  - `acp_thread::Diff` resolves a finalized diff on first use (`f41ac64812`), `buffer()` is
    private and `multibuffer()` takes `&mut Context`. **Deviation:** a new Zed hunk,
    `Diff::marley_buffer_diff` in `acp_thread/src/diff.rs` with its row, an accessor that reads a
    pending or resolved diff and returns `None` for a deferred one, never resolving it. The edit
    card's "+N −M" (#914) uses it, so a card of a thread reloaded from history shows its counts
    once the diff resolves. `plan_review.rs` resolves its own diff with `update`.
  - `Keystroke::parse` takes only `&'static str`: `parse_with_os` in `marley_browser`'s `chord`
    (`OperatingSystem::Unknown`; its source never names `secondary-`), and with the host's OS in
    `settings_change.rs` and `terminal_drive.rs` (which also keeps the OS on `Typing` for
    `unparse`). `KeyContext::new_with_defaults` and `KeymapFile::update_keybinding` take the OS.
  - `ProjectGroupKey::host()` returns a reference: three `rail.rs` calls lose `.as_ref()`,
    `find_or_create_workspace` takes `.cloned()`.
  - `ScrollWheelEvent` → `ScrollEvent`, `on_scroll_wheel` → `on_scroll` (deprecated aliases).
- **Review of the diff:** the `turns.rs` module doc had lines over 100 columns; reflowed. Nothing
  else: no panic on an input path, the checkpoints still run in the repository's job queue, each
  Zed hunk is additive with its row, nothing from Warp, no Zed function body carried into a Marley
  crate.
- **Gate run 1 (`scratchpad/950-gate-1.log`):** red on gates 2, 8, 13, 21 and the receipt.
  - gate:2: a redundant closure in `plan_review.rs` (`Diff::multibuffer` passed by path).
  - gate:8: upstream's new git source, `rust-minidump/minidump-writer` (`087340e2d7`, #66158),
    joins `deny.toml`'s allowed organizations with the commit cited.
  - gate:13: a false positive in the gate. During a merge it dropped upstream's lines by matching
    their text against every line Marley adds; Marley's test crate has `#[ctor::ctor(unsafe)]`
    with a SAFETY note, so upstream's new unjustified one in `multi_buffer_tests.rs` was taken
    for Marley's. `added_unjustified_unsafe` now matches against Marley's unjustified lines only
    (`unjustified_unsafe_since`).
  - gate:21: the lint nightly cannot build `Path::is_empty`, now in upstream's new `path` crate and
    in upstream's `terminal_tool.rs`. Under D5 the call stays as upstream wrote it: the gate
    switches `path_is_empty` on for those two packages alone through nightly cargo's
    `-Zprofile-rustflags` and per-package `--config` (`DYLINT_CRATE_FEATURES`), never for every
    crate (#797's E0635). A probe confirmed cargo accepts it before the run.
  - receipt: the doc reflow landed mid-run.
- **Gate run 2 (`950-gate-2.log`):** **GATE GREEN [diff]**, 17 passed.
- **Checklist:** merge ✓, ten conflicts ✓, turn checkpoints ✓, API breaks ✓, ledger and base ✓,
  review ✓, gate ✓.

## Phase 3 — Test
- **Build:** `cargo build --release -p zed --bin marley` (12m 33s), copied to
  `scratchpad/950-build/marley` so a later build cannot replace it mid-run.
- **Baseline:** 509, 714 and 753 on the installed pre-merge build (`94bb105044`; the one commit
  after it, `31266f5797`, is docs), `SHOT_DIR=scratchpad/base/<name>`: every check passes.
- **Scenario fix:** 509 and 753 checked that a turn's parent commit is titled `Checkpoint`, the
  subject of Zed's removed agent checkpoint. Zed's archive commit (D4) is titled `WIP unstaged`;
  the five patterns now expect that, with a comment naming #950. What the checks mean (the turn
  sits on a checkpoint, not on the user's `HEAD`) is unchanged.
- **Merged build** (`E2E_BINARY=scratchpad/950-build/marley`, `SHOT_DIR=scratchpad/merged/<name>`):
  - REQ-003, `509-per-turn-diffs.sh`: all 12 checks pass. Four turns pinned under the session, the
    40-day ref pruned and the 5-day ref kept, each turn on a checkpoint, turn 1 holding
    `README.md src/lib.txt`, turn 2 `src/util.txt` alone, turn 3 the task notification's, turn 4
    the failed refactor's, the trailers, `HEAD` and the index as the user left them. The shots
    show "Turns (4)" under the Claude Code row, but the scenario's click on that line misses it:
    the line is drawn at y 393, the click goes to `TURNS_Y=228`, with the "Needs you" card above
    Default among what moved it. The pre-merge build's shots miss in the same place —
    pre-existing, not in scope; the scenario's click points predate the rail as drawn now. The turn rows with their files are seen open in 714's shots on the same build.
  - REQ-004, `714-undo-a-turn.sh`: all checks pass. `714-02-confirm`: "Turns (4)" open with
    Shout gamma · 1 file, Fix the typo in util · 1 file, Shout iota · 1 file, Add a README · 2 files,
    and the question "Undo the turn "Add a README"?" naming remove `README.md`, restore
    `src/lib.txt`. `714-03-undone`: Add a README · 2 files · undone, the status bar from 4 changed to
    3. `714-08-changed-meanwhile`: the refusal "A file changed after the question opened, so
    nothing was written", Fix the typo in util marked kept.
  - REQ-005, `753-codex-turn-checkpoints.sh`: all six checks pass. `753-01-turns`: the Codex
    thread's "Turns (3)": Refactor the parser · 1 file · failed, Fix the typo in util · 1 file,
    Add a README · 2 files.
  - REQ-006, `just shot 950-merged` (`COMPOSITOR=sway`, `scratchpad/shots-950/950-merged.png`):
    Marley starts in the Marley layout, the rail with Default and the repo's group and its bash
    row, the terminal in the center at the user's prompt, and Zed's trust prompt over the new
    project, as after #797.
- **Focus:** every run reported one Marley window on Hyprland before and after, no rule added;
  sway stopped with the run's Marley. Every shot shows Marley alone.
- **Not reached by a scenario:** the conflicted hunks' own behaviors (#503, #545, #656, #825,
  #907/#915, #914's counts), checked by the review against their rows in Phase 2.
- **Gate:** the two scenario files changed after gate run 2; the gate runs again at Complete
  before the commit.
- **Checklist:** REQ-001 and REQ-002 (Phase 2), REQ-003 to REQ-006 above.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`, "Upstream Zed through 2026-10-29" (two claims checked against
  upstream and corrected before they stayed: auto fetch is off by default; README on a new
  window is a setting). `docs/marley/guide.md`'s per-turn diffs: the checkpoint is Zed's archive
  commit, holds untracked files of any size, needs a first commit.
  `docs/marley_architecture/marley_workbench.md`: `checkpoint_of` and `same_tree_in` in place of
  the removed API. The touchpoints doc: the base moved to `0373366f43`, the `terminal_tool.rs` row
  gone, a new `acp_thread/src/diff.rs` row, the `config_options.rs` row naming the read-only
  case; the other conflicted rows still describe what ships (gate:16 green). No plan slice moves:
  an upstream sync belongs to no slice.
- **Knowledge appended:**
  - F-claude-950-the-merge-filter-took-upstreams-unsafe-line-for-marleys-001
  - PR-claude-950-a-text-match-filter-matches-the-same-kind-of-line-001
  - PR-claude-950-a-lint-nightly-gap-in-upstream-code-is-closed-in-the-gate-001 (refines
    PR-claude-797-a-lint-nightly-gap-is-fixed-at-its-call-001 for upstream's own calls)
  - L-claude-950-merging-upstream-zed-001
  - AD-claude-950-upstream-wins-a-merge-conflict-001
  - AD-claude-950-turn-checkpoints-are-zeds-archive-commit-001
- **Brain:** `brain decide` on consultation `423ecb53ac9c4360b9e3b3589ec753d9`:
  `decisions/marleys-upstream-merges-keep-zeds-side-of-every-conflict`, follow-up by 2026-11-29.
- **Gate:** run 3 before the commit, for the two scenario files changed in Test.
