# TICKET-747 — Side panels open as tabs: the mechanism, Files, Git and Outline

- **Ticket:** LOCAL #747 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [747-side-panels-as-tabs.spec.md](../../pipeline/queued/747-side-panels-as-tabs.spec.md)
- **Source ticket:** Chad, 2026-10-11: "I wanted everything to open in the main panel now with a
  tab. THe includes everything in the bottom right. With the ability to split pane it and drag it
  all", then "everything opens in the main pane now then the user can just split panel it out
  themselves. that way its consistent". Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** open

## Summary
In the Marley layout every side panel sits in the right dock, and its status-bar button toggles
that dock. A panel instead opens as an ordinary tab in the active pane of the shown group, or that
tab comes forward when it is open; the tab drags and splits like any other. The tab hosts the
panel's own entity, which stays registered in its dock, so Zed's code still finds it. A group holds
one tab of each panel, so a split moves the tab rather than copying it. A dock that opens on a moved
panel closes before it draws, and a dock restored open from the last session stays closed. A panel
tab comes back after a restart. This ticket builds the tab and moves the project panel, the git
panel and the outline panel; the outline panel needs a small hunk so it keeps following the editor
while its own tab is the active item. The Zed layout keeps its docks.

## Acceptance
The project, git and outline buttons and keys each open a tab in the active pane; asked again, each
brings its tab forward; a tab dragged or split moves; the outline tab keeps listing the editor's
symbols while used; a restored dock stays closed; the tabs come back after a restart.
