# TICKET-940 — Rusty's Server, Secrets and Skills as the board draws them

- **Ticket:** LOCAL #940 (feature, the Marley look, Rusty)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [940-rusty-server-and-forms-as-the-board.spec.md](../../pipeline/queued/940-rusty-server-and-forms-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/E-rusty.md, boards RustyServer, RustyTabs, MoreTabs, MoreForms)
- **Boards:** RustyServer, RustyTabs (MoreTabs and MoreForms are a follow-up ticket)
- **Size:** M
- **Status:** open

## Summary

Rusty's Server page says less than its board. It shows one row with the server's name, with no
time since the last check, nothing on whether agents get Rusty's tools, and a dropped connection
gives only the reason: no "Trying again in N s" and no Try Now (the dropped card on Rusty's tabs
has Try Now but no countdown either). The embedding status is one sentence where the board draws a
bar, and the ten settings rows show a key where the board shows a plain name. The Secrets tab does
not say when it locks again, and its locked card spreads the PIN rule over two places. A staged
skill's scan result and its Reject / Approve Anyway only show in the detail pane, and a script's
Run likewise. This ticket brings the Server page, Secrets and Skills to their boards. The forms
and pickers of MoreTabs and MoreForms are a follow-up ticket.

## Acceptance

The Server page shows the status card (checked N s ago, connection, program, Rusty's tools for
agents), the countdown with Try Now while dropped (on the page and on Rusty's tabs' card), the
embedding bar and plain names on the ten settings; Secrets shows "locks in N m" and the board's
locked card; a staged skill's row carries its scan result with Reject and Approve Anyway, and a
script's row carries Run; shot in Marley Night and Marley Day with Rusty's stand-in.
