---
pipeline_id: 403a657f-bdd1-45e4-9ac1-41662b37cd2c
ticket: docs/planning/tickets/open/TICKET-559-block-bookmarks-and-find-in-block.md
status: Phase 4 — Complete PASS
title: "Bookmarks on blocks, and find within a block"
type: feature
slice: prong 1 T1 (stage-one block actions); the Warp blocks note, recommendation 7
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/pipeline/completed/528-block-filter.spec.md, docs/planning/pipeline/completed/473-block-navigation-keys.spec.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md, docs/zed_architecture/crates/search.md]
---

## Title
Warp lets you bookmark a block for the session and jump between bookmarks, and lets you find
within one block instead of the whole session. Marley has neither: Ctrl+Up and Ctrl+Down walk
every block, and Zed's terminal search runs over all the scrollback. Both act on the selected
block (#554) first, else the newest block with a row in view, and on the hovered block from its
buttons.

## Scope
### In
- **Bookmarks.** A set of block indices per terminal, kept for the session in
  `terminal_view::MarleyBlockMarks` (keyed by the terminal's id, as `MarleyBlockSelection` is)
  and dropped when the terminal goes. `marley::ToggleBookmark` on Ctrl+Shift+B in `Terminal`
  toggles the selected block, else the newest block with a row in view
  (`block_filter::block_to_filter`); a Bookmark button among a hovered block's actions
  (`MarleyBlockExtras`) and Toggle Bookmark in its right-click Block section toggle that block.
  A marked block shows `IconName::Bookmark` before its pill, always (through `MarleyBlockChip`,
  beside #555's Ask the agent), and the terminal's right edge shows a tick per marked block at
  its place in the scrollback, painted by the element before Zed's scrollbar.
- **Jumping.** `marley::PreviousBookmark` and `marley::NextBookmark` on Alt+Up and Alt+Down in
  `Terminal`: the focused terminal scrolls so the previous or next marked block's first line is at
  the top, counted from the viewport's top line as the block keys count (`block_scroll` over the
  marked blocks). In a terminal with no bookmark the keys reach the program, as → does with no
  suggestion.
- **Find within a block.** `marley::FindInBlock` (the palette; Ctrl+Shift+F while a block is
  selected), a Find button among a hovered block's actions and Find in Block in its Block section
  hold the terminal's search to that block, the lines from its prompt line (or its output's first
  line) to its output's end (or the cursor's line while it runs), and deploy Zed's search bar with
  the focus in its query. While the scope is set, `TerminalView::find_matches` keeps only the
  matches whose start lies in those lines, so the bar's count and Enter walk the block alone, and
  the block shows an outline in the accent color. An open bar searches again at once
  (`SearchEvent::MatchesInvalidated`). Closing the bar (Escape) clears the scope; Find on the
  scoped block clears it too and keeps the bar. Ctrl+Shift+F with no selection stays Zed's
  whole-terminal search.
- Save as Workflow's button (#558) moves to `IconName::Book`, so the bookmark icon means one
  thing.
- Two hunks in Zed's terminal view for the scope: a read of it in `find_matches`, and
  `search_bar_visibility_changed` implemented to clear it when the bar closes; the element gains
  the ticks and the scoped outline.

### Out (explicitly deferred)
- Bookmarks that survive a restart (Warp's end with the session too), and marks on the rail row.
- A case-sensitivity toggle for the terminal's search (Zed's `supported_options` has none for
  terminals; a `(?i)` wrap in `regex_search_for_query` is a small Zed hunk for a later ticket).
