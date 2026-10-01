---
pipeline_id: 5e00f997-355e-4943-9056-2b7e36afe8a5
ticket: docs/planning/tickets/open/TICKET-626-a-commands-colours-at-the-prompt.md
status: Phase 4 — Complete PASS
title: "A command's colours at the prompt"
type: feature
slice: prong 1 T6
references: [docs/marley/three-prong-plan.md]
---

## Title
The command typed at the prompt is drawn in the theme's syntax colours: in the prompt editor through
Zed's bash language, and on the shell's own prompt by colouring the typed range when the grid is
painted (plan T6, second half).

## Scope
### In
- The prompt editor (#624) takes the bash language from the language registry.
- `crates/terminal_view/src/terminal_element.rs` (Zed crate, an additive `// Marley:` hunk in
  `layout_grid`, before `cell_style`): a hook a Marley global answers with colour runs for the
  live prompt's typed range (from `note_input`'s start to the cursor), computed in
  `crates/marley_workbench` by highlighting the typed text with tree-sitter-bash and the theme's
  syntax colours. The grid's cells are not changed; readline's redraws stay the shell's.

### Out (explicitly deferred)
- Colouring a finished block's command line (its PS1 and command are history).
- Programs other than the shell at a prompt.

## Reference (§20)
- **Warp (behavior):** the input's command is highlighted as it is typed
  (`docs/warp_architecture/crates/syntax_tree.md`: incremental tree-sitter highlighting over the
  input buffer; marked "not built" in Marley).
- **Upstream Zed:** the `languages` crate's bash grammar and highlights (`crates/grammars/src/bash/
  highlights.scm`), the theme's syntax colours; `terminal_element::layout_grid`.

### Prior art
- **Behavior maps:** the note above.
- **Published material:** tree-sitter-bash's highlight queries.
- **Code we already ship:** tree-sitter-bash (`crates/grammars`, registered in
  `crates/languages/src/lib.rs`); `layout_grid` and `cell_style` (`terminal_element.rs`);
  `AnchoredBlocks::note_input` and `at_prompt` (#484). Writing colours into alacritty's cells
  would race readline's redraws, so the colour is a paint-time override.

## UI proof
`script/e2e/626-a-commands-colours-at-the-prompt.sh`: bash at a prompt.
- `prompt.png`: `echo "hi" | grep h $HOME` typed at the shell's prompt, the command words, the
  string and the variable in different theme colours;
- `editor.png`: the same line in the prompt editor (#624), coloured alike;
- `ran.png`: after Enter, the finished block's command drawn as the shell left it.

## Locked-In Decisions
- D1 — Colours at paint time over the typed range, never in the grid's cells.
- D2 — Zed's bash grammar and the theme's colours, so a theme change recolours both.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the user types a command at the shell's prompt, Marley shall draw its words in the theme's syntax colours. | Shot `prompt.png` |
| REQ-002 | WHILE the prompt editor holds a command, it shall colour it alike. | Shot `editor.png` |
| REQ-003 | WHEN the command runs, the block shall keep the shell's own drawing. | Shot `ran.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the editor's language, the paint hook and its row, the highlighter; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
