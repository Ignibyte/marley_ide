# A brain page's outline, and its title, name and properties edited in place: Notes

- **Local ticket doc:**
  docs/planning/tickets/open/TICKET-656-brain-page-outline-and-property-edits.md
- **Pipeline spec:** 656-brain-page-outline-and-property-edits.spec.md

## Phase 1: Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, on Rusty in Marley: "maybe rusty becomes Marley. We would take our
  rusty custom QML app and build it inside of marley", then "Plan it now". On 2026-10-03, for the
  first batch: "lets make a plan to begin the work and spec out the tickets" and "lets make sure we
  use the gpui components we found here", confirmed as "Queue all five". Drafting that batch split
  out R2b ("The Page tab's outline in Read mode and inline title and property edits",
  `docs/marley/rusty-in-marley.md`, the slices table), and #645's Out list and #646's D4 hand both
  halves to it. This follow-up batch, #654 to #659, queues the split slices; this is R2b, after
  #645.
- **Classification / tier:** feature, medium. The plan's table says S; the slice holds an outline
  column, one Ely port shared by three editors, five kinds of value editor with remove and add, a
  rename that re-points tabs, and a write queue, so M. One shippable piece: the outline and the
  edits share the tab's Read state and the in-place editor, and neither needs the other's ticket.
  One small Zed touch, extending an existing row. No new dependency: `chrono` is a workspace
  dependency `marley_workbench` already takes; `marley_rusty` adds it.
