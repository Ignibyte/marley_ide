# TICKET-440 — Agent CLIs in rail terminals

- **Ticket:** LOCAL #440 (feature, workbench shell W4)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/440-rail-agent-clis.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
The Warp-style agent: a CLI such as claude or codex running in a terminal. The project's + gains New Agent, listing the agent CLIs found on PATH; each opens a center terminal at the project root and starts the CLI once the shell is ready. The rail recognizes an agent in any terminal by argv, labels the row with the title the CLI sets, and shows working, waiting or exited.

## Acceptance
New Agent > Claude Code opens a center terminal running claude at the project root. A terminal where claude was started by hand shows the agent row, and its status follows output, quiet spells and the bell. Full EARS in the queued spec.
