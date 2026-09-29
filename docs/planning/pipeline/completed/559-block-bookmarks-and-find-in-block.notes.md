# Bookmarks on blocks, and find within a block — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-559-block-bookmarks-and-find-in-block.md
- **Pipeline spec:** 559-block-bookmarks-and-find-in-block.spec.md

## Phase 1 — Plan
- **Request:** the Warp blocks note (2026-09-25), recommendation 7: "Bookmarks and find within
  block. S each." Chad, 2026-09-26: every remaining finding gets built.
- **Classification / tier:** feature, S and S. A per-view state module, three actions, a hook's
  marks and buttons, an icon, ticks and an outline in the element, two hunks in the view. Zed
  paths: `terminal_view.rs` and `terminal_element.rs`, both with rows.
- **Recall (§18.3):**
  - AD-claude-473-the-block-keys-scroll-and-select-nothing-001: stage one keeps no selected
    block; the keys count from the viewport's top; the actions live in `marley_workbench` at
    the workspace's root. The jump keys follow the same shape.
  - F-claude-474-the-occluding-buttons-hid-under-the-pointer-001 and
    F-claude-474-a-press-elsewhere-lost-its-release-over-a-button-001: a block's buttons wrap in
    `marley_keep_from_terminal`, which stops the press and not the release; the new buttons take
    the same wrapper.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: two more hover buttons
    shift Copy and Rerun left; no e2e scenario clicks them by coordinates today.
  - PR-claude-history-range-read-needs-over-screen-test-001: a read over a line range must be
    tested with the range partly above the screen; the scope filter is tested with a block that
    is scrolled off.
  - Brain: no page on bookmarks or terminal search (searched 2026-09-26).
- **Promotion, 2026-09-29:** every seam re-read; the line numbers in the spec are today's.
  Since the draft: #554 (the selection, the Block section of the menu), #555 (`MarleyBlockChip`),
  #528 (`block_to_filter`, the selected block first), #558 (`MarleyBlockExtras`, Save as
  Workflow on `IconName::Bookmark`). PR-claude-595's rule applies to every hook the element asks:
  read the context's terminal or a global, never the view. L-claude-555: a scenario clicks rows
  it measured after the step that moves them. Brain (consultation 64748129): nothing on this
  seam.
- **Discovery:**
  - `crates/terminal_view/src/terminal_view.rs`: `MarleyTerminalFooter` (141),
    `MarleyTerminalSuggestion` (150), `dispatch_context` (1026, `Terminal`), the search impl
    (2033 to 2160: `supported_options` 2036, `clear_matches` 2049, `update_matches` 2054,
    `query_suggestion` 2067, `activate_match` 2082, `select_matches` 2095, `find_matches` 2109,
    `active_match_index` after it); no `search_bar_visibility_changed`.
    `crates/workspace/src/searchable.rs`: `SearchOptions` (52), the default
    `search_bar_visibility_changed` (88) and the trait method (297).
  - `crates/terminal/src/terminal.rs`: `Point` and `Range` (492 to 519), `Event` (729),
    `matches` (1617), `blocks` (1845), `last_content` (2134), `total_lines` (2156),
    `viewport_lines` (2160), `activate_match` (2172), `select_matches` (2185),
    `scroll_to_bottom` (2254), `find_matches` (3062).
  - `crates/marley_terminal/src/anchored.rs`: `AnchoredBlock` (38), `BlockSpan` (369),
    `visible_spans` (389), `block_scroll` (427), `bottom_shift` (454).
  - `crates/terminal_view/src/terminal_element.rs`: `LayoutState`'s Marley fields (52 to 57),
    the spans and blocks in prepaint (1618 to 1661), the paint order (1743 to 1857),
    `marley_block_spans` (2182), `marley_rows_bounds` (2199), `marley_gutter_bounds` (2219),
    `marley_wash` (2237), `marley_bar_color` (2247), `marley_pill` (2262),
    `marley_keep_from_terminal` (2311), `marley_block` (2320).
  - `crates/marley_workbench/src/blocks.rs` (the block keys, 15 to 59; `focused_terminal`, 63);
    `notifications.rs:49-51` (forget on release); `agent_events.rs:30` (state per view id).
  - Keys: `keymap.json:17-23`; `default-linux.json:544-545` (`alt-up`/`alt-down` in `Editor`),
    `:692` (`ctrl-shift-f` → `pane::DeploySearch` in `Workspace`), `:709` (`ctrl-shift-b` →
    `outline_panel::ToggleFocus` in `Workspace`), `:1320` (`ctrl-shift-f` → `buffer_search::Deploy`
    in `Terminal`). `crates/icons/src/icons.rs:54` (`Bookmark`).
  - The scrollbar: `terminal_view.rs:44-48` (`TerminalScrollHandle`, `Scrollbars`,
    `WithScrollbar`), `:180` (the handle); `crates/ui/src/components/scrollbar.rs` has no
    markers; the editor's markers are its own (`crates/editor/src/element.rs:6399-6509`).
