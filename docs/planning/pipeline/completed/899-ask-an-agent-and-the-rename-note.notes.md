# Ask an agent to fix this, and the inline rename note — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-899-agent-entries-in-the-editors-pop-ups.md
- **Pipeline spec:** 899-ask-an-agent-and-the-rename-note.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row after #856.
- **Classification / tier:** feature, M. Two small Zed hunks and a Marley action.
- **Recall (§18.3):** Send Selection has the agent targets, the reference and the picker; #856's
  heading global is the place for the footer. A probe showed inline assist needs an LLM provider
  the run lacks: split to #900. The brain had nothing.
- **Decisions:** D1 in the spec.

## Phase 2 — Code
- `marley::AskAgentToFix` (`marley_workbench.rs`), `send_selection::ask_to_fix` and
  `line_and_problem`: the caret's line from the newest selection, the worst diagnostic on that
  line (its first line), "Fix this: <reference> (<problem>) " typed with no Enter (D1), the picker
  for several agents.
- `MarleyCodeActionsLook` now carries `footer: fn(&App) -> AnyElement`; the menu draws it after
  the list. `search_looks::code_actions_footer` closes the menu (`editor::actions::Cancel`) and
  dispatches `AskAgentToFix`.
- `editor.rs`: `MarleyRenameLook`; the rename block draws "Renaming: Old → New · Enter" at its
  right, the new name read from the rename editor each frame.
- Found in Test and fixed: the footer's first click fell through to the editor (the caret moved,
  the menu closed, nothing sent): the menu's list occludes the editor but the footer sat outside
  it. The footer occludes now (F-claude-899-a-popover-row-outside-its-list-let-the-click-through-001).
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/899-ask-an-agent-and-the-rename-note.sh` (sway, rust-analyzer, a stand-in
Claude Code idle through the plugin's hook).
- `899-01-actions`: Ctrl+. on `wid`: "Code actions Ctrl-.", "Add field to Record Struct", then
  "Ask an agent to fix this…" in the accent (REQ-001).
- `899-02-sent`: the terminal tab in front, its prompt holding "Fix this: @src/main.rs#L17 (no
  field `wid` on type `Rail`)" with no Enter (REQ-001).
- `899-03-rename`: F2 on `Rail`, "Board" typed: the rename editor under the line and "Renaming:
  Rail → Board · Enter" at the right in the accent (REQ-002).
- `899-04-day`: the same in Marley Day (REQ-002).
- The first run's rename step pressed Ctrl+P with the terminal focused and made a new
  `main.rs`; the step clicks the file's tab now.

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, the ledger rows, look-plan W6.
- Knowledge: F-claude-899-a-popover-row-outside-its-list-let-the-click-through-001.
- Brain decision recorded on consultation 80efb6c3.
