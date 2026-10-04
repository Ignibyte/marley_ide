---
pipeline_id: 82e2078e-a618-468c-9bb5-a23a8e3c435b
ticket: docs/planning/tickets/open/TICKET-644-brain-view-in-the-rail.md
status: Phase 4 — Complete PASS
title: "The rail's Brain view: Rusty's vault behind a switch in the header"
type: feature
slice: Rusty in Marley R4 (R-D9, R-D10)
references: [docs/marley/rusty-in-marley.md, docs/marley/workbench-shell.md]
---

## Title
While `marley.rusty.enabled` is on and Rusty is connected (#643), the rail's header, which reads
PROJECTS today, becomes two icon buttons, Projects and Brain, and one key flips between them.
Brain swaps the rail's content for Rusty's vault: a fixed row with Today, a brain search field,
and the vault tree over `brain_tree`, where a click opens a page (its file in Zed's editor until
#645's Page tab takes the click) and the right-click menu and a drag make, rename, move and
delete pages and folders through Rusty's tools, never the disk. Chad, 2026-10-02, picking how
Rusty's screens are reached: "Switch in the rail header".

## Scope
### In
- **Order:** after #643 (the switch and the connection: `marley.rusty`, the client in
  `marley_workbench::rusty` with its `list_changed` signal, the crate `marley_rusty` with its
  fixtures and stand-in `rusty-mcp`), whose names this ticket takes as they ship, and before
  #645, in Chad's order (#643, #644, #645, #646, #647). Every open goes through one function,
  which opens the page's file in Zed's editor until #645 points it at its Page tab (D1).
- **The vault model** (`marley_rusty::vault`, pure, no gpui): the typed view of `brain_tree`'s
  `VaultNode` and of `brain_search`'s hits; the visible rows under a set of open folders, each
  with its depth and parent; the list keys' moves; where a drop lands and which drops are
  refused; the paths a rename and a move write; open folders re-keyed after a folder's rename or
  move. Ported in part from Ely's tree model, with Ely's MIT notice on the file (D13).
- **The stand-in:** #643's stand-in `rusty-mcp` gains `brain_tree`, `brain_daily_note`,
  `brain_search`, `brain_new_page`, `brain_new_folder`, `brain_rename`, `brain_delete_page` and
  `brain_delete_folder` over its scratch vault (`$RUSTY_STAND_IN_STATE/vault`, made with
  `archive/` as Rusty makes its vault), with Rusty's results and refusals for the cases the
  scenario drives, and `brain_vault_path` stored in its settings as that vault (#645 needs the
  same answer); it logs each `tools/call`, and announces `list_changed` when a file in its vault
  changes, as Rusty's watcher does, its own writes included (promotion).
- **The vault cache** (`marley_workbench::rusty`): `brain_tree` read once per change for every
  window's Brain view, off the main thread, kept in a global written only when the read differs
  (L-572); re-read on #643's `list_changed`, after each write the Brain view makes, and on
  Refresh; one read in flight and one queued.
- **The header switch** (`rail.rs`, `render_header`): the Projects and Brain buttons while Rusty
  is on and connected, PROJECTS otherwise; the end button follows the shown view (Add Project or
  New Page); the attention dot on Projects while Brain shows (D3).
- **The Brain view** (`marley_workbench::rusty::brain`, a view the rail holds and draws under
  its header): the fixed row with Today (D5), the search field (D8), the vault tree (D7), the
  opens through `open_page` (D1, D6), the menus, the inline name editor, the drag and the delete
  prompt (D10), the reading and error lines.
- **The key:** `marley::ToggleBrainView`, `secondary-alt-v` in the Marley keymap's Workspace
  block, with a toast when the Brain view cannot show (D4).
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`): the rail's header line
  and a Brain view article. It sits under `crates/marley_*`, which the commit receipt binds, so
  it changes in the Code phase, before the gate.
- `script/e2e/644-brain-view-in-the-rail.sh`.

### Out (explicitly deferred)
- **The Page tab:** #645 points `open_page` at `rusty::page::open_later`, its opener, so the
  tree's clicks, Enter, Today and a new page open Page tabs; no caller here changes.
- **Favourites** wait for Rusty's TICKET-037 (RQ4; open, unbuilt): Rusty serves none today (D9).
  The follow-up's scope, one line: a Favourites section between the search field and the tree
  over TICKET-037's tools, Ely's `Favorites` ported (a row per favourite, a star on hover to
  unpin, a drag to reorder), and Add to Favourites on a row's menu.
- **The other fixed-row entries:** Graph lands with #647's Graph tab, Tasks and Decisions with
  R7, Memory, Skills and Secrets with R8, each at its place in R-D9's order (D5).
- **Brain search in the Knowledge panel** (#646, with snippets and its case and pattern
  switches) and search filters (type chips, Ely's `SearchFilters`): the rail's field lists hits
  by title and slug and passes Rusty's operators as typed. Both call `brain_search`, so they
  should share one rule for when a query is sent (D8).
- **Live refresh past `list_changed`:** Rusty's change cursor (TICKET-035, `changes_since`) is
  not built and nothing here depends on it; with `connection: service`, Zed's HTTP transport
  opens no stream for server notifications (R-D2), so the tree re-reads only after its own
  writes and on Refresh.
- **Open Page tabs following a rename, move or delete made here:** `brain_rename`'s report names
  `from` and `to`; handing it to #645's tab is a later change.
- **The tree following the active Page tab** (unfolding to its page and selecting the row).
- **Keeping the chosen view and the open folders across a restart:** the rail starts on Projects.
- **Files that are not pages** (images, canvases): left out of the tree (D7).
- **F2 and Delete keys, a Move to… dialog, multi-select, copy and undo.**
- **The AI gate:** Zed hides every sidebar while its AI features are off
  (`MultiWorkspace::multi_workspace_enabled`, `multi_workspace.rs:439-441`), and the Brain view
  with it; decoupling the rail is workbench-shell.md's open decision 2 and a touchpoint in
  `multi_workspace.rs`. The Knowledge panel and the palette actions still work.

## Reference (§20)
Upstream Zed: the project panel's tree behavior is the reference the vault tree follows, kept as
Zed has it: one click opens a preview tab where `preview_tabs.enable_preview_from_project_panel`
allows and a double-click keeps it (`project_panel.rs:6397-6402`), a new entry and a rename are
an editor in the row (`:805-830`, `:2392`), a removal asks first through `Window::prompt`
(`:2768`, `:2881`), a drag onto a folder moves into it (`:6285`), and rows indent with indent
guides (`:7512`). Marley reimplements it in its own crate on the `ui` crate's public components;
no `project_panel` code is carried (§20, the code-layer wall). Until #645, a page opens as Zed
opens a file outside the project: `Workspace::project_path_for_path` with `visible: false`,
then `open_path_preview` (`workspace.rs:4359`, `:5139`). The `workspace::Sidebar` duties
(width, open state, persistence, the cycle actions, the switcher) stay the rail's, unchanged,
whichever view shows. The header switch follows R-D9's published references, Obsidian's ribbon
and VS Code's activity bar. Warp: no analog for a knowledge vault; no Warp source or spec used.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` §1.6 (the
  project panel is a `Panel`), the pane's single preview tab (`preview_item_id`, `:132-133`) and
  the file finder's `open_path_preview` under `PreviewTabsSettings` (`:414-415`).
  `docs/orca_architecture/05-terminal-and-workspace.md` §2.14: Orca's explorer is lazy,
  virtualized, with inline create and rename, drag-move and delete to the trash, and "Zed's
  project panel covers this". `docs/warp_architecture/crates/warp_files.md`: a file model, no
  tree view of use here. Marley's own maps: `docs/marley/workbench-shell.md` D1, D3, D7, D8 and
  its deferred AI gate; `docs/marley/rusty-in-marley.md` R-D1, R-D2, R-D8 to R-D10.
- **Published material:** Obsidian's file explorer (new note, new folder, rename, drag to move,
  delete) and ribbon, and VS Code's activity bar, the two shapes R-D9 names. The MCP
  specification's `notifications/resources/list_changed`. Rusty's own Qt Explorer, the screen
  this replaces (`/srv/stacks/rusty-v3/crates/rusty-app/qml/Explorer.qml`, MIT): New note with no
  name (`:133`), New folder, Rename in the row with `/` turned into `-` (`:150-157`), Move to…
  as `brain_rename` with `to: "<folder>/"` (`:158`), Delete (`:159-163`), the menu (`:385-392`),
  Favorites above the tree (`:196-226`).
