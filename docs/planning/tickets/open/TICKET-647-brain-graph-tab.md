# TICKET-647 — The Graph tab: Rusty's vault as a graph, local or whole

- **Ticket:** LOCAL #647 (feature, Rusty in Marley R5; `docs/marley/rusty-in-marley.md` R-D3 and
  R-D10)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/647-brain-graph-tab.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3's GraphView row, R-D10's
  `networkgraph-forcegraph-chorddiagram-parallelcoordinates` row, slice R5); Chad's batch of
  2026-10-02 and 2026-10-03: "lets make a plan to begin the work and spec out the tickets", "lets
  make sure we use the gpui components we found here", "Queue all five"
- **Status:** open

## Summary
Rusty's Qt app has a graph view (`GraphView.qml`) that draws the brain vault as dots and lines, as
Obsidian does. This ticket rebuilds it as a Marley center tab drawn from `rusty-mcp`'s
`brain_graph`: the whole vault, or the local graph of the focused page to a depth of one to four.
Pages are sized by their links and coloured by page type; a decision's typed edges (`consulted`,
`supersedes`, `follows_up`) are dashed, each kind in its own colour, apart from plain links. A
panel filters by page type, tag and path and turns tags, unresolved links, decision edges and
orphans on or off. A click opens the page in #645's Page tab; dragging pans, the wheel zooms and a
node can be dragged. The layout is Ely GPUI Components' Fruchterman-Reingold `Force`, ported into
`marley_rusty` with Ely's MIT notice, and runs on the background executor in batches, so the window
never waits on it; above 2,000 shown nodes it lays out the most linked and says so. It comes after
#643 (the switch, the client, the crate), #644 (the rail's Brain view), #645 (the Page tab) and
#646 (the Knowledge panel), and adds to them. Colour groups, display and force sliders and a tab
that survives a restart are the next slice (R5b).

## Acceptance
With Rusty on and a stand-in `rusty-mcp` serving a scratch vault, `rusty: open graph` and the
rail's Graph row open one Graph tab showing the vault, coloured by page type, with decision edges
dashed in their own colours. `rusty: open local graph` shows the focused page's neighbourhood,
which follows the page made active and widens with the depth. The legend hides a page type, the
filter keeps a tag or a path, and the four switches add or hide tags, unresolved targets, decision
edges and orphans. A click opens the page, a drag pans, the wheel zooms, a node drags. A change in
Rusty redraws the graph with nodes kept in place. A 3,000-page vault lays out its 2,000 most
linked off the main thread and says so. Rusty switched off empties the tab.
