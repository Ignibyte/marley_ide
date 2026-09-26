# TICKET-576 — A Browser tab's saved row survives another workspace's tab of the same item id

- **Ticket:** LOCAL #576 (bug, prong 3, after #494)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/576-browser-tabs-table-item-id-not-unique.spec.md
- **Source ticket:** found in TICKET-575's Plan, 2026-09-26, reading Zed's `terminals` table beside #494's `marley_browser_tabs`
- **Status:** closed

## Summary
#494's `marley_browser_tabs` keeps `item_id INTEGER UNIQUE` beside its `PRIMARY KEY(workspace_id, item_id)`, the constraint Zed dropped from its own `terminals` table in a later migration because an item's id is its entity id, which repeats across launches (L-claude-494). With it, a Browser tab saved in one workspace under item id N replaces, through `INSERT OR REPLACE`, the row another workspace's tab saved under N in an earlier launch, and that workspace's tab no longer comes back when it is next opened. A migration that rebuilds the table without the constraint, as Zed's did, keeps each workspace's rows apart.

## Acceptance
With two workspaces whose Browser tabs were saved under the same item id in different launches, each workspace restores its own tab.
