# TICKET-781 — Zed Agent shows a blank thread, and Select Model opens the old right panel

- **Ticket:** LOCAL #781 (bug)
- **Owner:** 9623da6e-a5cd-4dcb-b048-40009324e9f0
- **Pipeline doc:** none yet
- **Source ticket:** Chad, 2026-10-12: "if you click agent panel, choose zed agent it shows blank,
  then choose select model it brings up the old right agent panel. add this somewhere as a bug".
- **Status:** open

## Summary
In the Marley layout, opening the Agent Panel and choosing Zed Agent shows an empty thread; Select
Model then opens Zed's right-docked Agent Panel, which the Marley layout keeps out of sight since
threads open in tabs (#734, #772). Reproduce first, then find where the native agent's thread view
loses its content and which action reveals the dock.

## Acceptance
Choosing Zed Agent shows its thread with its message editor; Select Model opens the model picker
over that thread, and the right dock stays closed.