- Warp's hover snapshot on a tick (the prompt, the command and the last two lines of output).

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
  - The search: `SearchableItem for TerminalView` (`crates/terminal_view/src/terminal_view.rs:2302`, re-read 2026-09-29):
    `supported_options` (`:2305`, `case: false`, `regex: true`, `selection: false`),
    `update_matches` (`:2323`), `activate_match` (`:2351`), `select_matches`, `find_matches`
    (`:2378`, `regex_search_for_query` then `Terminal::find_matches`);
    `search_bar_visibility_changed` is the trait's default (`crates/workspace/src/searchable.rs:88`,
    `:297`), so an implementation is additive. `Terminal::find_matches`
    (`crates/terminal/src/terminal.rs:3171`, `search_matches` over the locked grid on the
    background), `matches` (`:1677`), `activate_match` (`:2271`, a selection and a
    `ScrollToPoint`), `select_matches` (`:2284`); a match is a `Range` of `Point`s (`:498`) whose
    `line` is the grid's, negative above the screen, so its absolute line is
    `marley_screen_top + line`.
  - The blocks: `AnchoredBlock` (`anchored.rs:38`: `prompt_line`, `output_start`, `output_end`),
    `block_scroll` (`:618`, from the viewport's top over a slice of blocks, reusable over the
    marked ones), `visible_spans` (`:580`), `Terminal::blocks` (`terminal.rs:1915`) and
    `last_content` (`:2233`, `marley_screen_top`, `display_offset`, `total_lines`,
    `screen_lines`). The block keys' action route: `blocks::scroll_to_block` (`blocks.rs:161`) and
    `focused_terminal` (`:315`), the block menu (`block_menu`, `:268`, #554), caught at the workspace's root.
  - The element: `marley_block` (`terminal_element.rs:2416`, extras first in its actions row since #558), the pill (`:2358`), the paint order (the washes at `:1772`, the gutter bars at `:1818`, the block
    elements at `:1829`, the suggestion at `:1833`); `marley_rows_bounds` (`:2295`) for the
    outline, as #554's selection outline uses it; the scrollbar is Zed's `WithScrollbar` over `TerminalScrollHandle`
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
  - Since the spec was drafted: `MarleyBlockSelection` (#554), `MarleyBlockChip` (#555),
    `MarleyBlockExtras` (#558) and `block_filter::block_to_filter` (#528) exist; the view emits
    `SearchEvent::MatchesInvalidated` on every wakeup (`terminal_view.rs:1381`) and the bar
    re-runs its search on it (`buffer_search.rs:1425`); the bar's `dismiss` calls
    `search_bar_visibility_changed(false)` (`:841`); `zed_actions::buffer_search::Deploy::find()`
    is the bar's action (`zed_actions/src/lib.rs:523`).
  - Does a crate we build own this seam? `search` and `terminal_view` own the bar and the
    matches; the blocks own the lines; nothing owns marks or a scope, which are small and
    Marley's.

## UI proof
UI-AFFECTING. `script/e2e/559-block-bookmarks-and-find-in-block.sh` (`compositor sway`, for
the hovered block's buttons). Fixtures: a scratch repository; the scenario's bash; `cat -v` with
Alt+Up and Alt+Down pressed before any mark; then five blocks typed (`seq 1 30`,
`echo needle one`, `seq 31 60`, `printf 'needle two\nneedle three\n'`, `seq 61 90`), more than a
screen. Shots: the keys' bytes printed by `cat -v` with no bookmark (`559-00-keys-pass`);
Ctrl+Shift+B at the prompt, the newest block marked and its tick (`559-01-marked`); a second block
marked from its hover Bookmark button, two marks and two ticks (`559-02-two-marks`); Alt+Down to
the bottom (`559-03-jumped-down`); Alt+Up, the marked block's first line at the top
(`559-04-jumped-up`);
Ctrl+Shift+B again, the newest block's mark gone (`559-05-unmarked`); the fourth block's Find
button, `needle` typed, the bar's count and the block outlined (`559-06-find-in-block`); Escape,
the outline gone, Ctrl+Shift+F and `needle`, the whole terminal's count (`559-07-whole-terminal`);
a block selected with Ctrl+Up, Ctrl+Shift+F, held to it (`559-08-selected`). REQ-008 (no mark
after a restart) holds by construction and gets no shot: the marks live in memory, and Zed
restores a terminal as a new shell, with no scrollback for a mark to sit on.

## Locked-In Decisions
- D1: The keys act on the selected block (#554) first, else the newest block with a row in view
  (#528's rule, `block_filter::block_to_filter`); the buttons and the menu on their block.
- D2: The marks are data in a Zed global, `MarleyBlockMarks` (bookmarks and search scopes by the
  terminal's entity id), which Marley's workbench writes and the view and element read, as
  `MarleyBlockSelection` is; a terminal's entries go when it is released, so a task rerun's new
  terminal and a restore start clean, as Warp's end with the session.
- D3: The jump keys are consumed only in a terminal with a bookmark; elsewhere they reach the
  program, so a TUI that reads Alt+Up keeps it until the user marks a block there.
- D4: Find within a block is Zed's search bar with a scope, not a second search: the scope filters
  `find_matches` by the block's lines, the bar's own count, Enter and Shift+Enter do the rest, and
  Escape clears it. Ctrl+Shift+F finds in the selected block while one is selected, and is Zed's
  whole-terminal search otherwise.
- D5: The ticks are painted by the element, not by `ui`'s scrollbar: adding markers to the shared
  scrollbar would be a Zed change every scrollbar user sees.
- D6: Two hunks in `terminal_view.rs` (the scope read in `find_matches`, an implementation of
  `search_bar_visibility_changed`) beside the global, and two paints in `terminal_element.rs`
  (the scoped outline beside the selection's, the ticks); none in `terminal.rs`.
- D7: The bookmark icon rides `MarleyBlockChip` and the buttons `MarleyBlockExtras`: Marley
  composes each hook from its parts (the bookmark and #555's chip; Bookmark, Find and #558's Save
  as Workflow), so no new element hook is needed for them.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses Ctrl+Shift+B in a terminal, the system shall toggle a bookmark on the selected block, else the newest block with a row in view, showing the icon before its pill. | Shots `559-01-marked`, `559-05-unmarked` |
| REQ-002 | WHEN the user clicks a hovered block's Bookmark button, the system shall toggle that block's bookmark. | Shot `559-02-two-marks` |
| REQ-003 | WHILE a terminal has bookmarks, its right edge shall show a tick per marked block at its place in the scrollback. | Shots `559-01-marked`, `559-02-two-marks` |
| REQ-004 | WHEN the user presses Alt+Up or Alt+Down in a terminal with bookmarks, the system shall scroll so the previous or next marked block's first line is at the top, or to the bottom when that line is on the live screen. | Shots `559-03-jumped-down`, `559-04-jumped-up` |
| REQ-005 | WHILE a terminal has no bookmark, Alt+Up and Alt+Down shall reach the program. | Shot `559-00-keys-pass`: `cat -v` prints the keys' sequences |
| REQ-006 | WHEN the user clicks a hovered block's Find button, the system shall open the search bar held to that block, outline the block, and count only its matches. | Shot `559-06-find-in-block` |
| REQ-007 | WHEN the search bar closes, the system shall clear the scope, and a later Ctrl+Shift+F with no block selected shall search the whole terminal. | Shot `559-07-whole-terminal` |
| REQ-008 | WHEN Marley restarts, no bookmark shall remain. | By construction: the marks live in memory, and a restored terminal is a new shell |
| REQ-009 | WHILE a block is selected, Ctrl+Shift+F shall open the search held to that block. | Shot `559-08-selected` |
| REQ-010 | The diff gate shall be green. | `script/gates.sh --diff` |

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
