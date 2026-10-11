# TICKET-751 — Restored scrollback

- **Ticket:** LOCAL #751 (feature, the finish line: restarts that lose nothing, second half)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [751-restored-scrollback.spec.md](../../pipeline/completed/751-restored-scrollback.spec.md)
- **Source ticket:** Chad, 2026-10-11, on the survey list's first item ("Restarts that lose
  nothing"): "1.) this is nice". The T3 Code survey's item 1 (`docs/t3code_architecture/README.md`)
  and report 05 (item 1); its open question 7, default on, the log file mode 0600, a setting to
  turn it off. Plan: [the finish line](../../design-notes/finish-line-2026-10-11.md), Now, after
  #750.
- **Status:** closed

## Summary
Every Marley restart empties every terminal: Zed brings each one back as a new shell in its folder.
Chad restarts Marley at each install. With this ticket each local terminal keeps its recent output
in a file only he can read, and a restored terminal shows it dimmed above the new shell's first
prompt. The file keeps text and plain formatting only, so no terminal query in it can answer into
the new shell, and no restored row becomes a block. `marley.restore_scrollback` turns it off and
deletes the files.

Sized honestly the whole item is L. This ticket is the first slice (M): the output itself. Blocks
redrawn from saved records on the restored rows (command, exit code, folder, times) are a second
slice of about the same size, proposed in the spec's Out and waiting for a number.

## Acceptance
After a relaunch a restored terminal shows its earlier output dimmed above a new prompt, capped at
5,000 lines; nothing appears at the new prompt from an old query; the file is mode 0600 and goes
when the terminal closes; with the setting off nothing is kept.
