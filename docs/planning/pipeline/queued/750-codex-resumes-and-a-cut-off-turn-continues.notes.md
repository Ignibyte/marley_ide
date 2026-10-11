# Codex resumes after a restart, and a cut-off turn offers Continue — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-750-codex-resumes-and-a-cut-off-turn-continues.md
- **Pipeline spec:** 750-codex-resumes-and-a-cut-off-turn-continues.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, on the survey list (`docs/planning/design-notes/finish-line-2026-10-11.md`),
  item 1, "Restarts that lose nothing": "1.) this is nice". The survey's text: "resume Codex too
  (`codex resume`, the id from the App Server), an 'Interrupted · Continue' chip on a resumed row"
  (`docs/t3code_architecture/README.md`, item 1; reports 01 item 2, 06 item 5). Open question 3's
  default, "a chip", stands. Queued after the tabs batch (#743 to #749).
- **Classification:** feature; S for each half by the survey's measure, M together with the Codex
  server for a restored terminal and the scenario's two stand-ins.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **Recall (§18.3):**
  - AD-claude-540: the session rides `marley_agent_sessions` under `MARLEY_TERMINAL_ID`; the id only
    from `SessionStart`, never from `~/.claude/projects`; `quitting` keeps rows at the quit;
    one resume per session per launch.
  - AD-claude-575 and PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001: the rows are
    read at `init`, never at a view's `deserialize`.
  - AD-claude-650: one App Server per Codex terminal, so a thread maps to its terminal by the
    socket; matching threads by folder and time was rejected. Off by default; the tested range
    0.155.1 to 0.158.0 (`versions.rs:103-115`).
  - #651's Out: "Resume after a restart (B1 part 4, B5): a restored terminal runs `codex resume
    --remote unix://<socket> <thread id>`, keyed by the thread #650 mapped to its
    `MARLEY_TERMINAL_ID`, in `resume.rs`'s table beside Claude Code's sessions." D10 kept it a
    slice of its own.
  - PR-claude-an-answer-for-a-seat-names-the-session-prompt-and-stop-it-belongs-to-001 and
    F-claude-566: what a later event acts on carries the session it belongs to; the chip's set is
    keyed by terminal and session.
  - PR-claude-keys-after-a-paste-go-in-a-later-write-001: Continue's Enter goes after the paste,
    through `paste_then`.
  - F-claude-547: a scenario's fake runs first on the terminal's PATH; the runner already sets
    `MARLEY_CODEX` and `CODEX_HOME` to the run's folder (`script/e2e.sh:692-695`).
  - Brain: no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Discovery (2026-10-11; re-verify at promotion):**
  - `resume.rs`: table 281–285, `Sessions` 37–42, `init` 51–100 (`quitting` set in `on_app_quit`
    76–80), `on_event` 105–132, `on_report` 136–145, `on_release` 149–159, `ended` 163–177,
    `resume_restored` 218–265 (Claude-only at 240).
  - `marley_agent.rs`: `send_payload` 135, the bypass arguments 153–164 (Codex: `--sandbox
    danger-full-access --ask-for-approval never`), `Remote` 192, `resume_line` 242–256,
    `is_session_id` 259–265, `launch_line_after` 274, `command` 293–312, `quote_argument` 321.
  - `codex_events.rs`: `THREAD_LABEL` 29, `Thread` 51 (`is_lead` 74; no folder field), `Turn` 107,
    `fold` 322–391 (`Input::Lead` 331, `TurnStarted` 339, a failed `TurnCompleted` 344, an
    `interrupted` one 355).
  - `codex_server.rs`: `CodexAppServer` 88–104, `prepare` 270–317, socket paths 325–332, `serve`
    352–446, `Lead` 805, `Following` 813, `server_tested` 988, `tick_over` 1016–1086, `follow_lead`
    1089, `heard` 1100–1194.
  - `agent_events.rs`: `AgentEvents` 45 (in memory only), `fold_codex` 131, `on_frame` 267–331,
    `end` 655–685 (calls `resume::ended`); the rail calls `end` from `note_claude_code`
    (`rail.rs:2299`).
  - `claude_events.rs`: `UserPromptSubmit` opens a turn (419–436), `Stop` and `StopFailure` close it
    (463–476), an interrupt `PostToolUseFailure` closes it (551); "an Escape fires no hook" (287).
  - `review_notes.rs`: `readiness` 93, `deliver` 108–158; `send_selection.rs:target_of` 330–350
    (`ready` is a seat in `Idle`); `terminal_drive.rs:paste_then` 313.
  - `rail.rs`: `render_terminal_row` 5486–5590, its marks at 5534/5555, `permission_chip` 9024,
    `terminal_marks` 9091; `marley_rail::TerminalRow` (628), `TerminalAgent` (341).
  - Settings: `settings_content/src/marley.rs` `resume_agents` 31–36, `codex_app_server` 202;
    `marley_workbench.rs` `ResumeAgents` 592–607; `marley_page.rs` "Resume Claude Code Sessions"
    218–240.
  - The box on 2026-10-11: Codex 0.162.0 (outside the tested range), Claude Code 2.1.296, and
    `marley.codex_app_server` unset in Chad's settings (off).

### Design
- **`marley_agent` (Marley crate):** `resume_line(kind, mode, session, folder, remote:
  Option<&Remote>)`: Claude Code as today; Codex `cd -- <folder> && <codex> resume [--remote
  'unix://<socket>' --cd '<folder>'] [mode's arguments] <id>`, the id checked by `is_session_id`.
  `RESUME_PROMPT = "Continue where you left off."`.
- **`resume.rs` (Marley crate):**
  - The table: two columns by migration, `agent TEXT NOT NULL DEFAULT 'claude-code'` and
    `turn_open INTEGER NOT NULL DEFAULT 0`; `Saved` gains both; `save_turn(terminal_id, open)`.
  - `init`: the rows with `turn_open` into `Sessions::cut` (terminal ids), before any restore.
  - `on_codex_lead(terminal, thread_id, folder)` from `codex_server`'s follow (a lead found or
    replaced); the lead's close calls `remove` unless `quitting`.
  - `on_seat_state(terminal, state)` from `agent_events` after each fold (Claude Code's frames,
    reports, Codex): writes `turn_open` when it changes, never while `quitting`.
  - `resume_restored`: the saved agent picks the line; for Codex, `codex_server::prepare` and a
    server for this terminal before the write (a variant of `serve` that takes an existing
    terminal), else the plain line. A resumed id in `cut` puts the terminal in `Sessions::offer`
    (terminal id → session id).
  - `offered(terminal_id) -> bool` for the rail; `continue_turn(view, cx)`: the seat's readiness;
    idle → #522's delivery of `RESUME_PROMPT`, then the offer goes; else a toast with the state.
    A turn start or session end for that terminal clears the offer.
