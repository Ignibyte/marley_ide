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
- **`marley_workbench::bookmarks`** (new): a `Global` `Bookmarks { by_view: HashMap<EntityId,
  ViewMarks { marked: BTreeSet<usize>, scope: Option<usize> }> }`, forgotten on the view's
  release (`observe_new` on `TerminalView` + `on_release`, as `notifications::init` does).
  Actions on every workspace (as `blocks::init` registers): `ToggleBookmark`, `PreviousBookmark`,
  `NextBookmark`, `FindInBlock`. "The newest block with a row in view": the last span of
  `visible_spans` over the current content. Jumping: `block_scroll` over the marked blocks
  (cloned into a `Vec<AnchoredBlock>` in index order), then `scroll_to_bottom` + `scroll_up_by`,
  as `scroll_to_block` does; no marked block → `cx.propagate()`.
- **The hook.** `MarleyBlockExtras::marks(terminal, cx) -> BlockMarks` (`HashMap<usize,
  Marks { bookmarked: bool, agent: bool, scoped: bool }>`) and `buttons(terminal, block, cx)`
  returning Bookmark (`IconName::Bookmark`, tooltip "Bookmark" or "Remove Bookmark") and Find
  (`IconName::MagnifyingGlass`, "Find in Block") for finished and running blocks alike. The
  element: the icon before the pill for `bookmarked`; an outline (`status().info`, one pixel,
  over `marley_rows_bounds`) for `scoped`; the ticks: for each marked block whose `output_start`
  is at or above the evicted count, a 2 × 6 px quad at the right edge at
  `(first_line − evicted) / (history + screen_lines)` of the element's height, painted after the
  gutter bars and before the block elements (Zed's scrollbar paints after the element, over it).
- **The scope.** `FindInBlock` sets `scope = Some(index)` (or clears it when it is that block),
  then `window.dispatch_action(buffer_search::Deploy::find().boxed_clone(), cx)` on the
  terminal's focus handle. `TerminalView::find_matches` (Zed, one hunk): after
  `term.find_matches`, if a `MarleyFindScope` global gives `Some((first, last))` for this view,
  keep the matches whose `start().line` mapped to an absolute line (`marley_screen_top + line`)
  lies in `first..=last`; the scope's lines: `prompt_line.unwrap_or(output_start)` to
  `output_end.unwrap_or(cursor line)`. `search_bar_visibility_changed(false)` (Zed, the second
  hunk) calls the same global's `closed(view)`, which clears the scope; the global is defined in
  `terminal_view` beside `MarleyTerminalSuggestion` and set by `bookmarks::init`. The bar's
  count comes for free.
- **Keys** (`keymap.json`, `Terminal`): `ctrl-shift-b` → `marley::ToggleBookmark`, `alt-up` →
  `marley::PreviousBookmark`, `alt-down` → `marley::NextBookmark`.
- **File manifest.** Marley crates: `crates/marley_workbench/src/{bookmarks.rs (new),
  marley_workbench.rs}`, `crates/marley_workbench/keymap.json`. Zed paths:
  `crates/terminal_view/src/terminal_view.rs` (`MarleyFindScope`, the filter, the visibility
  hook), `crates/terminal_view/src/terminal_element.rs` (`MarleyBlockExtras` if new, the icon,
  the outline, the ticks). Scripts: `script/e2e/559-block-bookmarks-and-find-in-block.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: both rows gain their hunks.

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | five blocks typed; Ctrl+Shift+B; later Ctrl+Shift+B again | `559-01-marked`, `559-05-unmarked` |
| REQ-002 | `pointer_to` the second block's first row; click Bookmark | `559-02-two-marks` |
| REQ-003 | the same shot: two ticks at the right edge, one high, one low | `559-02-two-marks` |
| REQ-004 | Alt+Up twice; Alt+Down | `559-03-jumped-up` (the second block's command row at the top), `559-04-jumped-down` |
| REQ-005 | in a fresh terminal (Ctrl-Shift-T… the rail's New Terminal) with no mark: Alt+Up, Alt+Down | the log: the hook log's frames show nothing typed by Marley; the shot shows the prompt unchanged |
| REQ-006 | `pointer_to` the fourth block; click Find; type `needle` | `559-06-find-in-block`: count 2, the outline |
| REQ-007 | Escape; `ctrl-shift-f`; type `needle` | `559-07-whole-terminal`: count 3, no outline |
| REQ-008 | `quit_marley`; `launch_marley` | `559-08-session-only` |

Not reachable by a scenario: the exact pixel place of a tick (read by eye against the block's
place in the scrollback); the count's text is read from the shot unless the bar exposes a debug
selector the runner can read.

### Risks
- Ctrl+Shift+B in `Terminal` shadows Zed's outline panel toggle while a terminal has the focus;
  the palette still reaches it, and Warp's key is the point.
- Alt+Up and Alt+Down are consumed only with a bookmark; a user who marks a block in a terminal
  running a TUI that reads them loses them there until the mark goes. D3 accepts it.
- A match that starts on the block's last line and runs past it is kept (its start is in
  scope); a match starting on the prompt line before the command is kept too. Both are Warp's
  behavior as far as its docs say.
- The scope's block can be rewrapped (#544) or evicted; the filter reads the block's current
  lines each time, and an evicted block yields no matches, which the count shows as 0.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
