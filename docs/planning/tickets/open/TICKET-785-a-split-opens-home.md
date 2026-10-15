# TICKET-785 — A split opens Home, and a tab's menu splits it out

- **Ticket:** LOCAL #785 (feature, the Marley layout)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** none yet
- **Source ticket:** Chad, 2026-10-15: "one thing the split left and right we shouldnt force a
  terminal to opne. instead we need open a home instead." Approved as "When you split a terminal
  or agent tab left, right, up or down, the new pane opens on Home instead of a new terminal",
  then "yes. also you should be able to right click on a tab split it out in another direction."
- **Status:** open

## Summary
Splitting a center pane clones its active tab (`SplitMode::ClonePane`). A terminal or agent tab
clones through `TerminalView::clone_on_split`, which starts a new shell in the new pane, so every
split of a terminal forces one open. Home (#701, #782) cannot be cloned (`can_split` is false), so
splitting it does nothing. In center panes, a split of a terminal or agent tab, and of Home,
opens Home in the new pane instead. A file still splits as Zed's does, with the same file in both
panes. The terminal dock's own split, in the Zed layout, keeps giving a new terminal. A tab's
right-click menu has no split entries today; it gets Split Right, Left, Up and Down, which move
that tab, not only the active one, into a new pane on that side, as dragging it to that edge
does. A pane left empty by the move gets Home rather than staying blank.

## Acceptance
No center split opens a terminal the user did not ask for: the new pane shows Home, and any tab
can be split out to any side from its right-click menu.
