---
pipeline_id: 4cab8f4a-e46c-48eb-a9fe-e80f9271c0e4
ticket: docs/planning/tickets/open/TICKET-552-codex-and-opencode-notifications.md
status: Phase 4 — Complete PASS
title: "Notification setup for Codex and OpenCode from the agent bar"
type: feature
slice: prong 1 T7b (desktop notifications from CLI agents); the Warp second pass, "Smaller"
references: [docs/planning/design-notes/warp-second-pass-2026-09-25.md, docs/planning/pipeline/completed/482-claude-code-notifications-chip.spec.md, docs/planning/pipeline/completed/478-terminal-notifications.spec.md, docs/planning/pipeline/completed/547-claude-code-events-slice-2.spec.md]
---

## Title
Marley's plugin makes Claude Code notify its terminal; Codex and OpenCode can do the same, each
its own way, and Warp's agent bar sets both up in a click. Codex has the mechanism built in, off
by default and tuned for terminals it knows; OpenCode has none of its own but loads plugin files
from a folder. Two chips, one config edit and one file.

## Scope
### In
- **Codex.** Under a terminal whose foreground is `codex`, the bar shows "Turn on Codex
  notifications" while `<CODEX_HOME or ~/.codex>/config.toml` lacks any of `notifications =
  true`, `notification_condition = "always"` and `notification_method = "osc9"` under `[tui]`. A
  click sets the three with `toml_edit` (the file's other keys, order and comments kept; a
  missing file is made), then a toast: "restart a running Codex to pick it up". `notifications`
  already set to a list of event names is left as it is (it counts as on).
- **OpenCode.** Under a terminal whose foreground is `opencode`, the bar shows "Connect OpenCode
  to Marley" while `<XDG_CONFIG_HOME or ~/.config>/opencode/plugins/marley.js` is missing, and
  "Update Marley's plugin for OpenCode" while its first line names an older version than the one
  Marley ships. A click writes the file (0644) and shows the toast. The plugin, ES module
  JavaScript with no dependency, exports one hook: on `session.idle`, `permission.asked` and
  `session.error` it writes `ESC ] 777 ; notify ; OpenCode ; <project> <message> BEL` to the
  controlling terminal (`/dev/tty`, D4), only when `TERM_PROGRAM` is `zed`, and never throws (a
  failure is swallowed).
  Its first line is `// marley-opencode-plugin <version>`.
- **The chips** follow `claude_plugin_chip`'s shape: a busy label while writing, an error toast
  on failure, no chip once the check passes. The checks read the files off the main thread when
  the terminal's foreground changes to the agent, as `ClaudePlugin` reads Claude Code's list at
  start.
