---
pipeline_id: 403a657f-bdd1-45e4-9ac1-41662b37cd2c
ticket: docs/planning/tickets/open/TICKET-559-block-bookmarks-and-find-in-block.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Bookmarks on blocks, and find within a block"
type: feature
slice: prong 1 T1 (stage-one block actions); the Warp blocks note, recommendation 7
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/pipeline/queued/528-block-filter.spec.md, docs/planning/pipeline/completed/473-block-navigation-keys.spec.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md, docs/zed_architecture/crates/search.md]
---

## Title
Warp lets you bookmark a block for the session and jump between bookmarks, and lets you find
within one block instead of the whole session. Marley has neither: Ctrl+Up and Ctrl+Down walk
every block, and Zed's terminal search runs over all the scrollback. Both land here without a
block selection, on the newest block in view and on the hovered block, and take the selection
when its ticket lands.

## Scope
### In
- **Bookmarks.** A set of block indices per terminal view, kept for the session in
  `marley_workbench::bookmarks` and dropped when the view goes. `marley::ToggleBookmark` on
  Ctrl+Shift+B in `Terminal` toggles the newest block with a row in view (#528's rule for a
  keyboard action without a selection); a Bookmark button on a hovered block toggles that block
  (through the `MarleyBlockExtras` hook, #556 D7). A marked block shows `IconName::Bookmark`
  before its pill, always, and the terminal's right edge shows a tick per marked block at its
  place in the scrollback (its first line over the lines that can be scrolled to), painted by the
  element before Zed's scrollbar, so the ticks show beside the thumb.
- **Jumping.** `marley::PreviousBookmark` and `marley::NextBookmark` on Alt+Up and Alt+Down in
  `Terminal`: the focused terminal scrolls so the previous or next marked block's first line is at
  the top, counted from the viewport's top line as the block keys count (`block_scroll` over the
  marked blocks). In a terminal with no bookmark the keys reach the program, as → does with no
  suggestion.
