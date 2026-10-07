# TICKET-674 — Every center tab has a row in the rail, a project's files under a Files row

- **Ticket:** LOCAL #674 (feature, the rail)
- **Owner:** unassigned (decided with Chad 2026-10-07)
- **Pipeline doc:** none yet
- **Source ticket:** `docs/planning/intake/rail-and-center-tabs.md` (decisions 1 and 2)
- **Status:** open

## Summary
Chad, 2026-10-06: "some things opened in the main pane dont show up on the left pane. So really
now we have two things that arent in sync." Asked whether to keep the top tabs or drop them, on
2026-10-07: "yes keep them and show up on the left". And on files: "Files makes sense to me put it
above containers".

The rail lists only center terminals and Browser tabs (`items_of_type`, `rail.rs:6119-6123`),
and highlights a row only when one of those two is in front (`active_rows`, `rail.rs:8209`).
Every other center tab gets a row under its project: files, Zed's own tabs (project search, diffs,
settings, previews, images) and Marley's (Harness and the rest that open in a project). A
project's file editors gather under one **Files (n)** row that folds like a project, starts open,
and sits above the project's own container rows. Whatever is in front has its row highlighted, or
the Files row while it is folded around it.

## Open points for Plan
- Which item kinds count as files (Zed's `ProjectItem`s with a path) and where the rest sit.
- Zed's preview tab, which a single click replaces: its row follows it, in italic as the tab is.
- The rows' order: the top tabs' order, pane by pane.
- Where the fold is kept: the window's saved blob, as the containers' fold was (#670).
- Follow the panes' add, remove and activate events, not a poll, so the rail never lags a tab.

## Acceptance
Opening a file, a project search and a Harness tab each adds a row under the project, and closing
it removes the row. Files sit under a Files row above the project's containers, open to start, and
fold with a click. The row of the tab in front is highlighted, and the Files row is highlighted
while folded around it. Proof: a scenario that opens and closes each kind and shots the rail beside
the tab bar.
