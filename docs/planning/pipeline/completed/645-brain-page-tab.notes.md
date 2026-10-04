# A brain page in a center tab: Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-645-brain-page-tab.md
- **Pipeline spec:** 645-brain-page-tab.spec.md

## Phase 1: Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, on Rusty in Marley: "maybe rusty becomes Marley. We would take our
  rusty custom QML app and build it inside of marley", then "Plan it now", and "Switch in the rail
  header" for how the screens are reached (R-D9). On 2026-10-03: "lets make a plan to begin the work
  and spec out the tickets" and "lets make sure we use the gpui components we found here",
  confirmed as "Queue all five". This is the third of the five: #643 the switch and connection,
  #644 the rail's Brain view, #645 the Page tab, #646 the Knowledge panel, #647 the Graph tab. Plan:
  `docs/marley/rusty-in-marley.md`, R-D3's NoteTab row, R-D4 and the slices table's R2 (whose
  backlinks and outgoing half moved to #646).
- **Classification / tier:** feature, medium. One new view and its opener in `marley_workbench`, a
  pure module in `marley_rusty`, a small Ely port, a keymap block, additions to #643's stand-in.
  One Zed touch, a line in a test list, extending an existing `zed-touchpoints.md` row. No new
  dependency: `pulldown-cmark` is already a workspace dependency (`Cargo.toml:817`, 0.13.4).
- **Recall (§18.3):**
  - AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001: an item opened by
    an opener that finds it again, every open through `Window::defer`, not a `SerializableItem`.
    This tab follows all three.
  - F-claude-599-a-page-that-moved-to-its-own-anchor-got-a-second-tab-001 and PR-claude-599: a tab
    that navigates is found by what it shows now. The opener matches a Page tab's current slug.
  - F-claude-503-a-tab-opened-inside-a-terminal-views-event-would-update-the-view-again-001: a pane
    change from inside an item's own event panics. Here the link click arrives inside the `Markdown`
    entity's update, which Zed's own preview defers out of (D9).
  - AD-claude-530-runbook-commands-go-to-the-last-terminal-at-its-prompt-001: Marley's one seam in
    Zed's `markdown` crate so far, a generic hook only the preview passes. This ticket needs no new
    seam (D1).
  - AD-claude-490-the-browser-tab-draws-its-own-chrome-and-dialogs-001: a Marley tab draws its own
    header with Back and Forward; the Page tab does too, with the Browser tab's keys.
  - L-claude-613-zeds-added-to-workspace-is-the-hook-for-an-item-that-changes-workspace-001: an item
    holding a workspace handle re-points it in `added_to_workspace`.
  - L-claude-574-an-item-a-private-module-shares-lives-in-a-public-module-001: `rusty::page` is a
    public module so #644's `open_page` reaches `open_later` past both visibility lints.
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001 and
    L-claude-607: the scenario sets `marley.rusty` and `preview_tabs.enabled` itself and edits the
    run's settings only from outside.
  - Completed pipelines 530 (the `markdown` and `markdown_preview` seams, read line by line), 565
    (`DecisionsView`), 609 (`AgentView`), 633 (a stand-in `rusty-mcp` on the PATH).
  - Brain (`rusty-cli brain search`, read only): two decisions,
    `decisions/marley-draws-rustys-knowledge-workspace-rusty-keeps-its-data-marley-d11-amended` and
    `decisions/rusty-files-marleys-rq1-rq5-as-ticket-035-to-039-and-m10-the-qt-app-frozen`; nothing
    on rendering. Promotion asks the brain (`brain ask`) the rendering question; Complete records
    the decision.
- **Discovery** (read 2026-10-03; three Explore surveys for the preview tabs, Zed's markdown crates
  and Rusty's tools, each claim re-read at the line):
  - **Rusty's `brain_render`** (`rusty-v3/crates/rusty-mcp/src/main.rs:1224-1252`, params
    `:570-583`): one call returns `RenderedPage` (`rusty-core/src/brain/mod.rs:177-193`) with
    `Rendered` flattened in (`render.rs:194-210`): `slug`, `title`, `page_type`, `properties`
    (`[{key, value}]` in file order), `raw` (the whole file, read from disk by `render_page`,
    `mod.rs:2369-2392`), `html`, `outline`, `links` (`[{target, slug, alias, embed}]`,
    `render.rs:181-191`, `slug` null when unresolved), `unresolved` (distinct targets), `tasks` (a
    count), `words`, `characters`. A missing slug answers JSON `null`. `links[].alias` is null for
    an ordinary `[[x|alias]]` (`render.rs:906`): the pass takes the text itself.
  - **Rusty's renderer:** pulldown-cmark 0.13.4 with tables, footnotes, strikethrough, task lists,
    heading attributes, YAML metadata blocks, wikilinks and math (`render.rs:563-570`); a wikilink's
    href is `rusty:page/SLUG#FRAG` or `rusty:new/TARGET` (`:894-926`); a plain local link resolves
    to a page (`:927-937`); `split_fragment` cuts at the first `#` or `^` (`:1209-1218`);
    `normalise_target` trims, drops a leading `/` and `./` and a `.md` suffix (`links.rs:159-163`).
    The frontmatter split is `frontmatter.rs:85-106` (`split_raw`).
  - **The vault path:** `<root>/<slug>.md`, refusing `..` (`vault.rs:446-455`); the root is the
    `brain_vault_path` setting or `~/.rusty/brain` (`core.rs:73-81`), and `setting_get` answers null
    while unset (`main.rs:1842-1848`): no tool names the default root.
  - **What happens to a disk edit:** `start_data_watcher` (`rusty-core/src/lib.rs:20-62`) debounces
    600 ms and emits `DataChanged`; `spawn_change_notifier` (`main.rs:2062-2082`) sends
    `resources/list_changed` to each client; `spawn_indexer` (`main.rs:2008-2058`) runs `sync_all`
    five seconds after a change. `sync_page` and `sync_all` (`mod.rs:1198-1288`) write no commit;
    `git_commit` is `git add -A` and a commit (`vault.rs:113-136`), run only by tool writes, so an
    outside edit is committed by the next tool write under its message. `write_raw`
    (`brain_write_page`, `mod.rs:2414-2436`) is a no-op for unchanged text.
  - **Rusty's page today** (`rusty-app/qml/NoteTab.qml`): `open`, `goBack`, `goForward` over a
    per-tab history, saving first and leaving edit mode (`:168-184`); Back and Forward buttons
    (`:381-382`), folder and name (`:384-387`), the favourites star, the READ, LIVE and EDIT toggle
    and the menu (`:390-422`); link handling by href scheme (`:242-258`), where `rusty:page/` drops
    the heading and `rusty:new/` calls `brain_new_page {folder: ""}`, which flattens the path
    (`mod.rs:2441-2474`); task boxes toggled by a regex over `raw` and saved (`:261-282`); the
    editable title (`:517-531`) and properties (`:537-576`). `rusty-app/src/markdown.rs` holds only
    the source editor's line tokenizer and `page_sections`; every rendering comes from
    `brain_render`.
  - **Zed's preview tabs** (`crates/workspace/src/pane.rs`): `preview_item_id` `:994`,
    `unpreview_item_if_preview` `:1020`, `replace_preview_item_id` `:1031` (closes the current
    preview, marks the new id; ignored while `preview_tabs.enabled` is off), `set_preview_item_id`
    `:1046` (`pub(crate)`), `handle_item_edit` `:1062`, `add_item` `:1347`, the tab's double click
    `:2940-2941`, `render_tab` passing `TabContentParams { preview }` `:2854-2873`,
    `NavigationMode::ClosingItem if is_preview => return` `:4938`. `item.rs`: `PreviewTabsSettings`
    `:66-75`, `TabContentParams.preview` `:133`, the default `tab_content` ignores it `:177-184`,
    `to_item_events` `:214`, `preserve_preview` `:378`, `ItemEvent::Edit` reaching
    `handle_item_edit` `:936-957`. The editor's italic `editor/src/items.rs:827`. The project
    panel's click `project_panel.rs:6396-6402` (click count 1 previews, more keeps and focuses).
    `Workspace::open_project_item` `workspace.rs:5458`, its two-step add `:5524-5541`;
    `items_of_type` `:4383`, `activate_item` `:5646`, `pane_for` `:6460`. Settings
    `default.json:1491-1511`.
  - **Zed's `markdown` crate:** `PARSE_OPTIONS` `parser.rs:13-23`; `CONDITIONAL_OPTIONS` and
    `UNWANTED_OPTIONS` with `ENABLE_WIKILINKS` `:984-987`, held by `all_options_considered`
    `:989-999`. `markdown.rs`: `LinkStyleCallback` `:65`, `MarkdownStyle` `:106-134`
    (`link_callback` `:115`, `themed` `:175`), `MarkdownOptions` `:513-520`,
    `Markdown::new_with_options` `:664`, `scroll_to_heading_when_parsed` `:947`, `scroll_to_heading`
    `:955`, `replace` `:992`, `MarkdownElement::new` `:1739`, `on_url_click` `:1803`, the link style
    taken from `link_callback` `:2927-2936`, the click handed to `on_url_click` inside the entity's
    mouse handler `:2469-2477`, task boxes `:2876-2908`, tables `:2984-3074`, code blocks
    `:2744-2856`, selection `:2352-2391`.
  - **Zed's `markdown_preview`:** the element's options and handlers
    `markdown_preview_view.rs:1069-1193`; `handle_url_click` `:1240-1291`, deferring before it
    updates the `Markdown` entity `:1254-1257`; save and dirty passed to the editor `:1699-1743`;
    `#530`'s `MarleyCodeBlockAction` `:66-80`. `git_ui/src/project_diff.rs:425, 503-536` passes the
    same to its editor. `Project::open_local_buffer` `project/src/project.rs:3179`;
    `Editor::for_buffer` `editor/src/editor.rs:1859`. `SaveOptions` `workspace/src/item.rs:40-44`.
  - **pulldown-cmark 0.13.4:** `LinkType::WikiLink { has_pothole }` `lib.rs:498-504`,
    `ENABLE_WIKILINKS` `:735`; `[[foo]]` comes as a `Link` with `dest_url` `foo` and the text `foo`,
    `[[bar|baz]]` with the text `baz` (`parse.rs:2879-2896`).
  - **Marley:** `decisions.rs:51-60` (opener) and `:414-420` (`Item`); `agent_tab.rs:29-61`
    (`open_later` through `window.defer`, `open`) and `:657-682`; `rusty.rs:1-40` (#633's offer,
    which #643 grows into the client); the `marley` actions `marley_workbench.rs:117`; Marley's
    keymap `crates/marley_workbench/keymap.json:133-147` (the Browser tab's Alt-Left and Alt-Right);
    `markdown_commands.rs` (#530). Zed's namespace test lists `"marley"` at `zed.rs:5954`
    (`test_action_namespaces` `:5868`). `ui` has `Chip` (`chip.rs`) and no key and value list.
    Icons: `FileMarkdown`, `ArrowLeft`, `ArrowRight`, `Pencil` (`icons/src/icons.rs`).
  - **The batch's drafts** (queued beside this one, 2026-10-03): #643's D3, D4, D5, D8 and D9 (the
    client, `MARLEY_RUSTY_MCP`, `list_changed` on the embedded connection only, the Python stand-in
    in `crates/marley_rusty/stand_in/`, the harness turning Rusty off in each run's copy); #644's
    D1 and D6 (`rusty::brain::open_page(slug, preview)`, the file in Zed's editor until #645, the
    project panel's click rule) and its stand-in additions (`setting_get brain_vault_path`,
    `list_changed` on `SIGUSR1`); #646's REQ-008 and #647's D12 call `rusty::page::open_later`;
    #646's Out list and D4 (see Risks).
  - **Zed's Linux keys:** `pane::GoBack` on `ctrl-alt--` and the mouse's back button
    (`default-linux.json:511-512`); Ctrl-E is `file_finder::Toggle` (`:704`). Markdown files may be
    formatted by prettier on save (`default.json:2654-2665`).
  - **The live vault** (read only, counts): 814 pages, 804 with frontmatter, 4514 wikilinks (1964
    piped, 182 with a heading), 31 pages with tables, 26 with code fences, 10 with tasks, 108 with a
    `## Timeline`, 4 local `.md` links; no embed, callout, image, cover or icon; one page with
    `%%`. Frontmatter keys: `title`, `type`, `created`, `updated` on all, then `status`, `tags`,
    `summary`, `question`, `decided`, `consulted` (a list of slugs), `url`, `projects` and others.
  - **Ely** (HEAD 2f8b2f6): `src/documents/render.rs:1-439` (`MarkdownRenderer`),
    `src/documents/tree.rs:1-262` (its parse, with `expect` and `unreachable!`),
    `src/documents/markdown.rs:10-16` (its options, no wikilinks), `src/navigation/toc.rs:13-249`
    (`Sections`, `Anchor`, `TableOfContents`, over `crate::motion`), `src/documents/pages.rs:19-183`
    (`PageCover`, `PageIcon`), `src/data_display/records.rs:12-86` (`DescriptionList`) and
    `:117-222` (`PropertyGrid`); the gallery's stories
    `examples/gallery/pages/documents/reading.rs:144-167` and `knowledge.rs:278-305`. `LICENSE-MIT`:
    "Copyright (c) 2026 Ely GPUI Component contributors".
- **Decisions:** D1 to D10 in the spec. In short: Marley's pass rewrites wikilinks into links with
  Rusty's own addresses and Zed's `markdown` crate draws the page unchanged; links resolve by
  normalised target; `rusty::OpenPage` lands here and its picker is the second slice; Zed's own
  preview-tab API; the tab's own history; Edit is a Zed editor in the tab that saves before it
  leaves; read-only properties in Ely's `DescriptionList` layout; live by re-reading; every open and
  link deferred; no restore after a restart.

### Design
- **Approach.**
  - `marley_rusty::page` (pure, no gpui):
    - `RenderedPage` and `Property`, the typed view of `brain_render` (serde, unknown fields
      ignored), unless #643's typed views already hold it; `None` for a `null` answer.
    - `body_of(raw) -> &str`: Rusty's `split_raw` rule; without frontmatter, the whole text.
    - `split_fragment` and `normalise_target`, after Rusty's (MIT, the same owner).
    - `page_markdown(body, links) -> String`: one pulldown-cmark pass with Rusty's options over the
      body with `into_offset_iter`; for each `Tag::Link` or `Tag::Image` whose link type is
      `WikiLink`, the text inside it (an alias when piped), the target split and normalised, the
      slug from the first `links` entry with that target (the page's own slug for an empty one),
      and a replacement `[TEXT](<rusty:page/SLUG#HEADING>)` or `[TEXT](<rusty:new/TARGET>)` with
      the text's Markdown punctuation backslash-escaped and `%`, `<` and `>` in the address
      percent-encoded. Replacements are spliced back to front so earlier ranges stay valid. Text
      outside wikilinks is left byte for byte.
    - `PageLink::parse(address) -> PageLink`: `Page { slug, heading }`, `Missing { target }`,
      `Heading(name)`, `Local { target }` (no scheme, not `#`, not `mailto:`), `External(address)`,
      decoding what the pass encoded.
    - `PageHistory`: the entries behind, the current one (slug and heading) and the entries ahead;
      `visit` drops what was ahead, `back` and `forward` move, `can_back` and `can_forward`; capped
      at 100 behind.
    - `page_file_in(root, slug) -> Option<PathBuf>`: `None` for a slug holding `..` or one that
      would leave the root, Rusty's own rule.
  - The stand-in (`crates/marley_rusty/stand_in/rusty-mcp`, #643's Python program on the standard
    library, with #644's vault tools): `brain_render` reads `<vault>/<slug>.md` on each call and
    answers `slug`, `title` and `page_type` (the frontmatter's, else the file name and the top
    folder), `properties` (the frontmatter's keys in order, scalars and lists, which is all the
    fixtures hold), `raw`, `links` (every `[[...]]` and `![[...]]`, split and normalised as above,
    resolved when `<vault>/<target>.md` exists), `unresolved`, and `null` for a missing page. It
    logs the call. `setting_get brain_vault_path` and `list_changed` on `SIGUSR1` are #644's. The
    three pages go into `fixtures/` beside #644's vault, or the scenario writes them.
  - `marley_workbench::rusty` (a public module): `OpenPage { slug: String, #[serde(default)]
    preview: bool }` (`#[derive(Action)]`, `#[action(namespace = rusty)]`, doc comments, as the
    palette shows them) and `actions!(rusty, [PageBack, PageForward, TogglePageEdit])`; `init`
    registers `OpenPage` on each workspace while `marley.rusty.enabled` is on, checking the switch
    each time it runs, as #642's toggle does; `pub mod page; mod properties;`.
  - `marley_workbench::rusty::page`:
    - `open_later(workspace: WeakEntity<Workspace>, slug, preview, window, cx)`: `window.defer`,
      then `open`: `items_of_type::<PageView>` finding `slug()`; found, `activate_item` with the
      focus only without `preview`, and, without `preview`, `unpreview_item_if_preview` on its
      pane (`pane_for`); not found, a new `PageView` in the active pane, with `preview` through
      `replace_preview_item_id` then `add_item` without the focus at the index it returned,
      without `preview` through `add_item` with the focus.
    - `PageView { workspace, client, focus_handle, history, generation, shown: Shown,
      markdown: Entity<Markdown>, scroll, mode: Read | Edit(Entity<Editor>), connected }`, where
      `Shown` is `Loading | Page(RenderedPage) | Missing | Failed(SharedString)`.
    - `load`: bump `generation`, call `brain_render { slug }` through #643's client off the main
      thread, drop the answer when the generation moved, else `markdown.replace(page_markdown(..))`
      and, with a heading, `scroll_to_heading_when_parsed(generate_heading_slug(heading))`.
    - The `Markdown` entity: `Markdown::new_with_options(.., Some(language_registry), None,
      MarkdownOptions { parse_html: true, render_mermaid_diagrams: true, parse_heading_slugs: true,
      ..Default::default() })`, the registry from the workspace's project, as the preview builds it.
    - `render`: `v_flex().key_context("RustyPage").track_focus(..)` with the actions; the header
      (`IconButton`s with `ArrowLeft` and `ArrowRight` and `Tooltip::for_action_title`, the folder
      parts joined with ` / ` in muted text, the name, a `Button` labelled Edit or Read with
      `IconName::Pencil` and its toggle state); in Read, the title (`Headline` size), the properties
      and the `MarkdownElement` (`MarkdownStyle::themed(MarkdownFont::Preview, ..)`, `link_callback`
      returning the muted text colour for `rusty:new/`, `on_url_click` deferring to `follow`,
      `scroll_handle`); in Edit, the editor filling the body; the states' lines.
    - `follow(address)`: `PageLink::parse`, then navigate (`history.visit`, `load`), scroll, toast
      through the workspace (`NotificationId::unique::<PageView>()`, "There is no page TARGET
      yet."), or `cx.open_url`; a `Local` target navigates by its normalised name.
    - `back`, `forward`: in Edit, save first (below), then move and `load`, landing in Read.
    - `toggle_edit`: Read to Edit: the vault root (`setting_get`, else `util::paths::home_dir()`
      joined with `.rusty/brain`), `page_file_in`, `project.open_local_buffer(path)`,
      `Editor::for_buffer`, the subscription to its events, focus. Refused with a line in the tab
      when the project is remote or the file is missing. Edit to Read: when dirty,
      `editor.save(SaveOptions { format: false, autosave: true, .. })`, then drop the editor and
      `load`; a failed save stays in Edit and shows its error.
    - `Item`: `tab_content` (the title, `.italic()` when `params.preview`; the slug's last part
      while loading), `tab_icon`, `tab_tooltip_text`, `to_item_events` (the editor's `Edit`,
      `UpdateTab`), and, while in Edit, `is_dirty`, `has_conflict`, `can_save`, `save`, `reload` and
      `for_each_project_item` passed to the editor; `added_to_workspace` keeps the workspace
      (L-613). `Focusable` returns the editor's handle in Edit.
    - The connection: #643's state; while not connected the view keeps `Shown` and draws the
      not-connected line, and `follow`, `back` and `forward` wait.
  - `marley_workbench::rusty::properties` (Ely's `DescriptionList`, MIT notice at the top): rows of
    a key `Label` (muted, a fixed width in rems) and its value, wrapping under the key when short of
    room, a hairline between rows (`cx.theme().colors().border_variant`); values: `Chip`s for an
    array, `Label` text for a string, number or boolean, compact JSON for an object, and
    "(empty)" in muted text for null.
  - #644's door: `rusty::brain::open_page(slug, preview)` calls `rusty::page::open_later` in place
    of opening `<vault>/<slug>.md` in Zed's editor; its callers (the tree's click rule, Enter,
    Today, a new page) stay as #644 wrote them.
  - `crates/marley_workbench/keymap.json`: a `RustyPage` block, `alt-left` to `rusty::PageBack` and
    `alt-right` to `rusty::PageForward`, with a comment as the Browser block has.
  - `crates/marley_workbench/guide/index.html`: the Brain article gains a line on the Page tab
    (Edit, Read, Back and Forward, the preview rule). It is under `crates/marley_*`, so the commit
    receipt binds it: changed in the Code phase.
- **File manifest.**
  - Marley: `crates/marley_rusty/src/page.rs` (new), `crates/marley_rusty/src/marley_rusty.rs` (the
    module), `crates/marley_rusty/Cargo.toml` (`pulldown-cmark.workspace = true`), the fixtures and
    the stand-in where #643 put them; `crates/marley_workbench/src/rusty.rs`,
    `crates/marley_workbench/src/rusty/page.rs` (new),
    `crates/marley_workbench/src/rusty/properties.rs` (new, Ely's notice), #644's `open_page` (in
    `marley_workbench::rusty::brain`),
    `crates/marley_workbench/keymap.json`, `crates/marley_workbench/guide/index.html`;
    `script/e2e/645-brain-page-tab.sh` (new, Test phase). `marley_workbench` already depends on
    `markdown`, `editor`, `project`, `workspace`, `ui` and `util`; #643 adds `marley_rusty`.
  - Zed: `crates/zed/src/zed.rs` (crate `zed`): `"rusty"` in `test_action_namespaces`' expected
    list, beside `"marley"`, if no earlier ticket of the batch added it. No other Zed file
    changes: the preview API, `Markdown`, `MarkdownElement`, `Editor` and `Project` are used as they
    are.
- **The ledger row it extends** (`docs/marley/zed-touchpoints.md`, written before the code, §14):
  `crates/zed/src/zed.rs` (`:68`): "`"marley"` in `test_action_namespaces`" becomes "`"marley"` and
  `"rusty"` (#645) in `test_action_namespaces`", the why naming Rusty's actions (`rusty::OpenPage`,
  `PageBack`, `PageForward`, `TogglePageEdit`). No other row, and no new one.

### Visual check plan
The scenario `script/e2e/645-brain-page-tab.sh`, `compositor sway`. Setup: the scratch repository
opened with `open_path`; the vault under `$E2E_WORK/vault` with `.git` initialised, its three pages
written by the scenario (Atlas's `## Why` link targets a heading below the decision page's fold);
#643's stand-in linked as `$E2E_WORK/bin/rusty-mcp`, named in `MARLEY_RUSTY_MCP` and pointed at
that vault; `profile_setting` for `marley.rusty.enabled` true, the embedded connection,
`preview_tabs.enabled` and `preview_tabs.enable_preview_from_project_panel` true (the user's own
settings may turn previews off); the run's keymap binding Ctrl+Alt+Shift+O to
`["rusty::OpenPage", {"slug": "ideas/nowhere"}]`. Steps reach the Brain view by #644's switch;
row, link and button positions are measured from the first shots, as other click scenarios do.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, 005, 006, 008 | One click on Atlas's row | `645-01-preview`: italic tab; title, properties with tag chips; headings, list, tasks; links in two colours |
| REQ-007 | The wheel over the body | `645-02-body`: highlighted `rust` block; the table |
| REQ-002 | One click on Sam's row | `645-03-replaced`: one italic tab, Sam |
| REQ-003 | A double click on Atlas's row | `645-04-kept`: one tab, Atlas upright |
| REQ-009 | Click "the renderer decision" | `645-05-wikilink`: the decision page; Back enabled |
| REQ-011 | Alt-Left | `645-06-back`: Atlas; Forward enabled |
| REQ-012 | Alt-Right | `645-07-forward`: the decision page |
| REQ-010 | Alt-Left; click the heading link | `645-08-heading`: `Why` at the top of the body |
| REQ-013 | Alt-Left; click `ideas/later` | `645-09-unresolved`: the toast; Atlas still shown |
| REQ-004 | One click on Sam's row, then on Atlas's | `645-10-found`: two tabs, Atlas in front |
| REQ-014 | Click Sam's tab; click Edit | `645-11-edit`: the source with frontmatter; title italic |
| REQ-015, 016 | Type a line at the end of the file | `645-12-edit-keeps`: title upright, unsaved dot |
| REQ-017 | Click Read | `645-13-read-saves`: the line rendered, no dot; `expect` on the file |
| REQ-018 | Append a line to `people/sam.md` from outside; `SIGUSR1` to the stand-in; settle | `645-14-live`: the line shown |
| REQ-019 | Ctrl+Alt+Shift+O | `645-15-missing`: no page `ideas/nowhere` |
| REQ-020 | Set `marley.rusty.enabled` false from outside; settle | `645-16-not-connected`: the page and the line |
| REQ-021, 022 | Not driven | Review of the diff |

The file check after `645-13`: `grep` for the typed line, and every line of the page as the
scenario wrote it still present in order (no formatter ran). Not reached by a scenario: a web
link's click (it would open the user's browser, so the review reads `follow`'s last arm), the close
prompt for unsaved edits (Zed's own, reached through the passed `is_dirty`; reviewed), and a real
Rusty (R-D8: the stand-in only). If #644's switch or rows are not clickable as planned, the steps
that open from the tree use bindings to `rusty::OpenPage` with and without `preview`, and the
click-count rule is reviewed in #644's view.

### Risks
- **Shapes this ticket does not own.** #643's client call, connection state and signal, its
  stand-in, and #644's `open_page` were drafted beside this ticket; the names here are theirs as
  drafted. Promotion re-reads each as shipped; the decisions do not depend on them.
- **Gaps between the batch's drafts.** #646 leaves the picker to "#645 if it takes it, else its own
  ticket", counts on "the Page tab's own table of contents (#645)" for the outline (its D4), and
  says property edits are #645's. This slice takes none of the three: the picker and the outline
  are named in Out as their own tickets, and property edits stay Out as the brief for this ticket
  set ("Out unless small"). Without an outline ticket, no screen shows a page's headings in Read.
- **Re-entrancy.** The link click comes inside the `Markdown` entity's update; anything that
  touches that entity or the pane must be deferred (D9). The review checks every handler.
- **The fallback vault root** is the user's real `~/.rusty/brain`. A scenario whose stand-in failed
  to answer `setting_get` would open a real page in Edit. The stand-in always answers, the scenario
  checks the editor's file is under `$E2E_WORK` before it types, and Edit refuses a path outside
  the root it read.
- **Formatting on save.** Zed allows prettier for Markdown; an implicit save passes `format: false`,
  while Ctrl-S follows the user's settings and may reformat a page on purpose.
- **The two parsers can differ** where Zed's options do and Rusty's do not (smart punctuation, GFM
  callouts), which changes how text looks, never which wikilinks there are: both sides find
  wikilinks with the same pulldown-cmark and options.
- **A page with thousands of lines** is drawn whole by `MarkdownElement` (as Zed's preview does);
  the largest vault pages are a few hundred lines.
- **The R-D4 correction.** Saves are reindexed and announced, not committed, until the next Rusty
  write; a crash before then loses nothing on disk but leaves the commit for later. Complete fixes
  the plan's wording; the Rusty-side request waits on Chad.
- **The `rusty` namespace** costs one line in Zed's test list; if Chad prefers no Zed touch, the
  actions move under `marley` (`marley: open brain page`) with no other change.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: a Brain page section beside #644's Brain view (open, preview and keep,
  links, Back and Forward, Edit and Read, what saves and when Rusty sees it), the palette and
  keys tables (`rusty::PageBack`, `rusty::PageForward`, `rusty: toggle page edit`).
- `docs/marley/walkthrough.md`: a stop after #644's Brain view stop, following this ticket's shots.
- `crates/marley_workbench/guide/index.html`: changed in the Code phase (above).
- Architecture (§21): `docs/marley_architecture/marley_workbench.md` (the Page tab, its opener, the
  Ely port), `marley_rusty`'s page (the pass and the history), `docs/marley/rusty-in-marley.md`
  (R2's status; R-D4's sentence on commits corrected; `rusty: open page` split as D3 says, the
  action here and the picker its own ticket), `docs/marley/three-prong-plan.md`'s prong 2 line for
  Rusty in Marley, the `zed.rs` row checked, `CHANGELOG.md` under Added.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20, the templates, the 633 and 642 specs.
- [x] Read the Rusty-in-Marley plan in full (R-D0 to R-D10, the slices, Rusty's triage) and the
      brief's shared ground.
- [x] Recall: the knowledge ledgers (AD-609, AD-530, AD-490, F-599, F-503, L-613, L-574, L-633,
      L-607), the completed pipelines 530, 565, 609 and 633, a read-only brain search.
- [x] Discovery with file:line: Rusty's `brain_render`, renderer, vault, watcher and `NoteTab.qml`;
      Zed's preview tabs, `markdown`, `markdown_preview`, `Project` and `Editor`; pulldown-cmark
      0.13.4; Marley's items, keymap and namespace test; the live vault's counts (read only).
- [x] Prior-art sweep, three legs; Ely's `render.rs`, `tree.rs`, `toc.rs`, `pages.rs` and
      `records.rs` read, with what is ported (`DescriptionList`) and what is not, and why.
- [x] The rendering decision locked with its reason (D1), and the action's home decided (D3: here;
      the picker a second slice, since #646 leaves it out too).
- [x] Read the batch's sibling drafts (#643, #644, #646, #647) and took their names: the stand-in
      and `MARLEY_RUSTY_MCP`, the client's signal, `rusty::brain::open_page`, `SIGUSR1`.
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D10, 22 EARS rows, phase plan.
- [x] Design: approach, file manifest by crate, the touchpoint row to extend, the visual check
      plan, risks, the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Reconciliation, 2026-10-03
- Creating a page from an unresolved link: until Rusty's TICKET-041 lands, never send a slashed
  `name`; split the target into `folder` and `name`, and check the result's slug.
- Rusty's TICKET-042 will give the page's file path in `brain_read_page`; the `setting_get
  brain_vault_path` fallback stands until then.
- A disk edit through Edit is indexed but committed only with Rusty's next tool write until
  Rusty's TICKET-043 lands; the spec must not claim the edit is committed.
- #646 has no outline; the Read-mode outline is the follow-up R2b.

### Promotion (2026-10-04)
- Promoted into `active/`; the BACKLOG row removed; the ticket in-progress. Pre-flight green, no
  other active pipeline, `/mnt/fast` 178G free. #643 and #644 landed (6bae63ef8c, fbadf3282f).
- **Brain:** `brain ask` (consultation `b5f8e1bd27b243ffb587dc6476129ce4`) on the rendering
  question returned due follow-ups on other work only; a search found Rusty's project page noting
  that `brain_render` carries the page's `file` since its TICKET-042.
- **The names as #643 and #644 shipped them:** the client is `rusty::call_tool(tool, arguments,
  cx) -> Task<Result<String, String>>` (a refusal's error is Rusty's message), the connection
  `rusty::is_connected`, `is_on` and `unavailable`, a change Rusty announces re-reads the vault
  through `rusty::Vault` (a global the tab can observe); the vault's folder is `rusty::vault_folder`
  (`brain_vault_path` from `settings_list`, else `$HOME/.rusty/brain`), not `setting_get`; the
  stand-in announces any change to a file in its vault (`$RUSTY_STAND_IN_STATE/vault`), with no
  `SIGUSR1`; #644's door is `rusty::brain::open_page(workspace, slug, preview, focus, window, cx)`,
  which this ticket points at `rusty::page::open_later` with the same arguments.
- **Seams re-read** (an Explore pass over Zed, pulldown-cmark and Rusty's main at `13249a8`):
  every cited API stands, with these corrections. `Item::tab_content(params, window, cx)` is
  overridden for the italic preview title (Marley's other items set only `tab_content_text`).
  `Button` takes `start_icon`, not `icon`. `MarkdownStyle::link_callback` is a public field of a
  private alias type, set with `Some(Rc::new(..))`. Zed's Markdown preview passes `can_save`,
  `save` and `reload` to its editor but not `is_dirty`; the Page tab passes `is_dirty` and
  `has_conflict` too, so the unsaved dot and the close prompt follow the buffer.
  `Pane::set_preview_item_id` is crate-private, so the opener takes `open_project_item`'s two steps
  (`replace_preview_item_id`, then `Workspace::add_item` at the index it gave). `brain_render`
  answers `null` (a success) for a missing page, records `alias: None` for every wikilink, and on
  Rusty's main carries `file`, the page's absolute path (TICKET-042), and takes `blocks: true`
  (TICKET-036). The workspace's `pulldown-cmark` is 0.13.4 with default features off; `[[a|b]]`
  comes as `Tag::Link { dest_url: "a" }` with the text `b`.
- **Decided at promotion:**
  - **D1 stands: Marley's pass and Zed's renderer,** not Rusty's TICKET-036 blocks. The blocks
    are on Rusty's main but not on the box until Chad reinstalls, and drawing them means a
    renderer of Marley's own beside Zed's; the pass needs only `raw` and `links`, which every
    Rusty answers. Chad's confirmation of D1 (the plan's P1) is taken on the plan's
    recommendation, so the queue keeps moving under his "continue on tickets until finished", and
    goes on the list to confirm with him when the queue is done.
  - **Edit's file is `brain_render`'s `file` when Rusty gives one** (TICKET-042), else
    `rusty::vault_folder` joined with `<slug>.md`, with `page_file_in`'s guard on the second; the
    stand-in answers `file` as Rusty's main does.
  - **R-D4's commit:** Rusty's TICKET-043 (on main) commits an edit made outside the tools on its
    own after a sync, so once Chad reinstalls, a save from Edit is indexed, announced and
    committed. Complete words R-D4 that way, with the old binaries' behaviour as the exception.
  - **`"rusty"` joins `test_action_namespaces`** (#643 and #644 did not add it); the `zed.rs` row
    widens first.

## Phase 2 — Code (2026-10-04)
- **Built, to the manifest:**
  - `crates/marley_rusty/src/page.rs` (new; `pulldown-cmark` added to the crate): `RenderedPage`
    (`slug`, `title`, `properties`, `raw`, `file`, `links`; `from_answer` reads Rusty's `null` as
    `None`), `Property`, `LinkOut`, `body_of`, `split_fragment`, `normalise_target`,
    `page_markdown`, `PageLink::parse`, `Visit`, `PageHistory`, `page_file_in`, `BRAIN_RENDER`.
  - The stand-in: `brain_render` over its vault (`raw`, `properties` from simple YAML, `file`,
    `links` resolved by path or a unique name, `null` for a missing page). Driven by hand over
    stdio before the scenario.
  - `marley_workbench::rusty`: `pub mod page; mod properties;`, the `Announced` global (bumped on
    each embedded `list_changed`), `page::init` from `rusty::init`. `rusty/page.rs` (new):
    `OpenPage`, `PageBack`, `PageForward`, `TogglePageEdit`, `init`, `open_later`, `open`,
    `PageView` and its `Item`. `rusty/properties.rs` (new, Ely's `DescriptionList` layout under
    its MIT notice). `rusty::brain::open_page` now calls `page::open_later`.
  - `Cargo.toml` of `marley_workbench`: `schemars` (the action's `JsonSchema`). `keymap.json`: the
    `RustyPage` block (Alt-Left, Alt-Right). `guide/index.html`: the Brain article's open line and
    "A page's tab". `crates/zed/src/zed.rs`: `"rusty"` in `test_action_namespaces`, its ledger row
    widened first.
  - `script/e2e/645-brain-page-tab.sh`, written and run in this phase so its places are set before
    the receipt binds it.
- **Deviations from the plan:**
  - **`OpenPage` deserializes by hand.** gpui's `#[derive(Action)]` beside a derived `Deserialize`
    trips clippy's `unsafe_derive_deserialize` in the Marley crates; a private fields struct with
    the derive and `deny_unknown_fields` builds the action. The unit actions carry `#[derive(Eq)]`.
  - **A heading scroll redraws once more** (`redraw_after_parse`): found by the scenario, F below.
  - **The scenario binds Ctrl+Alt+Shift+Y**, not O, which Zed binds twice.
  - **`brain_render`'s `file`** opens in Edit when Rusty gives it (promotion); the vault folder and
    `page_file_in` are the fallback. Zed's autoscroll brings a heading into view rather than to
    the top, so shot `645-08` says "in view".
  - The tab re-reads on `rusty::Announced`, a global the connection bumps on each announcement,
    not on the vault cache, which changes only when the tree's shape does.
- **Review of the diff** against REQ-001 to REQ-022: the opener matches a tab by the page it shows
  now (PR-599); preview through `replace_preview_item_id` and `add_item`, keep through
  `unpreview_item_if_preview`; Edit's buffer passes dirty, conflict, save, reload and project items,
  so the close prompt is Zed's (REQ-016); Read, Back, Forward and a link save first with
  `format: false`; a missing page is `brain_render`'s `null` (REQ-019); not connected keeps the page
  and draws the line (REQ-020); `cx.open_url` for anything with a scheme (REQ-021); `OpenPage` while
  Rusty is unavailable shows the reason and opens nothing (REQ-022). Re-entrancy: the opener and
  each link run in `window.defer`; the tab's tasks update it through its weak handle. Provenance:
  Zed's `markdown`, `workspace`, `editor` and `project` used as they are; Rusty's three helper
  rules re-implemented from Rusty's (MIT, the same owner); Ely's layout ported under its notice;
  nothing from Warp.
- **Checks:** clippy on `marley_rusty` and `marley_workbench` green after three rounds
  (`map_or`, a doc paragraph, `const fn`; the `Action` derive against `unsafe_derive_deserialize`,
  `Eq` on the unit actions, `Arc::clone`, `map_or_else`, by-reference windows). The box's cargo was
  shared with rustal-os's gate runs, coordinated with that session. The scenario: run 1 at guessed
  places (links, tabs) stopped at the file check; run 2 green but for two reds in the shots, the
  heading link that scrolled nothing (F) and the missing-page key Zed had bound; run 3, after both
  fixes, green with every shot as the spec says.
- **F found:** `F-claude-645-a-heading-link-scrolled-nothing-in-a-page-at-rest-001`.
- **Gate, run 1:** RED on gate:13 alone: the doc comment of `OpenPageFields` said "an unsafe method",
  which the source-ban grep reads as `unsafe` with no `SAFETY:`. Reworded. The other 16 passed.
- **Gate, run 2:** `GATE GREEN [diff]`, 17 passed, the receipt written.

## Phase 3 — Test (2026-10-04)
- **Scenario:** `script/e2e/645-brain-page-tab.sh`, under `compositor sway`, run on the gated tree:
  `just build`, then `just e2e script/e2e/645-brain-page-tab.sh` with `SHOT_DIR` in the scratchpad.
  Exit 0; its check passed: Sam's file holds the typed line and, in order, every line the scenario
  wrote (no formatter ran). `brain_render` was called 7 times for Atlas, 5 for Sam, 3 for the
  decision and once for `ideas/nowhere`.
- **Shots, each read:**
  - `645-01-preview` (REQ-001, 005, 006, 008): one click on Atlas in the Brain view: an italic
    "Atlas" tab with the markdown icon; the header with Back and Forward (both dim), `projects /
    atlas` and Edit; the title; the properties `title`, `type`, `status` and `tags`, the tags as
    chips (`rust`, `gpui`, `brain`); the body's heading, the list, the task boxes (one checked, one
    not), the `rust` block highlighted, the table's head; "the renderer decision" and the heading
    link in the link colour, `ideas/later` muted, Zed's site a link.
  - `645-02-body` (REQ-007): the wheel over the body: the code block and the whole table in
    columns.
  - `645-03-replaced` (REQ-002): one click on Sam: one page tab, italic "Sam".
  - `645-04-kept` (REQ-003): a double-click on Atlas: one page tab, "Atlas" upright.
  - `645-05-wikilink` (REQ-009): "the renderer decision" clicked: "Use Zed renderer" in the same
    tab, Back lit, Forward dim.
  - `645-06-back` (REQ-011): Alt-Left: Atlas again, Forward lit.
  - `645-07-forward` (REQ-012): Alt-Right: the decision page.
  - `645-08-heading` (REQ-010): back to Atlas, the heading link clicked: the decision page scrolled
    until `## Why` and its line are in view (Zed's autoscroll; the run before the fix showed the
    page's top, F-claude-645-a-heading-link-scrolled-nothing-in-a-page-at-rest-001).
  - `645-09-unresolved` (REQ-013): back to Atlas, `ideas/later` clicked: a toast "There is no page
    ideas/later yet."; the tab still Atlas.
  - `645-10-found` (REQ-004): one click on Sam (a second tab, italic) and one on Atlas: Atlas's tab
    in front, two page tabs.
  - `645-11-edit` (REQ-014): Sam's tab, Edit: the file with its frontmatter in Zed's editor, the
    button reading Read and pressed, the tab still italic.
  - `645-12-edit-keeps` (REQ-015, REQ-016): a line typed at the end: the tab upright with Zed's
    unsaved dot.
  - `645-13-read-saves` (REQ-017): Read: the page rendered with "A line typed in Edit.", no dot;
    the file check passed.
  - `645-14-live` (REQ-018): a line appended to Sam's file from outside: "A line written from
    outside." shown with no input (it joins the typed line's paragraph, as the file reads).
  - `645-15-missing` (REQ-019): the bound key with `ideas/nowhere`: a kept tab "nowhere" saying "No
    page ideas/nowhere in the brain.", Edit disabled.
  - `645-16-not-connected` (REQ-020): Atlas's tab, then `marley.rusty.enabled` off from outside: the
    tab keeps Atlas under "Rusty is off. Turn it on in the Rusty section of the Marley settings.";
    the rail back on Projects.
- **By review:** REQ-021 (a web link goes to `cx.open_url`; a scenario would open the user's
  browser), REQ-022 (`OpenPage` with Rusty unavailable shows the reason and opens nothing), the
  close prompt for unsaved edits (Zed's, through the passed `is_dirty`).
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and
  did not reload it".
- Every shot shows Marley only; none is in the repository. The fixes this ticket needed were made
  in the Code phase, before its green gate; the receipt stands.

## Phase 4 — Complete (2026-10-04)
- **Documented (§21):** `CHANGELOG.md` (Added: a brain page in a tab); `docs/marley/rusty-in-marley.md`
  (R2 shipped 2026-10-04; R-D4 reworded: the pass and Zed's renderer as shipped, TICKET-036 as a
  later lookup, and the commit of a save as Rusty's TICKET-043 makes it, with the old binaries as
  the exception); `docs/marley/three-prong-plan.md` (C2); `docs/marley_architecture/marley_rusty.md`
  (the `page` module, the stand-in's `brain_render`); `docs/marley_architecture/marley_workbench.md`
  ("A brain page in a tab"); `docs/marley/guide.md` (the Brain view's open line, "A brain page in a
  tab", the keys table); `docs/marley/walkthrough.md` (stop 2.13); the in-app guide page (Code
  phase). `docs/marley/zed-touchpoints.md`'s `crates/zed/src/zed.rs` row names `"rusty"` (#645),
  as shipped.
- **Knowledge (§19):** `F-claude-645-a-heading-link-scrolled-nothing-in-a-page-at-rest-001` (no
  rule: the lesson below carries the fix's shape);
  `AD-claude-645-a-brain-page-is-drawn-by-zeds-markdown-after-marleys-wikilink-pass-001`;
  `L-claude-645-zed-binds-ctrl-alt-shift-o-twice-001`,
  `L-claude-645-a-data-action-in-a-marley-crate-deserializes-by-hand-001`,
  `L-claude-645-a-markdown-heading-scroll-needs-a-frame-after-the-parse-001`.
- **Brain:** `brain decide` on consultation `b5f8e1bd27b243ffb587dc6476129ce4`:
  `decisions/marley-draws-a-brain-page-with-zeds-markdown-crate-after-its-own-wikilink-pass`
  (follow-up 2026-10-18).
- **Still to confirm with Chad** when the queue is done: D1, the rendering (the plan's P1), taken on
  the plan's recommendation.
- **Closed:** the ticket moved to `tickets/closed/`, its link at `completed/`; no BACKLOG row was
  left (promotion removed it). The pair archived to `pipeline/completed/`.
