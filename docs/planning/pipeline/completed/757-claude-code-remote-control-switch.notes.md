# Claude Code's Remote Control switch — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-757-claude-code-remote-control-switch.md
- **Pipeline spec:** 757-claude-code-remote-control-switch.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** the T3 survey's item 13 (`docs/t3code_architecture/README.md`) and report 04's item
  2: a setting that adds `--remote-control`, a chip on the row, and Marley's own Claude pushes held
  back while it is on. Its open question 5 asked whether that is acceptable, given that the session
  goes through Anthropic's service. Chad, 2026-10-11: "16.) yes and shelve". Deliberate, shelved.
- **Classification:** feature, S.
- **Recall (§18.3):**
  - AD-claude-535: one line to a local ntfy, nothing the agent wrote.
  - #532: the permission modes and their chip, the same place in the line and on the row.
  - The installed Claude Code is 2.1.296 and has the flag, whose name argument is optional.
- **Discovery:**
  - `marley_agent.rs`:
    - `command` (293-312) and `line_with` (213-231), whose `--` is written only before a prompt
      that starts with `-`;
    - `permission_mark` (476), which reads the foreground's arguments.
  - `rail.rs`: `terminal_marks` (9091-9103) and `permission_chip` (9024-9039).
  - `push.rs`: `on_change` (53-103), with `AgentKind::Claude` hard-coded at 89.
  - `settings_content/src/marley.rs`: `claude_code_permissions` (162), beside which the setting
    goes.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **The design** is written at promotion (`/pipeline:plan`).

### Design
- **Approach:** `LaunchMode` stays as it is. A `remote_control: bool` threads from the settings
  through `agents::launch_mode`'s caller into `command`. The mark reads `--remote-control` in the
  foreground's arguments as the permission mark reads the bypass flag. `push::on_change` asks the
  same question of the terminal before it posts.
- **File manifest:**
  - `crates/marley_agent/src/marley_agent.rs`, `crates/marley_workbench/src/agents.rs`, `rail.rs`
    and `push.rs`: Marley crates;
  - `crates/settings_content/src/marley.rs` and `assets/settings/default.json`: their existing rows
    in `docs/marley/zed-touchpoints.md` extended;
  - the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | Setting on; New Agent → Claude Code with a prompt | 757-01-chip; the stand-in's argument log |
| 003, 004 | Show another tab; the stand-in finishes; turn the setting off; start a second session and finish it | 757-02-quiet; the ntfy log |

- **Not reached:** the Claude app itself. The real session's link needs Chad's login, and a
  scenario never reaches outside the box.

### Risks
- Claude Code may rename the flag. The guide names the version it was checked on (2.1.296).
- The flag's optional name: a prompt written without `--` would become the session's name. D3
  covers it, and the scenario's argument log shows the `--`.

## Phase 1 — Plan (promoted 2026-10-12)
- **Promotion:** Chad, 2026-10-12, asked under the goal "continue until all tickets are finished":
  "Build it now". The pair moved to `active/`, the ticket is in progress, its Backlog row is gone.
  The brain's consultation is 52ddb208df3f46a5b8cdfd49bace385f.
- **Seams re-verified** at `902b067442`:
  - `marley_agent.rs`: `LaunchMode` (145), `line_with` (217), `command` (331), `resume_line`
    (Claude's line through `command`), `permission_mark` (612), `options` (662).
  - Every `LaunchMode` comes from `AgentPermissions::launch_mode` (`agents.rs:237`), reached through
    `agents::launch_mode` (373) and `resume.rs:438`.
  - `rail.rs`: the row's `TerminalAgent` (8665, the struct in `marley_rail.rs:377`, built in a
    `marley_rail` test at 3035 too), `agent_mark` (8856), `permission_chip` (9957),
    `terminal_marks` (10024), reading `marley_foreground_argv`.
  - `push.rs`: `on_change` (58) takes the `TerminalView`, so it can read the foreground's
    arguments.
  - The settings: `claude_code_permissions` at `settings_content/src/marley.rs:180`; the Marley page
    in `settings_ui/src/marley_page.rs`; `assets/settings/default.json:1853`. Each has its ledger row.
- **Design (changed from the queued one):** `LaunchMode` becomes a `Copy` struct `{ bypass,
  remote_control }` with `LaunchMode::ASK`, so the flag reaches every line `command` writes, the
  resume line included, with no signature changed. `AgentPermissions` reads
  `claude_code_remote_control` and sets `remote_control` for Claude Code only. `command` appends
  `--remote-control` after the bypass arguments; `line_with` writes `--` before any Claude Code
  prompt while it is on. `marley_agent::remote_control(kind, argv)` reads the flag in the
  foreground's options; `TerminalAgent.remote` carries it to a `remote` chip after the permission
  chip; `push::on_change` returns early for such a terminal. The setting gets a toggle in the
  Settings window's Agents section, as #726's Plan Review has.
- **Manifest:** `marley_agent.rs`, `agents.rs`, `rail.rs`, `push.rs`, `marley_rail.rs` (Marley
  crates); `settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`,
  `assets/settings/default.json` (Zed paths, rows extended first); the guides; the scenario.

## Phase 2 — Code
- **Built:** `marley_agent` (`LaunchMode` as a struct with `ASK`, `REMOTE_CONTROL_ARGUMENT`,
  `command` appending the flag for Claude Code, `line_with` writing `--` before its prompt,
  `remote_control`); `agents.rs` (`AgentPermissions.claude_code_remote_control`, `launch_mode`);
  `marley_rail` (`TerminalAgent.remote`, its test's constructor); `rail.rs` (`runs_remote`,
  `remote_chip`, `terminal_marks`); `push.rs` (the skip); `settings_content` (the field),
  `settings_ui` (the Agents row, the array at twenty-three), `default.json`; the ledger rows
  extended first; the guides.
- **Deviations from the queued design:** `LaunchMode` became a struct instead of a `bool`
  threaded through the callers, since every mode comes from `AgentPermissions::launch_mode`. A
  Settings row was added, as #726's setting has one.
- **The review:** `terminal_snapshot` went to 103 lines; the flag's read moved into `runs_remote`.
- **The gate:** `script/gates.sh --diff`: 17 of 17 green, receipt written.

## Phase 3 — Test
- **Scenario:** `script/e2e/757-claude-code-remote-control-switch.sh` under `compositor sway`, on
  #535's fake ntfy and private desktop bus, Claude Code started through #557's English path. The
  first run passed its checks; it then took the scratch branch off the name "remote" and moved
  `757-03` before the second turn, so the shot shows the second Claude Code running.
- **757-01-chip** (REQ-001, REQ-002): the new row "idle" with a `remote` pill; the terminal shows
  `claude --remote-control -- 'find the large files in this repo'`; the arguments' log holds
  `["--remote-control", "--", "find the large files in this repo"]` (check passed).
- **757-02-quiet** (REQ-003): the turn finished while the window was on another workspace; the
  fake ntfy logged nothing (check passed).
- **757-03-off** (REQ-004): with the setting off, `claude 'find the old logs'`, its row idle with no
  chip; its turn then pushed "repo: Claude finished" (check passed).
- **Not reached:** the Claude app itself, which needs Chad's login; a scenario stays on the box.

## Phase 4 — Complete
- CHANGELOG (Added); the guides (a section and an article); `marley_agent.md`, `marley_rail.md`
  and `marley_workbench.md`; the ledger rows for the three Zed paths.
- Knowledge: AD-claude-757-remote-control-rides-the-launch-mode-001.
