---
pipeline_id: 9ce1a7c2-9ace-488d-a4ef-543fb0227e40
ticket: docs/planning/tickets/open/TICKET-452-rail-rename-and-close.md
status: Phase 4 — Complete PASS
title: Rename and close terminals from the rail
type: feature
slice: workbench shell W6c
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/442-rail-persistence.spec.md]
---

## Title
A terminal row in the rail can be renamed and closed from the row itself: a right-click menu
with Rename and Close, a close button on hover, and a double-click to rename. Rename starts
Zed's own inline rename in the terminal's tab, and Close closes the tab as Zed does, so both
behave, persist and prompt exactly as Zed's tab controls.

## Scope
### In
- **The row's menu:** a right-click on a terminal row opens a context menu with Rename and
  Close.
- **Rename:** shows the row's terminal as a click does, then runs Zed's rename on its view
  (`TerminalView::rename_terminal`), which edits the title inline in the terminal's tab; Enter
  keeps it (`set_custom_title`, which Zed persists) and Escape drops it. A double-click on the
  row does the same. A task's terminal is not renamed, as in Zed.
- **Close:** a close button on the row, shown on hover, and the menu's Close close the terminal
  through its pane (`Pane::close_item_by_id` with `SaveIntent::Close`), so Zed's prompt for a
  running task comes up as it does for the tab.
- **Agent rows keep a user's name:** a terminal renamed by the user shows that name on its
  agent row, where the CLI's own title showed before.

### Out (explicitly deferred)
- Renaming inside the row itself: Zed's rename editor lives in the tab.
- Thread rows: their menu is Zed's Agent Panel's.
- Closing a project from its header.

## Reference (§20)
- **Warp:** a session in the session list is renamed and closed from the list itself, with a
  close control on hover (observed on Chad's Warp, `docs/planning/design-notes/session-tabs-vs-sidebar.md`;
  the rail grammar in `docs/warp_architecture/observed/beautifului-2026-08-12-notes.md`).
- **Upstream Zed:** a terminal tab's Rename (`terminal::RenameTerminal`,
  `crates/terminal_view/src/terminal_view.rs:102-105`, `:471-515`, confirmed at `:1555`) and its
  close (`Pane::close_item_by_id`, `crates/workspace/src/pane.rs:1662`), kept exactly; the
  rail only starts them.

### Prior art
- **Behavior maps:** the observed Warp session list above; the gpui-era rail's rename (#177)
  in `docs/planning/pipeline/completed/`, Marley's own history.
- **Published material:** none beyond Zed's code.
- **Code we already ship.**
  - `TerminalView::rename_terminal` builds a single-line editor in the tab, selected, focused;
    `menu::Confirm` saves through `finish_renaming`, blur or Escape drops it; task terminals are
    skipped (`terminal_view.rs:471-515`). `set_custom_title` persists (`needs_serialize`) and
    emits `ItemEvent::UpdateTab` (`:421-429`), which the rail already follows.
  - The rail's row title is `tab_content_text`, which is the custom title once set; an agent row
    uses the CLI's OSC title instead (`rail.rs`, `terminal_snapshot`).
  - `TerminalView::is_dirty` is true only while a task runs (`:1788-1793`), so closing prompts
    only then, as the tab's close does.
  - `ui::right_click_menu` (`crates/ui/src/components/right_click_menu.rs`) for the row's menu;
    `ContextMenu` entries carry `MENU_ITEM-<label>` selectors for tests (#439).

## UI proof
UI-AFFECTING.
- **Driven tests** (display-only terminals, as the rail's tests run): the right-click menu
  lists Rename and Close; Rename shows the terminal and puts its tab into renaming, and typing
  and Enter rename the terminal and its row; a double-click on a row starts the rename; the
  hover close button and the menu's Close remove the terminal and its row; an agent row shows a
  user's name once set.
- **Live drive:** right-click a terminal row, rename it, then close another from its hover
  button; screenshot each. It needs clicks, so it runs only while Chad is away from the desk;
  otherwise the Test phase records why.

## Locked-In Decisions
- D1 — Zed's rename and close, not new ones: the rail starts them, so persistence, prompts and
  keys stay Zed's.
- D2 — Rename shows the terminal first, so the tab it edits is on screen.
- D3 — A user's name wins over an agent CLI's title on its row.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a terminal row is right-clicked, a menu shall open with Rename and Close | driven test |
| REQ-002 | WHEN Rename is chosen or the row is double-clicked, the row's terminal shall be shown and its tab shall enter Zed's rename, and a name typed and confirmed shall become the terminal's title and the row's | driven tests |
| REQ-003 | WHEN a row's close button or the menu's Close is used, the terminal shall close through its pane as its tab's close does, and its row shall leave the rail | driven tests |
| REQ-004 | WHILE an agent CLI runs in a terminal the user renamed, its row shall show the user's name | driven test |
| REQ-005 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the row's menu, close button, double-click and the two handlers in
  `rail.rs`; fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run the tests; the live drive; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
