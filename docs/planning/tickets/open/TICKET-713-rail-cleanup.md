# TICKET-713 — Rail cleanup

- **Ticket:** LOCAL #713 (chore, the rail)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [713-rail-cleanup.spec.md](../../pipeline/queued/713-rail-cleanup.spec.md)
- **Source ticket:** the 2026-10-09 intake audit of `rail-internals.md`; the four items left,
  checked against the code. Chad, 2026-10-11, on the finish-line list's row 8 (the rail cleanup
  among the small fixes): "8.) yes to all". Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** open

## Summary
- **Unregistering.** `MultiWorkspace::register_sidebar` (`multi_workspace.rs:400-412`) pushes two
  subscriptions per call and never drops any, so each swap leaves the old sidebar's pair behind,
  and each round trip adds a live pair on the Zed sidebar the rail keeps. Register replaces the
  previous sidebar's pair (a Zed touchpoint). Home's page also holds the rail it first found
  (`home_page.rs:106`, `192-206`), which keeps a swapped-out rail alive and refreshing; it follows
  the window's current rail instead.
- **Refreshes.** Each chunk of a terminal's output refreshes the rail three or four times: the
  view's `Wakeup` (`rail.rs:2387-2399`), its `UpdateTab` (`2451-2453`), the pane's
  `ChangeItemTitle` (`2588-2596`) and, in the active pane, the workspace's `ActiveItemChanged`
  (`2425-2429`). Refresh only when a row would change.
- **The swap event.** "Sidebar Toggled" is recorded for every window built in the Marley layout,
  every swap and every restored closed rail (`multi_workspace.rs:500-507`, `528-551`). Marley's
  own opens and closes are silent; the user's toggles keep the event.
- **The restore.** A window built in the Marley layout saves itself before its restore applies
  (`open_sidebar` → `serialize`), and the save after the restore lists only the groups already
  reopened. Save the window's state only after the restore finishes.

## Acceptance
The rail looks and behaves as before; a terminal printing steadily no longer refreshes the rail on
each wakeup; after a swap only the new rail refreshes; a window killed during its restore comes back
with the groups it had.
