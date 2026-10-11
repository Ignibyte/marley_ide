# TICKET-767 — File choosers, downloads and Take Over in the Browser tab

- **Ticket:** LOCAL #767 (feature, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** not started; the design is drafted in [756's notes](../../pipeline/queued/756-the-browser-tools-agents-lack.notes.md)
- **Source ticket:** split from #756 (slice 3) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** open

## Summary
A page's file chooser opens Zed's path prompt, downloads land in a folder Marley names and show in the tab, `browser_upload` answers an open file chooser, and a Take Over key hands the page to the person as the terminal's Ctrl-I does.

## Acceptance
A file chooser and a download work in a Browser tab, and Take Over stops an agent's control until it is handed back.
