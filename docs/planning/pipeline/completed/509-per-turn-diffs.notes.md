# Per-turn diffs for Claude Code in a terminal — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-509-per-turn-diffs.md
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
- **Discovery:** each seam opened and checked on 2026-09-25, at commit `484a7f18cb`.
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
- **Decisions:** D1 to D10 in the spec.

### Promotion (2026-09-28)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #570's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore, over `8d136c2bc0`) ✓; the prior-art sweep ✓; spec and design updated ✓.
  No ledger rows: no Zed path changes (`git` and `git_ui` join the workbench's dependencies).
- **Recall, added:** #519, #547, #566 and #569 have shipped: the fold's turn ends, `after_fold`
  and its arms, `agent_events::end` (#547) and `forget`; L-claude-568-a-seat-holds-one-wait-so-each-entry-needs-its-own-terminal-001
  and #570's stand-in with `auto` steps and `sleep` events, which this scenario extends with
  edits; #541 (queued), which moves the workbench's spawns into `process.rs` and names #509's two
  git calls. The brain (consultation 15227281a7a6482bae8eedee45dd8810): nothing on this seam.
- **Measured:** the scans a checkpoint makes of the marley_ide tree take tens of milliseconds
  (`git status --porcelain` 21 ms, `ls-files --others --exclude-standard` 15 ms); the index copy
  and `write-tree` add little, so a checkpoint runs well inside a turn.
- **What the code says now** (the Explore report, 2026-09-28): every Zed seam the draft cites
  holds; the Marley side moved:
  - The fold ends a turn at Stop, StopFailure, a manual PostCompact, an interrupt (a lead
    PostToolUseFailure with `is_interrupt`), a SessionStart, SessionEnd and a new session id,
    and a lead UserPromptSubmit opens one unless it is the compaction continuation (which the
    fold drops, so it never reaches `after_fold`). `after_fold` runs for lead events only and has
    no arm for StopFailure, PostCompact or SessionEnd yet. Claude Code leaving the terminal is
    `agent_events::end`, from the rail's refresh; a closed terminal is `forget`.
  - `is_harness_injected` and `is_compact_continuation` are public; no function says which tag
    injected a prompt, and the tag list holds a slash command's envelope tags.
  - The prompt reaches Marley collapsed and cut to 300 characters; the seat's `cwd` label follows
    the latest lead event, and an event's `cwd` can be dropped under the plugin's bound.
  - `Repository` has `checkpoint`, `compare_checkpoints`, `diff_tree(Since)`, `update_ref` and
    `delete_ref`, each a job on the repository's serial queue, and no `commit_tree` or ref
    listing; `GitRepositoryCheckpoint { commit_sha: Oid }`.
  - `branch_for` (`agent_bar.rs:137-148`) is private and returns the branch.
  - `CommitView::open(sha, repo, workspace, None, None, window, cx)` diffs against the first
    parent, opens in the active pane, and dedupes by the full sha.
  - The rail's rows are flat; no row has children, and its only disclosure is a project's,
    whose state Zed keeps.

### Design
- **Changed at promotion** (each item overrides the drafted design after it):
  - **The boundaries** come from the fold's result, not the event alone: `turns::on_event(view,
    event, before, seat, cx)` from `after_fold` (lead events), with `claude_events::prompt_origin`
    (`User`, `SlashCommand(name)`, `Injected(tag)`, `Continuation`) for the title and the mark.
    Opens: a UserPromptSubmit that is not the continuation (a turn open closes first). Closes:
    Stop, StopFailure (failed), a manual PostCompact, an interrupt, SessionStart, SessionEnd, a
    new session id in the terminal. `turns::on_end(view)` from `agent_events::end` and `forget`
    closes what is open.
  - **The store**: a `Turns` global keyed by the terminal view's id: the session, the repository
    (weak) and its work directory, the open turn (title, mark, and its checkpoint as a shared
    task), and the listed turns (`Turn { title, files, failed, injected, sha }`). The rail
    observes it.
  - **The close** runs in a task: the open's checkpoint awaited, a new checkpoint, then
    `compare_checkpoints`; equal trees end there. Otherwise `turn_git::commit_tree_in(work_dir,
    "<close>^{tree}", open, message)`, the next `<n>` from `turn_git::turn_refs_in` (the highest
    number the session's refs hold, plus one), `update_ref`, the file count from `diff_tree`,
    and, for the session's first pinned turn, the prune from the same listing's dates and
    `delete_ref`. A close by a new prompt shares its checkpoint as the next open's.
  - **The spawns** (`turn_git.rs`, the adapter): `git commit-tree … -F -` with the author and
    committer set to Marley and the message on stdin, and `git for-each-ref --format='%(refname)
    %(objectname) %(committerdate:unix)' refs/marley/turns/`, through `util::command` in the work
    directory; #541's `process.rs` takes them over when it lands.
  - **The repository**: `turns::repository_for(folder, repositories)`, `branch_for`'s rule
    returning the repository, fed the event's `cwd`, else the seat's `cwd` label, else the
    terminal's working directory; a remote project's repositories are passed over.
  - **The rail**: `TerminalSnapshot.turns` and `turns_open` (copied to `TerminalRow`), the open
    set kept by the rail per terminal and not saved; `render_terminal_row` draws the "Turns (N)"
    line with a `Disclosure` under the card, and the turn rows under it while open, newest first:
    the title, `· N files`, `· failed`, `· injected`. A click on a turn opens
    `CommitView::open` with its full sha in the terminal's workspace. The keyboard walk does not
    step into turns (Out).
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
- **File manifest** (as promoted). Marley only: `crates/marley_agent/src/claude_events.rs`
  (`prompt_origin`); `crates/marley_workbench/src/turns.rs` (new: the store, the boundaries, the
  close, the repository rule); `crates/marley_workbench/src/turn_git.rs` (new: the two spawns);
  `crates/marley_workbench/src/agent_events.rs` (the calls from `after_fold`, `end` and
  `forget`); `crates/marley_workbench/src/marley_workbench.rs` (the modules and the global's
  init); `crates/marley_workbench/Cargo.toml` (`git`, `git_ui`); `crates/marley_rail/src/marley_rail.rs`
  (`TurnSnapshot`, the fields); `crates/marley_workbench/src/rail.rs` (the turns under a row, the
  click, the observer); `script/e2e/509-per-turn-diffs.sh` and `script/e2e/golden` (Test).
- **Ledger rows.** None: no Zed path changes. `Cargo.lock` regenerates.

### E2E plan
As promoted, over the draft below: the fixture repository is committed with the scenario's own
identity (`GIT_CONFIG_GLOBAL`, which Marley's git inherits too) and holds a planted turn ref whose
commit is dated 40 days back; #570's stand-in `claude` gains pseudo-events that write a file or
run a program in its working directory between its hook events; the turns run in one terminal,
each by an Enter. One row joins the table:

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-009 | the planted ref, then the session's first pinned turn | the run log: the planted ref gone, the session's refs there |

The draft's plan:

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
- Changed at promotion: Marley's `git` is the one on Marley's PATH, where Zed's checkpoints use
  the project environment's or a bundled one; the commit and the refs are plain objects and refs
  either reads.
- The rail's rows are flat: the turns render under a terminal row's card, so the rows below move
  down while a terminal's turns are open; scenarios that click rows below a terminal with turns
  would move (none has turns today).
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
- `git` must be on the PATH Marley runs with; Zed's own git work may use another (above).

## Phase 2 — Code
- **Checklist** (no task tool): `claude_events::prompt_origin` ✓; `turn_git.rs` ✓; `turns.rs` ✓;
  the three calls in `agent_events.rs` ✓; the modules and `git`, `git_ui` ✓; `marley_rail`'s
  `TurnSnapshot` and the fields ✓; the rail's turns, toggle, click and observer ✓; fmt ✓; clippy ✓;
  the review ✓.
- **Built.**
  - `marley_agent::claude_events::prompt_origin(prompt) -> PromptOrigin` (`User`,
    `SlashCommand(name)`, `Injected(Option<tag>)`, `Continuation`): the continuation first, then
    the tag at the head (a slash command's envelope tags give the command's name from
    `<command-name>`), then the prefixes with no tag, else the user's.
  - `marley_workbench::turn_git` (the adapter): `commit_tree_in(work_dir, tree, parent, message)`
    runs `git commit-tree <tree> -p <parent> --no-gpg-sign` with the author and committer
    `Marley <marley@localhost>` and one `-m` per paragraph; `turn_refs_in(work_dir)` runs
    `git for-each-ref --format='%(refname) %(objectname) %(committerdate:unix)'
    refs/marley/turns/`. Both through `util::command` in the work directory, no shell.
  - `marley_workbench::turns`: the `Turns` global (seats by the terminal view's id, and the
    sessions that have pinned a turn); `Turn { title, files, failed, injected, sha, repository }`;
    `Turns::of(view)` and `Turns::repository_of(view, sha)` for the rail;
    `on_event` from `after_fold` (lead events), `on_end` from `agent_events::end`, `forget` from
    `agent_events::forget`. A UserPromptSubmit that is not the continuation closes the open turn
    and opens one in the innermost local repository holding the event's `cwd`, else the seat's
    `cwd` label, else the terminal's working directory, reusing the close's checkpoint when the
    repository is the same. Stop, SessionEnd, SessionStart, a manual PostCompact, an interrupt and
    a new session id close; StopFailure closes as failed. A close takes a checkpoint and spawns
    the record: both checkpoints awaited, `compare_checkpoints` (equal ends it), the next number
    from the session's refs, `commit-tree` of `<close>^{tree}` on the open's checkpoint with the
    title and the `Marley-Session` and `Marley-Turn` trailers, `update_ref`, the file count from
    `diff_tree(Since)`, the prune on the session's first pin (`delete_ref` for each ref whose
    commit is older than 30 days), and the row.
  - `marley_rail`: `TurnSnapshot { title, files, failed, injected, sha }`; `TerminalSnapshot` and
    `TerminalRow` gain `turns` (newest first) and `turns_open`, copied by `rail_rows` and
    `switcher_rows`; the tests' literals gain the fields.
  - `rail.rs`: the `turns_open` set (by terminal id, pruned to the live terminals at each
    refresh, not saved); `terminal_snapshot` fills the turns from `Turns::of`; the rail observes
    the `Turns` global beside `AgentEvents`; `render_terminal_row` puts `render_turns` under the
    card: a "Turns (N)" line with a `Disclosure` (a click on the line or the disclosure toggles),
    and while open each turn, newest first, with its title (truncated, the whole in a tooltip),
    `· N files`, `· failed` (error color) and `· injected`; a click shows the terminal's project
    and opens `CommitView::open` with the turn's full sha and the repository the turn was taken
    in.
- **Deviations from the design, and why.**
  - The message goes as `-m` paragraphs, not on stdin (`-F -`): the text is at most the plugin's
    300 characters, git's option parser takes `-m`'s value even when it starts with a dash, and
    nothing passes through a shell, so a pipe and its writer add nothing.
  - No init of the global: `try_global` reads it and `default_global` makes it at the first
    write, as `AgentEvents` does.
  - `on_event(view, id, event, seat, cx)` takes no `before`: no boundary needs the state before
    the frame.
  - `TurnSnapshot` carries the full sha: the click reads it from the row it drew, so a turn
    recorded between the draw and the click cannot shift it onto another commit.
  - The turns draw inside the terminal row's element, under its card, not as `Row` variants
    (as promoted); the keyboard walk does not step into them (Out).
  - `forget` drops the seat and its rows; the refs keep the commits.
  - The rail's two global observers share `_agent_events: [Subscription; 2]`.
  - The design's mark order stands, `title · N files · failed · injected`; the spec's UI proof
    lists "task notification · injected · 1 file", and Test reads the design's order.
- **The review** (against REQ-001 to REQ-009, re-entrancy, provenance, errors):
  - Found: two turns of one terminal closing close together (a prompt's close, then a quick
    Stop) could both list the refs before either pinned, take the same `<n>`, and the second
    `update_ref` would overwrite the first's ref, leaving its commit to `git gc`; the rows could
    also list out of order. Fixed: each seat keeps its last record as a shared task, and the next
    close's record awaits it first; a detached waiter keeps a record running when the terminal
    closes (F-509 at Complete).
  - Found: the prune's flag lived on the seat and was read at the close, so two quick closes both
    pruned, and a new session in the terminal reset it for the old session's late record. Fixed:
    the global keeps the sessions that have pinned, checked and set at the pin.
  - Found: the session id names a ref, and an event without one, or one git would refuse, would
    fail every pin. Fixed: the event's id, else the seat's session label, and a turn opens only
    for an id of ASCII letters, digits, `-` and `_` (Claude Code's are UUIDs).
  - Found: `Turns::repository_of` gave the seat's current repository, so a turn taken before
    Claude Code moved to another repository would have opened in the wrong one. Fixed: each
    `Turn` keeps the repository it was taken in, and the click finds it by the turn's sha.
  - Checked, no change: every entity read or update in `on_event` runs inside the terminal
    view's update and touches only the workspace, the project, the git store and the repository;
    `forget` runs in the view's release; the rail's click runs in the rail's update and updates
    the `MultiWorkspace`, as `activate_terminal` does. No `unwrap`, no `let _ =` on a fallible
    call; errors log through `log_err` (a turn that cannot be recorded leaves no row, the
    terminal is unaffected). Provenance: the Zed calls are the public `Repository` jobs and
    `CommitView::open`; nothing is carried over from a Zed function body; nothing from Warp.
- **Gates.** `rustfmt` on every touched file. `just clippy marley_agent marley_rail
  marley_workbench` (`--all-targets`, `-D warnings`): the first run found five, each fixed at the
  source: `Rail::new` over 100 lines with the new observer (the filter's field and its subscription
  moved to `Rail::filter_field`), `turn_git.rs`'s first doc paragraph (now one line), the 30 days
  as `Duration::from_hours(30 * 24)`, the open-turn check as `seat.open.as_ref()?`, and
  `Closing.closing` renamed `end`. The second and third runs (after the repository fix) are clean.
  The logs are in the scratchpad.
- **Meanwhile**: #570's release install ran its golden set on the release build: 44 of 45 passed,
  and `481-rich-input` failed its check "the rich input's two lines and the typed one reached the
  agent": the stand-in's reply to the paste's first line landed on the echo of the second
  (`second lineclaude got: hello rich input`), before the Enter's echo. The rich input writes the
  paste and the carriage return in two writes (`rich_input.rs:132-133`, unchanged since #549), so
  a reply can come between them. It passed when run again alone against the same binary (34 s).
  Nothing was installed; the install goes with #509's, and Test makes 481's check look for the
  reply anywhere on a line.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan as promoted ✓; 509 in the golden set
  ✓; 481's check made to hold under the echo race ✓; `just build` ✓; the scenario run and every
  shot read ✓; the golden set ✓; `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/509-per-turn-diffs.sh` (compositor sway, no Chromium, no browser
  fixture): a scratch repository under a git config of the scenario's own (`GIT_CONFIG_GLOBAL`,
  `GIT_CONFIG_NOSYSTEM`), committed with `src/lib.txt` and `src/util.txt`, a change of the user's
  to `src/util.txt` staged, and two planted turn refs, `s-old/1` committed 40 days back and
  `s-recent/1` 5 days back. #566's stand-in `claude` gains three pseudo-events (`sleep`, `write`
  and `run`) and a `turns` case of five steps in one terminal, each by an Enter, each prompt
  followed by two seconds before its edits (L-509, the start's checkpoint first): "Add a README"
  writes `README.md` and runs `sed -i` on `src/lib.txt`, then Stop; "Fix the typo in util" runs
  `sed -i` on `src/util.txt` and sends no Stop; "Explain the build" changes nothing, then Stop; a
  `<task-notification>` prompt edits `src/lib.txt`, then Stop; "Refactor the parser" edits
  `src/util.txt`, then StopFailure `rate_limit`. The real `claude` never ran. Run 1 passed its 12
  checks, but its click on "Turns (4)" landed on the card above it (the line sits at y 228, under
  a three-line card), so its shots after the first showed the turns closed; run 2, with the points
  measured from run 1's shot, passed all 12 and showed every step. Run 3, after the gate's
  shellcheck asked for the turn loop's variable to be used (the loop now echoes each turn), passed
  all 12 again with the same shots.
- **The shots** (run 2, read one by one; the rail cropped and enlarged):
  - `509-00-row` (REQ-001): the Claude Code row, "failed · Refactor the parser" and `rate_limit`,
    and under its card "› Turns (4)", closed.
  - `509-01-turns` (REQ-001, REQ-004, REQ-005, REQ-006): the line opened, newest first: "Refactor
    the parser · 1 file · failed" (failed in the error color), "task notification · 1 file ·
    injected", "Fix the typo in util · 1 file", "Add a README · 2 files"; "Explain the build",
    which changed nothing, has no row.
  - `509-02-first-turn` (REQ-002): "Add a README" clicked: the commit view's tab "e0dc8d6 — Add a
    README", author Marley, `marley@localhost`, +4 −1: `README.md` added (three lines) and
    `src/lib.txt` `alpha` to `ALPHA`, the change a `sed -i` made through the shell, not a tool;
    nothing of the later turns, nothing of the user's staged change.
  - `509-03-closed-by-a-prompt` (REQ-003): "Fix the typo in util" clicked: "b963985 — Fix the typo
    in util", +1 −1, `src/util.txt` `utl` to `util` alone, "staged by the user" unchanged context
    on both sides, and nothing of the refactor that came two prompts later.
  - The run log (REQ-005, REQ-007, REQ-009): the refs after the session are `s-recent/1` and
    `s-turns/1` to `4`, four for five turns, "Explain the build" pinning nothing, and `s-old/1`
    gone; each turn is `Marley <marley@localhost>` with the turn's title, its
    parent a Zed checkpoint (message `Checkpoint`), and the trailers `Marley-Session: s-turns` and
    `Marley-Turn: N`; the files per turn are `README.md src/lib.txt`, `src/util.txt`,
    `src/lib.txt`, `src/util.txt`; `HEAD` and `git diff --cached` are byte for byte as before
    the session, and `git status --porcelain` shows the agent's edits unstaged (` M src/lib.txt`,
    `MM src/util.txt`, `?? README.md`); the stand-in read the five Enters and nothing else.
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **481's check** (the flake the release golden run hit, pre-existing and fixed here since it
  blocks installs): `stand_in_got` reads each `claude got: ` line from where it starts
  (`sed -n 's/.*claude got: /claude got: /p'`) and matches it whole, so a reply glued to the
  echo of the paste's second line counts, and a line that never reached the stand-in still fails.
  Checked by hand against the failed run's text before the golden set.
