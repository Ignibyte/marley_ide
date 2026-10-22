---
pipeline_id: 9cd98b17-cde6-4f0f-98c9-292201b45763
ticket: docs/planning/tickets/closed/TICKET-899-agent-entries-in-the-editors-pop-ups.md
status: Phase 4 — Complete PASS
title: Ask an agent to fix this, and the inline rename note
type: feature
slice: The Marley look, W6 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
The code actions menu ends with "Ask an agent to fix this…", which sends the caret's line, with
the diagnostic under the caret, to an agent in a terminal of the window, as Send Selection does;
and inline rename shows "Renaming: Old → New · Enter" at its line's end.

## Scope
### In
- `send_selection.rs`: `marley::AskAgentToFix`: "Fix this:", the line's reference, and the
  diagnostic's message, to the one agent or through the picker.
- `crates/editor/src/code_context_menus.rs` (Zed crate): `MarleyCodeActionsLook` gains a footer
  the menu draws after its actions.
- `crates/editor/src/editor.rs` (Zed crate): `MarleyRenameLook`, a global; with it set the rename
  block draws the note at its right.
- `search_looks.rs`: the footer and the rename look.

### Out (explicitly deferred)
- Inline assist's line: TICKET-900 (a run has no LLM provider to open it).

## Reference (§20)
Upstream Zed for the menus and the rename block (`editor::code_context_menus`,
`Editor::rename`); Marley's Send Selection (#522, #719) for the send.

### Prior art
- `send_selection::send_selection`, `agent_targets`, `reference`, `send_text`.
- #856's `MarleyCodeActionsLook` heads the menu.
- The rename block draws a one-line editor at the old name's x.
- The brain (consultation 80efb6c3): nothing on this seam.

## UI proof
Scenario `script/e2e/899-ask-an-agent-and-the-rename-note.sh` (sway): #856's scratch crate, and
a stand-in Claude Code in the window's terminal.
- `899-01-actions`: Ctrl+. on the error: the actions, then "Ask an agent to fix this…" (REQ-001).
- `899-02-sent`: the entry clicked: the terminal holds "Fix this:", the line and the error
  (REQ-001).
- `899-03-rename`: F2 on `Rail`, "Board" typed: "Renaming: Rail → Board · Enter" (REQ-002).
- `899-04-day`: Marley Day, the rename (REQ-002).

## Locked-In Decisions
- D1 — The fix request is typed, not submitted: the user reads it and presses Enter, as Send
  Selection leaves its reference.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The code actions menu shall end with "Ask an agent to fix this…", which types "Fix this:", the caret's line and the diagnostic there into an agent's terminal. | `899-01-actions`, `899-02-sent` |
| REQ-002 | WHILE an inline rename is open, the line shall show "Renaming: Old → New · Enter", in Marley Night and Marley Day. | `899-03-rename`, `899-04-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
