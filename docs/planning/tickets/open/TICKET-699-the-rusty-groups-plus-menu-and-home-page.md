# TICKET-699 — The Rusty group's + menu and its home page

- **Ticket:** LOCAL #699 (feature)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-09, from daily use: "Rusty is treated as a panel. in its + i
  think we should have quick links inside of there for rust things" and "When rusty panel is
  empty and i click on it, it defaults to the zed open project panel. lets default it to
  basically the home page. or have the home page open all the time"
- **Status:** open

## Summary
The Rusty group (#675) is a projectless workspace, so its center pane behaves like any other:
- **The + menu** on its tab bar is Zed's: New File, Open File, Search Project, Search Symbols,
  New Terminal, New Center Terminal. None of it reaches Rusty.
- **An empty Rusty group** shows Zed's Welcome page (Open Project, recent projects), since a pane
  with no tab in a workspace with no folder draws it. The group is empty after a restart (the home
  page is not restored) and after its last tab closes; only the rail's Rusty button puts the home
  page back (#679).

The change:
- In the Rusty group, the + menu starts with Rusty's quick links: the home page, the eight screens
  (Brain, Today's Note, Graph, Tasks, Decisions, Memory, Skills, Secrets), Open Page… and the
  three captures. Zed's own entries follow under a separator. Other groups and projects keep
  Zed's menu unchanged.
- The Rusty group never shows Zed's Welcome page: whenever the group is shown, or its last tab
  closes, with no tab open, its home page opens.

## Acceptance
- With the Rusty group shown, its + menu lists Rusty's quick links above Zed's entries, and each
  opens its screen in the group; a project's + menu is unchanged.
- Closing every tab of the Rusty group leaves its home page showing, not Zed's Welcome page.
- After a restart, clicking the Rusty group in the rail shows its home page.
