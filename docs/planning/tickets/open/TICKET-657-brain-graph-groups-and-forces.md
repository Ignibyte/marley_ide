# TICKET-657 — Colour groups, sliders and arrows on the Graph tab, restored after a restart

- **Ticket:** LOCAL #657 (feature, Rusty in Marley R5b; `docs/marley/rusty-in-marley.md` R-D3 and
  R-D10)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/657-brain-graph-groups-and-forces.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (slice R5b in the slices table); #647's Out
  (`docs/planning/pipeline/queued/647-brain-graph-tab.spec.md`, "R5b, the second slice"); Chad's
  batch of 2026-10-02 and 2026-10-03: "lets make a plan to begin the work and spec out the
  tickets", "lets make sure we use the gpui components we found here", "Queue all five"
- **Status:** open

## Summary
#647's Graph tab ships Rusty's default forces and display and is not restored after a restart.
This ticket adds the rest of Rusty's graph panel (`GraphView.qml:503-559`), as Obsidian's graph
view has it: colour groups (a query and a colour each; a shown node takes the first matching
group's colour, else its page type's), a Display section (Arrows, Text fade threshold, Node size,
Link thickness) and a Forces section (Center force, Repel force, Link force, Link distance) that
act on #647's layout. Zed's `ui` crate has no slider, so Ely GPUI Components' `Slider`
(`src/forms/slider.rs`) is ported into `marley_workbench::rusty` with Ely's MIT notice. Arrows go
on the edges that have a direction: links and a decision's typed edges, not tag edges. The graph
settings are one record for all of Marley in Zed's key-value store, as Rusty keeps one in its
window state; the tab becomes a Zed `SerializableItem` with its own table, as Marley's Browser
tabs are, so it comes back after a restart with its scope, centre, filter and panel. With Rusty
off at the start, no Graph tab comes back. It comes after #647 and adds to it, replacing none of
it; through #647 it rests on #643 to #646.

## Acceptance
With Rusty on and a stand-in `rusty-mcp` serving a scratch vault, a group's query colours the
nodes it matches (the first matching group winning), its swatch cycles the colour, and removing it
gives its nodes back. Arrows point at each link's target and never at a tag. Node size, link
thickness and the text fade threshold change the drawing, and the four force sliders lay the graph
out again with their values. A slider follows a drag, a press on its track and its keys. After a
quit and a relaunch the Graph tab is back where it was with its scope, centre, filter and panel,
and every group and slider value is kept; a new Graph tab after a close starts with the same
settings. With Rusty off at the start, no Graph tab is restored and nothing calls Rusty.
