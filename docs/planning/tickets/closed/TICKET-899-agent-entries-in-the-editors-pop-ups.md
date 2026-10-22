# TICKET-899 — Agent entries in the editor's pop-ups, and the inline rename note

- **Ticket:** LOCAL #899 (feature, the Marley look, W6)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [899-ask-an-agent-and-the-rename-note.spec.md](../../pipeline/completed/899-ask-an-agent-and-the-rename-note.spec.md)
- **Source ticket:** split from TICKET-856 while planning it (2026-10-22): these are new
  behaviour, where #856 puts Zed's pop-ups in the board's look.
- **Boards:** EditorPopups
- **Size:** M
- **Status:** closed

## Summary

Code actions end with "Ask an agent to fix this…", which sends the line and its diagnostic to an
agent as Send Selection does; inline rename shows "Renaming: Old → New · Enter" at the line's
end; and inline assist shows the selection being sent to an agent, as the EditorPopups board
draws them.

## Acceptance

Each works and looks as the board shows; shot in Marley Night and Marley Day.

## Notes

- Planning (2026-10-22) split inline assist's line into TICKET-900: a run has no LLM provider.
