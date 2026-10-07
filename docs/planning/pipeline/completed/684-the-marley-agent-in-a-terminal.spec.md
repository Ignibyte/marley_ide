---
pipeline_id: aadb700b-4ecf-45e8-9c86-da6f4e417510
ticket: docs/planning/tickets/open/TICKET-684-the-marley-agent-in-a-terminal.md
status: Phase 4 — Complete PASS
title: The Marley agent in a terminal
type: feature
slice: prong 2 C; phase 1 item 6 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/pipeline/completed/683-the-marley-agent-in-the-agent-panel.spec.md
---

## Title
`marley: open marley agent in terminal`: the same Marley agent as #683, as Claude Code's own
interface in a Marley terminal tab, for someone who prefers it to the Agent Panel.

## Scope
### In
- The action, in the palette only while `marley.assistant.enabled` is on (hidden while off, as
  Rusty's commands are while Rusty is off, #661).
- It writes #683's instructions to `<data dir>/assistant/instructions.md` and starts, in a new
  center terminal of the active project, `claude --append-system-prompt-file <that file>
  --disallowedTools Bash Edit Write NotebookEdit MultiEdit`, through the path New Agent Thread's
  Claude Code takes (`agents::start_in_terminal`), so the terminal is an agent terminal with
  Marley's MCP server through Claude Code's Marley plugin.
- The instructions text is #683's, shared.

### Out (explicitly deferred)
- A button in the Agent Panel's Marley entry (the action is in the palette and can be bound to a
  key).
- Codex in a terminal (TICKET-687's).

## Reference (§20)
N/A — Marley-specific: Claude Code's own command-line flags (`--append-system-prompt-file`,
`--disallowedTools`, from `claude --help` 2.1.293) in a terminal Marley starts the way it starts
every agent CLI (#532).

### Prior art
- **Published material:** `claude --help` 2.1.293: `--append-system-prompt-file`,
  `--disallowedTools <tools...>`.
- **The code we ship:** `crates/marley_workbench/src/agents.rs` (`start_in_terminal`,
  `agent_line`, the agent terminal's environment); `rusty.rs:316` (`filter_palette`, a
  namespace and action types hidden while a layer is off); #683's `assistant.rs`
  (`INSTRUCTIONS`, `DISALLOWED_TOOLS`, `claude_program`).

## UI proof
`script/e2e/684-the-marley-agent-in-a-terminal.sh` (`compositor sway`). A fake `claude`
(`MARLEY_CLAUDE`) that writes its arguments to a file and prints them. With the switch off, the
palette lists no such command (`684-01-hidden`); on, the command opens a terminal tab running the
fake, which shows its arguments (`684-02-terminal`), and the file holds
`--append-system-prompt-file`, the instructions file's path and the five tools, and the
instructions file holds #683's text.

## Locked-In Decisions
- D1 — One instructions text for both #683 and this ticket, written to a file Claude Code reads.
- D2 — The terminal is an ordinary agent terminal (`AgentKind::Claude`), so the rail, the
  plugin's hooks and Marley's MCP server treat it as any Claude Code.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.assistant.enabled` is off, the command palette shall not list "marley: open marley agent in terminal". | `684-01-hidden` |
| REQ-002 | WHEN the user runs the command with the switch on, the system shall open a center terminal in the active project running `claude` with `--append-system-prompt-file <instructions>` and `--disallowedTools Bash Edit Write NotebookEdit MultiEdit`. | `684-02-terminal`; the fake's argument file |
| REQ-003 | The instructions file shall hold the Marley agent's instructions. | The file's text holds "You are Marley's own agent" |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the action in `assistant.rs`, the palette filter, the instructions file; the
  scenario; a review; `just gate-diff`.
- **P3 Test** — the scenario; every shot read.
- **P4 Complete** — docs, ledger, close, archive, commit, push, install.
