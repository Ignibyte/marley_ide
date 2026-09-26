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
  `520a6e22a7`. #516, active the same night, edits `mcp.rs`, `browser_tools.rs` and
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
