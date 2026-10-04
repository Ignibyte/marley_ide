# The rail's Brain view: Rusty's vault behind a switch in the header — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-644-brain-view-in-the-rail.md
- **Pipeline spec:** 644-brain-view-in-the-rail.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, on Rusty and Marley: "maybe rusty becomes Marley. We would take
  our rusty custom QML app and build it inside of marley", then "Plan it now". Picking how
  Rusty's screens are reached, from the choices in `docs/marley/rusty-in-marley.md` R-D9:
  "Switch in the rail header". On the components: "lets make sure we use the gpui components we
  found here" (R-D10). For this batch: "lets make a plan to begin the work and spec out the
  tickets", confirmed as "Queue all five" (#643 to #647).
- **Classification / tier:** feature, large. The plan sizes R4 M; with brain search and the key
  it is the batch's largest. One pure module added to #643's `marley_rusty`, one new view file
  and a cache in `marley_workbench::rusty`, the rail's header and render changed, one action and
  one binding. No Zed touchpoint, no new dependency (Ely is copied, not linked; `marley_rusty`
  is #643's). If the Code phase runs long, the natural cut is the write side (REQ-012 to
  REQ-019) as a slice of its own; the read side ships whole without it.
- **Recall (§18.3):**
  - AD-claude-453-the-rails-keys-are-zeds-list-actions-001: the rail binds no list key of its
    own; `MarleyRail menu` gets left and right from Zed. The Brain view takes the same actions
    in `MarleyBrain menu`.
  - AD-claude-457-the-rails-filter-is-zeds-sidebar-filter-001 and
    L-claude-457-a-single-line-editor-hands-zeds-list-keys-to-its-container-001: Escape clears,
    then leaves the field, then propagates; Up and Down in a single-line editor arrive as
    `menu::SelectPrevious` and `SelectNext` on the container, and Enter as `menu::Confirm`.
    `secondary-f` is bound in `MarleyRail && !Picker`, which the Brain view sits inside.
  - AD-claude-452-the-rail-starts-zeds-own-rename-and-close-001: terminals rename through Zed's
    tab rename, and an editor in the row was rejected there. Vault rows have no Zed rename to
    start, so the project panel's editor in the row is the Zed behavior to follow (D10).
  - AD-claude-534-the-harness-is-followed-by-polling-in-a-section-outside-the-rails-model-001: a
    rail section outside `marley_rail`'s model, keys and filter already exists; D2 goes further,
    a whole view outside it.
  - AD-claude-602-the-rail-owns-its-order-001 and
    L-claude-602-nested-drop-targets-need-a-drag-type-each-001: a drop goes to the deepest
    element with an `on_drop` of its type, which takes the drag before `can_drop` says no; hover
    reads false for everything while a drag runs; a successful drop stops propagation, so the
    drop handler ends what the drag began. The vault's drag gets its own type, and every row is
    a target (a page row meaning its folder), so no drop falls through to the list.
  - F-claude-600-a-hand-deployed-menu-lost-the-keyboard-to-the-rails-own-focus-001 and
    L-claude-600-a-hand-deployed-menu-stops-and-prevents-its-mouse-down-001: the rail's root
    tracks focus, so a menu deployed from a right mouse-down stops and prevents it.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001 and
    L-claude-635-a-scenarios-fixed-menu-steps-can-pass-on-the-wrong-entry-001: the scenario
    walks menus by position and reads the shot to see the entry it reached.
  - L-claude-581-a-prompt-opened-from-a-context-menu-keeps-the-keys-001: Zed's prompt opened from
    a menu entry takes Enter and Escape; after the answer no element has focus, so the scenario
    clicks before its next keys.
  - L-claude-604-the-rails-enter-opens-through-open-row-not-a-rows-click-001: Enter and a click
    each open through one function; the Brain view's Confirm and its click share one too.
  - F-claude-606-closed-headers-would-have-shared-their-element-ids-001: element ids from a
    value that is unique per row; vault paths are.
  - F-claude-438-a-a-sidebar-flag-read-a-value-only-render-wrote-001: the header's state (Rusty
    connected, the dot) comes from what `refresh` and the connection's observer store, not from
    what `render` computes.
  - L-claude-572-an-observed-global-written-at-every-wakeup-redraws-the-rail-001: the vault cache
    is written only when a read differs; its in-flight flags live where nothing observes them.
  - L-claude-534-zeds-mcp-client-sees-no-server-exit-001: Zed's MCP client does not see a stdio
    server exit and drops an error's code for its message. #643 owns the connection state;
    this ticket shows Rusty's messages, which survive.
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001 and
    L-claude-603-a-scenario-fakes-a-program-for-marley-through-setups-path-001: a PATH exported
    in `setup` reaches Marley's own spawns, so the stand-in reaches the embedded connection.
  - L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001: the scenario turns
    Rusty on by editing the run's settings from outside before anything in Marley writes them.
  - Completed pipelines read: 438 (the rail and the `Sidebar` trait), 452, 453, 457 (the
    filter), 534 (the Harness section), 600 (the empty-space menu, `GroupNamePrompt`), 602 and
    613 (drag types and drops in the rail), 633 (the stand-in `rusty-mcp` scenario). Queued 642
    for the off-feature toast and the e2e harness's settings copy.
  - Brain (`rusty-cli brain search`, read only):
    `decisions/marley-draws-rustys-knowledge-workspace-rusty-keeps-its-data-marley-d11-amended`,
    `decisions/rusty-files-marleys-rq1-rq5-as-ticket-035-to-039-and-m10-the-qt-app-frozen`
    (TICKET-037 is favourites, D9), `decisions/the-rails-keys-are-zeds-list-actions`. Promotion
    asks the brain on D2 and D8; Complete records them.
