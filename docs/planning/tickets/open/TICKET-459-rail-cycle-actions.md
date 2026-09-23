# TICKET-459 — Zed's project and thread cycling in the Marley layout

- **Ticket:** LOCAL #459 (feature, workbench shell)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from #454 at its promotion)
- **Source ticket:** ../../pipeline/completed/454-rail-switcher.spec.md (Out) · ../../../marley/workbench-shell.md (D8)
- **Status:** open

## Summary
Zed's `multi_workspace::NextProject`, `PreviousProject`, `NextThread` and `PreviousThread`
reach the sidebar's `cycle_project` and `cycle_thread` hooks, even while it is closed
(`crates/workspace/src/multi_workspace.rs:2105-2140`). The rail leaves both at the trait's
no-op defaults, so in the Marley layout the four palette entries do nothing. Zed's sidebar
walks its project headers and its shown thread and terminal rows, with wrap, and activates
the target (`crates/sidebar/src/sidebar.rs:7142-7267`). The rail can walk its own projects
and shown rows through the handlers its clicks use. No default keymap binds the four actions;
vim binds `] p` and `[ p` only in `ThreadsSidebar`.

## Acceptance
From the palette in the Marley layout, Next Project and Previous Project show the next or
previous project in the rail's order, and Next Thread and Previous Thread open the next or
previous terminal or thread row, wrapping at the ends. The EARS criteria come at promotion.