- **Left to review, not run**: the other turn ends (a manual `/compact`'s PostCompact, an
  interrupt, a new session in the terminal, Claude Code leaving it): each is one arm on the fold's
  events, which a stand-in could send, and this scenario keeps to the spec's five turns. **Not
  reachable by a scenario**: a remote project, which the e2e runner cannot open; and the
  checkpoint's exclusions, which are Zed's own rules.
- **The gate**: the first run was red on one gate, shellcheck SC2034 (the turn loop's variable unused); the loop now echoes each turn, the scenario ran again (run 3), and the second run printed `GATE GREEN [diff]`, 16 passed and 0 failed. Both logs are kept in the scratchpad.
- **The golden set** with 509 added: all 46 passed on the debug build (`just regress`), 481 with its new check in 33 s and 509 in 71 s; the golden run's 509 shots match run 2's. It ran before the loop's echo was added, which changes only the run log.
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md` (Added: per-turn diffs); `docs/marley/three-prong-plan.md` (C1's
  row: per-turn diffs shipped); `docs/marley/tutorial-outline.md` (the row marked shipped);
  `docs/marley/guide.md` ("Per-turn diffs" under Agent CLIs in terminals: the boundaries, the
  checkpoints, the refs, the rail's rows and their marks, the commit view, the prune, what a turn
  leaves out); `docs/marley_architecture/marley_workbench.md` ("Per-turn diffs": the global, the
  boundaries, the chained records, the adapter, the rail); `marley_agent.md` (`prompt_origin`);
  `marley_rail.md` (`TerminalSnapshot::turns`). No Zed path changed, so no touchpoint row; the
  guide's "What is planned" list waits for the documentation pass after the remaining tickets.
- **Knowledge appended.** `failures.md`: F-claude-509-two-quick-closes-could-pin-one-turn-number-twice-001,
  F-claude-509-a-turn-would-have-opened-in-the-repository-its-terminal-moved-to-001,
  F-claude-509-an-event-with-no-usable-session-id-would-have-failed-every-pin-001,
  F-claude-481-the-rich-inputs-check-raced-the-echo-of-its-paste-001. `prevention-rules.md`:
  PR-claude-a-number-taken-across-awaits-is-taken-by-one-task-at-a-time-001,
  PR-claude-a-check-on-terminal-text-reads-a-reply-from-where-it-starts-001. `lessons.md`:
  L-claude-509-a-zed-checkpoint-is-a-commit-on-head-of-the-whole-tree-001,
  L-claude-509-a-scenario-agent-edits-after-its-prompts-checkpoint-001. `architecture-decisions.md`:
  AD-claude-509-a-turn-is-a-commit-of-its-end-on-its-start-under-marleys-refs-001.
- **Brain.** Consultation `15227281a7a6482bae8eedee45dd8810` closed with `brain decide`
  (`decisions/marley-keeps-each-claude-code-turn-as-a-commit-of-its-end-on-its-start-under-refsmarleyturns`),
  follow-up by 2026-10-28: whether the refs pile up between prunes.
- **Closed.** TICKET-509 moved to `tickets/closed/`, its pipeline link at `completed/`; the
  backlog row went at promotion.
- **The receipt.** `GATE GREEN [diff]` on the tree after run 3; only docs changed since.
