# TICKET-820 — A header's right-click offers what its + does

- **Ticket:** LOCAL #820 (bug, the Marley layout's rail)
- **Owner:** 6bc788ef-5565-4849-9e73-e341a6348e9f
- **Pipeline doc:** ../../pipeline/completed/820-header-menus-offer-what-the-plus-does.spec.md
- **Source ticket:** Chad, 2026-10-19: "Right clicking on the left panel some of the features the
  + has is missing so lets rectify that".
- **Status:** closed

## Summary
A header's `+` offers New Terminal, New Browser Tab, New Agent…, Reopen Thread…, New Agent in
Worktree and the project's Launch entries; its right-click offers none of them (a project's has
Move Up/Down, Archived Threads, Clear Browser Data… and Remove Project, a group's Rename and
Remove, Default's a label). The rail's empty space offers New Group…, New Terminal, New Browser
Tab and the old list of agent CLIs from before #744, with no New Agent… or Reopen Thread….

## Acceptance
Every open header's right-click starts with its `+`'s entries, and the empty space's offers New
Agent… and Reopen Thread… in place of the old agent list.
