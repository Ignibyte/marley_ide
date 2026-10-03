# Open a brain page by name, or make it, from a picker: Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-654-rusty-open-page-picker.md
- **Pipeline spec:** 654-rusty-open-page-picker.spec.md

## Phase 1: Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, on Rusty in Marley: "maybe rusty becomes Marley. We would take our
  rusty custom QML app and build it inside of marley", then "Plan it now". On 2026-10-03: "lets
  make a plan to begin the work and spec out the tickets" and "lets make sure we use the gpui
  components we found here". This is the plan's slice R3a (`docs/marley/rusty-in-marley.md`, the
  slices table: "`rusty: open page`: a picker over `brain_list_pages` by title, create on a miss"),
  R-D3's QuickSwitcher row ("favourites first, create on a miss") and R-D10's
  `searchpalette-spotlightsearch-quicklauncher` row. #645's Out list names it as "The second slice,
  its own ticket", including "A click on an unresolved link creates its page the same way"; #646's
  Out list leaves it here too. Queued in the follow-up batch (#654 to #659), after #645.
- **Classification / tier:** feature, small to medium. One modal view and its delegate in
  `marley_workbench::rusty`, one pure module and one small type in `marley_rusty`, one tool in the
  Python stand-in, a changed field on #645's action, two call sites and one link arm in #645's
  `PageView`, one keymap line, one new dependency edge on a Zed crate (`fuzzy_nucleo`). No Zed
  crate changes and no `zed-touchpoints.md` row: `"rusty"` in `zed.rs`'s `test_action_namespaces`
  is #645's. One shippable slice. The unresolved link's create is in it because it is the same
  create path and #645's Out puts it here; favourites and aliases are Out, each waiting on Rusty.
- **Recall (§18.3):**
  - AD-claude-450-new-agent-is-a-picker-behind-the-marley-keymap-001 and completed pipeline 450: a
    picker in the workspace's modal layer behind a `Workspace` binding in the Marley keymap, the
    chord swept against every context of every keymap Zed ships and Hyprland's. D1 and D9 follow it.
  - AD-claude-449-terminal-keys-catch-zeds-actions-and-the-keymap-waits-for-new-keys-001, and its
    brain decision, "Marley's terminal keys catch Zed's actions; the Marley keymap waits for keys
    with no Zed action": the Marley keymap takes keys with no Zed action behind them. Why
    `secondary-alt-o` (Open Recent) and Ctrl+O (Open Files) are not taken (D9).
  - L-claude-637-a-propagating-action-lets-the-keys-next-binding-run-001: a handler that propagates
    lets the key's next binding run. It would let `secondary-alt-o` be Rusty's only while Rusty is
    on; rejected for the two meanings (D9).
  - Completed pipeline 563 and F-claude-563-text-for-action-misses-a-terminal-binding-001: the
    shortcut note when a Marley key takes a terminal's keystroke, named through
    `bindings_for_action_in`. The new key takes Ctrl+Alt+U from a terminal and shows it (REQ-017).
  - AD-claude-454-the-rails-switcher-is-zeds-thread-switcher-over-the-rails-rows-001: Zed's
    switcher rules, the second entry selected, recency in memory. This picker selects the row after
    the active page the same way, and keeps recency in Zed's store instead (D4: no Page tab survives
    a restart to carry it).
  - AD-claude-453-the-rails-keys-are-zeds-list-actions-001 and L-claude-457: a list answers Zed's
    `menu` actions and binds no keys of its own; the `Picker` already does.
  - BF-symbol-picker-render-cap-diverges-from-nav-001: a cap on rows drawn must bound navigation
    too. The merged list is cut to 100 and `match_count` is its length (D3).
  - PR-claude-per-keystroke-refetch-needs-a-generation-001: a server re-query per keystroke needs a
    generation. Here nothing goes to Rusty per keystroke; the `Picker` drops the older match task.
  - PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001 and #645's D9: `confirm`
    runs inside the picker's update; the open goes through `open_later`, which defers itself, and
    the link's create acts in `window.defer` as #645's other link arms do.
  - L-claude-587-a-zed-toast-is-a-notification-under-its-id-001: each notice its own id (the
    refusal, the renamed slug, the off and not-connected toasts).
  - L-claude-574-an-item-a-private-module-shares-lives-in-a-public-module-001: `rusty::page_picker`
    is public so `rusty::page` can record an open past both visibility lints.
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001,
    L-claude-531 (stand-ins named, never first on the PATH) and L-claude-607 (settings edited from
    outside): the scenario sets `marley.rusty` and names its stand-in.
  - L-claude-540-a-queued-spec-is-redesigned-against-what-shipped-since-001: promotion re-reads
    #643 to #645 as they shipped (the Phase Plan's P1).
  - Completed pipelines 450 (`NewAgentPicker`), 563 (the note), 633 (a stand-in `rusty-mcp`), 637
    (key dispatch through propagation).
  - Brain (`rusty-cli brain search`, read only): "quick switcher" finds Rusty v3's project page
    (favourites starred in the Qt switcher, TICKET-013) and the keymap decision above; nothing on a
    Marley picker. Promotion asks the brain (`brain ask`) on D4 and D9; Complete records them.
