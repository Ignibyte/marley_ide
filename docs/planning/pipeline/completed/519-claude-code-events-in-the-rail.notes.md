# Claude Code's hook events in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-519-claude-code-events-in-the-rail.md
- **Pipeline spec:** 519-claude-code-events-in-the-rail.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25, on the Orca survey: "we will be taking what it does well and
  bring it in here". The survey's first item is agent state from Claude Code's hooks, carried
  in-band, and it lands before #508 and #509 (`docs/orca_architecture/README.md`; reports 01
  item 1, 05 item 1, 06 item 3). Queued overnight by the spec drafters; not promoted.
- **Classification / tier:** feature, prong 2 (C1's first piece). Marley crates for all of it but
  one hunk in a Zed crate (`terminal_view`). Size M.
- **Recall (§18.3):**
  - AD-claude-478 and AD-claude-482: the terminal reads OSC 9 and OSC 777 `notify`, and the
    plugin answers hooks with a `terminalSequence` only where `TERM_PROGRAM` is `zed`. This
    ticket reuses both paths.
  - L-claude-482: `claude -p` drops a hook's `terminalSequence` (print mode registers no writer).
    The live check must be an interactive session in a pty, in a folder Claude Code trusts with
    no settings of its own (`/srv/stacks`).
  - L-claude-478: a new `terminal::Event` variant breaks every exhaustive match outside the
    terminal crates (`agent_ui`). The design adds no variant.
  - PR-claude-474 and AD-claude-474: a frame is output until a nonce says otherwise; showing a
    field needs no check, acting on one does (D4).
  - L-claude-480: an e2e fake acts the program out so the result shows on screen; the fake
    `claude` here runs the real hook.
  - L-claude-491 (sessions need room and a close) and AD-claude-491: the bridge closes its
    session; the harness-side `fleet_snapshot` client does the same.
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery:** each seam opened and checked on 2026-09-25; line numbers are at commit
  `484a7f18cb`. #516, active the same night, edits `mcp.rs`, `browser_tools.rs` and
  `marley_mcp`, so promotion re-reads those.
  - The plugin: `crates/marley_workbench/claude_plugin/marley/hooks/hooks.json` (Notification
    for `permission_prompt` and `idle_prompt`, and Stop, all to `notify.sh`);
    `hooks/notify.sh:5` (the `TERM_PROGRAM` gate) and `:13-14` (the project's name cleaned, the
    fixed sentence printed); `bin/marley-mcp-bridge` (Python 3, standard library: the precedent
    for D3). Version 1.1.0 in `.claude-plugin/plugin.json` and the marketplace's
    `.claude-plugin/marketplace.json`.
  - `crates/marley_workbench/src/claude_plugin.rs:28` (`BRIDGE`), `:31-59` (`FILES`, six
    entries, each with its program bit), `:139` (`installed_in`), `:168` (`install`, whose toast
    says to run `/reload-plugins`). Claude Code's `installed_plugins.json` is
    `{"version", "plugins": {"<name>@<marketplace>": [{"scope", "installPath", "version", ...}]}}`,
    and `installPath` is a per-version copy under `~/.claude/plugins/cache/` (read on the box;
    Chad's list has no `marley@marley` today). The 2.1.283 bundle has `plugin marketplace update
    [name]` and `plugin update <plugin>` (with `-y`).
  - The escape: `crates/marley_dcs/src/notification.rs:11` (`MAX_NOTIFICATION`, 4 KiB; a longer
    OSC is not reported), `:92-117` (`777;notify;<title>;<body>`, split at the first `;` after
    the title), `:120` (control characters dropped; base64 has none and no `;`).
    `vendor/alacritty_terminal/src/marley_hooks.rs:71-92` runs the scanner on every byte the
    parser sees, the alternate screen included, unlike the shell hooks.
  - Zed's terminal: `crates/terminal/src/terminal.rs:741` (`Event::MarleyNotification`),
    `:1805-1810` (emitted); `crates/terminal_view/src/terminal_view.rs:1185-1190` (the arm that
    sets `has_bell` for every notification); `crates/agent_ui/src/agent_panel.rs:2272` (ignores
    the event; unchanged).
  - `crates/marley_workbench/src/notifications.rs:28-45` (`init` subscribes each terminal view to
    its terminal and matches the event) and `:49-72` (`notify`).
  - `marley_fleet`: `src/session.rs:13-26` (`State`), `:43-50` (`Question`), `:55-77`
    (`Session`, its `labels` a `BTreeMap` rendered, never matched); `src/reducer.rs:17-80`
    (`SessionEvent`: `Upsert`, `StateChange`, `QuestionRaised`, `QuestionCleared`, `Heartbeat`,
    `Ended`), `:147-208` (`apply`: auto-vivifies a seat, never removes one);
    `src/attention.rs:41-43` (`is_stale` flags every seat not Done).
  - `crates/marley_agent/src/marley_agent.rs:73-79` (`agent_kind_of`), `:117` (`WAITING_AFTER`,
    2 s), `:121-127` (`agent_status`), `:131` (`status_line`). The crate has no dependencies yet.
  - `crates/marley_rail/src/marley_rail.rs:46-68` (`TerminalSnapshot`, and `TerminalAgent`, which
    is `Copy` today).
  - `crates/marley_workbench/src/rail.rs:375-387` (`note_output` and the quiet timers),
    `:1666-1721` (`terminal_snapshot`, where the agent's status is judged), `:1734`
    (`build_snapshot`), `:1268` (`render_terminal_row`).
  - `crates/marley_workbench/src/agent_bar.rs:93` (`contents`), `:129` (`agent_in`, the
    foreground's agent), `:305` (`claude_plugin_chip`).
  - `crates/marley_workbench/src/mcp.rs:76-82` (the server's shared data, built with a default
    snapshot and dropped from scope after `spawn`), `:219-231` (`answer`);
    `crates/marley_mcp/src/transport.rs:34-46` (`ServerData.snapshot`), `:369` (`signal_change`).
  - `crates/terminal/src/terminal.rs:3615` (`foreground_process_command_from_argv`: for `node`,
    `python`, `python3`, `bun` or `deno` the script's name is the command), which lets a Python
    fake pass as `claude`.
  - Claude Code 2.1.283's bundle, read with `grep -a`: `LTn=new Set([0,1,2,9,99,777])`,
    `NTn=4096`, the payload filter `FTn` (keeps code points 32 and up, minus 127 to 159), `_C`
    (wraps for tmux and screen); the payloads: UserPromptSubmit `prompt`, `session_title`;
    PreToolUse `tool_name`, `tool_input`, `tool_use_id`; PostToolUse adds `tool_response`,
    `duration_ms`; PostToolUseFailure `error`, `is_interrupt`; PermissionRequest `tool_name`,
    `tool_input`, `permission_suggestions`; Stop `stop_hook_active`, `last_assistant_message`;
    StopFailure `error`, `error_details`, `last_assistant_message`; PostCompact `trigger`,
    `compact_summary`; SubagentStart `agent_id`, `agent_type`; SubagentStop adds
    `agent_transcript_path`, `last_assistant_message`; SessionStart `source`, `model`;
    SessionEnd `reason`; common `session_id`, `transcript_path`, `cwd`, `prompt_id`,
    `permission_mode`, `agent_id`, `agent_type`.
  - Orca's recording `src/shared/__fixtures__/claude-cancel-shell-hooks.jsonl` (2.1.280): an
    Escape mid-tool and mid-stream fires no hook, `<task-notification>` prompts arrive as
    UserPromptSubmit, SessionEnd's reason is `prompt_input_exit`.
- **Decisions:** D1 to D9 in the spec.

### Slice 1 at promotion (2026-09-26): what changed from the queued design
- **Recall at promotion:** L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001
  (only an interactive session writes a hook's sequence; a Python pty proves a real run);
  L-claude-478-a-new-terminal-event-reaches-every-exhaustive-match-001 and
  L-claude-478-omarchys-notifications-go-to-quickshell-001 (`busctl --user monitor`);
  L-claude-440-testing-agent-clis-without-a-pty-001. Brain consultation
  cf47828359624003b1ffa447279c2a19: nothing on this seam.
- **Cut to slice 1** (#547 takes the rest): the design's items 1 to 6 without the stale form
  (D7), the version bump without the update chip (D9's chip), and no MCP publishing (item 7).
- **Seams re-verified** (an Explore sweep at `3dbeb53d86`). What it changed:
  - The rail never sees a notification: the view re-emits only `Wakeup`. The rail observes the
    `AgentEvents` global instead (`cx.observe_global`), which `on_frame` updates.
  - The branch goes in `notifications::init`'s subscription, before `notify`, which returns early
    for a focused terminal.
  - `row_card` has one fixed-height subtitle and is shared with thread rows, and a test pins equal
    heights for rows with and without a second line. An event row keeps the status on its second
    line, with the prompt after it (`Claude Code · working · Add a README…`), and gains a third
    line for the activity (the tool and its input, what it waits on, the last message, the
    error); only such rows grow, so the pinned rows keep their height.
  - `TerminalAgent` stays `Copy` (seven sites copy it); the event text rides beside it as
    `TerminalSnapshot::agent_detail` and `TerminalRow::agent_detail`, an `AgentDetail { summary,
    activity }` of strings.
  - `marley_agent::AgentStatus` gains `Idle` and `Failed` (labels `idle`, `failed`).
  - `marley_fleet::Question` has no tool field: its `prompt` holds `Write: README.md`.
  - `agent_in` reads a cached foreground that refreshes on output; a frame can beat it (a
    SessionStart right after launch). D4 stands: such a frame is dropped, and the next one lands.
  - Seats end on SessionEnd, reset on a SessionStart or a new `session_id`, and go when the view
    is released; a seat is shown only while Claude Code is the terminal's foreground.
  - Orca's `src/shared/harness-injected-user-turns.ts` (MIT) gives the injected-turn tags and
    prefixes; `claude_events` names it with its notice.

### Design
- **Approach.**
  1. `hooks/event.py`: read at most 1 MiB of stdin; parse; build the summary of the spec's
     scope (preview keys: `file_path` for Read, Write, Edit, MultiEdit and NotebookEdit;
     `command` for Bash; `pattern` for Glob and Grep; `url` for WebFetch; `query` for WebSearch;
     `description` for Task; the first question's text for AskUserQuestion; else the first string
     value); cut and bound (the spec's limits); `base64.b64encode`; print the answer with
     `json.dumps`, which escapes ESC and BEL as `\u001b` and `\u0007`. `{}` outside
     `TERM_PROGRAM=zed` and on any exception.
  2. `marley_agent::claude_events` (new file, pure): `HookEvent` (serde, unknown fields ignored,
     `v` must be 1), `decode(body) -> Result<HookEvent, DecodeError>` (base64, then at most 3,000
     bytes of JSON), and `fold(seat, previous: Option<&Session>, event, now_ms) -> Vec<SessionEvent>`
     for D6, which starts from the previous seat's labels so a harness-injected prompt keeps the
     user's, and emits `Upsert` plus `QuestionRaised` or `QuestionCleared`, or `Ended`. The labels
     keep the tools in flight (`tool_use_id`, tool, preview, agent) so a PermissionRequest pairs
     with the PreToolUse before it and its wait ends at that tool's PostToolUse or
     PostToolUseFailure, never at another tool's.
     `is_harness_injected` and `is_compact_continuation` carry Orca's observed tags and prefixes;
     the file names `src/shared/harness-injected-user-turns.ts` and keeps Orca's MIT notice
     (README, "The source, and the licence"). Label keys: `agent` (`claude-code`), `prompt`,
     `tool`, `preview`, `message`, `error`, `subagents`, `session_id`, `prompt_id`,
     `transcript_path`, `permission_mode`, `cwd`.
  3. `marley_workbench::agent_events` (new): a global entity holding the `FleetSnapshot` and each
     seat's terminal view. `on_frame(view, body, cx)`: drop it unless `agent_bar::agent_in` says
     Claude Code is the foreground (D4), decode, fold against the previous `Session`, `apply`,
     emit a change, publish to the MCP server. The rail's refresh ends a seat whose terminal is
     gone or whose foreground is no longer Claude Code; once 32 seats have ended, the snapshot is
     rebuilt from the live ones (an `Upsert` per live seat, then its `QuestionRaised`), the
     reducer's own path, since `apply` never removes a seat.
  4. `notifications.rs`: a title equal to `marley_terminal::AGENT_EVENT_TITLE` goes to
     `agent_events::on_frame`; every other title is shown as before.
  5. `terminal_view.rs` (Zed): the #478 arm sets `has_bell` unless the title is
     `marley_terminal::AGENT_EVENT_TITLE` (`terminal_view` depends on `marley_terminal` already).
  6. The rail: `TerminalAgent` gains the event's state and detail (prompt, activity line,
     subagent count) and loses `Copy`; `AgentStatus` gains idle, failed and a stale form carrying
     its minutes. `terminal_snapshot` reads the seat for the view's id when there is one and
     falls back to `agent_status` when there is not. The rail subscribes to `AgentEvents` and,
     while a working seat can go stale, keeps a one-minute timer so the minutes move.
  7. `mcp.rs`: the `McpServer` global keeps the transport's `Shared`; a `publish(&FleetSnapshot)`
     replaces `ServerData.snapshot` and calls `transport::signal_change`.
  8. `claude_plugin.rs`: `FILES` gains `marley/hooks/event.py` (a program);
     `installed_version_in(config_dir)` reads the entry's `version`; the chip shows "Update
     Marley's plugin" while it is older than `plugin.json`'s, and `update` writes the marketplace,
     then runs the two `claude plugin` commands through `util::command::new_command`, as
     `install` does.
- **File manifest.**
  - Marley: `crates/marley_workbench/claude_plugin/marley/hooks/hooks.json`,
    `crates/marley_workbench/claude_plugin/marley/hooks/event.py` (new),
    `crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json`,
    `crates/marley_workbench/claude_plugin/.claude-plugin/marketplace.json`,
    `crates/marley_workbench/src/claude_plugin.rs`, `crates/marley_workbench/src/agent_bar.rs`,
    `crates/marley_workbench/src/notifications.rs`,
    `crates/marley_workbench/src/agent_events.rs` (new),
    `crates/marley_workbench/src/rail.rs`, `crates/marley_workbench/src/mcp.rs`,
    `crates/marley_workbench/src/marley_workbench.rs` (the module and its `init`),
    `crates/marley_workbench/Cargo.toml` (`marley_fleet`, which it reaches only through
    `marley_mcp` today),
    `crates/marley_agent/src/claude_events.rs` (new), `crates/marley_agent/src/marley_agent.rs`,
    `crates/marley_agent/Cargo.toml` (`serde`, `serde_json`, `base64`, `marley_fleet`),
    `crates/marley_rail/src/marley_rail.rs`, `crates/marley_terminal/src/marley_terminal.rs`
    (`AGENT_EVENT_TITLE`), `script/e2e/519-claude-code-events-in-the-rail.sh` (Test).
  - Zed: `crates/terminal_view/src/terminal_view.rs` (the #478 arm, one condition).
  - Generated: `Cargo.lock`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the existing
  `crates/terminal_view/src/terminal_view.rs` row gains "the `Event::MarleyNotification` arm
  leaves a notification titled `marley-event` unmarked (#519)". No other Zed path changes, and
  `marley_agent`'s new dependencies are all in `[workspace.dependencies]` already.

### E2E plan
Fixtures: a scratch repository; a HOME whose `.bashrc` is the scenario's; `$E2E_WORK/bin` first
on the PATH with the fake `claude` (Python). With `plugin` as its first argument the fake appends
its arguments to `$E2E_WORK/claude-plugin.log` and exits 0; otherwise it prints a banner and, at
each line on stdin, sends its next recorded event through the real
`crates/marley_workbench/claude_plugin/marley/hooks/event.py` and writes the answer's
`terminalSequence` to stdout. `CLAUDE_CONFIG_DIR` is exported to a scratch folder before the
launch. Setup starts `busctl --user monitor org.freedesktop.Notifications` into a log, stopped by
teardown.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-009 | type `claude`, wait 3 s before the first Enter | `519-00-before-events` |
| REQ-001, REQ-002 | Enter three times: SessionStart, UserPromptSubmit, PreToolUse (Bash `ls -la`) | `519-01-working` |
| REQ-003 | Enter five times: the Bash's PostToolUse, PreToolUse (Write `README.md`), PreToolUse (Read `src/lib.txt`), PermissionRequest (Write), the Read's PostToolUse | `519-02-waiting` |
| REQ-004 | Enter three times: the Write's PostToolUse (the wait ends), SubagentStart, a subagent's PreToolUse (Grep) | `519-03-subagent` |
| REQ-005 | Enter three times: the subagent's PostToolUse, SubagentStop, Stop with a last message | `519-04-idle` |
| REQ-006 | Enter twice: a `<task-notification>` prompt, then the post-compaction continuation | `519-05-injected` |
| REQ-007 | Enter twice: a new prompt, StopFailure (`rate_limit`) | `519-06-failed` |
| REQ-008 | no bell dot in the shots above; the `busctl` log has no `Notify` whose app name is `Marley` | the shots; the run log |
| REQ-011 | the harness-side client (491's, with a `fleet` command) calls `fleet_snapshot` through the bridge | the run log: the seat, its state `error`, its labels |
| REQ-012 | setup runs `event.py` without `TERM_PROGRAM=zed`, and with a 100 KB Write payload | the run log: `{}`, then the answer's byte count under 4,096 |
| REQ-013 | a scratch `installed_plugins.json` lists `marley@marley` 1.1.0; click the chip | `519-07-update-chip`; `claude-plugin.log` holds both commands |
| REQ-010 | only with `E2E_LONG=1`: a new prompt and a PreToolUse, then 31 minutes with no event | `519-08-no-update` |
| REQ-014 | Test, outside the scenario: a Python pty runs `claude --plugin-dir <plugin> "Reply with the single word ok."` in `/srv/stacks` with `TERM_PROGRAM=zed` and records the raw output | the live check's log: the decoded frames by event |

What no scenario reaches: a real model turn held steady for shots (REQ-014 is the live check
instead), and SessionStart's frame, which may arrive before Claude Code registers the writer that
`terminalSequence` goes through (L-claude-482); the live check says whether it arrives, and no
row needs it. #540, drafted the same night, takes the session id from it; every event carries
`session_id`, so #540 can take it from the first frame that does arrive.

### Risks
- Claude Code might not write `terminalSequence` for every registered event. The field is
  documented now, and the 2.1.283 code emits it for any hook's result; the live check at
  promotion proves the events the rows need before code is written.
- Two Python starts per tool call (PreToolUse, PostToolUse), about 30 ms each. Measured during
  the live check; if it drags, PostToolUse goes (PreToolUse names the tool and the next event
  moves the row on).
- Interrupts: after an Escape the row reads `working` until the next event (Out, with the
  reason). The 30-minute rule is the backstop.
- Rows grow to four lines for an agent with events; long prompts and paths are cut to one line
  each by the row's label.
- Upstream: one condition inside a Marley hunk of `terminal_view.rs`, whose row exists.
- If the slice runs long, the chip's update (REQ-013) and `fleet_snapshot` (REQ-011) split off as
  a follow-up; the rows (REQ-001 to REQ-010) are the first slice.

## Phase 2 — Code
- **Built (slice 1).**
  - The plugin: `hooks/event.py` (new, a program), `hooks.json` registering the twelve events
    beside `notify.sh`'s two, version 1.2.0 in `plugin.json` and `marketplace.json`, and
    `claude_plugin::FILES` shipping `event.py` (seven files).
  - `marley_terminal::AGENT_EVENT_TITLE` (`marley-event`); Zed's `terminal_view` arm leaves a
    notification with that title unmarked, its ledger row widened.
  - `marley_agent`: `AgentStatus::Idle` and `Failed`; `claude_events` (new, pure): `HookEvent`,
    `decode` (base64, at most 3,000 bytes, `v` 1), `fold` (D6 through a private `Moving` seat,
    one handler per event kind), `seat_status`, `seat_summary` and `seat_activity` for the row,
    and Orca's injected-turn tags and prefixes with its source named.
  - `marley_rail`: `TerminalSnapshot::activity` and `TerminalRow::activity`, copied by
    `rail_rows` and `switcher_rows`; the tests' literals gained the field.
  - `marley_workbench`: `agent_events` (new): the `AgentEvents` global over one
    `FleetSnapshot`, `on_frame` (drops a frame unless `agent_bar::agent_in` says Claude Code,
    decodes, folds against the seat, applies), `forget` on a view's release (the snapshot folded
    again without the seat); `notifications::init` sends a `marley-event` title to `on_frame`
    and registers `forget`; the rail observes `AgentEvents` (`observe_global_in`), reads the seat
    in `terminal_snapshot` for a Claude Code row, puts the subagents and the prompt after the
    status and the activity on a third line; `row_card` takes the lines under the title, and a
    row with two of them is 3.5 rem tall, the others keep `h_11`.
- **Deviations from the design, and why.**
  - One `tool` label holds `Tool: preview`; no separate `preview` label. The row shows the line
    as it is and the question names it the same way, so a split would only be joined again.
  - The tools in flight are labels too (`lead_tool:<id>`, `subagent_tool:<id>`) with
    `waiting_on`, so `fold` stays a pure function of the previous seat and the event.
  - `activity` alone rides on the snapshot and the row, not an `AgentDetail` pair: the prompt and
    the subagents join the status line, which `subtitle` already carries.
  - A user's prompt (not a harness's) starts a new turn: it ends a wait and forgets the tools in
    flight, since a turn cannot go on waiting once the user has typed the next prompt. An
    injected prompt starts a turn too but keeps the user's prompt on the row.
  - The subagent count is the turn's: `end_turn` clears it, so an interrupted subagent that never
    reports `SubagentStop` cannot leave a stale count on the next turn.
  - A permission's question reads `Permission for Write: README.md`; `AskUserQuestion`'s is its
    question's text.
  - `agent_events` needs no `init`: `default_global` creates the global at the first frame, and
    `observe_global_in` fires for it.
- **Review of the diff.**
  - Re-entrancy: `on_frame` runs inside the terminal view's subscription and reads only the
    `Terminal` entity and the global; the rail's observer runs at the effect flush, after the
    view's update ends, so its `view.read` never meets a view being updated.
  - Errors: a frame that does not decode is logged at debug and dropped (display data, D4);
    nothing on these paths can panic (no indexing, `strip_prefix` for the ids).
  - Provenance: Orca's MIT list named in the module doc; nothing from Warp.
  - Upstream: the one Zed hunk is a condition inside #478's Marley arm.
- **Checks.** `cargo check` and `cargo clippy --all-targets -D warnings` over `marley_agent`,
  `marley_rail`, `marley_workbench` and `terminal_view` clean; `cargo fmt --check` clean.

## Phase 3 — Test
- **The scenario:** `script/e2e/519-claude-code-events-in-the-rail.sh`, `compositor sway`. Setup
  checks the hook itself, writes a HOME whose `.bashrc` puts `$E2E_WORK/bin` first on the PATH,
  the stand-in `claude` (Python; it runs the real `hooks/event.py` per recorded payload at each
  Enter and writes each answer's `terminalSequence`), the six steps, and starts
  `busctl --user monitor org.freedesktop.Notifications` into a log. The plan's `SessionStart`
  and a `Task` around the subagent were added to the steps, so the lead's line shows the Task
  while the subagent's Grep runs, as a real session's does.
- **Run** (debug build, then again after the fixes below; SHOT_DIR in the scratchpad): every
  check passes: `{}` outside a Marley terminal; for 100 KB prompt, tool input and message with
  5,000-byte paths, 555-, 463- and 543-byte sequences carrying the event and the fields the row
  shows; the stand-in acted out all six steps (`terminal-read claude`); the monitor saw the
  service's calls (a `GetServerInformation` of our own), and no `marley-event` in any call.
  Sway: the user's Hyprland had no Marley window before or after, and no rule or reload.
- **The shots, read (the rail rows cropped and enlarged):**
  - `519-00-before-events`: `Claude Code · waiting`, two lines, from the quiet timer (REQ-009).
  - `519-01-working`: `working · Add a README to the…`, then `Bash: ls -la` on a third line;
    the row is the one taller row (REQ-001, REQ-002).
  - `519-02-waiting`: `waiting · Add a README to the…`, then `Permission for Write: README…`,
    after the Read's PostToolUse: the parallel tool finishing did not end the wait (REQ-003).
  - `519-03-subagent`: `working · 1 subagent · Add a RE…`, then `Task: Find the TODOs`: the
    count, the lead's state and prompt, and the subagent's Grep not on the lead's line (REQ-004).
  - `519-04-idle`: `idle · Add a README to the proj…`, then `I added README.md with a sho…`
    (REQ-005).
  - `519-05-injected`: `idle · Add a README to the proj…`, then `Noted.`: the
    `<task-notification>` and the continuation left the user's prompt (REQ-006).
  - `519-06-failed`: `failed · Run the tests`, then `rate_limit` (REQ-007).
  - No bell dot in any shot, and the terminal shows only the stand-in's lines (REQ-008).
- **Reds, and what changed.**
  - The first run's rows read `Claude Code · working · Add a…`: the agent's name took the
    rail's width and the prompt showed three letters. The icon names the agent, so an event
    row's line now starts with the state (`claude_events::seat_line`, which replaces
    `seat_summary`); a row without events keeps `Claude Code · waiting`.
  - The size check, with 5,000-byte paths, passed on a sequence that carried no prompt:
    `event.py` dropped the prompt before the paths. It now drops an overlong transcript path and
    working directory first, and the check requires each row's fields in the summary.
- **REQ-012:** the setup checks above, in the run log.
- **REQ-014, the live check** (`scratchpad/night/519/live-check.py`): Claude Code 2.1.283 in a
  Python pty, `--plugin-dir` at the repository's plugin, in `/srv/stacks` with
  `TERM_PROGRAM=zed`, asked to Read a file and reply `ok`. Its output carried `marley-event`
  frames for UserPromptSubmit (the prompt), PreToolUse and PostToolUse (Read and the path) and
  Stop (`ok`). No SessionStart frame, as L-claude-482 predicted: no row depends on one.
- **Cost:** `event.py` takes 14 ms a call (median of 20, 19 ms at most), two per tool call.
- **Golden set:** 519 joins it (its checks guard the hook's bound and the frames' route);
  `just regress` on the debug build: all 11 pass. The rows themselves are checked by shots; #547's
  `fleet_snapshot` gives the scenario a check of the seat's state.
- **Gate:** `script/gates.sh --diff` first red on two docs findings (the first paragraph of
  `seat_line`'s doc too long for clippy; the module doc of `agent_events` linking the
  crate-private `on_frame`), fixed at the source; then `GATE GREEN [diff]`, 16 passed.
- **Not reached by any scenario:** a real model's turns held steady for shots (the live check
  stands in), and the frames of a real permission prompt and subagent (the stand-in's recorded
  payloads carry the fields 2.1.283's bundle sends).
- **Verdict:** Phase 3 PASS.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added); `docs/marley/three-prong-plan.md` (C1's first piece
  shipped); `docs/marley_architecture/marley_agent.md` (`claude_events`, the dependencies),
  `marley_rail.md` (`activity`), `marley_workbench.md` (the notification route,
  `agent_events`, the rail's rows, the plugin's 1.2.0 hooks); the `terminal_view.rs` row in
  `docs/marley/zed-touchpoints.md` describes what shipped.
- **Knowledge:** F-claude-519-a-frame-bound-dropped-the-shown-fields-before-the-unbounded-ones-001,
  PR-claude-drop-the-unbounded-fields-first-001,
  AD-claude-519-claude-codes-hook-events-ride-in-band-into-marley-fleet-001.
- **Brain:** consultation cf47828359624003b1ffa447279c2a19 closed with
  `decisions/marley-claude-codes-hook-events-ride-in-band-into-marley-fleet` (follow-up by
  2026-10-10).
- **Ticket:** closed; TICKET-547 stays queued for slice 2.
