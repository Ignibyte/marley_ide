# TICKET-774 — The Debug button at the status bar's left

- **Ticket:** LOCAL #774 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [774's spec](../../pipeline/completed/774-the-debug-button-at-the-left.spec.md); the design is #748's D6 and D7
- **Source ticket:** split from #748 at its promotion (2026-10-11). Chad, 2026-10-11: "yes debug
  moves to bottom left and stays the exception". Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** closed

## Summary
The Debug panel stays in the bottom dock. In the Marley layout its button leaves the bottom dock's
buttons at the status bar's right and sits at the status bar's left end, toggled while the bottom
dock shows it; a click does what Zed's button did. `debugger.button` defaults to false in the
Marley layout (Zed's value kept for the switch back), and a Marley status item draws the button
after the left dock's buttons. Zed's layout keeps Zed's button.

## Acceptance
In the Marley layout the Debug button sits at the status bar's left and opens the bottom dock on
the Debug panel; in Zed's layout it is back among the bottom dock's buttons.
