---
pipeline_id: 8e4ae270-e0eb-479d-9c4c-b66202a4b114
ticket: docs/planning/tickets/open/TICKET-658-rusty-tasks-tab.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The Tasks tab: Rusty's to-do lists in a center tab"
type: feature
slice: Rusty in Marley R7, the Tasks half (rusty-in-marley.md R-D3, R-D9, R-D10)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/queued/644-brain-view-in-the-rail.spec.md, docs/planning/pipeline/completed/602-drag-to-reorder-the-rail.spec.md]
---

## Title
A Tasks center tab draws Rusty's to-do lists from `rusty-mcp`'s twelve task tools: the lists
(Rusty's task groups) in a column on the left, the chosen list's tasks on the right, in Rusty's
order. A task is added from a field above the list, checked done by its checkbox or Space, renamed
in its row, archived and restored, deleted for good after a prompt, and moved by a drag or by
Alt-Up and Alt-Down; lists are made, renamed and deleted the same way. Every change is one Rusty
tool call, read back. The tab opens from the rail Brain view's fixed row (#644) and from `rusty:
open tasks`. Rusty announces its own process's writes with `list_changed`, and the tab reads again
on it; a task another process writes (an agent's own `rusty-mcp`) is announced to no one until
Rusty's TICKET-035, so the tab also reads when it takes the focus and on its Refresh button. It is
the screen Rusty's Qt app draws as `TasksPage.qml`, and the Tasks half of the plan's slice R7.

## Scope
### In
- **Order:** after #644, in the batch's order. Built on #643 (the `marley.rusty` switch; the client
  in `marley_workbench::rusty` with its connection state, its tool call and its `list_changed`
  signal; `crates/marley_rusty` with `fixtures/` and the Python stand-in `stand_in/rusty-mcp`,
  named in `MARLEY_RUSTY_MCP`), on #644 (the rail Brain view, `rusty/brain.rs`, and its fixed row,
  where an entry appears with its tab; the stand-in's `SIGUSR1`) and on #645's `rusty` action
  namespace and the way it registers `rusty::OpenPage`. It takes their names as they ship. Where
  #647 has landed, its Graph entry sits before Tasks in the fixed row, R-D9's order.
- **The task model** (`marley_rusty::tasks`, pure, no gpui): the twelve tool names; `TaskList` and
  `Task`, typed views of Rusty's `TaskHeader` and `UserTask`; each answer's parse (a list of lists,
  a list of tasks, a new id, `toggle_task`'s new state, the other writes' words); `TaskWrite`, one
  variant per write tool with its arguments; the name rule (D4); the keyboard move (D8); the
  selection kept across a read (D3).
- **The stand-in** (#643's, with #644's additions and, if it landed first, #655's
  `list_task_groups` and `list_tasks` over `tasks.json` in its state folder): the twelve tools over
  that file, in Rusty's answer shapes and with Rusty's rules and refusals for the cases the
  scenario drives (D12).
- **The tab** (`marley_workbench::rusty::tasks_tab`, new): `TasksView`, an `impl workspace::Item`
  with `Focusable` and `Render`, titled "Tasks" with `IconName::ListTodo`; the lists column, the
  list's header (its name, Show Archived, Refresh), the add field, the task rows, the notice line
  (D2); the rows' checkbox, rename in the row, menu, archive, restore and delete prompt (D5); the
  lists' new row, rename, menu and delete prompt (D6); the drag (D7); the keys (D8).
- **Opening** (D1): `rusty::OpenTasks` (`rusty: open tasks`), registered with #645's `rusty`
  actions; the fixed row's Tasks entry (`IconName::ListTodo`, tooltip "Tasks") dispatching it
  through the window; one Tasks tab per workspace, focused if open, through
  `tasks_tab::open_later`.
- **Reads and refresh** (D9): on opening and on connecting; after each of the tab's own writes; on
  `list_changed` while the tab shows, else at its next showing; when the tab takes the keyboard
  focus from outside it and when Marley's window comes back to the front while it shows; on
  Refresh. One read in flight at a time.
- **Writes** (D10): one Rusty tool per change, one at a time in the order made, each read back; a
  refusal shown in a toast.
- **States** (D11): Rusty off, not connected, reading, no lists, an empty list, a failed read.
- **The rail's drag helpers:** `rail_order.rs`'s `drag_preview` and `drop_line` (#602) reachable
  from `rusty::tasks_tab`, so the tab draws the same preview and line (D7).
- **The keymap** (`crates/marley_workbench/keymap.json`): a `RustyTaskList && not_editing` block
  with the list's seven keys (D8).
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`): a Tasks article. It sits
  under `crates/marley_*`, which the commit receipt binds, so it changes in the Code phase.
- `script/e2e/658-rusty-tasks-tab.sh`.

