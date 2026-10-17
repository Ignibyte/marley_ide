# TICKET-798 — The agent panel's width in percent

- **Ticket:** LOCAL #798 (feature, the agent panel)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** [798-agent-panel-width-in-percent.spec.md](../../pipeline/completed/798-agent-panel-width-in-percent.spec.md)
- **Source ticket:** Chad, 2026-10-17: "Can we make the agent window able to expand the size in percentage. Maybe a toggle at the bottom?"
- **Status:** closed

## Summary
An agent thread is resized only by dragging an edge. A button in the thread's message editor now
shows the width of the thread's place in percent, and each click steps it: a third, half, two
thirds, the whole window, then a third again. The place is the Agent Panel's dock (its share of the
window) or, in the Marley layout where threads open in tabs, the pane that holds the thread's tab
(its share of its row).

## Acceptance
The button shows the place's width in percent, a click moves it to the next size with the label
following, a lone pane shows no button, and the panel's width holds across a restart.
