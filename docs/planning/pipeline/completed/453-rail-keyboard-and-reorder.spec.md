---
pipeline_id: 30b95279-041c-487c-b20c-1d0de01136cc
ticket: docs/planning/tickets/closed/TICKET-453-rail-keyboard-and-reorder.md
status: Phase 4 — Complete PASS
title: Keyboard navigation and project reorder in the rail
type: feature
slice: workbench shell W6d
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/452-rail-rename-and-close.spec.md, docs/planning/tickets/closed/TICKET-457-rail-filter.md]
---

## Title
The rail works from the keyboard. With the rail focused, up and down move a highlight over its
rows, Home and End jump to the first and last, Enter opens what the row holds, left folds a
project or climbs to it, and right unfolds it. A project header's right-click menu moves the
project up or down. Narrowed at promotion (2026-09-23): the filter is TICKET-457.

## Scope
### In
- **The cursor** (`marley_rail`, pure): `Focus::cursor`, the row the keyboard is on while the
  rail holds focus; `selection` prefers it while its row is shown, so there is still one
  highlighted row. `step` (the next or previous shown row, stopping at the ends), `first_row`,
  `last_row` and `parent` (a terminal's or thread's project header) move it.
- **The keys** (`marley_workbench::rail`): Zed's defaults bind up and down
  (`menu::SelectPrevious`, `SelectNext`), Home and End (`SelectFirst`, `SelectLast`) and Enter
  (`Confirm`) with no context, and left and right (`SelectParent`, `SelectChild`) in the `menu`
  context, so the rail's key context becomes `MarleyRail menu`, as Zed's Threads Sidebar's
  does. Enter opens the row as a click does: a project shows, a terminal shows and takes
  focus, a thread opens in the Agent Panel. Left on an open project folds it, on a child row
  moves to its header; right on a folded project unfolds it. The cursor starts from the
  highlighted row and is dropped when focus leaves.
- **Reorder:** a project header's right-click menu with Move Project Up and Move Project Down
  (`MultiWorkspace::move_project_group_up`, `move_project_group_down`), each disabled at its end.

### Out (explicitly deferred)
- The filter field and `ctrl-f`: TICKET-457.
- Drag-and-drop reordering.
- Keys for rename and close on the selected row.

## Reference (§20)
- **Warp:** the session list is walked with the arrow keys and a session opens on Enter
  (observed on Chad's Warp, `docs/planning/design-notes/session-tabs-vs-sidebar.md`).
- **Upstream Zed:** the Threads Sidebar's keyboard model: its key context adds `menu`, and its
  `menu::SelectNext`, `SelectPrevious`, `SelectParent`, `SelectChild` and `Confirm` handlers
  walk and open its rows (`crates/sidebar/src/sidebar.rs:3287-3301`, `:3554-3570`,
  `:4342-4380`); `MultiWorkspace::move_project_group_up/down` for reorder
  (`crates/workspace/src/multi_workspace.rs:892-930`). Marley keeps the keys and the actions.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/09-vim-keymap-contexts.md` (key
  contexts); the observed Warp session list above.
- **Published material:** Zed's key-binding docs on contexts (`docs/src/key-bindings.md`).
- **Code we already ship.**
  - Zed's default keymap binds up, down, Enter, Home, End, Page Up and Page Down to `menu::*`
    with no context (`assets/keymaps/default-linux.json:3-22`), and left and right in `menu`
    (`:51-56`): a `menu` key context adds left and right, with no Marley binding.
  - The rail's own row model and handlers: `rail_rows`, `selection`, `toggle_expanded`,
    `activate_workspace`, `activate_terminal`, `open_thread` (#438, #439).
  - `ui::right_click_menu` and `ContextMenuEntry::disabled` for the header's menu (#452).

## UI proof
UI-AFFECTING.
- **Driven tests:** with the rail focused, `menu::SelectNext` and `SelectPrevious` walk the
  rows and stop at the ends, `SelectFirst` and `SelectLast` jump, `SelectParent` and
  `SelectChild` fold, unfold and climb, and `Confirm` opens a terminal row with focus in the
  terminal; the cursor is gone once focus leaves. With Zed's default keymap bound, the arrow keys
  and Enter do the same from simulated keystrokes. Move Project Down and Up reorder the rail.
  Unit tests for the pure cursor.
- **Live drive:** focus the rail, walk it with the arrows, open a terminal with Enter, reorder a
  project; screenshot each. It needs keys and clicks, so it runs only while Chad is away from
  the desk; otherwise the Test phase records why.

## Locked-In Decisions
- D1 — Zed's own keys: the context-free ones reach any focused list, and the `menu` key
  context, as Zed's sidebar has it, adds left and right; no Marley binding.
- D2 — One highlighted row: while the rail holds focus the cursor is the selection
  (`PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`).
- D3 — Enter does what a click does, through the same handlers.
- D4 — The cursor stops at the ends rather than wrapping, as Zed's project panel does.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the rail has focus, up and down shall move the one highlighted row to the next or previous shown row, stopping at the first and the last, and Home and End shall move it to the first and the last | unit tests + driven tests |
| REQ-002 | WHEN Enter is pressed on a highlighted row, the rail shall do what a click on that row does | driven test |
| REQ-003 | WHILE the highlight is on a project header, left shall fold an open project and right shall unfold a folded one; on a terminal or thread row, left shall move the highlight to its project's header | unit tests + driven tests |
| REQ-004 | WHEN the rail loses focus, the highlight shall return to the row the window shows | driven test |
| REQ-005 | WHEN Move Project Up or Down is chosen from a project header's menu, the project shall move one place in the rail, and each shall be disabled at its end | driven test |
| REQ-006 | With Zed's default keymap, the arrow keys and Enter shall reach these routes from the keyboard | driven test with simulated keystrokes |
| REQ-007 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the cursor in `marley_rail`, the key handlers and the header menu in `rail.rs`;
  fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run the tests; the live drive; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
