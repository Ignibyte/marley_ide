# TICKET-754 — CLI sessions in Threads

- **Ticket:** LOCAL #754 (feature, the finish line: old CLI sessions in Threads)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [754-cli-sessions-in-threads.spec.md](../../pipeline/queued/754-cli-sessions-in-threads.spec.md)
- **Source ticket:** Chad, 2026-10-11, on the survey list's item 4 ("Old CLI sessions in Threads …
  finishes the Reopen Thread you just asked for"): "4.) yes". Earlier the same day, of New Agent from
  Thread: "my thought is that you would re open it". The T3 Code survey's item 3
  (`docs/t3code_architecture/README.md`), reports 01 (item 4) and 02 (item 2). Plan:
  [the finish line](../../design-notes/finish-line-2026-10-11.md), Now, after the tabs batch
  (#743 to #749) and #750.
- **Status:** open

## Summary
Zed's thread store knows only the conversations its Agent Panel had. The Claude Code and Codex
sessions Chad runs in Marley's terminals are found again only by `/resume` in the right folder, one
CLI at a time. With this ticket Marley reads each CLI's own session files (a pure parser per CLI,
read-only, bounded) and lists the interactive sessions beside Zed's conversations: on the Threads
page (#737), in the rail's Threads list (#745) and in Reopen Thread… (#744). The page's search also
finds a session by the words of its prompts. Reopening one runs `claude --resume <id>` or `codex
resume <id>` in a center terminal in the session's folder; a session already running in a Marley
terminal brings that terminal forward. `marley.list_cli_sessions` turns it off.

## Acceptance
The Threads page, the rail's Threads list and Reopen Thread… list the fixture's Claude Code and
Codex sessions with titles and folders; a search finds one by a word of its prompt; choosing one
opens a terminal running the CLI's resume in its folder, or brings forward the terminal running it.
