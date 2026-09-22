# TICKET-438 — The Marley layout switch and the first rail

- **Ticket:** LOCAL #438 (feature, workbench shell W2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/438-marley-layout-and-rail.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
The core of the shell. A marley settings block carries marley.layout (zed or marley). A new crate, marley_workbench, builds either Zed's Threads Sidebar or the Marley rail for each window and swaps them live when the setting changes. It applies the Marley defaults in the Marley layout and restores Zed's on the way out. The first rail shows each project, its center terminals under it, a visible + with New Terminal, one selected row, and the window controls.

## Acceptance
Switching marley.layout swaps the sidebar in every window without a restart. The rail lists projects and their terminals; a click switches to the terminal and New Terminal opens one at the project root. Full EARS in the queued spec.
