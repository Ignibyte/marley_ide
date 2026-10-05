---
pipeline_id: a1dc2d17-aaea-4fb8-a462-d777636e0a27
ticket: docs/planning/tickets/open/TICKET-656-brain-page-outline-and-property-edits.md
status: Phase 4 — Complete PASS
title: "A brain page's outline, and its title, name and properties edited in place"
type: feature
slice: Rusty in Marley R2b (outline and inline edits), R-D4; prong 2 (D11 as amended)
references: [docs/marley/rusty-in-marley.md, docs/marley/three-prong-plan.md]
---

## Title
#645's Page tab gains the two things Rusty's app has that #645 left out. In Read, a column
beside the body lists the page's headings from `brain_render`'s `outline`, indented by level, and a
click brings that heading to the top of the body; in Edit, the tab hands its editor to Zed, so Zed's
own outline panel lists the source's headings. And the page is edited where it is drawn: the title
writes the `title` property, the name in the header renames the page through `brain_rename` (Rusty
rewrites every link to it), and each property edits by its value's kind (text, number, date,
checkbox, list) through `brain_set_property`, with a remove button (`brain_remove_property`) and
Add property. The in-place editor is Ely GPUI Components' `InlineEdit`, ported onto Zed's
single-line `Editor`. Behind `marley.rusty.enabled` (#643), off by default.

## Scope
### In
- **Builds on #643, #644 and #645, after #645 in order.** #643's `crates/marley_rusty` (pure, with
  its fixtures and the Python stand-in `stand_in/rusty-mcp`, which a scenario names in
  `MARLEY_RUSTY_MCP`, never first on the PATH) and its client in `marley_workbench::rusty`, with the
  connection state and the change signal on `notifications/resources/list_changed`; #644's Brain
  view, its tree rename and move, and `marley_rusty::vault::rename_target`; #645's
  `rusty::page::PageView`, `rusty::OpenPage { slug, preview }`, `rusty::page::open_later`,
  `marley_rusty::page` (`body_of`, the wikilink pass, `PageHistory`) and `rusty/properties.rs` (its
  port of Ely's `DescriptionList`). This ticket adds to all of them and takes their names as they
  ship.
- **The outline, in Read.** A column at the body's right, `rems(14.)` wide with its own scroll and a
  hairline at its left, lists every entry of `brain_render`'s `outline` (`level`, `text`, `line`) in
  order: a `ui::ListItem` per heading, indented by its level less the page's shallowest level, the
  shallowest in the default text colour and deeper ones muted, each label the heading's text with
  its Markdown taken out (`[[people/sam|Sam]]` reads `Sam`), truncated with the whole text as its
  tooltip. A click brings that heading to three lines under the top of the body. It follows the page
  as it is read again. A page with no headings shows no column.
- **The outline toggle.** `rusty::TogglePageOutline` and an `IconName::ListTree` button in the
  header between Forward and the folder; on in each new tab, kept per tab, not across a restart;
  disabled with the tooltip "No headings" for a page without any.
- **The outline, in Edit.** `PageView::act_as_type` answers `Editor` with its editor while Edit
  shows, as Zed's Markdown preview and `ProjectDiff` do, and the tab emits
  `ItemEvent::UpdateBreadcrumbs` on each change between Read and Edit, so Zed's `outline_panel`,
  which follows the active item through `act_as::<Editor>`, lists the source's headings in Edit and
  lets them go in Read. The tab's own column shows only in Read.
- **Landing at the top.** A heading is found by its line: Rusty's `line` is a line of the body,
  whose start `marley_rusty::page` turns into a byte offset of the text Zed's `Markdown` parses
  (through #645's pass), and the tab asks `Markdown` to bring that offset to the top of its scroll
  view. Zed's own request brings a target below the view only to its bottom edge, so `markdown`
  gains a small top-aligned request (D3), unless #645 added one for its heading links first.
- **The in-place editor** (`marley_workbench::rusty::inline_edit`, Ely's `InlineEdit` ported, its
  MIT notice on the file): the value as text, with a hover background and a pencil on hover; a
  click puts a single-line `Editor` in its place with the whole value selected and the focus in it;
  Enter or leaving it while Marley's window is active keeps the text, Escape keeps nothing; the
  focus goes back to the tab. One is open at a time.
- **The title.** The large title is an in-place editor over `brain_render`'s `title`. A new title is
  written with `brain_set_property { slug, key: "title", value }`; an empty or unchanged one writes
  nothing.
- **The name.** The header's name (the slug's last part) is an in-place editor. A new name renames
  the page in its folder with `brain_rename { from: slug, to: "<folder>/<name>" }`, the name made by
  #644's `rename_target` (trimmed, a typed `/` turned into `-`, nothing for an empty or unchanged
  one). On Rusty's answer (`from`, `to`, `kind`, `pages_rewritten`) every Page tab of the workspace
  follows (D9) and a toast says "Renamed to TO; links updated in N pages." (one page; "no page
  linked to it" for none).
- **The tree follows too.** #644's tree rename and move call the same `rusty::page::follow_rename`
  with Rusty's answer, so open Page tabs follow a rename or move made in the Brain view (#644's Out
  item, for rename and move; delete stays out).
