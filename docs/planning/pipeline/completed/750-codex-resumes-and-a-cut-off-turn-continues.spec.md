---
pipeline_id: f9daf23c-5ae8-4c14-88e3-31b4eb62fa6d
ticket: docs/planning/tickets/open/TICKET-750-codex-resumes-and-a-cut-off-turn-continues.md
status: Phase 4 — Complete PASS
title: Codex resumes after a restart, and a cut-off turn offers Continue
type: feature
slice: the finish line (docs/planning/design-notes/finish-line-2026-10-11.md), item 1, restarts that lose nothing, first half
references:
  - docs/planning/pipeline/completed/540-session-resume-after-restart.spec.md
  - docs/planning/pipeline/completed/650-codex-app-server-state.spec.md
  - docs/planning/pipeline/completed/651-codex-approvals-and-prompts.spec.md
  - docs/planning/pipeline/completed/522-review-notes-to-the-agent.spec.md
  - docs/t3code_architecture/01-agents-and-providers.md
  - docs/t3code_architecture/06-orchestration-mcp-and-automations.md
---

## Title
After a Marley relaunch, a terminal whose Codex ran on Marley's own App Server (#650) types
`codex resume <thread id>` in the thread's folder, as #540 does `claude --resume <id>` for Claude
Code. A resumed agent whose turn was still open when Marley stopped (it was working or waiting, and
no turn end came) shows **Interrupted · Continue** on its rail row; Continue types "Continue where
you left off." and Enter once the agent is at its prompt. Marley never continues a turn by itself
(T3 open question 3, default a chip). Chad, 2026-10-11, on "Restarts that lose nothing": "1.) this is
nice". The second half, restored scrollback, is #751.

## Scope
### In
- **The agent per session.** `marley_agent_sessions` gains the agent (`claude-code` or `codex`) and
  whether the session's turn is open, both with defaults, so #540's rows read as Claude Code with
  no open turn.
- **Codex's session.** When #650 follows a Codex terminal's lead thread, its id and the folder its
  App Server started in are saved under the terminal's `MARLEY_TERMINAL_ID`, as `SessionStart` saves
  Claude Code's. A new lead (Codex's `/new`) replaces it. The lead closing (`thread/closed`,
  `notLoaded`) or Codex leaving the foreground while Marley runs drops it; a quit keeps it
  (`quitting`, as #540).
- **Codex resumed.** At relaunch, with `marley.resume_agents` on, a restored local terminal whose
  row is Codex's types `cd -- <folder> && <codex> resume [#532's arguments] <id>` through the same
  startup handshake. With `marley.codex_app_server` on and the installed Codex in #648's tested
  range, Marley starts the terminal's App Server first and the line names it (`--remote
  unix://<socket> --cd <folder>`), so #650 follows the resumed thread; otherwise the line has no
  `--remote`, and the row reads the quiet timer as any Codex without the server does.
