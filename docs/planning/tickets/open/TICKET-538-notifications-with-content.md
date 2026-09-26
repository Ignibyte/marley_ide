# TICKET-538 — Notifications that say what happened

- **Ticket:** LOCAL #538 (feature, prong 1 T7b's follow-on, with prong 2's attention)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/538-notifications-with-content.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Orca survey: "we will be taking what it does well and bring it in here" (docs/orca_architecture/README.md); notifications with content and unread are report 01 §2.5 and §3 item 4.
- **Status:** open
- **Backlog:** Queue, after #519. The banner's words (the event, the last message, the tool and its input) and the event each mark belongs to come from #519's events; without them there is nothing to say.

## Summary
A Claude Code banner in Marley today is the plugin's fixed sentence ("marley_ide needs your
permission"), so every alert means opening the terminal to learn what happened. With #519's events
the banner says it: the title `marley_ide: Claude finished` (or needs input, failed), the body the
turn's last message cut short, or `Using Bash: npm test`. A terminal that had an event while Chad
was elsewhere is marked unread even when no banner showed, and focusing it clears the mark; a
repeated ping of the same state does not light it again. A burst of events from one project makes
one banner.

## Acceptance
A finished turn, a permission request and a failed turn from Claude Code in a terminal Chad is not
in each show a banner that names the project and the event and says what happened; the terminal is
marked unread until it is focused; a second event from the same project within 5 seconds shows no
banner but still marks its terminal.
