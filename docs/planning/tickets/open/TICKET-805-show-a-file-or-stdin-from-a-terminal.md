# TICKET-805 — Show a file or stdin from a terminal

- **Ticket:** LOCAL #805 (feature, the block terminal)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/queued/805-show-a-file-or-stdin-from-a-terminal.spec.md
- **Source ticket:** Chad, 2026-10-17 (as #803)
- **Status:** open

## Summary
The explicit way, with nothing guessed: `show data.csv`, `some-command | show --as csv` or
`show report.json` in a Marley terminal asks Marley, through `marley-agent`'s socket (which checks
the caller, so no printed text can fake it), to show a file or stdin in a tab: a table, a #799
component tree, markdown or an image, typed by extension or `--as`. Outside Marley it prints the
text.

## Acceptance
`show data.csv` opens a table tab; `jq . x.json | show --as component` draws the component;
`show notes.md` outside Marley prints the file.