### Out (explicitly deferred)
- **The project's task group link (#655):** a rail project's own list (R-D5's join, by the page's
  `task_group` property) and the Knowledge panel's project view of it are #655's. The tab's chosen
  list is set through one function, `TasksView::show_list`, which #655's view calls; nothing here
  guesses a project's list. The button on that view's Tasks header that opens this tab on the
  project's list comes with whichever of #655 and #658 lands second (#655's Out): if #655 has
  landed when this ticket is promoted, Plan adds the button here with a shot of its own.
- **The Decisions half of R7** (#659): a Decisions tab over `brain_due` and decision
  pages, with Marley's System One tab renamed "System One calls", the plan's open decision 4,
  which rides with it.
- **`changes_since`** (Rusty's TICKET-035, open, not built): nothing here depends on it. Until it
  lands, a write made by another `rusty-mcp` process (an agent's stdio instance adding a task)
  reaches the tab at its next read: its focus, Refresh, its own next write, or the next
  `list_changed` Marley's `rusty-mcp` sends for anything else (a vault file change, the sentinel
  `rusty-cli refresh` touches). With the cursor, the tab would poll it as the rail polls the
  harness's `fleet_events`. With `connection: service` no `list_changed` arrives at all (#643's
  D5).
- **Moving a task to another list, and a board of lists** (Ely's `KanbanBoard`): no Rusty tool
  changes a task's list (`header_id`), so it needs a Rusty-side tool first. Its scope, one line: a
  task dropped on a list in the column moves there, and a Board view shows the lists as columns,
  Ely's `KanbanBoard` ported.
- **Reordering the lists:** `task_headers` keeps a `sort_order`, but no tool sets it; a
  Rusty-side request.
- **What Rusty's store does not hold:** counts per list, due dates, notes on a task, subtasks, a
  link from a task to a brain page or a project, a "clear done" action. `user_tasks` holds a
  title, done, archived, an order and a creation time, and Rusty's page shows no more.
- **Keys for the lists column** (a list is chosen with the mouse, as on Rusty's page),
  multi-select, undo.
- **The tab restored after a restart** (`SerializableItem`) and the chosen list remembered: as the
  Agent and Graph tabs, it opens fresh.
- **Rusty's `rusty://tasks` resources:** the tab reads tools, as every screen of the plan does.
- Memory, Skills and Secrets (R8).

## Reference (§20)
Upstream Zed, kept as it is. The tab is a `workspace::Item`, opened or brought forward as Marley's
`decisions::open` does, and it reads only while it shows, as the Agent tab does (AD-609). Its
rows follow Zed's own lists: a checkbox per row that Space toggles, as the git panel's staging rows
(`git_ui/src/git_panel.rs:7952`, `assets/keymaps/default-linux.json:1062`); a rename in the row on
Enter or F2, Delete for the recoverable act and Shift-Delete for the permanent one after a prompt,
as the project panel (`project_panel.rs:805-830`, `:2768`; keymap `:1009-1013`); a row moved one
place by Alt-Up and Alt-Down, as the collab panel moves a channel (`collab_panel.rs:2330`, keymap
`:1196-1197`) and the notebook a cell (`:1566-1567`); and a drag that takes the target's place
behind a preview and a line on the side it lands, as the pane's tab bar drags a tab
(`workspace/src/pane.rs:2969-3000`), which Marley's rail follows (#602). The screen is Rusty's
`TasksPage.qml` (MIT, `/srv/stacks/rusty-v3/crates/rusty-app/qml/TasksPage.qml`), the page this
rebuilds. Warp has no to-do lists; no Warp source or spec used.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (the `Item`
  trait; `SerializableItem` a separate facet, `:76`, `:191`, why restoring is Out) and
  `09-vim-keymap-contexts.md` (the key-context predicate model D8's `not_editing` relies on).
  `docs/orca_architecture/02-worktrees-and-review.md` §2.5: Orca's sidebar has drag order,
  double-click rename and a board of statuses, the same row habits, and no to-do store.
  `docs/warp_architecture/`: no to-do list or reorderable list. The plan
  `docs/marley/rusty-in-marley.md` (R-D0, R-D2, R-D3, R-D8 to R-D10, Rusty's triage).
- **Published material:** Rusty (MIT), read at `295565c`, the store and the screen:
  `rusty-mcp/src/main.rs` (the parameters `:43-74`, `:180-203`, `:464-471`; the tools
  `list_task_groups` `:774`, `create_task_group` `:779`, `list_tasks` `:787`, `create_task` `:799`,
  `toggle_task` `:811`, `archive_task` `:819`, `reorder_tasks` `:963`, `rename_task_group` `:1507`,
  `delete_task_group` `:1520`, `update_task_title` `:1533`, `unarchive_task` `:1546`,
  `delete_task` `:1559`; `json_result` `:36-41`; `mutate` `:734-742`, which announces on its own
  process's bus only; `spawn_change_notifier` `:2062-2082`); `rusty-core/src/engine/user_tasks.rs`
  (`TaskHeader` `:10-18`, `UserTask` `:20-37`, new rows last `:74-92` and `:154-173`,
  `toggle_complete` `:175-193`, `reorder` `:228-252`, the other writes an `UPDATE` or `DELETE`
  that succeeds on a missing id); `rusty-core/src/lib.rs:17-73` (the watcher and the sentinel
  `~/.rusty/.changed` that `rusty-cli refresh` touches, `rusty-cli/src/main.rs:1024-1034`, for
  "DB-only changes"); Rusty's `TICKET-035-change-cursor.md` ("Why": a task an agent adds through
  its stdio `rusty-mcp` reaches no other process); `qml/TasksPage.qml` (332 lines, below in the
  notes with lines). The store skill `/tasks` writes `rusty.db` with raw SQL and then runs
  `rusty-cli refresh`. The MCP specification's `notifications/resources/list_changed`.
- **Code we already ship, Zed and `Cargo.lock`:**
  - `crates/ui`, `crates/gpui` and `crates/settings_ui` hold no reorderable list (searched for
    reorder and sortable: one hit, in gpui's text line code). Zed reorders by a pattern on gpui's
    drag API, not a component: the pane's tab bar (`pane.rs:2969-3000`, `DraggedTab` `:524`).
    Taken as #602 took it.
  - `ui::Checkbox` (`toggle.rs:43-156`, `label` `:140`), `ListItem` (`list_item.rs:32`,
    `on_secondary_mouse_down` `:185`, `start_slot` `:226`, `end_slot` `:231`), `Label`'s
    `strikethrough` (`label.rs:199`), `ContextMenu` (`context_menu.rs:213`, `build` `:284`),
    `IconButton`, `Tooltip`, `editor::Editor::single_line`, `Window::prompt`; icons `ListTodo`
    (`icons.rs:187`), `Plus` (`:213`), `RotateCw` (`:230`), and no grip icon. gpui's `on_drag`,
    `drag_over`, `can_drop`, `on_drop` (`elements/div.rs:568-615`), `Context::on_focus_in`, which
    fires only when the focus enters from outside (`app/context.rs:565-587`, `window.rs:311-313`),
    and `observe_window_activation` (`app/context.rs:420`). `workspace::item::Item`
    (`item.rs:177-262`, `deactivated` `:216`, called as a pane changes its active item,
    `pane.rs:1490`). `context_server`'s `on_notification`, through #643's client. Taken.
  - `Cargo.lock`: nothing owns a to-do list or a sortable list; `serde_json` parses the answers.
- **Marley's own:** `rail_order.rs` (#602: `DraggedRailRow` `:186-222`, `drag_preview` `:232-245`,
  `drop_line` `:248-257`, `draggable_row` `:340-369`; declared `#[path] mod order` in `rail.rs:89`)
  and `marley_rail::move_to` (`marley_rail.rs:1037-1052`, generic over the ids), both taken for D7;
  `decisions.rs:51-60` (`open`: focus the open tab of a type or add one) and `agent_tab.rs:31-61`
  (`open_later` in `window.defer`), D1's shape; `rusty.rs` (#633's offer, 90 lines, which #643
  grows into the client), #644's `rusty/brain.rs` and its fixed row, #645's `rusty` actions; the
  rail's list keys (AD-453) and filter (AD-457); `groups::GroupNamePrompt` (a modal name prompt,
  rejected for names in a row, as #644's D10 rejected it).
- **Ely GPUI Components (R-D10), read at `2f8b2f6`, nothing ported:**
  - `src/lists/sortable.rs` (`SortableList`, 150 lines; the `sortablelist-reorderablelist` story,
    `examples/gallery/pages/lists/rows.rs:110-131`): rows with a grip, a cursor that Up and Down
    move, Alt with Up or Down moving the current row and stopping at either end (`nudged`,
    `:17-24`), `on_reorder(from, to)`. Taken as behavior only: Alt-Up and Alt-Down move the
    selected task and stop at either end, which is also Zed's own binding for moving a channel, a
    cell and a line. No code.
  - `src/motion/reorder.rs` (`Reorder`, 226 lines): the held row follows the pointer while the
    others glide aside (`motion::Flip`, `src/motion/list.rs`, 283 lines), each row measured by a
    `canvas` into `use_keyed_state`, the landing place by the middles above (`landing`, `:59-76`).
    Left (D7).
  - `src/project/board.rs` (`KanbanBoard`, 473 lines; R-D10's other R7 story): cards moved between
    columns by `on_move(card, column, index)`. Left: no Rusty tool moves a task to another list
    (Out).
  - The sweep's answer: Zed's `ui` has no reorder component, but Zed's tab bar and Marley's rail
    already reorder lists on gpui's drag API, and Zed's own wins over a port (R-D10), so no Ely
    file is copied and no Ely notice is needed.

## UI proof
`script/e2e/658-rusty-tasks-tab.sh` (`compositor sway`: it clicks, right-clicks and drags).
Fixtures: a scratch repository opened with `open_path`, holding `notes.txt`; #643's stand-in linked
as `$E2E_WORK/bin/rusty-mcp` and named in `MARLEY_RUSTY_MCP` (never first on the PATH: Marley takes
its PATH from the login shell, L-531), its state in `$E2E_WORK/rusty` with a `tasks.json` the setup
writes and #644's scratch vault; every call logged with its pid. The made-up lists: Home (Water the
plants; Call the plumber, done; Return the library books; Fix the gate, archived; Buy stamps; Clean
the gutters), Work (Write the release notes; Review the pull request; Book the train) and Someday
(Learn the banjo). Setup sets `marley.rusty.enabled` true and `connection` `embedded` in the run's
settings; later changes are edits of that file from outside (L-607). A "quiet write" is the
scenario's own edit of `tasks.json`, which the stand-in does not watch, as another `rusty-mcp`'s
write to `rusty.db` is not announced; `kill -USR1` makes the stand-in send `list_changed`, as
`rusty-cli refresh` does. Never the user's Rusty (R-D8). Shots:
- `658-01-from-rail`: Brain in the rail's header, then Tasks in the fixed row: the Tasks tab;
  Home chosen above Work and Someday; Water the plants, Call the plumber struck through, Return the
  library books, Buy stamps, Clean the gutters; no Fix the gate.
- `658-02-from-palette`: `notes.txt` opened in front, then `rusty: open tasks`: the same tab in
  front, one Tasks tab in the tab bar.
- `658-03-list-chosen`: Work clicked: its three tasks.
- `658-04-added`: "Pack the charger" typed in the add field, Enter: listed last, the field empty
  and focused; then three spaces and Enter: the log holds one `create_task`.
- `658-05-done`: Write the release notes' checkbox clicked: struck through.
- `658-06-moved-selection`: Home; Water the plants clicked; Down: Call the plumber selected.
- `658-07-space`: Space: Call the plumber no longer struck through.
- `658-08-renamed`: F2, "Call the electrician", Enter: the new title.
- `658-09-archived`: Delete: Call the electrician gone, Return the library books selected.
- `658-10-archived-shown`: Show Archived clicked: Call the electrician and Fix the gate in their
  places, muted, each marked Archived.
- `658-11-restored`: Restore from Fix the gate's menu: Fix the gate drawn as open.
- `658-12-delete-prompt`: Show Archived off; Delete for Good… from Return the library books' menu:
  the prompt, the title in a code block.
- `658-13-deleted`: Delete answered: Return the library books gone.
- `658-14-dragging`: Clean the gutters pressed and moved over Water the plants, held: the preview
  under the pointer, a line on Water the plants' top edge, the rows in place.
- `658-15-dropped`: released: Clean the gutters, Water the plants, Fix the gate, Buy stamps; the
  log's `reorder_tasks` with those ids.
- `658-16-moved-by-keys`: Fix the gate clicked, then Alt-Up three times: Fix the gate, Clean the
  gutters, Water the plants, Buy stamps; the log holds two more `reorder_tasks`, the third press,
  at the top, calling nothing.
- `658-17-new-list`: New List, "Reading", Enter: Reading last and chosen, "Nothing open. Type above
  to add a task.", the add field focused.
- `658-18-list-renamed`: Rename from Reading's menu, "Books", Enter: Books.
- `658-19-list-delete-prompt`: Delete List… from Books' menu: the prompt, the name in a code block
  and its tasks going with it.
- `658-20-list-deleted`: Delete answered: Books gone, Home chosen.
- `658-21-live`: a quiet write adds Sweep the porch to Home, then `kill -USR1`: listed with no
  click.
- `658-22-read-on-show`: `notes.txt` in front; a quiet write adds Oil the hinges, then `kill
  -USR1`; the log shows no `list_tasks` after the signal; the Tasks tab clicked: Oil the hinges
  listed.
- `658-23-quiet-write`: the Brain view's search field clicked (the tab stays in front); a quiet
  write adds Wash the car: not listed.
- `658-24-read-on-focus`: the task list clicked: Wash the car listed.
- `658-25-refresh`: with the focus in the tab, a quiet write adds Feed the cat; Refresh clicked:
  listed.
- `658-26-refused`: Buy stamps removed by a quiet write, then its checkbox clicked: a toast with
  Rusty's "Query error: Query returned no rows"; Buy stamps gone after the read.
- `658-27-off`: `marley.rusty.enabled` set false: the tab says Rusty is off; the log shows no call
  after it.

## Locked-In Decisions
- D1 — **One Tasks tab per workspace, from the fixed row and the palette.** `rusty::OpenTasks`
  ("Opens Rusty's to-do lists in a center tab, or brings forward the one open.") is registered
  with #645's `rusty` actions, under the same rule as `rusty::OpenPage`. The fixed row of #644's
  Brain view gains Tasks (`IconName::ListTodo`, tooltip "Tasks"), dispatching the action through
  the window (L-515), after Graph where #647 has landed, else after Today. Both run
  `tasks_tab::open_later`, in `window.defer` since the opener walks the workspace's items
  (AD-609): the workspace's Tasks tab comes forward with the focus, else a new one goes into the
  active pane with the focus, as `decisions::open` does. The tab reads "Tasks", with
  `IconName::ListTodo` and the tooltip "Rusty's to-do lists". The chosen list is set only through
  `TasksView::show_list(id)`, the seam #655 calls. Rejected: a tab per list (Rusty's page switches
  lists in place, and so does the column); a dock panel (R-D3 puts Tasks in the center, and a list
  with an add field and a header needs the width).
- D2 — **Rusty's two columns.** The lists in a column of 240 px (Rusty's width, `TasksPage.qml:143`)
  under a "Lists" label with a New List button (`IconName::Plus`); a divider; then the chosen list:
  a header with its name in a large label, Show Archived (`ui::Checkbox` with its label) and Refresh
  (`IconName::RotateCw`, tooltip "Refresh"), the add field, the task rows in a scrolling column
  whose `ScrollHandle` keeps the selected row in view, and a notice line. The rows are a plain
  column, not a `uniform_list`: the store here holds 5 lists and 19 tasks, at most 7 open in one
  list (counted read only, 2026-10-03).
- D3 — **Rusty's order, as Rusty serves it.** Lists by their `sort_order`, tasks by theirs; a done
  task stays in its place, struck through, as on Rusty's page, not sorted to the end as the
  `/tasks` skill's listing does, so the order a drag sets is the order shown. With Show Archived
  on, `list_tasks` takes `include_archived: true` (Rusty's checkbox) and archived tasks show in
  their places. Across a read the selected task is kept by id, else the same index or the last
  row (Rusty's `Math.min`, `TasksPage.qml:72-74`); the chosen list is kept by id, else the first
  list is chosen (`:61-69`).
- D4 — **Names and titles are trimmed, and an empty or unchanged one is not sent.** Rusty stores
  any string it is given, an empty title included (`user_tasks.rs:74-92`, `:154-173`), so the rule
  is Marley's, as it is Rusty's page's (`TasksPage.qml:33`, `:39`, `:52`, `:117`). An empty add
  field calls nothing; an empty or unchanged rename cancels; an empty new list name drops the
  row.
- D5 — **A task row.** Zed's `ui::Checkbox` for done, whose click toggles through `toggle_task` and
  stops its propagation, so the row's own click does not also run (PR-a-button-in-a-card); the title
  as a `Label`, struck through and muted when done; an archived row muted, with a small "Archived"
  label at its end (Rusty's half opacity and "archived", `:249`, `:301`). One click selects the row
  and focuses the list; a double click on the title, Enter or F2 starts the rename: a single-line
  editor in place of the title with the title selected, Enter commits, Escape cancels, losing the
  focus commits, as the project panel's editor in a row (#644's D10). The row's right-click menu
  holds Rename, Archive (Restore on an archived row) and Delete for Good…, Rusty's three entries
  (`:128-135`); a menu deployed by hand stops and prevents its mouse-down (L-600). Archive asks
  nothing, since Restore undoes it. Delete for Good asks first with `Window::prompt` at Warning:
  "Delete this task for good?", the title in a fenced code block (the prevention rule from F-592),
  "Archive hides a task and keeps it.", and the buttons Delete and Cancel. The add field is an
  `Editor::single_line` reading "Add a task and press Enter" ("Create a list first" while no list
  exists, read only then); Enter sends `create_task`, empties the field and keeps the focus in it,
  and the task lands last, where Rusty puts a new one. Rejected: deleting for good without a prompt,
  as Rusty's page does (nothing brings the task back, and Shift-Delete sits beside Delete; Zed's
  project panel asks before its permanent delete); Rusty's grip handle (a whole-row drag is how the
  rail's rows and the tabs move, and Zed's icons hold no grip).
- D6 — **A list row.** Each list is a row with its name, the chosen one drawn pressed; one click
  chooses it. Its right-click menu holds two entries, Rename and Delete List…; New List puts an
  empty single-line editor row at the column's end: Enter calls `create_task_group`, chooses the new
  list by the id Rusty answers and moves the focus to the add field; Escape, or an empty name, drops
  the row. Rename edits in the row, as a task does. Delete List asks first, close to Rusty's words
  (`TasksPage.qml:106`): "Delete this list and every task in it?", the name in a fenced code block
  and "Its tasks, archived ones too, go with it."; Delete calls `delete_task_group`, and the first
  list left is chosen. Rejected: Rusty's modal Rename dialog (rows rename in place in Zed and in
  #644's tree); its always-shown "+ new list" field (a field under every column for a rare act).
- D7 — **The drag is Zed's tab bar's, as Marley's rail draws it, not Ely's.** A drag of a task row
  past gpui's drag threshold carries a `DraggedTask { list, id, position, title }`, a drag type of
  its own (L-602). What follows the pointer is #602's `drag_preview`, and the line is its
  `drop_line`, on the target's top edge when the target sat above the dragged task, on its bottom
  edge otherwise; every task row of the same list takes the drop (`can_drop`), and the drop puts the
  task in the target's place (#602's D6) by `marley_rail::move_to`. The new order of every shown
  task goes to `reorder_tasks { group_id, task_ids }`. With archived tasks hidden the ids sent are
  the shown ones, and Rusty keeps the hidden ones after them in their order
  (`user_tasks.rs:228-252`). A drop on the lists column, the header or below the rows changes
  nothing. The rows hold still for the whole drag, as the rail's do (#602's D5, F-602): a read that
  lands meanwhile is drawn when the drag ends. `rail_order.rs`'s two helpers become reachable from
  `rusty::tasks_tab` (`pub` in a module `rail.rs` declares `pub`, §14's rule for a module its
  siblings reach). Rejected: porting Ely's `Reorder`, where the held row follows the pointer and the
  others glide aside: the rail and every pane's tab bar already drag one way in this window, a
  second feel would sit beside them, and the port would bring Ely's `Flip` animation, its keyed
  state and a `canvas` measuring every row each frame; Ely's `landing` is not needed when a drop
  takes the target's place; a copy of #602's two helpers in the tab.
