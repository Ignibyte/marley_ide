# TICKET-714 — Undo a turn

- **Ticket:** LOCAL #714 (feature; size medium; the finish line, after #753)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [714-undo-a-turn.spec.md](../../pipeline/completed/714-undo-a-turn.spec.md)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 1; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"; the T3 Code survey, report 03 item 1 and report 01 item 7; Chad, 2026-10-11: "meh not sure. convince me of this or not", then, after the case for and against, "ok we can build it". Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md), row 5.
- **Status:** closed

## Summary
Keep and Undo This Turn… in a right-click menu on each turn row of #509. A turn is already a
commit of its end on its start under `refs/marley/turns/`. Undo takes out that turn's changes and
nothing else: a three-way merge against the tree as it is now, so later turns and the user's own
edits stay. It asks first, naming the files. It refuses, says why and changes nothing on a
conflict, while any agent works in the same checkout or a folder inside or around it, while a file
it would change has unsaved edits, or when a file changed while it asked. The user's index, `HEAD`
and the turn refs are never touched. Keep marks the turn reviewed. It builds after #753, which
records Codex's turns, so one Undo covers Claude Code and Codex.

## Acceptance
Undo on a turn row puts that turn's files back as they were before it and removes the files it
created, keeping later turns and the user's edits; a conflicting undo, or one while another agent
works in the checkout, is refused with the reason and changes nothing; Keep marks the row.