- **Properties by kind** (D7), in #645's `DescriptionList` rows, the key in its column and the
  editor in the value's place: text and an empty value as text; a number, kept only if it reads as a
  number; a date (a string that is exactly a `YYYY-MM-DD` calendar date), kept only as one; a
  checkbox (`ui::Checkbox`), whose click writes the other value; a list of text as `ui::Chip`s, each
  with a remove button, and an add button that opens an in-place editor for one more item, still
  open after Enter for the next. An object, or a list holding anything but text, is drawn as #645
  draws it, with no editor. A malformed number or date keeps the editor open with "Enter a number"
  or "Enter a date as YYYY-MM-DD" under it and writes nothing.
- **Remove and add.** Each row ends with a muted `IconName::Close` button, "Remove property", that
  calls `brain_remove_property { slug, key }`. Below the rows, "Add property" opens a menu of the
  five kinds; picking one opens an in-place editor for the key, its kind named beside it; Enter
  writes the key with the kind's empty value (`""`, `[]`, `0`, `false`, today's local date). A key
  the page already has is refused with "A property named KEY exists".
- **Writes.** One at a time per tab, in order, each for the slug it was made on. The row shows the
  new value at once; the tab reads the page once after its writes, and a change signal that comes
  while one is under way is held for that read (D8). A refusal shows Rusty's message in a toast and
  the value as Rusty holds it.
