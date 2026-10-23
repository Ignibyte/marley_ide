# TICKET-925 — Toasts, the plan entry and notifications as the board words them

- **Ticket:** LOCAL #925 (feature, the Marley look, notices)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [925-notices-as-the-board.spec.md](../../pipeline/queued/925-notices-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/C-threads.md, board Inbox)
- **Boards:** Inbox
- **Size:** S–M
- **Status:** open

## Summary

Marley's notices work, but they say less than the Inbox board. Every toast's dot is the accent
unless its text reads as a failure. The plan entry in Needs you says "Plan to review" with no
title and a quiet Open. A long command's desktop notification is titled with the command and
says only how long it took. A phone push is always titled "Marley" with one line. This ticket
gives toasts a kind and a dot to match, names the plan and offers Open the plan, and words the
desktop notification and the phone push as the board does. The push still carries nothing the
agent wrote (#535).

## Acceptance

A warning toast shows a yellow dot, a success green, an error red, a question the accent and any
other grey; the plan entry reads "Plan to review: “<title>”" with an accent Open the plan; a long
command's banner reads "<command> finished · exit N" over "<terminal> ran <time> while you were in
<item>."; a push is titled with the event and its body is "project · detail". Shot in Marley Night
and Marley Day; the banner and the push checked in their logs.