- **`codex_server.rs`:** hands the lead and the server's folder to `resume::on_codex_lead`; a
  `serve_for(terminal, prepared)` used by the resume.
- **`agent_events.rs`:** after `on_frame`, `fold_codex` and a report's fold, `resume::on_seat_state`.
- **`rail.rs`:** a clickable chip in `render_terminal_row` beside `terminal_marks`, "Interrupted ·
  Continue", with the tooltip; `cx.stop_propagation()` as the close button does.
- **`review_notes.rs`:** its delivery made `pub(crate)` for a plain text, or a small shared helper.
- **Zed paths (rows exist, each widened):** `crates/settings_ui/src/marley_page.rs` (the item's title
  and description name Codex), `crates/settings_content/src/marley.rs` (the doc of
  `resume_agents`). No new setting.
- **File manifest:** `crates/marley_agent/src/marley_agent.rs`; `crates/marley_workbench/src/{resume.rs,
  codex_server.rs, agent_events.rs, rail.rs, review_notes.rs}` (Marley crates); the two Zed paths
  above; `docs/marley/guide.md`; `script/e2e/750-codex-resumes-and-a-cut-off-turn-continues.sh`,
  `script/e2e/codex-fixture.sh` (the `resume` subcommand).

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 004 | Claude stand-in in `sub`, `work` (a `UserPromptSubmit`, no `Stop`) | 750-01-working |
| 004 | `quit_marley`, `launch_marley`, settle | 750-02-chip: resumed (log `--resume <id> @ …/sub`), the chip on the row |
| 006 | Clicks Continue | 750-03-continued: the log's line "Continue where you left off."; no chip |
| 005 | `done`, relaunch | 750-04-finished: resumed, no chip |
| 001, 002, 004 | `profile_setting marley.codex_app_server true`; a new terminal, New Agent with Codex; `work`; relaunch | 750-05-codex: the log's `resume --remote unix://… <thread id>` in the repository; the Codex row with the chip |
| 006 | Clicks Continue on the Codex row | 750-06-codex-continue: the stand-in's log holds the prompt |
| 003 | Switch off, a new Codex session, relaunch | 750-07-codex-off: a plain prompt; no resume logged |
| 007, 008, 009, 010 | review | `continue_turn`'s readiness, the offer's clearing, the lead's drop, no path that types without a click |
Not reachable by a scenario: a real Claude Code or Codex resuming a real session (an account and a
model call); the stand-ins print what the hooks and the App Server send.

### Risks
- **A wrong "cut".** An Escape that fires no hook leaves the turn open, so a deliberate stop can
  show the chip. It only asks (D5); the tooltip says what Continue does.
- **The resumed Codex's server.** `resume_restored` writes into a terminal Zed made, not one
  `start_in_terminal` made, so the server must be started for it and the line written only after
  the socket exists (bounded at 5 s, #650); on failure the plain line, logged.
- **Codex rows that never resume for Chad today.** With the switch off and Codex 0.162.0 outside
  the range, nothing saves a Codex thread. The guide says so; the range check is its own chore.
- **A report's session** (#652) is Claude Code's today; with the agent column it resumes as the
  agent the report named, never as Claude Code by default.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