- **Decisions:** D1 to D6 in the spec.

### Design
- **`MarleyBlockMarks`** (Zed, `terminal_view.rs`, beside `MarleyBlockSelection`): a `Default`
  global of `bookmarks: HashMap<EntityId, BTreeSet<usize>>` and `search_scopes:
  HashMap<EntityId, usize>`, keyed by the terminal's entity id; `bookmarked(terminal, cx)` and
  `search_scope(terminal, cx)` read it. `search_bar_visibility_changed(false)` (new in the view's
  `SearchableItem` impl) removes the terminal's scope and notifies. `find_matches`: when a scope
  is set, the matches of `term.find_matches` are awaited on the background and kept when their
  start's grid line, plus `marley_screen_top`, lies in the block's lines
  (`marley_terminal::block_lines`).
- **The element** (Zed, `terminal_element.rs`): beside #554's outline, the scoped block's outline
  in `text_accent`; after the block elements, a tick per bookmarked block at the right edge, a
  quad 8 × 2 px at `marley_terminal::scrollback_fraction` of the element's height, in
  `text_accent`.
- **`marley_terminal`** (pure, `anchored.rs`): `block_lines(block, cursor_line)`, the absolute
  lines a block spans (`visible_spans`'s rule), and `scrollback_fraction(line, screen_top,
  total_lines, screen_lines)`, where a line sits in the lines that can be scrolled to, or none.
- **`marley_workbench::bookmarks`** (new): `init` sets `MarleyBlockMarks`, `MarleyBlockChip`
  (the bookmark icon, then `send_block::chip`) and `MarleyBlockExtras` (Bookmark, Find, then
  `workflows::block_buttons`), forgets a terminal's marks on its release (`observe_new` on
  `Terminal`), and registers `ToggleBookmark`, `PreviousBookmark`, `NextBookmark` and
  `FindInBlock` at the workspace's root. `toggle` and `find_in` are `pub(crate)` for the block
  menu. Jumping clones the marked blocks and runs `block_scroll` over them, then
  `scroll_to_bottom` and `scroll_up_by`; no mark, or the alternate screen, `cx.propagate()`.
  `find_in` (deferred: the action runs while the workspace is leased) sets or clears the scope,
  dispatches `zed_actions::buffer_search::Deploy::find()` on the view's focus handle when it
  sets one, and emits `SearchEvent::MatchesInvalidated` on the view so an open bar searches again.
- **`send_block.rs`, `workflows.rs`:** their hooks become `pub(crate)` parts; their `set_global`
  calls go. Save as Workflow takes `IconName::Book`.
- **`blocks.rs`:** Toggle Bookmark and Find in Block in the Block section after Send to Agent.
- **Keys** (`keymap.json`): `Terminal`: `ctrl-shift-b` → `marley::ToggleBookmark`, `alt-up` →
  `marley::PreviousBookmark`, `alt-down` → `marley::NextBookmark`; `Terminal &&
  MarleyBlockSelected`: `ctrl-shift-f` → `marley::FindInBlock`.
- **File manifest.** Marley crates: `crates/marley_terminal/src/anchored.rs`,
  `crates/marley_workbench/src/{bookmarks.rs (new), marley_workbench.rs, blocks.rs,
  send_block.rs, workflows.rs}`, `crates/marley_workbench/keymap.json`. Zed paths:
  `crates/terminal_view/src/terminal_view.rs` (the global, the filter, the visibility hunk),
  `crates/terminal_view/src/terminal_element.rs` (the outline, the ticks). Scenario:
  `script/e2e/559-block-bookmarks-and-find-in-block.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: both rows gain their hunks before the code.

