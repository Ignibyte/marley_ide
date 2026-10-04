---
pipeline_id: 2a881e7d-e981-4f2b-ad77-893f98aedad1
ticket: docs/planning/tickets/closed/TICKET-652-shared-claude-plugin-marleys-half.md
status: Phase 4 — Complete PASS
title: "Agent reports reach Marley through `$MARLEY_BIN report`, and the rail and resume read them first"
type: feature
slice: prong 2 C1 (a terminal's own seat, after #519's hook frames); design note B2, Marley's half; rustal-harness MREQ-009, its D169
references: [docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/520-terminal-identity.spec.md, docs/planning/pipeline/completed/540-session-resume-after-restart.spec.md, docs/planning/pipeline/completed/561-browser-env-opener.spec.md, docs/planning/pipeline/completed/648-agent-version-check.spec.md]
---

## Title
Marley's half of the Claude Code plugin it shares with rustal-harness (B2, the harness's D169),
the state side. Marley answers the harness's MREQ-009: every local terminal names `MARLEY_BIN`, a
small program Marley writes, beside `MARLEY_TERMINAL_ID`, and the shared plugin's mod reports by
running `"$MARLEY_BIN" report` and `"$MARLEY_BIN" release` with the arguments `rh report` and
`rh release` take, so Marley reads the same fields the harness does (its TICKET-099 contract).
Marley knows the reporter by its processes, holds one authority per terminal, and lets the
reports set the terminal's state on the rail and the session a restart resumes. The hook frames
of #519 stay as the fallback and keep carrying the row's details. Loading the shared plugin into
Marley's terminals is decided here and built in the next slice, because it waits on the harness.

