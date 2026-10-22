# TICKET-891 — Project search, branches, history, extensions and the guide's Try it

- **Ticket:** LOCAL #891 (feature, the Marley look, W5)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-884 while planning it (2026-10-22): #884 put the guide in
  Marley's colours.
- **Boards:** SearchMore
- **Size:** S–M
- **Status:** open

## Summary

Project search and replace ("14 matches in 6 files", file and line over each hit), the branch
picker's dots and notes (current, behind, worktree, remote), history in the Changes inspector, the
extensions list, and the guide's "Try it" links that run a command, as the SearchMore board draws
them.

## Acceptance

Each part looks as the board shows; shot in Marley Night and Marley Day.

## Notes

- Zed's branch picker already groups local and remote branches, ticks the current one and gives
  each branch's last commit (seen by `script/e2e/891-branch-picker.sh`); the board adds dots and
  notes.
- The Changes inspector holds Zed's git panel; a history view there is a view that is not a panel.
