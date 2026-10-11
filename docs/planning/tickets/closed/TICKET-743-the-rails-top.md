# TICKET-743 — The rail's top: the Rusty group leaves, Home becomes Default

- **Ticket:** LOCAL #743 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [743-the-rails-top.spec.md](../../pipeline/completed/743-the-rails-top.spec.md)
- **Source ticket:** Chad, 2026-10-11: "The rusty panel shouldnt be there anymore on start up",
  "i cant manually remove it. but rusty now is in the bottom panel which is what we want", and
  the Home group renamed Default. Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** closed

## Summary
#700 made a Rusty group on top of the rail from the start, and #675 opens every Rusty screen in it.
Since #738 and #739 Rusty opens in a tab of whatever group is shown, so the Rusty group only takes a
row, and its menu offers no way to remove it. The Rusty group is no longer made, a restored one goes,
and Rusty's screens and pages open in the shown group. The Home group, where work with no project
lands, is renamed Default so it is not mistaken for the Home page the status bar's Home button opens.

## Acceptance
A window that restores a Rusty group starts without it; Rusty's Brain screen opens in the shown
group; the rail's first row reads Default.
