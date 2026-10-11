---
pipeline_id: 5d29c4b8-9ac3-4176-96d1-f3cb53345478
ticket: docs/planning/tickets/open/TICKET-753-codex-turn-checkpoints.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Codex turn checkpoints
type: feature
slice: prong 2; T3 Code survey item 5 ("Codex gets what Claude Code gets"), second half
references:
  - docs/t3code_architecture/README.md
  - docs/t3code_architecture/03-git-worktrees-and-prs.md
  - docs/planning/pipeline/completed/509-per-turn-diffs.spec.md
  - docs/planning/pipeline/completed/650-codex-app-server-state.spec.md
  - docs/planning/tickets/open/TICKET-714-undo-a-turn.md
  - docs/planning/pipeline/queued/752-codex-push-and-answers.spec.md
---

## Title
Per-turn checkpoints for Codex. A Codex on its own App Server (#650) opens a turn at its lead
thread's `turn/started` and closes it at `turn/completed`; Marley checkpoints the repository at
both, as #509 does for Claude Code, and a turn that changed files becomes a commit of its end on
its start under `refs/marley/turns/<thread>/<n>`, listed under the terminal's row. The Turns rows
and their diffs then cover Codex. Chad, 2026-10-11: "3.) yes".

## Scope
### In
- **Turns from the App Server.** For a terminal whose Codex runs on its own App Server, on the
  lead thread Marley follows:
  - a turn opens at `turn/started` (the turn's id kept);
  - it closes at that turn's `turn/completed` (`completed`, `interrupted` or `failed`), at the
    thread's `thread/closed`, when the server stops, when Codex leaves the terminal, when the
    terminal closes, and at a `turn/started` for another turn while one is open.
- **The same recorder as #509.** `turns.rs` takes the checkpoints, compares them, writes the
  commit, pins it and lists it, unchanged in what it does. The session that names the refs is the
  lead thread's id; turn numbers count on from that thread's refs, so a resumed thread adds turns.
- **The folder.** The innermost local repository holding the thread's `cwd`, else the terminal's
  working directory, as #509 finds Claude Code's.
- **The title.** The text of the turn's `userMessage` item, on one line; `a prompt` when none came
  before the close.
- **The row.** Under the terminal's row, "Turns (N)" and its turns, newest first, with the title,
  the number of files, and `failed` for a turn that ended `failed`. A click opens Zed's
  `CommitView` on the turn's commit, as #509's rows do.
- **#714's need, noted.** Undo This Turn (#714) reverse-applies a turn's commit; with this ticket
  a Codex turn has one. #714 stays undecided and is not built here.
- `script/e2e/753-codex-turn-checkpoints.sh`; the stand-in Codex (`codex-fixture.sh`) gains cues
  for a prompt's turn and for edits between its start and end.
- The guide's "Per-turn diffs" names Codex.

### Out (explicitly deferred)
- Undo This Turn and Keep (#714).
- Codex without its App Server: no turn boundaries reach Marley.
- Sub-agents' threads: Marley follows the lead alone (#650); their edits land in the lead's turn,
  since the tree is one.
- Remote repositories, as #509's D8.
- Codex's turns from before Marley joined the server (after a Marley restart, the resume slice).
- Marking a turn Marley started itself (#715's Continue at Reset): it is titled by its text.

## Reference (§20)
- **Upstream Zed:** `project`'s git checkpoints (`Repository::checkpoint`, a temporary index that
  leaves the user's index, branches and `HEAD` alone) and `git_ui`'s `CommitView`, kept as #509
  uses them.
- **T3 Code** (MIT, report 03 §2.1): checkpoint refs per working directory with an ordinal per run,
  the thread's start materialized before the first end (`CheckpointService.ts:25`, `:166`;
  `CheckpointCaptureService.ts:106-125`), for every provider alike. Marley keeps #509's shape: one
  commit per turn on its own start, so the commit view shows the turn alone.
- **Warp:** Interactive Code Review gathers an agent's edits into a diff for Claude Code and Codex
  (docs.warp.dev/agents/local-agents/interactive-code-review/), as #509 cites. No Warp code read.

### Prior art
- **Behavior maps.** T3 report 03 §2.1 (checkpoints as hidden refs; "Only Claude Code's turns are
  recorded; Codex in a terminal gets none") and §3 item 1 (Undo This Turn, #714's source); report
  01 §3 item 7; the README's item 5 ("turn checkpoints taken on the App Server's turn start and
  end") and its fact list ("per-turn diffs (#509) cover Claude Code only").
- **Published material.** Codex's App Server protocol, generated from the installed 0.162.0
  (`codex app-server generate-json-schema --experimental`): `TurnStartedNotification {threadId,
  turn}` with `Turn.id` (UUIDv7), `TurnCompletedNotification` with `Turn.status` (`completed`,
  `interrupted`, `failed`), the `userMessage` thread item (`content[]` of `UserInput`, `text` for
  typed text), `Thread.cwd` (required). The same notifications at 0.155.1
  (`app-server-protocol/src/protocol/common.rs:1935`). git's `commit-tree` and `update-ref`, as
  #509.
- **Code we already ship.**
  - `crates/marley_workbench/src/turns.rs:108-160` (`on_event`: Claude Code's hook events open and
    close), `:216-262` (`open`), `:293-327` (`close`), `:343-421` (`record`: compare, number,
    commit, pin, list), `:163-179` (`on_end`, `forget`), `:182-190` (`ref_safe`); `turn_git.rs`
    (the two spawns). Everything after the boundary is agent-neutral already.
  - `crates/marley_workbench/src/agent_events.rs:336-347` (`after_fold`, the only caller of
    `turns::on_event`), `:655-669` (`end`, which closes turns of any agent that left),
    `:688-689` (`forget`).
  - `crates/marley_workbench/src/codex_server.rs:1100-1196` (`heard`: `item/started` read for file
    changes at `:1121-1130`, the lead's notifications folded at `:1192`), `:856-866`
    (`Following::fold`), `:879-886` (`view_of`), `:72-84` (`OPTED_OUT`, which keeps `item/started`
    and `item/completed`).
  - `crates/marley_agent/src/codex_events.rs:222-227` (`turn/started` and `turn/completed`
    decoded, the turn's id not kept), `:105-121` (`Turn`, `TurnError`), `:50-68` (`Thread`,
    without `cwd`).
  - `crates/marley_workbench/src/rail.rs:5724-5760` (the "Turns (N)" line and its rows), `:7851`
    (a row's turns are `Turns::of(view)`, whatever the agent).
  - Zed: `Repository::checkpoint`, `compare_checkpoints`, `update_ref`, `diff_tree`
    (`crates/project/src/git_store.rs`), `CommitView::open` (`crates/git_ui/src/commit_view.rs`),
    as #509's prior art lists them.
  Nothing else owns the seam: Codex's App Server keeps no file checkpoint a client can read (the
  survey: "Codex restores no files"), and no crate in `Cargo.lock` reads Codex's protocol.

## UI proof
`script/e2e/753-codex-turn-checkpoints.sh`, under `compositor sway` (the disclosure and a turn are
clicked). A scratch repository committed with `src/lib.txt` and `src/util.txt`, a staged change
of the user's to `src/util.txt`, a git identity of the scenario's own through
`GIT_CONFIG_GLOBAL`; #650's stand-in `codex` with `marley.codex_app_server` on, and new cues:
`prompt <text>` (a turn: `turn/started`, then its `userMessage` item), `write <path> <text>` and
`sh <command>` (the server edits the thread's `cwd`, two seconds after the turn's start, as a
model's first reply would), and `interrupt` (the turn completes `interrupted`). Four turns: "Add a
README" writes `README.md` and runs `sed -i` on `src/lib.txt`, then `done`; "Explain the build"
changes nothing, then `done`; "Fix the typo in util" edits `src/util.txt`, then `interrupt`;
"Refactor the parser" edits `src/util.txt`, then `fail`. Shots:
- `753-01-turns`: the Codex row's "Turns (3)", opened: "Refactor the parser · 1 file · failed",
  "Fix the typo in util · 1 file", "Add a README · 2 files"; "Explain the build" absent.
- `753-02-first-turn`: "Add a README" in the commit view: `README.md` added and the `sed` change
  to `src/lib.txt`, nothing of the later turns or the user's staged change.
- `753-03-interrupted`: "Fix the typo in util" in the commit view: its one change.
The run log records `git for-each-ref refs/marley/turns/` (three refs under the thread's id), each
commit's parent (a checkpoint), and `git status --porcelain`, `git diff --cached` and `git
rev-parse HEAD` before and after, which match.

## Locked-In Decisions
- **D1:** One recorder for both agents. Codex adds an entry point to `turns.rs` beside Claude
  Code's; the checkpoints, the commit, the ref, the prune and the rows stay #509's
  (AD-claude-509-a-turn-is-a-commit-of-its-end-on-its-start-under-marleys-refs-001). Rejected: a
  second turn store for Codex
  (PR-claude-two-engines-answering-the-same-question-will-diverge-001).
- **D2:** The boundaries are the App Server's own: the lead's `turn/started` and that turn's
  `turn/completed`, matched by the turn's id. A steer stays inside its turn. Rejected: the seat's
  state changes (an approval's wait and its end are not a turn's).
- **D3:** The refs are `refs/marley/turns/<thread id>/<n>`: a Codex thread id is a UUID, ref-safe
  as Claude Code's session id is, and a resumed thread counts on.
- **D4:** The title is the turn's `userMessage` text. Rejected: the thread's `preview` (it is the
  thread's first prompt, not this turn's).
- **D5:** Rejected: rebuilding a turn from Codex's `fileChange` items (a shell command's edits
  would be missed, AD-509's first rejection).
- **D6:** No switch of its own: `marley.codex_app_server`, off by default, and #648's tested range
  govern it, as every App Server feature. Nothing leaves the box.
- **D7:** Local repositories only, as #509's D8.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Codex turn on its App Server ends having changed files, the terminal's rail row shall list the turn with its prompt and the number of files it changed. | Shot `753-01-turns` |
| REQ-002 | WHEN a listed Codex turn is clicked, Marley shall open a diff of that turn's changes alone, edits made through the shell included. | Shot `753-02-first-turn` |
| REQ-003 | WHEN a Codex turn ends `interrupted`, the turn shall close there with its changes. | Shot `753-03-interrupted` |
| REQ-004 | WHEN a Codex turn ends `failed`, its row shall be marked failed. | Shot `753-01-turns` |
| REQ-005 | WHEN a Codex turn changes nothing, Marley shall list no turn for it and pin nothing. | Shot `753-01-turns`; the run log's refs |
| REQ-006 | WHEN a Codex turn is listed, its commit shall be pinned under `refs/marley/turns/<thread id>/<n>` with the turn's start as its parent, and the user's index and `HEAD` shall be as they were. | The run log |
| REQ-007 | WHEN the thread closes, the server stops, Codex leaves the terminal or the terminal closes while a Codex turn is open, Marley shall close the turn. | Review of the diff |
| REQ-008 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes: re-read #650's range against
  the installed Codex, `turns.rs` and the follow as they stand after #752.
- **P2 Code** — `codex_events` (the turn's id, the `userMessage` text, `Thread.cwd`);
  `codex_server` (the turn boundaries and the title to `turns.rs`); `turns.rs` (the Codex entry
  point); the stand-in's cues; the guide; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot and the run log.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