- **The open turn.** Whenever a seat with a saved session moves into or out of a turn, the row is
  written: working or waiting (approval, input) is open; idle, failed and ended are closed. The
  states come from wherever the seat's come today: Claude Code's hook frames (#519), an agent's
  report (#652) and Codex's App Server (#650). Nothing is written while Marley quits.
- **The chip.** At `init` the rows whose turn was open are read into memory. When such a session
  resumes this launch, its terminal's rail row shows **Interrupted · Continue**, with a tooltip
  saying the restart cut the turn off and what Continue types. The chip leaves when Continue is
  clicked, a turn starts in that terminal, the session ends, or the terminal closes.
- **Continue.** A click types "Continue where you left off." and Enter through #522's delivery
  (`terminal_drive::paste_then`), only while the seat is idle at its prompt; otherwise a toast names
  the seat's state ("working", "asking for permission", "no idle signal") and nothing is typed.
- The Marley page's "Resume Claude Code Sessions" item and `resume_agents`' doc name Codex too.
- `docs/marley/guide.md`: Codex resumes; the chip.
- `script/e2e/750-codex-resumes-and-a-cut-off-turn-continues.sh`; the Codex stand-in
  (`script/e2e/codex-fixture.sh`) learns `resume`.

### Out (explicitly deferred)
- **A Codex run without Marley's App Server** (the switch off, the default; a Codex typed by hand;
  a version outside the tested range): Marley has no way to learn which thread such a terminal
  runs (Codex's hooks run in its daemon, OSC 9 carries no id), so it is not resumed. Matching
  rollout files by folder and start time was rejected for #650 (AD-claude-650) and is rejected here.
- **Widening #648's tested range**: done by #771 (0.155.1 to 0.162.0), so the box's Codex reaches
  this ticket with `marley.codex_app_server` on.
- **T3's background-work note** ("this background work was cancelled", `RestartBackgroundNote.ts`):
  what Claude Code's hooks report for background shells is unchecked (report 06, item 5).
- **Continuing by itself**, T3's setting (off there too): Chad's default is the chip.
- **Prompts through the App Server** (`turn/start`, B1 part 3): Continue is typed, as review notes
  are, for both agents.
- **The report's `resume_argv`** (#652, validated, unused): Marley builds the line itself.
- Restored scrollback (#751); past sessions listed in Threads (#754); remote terminals; the
  harness's sessions (#632).

## Reference (§20)
N/A — Marley-specific. Each CLI resumes its own session (`claude --resume <id>`, `codex resume
<id>`), and Zed and Warp restore no process: Zed brings a terminal back as a new shell in its folder
(`TerminalView::deserialize`), and Warp's Session Restoration brings back "your windows, tabs, panes,
and recent Blocks" (docs.warp.dev/terminal/sessions/session-restoration, cited by #540). The
behavior matched is T3 Code's restart continuation (report 01 §2.7, report 06 §2.14: a run cut while
its turn was live gets "Continue where you left off.", never after the user's own stop), offered
as a chip by Chad's default for T3's open question 3, and Orca's cold restore, which relaunches
Codex with `codex resume <id>` (Orca report 05 §2.2, `getAgentResumeArgv`).

### Prior art
- **The code we ship (the highest-yield leg):**
  - `crates/marley_workbench/src/resume.rs` (#540): the table `marley_agent_sessions(terminal_id,
    session_id, folder)` (281–285), `Sessions` (37–42), `on_event` (105–132), `on_report`
    (136–145, #652), `ended` (163–177), `resume_restored` (218–265), which hard-codes Claude Code
    (`launch_mode(…, AgentKind::Claude)` at 240) and writes after `start_init_command_startup_handshake`
    or `STARTUP_TIMEOUT`. This ticket widens it; no new store.
  - `marley_agent::resume_line` (`marley_agent.rs:242`), Claude-only, and `is_session_id` (259),
    which accepts Codex's ids (8-4-4-4-12 hexadecimal, version 7); `command` (293) already builds
    `--remote 'unix://<socket>' --cd '<folder>'` from `Remote` (192); `quote_argument` (321).
  - `codex_server.rs` (#650): `prepare` (270, the switch, a local project, the tested range),
    `serve` (352, a server per terminal), `Lead` (805) in `Following` (813), `tick_over` (1016) and
    `heard` (1100) find and follow the lead; `marley_agent::codex_events::fold` (322) sets the seat's
    `thread` label and moves it to working on `turn/started`; a `turn/completed` that was
    `interrupted` changes nothing (355), so the user's own interrupt reads as a closed turn once
    `idle` comes.
  - `agents.rs`: `start_in_terminal` (417) starts a Codex's server before it types, which
    `resume_restored` does not; `agent_line` (338), `launch_mode` (368), `STARTUP_TIMEOUT` (57).
  - `agent_events.rs`: `on_frame` (267, Claude Code's frames), `fold_codex` (131), `end` (655); the
    seat states and #569's stall labels. Nothing persists a seat's state today.
  - `review_notes.rs` (#522): `readiness` (93) and `deliver` (108–158), which refuse unless the
    target seat is idle and then call `terminal_drive::paste_then` (313), the Enter after
    `AFTER_PASTE` (PR-claude-keys-after-a-paste-go-in-a-later-write-001).
  - `rail.rs`: `render_terminal_row` (5486), `terminal_marks` (9091), `permission_chip` (9024); no
    mark takes a click yet, and the row's close `IconButton` (5499–5507) is the pattern for a control
    inside a row (`cx.stop_propagation()`).
- **Behavior maps:** T3 report 01 §2.7 and §3 item 2 (`RestartContinuation.ts:28-170`: only a run
  whose turn was live, never after a user's stop, a newer prompt or `/compact`); report 06 §2.14 and
  §3 item 5 (the chip, "Interrupted · Continue"; a turn ended by Escape fires no hook, so "mid-turn"
  is a guess and the chip asks rather than acts); Orca report 05 §2.2 and report 01 §2.6; the
  design note `claude-and-codex-on-their-own-tools-2026-10-02.md` B5 and #651's Out (B1 part 4:
  `codex resume --remote unix://<socket> <thread id>`).
- **Published material:** `codex resume --help` (0.162.0): `[SESSION_ID] [PROMPT]`, `--remote
  <ADDR>`, `-C/--cd`, `--sandbox`, `--ask-for-approval`; with no `--all` it filters by folder.
  Codex's App Server protocol: `thread/resume`, `turn/started`, `turn/completed` with status
  `completed | interrupted | failed`. Claude Code's hooks: `UserPromptSubmit`, `Stop`, `StopFailure`.

## UI proof
`script/e2e/750-codex-resumes-and-a-cut-off-turn-continues.sh`, `compositor sway`, on #540's
set-up (`quit_marley`, `launch_marley` on the same profile; a scratch repository with `sub`).
- A stand-in `claude` (#540's, Python, first on the terminal's PATH) that prints `SessionStart`,
  and on `work` a `UserPromptSubmit` frame and `working` with no `Stop`, on `done` both frames, and
  logs every line it reads.
- #650's Codex stand-in (`codex-fixture.sh`, named by `MARLEY_CODEX`), with
  `profile_setting marley.codex_app_server true`; its `work` cue leaves a turn open, `done` closes
  it; it learns `resume <id>` (logged with `--remote` and its folder; its TUI resumes the thread on
  the server).

Shots:
- `750-01-working`: Claude Code in `sub` after `work`; its row working.
- `750-02-chip`: after a relaunch, the resumed Claude Code; its row shows Interrupted · Continue.
- `750-03-continued`: Continue clicked; the log's last line is "Continue where you left off."; the
  chip gone.
- `750-04-finished`: `done`, a relaunch: resumed, no chip.
- `750-05-codex-working`: a Codex from New Agent in the repository after `work`; its row working.
- `750-06-codex`: a relaunch: the log's `resume --remote unix://… <thread id>` in its folder; its
  row Codex's, with the chip.
- `750-07-codex-continue`: Continue clicked on the Codex row; the stand-in's log has the prompt.
- `750-08-codex-no-server`: `marley.codex_app_server` off, a new Codex, `work`: started with no
  `--remote`.
- `750-09-codex-off`: a relaunch: that terminal a plain shell; every resume in the log is the first
  thread's.

## Locked-In Decisions
- **D1 — One table, one key.** The agent and the open turn are two columns of #540's
  `marley_agent_sessions`, keyed by `MARLEY_TERMINAL_ID` (#575). Rejected: a second table for Codex
  (two stores for one fact, PR-claude-two-engines-answering-the-same-question-will-diverge-001).
- **D2 — Codex's id comes only from the App Server Marley started for that terminal** (#650's lead).
  Rejected: rollout files matched by folder and start time (ambiguous for two Codex in one folder,
  AD-claude-650); Codex's hooks (they run in its daemon with another terminal's environment);
  the shared daemon (AD-claude-650).
- **D3 — The resume needs the id, not the server.** With the switch on and the version in range,
  the resumed Codex gets a new App Server and `--remote`, so #650 follows it; otherwise plain
  `codex resume <id>`. Rejected: refusing to resume when the switch was turned off after the run.
- **D4 — The open turn is written as it changes,** not read at the quit, so a crash or a reboot,
  where `on_app_quit` may not run, still knows. Rejected: a snapshot in `on_app_quit` only.
- **D5 — Open means working or waiting.** A turn waiting on an approval or an answer was cut as
  much as a working one. The user's interrupt closes the turn (Codex's `turn/completed`
  `interrupted`, Claude Code's interrupt `PostToolUseFailure`), so it gets no chip; an Escape that
  fires no hook leaves the turn open, and the chip asks rather than acts (report 06, item 5).
- **D6 — A chip, never an automatic continue** (Chad's default for T3's open question 3). No new
  setting: the chip comes only with a resume, which `marley.resume_agents` governs, and acts only on
  a click.
- **D7 — Continue is typed,** through #522's delivery, only while the seat is idle, for both
  agents. Rejected: `claude --resume <id> "Continue where you left off."` on the resume line (that
  acts without a click); `turn/start` through the App Server (B1 part 3, not built).
- **D8 — The prompt is T3's text,** "Continue where you left off.", since it names nothing the
  agent could misread.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley relaunches and a restored local terminal ran a Codex on Marley's App Server at the quit, the system shall type `codex resume … <thread id>` into it, in the folder the server started in, once its shell is ready. | Shot 750-06; the stand-in's log |
| REQ-002 | WHERE `marley.codex_app_server` is on and the Codex is in the tested range, the resume line shall name a new App Server for that terminal (`--remote unix://<socket>`), and the row shall show the resumed thread's state. | Shot 750-06; the log |
| REQ-003 | WHEN a Codex ran without Marley's App Server, the system shall not resume it. | Shots 750-08, 750-09; no resume of it in the log |
| REQ-004 | WHEN a resumed Claude Code or Codex was working or waiting at the quit, its rail row shall show Interrupted · Continue. | Shots 750-02, 750-06 |
| REQ-005 | WHEN a resumed agent's turn had ended before the quit, its row shall show no chip. | Shot 750-04 |
| REQ-006 | WHEN Continue is clicked while the agent is idle at its prompt, the system shall type "Continue where you left off." and Enter into its terminal and remove the chip. | Shots 750-03, 750-07; the logs |
| REQ-007 | WHEN Continue is clicked while the agent is not idle, the system shall type nothing and name the agent's state in a toast. | Review of the diff |
| REQ-008 | WHEN a turn starts in a terminal that shows the chip, or its session ends, the system shall remove the chip. | Review of the diff |
| REQ-009 | WHEN the user exits Codex, or Codex leaves the terminal's foreground while Marley runs, the system shall drop its saved thread. | Review of the diff |
| REQ-010 | The system shall not continue an agent's turn without a click. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes, re-verified at promotion.
- **P2 Code** — `marley_agent::resume_line` per agent; `resume.rs` (the columns, Codex's lead, the
  open turn, the chip's set, the Codex server for a restored terminal); `codex_server.rs` (the lead
  handed to `resume`); `agent_events.rs` (state changes handed to `resume`); `rail.rs` (the chip and
  its click); `review_notes.rs` (delivery shared); the Marley page's label and the setting's doc;
  the guide; the scenario and the Codex stand-in's `resume`; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
