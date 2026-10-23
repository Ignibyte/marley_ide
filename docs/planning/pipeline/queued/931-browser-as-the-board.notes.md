# The Browser tab as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-931-browser-as-the-board.md
- **Pipeline spec:** 931-browser-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11 (Browser and BrowserTools).
- **Classification / tier:** feature, M. Marley crate only (`marley_workbench`); no Zed crate hunk.
- **Recall (§18.3):**
  - AD-claude-496 (picks staged, sent to the last terminal), AD-claude-490 (the tab draws its own
    chrome and dialogs), AD-claude-581 (Clear asks first, deletes the profile), AD-claude-767
    (Take Over and its bar).
  - F-claude-767-take-over-read-the-agent-chips-five-second-record-001: the bar reads
    `last_agent` (`agent_name`), never the chip's five-second record. The new wording keeps that
    source.
  - F-claude-474-the-occluding-buttons-hid-under-the-pointer-001 and
    F-claude-899-a-popover-row-outside-its-list-let-the-click-through-001: what occludes and what
    does not decides where a click goes. The pick mark must not occlude; the page under it takes
    the click.
  - F-claude-496-escape-never-reached-a-browser-page-001: keys in the tab's fields and the page
    are routed by key context; the Note's Enter and the script field's Ctrl Enter go through
    `Field > Editor` contexts.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001: this scenario starts no agent CLI; the
    "agent" is `mcp_agent`, outside any terminal. A New agent choice is not clicked.
- **Discovery:** `crates/marley_workbench/src/browser.rs`: `render_address` (7038),
  `render_drawer` (7170), `console_row` (8517), `render_annotations` (7466) and `annotation_box`
  (9342), `Render for BrowserView` (8621), `render_pick_row` (8191), `render_listener_place`
  (8347), `caption` (6160), `render_pick_options` and `render_to_choice` (8016, 8108),
  `control_bar` (5858), `render_dialog` (7603), `clear_project_browser_data` (10097),
  `toggle_scripts` (6465), `render_new_script` (7905), `end_drag` (6693, the note's editor),
  `render_note_field` (7572), `BrowserView::new` (5269). `marley_workbench.rs` (actions list
  near 356). `crates/marley_workbench/keymap.json` and `keymap-macos.json` (rows 179, 225, 232,
  254).
- **Decisions:** D1–D8 in the spec. D1 (drawer open) changes no #887 decision: #887 D2 made the
  drawer the tab's own and left its first state open to choose.

### Design
- Pick mark: `render_pick_marks(placement, cx) -> Vec<AnyElement>` reads the hub's picks for the
  tab (`pick.tab == target && !pick.dismissed`), keeps those whose `url` matches the page's
  current URL without its fragment (both pass the same redaction, `redact_url`), and draws
  `annotation_box(placement.place(pick.bundle.page_box), accent, false, false)` with an absolute
  badge (`pick_selector` · id) on the accent at its top right. No `.occlude()`, no `.id()`, no
  handlers. `placement.shows(&bounds)` skips a box out of the frame, as `render_annotation` does.
- `pick_selector(bundle: &PickBundle) -> String`: the tag, then `.` and the first token of the
  start tag's `class="…"`, from `bundle.html` up to the first `>`; a free function in
  `browser.rs`, beside `listener_place`.
- Pick card head: a struct `PickHead { selector: String, place: Option<ListenerPlace>, text: String }`
  built in `render_tray` into `TrayRow` (named fields, no tuples, per `.rules`). The
  `ListenerPlace` link moves into the head row, still opening its file.
- Note: `Editor::auto_height(3, 3, window, cx)`; Zed binds Enter to `editor::Newline` in an
  auto-height editor, so the card listens for `editor::Newline` in `MarleyPickCaption` and sends,
  as #903's Home prompt did. The Note takes no newline: Enter sends (D5), and a long note wraps.
- To: `DropdownMenu::new("browser-pick-to", label_of(self.pick_to), ContextMenu::build(...))`
  with two entries setting `pick_to`; the label is `last_terminal_label` or "A new agent".
- Take-over bar: a small `Indicator::dot().color(Color::Success)` before the sentence; the
  button's label row ends with `KeyBinding::for_action_in(&TakeOverBrowser, &self.focus_handle, cx)`.
- Clear-data dialog: count `live_views(cx).filter(|view| view.read(cx).project == key)` before the
  prompt; `clear_project_browser_data` takes `&mut Context<Workspace>`, which derefs to `App`.
  Plural or singular wording by the count.
- New-script field: `NewGlobalPlaywrightScript` in the `actions!` list of `marley_workbench.rs`;
  `render_new_script` handles it with `create_script(Scope::Global, …)`; the hint is a muted
  `Label` "Ctrl Enter: every project" inside the field's row.
- Annotate note: placeholder "A note for this box"; `render_note_field` adds a muted label "Enter
  keeps it · Esc drops it" after the editor, inside the occluding card.
- Files: `crates/marley_workbench/src/browser.rs` (Marley crate),
  `crates/marley_workbench/src/marley_workbench.rs` (Marley crate),
  `crates/marley_workbench/keymap.json`, `crates/marley_workbench/keymap-macos.json` (Marley
  crate). No `docs/marley/zed-touchpoints.md` row: no Zed path changes.

### Visual check plan
- REQ-001: a served `div.card` with a button; Ctrl Shift C, a click on the button's card, pick mode
  ends; `931-01-picked` shows the accent box and the `div.card · 1` badge. `931-09-day` repeats it
  in Marley Day.
- REQ-002: `931-01-picked`, the address box's project chip.
- REQ-003: the page logs one `console.error` at load; the tab opens with the drawer on Console;
  `931-01-picked` shows "Console" and a red 1.
- REQ-004: `931-01-picked`, the error row tinted, the info row not.
- REQ-005, REQ-006: `931-02-card`, the picks panel cropped by reading: the head, the source place
  (the button's click listener in the served `index.html`), the text, the Note's three rows and
  placeholder. Then text typed in the Note and Enter: `931-02b-sent` shows the terminal holding
  the pick's line, which proves Enter sends.
- REQ-007: `931-02-card` the closed dropdown; a click opens it, `931-03-to-open`.
- REQ-008: `mcp_agent click-on button "Save changes"`; `931-04-driving` shows the bar.
- REQ-009: a click on the page's "Reset" button calls `confirm()`; `931-05-asks`; Cancel.
- REQ-010: the palette's `marley: clear project browser data`; `931-06-clear`; Escape (Cancel).
- REQ-011: the toolbar's Scripts button; `931-07-script` shows the field; a name and Ctrl Enter;
  the tray's row reads the script for every project (and its editor tab opens).
- REQ-012: Annotate, a drag on the page; `931-08-note`; Escape drops it.
- Every criterion has a shot; nothing here needs a live remote service.

### Risks
- Enter in the three-row Note: Zed's `Editor && mode == auto_height` binding may win over
  `MarleyPickCaption > Editor`. Listening for `editor::Newline` in the card's context avoids the
  race (#903's prompt).
- Ctrl Enter in a single-line editor: Zed may bind it in `Editor` (`editor::NewlineBelow` is a
  full-mode binding). Code checks which binding wins in `MarleyScriptName > Editor` with
  `debug: open key context view`.
- URL match for the mark: a page that changes only its query keeps the same document in some
  apps. D2 ties the mark to the document; a `within` navigation (`Navigation { within: true }`)
  keeps it.
- The drawer open by default shortens every page by 220 px; earlier scenarios that measured page
  coordinates (#896's `PAGE_Y`) keep working, since the drawer sits under the page, but a page
  taller than the rest is cut sooner.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
