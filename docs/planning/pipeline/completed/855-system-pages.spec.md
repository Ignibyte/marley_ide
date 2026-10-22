---
pipeline_id: 6b0a64ed-3d3f-47a3-81c6-a755846b8751
ticket: docs/planning/tickets/open/TICKET-855-system-pages.md
status: Phase 4 — Complete PASS
title: A System tab with This build and Where data lives
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
`marley: open system pages` opens a System tab: a list of pages at its left and the chosen page
as the SystemPages board draws one, a title and what it covers, a lead line, rows (a dot, a name
over a detail, a value) and buttons. Two pages to start: This build (channel, version and commit,
data folder, platform; Copy build info) and Where data lives (Marley's folders with their sizes;
Open the folder).

## Scope
### In
- `marley_workbench/src/system_pages.rs` (new): the action, the tab, the two pages; folder sizes
  counted off the main thread.

### Out (explicitly deferred)
- Install and update, Troubleshooting, Shell integration, the MCP endpoint, Marley's Chromium,
  Fleet providers, and For developers: TICKET-885.
- Clear terminal history: a destructive button wants its own ticket and its confirmation.

## Reference (§20)
N/A — Marley-specific: pages Marley does not have yet; the SystemPages board.

### Prior art
- `component_library.rs` (a tab opened by an action), `release_channel` (`ReleaseChannel`,
  `AppVersion`, `AppCommitSha`), `paths` (`config_dir`, `data_dir`, `database_dir`,
  `logs_dir`, `extensions_dir`, `languages_dir`); Marley's own folders under `data_dir`
  (`terminal_history`, `browser`, `handoffs`).
- The brain (consultation 901fe435).

## UI proof
Scenario `script/e2e/855-system-pages.sh` (sway): a scratch repository.
- `855-01-build`: the tab with its page list and This build (REQ-001).
- `855-02-data`: Where data lives with each folder's size (REQ-002).
- `855-03-day`: Marley Day (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — A folder past 500 MB gets the warning dot and value; a missing folder reads "none yet".
- D2 — Sizes are counted when the page opens and again on its Count again button, never on a
  timer.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user runs `marley: open system pages`, Marley shall open a System tab listing its pages and showing This build: the channel, the version and commit, the data folder and the platform, with Copy build info, in Marley Night and Marley Day. | `855-01-build`, `855-03-day` |
| REQ-002 | WHEN the user picks Where data lives, the tab shall show each of Marley's folders with its path and size, with Open the folder. | `855-02-data` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
