# TICKET-929 — Table tabs, restored blocks, dropped links and the mic as the board draws them

- **Ticket:** LOCAL #929 (feature, the Marley look, terminal)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [929-terminal-more-as-the-board.spec.md](../../pipeline/completed/929-terminal-more-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/A-core.md, board TerminalMore)
- **Boards:** TerminalMore
- **Size:** S–M
- **Status:** closed

## Summary

Five small terminal states differ from the TerminalMore board. A table tab is Zed's table view
with nothing over it to say what it is or where it came from. Blocks restored after a restart run
straight into the live prompt with no line between them. A remote terminal whose link dropped
says "Rerun to try now" with no button and does not say that the tmux session keeps running. The
microphone turns red while it listens, with no words. A harness row whose state Marley only read
off the screen looks like one the agent reported. This ticket draws each as the board does.

## Acceptance

A table tab has a header with its kind, name, row count and command; a restored terminal shows
"Marley started again · HH:MM" between the restored blocks and the live prompt; a dropped link
reads "<host> stopped answering · checking again in N s · N keys held back" with Try Now and the
tmux line under it; the microphone reads "Listening · Voxtype" while it records; a state read off
the screen is drawn grey with "read off the screen". Shot in Marley Night and Marley Day.
