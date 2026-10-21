# TICKET-868 — Files pops out from the left

- **Ticket:** LOCAL #868 (feature, the Marley look)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** Chad, 2026-10-21: "the project panel shouldnt be in the main tab it should be
  a left hand nav item you can view that pops out instead."
- **Boards:** Rail, SidePanels
- **Size:** M
- **Status:** open

## Summary

The project panel (Files) stops opening as a center tab (#747). A Files item at the left, by the
rail, opens the file tree beside the rail, and the same item, Escape or a second click closes it.
The Git, Outline and other moved panels keep #747's tabs (or #837's Inspector).

## Acceptance

Files has a nav item at the left; a click pops the tree out beside the rail and the editor stays
in view; every way Zed asks for the project panel (its key, Reveal in Project Panel) opens it there;
no Files tab opens. Shot in Marley Night and Marley Day.

## Decision

Decision taken by default, pending Chad: the tree pops out in Zed's left dock, between the rail
and the editor, resizable, and stays out until closed (rather than an overlay that closes when you
click away).
