---
pipeline_id: bd7bc16b-fe35-4270-962e-858e1d703a46
ticket: forge#180 (81ab5a37-70df-4cea-8993-ee0bcb804f78) · local docs/planning/tickets/open/TICKET-180-agent-tail.md
aar_id: 8d34ce20-8dd8-4e60-ac2a-952bc5da42f9
status: Phase 5 — Complete PASS
title: M12 — Agents cockpit tab: a live output tail per agent
type: feature
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/agent_view.rs (PURE: agent_tail)
  - crates/marley_app/src/app.rs (SHIM: agent_tail_lines + the Agents-row tail render)
---

## Title
The Agents cockpit tab OBSERVES each agent: below its status row, a live multi-line tail of the agent's
recent output — watch progress without switching to the pane (the #173 pump keeps even background agents
fresh). The observe core of the chad-flagged M3/M5 thread.

## Scope
### In
- PURE `agent_view.rs`: `agent_tail(output, n) -> Vec<String>` — strip trailing BLANK lines, take the last
  `n`, `trim_end` each (indentation preserved); empty output → `[]`; `n > len` → all.
- SHIM `app.rs`: `agent_tail_lines(pane, n)` (grids().find_map(terminal) → content_row_texts join →
  agent_tail); the Agents row render stacks the tail (muted, smaller, indented) under each `agent_row_text`.
  Uniform N=6.

### Out
- A per-agent selection model for a "fuller focused tail" (the cockpit has no row selection yet — a
  follow-up); ANSI colour in the tail (plain text, like the last-line).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `agent_tail` shall return the last `n` output lines with trailing blanks stripped and indentation preserved; empty → []; n>len → all — mutation on the window + the strip. | unit |
| REQ-002 (visual) | WHEN an agent is running, its Agents-tab row shall show a multi-line tail of its recent output that updates as it streams. | driven (two frames) |
| REQ-003 | gate GREEN; the pure fn cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the pure fn + the shim helper + the row render. P3.5 self-review (the strip/window edges;
the cross-grid read; render cost). P4 unit + driven + gate. P5 docs.
