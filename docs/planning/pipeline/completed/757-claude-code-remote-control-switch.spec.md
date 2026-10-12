---
pipeline_id: 7194a006-da52-4d6c-8963-46c186795aa2
ticket: docs/planning/tickets/open/TICKET-757-claude-code-remote-control-switch.md
status: Phase 4 — Complete PASS
title: Claude Code's Remote Control switch
type: feature
slice: the phone path (docs/t3code_architecture/README.md item 13; report 04 item 2), shelved
references:
  - docs/planning/pipeline/completed/535-phone-push-notifications.spec.md
  - docs/planning/pipeline/completed/532-agent-permission-modes.spec.md
---

## Title
A switch that starts Claude Code with `--remote-control`, so the session shows on the Claude phone
app, with a chip on its row and Marley's own pushes held back for it. Chad, 2026-10-11: "16.) yes
and shelve".

## Scope
### In
- **`marley.claude_code_remote_control`**, a boolean, `false` by default.
- **The launch line.** While the setting is on, `marley_agent::command` adds `--remote-control` to
  Claude Code's line. That reaches every Claude Code line Marley builds: the rail's +, Home, worktree
  agents, English, launch configs, New Agent, resume. When a prompt follows, the line writes `--`
  before it, so the prompt is never taken for the flag's optional session name.
- **The chip.** A row whose Claude Code's arguments hold `--remote-control` shows a `remote` chip
  beside the permission chip. Its tooltip reads "Remote Control: this session is on the Claude app".
- **Push.** `push.rs` skips the events of a terminal whose Claude Code runs with the flag. Claude's
  app pushes for it. Desktop notices stay.
- `docs/marley/guide.md`: the setting, and the note that the session goes through Anthropic's
  service.

### Out (explicitly deferred)
- The Marley agent's own line (`assistant.rs:1040-1047`) and harness seats: they build their own
  lines.
- A per-project value, as `agent_permissions_by_project` gives the permission modes.
- Reading Claude Code's own `/config` push switches.

## Reference (§20)
N/A — Marley-specific. Claude Code's flag is the vendor's own feature; Marley only adds the flag and
shows it.

### Prior art
- **The code we ship:**
  - `crates/marley_agent/src/marley_agent.rs`: `command` (293-312), where the bypass arguments go
    (305-310), and `line_with` (213-231), which writes `--` only before a prompt that starts with
    `-`.
  - `crates/marley_workbench/src/agents.rs`: `agent_line` (338-364) and `launch_mode` (368-378).
  - The chip: `rail.rs` `terminal_marks` (9091-9103), `permission_chip` (9024-9039) and `agent_mark`
    (7946-7967), reading the foreground's arguments through `marley_agent::permission_mark`
    (`marley_agent.rs:476`).
  - `push.rs`: `on_change` (53-103), which hard-codes Claude (89).
- **Behavior maps:** `docs/t3code_architecture/04-remote-and-mobile.md` §3 item 2 and open
  question 1.
- **Published material:** Claude Code 2.1.296's `--help`: `--remote-control [name]`, "Start an
  interactive session with Remote Control enabled (optionally named)", and
  `--remote-control-session-name-prefix`.

## UI proof
`script/e2e/757-claude-code-remote-control-switch.sh`, under `compositor sway`, with #535's fake
ntfy and its stand-in `claude`. The stand-in logs its arguments and acts out a session.

Shots:
- `757-01-chip`: with the setting on, New Agent starts Claude Code with a prompt. The row shows
  `remote`. The stand-in's log holds `--remote-control -- '<prompt>'`.
- `757-02-quiet`: the session finishes while another tab is shown. The fake ntfy logs nothing for
  it. A second session started after the setting is turned off pushes its line.

## Locked-In Decisions
- **D1:** off by default. The session goes through Anthropic's service, so it is Chad's choice per
  machine.
- **D2:** one place, `command`, so every Marley-built Claude Code line follows the setting.
- **D3:** `--` before a prompt while the flag is on, since the flag takes an optional name.
  - Rejected: `--remote-control=<name>`. A name per session is not asked for.
- **D4:** Marley's push skips a session with the flag, and desktop notices stay.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.claude_code_remote_control` is on, the system shall start Claude Code with `--remote-control`, with `--` before any prompt. | Shot 757-01 and the stand-in's log |
| REQ-002 | WHILE a row's Claude Code runs with `--remote-control`, the row shall show the `remote` chip. | Shot 757-01 |
| REQ-003 | WHEN such a session needs input, finishes or fails, the system shall push nothing through Marley's ntfy. | Shot 757-02 and the ntfy log |
| REQ-004 | WHILE the setting is off, the system shall start Claude Code as before. | Shot 757-02 and the stand-in's log |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `settings_content` (the setting; the existing `marley.rs` row) and
  `assets/settings/default.json`, `marley_agent` (`command`, `line_with`, the mark), `agents.rs`,
  `rail.rs`, `push.rs`, the guide; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
