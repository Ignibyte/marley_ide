---
pipeline_id: 0132e14d-78c7-470d-9b44-9c97bd6126b1
ticket: docs/planning/tickets/open/TICKET-900-inline-assist-offers-the-agent.md
status: Phase 4 — Complete PASS
title: Inline assist offers to send the selection to an agent
type: feature
slice: The Marley look, W6 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
Under inline assist's prompt, a line offers the terminal agent the window runs: "Or send lines
20–22 to Claude Code in “Fix the rail test” · marley: send selection to agent", as the
EditorPopups board draws it. A click on it sends the selection as `marley: send selection to
agent` does.

## Scope
### In
- `inline_prompt_editor.rs` (Zed): a look hook that asks Marley for a line under a buffer's
  prompt, given the prompt's lines.
- `buffer_codegen.rs` (Zed): `BufferCodegen::range`, so the prompt reads its lines from the
  codegen rather than from the editor that is drawing it.
- `agent_ui.rs` (Zed): the hook's re-export.
- `send_selection.rs`, `search_looks.rs` (Marley): the line, from `agent_targets`.

### Out (explicitly deferred)
- The terminal's inline assist: the board draws the editor's.

## Reference (§20)
N/A — Marley-specific: Zed's inline assist (`agent_ui::inline_prompt_editor`) with Marley's send
of a selection (#549).

### Prior art
- `send_selection::agent_targets` and `send_selection` (#549); `MarleyThreadLook` (#864) and
  `MarleyCodeActionsLook` (#899) are the hook pattern.
- Zed's Ollama provider counts as configured once `GET /api/tags` lists a model; a stand-in
  server answering it and `POST /api/show` lets a run open inline assist.
- The brain (consultation 26d26b34): nothing on this seam.

## UI proof
Scenario `script/e2e/900-inline-assist-offers-the-agent.sh` (sway): a scratch Rust file; a
stand-in Ollama server and `agent.default_model` pointing at it; a stand-in `claude` in a terminal
of the window.
- `900-01-line`: three lines selected, inline assist open: the line names lines and the agent
  (REQ-001).
- `900-02-sent`: the line clicked: the stand-in claude's prompt holds the reference (REQ-002).
- `900-03-day`: the prompt and its line in Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The line shows only while an agent runs in a terminal of the window; with several it
  names how many, and the click opens the picker as the action does.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE an agent runs in a terminal of the window, inline assist's prompt in a file shall show a line naming the selected lines, the agent and `marley: send selection to agent`, in Marley Night and Marley Day. | `900-01-line`, `900-03-day` |
| REQ-002 | WHEN the user clicks the line, Marley shall send the selection to that agent as `marley: send selection to agent` does. | `900-02-sent` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
