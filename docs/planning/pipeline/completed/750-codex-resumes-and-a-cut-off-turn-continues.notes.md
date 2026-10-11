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

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress. #771 has
  widened the range to 0.162.0 (the spec's Out updated).
- **Seams checked again** at `f366712fdf` (an Explore pass over the eleven). `resume.rs` holds as
  described (one migration; `resume_restored` hard-codes Claude Code at 240); `resume_line`
  (`marley_agent.rs` 242) is Claude-only and takes no `Remote`; `command` (293) builds `<program>
  --remote 'unix://<socket>' --cd '<folder>'`; `codex_server::prepare` 270, `start` 336, `serve` 352,
  `Lead` 805, `Following` 813, `tick_over` 1016, `follow_lead` 1089, `heard` 1100; `codex_events::fold`
  322; `agents::start_in_terminal` 398 (the server started before the write, 473-486);
  `review_notes::deliver` 108 with `Target.ready` (`send_selection.rs` 343);
  `terminal_drive::paste_then` 313; `rail.rs` `render_terminal_row` 5933, `terminal_marks` 9531, the
  close button 5943-5955. `codex resume --help` (0.162.0) takes `--remote`, `-C/--cd`, `-s` and
  `-a`.
- **What changed from the draft, and where the design takes it:**
  - **No single place changes a seat's state:** six `marley_fleet::apply` sites in
    `agent_events.rs`. `resume` observes the `AgentEvents` global instead and, for each terminal with
    a saved session, writes `turn_open` when its seat moves into or out of working or waiting. One
    observer, no hook at the six sites.
  - **No hook tells another module a terminal's lead thread:** `follow_lead` (and the lead's drop
    in `tick_over` and `heard`) call `resume::on_codex_lead` and `resume::on_codex_lead_gone`;
    `Following` gains the server's folder, passed down from `serve`.
  - **`turn/completed` `interrupted` leaves the seat as it was**; the `idle` status that follows
    closes the turn, which D5 needs.
  - **The Codex stand-in cannot resume:** `codex-fixture.sh`'s TUI learns `resume <id>` (resumes
    that thread on its server, idle) and `exit`, and logs each prompt line.
  - The settings page is `crates/settings_ui/src/marley_page.rs` (as the notes' design already
    said).
- **Brain:** consultation 79b9a86c25244ebea7460b0c00b5ca05.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete

## Phase 2 — Code
- **Built:**
  - `marley_agent.rs`: `RESUME_PROMPT`; `resume_line(kind, mode, session, folder, remote)`, Codex's
    `codex resume [--remote 'unix://<socket>' --cd '<folder>'] [the mode's arguments] <id>` beside
    Claude Code's, None for Gemini and OpenCode.
  - `resume.rs`: `Agent` and `Saved.agent`; two migrations (`agent`, `turn_open`); `Sessions` gains
    `open`, `cut`, `offers` (by seat, with whether the agent has been idle since); `init` reads the
    rows with an open turn into `cut`; `on_codex_lead`; `note_turns`, an observer of the
    `AgentEvents` global that writes `turn_open` when a saved session's seat moves into or out of
    working or waiting and clears an offer on a turn start after idle or an end; `offered` and
    `continue_turn` for the rail; `resume_restored` picks the agent's line and, for Codex, starts a
    new App Server for the terminal (`codex_server::prepare`, `start`) before the line, the plain
    line when it does not come up.
  - `codex_server.rs`: `Following.folder`; `keep_lead` (from `follow_lead`) and `let_lead_go` (the
    lead's close in `heard`, Codex leaving the foreground in `tick_over`).
  - `rail.rs`: `interrupted_chip` (the label, its tooltip, its click with `stop_propagation`, a
    toast for a refusal); `CardLine`, so a row's line under its title can be an element: the chip
    sits on a line of its own under the row's state.
  - Zed paths (rows exist): `settings_ui/src/marley_page.rs`, the item titled Resume Agent Sessions
    with a description naming Codex; `settings_content/src/marley.rs`, `resume_agents`' doc.
  - `script/e2e/codex-fixture.sh`: the TUI's `resume [--remote …] <id>` (`thread/resume` with its
    `cwd`; a server that does not know the thread makes an idle one), a `tui.log` of each line it
    reads.
- **Deviations from the design:**
  - One observer of `AgentEvents` (`note_turns`) in place of a call after each of the six
    `marley_fleet::apply` sites; the promotion entry says why.
  - The chip is read at render time (`resume::offered(seat)`), not a field of `TerminalRow`.
  - `review_notes.rs` is untouched: `continue_turn` uses `terminal_drive::paste_then` directly
    with the seat's state, which is all #522's delivery adds for a fixed line.
  - The resumed Codex's App Server gets no agent variables (`Vec::new()`): a restored terminal is
    Zed's, made without `agent_env`, and a resumed Claude Code runs in the same plain shell. A
    resumed server has no askpass proxy and no `GIT_PROMPTS_OFF`; a candidate for later, with
    Claude Code's resumed shell.
  - `agent_events.rs` is untouched (the observer needs no hook there).
- **Review of the diff:**
  - `continue_turn` types only from `State::Idle` and otherwise returns the state for the toast
    (REQ-007); no path types without a click (D6).
  - `on_release` keeps the row at the quit (`quitting`), so a quit's `thread/closed` or Codex's exit
    does not drop the session it is about to resume.
  - `save` writes a new session with its turn closed, and dedupes an unchanged one, so a lead that
    is told again does not reset an open turn.
  - Each resume and each offer is per seat; a terminal released removes both.
  - Found in Test and fixed here: the chip beside the title took the row's width and cut the title
    to "cp…"; it now has a line of its own (`CardLine`).
