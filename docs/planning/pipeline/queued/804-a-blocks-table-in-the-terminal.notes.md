# A block's table drawn in the terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-804-a-blocks-table-in-the-terminal.md
- **Pipeline spec:** 804-a-blocks-table-in-the-terminal.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see #803): "formatted automatically in a table".
- **Discovery (drafted by /spec; re-verified at promotion):** the row map
  (`crates/marley_terminal/src/display_rows.rs`, `RowMap::new`); its build and the cases that turn
  it off (`crates/terminal_view/src/terminal_element.rs`, prepaint); the header rows' element and
  the filtering of their cells; `marley_block` (the header); inline assist's `BlockProperties`
  (`crates/terminal_view/src/terminal_view.rs`). A tall pixel gap was ruled out: the map adds
  uniform space above rows only, and a gap pushes grid rows out of view.
