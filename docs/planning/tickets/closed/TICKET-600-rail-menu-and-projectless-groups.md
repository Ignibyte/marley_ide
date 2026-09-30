# TICKET-600 — The rail's right-click menu, and projectless groups

- **Ticket:** LOCAL #600 (feature, workbench shell: the rail)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/600-rail-menu-and-projectless-groups.spec.md
- **Source ticket:** Chad, 2026-09-30: "We should add a right click ability on the left panel like warp where we can create a new group and new projectless items". He chose a projectless group: a named group with no folder, whose + makes terminals, agent CLIs and Browser tabs that start in the home folder.
- **Status:** closed

## Summary
Everything in the rail belongs to a project folder today, so a terminal for a quick look at the
system, or an agent CLI with no repository, has to borrow some project. Right-click on the rail's
empty space opens a menu, as Warp's tab list does: New Group… makes a named group with no folder,
and New Terminal, New Browser Tab and the agent CLIs make a projectless item in the group named
Home. A group lists like a project, with its name, a group icon, its chevron and its `+`; its
terminals and agents start in the home folder, and its Browser tabs use a Chromium of the group's
own. The group's header menu renames or removes it. Groups last for the session; #601 brings them
back after a restart.

## Acceptance
Right-click the rail below the last row, choose New Group…, name it "Scratch": a Scratch group
appears; its + opens a terminal whose `pwd` is the home folder; New Terminal from the empty-space
menu lands in a Home group; adding a project afterwards leaves both groups in place.