- **Gate:** `just gate-diff` RED on rustfmt alone (the `CardLine` match), 16 passed; after `cargo
  fmt`, GREEN, 17 passed. It ran after the in-app guide's edit, which the receipt covers; the
  formatting moved no line of behavior, so the Test phase's shots stand.

## Phase 3 — Test
- **Scenario:** `script/e2e/750-codex-resumes-and-a-cut-off-turn-continues.sh` (`compositor sway`),
  a scratch repository with `sub`; #540's Claude stand-in (`work` opens a turn, `done` ends it, every
  line logged) and #650's Codex stand-in at 0.155.1 with `marley.codex_app_server` on. A quit while
  an agent works asks #550's question, which `quit_working` confirms.
- **Runs that found something:**
  - The Codex row came back with no chip: its `work` never reached Codex. New Agent asks "Where
    should Codex start?" when the terminal in front is in another folder (`sub`), so the scenario's
    Return went to that question. `picker_codex` answers it (the project, first).
  - The chip beside the title cut the title to "cp…" (750-06 of that run). Fixed in the code: the
    chip has a line of its own under the row's state (`CardLine`), and lost its border, which did
    not show on a selected row.
  - The run's own log came back as four lines and zeros: `tee /dev/stderr` reopened the redirected
    log and truncated it. The check now echoes, and matches the stand-in's unquoted arguments.
  - The switch-off check allowed one more resume without saying whose; it now requires every
    resume to be the first thread's.
- **Shots (the last run, every check passing):**
  - `750-01-working` (REQ-004 setup): Claude's row "working · work"; the stand-in printed
    `working`.
  - `750-02-chip` (REQ-004): after the relaunch, the stand-in printed `resumed 11111111-… in
    …/repo/sub`; the row reads the title, `idle`, and **Interrupted · Continue** in the accent color
    on a third line; the title keeps its width. `check the restored terminal resumed the session in
    sub: pass`.
  - `750-03-continued` (REQ-006): the stand-in's last line is "Continue where you left off."; the
    row is back to two lines with no chip. `check Claude Code read the resume prompt: pass`.
  - `750-04-finished` (REQ-005): after `work`, `done` and a relaunch, resumed (`claude run 3`) and
    no chip.
  - `750-05-codex-working`: the Codex tab `codex … --remote 'unix:///…'`, its stand-in printing `cue:
    work`; the row "working · 12k tokens".
  - `750-06-codex` (REQ-001, REQ-002, REQ-004): the restored tab typed `cd -- '<repo>' && '<codex>'
    resume --remote unix://…/codex/1.sock --cd <repo> c2cc46fc-…`, the stand-in `resumed c2cc46fc`;
    the row `idle` with Interrupted · Continue under it. `check Codex resumed on a new App Server
    in the repo: pass` (the thread is the first server's, the folder the repository).
  - `750-07-codex-continue` (REQ-006): the TUI read "Continue where you left off." (`cue: Continue
    where you left off.`); the chip gone. `check Codex read the resume prompt: pass`.
  - `750-08-codex-no-server` (REQ-003 setup): with the switch off, a new Codex started as plain
    `codex` (no `--remote`); its row "Codex · waiting".
  - `750-09-codex-off` (REQ-003): after the relaunch that terminal is a plain `repo — bash` shell
    with Marley's prompt; the log's two resumes are both the first thread's (the second a plain
    `codex resume <id>`, the switch being off). `check only the App Server's thread was resumed:
    pass`.
- REQ-007 (Continue while the agent is not idle) by the review: `continue_turn` returns the state
  word for the toast and types nothing.
- **Seen, not in scope:** the New Agent picker's folder question lists the profile's recent
  projects (the runner copies the user's profile); a resumed agent's terminal tab keeps the
  `python3 …` title of the stand-in.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added: Codex resumes, Interrupted · Continue); `docs/marley/guide.md`
  (Starting one: what comes back after a restart, the chip) and the in-app guide (the restart note);
  `docs/marley_architecture/marley_workbench.md` (Agent sessions across a restart, #750's two
  bullets) and `docs/marley_architecture/marley_agent.md` (`resume_line` for Codex,
  `RESUME_PROMPT`); `docs/marley/zed-touchpoints.md`, the rows of `settings_content/src/marley.rs`
  and `settings_ui/src/marley_page.rs` widened.
- **Ledgers:** F-claude-750-a-chip-beside-a-rail-rows-title-took-its-width-001,
  L-claude-750-new-agent-asks-where-when-the-front-terminal-is-elsewhere-001,
  L-claude-750-tee-dev-stderr-truncates-a-redirected-log-001,
  AD-claude-750-a-cut-off-turn-asks-before-it-continues-001.
- **Brain:** consultation 79b9a86c25244ebea7460b0c00b5ca05, `brain decide`
  (`decisions/marley-a-cut-off-agent-turn-asks-before-it-continues-750`).
- **For later:** a resumed agent's shell and a resumed Codex's App Server get none of `agent_env`'s
  variables (the askpass proxy, `GIT_PROMPTS_OFF`); Claude Code's resume has the same gap since #540.
- **Open with Chad:** `marley.codex_app_server` is off in his settings, so his Codex sessions are
  not saved until he turns it on.
- Ticket closed, pipeline archived, committed.
