# Inline assist offers to send the selection to an agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-900-inline-assist-offers-the-agent.md
- **Pipeline spec:** 900-inline-assist-offers-the-agent.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "lets finish up the remaining tickets besides the mac issues".
- **Classification / tier:** feature, S. Marley crate plus three Zed files in `agent_ui`.
- **Recall (§18.3):** F-claude-865: a hook Zed calls from a render never reads the view that is
  rendering; the prompt is a block the editor draws, so its lines come from the codegen. #899's
  split note: a run has no LLM provider; the stand-in Ollama server answers that.
- **Design:** `MarleyInlineAssistLook { line: fn(&WeakEntity<Workspace>, Range<u32>, &App) ->
  Option<AnyElement> }`; the prompt computes the lines from `BufferCodegen::range` and the
  multibuffer's snapshot; Marley's line reads `agent_targets` and dispatches
  `SendSelectionToAgent` on click.

## Phase 2 — Code
- Zed: `BufferCodegen::range`; `MarleyInlineAssistLook` and its line in `PromptEditor::render`,
  for a buffer's prompt only; the re-export in `agent_ui.rs`. Three ledger rows.
- Marley: `send_selection::inline_assist_line` and `send_lines`; `search_looks::init` sets the
  look.
- Gate: `just gate-diff` 17 passed, 0 failed (after the Phase 3 fixes).

## Phase 3 — Test
- Scenario `script/e2e/900-inline-assist-offers-the-agent.sh` under sway, with #557's stand-in
  Ollama and claude. #899's note that a run has no LLM provider was out of date: #557's stand-in
  answers it.
- Probe: the line wrapped to two rows, since the stand-in's terminal title holds its script's
  newline. Fixed: the title's first line, cut at 32 characters, and the line truncates.
- First full run: the click sent `@src/main.rs` with no lines (F-claude-900-…). Fixed: the click
  sends the prompt's lines.
- `900-01-line` (REQ-001): under the prompt, "Or send lines 20–22 to Claude Code in “repo —
  python3 -u -c import sys”" and "· marley: send selection to agent".
- `900-03-day` (REQ-001): the same line in Marley Day.
- `900-02-sent` (REQ-002): the stand-in's terminal holds `@src/main.rs#L20-22` at its prompt.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md` (inline assist's line), look-plan EditorPopups, ledger rows.
- Knowledge: F-claude-900-inline-assist-leaves-the-editor-a-caret-001.
- Brain: decided on consultation 26d26b3451c341ab8a4b2a35ff1d31fd.
