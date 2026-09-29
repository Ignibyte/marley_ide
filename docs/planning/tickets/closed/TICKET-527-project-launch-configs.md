# TICKET-527 — A project's launch configs

- **Ticket:** LOCAL #527 (feature, workbench shell: the rail's +; Warp once-over item 4)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/527-project-launch-configs.spec.md
- **Source ticket:** Chad, 2026-09-25: "yes please" to Warp's Tab Configs (item 4 of `docs/planning/design-notes/warp-once-over-2026-09-25.md`), with Orca's repo-declared first tabs (`docs/orca_architecture/05-terminal-and-workspace.md` §2.7 and §3 item 6, `docs/orca_architecture/02-worktrees-and-review.md` §2.2)
- **Status:** closed

## Summary
The rail's + opens one thing at a time, so starting work on a project means opening the dev
server's terminal, Claude Code and a Browser tab on the server's URL by hand, every time. A file
in the project, `.zed/marley.json`, names launch configs: terminals that each run a command,
agent CLIs, and Browser tabs on a URL, each as a tab or split off the pane before it. The
project's + lists the configs, and one click opens a config's whole set. A command runs only
after the user approves the config's exact text: Marley keeps a hash of what was approved and
asks again when the text changes. Opening configs for each new worktree comes with #510, as the
second slice.

## Acceptance
The project's + lists the configs in `.zed/marley.json`; choosing one asks once for approval of
its exact text, then opens its terminals, agents and Browser tabs in their panes; a changed config
asks again, and a file that does not parse says why in the menu.
