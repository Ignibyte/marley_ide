# TICKET-748 — Side panels open as tabs: Collab, Fleet, Knowledge and Containers; Debug to the left

- **Ticket:** LOCAL #748 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [748-more-side-panels-as-tabs.spec.md](../../pipeline/queued/748-more-side-panels-as-tabs.spec.md)
- **Source ticket:** Chad, 2026-10-11 (see #747), and of Debug: "yes debug moves to bottom left and
  stays the exception". Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** open

## Summary
The rest of the right dock's panels take #747's tab: the collab panel, Fleet, Knowledge and
Containers. Two of them relied on the dock. Fleet stops reading its providers once no dock shows
it, so it learns to count its tab. Knowledge follows the active item, which would be its own tab,
so it skips its tab as #747's outline does. A tab closes when its panel turns disabled (Knowledge
while Rusty is off, Containers while its setting is off). The debug panel stays in the bottom dock,
the one exception, and its button moves to the left end of the status bar. The Agent Panel is then
the only panel left in the right dock; it stays there until Chad decides otherwise.

## Acceptance
Each of the four buttons opens its panel as a tab and brings it forward when open; the Fleet tab
lists its agents; the Knowledge tab keeps its page while used; the debug button sits at the status
bar's left and still opens the bottom dock.
