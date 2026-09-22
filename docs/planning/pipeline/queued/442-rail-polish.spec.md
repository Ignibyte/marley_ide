---
pipeline_id: b2afe478-3f5c-47e4-b19a-8c6cc94f5bc6
ticket: docs/planning/tickets/open/TICKET-442-rail-polish.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active for Phase 2 Design
title: Rail persistence and polish
type: feature
slice: workbench shell W6
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/queued/438-marley-layout-and-rail.spec.md]
---

## Title
Finish the rail. A rail Chad closes stays closed across restarts, and terminal rows can be
renamed and closed from the rail. The rail gains keyboard navigation and a filter field, and
`ctrl-tab` switches between recent terminals and threads. Projects reorder from the header
menu, and one width holds across both layouts.

## Scope
### In
- **Closed-rail memory:** the rail records a user close in its serialized state and closes
  itself after restore, deferred out of the `MultiWorkspace` update that restore runs in.
- **Width:** stored under the field names Zed's sidebar blob uses (`width`,
  `width_set_by_user`), so a width set in either layout carries to the other and neither side
  logs a parse error.
- **Rename:** a row's context menu and double-click rename a terminal through
  `TerminalView::set_custom_title` (Zed persists custom titles).
- **Close:** a row's `×` and context menu close the terminal item, with Zed's usual prompt
  when a process is running.
- **Keyboard:** the rail's key context (`MarleyRail`) binds up, down, enter, left and right to
  `menu::*` for selection, confirm and collapse, and `ctrl-f` to the filter.
- **Filter:** a filter field under the header narrows rows by fuzzy match on titles and
  project names (Zed's `fuzzy` crate).
- **Switcher:** `toggle_thread_switcher` opens an MRU switcher over terminals and threads;
  `ctrl-tab` in the rail's and the workspace's Marley contexts.
- **Reorder:** Move Project Up and Move Project Down in the project header menu, using
  `MultiWorkspace::move_project_group_up` and `move_project_group_down`.

### Out (explicitly deferred)
- Drag-and-drop reordering of rows or projects.
- Pinned rows and custom sections.

## Reference (§20)
- **Warp:** the session list can be searched ("Search tabs"), renamed and closed, and the
  active session switches from the keyboard (observed on Chad's Warp,
  `docs/planning/design-notes/session-tabs-vs-sidebar.md`; the rail grammar in
  `docs/warp_architecture/observed/beautifului-2026-08-12-notes.md`: a quick-search field with
  a keycap hint).
- **Upstream Zed:** the Threads Sidebar's filter, keyboard navigation and MRU switcher
  (`crates/sidebar`), and `MultiWorkspace`'s project reorder, kept as behavior references.

### Prior art
- **Behavior maps:** the observed Warp session list and rail grammar above; the gpui-era
  rail's search (`filter_sessions`, #112) and rename (#177), Marley's own code.
- **Published material:** Zed's `docs/src/ai/parallel-agents.md` (switching threads, the
  switcher binding).
- **Code we already ship:** `TerminalView::set_custom_title` and `custom_title`
  (`crates/terminal_view/src/terminal_view.rs:417-429`); pane item close with its prompts;
  `menu::{SelectNext, SelectPrevious, Confirm, SelectParent, SelectChild}`; the `fuzzy` crate
  (`StringMatchCandidate`, `match_strings`); `picker` for the switcher; `ui::ListItem` and
  `ContextMenu`; `MultiWorkspace::move_project_group_up/down` (`multi_workspace.rs:892-930`);
  the `SerializedSidebar` field names in `crates/sidebar/src/sidebar.rs:106-133` (read, not
  copied).

## UI proof
UI-AFFECTING.
- **Driven tests:** close the rail, serialize, restore, still closed; width round-trips across
  a layout switch; rename persists through a workspace reload; close removes the item; arrow
  keys and enter move and confirm the selection; the filter narrows rows; `ctrl-tab` cycles
  MRU order; reorder changes the header order.
- **Live drive:** restart the app with the rail closed and confirm it stays closed; rename a
  terminal, filter for it, switch with `ctrl-tab`; screenshot each step.

## Locked-In Decisions
- D1 — The rail's blob shares Zed's field names for width; extra fields are ignored by Zed.
- D2 — Close-state restore is deferred so no entity is updated while it is being updated.
- D3 — Rename uses Zed's custom title so persistence comes from Zed.
- D4 — The filter matches with Zed's `fuzzy` crate.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user closes the rail and the window is restored, the rail shall stay closed | driven test through serialize and restore |
| REQ-002 | WHEN the user sets the rail's width, the width shall survive a restart and a switch to Zed's sidebar and back | driven test |
| REQ-003 | WHEN a terminal row is renamed, the terminal's title shall change and survive a workspace reload | driven test |
| REQ-004 | WHEN a terminal row's close control is used, the terminal item shall close, prompting if its process is running | driven test |
| REQ-005 | WHILE the rail has focus, up and down shall move the selection, enter shall open the selected row, and left and right shall collapse and expand projects | driven test |
| REQ-006 | WHEN text is typed in the filter, the rail shall show only rows whose title or project fuzzy-matches it | unit test + driven test |
| REQ-007 | WHEN `ctrl-tab` is pressed in the Marley layout, a switcher shall open over terminals and threads in most-recently-used order | driven test |
| REQ-008 | WHEN Move Project Up or Down is chosen, the project's header shall move one place in the rail | driven test |
| REQ-009 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P2 Design** — the blob schema, the rail's key context and bindings, the filter's place in
  the row module, the switcher's data.
- **P3 Implement** — each item above.
- **P3.5 Inspect** — critics: restore re-entrancy, focus traps in the filter, keymap
  collisions, blob compatibility with Zed's sidebar.
- **P4 Validate** — tests, `script/gates.sh --diff`, the live drive.
- **P5 Complete** — CHANGELOG, crate note, ledger, close, archive; the plan's slice table
  marked done.