- **Discovery:**
  - The rail, `crates/marley_workbench/src/rail.rs` (8276 lines): `Rail` `:120-228` (no view
    field yet); `new` `:739-855` (observes the settings store at `:790-796`); `render_header`
    `:3928-3970` (window controls, the PROJECTS label `:3955-3959`, the empty space's right-click
    `:3960-3964`, Add Project `:3966`); `render_blocks` `:3972`, `render_rows` `:4197`,
    `render_filter` `:4229`, `render_add_project` `:4275`; `focus_filter` `:2730`, `cancel`
    `:2741`, `confirm` `:2766`, `open_row` `:2772`; `show_toast` `:3321`; `empty_space_menu`
    `:3766`; `Focusable` `:8068-8072`; `impl Sidebar` `:8075-8213`; `impl Render` `:8215-8270`
    (key context `MarleyRail menu`, `track_focus`, the list actions, then header, filter, inbox,
    rows). Its drags: `rail_order.rs` `header_block` `:309`, `end_drag` `:467`, called from the
    root's `on_mouse_up` (`rail.rs:8236-8243`).
  - The row model, `crates/marley_rail/src/marley_rail.rs` (2776 lines, gpui-free, depends on
    `marley_agent` alone): `RailSnapshot` `:477`, `Selection` `:499`, `Row` `:666`, `selection`
    `:710`, `step` `:1227`, `cycle_project` `:1262`, `cycle_row` `:1282`, `rail_rows` `:1365`,
    `switcher_rows` `:1486`, `has_attention` `:1555`. Every function walks projects.
  - The `Sidebar` trait, `crates/workspace/src/multi_workspace.rs:121-160`; `sidebar()` `:414`;
    `multi_workspace_enabled` `:439-441` (`disable_ai` and `agent.enabled`; `toggle_sidebar`
    returns early on it, `:443-446`); the root carries the workspace's key context and actions
    (`:2087-2091`).
  - Finding a window's rail: `marley_workbench.rs:858-876` (`sidebar().to_any()`,
    `downcast::<Rail>()`). The `marley` actions: `actions!` `:117`; `init` `:741`
    (`rusty::init` `:770`).
  - The keymap, `crates/marley_workbench/keymap.json`: the Workspace block `:5-16`
    (`secondary-alt-n`, `ctrl-shift-t`); the rail's block at its end (`secondary-f`,
    `ctrl-tab`). Zed's Linux keymap: `ctrl-alt-b` is `workspace::ToggleRightDock`
    (`assets/keymaps/default-linux.json:678`), `ctrl-alt-j` and `ctrl-alt-;` the sidebar
    (`:681-682`); bound `ctrl-alt-` letters are a b c d e f g h i j k l p r s y z, so `v` is
    free. Hyprland binds no Ctrl-Alt chord here.
  - Zed's `ui` for the tree: `ListItem` (`crates/ui/src/components/list/list_item.rs:32`,
    `indent_level` `:203`, `indent_step_size` `:208`, `toggle` `:213`, `on_secondary_mouse_down`
    `:185`, `Toggleable` `:285`); `Disclosure` (`disclosure.rs:9`); `indent_guides`
    (`indent_guides.rs:53`, `IndentGuideColors::panel` `:22`, `with_compute_indents_fn` `:84`);
    `ContextMenu` (`context_menu.rs:213`, `build` `:284`); `IconButton` (`icon_button.rs:33`,
    `indicator` `:117`, `toggle_state` `:148`). Icons that exist: `ListTree` `:188`, `BookCopy`
    `:58`, `Notepad` `:201`, `Folder` `:148`, `FolderOpen` `:151`, `FileTextOutlined` `:141`,
    `Plus` `:213` (`crates/icons/src/icons.rs`); none named Brain, Calendar or Graph.
  - Zed's project panel, read for behavior (`crates/project_panel/src/project_panel.rs`):
    `uniform_list` `:7487`, indent guides `:7512`, rows `ListItem::indent_level` `:6417-6419`;
    the click's preview rule `:6397-6402`; the row editor `:805-830`, `add_entry` `:2392`,
    `confirm_edit` `:2082`; the removal prompt `:2768`, `:2881`; drag `:6088`, drop `:6285`,
    `drag_onto` `:5058`; the context menu `:1118-1275`, on empty space `:7832`. Bound to
    `Project`, `Worktree` and `ProjectEntryId` (`:35-37`, `:95`, `:4531-4541`).
  - Preview tabs: `PreviewTabsSettings` (`crates/workspace/src/item.rs:67-75`,
    `enable_preview_from_project_panel`); `Pane::open_item` `:1071` and its preview logic
    `:1106-1155`; a double-click on a tab keeps it (`pane.rs:4523`). A file outside the project
    opens as a preview through `Workspace::project_path_for_path(.., visible: false)` and
    `open_path_preview` (`workspace.rs:4359`, `:5139`); `open_abs_path` always keeps (`:5078`).
    The Page tab's own preview support is #645's.
  - #645's draft (queued 2026-10-03, `645-brain-page-tab.spec.md`) builds on this ticket: its
    scenario opens a page by clicking it in the Brain view, and its scope points the tree's
    clicks at `rusty::page::open_later(workspace, slug, preview, window, cx)` (with
    `rusty::OpenPage { slug, preview }` for actions), taking the vault's folder from
    `setting_get brain_vault_path` or Rusty's default. #647's draft adds the fixed row's Graph
    entry (`rusty::OpenGraph`). #646's draft has its own brain search in the Knowledge panel,
    sent 250 ms after typing stops.
  - Rusty's tools (`/srv/stacks/rusty-v3/crates/rusty-mcp/src/main.rs`, 85 tools): `brain_tree`
    `:1217` (no parameters; one nested `VaultNode`: `name`, `path`, `kind` folder, page or file,
    `pages`, `children`, `rusty-core/src/brain/vault.rs:51-63`; folders first, sorted by
    lowercase name, `:362-369`; dot entries skipped, `:324`; a page's path is its slug without
    `.md`); `brain_new_page` `:1264` (`folder`, optional `name`; Untitled, Untitled 1 when no
    name; an existing name gets a number; "No folder x"; `brain/mod.rs:2441-2474`);
    `brain_new_folder` `:1278` (`path`; "Already exists: x"; parents made); `brain_rename`
    `:1296` (`from`, `to`; pages and folders; `to` ending in `/` means into that folder, so `/`
    alone is the root; rewrites links; "Already exists", "Not found", "Cannot move a folder into
    itself"; `mod.rs:2505-2637`); `brain_delete_page` `:1630` (`slug`; the file moves to
    `archive/`); `brain_delete_folder` `:1286` (`path`; recursive, into `archive/<name>_<secs>`;
    refuses the vault's root); `brain_search` `:831` (`query`, `limit` default 10, `page_type`,
    `case_sensitive`, `regex`; hits `slug`, `page_type`, `title`, `snippet`, `rank`; embeds the
    query when a provider is set, `:845-851`); `brain_daily_note` `:1006` (optional `date`,
    local today by default; makes `daily/<date>` when missing and announces it only then).
    Errors arrive as JSON-RPC errors carrying Rusty's message (`json_result`, `:36-41`).
    `list_changed` goes to every peer on each `DataChanged` (`:2062-2082`), which writes and the
    vault's file watcher (about 600 ms debounce) raise. `setting_get` `:1843` answers null for an
    unset `brain_vault_path`; the default `~/.rusty/brain` is computed in
    `rusty-core/src/core.rs:73-81` (D1).
  - Favourites: no tool. Bookmarks are the Qt app's window state, `~/.config/rusty/workspace.json`
    (`rusty-app/src/terminals.rs:121-129`, a double-encoded `bookmarks` string, `qml/Main.qml:72`,
    `:114`, `:128`); its Favorites section shows the file and folder bookmarks
    (`qml/Explorer.qml:27`, `:196-226`). Rusty's TICKET-037 (`docs/planning/tickets/open/
    TICKET-037-bookmarks-in-the-vault.md`) is open with no pipeline yet: a git-tracked vault file
    with list, add, remove, reorder and set-favourite tools.
  - The vault on this box: 807 pages, 26 folders, 28 other files; a whole `brain_tree` is on the
    order of 130 KB of JSON.
  - Rusty's Qt Explorer (`qml/Explorer.qml`, 536 lines): New note with no name (`:133`), New
    folder through a dialog (`:134-138`), Rename in the row with `/` turned into `-`
    (`:150-157`), Move to… (`:158`), Delete (`:159-163`), the row menu (`:385-392`); file rows
    show and do not open (`:250`).
  - Ely (`repos/ely`, HEAD `2f8b2f6`, `LICENSE-MIT:3` "Copyright (c) 2026 Ely GPUI Component
    contributors"): `src/lists/tree/model.rs` (403 lines; `holds` `:95-101`, `rows` `:134-175`,
    `Move` and `step` `:178-208`, `find` `:237`, `place_at` `:257-264`, `can_drop` `:266-269`;
    gpui used only for `SharedString`); `src/lists/files.rs` (317 lines, `FileTree` `:161-245`,
    wraps `Tree`; builds folders from path strings `:88-156`); `src/lists/tree/row.rs` (306
    lines, rendering and guides); `src/documents/knowledge.rs` (`Favorites` `:121-256`, drag
    reorder through `motion::Reorder`); `src/forms/inline.rs` (`InlineEdit` `:90-203`).
  - The e2e harness: `script/e2e.sh:625-642` copies the user's settings and writes
    `marley.rusty_tools: false`; #643 moves that to `marley.rusty`. Scenario helpers: 633's
    `set_setting` (`script/e2e/633-rusty-tools-for-zeds-agents.sh`), 613's pointer drag
    (`script/e2e/613-move-a-terminal-to-another-project.sh:40-56`).
- **Decisions:** D1 to D13 in the spec. In short: in Chad's order, before #645, every open
  going through `open_page`, which opens the page's file in Zed's editor until #645 repoints it
  at its Page tab; a view of its own beside `marley_rail`'s model, with a pure row model in
  `marley_rusty::vault`; Projects and Brain buttons only while Rusty is on and connected, the
  end button and an attention dot following the view; `secondary-alt-v` flips, with a toast
  when it cannot; the fixed row ships Today alone, others with their tabs; preview on one
  click, kept on two, as the project panel opens files; the whole vault in a `uniform_list`
  with Zed's `ui` rows and list keys; search on Enter only, because of the embedding provider;
  favourites wait for TICKET-037; every write a Rusty tool, names edited in the row, delete
  asked first; `list_changed` re-reads; the stand-in only; Ely's tree model ported, Zed's `ui`
  draws.

### Design
- **Approach.**
  - **`marley_rusty::vault`** (new file `crates/marley_rusty/src/vault.rs`, declared in the
    crate root #643 makes; gpui-free; Ely's MIT notice at the top naming
    `src/lists/tree/model.rs` at `2f8b2f6`): `VaultNode { name, path, kind, pages, children }`
    deserialized from `brain_tree`, `NodeKind { Folder, Page, File }` with any other kind read as
    `File`; `SearchHit { slug, title, page_type }` from `brain_search`; `VaultRow { path, name,
    kind, depth, parent, open, pages }`; `rows(root, open) -> Vec<VaultRow>` (Ely's `rows`:
    files skipped, the root node itself not listed); `Key { Next, Previous, First, Last, Child,
    Parent }` and `step(rows, at, key) -> Option<Move>` with `Move { To(usize), Open(String),
    Close(String) }` (Ely's `step`); `folder_of(path)`; `rename_target(path, typed) ->
    Option<String>` (trimmed, `/` to `-`, `None` when empty or unchanged); `move_target(root,
    dragged, onto) -> Option<String>` where `onto` is a folder, a page (its folder) or the root,
    giving `"<folder>/"` or `"/"`, and `None` onto itself, under itself (Ely's `holds` and
    `can_drop`) or into its own folder; `reopen(open, from, to)` re-keying open folders under a
    moved folder.
  - **The stand-in** (#643's, wherever #643 keeps it with `marley_rusty`'s fixtures): the eight
    tools over its scratch vault, walking the folder as Rusty does (folders first, lowercase
    order, dot entries skipped, `.md` dropped, recursive page counts); `brain_daily_note` makes
    `daily/<local date>.md`; `brain_search` matches the words in titles and bodies, case
    ignored; `brain_new_page`, `brain_new_folder`, `brain_rename` (the `/` ending included) and
    both deletes (into `archive/`) with Rusty's messages for an existing name, a missing path and
    the root. Each `tools/call` appends its name and arguments to a log the scenario names; each
    write and a `SIGUSR1` send `notifications/resources/list_changed`; it writes its pid to a
    file and exits at start while a stop file exists. No link rewriting: that is Rusty's, and no
    shot rests on it.
  - **The cache** (`crates/marley_workbench/src/rusty.rs`): a `VaultTree` global holding the last
    read (`Option<Result<VaultNode, String>>`), observed by the Brain views and written only when
    a read differs; `reread_vault(cx)` starts a `brain_tree` call through #643's client unless
    one is under way, in which case it queues one more; the in-flight and queued flags sit in a
    global nothing observes. Hooked to #643's `list_changed` while any Brain view exists, and
    called after each write and by Refresh. The parse runs in the background task.
  - **`BrainView`** (new file `crates/marley_workbench/src/rusty/brain.rs`, declared in `rusty.rs` as
    `mod brain;`, the folder layout #645 and #647 use (reconciled 2026-10-03; no `mod.rs`)): a focus handle; the
    search `Editor::single_line` ("Search the brain…"); the query last searched and its hits; the
    open folders; the selected path; the editing state (`NewPage { folder }`, `NewFolder {
    parent }` or `Rename { path }`, with its editor and blur subscription); a
    `UniformListScrollHandle`; the deployed menu and its position; the visible rows recomputed
    when the cache or the open set changes. `render`: `key_context("MarleyBrain menu")`,
    `track_focus`, the list actions, `menu::Confirm` and `menu::Cancel`; the fixed row (Today);
    the search field; then the hits, the tree (`uniform_list` of `ListItem`s with
    `indent_level`, a `Disclosure` on folders, the page count at a folder's end,
    `indent_guides(px(20.), IndentGuideColors::panel(cx))`), or the reading or error line. A
    page's click reads `click_count`: one calls `open_page(slug, preview)` with `preview` from
    `PreviewTabsSettings`, two with `preview: false`; a folder's click folds. `open_page` (in
    the same file) joins the vault's folder and `<slug>.md`, finds or makes the project path with
    `project_path_for_path(.., visible: false)` and calls `open_path_preview`; the folder is read
    with `setting_get brain_vault_path` when Rusty connects and kept beside the cache, Rusty's
    default `~/.rusty/brain` standing in for a null answer. #645 replaces its body with its
    opener.
    Menus: `ContextMenu::build` deployed at the pointer from `on_secondary_mouse_down`, with
    `stop_propagation` and `prevent_default` (L-600). The drag: a `DraggedVaultEntry { path,
    name, kind }` with a small label preview; every row and the list's empty space take
    `drag_over` (the drop-target background) and `on_drop` only where `move_target` gives a
    target, and the drop handler ends the drag (L-602). Delete: `window.prompt(PromptLevel::
    Warning, …, &["Delete", "Cancel"])`, the tool only on the first answer. Calls go through
    #643's client; a refusal becomes a workspace toast with `NotificationId::unique::<
    BrainView>()` and Rusty's message; a success calls `reread_vault` and, for a new page,
    selects it, unfolds its folder and opens it kept.
  - **The rail** (`rail.rs`): `view: RailView { Projects, Brain }`, the chosen one; `brain:
    Option<Entity<BrainView>>`, made when Brain first shows and dropped when Rusty turns off; an
    observer of #643's connection state that refreshes the rail; `brain_shown(cx)` (Brain chosen
    and Rusty connected). `render_header` draws the two `IconButton`s while Rusty is on and
    connected (Projects' `indicator` while Brain shows and `has_attention` or the inbox holds an
    entry), PROJECTS otherwise; the end is `render_add_project` or a New Page `IconButton` that
    hands off to the view. `render` draws the Brain view in place of the filter, the inbox and
    the rows while it shows. `Focusable` returns the view's handle while it shows, so Zed's focus
    sidebar action lands in the tree. `focus_filter` focuses the view's search field while Brain
    shows. `pub(crate) fn toggle_brain_view` flips, opens the rail when closed and focuses the
    shown view, deferred outside the `MultiWorkspace`'s update as the rail's closes are
    (F-claude-442-a).
  - **The action** (`marley_workbench.rs`): `ToggleBrainView` in `actions!(marley, …)`, its doc
    "Flips the rail between its Projects view and its Brain view, Rusty's vault, while Rusty is
    on."; registered on every workspace in `init`: it finds the window's rail as
    `register_sidebar` does and calls `toggle_brain_view`; with no rail (the Zed layout), the AI
    gate closed (`multi_workspace_enabled`), Rusty off or not connected, it shows the toast D4
    words instead.
  - **The keymap** (`crates/marley_workbench/keymap.json`): `"secondary-alt-v":
    "marley::ToggleBrainView"` in the Workspace block, with a comment naming the Brain view and
    why the chord is free.
  - **The in-app guide page** (`crates/marley_workbench/guide/index.html`): the rail's header line
    (`:453` names PROJECTS) and a Brain view article; changed in the Code phase, since the receipt
    binds it.
- **File manifest.**
  - Marley: `crates/marley_rusty/src/vault.rs` (new) and its `mod` line in the crate root;
    #643's stand-in; `crates/marley_workbench/src/rusty.rs`; `crates/marley_workbench/src/
    rusty/brain.rs` (new); `crates/marley_workbench/src/rail.rs`;
    `crates/marley_workbench/src/marley_workbench.rs`; `crates/marley_workbench/keymap.json`;
    `crates/marley_workbench/guide/index.html`; `crates/marley_workbench/Cargo.toml` only if #643
    left `marley_rusty` out of it; `script/e2e/644-brain-view-in-the-rail.sh` (new, Test phase).
  - Zed: none. The `marley` action namespace is already in `crates/zed/src/zed.rs`'s
    `test_action_namespaces`; the `rusty` namespace is #645's.
- **The ledger rows it extends** (`docs/marley/zed-touchpoints.md`): none.

### Visual check plan
The scenario `script/e2e/644-brain-view-in-the-rail.sh`, `compositor sway`. Setup: a scratch
repository opened with `open_path`; `terminal_env HOME` with a `.bashrc` that sets `PS1='$ '`;
in `$E2E_WORK/bin`, #643's stand-in as `rusty-mcp` named in `MARLEY_RUSTY_MCP` (#643's rule; never first on the PATH), its scratch vault at
`$E2E_WORK/vault` with the nine made-up pages the spec lists, its call log at
`$E2E_WORK/rusty-calls`, its pid file and stop file beside them, and `setting_get
brain_vault_path` answered with `$E2E_WORK/vault`; `set_setting marley.rusty.connection
'"embedded"'`, and a check that the harness's copy holds `marley.rusty.enabled` false. Every
shot's coordinates are measured on the first run, as 613's are; menus are walked by position and
their shots read (L-635). Before #645 a page's tab is its `.md` file in Zed's editor; a run after
#645 sees Page tabs in the same places, and the Test phase notes which it saw.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | Trust the project; Rusty off | `644-01-header-off`: PROJECTS and Add Project |
| REQ-002 | `secondary-alt-v` | `644-02-off-toast`: "Rusty is off…"; the rows unchanged |
| REQ-003 | `set_setting marley.rusty.enabled true`; settle until connected | `644-03-switch`: Projects pressed, Brain beside it |
| REQ-004 | Click Brain | `644-04-brain`: Today, the field, `daily` 1, `decisions` 2, `notes` 2, `projects` 2, `home`; New Page at the end |
| REQ-005 | Click `projects`, then `marley` | `644-05-unfolded`: `plan` and `shell` two steps in, with guides |
| REQ-006 | Unfold `notes`; one click on `alpha` | `644-06-preview`: an italic `alpha` tab; the row selected |
| REQ-007 | Double-click `alpha` | `644-07-kept`: the tab upright |
| REQ-008 | Click Today | `644-08-today`: a kept tab for today's note; `daily` counts 2 |
| REQ-009, REQ-010 | Click the field; type "beta"; Enter | `644-09-search`: `beta` listed; the log holds one `brain_search` |
| REQ-011 | Escape, Escape; Home, Down, Down (`notes`, open since REQ-006), Left, Right, Down, Enter | `644-10-keys`: `notes` unfolded again, `alpha` selected and its tab in front |
| REQ-012 | Right-click `notes` | `644-11-menu`: New Page, New Folder, Rename, Delete |
| REQ-013 | New Page; "gamma"; Enter | `644-12-new-page`: `gamma` selected, its tab kept; the log's `brain_new_page` |
| REQ-014 | Right-click `notes`; New Folder; "drafts"; Enter | `644-13-new-folder`: `drafts` under `notes`; the log's `brain_new_folder` |
| REQ-015 | Right-click `gamma`; Rename; "delta"; Enter | `644-14-renamed`: `delta`; the log's `brain_rename` |
| REQ-016 | Right-click `delta`; Rename; "alpha"; Enter | `644-15-refused`: "Already exists: notes/alpha"; `delta` stays |
| REQ-017 | Drag `delta` onto `decisions` | `644-16-moved`: `delta` under `decisions`; the log's `to: "decisions/"` |
| REQ-018 | Right-click `marley` (under `projects`); Delete | `644-17-delete-prompt`: the name, "2 pages", `archive/` |
| REQ-019 | Enter on Delete; click the tree | `644-18-deleted`: `marley` gone, `archive` listed; the log's `brain_delete_folder` |
| REQ-020 | Write `notes/epsilon.md`; `kill -USR1` the stand-in's pid | `644-19-live`: `epsilon` under `notes` |
| REQ-021 | Click the terminal; `sleep 3; printf '\a'`; Enter; click the tree before the bell | `644-20-attention`: a dot on Projects |
| REQ-022 | `secondary-alt-v`; then again (not shot) | `644-21-flipped`: the Projects rows, Projects pressed |
| REQ-023 | Make the stop file; kill the stand-in; settle | `644-22-down`: PROJECTS, the rows |
| REQ-024 | The log against the steps; the diff | review |

Not reached by a scenario: the real `rusty-mcp` (link rewriting on rename, the vault's git
commits, the embedding of a query), which is Rusty's and would touch the user's brain (R-D8);
`connection: service`, where no `list_changed` arrives (R-D2), left to the review; the toasts
for the Zed layout and for the AI gate, left to the review; the flip back to Brain after a
reconnect, left to the review. The Brain view's own keys never reach the PTY: they act only
while the view holds focus, and the flip key carries Ctrl and Alt (PR-claude-unmodified-
terminal-chords-yield-to-the-pty-001).

### Risks
- **Size.** The batch's largest. The write side (D10) is the cut if the Code phase needs one;
  the read side (switch, tree, opens, Today, search, live, dot, key) ships whole without it.
- **Names taken from #643.** This spec names #643's connection state, client call,
  `list_changed` signal and stand-in before #643 is built. Promotion re-reads what it shipped
  and takes its names.
- **The editor stand-in for the Page tab.** Until #645, a page opens as a Markdown file: the
  vault becomes an invisible worktree of the project, as any file opened from outside the
  project does, and an edit saved there is a source edit Rusty's watcher indexes and commits
  (R-D4). The vault's folder comes from `brain_vault_path` or Rusty's default, a copy of Rusty's
  rule #645 also carries; with `connection: service` the service's HOME is assumed to be the
  user's.
- **Focus inside the rail.** The Brain view's focus sits inside the rail's root, which tracks the
  rail's own handle and drops the rail's cursor on focus out (`rail.rs:774-777`); the rail's list
  actions are also in the path, below the view's, which handles them first. The menu rules of
  L-600 and the prompt's lost focus (L-581) apply.
- **Two drag types in one element tree.** The rail's root ends its own drags on mouse-up
  (`end_drag`); a vault drag is a different type, and its drop handler ends it (L-602). The
  review checks that `end_drag` ignores a drag that is not the rail's.
- **`uniform_list` in a flex column** needs a bounded height (`flex_1` and `min_h_0`), or it
  draws nothing; Ely's FileTree says the same ("Give it a height").
- **Re-reading the whole tree on each change.** Rusty's watcher announces every file change in
  the vault (about 600 ms apart at most), and each re-read moves about 130 KB over stdio; parsed
  off the main thread and dropped when equal. TICKET-035's cursor would make it incremental.
- **A possible Rusty bug, seen in the source only:** `delete_folder` leaves `archive/<name>_<secs>/
  *.md` as pages, which Rusty's indexer would take in again as `archive/…` slugs; single deleted
  pages escape it (their file name ends `.md_<secs>`). Marley shows what `brain_tree` serves.
  Reported for Rusty, not fixed here.
- **Soft delete.** Delete moves into `archive/` and leaves links to the page; the prompt says
  so, so nobody expects links to be cleaned up.
- **Two dates for today in Rusty.** `brain_daily_note` uses local time and `brain_capture` UTC;
  Today calls the first, so it matches the user's clock.
- **The AI gate.** With `disable_ai` on, or the agent turned off, Zed hides the rail and the
  Brain view with it (Out); the key says so in a toast.
- **The golden scenarios** start with Rusty off in every run's copy (#643's harness line), so the
  header still reads PROJECTS and no golden coordinate moves.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: `:156-158` (the header reads PROJECTS and carries Add Project), `:307`
  (the right-click "beside PROJECTS"), and a Brain view section after the rail's: the switch,
  the key, Today, search on Enter, the tree, the menus and the drag, and that favourites come
  with Rusty's bookmarks.
- `docs/marley/walkthrough.md`: `:264` (the rail's header) and a new stop for the Brain view,
  which needs Rusty installed and `marley.rusty.enabled` on.
- `crates/marley_workbench/guide/index.html`: `:453` and the Brain view article, in the Code
  phase (above).
- Architecture (§21): `docs/marley_architecture/marley_workbench.md` (the rail at `:90`, a
  Brain view section, the cache in `rusty`); `marley_rusty`'s architecture doc, which #643
  starts (the `vault` module and Ely's port); `docs/marley/rusty-in-marley.md` (R4's row shipped,
  favourites waiting on TICKET-037, the batch order D1 set); `CHANGELOG.md` under Added.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20 and the templates.
- [x] Read the plan (`docs/marley/rusty-in-marley.md`, R-D0 to R-D10, the slices, Rusty's
      triage) and the rail's design (`docs/marley/workbench-shell.md` D0 to D8 and its deferred
      AI gate; `docs/planning/design-notes/simple-rail-shelf.md`, the gpui-era rail, read as
      history; `docs/planning/intake/rail-internals.md`).
- [x] Recall: the ledgers (AD-452, AD-453, AD-457, AD-534, AD-602, F-438-a, F-600, F-606, L-457,
      L-500, L-534, L-572, L-581, L-600, L-602, L-603, L-604, L-607, L-633, L-635), the completed
      pipelines 438, 452, 453, 457, 534, 600, 602, 613 and 633, queued 642, and a read-only
      brain search.
- [x] Discovery with file:line: the rail, `marley_rail`, the `Sidebar` trait and the AI gate,
      the keymaps, Zed's `ui`, the project panel, preview tabs, Rusty's eight tools, its
      favourites, its Qt Explorer, Ely's tree, FileTree, Favorites and InlineEdit, the e2e
      harness.
- [x] Prior-art sweep, three legs: Zed's, Orca's and Warp's maps; Obsidian, VS Code, the MCP
      specification and Rusty's own screen; Zed's `ui`, gpui, `workspace`, `project_panel`
      (read, not reused), Marley's rail, `Cargo.lock`, and Ely.
- [x] How the Brain view fits the rail's model, locked with its reason (D2); the order against
      #645 locked, Chad's, with what a click does before #645 (D1), checked against #645's
      draft; the fixed row's unbuilt entries left out (D5), Graph left to #647's draft;
      favourites checked and deferred (D9).
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D13, twenty-four EARS rows, the
      phase plan.
- [x] Design: approach, file manifest by crate, no touchpoint rows, the visual check plan, risks,
      the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Reconciliation, 2026-10-03
- Rusty's TICKET-040: a deleted folder's pages come back from `archive/` into the index until it
  lands, so after a folder delete the tree re-reads `brain_tree`, not `brain_list_pages`, and the
  Plan phase decides whether `archive/` shows in the tree or is hidden like a dot-folder.
- Rusty's TICKET-041: `brain_new_page` turns `a/b` into a root page `a-b`; new page in a folder
  sends the folder as `folder`, never a slashed `name`.
- The search field sends on Enter (D-rule shared with #646); the stand-in is named in
  `MARLEY_RUSTY_MCP` (#643); the file is `rusty/brain.rs`.

### Promotion (2026-10-04)
- Promoted into `active/`; the BACKLOG row removed; the ticket in-progress. Pre-flight green, no
  other active pipeline, cargo idle, `/mnt/fast` 202G free. #643 landed as 6bae63ef8c.
- **Brain:** `brain ask` (consultation `77214de5010c4774a998d8185f8bb42d`) on D2 and D8 returned
  due follow-ups on other work only; a search found nothing on the rail's view or on search
  timing beyond the plan's own decisions.
- **#643's names, as shipped** (`crates/marley_workbench/src/rusty.rs`): the `Rusty` global
  (`source`, `state`, `settings`, `server`, `keeper`) with `State::Connected { server, via }`;
  `call(server, tool, arguments, cx) -> Result<String, String>`, its errors worded
  "`<tool>` failed: …" (Zed's client passes a JSON-RPC error's message through,
  `context_server/src/client.rs:456`) and "`<tool>` was refused: …"; `list_changed` arrives in
  `connected`'s loop, which re-reads the settings (`:371-377`), on the embedded connection only;
  the stand-in is `crates/marley_rusty/stand_in/rusty-mcp`, its state in `$RUSTY_STAND_IN_STATE`,
  its log `calls` with each request's pid. The vault cache hooks in beside `read_settings`.
- **Seams re-read against the tree** (an Explore pass for Zed's crates and Rusty, the rail by
  hand): every cited Zed API stands (`ListItem` `list_item.rs:32-286`, `Disclosure`
  `disclosure.rs:23`, `indent_guides` `indent_guides.rs:53` with `with_compute_indents_fn` `:84`,
  `ContextMenu::build` `context_menu.rs:284`, `IconButton::indicator` `icon_button.rs:117`,
  `uniform_list` `uniform_list.rs:22` with `with_decoration` `:653`, `PreviewTabsSettings`
  `item.rs:67`, `project_path_for_path` `workspace.rs:4359`, `open_path_preview` `:5139`,
  `Window::prompt` `window.rs:6448`, `Toast::new` `workspace.rs:786`); the icons named in D3, D5
  and D7 exist, with `RotateCw` for Refresh. `secondary-alt-v` is bound in neither Zed's Linux
  keymap nor Marley's. The rail moved: `Rail` `rail.rs:120`, `new` `:739`, `focus_filter`
  `:2730`, `cancel` `:2741`, `show_toast` `:3321`, `empty_space_menu` `:3766`, `render_header`
  `:3928`, `render_filter` `:4229`, `Focusable` `:8173`, `impl Sidebar` `:8180`, `impl Render`
  `:8320`; `end_drag` (`rail_order.rs:467`) acts only on the rail's own drag flag, so a vault
  drag leaves it alone. `register_sidebar` `marley_workbench.rs:866`; `rusty::init` `:778`; the
  Workspace block of `keymap.json` `:5-16`. The in-app guide's PROJECTS line is still
  `guide/index.html:453`.
- **Rusty's tools, re-read** (`rusty-v3`, `main.rs`, `brain/mod.rs`, `brain/vault.rs`): as the
  queued notes say, with these exact shapes. `VaultNode`'s `kind` is a plain string and
  `children` is always present; a folder's `pages` counts its pages at any depth.
  `brain_new_page { folder, name }` answers the new slug as a bare JSON string, and `folder: ""`
  is the root; `brain_new_folder` answers the cleaned path; `brain_rename` answers `{ from, to,
  kind, pages_rewritten }`, and `to: "/"` is the root; `brain_delete_page` answers `"deleted"`,
  `brain_delete_folder` the archive path; `brain_daily_note` answers a `BrainPage` with `slug`.
  Refusals: "Already exists: …", "Not found: …", "Cannot move a folder into itself", "No folder
  …", "Page not found: …", "Folder not found: …", "Refusing to delete the vault root".
  `brain_tree` lists `archive/` (only dot entries are left out), and Rusty makes `archive/` with
  the vault (`ensure_dirs`); a deleted page lands there as `<name>.md_<secs>`, a file, and a
  deleted folder as `archive/<name>_<secs>/`, whose pages count.
- **Decided at promotion:**
  - **`archive/` is shown as Rusty serves it** (the reconciliation's open point): D7 stands;
    Marley shows what `brain_tree` serves, and hiding a folder Rusty lists would be a rule of
    Marley's on Rusty's data. Rusty's TICKET-040, reported landed on its main the same day by the
    rusty-v3 session, leaves the root's `archive/` out of `brain_tree` and refuses writes into
    it; the stand-in models that Rusty, the one Marley meets once Chad reinstalls, so no shot
    lists `archive`.
  - **The vault's folder comes from `settings_list`** (a deviation from D1's `setting_get`):
    `settings_list` lists every stored setting (only credential-like values masked,
    `main.rs:991-1003`), `brain_vault_path` among them when stored, and #643 already reads it on
    connect and on each `list_changed`. Unset, Rusty's own default is `$HOME/.rusty/brain`
    (`core.rs:73-80`), Marley's rule too. The stand-in seeds `brain_vault_path` with its scratch
    vault, so a scenario never opens a file of the user's.
  - **The stand-in announces vault changes as Rusty's watcher does** (a deviation from the
    `SIGUSR1` and stop-file design): it watches its vault's files as it watches its settings
    file, so a page the scenario writes is announced with no signal, and its own writes are too.
    The connection is taken down as #643's scenario does, by moving the program away and killing
    Marley's stand-in; no stop file is needed.
  - The toasts for refusals drop `call`'s "`<tool>` failed: " prefix and say what was being done,
    with Rusty's message after it ("Could not rename delta: Already exists: notes/alpha").
- **Ely**, for D13: this session's scratchpad holds no clone; the /spec session's
  (`…/531f3d65-…/scratchpad/repos/ely`, HEAD `2f8b2f6`) is read for `src/lists/tree/model.rs`
  and `LICENSE-MIT`.

## Phase 2 — Code (2026-10-04)
- **Built, to the manifest:**
  - `crates/marley_rusty/src/vault.rs` (new, declared in the crate root): the tool names
    (`BRAIN_TREE` … `BRAIN_DELETE_FOLDER`), `VAULT_PATH_KEY`, `SEARCH_LIMIT` (50); `VaultNode`
    (`holds`, `find`), `NodeKind` (any other kind read as `File`), `SearchHit`,
    `slug_from_answer`, `page_slug_from_answer`, `RenameReport`; `VaultRow`, `rows`, `Key`,
    `Move`, `step`, `folder_of`, `name_of`, `folders_above`, `child_path`, `rename_target`,
    `move_target`, `reopen`. `rows`, `step`, `holds` and the drop guard ported from Ely's
    `src/lists/tree/model.rs` at `2f8b2f6`, Ely's MIT notice on the file.
  - The stand-in: the eight vault tools over `$RUSTY_STAND_IN_STATE/vault` (made with
    `archive/`, stored as `brain_vault_path`), Rusty's answers and refusals, refusals as JSON-RPC
    errors; its watch now covers the vault's files as well as its settings file. Driven by hand
    over stdio before the scenario: every tool, both refusals and the announcement checked.
  - `marley_workbench::rusty`: the `Vault` and `VaultReads` globals, `want_vault`,
    `reread_vault` (one read in flight, one queued, parsed off the main thread, written only when
    it differs, dropped when the source moved), hooked to each connection and each embedded
    `list_changed`; `call_tool`, `is_connected`, `is_on`, `unavailable`, `vault_folder`; the
    `Rusty` global made `pub(crate)` for the rail's observer; `pub mod brain`.
  - `crates/marley_workbench/src/rusty/brain.rs` (new): `init` (the action on every workspace,
    deferred), `toggle_in_window`, `open_page`, `BrainView` (fixed row, search, tree, hits, keys,
    menus, name editor, drag, delete prompt), `DraggedVaultEntry`.
  - `rail.rs`: `BrainSide { view, entity, connected, _rusty }` and `RailView`; `rusty_changed`,
    `shown_brain`, `show_view`, `brain_refusal`, `toggle_brain_view`, `follow_focus`;
    `render_header` with `render_view_switch` and `render_new_page`; the body swapped in
    `render`; `focus_filter` and `cancel` aware of the Brain view.
  - `marley_workbench.rs`: `ToggleBrainView` in `actions!(marley, …)`. `keymap.json`:
    `secondary-alt-v` in the Workspace block. `Cargo.toml`: `smallvec` (the indent guides'
    callback returns one). `guide/index.html`: the Brain view article and its contents line, the
    PROJECTS line.
  - `script/e2e/644-brain-view-in-the-rail.sh`, written in this phase, as #643's was, so its click
    places are set before the receipt binds it.
- **Deviations from the plan:**
  - **Opening the rail happens outside the rail's update.** The design had `toggle_brain_view`
    open the sidebar; `MultiWorkspace::open_sidebar` reads the sidebar (its side and its focus
    handle, `multi_workspace.rs:500-526`), so from inside the rail's update it would panic.
    `toggle_in_window` opens the sidebar first, then updates the rail. Found in review, before any
    run.
  - **`Focusable` stays the rail's own handle.** The design returned the Brain view's handle while
    it shows; Zed copies the sidebar's handle into every workspace when the sidebar opens
    (`apply_open_sidebar`), so a handle taken while one view showed would be stale after a flip.
    The rail forwards focus given to its own handle into the Brain view instead (`follow_focus`,
    `on_focus`), which also covers a click on the header.
  - **The Brain view's state is one field, `BrainSide`.** A fourth `bool` in `Rail` trips clippy's
    `struct_excessive_bools`, and four new fields put `Rail::new` five lines over the 100-line cap;
    one field built by `BrainSide::new`, the focus subscriptions built by `follow_focus`, and
    `observe_marks` called in the struct literal bring it back under.
  - **`rusty::brain` is a `pub` module.** Its `pub(crate)` items trip clippy's
    `redundant_pub_crate` inside a private module, and `pub` items trip `unreachable_pub` (warn,
    so an error under `-D warnings`); `rusty` is itself `pub`, so a `pub mod brain` satisfies both.
  - **Chevron icons, not `ui::Disclosure`,** in a row's start slot, as the project panel draws its
    own: a `Disclosure` is a button with its own click inside a row that already folds on click,
    and a page row needs a blank of the same width to line up.
  - **The rail's Escape passes on while Brain shows,** so it never clears the hidden project
    filter.
- **Review of the diff** against REQ-001 to REQ-024: PROJECTS and no switch unless connected
  (`render_header` reads `brain.connected`, written by the observer, F-438-a); the toast's words
  from `rusty::unavailable` (REQ-002); the switch's pressed state and the attention dot
  (`has_attention` or an inbox entry, REQ-021); the click rule from `PreviewTabsSettings`
  (REQ-006, REQ-007); search only in `confirm` (REQ-010); every write through `Self::write` and
  `call_tool`, nothing in the view or the rail touching the disk (REQ-024; `open_page` reads,
  through Zed's project); a refusal is a toast and changes no row (REQ-016); delete prompts
  first (REQ-018). Re-entrancy: the rail's observer reads globals only; the action runs deferred,
  opens the sidebar before the rail's update; the Brain view's tasks update it through its weak
  handle; `reread_vault` writes globals only in its spawned task. Provenance: Zed's `ui`,
  `workspace` and `editor` used as they are; `project_panel` read for behavior only; Ely's model
  ported under its notice; nothing from Warp.
- **Rusty moved during the phase.** The rusty-v3 session reported Rusty's TICKET-035 and 040 to
  048 on its main (later 036, 037, 038 and 043 too), live once Chad reinstalls Rusty. Recorded in
  `docs/marley/rusty-in-marley.md`'s triage. For this ticket, 040: `brain_tree` leaves the root's
  `archive/` out and writes into it are refused. The stand-in follows it (its tree, search and
  three writers), the scenario's row numbers moved up one, and the spec's shots `644-04` and
  `644-18` no longer name `archive`. `{folder, name}` still makes a page (041 adds `{path}`, for
  #654).
- **Checks:** clippy on `marley_rusty` and `marley_workbench` (`--all-targets`, `-D warnings`)
  green after five rounds (doc paragraphs, `Self`, generic hashers; `struct_excessive_bools` and
  the line cap, which became `BrainSide`; `redundant_pub_crate` against `unreachable_pub`, which
  became `pub mod brain`; by-reference windows and contexts, associated functions, `map_or_else`,
  `clone_from`, semicolons). The stand-in driven by hand over stdio. `just build` green. The
  scenario run twice before the gate: run 1 green to `644-17`, where the guessed row pitch (22 px,
  first row at 114) had drifted onto `projects` by row 12 and the delete check failed; run 2, with
  the measured pitch (23 px, 117), the first toast closed, and the terminal's tab clicked before
  the bell (an editor tab stood in front of it), green on every check. The box's cargo was held
  by another project's Miri runs for about two hours before clippy; the wait was coordinated with
  that session, nothing stopped.
- **Gate, run 1:** RED on gate:14 alone: the public `rusty` module's doc linked to the
  crate-private `Vault`, and the public `rusty::brain` module's doc to `open_page`
  (`rustdoc::private_intra_doc_links`). Both are plain code now. The other 16 passed.
- **Gate, run 2:** `GATE GREEN [diff]`, 17 passed, the receipt written.

## Phase 3 — Test (2026-10-04)
- **Scenario:** `script/e2e/644-brain-view-in-the-rail.sh`, under `compositor sway` (clicks,
  right-clicks and a drag), run on the gated tree: `just build`, then `just e2e
  script/e2e/644-brain-view-in-the-rail.sh` with `SHOT_DIR` in the scratchpad. Exit 0; every
  check passed: the harness's copy turns Rusty off; no `rusty-mcp` ran while off; the Brain view
  read the vault; Today asked for the daily note; nothing was searched while typing and Enter
  searched once; the page made with `brain_new_page {"folder": "notes", "name": "gamma"}`, the
  folder with `brain_new_folder {"path": "notes/drafts"}`, the rename `notes/gamma` →
  `notes/delta`, the move `to: "decisions/"`, the delete `brain_delete_folder {"path":
  "projects/marley"}`. The vault calls the run printed are only tool calls (REQ-024, with the
  review): `brain_tree` after each change, the writes above and the refused rename.
- **Shots, each read** (row places measured on the Code phase's runs: first row y 117, 23 px):
  - `644-01-header-off` (REQ-001): Rusty off: the header reads PROJECTS beside Add Project; the
    filter and the project's rows under it.
  - `644-02-off-toast` (REQ-002): Ctrl+Alt+V while off: a toast, "Rusty is off. Turn it on in the
    Rusty section of the Marley settings."; the rail still shows Projects.
  - `644-03-switch` (REQ-003): `marley.rusty.enabled` set true from outside: PROJECTS gives way to
    the ListTree and BookCopy buttons, Projects pressed (accent, read in an 8× crop), Add Project
    still at the end.
  - `644-04-brain` (REQ-004): Brain clicked: the Today notepad under the header, "Search the
    brain…", and `daily` 1, `decisions` 2, `notes` 2, `projects` 2, then `home`; no `archive`
    (Rusty's TICKET-040, the stand-in's tree); `+` (New Page) at the header's end; the tooltip
    "Brain Ctrl-Alt-V".
  - `644-05-unfolded` (REQ-005): `projects` and `projects/marley` open: `plan` and `shell` two steps
    in, chevrons turned, indent guides drawn.
  - `644-06-preview` (REQ-006): `notes` open, one click on `alpha`: an `alpha.md` tab with its
    title in italics, the row selected.
  - `644-07-kept` (REQ-007): a double-click: the same tab, upright.
  - `644-08-today` (REQ-008): Today: a kept `2026-10-04.md` tab in front, `daily` opened and
    counting 2, today's row selected.
  - `644-09-search` (REQ-009, REQ-010): "beta" and Enter: one hit, `beta notes/beta`, in the
    tree's place, the field's clear button shown; the log holds one `brain_search`, none while
    typing.
  - `644-10-keys` (REQ-011): Escape twice, Home, Down ×4, Left, Right, Down, Enter: `notes` open
    again, `alpha` selected, its tab in front.
  - `644-11-menu` (REQ-012): `notes` right-clicked: New Page, New Folder, a separator, Rename,
    Delete, the first entry highlighted.
  - `644-12-new-page` (REQ-013): New Page, "gamma", Enter: `gamma` under `notes` (3), selected; a
    kept `gamma.md` tab in front.
  - `644-13-new-folder` (REQ-014): New Folder, "drafts", Enter: `drafts` first under `notes` with
    0, selected.
  - `644-14-renamed` (REQ-015): Rename on `gamma`, "delta", Enter: `delta` in its place, selected.
    The open tab keeps the old name until Zed sees the file move (Out: tabs following a rename).
  - `644-15-refused` (REQ-016): Rename on `delta` to "alpha": a toast "Could not rename delta:
    Already exists: notes/alpha"; `delta` unchanged.
  - `644-16-moved` (REQ-017): `delta` dragged onto `decisions`: listed first under it (3),
    `decisions` opened, the row selected; `notes` back to 2.
  - `644-17-delete-prompt` (REQ-018): Delete on `projects/marley`: Zed's prompt, "Delete the folder
    projects/marley and its 2 pages?", "Rusty moves it into archive/ with everything in it, and
    leaves the links to its pages.", Delete and Cancel.
  - `644-18-deleted` (REQ-019): Delete answered: `marley` gone, `projects` counting 0.
  - `644-19-live` (REQ-020): `notes/epsilon.md` written from outside: `epsilon` listed under
    `notes` (3) with no click.
  - `644-20-attention` (REQ-021): the terminal's tab, `sleep 3; printf '\a'`, a click in the tree:
    the Brain view shows and Projects carries the accent dot (read in an 8× crop); the terminal's
    tab has its unread dot.
  - `644-21-flipped` (REQ-022): Ctrl+Alt+V: the Projects view, Projects pressed, the terminal row
    with its dot.
  - `644-22-down` (REQ-023): Ctrl+Alt+V back to Brain, the stand-in moved away and Marley's killed:
    the header reads PROJECTS and the project's rows show.
- **By review:** REQ-024 (every change a tool call; `open_page` reads; nothing in the view or the
  rail writes a file). The toasts for the Zed layout and the AI gate, the service connection's
  missing announcements, and the flip back to Brain after a reconnect are reached by no scenario
  (the spec's visual check plan says why).
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and
  did not reload it".
- Every shot shows Marley only; none is in the repository. No fix in this phase: the receipt from
  Phase 2's run 2 stands.

## Phase 4 — Complete (2026-10-04)
- **Documented (§21):** `CHANGELOG.md` (Added: Rusty's vault in the rail);
  `docs/marley/rusty-in-marley.md` (R4 shipped 2026-10-04, and the triage's "Landed on Rusty's
  main, 2026-10-04" with TICKET-035, 036, 037, 038 and 040 to 048, at the rusty-v3 session's
  request); `docs/marley/three-prong-plan.md` (C2); `docs/marley_architecture/marley_rusty.md`
  (the `vault` module, Ely's port, the stand-in's vault tools); `docs/marley_architecture/
  marley_workbench.md` (the rail's two views, and "The rail's Brain view"); `docs/marley/guide.md`
  (the rail's header, the right-click line, "The Brain view", the keys table);
  `docs/marley/walkthrough.md` (stop 2.12); the in-app guide page (Code phase). No path outside
  the Marley-owned set changed but `Cargo.lock` (`smallvec` for `marley_workbench`), so no
  ledger row.
- **Knowledge (§19):** `F-claude-644-opening-the-sidebar-from-inside-the-rails-update-would-panic-001`
  (its class covered by `PR-claude-defer-in-does-not-leave-the-entitys-own-update-001`, so no new
  rule); `AD-claude-644-the-brain-view-is-a-view-of-its-own-and-searches-on-enter-001`;
  `L-claude-644-a-sidebar-with-two-views-forwards-focus-into-the-shown-one-001`,
  `L-claude-644-a-module-nested-in-marley-workbench-is-pub-for-two-lints-001`,
  `L-claude-644-the-rails-struct-sits-at-clippys-bool-and-line-caps-001`,
  `L-claude-644-the-stand-in-models-rustys-main-and-its-watcher-001`.
- **Brain:** `brain decide` on consultation `77214de5010c4774a998d8185f8bb42d`:
  `decisions/marleys-brain-view-is-a-view-of-its-own-in-the-rail-and-searches-on-enter`
  (follow-up 2026-10-18).
- **Closed:** the ticket moved to `tickets/closed/`, its link at `completed/`; no BACKLOG row was
  left (promotion removed it). The pair archived to `pipeline/completed/`.
