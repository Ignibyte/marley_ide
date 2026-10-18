# TICKET-810 — The Take Over bar can be closed

- **Ticket:** LOCAL #810 (bug, agents in terminals and the Browser tab)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** (set at Plan)
- **Source ticket:** Chad, 2026-10-17: "i had the mcp from outside type something into manager.
  Now i have a button that is take over, hand back, and never leaves. It should be able to exit it."
- **Status:** open

## Summary
Once an agent types into a terminal's program (#525), the bar under the terminal says what it
typed, with Take Over; after Take Over it says "You have control" with Hand Back. The bar stays
while the program it typed into runs, so a long-running agent CLI such as a harness manager keeps
it for good, and Hand Back only brings Take Over back. The Browser tab's #767 bar ("… acted in
this page", Take Over) stays the same way. Each bar gets a close button that hides it until an
agent acts again, and Hand Back hides it too.

## Acceptance
After an agent typed into a terminal, the bar's close button hides it; Hand Back hides it; the
next write shows it again. The same holds for the Browser tab's bar.
