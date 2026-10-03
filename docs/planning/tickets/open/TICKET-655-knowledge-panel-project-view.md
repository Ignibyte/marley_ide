# TICKET-655: The Knowledge panel's project view: a project's brain page, follow-ups and tasks

- **Ticket:** LOCAL #655 (feature, Rusty in Marley R6: the project join of R-D5 and the panel's
  project view; the Graph tab's project centre)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/655-knowledge-panel-project-view.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, R-D5, slice R6), split out of #646
  (its Out list and the discovery its notes kept) and #647 (its Out list); Chad, 2026-10-02:
  "manage these projects", and for the Rusty batches "lets make a plan to begin the work and spec
  out the tickets"
- **Status:** open

## Summary
Rusty knows about the projects on the box: 101 project pages in the brain, decisions that name
them, and to-do lists. Nothing joins a rail project to any of it. This ticket writes the join and
shows it. A workspace's project (its folders, as Zed groups them for the rail) resolves to a brain
project page: first the page whose `path:` property lists one of its folders, then the one page
named like a folder. With no page focused, #646's Knowledge panel shows that page's title and
summary, the follow-ups due among the decisions linked to it, and the open tasks of its Rusty task
group, which the page names in a `task_group` property or, failing that, is the group named like
the page. When nothing matches, or several pages share a name, the panel offers to link one: the
user picks the page and Marley adds the project's folder to its `path:` through Rusty's
`brain_set_property`, keeping what the property held. A task group is linked the same way. #647's
Graph tab, with no page focused, centres its local graph on the project's page. Everything reads
through #643's connection, writes only on the user's pick, and exists only while
`marley.rusty.enabled` is on.

## Acceptance
With a project whose folder a project page lists, and no Page tab in front, the panel shows that
page, its summary, its due follow-ups (and no other decision's) and its task group's open tasks;
Open Page opens it. A `task_group` property beats the group named like the page, and a change
Rusty announces shows without input. With only a name to go on, the panel says so; with two pages
of that name it lists both and picks neither; with none it says which folder no page lists. Each
case links through a pick that writes `path:` (or `task_group`) through `brain_set_property`,
keeping the earlier value, after which the page shows as matched by path. The Graph tab opened or
switched to Local with no page focused centres on the project's page, and a Page tab made active
takes over the centre.
