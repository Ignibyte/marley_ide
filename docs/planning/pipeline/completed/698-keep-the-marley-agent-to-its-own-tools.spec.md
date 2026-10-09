---
pipeline_id: 3437ea50-86c6-423e-8db0-1a985d8ce026
ticket: docs/planning/tickets/open/TICKET-698-keep-the-marley-agent-to-its-own-tools.md
status: Phase 4 — Complete PASS
title: Keep the Marley agent to its own tools on Claude Code and Codex
type: bug
slice: the Marley agent (docs/planning/intake/marley-agent-manager-foreman.md), found while planning #696
references:
  - docs/planning/pipeline/completed/687-the-marley-agent-on-codex-and-zeds-agent.spec.md
  - docs/planning/pipeline/completed/696-the-marley-and-rusty-agents-out-of-the-box.spec.md
---

## Title
The Marley agent edits no file and runs no command, its instructions say; on Claude Code it can
still call Marley's `terminal_run`, `terminal_type` and the browser's write tools, since Zed hands
every agent the whole `marley` server. Its sessions now disallow every Marley tool but its eight.

## Scope
### In
- The Marley entry's Claude Code sessions: `disallowedTools` gains `mcp__marley__<tool>` for every
  tool in `marley_mcp`'s registry outside the eight the Marley profile turns on, built from the
  registry so a new tool is covered.
- Codex: find whether `mcp_servers.marley.disabled_tools` in `CODEX_CONFIG` holds against the
  server ACP passes.

### Out (explicitly deferred)
- Codex, found not to hold (see Prior art): a limit there needs Marley's server to tell the
  Marley entry's calls from other agents'; a ticket of its own.
- `marley: open marley agent in terminal` (#684), Claude Code in a terminal, which the ticket does
  not name.

## Reference (§20)
N/A — Marley-specific: the Marley agent's limits on Claude Code's ACP adapter and Codex's. Kept
from them: Claude Code's `disallowedTools` names an MCP tool as `mcp__<server>__<tool>` (as #696's
`mcp__marley` names a whole server).

### Prior art
- **The code we ship:** `marley_mcp::registry::registry()`, the single table of Marley's tools,
  and `ToolSpec::name`; `assistant::{session_meta, PROFILE_TOOLS, DISALLOWED_TOOLS}`.
- **codex-acp** (zed-industries/codex-acp, `src/codex_agent.rs`, `build_session_config`, read on
  2026-10-09 at main 296069e8): each server ACP passes is inserted into the session's
  `mcp_servers` by name with `disabled_tools: None` and `enabled_tools: None`. That replaces any
  entry of the same name the config held, so a `CODEX_CONFIG` `mcp_servers.marley.disabled_tools`
  can't hold for the `marley` server Zed passes.
- **Behavior maps:** nothing on agents' tool limits.

## UI proof
`script/e2e/698-keep-the-marley-agent-to-its-own-tools.sh`, under `compositor sway`, with a scripted
ACP agent in the adapter's place (`MARLEY_ASSISTANT_ADAPTER`) that logs what it reads.

Shots:
- `698-01-marley-thread`: a Marley thread on the scripted agent; its `session/new` `_meta` holds
  the limits (the scenario's check prints them).

## Locked-In Decisions
- **D1:** the list is the registry's tools, served or not, minus `PROFILE_TOOLS`, each as
  `mcp__marley__<name>`; a tool added to the registry is disallowed until it joins the profile.
- **D2:** Codex gets no `mcp_servers` entry: it would not hold, and nothing is sent that only
  looks like a limit.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Marley thread starts on Claude Code, its session `_meta` shall disallow every Marley tool outside its eight, `terminal_run` and `terminal_type` among them. | Shot 698-01; the scenario's check of the logged `_meta` |
| REQ-002 | WHEN a Marley thread starts on Claude Code, its session `_meta` shall disallow none of its eight tools. | The same check |
| REQ-003 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `assistant.rs`; a review of the diff; the gate.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG, architecture docs, ledger, close, archive, commit.
