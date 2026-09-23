# TICKET-456 — Each dock as it was across a layout round trip

- **Ticket:** LOCAL #456 (bug, workbench shell W6g)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from #451 at its promotion)
- **Source ticket:** ../../pipeline/completed/451-marley-layout-presets.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
A layout switch moves the Agent Panel between docks (Zed's `agent.dock` is left, the Marley
layout's right). A dock that gains the panel while it was visible opens and shows it,
displacing the panel it showed; a dock that loses its active panel closes
(`crates/workspace/src/dock.rs:638-700`, `:895-918`). So with the right dock open on another
panel and the Agent Panel open on the left, a switch to the Marley layout and back leaves the
right dock closed and its panel lost (#438 inspect S9). The fix needs memory across the round
trip: per workspace, the panel each dock showed when the Agent Panel displaced it, restored
once the Agent Panel leaves again.

## Acceptance
After a switch to the Marley layout and back, each dock is open or closed as it was, showing
the panel it showed; the Agent Panel is visible in the dock its layout names if it was visible
before. The EARS criteria come at promotion.
