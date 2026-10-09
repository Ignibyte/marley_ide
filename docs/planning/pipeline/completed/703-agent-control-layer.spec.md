---
pipeline_id: 2cac49dd-1252-480c-b9e0-1f5c17059b03
ticket: docs/planning/tickets/open/TICKET-703-agent-control-layer.md
status: Phase 4 — Complete PASS
title: The agent-control layer: an activity log and a kill switch
type: feature
slice: Marley's MCP server (docs/planning/intake/zed-control-over-mcp.md), the first of #703 to #707
references:
  - docs/planning/intake/zed-control-over-mcp.md
---

## Title
Before Marley's MCP server gains tools that drive Zed itself (#704 to #707), every write tool an
agent calls is logged where the user can see it, and one switch stops them all.

Chad, 2026-10-09: "yes i would love to have full control over the zed ide. We do need to think
about security across this too … We dont want restrictive approve every single thing that marley
does … But any sort of security would be ideal".

## Scope
### In
- **The activity log.** Every call of a write-tier tool (`terminal_type`, `terminal_run`, the
  browser's write tools, `settings_change`, `keymap_change`, `seat_add`, and every later one)
  makes a row:
  - when, and who (`click_pause::Who`'s words: Claude Code, Zed's agent, a client's name);
  - the tool, a one-line summary of what it acts on (the command, the URL, the key), redacted;
  - its outcome, done or refused with the refusal's code.

  Rows are appended to `<data dir>/agent_control/activity-<day>.jsonl` (0600) and kept in memory.
  Today's are read back at start.
- **The kill switch,** `marley.agent_control.stopped`.
  - On, every write-tier tool refuses with `agent_control_stopped` and the next step (how to
    resume), and the refusal is logged. Read tools keep working.
  - `marley: stop agent control` and `marley: resume agent control` set it, and so do the Home
    card's button and the Settings page's toggle.
- **The Agent Activity tab** (`marley: open agent activity`, in the Home group). It shows the
  switch's state with a Stop or Resume button, and the rows, newest first, refusals marked.
- **A card on Home's page** (#701), AGENT ACTIVITY: the switch's state and button, the five newest
  rows, and Open Activity.

### Out (explicitly deferred)
- The per-area modes and the once-per-session question move to #704, whose `editor_open` is the
  first tool to use them. A setting ships with the tool it governs.
- The rail mark on the acting agent's row: a follow-up ticket.
- Pruning old day files.

## Reference (§20)
N/A — Marley-specific: Marley's own MCP server and its agents. Upstream Zed has no
agent-activity log or kill switch for context servers; its tool permissions decide per tool and
log nothing the user can browse. No Warp analog read.

### Prior art
- **The code we ship:**
  - `marley_system_one::files` keeps a day log as JSON lines in the data dir: `append_in` (0600,
    append-only) and `read_day_in` (a missing file is empty, bad lines are skipped). Its pattern is
    copied here, since its functions are typed to System One's rows.
  - `click_pause::Who::of` sorts callers into words.
  - `mcp::model_redactor` masks keys and tokens whatever the agents' redaction says.
  - `marley_mcp::lookup` gives a tool's `Tier`. `mcp::answer` is the app's one chokepoint for
    every tool the app answers.
  - `settings::update_settings_file` writes a user setting, as `settings_change` does.
  - Home's page builds cards with `rusty::home_tab::card` and `row`.
- **Behavior maps:** none on this seam in `docs/zed_architecture/`.
- **Published:** MCP's tool annotations (`readOnlyHint`, `destructiveHint`) describe tools to
  clients. They gate nothing on the server, so the tier stays Marley's registry.

## UI proof
`script/e2e/703-agent-control-layer.sh`, under `compositor sway`, with #491's scripted MCP client
reading the profile's `mcp-endpoint.json`.

Shots:
- `703-01-activity`: the client's `terminal_run` ran; the Agent Activity tab lists it, done.
- `703-02-stopped`: after `marley: stop agent control`, the tab says agents' write tools are
  stopped, with Resume.
- `703-03-refused`: the client's next `terminal_run` was refused (its reply checked), while its
  `terminal_list` still answered; the tab lists the refusal.
- `703-04-home-card`: Home's page, its AGENT ACTIVITY card with the state, the rows and the
  button.

## Locked-In Decisions
- **D1:** the switch is a setting (`marley.agent_control.stopped`), so it survives a restart and
  shows in Settings; the actions write it.
- **D2:** the switch is checked in `mcp::answer` from the tool's tier, so every write tool,
  present and future, is covered without each one knowing.
- **D3:** an answer's outcome reaches the log through a hook on `marley_mcp::AppCall`, set before
  routing, so tools that answer from their own tasks are logged too.
- **D4:** the summary is redacted with `model_redactor`, which is always on, and cut to a line.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls a write-tier tool, the system shall add a row with the caller, the tool, a summary and the outcome to the activity log and the Agent Activity tab. | Shot 703-01; the day file |
| REQ-002 | WHILE `marley.agent_control.stopped` is on, every write-tier tool shall refuse with `agent_control_stopped`, and read tools shall keep answering. | The client's replies; shot 703-03 |
| REQ-003 | WHEN the user runs `marley: stop agent control` or `marley: resume agent control`, the system shall set the switch, and the tab and the Home card shall show the state with the other button. | Shots 703-02, 703-04 |
| REQ-004 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the setting and its page item, `agent_activity.rs` (log, switch, tab, actions), the
  `AppCall` hook, the check and the hook in `mcp::answer`, Home's card; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
