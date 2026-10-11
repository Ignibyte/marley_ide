# Codex turn checkpoints — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-753-codex-turn-checkpoints.md
- **Pipeline spec:** 753-codex-turn-checkpoints.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, answering the T3 Code survey's item 5 ("Codex gets what Claude
  Code gets", `docs/t3code_architecture/README.md`): "3.) yes". This ticket is that item's second
  half, after #752: "turn checkpoints taken on the App Server's turn start and end". Queued after
  the tabs batch, #743 to #749, and after #752.
- **Classification:** feature, prong 2. Marley crates only (`marley_agent`, `marley_workbench`); no
  Zed path, no setting. Size S to M: the recorder exists, and the work is the boundaries, the
  title and the scenario's stand-in cues.
- **#714.** Undo This Turn (`docs/planning/tickets/open/TICKET-714-undo-a-turn.md`, Deliberate,
  "for future use") reverse-applies a turn's commit. For Claude Code the commits exist (#509); for
  Codex they exist once this ticket ships. #714 is not specced here, and nothing here depends on
  its answer.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.

### Recall (§18.3)
- **Ledger.**
  - AD-claude-509-a-turn-is-a-commit-of-its-end-on-its-start-under-marleys-refs-001: the shape kept
    whole; its rejections (tool calls, a text diff, Marley's own store, `git2`) hold for Codex.
  - L-claude-509-a-zed-checkpoint-is-a-commit-on-head-of-the-whole-tree-001: the checkpoint takes
    the whole tree, so two agents in one checkout share each other's edits in their turns; #510's
    worktrees remove the overlap.
  - L-claude-509-a-scenario-agent-edits-after-its-prompts-checkpoint-001: the stand-in edits two
    seconds after `turn/started`, as Claude Code's stand-in waits after its prompt's frame.
  - F-claude-509-two-quick-closes-could-pin-one-turn-number-twice-001 and its prevention rule: the
    seat's last record is awaited before the next; kept, since Codex goes through the same `close`.
  - F-claude-509-a-turn-would-have-opened-in-the-repository-its-terminal-moved-to-001: each turn
    keeps the repository it was taken in, and a click finds it by the turn's sha; a Codex turn
    gets the same, since it goes through the same `Turn`.
  - AD-claude-650-marley-joins-a-codex-app-server-of-its-own-per-terminal-001 and
    L-claude-650-a-codex-thread-resume-renames-its-client-001: the lead alone; Marley subscribes a
    lead made before it joined only once idle, so a turn already running then is not seen.
  - PR-claude-two-engines-answering-the-same-question-will-diverge-001: one recorder (D1).
- **Completed pipelines.** #509 (the recorder, the scenario's shape), #650 (the follow and its
  stand-in), #651 (`item/started` read beside the fold; its notes on the TUI drawing a user
  message it did not send), #540 (resume; a resumed Codex thread counts on, D3).
- **Brain:** no `rusty` MCP server in this repository's sessions; the promotion runs `brain_ask`.

### Discovery
- **The recorder** (`crates/marley_workbench/src/turns.rs`): `on_event` (`:108-160`) is Claude
  Code's: it reads a `HookEvent`'s name, `session_id`, `prompt` and `cwd`, and calls `open`
  (`:216-262`) or `close` (`:293-327`). `open` needs the `TerminalView` (its workspace's project and
  git store, the terminal's working directory), the session, a folder and a title. `close` and
  `record` (`:343-421`) need only the view's id. `on_end` and `forget` (`:163-179`) are already
  agent-neutral.
- **The follow** (`crates/marley_workbench/src/codex_server.rs`): `heard` (`:1100-1196`) gets every
  notification; it reads `item/started` for file changes (`:1121-1130`) and folds the lead's
  decoded notifications (`:1192`). It holds the view's id (`Following.view`) and finds it through
  `view_of` (`:879-886`). `OPTED_OUT` (`:72-84`) drops deltas only, so `item/started` and
  `item/completed` arrive.
- **The shapes** (`crates/marley_agent/src/codex_events.rs`): `Notification::TurnStarted { thread }`
  (`:222`) drops the turn; `TurnCompleted { thread, turn }` (`:223-227`) keeps `status` and
  `error`, not `id` (`Turn`, `:105-115`). `Thread` (`:50-68`) has no `cwd`. Codex's schema (0.162.0,
  generated for this draft): `turn/started` carries `turn.id`; `TurnStatus` is `completed`,
  `interrupted`, `failed`, `inProgress`; a `userMessage` item has `content[]` of `UserInput`,
  whose text inputs are `{type: "text", text}`; `Thread.cwd` is required.
