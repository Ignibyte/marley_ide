# TICKET-695 — Merge upstream Zed

- **Ticket:** LOCAL #695 (chore, upstream sync)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [695-merge-upstream-zed.spec.md](../../pipeline/completed/695-merge-upstream-zed.spec.md)
- **Source ticket:** Chad, 2026-10-08: "lets bring down upstream and ensure everything is good"
- **Status:** closed

## Summary
The fork carried upstream Zed only up to 2026-09-18 (`78648aaf7d`, v1.22). This merges
`upstream/main` as of 2026-10-08 (`9ab0715969`): 319 commits and 812 files. Conflicts are resolved
against `docs/marley/zed-touchpoints.md`. The merge then passes the gate and the e2e regression
set, and is installed. After this one, the merge repeats every two weeks or at each Zed release.

## Acceptance
- The fork builds on upstream's 2026-10-08 main, with every Marley hunk kept.
- The gate is green, and the golden e2e set passes.
