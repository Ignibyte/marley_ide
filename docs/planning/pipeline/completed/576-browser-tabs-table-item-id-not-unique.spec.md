---
pipeline_id: 0ccacbce-6025-48ed-9b9c-dffb35323335
ticket: docs/planning/tickets/closed/TICKET-576-browser-tabs-table-item-id-not-unique.md
status: Phase 4 — Complete PASS
title: "A Browser tab's saved row survives another workspace's tab of the same item id"
type: bug
slice: prong 3 (B1c, #494's restore)
references: [docs/planning/pipeline/completed/494-browser-restore.spec.md, docs/planning/pipeline/completed/575-terminal-id-across-restore.notes.md]
---

## Title
#494's `marley_browser_tabs` keeps `item_id INTEGER UNIQUE` beside `PRIMARY KEY(workspace_id,
item_id)`. An item's id is its entity id, which repeats across launches (L-claude-494), so a
Browser tab saved in one workspace under item id N replaces, through `INSERT OR REPLACE`, the row
another workspace's tab saved under N in an earlier launch, and that workspace's tab does not come
back. Zed's own `terminals` table had the same constraint and dropped it in a later migration.
Here a second migration rebuilds `marley_browser_tabs` without it, keeping its rows.

## Scope
### In
- `crates/marley_workbench/src/browser.rs` (`mod persistence`): a second entry in
  `MarleyBrowserTabsDb::MIGRATIONS` that creates the table again without `UNIQUE(item_id)`, copies
  the rows, drops the old table and renames the new one.

### Out (explicitly deferred)
- Anything else in the Browser tabs' restore (#494's rules stay).

## Reference (§20)
- **Upstream Zed:** `terminal_view/src/persistence.rs`, whose `terminals` migrations drop the
  same constraint ("Remove the unique constraint on the item_id table": a new table, a copy, a
  drop, a rename), and `sqlez`'s domain migrations, which run each new entry of `MIGRATIONS` once
  on an existing database. Marley follows the same steps for its own table.
- **Warp:** N/A. Marley's own persistence.

### Prior art
- **Behavior maps.** None apply.
- **Published material.** SQLite has no `ALTER TABLE … DROP CONSTRAINT`; the documented way is a
  new table, a copy, a drop and a rename.
- **The code we already ship.** Zed's `terminals` migration above; #575's `marley_terminal_ids`,
  keyed by the pair with no `UNIQUE(item_id)`. Does a crate we build own this seam? `sqlez` runs
  the migration; the table is Marley's.

## UI proof
UI-AFFECTING: which Browser tabs come back at a launch.
`script/e2e/576-browser-tabs-table-item-id-not-unique.sh` (`compositor sway`; an offline Chromium
serving two pages; two scratch repositories, repo-a and repo-b; Chromium stopped between
launches, so each restored tab opens its saved URL). Shots:
- `576-01-a`: repo-a with a Browser tab on page A.
- `576-02-b`: repo-b, in the next launch, with a Browser tab on page B.
- `576-03-a-again`: repo-a in a third launch, its Browser tab back on page A.

## Locked-In Decisions
- D1: A new migration, not an edit of the first: a database the first ran on must get the new
  table too.
- D2: The rebuilt table keeps the first's columns, keys and foreign key, less `UNIQUE(item_id)`,
  and every row.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN two workspaces' Browser tabs are saved in different launches under the same item id, each workspace shall restore its own tab. | Shot `576-03-a-again`; the run log: the saved rows, and `browser_tabs` after the third launch |
| REQ-002 | WHEN Marley starts on a database whose table has the old constraint, the migration shall keep the table's rows. | The run log: the table's schema and its rows after the first launch of the new build |
| REQ-003 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec, and the design in the notes.
- **P2 Code:** the migration. fmt and clippy clean; a review.
- **P3 Test:** write and run the scenario (red on the build before where the ids meet), read every
  shot, `just regress`, `script/gates.sh --diff`.
- **P4 Complete:** CHANGELOG (Fixed), the crate note, knowledge, close, archive, commit.
