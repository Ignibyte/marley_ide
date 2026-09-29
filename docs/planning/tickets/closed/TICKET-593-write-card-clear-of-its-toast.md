# TICKET-593 — Keep the terminal write card's Allow and Deny clear of its toast

- **Ticket:** LOCAL #593 (bug, T-series agent terminal writes, after #525)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/593-write-card-clear-of-its-toast.spec.md
- **Source ticket:** found in #525's visual check, after the fact (shot 525-02-ask)
- **Status:** closed

## Summary
When an agent's write to a running program waits for the user (#525), Marley shows a card under
the terminal with Deny and Allow at its right end, and a toast at the workspace's bottom right.
The toast lands on the card's buttons in the common layout, a terminal filling the center, so
Allow cannot be clicked while the toast shows; a click there hits the toast, and the write runs
out its 25 seconds. The buttons must be reachable while the toast shows.

## Acceptance
With the toast showing, the card's Allow and Deny sit where the toast does not cover them, and a
click on each answers the write.
