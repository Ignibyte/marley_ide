# TICKET-675 — The Rusty group: where every Rusty screen opens

- **Ticket:** LOCAL #675 (feature, the rail; Rusty in Marley)
- **Owner:** unassigned (decided with Chad 2026-10-07)
- **Pipeline doc:** none yet
- **Source ticket:** `docs/planning/intake/rail-and-center-tabs.md` (decision 3), #600 (folderless
  groups), #672 (Rusty's icons in the header)
- **Status:** open

## Summary
Chad, 2026-10-06: "anthing that opens there should be in a group called Rusty (have a placeholder
for an icon when i get it)", and "Maybe we should have a pane thats always open (not closable)
that is the default pane something opens if no pane is obvious? This would include things like the
brain." On 2026-10-07: "rusty group makes sense".

Today every Rusty screen opens in the shown project's active pane (`add_item_to_active_pane` in
each `open_later`, `rusty/*_tab.rs`, and `page::open`), so a brain page opened in one project is
out of sight in the next. A folderless group (#600, `groups.rs`) named Rusty, made while Rusty is
on and listed in the rail with a placeholder icon, takes them all: Today, Graph, Tasks, Decisions,
Memory, Skills, Secrets and every page, whatever project is shown. Opening one shows the Rusty
group with that tab in front. Its tabs are its rows (#674). It has no close or rename, and it
comes back after a restart as #601's groups do. With Rusty off, nothing of it shows (#661).

## Open points for Plan
- A `rusty` flag on `groups::Group` beside `home`, or a reserved id, so the group is found again
  and never offered for closing or renaming.
- The placeholder icon: one named constant, so swapping in Chad's icon is one line.
- The Knowledge panel in the right dock follows the open page; the Rusty group's workspace needs
  it registered too.
- Pages opened from a project's own view in the Knowledge panel (R6) still go to the Rusty group.
- Ordering: after #672 and #674, which give the icons and the rows it relies on.

## Acceptance
With Rusty on, the rail lists a Rusty group with its placeholder icon. Opening Graph from the
header while a project is shown switches to the Rusty group with Graph in front, and the project's
tabs are untouched. A page opened from the Brain view's tree lands there too. The group offers no
close, survives a restart, and is gone while Rusty is off. Proof: a scenario with a scratch Rusty
store, shots of each step.
