# TICKET-673 — Containers move to a panel on the right, with a button in the status bar

- **Ticket:** LOCAL #673 (feature, the rail's ports)
- **Owner:** claude-opus-5-5, 2026-10-07 (Chad's request)
- **Pipeline doc:** ../../pipeline/completed/673-containers-panel-on-the-right.spec.md
- **Source ticket:** #614 (container ports in the rail), #669 (Containers in the Rail), #670
  (the fold)
- **Status:** closed

## Summary
Chad, 2026-10-06: "can we just move the containers icon down as an icon in the botton bar and it
opens on the right?" The rail lists the machine's containers that no project holds under a
CONTAINERS header after the projects (`render_containers`, `rail.rs:4337`), folded since #670.
That list moves to a right-dock panel. A dock panel whose `icon()` returns one gets its button in
the status bar from Zed's `PanelButtons` (`workspace/src/dock.rs:1587`) with no Zed change, as
the Fleet panel (`fleet.rs:299`, `:1498`) and Rusty's Knowledge panel do.

A container whose Compose folder is in a project still shows under that project in the rail.
Only the unowned list moves.

## Where
- A new `Panel` in `marley_workbench` (`DockPosition::Right`, an icon, the rows
  `render_containers` draws today and their clicks: open in a Browser tab, the row's menu).
- `rail.rs`: the Containers section, `ContainersFold` and `toggle_containers` go. The saved blob's
  `marley_containers_open` is still read without error, so an older window restores.
- `marley.rail_containers` (#669) keeps its job, whether the machine's containers are listed, now
  for the panel. Test runs still turn it off. Plan decides whether its name and doc change.

## Acceptance
The rail shows no CONTAINERS section. The status bar has a containers button, and a click opens
the panel in the right dock with the same rows the rail showed, each still opening its port. A
project's own containers stay under the project. Proof: a sway scenario with #670's fake `docker`
and stand-in `docker-proxy`: the rail without the section, the status bar button, the open panel,
and a row opened in a Browser tab.