### Visual check plan
| REQ | Scenario part | Shot |
|---|---|---|
| REQ-005 | `cat -v`, Alt+Up, Alt+Down, Enter, Ctrl-D, with no mark | `559-00-keys-pass`: `^[[1;3A^[[1;3B` |
| REQ-001, REQ-003 | five blocks; Ctrl+Shift+B at the prompt | `559-01-marked`: the icon before the newest pill, one tick low at the right edge |
| REQ-002, REQ-003 | scroll so the second block's first row shows; hover it; click Bookmark | `559-02-two-marks`: two icons, two ticks |
| REQ-004 | Alt+Down; Alt+Up | `559-03-jumped-down`: the bottom; `559-04-jumped-up`: the marked echo block's first line at the top |
| REQ-001 | Ctrl+Shift+B again | `559-05-unmarked`: the newest block's icon and tick gone |
| REQ-006 | hover the fourth block; click Find; type `needle` | `559-06-find-in-block`: the bar's count 2, the outline |
| REQ-007 | Escape; Ctrl+Shift+F; `needle` | `559-07-whole-terminal`: count 3, no outline |
| REQ-009 | Escape; Ctrl+Up to the fourth block; Ctrl+Shift+F; `needle` | `559-08-selected`: count 2, the outline |

Nothing is out of reach but REQ-008, which holds by construction (the spec says why). The tick's
place is read by eye against the block's place in the scrollback.

### Risks
- Ctrl+Shift+B in `Terminal` shadows Zed's outline panel toggle while a terminal has the focus;
  the palette still reaches it, and Warp's key is the point.
- Alt+Up and Alt+Down are consumed only with a bookmark (D3).
- The scope maps grid lines through the last frame's `marley_screen_top`; output that scrolls
  the grid between that frame and the search moves the window by those lines until the next
  wakeup searches again, which it does on every wakeup.
- A match that starts in the block and runs past it is kept; an evicted block's scope yields
  no match, which the count shows as 0.

## Phase 2 — Code
- **Built:**
  - `terminal_view.rs`: `MarleyBlockMarks` (bookmarks and search scopes by the terminal's id,
    `bookmarked`, `search_scope`); `marley_search_lines`, the scoped block's grid lines through
    the last frame's `marley_screen_top`; in `find_matches`, a scoped search awaits
    `term.find_matches` on the background and keeps the matches whose start's line is in them;
    `search_bar_visibility_changed`, which removes the scope when the bar closes.
  - `terminal_element.rs`: after #554's outline, the scoped block's outline in `text_accent`,
    and an 8 × 2 px tick at the right edge per bookmarked block (`marley_bookmark_ticks`, none on
    the alternate screen).
  - `marley_terminal/src/anchored.rs`: `block_lines` and `scrollback_fraction` (an `f64` from
    exact `u32` conversions, so the Marley crate needs no cast), re-exported.
  - `marley_workbench/src/bookmarks.rs` (new): `init` sets the marks, `MarleyBlockChip` (the
    bookmark, then `send_block::chip`) and `MarleyBlockExtras` (Bookmark, Find, then
    `workflows::block_buttons`), drops a terminal's marks on its release, and registers
    `ToggleBookmark`, `PreviousBookmark`, `NextBookmark` and `FindInBlock`; `is_bookmarked`,
    `toggle` and `find_in` serve the block menu.
  - `send_block.rs` lost its `init`; `workflows.rs`' init no longer sets the extras hook, and Save
    as Workflow is on `IconName::Book`. `blocks.rs`: Bookmark (or Remove Bookmark) and Find in
    Block after Send to Agent. `marley_workbench.rs`: the module, its init, the four actions. The
    keymap: `ctrl-shift-b`, `alt-up`, `alt-down` in `Terminal`; `ctrl-shift-f` in
    `Terminal && MarleyBlockSelected`.
