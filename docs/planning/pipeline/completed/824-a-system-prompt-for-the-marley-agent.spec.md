---
pipeline_id: 606a25d0-cfda-493d-a6d3-b80ea66656d4
ticket: docs/planning/tickets/closed/TICKET-824-a-system-prompt-for-the-marley-agent.md
status: Phase 4 — Complete PASS
title: A system prompt for the Marley agent
type: feature
slice: Marley's own agent (#683, #687, #696, #698)
references: []
---

## Title
Marley's own agent gets a real system prompt: its identity, its jobs and tools, how it answers and
what it hands off; Rusty's shares the answering part (Chad, 2026-10-20: "we should build a good
system prompt for marley").

## Scope
### In
- `assistant.rs`'s `INSTRUCTIONS` rewritten in four parts: who it is, what it does (the tools it
  has, unchanged), how it answers, and what it leaves to others.
- The answering part shared with `RUSTY_INSTRUCTIONS` through one macro, so the two read alike.
- The phrases older scenarios check stay: "Marley's own agent", "You are Rusty", "You do not edit
  files, run commands or change code here".

### Out (explicitly deferred)
- The MCP server's `INSTRUCTIONS` that every agent gets (#680): a tool guide under 2 KB, not a
  persona; unchanged.
- A setting to edit the prompt, and the ChatGPT-like look of the thread (the design canvas of the
  same day).

## Reference (§20)
N/A — Marley-specific: the text Marley appends to its own agent's system prompt. Upstream Zed's
agent sends a `systemPrompt.append` from the session's `_meta` (Claude Code's adapter) and Codex
its `developer_instructions`; both paths stay as #683 and #687 built them.

### Prior art
- `assistant.rs` already hands the text to all three paths: the Agent Panel's `_meta`
  (`systemPrompt.append`), Codex's `developer_instructions`, and the terminal's
  `--append-system-prompt-file`. Only the text changes.
- The no-ai-slop rules and Chad's own preferences (answers first, one question at a time) are
  the source of the answering part.

## UI proof
`script/e2e/824-a-system-prompt-for-the-marley-agent.sh`, `compositor sway`, #798's set-up (a
scripted ACP agent in the adapter's place): a Marley thread started from the repo row's +.
- `824-01-thread` — the thread answering; the run reads the scripted agent's log and checks the
  `session/new` `_meta` carries the new prompt's four headings (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — The prompt stays in the binary, not a file the user edits: the agent's limits (no edits,
  the disallowed tools) are part of the same contract.
- D2 — The answering part is one text shared by both entries.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Marley thread starts, its `session/new` shall carry the prompt with its four parts: who it is, what it does, how it answers, what it leaves to others. | `824-01-thread`, the run's check |
| REQ-002 | The prompt shall keep "Marley's own agent" and "You do not edit files, run commands or change code here", and Rusty's "You are Rusty", for the older scenarios. | review of the diff |
| REQ-003 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec and the notes. **P2 Code** — the text; the gate. **P3 Test** — the
  scenario. **P4 Complete** — CHANGELOG, guide, close, archive, commit.
