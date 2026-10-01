# A command's colours at the prompt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-626-a-commands-colours-at-the-prompt.md
- **Pipeline spec:** 626-a-commands-colours-at-the-prompt.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, second batch (#624 to #627): T3 and T6, the prompt.
- **Recall (§18.3):**
  - Changing alacritty's cell colours races readline's redraws; a paint-time override in `layout_grid` does not.
  - tree-sitter-bash and its highlights are already in the tree.
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted;** seams re-verified: `TerminalElement::layout_grid` (static, two calls in prepaint)
  computes each cell's `TextRun` with `cell_style` and already takes one ranged style (the hovered
  link); `Language::highlight_text(&rope, range)` gives `(Range, HighlightId)` runs, and the theme's
  `syntax().get(id)` their `HighlightStyle`; the bash language is registered as "Shell Script";
  `autosuggest::typed_text` and `AnchoredBlocks::input_start` give the typed line and its first
  column.
- **Recall:** the queued notes stand. The brain (`rusty-cli brain ask`, consultation 21314470fe984942b40021f8445222ab):
  nothing on this seam.

### Design
- **`crates/terminal_view/src/terminal_view.rs`** (Zed crate): `MarleyPromptColors { line, runs }`
  and the global hook `MarleyPromptColoring`.
- **`crates/terminal_view/src/terminal_element.rs`** (Zed crate): prepaint asks the hook once a
  frame; `layout_grid` takes the answer and sets the colour of a cell on that line inside a run.
- **`crates/marley_workbench/src/prompt_colors.rs`** (new): loads "Shell Script" from the first
  project's registry into a global; the hook highlights the typed line and maps each run to its
  columns from the input's start, in the theme's syntax colours.
- **`crates/marley_workbench/src/rich_input.rs`:** the shell's editor's buffer takes the language.
- **Ledger:** the two `terminal_view` rows.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | bash, `echo "hi" \| grep h $HOME` typed at the prompt | `prompt.png` |
| REQ-002 | Ctrl+G, the editor holding it | `editor.png` |
| REQ-003 | Enter | `ran.png` |

## Phase 2 — Code (2026-09-30)
- **Built:** `MarleyPromptColors` and `MarleyPromptColoring` (`terminal_view.rs`); in
  `terminal_element.rs`, `layout_grid`'s body moved into `marley_layout_grid` with the colours as
  one more argument, `layout_grid` calling it with none, prepaint asking the hook and the two
  calls going through it; `prompt_colors.rs` (new); the shell editor's language in `rich_input`;
  the ledger clauses.
- **Deviation:** the plan added a parameter to `layout_grid`; Zed's `repl` crate calls it, so the
  signature stays and a private twin takes the colours (no `repl` hunk).
- **Clippy found:** imports (`Rope` from `language`, `ActiveTheme` from `ui`) and `or_fun_call`.
- **The gate found:** shellcheck SC2016 on the scenario's single-quoted `$HOME`; escaped in double
  quotes. GREEN after.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/626-a-commands-colours-at-the-prompt.sh` (sway). One run.
- **Shots:**
  - `prompt.png` (REQ-001): `$ echo "hi" | grep h $HOME` at the shell's prompt: `echo` and `grep`
    blue, `"hi"` green, the pipe and `$` in their own colours, `h` plain.
  - `editor.png` (REQ-002): the shell's editor holding the same line in the same colours.
  - `ran.png` (REQ-003): after Enter, the block's command line drawn plain, as the shell drew it,
    and grep's message (a folder, exit 2); the new prompt empty.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide; `marley_workbench.md`; the plan's T6 row; the ledger.
- **Knowledge:** L-claude-626-a-zed-function-with-an-outside-caller-gets-a-marley-twin-001,
  L-claude-626-type-a-literal-dollar-without-a-shellcheck-disable-001.
- **Brain:** the consultation closed with `brain decide`.

