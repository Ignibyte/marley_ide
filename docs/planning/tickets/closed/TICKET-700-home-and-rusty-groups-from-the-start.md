# TICKET-700 — Home and Rusty groups from the start, on top of the rail

- **Ticket:** LOCAL #700 (feature)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [700-home-and-rusty-groups-from-the-start.spec.md](../../pipeline/completed/700-home-and-rusty-groups-from-the-start.spec.md)
- **Source ticket:** Chad, 2026-10-09: "Marley starts a fresh install with the home panel existing,
  rusty existing if its enabled. The order should be Home then Rusty." Asked whether they sit at
  the top or the bottom of the rail: "top correct".
- **Status:** closed

## Summary
Today a window's Home group (#600, #676) is made the first time something lands there, and its
Rusty group (#675) the first time a Rusty screen opens; the rail lists both after the projects. Now:
- every window in the Marley layout has its Home group from the start, and its Rusty group while
  Rusty is on (made when Rusty turns on later, too);
- the rail lists Home, then Rusty, then the projects, then any named group; a header order the
  user dragged still wins;
- a restart neither duplicates nor loses them: groups restored from the last session are adopted
  first.

## Acceptance
- A fresh profile's window lists Home then Rusty above its project.
- After a quit and a relaunch, the rail lists one Home and one Rusty, in that order, above the
  project.
- With Rusty off, Home alone sits above the project.