- **Deviations:** the marks are data in a Zed global rather than callbacks (D2), so the view clears
  a scope itself; ticks and the outline are read in `paint`, which reads the view already.
- **Review of the diff:** every hook the element asks reads the terminal or a global, never the
  view (PR-claude-595); `find_in` is deferred, since the action runs while the workspace is
  leased, and `FocusHandle::dispatch_action` runs the bar's Deploy at once from the view's node;
  an open bar searches again on `MatchesInvalidated`; the jump keys propagate with no mark and on
  the alternate screen; a match's line is alacritty's grid line (0 the live screen's top), so
  `marley_screen_top + line` is its absolute line; each Zed hunk is additive.
- **Gate:** run 1 red: rustfmt (the element's `if let`) and two `too_long_first_doc_paragraph`
  in `anchored.rs`. Run 2: GATE GREEN [diff], with the scenario in the tree. Its three warnings
  (`terminal_panel.rs:327`, `terminal_view.rs:1657`, `:2097`) are Zed's own code: pre-existing,
  not in scope.

## Phase 3 — Test
- **Scenario:** `script/e2e/559-block-bookmarks-and-find-in-block.sh` under `compositor sway`,
  run once after `just build`: exit 0, its check passes (`cat -v` printed the keys' sequences).
  Every default coordinate matched the shots (the top row at 77, the printf block's first row at
  292, Bookmark at 1198, Find at 1220), so the scenario is as the gate saw it.
- **Shots, each read:**
  - `559-00-keys-pass` (REQ-005): `cat -v` echoed and printed `^[[1;3A^[[1;3B` twice, with no
    bookmark in the terminal.
  - `559-01-marked` (REQ-001, REQ-003): the bookmark before `seq 61 90`'s pill; one tick at the
    right edge at about 0.69 of the height, the block's place among the kept lines.
  - `559-01a-revealed`: Ctrl+Up four times selected `echo needle one` and put its first line on
    the top row. `559-01b-hover`: its hover row reads Bookmark, Find, Save as Workflow (the book),
    Filter, Copy, Rerun.
  - `559-02-two-marks` (REQ-002, REQ-003): echo's block marked from the button; two ticks, at
    about 0.34 and 0.69.
  - `559-03-jumped-down` (REQ-004): Alt+Down, the bottom (the next mark is on the live screen).
    `559-04-jumped-up`: Alt+Up, echo's first line on the top row.
  - `559-05-unmarked` (REQ-001): Ctrl+Shift+B again, `seq 61 90`'s bookmark and tick gone,
    echo's tick left.
  - `559-05a-hover`, `559-06-find-in-block` (REQ-006): the printf block's Find button; the bar
    open with `needle`, `1/3` (the command's and the two output lines'), the block outlined in the
    accent color.
  - `559-07-whole-terminal` (REQ-007): after Escape, no outline; Ctrl+Shift+F and `needle`,
    `5/5`, echo's two matches counted too.
  - `559-08-selected` (REQ-009): Ctrl+Up twice selected the printf block; Ctrl+Shift+F and
    `needle`, `3/3`, the block outlined.
- **REQ-008:** holds by construction, as the spec says; no shot.
- **Gate:** no source or scenario change since run 2's green.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: bookmarks on blocks, and find within a block);
  `docs/marley/three-prong-plan.md` T1; `docs/marley_architecture/marley_workbench.md` (the new
  section, and #555's and #558's hooks now parts of `bookmarks`' composition) and
  `terminal_blocks.md` (`block_lines`, `scrollback_fraction`); the touchpoint rows for
  `terminal_view.rs` and `terminal_element.rs` describe what shipped.
- **Knowledge:** `AD-claude-559-block-marks-are-data-and-find-in-block-is-zeds-bar-001`,
  `L-claude-559-a-per-block-hook-has-one-slot-so-compose-it-001`. No `F-…` block: the gate's reds
  were formatting and doc length, and the run was green first time. Brain:
  `decisions/marleys-block-bookmarks-and-find-in-block-live-in-a-zed-data-global-and-zeds-own-search-bar`
  on consultation 64748129, follow-up by 2026-10-29.
- **Closed** TICKET-559, archived the pair.
