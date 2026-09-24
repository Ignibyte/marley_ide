# TICKET-487 — Scenarios that click run Marley in a headless sway

- **Ticket:** LOCAL #487 (chore, prong 3 tooling: the e2e harness)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/487-e2e-pointer-in-headless-sway.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 D18
- **Status:** open

## Summary
The e2e harness sends keys to Marley's window through Hyprland and cannot click: Hyprland has
no dispatcher that sends a pointer event to one window, and a real click would move Chad's
pointer. The browser tab is driven mostly by the mouse, so its scenarios need clicks, drags
and the wheel. A scenario that sets `COMPOSITOR=sway` runs Marley in a headless sway of its
own, whose seat is a virtual pointer and a virtual keyboard only that compositor sees.

## Acceptance
A scenario under sway clicks a button that only a click reaches (the agent bar's Rich Input
button), scrolls with the wheel and types, and every shot shows the result, with no window on
Chad's desktop and no change to his Hyprland.
