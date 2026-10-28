# Settings and the keymap as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-941-settings-and-keymap-as-the-board.md
- **Pipeline spec:** 941-settings-and-keymap-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, Settings and Keymap).
- **Classification / tier:** feature, M. Mostly Zed crate hunks (`ui`, `settings_ui`,
  `settings_content`, `settings`, `keymap_editor`), each small, plus one line in
  `marley_workbench`.
- **Recall (§18.3):**
  - #844's scenario opens the Settings window with `marley: open settings` and searches with
    Ctrl+F; #847's searches "kind". Reuse both.
  - F-claude-834 / PR-claude-834 (a flex slot got no width): a tile added before a nav label must
    not squeeze the label; give the tile `flex_none`.
  - F-claude-833 / PR-claude-833 (a status item shrank below its content): same caution for the
    "Find by keys" button in the keymap's search row.
  - Memory note "script/clippy runs cargo-shear and typos repo-wide": new words in titles
    ("Km", "Wl", "Sf", "Pn", "Vc", "Nw", "Dv") may trip `typos`; check its config before Phase 2
    or spell the letters through a table the typos config allows.
  - No failure names `keymap_editor.rs` or `KeybindSource`.
- **Discovery:** `crates/ui/src/components/tree_view_item.rs` (a `start_slot` field and
  builder, drawn before the label); `crates/settings_ui/src/settings_ui.rs` (`render_nav`,
  `marley_row` and its callers); `crates/settings_ui/src/marley_page.rs` (`layout_section`, a new
  `at_the_prompt_section`, the page's section list, 93 titles, a section glyph table);
  `crates/settings_content/src/marley.rs` (`MarleyLayout`); `crates/settings/src/keymap_file.rs`
  (`KeybindSource`); `crates/keymap_editor/src/keymap_editor.rs` (header, source labels, filters,
  override tooltip, the search row, the dialog); `crates/marley_workbench/src/marley_workbench.rs`
  (`load_keymap_from`).
- **Decisions:** D1–D6 in the spec.

### Design
- Tiles: `TreeViewItem::start_slot(impl IntoElement)`; `render_nav` passes, for a root entry,
  `marley_page::section_tile(entry.title, cx)`, a function in Marley's file that looks the title
  up in a `const SECTION_GLYPHS: [SectionGlyph; 16]` (a struct with named fields `title` and
  `letters`) and draws an 18 px rounded tile (`element_background`, muted letters; Marley's with
  the accent's background and `text_accent`), as Home's `glyph_tile` does (#903).
- Marley first: swap `Zed` and `Marley` in `MarleyLayout`; check every `MarleyLayout::VARIANTS`
  and `VariantArray` index use (the dropdown renderer for it, #515) for a hard-coded position.
- `marley_row(json_path)` becomes `marley_row(json_path, page_title)` (or a `MarleyRow` check on
  the window's current page): true for `marley.` keys and for any row on the page titled
  "Marley". Its callers (`render_settings_item_layout`, the segmented hunk) pass the page.
- Agent text full width: a `SettingItem` with `json_path: Some("agent.limit_content_width")`,
  `pick` mapping `Some(&true)` to `Some(&false)` and back through two `static` booleans, `write`
  storing `value.map(|full_width| !full_width)` into `settings_content.agent…limit_content_width`;
  `files: USER`. Its default shows as off (Zed's default is a limited column).
- "At the prompt": a function returning `[SettingsPageItem; 4]` (the header and the three items
  moved out of `layout_section`, which shrinks to six with the new row); the page's section list
  takes it after Layout.
- Sentence case: one pass over `marley_page.rs`'s `title:` strings; the descriptions are not
  touched.
- `KeybindSource::Marley` added between `Base` and `Default` with `name()` "Marley", a `MARLEY`
  meta constant and its `from_meta` arm; `load_keymap_from` sets `KeybindSource::Marley.meta()`.
  In `keymap_editor.rs`: `SourceFilters::allows` counts `Marley` with `zed_defaults`; every match
  on `Default` for editing and overriding treats `Marley` the same; the override tooltip names
  "Marley's keymap"; the From cell uses a `source_label(source)` that reads Yours, Zed, Marley,
  Base, Vim.
- Find by keys: `Button::new("keymap-find-by-keys", "Find by keys").toggle_state(…)` in the
  `IconButton`'s place, same `on_click` and tooltip.
- The dialog: the header's label `format!("New keys for {}", humanized name)`; the conflict
  sentence `format!("Also {}, in {}", humanized name, context)` (the context string the table's
  Where column shows), "and N more" after it; the save button's label from the conflict state.
- File manifest and ledger rows (`docs/marley/zed-touchpoints.md`):
  - `crates/ui/src/components/tree_view_item.rs` (Zed crate): row exists (#895); append "a
    `start_slot` drawn before the label, for the Settings list's tiles (#941)".
  - `crates/settings_ui/src/settings_ui.rs` (Zed crate): row exists; append "the navbar's section
    tiles; `marley_row` true for every row on the Marley page (#941)".
  - `crates/settings_ui/src/marley_page.rs` (Marley's file in a Zed crate): row exists; append
    "the At the prompt section, Agent text full width, sentence-case titles, the section tiles'
    letters (#941)".
  - `crates/settings_content/src/marley.rs` (Marley's file): row exists; append "`MarleyLayout`
    lists Marley first (#941)".
  - `crates/settings/src/keymap_file.rs` (Zed crate): new row, "`KeybindSource::Marley`, between
    Base and Default, so Marley's keymap is its own source; at a merge, keep it between them".
  - `crates/keymap_editor/src/keymap_editor.rs` (Zed crate): new row, "column names, source
    labels, the Marley source in filters and overrides, Find by keys, the record dialog's words
    (#941)".
  - `crates/marley_workbench/src/marley_workbench.rs` (Marley crate): no row.

### Visual check plan
- REQ-001 to REQ-005: `marley: open settings`; `941-01-settings` (the list's tiles; Layout's
  Marley | Zed; Agent text full width; the At the prompt card; titles). `941-06-day` in Marley Day.
- REQ-004: click the switch (place from the first run's shot); `941-02-full-width`; the steps
  print the profile's `settings.json` line to the log.
- REQ-006, REQ-007: `zed: open keymap`; type "filter block", then the user's bound action;
  `941-03-keymap`.
- REQ-008: click Find by keys; `941-04-find-by-keys`.
- REQ-009: open a Marley binding's edit (double-click its row), type keys that another binding
  holds; `941-05-record`; Escape, nothing saved.

### Risks
- #937 renames the System One link's words and #938 relabels `ask_every`, both on the Marley
  page; whichever lands second keeps their words in sentence case.
- `KeybindSource`'s discriminants are the meta indices; inserting `Marley` renumbers `Default`
  and `Unknown`. Any code that stores or compares a raw index (not the enum) breaks; grep for
  `KeyBindingMetaIndex(` across the tree in Phase 2.
- D3's precedence change: a `null` binding in Zed's default keymap on a key Marley binds would
  stop hiding Marley's binding. If Phase 2 finds one, record it and keep the old behaviour by
  giving `Marley` the same rank as `Default` in gpui's comparison (a small hunk), or keep the
  binding Marley wants.
- Seven hunks in Zed crates make this ticket's upstream cost the highest of the batch; each stays
  additive and commented.
- Moving three items out of `layout_section` changes the arrays' lengths; the page's sub-page and
  search index follow the arrays. The window's search matches titles without regard to case, so
  sentence case changes what a result shows, not what it finds; confirm in Phase 2.

## Phase 2 — Code
- **Built:** as the design names.
  - Zed hunks: `tree_view_item.rs` (`start_slot`), `settings_ui.rs` (tiles in `render_nav`;
    `marley_row` or the page's title for the Marley look), `keymap_file.rs` (`KeybindSource::Marley`)
    and `keymap_editor.rs` (filter arm, override words, header, `source_label`, Find by keys, the
    dialog's words).
  - Marley's files in Zed crates: `marley_page.rs` (`PAGE_TITLE`, `SECTION_GLYPHS`,
    `section_tile`, `prompt_section`, Agent text full width, sentence-case row titles) and
    `settings_content/src/marley.rs` (Marley first).
  - Marley crate: `load_keymap_from` binds with `KeybindSource::Marley`, and its test's
    expectation follows.
  - All six ledger rows were written first.
- **D3 check:** Zed's Linux keymap has one `null`, `f5` in "Workspace && debugger_running".
  Marley binds `f5` in "MarleyBrowser" (Browser Reload), which a running debugger used to silence.
  It now reloads, as D3 intends. No other key overlaps.
- **Deviations:**
  - Section headers keep their case (D6 names row titles only). "Component Library" and "Rusty's
    Server" keep their capitals as the names of a tab and a page.
  - "Edit Keystroke" and "Edit Context" are unchanged.
- **Gate:**
  - The first run failed gate:18 on "Pn". `.config/typos.toml`'s `extend-ignore-re` gains
    `letters: "Pn"` with its reason, as it lists `"ba"`.
  - gate:22 failed once on semgrep itself and passed on the rerun.
  - Then GREEN, and again on the final tree.

## Phase 3 — Test
- **Scenario:** `script/e2e/941-settings-and-keymap-as-the-board.sh` (sway).
  - The run's `keymap.json` binds Ctrl+Alt+Shift+K to New File.
  - The copied `agent.limit_content_width` is taken out first, as #822's scenario does.
- **Reds, the scenario's own:**
  - The first full-width check passed on Chad's copied setting before any click. The check now
    runs before the click too, and the row is found by searching "full width".
  - Marley's default is full width (#822), so the switch reads on and the click writes `true`.
  - The record dialog opens from the row's pencil, and its keys field records only after a click.
  - While it records, Escape is taken as keys, so Cancel closes it.
- **Shots:**
  - `941-01-settings`: the tiles (M in the accent, G, Ap, Km, Ed, Ln, Sf, Wl, Pn, Db, Tm, Vc, Co,
    AI, Nw, Dv), Layout Marley | Zed, and sentence-case rows.
  - `941-02-found`: Agent text full width on, with its key `agent.limit_content_width`.
    `941-02-full-width`: off, "changed · reset"; both checks passed.
  - `941-03-keymap`: Action / Arguments / Keys / Where / From, with Ctrl-Alt-Shift-K "Yours" and
    Zed's "Zed". `941-03-keymap-marley`: Browser Reload's F5 and Ctrl-R, "Marley".
  - `941-04-find-by-keys`: Find by keys pressed, keystroke search on.
  - `941-05-record`: "New keys for marley: browser reload", Ctrl-L recorded, "Also marley: focus
    address bar, in MarleyBrowser", Cancel and Save Anyway.
  - `941-06-day-keymap`, `941-06-day`: the keymap and Settings in Marley Day.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`, `docs/marley/look-plan.md`, the architecture note, and the six
  ledger rows.
- **Knowledge:** L-claude-941-a-scenario-shows-marleys-default-only-after-clearing-the-copied-setting-001.
- **Brain:** consultation 3f5528f65ff84b7b8e11d69a6bac6185 closed with `brain no-decision`.
- **Ticket:** closed.