- **Code we already ship:**
  - Zed's `ui`: `ListItem` with `indent_level`, `toggle_state`, `on_secondary_mouse_down`
    (`components/list/list_item.rs:32-272`), `Disclosure` (`disclosure.rs:9-73`),
    `indent_guides` with `IndentGuideColors::panel` (`indent_guides.rs:22-103`), `ContextMenu`
    (`context_menu.rs:213-680`), `IconButton` with `toggle_state` and `indicator`
    (`button/icon_button.rs:33-155`), `Tooltip::for_action`. gpui's `uniform_list`
    (`elements/uniform_list.rs:22`) and its drag and drop (`on_drag`, `drag_over`, `can_drop`,
    `on_drop`). `workspace::item::PreviewTabsSettings` (`item.rs:67-75`),
    `Workspace::project_path_for_path` and `open_path_preview` (`workspace.rs:4359`, `:5139`;
    `open_abs_path` always keeps the tab, `:5078`), `Window::prompt`, `Editor::single_line`,
    `menu`'s list actions. Taken.
  - `project_panel` read for behavior and not reused: it is bound to `Project`, `Worktree` and
    `ProjectEntryId` throughout (`project_panel.rs:35-37`, `:95`, `:4531-4541`) and its file
    operations go to the disk. `ui::TreeViewItem` (`tree_view_item.rs:8`) draws two levels only
    and takes no icon. `ToggleButtonGroup` (`toggle_button.rs:171`, the segmented control
    `git_picker.rs:198` uses) is wider than the header leaves. All three rejected.
  - Marley's own: the rail (`rail.rs`: `render_header` `:3928`, `render_filter` `:4229`,
    `focus_filter` `:2730`, `cancel` `:2741`, `show_toast` `:3321`, `Focusable` `:8068`,
    `render` `:8215`); `marley_rail::has_attention` (`marley_rail.rs:1555`); the Harness section
    drawn outside the rail's model (AD-534); `register_sidebar`'s lookup of the window's rail
    (`marley_workbench.rs:876`); `groups::GroupNamePrompt` (`groups.rs:433`), rejected for names
    in the tree (D10); #642's off-feature toast (`system_one::check_toast`'s pattern). #643's
    client, connection state, `list_changed` signal and stand-in, which this ticket calls; #645's
    opener (`rusty::page::open_later`), which takes over `open_page` after it.
  - `Cargo.lock`: nothing owns a vault tree; `fuzzy` is not needed (search is Rusty's).
  - Ely GPUI Components (R-D10; shallow clone at the scratchpad's `repos/ely`, HEAD `2f8b2f6`):
    `src/lists/tree/model.rs` (`rows` `:134-175`, `step` `:185-208`, `holds` `:95-101`,
    `can_drop` `:266-269`) is ported (D13). `src/lists/files.rs` (`FileTree`, 317 lines) is a
    wrapper that builds that tree from path strings, which `brain_tree` already returns nested:
    nothing taken. `src/lists/tree/row.rs` (rows, guides, its F2 `Input`) is left for Zed's
    `ui`, which wins over a port. `src/documents/knowledge.rs` `Favorites` (`:121-256`) waits
    with D9. `src/forms/inline.rs` `InlineEdit` (`:90-203`) commits on Enter or blur and cancels
    on Escape, as Zed's editor in a row does: nothing taken.

## UI proof
`script/e2e/644-brain-view-in-the-rail.sh` (`compositor sway`: it clicks, right-clicks and drags
in the rail). Fixtures: a scratch repository opened with `open_path`; a terminal HOME whose
`.bashrc` sets a plain prompt; #643's stand-in `rusty-mcp`, named in `MARLEY_RUSTY_MCP` (never put first on the PATH: Marley takes its PATH from the login shell, L-531, so the real `rusty-mcp` could win; reconciled 2026-10-03 to #643's rule), over a scratch vault
(`$RUSTY_STAND_IN_STATE/vault`) the scenario fills with made-up pages, never the user's (R-D8):
`daily/2026-01-05`, `decisions/use-a-rail` and `decisions/keep-zed`, `notes/alpha` and
`notes/beta`, `projects/marley/plan` and `projects/marley/shell`, and `home` at the root, beside
the empty `archive/` Rusty makes with a vault; the stand-in stores `brain_vault_path` as that
vault. Setup sets `marley.rusty.connection` to
`embedded` and leaves `marley.rusty.enabled` false, as the harness writes it; later changes are
edits of the run's settings from outside (L-607). Run before #645, a page opens as its file in
Zed's editor; run after, as a Page tab, and the shots say which. Shots:
- `644-01-header-off`: Rusty off: the header reads PROJECTS beside Add Project.
- `644-02-off-toast`: `secondary-alt-v` while off: the toast; the rail still on Projects.
- `644-03-switch`: `marley.rusty.enabled` set true, connected: Projects (pressed) and Brain.
- `644-04-brain`: Brain clicked: Today, the search field, the folders `daily`, `decisions`,
  `notes`, `projects` with their page counts, then `home`; `archive/` left out, as Rusty leaves it
  since its TICKET-040; New Page at the header's end.
- `644-05-unfolded`: `projects` and `projects/marley` unfolded: nested rows, indent guides.
- `644-06-preview`: one click on `alpha`: an `alpha.md` tab, title in italics; the row selected.
- `644-07-kept`: a double-click on `alpha`: the same tab, upright.
- `644-08-today`: Today: a kept tab for `daily/<today>.md`; `daily` counts 2.
- `644-09-search`: "beta" typed, Enter: `notes/beta` listed in the tree's place; the stand-in's
  log shows one `brain_search`.
- `644-10-keys`: Escape twice; Home, Down, Down to `notes`, Left (folds it), Right (unfolds
  it), Down, Enter: `notes` unfolded, `alpha` the selected row and its tab in front.
- `644-11-menu`: `notes` right-clicked: New Page, New Folder, Rename, Delete.
- `644-12-new-page`: New Page, "gamma", Enter: `notes/gamma` selected, its tab kept.
- `644-13-new-folder`: New Folder on `notes`, "drafts", Enter: `notes/drafts` listed.
- `644-14-renamed`: Rename on `gamma`, "delta", Enter: `delta` in its place.
- `644-15-refused`: Rename on `delta`, "alpha", Enter: a toast with Rusty's "Already exists:
  notes/alpha"; `delta` unchanged.
- `644-16-moved`: `delta` dragged onto `decisions`: listed under it.
- `644-17-delete-prompt`: Delete on `projects/marley`: the prompt names it, its 2 pages and
  `archive/`.
- `644-18-deleted`: Delete answered: `projects/marley` gone, `projects` counting 0.
- `644-19-live`: `notes/epsilon.md` written into the scratch vault from outside: `epsilon`
  listed with no click.
- `644-20-attention`: a bell rung in the project's terminal while Brain shows: a dot on Projects.
- `644-21-flipped`: `secondary-alt-v`: the Projects view, Projects pressed.
- `644-22-down`: the stand-in moved away and Marley's killed, so it stays down: the header reads
  PROJECTS, the rows show.

## Locked-In Decisions
- D1 — **Before #645, in Chad's order; a page opens as its file until then.** The batch runs
  #643, #644, #645, #646, #647, and #645 builds on this tree: its scenario opens a page by
  clicking it here, and its scope points the tree's clicks at its opener. Every open in the
  Brain view (a page row's click, Enter on a page or a hit, Today, a new page) goes through one
  function, `rusty::brain::open_page(slug, preview)`. In this ticket it opens `<vault>/<slug>.md`
  in Zed's editor as Zed opens a file outside the project (`project_path_for_path` with
  `visible: false`, then `open_path_preview`), the vault being `brain_vault_path` as
  `settings_list` gives it (read when Rusty connects and on each `list_changed`, #643), or Rusty's
  default `$HOME/.rusty/brain` when Rusty stores none (`core.rs:73-80`): the rule #645's Edit view
  uses. (Promotion: `settings_list` already carries the key, so no `setting_get` call.) #645 points it at `rusty::page::open_later`
  and no caller changes. Opening reads a file; a save from that buffer is a source edit, which
  Rusty's watcher indexes and commits (R-D4); the Brain view itself writes nothing to disk.
  Rejected: promoting this after #645 (its scenario reaches its tab through this tree); a click
  that does nothing until #645 lands.
- D2 — **A view of its own, not rows in `marley_rail`.** The Brain view is an entity,
  `BrainView` in `marley_workbench::rusty::brain`, which the rail holds and draws under its
  header in place of its filter, inbox and rows, with its own pure row model in
  `marley_rusty::vault`. `marley_rail`'s model is the window's projects: `Row`, `Selection`,
  `walk`, `step`, `cycle_project`, `cycle_row`, the switcher, the filter and the attention order
  all walk projects, and Zed's Next and Previous Project and Thread reach them through the
  `Sidebar` trait (workbench-shell D8). Vault rows in that enum would put pages in the project
  cycle, the switcher and the filter, or need a guard in each. AD-534 already draws a rail
  section outside the model. The rail keeps every `Sidebar` duty whichever view shows, and
  `marley_rail` keeps no Rusty tool shapes. Rejected: vault variants of `marley_rail::Row`; a
  left-dock Vault panel (R-D9 replaced it); Zed's project panel over the vault added as a
  worktree (the vault would list among the project's folders, and its file operations go to the
  disk, not to Rusty's tools).
- D3 — **The header.** Projects (`IconName::ListTree`) and Brain (`IconName::BookCopy`) are
  `IconButton`s with `toggle_state` on the shown one and tooltips naming the key, drawn only
  while `marley.rusty.enabled` is on and #643's connection is up; otherwise the header reads
  PROJECTS, as today. The end button follows the shown view: Add Project, or New Page (in the
  selected row's folder, or the root). While Brain shows, Projects carries the rail's attention
  dot (`Indicator::dot`, `Color::Accent`) when the Projects view has anything that needs the user
  (`marley_rail::has_attention`, or a Needs-you entry). The rail keeps the chosen view: while
  Rusty is off or not connected it shows Projects, and Brain again once Rusty connects. The
  header's empty space keeps #600's menu in the Projects view only. Rejected: Zed's
  `ToggleButtonGroup` (a segmented control wider than the header leaves beside the window
  controls); a brain icon of Marley's own (a touchpoint in `icons` and `assets` for one glyph).
- D4 — **The key.** `marley::ToggleBrainView`, bound to `secondary-alt-v` in the Marley keymap's
  Workspace block: free in Zed's Linux keymap and in the Marley keymap (`ctrl-alt-b` is Zed's
  `ToggleRightDock`, `ctrl-alt-j` its sidebar toggle). The MultiWorkspace's root carries the
  workspace's key context and actions (`multi_workspace.rs:2087-2091`), so it reaches from the
  editor, a terminal and the rail alike. It flips the chosen view, opens the rail when closed,
  and focuses the shown view: the tree, or the rail's rows. When the Brain view cannot show, it
  shows a toast saying why: "Rusty is off. Turn it on in the Rusty section of the Marley
  settings.", Rusty not connected (with #643's state), the Zed layout, or the rail hidden while
  Zed's AI features are off. Rejected: a binding in `MarleyRail` only (the key would not reach
  from the editor); hiding the action while off (#642's D4 keeps a pointer to the switch).
- D5 — **The fixed row.** Under the header: the fixed row, the search field, then the tree. The
  row keeps R-D9's order (Today, Graph, Tasks, Decisions, Memory, Skills, Secrets), and an entry
  appears with its tab: this ticket ships Today alone (`IconName::Notepad`, tooltip "Today's
  Note"). Today calls `brain_daily_note` with no date (Rusty's local date; it makes the note
  when missing and announces it) and opens the slug kept through `open_page`. Graph's entry is
  #647's (its scope adds the row's Graph button). Rejected: the other six drawn disabled with
  the slice they wait on (five dead buttons out of seven, for slices with no ticket yet).
- D6 — **Opening a page.** One click on a page row calls `open_page` with `preview` true where
  `preview_tabs.enabled` and `enable_preview_from_project_panel` are on, and a double-click with
  `preview` false, as the project panel's click does (#645 keeps the same rule for its tab);
  Enter does what one click does. A click on a folder row folds or unfolds it.
- D7 — **The tree.** `brain_tree` takes no root or depth (`main.rs:1217`), so the vault is read
  whole and drawn as Rusty orders it, folders first (`vault.rs:362-369`). Folders and pages are
  listed, a folder with its page count (`pages`) muted at the row's end; `file` nodes are left
  out (no tab shows them, and `brain_rename` and `brain_delete_page` take pages and folders);
  `archive/`, where Rusty's deletes go, is listed as Rusty serves it. Drawn with Zed's `ui`: a
  `uniform_list` (the vault here holds 807 pages), `ListItem` rows with `indent_level`, a
  `Disclosure`, `indent_guides` with `IndentGuideColors::panel`, a 20 px step (the project
  panel's default; reading its setting would link `project_panel` for one number), and
  `IconName::Folder`, `FolderOpen` and `FileTextOutlined`. All folders start closed; which are
  open is the view's, in memory. Keys are Zed's list actions (AD-453) in the key context
  `MarleyBrain menu`: `SelectNext`, `SelectPrevious`, `SelectFirst`, `SelectLast`,
  `SelectChild` (opens a folder, or steps into an open one), `SelectParent` (closes it, or goes
  to the parent) and `Confirm`. One row is selected: the keyboard's while the view holds focus,
  otherwise the last one clicked, opened or made. Element ids come from vault paths, which are
  unique (F-606).
- D8 — **Brain search on Enter.** The field calls `brain_search { query, limit: 50 }` when Enter
  is pressed, never per keystroke: with an embedding provider set, Rusty embeds every query
  (`main.rs:845-851`), and this box names OpenAI (the plan's decision 3), so searching as you
  type would send each prefix off the machine. The hits (title, slug) replace the tree until the
  field is cleared; Rusty's `tag:`, `path:`, `file:` and `type:` pass as typed. Up and Down reach
  the hits from the field (L-457); Enter on an unchanged query opens the keyboard's hit; Escape
  clears the field, then returns to the tree, then passes on (AD-457). `secondary-f`
  (`agents_sidebar::FocusSidebarFilter`) focuses the field while Brain shows. A change re-reads
  the tree but does not rerun a search. Rejected: search as you type; a local filter by name in
  its place (it is not brain search).
- D9 — **Favourites wait for Rusty's TICKET-037.** No Rusty tool serves favourites or bookmarks.
  The Qt app keeps bookmarks in its own window state, `~/.config/rusty/workspace.json`
  (`rusty-app/src/terminals.rs:121-129`), and its favourites are the file and folder bookmarks
  (`Explorer.qml:27`, `:196-226`). Marley reads no app's private file (R-D1). Nothing is drawn
  for favourites until TICKET-037's tools exist.
- D10 — **Writes through Rusty's tools only.** Right-click menus: a folder row offers New Page,
  New Folder, Rename and Delete; a page row Open, Rename and Delete; the tree's empty space New
  Page, New Folder and Refresh at the vault's root. New Page and New Folder put a row with an
  empty single-line editor under the folder, as the project panel does: Enter calls
  `brain_new_page { folder, name }` (an empty name lets Rusty call it Untitled) or
  `brain_new_folder { path }`; Escape, or an empty folder name, drops the row. A new page opens
  kept through `open_page` and is selected. Rename puts the editor in place of the name, the name
  selected; Enter calls `brain_rename { from, to }` in the same folder, a typed `/` becoming `-`
  as Rusty's own app and `brain_new_page` do; an unchanged name calls nothing; Escape cancels and
  losing focus commits, as the project panel does. Move: a page or folder row dragged onto a
  folder row goes into it, onto a page row into that page's folder, onto the tree's empty space
  to the root, by `brain_rename { from, to: "<folder>/" }`; a drop onto itself, under itself
  (Ely's `can_drop`) or into the folder it is in calls nothing. Delete asks first with
  `Window::prompt`, naming the page, or the folder with its page count, and saying Rusty moves
  it into `archive/` and leaves the links to it; Delete calls `brain_delete_page { slug }` or
  `brain_delete_folder { path }`. Each success re-reads the tree at once; a refusal shows
  Rusty's message in a toast and changes nothing shown. Open folders under a renamed or moved
  folder stay open. Menus opened by hand stop and prevent their mouse-down (L-600). Rejected: a
  modal name prompt like #600's (trees rename in place in Zed, Obsidian and Rusty's app); a new
  page made as Untitled and renamed after (two writes and two commits in the vault's git for a
  name about to be replaced).
- D11 — **Live.** On #643's `list_changed`, which Rusty sends for every write and every file
  change its watcher sees (`main.rs:2062-2082`), the cache re-reads `brain_tree`. While the read
  is under way the view keeps the last tree; before the first read it says it is reading the
  vault; a failed read shows Rusty's message in the tree's place until a later read succeeds.
- D12 — **Scenarios never touch the real brain** (R-D8). The scenario runs #643's stand-in over a
  scratch vault of made-up pages; the harness keeps `marley.rusty.enabled` false in every run's
  copy (#643), so no other scenario's header changes.
- D13 — **What Ely gives.** Ported into `marley_rusty::vault` with Ely's MIT notice ("Copyright
  (c) 2026 Ely GPUI Component contributors", HEAD `2f8b2f6`, `src/lists/tree/model.rs`) on the
  file, on `String` keys and Rusty's node: the visible-row walk over an open set with each row's
  parent (`rows`), the key moves (`step`: Right opens or enters, Left closes or climbs), and the
  drop guard (`holds`, `can_drop`). Left: Ely's rendering, theme, Lucide icons, `SearchInput`,
  `IconTheme`, git notes, lazily loaded children, checkboxes, multi-select, Before and After drop
  places (the vault has no order of its own) and its F2 `Input`; Zed's `ui` draws the tree, and
  Zed's own wins over a port.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.rusty.enabled` is off, the rail's header shall read PROJECTS with no view switch. | Shot `644-01-header-off` |
| REQ-002 | WHEN `marley::ToggleBrainView` runs while Rusty is off, the system shall show a toast saying Rusty is off and where to turn it on, and keep the Projects view. | Shot `644-02-off-toast` |
| REQ-003 | WHILE Rusty is on and connected, the rail's header shall show a Projects button and a Brain button, the shown view's pressed. | Shot `644-03-switch` |
| REQ-004 | WHEN the user clicks Brain, the rail shall show the fixed row with Today, the search field and the vault's top level with each folder's page count, and New Page at the header's end. | Shot `644-04-brain` |
| REQ-005 | WHEN the user clicks a folder row, the tree shall unfold it, with its rows indented under it. | Shot `644-05-unfolded` |
| REQ-006 | WHEN the user clicks a page row once, the system shall open the page in a preview tab. | Shot `644-06-preview` |
| REQ-007 | WHEN the user double-clicks a page row, the system shall keep the page's tab. | Shot `644-07-kept` |
| REQ-008 | WHEN the user clicks Today, the system shall open today's daily note in a kept tab. | Shot `644-08-today` |
| REQ-009 | WHEN the user presses Enter in the search field, the Brain view shall list Rusty's hits for the query in the tree's place. | Shot `644-09-search` |
| REQ-010 | WHILE the user types in the search field, the system shall call `brain_search` only on Enter. | The stand-in's log at `644-09-search` (one call); review |
| REQ-011 | WHILE the Brain view holds focus, Zed's list keys shall move the selected row, unfold and fold folders, and open the selected page. | Shot `644-10-keys` |
| REQ-012 | WHEN the user right-clicks a folder row, the system shall offer New Page, New Folder, Rename and Delete. | Shot `644-11-menu` |
| REQ-013 | WHEN the user names a new page and presses Enter, the system shall create it through `brain_new_page`, list it and open it. | Shot `644-12-new-page`; the stand-in's log |
| REQ-014 | WHEN the user names a new folder and presses Enter, the system shall create it through `brain_new_folder` and list it. | Shot `644-13-new-folder`; the stand-in's log |
| REQ-015 | WHEN the user renames a row and presses Enter, the system shall rename it through `brain_rename` and list the new name. | Shot `644-14-renamed`; the stand-in's log |
| REQ-016 | IF Rusty refuses a write, THEN the system shall show Rusty's message in a toast and leave the tree as it was. | Shot `644-15-refused` |
| REQ-017 | WHEN the user drops a row on a folder row, the system shall move it into that folder through `brain_rename`. | Shot `644-16-moved`; the stand-in's log |
| REQ-018 | WHEN the user chooses Delete, the system shall ask first, naming the row, a folder's page count and `archive/`. | Shot `644-17-delete-prompt` |
| REQ-019 | WHEN the user confirms Delete, the system shall delete the row through `brain_delete_page` or `brain_delete_folder`. | Shot `644-18-deleted`; the stand-in's log |
| REQ-020 | WHEN Rusty announces `list_changed`, the Brain view shall show the vault's new state without a click. | Shot `644-19-live` |
| REQ-021 | WHILE the Brain view shows and a project row needs the user, the Projects button shall carry the attention dot. | Shot `644-20-attention` |
| REQ-022 | WHEN `marley::ToggleBrainView` runs while the Brain view shows, the rail shall show the Projects view. | Shot `644-21-flipped`; the flip back, review |
| REQ-023 | WHEN Rusty's connection goes down while the Brain view shows, the rail shall show the Projects view and the header PROJECTS. | Shot `644-22-down` |
| REQ-024 | The Brain view shall change the vault only through Rusty's tools, never by writing a file. | Review of the diff; the stand-in's log |

## Phase Plan
- **P1 Plan** — promote after #643 ships; re-read the names this spec takes from it (the
  connection state, the client's call, its `list_changed` signal, the stand-in and how it finds
  its vault); if #645 shipped first after all, `open_page` calls its opener and the shots show
  Page tabs; confirm Zed's prompt answers Delete from the keys; ask the brain (`brain_ask`) on D2
  and D8.
- **P2 Code** — the `README.md` marker first; `marley_rusty::vault`; the stand-in's tools; the
  cache and the vault's folder; `BrainView` and `open_page`; the rail's header, swap, focus and
  dot; the action and the key; the in-app guide page; a review of the diff; `script/gates.sh
  --diff` green.
- **P3 Test** — write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG (Added) and architecture docs (§21), the user docs the notes list,
  ledger capture (§19), `brain_decide` for D2 and D8, close the ticket, archive, commit.