- **What stays read only:** everything while Marley has no connection to Rusty (#645's
  not-connected line), and the header's name while Edit shows; the properties and the title are not
  drawn in Edit (#645).
- **The stand-in** (#643's, with #644's and #645's tools) gains: `outline` in `brain_render` by
  Rusty's rule; numbers and `true`/`false` read as JSON numbers and booleans in `properties`;
  `brain_set_property` (unless #655, which drafts the same, lands it first) and
  `brain_remove_property` over the scratch vault's frontmatter, keeping the other keys in order and
  the body byte for byte; and, in #644's `brain_rename`, the rewrite of `[[from]]`, `[[from|` and
  `[[from#` in the other pages with their count as `pages_rewritten`. Each call is logged with its
  arguments, as before.
- **Zed's `markdown` crate:** `Markdown::request_autoscroll_to_top`, a field and one branch in the
  element's controlled autoscroll, extending the `crates/markdown/src/markdown.rs` row of
  `docs/marley/zed-touchpoints.md` (D3).
- The in-app guide page's Brain article line, and
  `script/e2e/656-brain-page-outline-and-property-edits.sh`.

### Out (explicitly deferred)
- **Marking the section in view** (Ely's `TableOfContents` line, its `in_view`): it needs where each
  heading was painted, which `MarkdownElement` keeps to itself (`position_for_source_index` is
  private, `Markdown::parsed_markdown` is test-only). A later seam in `markdown` (a callback with
  the painted top of chosen source offsets) would give it.
- **Rusty's TICKET-036, the structured render:** blocks with source byte ranges would replace the
  line rule this ticket maps through, and would carry headings Rusty's line rule misses (setext,
  inside a quote or a list).
- **Rusty's TICKET-035, the change cursor:** nothing here depends on it; a property written by
  another process into the database alone is not announced (page files are watched).
- **Rusty's TICKET-037, bookmarks:** Rusty's app bookmarks a heading from the outline's right-click
  menu; waits on TICKET-037's tools.
- **Rusty's TICKET-043:** a property write is a tool write, so Rusty commits it, and its `git add
  -A` also takes any Edit-mode save not yet committed under the property's message until TICKET-043
  makes tool commits name only their paths.
- **Tags as Rusty's app edits them:** the `Tags` kind in Add property, completion from `brain_tags`
  (Rusty's TICKET-024 in its app), a chip's click searching `tag:` (#646's search does that).
- **More property kinds and moves:** Obsidian's Date & time, multi-line text, values as links (the
  `consulted` slugs, `url`), renaming a key, changing a property's kind, reordering properties.
- **The page menu** (move to a folder, delete, open the local graph), a key or palette action for
  the title and the name, and the outline's folding, filter and drag.
- **Two Rusty-side requests**, filed in Rusty when Chad confirms: a rename that moves the title
  along writes the frontmatter through `BrainFrontmatter`, whose extra keys sit in a `HashMap`, so
  that page's properties lose their order (`mod.rs:2554-2567`, `frontmatter.rs:38-40`); and an alias
  kept for a page's old title, since Rusty resolves `[[Title]]` by title (`mod.rs:3223-3261`; no
  link in the live vault does).
- Restoring the outline toggle after a restart (Page tabs are not restored, #645's D10).

## Reference (§20)
Upstream Zed, kept as it is: `outline_panel`, which follows the workspace's active item and takes
its outline from `act_as::<Editor>` (`workspace_active_editor`, `outline_panel.rs:5424-5433`),
and the items that hand it an editor they hold (`markdown_preview::MarkdownPreviewView` and
`git_ui::ProjectDiff`, each `act_as_type`); the workspace's `ItemEvent::UpdateBreadcrumbs`, which
makes the workspace announce the active item again; the project panel's in-place rename (a
single-line `Editor` in the row, Enter keeps, Escape cancels, losing the focus keeps only while the
window is active); and `markdown`'s autoscroll, to which Marley adds a top-aligned request beside
Zed's. The screen's behavior follows Rusty's own app (MIT, the same owner): `NoteTab.qml`'s inline
title, its typed `PropertyRow` with a remove button, Add property with a kind, and the rename's
history fix; `RightPane.qml`'s outline (indented by level, the top level brighter, a click scrolls
the page). No Warp behavior applies: Warp has no page properties, and its editor's `outline` module
is named in `docs/warp_architecture/crates/warp_editor.md:21` with no behavior described.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` §4 (the
  outline panel follows the active editor through a workspace subscription and fetches symbols per
  buffer with `buffer_outline_items`; one outline model, two surfaces). `docs/warp_architecture/`
  has no outline or properties behavior (grepped for outline, table of contents, frontmatter);
  `docs/orca_architecture/` neither.
- **Published material:** Obsidian's Outline view (the active note's headings, a click jumps) and
  Properties (kinds Text, List, Number, Checkbox, Date, Date & time; a kind belongs to the
  property's name across the vault, kept in `.obsidian/types.json`), and its inline title (the file
  name, whose edit renames the file and updates links). The brain's `research/knowledge/obsidian`
  page ("Properties are typed vault-wide by name"; renaming a note rewrites every link to it).
  Rusty's tools (`rusty-v3/crates/rusty-mcp/src/main.rs`): `brain_set_property` (`:1424-1437`,
  "text, number, true/false, a YYYY-MM-DD date as text, or a list of strings; other keys keep their
  order and the body is untouched"), `brain_remove_property` (`:1439-1447`), `brain_rename`
  (`:1295-1304`, "every link to it in the vault is rewritten and the index follows"), and
  `brain_render`'s `outline` (`rusty-core/src/brain/render.rs:169-178`, `:254-280`).
- **The code we already ship:**
  - `outline_panel` (Zed) owns the Edit half: it lists any full-mode `Editor` an item hands to
    `act_as::<Editor>` and moves that editor's selection on a click; it cannot take a list from an
    item with no editor, so it cannot serve Read. Taken unchanged.
  - `markdown_preview` (Zed): `act_as_type` hands its source editor to Zed
    (`markdown_preview_view.rs:1636-1651`); `git_ui::ProjectDiff` the same
    (`project_diff.rs:546-562`). The pattern for the Edit half.
  - `workspace` (Zed): `ItemEvent::UpdateBreadcrumbs` from the active item runs
    `active_item_path_changed`, which emits `Event::ActiveItemChanged` (`item.rs:926-934`,
    `workspace.rs:6699-6705`), the event the outline panel reads.
  - `markdown` (Zed): `request_autoscroll_to_source_index` (`markdown.rs:997-1009`) and the
    controlled autoscroll (`:2493-2524`), which brings a target three lines inside the nearer edge;
    `scroll_to_heading` by Zed's heading slugs (`:955-963`), built from the rendered text with `-1`
    for a repeat (`parser.rs:104-150`); `parsed_markdown()` test-only (`:1060-1063`). Extended by
    one request (D3).
  - `project_panel` (Zed): the in-place rename's editor and its blur rule
    (`project_panel.rs:805-840`, `EditorEvent::Blurred if window.is_window_active()`).
    `editor::Editor::single_line`, whose Enter and Escape reach the container as `menu::Confirm` and
    `menu::Cancel` (L-457).
  - `gpui` (Zed's): a focused element's key-up becomes a click only when its key-down came with the
    same focus generation (`elements/div.rs:3010-3075`), so a commit on Enter's press cannot reopen
    the editor.
  - `ui` (Zed): `ListItem` with `indent_level` and `indent_step_size`, as the outline panel draws
    its entries (`outline_panel.rs:2918-2921`); `Checkbox` (`toggle.rs:43-60`); `Chip`;
    `IconButton`; `PopoverMenu` with a `ContextMenu` for the kinds; `Tooltip`; `Label`.
    `ui_input::InputField` was considered: a labelled field that is always an editor, and a
    dependency `marley_workbench` does not have; Zed's project panel and Marley's own rows switch
    between a label and a single-line `Editor` instead.
  - `chrono` (Cargo.lock; already `marley_workbench`'s): `NaiveDate` for the date check, `Local` for
    today. `serde_json` for the values.
  - Marley: #645's `PageView`, its pass and `PageHistory`, and its `DescriptionList` port; #644's
    `rename_target` and tree rename; `groups.rs:440-475` (a single-line editor opened with its text
    selected); `decisions.rs:124`, `block_filter.rs:161` (single-line editors in Marley views).
  - Rusty's app (`rusty-app/qml`): `NoteTab.qml` `renameTo` (`:205-209`), the title field renaming
    the file on `editingFinished` (`:518-531`), `setProperty` and `removeProperty` (`:213-214`),
    `addProperty` with the kinds' empty values (`:216-229`), `kindOf` (`:232-239`), the add row's
    kinds (`:564`), `PropertyRow` (`:713-866`), the renamed answer replacing the current history
    entry (`:360`); `RightPane.qml:242-285` (the outline list).
  - Ely GPUI Components at `2f8b2f6` (`MIT OR Apache-2.0`), story `inlineedit-editabletext` and
    `markdownrenderer-tableofcontents-documentoutline-readingprogress`:
    - `src/forms/inline.rs` (`Editing` `:18-88`, `InlineEdit` `:90-203`) is ported: the read state
      (the value or a placeholder, a hover background, a pencil shown on hover, a text cursor), the
      open state (a field on the value, all of it selected and focused), Enter or leaving keeps,
      Escape reverts, the focus handed back. Left: Ely's `TextInput` and `Input` (Zed's `Editor`
      instead), its theme tokens and focus ring, `tab_stop`, `use_keyed_state` (the tab holds the
      state), and its commit on Enter's release (gpui above).
    - `src/navigation/toc.rs` (`Sections` `:13-34`, `in_view` `:36-41`, `Anchor` `:43-94`,
      `TableOfContents` `:96-249`) is read and not ported: its entries scroll to `Anchor`s that wrap
      each heading inside Ely's own renderer, which `MarkdownElement` has no place for; its marker
      animates through Ely's `motion`; Zed's `ListItem` draws the rows (Zed's own wins). `in_view`
      waits with the section-in-view mark (Out).
    - `src/data_display/records.rs` (`PropertyGroup` and `PropertyGrid`, `:88-222`) is read and not
      ported: its row, a name in a muted column and the editor beside it, is what #645's
      `DescriptionList` port already draws; its folding groups have no use for one frontmatter.

## UI proof
`script/e2e/656-brain-page-outline-and-property-edits.sh` (`compositor sway`: it clicks headings,
values, buttons and rows). Setup writes a scratch vault under `$E2E_WORK/vault`, never the user's
brain (R-D8): `notes/draft-idea` (frontmatter `title: Draft idea`, `type`, `status: open`,
`stars: 3`, `reviewed: false`, `due: 2026-10-10`, `tags` a block list of `idea` and `marley`; a body
of `## Context`, `### Background`, a fenced block holding `# not a heading`, `## Links to
[[people/sam|Sam]]` over a line linking Sam, `## Notes` ("First notes."), `## Plan`, `### Steps`,
forty filler lines, `## Notes` again ("Second notes."), thirty more, `## Why` ("The reason is
here."), and `## Timeline`), `notes/linker` (linking `[[notes/draft-idea|the draft]]`) and
`people/sam` (no headings). It links #643's stand-in as `$E2E_WORK/bin/rusty-mcp`, names it in
`MARLEY_RUSTY_MCP`, points it at the vault and its log, sets `marley.rusty` on with the embedded
connection in the run's settings copy, and binds in the run's keymap Ctrl+Alt+Shift+O to
`["rusty::OpenPage", {"slug": "notes/draft-idea"}]` and Ctrl+Alt+Shift+B to
`outline_panel::ToggleFocus` (L-633). Each write check reads the stand-in's log for the call and the
value's JSON type. Shots:
- `656-01-outline`: Ctrl+Alt+Shift+O: Draft idea in a kept tab; the outline lists Context,
  Background (indented), Links to Sam, Notes, Plan, Steps (indented), Notes, Why, Timeline; `Why`'s
  section is below the fold (L-620).
- `656-02-outline-scroll`: a click on Why: "Why" three lines under the top of the body, "The reason
  is here." under it.
- `656-03-same-name`: a click on the second Notes: "Second notes." under the heading at the top.
- `656-04-outline-hidden`: the header's Outline button: no column, the body wider.
- `656-05-edit-outline-panel`: the button again, Edit, Ctrl+Alt+Shift+B: the source in the editor,
  Zed's outline panel listing its headings, no column in the tab. Then Read, and the panel's dock
  closed.
- `656-06-title-open`: a click on the title: an editor holding "Draft idea", all of it selected; the
  outline column back.
- `656-07-title-set`: "Draft plan", Enter: the title and the tab read Draft plan; the log holds
  `brain_set_property` with `title` and `"Draft plan"`.
- `656-08-title-escape`: a click on the title, "zzz", Escape: Draft plan; no new call in the log.
- `656-09-text`: a click on `status`'s value, "doing", Enter: `doing`; the log holds `"doing"`.
- `656-10-number`: `stars`, "4", Enter: `4`; the log holds `4`, a number.
- `656-11-number-refused`: `stars`, "four", Enter: "Enter a number" under the open editor; no call.
  Then Escape.
- `656-12-date-on-blur`: `due`, "2026-11-01", then a click on the body: `2026-11-01`; the log holds
  `"2026-11-01"`.
- `656-13-checkbox`: `reviewed`'s box: checked; the log holds `true`.
- `656-14-list-add`: `tags`' add button, "rusty", Enter: chips idea, marley, rusty; the log holds
  the three as a list. Then Escape.
- `656-15-list-remove`: the remove button on `idea`: marley, rusty; the log holds the two.
- `656-16-property-removed`: `status`'s Remove property: the row gone; the log holds
  `brain_remove_property` with `status`.
- `656-17-property-added`: Add property, Number, "priority", Enter: `priority` `0` last; the log
  holds `0`, a number.
- `656-18-key-taken`: Add property, Text, "stars", Enter: "A property named stars exists"; no call.
  Then Escape.
- `656-19-no-headings`: a click on Sam in the body: Sam's page, no column, the Outline button
  disabled. Then Alt-Left: Draft plan again, Sam ahead in the history.
- `656-20-renamed`: a click on the header's name, "first-idea", Enter: the header reads
  `notes / first-idea`, the title still Draft plan, the toast says links were updated in 1 page; the
  log holds `brain_rename` from `notes/draft-idea` to `notes/first-idea`.
- `656-21-history-follows`: Alt-Right, then Alt-Left: first-idea's page, not "No page".
- `656-22-rename-refused`: the name, "linker", Enter: a toast with "Already exists: notes/linker";
  the header still reads `first-idea`.
- `656-23-tree-rename-follows`: in the Brain view, `people/sam` renamed to `samuel` through #644's
  menu; then Alt-Right in the tab: `people / samuel` in its header.
- `656-24-not-connected`: `marley.rusty.enabled` set false from outside, then a click on the title:
  no editor; the tab's not-connected line.

## Locked-In Decisions
- D1: **The Read outline is the tab's; the Edit outline is Zed's.** Zed's outline panel lists only
  an editor an item hands it through `act_as::<Editor>`, and its click moves that editor's
  selection, so a Read tab, which draws `MarkdownElement` and holds no editor, cannot feed it. In
  Read the tab draws Rusty's `outline` in a column of its own; in Edit it hands its editor to Zed,
  as Zed's Markdown preview does, and announces each Read and Edit change with `UpdateBreadcrumbs`,
  since the panel looks again only on `ActiveItemChanged` and drops an item whose editor is gone.
  Rejected: a hidden editor over the vault file in Read to feed Zed's panel (a buffer per page only
  for this, clicks moving an unseen cursor, the panel's file header naming the vault path); the
  outline in #646's Knowledge panel (its D4 leaves it here); headings from Zed's parse
  (`parsed_markdown()` is test-only, so a touchpoint) or a second parse in Marley (Rusty's `outline`
  is in the answer the tab already reads).
- D2: **A heading is found by its line, not its slug.** Rusty's `line` counts lines of the body
  after `strip_comments` and `mark_callouts`, which keep every line (`render.rs:285-387`); Marley's
  body is cut by the same rule (#645's `body_of`), so the line's start is a byte offset of the body,
  and #645's pass maps it to the text Zed parses (a wikilink's replacement changes lengths, never
  which line comes first). Rejected: Zed's heading slugs, built from the rendered text with a `-1`
  suffix for a repeat, against Rusty's text as written: 52 headings of the live vault hold a
  wikilink and 72 hold Markdown punctuation, so their slugs would differ, and two pages repeat a
  heading.
- D3: **The heading lands at the top through one small seam in Zed's `markdown`.**
  `Markdown::request_autoscroll_to_top(source_index, cx)` asks what
  `request_autoscroll_to_source_index` asks and sets a flag the element takes with the request; in
  the controlled autoscroll the flag puts the target three lines under the viewport's top wherever
  it is. Every other caller, with no flag, scrolls as Zed does. It extends the existing
  `crates/markdown/src/markdown.rs` row (#530's), with a `// Marley:` comment on the hunk. If #645
  adds a top-aligned request for its heading links first, this ticket uses it and adds no hunk.
  Rejected: Zed's request alone (a heading below the view stops three lines above the bottom edge,
  `markdown.rs:2512-2518`, with its section out of sight); scrolling to the end first, then asking
  Zed's request (a frame of the page's end before the heading); Ely's anchors (D1).
- D4: **The outline column is Zed's `ListItem`s.** Indented by level from the shallowest, as Zed's
  outline panel indents; the top level brighter, as Rusty's `RightPane.qml` draws it; labels from
  `marley_rusty::page::outline_label`, which runs the heading's text through pulldown-cmark with
  Rusty's options and keeps its text and code; on per new tab, kept per tab. The largest outline in
  the live vault has 175 headings, so a plain scrolled list. No section-in-view mark (Out).
- D5: **The title and the name are two edits.** The title edits the `title` property with
  `brain_set_property`; the name in the header renames the page with `brain_rename`. In the live
  vault 642 of the 805 titled pages have a title that is not their file name (a decision titled in a
  sentence over a kebab-case file). Rusty's app renames the file to the typed title
  (`NoteTab.qml:205-209`, `:528`), which would put that sentence into the slug, and Rusty moves the
  title along only when it was the old file name (`mod.rs:2554-2567`), so the title shown would stay
  as it was. Rejected: the title as the rename (Rusty's app and Obsidian, where the title is the
  file name); a rename that also sets the title (two writes and two commits per edit); choosing
  between the two by whether the title equals the name (one gesture, two writes the user cannot
  tell apart). The brief for this ticket called the title's edit a rename; Chad confirms at
  promotion (P1).
- D6: **The in-place editor is Ely's `InlineEdit` on Zed's `Editor`.** A click opens a single-line
  `Editor` with the value set and selected and the focus in it. `menu::Confirm` keeps,
  `menu::Cancel` keeps nothing, and `EditorEvent::Blurred` keeps only while
  `window.is_window_active()`, the project panel's rule, so switching windows commits nothing. Each
  editor's blur commits its own target, found by its entity, so opening a second one commits the
  first. The focus goes back to the tab's handle. A `menu::Cancel` with nothing open propagates
  (L-457). Rejected: `ui_input::InputField` (Prior art); a modal prompt as #600's groups use (it
  edits a name away from where it is drawn).
- D7: **Typed by the value, as Rusty's app types it.** `marley_rusty::page::PropertyKind` from the
  JSON value: an array of strings is a list, a number a number, a boolean a checkbox, a string that
  `chrono::NaiveDate` reads as exactly `%Y-%m-%d` a date, null or any other string text, and an
  object or an array holding a non-string read only. Rusty keeps no kind per name, and its tool
  takes the value's JSON type, so the kind follows the value rather than Obsidian's per-name
  registry. `parse_value(kind, typed)`: text as typed, trimmed; a number as an `i64` if it reads as
  one, else a finite `f64`; a date only as a real calendar date; an unchanged value writes nothing.
  A list item is added as typed, trimmed, unless empty or already in the list. In the live vault:
  946 lists, no nested mapping, 2292 values that are exactly a date and none with a time, six empty
  values, at most 17 properties on a page.
- D8: **Writes go one at a time, and the read waits for them.** Rusty announces a change before it
  answers the call that made it (`mutate`, `main.rs:734-742`), so a read on that signal could ask
  for the old slug before a rename's answer arrives and draw "No page". Each tab queues its writes
  (`SetProperty`, `RemoveProperty`, `Rename`) and sends the next when the last has answered; a
  change signal that comes while one is under way is held, and one read follows the last answer. The
  queue keeps the order of quick edits (a checkbox clicked twice), which two calls in flight at
  once would not promise. The value shows at once; the read shows what Rusty stored; a refusal is a
  toast with Rusty's message, as #644's tree shows one.
- D9: **A rename re-points every Page tab of the workspace.** `rusty::page::follow_rename(workspace,
  report, cx)`, run in `window.defer` (it updates other Page tabs, F-503): in each tab the slug
  shown and every history entry equal to `from` become `to`, or, for a folder, those under `from/`
  move under `to/` (`PageHistory::rename`), and the tab reads again. Tabs are found by the page they
  show now (PR-599), so the opener finds the renamed tab under `to`. #644's tree rename and move
  call it. Rusty's app fixes only the current entry (`NoteTab.qml:360`), so Back can reach a page
  that moved.
- D10: **Add property asks the kind first.** A click on "Add property" opens a menu of the five
  kinds (`PopoverMenu`, which keeps its own focus, L-600); the key's editor opens after the pick, so
  no click on a kind can blur a half-typed key into a write. One write, with Rusty's app's empty
  values and today's local date for a date. A key the page has is refused.
- D11: **No edits without the connection or under Edit.** While #645's not-connected line shows,
  nothing opens; while Edit shows, the header's name is plain text, since a rename would move the
  file under the open buffer.
- D12: **One Ely port, `InlineEdit`.** In `rusty/inline_edit.rs` with Ely's MIT notice; nothing of
  Ely's theme, fonts, icons or `gpui` comes along, and no crate depends on Ely's. `TableOfContents`
  and `PropertyGrid` are read and not ported (Prior art).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a Page tab shows a page with headings in Read, the tab shall list the page's headings beside the body, in order, indented by level. | Shot `656-01-outline` |
| REQ-002 | The outline shall show each heading's text without its Markdown, a wikilink by its alias or target. | Shot `656-01-outline` |
| REQ-003 | WHEN a heading in the outline is clicked, the tab shall scroll the body so that heading sits near its top. | Shot `656-02-outline-scroll` |
| REQ-004 | WHEN the second of two headings with the same text is clicked, the tab shall scroll to the second. | Shot `656-03-same-name` |
| REQ-005 | WHEN the Outline button is pressed, the tab shall hide the outline if it shows and show it if it is hidden. | Shots `656-04-outline-hidden`, `656-06-title-open` |
| REQ-006 | WHILE the page has no headings, the tab shall show no outline and the Outline button disabled. | Shot `656-19-no-headings` |
| REQ-007 | WHILE Edit shows, the tab shall show no outline of its own, and Zed's outline panel shall list the source's headings. | Shot `656-05-edit-outline-panel` |
| REQ-008 | WHEN the title is clicked, the tab shall show it in an editor with all of it selected. | Shot `656-06-title-open` |
| REQ-009 | WHEN Enter is pressed in the title's editor with a new title, the system shall write it with `brain_set_property` as `title`, and the title and the tab's label shall show it. | Shot `656-07-title-set`; the log |
| REQ-010 | WHEN Escape is pressed in one of the tab's editors, the system shall write nothing and show the value as it was. | Shot `656-08-title-escape`; the log |
| REQ-011 | WHEN one of the tab's editors loses the focus while Marley's window is active, the system shall keep its text as Enter does. | Shot `656-12-date-on-blur`; review for an inactive window |
| REQ-012 | WHEN a text value is changed, the system shall write the new text. | Shot `656-09-text`; the log |
| REQ-013 | WHEN a number value is changed to a number, the system shall write it as a JSON number. | Shot `656-10-number`; the log |
| REQ-014 | IF a number or a date value is changed to text of another form, THEN the system shall write nothing and say what form it needs. | Shot `656-11-number-refused`; review for a date |
| REQ-015 | WHEN a date value is changed to a `YYYY-MM-DD` calendar date, the system shall write that text. | Shot `656-12-date-on-blur`; the log |
| REQ-016 | WHEN a checkbox value is clicked, the system shall write the other boolean. | Shot `656-13-checkbox`; the log |
| REQ-017 | WHEN an item is added to a list value, the system shall write the list with the item at its end. | Shot `656-14-list-add`; the log |
| REQ-018 | WHEN an item's remove button is clicked in a list value, the system shall write the list without that item. | Shot `656-15-list-remove`; the log |
| REQ-019 | WHERE a value is an object or a list holding anything but text, the tab shall show it with no editor. | Review |
| REQ-020 | WHEN a property's remove button is clicked, the system shall remove it with `brain_remove_property`. | Shot `656-16-property-removed`; the log |
| REQ-021 | WHEN a property is added with a kind and a new key, the system shall write the key with that kind's empty value. | Shot `656-17-property-added`; the log |
| REQ-022 | IF the key added is already a property of the page, THEN the system shall write nothing and say it exists. | Shot `656-18-key-taken`; the log |
| REQ-023 | WHEN the name in the header is changed, the system shall rename the page in its folder with `brain_rename`, and the tab shall show the new name and say in how many pages Rusty updated links. | Shot `656-20-renamed`; the log |
| REQ-024 | WHEN the typed name holds a `/`, is empty or is unchanged, the system shall send a `-` in the slash's place, or send nothing. | Review |
| REQ-025 | WHEN a page is renamed or moved, from its tab or from the Brain view, every Page tab of the workspace shall follow it in the page it shows and in its history. | Shots `656-21-history-follows`, `656-23-tree-rename-follows` |
| REQ-026 | IF Rusty refuses a write, THEN the system shall show Rusty's message and the value as Rusty holds it. | Shot `656-22-rename-refused` |
| REQ-027 | The tab shall send its writes one at a time in order, and hold Rusty's change signal until the last has answered. | Review |
| REQ-028 | WHILE Marley has no connection to Rusty, the tab's title, name and values shall open no editor. | Shot `656-24-not-connected` |
| REQ-029 | WHILE Edit shows, the header's name shall open no editor. | Review |
| REQ-030 | WHERE a caller of Zed's `markdown` does not ask for the top, the system shall scroll as Zed does. | Review of the hunk |
| REQ-031 | The ported file shall carry Ely's MIT notice, and no crate shall depend on Ely's. | Review; the gate (gate:2, cargo-deny) |

## Phase Plan
- **P1 Plan**: promote after #645 completes (and #643 and #644 before it); re-verify every seam the
  notes cite against the code as shipped: #645's `PageView`, whether its pass keeps the body's lines
  or hands back its splices, whether it added a top-aligned autoscroll for heading links and
  `act_as_type`; #644's `rename_target` and its tree rename and move; the stand-in's `brain_render`
  fields and scalar types; ask the brain (`brain ask`) on D3 and D5; Chad confirms D5 (the title
  edits the property, the name renames) and the `markdown` touchpoint.
- **P2 Code**: the `README.md` marker first; the `markdown.rs` row widened before its hunk; the
  hunk; `marley_rusty::page`'s additions and `chrono` in its manifest; the stand-in's additions; the
  `inline_edit` port; the outline column, its action and button, `act_as_type` and the
  `UpdateBreadcrumbs` event; the title, the name and `follow_rename`; the property editors, remove
  and Add property; the write queue; #644's tree calls; the in-app guide line; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test**: write and run the scenario, read every shot.
- **P4 Complete**: CHANGELOG (Added) and architecture docs (§21), the user docs the notes list, the
  `markdown.rs` row checked, R2b's status and D5 recorded in the plan, ledger capture (§19), the
  brain decision, close the ticket, archive, commit.