- **The notification itself** needs nothing new: Codex's OSC 9 and the plugin's OSC 777 arrive
  as `Event::MarleyNotification` and show on the desktop unless the terminal is the focused one
  (#478), with the click that shows the terminal.
- `crates/marley_workbench/agent_plugins/opencode/marley.js`, shipped with `include_str!` as
  the Claude plugin's files are.

### Out (explicitly deferred)
- Codex's `notify` program hook (an external command with a JSON payload): it runs outside the
  terminal, so nothing it prints reaches Marley; the TUI's own notifications do.
- OpenCode's npm plugin route (`"plugin": [...]` in `opencode.json`): a published package is a
  release Marley would have to keep; a file in the plugin folder is Marley's to write.
- Gemini CLI, and the other CLIs Warp detects (Grok, omp, agy: one entry each in `marley_agent`
  when Chad runs them).
- Hook events for the rail from Codex or OpenCode (#519's frames are Claude Code's; the same
  plugin file could carry `marley-event` frames later, once the rail's fold knows their events).
- Notifications that say what happened (#538) for these two: the messages are fixed sentences,
  as `notify.sh`'s are.

## Reference (§20)
- **Warp, agent notifications** (https://docs.warp.dev/agents/capabilities/agent-notifications/,
  https://docs.warp.dev/agents/cli-agents/codex/, https://docs.warp.dev/agents/cli-agents/opencode/,
  read 2026-09-26): an "Enable CLI agent notifications chip" in the agent utility bar; for
  Codex, add `notification_condition = "always"` under `[tui]` in `~/.codex/config.toml` and
  restart (its newer page installs a Warp plugin from a marketplace instead); for OpenCode, add
  `"@warp-dot-dev/opencode-warp"` to the `plugin` array. Marley keeps the chip per agent and the
  restart note; it sets Codex's built-in notifications (with the method forced to OSC 9, since
  Marley's `TERM_PROGRAM` is `zed`, a name Codex's `auto` need not know) and writes a local
  OpenCode plugin file instead of a package. Warp's plugins were not read (§20).
- **Upstream Zed:** nothing to keep; Zed shows no agent's notifications.
- **Marley's own precedent:** #482's chip and plugin for Claude Code, #547's update chip.

### Prior art
- **Behavior maps and reports.** The Warp second pass, "Smaller" ("If Codex's own notifications
  arrive as OSC 9, Marley shows them already and a chip only has to set the config. S, after
  checking what Codex prints in a Marley terminal"). `docs/warp_architecture/crates/input_classifier.md`
  names `codex` among the CLIs Warp forces to shell, the same list `marley_agent` knows.
- **Published material.** Codex's configuration reference (the `[tui]` table: `notifications`,
  "Enable TUI notifications; optionally restrict to specific event types", a boolean or a list;
  `notification_condition`, `unfocused` by default or `always`, "Control whether TUI
  notifications fire only when the terminal is unfocused or regardless of focus";
  `notification_method`, "Notification method for terminal notifications (default: auto)",
  `auto`, `osc9` or `bel`; and the top-level `notify` program). The installed Codex 0.155.1's
  strings carry `NotificationCondition`, `unfocused`, `always`, `osc9`. OpenCode's plugin docs
  (https://opencode.ai/docs/plugins/): local plugins load from `.opencode/plugins/` and
  `~/.config/opencode/plugins/`; a plugin exports an async function taking `{ project, client,
  $, directory, worktree }` and returns hooks; the `event` hook receives `{ event }` with types
  such as `session.idle`, `permission.asked`, `session.error`, `message.updated`; the installed
  OpenCode 1.18.31's strings say "Use plugins to send OS notifications when sessions complete"
  and name both folder spellings. rxvt's and Ghostty's OSC 777 form, iTerm2's OSC 9 form
  (`marley_dcs::notification`'s module doc).
- **The code we already ship.**
  - `claude_plugin.rs`: the `FILES` table with `include_str!` and program bits (`:35`),
    `ClaudePlugin` (`:69`) with `installed`, `installed_version`, `installing`, `updating`,
    `needs_update` (`:94`, semver), `init` reading `CLAUDE_CONFIG_DIR` (`:120`), `set_up` reading
    the list off the main thread (`:139`), `write_plugin_in` (`:170`), `installed_in` (`:186`),
    `install` with its toast and error (`:229`), `update` (`:312`). `agent_bar.rs`:
    `claude_plugin_chip` (`:305`), gated on `AgentKind::Claude` (`:198-202`); `agent_in`
    (`:129`). `marley_agent.rs`: `AgentKind` (`:32`, `Codex` and `OpenCode` among them),
    `agent_kind_of` (`:76`). `notifications.rs`: the subscription (`:38-47`) and `notify`
    (`:59`). `marley_dcs::NotificationScanner` (`notification.rs:38`) parses OSC 9 and 777.
    `marley_workbench::run_program` (`marley_workbench.rs:295`) is not needed: no program runs.
  - `toml_edit` 0.22 is a workspace dependency (`Cargo.toml:888`, `default-features = false`);
    `serde_json` for nothing here. `util::paths::home_dir()`.
  - Does a crate we build own this seam? `marley_workbench` owns the bar, the chips and the
    plugin precedent; `toml_edit` owns the comment-preserving edit; nothing else is needed.

## UI proof
UI-AFFECTING: two chips in the agent bar. `script/e2e/552-codex-and-opencode-notifications.sh`
(`compositor sway`, for the clicks), on a private session bus whose notification server logs each
banner (#538, #551). Fixtures: `CODEX_HOME` and `XDG_CONFIG_HOME` exported for Marley's process to
scratch folders, a `config.toml` there with a comment, a `model` key and a `[tui]` table holding
one unrelated key; stand-in `codex` and `opencode` first on the terminal's PATH (`exec -a` over a
Python that prints a line and reads stdin). Shots: `codex` typed, the bar with "Turn on Codex
notifications" (`552-01-codex-chip`); a click, the toast, no chip, the log's `config.toml` with the
three keys under `[tui]` and the comment and the other keys kept (`552-02-codex-written`); Ctrl-D,
`opencode`, the bar with "Connect OpenCode to Marley" (`552-03-opencode-chip`); a click, the toast,
no chip, the file in the log (`552-04-opencode-written`); the file's first line set to an older
version by the scenario, Ctrl-D and `opencode` again, "Update Marley's plugin for OpenCode" and its
click (`552-05-opencode-update`); in a terminal not in front, `node` loading the written plugin and
handing its hook a `session.idle` event: one banner `OpenCode` / `repo finished`, then with
`TERM_PROGRAM` unset none (the private bus's log; `552-06-notified`).

## Locked-In Decisions
- D1: Codex is configured, not wrapped: its TUI already notifies through the terminal; Marley
  sets `notifications`, `notification_condition = "always"` (Marley decides focus itself, #478)
  and `notification_method = "osc9"` (Codex's `auto` need not know `TERM_PROGRAM=zed`). Nothing
  else in the file is touched, and comments survive (`toml_edit`).
- D2: OpenCode gets a file, not a package: `~/.config/opencode/plugins/marley.js`, written whole
  by Marley and versioned in its first line, so an update is a rewrite and the chip can tell.
- D3: Both stay silent outside Marley: Codex's OSC 9 goes to whatever terminal runs it (a
  terminal that ignores OSC 9 shows nothing, and `always` costs nothing there); the OpenCode
  plugin checks `TERM_PROGRAM` as `notify.sh` does.
- D4: The plugin writes to the controlling terminal, `/dev/tty` (changed at promotion): OpenCode
  may run plugins in a server process whose stdout is not the terminal, and a process with a
  controlling terminal reaches it through `/dev/tty` whatever its stdout is; with none
  (`opencode serve`), the open fails and the hook stays silent. No live run of the real CLIs: a
  run reaches the network and an account, which neither a scenario nor the Plan may, so the
  stand-ins speak the documented behaviour and the real launch is Chad's to see.
- D5: The chips read Codex's `CODEX_HOME` and the XDG config home, so a scenario points both at
  scratch folders and Chad's own files are never touched by a run.
- D6: Fixed sentences per event, as the Claude plugin's: "<project> finished", "<project> needs
  your permission", "<project> hit an error"; #538 words them later.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Codex runs in a terminal and its `config.toml` lacks any of the three keys under `[tui]`, the agent bar shall show "Turn on Codex notifications". | Shot `552-01-codex-chip` |
| REQ-002 | WHEN the user clicks it, the system shall write the three keys under `[tui]`, keep the file's other keys and comments, show a toast naming the restart, and drop the chip. | Shot `552-02-codex-written`; the log's file |
| REQ-003 | WHILE OpenCode runs in a terminal and the plugin file is missing, the agent bar shall show "Connect OpenCode to Marley". | Shot `552-03-opencode-chip` |
| REQ-004 | WHEN the user clicks it, the system shall write the plugin file with the shipped version in its first line, show the toast, and drop the chip. | Shot `552-04-opencode-written`; the log's file |
| REQ-005 | WHILE the plugin file names an older version than Marley ships, the bar shall offer the update, and a click shall rewrite the file. | Shot `552-05-opencode-update` |
| REQ-006 | WHEN the plugin's hook receives `session.idle`, `permission.asked` or `session.error` with `TERM_PROGRAM` set to `zed`, it shall write one OSC 777 notify to stdout, and with the variable unset it shall write nothing. | Shot `552-06-notified`; the log's `busctl` record and the stand-in's captured output |
| REQ-007 | WHEN Codex's or the plugin's notify reaches a Marley terminal that is not the focused one, the system shall show a desktop notification whose click shows that terminal. | Shot `552-06-notified` (#478's path, unchanged) |
| REQ-008 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, the live checks before any code: `codex` 0.155 started
  in a pty with the three keys, what it prints at start and, if a model is reachable, at a turn's
  end; `opencode` 1.18 with the plugin file, its log showing the plugin loaded and, on a fake
  event, the OSC on its stdout. If OpenCode's default launch does not put the plugin's stdout on
  the terminal, split the OpenCode half out (see the report) and ship Codex's chip.
- **P2 Code:** no Zed path; `agent_plugins/opencode/marley.js`, `codex_config.rs` and
  `opencode_plugin.rs` in `marley_workbench` (the checks, the writes, the version), the two chips
  in `agent_bar.rs`, `toml_edit` in `crates/marley_workbench/Cargo.toml`; fmt and clippy clean; a
  review of the diff.
- **P3 Test:** write and run the scenario and read every shot; the live checks recorded;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley/three-prong-plan.md` (T7b's status);
  `docs/marley_architecture/marley_workbench.md`; the ledger capture; close the ticket, archive,
  commit.
