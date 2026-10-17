# TICKET-798 — The agent panel's width in percent

- **Ticket:** LOCAL #798 (feature, the agent panel)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/queued/798-agent-panel-width-in-percent.spec.md
- **Source ticket:** Chad, 2026-10-17: "Can we make the agent window able to expand the size in percentage. Maybe a toggle at the bottom?"
- **Status:** open

## Summary
The agent panel is resized only by dragging its edge. Zed already keeps the panel's width as a
share of the window when its flexible size is on (`agent.flexible`, `Workspace::dock_flex_for_size`),
but nothing shows or sets that share. A button in the message editor's bottom row shows the
panel's width as a percent of the window, and each click steps it to the next size: 33, 50 and
66 percent, then the whole window, then back to 33.

## Acceptance
The button shows the panel's current width in percent, a click moves the panel to the next size
with the button's label following, and the size holds across a restart.