- **Find within a block.** `marley::FindInBlock` (the palette; on the newest block with a row in
  view) and a Find button on a hovered block set the view's search scope to that block, the lines
  from its prompt line (or its output's first line) to its output's end (or the cursor's line
  while it runs), and deploy Zed's search bar with the focus in its query. While a scope is set,
  `TerminalView::find_matches` keeps only the matches whose start lies in those lines, so the
  bar's count and Enter walk the block alone, and the scoped block shows an outline in the
  accent color. Closing the bar (Escape) clears the scope; `marley::FindInBlock` on the scoped
  block clears it too and keeps the bar. The scope is kept per view for the session.
- Two hooks in Zed's terminal view for the scope: a read of it in `find_matches`, and
  `search_bar_visibility_changed` implemented to tell the workbench the bar closed; the marks and
  the buttons ride `MarleyBlockExtras`.

### Out (explicitly deferred)
- A block selection and the actions on it (recommendation 1, its own ticket): when it lands,
  Ctrl+Shift+B, Ctrl+Shift+F and Find act on the selected block first, and the newest block in
  view second.
- Bookmarks that survive a restart (Warp's end with the session too), and marks on the rail row.
- A case-sensitivity toggle for the terminal's search (Zed's `supported_options` has none for
  terminals; a `(?i)` wrap in `regex_search_for_query` is a small Zed hunk for a later ticket).
- Warp's hover snapshot on a tick (the prompt, the command and the last two lines of output).
- A block context menu with Toggle Bookmark and Find in Block (the menu is recommendation 1's).

## Reference (§20)
- **Warp, block actions** (https://docs.warp.dev/terminal/blocks/block-actions/, read
  2026-09-26): "Toggle bookmark" in the block's menu, `Ctrl+Shift+B` on Linux, `Alt+↑` and
  `Alt+↓` to move between bookmarks, an indicator whose "position reflects the approximate
  position of the Block in the Block history", and "Bookmarks only persist while the session is
  open, once you close the session they are lost." Find (https://docs.warp.dev/terminal/blocks/find/):
  `Ctrl+Shift+F`, "searches for matches in all your Blocks from the bottom up", scoped to the
  pane, with a per-block option in the find modal and regex and case toggles. Marley keeps the
  key, the jump keys, the indicator on the scrollbar's side, the session-only life, and the
  per-block scope on Zed's search bar; the case toggle waits. The behavior map
  `docs/warp_architecture/subsystems/03-terminal-session-core.md` was read for the shape of
  blocks only; no Warp code.
- **Upstream Zed:** the terminal's search (`SearchableItem for TerminalView`, `buffer_search`),
  kept: the bar, its regex toggle, `find_matches` on the background, `activate_match` selecting
  and scrolling. Zed's own `Ctrl+Shift+F` in `Terminal` deploys the bar over the whole terminal
  and stays; Ctrl+Shift+B in `Workspace` toggles the outline panel, which a `Terminal` binding
  beats while a terminal has the focus.

### Prior art
- **Behavior maps and reports.** The Warp blocks note's table (Toggle bookmark, Find within
  block, their keys and Marley's gaps) and its "Bookmarks and find within block" paragraph
  (a set of block indices per terminal, `Alt+↑/↓`, a mark on the pill, markers on the scrollbar;
  the search filtered on the block's absolute line range, one touchpoint). #528's D4 (the newest
  block with a row in view for a keyboard action without a selection) and its two-hook shape.
  AD-claude-473 (the block keys select nothing; stage one keeps no selected block).
  `docs/zed_architecture/crates/search.md` (the search crate's shape).
- **Published material.** Warp's docs above. Zed's docs on the terminal's search bar
  (zed.dev/docs/terminal).
- **The code we already ship.**
  - The search: `SearchableItem for TerminalView` (`crates/terminal_view/src/terminal_view.rs:2033`):
    `supported_options` (`:2036`, `case: false`, `regex: true`, `selection: false`),
    `update_matches` (`:2054`), `activate_match` (`:2082`), `select_matches`, `find_matches`
    (`:2109`, `regex_search_for_query` then `Terminal::find_matches`);
    `search_bar_visibility_changed` is the trait's default (`crates/workspace/src/searchable.rs:88`,
    `:297`), so an implementation is additive. `Terminal::find_matches`
    (`crates/terminal/src/terminal.rs:3062`, `search_matches` over the locked grid on the
    background), `matches` (`:1617`), `activate_match` (`:2172`, a selection and a
    `ScrollToPoint`), `select_matches` (`:2185`); a match is a `Range` of `Point`s (`:498`) whose
    `line` is the grid's, negative above the screen, so its absolute line is
    `marley_screen_top + line`.
  - The blocks: `AnchoredBlock` (`anchored.rs:38`: `prompt_line`, `output_start`, `output_end`),
    `block_scroll` (`:427`, from the viewport's top over a slice of blocks, reusable over the
    marked ones), `visible_spans` (`:389`), `Terminal::blocks` (`terminal.rs:1845`) and
    `last_content` (`:2134`, `marley_screen_top`, `display_offset`, `total_lines`,
    `screen_lines`). The block keys' action route: `blocks::scroll_to_block` (`blocks.rs:31`) and
    `focused_terminal` (`:63`), caught at the workspace's root.
  - The element: `marley_block` (`terminal_element.rs:2320`), the pill (`:2262`), the actions row
    (`:2366`), the paint order (the washes at `:1772`, the gutter bars at `:1818`, the block
    elements at `:1829`, the suggestion at `:1833`); `marley_rows_bounds` (`:2199`) for the
    outline; the scrollbar is Zed's `WithScrollbar` over `TerminalScrollHandle`
    (`terminal_view.rs:44`, `:180`), and `ui`'s scrollbar draws no markers (the editor paints its
    own in `crates/editor/src/element.rs:6399`), so the ticks are Marley's paint at the element's
    right edge.
  - Keys: Marley's `keymap.json:17-23` (`Terminal`: `secondary-up`, `secondary-down`, `ctrl-g`,
    `right`); Zed binds `ctrl-shift-b` in `Workspace` (`default-linux.json:709`,
    `outline_panel::ToggleFocus`), `ctrl-shift-f` in `Terminal` (`:1320`, `buffer_search::Deploy`)
    and `alt-up`/`alt-down` in `Editor` only (`:544-545`). `IconName::Bookmark` exists
    (`crates/icons/src/icons.rs:54`).
  - Per-view state: `agent_events` keys seats by the view's `EntityId` and forgets them on
    release (`notifications.rs:49-51`); the same shape for bookmarks and scopes.
  - Does a crate we build own this seam? `search` and `terminal_view` own the bar and the
    matches; the blocks own the lines; nothing owns marks or a scope, which are small and
    Marley's.

## UI proof
UI-AFFECTING. `script/e2e/559-block-bookmarks-and-find-in-block.sh` (`compositor sway`, for
the hovered block's buttons). Fixtures: a scratch repository; the scenario's bash; five blocks
typed (`seq 1 40`, `echo needle one`, `seq 41 80`, `printf 'needle two\nneedle three\n'`,
`seq 81 120`), enough to scroll. Shots: Ctrl+Shift+B at the prompt, the newest block marked
(`559-01-marked`); the pointer on the second block's first row, its Bookmark button, a click, two
marks and two ticks (`559-02-two-marks`); Alt+Up twice, the second block's first line at the top
(`559-03-jumped-up`); Alt+Down, the fifth block at the top (`559-04-jumped-down`); Ctrl+Shift+B on
the newest block again, its mark gone (`559-05-unmarked`); the pointer on the fourth block, its
Find button, a click, `needle` typed, the bar's count `2` and the block outlined
(`559-06-find-in-block`); Escape, the outline gone, `ctrl-shift-f` and `needle` typed, the count
`3` (`559-07-whole-terminal`); a quit and a launch on the same profile, no mark
(`559-08-session-only`). Machine checks: the search bar's count read from the shot's debug
selector where the runner can, else the shots.

## Locked-In Decisions
- D1: No selection: the keys act on the newest block with a row in view (#528's rule), the
  buttons on the hovered block, and the selection ticket puts its block first when it lands.
- D2: Bookmarks live in `marley_workbench`, per view, for the session; the element learns of
  them through `MarleyBlockExtras`, not through the `Terminal`. A view whose terminal is replaced
  (a task rerun) keeps nothing, and a restore starts clean, as Warp does.
- D3: The jump keys are consumed only in a terminal with a bookmark; elsewhere they reach the
  program, so a TUI that reads Alt+Up keeps it until the user marks a block there.
- D4: Find within a block is Zed's search bar with a scope, not a second search: the scope filters
  `find_matches` by the block's absolute lines, the bar's own count, Enter and Shift+Enter do the
  rest, and Escape clears it. Ctrl+Shift+F stays Zed's whole-terminal search.
- D5: The ticks are painted by the element, not by `ui`'s scrollbar: adding markers to the shared
  scrollbar would be a Zed change every scrollbar user sees.
- D6: Two additive hunks in `terminal_view.rs` (the scope read in `find_matches`, an
  implementation of `search_bar_visibility_changed`) and none in `terminal.rs`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses Ctrl+Shift+B in a terminal, the system shall toggle a bookmark on the newest block with a row in view, showing the icon before its pill. | Shots `559-01-marked`, `559-05-unmarked` |
| REQ-002 | WHEN the user clicks a hovered block's Bookmark button, the system shall toggle that block's bookmark. | Shot `559-02-two-marks` |
| REQ-003 | WHILE a terminal has bookmarks, its right edge shall show a tick per marked block at its place in the scrollback. | Shot `559-02-two-marks` |
| REQ-004 | WHEN the user presses Alt+Up or Alt+Down in a terminal with bookmarks, the system shall scroll so the previous or next marked block's first line is at the top. | Shots `559-03-jumped-up`, `559-04-jumped-down` |
| REQ-005 | WHILE a terminal has no bookmark, Alt+Up and Alt+Down shall reach the program. | The log: the hook log shows the keys' bytes at the prompt |
| REQ-006 | WHEN the user clicks a hovered block's Find button, the system shall open the search bar scoped to that block, outline the block, and count only its matches. | Shot `559-06-find-in-block` |
| REQ-007 | WHEN the search bar closes, the system shall clear the scope, and a later Ctrl+Shift+F shall search the whole terminal. | Shot `559-07-whole-terminal` |
| REQ-008 | WHEN Marley restarts, no bookmark shall remain. | Shot `559-08-session-only` |
| REQ-009 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, check whether the selection ticket and `MarleyBlockExtras`
  (#556 D7) have landed and take the selection as the first target; ask the brain.
- **P2 Code:** the ledger rows first (`crates/terminal_view/src/terminal_view.rs` and
  `terminal_element.rs`); `marley_workbench::bookmarks` (the sets, the scope, the actions, the
  hook's marks and buttons); the element's icon, ticks and outline; the two hunks in the view;
  the keymap; fmt and clippy clean; a review of the diff.
- **P3 Test:** write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley/three-prong-plan.md` (T1's status);
  `docs/marley_architecture/marley_workbench.md`; the ledger capture; close the ticket, archive,
  commit.
