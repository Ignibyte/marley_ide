---
pipeline_id: 4c761fe5-e412-4a65-a2d0-d1eb8d970474
ticket: docs/planning/tickets/open/TICKET-854-search-and-more.md
status: Phase 4 — Complete PASS
title: Open Recent as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Each row of Open Recent leads with the project's mark (its logo, or its letter on its tint, as the
rail draws it) and ends with where it is: "this window" for the window's own folders, "ssh" for a
remote project, else when it was last opened ("today", "yesterday", "3 d ago").

## Scope
### In
- `crates/recent_projects/src/recent_projects.rs`: a `MarleyRecentLook` global asked by
  `render_match` for both row kinds (a `Marley:` hunk); without it, Zed's rows as they are.
- `marley_workbench`: the global set at init, drawing `project_icons::project_mark` and the
  "where".

### Out (explicitly deferred)
- Project search and replace, the branch picker, history, extensions and the Marley Guide in a
  tab: TICKET-884.
- "open" for a project open in another window: the picker does not know it.

## Reference (§20)
Upstream Zed, `recent_projects`: the picker's rows (`render_match`); Marley adds a mark and a
label at a hook, as #839 did for the palette.

### Prior art
- `project_icons::{project_mark, want}` (#831); `RecentProjectEntry.timestamp`; #839's
  `MarleyPaletteGroups` hook pattern.
- The brain (consultation 6d2556b0).

## UI proof
Scenario `script/e2e/854-open-recent.sh` (sway): the run's copy of the database lists recent
projects; the scenario only looks and closes the picker with Escape.
- `854-01-recent`: each row's mark, its name over its folder, and where it is (REQ-001).
- `854-02-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — "when" counts whole days since the project was last opened.
- D2 — A remote project's mark is its letter: its folder is not on this machine to search.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Open Recent shall lead each row with the project's mark and end it with where the project is, in Marley Night and Marley Day. | `854-01-recent`, `854-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
