# TICKET-803 — A block's output opens as a table

- **Ticket:** LOCAL #803 (feature, the block terminal)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/queued/803-a-blocks-output-opens-as-a-table.spec.md
- **Source ticket:** Chad, 2026-10-17: "i'd like to spec out and research from alacritty how we may be able to transform command line output into gpui components if possible. For example if we read an output from a csv it could be formatted automatically in a table."
- **Status:** open

## Summary
A finished command whose output reads as a table (CSV, TSV, JSON lines, a JSON array of objects,
a markdown table) gets a Table chip in its block header. The chip, and Open Output as Table in
the block menu, open Zed's sortable, filterable table view (`tabular_data_preview`) in a tab
beside the terminal. A `cat`, `head`, `tail` or `bat` of a tabular file opens the file itself.
The terminal's text is left as it was. Research: `docs/planning/design-notes/terminal-output-as-components-2026-10-17.md`.

## Acceptance
`cat data.csv` shows the Table chip and opens the table in a tab; `ls -l` and colored output show
no chip.