- D8 — **The keys.** The task list has its own focus handle and the key context `RustyTaskList`,
  inside the tab's `RustyTasks menu`; the list's context carries `not_editing` while no row editor
  is open, as the project panel's does (`default-linux.json:1029-1031`), so a space typed in a
  name is a space.
  - Up, Down, Home, End: Zed's list actions from the `menu` context, as the rail takes them
    (AD-453).
  - Space: toggle the selected task (Rusty's key; the git panel's toggle, `:1062`).
  - Enter and F2: rename it (the project panel's two keys, `:1009-1010`; Rusty's F2).
  - Delete and Backspace: archive it, or restore an archived one (Rusty's Delete; the project
    panel's keys for its recoverable trash, `:1011-1012`).
  - Shift-Delete: delete it for good, asked first (the project panel's permanent delete, `:1013`).
  - Alt-Up and Alt-Down: move it one place, and call nothing at either end (Zed's
    `collab_panel::MoveChannelUp` and `MoveChannelDown`, `:1196-1197`, its notebook's cells,
    `:1566-1567`, its editor's lines, `:544-545`; Ely's `SortableList`).
  - Escape: to the add field (Rusty's, `:235`).
  In the add field, Enter adds, Down goes to the list's first row (Rusty's, `:207`), and Escape
  clears the field, then propagates (AD-457, L-457). Space, F2, Delete, Backspace, Shift-Delete,
  Alt-Up and Alt-Down are bound in Marley's keymap under `RustyTaskList && not_editing`; Enter and
  Escape arrive as `menu::Confirm` and `menu::Cancel`, which the tab handles by where the focus
  is. Rejected: Rusty's Ctrl-Up and Ctrl-Down to move (Zed binds them to `editor::LineUp` and
  `LineDown`, and in the tab switcher to moving its selection, `:85-86`, `:1282-1283`); list keys
  of the tab's own (AD-453).
- D9 — **Reads and refresh.** A read is `list_task_groups`, then `list_tasks { group_id,
  include_archived }` for the chosen list, through #643's client and its bound, parsed off the main
  thread (L-482). The tab reads:
  - when it opens, and when #643's connection comes up;
  - after each of its own writes, once Rusty answers;
  - on #643's `list_changed` while it is its pane's active item in a window's shown workspace,
    asked at that moment (PR-607); otherwise it marks itself stale and reads at its next draw, as
    the Agent tab's render starts its reads (#609, AD-609). Marley's `rusty-mcp` sends
    `list_changed` for its own process's writes, for a vault or skill file its watcher sees change,
    and when `~/.rusty/.changed` is touched by `rusty-cli refresh`, which the `/tasks` skill runs
    after its raw SQL;
  - when it takes the keyboard focus from outside it (`on_focus_in`), and when Marley's window
    becomes active again while it shows (`observe_window_activation`): a task another `rusty-mcp`
    writes is announced to no other process (Rusty's `mutate` emits on its own bus; TICKET-035's
    "Why"), and these are the moments the user comes back from where an agent ran;
  - on Refresh.
  With `connection: service`, which receives no notification (#643's D5), the tab also marks
  itself stale when its pane shows another item (`Item::deactivated`), so each showing reads (as
  #647's D9). One read is in flight at a time, and any further asking folds into one more after it.
  Rejected: a poll on a timer (TICKET-035 is the poll Rusty is building, and a timer would read
  every few seconds for a change that rarely comes); reading `rusty://tasks` (the plan's screens
  read tools, and the resource leaves archived tasks out).
- D10 — **Writes go one at a time, each read back.** Each change is one tool: `create_task_group`,
  `rename_task_group`, `delete_task_group`, `create_task`, `toggle_task`, `update_task_title`,
  `archive_task`, `unarchive_task`, `delete_task`, `reorder_tasks`. Writes wait in a queue and run
  in the order made, each after the last one's answer, so two quick moves land in order. A success
  reads again (D9); a refusal shows the first line of Rusty's message in a toast
  (`NotificationId::unique::<TasksView>()`) and reads again, so what shows is what Rusty holds. A
  reorder is drawn at once, so the dropped row does not jump back while the call runs; the read
  after it confirms or undoes it. Nothing else is drawn before Rusty answers, a few milliseconds
  over stdio. Rusty answers most writes on an id it does not hold with success (an `UPDATE` or a
  `DELETE` that touches no row) and `toggle_task` with "Query error: Query returned no rows"; the
  read after either shows the truth. `create_task` into a list another process deleted also succeeds
  and leaves the task in no list (SQLite's foreign keys are off in Rusty's store); the read after it
  shows the list gone and chooses the first. Rejected: drawing every write before its answer (a
  refusal would flash).
- D11 — **Off, not connected, and the empty states.** With `marley.rusty.enabled` off the action
  answers as #645's `rusty:` actions do, and an open tab drops its lists, its queue and its tasks
  and says "Rusty is off. Turn it on in the Rusty section of the Marley settings." (#644's D4
  words), calling nothing; turned on again, it reads. While #643's connection is down or starting,
  the tab keeps what it last read, refuses edits (the field read only, the checkboxes and menus
  disabled, no drag) and says "Marley is not connected to Rusty:" with #643's reason; on connecting
  it reads. Before the first read: "Reading Rusty's lists…". No list: "No list yet. Make one with
  +.". An empty list: "Nothing open. Type above to add a task.", or with Show Archived "Nothing
  here" (Rusty's words, `:314`). A failed read keeps the last lists and shows the tool's first line
  in the notice line with a Read Again button.
- D12 — **Scenarios never touch the real store** (R-D8). #643's stand-in gains the twelve tools over
  `tasks.json` in its state folder (`$RUSTY_STAND_IN_STATE`); where #655 landed first, its two read
  tools and its file are kept as they are and the ten writes join them. The file is seeded from
  `fixtures/tasks.json` only when the state folder has none (#655's fixture, or, if #655 has not
  landed, one made-up list this ticket adds), and this ticket's scenario writes its own lists into
  the state folder before Marley starts, so its ids are its own, as #644's scenario fills its own
  vault. The file is read at each call and written after each write, answering as `json_result`
  does: a list's or a task's fields as `TaskHeader` and `UserTask` serialize, a new id as a number,
  `toggle_task`'s new state as a boolean, the other writes' words (`"renamed"`, `"deleted"`,
  `"archived"`, `"unarchived"`, `"updated"`, `"reordered"`), each as JSON text in a text block. It
  keeps Rusty's rules: a new list or task goes last; `reorder_tasks` puts the named ids first and
  refuses an id not in the list ("Task N is not in group G"); `toggle_task` on a missing id answers
  "Query error: Query returned no rows"; the other writes on a missing id succeed. It sends
  `list_changed` after each of its task writes and on `SIGUSR1` (#644's), and does not watch
  `tasks.json`, so the scenario's own edit stands for a write another process makes. Every list and
  task in the fixture and the scenario is made up.
- D13 — **What Ely gives: nothing copied.** Zed's `ui` has no reorderable list, but the pane's tab
  bar and Marley's rail (#602) already reorder lists on gpui's drag API, and R-D10 says Zed's own
  wins over a port. From `SortableList` the tab takes a behavior, Alt-Up and Alt-Down stopping at
  either end, which is also Zed's own binding (D8); `Reorder` is left (D7); `KanbanBoard` waits for
  a Rusty tool that moves a task between lists (Out). No Ely file is copied, so no Ely notice goes
  on any file.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks Tasks in the rail Brain view's fixed row while Rusty is on, the system shall open a Tasks tab with Rusty's lists on the left, the first chosen, and its tasks on the right. | Shot `658-01-from-rail` |
| REQ-002 | WHEN `rusty: open tasks` runs while the workspace has a Tasks tab, the system shall bring that tab forward and open no second one. | Shot `658-02-from-palette` |
| REQ-003 | The Tasks tab shall list a list's tasks in Rusty's order, a done task struck through in its place, and leave archived tasks out. | Shot `658-01-from-rail` |
| REQ-004 | WHEN the user clicks a list, the tab shall show that list's tasks. | Shot `658-03-list-chosen` |
| REQ-005 | WHEN the user types a title in the add field and presses Enter, the system shall add the task through `create_task`, list it last and empty the field. | Shot `658-04-added`; the stand-in's log |
| REQ-006 | The system shall send Rusty no title or name that is empty once trimmed, and no rename that leaves a title or name unchanged. | The log at `658-04-added` (one call); review |
| REQ-007 | WHEN the user clicks a task's checkbox, the system shall toggle it through `toggle_task` and show it done or not. | Shot `658-05-done`; the log |
| REQ-008 | WHILE the task list holds the focus, Zed's list keys shall move the selected task. | Shot `658-06-moved-selection` |
| REQ-009 | WHILE the task list holds the focus, Space shall toggle the selected task. | Shot `658-07-space`; the log |
| REQ-010 | WHEN the user renames a task in its row and presses Enter, the system shall change its title through `update_task_title`. | Shot `658-08-renamed`; the log |
| REQ-011 | WHEN the user archives a task, the system shall archive it through `archive_task` and drop it from the list. | Shot `658-09-archived`; the log |
| REQ-012 | WHILE Show Archived is on, the tab shall list archived tasks in their places, muted and marked Archived. | Shot `658-10-archived-shown` |
| REQ-013 | WHEN the user restores an archived task, the system shall bring it back through `unarchive_task`. | Shot `658-11-restored`; the log |
| REQ-014 | WHEN the user chooses Delete for Good, the system shall ask first, naming the task. | Shot `658-12-delete-prompt` |
| REQ-015 | WHEN the user confirms the delete, the system shall delete the task through `delete_task`. | Shot `658-13-deleted`; the log |
| REQ-016 | WHILE the user drags a task over another task of its list, the tab shall show a preview under the pointer and a line on the edge where the task would land. | Shot `658-14-dragging` |
| REQ-017 | WHEN the user drops a task on another task of its list, the system shall put it in that task's place and send the list's new order through `reorder_tasks`. | Shot `658-15-dropped`; the log |
| REQ-018 | WHEN the user presses Alt-Up or Alt-Down in the task list, the system shall move the selected task one place through `reorder_tasks`, and call nothing at either end. | Shot `658-16-moved-by-keys`; the log's count |
| REQ-019 | WHEN the user names a new list and presses Enter, the system shall make it through `create_task_group` and choose it. | Shot `658-17-new-list`; the log |
| REQ-020 | WHEN the user renames a list in its row and presses Enter, the system shall rename it through `rename_task_group`. | Shot `658-18-list-renamed`; the log |
| REQ-021 | WHEN the user chooses Delete List, the system shall ask first, naming the list and saying its tasks go with it. | Shot `658-19-list-delete-prompt` |
| REQ-022 | WHEN the user confirms the list's delete, the system shall delete it through `delete_task_group` and choose the first list left. | Shot `658-20-list-deleted`; the log |
| REQ-023 | WHEN Rusty sends `list_changed` while the Tasks tab is its pane's active item, the tab shall show Rusty's lists read again with no step by the user. | Shot `658-21-live` |
| REQ-024 | WHEN Rusty sends `list_changed` while the Tasks tab is hidden, the system shall hold the read until the tab next shows. | Shot `658-22-read-on-show`; the log |
| REQ-025 | WHEN the Tasks tab takes the keyboard focus from outside it, the system shall read Rusty's lists again. | Shots `658-23-quiet-write`, `658-24-read-on-focus` |
| REQ-026 | WHEN Marley's window becomes active while the Tasks tab shows, the system shall read Rusty's lists again. | Review |
| REQ-027 | WHEN the user clicks Refresh, the system shall read Rusty's lists again. | Shot `658-25-refresh` |
| REQ-028 | IF Rusty refuses a write, THEN the system shall show Rusty's message in a toast and the lists as Rusty reads them back. | Shot `658-26-refused` |
| REQ-029 | WHILE Marley is not connected to Rusty, the Tasks tab shall keep the lists last read, refuse edits and say why. | Review |
| REQ-030 | WHEN `marley.rusty.enabled` turns off while the Tasks tab is open, the tab shall drop its lists, say Rusty is off and make no further call. | Shot `658-27-off`; the log |
| REQ-031 | The Tasks tab shall change Rusty's lists and tasks only through Rusty's task tools, one call at a time in the order the user made them. | Review of the diff; the log |

## Phase Plan
- **P1 Plan** — promote after #644 ships (and with whatever of #645 to #647 has landed); re-read the
  names this spec takes: #643's connection state, tool call, `list_changed` signal and the
  stand-in's state folder; #644's fixed row and `SIGUSR1`; #645's `rusty` actions and their
  registration; whether #655 has landed (its `tasks.json`, its two read tools, and the Tasks header
  button that then comes here); re-verify that `on_focus_in` fires on a click on a pane's tab and
  that a pane's change of active item calls `Item::deactivated`; grep Zed's keymaps for D8's keys
  from the list and from each field (PR-a-views-key-needs-its-fields-context-too); ask the brain
  (`brain_ask`) on D7 and D9; confirm with Chad the prompt before Delete for Good (Rusty's page has
  none) and Alt-Up and Alt-Down over Rusty's Ctrl-Up and Ctrl-Down.
- **P2 Code** — the `README.md` marker first; `marley_rusty::tasks`; the fixture and the
  stand-in's tools; `rail_order.rs`'s helpers made reachable; `rusty/tasks_tab.rs`, its actions
  and registration; the fixed row's Tasks entry; the keymap block; the in-app guide page; a review
  of the diff (re-entrancy on the deferred opener, errors reaching the toast, the write queue, the
  keys from each field); `script/gates.sh --diff` green.
- **P3 Test** — write and run the scenario under sway, read every shot.
- **P4 Complete** — CHANGELOG (Added) and architecture docs (§21), the user docs the notes list,
  ledger capture (§19), `brain_decide` for D7 and D9, close the ticket, archive, commit.
