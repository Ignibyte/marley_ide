---
pipeline_id: 3ebc76d4-ccfa-463c-ae92-e67ae9921a0a
ticket: docs/planning/tickets/closed/TICKET-820-header-menus-offer-what-the-plus-does.md
status: Phase 4 — Complete PASS
title: A header's right-click offers what its + does
type: bug
slice: the Marley layout's rail (#744 the + menu, #600 groups, #700 Default)
references: []
---

## Title
The rail's `+` and its right-click menus drifted apart: #744 gave the `+` one New Agent… picker and
Reopen Thread…, and the headers' right-click menus never had those or New Terminal. Chad expects a
right-click to offer what the `+` does.

## Scope
### In
- `Rail::new_entries`: the `+`'s entries in one builder (New Terminal, New Browser Tab, New Agent…,
  Reopen Thread…, New Agent in Worktree, Launch), which `render_project_menu` uses.
- An open project's, group's and Default's right-click menu starts with `new_entries`, then its
  own entries after a separator; Default's keeps its line after them.
- The empty space's menu: New Group…, then New Terminal, New Browser Tab, New Agent… and Reopen
  Thread… in Default, as it already places its entries; the old Agent CLIs list goes, since New
  Agent… lists every agent (#744).

### Out (explicitly deferred)
- A closed project's header (#606): no `+`, its right-click unchanged.
- #819's tmux entries: they join `new_entries` there.

## Reference (§20)
Upstream Zed: a project panel's right-click and its `+` share their New File/New Folder entries;
Zed's own pane tab bar `+` and its context menu likewise offer the same New items. Marley's rail
follows that: the right-click is the `+` plus the header's own entries.

### Prior art
- `crates/marley_workbench/src/rail.rs`: `render_project_menu` (the `+`, built inline),
  `project_context_menu`, `group_context_menu` (Default returns a label), `HeaderMenu`,
  `deploy_empty_menu` (its Agent CLIs list predates #744's picker), `worktree_agent_entries`,
  `launch_entries`, `in_home`.
- #744's notes: New Agent… is the one way to start an agent from the `+`.

## UI proof
`script/e2e/820-header-menus-offer-what-the-plus-does.sh`, under sway:
- `820-01-project-menu`: right-click `repo`'s header: New Terminal, New Browser Tab, New Agent…,
  Reopen Thread…, New Agent in Worktree, then Move Project Up… (REQ-001).
- `820-02-default-menu`: right-click Default's header: the same first entries (REQ-002).
- `820-03-empty-menu`: right-click the rail's empty space: New Group…, New Terminal, New Browser
  Tab, New Agent…, Reopen Thread…, and no Agent CLIs header (REQ-003).
- `820-04-terminal-from-menu`: New Terminal chosen from `repo`'s right-click opens a terminal in
  `repo` (REQ-004).

## Locked-In Decisions
- D1 — One builder for the `+` and the right-click, so they cannot drift again.
- D2 — The `+`'s entries come first in a right-click, then the header's own after a separator.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user right-clicks an open project's header, the menu shall start with the entries its `+` offers. | Shot `820-01-project-menu` |
| REQ-002 | WHEN the user right-clicks a projectless group's or Default's header, the menu shall start with the entries its `+` offers. | Shot `820-02-default-menu` |
| REQ-003 | WHEN the user right-clicks the rail's empty space, the menu shall offer New Agent… and Reopen Thread… and no separate list of agent CLIs. | Shot `820-03-empty-menu` |
| REQ-004 | WHEN the user picks an entry from a header's right-click, it shall do what the same entry of the `+` does. | Shot `820-04-terminal-from-menu` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `new_entries`, the three menus; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the scenario on the debug build, every shot read.
- **P4 Complete** — CHANGELOG, the crate note, close, archive, commit.
