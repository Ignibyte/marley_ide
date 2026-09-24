---
pipeline_id: 78e76432-3cfb-46e4-952a-8205cbefc4ff
ticket: docs/planning/tickets/open/TICKET-482-claude-code-notifications-chip.md
status: QUEUED — Phase 1 Plan drafted; ready to promote
title: "Enable Claude Code notifications: Marley's plugin for Claude Code"
type: feature
slice: prong 1 T7b
references: [docs/planning/pipeline/queued/478-terminal-notifications.spec.md, docs/planning/pipeline/completed/477-agent-bar.spec.md]
---

## Title
A chip in the agent bar that installs Marley's plugin for Claude Code, whose hooks make Claude
Code send the notifications #478 shows.

## Scope
### In
- **The plugin**, written by Marley under its data directory as a local marketplace:
  `.claude-plugin/marketplace.json` naming the plugin `marley`, and the plugin with its
  `plugin.json` and `hooks/hooks.json`. `Notification` hooks for `permission_prompt`,
  `idle_prompt` and the agent-view matchers, and a `Stop` hook, each answer with JSON whose
  `terminalSequence` is an OSC 777 notify with a fixed title and body (the project folder's
  name, stripped of control characters and quotes), and only when `TERM_PROGRAM` is `zed`, as in
  every Marley terminal.
- **The chip**, "Enable Claude Code notifications", in the agent bar (#477) while Claude Code is
  the agent and `~/.claude/plugins/installed_plugins.json` has no `marley@marley`. A click writes
  the plugin, then runs `claude plugin marketplace add <dir>` and
  `claude plugin install marley@marley` through `smol::process::Command`, with `claude` from
  the PATH; a failure shows in the workspace as an error. Sessions started after the install use
  it.

### Out (explicitly deferred)
- Dismissing the chip for good; uninstalling from Marley (`claude plugin` can).
- Codex's and Gemini's own hooks.

## Reference (§20)
- **Warp:** "Enable Claude Code notifications" installs Warp's plugin
  (`claude plugin marketplace add warpdotdev/claude-code-warp`,
  `claude plugin install warp@claude-code-warp`;
  https://docs.warp.dev/agents/cli-agents/claude-code/). Neither Warp's code nor its plugin was
  read.
- **Upstream Zed:** none; Zed's own agent threads notify through the Agent Panel.

### Prior art
- **Published material:** Claude Code's hooks (`Notification` with its matchers, `Stop`), the
  hook answer's `terminalSequence` (OSC 0, 1, 2, 9, 99, 777 and BEL only), plugins and local
  marketplaces (`claude plugin marketplace add <path>`), `installed_plugins.json` keyed
  `name@marketplace`.
- **Code we already ship:** #478's OSC 777 notifications; #477's agent bar; `which` for finding
  `claude`, as `agents.rs` does; `paths::data_dir`.

## UI proof
UI-AFFECTING: a chip in the agent bar.
- **Driven tests** (`marley_workbench`): with a fake `claude` in the foreground and no
  `installed_plugins.json`, the chip shows; a click writes the plugin and runs a fake `claude`
  on the PATH, which logs `plugin marketplace add <dir>` and `plugin install marley@marley`;
  with the plugin listed, no chip.
- **Unit tests:** the plugin's files (valid JSON, the matchers); each hook command run with and
  without `TERM_PROGRAM=zed`.
- **Live drive:** the chip in the bar under a stand-in `claude`; the install itself changes
  Chad's Claude Code, so it is not run live.

## Locked-In Decisions
- D1 — A plugin, not `preferredNotifChannel`, which is global and would reach every terminal.
- D2 — Fixed messages per event, so a hook needs no JSON parsing and no `jq`.
- D3 — The hooks stay silent outside Marley, so the plugin is harmless in other terminals.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Claude Code runs and Marley's plugin is not installed, the agent bar shall show "Enable Claude Code notifications" | driven |
| REQ-002 | WHEN the chip is clicked, Marley shall write the plugin and run `claude plugin marketplace add` and `claude plugin install marley@marley` | driven |
| REQ-003 | WHILE the plugin is installed, the chip shall not show | driven |
| REQ-004 | The plugin's hooks shall answer with an OSC 777 `terminalSequence` where `TERM_PROGRAM` is `zed`, and with nothing elsewhere | unit |
| REQ-005 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; promotion confirms the plugin and hook schemas against Claude Code's
  docs and asks the brain.
- **P2 Code** — the plugin writer, the chip, the installer.
- **P3 Test** — unit and driven tests, negative checks, a capture, the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
