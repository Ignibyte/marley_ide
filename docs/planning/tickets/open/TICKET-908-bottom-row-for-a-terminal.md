# TICKET-908 — The bottom row for a terminal

- **Ticket:** LOCAL #908 (feature, the Marley look, the bottom row)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [908-bottom-row-for-a-terminal.spec.md](../../pipeline/queued/908-bottom-row-for-a-terminal.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/B-rail-chrome.md
  BottomRow and TmuxSessions; docs/marley/design-review-2026-10-23/A-core.md Terminal and
  TerminalSession), boards BottomRow, TmuxSessions, Terminal
- **Boards:** BottomRow, TmuxSessions, Terminal
- **Size:** S–M
- **Status:** open

## Summary

When a terminal is in front, the bottom row should say where it is and what went wrong. The
BottomRow board draws the folder, the branch with its changed count, the failed command as a
button that asks an agent to fix it, the block count with the bookmarked ones, and the terminal's
size. Marley's row hides the folder at the project root, has no branch, changed count or size,
and draws a failure as red text that does nothing. Its tmux count also hides how many sessions
are detached, and a click on it does nothing; the TmuxSessions board says it opens Sessions.

## Acceptance

With a terminal in front, the bottom row shows its folder, branch and changed count, the failed
command as an Ask an agent button that sends the block to an agent, the block and bookmark
counts, the size in columns and rows, and "tmux: N · M detached", whose click opens the rail's
Sessions view; shot in Marley Night and Marley Day.
