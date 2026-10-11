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
