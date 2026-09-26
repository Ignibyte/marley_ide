# TICKET-517 — Marley's own regression suite: a golden set that checks itself before an install

- **Ticket:** LOCAL #517 (chore, the e2e runner and the installer)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/517-regression-suite.spec.md
- **Source ticket:** Chad, 2026-09-25, on item 6 of the Orca questions (Marley's own regression
  suite, re-running old scenarios and a golden set before every install, each run checking
  itself through Marley's MCP server): "ok yes ill defer to you on that"; the Orca survey's
  engineering report (docs/orca_architecture/07, the golden e2e set)
- **Status:** closed

## Summary
Every Marley feature has an e2e scenario, and each runs once, at its own ticket's Test phase, where
someone reads its shots. Nothing runs it again, so a change that breaks an older feature goes
unseen until Chad hits it, possibly in the build he just installed. This ticket names a golden
set of scenarios, gives each one machine checks through Marley's own MCP server (the terminal
list and blocks, the Browser tab's pages and snapshot) so a run passes or fails with no one
reading shots, adds `just regress` to run the set against a given binary, and makes `just
install` run it against the release build it is about to install, keeping the installed one on
a red.

## Acceptance
`just regress` runs the golden set and prints a line per scenario and a verdict; a scenario whose
check fails exits non-zero naming the check; `just install` runs the set against the new release
build and installs it only when every scenario passed, unless `--skip-regress` says otherwise.
