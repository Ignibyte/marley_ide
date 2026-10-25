# TICKET-911 — The rail's menus and strip

- **Ticket:** LOCAL #911 (feature, the Marley look, the rail)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [911-rail-menus-and-strip.spec.md](../../pipeline/completed/911-rail-menus-and-strip.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/B-rail-chrome.md, boards RailMenus and RailStates)
- **Boards:** RailMenus, RailStates
- **Size:** M
- **Status:** closed

## Summary

The rail's right-click menus lack entries the RailMenus board draws, and say some things
differently. A project header's menu has no Fold, and it ends in "Remove Project" in the default
colour where the board has a red "Close Project". A terminal row's menu has no Turns (N), Send
Block to Agent or Split Right, and its Close is not red. The empty space's menu says "New
Terminal" where the board says "New Terminal in Default". The shrunk strip has no `+` at its foot
to add a project, and New Group… opens a modal where the board has an inline field at the rail's
foot. This ticket adds the entries, the wording, the red, the strip's `+` and the inline field.

## Acceptance

Each menu lists the board's entries in its words, the closing entry red, and each new entry does
what it names; the strip's `+` adds a project; New Group… names the group in a field at the rail's
foot. Shot in Marley Night and Marley Day.
