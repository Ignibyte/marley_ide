# The Tasks tab: Rusty's to-do lists in a center tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-658-rusty-tasks-tab.md
- **Pipeline spec:** 658-rusty-tasks-tab.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, the idea behind the plan: "an all in one system for Marley ...
  so that you can manage these projects", then "maybe rusty becomes Marley. We would take our
  rusty custom QML app and build it inside of marley" and "Plan it now"
  (`docs/marley/rusty-in-marley.md`, What Chad asked for). The plan's R-D3 row for Rusty's
  TasksPage: "A **Tasks** tab over the twelve task tools"; R-D9's fixed row lists Tasks after
  Today and Graph; R-D10's table names `sortablelist-reorderablelist` and the Kanban story for
  R7. On the components: "lets make sure we use the gpui components we found here". The first
  batch (#643 to #647) was confirmed with "Queue all five"; the plan's slices table says R7
  follows the split-out slices, and this ticket is R7's Tasks half in the follow-up batch, #654 to
  #659, 2026-10-03. The project's own list is #655's; the Decisions half of R7 is #659.
- **Classification / tier:** feature, M (the plan sizes R7 M for Tasks and Decisions together;
  this is one half, with list editing and a drag). One pure module added to #643's
  `marley_rusty`, a fixture and twelve tools in the stand-in, one new tab module in
  `marley_workbench::rusty`, one entry in #644's fixed row, a keymap block, two of #602's helpers
  made reachable. No Zed touchpoint, no new dependency, no spawn site, no socket: every call goes
  through #643's client. If the Code phase runs long, the natural cut is the list editing
  (REQ-019 to REQ-022): lists can be made in Rusty's app or by the `/tasks` skill meanwhile, and
  the tasks side ships whole without it.
- **Recall (§18.3):**
  - AD-claude-453-the-rails-keys-are-zeds-list-actions-001: the rail binds no list key of its
    own and takes Zed's from the `menu` context; the tab's key context is `RustyTasks menu` for
    the same reason (D8).
  - AD-claude-457-the-rails-filter-is-zeds-sidebar-filter-001 and
    L-claude-457-a-single-line-editor-hands-zeds-list-keys-to-its-container-001: in a single-line
    editor Up, Down and Escape arrive on the container as `menu::SelectPrevious`, `SelectNext`
    and `menu::Cancel`, and Enter as `menu::Confirm` (only a full editor binds it); a container
    with nothing to do on Cancel calls `cx.propagate()`. The add field and the row editors rely
    on it.
  - AD-claude-452-the-rail-starts-zeds-own-rename-and-close-001: the rail renames terminals
    through Zed's own tab rename. A task has no Zed rename to start, so the project panel's
    editor in the row is the Zed behavior to follow, as #644's D10 found for vault rows.
  - AD-claude-602-the-rail-owns-its-order-001 and
    L-claude-602-nested-drop-targets-need-a-drag-type-each-001: a drop goes to the deepest element
    with an `on_drop` of its type, which takes the drag before `can_drop` says no; hover reads
    false during a drag; a successful drop stops propagation, so the drop handler ends what the
    drag began. The tab's drag gets its own type, `DraggedTask`, and only task rows take it.
  - F-claude-602-a-drag-begun-after-the-hold-was-let-go-held-nothing-001 and #602's D5: the rows
    must hold still for the whole drag, whatever else arrives. A read that lands mid-drag (a
    `list_changed`, a focus) would move rows under the pointer and stale the drag's position, so
    the tab holds a read's answer until the drag ends (D7).
  - AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001: a center tab not
    restored after a restart, reading while it is its pane's active item, every open through
    `Window::defer` because the opener walks the workspace's items. D1 and D9 follow it.
  - F-claude-607-polling-followed-set-active-not-what-shows-001 and
    PR-claude-607-ask-the-dock-whether-a-panel-shows-001: whether a surface shows is asked at the
    moment, not counted from events. D9's "shows" is asked of the pane and the window when
    `list_changed` arrives.
  - L-claude-515-dispatch-through-the-window-from-inside-an-action-001 and
    F-claude-515-an-app-dispatch-inside-an-action-found-no-window-001: the fixed row's entry
    dispatches `rusty::OpenTasks` through the window.
  - F-claude-600-a-hand-deployed-menu-lost-the-keyboard-to-the-rails-own-focus-001 and
    L-claude-600-a-hand-deployed-menu-stops-and-prevents-its-mouse-down-001: the rows' menus.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001 and
    L-claude-635-a-scenarios-fixed-menu-steps-can-pass-on-the-wrong-entry-001: the scenario walks
    menus by position and reads the shot to see the entry it reached.
  - L-claude-581-a-prompt-opened-from-a-context-menu-keeps-the-keys-001: Zed's prompt opened from
    a menu entry takes Enter and Escape; after the answer no element has focus, so the scenario
    clicks before its next keys, and the tab's focus-in read (D9) runs on that click.
  - PR-claude-a-repositorys-text-in-a-zed-prompt-goes-in-a-code-block-001 (from F-592): a prompt's
    text is Markdown on Linux; a task title or a list name is text that can come from an agent,
    so it goes in a fenced code block (D5, D6).
  - PR-claude-a-views-key-needs-its-fields-context-too-001: a key bound in a view's context also
    fires from its fields unless the binding says otherwise; Space, Delete and Backspace must
    reach the add field and the row editors as text. Hence the list's own context and
    `not_editing` (D8).
  - PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001: the checkbox stops its
    click (D5).
  - F-claude-438-a-a-sidebar-flag-read-a-value-only-render-wrote-001 and
    L-claude-572-an-observed-global-written-at-every-wakeup-redraws-the-rail-001: the stale mark is
    written by the `list_changed` handler and `deactivated`, not by render; render only reads it
    and starts the read.
  - PR-claude-a-state-another-view-lists-is-announced-by-an-event-001: the tab listens to #643's
    `list_changed` signal and connection state the way #643 announces them (an event or a
    global), checked at promotion.
  - L-claude-534-zeds-mcp-client-sees-no-server-exit-001: #643 owns the connection's state; the tab
    shows Rusty's messages, which survive Zed's client.
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001 and
    L-claude-507-sync-work-for-the-background-executor-goes-in-future-lazy-001: the answers are
    parsed in `cx.background_spawn(futures::future::lazy(..))`.
  - L-claude-531-marley-takes-its-path-from-the-login-shell-so-stand-ins-are-named-001,
    L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001 and
    L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001: the stand-in is named
    in `MARLEY_RUSTY_MCP`; the harness keeps Rusty off in every run's copy; the scenario turns
    Rusty on and off by editing the run's settings from outside.
  - L-claude-635-a-harness-helper-replaces-a-scenarios-own-of-the-same-name-001: the scenario's
    helpers (`quiet_add_task`, `quiet_remove_task`, `calls_of`) take names no harness helper has.
  - L-claude-465-a-doc-opens-with-one-short-line-001 and
    L-claude-504-rustdoc-checks-what-clippy-and-dylint-do-not-001: two new modules with docs.
  - Completed pipelines read: 453 (the rail's keys and Move Project Up and Down), 457 (the
    filter), 565 (`decisions::open`), 581 (a prompt from a menu), 600 (menus and
    `GroupNamePrompt`), 602 (drag to reorder), 609 (the Agent tab), 633 (the stand-in scenario).
    Queued: 643 (the client, the stand-in, the harness lines), 644 (the fixed row, `SIGUSR1`,
    the menus and the rename in the row), 645 (the `rusty` actions), 646, 647 (D9's stale rule and
    the service connection's showings).
  - Brain (`rusty-cli brain search`, read only): no page on a tasks screen in Marley;
    `decisions/the-rail-renames-and-closes-terminals-through-zeds-own-tab-rename-and-close`
    (AD-452 above) and
    `decisions/marley-draws-rustys-knowledge-workspace-rusty-keeps-its-data-marley-d11-amended`
    (D11: Rusty is the only writer, hence D10). Promotion asks the brain on D7 and D9.
- **Discovery:**
  - Rusty's tools (`/srv/stacks/rusty-v3/crates/rusty-mcp/src/main.rs`, HEAD `295565c`): the
    parameters `ListTasksParams` `:43-51` (`group_id`, `include_archived` default false),
    `CreateTaskParams` `:53-60`, `TaskIdParams` `:62-67`, `GroupNameParams` `:69-74`,
    `RenameGroupParams` `:180-187`, `GroupIdParams` `:189-194`, `UpdateTaskTitleParams`
    `:196-203`, `ReorderTasksParams` `:464-471` ("tasks left out follow in their current order");
    the tools at `:774`, `:779`, `:787`, `:799`, `:811`, `:819`, `:963`, `:1507`, `:1520`,
    `:1533`, `:1546`, `:1559` (twelve, listed again at `:2134-2144` and `:2184`). Reads go
    through `json_result` (`:36-41`: JSON text in a text block, an error as a JSON-RPC internal
    error carrying the message); writes through `mutate` (`:734-742`), which emits `DataChanged`
    on its own process's bus, and `spawn_change_notifier` (`:2062-2082`) sends
    `notifications/resources/list_changed` to that process's peers only. Answers:
    `list_task_groups` `[{id, name, sort_order}]`; `list_tasks` `[{id, header_id, title,
    completed, archived, sort_order, created_at}]`; `create_task_group` and `create_task` the new
    id; `toggle_task` the new state; `archive_task` `"archived"`, `unarchive_task`
    `"unarchived"`, `update_task_title` `"updated"`, `delete_task` and `delete_task_group`
    `"deleted"`, `rename_task_group` `"renamed"`, `reorder_tasks` `"reordered"`. Resources
    `rusty://tasks` (every list with its unarchived tasks, `:752-757`, `:1924-1927`) and
    `rusty://tasks/{group_id}` (`:1962-1964`): not used (D9).
  - Rusty's store (`rusty-core/src/engine/user_tasks.rs`): `TaskHeader` `:9-18`, `UserTask`
    `:20-37`; `list_headers` by `sort_order` `:53`; `create_header` and `create_task` put the new
    row last (`:74-92`, `:154-173`) and take any string; `list_tasks` by `sort_order`, archived
    left out unless asked (`:125-153`); `toggle_complete` returns the new state and fails with
    "Query error: Query returned no rows" on a missing id (`:175-193`); `archive_task`,
    `unarchive_task`, `update_title`, `delete_task` (`:196`, `:207`, `:218`, `:255`),
    `rename_header` and `delete_header` (the tasks first, then the list) succeed on a missing id;
    `reorder` (`:228-252`) refuses an id not in the list ("Task N is not in group G"), drops
    repeats, puts the named ids first and the rest after in their order, archived ones included.
    No tool sets a list's `sort_order` or a task's `header_id`.
  - Live refresh in Rusty: `rusty-core/src/lib.rs:17-73` watches the notes, the vault, the skills
    and the sentinel `~/.rusty/.changed` ("DB-only changes (tasks/memories) that aren't markdown
    files"), 600 ms debounce, then `DataChanged`; `rusty-cli refresh` writes the sentinel
    (`rusty-cli/src/main.rs:1024-1034`); the store skill `/tasks`
    (`~/.rusty/skills/.claude/skills/tasks/SKILL.md`) writes `rusty.db` with raw SQL and runs
    `rusty-cli refresh` after each change, so its writes reach Marley's `rusty-mcp` as
    `list_changed`. An agent's own stdio `rusty-mcp` calling `create_task` touches no sentinel:
    Rusty's `docs/planning/tickets/open/TICKET-035-change-cursor.md` ("Why") states it.
  - Rusty's page (`crates/rusty-app/qml/TasksPage.qml`, 332 lines): the keys in its header
    comment (`:5-9`: Enter adds, arrows move, Space toggles, F2 renames, Delete archives,
    Ctrl+Up and Ctrl+Down reorder or drag the handle, Escape returns to the add box); `refresh`
    `:22-29`; the writes `:32-54` (trimmed, empty refused); every write followed by a read
    `:77-88`; `onDataChanged` reads all `:92`; the list delete dialog `:97-107` ("Delete the list
    "…" and every task in it?"); the modal rename `:109-119`; the menus `:121-135` (Rename…,
    Delete list…; Rename (F2), Archive (Delete) or Restore, Delete for good, with no prompt); the
    240 px column `:143`; "+ new list" `:176-181`; Show archived `:196`; the add field `:198-208`
    (Escape to the list, Down into it); the list's keys `:229-238`; the drag handle `:255-277`;
    the checkbox `:278-282`; the strikeout `:290`; the row editor `:292-300`; "archived" `:301`;
    the double tap `:307`; the empty texts `:311-318`; the notice line `:320-329`.
  - The store on this box, counted read only (`sqlite3 -readonly`, counts only, 2026-10-03): 5
    lists, 19 tasks (2 done, 2 archived), at most 7 unarchived in one list.
  - Marley:
    - `crates/marley_workbench/src/rusty.rs` (90 lines, #633's offer; #643 grows it into the
      client, #644 adds `mod brain;`, #645 `actions!(rusty, …)` and `page`, #647 `graph_tab`).
    - `crates/marley_workbench/src/decisions.rs:51-60` (`open`: `items_of_type`, then
      `activate_item` or `add_item_to_active_pane`); `agent_tab.rs:31-61` (`open_later` in
      `window.defer`, then `open`).
    - `crates/marley_workbench/src/rail_order.rs` (#602): `DraggedRailRow` `:186-222` with its
      `Render`; `drag_preview` `:232-245` and `drop_line` `:248-257`, private; `draggable_row`
      `:340-369` (`on_drag`, `can_drop`, `drag_over` with `drop_line`, `on_drop`). Declared in
      `rail.rs:89-90` as `#[path = "rail_order.rs"] mod order;`; `rail` is a private module of
      the crate root (`marley_workbench.rs:65`).
    - `crates/marley_rail/src/marley_rail.rs:1037-1052` (`move_to(order, moved, target,
      before)`, generic, pure); `marley_workbench/Cargo.toml:45` already depends on it.
    - `crates/marley_workbench/keymap.json`: `alt-up` and `alt-down` are bound only under
      `Terminal` (`:41-43`, bookmarks); nothing binds the tab's keys elsewhere in it.
    - `crates/marley_workbench/guide/index.html` (1,775 lines; the System One and Decisions
      articles `:241-243`).
  - Zed:
    - `crates/workspace/src/item.rs:177-262` (`Item`: `tab_content_text`, `tab_icon`,
      `tab_tooltip_text`, `deactivated` `:216`); `pane.rs:1490` (`deactivated` on a change of the
      active item); `pane.rs:524` (`DraggedTab`), `:2969-3000` (its `on_drag`, `drag_over` with a
      2 px border on the side it lands, `on_drop`).
    - `crates/gpui/src/elements/div.rs:568-615` (`on_drop`, `can_drop`, `on_drag`);
      `crates/gpui/src/app/context.rs:420` (`observe_window_activation`), `:565-587`
      (`on_focus_in`: "does not fire if the given focus handle - or one of its descendants - was
      previously focused"), `window.rs:311-313` (`is_focus_in`).
    - `crates/ui/src/components/toggle.rs:15`, `:43-156` (`Checkbox`, `label` `:140`, `on_click`
      `:90`, `disabled` `:78`); `list/list_item.rs:32`, `:185`, `:226`, `:231`;
      `label/label.rs:199` (`strikethrough`); `context_menu.rs:213`, `:284`;
      `crates/icons/src/icons.rs` (`ListTodo` `:187`, `Plus` `:213`, `RotateCw` `:230`, `Archive`
      `:32`, `Trash` `:285`; no grip or drag icon).
    - `crates/git_ui/src/git_panel.rs:7952` (a `Checkbox` per row);
      `crates/project_panel/src/project_panel.rs:805-830` (the editor in a row), `:2768` (the
      removal prompt); `crates/collab_ui/src/collab_panel.rs:2303`, `:2330` (`move_channel`,
      `move_channel_up`).
    - `assets/keymaps/default-linux.json`: `menu` list keys `:5-17`; `ctrl-up` and `ctrl-down`
      `editor::LineUp` and `LineDown` `:85-86`; `alt-up` and `alt-down` `editor::MoveLineUp` and
      `MoveLineDown` `:544-545`; the project panel's Enter, F2, Backspace, Delete and Shift-Delete
      `:1009-1013` and its `not_editing` space `:1029-1031`; the git panel's space `:1062`; the
      collab panel's `MoveChannelUp` and `MoveChannelDown` `:1196-1197`; the tab switcher's
      `ctrl-up` and `ctrl-down` `:1282-1283`; the notebook's cells `:1566-1567`.
    - `crates/ui/src`, `crates/gpui/src`, `crates/settings_ui/src`: no reorder or sortable list
      (grep: one hit, `gpui/src/text_system/line.rs`).
  - Ely (`repos/ely` in the scratchpad, HEAD `2f8b2f6`, `LICENSE-MIT:3` "Copyright (c) 2026 Ely
    GPUI Component contributors"): `src/lists/sortable.rs` (150 lines; `nudged` `:17-24`,
    `SortableList` `:26-56`, the Alt key `:115-134`, a grip `IconName::GripVertical` `:99`);
    `src/motion/reorder.rs` (226 lines; `Hold` `:17-24`, `landing` `:59-76`, the measuring
    `canvas` `:120-139`, `on_drag_move` and `on_drop` `:172-208`); `src/motion/list.rs` (`Flip`
    `:63`); `src/project/board.rs` (473 lines, `KanbanBoard` `:111`, `on_move` `:134`); the story
    `examples/gallery/pages/lists/rows.rs:110-131` ("Drag a row to its new place, or focus the
    list and press Alt with Up or Down. The others glide aside.").
  - The e2e runner: `script/e2e.sh` `press` `:407` (modifiers under sway through `wtype`),
    `pointer_to`, `pointer_down`, `pointer_up`, `click`, `click_with` `:500-545`,
    `profile_setting` `:562`; #643's harness lines keep Rusty off in every copy; #644's scenario
    reaches the Brain view.
- **Decisions:** D1 to D13 in the spec. In short: one Tasks tab per workspace, from the fixed row
  and the palette through a deferred opener, its list set through one function #655 calls;
  Rusty's two columns; Rusty's order with done tasks in place; trimmed names, never empty or
  unchanged; rows with Zed's checkbox, a rename in the row, Rusty's menu and a prompt before
  Delete for Good; lists made and renamed in a row and deleted after a prompt; the drag is #602's
  preview and line on Zed's tab-bar pattern, with `marley_rail::move_to`, not Ely's `Reorder`;
  the list's keys from Zed's lists, Alt-Up and Alt-Down to move, in a context of the list's own
  with `not_editing`; reads on opening, connecting, each write, `list_changed` while shown (else
  stale), the focus coming in, the window coming back, and Refresh, one in flight; writes one at
  a time and read back, a reorder drawn at once; off empties, down keeps and refuses; the
  stand-in's twelve tools over a file it does not watch; nothing of Ely copied.

### Design
- **`marley_rusty::tasks`** (new file `crates/marley_rusty/src/tasks.rs`, `pub mod tasks;` in the
  crate root #643 makes; no Ely code, no notice):
  - The tool names as constants, the twelve of them.
  - `TaskList { id: i64, name: String }` and `Task { id: i64, list: i64, title: String, done:
    bool, archived: bool, created_at: i64 }`, deserialized from Rusty's fields (`header_id` as
    `list`, `completed` as `done`; `sort_order` read and dropped, since every answer comes in
    order).
  - `lists_from_answer(&str)`, `tasks_from_answer(&str)`, `id_from_answer(&str)`,
    `done_from_answer(&str)`, each `Result<_, String>` with an error naming what did not parse; a
    write's word is checked by the call's success, not its text.
  - `TaskWrite { NewList { name }, RenameList { id, name }, DeleteList { id }, NewTask { list,
    title }, Toggle { id }, Retitle { id, title }, Archive { id }, Restore { id }, Delete { id },
    Reorder { list, ids } }` with `tool(&self) -> &'static str` and `arguments(&self) ->
    serde_json::Value` in Rusty's parameter names; a `TaskWrite` is what the tab's queue holds
    (D10).
  - `typed_name(typed: &str, current: Option<&str>) -> Option<String>`: trimmed; `None` when
    empty or equal to `current` (D4).
  - `moved_one(ids: &[i64], id: i64, up: bool) -> Option<Vec<i64>>`: the order with `id` one
    place up or down, `None` at either end or for an id not listed (D8; Ely's rule as behavior,
    written here).
  - `kept_task(tasks: &[Task], selected: Option<i64>, index: Option<usize>) -> Option<usize>` and
    `kept_list(lists: &[TaskList], chosen: Option<i64>) -> Option<i64>` (D3).
- **The scenario's lists**, written by its setup into `$RUSTY_STAND_IN_STATE/tasks.json` before
  Marley starts, in the file's shape (#655's, if it landed first; else `{ "next_id", "lists",
  "tasks" }` in Rusty's field names): Home 1 (tasks 11 to 16 in the spec's order, 12 done, 14
  archived), Work 2 (21 to 23) and Someday 3 (31), `next_id` 40. `fixtures/tasks.json`, which the
  stand-in seeds from when the state has no file, is #655's; if #655 has not landed, this ticket
  adds it with one made-up list and two tasks.
- **The stand-in** (`crates/marley_rusty/stand_in/rusty-mcp`, #643's Python, with #644's vault tools
  and `SIGUSR1`, and #655's two read tools if it landed first): the twelve tools in `tools/list` and
  `tools/call`, over `$RUSTY_STAND_IN_STATE/tasks.json`, seeded from the fixture when the state has
  none (found beside the script through its real path, as `settings.json` is), read at each call and
  written after each write with a rename into place; Rusty's answers, rules and refusals as the
  spec's D12 lists them; each call logged in `calls` as #643 logs them; `list_changed` after each
  task write. The file is not watched (#643 watches only `settings.json`).
- **`marley_workbench::rusty::tasks_tab`** (new file
  `crates/marley_workbench/src/rusty/tasks_tab.rs`, declared in `rusty.rs` as `mod tasks_tab;`,
  #644's folder layout, no `mod.rs`):
  - `open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App)`, deferred, and
    `open`, as `agent_tab.rs:31-61` with `decisions::open`'s single-tab rule.
  - `TasksView`: the workspace; `focus_handle` (the tab's root, key context `RustyTasks menu`);
    `list_focus` (the task list, key context `RustyTaskList` plus `not_editing` while `editing`
    is `None`); `lists: Vec<TaskList>`, `chosen: Option<i64>`, `tasks: Vec<Task>`, `selected:
    Option<usize>`; `show_archived: bool`; `add_field: Entity<Editor>`; `editing:
    Option<Editing>` with `Editing { NewList, RenameList(i64), RenameTask(i64) }`, its editor and
    its blur subscription; `queue: VecDeque<TaskWrite>` and the running write's `Task`; the
    running read's `Task` and `read_again: bool`; `stale: bool`; `state` (Off, NotConnected(reason),
    Reading, Ready, ReadFailed(line)); the deployed menu and its position; a `ScrollHandle`; the
    subscriptions (#643's connection state and `list_changed`, `on_focus_in(&focus_handle)`,
    `observe_window_activation`).
  - `show_list(id)`: sets `chosen` and reads; #655's seam.
  - `read`: unless one runs (then `read_again`), `list_task_groups`, then `kept_list`, then
    `list_tasks { group_id, include_archived: show_archived }`, parsed in a lazy background
    future; on its end, stores, keeps the selection (`kept_task`), notifies, and runs once more if
    `read_again`. `shows(window, cx)`: the workspace is the window's shown workspace and the tab is
    its pane's active item, asked each time (PR-607).
  - `on_list_changed`: `read` if `shows`, else `stale = true`. `Item::deactivated`: `stale = true`
    under `connection: service`. `render`: if `stale` and no read runs, clear it and start one
    (AD-609). `on_focus_in` and the window's activation (active, and `shows`): `read`.
  - `write(TaskWrite)`: refused while not connected; pushed to the queue; the next one runs when
    none does, through #643's call; success reads; a refusal shows a workspace toast
    (`NotificationId::unique::<TasksView>()`) with Rusty's first line, then reads. A `Reorder`
    rewrites `tasks` at once before it queues.
  - `render`: `h_flex` of the lists column (`v_flex().w(px(240.))`: "Lists" `Label` and the New
    List `IconButton`; a `ListItem` per list with `toggle_state` on the chosen one, its
    `on_secondary_mouse_down` menu (Rename, Delete List…), the editor row when `NewList` or
    `RenameList`), a vertical divider in `border_variant`, and the list side (`v_flex().flex_1()`:
    the header row with the name in `LabelSize::Large`, `Checkbox::new("show-archived",
    ..).label("Show archived")` and the Refresh `IconButton` with its tooltip; the add field; the
    rows in a `v_flex().id("rusty-tasks").overflow_y_scroll().track_scroll(..)`, each a
    `div().id(("rusty-task", id))` carrying `on_drag(DraggedTask)`, `can_drop` (same list, not
    itself), `drag_over` with `drop_line`, `on_drop` (the new order by `marley_rail::move_to`,
    then `write(Reorder)`), around a `ListItem` with the checkbox in `start_slot`, the title
    `Label` (`strikethrough` and `Color::Muted` when done) or the row editor, "Archived" in
    `end_slot`, `toggle_state` when selected, `on_click` (select and focus the list; two clicks
    rename) and the row menu (Rename, Archive or Restore, Delete for Good…); the state line or
    the notice line with Read Again).
  - `DraggedTask { list, id, position, title }` with `Render` through #602's `drag_preview`.
  - Actions in `rusty.rs`'s `actions!(rusty, …)` (#645's): `OpenTasks` (documented, for the
    palette), and the list's `ToggleTask`, `RenameTask`, `ArchiveTask`, `DeleteTask`,
    `MoveTaskUp`, `MoveTaskDown`, each documented with what it does to the selected task. The
    tab's root handles `menu::SelectNext`, `SelectPrevious`, `SelectFirst`, `SelectLast`,
    `Confirm` (the add field: add; a row editor: commit; the list: rename) and `Cancel` (a row
    editor: cancel; the list: to the add field; the add field: clear, else propagate). The
    deletes ask with `window.prompt(PromptLevel::Warning, …, &["Delete", "Cancel"])` and write
    only on the first answer.
  - `rusty::init` registers `OpenTasks` on each workspace as #645 registers `OpenPage`.
- **`rusty/brain.rs`** (#644's): the fixed row's Tasks `IconButton` (`IconName::ListTodo`,
  tooltip "Tasks") after Graph or Today, dispatching `rusty::OpenTasks` with
  `window.dispatch_action`.
- **`rail_order.rs` and `rail.rs`**: `drag_preview` and `drop_line` become `pub`, and `rail.rs`
  declares `pub mod order` (still under `#[path]`), so `crate::rail::order::{drag_preview,
  drop_line}` reaches them; §14's rule for a module its siblings reach, no
  `#[allow(unreachable_pub)]`. If `rail`'s privacy makes clippy's `redundant_pub_crate` or
  `unreachable_pub` disagree, the Code phase follows §14's table, not an allow.
- **The keymap** (`crates/marley_workbench/keymap.json`): a block with a comment naming the Tasks
  tab (#658) and why each key is Zed's: `"context": "RustyTaskList && not_editing"`, `space`
  `rusty::ToggleTask`, `f2` `rusty::RenameTask`, `delete` and `backspace` `rusty::ArchiveTask`,
  `shift-delete` `rusty::DeleteTask`, `alt-up` `rusty::MoveTaskUp`, `alt-down`
  `rusty::MoveTaskDown`.
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`): a Tasks article beside
  the Brain articles #644 to #647 add: opening, the two columns, the keys, the drag, and when the
  tab reads again (a task an agent adds through its own Rusty shows when you come back to the tab
  or press Refresh).
- **File manifest.**
  - Marley: `crates/marley_rusty/src/tasks.rs` (new) and its `pub mod` line in
    `src/marley_rusty.rs`; `crates/marley_rusty/fixtures/tasks.json` (new only if #655 has not
    made it);
    `crates/marley_rusty/stand_in/rusty-mcp`; `crates/marley_workbench/src/rusty.rs`;
    `crates/marley_workbench/src/rusty/tasks_tab.rs` (new);
    `crates/marley_workbench/src/rusty/brain.rs`; `crates/marley_workbench/src/rail.rs` and
    `rail_order.rs`; `crates/marley_workbench/keymap.json`;
    `crates/marley_workbench/guide/index.html`; `script/e2e/658-rusty-tasks-tab.sh` (new, Test
    phase).
  - Zed: none. The `rusty` namespace is in `crates/zed/src/zed.rs`'s `test_action_namespaces`
    since #645.
- **The ledger rows it extends** (`docs/marley/zed-touchpoints.md`): none.

### Visual check plan
The scenario `script/e2e/658-rusty-tasks-tab.sh`, `compositor sway`. Setup: a scratch repository
with `notes.txt`, opened with `open_path`; in `$E2E_WORK/bin`, #643's stand-in as `rusty-mcp`, named
in `MARLEY_RUSTY_MCP`; `RUSTY_STAND_IN_STATE=$E2E_WORK/rusty`, holding the scenario's own
`tasks.json` (the lists above) and #644's scratch vault; `profile_setting marley.rusty.enabled true`
and `connection "embedded"`, after a check that the harness's copy held Rusty off. Helpers of the
scenario's own: `quiet_add_task LIST TITLE` and `quiet_remove_task ID` (a Python one-liner over
`tasks.json` with a rename into place), `calls_of TOOL` (counts the log's lines), `signal_rusty`
(`kill -USR1` the pid that called `initialize`). Coordinates are measured from the first run's
shots, as 602's and 613's are; menus are walked by position and their shots read (L-500, L-635);
after a prompt the scenario clicks before its next keys (L-581).

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-003 | Click Brain in the header; click Tasks in the fixed row | `658-01-from-rail`: Home chosen; five tasks in order, Call the plumber struck through; no Fix the gate; the log's `list_task_groups` and `list_tasks` |
| REQ-002 | Ctrl-P, `notes.txt`, Enter; Ctrl-Shift-P, "rusty: open tasks", Enter | `658-02-from-palette`: the Tasks tab in front; one Tasks tab in the bar |
| REQ-004 | Click Work | `658-03-list-chosen`: three tasks |
| REQ-005, REQ-006 | Click the add field; "Pack the charger"; Enter; three spaces; Enter | `658-04-added`: last, the field empty; `calls_of create_task` is 1 |
| REQ-007 | Click Write the release notes' checkbox | `658-05-done`: struck through; the log's `toggle_task {id: 21}` |
| REQ-008 | Click Home; click Water the plants; Down | `658-06-moved-selection`: Call the plumber selected |
| REQ-009 | Space | `658-07-space`: Call the plumber not struck through; `toggle_task {id: 12}` |
| REQ-010 | F2; "Call the electrician"; Enter | `658-08-renamed`; `update_task_title` |
| REQ-011 | Delete | `658-09-archived`: gone; `archive_task {id: 12}` |
| REQ-012 | Click Show archived | `658-10-archived-shown`: two muted rows marked Archived; `list_tasks` with `include_archived: true` |
| REQ-013 | Right-click Fix the gate; Down to Restore; Enter | `658-11-restored`; `unarchive_task {id: 14}` |
| REQ-014 | Click Show archived; right-click Return the library books; Delete for Good… | `658-12-delete-prompt`: the title in a code block |
| REQ-015 | Enter; click the list | `658-13-deleted`; `delete_task {id: 13}` |
| REQ-016 | `pointer_down` on Clean the gutters; `pointer_to` over Water the plants | `658-14-dragging`: the preview, the line on Water's top edge |
| REQ-017 | `pointer_up` | `658-15-dropped`: Clean, Water, Fix, Buy; `reorder_tasks {group_id: 1, task_ids: [16, 11, 14, 15]}` |
| REQ-018 | Click Fix the gate; Alt-Up; then Alt-Up twice | `658-16-moved-by-keys`: Fix the gate first; `calls_of reorder_tasks` is 3 (the drop and two moves) |
| REQ-019 | Click New List; "Reading"; Enter | `658-17-new-list`: Reading chosen, the empty text; `create_task_group` |
| REQ-020 | Right-click Reading; Rename; "Books"; Enter | `658-18-list-renamed`; `rename_task_group` |
| REQ-021 | Right-click Books; Delete List… | `658-19-list-delete-prompt` |
| REQ-022 | Enter; click the lists column | `658-20-list-deleted`: Home chosen; `delete_task_group` |
| REQ-023 | `quiet_add_task 1 "Sweep the porch"`; `signal_rusty` | `658-21-live`: listed, no click |
| REQ-024 | Ctrl-P `notes.txt`; `quiet_add_task 1 "Oil the hinges"`; `signal_rusty`; settle; `calls_of list_tasks` unchanged; click the Tasks tab | `658-22-read-on-show`: Oil the hinges listed |
| REQ-025 | Click the Brain view's search field; `quiet_add_task 1 "Wash the car"`; settle; shot; click the task list | `658-23-quiet-write`: not listed; `658-24-read-on-focus`: listed |
| REQ-027 | `quiet_add_task 1 "Feed the cat"`; click Refresh | `658-25-refresh`: listed |
| REQ-028 | `quiet_remove_task 15`; click Buy stamps' checkbox | `658-26-refused`: the toast with Rusty's message; Buy stamps gone |
| REQ-030 | `profile_setting marley.rusty.enabled false`; settle | `658-27-off`: "Rusty is off…"; no call in the log after the edit |
| REQ-026, REQ-029, REQ-031 | The diff; the log against the steps | review |

Not reached by a scenario: the real `rusty-mcp` (its SQLite, its own `list_changed` after a
`rusty-cli refresh`, an agent's stdio instance writing), which would touch the user's store
(R-D8): the quiet write and `SIGUSR1` stand for them. Marley's window coming back to the front
(REQ-026): the headless sway has no other window to leave to. `connection: service`, where no
`list_changed` arrives and each showing reads, and the not-connected state (REQ-029): left to the
review, as #644's and #647's are. The list's keys never reach a PTY: they act only while the task
list holds the focus.

### Risks
- **Names taken from #643 to #647 before they are built.** The client's call, the connection
  state, the `list_changed` signal, the stand-in's state folder and `SIGUSR1`, the fixed row's API
  and the `rusty` actions' registration are named from their queued specs. Promotion re-reads
  what shipped and takes its names.
- **Keys reaching fields** (PR-a-views-key-needs-its-fields-context-too). Space, Delete and
  Backspace bound in a context the add field sat in would eat typing. The list's own context and
  `not_editing` keep them out of every field; the Code phase greps Zed's keymaps for each key from
  the list and from each field, and checks `shift-delete` and `alt-up` against `Workspace` and
  `Pane` bindings, which run before a view's own.
- **Space under vim mode.** Zed's vim keymap binds `space f`, `space /` and more under `!Editor &&
  !Terminal` (`assets/keymaps/vim.json:993-1009`), which the task list matches, so Space there is
  also the start of a sequence. gpui ignores a pending sequence loaded before the binding that
  matched (`gpui/src/keymap.rs:228-239`), and Marley's keymap loads after vim's
  (`zed/src/zed.rs:2362-2380`), so Space toggles at once; a later change to that order would make
  it wait for the pending-key timeout, as the project panel's Space does under vim.
- **Reads on every focus.** Each return of the focus to the tab costs two small calls; a prompt's
  answer leaves no element focused (L-581), so the next click into the tab reads once more. Cheap
  over stdio, and the one way to see another process's write before TICKET-035.
- **A reorder racing a read.** The order is drawn at once and the write queued; a read that
  started before the write and ends after it would draw the old order for a moment. The read
  after the write corrects it; the Code phase drops a read's answer that began before the last
  write was made, as #645 drops a stale page.
- **Archived tasks moved by a reorder.** With them hidden, Rusty puts them after the shown ones
  (`reorder`), so Show Archived afterwards shows them at the end. Rusty's own page behaves the
  same; the guide says so.
- **The lint dance for `rail_order`'s helpers.** `rail` is private and `order` nested in it; §14's
  rule for a module its siblings reach applies. If it fights clippy, the fallback is two small
  copies in the tab with a comment pointing at #602's, decided in the Code phase and recorded.
- **"Tasks" beside Zed's tasks.** Zed's `task` crate runs commands (`task: spawn`), and its task
  terminals are tabs too. The `rusty:` namespace, `IconName::ListTodo` and the tooltip "Rusty's
  to-do lists" keep them apart; the guide names the difference.
- **A task added to a list another process deleted.** Rusty's `create_task` checks nothing, and
  its store never turns SQLite's foreign keys on (no `PRAGMA foreign_keys` in `rusty-core`; the
  `REFERENCES task_headers(id) ON DELETE CASCADE` in `engine/db.rs:124-130` is not enforced), so
  the task lands in no list and no screen shows it. Marley cannot know the list is gone before its
  next read; the read after the write chooses the first list. A possible Rusty bug, seen in the
  source only, reported for Rusty, not fixed here.
- **Missing ids answer success.** Rusty's archive, restore, rename and delete on a task another
  process removed succeed and change nothing; only the read after shows it. Shown as Rusty
  answers; a Rusty-side check would be a request, not this ticket's.
- **The golden scenarios** keep Rusty off in every run's copy (#643's harness line), so nothing of
  this tab appears in them.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: a Tasks section with the Brain view's (#644 to #647): opening it, the
  two columns, the keys, the drag, the delete prompts, archived tasks and Show Archived, and when
  the tab reads again (an agent's own Rusty, `rusty-cli refresh`, Refresh).
- `docs/marley/walkthrough.md`: a stop for the Tasks tab, after the Brain view's, which needs
  Rusty installed and `marley.rusty.enabled` on.
- `crates/marley_workbench/guide/index.html`: the Tasks article, in the Code phase (above).
- Architecture (§21): `docs/marley_architecture/marley_workbench.md` (the Tasks tab in `rusty`,
  its reads and its write queue); `marley_rusty`'s architecture doc, which #643 starts (the
  `tasks` module and the stand-in's task tools); `docs/marley/rusty-in-marley.md` (R7's Tasks half
  shipped; the Rusty-side requests this ticket found: a tool to move a task between lists, a tool
  to order the lists); `CHANGELOG.md` under Added.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20 and the three templates.
- [x] Read the plan (`docs/marley/rusty-in-marley.md`, R-D0 to R-D10, the slices, Rusty's
      triage), the brief's three parts, and the queued #643 to #647 for their names.
- [x] Recall: the ledgers (AD-452, AD-453, AD-457, AD-602, AD-609, F-438-a, F-515, F-600, F-602,
      F-607, L-457, L-465, L-482, L-500, L-504, L-507, L-515, L-531, L-534, L-572, L-581, L-600,
      L-602, L-607, L-633, L-635, PR-592, PR-607, PR-a-views-key, PR-a-button-in-a-card,
      PR-a-state-another-view-lists), the completed pipelines 453, 457, 565, 581, 600, 602, 609
      and 633, the queued 643 to 647, and a read-only brain search.
- [x] Discovery with file:line: Rusty's twelve tools, their answers and refusals, the store, the
      watcher and the sentinel, TICKET-035, the `/tasks` skill, `TasksPage.qml`; Marley's
      `rusty.rs`, `decisions.rs`, `agent_tab.rs`, `rail_order.rs`, `marley_rail::move_to`, the
      keymap, the guide page; Zed's `Item`, the pane's drag and `deactivated`, gpui's drag and
      focus hooks, `ui`'s checkbox, list item, label and menu, the git, project and collab panels,
      Zed's Linux keymap; Ely's `SortableList`, `Reorder`, `Flip` and `KanbanBoard`; the e2e
      runner; the store's counts.
- [x] Prior-art sweep, three legs: Zed's, Orca's and Warp's maps; Rusty's code and page, Rusty's
      TICKET-035, the MCP specification; Zed's `ui`, gpui, `workspace`, `git_ui`,
      `project_panel`, `collab_ui`, the keymap, `Cargo.lock`, Marley's rail and tabs, and Ely.
- [x] The brief's port question answered: Zed's `ui` has no reorderable list, but Zed's tab bar
      and Marley's rail (#602) reorder on gpui's drag API, so the tab takes #602's drag and no Ely
      code (D7, D13).
- [x] How the tab refreshes, locked with its reasons (D9): `list_changed` while shown, stale while
      hidden, the focus and the window, Refresh; TICKET-035 named in Out.
- [x] #655's link and the Decisions half of R7 (#659) left out, with the seam #655 calls; both
      drafts read (`655-knowledge-panel-project-view`, `659-rusty-decisions-tab`) so the button
      on #655's Tasks header, the stand-in's `tasks.json` and the fixed row's order agree.
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D13, thirty-one EARS rows, the
      phase plan.
- [x] Design: approach, file manifest by crate, no touchpoint rows, the visual check plan, risks,
      the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Reconciliation, 2026-10-03
- A `rusty:` action run while Rusty is off or not connected shows a toast saying so and where to
  turn it on, and opens nothing (rusty-in-marley.md R-D0, settled across #643 to #659).
- Every scenario names its stand-in in `MARLEY_RUSTY_MCP`, never first on the PATH (#643).
- Rusty's TICKET-045 (confirmed): foreign keys are off and a write on a missing id returns
  success. Until it lands the tab treats a write's success as "maybe" and re-lists the group after
  every write, so a task written into a list another process deleted never shows as saved.
