# TICKET-930 — tmux sessions as the board draws them

- **Ticket:** LOCAL #930 (feature, the Marley look, tmux sessions)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [930-tmux-sessions-as-the-board.spec.md](../../pipeline/queued/930-tmux-sessions-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/B-rail-chrome.md, board TmuxSessions)
- **Boards:** TmuxSessions
- **Size:** M
- **Status:** open

## Summary

The rail's Sessions view and the Sessions page as the TmuxSessions board draws them. In the
rail, a session not attached here shows a faded tile, and a host Marley does not look on sits in
the list with a grey dot and "Not looked on yet. Add to tmux hosts". On the page, the table gets
column headers, each state is a coloured chip (attached here, attached elsewhere, detached), a
session attached here offers Show in place of Attach, the header counts attached here apart from
the rest, and the hosts to look on are checkboxes. In a terminal's bottom row, the tmux count
says how many sessions are detached, and a click opens the Sessions page. #832 and #861 built
these screens; the board draws them differently.

## Acceptance

The Sessions view, the Sessions page and the bottom row's tmux count look as the board shows,
and Show, Add to tmux hosts, the page's checkboxes and the count's click work; shot in Marley
Night and Marley Day.
