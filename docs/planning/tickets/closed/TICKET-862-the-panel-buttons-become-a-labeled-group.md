# TICKET-862 — The panel buttons become one labeled group

- **Ticket:** LOCAL #862 (feature, the Marley look, W1.4 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [862-panel-buttons-as-a-labeled-group.spec.md](../../pipeline/completed/862-panel-buttons-as-a-labeled-group.spec.md)
- **Source ticket:** split from TICKET-833 while planning it (2026-10-21), from the design
  canvas's BottomRow board.
- **Boards:** BottomRow
- **Size:** M
- **Status:** closed

## Summary

The panel buttons at the bottom row's right become one labeled group (Home, Files, Changes, Outline, Knowledge, Fleet, Collab, Agent), each with its icon and, where room allows, its label, and an accent dot on a panel already open as a tab. Zed draws three `PanelButtons` items (left, right and bottom docks) with no labels; Marley replaces or regroups them, a change in `crates/zed/src/zed.rs` or `workspace` with its ledger row.

## Acceptance

The bottom row's right end shows the panels as one labeled group with the open-as-tab dot, in every focus; shot in Marley Night and Marley Day.
