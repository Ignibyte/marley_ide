# The agent bar, with the folder and branch — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-477-agent-bar.md
- **Pipeline spec:** 477-agent-bar.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23, the third of five things to bring over from Warp's bar: "has
  the location of what folder and which branch you are on on the bottom right". The bar is the
  container the other four go in.
- **Recall.** The rail already recognizes agents in terminals and gives them a waiting status
  (`rail.rs`, `marley_agent`); `docs/marley/workbench-shell.md` asks for the branch on the
  project header, not yet built.
- **Seams read:** `TerminalView::render` (a `div` root, one full-size container with the
  element); `Terminal::foreground_process_command_name` and `working_directory`; the git
  store's branch as the title bar reads it.
