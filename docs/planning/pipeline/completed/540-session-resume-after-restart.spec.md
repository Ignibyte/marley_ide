---
pipeline_id: 4ab49a90-0f86-4ebc-9814-ae748130e35a
ticket: docs/planning/tickets/closed/TICKET-540-session-resume-after-restart.md
status: Phase 4 — Complete PASS
title: "Claude Code sessions resumed after a Marley restart"
type: feature
slice: prong 2, sessions (report 05 §3 item 3, report 01 §3 item 7)
references: [docs/planning/tickets/closed/TICKET-540-session-resume-after-restart.md, docs/orca_architecture/05-terminal-and-workspace.md, docs/orca_architecture/01-agents-and-sessions.md, docs/marley/three-prong-plan.md]
---

## Title
After a Marley relaunch, a terminal that was running a Claude Code session at the quit types
`claude --resume <id>` in the folder the session started in, with the id #519's `SessionStart`
reported, so the conversation comes back. A session that ended before the quit comes back as a
plain shell, as today. Chad picked it on 2026-10-01 ("lets do 637, 631, 540"). The harness
question it waited on does not touch it: Marley's own terminals are Zed terminals, which a quit
ends, and the embedded harness (#632) keeps only its own sessions alive.

## Scope
### In
- The session per terminal, from the `SessionStart` and `SessionEnd` frames #519's
  `agent_events` receives (`session_id`, `cwd`, `source`, `reason`), the lead's only:
  - `SessionStart` with source `startup`, `clear`, `fork` or `resume`: the terminal's session is
    the event's id, its folder the event's `cwd`; `compact` changes nothing;
  - `SessionEnd` with reason `prompt_input_exit` or `logout`: the session is dropped; `clear` and
    `resume` wait for the `SessionStart` that follows; `other` keeps it, as a quit reads;
  - the rail ending the seat because Claude Code left the foreground without a `SessionEnd`
    (#547) drops it too, unless Marley is quitting.
- Kept in a table of Marley's own, `marley_agent_sessions(terminal_id, session_id, folder)`,
  keyed by the terminal's `MARLEY_TERMINAL_ID`, which a restored terminal keeps (#575). Rows for
  a terminal id no saved terminal holds any more go at start.
- At relaunch: a new terminal view whose terminal holds a saved session, with the setting on,
  types `cd <folder> && claude --resume <id>` and Enter once its shell is ready, through the
  same startup handshake an agent's launch uses, with the arguments Claude Code's permission
  setting asks for in the project (#532).
- The id is typed only in the 8-4-4-4-12 hexadecimal form, and the folder only as one quoted
  argument (`marley_agent::quote_argument`).
- One session resumes in one terminal per launch; another terminal holding the same id comes
  back as a plain shell.
- `HookEvent` gains `reason`, which the plugin already sends.
- `marley.resume_agents` (default on) in `MarleySettingsContent`, `default.json` and the Agents
  section of the Marley page.

### Out (explicitly deferred)
- Codex and the other agents: #519 carries Claude Code's hooks only.
- The scrollback: Zed restores an empty terminal, and the resumed Claude Code draws its own
  conversation.
- A history of past sessions per project with Resume (report 01 §3 item 7's history half).
- Remote terminals: their sessions live on the host.
- The harness's sessions (#534, #632): the harness keeps those processes alive.
- The agent terminal's environment (#537's git prompts off, #596's ssh passphrase proxy): a
  restored terminal is a plain one, so the resumed Claude Code runs with the shell's own.

## Reference (§20)
Upstream Zed (`terminal_view`): `TerminalView::deserialize` brings a terminal back as a new shell
at its saved working directory, and Marley's identity hook (#575) gives it the id it had; Marley
keeps both and adds the agent's session, keyed by that id. Warp's Session Restoration restores
"your windows, tabs, panes, and recent Blocks automatically when you relaunch Warp" and says
nothing of the processes in them (docs.warp.dev/terminal/sessions/session-restoration). The
behavior Marley matches is Orca's cold restore (report 05 §2.2): a pane whose agent's session id
its hooks reported is relaunched with `claude --resume <id>` (`getAgentResumeArgv`,
`src/shared/agent-session-resume.ts`), in the folder the session started in, since
`claude --resume` finds sessions by that folder (report 01 §2.6, Orca #9361).

### Prior art
- **Behavior maps.** Report 05 §2.2 (Orca's cold restore, resume ids saved as they come), §2.8,
  §3 item 3 (never resume one session twice); report 01 §2.6 (a live pane's session id comes only
  from its hooks) and §3 item 7.
- **Published material.** Claude Code's CLI reference: `--resume` resumes a session by id and
  keeps its id; its hooks reference: `SessionStart`'s sources (`startup`, `resume`, `clear`,
  `compact`) and `SessionEnd`'s reasons (`clear`, `logout`, `prompt_input_exit`, `other`), with
  `session_id` and `cwd` on every hook.
- **Code we already ship.** #575's `terminal_ids.rs` (a Marley table that follows Zed's restore,
  the id the restored terminal keeps): the key this ticket needs, so no Zed table changes.
  `agents.rs::start_in_terminal` (the startup handshake, then `write_init_command_after_startup`,
  which types only into a terminal that took no input); `marley_agent::launch_line_after` and
  `quote_argument` (one quoted word for bash, zsh and fish); `agent_events::on_frame` and `end`
  (#519, #547); `marley_terminal::identity::is_terminal_id`.
  The queued draft (2026-09-25) put the session in two new columns of Zed's `terminals` table;
  #575 shipped after it and made that unnecessary.

## UI proof
`script/e2e/540-session-resume-after-restart.sh`, `compositor sway` (a click focuses the
terminal; `quit_marley` and `launch_marley` relaunch on the same profile). A fake `claude` first
on the terminal's PATH (a Python script, which the rail names by its file) prints #519's
`SessionStart` frame for the id in a file the scenario writes, with its folder and source, prints
`session <id> in <folder>` (`resumed <id> in <folder>` when started with `--resume`), logs its
argument list and folder to a log, and on the line `exit` prints `SessionEnd`
(`prompt_input_exit`) and ends. Shots: `540-01-running`, `540-02-resumed`, `540-03-again`,
`540-04-exited`, `540-05-off`.

## Locked-In Decisions
- D1 — The session rides Marley's own table keyed by the terminal's `MARLEY_TERMINAL_ID`, which
  #575 keeps across restores; no Zed file changes but the setting's.
- D2 — The id comes only from #519's `SessionStart`, never from `~/.claude/projects` or the screen.
- D3 — The resumed command goes to the session's folder first (`cd -- <folder> &&`), the folder
  as one quoted word and the id checked, since `claude --resume` finds a session by its folder
  and the restored shell may start elsewhere (#577).
- D4 — `SessionEnd` with `other` keeps the session (a quit closes the terminal under Claude
  Code); an exit the user makes (`prompt_input_exit`, `logout`) drops it, and so does Claude Code
  leaving the foreground without one while Marley runs.
- D5 — One session, one resume per launch.
- D6 — On by default, `marley.resume_agents` to turn it off.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley relaunches and a restored terminal was running a Claude Code session at the quit, the system shall type `claude --resume <id>` into it once its shell is ready | shot `540-02-resumed`; the fake's log |
| REQ-002 | WHEN such a terminal resumes, the command shall run in the session's first folder | the fake's log (`<repo>/sub`); shot `540-02-resumed` |
| REQ-003 | WHEN Marley relaunches again, a resumed session shall be resumed again | shot `540-03-again`; the log |
| REQ-004 | WHEN a terminal's Claude Code exited before the quit (`prompt_input_exit`), the system shall restore a plain shell and type nothing | shot `540-04-exited`; no new log entry |
| REQ-005 | WHERE `marley.resume_agents` is false, the system shall resume nothing | shot `540-05-off`; no new log entry |
| REQ-006 | WHEN two restored terminals hold the same session id, the system shall resume it in one only | review of the diff (the per-launch claim) |
| REQ-007 | WHEN a saved id is not a UUID, or the folder holds shell syntax, the system shall type no command built from it | review of the diff (`marley_agent::resume_line`) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `HookEvent::reason`; `marley_agent::resume_line`; `resume.rs` (the table, the
  bookkeeping, the resume); the hooks in `agent_events`; the setting; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the touchpoint rows
  checked, ledger capture, close the ticket, archive, commit.