- **Discovery** (read 2026-10-03; one Explore survey of Rusty, each claim used here re-read at the
  line; Zed and Marley read directly):
  - **Rusty's list** (`/srv/stacks/rusty-v3` at `295565c`): `brain_list_pages`
    (`crates/rusty-mcp/src/main.rs:921-931`, "List brain pages, newest first, optionally by type";
    `ListPagesParams { page_type, limit }` `:98-105`) answers `json_result` of
    `BrainManager::list_pages` (`crates/rusty-core/src/brain/mod.rs:573-626`): `SELECT slug,
    page_type, title, updated_at FROM brain_pages ... ORDER BY updated_at DESC LIMIT ?`, the limit
    50 when absent (`:579`), no upper bound. Each item is `BrainPageSummary` (`mod.rs:54-65`): four
    fields, no aliases. `updated_at` moves when `sync_page` sees a changed hash (`:1210-1238`) or a
    timeline entry is added (`:1405-1407`). A read, so no `list_changed`. SQLite only, no
    embedding. The Qt app asks `{ limit: 100000 }` (`rusty-app/qml/Main.qml:372`).
  - **Aliases:** stored (`frontmatter.rs:26-28`), indexed into `brain_aliases` (`engine/db.rs:
    254-260`, `mod.rs:2874-2888`), used by the link resolver (`resolve_on`, `mod.rs:3221-3261`) and
    by `brain_resolve_slug` (`main.rs:1651-1657`; `resolve_slug` `mod.rs:2275-2307`: a `LIKE
    %partial%` over aliases then titles, ten of each, slugs only). Returned by no list; only inside
    `brain_read_page`'s `frontmatter`.
  - **Rusty's create:** `brain_new_page` (`main.rs:1264-1276`, through `mutate`, so a success
    announces `list_changed`; `NewPageParams { folder: String (default), name: Option<String> }`
    `:593-601`) runs `new_page` (`mod.rs:2438-2474`): `..` refused ("Invalid folder"), a folder that
    is not there refused ("No folder {folder}", `:2446-2448`), the name trimmed, `.md` dropped and
    `/` turned into `-` (`:2449-2453`), a clash numbered `X 1`, `X 2` (`:2459-2464`), the type from
    the top folder, the template body, a commit `create: X`, the slug returned. `brain_new_folder`
    makes nested folders and refuses an existing one ("Already exists", `vault.rs:248-259`).
  - **Rusty's switcher** (`rusty-app/qml/QuickSwitcher.qml`): opened by Ctrl+O and Ctrl+T, both off
    while a terminal has the focus (`Main.qml:642-644`); filters the app's whole page list on the
    client (`Main.qml:744`); scores letters in order with bonuses, the best of title and slug plus
    one for a title match, 50 shown (`:31-63`); favourites first on an empty query, 30 shown
    (`:46-51`); Enter opens or, with no match, creates; Shift+Enter always creates (`:64-86`);
    `createPage` sends `folder: ""` (`Main.qml:506`), TICKET-041's bug. No recent list (only an
    in-memory `lastPageSlug`).
  - **Rusty's tickets:** TICKET-037 (bookmarks and favourites in a vault file; names Marley's
    switcher putting favourites first, `:22`), TICKET-040 (archived folders reindexed), TICKET-041
    (`open/TICKET-041-page-from-a-link-keeps-its-folder.md`: REQ-001 the slug equals the target,
    REQ-002 a missing folder made first, REQ-003 an existing slug returned, REQ-004 a plain name as
    today). TICKET-042 and TICKET-043 touch nothing here.
  - **Zed's picker** (`crates/picker/src/picker.rs`): `ConfirmInput` `:89-96` (bound to Alt+Enter,
    `default-linux.json:23`, `:1229`); `PickerDelegate` `:164-442` (`separators_after_indices`
    `:172`, `update_matches` `:226`, `confirm` `:255`, `confirm_input` `:293`, `dismissed` `:300`,
    `render_match` `:377`); `Picker::uniform_list` `:460`; `confirm` and `secondary_confirm`
    `:1034`, `:1050`; `update_matches` `:1230`. Marley already depends on it
    (`crates/marley_workbench/Cargo.toml:52`).
  - **Zed's file finder** (`crates/file_finder/src/file_finder.rs`): a second Toggle cycles
    (`:84-103`); `Match::CreateNew` `:433`, pushed when no entry is at the typed path and the query
    does not end in `/` (`:1235-1246`), sorted last (`:624-628`), labelled "Create File: …"
    (`:1339-1340`), opened through `open_path_preview` (`:1621-1636`); the active file first
    (`:631-639`) and skipped by the selection (`:1528-1545`); history on an empty query
    (`:1906-1932`); `allow_preview` from `enable_preview_from_file_finder` (`:1568`); its key
    context `FileFinder` (`:1547-1551`). Settings `assets/settings/default.json:1501` (preview from
    the finder, false) and `:1556` (`skip_focus_for_active_in_search`, true).
  - **Matching:** `fuzzy_nucleo::match_strings_async` (`crates/fuzzy_nucleo/src/strings.rs:100-108`:
    candidates, query, `Case`, `LengthPenalty`, max results, a cancel flag, the executor),
    `StringMatchCandidate::new(id, impl Into<SharedString>)` (`:27`), `StringMatch { candidate_id,
    score, positions, string }` (`:42-47`); `Query::build` splits the query on whitespace into a
    nucleo pattern (`fuzzy_nucleo.rs:67-87`); `Case::smart_if_uppercase_in` (`:22`), `LengthPenalty`
    (`:36-44`). Used by `file_finder`, `command_palette`, `tab_switcher`, `outline`,
    `recent_projects`. `fuzzy::match_strings` (`crates/fuzzy/src/strings.rs:117-126`) keeps the
    query's characters, spaces included (`:145-146`); Marley's `agents.rs:762`, `remote.rs` and
    `send_selection.rs` use it. `ui::HighlightedLabel::new(text, positions)`
    (`label/highlighted_label.rs:20`).
  - **The modal layer and actions:** `Workspace::toggle_modal` `workspace.rs:8614`,
    `active_modal` `:8603`; `ModalLayer::toggle_modal` `modal_layer.rs:152-172` (a modal of the same
    type closes and returns); `ModalView` `:49`. gpui's `available_actions`
    (`key_dispatch.rs:363-380`) builds each listener's action by `build_action_type`
    (`action.rs:336-343`), which passes `{}` (`:351-368`): an action with a required field is never
    available to the palette, one whose fields all default is.
  - **Key dispatch:** `Keymap::bindings_for_input` (`crates/gpui/src/keymap.rs:165-243`) sorts by
    depth, then by load order; `binding_enabled` gives a binding with no context the deepest depth
    (`:246-252`); `Window::dispatch_key_event` tries each matched binding until one is handled
    (`window.rs:5946-5958`).
  - **Keymaps:** `assets/keymaps/default-linux.json`: `ctrl-o` `workspace::OpenFiles` with no
    context (`:28`); `alt-ctrl-o` and `ctrl-r` `projects::OpenRecent` in `Workspace` (`:635`,
    `:648-649`); `alt-ctrl-shift-o` `projects::OpenRemote` (`:653`); `ctrl-shift-o`
    `outline::Toggle` in `Editor && mode == full` (`:628-630`); `ctrl-alt-p`
    `agent::ManageProfiles` in `AcpThread` (`:244`, `:249`); `ctrl-alt-shift-o` and
    `ctrl-alt-shift-p`, the frame overlay's keys (`:914-918`); `ctrl-q` `zed::Quit` (`:35`); the
    Terminal block passing `ctrl-o`, `ctrl-q`, `ctrl-r` to the program (`:1295`, `:1313-1316`).
    `default-macos.json`: `alt-cmd-o` Open Recent (`:710`), `cmd-alt-p` (`:289`), `cmd-alt-m`
    (`:260`), `alt-cmd-w` (`:320`, `:563`). `specific-overrides.json:17-19` (`ctrl-alt-p`
    `picker::TogglePreview` in `(Picker && with_preview) > Editor`). Base keymaps: JetBrains binds
    `ctrl-alt-o` (`linux/jetbrains.json:44`), TextMate binds `ctrl-alt-u` on macOS
    (`macos/textmate.json:43`); nothing else in `assets/keymaps/` binds `ctrl-alt-u`, `alt-ctrl-u`
    or `cmd-alt-u`. Marley's keymap (`crates/marley_workbench/keymap.json:7-15`: `secondary-alt-n`;
    #644 adds `secondary-alt-v`) binds neither. Omarchy's Hyprland bindings
    (`~/.local/share/omarchy/default/hypr/bindings/tiling.lua:2, 49-50`) use Ctrl+Alt with Delete
    and Tab only; Chad's `base_keymap` is `Zed` and he has no keymap file of his own.
  - **Marley:** `agents.rs:575-600` (`new_agent`: the note from a terminal, `toggle_modal`),
    `:605-661` (`NewAgentPicker`: `Picker::uniform_list`, key context, `rems(34.)`), `:734-772`
    (`update_matches` in `spawn_in`, `log_err` on the update). `shortcut_note.rs:1-80` (`taken`,
    scope `marley-shortcut-note`, one row per action name in Zed's key-value store).
    `db/src/kvp.rs:31` (`scoped_kv_store`), `:89-118` (`scoped`, `read`, `write`). `launch.rs`,
    `terminal_size.rs` and `groups.rs` use the same store.
  - **The harness:** `script/e2e.sh:623-645`: the run's profile, the user's settings and database
    (`cp -r "$data/db"`, `:643`) copied into it, Marley started with `--user-data-dir` (`:301`,
    `:304`), the profile removed at the end; `profile_setting` (`:558-578`).
  - **The batch's drafts:** #643's `Rusty` global (state `Off`, `Starting`, `Connected`, `Down`,
    `Missing`) and `call` shaped like `harness::call` (its notes, Design); #644's stand-in with
    `brain_new_page` and Rusty's messages, its stop file and pid file, and its off toast (its D4,
    D10); #645's `OpenPage { slug: String, preview }` (required slug, so not in the palette, its
    Scope), `open_later`, `PageView`'s `load` and `follow`, `PageLink::Missing { target }`,
    `split_fragment`, `normalise_target` (its notes, Design) and its REQ-013.
  - **Ely** (HEAD `2f8b2f6`, re-cloned into the scratchpad's `repos/ely` for this draft):
    `src/navigation/palette/mod.rs` (`fuzzy` `:38-88`, its `expect` `:34`, `Palette` `:140-169`,
    rows as children `:236-311`, the list `:366-377`, the entrance animation `:391-395`, the pick
    `:221-231`) and `kinds.rs` (`ranked` `:14-29`, `CommandPalette`'s `panic!` `:137`, `SHOWN` 50
    `:196-197`, `QuickOpen` `:199-305`, `QuickSwitcher` `:307-386` with its start on the second row
    `:358-365` and its `assert!` `:324`, `SearchPalette` `:388-487` with its `assert!` `:448` and
    `Highlight::matching` `:458`). `LICENSE-MIT`: "Copyright (c) 2026 Ely GPUI Component
    contributors". Nothing ported.
- **Decisions:** D1 to D11 in the spec. In short: Zed's `Picker` in the modal layer; one
  `brain_list_pages` read per open, matched in Marley on title and slug; Zed's `fuzzy_nucleo`; the
  recently opened kept by Marley in Zed's key-value store; Enter opens a kept tab through #645's
  opener; a create row last, the path split into `folder` and `name`; an unresolved link makes its
  page the same way; the action's slug optional so the palette lists it; `secondary-alt-u`;
  scenarios on a stand-in with the recent list cleared; no Zed crate changed.

### Design
- **Approach.**
  - `marley_rusty::page` (#645's module): `NewPage { folder: String, name: String }` and
    `NewPage::from_target(target) -> Option<NewPage>`: `split_fragment` off, `normalise_target`,
    `None` for an empty path or one ending in `/` or holding `..` as a part (Rusty refuses it
    anyway; Marley never sends it), else the split at the last `/`.
  - `marley_rusty::switcher` (new, pure, no gpui):
    - `PageSummary { slug, title, page_type, updated_at }` (serde, unknown fields ignored) and
      `parse_page_list(text) -> Result<Vec<PageSummary>>` over the answer's text block.
    - `RecentPages`: `visit(slug)` (to the front, once, at most 20), `from_json`, `to_json`,
      `listed(&pages) -> Vec<usize>` (indexes into the list, in recent order, slugs the list holds).
    - `empty_order(pages, recent, active) -> Order { rows, separator_after, selected }`: the recent
      group (the active page first when it is in it), then the rest in the list's order; selected 1
      when the first row is the active page and there is a second, else 0.
    - `merge(title_hits, slug_hits, recent, cap) -> Vec<Hit>`: plain `(page, score, positions)`
      inputs from the two match calls; one `Hit { page, score, title_positions, slug_positions }`
      per page at its better score, the other field's positions kept only when that field scored
      the same; sorted by score, then recent rank, then list order; cut to `cap`.
    - `create_target(query, pages) -> Option<NewPage>`: `NewPage::from_target(query)`, `None` when
      the normalised path is a listed slug.
  - The stand-in (`crates/marley_rusty/stand_in/rusty-mcp`, #643's, with #644's tools):
    `brain_list_pages` walks the scratch vault as #644's `brain_tree` does (dot entries skipped),
    reads each page's frontmatter for `title` and `type`, sorts by mtime newest first, honours
    `limit` (50 when absent, as Rusty) and `page_type`, logs the call.
  - `marley_workbench::rusty` (#643's adapter, #645's actions): `OpenPage`'s `slug` becomes
    `Option<String>` with `#[serde(default)]` and its doc comment says that without a slug it opens
    the picker; the handler: a slug goes to #645's path unchanged; none goes to
    `page_picker::toggle(workspace, window, cx)`. `pub mod page_picker;`.
  - `marley_workbench::rusty::page_picker` (new):
    - `toggle`: the switch (`MarleySettings::rusty`): off, #644's toast; #643's state not
      `Connected`, the toast "Marley is not connected to Rusty: REASON"; else, when a terminal has
      the focus, `shortcut_note::taken(&OpenPage::default(), "opened Rusty's page picker", ..)`,
      then `workspace.toggle_modal(.., |window, cx| PagePicker::new(..))`.
    - `PagePicker { picker: Entity<Picker<PagePickerDelegate>> }`: `ModalView`,
      `EventEmitter<DismissEvent>`, `Focusable`, `Render` (`v_flex().key_context("RustyPagePicker")
      .w(rems(34.))`), as `NewAgentPicker`.
    - `PagePickerDelegate { modal, workspace, active: Option<String>, list: List, recent:
      Vec<String>, hits: Vec<Row>, selected, creating: bool }` where `List` is `Reading |
      Read(Arc<[PageSummary]>) | Failed(SharedString)` and `Row` is `Page(Hit) | Create(NewPage)`.
      `new` reads the active item's slug (a `PageView`'s current page) and the recent global, and
      spawns the `brain_list_pages { limit: 100000 }` call through #643's `call` within 5 s; its
      answer sets `list` and calls `picker.refresh(window, cx)` so the typed query is matched.
    - `update_matches(query)`: nothing until `Read`; an empty query takes `empty_order`; else two
      `fuzzy_nucleo::match_strings_async` calls (titles, slugs; 100 each) in `spawn_in`, `merge`,
      then the create row from `create_target` appended; the update sets `hits` and `selected`
      (`log_err` on a dropped picker).
    - `separators_after_indices`: `separator_after` on an empty query.
    - `no_matches_text`: "Reading the brain…", the failure's first line, or "No pages".
    - `render_match`: `ListItem` with `Icon::new(IconName::FileMarkdown)` muted, the title as a
      `HighlightedLabel`, the slug as a muted small `HighlightedLabel`; the create row with
      `IconName::Plus`, "Create page:" and the path, or "Creating…" while `creating`.
    - `confirm(_secondary)`: a page row: emit `DismissEvent`, then `page::open_later(workspace,
      slug, false, window, cx)`. The create row, unless `creating`: set `creating`, call
      `page::create(..)`; on `Ok(slug)` dismiss, `open_later(slug, false)`, and the renamed toast
      when `slug` is not the path; on `Err(message)` clear `creating` and show the refusal toast
      (`NotificationId::composite::<PagePicker>("refused")`), the picker kept.
    - `dismissed`: emit `DismissEvent` on the modal, as `NewAgentDelegate` does.
    - `RecentPages` as a gpui global (`Global`), loaded at `rusty::init` from
      `KeyValueStore::global(cx).scoped("marley-rusty-recent-pages").read("pages")`, and
      `pub fn opened(slug, cx)`: `visit`, then a background `write("pages", to_json)` with
      `log_err`.
  - `marley_workbench::rusty::page` (#645's): `open_later` calls `page_picker::opened(slug, cx)`;
    `PageView`'s navigation by a link, Back or Forward calls it with the page it moves to (not the
    live re-read). `pub(crate) async fn create(client, NewPage) -> Result<String, String>`:
    `brain_new_page { folder, name }` through #643's `call` within 5 s, the slug parsed from the
    answer, a refusal's first line as the error. `follow`'s `Missing { target }` arm, in
    `window.defer`: `NewPage::from_target(target)`, then `create`; `Ok(slug)` navigates the tab to
    it (`history.visit`, `load`); `Err` (or no `NewPage`) shows the refusal toast and changes
    nothing.
  - `crates/marley_workbench/keymap.json`: `"secondary-alt-u": "rusty::OpenPage"` in the
    `Workspace` block, with a comment naming the sweep and that Ctrl+O stays Zed's and the
    terminal's.
  - `crates/marley_workbench/guide/index.html`: the Brain article gains the picker's line (the key,
    recent first, create on a miss and the folder rule, links that create).
- **File manifest.**
  - Marley: `crates/marley_rusty/src/switcher.rs` (new), `crates/marley_rusty/src/page.rs`
    (`NewPage`), `crates/marley_rusty/src/marley_rusty.rs` (the module),
    `crates/marley_rusty/stand_in/rusty-mcp` (`brain_list_pages`);
    `crates/marley_workbench/src/rusty.rs` (`OpenPage`'s field, the handler, the module, the
    global's load), `crates/marley_workbench/src/rusty/page_picker.rs` (new),
    `crates/marley_workbench/src/rusty/page.rs` (the two `opened` calls, `create`, the `Missing`
    arm), `crates/marley_workbench/Cargo.toml` (`fuzzy_nucleo.workspace = true`; `picker`, `fuzzy`,
    `db`, `menu` and `ui` are there already), `crates/marley_workbench/keymap.json`,
    `crates/marley_workbench/guide/index.html`; `script/e2e/654-rusty-open-page-picker.sh` (new,
    Test phase).
  - Zed: none. `picker`, `fuzzy_nucleo`, `workspace`'s modal layer and toasts, `db::kvp` and `ui`
    are used as they are. `"rusty"` in `crates/zed/src/zed.rs`'s `test_action_namespaces` is #645's
    line; this ticket adds no action namespace.
- **Ledger rows:** none in `docs/marley/zed-touchpoints.md` (no file outside the Marley-owned paths
  changes). `.config/spawn-sites.txt` unchanged: the calls ride #643's client.

### Visual check plan
The scenario `script/e2e/654-rusty-open-page-picker.sh`, `compositor sway`. Setup: a scratch
repository opened with `open_path`; a terminal HOME whose `.bashrc` sets a plain prompt; the vault
under `$E2E_WORK/vault` with `.git` initialised and the seven pages of the spec's UI proof, their
mtimes set with `touch -d` in the order listed; #643's stand-in linked as `$E2E_WORK/bin/rusty-mcp`,
named in `MARLEY_RUSTY_MCP` and pointed at the vault, its log and pid files in `$E2E_WORK/rusty`;
`profile_setting` for `marley.rusty.enabled` true and `marley.rusty.connection` `embedded`; a
Python `sqlite3` step that deletes the scope `marley-rusty-recent-pages` and the row
`rusty::OpenPage` of the scope `marley-shortcut-note` from every `db.sqlite` under
`$E2E_PROFILE/db`. Keys go through
`wtype`; the link clicks are measured from the shot before them, as #645's scenario does. The
center terminal has the focus at the start.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, 002, 016, 017 | Ctrl+Alt+U with the terminal focused; settle | `654-01-listed`: seven rows in mtime order, title and slug; the note naming Ctrl+Alt+U |
| REQ-003, 010 | Type "quart rev" | `654-02-by-title`: Quarterly review first, title lit; the create row last |
| REQ-004 | Select all, type "peop sam" | `654-03-by-slug`: Sam, `people/sam` lit |
| REQ-011 | Select all, type "people/sam" | `654-04-exact`: Sam alone |
| REQ-005 | Select all, "quart", Enter | `654-05-opened`: no picker; a kept, focused Quarterly review tab |
| REQ-006, 007, 016 | Ctrl+Alt+U, "sam", Enter; Ctrl+Alt+U, "atlas", Enter; Ctrl+Alt+U | `654-06-recent`: Atlas, Sam, Quarterly review, separator, the other four; Sam selected |
| REQ-008 | Enter | `654-07-back`: Sam in front; three tabs |
| REQ-010 | Ctrl+Alt+U, "projects/marley/review" | `654-08-create-row`: the create row alone, selected |
| REQ-012 | Enter; settle | `654-09-created`: a kept "review" tab; the log's `brain_new_page` arguments |
| REQ-013 | Ctrl+Alt+U, "drafts/soon", Enter; settle | `654-10-refused`: the toast "No folder drafts"; the query kept |
| REQ-014 | Escape; click Atlas's tab; click `ideas/later` | `654-11-link-created`: "later" in the tab, Back enabled; the log's arguments |
| REQ-014 | Alt-Left | `654-12-link-resolved`: Atlas; `ideas/later` in the link colour |
| REQ-015 | Click `drafts/soon` | `654-13-link-refused`: the toast; Atlas shown |
| REQ-018 | Ctrl+Shift+P, "open page" | `654-14-palette`: `rusty: open page` with its key |
| REQ-018 | Enter | `654-15-from-palette`: the picker |
| REQ-019 | Escape; the stand-in's stop file; kill its pid; settle; Ctrl+Alt+U | `654-16-not-connected`: the toast with the reason; no picker |
| REQ-020 | `marley.rusty.enabled` false from outside; settle; Ctrl+Alt+U | `654-17-off`: "Rusty is off…"; no picker |
| REQ-009, 021, 022, 023 | Not driven | Review of the diff |

The scenario checks the stand-in's log after `654-09` and `654-11` with `grep`: one `brain_new_page`
each, the `folder` and `name` as listed, and no `name` holding `/`. Not reached by a scenario: a
restart with the recent list kept (the runner starts Marley once), a failed list read, and Rusty
numbering a clash (the stand-in lists every page on disk, so no clash reaches it); each is read in
the review. Never the user's brain (R-D8). If the note's toast covers a row in `654-01`, the shot
waits for it to settle and the list's order is read from the rows above it.

### Risks
- **Shapes this ticket does not own.** #643's `call` and state, #644's stand-in and toast, and
  #645's action, opener, `PageView` and link parsing are drafts beside this one. Promotion re-reads
  each as shipped (L-540); the decisions do not depend on their exact names.
- **Rusty's tickets landing first.** If TICKET-041 lands, the split stays valid (its REQ-004 keeps
  `folder` and `name`), a missing folder is made by Rusty, and the refusal shot changes; if
  TICKET-037 lands, favourites belong first and Out's follow-up may fold in.
- **A key with no meaning in its letter.** `secondary-alt-u` is the free chord, not a mnemonic one;
  the palette shows it beside `rusty: open page` and the user's keymap wins over Marley's. Chad
  confirms it at promotion.
- **Private slugs in a local store.** The recent list holds slugs of the user's brain in Zed's
  database on this machine; nothing sends it anywhere. The e2e copy holds them until the run ends,
  and the scenario clears them before Marley starts.
- **Accidental pages.** A click on an unresolved link makes a page and a commit in the vault's git,
  as Obsidian and Rusty's app do; the Brain view's Delete moves it to `archive/`.
- **Two parsers of a path.** Marley's normalising copies Rusty's (`clean_rel`, `normalise_target`);
  if they differ, Rusty's answer wins and the renamed toast says what it made.
- **Archived pages** may be listed until TICKET-040; a stale index after an outside edit shows the
  old title until Rusty's indexer runs (about 5 s).
- **The list's size.** 814 pages today, one JSON answer of about 100 KB, matched on the background
  executor; a vault ten times larger stays well inside one read.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: the Brain section gains the picker (the key, the palette entry, recent
  first, the create row and the folder rule, links that create), and the keys table its row.
- `docs/marley/walkthrough.md`: a stop after #645's Page tab stop, following this ticket's shots.
- `crates/marley_workbench/guide/index.html`: changed in the Code phase (above).
- Architecture (§21): `docs/marley_architecture/marley_workbench.md` (the picker, the recent list
  and its store), `marley_rusty`'s page (`switcher`, `NewPage`), `docs/marley/rusty-in-marley.md`
  (R-D3's QuickSwitcher row: recently opened first until TICKET-037; R3a's status; the aliases
  request beside RQ1 to RQ5), `CHANGELOG.md` (Added; Changed: an unresolved link makes its page).

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20, the ticket and pipeline templates, and 633's
      completed spec for the voice.
- [x] Read the Rusty-in-Marley plan in full (R-D0 to R-D10, the slices, Rusty's triage) and the
      brief's three parts.
- [x] Read the queued #643, #644 and #645 specs and notes, and #646's and #647's lines on the
      picker and the opener; took their names.
- [x] Recall: the knowledge ledgers (AD-449, AD-450, AD-453, AD-454, L-637, F-563, the
      symbol picker's render cap, PR per-keystroke, PR callback mid-update, L-587, L-574, L-633,
      L-531, L-607, L-540), the completed pipelines 450, 563, 633 and 637, a read-only brain
      search.
- [x] Discovery with file:line: Rusty's `brain_list_pages`, aliases, `brain_new_page`,
      `QuickSwitcher.qml` and tickets 037, 040, 041 (an Explore survey, re-read at the lines used);
      Zed's `picker`, `file_finder`, `fuzzy_nucleo`, `fuzzy`, the modal layer, action building and
      key dispatch; every keymap Zed ships, Marley's, Omarchy's Hyprland bindings; Marley's
      `NewAgentPicker`, shortcut note and key-value store; the e2e runner's database copy.
- [x] Prior-art sweep, three legs; Ely's `SearchPalette`, `QuickOpen` and `QuickSwitcher` read
      (re-cloned at `2f8b2f6`), with why nothing is ported; Obsidian's Quick switcher page read.
- [x] Aliases checked: Rusty does not serve them in any list; Out with a Rusty-side request.
- [x] The key decided and locked (D9), with the sweep and each rejected chord's reason.
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D11, 23 EARS rows, phase plan.
- [x] Design: approach, file manifest by crate, no touchpoint row, the visual check plan, risks,
      the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Reconciliation, 2026-10-03
- A `rusty:` action run while Rusty is off or not connected shows a toast saying so and where to
  turn it on, and opens nothing (rusty-in-marley.md R-D0, settled across #643 to #659).
- Every scenario names its stand-in in `MARLEY_RUSTY_MCP`, never first on the PATH (#643).
