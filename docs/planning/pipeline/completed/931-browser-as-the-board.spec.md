---
pipeline_id: cabf4871-eef6-4f7c-b904-19d2bcb0ea7e
ticket: docs/planning/tickets/open/TICKET-931-browser-as-the-board.md
status: Phase 4 — Complete PASS
title: The Browser tab as the board draws it
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/F-tools.md, docs/marley/design-review-2026-10-23/README.md]
---

## Title
The Browser tab drawn as the Browser and BrowserTools boards draw it: a mark that stays on a
picked element, the project as a chip in the address box, a console drawer that is open under
the page with its error count and tinted error rows, a pick card headed by its selector and
source line with a three-row Note, To as a dropdown, and the boards' words on the take-over bar,
the page's dialog, the clear-data dialog, the new-script field and the annotate note.

## Scope
### In
All in `crates/marley_workbench/src/browser.rs` (Marley crate) and `crates/marley_workbench/keymap.json`
/ `keymap-macos.json` (Marley crate), unless named otherwise.
- **Pick mark.** `render_pick_marks`, new, beside `render_annotations`: each pick of the tab still
  in the tray, while the page shows the document it was picked in, drawn over the frame at its
  `bundle.page_box` through `Placement::place` as an accent box with a badge at its top right
  (`div.card · 2`: the selector chip text and the pick's id). It takes no clicks. Called from
  `Render for BrowserView` next to `annotations`.
- **Selector chip text.** `pick_selector`, new: the element's tag and first class, read from the
  start tag of `bundle.html`; the tag alone when it has no class.
- **Project chip.** `render_address`: the project name as a `Chip` with the accent text on an
  accent tint, not a muted `Label`.
- **Console drawer open.** `BrowserView::new` and `restore` start with `drawer:
  Some(Drawer::Console)`; the toolbar's Console button and the drawer's close still put it away
  for the tab.
- **Console tab count.** `render_drawer`: the Console tab reads "Console", then the error count in
  the error colour when there are errors (`is_error`), as the toolbar button counts (#887 D1).
- **Error rows.** `console_row`: an error or exception row sits on the error colour at a low
  opacity, rounded, as the board's tinted row.
- **Pick card head.** `render_pick_row`: the selector chip (`pick_selector` · id) on an accent
  tint, then the source place (`listener_place`'s label, or the React component's file and line),
  then the element's text (`summary`) on the line under; the discard stays at the right.
  `render_listener_place` no longer takes a row of its own while its place is in the head.
- **Note.** `caption`: a three-row auto-height editor with the placeholder "What should change
  here?"; Enter still sends the pick (#896 D4).
- **To.** `render_pick_options`: a `DropdownMenu` holding the terminal used last (its
  `last_terminal_label`) and "A new agent", in place of the two `render_to_choice` rows.
- **Take-over bar.** `control_bar`: "<who> is driving this page" after a live dot, and "Take
  Over" with its key (`KeyBinding` for `TakeOverBrowser`, Ctrl I). The taken state's words stay.
- **Page dialog.** `render_dialog`: "<origin> asks", or "This page asks".
- **Clear-data dialog.** `clear_project_browser_data`: "Clear <project>'s browser data?" and
  "Its N Browser tabs close, and every site signs out. Other projects keep theirs." N counts the
  project's live Browser tabs (`live_views` filtered by project, as `close_project_tabs` does);
  one tab reads "Its Browser tab closes".
- **New-script field.** `toggle_scripts`, `render_new_script`: one field, placeholder "New script
  name, then Enter"; Enter keeps it for this project, Ctrl Enter for every project
  (`NewGlobalPlaywrightScript`, new action in `marley_workbench.rs`, bound in the
  `MarleyScriptName > Editor` context); a muted hint at the field's end names Ctrl Enter. The
  two buttons go.
- **Annotate note.** `end_drag`, `render_note_field`: placeholder "A note for this box", and the
  hint "Enter keeps it · Esc drops it" beside the field.

### Out (explicitly deferred)
- The bottom row for a Browser tab (title chip, address, console errors, picks ready, recording):
  #909 (the bottom row for the other screens).
- A choice among several agent terminals in To: #896 Out; the dropdown holds the terminal used
  last and a new agent.
- Record This's timeline and Draft a test, and each script's last run and result: #932.
- The tab and rail rows' Browser state dots: #827 Out, and the tab-tile ticket of build order
  item 1.

## Reference (§20)
N/A — Marley-specific: Marley's own Browser tab (#488, #496, #498, #767, #879, #887, #896) and the
Browser and BrowserTools boards. Zed's `ui::DropdownMenu`, `ui::Chip` and `ui::KeyBinding` draw the
dropdown, the chip and the key, as Zed's own settings pages use them.

### Prior art
- `browser.rs:7038` `render_address`: the project is `Label::new(project.name).color(Muted)`.
- `browser.rs:7133` `render_console_button` already counts errors (#887 D1);
  `browser.rs:7222` the drawer's Console tab counts `console.len()`; `browser.rs:8517`
  `console_row` colours only the level word.
- `browser.rs:5085` `drawer: Option<Drawer>`, `None` at start; #887 D2: the drawer is the tab's own.
- `browser.rs:7466` `render_annotations`, `render_annotation`, `annotation_box` (`:9342`) and
  `Placement::place` draw a box in page coordinates that follows scroll and zoom; the pick's
  `bundle.page_box` (`marley_browser/src/pick.rs:510`) is the same `PageBox`. Today a pick is
  marked only by Chromium's inspect highlight while pick mode is on (`pick.rs:842-871`).
- `browser.rs:8191` `render_pick_row` heads with "Pick {id}" and the summary;
  `render_listener_place` (`:8347`) and `listener_place` (`:9892`) give the source place.
  `PickBundle::summary` (`pick.rs:825`); the pick script's CSS path takes two classes
  (`pick.rs:175`).
- `browser.rs:6160` `caption`: `Editor::single_line`, "What should the agent know? Enter
  sends"; `keymap.json:225` binds Enter to `marley::SendPick` in `MarleyPickCaption > Editor`.
- `browser.rs:8108` `render_to_choice`, `PickTo`; `ui::DropdownMenu` (`ui/src/components/dropdown_menu.rs:22`),
  used by `rusty/memory_tab.rs`.
- `browser.rs:5858` `control_bar`: "<who> acted in this page" and a plain Take Over;
  `keymap.json:179` binds Ctrl I to `marley::TakeOverBrowser` in `MarleyBrowser`.
- `browser.rs:7616` `render_dialog`: "<origin> says".
- `browser.rs:10097` `clear_project_browser_data`; `close_project_tabs` (`:4987`) and
  `live_views` find the project's tabs.
- `browser.rs:6465` `toggle_scripts` ("A new script's name"), `:7905` `render_new_script` with
  "For This Project" and "For All Projects"; `keymap.json:232`; `playwright_scripts::Scope`.
- `browser.rs:6710`: the annotate note's placeholder "A note for this box; Enter keeps it";
  `keymap.json:254` binds Escape to `DropAnnotation`.
- docs/zed_architecture/ and docs/warp_architecture/: nothing on this seam (Marley's own tab).

## UI proof
Scenario `script/e2e/931-browser-as-the-board.sh` (sway, `browser-fixture.sh`): an offline
Chromium; a served page with a `div.card` holding a button, a `console.error`, and a button that
calls `confirm()`; the terminal used first, its `.bashrc` setting the prompt only (no agent is
started); `mcp_agent click-on` acts as an agent in the page.
- `931-01-picked`: the card picked (Ctrl Shift C, a click), pick mode off: the accent mark and
  badge on the page, the drawer open with "Console 1" in red and the error row tinted, the
  project chip in the address box (REQ-001, REQ-002, REQ-003, REQ-004).
- `931-02-card`: the picks panel: the card's head (selector chip, source place, text), the
  three-row Note with its placeholder, and To as a dropdown (REQ-005, REQ-006, REQ-007).
- `931-02b-sent`: text in the Note and Enter: the terminal holds the pick's line (REQ-006).
- `931-03-to-open`: the dropdown opened: the terminal used last and A new agent (REQ-007).
- `931-04-driving`: after `mcp_agent click-on`: the take-over bar's words, dot and Ctrl I
  (REQ-008).
- `931-05-asks`: the page's `confirm()`: "<origin> asks" (REQ-009).
- `931-06-clear`: `marley: clear project browser data`: the dialog's title, tab count and "Other
  projects keep theirs"; Cancel (REQ-010).
- `931-07-script`: the Scripts tray: the one field with its placeholder and Ctrl Enter hint; a
  name and Ctrl Enter: the script kept for every project (REQ-011).
- `931-08-note`: Annotate, a box dragged: the note's placeholder and its "Enter keeps it · Esc
  drops it" hint (REQ-012).
- `931-09-day`: Marley Day, the picked card and the panel again (REQ-001 to REQ-007).

## Locked-In Decisions
- D1 — The drawer starts open on Console in every Browser tab, as the board draws it, so a page's
  errors show without a click. The Console button and the drawer's close still put it away for
  that tab (#887 D2). It costs the page 220 px; a tab the user closed it in stays closed.
- D2 — A pick's mark stays while the pick is in the tray (sent or not) and the page shows the
  document it was picked in; discarding the pick or leaving the document takes it away. It never
  takes a click: the page under it stays usable.
- D3 — The mark sits where the element was at the pick (`bundle.page_box`); a page that moves the
  element later is what Check (#505) is for.
- D4 — The selector chip is the tag and first class from the element's own start tag, then
  " · " and the pick's id: short, and what the board draws. The full locators stay in the agent's
  `browser_pick`.
- D5 — The Note is three rows; Enter still sends that pick alone (#896 D4).
- D6 — The new-script field keeps both scopes: Enter keeps it for this project (the common case,
  and what Enter does today), Ctrl Enter for every project, named by a muted hint at the field's
  end. The two buttons go, as the board draws one field.
- D7 — The take-over bar names the agent that acts ("Claude Code is driving this page"); Marley
  knows it, and the board's "Agents" is its placeholder.
- D8 — Every other behaviour stays: To's two targets (#896), the dialog's Cancel and OK (#887),
  Clear's ask-first and its deletion (AD-claude-581).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a pick is in the tray and its page shows the document it was picked in, the tab shall draw an accent box on the element with a badge naming its tag, class and pick id, in Marley Night and Marley Day. | `931-01-picked`, `931-09-day` |
| REQ-002 | The address box shall show the tab's project as an accent chip. | `931-01-picked` |
| REQ-003 | WHEN a Browser tab opens, its drawer shall be open on Console, and its Console tab shall show the error count in the error colour. | `931-01-picked` |
| REQ-004 | The console drawer shall draw an error or exception row on a tinted background. | `931-01-picked` |
| REQ-005 | A pick's card shall lead with its selector chip and pick id, then its source place, then the element's text. | `931-02-card`, `931-09-day` |
| REQ-006 | A pick's Note shall be three rows tall with the placeholder "What should change here?", and Enter shall send that pick. | `931-02-card`, `931-02b-sent` |
| REQ-007 | The picks panel's To shall be a dropdown holding the terminal used last and A new agent. | `931-02-card`, `931-03-to-open` |
| REQ-008 | WHILE an agent has acted in the page and the user has not taken it, the bar shall read "<agent> is driving this page" after a live dot, with Take Over and its Ctrl I key. | `931-04-driving` |
| REQ-009 | WHEN the page shows its own alert, confirm or prompt dialog, the card's title shall read "<origin> asks". | `931-05-asks` |
| REQ-010 | WHEN the user asks to clear a project's browser data, the dialog shall name the project, count its open Browser tabs and say other projects keep theirs. | `931-06-clear` |
| REQ-011 | The scripts tray shall offer one field, "New script name, then Enter", where Enter keeps the script for this project and Ctrl Enter for every project. | `931-07-script` |
| REQ-012 | WHILE the user writes an annotation's note, the field shall show the hint "Enter keeps it · Esc drops it". | `931-08-note` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the In items in `browser.rs`, the new action and its keymap rows; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
