# TICKET-804 — A block's table drawn in the terminal

- **Ticket:** LOCAL #804 (feature, the block terminal)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/completed/804-a-blocks-table-in-the-terminal.spec.md
- **Source ticket:** Chad, 2026-10-17 (as #803): "it could be formatted automatically in a table"
- **Status:** closed

## Summary
The table #803 detects is drawn in the terminal itself, over the block's own output rows, the way
Marley's block headers are drawn over their rows (#628): the rows' cells are hidden and the table
scrolls with the text. A toggle in the block header switches a block between its text and its
table, and the setting `marley.block_tables` (`offer`, the default, or `in_place`) decides whether
a detected table opens in place by itself. Copy, search and links keep reading the text.

## Acceptance
With `in_place`, `cat data.csv` shows a table where its text was, in the same rows; the toggle
brings the text back, and a copy of the block gives the CSV.
