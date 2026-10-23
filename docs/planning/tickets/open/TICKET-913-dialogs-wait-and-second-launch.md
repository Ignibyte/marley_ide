# TICKET-913 — Wait before closing, a second launch says so, the tmux close prompt

- **Ticket:** LOCAL #913 (feature, the Marley look, dialogs)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [913-dialogs-wait-and-second-launch.spec.md](../../pipeline/queued/913-dialogs-wait-and-second-launch.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/G-system.md, board Dialogs: "Close dialog 'Wait'
  button", "Close dialog wording", "Second-launch notice";
  docs/marley/design-review-2026-10-23/B-rail-chrome.md, board Chrome: "tmux close prompt")
- **Boards:** Dialogs, Chrome
- **Size:** M
- **Status:** open

## Summary

Three dialogs do not yet say or do what the boards draw. Closing a working agent's tab offers
Cancel, Show and Close; the board offers Wait, which closes the tab once the agent's turn ends,
and its words name the turn and the undo hold. A second launch that hands the running Marley a
folder prints a line in its own terminal, and the running Marley says nothing; the board shows a
notice there. The tmux close prompt has its title and a fixed explanation; the board gives it a
`tx` tile and a line with the host, the windows and what runs.

## Acceptance

Wait closes the tab when the turn ends; the close dialog names the turn and the hold; a second
launch's folder opens with a notice in the running Marley; the tmux prompt shows its tile, host,
windows and running program with Cancel · Hide · Close Session; shot in Marley Night and Marley
Day.
