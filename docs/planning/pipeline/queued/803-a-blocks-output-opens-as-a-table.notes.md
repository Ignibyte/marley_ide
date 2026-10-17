# A block's output opens as a table — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-803-a-blocks-output-opens-as-a-table.md
- **Pipeline spec:** 803-a-blocks-output-opens-as-a-table.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see the ticket).
- **Research:** `docs/planning/design-notes/terminal-output-as-components-2026-10-17.md`.
- **Discovery (drafted by /spec; re-verified at promotion):** `AnchoredBlock` and
  `AnchoredBlocks` (`crates/marley_terminal/src/anchored.rs`); `Terminal::block_output`,
  `block_output_kept` (`crates/terminal/src/terminal.rs`); `absolute_lines_text`
  (`crates/terminal/src/alacritty.rs`); `MarleyBlockChip` (`crates/terminal_view/src/terminal_view.rs`,
  set in `crates/marley_workbench/src/bookmarks.rs`); the block menu
  (`crates/marley_workbench/src/blocks.rs`); `TableView::new`, `set_contents`,
  `TableCell::Generated`, `TabularDataPreviewPane::open_preview_to_the_side_of_pane`
  (`crates/tabular_data_preview`). `marley_workbench` does not depend on `tabular_data_preview`
  yet.
