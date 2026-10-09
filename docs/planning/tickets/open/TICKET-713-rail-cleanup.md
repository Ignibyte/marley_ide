# TICKET-713 — Rail cleanup

- **Ticket:** LOCAL #713 (chore, the rail)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** the 2026-10-09 intake audit of `rail-internals.md`; the four items left,
  checked against the code.
- **Status:** open

## Summary
- **Unregistering.** `MultiWorkspace::register_sidebar` (`multi_workspace.rs`, around line 400) has
  no way to unregister, so a sidebar swap keeps the old subscriptions. Add one (a Zed
  touchpoint).
- **Refreshes.** Every terminal `Wakeup` refreshes the rail (`rail.rs`, `note_output` into
  `refresh`). Refresh only on events that change a row.
- **The swap event.** "Sidebar Toggled" is still recorded on a swap (`multi_workspace.rs` around
  lines 505 and 533). It costs nothing while #514 keeps telemetry off; make the swap silent.
- **The restore.** Save the rail's state only after the restore finishes.

## Acceptance
The rail looks and behaves as before; a terminal printing steadily no longer refreshes the rail
on each wakeup.
