# TICKET-794 — Home goes missing when the Default group restores a tab

- **Ticket:** LOCAL #794 (bug, Home)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** [794-home-after-a-restored-tab.spec.md](../../pipeline/completed/794-home-after-a-restored-tab.spec.md)
- **Source ticket:** seen on the Mac on 2026-10-16 (#787's notes; again in #791's Mac shot): a
  relaunch restored only the first-run Onboarding tab in the Default group, and the rail had no
  Home.
- **Status:** closed

## Summary
Marley puts its Home page first in the Default group's workspace (`home_page::fill_shown`, on
`ActiveWorkspaceChanged` and on a change to the groups, through `ensure`), and again when the last
tab closes (`fill_emptied`). On the Mac, the log said `rusty/home_tab.rs:193 … entity released`
right after the start: Home's cards finished loading after their page had been dropped. The likely
order: `fill_shown` adds Home, then Zed's session restore rebuilds the center pane with the saved
tabs (here Onboarding), which drops Home, and no event runs `ensure` again. Not yet checked on
Linux; a scenario that restores a saved tab in the Default group on start shows whether it holds
there too.

## Acceptance
After a relaunch that restores saved tabs in the Default group, Home is the group's first tab and
the rail lists it.
