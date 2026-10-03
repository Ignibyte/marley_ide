# Agent reports reach Marley through `$MARLEY_BIN report`, and the rail and resume read them first — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-652-shared-claude-plugin-marleys-half.md
- **Pipeline spec:** 652-shared-claude-plugin-marleys-half.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** the brainstorm's B2
  (`design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md`): one Claude Code plugin
  with a mod, shared with rustal-harness, Marley's plugin folded in. Chad's words: "we need to
  brain storm integration into claude and codex using their tools instead of fighting them", and
  for B2 "Yes, with the harness". The harness took it as D169: its M13 plugin
  (`crates/harness-runtime/src/claude/plugin/`) is the shared one, and its MREQ-009
  (`docs/planning/MARLEY_REQUESTS.md`) asks Marley to "name the variable its launcher sets for a
  Claude session it starts, and how the mod reaches Marley: the program it runs or the server it
  calls, and what each carries". The batch's lead asked for Marley's half, the state side: the
  answer to MREQ-009, the shared plugin loaded in Marley's terminals, and the rail and resume
  reading the mod's reports first with the OSC 777 path as the fallback.
- **Classification / tier:** feature, prong 2 C1 (a terminal's own seat), L as the note sized
  B2, split: this slice is M to L (the contract, the program, the socket, the authority, the rail
  and resume); loading the plugin is the next slice (Out), since it waits on the harness. Marley
  crates `marley_agent`, `marley_mcp`, `marley_terminal`, `marley_workbench`; one Zed crate,
  `terminal`, on its existing touchpoint row; no setting in this slice, since nothing reports
  until the next slice or a user's wrapper does.
- **Recall (§18.3):**
  - AD-claude-519-claude-codes-hook-events-ride-in-band-into-marley-fleet-001: the hook frames,
    the seat per terminal view in `marley_fleet`, the pure fold. It rejected "a POST to
    `marley_mcp` from the hook (an endpoint file and a bearer in every terminal, and no route from
    a remote host)". This ticket keeps the frames for remote terminals and for the labels, and the
    socket needs no bearer: the peer's credentials and its processes stand in for one (D3, D4).
  - AD-claude-482-claude-code-sends-marleys-notifications-through-a-plugin-001 called
    `terminalSequence` undocumented; #648's draft (2026-10-03) found Claude Code's hooks reference
    now documents it. The frames stay the fallback here for another reason: they carry no
    identity by process and no contract the harness shares.
  - AD-claude-520-each-terminal-names-itself-and-the-bridge-names-the-caller-001 and
    F-claude-520-a-key-removed-from-the-builders-map-still-reached-the-program-001 with
    PR-claude-empty-a-variable-the-child-must-not-inherit-001: the new variable goes beside the id
    and is emptied, never removed, where it names nothing (D1).
  - AD-claude-561-marley-exports-its-opener-as-browser-in-every-local-terminal-001,
    F-claude-561-an-opener-on-the-default-endpoint-would-open-tabs-in-another-marley-001 and
    PR-claude-a-program-marley-writes-finds-the-marley-that-wrote-it-001: a program Marley writes
    for terminals finds its Marley from a file beside it, so an e2e run never reaches the user's
    Marley (D2).
  - AD-claude-583-chromium-on-its-pipe-behind-marleys-relay-001: a hidden mode of the app's
    executable and a 0600 Unix socket. The socket is taken; the hidden mode is set aside for the
    cost of starting the app for each report (D2), as #649's draft sets aside `marley edit`.
  - AD-claude-513-one-marley-per-data-directory-001,
    F-claude-513-a-socket-path-too-long-read-as-a-running-marley-001 and
    PR-claude-check-a-unix-socket-path-against-sun-path-001: a socket under a profile copy's
    folder outgrew `sun_path`; this one lives under `$XDG_RUNTIME_DIR` and is checked first.
  - AD-claude-540-a-claude-code-session-resumes-by-the-terminals-id-001: the table, the line, why
    `other` keeps a row and a quit does not drop it; D7 changes only where the id comes from.
  - F-claude-547-a-scenarios-click-ran-the-real-claude-001 and
    PR-claude-name-the-fakes-the-app-runs-001: the stand-in `claude` is typed in a terminal, so
    the PATH of the scenario's `.bashrc` finds it; nothing here makes Marley start `claude` itself.
  - L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001 and the completed
    pipelines 482, 519, 520, 543 ran a real Claude Code with `--plugin-dir` for a live check. This
    batch's rule is no real turn, so the mod's own runs stay the harness's tests; Marley's
    scenario acts the mod out with a stand-in.
  - The completed pipelines 519 and 547 (the frames, the seat, `fleet_snapshot`, the foreground
    end), 540 (resume and its scenario's relaunch), 520 and 575 (the id), 561 (the opener beside
    the id), 583 (the relay), 538 and 535 (banner and push from a seat change). The queued #648
    (the version table this ticket's next slice adds a row to) and #649 (its editor helper, the
    same kind of program).
  - Brain: `rusty-cli brain ask` on the plugin's loading and the mod's reports surfaced only
    unrelated follow-ups, and `brain search` found no page; the Planner's `brain_ask` at promotion
    is owed.
- **Discovery (direct reads; every path below was opened):**
  - The harness's mod, `crates/harness-runtime/src/claude/plugin/register.js` (rustal-harness,
    read only): `report(root, state, id)` builds `['--state', root, 'report', '--source', SOURCE,
    '--seq', seq, state]` plus `--session-id` and three `--resume-arg=` elements (46-53);
    `session.start` reads `RH_BIN` and `RH_STATE`, reports `idle`, and spawns `rh listen` only
    when both are set (55-108); `turn.start` reports `working` (110-124); `turn.complete` reports
    `idle` or `error`, the activity "the last turn was interrupted", usage and quota (127-159);
    `tool.check` returns Claude Code's verdict untouched when the variables are missing (164-179);
    `session.end` releases except for `clear` and `resume` (181-192). `hooks.json` holds only
    `"modules": ["./register.js"]`; `plugin.json` names the plugin `rustal-harness`. `plugin.rs`
    writes the copy at `claude-plugin/<16 hex of the digest>/`, files 0400, folders 0700, checked
    at each open (38-123).
  - The contract: `docs/AGENT_SEATS.md` (fields 32-44, refusals 54-64, identity by the caller's
    parents 48-52, the seat's labels 66-88); `crates/harness-cli/src/main.rs:277-325`, the clap
    definitions of `Report` and `Release` (`--prompt` requires `--option`, the three token counts
    require one another, `--resume-arg` allows hyphen values), which the program mirrors.
  - `docs/CLAUDE_CODE.md` "In a harness terminal" and "The plugin": the seat runs under
    `/usr/bin/env -i` with `RH_STATE`, `RH_BIN` and `RH_WORKSPACE_UUID` named, the pinned
    Claude Code with `--plugin-dir`; why the mod runs `rh report` rather than the MCP server.
    `docs/M13_PLAN.md` "One plugin, two hosts" (81-105); D169 in `docs/DECISIONS.md:129-141`.
  - `crates/marley_workbench/src/mcp.rs:305-360`: the opener's constant (`include_str!` of
    `bin/marley-open-url`), `offer_browser_opener` (the path set at once, the file written off the
    main thread), `give_browser_opener`, and `write_program_in(data_dir, name, contents)`, which
    writes a program 0755 into `<data_dir>/mcp/`. `crates/marley_workbench/bin/marley-open-url`:
    Python 3's standard library, its endpoint found beside it.
  - `crates/terminal/src/terminal.rs`: `marley_browser_opener` read before the builder's future
    (1245); the #520 hunk (1307-1333) that sets `MARLEY_TERMINAL_ID`, empties both variables for a
    task or remote terminal and takes the restored id; `BROWSER` (1335-1343). `Terminal::pid`
    (3627) is the PTY's foreground group, `pid_getter` (3634) gives `ProcessIdGetter`, whose
    `fallback_pid` is the shell (`crates/terminal/src/pty_info.rs:26-47`).
  - `crates/marley_terminal/src/identity.rs:10-32` (the variables) and
    `crates/marley_terminal/src/shell_integration.rs:145-160` (`BROWSER_OPENER`, a static the
    workbench sets, which the builder reads): the pattern for D1's value.
  - `crates/marley_mcp/src/transport.rs`: the loopback server binds `127.0.0.1:0` (187), threads
    per connection, a bearer on every request; `marley_mcp.rs:150-166`, `Caller`, "never an
    authority". `crates/marley_browser/src/relay.rs:300-311`, `bind_unix` at 0600;
    `service.rs:279-295`, `relay_socket_in` and `socket_fits`. `crates/zed/src/main.rs:217-221`,
    the relay's hidden mode before `Args::parse`, read and not used.
  - `crates/marley_workbench/src/agent_events.rs`: `on_frame` (132-182) decodes, runs
    `resume::on_event`, folds with `claude_events::fold`, applies, then the stall watch and
    `after_fold`; `end` (508-534) ends seats whose Claude Code left the foreground and calls
    `resume::ended`; `forget` (537). `notifications.rs:103-124`: a frame's change goes to
    `on_seat_change` (banner, #538) and `push::on_change` (#535), not for a `SessionStart`.
  - `crates/marley_agent/src/claude_events.rs`: `fold` (215-246), `Moving::note_session`
    (297-320, a new session id starts the seat over), `take` (322 on: `SessionStart`,
    `UserPromptSubmit`, the tools, `PermissionRequest`'s wait, `Stop`, `StopFailure`).
  - `crates/marley_workbench/src/resume.rs`: `on_event` (104-129) saves from `SessionStart` and
    drops on `prompt_input_exit` or `logout`; `ended` (133-147) drops unless quitting;
    `resume_restored` (188-235) types `marley_agent::resume_line`.
  - `crates/marley_workbench/claude_plugin/marley/hooks/event.py:136-151`: `TERM_PROGRAM` or
    `MARLEY_REMOTE`, then the frame; `claude_plugin.rs:20-60` and `181-208`: the marketplace and
    the read of `installed_plugins.json`, the route D9 sets aside.
  - `.config/spawn-sites.txt`: no line is needed; Marley starts no process here, the program only
    connects.
  - `script/e2e.sh` keeps a runner variable named `MARLEY_BIN` (68-69, 604, 716), assigned at its
    top and never exported by it; `script/e2e/540-session-resume-after-restart.sh` (the stand-in
    and the relaunch); `script/e2e/browser-fixture.sh:149-159, 791-803` (`mcp_agent fleet`).
  - `docs/planning/pipeline/queued/648-agent-version-check.spec.md`: `marley_agent::versions`,
    rows of `id`, agent and a tested range; its D3 expects "B2's open from 2.1.287".
    `649-rich-input-through-the-agents-editor-key.spec.md` D1: a program in the data directory,
    not a `marley edit` subcommand, since "a subcommand would start the whole app binary for every
    edit".
  - On the box, 2026-10-03: `claude --version` reads 2.1.288, one past the harness's pin.
- **Decisions:** D1 to D10 in the spec. Rejected along the way: a hidden mode of the app's
  executable and a Rust binary of its own (D2), Marley's MCP server as the way in and the
  single-instance datagram socket (D3), the terminal named by the variable alone (D4), authority
  by source alone as the harness has it (a nested `claude -p` shares the source, D5), running the
  reported argv on resume (D7), `--plugin-dir` and the marketplace for the next slice (D9), and
  one ticket for both halves (the loading waits on the harness).

### Design
- **Approach.** Marley grows the harness's report contract for its own terminals. The mod, or any
  wrapper, runs `"$MARLEY_BIN" report ...`; the program turns the arguments into TICKET-099's
  JSON and sends one line over the socket its `agent-socket` file names; Marley reads the peer's
  credentials, walks its parents to a terminal, checks the authority and the fields, applies the
  report to the terminal's seat and its saved session, and answers one line. The hook frames keep
  folding, under D6's precedence.
- **The program.** `bin/marley-agent`, Python 3, standard library only. `argparse` with two
  subcommands mirroring `rh`'s clap definitions; a value that starts with a hyphen comes as
  `--resume-arg=VALUE`, which the mod already writes. It reads `MARLEY_TERMINAL_ID` from its
  environment and the socket's path from `agent-socket` beside its own file (`__file__`), connects
  with a deadline (900 ms for `release`, 4 s for `report`), writes the line, reads the answer, and
  exits 0 on `{"ok": true}`, 1 with `<name>: <reason>` on stderr on a refusal or with
  `marley_not_running` when the socket is missing or does not answer, and 2 on a usage error. It
  checks no field rule: Marley does, so the two cannot disagree.
- **The wire.** Request: `{"verb": "report", "terminal": "<MARLEY_TERMINAL_ID>", "report":
  {source, seq, state, question?, progress?, session_id?, resume_argv?, usage?, quota?}}` (the
  report object is TICKET-099's JSON, the shape `agent_report` takes in the harness's MCP), or
  `{"verb": "release", "terminal": ..., "source": ...}`. Answer: `{"ok": true}` or
  `{"refused": "<name>", "reason": "<text>"}`. At most 64 KiB in, a 2 s read timeout, one request
  per connection; later verbs (`listen`, `approve`) may keep the connection open.
- **The process walk.** Off the main thread, the socket's thread reads the peer's chain from
  `/proc/PID/stat` (the fields after the last `)`: the parent at index 1 and the start time at
  index 19), up to 32 entries. On the main thread, the chain is matched against each local
  terminal's shell pid (`pid_getter().fallback_pid()`), nearest first; the entry right after the
  reporter is the agent, kept with its start time as the authority. Linux only; elsewhere every
  report is `agent_unknown`.
- **The seat.** `AgentEvents` keeps an `Authority { source, agent_pid, agent_started, seq,
  reported: State, question, session_id, resume_argv }` per terminal view, outside the fleet
  snapshot, and writes the labels `source`, `state.source` (`reported`), `session_id`,
  `progress.percent`, `progress.activity`, `usage.input_tokens`, `usage.output_tokens`,
  `usage.cache_read_tokens`, `quota.KIND.percent_used`, `quota.KIND.resets_at_ms` onto the seat.
  A report upserts the seat (creating it if no frame came yet), raises or clears its question,
  and sets its state. `on_frame`, after `claude_events::fold`, checks that the authority's
  process still lives and then replaces the folded state with the authority's, except a folded
  `waiting` over a reported `working`. A release ends the seat (`SessionEvent::Ended`) and drops
  the authority; `forget` and `end` drop it too.
- **File manifest.**
  - `crates/marley_workbench/bin/marley-agent` (Marley, new): the program.
  - `crates/marley_agent/src/report.rs` (Marley, new): the report, its field rules and refusal
    names, the wire's serde types, herdr's argv check, the `/proc/PID/stat` reading.
    `marley_agent.rs` declares the module.
  - `crates/marley_mcp/src/transport.rs` (Marley): `AgentSocket` (bind at 0600 after the
    `sun_path` check, accept loop, peer credentials through `nix`, one bounded line, the chain
    read, a channel to the app). `crates/marley_mcp/Cargo.toml`: `nix` from the workspace with
    `socket`, already in `Cargo.lock`.
  - `crates/marley_workbench/src/agent_reports.rs` (Marley, new): at init, the socket's path from
    the data directory's digest, the socket started, the program and `agent-socket` written with
    `mcp::write_program_in` (made `pub(crate)`), the program's path handed to `marley_terminal`;
    the intake on the main thread (match, authority, `seq`, apply). `marley_workbench.rs`: the
    module and `agent_reports::init(cx)`.
  - `crates/marley_workbench/src/agent_events.rs` (Marley): the authority per view, the
    precedence in `on_frame`, `apply_report`, the drops in `forget` and `end`.
  - `crates/marley_workbench/src/notifications.rs` and `push.rs` (Marley): the banner and push
    reached from a reported change, with the view and its window.
  - `crates/marley_workbench/src/resume.rs` (Marley): `on_report(terminal, session, folder)`,
    `on_release(terminal)`.
  - `crates/marley_terminal/src/identity.rs` (Marley): `BIN_VARIABLE`, a static set by the
    workbench as `BROWSER_OPENER` is, and `agent_environment(env, program, local_interactive)`,
    which the next slice extends with `CLAUDE_CODE_PLUGIN_DIRS`.
  - `crates/terminal/src/terminal.rs` (Zed, `terminal`): in `TerminalBuilder::new`, the program's
    path read before the future as `marley_browser_opener` is, and one call to
    `marley_terminal::identity::agent_environment` after the #520 hunk. Extends the row for
    `crates/terminal/src/terminal.rs`: "after the id hunk, `MARLEY_BIN` from
    `marley_terminal::identity::agent_environment`, the path of Marley's `marley-agent` for a
    local interactive terminal and empty for any other, read before the future (#652)".
  - `script/e2e/652-shared-claude-plugin-marleys-half.sh` (Test phase); `browser-fixture.sh`'s
    `fleet` printer gains `source` and `state.source` if the Test needs them.
- **Not touched:** `crates/zed/src/main.rs`, `claude_plugin/marley/` (the frames stay as they
  are), `.config/spawn-sites.txt`, `marley_fleet` (labels only), `settings_content` (no setting
  this slice).

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-021 | terminal A in `sub` runs the stand-in, whose first lines print `MARLEY_BIN` and `MARLEY_TERMINAL_ID` | `652-01-environment` (the path under the run's profile copy) |
| REQ-002 | the palette's `task: spawn`, task `env` | `652-02-task` (`MARLEY_BIN=[]`) |
| REQ-003, REQ-004, REQ-005 | step 1: a `SessionStart` frame for `hook-1`, a report `idle` for `rep-2` (exit 0); step 2: a `UserPromptSubmit` frame "Add a README", no report | `652-03-reported` (row `idle · Add a README`) |
| REQ-005, REQ-006 | step 3: a report `working`, a `PreToolUse` frame `Bash: cargo test`, a `PermissionRequest` frame for it | `652-04-waiting` |
| REQ-007 | step 4: the `PostToolUse` frame, then a report `idle` with the activity, no `Stop` frame; settle 1 | `652-05-interrupted` |
| REQ-008, REQ-009, REQ-011, REQ-012 | step 5: an older `seq`; a report and a release from a child `sh`; `--resume-arg="it's"`; `idle --prompt Go? --option yes`; a copy of the program in `$E2E_WORK/lost/` whose `agent-socket` names a missing path, timed | `652-06-refused` (each name; row still idle) |
| REQ-010 | the runner, outside Marley, runs `$E2E_PROFILE/mcp/marley-agent report ...` with the id the stand-in wrote to `$E2E_WORK/env.txt`; `expect` reads `agent_unknown` | none: a machine check and the review |
| REQ-013 | step 6: the release, timed; the stand-in ends | `652-07-released` (exit 0, under 1000 ms; a plain terminal row) |
| REQ-014 | terminal B: a frames-only stand-in (`SessionStart`, `UserPromptSubmit`, `PreToolUse`) | `652-08-fallback` (row working with the frame's tool) |
| REQ-015 | in B: the stand-in starts a helper process that reports `idle` and exits without a release; the stand-in stays in front and then prints a `UserPromptSubmit` and a `PreToolUse` frame | `652-09-lapsed` (row working with the frame's tool, not the helper's `idle`) |
| REQ-016, REQ-017 | terminal C in `sub`: a `SessionStart` frame for `hook-5`, a report for `rep-6`, kept running; quit and relaunch | `652-10-resumed`; `expect` that the log's last line is `--resume rep-6 @ …/sub`; A resumes nothing |
| REQ-018 | after steps 1 and 4, `mcp_agent fleet` reads A's seat | none: `expect` on `source mod:claude-code` and `state.source reported` |
| REQ-019, REQ-020 | the banner and push called from the report path; the socket's mode, uid check and length check | the diff |

The rows' places come from the first shots, as #540's did. What no scenario can reach: the
harness's real mod in a real Claude Code, since the batch runs no real turn and the mod has no
Marley host until the harness answers MREQ-009; the stand-in runs the same argv the mod will, and
the Phase 3 entry says so.

### Risks
- **Python 3 on the mod's PATH.** `$.process.run` takes argv with no shell; the program starts
  through `#!/usr/bin/env python3`, as `event.py` and the bridge already do in the same sessions.
  A Claude Code started with a PATH without `python3` loses the reports and keeps the frames, the
  same failure the frames already have.
- **The agent's parent.** D5 rests on `$.process.run` starting the program from the Claude Code
  process with no shell (the mods API page). If a future Claude Code runs mods in a helper
  process, the parent is that helper; it is still one process per session, so the rule holds, but
  the promotion re-reads the types of the version installed.
- **`/cd` and the folder.** D7 reads the agent process's working directory. If Claude Code's
  `/cd` moves the session without a `chdir`, a resume would start in the old folder and
  `claude --resume` would not find the session; the hook frame's `cwd` is the fallback, and the
  next slice can ask the harness for a `cwd` field.
- **The variable's name.** `script/e2e.sh` has a runner variable `MARLEY_BIN`; it is assigned at
  the runner's top, so a value inherited from a Marley terminal never reaches it, but a reader
  could confuse them. The runner's could be renamed in the Test phase if it trips anyone.
- **Frames and reports out of step.** A frame can arrive before or after the report for the same
  moment (a `Stop` frame and the `idle` report). D6 keeps the state from the report and the
  labels from the frame, so the order changes nothing the row shows; the review checks the stop
  kind still reads the frame's `Stop`.
- **The main thread.** A busy main thread delays the answer; the program gives up at D8's limits
  and the mod swallows the error, so the seat may miss one report and take the next.
- **A liveness read per frame.** D5 reads `/proc/PID/stat` of the authority's process at each
  frame, on the main thread; it is one small file, at the rate hooks fire. If the review finds it
  measurable, the socket's thread can watch the process instead.
- **The harness may change the contract** before it answers MREQ-009; the promotion re-reads
  `AGENT_SEATS.md` and `register.js`.

### The reply to MREQ-009 (ready to send)
Marley's answer, for rustal-harness's `docs/planning/MARLEY_REQUESTS.md` MREQ-009:

- **The variables.** Every local interactive Marley terminal gives its programs
  `MARLEY_TERMINAL_ID` (a UUID, since Marley's #520) and `MARLEY_BIN` (a program Marley writes
  into its data directory). Report to Marley when both are non-empty. A task, a remote terminal
  and a terminal of a remote project get `MARLEY_BIN` empty, not removed, so treat an empty value
  as unset. Marley needs no `RH_STATE`: the program finds its Marley from where it was written.
- **The program.** `[MARLEY_BIN, 'report', ...]` with exactly the arguments `rh report` takes
  after `--state ROOT`, and `[MARLEY_BIN, 'release', '--source', SOURCE]`. Same source
  (`mod:claude-code`), same `seq` rule, same fields and limits, same refusal names
  (`agent_unknown`, `agent_report_stale`, `agent_report_authority`, the field names,
  `agent_release_authority`, `agent_release_none`), plus `marley_not_running` when no Marley
  answers. Exit 0 when accepted; otherwise non-zero with the name first on stderr. A release
  answers within 1 s, a report within 4 s. It is Python 3, as Marley's own hooks are.
- **What it carries.** One JSON line over a Unix socket of Marley's own. Marley knows the caller
  by its processes, as the harness does: the parent chain must reach a Marley terminal's shell,
  and the reporter's parent (the Claude Code process) becomes the terminal's authority, so a
  `claude -p` that the session runs in the same terminal cannot move the row.
- **Loading.** Marley will put its copy of the plugin at the front of `CLAUDE_CODE_PLUGIN_DIRS` in
  its local terminals (from 2.1.280; the mod needs 2.1.287), behind a setting off by default and
  Marley's version table. A harness seat runs under `env -i`, so it never sees Marley's
  variables or that entry; an installed copy under the same manifest name is replaced by the
  session-only one.

What Marley needs from the harness:
1. Marley's host in the mod: when `RH_BIN` or `RH_STATE` is unset or empty, and `MARLEY_BIN` and
   `MARLEY_TERMINAL_ID` are both non-empty, report and release through `MARLEY_BIN` as above; the
   harness first when both are set; nothing when neither. The same events as today: `idle` at
   `session.start`, `working` at a main `turn.start`, `idle` or `error` at a main
   `turn.complete` with the interrupted activity, usage and quota, a release at `session.end`
   except for `clear` and `resume`.
2. Under Marley's host, no `rh listen` or `delivered` and no `approve`: `tool.check` returns
   Claude Code's own verdict, so its dialog shows in Marley's terminal as today. Marley adds
   `listen`, `delivered` and `approve` to its program in later slices, after TICKET-101's and
   TICKET-102's shapes.
3. Reports only from an interactive session (`session.start`'s `isInteractive`), so a `claude -p`
   that a session starts reports for no one. Marley refuses it anyway; this keeps it quiet.
4. The test the harness planned, that a session reports to one host only, run with Marley's
   variables set beside `RH_*` and alone.
5. A revision Marley can record (this checkout has no git metadata: a tag, a commit on the
   repository's origin, or the files' SHA-256 as `rh` computes it), and a license for
   `plugin.json`, `hooks.json` and `register.js` that Marley's public repository can carry
   (MIT OR Apache-2.0, as Marley's crates), or Chad's word that they may be published there.
6. The Claude Code versions the mod is tested on, kept beside the files, for Marley's version
   table (#648); 2.1.287 now. This box already runs 2.1.288.
7. Before Marley retires its hook frames (a later ticket, not this one): under Marley's host, a
   `waiting` report while Claude Code shows its own permission dialog or an `AskUserQuestion`
   (`tool.check`'s `ask` verdict, with the tool and what it acts on as the activity), and
   `working` again when that call's `tool.call` resolves.
8. Optional: a manifest description that names both hosts, since Marley's users will see the
   plugin in `/plugin`.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION.md §3, §7, §14, §18, §19, §20, the three templates, and #633's and #640's
      pairs for shape and tone.
- [x] Read the brainstorm note in full (the fights, the tools, B1 to B7, Chad's answers, "The
      harness's side").
- [x] Read the harness's plugin (`plugin.json`, `hooks.json`, `register.js`, `plugin.rs`),
      `M13_PLAN.md` "One plugin, two hosts", `CLAUDE_CODE.md` "In a harness terminal",
      `AGENT_SEATS.md`, MREQ-009, D169, and `rh report`'s clap definitions; the saved Claude Code
      pages on mods, plugin loading and the variables.
- [x] Read Marley's plugin (`claude_plugin.rs`, `hooks.json`, `event.py`, the bridge's header),
      `claude_events.rs`, `agent_events.rs`, `resume.rs`, `notifications.rs`, `mcp.rs`'s
      programs, the opener, the terminal builder's identity hunk, `pty_info.rs`, the MCP
      transport, the relay's socket.
- [x] Recall (§18.3): the four ledgers grepped (plugin, `terminalSequence`, socket, resume, the
      ids, the opener, the relay) and the completed pipelines 482, 519, 520, 540, 547, 561, 583;
      the queued #648 and #649 read once they landed; the brain asked and searched.
- [x] Decided the split: this slice is the report path and its readers; loading the plugin is the
      next slice, with its decisions recorded here (D9, D10) so the reply can name them.
- [x] Changed D2 after reading #649: a program in the data directory, not a hidden mode of the
      app's executable, which also left `crates/zed/src/main.rs` untouched.
- [x] Reference (§20) and Prior art's three legs filled; Warp's source not read; the harness's
      code read as Ignibyte's own, no code carried.
- [x] The reply to MREQ-009 drafted above, with what Marley needs from the harness.
- [x] Spec, notes and ticket doc written; no other file touched, no cargo run, no agent run beyond
      `claude --version`.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete

### The harness's answer, 2026-10-03
rustal-harness recorded MREQ-009 as answered (its D176) and queued its side as TICKET-108,
"Marley's host in the shared Claude Code plugin" (acceptance MH-001 to MH-007), not started.
- Asks 1 to 4, 7 and 8 accepted as written; "interactive only" applies to both hosts. MH-006 adds
  that a failing or hanging `MARLEY_BIN`, `marley_not_running` included, leaves the session usable.
- Ask 5: the revision is the plugin's SHA-256 as `rh` computes it (length-prefixed path and
  contents of the three files; today `aedcac8e7a28a370be01ca8b680d18d3cfc1b05e1b2296186c98dfe722214fd1`,
  changing when TICKET-108 lands). The licence was open (the harness declares none); Chad
  decided it the same day: the three plugin files are "MIT OR Apache-2.0", so Marley's public
  repo may carry them (the whole harness repo stays as it is). Relayed to rustal-harness to mark
  the files; the loading slice no longer waits on it, only on TICKET-108.
- Ask 6: the file beside the plugin names only the harness's gate-tested pin, Claude Code
  2.1.287; versions Marley runs beyond that belong in Marley's own table (#648).
- Ask 7: `waiting` with a one-line activity of at most 120 characters; `working` again once
  `tool.call`'s `next(e)` resolves, for denials and approvals alike (MH-004).
- Timing: the mod gives a release exactly 1000 ms because Claude Code allows all `session.end`
  hooks 1.5 s, so `$MARLEY_BIN release` must answer well under 1 s, Python startup included;
  the Code phase measures it and keeps the helper's imports minimal.
