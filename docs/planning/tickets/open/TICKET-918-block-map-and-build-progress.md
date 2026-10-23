# TICKET-918 — A block map by state, build progress, and the exit-127 hint

- **Ticket:** LOCAL #918 (feature, the Marley look, terminal blocks)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [918-block-map-and-build-progress.spec.md](../../pipeline/queued/918-block-map-and-build-progress.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/A-core.md,
  board TerminalSession: "Block map", "Determinate build progress", "Exit 127 with a
  distro-aware suggestion")
- **Boards:** TerminalSession
- **Size:** M
- **Status:** open

## Summary

Three things the TerminalSession board draws on a terminal that Marley does not. The right edge
gets a mark for every block, colored by its state, where today only bookmarked blocks get a
tick. A running cargo build shows how far it has got ("1781 of 1813") and a bar that fills to
that share, where today a bar slides with no meaning. When a command fails because its program
is missing, and that program is another distro's package manager, the block says which distro
this box runs and offers the same command in this box's own package manager ("This box runs
Arch. Try pacman -Qs libfoo").

## Acceptance

A terminal with a done, a failed, a running and a bookmarked block shows four marks in their
state colors; a running stand-in cargo build shows "N of M" and a filled bar; `dpkg -l | grep
libfoo` on this box offers `pacman -Qs libfoo`, which a click types at the prompt; shot in
Marley Night and Marley Day.