- **Recall (§18.3):**
  - AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001 and
    F-claude-503-a-tab-opened-inside-a-terminal-views-event-would-update-the-view-again-001: a pane
    or another item changed from inside an item's own update panics; `follow_rename` touches every
    Page tab, so it runs in `window.defer` (D9).
  - F-claude-600-a-registry-lookup-read-the-workspace-its-caller-was-updating-001: a lookup reached
    inside an entity's update must not read that entity; `follow_rename` is deferred out of the
    renaming tab's update before it reads the tabs.
  - F-claude-599-a-page-that-moved-to-its-own-anchor-got-a-second-tab-001 and PR-claude-599: a tab
    that navigates is found by what it shows now; after a rename the opener finds the tab under `to`
    (D9).
  - L-claude-457-a-single-line-editor-hands-zeds-list-keys-to-its-container-001: Enter and Escape in
    a single-line `Editor` arrive as `menu::Confirm` and `menu::Cancel`; a container's `Cancel` with
    nothing to do propagates (D6).
  - L-claude-490-a-view-that-forwards-keys-sees-its-editors-keys-001: keys from a child editor reach
    the tab's handlers; Alt-Left in an in-place editor reaches `rusty::PageBack` (Risks).
  - L-claude-600-a-hand-deployed-menu-stops-and-prevents-its-mouse-down-001: Add property's kinds
    open from Zed's `PopoverMenu`, which keeps the menu's focus; a scenario reaches entry N with N−1
    Downs (D10).
  - AD-claude-530-runbook-commands-go-to-the-last-terminal-at-its-prompt-001: Marley's seam in
    Zed's `markdown` so far is a generic hook only Marley passes; D3's request is the same shape.
  - AD-claude-452-the-rail-starts-zeds-own-rename-and-close-001: the rail renames terminals through
    Zed's own tab rename and rejected an editor in the row; a brain page has no Zed rename, and
    Rusty's app, Obsidian and Zed's project panel rename in place, so the name edits where it is.
  - L-claude-531, L-claude-633 (both), L-claude-607 and L-claude-620: the stand-in named in
    `MARLEY_RUSTY_MCP`, the run's settings copy and keymap, settings edited from outside before
    Marley writes the file, and a scroll shot with its target out of view first.
  - Completed pipelines 530 (the `markdown` seam, read line by line), 457 (single-line editors in a
    keyboard list), 600 (menus and lookups), 599 and 609 (items and openers).
  - The queued first batch: #643 (client, stand-in, `MARLEY_RUSTY_MCP`), #644 (`rename_target`, the
    tree's rename and move, its Out item on open tabs following them), #645 (`PageView`, the pass,
    `PageHistory`, the `DescriptionList` port, its Out list naming this slice), #646 (its D4: the
    outline is R2b's, Zed's outline panel for Edit), #647.
  - Brain (`rusty-cli brain search`, read only): `research/knowledge/obsidian` ("Properties are
    typed vault-wide by name"; renaming a note rewrites every link to it) and `projects/rusty-v3`
    (Rusty's typed properties editor over `brain_set_property` and `brain_remove_property`, the tags
    completion of its TICKET-024); nothing on an outline or in-place edits in Marley. Promotion asks
    the brain (`brain ask`) on D3 and D5; Complete records the decision.
- **Discovery** (read 2026-10-03; each claim read at the line):
  - **Rusty's tools** (`rusty-v3/crates/rusty-mcp/src/main.rs`, at `295565c`): `brain_rename`
    `:1295-1304` (params `from`, `to`, `:610-618`; a `to` ending in `/` moves into that folder);
    `brain_set_property` `:1424-1437` (params `slug`, `key`, `value`, `:658-667`);
    `brain_remove_property` `:1439-1447` (`:669-676`); `brain_render` `:1224-1252`. Every write goes
    through `mutate` (`:734-742`), which emits `DataChanged` before it returns the answer, and the
    notifier sends `list_changed` from another task (`:2062-2082`), so the signal can arrive before
    the answer (D8).
  - **Rusty's core** (`rusty-core/src/brain/`): `rename` `mod.rs:2506-2531` and `rename_page`
    `:2533-2584` (refuses "Already exists: TO", rewrites links everywhere, moves the index rows, one
    commit "move: FROM to TO (N pages updated)"); "A title that was the old file name follows the
    new one" `:2554-2567`, which writes the frontmatter through `BrainFrontmatter` and
    `render_page`, whose extra keys are a `HashMap` (`frontmatter.rs:38-40`, `:277-292`);
    `RenameReport` `mod.rs:196-206` (`from`, `to`, `kind`, `pages_rewritten`). `set_property`
    `mod.rs:1084-1097` and `remove_property` `:1099-1112` read the file, edit it with
    `frontmatter::set_property` (`frontmatter.rs:329-341`: order kept, the body byte for byte,
    frontmatter made when there is none, "A property needs a key") and `remove_property`
    (`:343-348`), and write through `write_edited` (`mod.rs:1115-1138`: a version, the file, the
    index, a commit "property: KEY on SLUG"; unchanged text a no-op). `properties_of`
    `frontmatter.rs:158-175` gives each key with its YAML value as JSON (a date stays a string).
    `fill_defaults` `:46-53` (an empty title is the file name). The resolver `mod.rs:3223-3261`:
    slug, then a unique last segment, then a unique title or alias.
  - **Rusty's outline:** `Heading { level, text, line }` `render.rs:169-178`; `render` `:221-251`
    computes it over `mark_callouts(strip_comments(body))`; `outline` `:254-280` (a trimmed line
    starting with one to six `#` and a space or tab, outside backtick and tilde fences; trailing
    `#`s trimmed; zero-based line); `strip_comments` `:285-334` and `mark_callouts` `:345-387` emit
    one line per input line, so `line` is a line of the body as cut by `split_raw`
    (`frontmatter.rs:85-106`).
  - **Rusty's app** (`rusty-app/qml`): `NoteTab.qml` `renameTo` `:205-209` (the typed text, `/` to
    `-`, renamed in the page's folder), the title `TextInput` `:518-531` whose `editingFinished`
    renames when the text differs from the title (`:528`), `setProperty` and `removeProperty`
    `:213-214`, `addProperty` `:216-229` (`""`, `[]`, `0`, `false`, today), `isList`, `listOf`,
    `kindOf` `:230-239` (list, number, bool, a string starting `YYYY-MM-DD` as a date, object,
    text), the add row's kinds `:564`, `PropertyRow` `:713-866` (chips with a remove icon `:747`,
    the add field `:753`, `CheckBox` `:821`, the text field `:827-842` writing a number only when it
    reads as one, objects as JSON `:844`, "Remove property" `:860`), the answers `:359-360` (a
    property write reloads; a rename replaces only the current history entry). `RightPane.qml`
    `:242-285`: the outline (indent `(level - 1) * 14`, level 1 in the foreground colour, a click
    calling `scrollToHeading`, "No headings.", a right-click "Bookmark heading").
  - **Zed's outline panel** (`crates/outline_panel/src/outline_panel.rs`): the workspace
    subscription `:1090-1114` reads `workspace_active_editor` `:5424-5433` (`act_as::<Editor>`, full
    mode) on `Event::ActiveItemChanged`, else `clear_previous` `:3347-3373`; `replace_active_editor`
    `:3308-3345`; `should_replace_active_item` `:5008-5012` (no current item, or another one);
    outline items from `editor.buffer_outline_items` `:3670`; rows as `ListItem` with `indent_level`
    `:2918-2921`. `ctrl-shift-b` toggles its focus on Linux
    (`assets/keymaps/default-linux.json:709`), and Marley's keymap takes that key only in a terminal
    (`crates/marley_workbench/keymap.json:38-41`).
  - **Items that hand Zed an editor:** `markdown_preview_view.rs:1636-1651`,
    `git_ui/src/project_diff.rs:546-562`. `workspace/src/item.rs:926-934`
    (`ItemEvent::UpdateBreadcrumbs` from the active item runs `active_item_path_changed`) and
    `workspace.rs:6699-6705` (which emits `Event::ActiveItemChanged`); `ItemEvent::UpdateTab`
    `item.rs:891-923` emits `ChangeItemTitle`, which also reaches `active_item_path_changed`
    (`workspace.rs:6230-6235`).
  - **Zed's `markdown`** (`crates/markdown/src/markdown.rs`): `scroll_to_heading_when_parsed`
    `:947-953`, `scroll_to_heading` `:955-963`, `replace` `:992-995`,
    `request_autoscroll_to_source_index` `:997-1009` (pending while parsing), `reset` `:1032-1059`
    (clears the request `:1050`), `parsed_markdown` test-only `:1060-1063`, the one struct literal
    in `new_with_options` `:664-700` (`autoscroll_request: None` `:687`), `scroll_handle`
    `:1864-1867` (controlled autoscroll), `autoscroll` `:2493-2537`, its controlled arm `:2505-2524`
    (above the top goal: up to three lines under the top; below the bottom goal: down to three lines
    above the bottom), called from `prepaint` `:3345`; `position_for_source_index` private
    `:4786-4799`. `parser.rs:104-150` `build_heading_slugs` (rendered text, `-1` for a repeat);
    `util/src/markdown.rs:4-17` `generate_heading_slug`. Row 86 of `docs/marley/zed-touchpoints.md`
    holds #530's change to this file.
  - **The in-place editors Zed has:** `project_panel.rs:805-840` (a single-line `Editor`; on
    `EditorEvent::Blurred if window.is_window_active()` it confirms the edit). `gpui`
    `elements/div.rs:3010-3075` (a key-up is a click only when its key-down came while the element
    had the focus, same focus generation). `ui_input/src/input_field.rs:21-145` (`InputField`, a
    labelled field with `set_error`; `marley_workbench` does not depend on `ui_input`).
    `ui/src/components/list/list_item.rs:70`, `:157`, `:203-211` (`new`, `spacing`, `indent_level`,
    `indent_step_size`); `toggle.rs:43-60` (`Checkbox`); `chip.rs:14-90` (`Chip`, no close button);
    `dropdown_menu.rs:22-80`. Icons: `ListTree`, `Pencil`, `Close`, `Plus` (`icons/src/icons.rs`).
    Zed's Linux keymap binds neither `alt-left` nor `alt-right` in `Editor` (only in a dev context,
    `default-linux.json:1556-1557`).
  - **Marley:** `groups.rs:440-475` (a single-line editor with its text set and selected),
    `decisions.rs:124`, `block_filter.rs:161`; `marley_workbench/Cargo.toml` takes `chrono`,
    `editor`, `markdown`, `menu`, `serde_json` and `ui` (`:20-88`). `rusty.rs` holds #633's offer;
    `rusty/` does not exist yet (#643 to #645 make it).
  - **The live vault** (read only, counts, `archive/` left out): 808 pages, 805 with a `title`, 163
    whose title is their file name with case aside (63 exactly); 3 file names with spaces, 11 with
    capitals. 3323 headings in 617 pages, at most 175 on one page, 52 holding a wikilink, 72 holding
    Markdown punctuation, two pages repeating a heading, no setext heading. Wikilink targets: 4300
    resolve by slug, 14 by last segment, none by title, 182 point into the page itself, 33 other.
    Frontmatter: 946 list values, no nested mapping, six empty values, 2292 values that are exactly
    a date and none with a time, at most 17 keys on a page; `stars` a number on 173 pages.
  - **Ely** (HEAD `2f8b2f6`, `LICENSE-MIT:3` "Copyright (c) 2026 Ely GPUI Component contributors"):
    `src/forms/inline.rs:1-203` (`Editing` `:18-88`: `begin` sets the text, selects it all and
    focuses; Submit keeps unless focus is to be handed back, Blur keeps; `finish(cancel)`;
    `InlineEdit` `:90-203`: Escape on key-down cancels and gives focus back, Enter on key-up keeps,
    the read state `:160-201`), `src/navigation/toc.rs:1-264` (`Sections`, `in_view`, `Anchor`,
    `TableOfContents`), `src/data_display/records.rs:88-222` (`PropertyGroup`, `PropertyGrid`).
- **Decisions:** D1 to D12 in the spec. In short: the Read outline is the tab's own column from
  Rusty's `outline`, the Edit outline is Zed's panel through `act_as_type`; a heading is found by
  its line and brought to the top by a small `markdown` seam; the title edits the property and the
  name renames (a change from Rusty's app, for Chad at promotion); Ely's `InlineEdit` on Zed's
  single-line `Editor` with the project panel's blur rule; kinds from the value as Rusty's app types
  them; writes queued per tab with the change signal held; a rename re-points every Page tab and
  #644's tree uses the same function; Add property asks the kind first; nothing edits without the
  connection or in Edit; one Ely port.

### Design
- **Approach.**
  - **`marley_rusty::page`** (#645's module, pure):
    - `Heading { level: u8, text: String, line: usize }` and `outline: Vec<Heading>` with
      `#[serde(default)]` in #645's typed view of `brain_render`, unless it already reads it.
    - `outline_label(text) -> String`: pulldown-cmark over the heading's text with Rusty's options
      and `ENABLE_WIKILINKS`, keeping `Text` and `Code` events, so `[[people/sam|Sam]]` gives `Sam`
      and `` `x` `` gives `x`; the text as written when nothing is left.
    - `line_start(body, line) -> Option<usize>`: the byte offset of zero-based line `line`, counting
      `\n` as `str::lines` does.
    - The offset in the text Zed parses: #645's `page_markdown` splices replacements back to front;
      it returns them with the text (`PageMarkdown { text, splices }`) and
      `text_offset(body_offset)` adds the length change of every splice that ends before it (a body
      offset inside a splice maps to the splice's start). If #645 shipped a mapping already, this
      uses it.
    - `PropertyKind { Text, Number, Date, Checkbox, List, ReadOnly }` and `PropertyKind::of(&Value)`
      (D7); `parse_value(kind, current, typed) -> Result<Option<Value>, ValueError>` (`None` when
      unchanged); `ValueError { NotANumber, NotADate }` with the two messages; `list_with(value,
      item)` and `list_without(value, index)`; `NewKind { Text, List, Number, Checkbox, Date }` and
      `empty_value(kind, today: NaiveDate)`; `key_taken(properties, key)`.
    - `RenameReport { from, to, kind, pages_rewritten }` (serde); `renamed(slug, report) ->
      Option<String>` (a page: equal; a folder: under `from/`); `PageHistory::rename(report)`;
      `rename_note(report) -> String` for the toast.
  - **`crates/marley_rusty/Cargo.toml`:** `chrono.workspace = true`.
  - **The stand-in** (`crates/marley_rusty/stand_in/rusty-mcp`, Python on the standard library):
    `brain_render`'s `outline` by Rusty's rule over the body after the frontmatter; scalar reading
    (`true`, `false`, integers and decimals typed; quotes stripped; `[a, b]` and block lists as
    lists of strings) where #645's does not already; `brain_set_property`, unless #655 (queued
    beside this ticket, drafting the same for its project view) lands it first (the key's line, or
    its block list's lines, replaced in place or appended at the end of the frontmatter; a string
    quoted when it holds `: ` or ` #` or starts with a YAML indicator; a list written as a block
    list; the body untouched; answers `{slug, title}`) and `brain_remove_property`; in #644's
    `brain_rename`, the rewrite of `[[FROM]]`, `[[FROM|` and `[[FROM#` to `TO` in every other page
    and their count as `pages_rewritten`, and the `RenameReport` shape. Each call logged with its
    arguments as JSON; `list_changed` after each write (#644's).
  - **`marley_workbench::rusty::inline_edit`** (new, Ely's MIT notice and `src/forms/inline.rs` at
    `2f8b2f6` named at the top): `EditTarget { Title, Name, Value(String), ListItem(String),
    NewKey(NewKind) }`; `InlineEdit { open: Option<Open> }` with `Open { target, editor:
    Entity<Editor>, error: Option<SharedString>, _subscription }`; `open(target, text, window, cx)`
    (`Editor::single_line`, `set_text`, `select_all`, focus) commits any editor already open first;
    `Commit { target, text }` handed back to the tab on `menu::Confirm` and on
    `EditorEvent::Blurred` while `window.is_window_active()`, matched to its own editor by entity
    id; `cancel` on `menu::Cancel`, propagating when nothing is open; `set_error` keeps the editor
    open with a line in `Color::Error` under it; focus back to the tab's handle after a commit or a
    cancel. The read state, `inline_text(id, value, placeholder, on_click)`: a `div` with
    `cursor_text`, a hover background (`ghost_element_hover`), the value or the muted placeholder
    ("Empty"), and `IconName::Pencil` shown on group hover.
  - **`marley_workbench::rusty::page`** (#645's `PageView`):
    - The outline column in Read while `show_outline` and the page has headings, beside the
      scrolling body: `v_flex().id("rusty-page-outline").flex_none().w(rems(14.)).border_l_1()` in
      `border_variant`, `overflow_y_scroll`, a `ListItem` per heading (`indent_level(level - min)`,
      `indent_step_size(px(12.))`, `ListItemSpacing::Dense`, a small `Label`, default colour at the
      shallowest level and muted below, `truncate`, the whole text as tooltip), each click calling
      `scroll_to_line(line)`.
    - `scroll_to_line`: `line_start` on the shown body, `text_offset`, then
      `markdown.update(|markdown, cx| markdown.request_autoscroll_to_top(offset, cx))`.
    - `show_outline: bool`, true for a new tab; `rusty::TogglePageOutline` and the header's
      `IconButton` (`ListTree`, toggle state, `Tooltip::for_action_title` or "No headings" when
      disabled).
    - `act_as_type`: `Self`, and the editor while Edit shows. Each Read and Edit change emits the
      tab's event that `to_item_events` maps to `ItemEvent::UpdateBreadcrumbs` and `UpdateTab`.
    - The title in Read, connected: `inline_text` over the title in #645's large label style, or the
      open editor; a commit queues `SetProperty { key: "title" }` when non-empty and changed.
    - The header's name in Read, connected: `inline_text` over the slug's last part; a commit queues
      `Rename { to }` from `rename_target(slug, typed)` joined to the folder.
    - The write queue: `writes: VecDeque<(String, Write)>` (the slug it was made on), `writing`,
      `held_change`. `queue` pushes and `pump`s; `pump` makes #643's client call off the main thread
      and, on the answer, on success for `Rename` defers `follow_rename`, on failure shows Rusty's
      message as a toast (`NotificationId::unique::<PageWriteToast>()`), then sends the next or,
      with the queue empty, reads the page once. #645's change-signal handler sets `held_change`
      while `writing` and reads otherwise. `shown_values: HashMap<String, Value>` holds a written
      value over the shown page's until the next read lands.
    - `follow_rename(workspace, report, window, cx)` (`pub`, for #644's tree): in `window.defer`,
      for each `PageView` of `workspace.items_of_type`, `history.rename`, the current slug moved,
      `UpdateTab` emitted, a read; then the rename toast.
  - **`marley_workbench::rusty::properties`** (#645's `DescriptionList` port): each row's value cell
    by `PropertyKind`: Text, Number and Date as `inline_text` or the open editor with its error
    line; Checkbox as `ui::Checkbox` whose click queues the other boolean; List as #645's chips,
    each followed by an `IconButton` (`Close`, `IconSize::XSmall`, "Remove TAG") and the list ended
    by an `IconButton` (`Plus`, "Add an item") that opens `EditTarget::ListItem`, whose commit
    queues the longer list and opens it again empty; ReadOnly as #645 draws it. Each row ends with
    an `IconButton` (`Close`, muted, "Remove property") queuing `RemoveProperty`. Under the rows a
    `PopoverMenu` triggered by a `Button` ("Add property", `IconName::Plus`) with a `ContextMenu` of
    the five kinds, each opening `EditTarget::NewKey(kind)` in a row of its own with the kind's name
    muted beside the editor; its commit refuses a taken key with `set_error` and queues a
    `SetProperty` of the key with `empty_value(kind, Local::now().date_naive())`. While Edit shows
    or the tab is not connected, the rows draw as #645 draws them.
  - **#644's tree** (`marley_workbench::rusty::brain`): after `brain_rename` succeeds, its rename
    and its move call `rusty::page::follow_rename` with Rusty's answer.
  - **`crates/marley_workbench/guide/index.html`:** the Brain article gains a line on the outline,
    the title, the name and the properties. Under `crates/marley_*`, so changed in the Code phase.
  - **Zed's `markdown`** (`crates/markdown/src/markdown.rs`): a field `autoscroll_to_top: bool` on
    `Markdown` (false in `new_with_options`), `pub fn request_autoscroll_to_top(&mut self,
    source_index: usize, cx: &mut Context<Self>)` setting it and calling
    `request_autoscroll_to_source_index`; `reset` clears it with the request;
    `MarkdownElement::autoscroll` takes it with the request, and in the controlled arm a set flag
    gives `current_offset.y + (top_goal - position.y)`, clamped as Zed clamps. `// Marley:` on each
    hunk.
- **File manifest.**
  - Marley `marley_rusty`: `src/page.rs` (#645's; the additions above), `Cargo.toml` (`chrono`),
    `stand_in/rusty-mcp` (the additions above).
  - Marley `marley_workbench`: `src/rusty.rs` (`TogglePageOutline` in the `rusty` actions,
    `mod inline_edit;`), `src/rusty/inline_edit.rs` (new, Ely's notice), `src/rusty/page.rs` and
    `src/rusty/properties.rs` (#645's), `src/rusty/brain.rs` (#644's, two calls),
    `guide/index.html`; `script/e2e/656-brain-page-outline-and-property-edits.sh` (new, Test phase).
  - Zed `markdown`: `crates/markdown/src/markdown.rs` (above). No other Zed file: `outline_panel`,
    `workspace`, `editor`, `ui` and `markdown_preview` are used as they are, and the `rusty` action
    namespace is already in `zed.rs`'s test list from #645.
- **The ledger row it extends** (`docs/marley/zed-touchpoints.md`, row 86,
  `crates/markdown/src/markdown.rs`, written before the hunk, §14): the change column gains
  "`Markdown::request_autoscroll_to_top` and its `autoscroll_to_top` flag, taken with the autoscroll
  request; in `MarkdownElement::autoscroll`'s controlled arm a flagged request puts the target three
  lines under the viewport's top (#656)"; the why column "the brain Page tab's outline brings a
  heading to the top of the page, where Zed's request brings it only into view (#656)"; the merge
  column "keep the flag beside `autoscroll_request`, cleared in `reset`, and its one branch in the
  controlled arm; every other caller leaves it unset, so no other Markdown view scrolls
  differently". No new row.

### Visual check plan
The scenario `script/e2e/656-brain-page-outline-and-property-edits.sh`, `compositor sway`. Setup:
the scratch repository opened with `open_path`; the vault under `$E2E_WORK/vault`, `.git`
initialised, its three pages written by the scenario (Draft idea's `## Why` far enough below its
fold that it is out of view at first, L-620); #643's stand-in linked as `$E2E_WORK/bin/rusty-mcp`,
named in `MARLEY_RUSTY_MCP`, pointed at the vault and at `$E2E_WORK/rusty.log`; `profile_setting`
for `marley.rusty.enabled` true and the embedded connection; the run's keymap with
Ctrl+Alt+Shift+O for `["rusty::OpenPage", {"slug": "notes/draft-idea"}]` and Ctrl+Alt+Shift+B for
`outline_panel::ToggleFocus`. Positions of headings, values and buttons are measured from the first
shots, as other click scenarios do. Each "the log" check is a `grep` of the call's line for the
tool, the key and the value with its JSON type (`4`, not `"4"`; `true`, not `"true"`).

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, 002 | Ctrl+Alt+Shift+O | `656-01-outline`: the nine entries, indents, "Links to Sam"; `Why` out of view |
| REQ-003 | Click Why in the outline | `656-02-outline-scroll`: Why near the top, its line under it |
| REQ-004 | Click the second Notes | `656-03-same-name`: "Second notes." under the heading |
| REQ-005 | Click the Outline button | `656-04-outline-hidden`: no column |
| REQ-007 | Click it again; Edit; Ctrl+Alt+Shift+B | `656-05-edit-outline-panel`: Zed's panel lists the headings; no column; then Read, dock closed |
| REQ-008, 005 | Click the title | `656-06-title-open`: editor, text selected; the column back |
| REQ-009 | "Draft plan", Enter | `656-07-title-set`: title and tab label; the log |
| REQ-010 | Click the title, "zzz", Escape | `656-08-title-escape`: Draft plan; the log unchanged |
| REQ-012 | `status`, "doing", Enter | `656-09-text`: `doing`; the log |
| REQ-013 | `stars`, "4", Enter | `656-10-number`: `4`; the log's number |
| REQ-014 | `stars`, "four", Enter; then Escape | `656-11-number-refused`: the line; the log unchanged |
| REQ-015, 011 | `due`, "2026-11-01", click the body | `656-12-date-on-blur`: the date; the log |
| REQ-016 | Click `reviewed`'s box | `656-13-checkbox`: checked; the log's `true` |
| REQ-017 | `tags`' add, "rusty", Enter; then Escape | `656-14-list-add`: three chips; the log's list |
| REQ-018 | Remove on `idea` | `656-15-list-remove`: two chips; the log |
| REQ-020 | `status`'s Remove property | `656-16-property-removed`: no row; the log |
| REQ-021 | Add property, Number, "priority", Enter | `656-17-property-added`: `priority 0`; the log's `0` |
| REQ-022 | Add property, Text, "stars", Enter; then Escape | `656-18-key-taken`: the line; the log unchanged |
| REQ-006 | Click Sam in the body; then Alt-Left | `656-19-no-headings`: no column, button disabled |
| REQ-023 | Click the header's name, "first-idea", Enter | `656-20-renamed`: `notes / first-idea`, title kept, the toast; the log |
| REQ-025 | Alt-Right, Alt-Left | `656-21-history-follows`: first-idea's page |
| REQ-026 | The name, "linker", Enter | `656-22-rename-refused`: Rusty's message; `first-idea` kept |
| REQ-025 | Brain view: rename `people/sam` to `samuel` (#644's menu); Alt-Right in the tab | `656-23-tree-rename-follows`: `people / samuel` |
| REQ-028 | `marley.rusty.enabled` false from outside; settle; click the title | `656-24-not-connected`: no editor; the line |
| REQ-011 (inactive window), 014 (date), 019, 024, 027, 029, 030, 031 | Not driven | Review of the diff; the gate for 031 |

Not reached by a scenario: an editor losing the focus to another window (the headless sway has one
client; the review reads the `is_window_active` check), a date refused (the same path as the number,
reviewed), the order of two writes in flight (the stand-in answers at once; the review reads the
queue), Rusty's real link rewriting and its title rule (R-D8: the stand-in only; Rusty's own tests
hold them), and Zed's other actions on the editor `act_as_type` exposes (reviewed). Nothing in the
scenario writes `settings.json` from inside Marley before shot 24's outside edit (L-607). If #644's
tree is not clickable as planned, shot 23 renames through a binding the scenario adds to the tree's
rename action, and the follow is still read off the tab.

### Risks
- **Shapes this ticket does not own.** #643's client and signal, #644's `rename_target` and tree,
  and #645's `PageView`, pass, `PageHistory` and properties port were drafted beside this ticket and
  ship before it; the names here are theirs as drafted. P1 re-reads each as shipped; the decisions
  do not depend on their names.
- **#645's heading links meet the same edge.** #645's REQ-010 shot expects `Why` at the top after a
  heading link, but Zed's controlled autoscroll brings a heading below the view only to three lines
  above the bottom (`markdown.rs:2512-2518`). If #645 adds a top-aligned request, this ticket uses
  it and adds no hunk; if it does not, this ticket's request serves both, and P1 says so in #645's
  archive notes.
- **The next frame.** The autoscroll is applied in `prepaint` and shows on the following frame. A
  tab with nothing else changing may not draw one at once; if shot 02 shows the old place, the
  click handler asks for another frame after the request. The shot is the proof.
- **The line map.** It needs #645's pass to keep its splices or never to move a line break; P1 reads
  the pass as shipped. Rusty's line rule lists a `#` line inside an HTML block or an indented code
  block and misses a heading inside a quote, a list or a setext heading (the live vault has no
  setext heading); a click on such an entry still scrolls to its line.
- **The editor Zed sees in Edit.** `act_as::<Editor>` hands the vault buffer to every Zed action
  that reads the active editor (go to line, Zed's own Markdown preview, copy path,
  `outline::Toggle`), as Zed's preview and `ProjectDiff` do. The review runs the obvious ones.
- **The outline panel's refresh.** It replaces its editor only when it holds no item or another one
  (`should_replace_active_item`); Read clears it, so each Edit is new to it. Shot 05 shows it.
- **Back and Forward from an open editor.** Zed binds neither Alt-Left nor Alt-Right in `Editor`, so
  in the tab's key context they reach `rusty::PageBack` and `PageForward` while an in-place editor
  is open; the editor loses the focus, keeps its text, and the write goes to the slug it was made on
  (D8). Accepted; a `!Editor` in #645's keymap block would also take Back from Edit.
- **D5 changes what the brief and Rusty's app do.** The title no longer renames. If Chad wants the
  title to rename, D5's first rejected choice is a small change: the title's commit queues `Rename`
  in place of `SetProperty`.
- **A title edit can strand a title link.** Rusty resolves `[[Old Title]]` by title or alias; a new
  title leaves such a link unresolved. The live vault has none; the alias request is Rusty-side
  (Out).
- **Rusty's rename and frontmatter order.** For a page whose title is its file name, Rusty's rename
  rewrites its frontmatter through a `HashMap` of extra keys, so that page's properties may come
  back in another order; Rusty-side (Out). The scenario renames a page whose title differs.
- **TICKET-043.** A property write commits through Rusty, and its `git add -A` sweeps any Edit save
  not yet committed under the property's message.
- **Width.** The column takes 14 rem from the body; in a narrow pane the toggle hides it.
- **The scenario's toasts** can be gone or not yet drawn when a shot is taken; shots wait for the
  stand-in's log line and settle first.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: the Brain page section gains the outline (Read, and Zed's outline panel in
  Edit), the title, the name and what a rename rewrites, the properties by kind, Add property and
  Remove, and what Rusty writes and commits for each; the palette table gains
  `rusty: toggle page outline`.
- `docs/marley/walkthrough.md`: a stop after #645's, following this ticket's shots.
- `crates/marley_workbench/guide/index.html`: changed in the Code phase.
- Architecture (§21): `docs/marley_architecture/marley_workbench.md` (the outline column, the
  in-place editor port, the write queue, `follow_rename`), `marley_rusty`'s page notes (the line
  map, the kinds), `docs/marley/rusty-in-marley.md` (R2b's status; D5's title and name; the Ely
  table's `inlineedit-editabletext` row as ported, `markdownrenderer-tableofcontents-…` as read),
  `docs/marley/three-prong-plan.md`'s prong 2 line, the `markdown.rs` row checked, `CHANGELOG.md`
  under Added.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20, the templates, the brief's three parts, and the
      633 spec for shape and tone.
- [x] Read the Rusty-in-Marley plan in full (R-D0 to R-D10, the slices, Rusty's triage) and the
      first batch's specs and notes (#643 to #647), taking their names.
- [x] Recall: the knowledge ledgers (AD-609, F-503, F-600, F-599, PR-599, L-457, L-490, L-600,
      AD-530, AD-452, L-531, L-633, L-607, L-620), the completed pipelines 530, 457, 600, 599 and
      609, a read-only brain search.
- [x] Discovery with file:line: Rusty's rename, property and render tools and their core, its
      outline rule, `NoteTab.qml` and `RightPane.qml`; Zed's outline panel, the items that hand it
      an editor, the workspace's events, `markdown`'s autoscroll and slugs, the project panel's
      rename, gpui's keyboard click, `ui`'s parts; the live vault's counts (read only).
- [x] Decided the outline's home (the brief's question): a non-editor item cannot feed Zed's outline
      panel, so Read gets the tab's own column and Edit hands Zed the editor (D1).
- [x] Prior-art sweep, three legs; Ely's `inline.rs` (ported), `toc.rs` and `records.rs` (read, not
      ported) read, with what is taken and what is left.
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D12, 31 EARS rows, phase plan.
- [x] Design: approach, file manifest by crate, the touchpoint row to extend, the visual check plan,
      risks, the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Reconciliation, 2026-10-03
- A `rusty:` action run while Rusty is off or not connected shows a toast saying so and where to
  turn it on, and opens nothing (rusty-in-marley.md R-D0, settled across #643 to #659).
- Every scenario names its stand-in in `MARLEY_RUSTY_MCP`, never first on the PATH (#643).