## Scope
### In
- **The program.** `crates/marley_workbench/bin/marley-agent`, Python 3's standard library like
  the bridge (#491) and the opener (#561), written at startup into `<data_dir>/mcp/` with a file
  beside it naming this Marley's socket. `marley-agent report STATE --source S --seq N [...]`
  and `marley-agent release --source S` take `rh report`'s and `rh release`'s arguments after
  `--state ROOT`: `--session-id`, `--resume-arg` (repeated, `--resume-arg=VALUE` for a value
  that starts with a hyphen), `--percent`, `--activity`, `--prompt` with `--option` (repeated),
  `--input-tokens` with `--output-tokens` and `--cache-read-tokens`, and
  `--quota KIND:PERCENT_USED[:RESETS_AT_MS]` (repeated). It sends them with its
  `MARLEY_TERMINAL_ID`, prints nothing on success and exits 0; otherwise it exits non-zero with
  the refusal's name and reason on stderr, or 2 for arguments `rh` would refuse to parse.
- **The variable.** A local interactive terminal gives its programs `MARLEY_BIN`, the program's
  path, beside #520's `MARLEY_TERMINAL_ID`; a task, a remote terminal and a terminal of a remote
  project get it empty. One call from Zed's terminal builder, after the #520 hunk, sets it; the
  next slice extends that call inside `marley_terminal` without touching Zed again.
- **The way in.** A Unix stream socket in `marley_mcp` (`agent_socket`, beside the transport): one JSON line in, one out,
  mode 0600 under `$XDG_RUNTIME_DIR/marley/`, named by a digest of Marley's data directory, its
  path checked against `sun_path`. Peer credentials give the reporter's process and user.
- **The checks.** TICKET-099's field rules and refusal names, applied by Marley, not the program
  (`marley_agent::report`).
- **Identity by process.** Marley walks the reporter's parents, at most 32, to the shell of one
  of its local terminals; that terminal must carry the `MARLEY_TERMINAL_ID` the program sends.
  Anything else is `agent_unknown`.
- **Authority.** A terminal's first accepted report makes its source and the reporter's parent
  process (the agent) the terminal's authority, until a release or that process ends. `seq`
  rises per terminal and source. Reports and releases from another source or another process
  are refused.
- **The rail.** While a terminal holds an authority, its seat's state and question are the last
  accepted report's; hook frames fold their labels and move no state, except that a frame's
  permission request or question shows `waiting` over a reported `working` until that call ends
  or the next report. With no authority, the frames move the state as today. The seat carries
  the harness's label names (`source`, `state.source` `reported`, `progress.*`, `usage.*`,
  `quota.*`), so `fleet_snapshot` serves what the harness's fleet serves; the report's session id
  rides as `report.session_id`, since the frames' fold starts a seat over when `session_id`
  changes (amended at promotion). A reported change
  makes the banner (#538) and the push (#535) that a frame's change makes.
- **Resume.** While a terminal holds an authority, a restart resumes the session id it last
  reported, in the agent process's folder, with #540's line. A release while Marley runs drops
  the terminal's saved session; one while Marley quits does not.
- **The reply to MREQ-009,** drafted in the notes: what Marley names and what it needs from the
  harness.
- `script/e2e/652-shared-claude-plugin-marleys-half.sh`, with a stand-in `claude` that acts out
  the mod's reports and the hook frames.

### Out (explicitly deferred)
- **The next slice, loading the shared plugin** (one ticket, M): the harness's three plugin files
  carried at the revision that has Marley's host, written by digest into Marley's data directory,
  put at the front of `CLAUDE_CODE_PLUGIN_DIRS` in local terminals behind
  `marley.claude_code_shared_plugin` (off by default), with #648's row for the mod
  (`claude_shared_plugin`, Claude Code from 2.1.287, open above, as #648's D3 foresees). It waits
  on the harness's reply to MREQ-009 (Marley's host in the mod) and on a license for the files
  that Marley's public repository can carry. D9 and D10 decide it now so the reply can name it.
- **Prompts in** (one ticket): `marley-agent listen` and `delivered`, so a send from Marley's
  rail, rich input or review notes reaches Claude Code through `$.prompt.submit` and an interrupt
  through `$.turn.abort`, retiring the 200 ms paste wait; after TICKET-102's shape.
- **Approvals** (one ticket): `marley-agent approve`, so the approvals inbox answers a call Claude
  Code would put to the user through `tool.check`; after TICKET-101's ask shape, which the harness
  settles first.
- **Retiring the hook path** (one ticket, after parity is shown): `event.py`'s
  `terminalSequence`, the injected-tag table (`UserPromptSubmit`'s `source` field in 2.1.277's
  types may replace it), and Marley's own plugin folded into the shared one (its MCP bridge).
- Drawing progress, usage and quota on a terminal's row: the labels ride on the seat here; the
  drawing reuses #640's reading for these rows in a follow-up.
- Running another agent's reported resume argv (B5); Codex in Marley's terminals (B1).
- A report from a program the rail does not recognize as an agent lands on the seat and in
  `fleet_snapshot` but draws no agent row; a local `tmux` inside a Marley terminal puts the agent
  outside the terminal's processes, so its reports are refused and its frames still count.

## Reference (§20)
N/A — Marley-specific. The report contract is rustal-harness's (TICKET-099, D163), modeled on
herdr's `pane.report_agent`; Marley takes the same reports for its own terminals so one plugin
serves both hosts (D169). Warp's map has no report channel for a CLI agent in its terminal:
third-party harnesses are children that Warp's server dispatches
(`docs/warp_architecture/subsystems/04-agent-ai-mcp.md`, "Multi-agent / orchestration").
Upstream Zed learns an external agent's state through ACP (`agent_servers`) for agents it starts
itself, never for a CLI in a terminal. Marley extends Zed's `terminal` crate where #520 set the
terminal's id and #561 set `BROWSER`, keeping that behavior as it is.

### Prior art
- **Behavior maps.** `docs/orca_architecture/01-agents-and-sessions.md` §2.1 and §2.2: Orca gives
  every PTY identity variables (`ORCA_PANE_KEY` and its hook server's port and token) and its hooks
  post to a loopback listener with a token per start; its hook exits when `CLAUDE_JOB_DIR` is set,
  because a background job inherited the pane's environment (Orca #9236), the case D5's authority
  by process covers. herdr, as the harness's `M13_PLAN.md` and `AGENT_SEATS.md` record it:
  `pane.report_agent`, the first reporter as the authority, `seq` ordering, and
  `validate_resume_argv`'s checks. `docs/warp_architecture/`: nothing for a terminal CLI's state.
  `docs/zed_architecture/`: `agent_servers` speaks ACP only.
- **Published material.** Claude Code's pages as the harness saved them on 2026-10-02
  (`.artifacts/m13/ticket-091-plan-1/sources/`), not re-fetched for this draft:
  `plugins__mods__api.md` (`$.process.run` takes an argument list and uses no shell, so the
  reporter's parent is the Claude Code process), `plugins__mods__events.md` (`tool.check`,
  `classic.*`, a failed hook's fallback), `plugins__loading.md` (an `@inline` plugin from
  `--plugin-dir` or `CLAUDE_CODE_PLUGIN_DIRS` replaces a same-named installed one),
  `env-vars.md` (`CLAUDE_CODE_PLUGIN_DIRS` needs 2.1.280), and the mods types
  (`SessionStartInput.isInteractive`). Linux `unix(7)` (`SO_PEERCRED`) and `proc(5)`
  (`/proc/PID/stat`'s parent and start time).
- **The code we already ship.** rustal-harness is Ignibyte's own and was read and its ideas
  reused, no code carried: the contract's field rules and refusal names (`AGENT_SEATS.md`), the
  arguments `register.js` builds (46-53) and `rh`'s clap definitions of `Report` and `Release`
  (`harness-cli/src/main.rs:277-325`), which the program's parser mirrors; the digest-named
  read-only copy of `claude/plugin.rs` (for the next slice); its finding that a mod's
  `$.mcp.call` waits on a dialog and is refused at `session.end`. Marley's own:
  `marley_workbench::mcp`'s `write_program_in` and the programs it writes into `<data_dir>/mcp/`
  (the bridge, #491; the opener, #561, set by `marley_terminal::shell_integration`'s
  `BROWSER_OPENER` static), the pattern this ticket follows; `marley_mcp::transport` (#491's
  loopback server, #520's `Caller`, set aside, D3); `marley_browser::relay` (#583: a 0600 Unix
  socket, `service::socket_fits`; its hidden mode of the app's executable set aside, D2);
  `marley_workbench::single_instance` (#513: Zed's datagram socket, which answers nothing);
  `claude_plugin.rs` (#482, #547: the marketplace route, set aside for the next slice, D9);
  `claude_events::fold` and `agent_events::on_frame` (#519, #547); `resume.rs` (#540);
  `notifications::on_seat_change` and `push::on_change` (#538, #535). Zed's `terminal`:
  `ProcessIdGetter::fallback_pid` (the PTY's shell) and `Terminal::pid_getter`. `Cargo.lock`:
  `nix` 0.30 with `socket` (`getsockopt` of `PeerCredentials`, already built for Zed's
  `sandbox`), `smol::net::unix` (the relay's listener), `sha2`, `serde_json`. `sysinfo` was read
  for the parent walk and not chosen: its `System` keeps a handle per process (the regression
  test in `pty_info.rs`), where two reads of `/proc/PID/stat` do.

## UI proof
`script/e2e/652-shared-claude-plugin-marleys-half.sh` (`compositor sway`, for the clicks that pick
terminals). Setup: a scratch repository with a folder `sub` and a `.zed/tasks.json` task `env`
that prints `MARLEY_BIN`; a HOME whose `.bashrc` puts a stand-in `claude` first on the PATH, a
Python script that prints its variables, logs its arguments and folder, and on each line `n`
runs the next step of a script the scenario wrote: print a hook frame as #519's plugin would,
run `"$MARLEY_BIN" report ...` or `release` as the mod would and print the exit status, the first
stderr word and the time taken, run one from a child process (not the stand-in), or run a copy of
the program whose socket file names a missing socket; `marley.no_update_after_minutes` 0. The
hook frames name session `hook-…`, the reports session `rep-…`, so each shot shows which source
the row follows. Shots:
- `652-01-environment`: the stand-in's first lines: `MARLEY_BIN` naming `marley-agent` under the
  run's profile copy, and the terminal's id;
- `652-02-task`: the task `env`'s output: `MARLEY_BIN` empty;
- `652-03-reported`: after a report of `idle` and then a prompt frame: the row reads idle with the
  frame's prompt;
- `652-04-waiting`: after a report of `working` and a permission frame: the row waits on
  `Permission for Bash: cargo test`;
- `652-05-interrupted`: after the tool's end frame and a report of `idle` with the activity "the
  last turn was interrupted", and no `Stop` frame: the row reads idle;
- `652-06-refused`: the stand-in's lines for a stale `seq`, a report and a release from a child
  process, an apostrophe in a resume argument, a question without `waiting`, and a missing socket,
  each with its refusal's name; the row still idle;
- `652-07-released`: the release's line, exit 0 and its time; the row back to a plain terminal;
- `652-08-fallback`: a second terminal whose stand-in sends frames only: its row working with the
  frame's tool;
- `652-09-lapsed`: in that terminal, a stand-in whose helper process reported `idle` and exited
  without a release, the stand-in still in front, then the stand-in's frames: the row follows the
  frames again;
- `652-10-resumed`: after a quit and a relaunch: a third terminal, whose stand-in had reported
  `rep-…` and kept running, typed `claude --resume rep-…` in `sub`; the released first terminal is
  a plain shell.

## Locked-In Decisions
- D1 — The variables, Marley's answer to MREQ-009: `MARLEY_TERMINAL_ID` (since #520) and
  `MARLEY_BIN`, the program's path, in every local interactive terminal; `MARLEY_BIN` empty, not
  removed, in a task, a remote terminal and a terminal of a remote project
  (PR-claude-empty-a-variable-the-child-must-not-inherit-001). The mod reports to Marley when
  both are non-empty. `MARLEY_BIN` mirrors `RH_BIN`; Marley needs nothing like `RH_STATE`, since
  the program finds its Marley from where it was written.
- D2 — The command is a program Marley writes, as the opener (#561) is: Python 3's standard
  library, in `<data_dir>/mcp/marley-agent`, reading its socket's path from `agent-socket` beside
  it (PR-claude-a-program-marley-writes-finds-the-marley-that-wrote-it-001). The mod builds
  `[MARLEY_BIN, 'report', ...]` where it builds `[RH_BIN, '--state', ROOT, 'report', ...]`.
  Rejected: a hidden mode of Marley's executable, as the relay is (AD-claude-583), since the
  app's executable would start for each report, several a turn, and a release must end inside
  `session.end`'s one second (#649 turns down `marley edit` for the same cost); a Rust binary of
  its own, which would need building, installing and finding.
- D3 — The way in is a Unix socket of Marley's own, not its MCP server: a mod's `$.mcp.call` waits
  on a permission dialog, reaches only a connected server and is refused at `session.end` (the
  harness's TICKET-100); a program reaching the HTTP server would need the endpoint file's bearer
  and three requests, Marley would learn nothing of the caller's process, and an `agent_report`
  tool there would be listed to every model. Not Zed's single-instance datagram socket: it
  answers nothing, so a refusal could not come back. Under `$XDG_RUNTIME_DIR` (the user's, 0700)
  with a name from the data directory's digest, so a second Marley and an e2e profile have their
  own (PR-claude-check-a-unix-socket-path-against-sun-path-001). In `marley_mcp::transport`,
  where §14 keeps sockets.
- D4 — The caller is known by its processes, never by what it sends, as in the harness: the
  peer's uid must be Marley's, and its parent chain must meet a local terminal's shell (the PTY's
  child). The `MARLEY_TERMINAL_ID` the program sends is a check on that, not the key.
- D5 — Authority is the source and the reporter's parent process, kept by pid and start time so a
  reused pid is not mistaken for it. A `claude -p` that a session runs in the same terminal would
  load the same plugin and report with the same source; its parent differs, so it is refused.
  The authority lapses with its process, checked at each report, at each hook frame, and when the
  terminal's foreground changes (#547). `seq` is per terminal and source for Marley's run; the
  mod's starts at the wall clock's milliseconds, so it rises across restarts.
- D6 — Precedence on the seat: the report sets the state and the question; the hook frames keep
  every label they set today (prompt, tool, message, error, subagents, the turn's facts for the
  stop kind) and set no state, with one exception, `waiting` from a frame over a reported
  `working`, because the mod leaves Claude Code's own dialog alone under Marley's host in this
  slice and would never report it. No authority: the frames as today. This is the fallback until
  the parity ticket retires the frames.
- D7 — Resume follows the report: the session id it last reported (it follows `/clear` and
  `/resume` at the next turn), in the agent process's working directory (`/proc/PID/cwd`), with
  #540's line and launch mode. The reported argv is kept on the seat and not run: for Claude Code
  Marley rebuilds its own line, as the harness rebuilds an interface's.
- D8 — The fields, limits and refusal names are TICKET-099's, letter for letter, so one mod and
  one wrapper work under either host: the same `source` charset, `seq` rule, six states, question,
  progress, session id, herdr's argv checks, usage and quota bounds. A name only Marley has:
  `marley_not_running`, when the socket does not answer. The program waits at most 900 ms for a
  release and 4 s for a report, inside the mod's 1 s and 5 s budgets; Marley reads the process
  chain off the main thread, and only the match and the fold run on it.
- D9 — The next slice loads the plugin through `CLAUDE_CODE_PLUGIN_DIRS`, set by the same call as
  D1, at the front of the inherited value with any other Marley copy removed. Not `--plugin-dir`:
  only sessions Marley launches would get it, not a `claude` the user types. Not the marketplace:
  the mod would load in every session on the machine, cached per version, checked through
  `installed_plugins.json` (F-claude-547). The no-double-load rule holds: a harness seat runs
  under `env -i` with its own `--plugin-dir`, so no Marley variable reaches it; an installed copy
  of the same plugin is replaced by the session-only one of the same manifest name; Marley's own
  `marley@marley` is another plugin and stays for its MCP bridge and the frames.
- D10 — The next slice carries the files byte for byte at a recorded revision (the checkout here
  has no git metadata, so a tag, a commit on its origin, or the files' digest as `rh` computes
  it), writes them into `<data_dir>/claude-code/shared/<digest>/` with files 0400 and folders
  0700 as `rh` does, and turns them on with `marley.claude_code_shared_plugin`, off by default:
  it puts a second plugin with a mod into every Claude Code session in Marley's terminals before
  parity with the frames is shown. #648's table gets the row `claude_shared_plugin`, Claude Code
  from 2.1.287 and open above; outside it, #648's chip says why.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a local interactive terminal starts, the system shall give its programs `MARLEY_BIN`, naming the `marley-agent` program in the running Marley's own data directory. | Shot `652-01-environment` |
| REQ-002 | WHERE a terminal is a task, a remote terminal or a terminal of a remote project, the system shall give its programs `MARLEY_BIN` empty. | Shot `652-02-task`; review |
| REQ-003 | WHEN a program in a Marley terminal runs `"$MARLEY_BIN" report` with arguments `rh report` accepts after `--state ROOT`, the system shall take the report for that terminal and the program shall exit 0. | Shot `652-03-reported` |
| REQ-004 | WHILE a terminal holds a report authority, the rail shall show the last accepted report's state on its row, whatever state the hook frames give. | Shot `652-03-reported` |
| REQ-005 | WHILE a terminal holds a report authority, the rail shall keep showing the prompt and the tool in flight that its hook frames carry. | Shots `652-03-reported`, `652-04-waiting` |
| REQ-006 | WHILE a terminal's last report is `working`, WHEN a hook frame says Claude Code asks a permission, the rail shall show the row waiting on it until that call ends or the next report comes. | Shot `652-04-waiting` |
| REQ-007 | WHEN a report says `idle` while the hook frames left the row working, the rail shall show the row idle within one second. | Shot `652-05-interrupted` |
| REQ-008 | IF a report's `seq` is not higher than the last one accepted from its source for that terminal, THEN the system shall refuse it as `agent_report_stale` and leave the row as it was. | Shot `652-06-refused` |
| REQ-009 | IF a report or a release comes from a process whose parent is not the authority's while the authority's process lives, THEN the system shall refuse it as `agent_report_authority` or `agent_release_authority`. | Shot `652-06-refused` |
| REQ-010 | IF the reporting process runs in none of Marley's local terminals, or the `MARLEY_TERMINAL_ID` it sends is not that terminal's, THEN the system shall refuse it as `agent_unknown`. | Review; the scenario's `expect` on a report run from outside Marley |
| REQ-011 | IF a report breaks one of TICKET-099's field rules, THEN the system shall refuse it with the harness's name for that rule and change nothing. | Shot `652-06-refused` (`agent_report_argv`, `agent_report_question`); review for the rest |
| REQ-012 | IF the Marley that wrote the program does not answer on its socket, THEN the program shall exit non-zero within one second with `marley_not_running` on stderr. | Shot `652-06-refused` |
| REQ-013 | WHEN the authority's source runs `"$MARLEY_BIN" release`, the system shall end the terminal's seat, and the program shall exit 0 within one second. | Shot `652-07-released` |
| REQ-014 | WHILE a terminal holds no report authority, the rail shall move its row by the hook frames as before this change. | Shot `652-08-fallback` |
| REQ-015 | WHEN the authority's process ends without a release, the system shall drop the authority, so the next hook frames move the row. | Shot `652-09-lapsed` |
| REQ-016 | WHEN Marley restarts with a terminal whose authority reported a session id, the system shall resume that session in the agent's folder, not the session a hook frame named. | Shot `652-10-resumed`; the scenario's `expect` on the stand-in's log |
| REQ-017 | WHEN the authority releases while Marley runs, the system shall drop the terminal's saved session, so the next launch resumes nothing there. | Shot `652-10-resumed` |
| REQ-018 | WHEN a report is accepted, the system shall serve the terminal's seat in `fleet_snapshot` with `source`, `state.source` `reported`, `report.session_id`, and the report's progress, usage and quota under the harness's label names. | The scenario's `expect` on `fleet_snapshot`; review |
| REQ-019 | WHEN a report moves a terminal the user is not looking at to idle, waiting or failed, the system shall show the banner and send the push that the same move from a hook frame gives. | Review |
| REQ-020 | The socket shall take connections only from the user Marley runs as, at mode 0600, at a path checked against `sun_path` before it binds. | Review |
| REQ-021 | The program shall use only Python 3's standard library and shall reach the Marley whose data directory it was written to, never a default path. | Shot `652-01-environment` (the profile copy's path); review |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes. At promotion: re-read the harness's
  `AGENT_SEATS.md` and `register.js` for any change to the fields or arguments; see whether the
  harness answered MREQ-009 and send the reply in the notes if not; `brain_ask`.
- **P2 Code** — `marley_agent::report` (the contract's rules and the wire), the socket in
  `marley_mcp::transport`, the program `bin/marley-agent`, `marley_workbench::agent_reports` (the
  program and its socket file written, the intake, the process walk, the authority), the
  precedence in `agent_events.rs`, resume in `resume.rs`, the variable in
  `marley_terminal::identity`; the row of `docs/marley/zed-touchpoints.md` for
  `crates/terminal/src/terminal.rs` extended before that edit; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario with its stand-in `claude`, read
  every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21: `docs/marley_architecture/` for
  `marley_agent`, `marley_mcp`, `marley_terminal` and `marley_workbench`, the plan's C1 row, the
  touchpoint row checked), ledger capture (§19), the reply to MREQ-009 recorded as sent, close
  the ticket, archive, commit.