- **The rows** (`crates/marley_workbench/src/rail.rs:5724-5760`, `:7851`): a terminal row's turns
  are `Turns::of(view)` whatever its agent; nothing in the rail changes.
- **The stand-in** (`script/e2e/codex-fixture.sh:241-260`): `work` sends `turn/started` with a
  fresh turn id and a `reasoning` item; `done` and `fail` send `turn/completed`. It edits no file
  and sends no `userMessage` item.
- **Line numbers** in `rail.rs` are at `19dda499a2`; #743's Code phase was editing `rail.rs` while
  this was drafted, so the promotion reads them again.

### Design
- **`codex_events`** (Marley crate): `Notification::TurnStarted { thread, turn: String }` and
  `TurnCompleted`'s `Turn.id`; `Thread.cwd: Option<String>`; `user_message_started(params) ->
  Option<(thread, turn, text)>` beside `file_change_started`, the text inputs joined.
- **`turns.rs`** (Marley crate): `on_codex(view, id, Boundary, cx)` where `Boundary` is
  `Started { thread, turn, folder }`, `Titled { turn, text }` or `Ended { turn, failed }`.
  `Started` closes an open turn, then opens one titled `a prompt`, the thread's id as the session;
  `Titled` sets the open turn's title when its id matches; `Ended` closes it when its id matches.
  The open turn keeps its Codex turn id. `open` takes the folder it is given; the rest is #509's.
- **`codex_server.rs`** (Marley crate): in `heard`, for the lead: `turn/started` sends `Started`
  with the thread's `cwd` (kept from `thread/started`, `thread/read` or `thread/resume`) or the
  terminal's working directory; a `userMessage` `item/started` sends `Titled`; `turn/completed`
  sends `Ended` (`failed` for status `failed`). `thread/closed` and the server's stop close through
  `turns::on_end`. Each runs in `cx.update` with the view's entity, found as `view_of` finds its id.
- **The guide:** "Per-turn diffs" names Codex on its App Server.
- **File manifest:** `crates/marley_agent/src/codex_events.rs`, `crates/marley_workbench/src/turns.rs`,
  `crates/marley_workbench/src/codex_server.rs` (Marley crates); `script/e2e/codex-fixture.sh`,
  `script/e2e/753-codex-turn-checkpoints.sh`; `docs/marley/guide.md`.
- **Ledger rows:** none; no Zed path, no new spawn site (`turn_git.rs`'s two spawns serve both).

### Visual check plan
| REQ | The scenario sets up and does | The shot |
|---|---|---|
| 001, 004, 005 | Codex from the picker on the stand-in; `prompt Add a README`, `write README.md …`, `sh sed -i … src/lib.txt`, `done`; `prompt Explain the build`, `done`; `prompt Fix the typo in util`, `write src/util.txt …`, `interrupt`; `prompt Refactor the parser`, `write src/util.txt …`, `fail`; the row's Turns opened | `753-01-turns` |
| 002 | "Add a README" clicked | `753-02-first-turn` |
| 003 | "Fix the typo in util" clicked | `753-03-interrupted` |
| 006 | the run log: `for-each-ref`, each commit's parent, the user's state before and after | the log |
| 007 | none: the closes on a thread's close, a stop, a leave and a terminal's close are read in the diff | review |
| 008 | the gate | `script/gates.sh --diff` |

What no scenario reaches: a real Codex's turn (it needs a model); the stand-in plays its
notifications in Codex's generated shapes.

### Risks
- **The installed Codex is 0.162.0**, past #650's tested range; the promotion brings the range and
  the stand-in up to date, as #752's does. The notifications read here have the same names at
  0.155.1 and 0.162.0.
- **The start checkpoint races the first edit.** `turn/started` reaches Marley over the socket
  before the model's first reply in practice; an edit Codex made before the checkpoint lands in the
  turn before (L-claude-509's case).
- **A user message's item may come after `turn/started`**, or not at all for a turn Codex starts
  itself (a compaction); the title then stays `a prompt`. A compaction changes no file, so it is
  not listed.
- **A turn already running when Marley subscribes** (a lead made before Marley joined) has no
  start checkpoint; it is not listed, and the next turn is.
- **Two agents in one checkout** share each other's edits in their turns, as #509's do.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress. Brain:
  consultation 6c50998fc566464fa6398c767e1d0067, nothing on this seam.
- **Seams checked again** at `ad27c1d635` (an Explore pass over all eight): `turns.rs` as cited
  (`on_event` 108-160, `on_end` 163, `forget` 171, `open` 216-261, `close` 293-328, `record`
  343-420); `turn_git.rs` unchanged; `agent_events.rs` `after_fold` 344-379, `end` 663-693, `forget`
  696; `codex_server.rs` `OPTED_OUT` 73-87 (no turn or item notification opted out), `Following`
  816-832, `fold` 876-887 with #752's `push_change` 902-920, `heard` 1148-1243 (`item/started` at
  1169, `ThreadClosed` 1234, the fold 1241), `server_stopped` 1325; `codex_events.rs` `Thread.cwd`
  (#752), `Turn` 108-116, `TurnStarted { thread }` 160 and its decode 225; `rail.rs` `render_turns`
  6231-6324 (a `· failed` mark exists), the row's turns from `Turns::of` at 8379, `open_turn` 4722.
  #771 put 0.162.0 in the tested range, so the draft's first risk is gone.
- **What differs from the draft, and where the design takes it:**
  - **The rail's end never closes a Codex turn:** `agent_events::end` skips a `Done` or `Error`
    seat (667-671), which a `thread/closed` (`Ended`) and a server stop (`Error`) leave. The follow
    closes the turn itself at `ThreadClosed` and at `server_stopped`; Codex leaving the foreground
    leaves the seat live, so the rail's `end` still closes it there, and the terminal's close goes
    through `forget`.
  - **No turn id and no prompt are read:** `Turn` gains `id`, `TurnStarted` its turn's id, and
    `user_message_started` reads a `userMessage` item's text from `item/started`.
  - **`turns::open` needs the view:** the follow finds the `TerminalView` through
    `mcp::terminals`, as `push_change` does, and calls `turns::on_codex` in its update. The rule
    that a new session closes the open turn lives in `on_event`; `on_codex` keeps it too.
  - **The folder** is the seat's `cwd` label, which `Input::Lead` sets from `Thread.cwd` (#752),
    else the server's start folder (`Following.folder`).
  - **The stand-in** has no `interrupt`, `write`, `sh` or `userMessage`: its new cues are `prompt
    <text>` (a turn and its `userMessage` item), `write <path> <text>`, `sh <command>` (in the
    thread's `cwd`) and `interrupt`; the scenario waits between a prompt and its edits.

## Phase 2 — Code
- **Built:**
  - `codex_events`: `Turn.id`; `Notification::TurnStarted { thread, turn }`;
    `user_message_started(params)` (the thread, the turn and the `userMessage` item's text inputs
    on one line).
  - `turns.rs`: `CodexTurn::{Started, Titled, Ended}` and `on_codex`: a start closes the open turn
    and opens one titled `a prompt` with the thread's id as its session; the prompt retitles the
    open turn with the same Codex turn id; the end closes it, `failed` for a failed turn. `Open`
    keeps the Codex turn's id. The rest is #509's.
  - `codex_server.rs`: `Lead.cwd` from `Thread.cwd`; `Following::tell_turns` (the view through
    `mcp::terminals`, `turns::on_codex` in its update), `follow_turn` (`turn/started` with the
    lead's folder or the server's, `turn/completed`), `end_turn` at `thread/closed` and at the
    server's stop; `item_started` (the file change's paths, moved out of `heard`, and the user's
    message).
  - The stand-in Codex's cues `prompt`, `write`, `sh`, `interrupt`.
  - Guides: the in-app "Each agent turn as its own diff", `docs/marley/guide.md`'s "Per-turn
    diffs".
- **Deviations from the plan:**
  - `heard` passed clippy's 100 lines; its `item/started` handling became `item_started` and its
    turn arms `Following::follow_turn`.
  - The folder is the lead's `cwd`, kept on `Lead`, rather than the seat's label: the follow holds
    it without reading the seat.
- **Review of the diff:**
  - REQ-007: `thread/closed` and the server's stop close the turn in the follow, since the rail's
    `end` passes over the `Done` and `Error` seats they leave; Codex leaving the foreground leaves
    the seat live, so the rail's `end` closes it; the terminal's close goes through `forget`.
  - A `turn/completed` for another turn id, or a `userMessage` of another turn, changes nothing;
    a `turn/started` while a turn is open closes it first.
  - Re-entrancy: `tell_turns` runs from the follow's `cx.update`, outside any entity's update, and
    updates the view alone.
  - Nothing leaves the box; no new spawn site (`turn_git.rs`'s two serve both agents).
- **Gate:** `just gate-diff` GREEN, 17 passed, after the guides' edits.

## Phase 3 — Test
- **Scenario:** `script/e2e/753-codex-turn-checkpoints.sh` (`compositor sway`). One calibration run
  placed "Turns (3)" at (74, 268); the full run then passed every check.
- **Shots:**
  - `753-00-row`: the Codex row `failed · 15k tokens`, and "Turns (3)" under it; the TUI's twelve
    cues in its terminal.
  - `753-01-turns` (REQ-001, REQ-004, REQ-005): opened, newest first: "Refactor the parser · 1
    file · failed", "Fix the typo in util · 1 file", "Add a README · 2 files"; no row for "Explain
    the build".
  - `753-02-first-turn` (REQ-002): the commit view of "Add a README", author Marley: `README.md`
    added (`# repo`) and `src/lib.txt` `beta` → `gamma`, the shell's `sed`; nothing of the later
    turns or the user's staged change.
  - `753-03-interrupted` (REQ-003): "Fix the typo in util": `src/util.txt` from its start
    checkpoint (the user's staged line in the tree) to `the util module`, its one change.
- **Checks** (REQ-006): three refs under `refs/marley/turns/<thread id>/`, numbered 1 to 3, each
  `Marley <marley@localhost>` on a parent titled `Checkpoint`, with the files above; `HEAD` and
  the index as before the session.
- REQ-007 by the review.
- **Not reached by a scenario:** a real Codex's turn (a model); the closes at a thread's close and
  a server's stop (read in the diff).

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added); the in-app guide ("Each agent turn as its own diff") and
  `docs/marley/guide.md` ("Per-turn diffs"); `docs/marley_architecture/marley_workbench.md`
  (Codex's turns). No Zed path, so no ledger row.
- **Ledgers:** L-claude-753-a-seat-left-done-or-error-is-past-the-rails-end-001,
  AD-claude-753-a-codex-turn-is-the-app-servers-turn-recorded-by-509s-recorder-001.
- **Brain:** consultation 6c50998fc566464fa6398c767e1d0067, `brain decide`.
- Ticket closed, pipeline archived, committed.
