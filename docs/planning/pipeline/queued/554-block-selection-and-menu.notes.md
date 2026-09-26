# Block selection and the block menu — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-554-block-selection-and-menu.md
- **Pipeline spec:** 554-block-selection-and-menu.spec.md

## Phase 1 — Plan
- **Request:** the Warp blocks note's recommendation 1 (2026-09-25): block selection, the block
  menu, and its copy and reinput items. Chad's answers of 2026-09-26 settle "copy as context"
  (both forms; Copy as Markdown lands here) and workflows (`tasks.json`, a later ticket); open
  question 6 (the block keys) got no answer. Drafted in the spec batch of 2026-09-26.
- **Classification / tier:** feature, size M for the selection and S for the menu. Three hooks
  in Zed's `terminal_view` crate, each a hunk with its ledger row; the rest in
  `marley_workbench` and `marley_terminal`.
- **Recall (§18.3):**
  - AD-claude-473-the-block-keys-scroll-and-select-nothing-001: the keys scroll, the handlers
    live at the workspace root, `secondary-up` and `secondary-down` in `Terminal`; a selected
    block was left for when hover actions need one. That time is now.
  - AD-claude-474-hover-actions-live-on-one-element-per-block-001 and
    F-claude-474-the-occluding-buttons-hid-under-the-pointer-001: one element per block, a
    wrapper stops the press; the outline is a paint, not an element, so it changes no hit test.
  - AD-claude-474-a-blocks-command-is-trusted-only-with-the-terminals-nonce-001 and
    F-claude-474-rerun-would-have-run-a-command-that-output-printed-001: only a verified command
    becomes input. Reinput and Reinput with sudo check `command_verified`.
  - AD-claude-484-autosuggestions-read-the-typed-command-from-the-grid-001: `Terminal::input`
    already carries a Marley hunk; the clear on `SelectionsChanged` needs none.
  - F-claude-496-ctrl-shift-c-in-a-tab-field-opened-the-collab-panel-001: a key bound in a
    view's context can lose to a deeper negated binding; the selected-block keys bind in
    `Terminal && MarleyBlockSelected`, the terminal's own depth.
  - F-claude-546-block-reads-answered-from-the-alternate-screen-001: no block on the alternate
    screen; the keys propagate there.
  - Brain: no page on block selection (searched 2026-09-26).
- **Discovery:**
  - `crates/marley_workbench/src/blocks.rs:15` `init` (`register_action_renderer` with
    `on_action` for `PreviousBlock` and `NextBlock`); `:31` `scroll_to_block` (`sync`,
    `last_content`, `block_scroll`, `scroll_to_bottom` then `scroll_up_by`); `:63`
    `focused_terminal`.
  - `crates/terminal_view/src/terminal_element.rs:52` `LayoutState`'s Marley fields; `:1618` to
    `:1661` the spans and the block elements in `prepaint` (`:1632` Rerun's gate); `:1772` the
    washes, `:1816` the gutter bars, `:1829` the block elements in `paint`; `:2182`
    `marley_block_spans`; `:2199` `marley_rows_bounds`; `:2237` `marley_wash`; `:2262`
    `marley_pill`; `:2311` `marley_keep_from_terminal`; `:2320` `marley_block` (`:2336` Copy,
    `:2350` Rerun's `\u{15}{command}\r`).
  - `crates/terminal_view/src/terminal_view.rs:141` `MarleyTerminalFooter`, `:150`
    `MarleyTerminalSuggestion` (the hook shapes); `:549` `deploy_context_menu` (`:561`
    `ContextMenu::build`, `:571` Copy, `:579` Select All, `:590` Add to Agent Thread, `:595`
    Close Terminal Tab); `:1026` `dispatch_context` (`:1028` `Terminal`, `:1103` `selection`);
    `:1443` the right-click (`:1450` `select_word_at_event_position`, `:1461` the call).
  - `crates/terminal/src/terminal.rs:729` `Event` (`SelectionsChanged`); `:1947` the
    `SetSelection` arm (`:1951` the PRIMARY write, the emit at its end); `:2185`
    `select_matches`; `:2204` `set_selection`; `:2316` `input`; `:2428` `write_input`
    (`Scroll::Bottom`, `SetSelection(None)`); `:1845` `blocks`; `:1852` `marley_anchored`;
    `:1859` `block_output`; `:1869` `block_output_kept`; `:2134` `last_content` (`:536`
    `Content`: `display_offset` `:540`, `screen_lines` `:542`, `terminal_bounds` `:547`,
    `marley_screen_top` `:555`).
  - `crates/marley_terminal/src/anchored.rs:38` `AnchoredBlock`; `:63` `BlockTimes`; `:116`
    `command_verified`; `:170` `times`; `:176` `at_prompt`; `:369` `BlockSpan { index, rows,
    starts_in_view, state, exit_code }`; `:389` `visible_spans(blocks, top, screen_lines,
    cursor_line)`; `:427` `block_scroll`. `crates/marley_terminal/src/block.rs:29`
    `BlockState`, `:41` `ExitCode`, `:46` `PromptInfo` (`pwd`, `git_branch`).
    `crates/marley_terminal/src/keys.rs:78` `ctrl_byte`.
  - `crates/ui/src/components/context_menu.rs:284` `build`; `:450` `header`; `:469`
    `separator`; `:488` `entry`; `:731` `action_disabled_when`.
  - `crates/gpui/src/window.rs:7609` `outline`.
  - `assets/keymaps/default-linux.json:1295` the `Terminal` block; `crates/marley_workbench/keymap.json:17`
    Marley's.
  - `crates/marley_workbench/src/rich_input.rs:31` `Prompts`, `:88` released with the view.
- **Decisions:** D1 to D8 in the spec.

### Design
- **The global** (`terminal_view.rs`, beside the hooks): `pub struct MarleyBlockSelection(pub
  HashMap<EntityId, usize>)` with `impl Global`. `dispatch_context` adds `MarleyBlockSelected`
  when the map holds the view's id. `terminal_element.rs`, in `paint` after the gutter bars: for
  the selected index's span (from `marley_spans`), `window.paint_quad(outline(marley_rows_bounds(..),
  accent, BorderStyle::Solid))`.
- **The hook** (`terminal_view.rs`): `pub struct MarleyBlockMenu(pub Arc<dyn Fn(&MarleyBlockMenuContext,
  ContextMenu, &mut Window, &mut App) -> ContextMenu>)` with `impl Global`; the context carries
  the view's weak handle, its terminal and the click's position. `deploy_context_menu` calls it
  after Zed's last item. The workbench's renderer: the row under the pointer from the position,
  the content's bounds and line height; the block from `visible_spans(blocks,
  marley_screen_top - display_offset, screen_lines, cursor_line)`; none on the alternate screen
  or off a block; else select it and append the section.
- **`block_selection.rs`.** `init`: the actions `SelectBlockUp`, `SelectBlockDown`,
  `ClearBlockSelection`, `Reinput`, `ReinputWithSudo`, `CopyBlockCommand`, `CopyBlockOutput`,
  `CopyBlockBoth`, `CopyBlockMarkdown` on the workspace (with a `block` argument for the menu's
  items, or the selected block); `select(view, index, cx)` writes the global, subscribes once to
  the terminal's `SelectionsChanged` (the subscription kept beside the entry, dropped on clear),
  and scrolls the block's first row into view when it is off screen (the `scroll_to_block`
  arithmetic); `clear(view, cx)`. `blocks.rs`'s handlers become select-and-scroll (D1).
- **The copies.** Read the block and `block_output` (or `block_output_kept` false: the note);
  `block_markdown(command, output: Option<&str>, state, exit, duration, cwd, branch) -> String`
  in `marley_terminal`, the fence with `$ ` before the command.
- **Reinput.** Rerun's path without the return: `terminal.input(format!("\u{15}{command}"))`,
  or `"\u{15}sudo {command}"`; offered when `command_verified` and the last block is Finished.
- **The keymap.** `"context": "Terminal && MarleyBlockSelected"`: `up`, `down`, `escape`,
  `ctrl-shift-i`.
- **File manifest.** Zed: `crates/terminal_view/src/terminal_view.rs` (the global, the context
  flag, the menu hook), `crates/terminal_view/src/terminal_element.rs` (the outline). Marley:
  `crates/marley_workbench/src/block_selection.rs` (new), `blocks.rs`, `marley_workbench.rs`
  (the module, the actions), `keymap.json`; `crates/marley_terminal/src/block.rs` or a new
  `markdown.rs` (`block_markdown`). Scripts: `script/e2e/554-block-selection-and-menu.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `terminal_view.rs` row gains the
  global, the flag and the hook; the `terminal_element.rs` row gains the outline.

### E2E plan

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | three blocks; `ctrl-up` | `554-01-selected`: the outline on `false` |
| REQ-002 | `up` | `554-02-moved`: on `echo two` |
| REQ-003 | `escape` | `554-03-cleared` |
| REQ-003 | `ctrl-up`, `x` | `554-04-typed-clears`: no outline, `x` at the prompt |
| REQ-004 | right-click on `echo two`'s first row | `554-05-menu` |
| REQ-005 | Copy Command; `ctrl-n`, `ctrl-v` | `554-06-copy-command` |
| REQ-005 | Copy Both; the buffer, `ctrl-a`, `ctrl-v` | `554-07-copy-both` |
| REQ-005 | Copy as Markdown; the buffer, `ctrl-a`, `ctrl-v` | `554-08-copy-markdown` |
| REQ-006 | right-click `false`, Reinput | `554-09-reinput`: `false` at the prompt, no new block |
| REQ-006 | `ctrl-u`; Reinput with sudo | `554-10-reinput-sudo`: `sudo false` |
| REQ-008 | `ctrl-u`; `ctrl-up`, `ctrl-shift-i` | `554-12-reinput-key` |
| REQ-007 | `ctrl-u`; the printed frames; right-click the unverified block | `554-11-unverified`: both items disabled |

The menu's coordinates come from the shot before the click, as #500's scenario reads its rows;
a `Block` header at a fixed offset under Zed's last item keeps them stable. Not reachable: none.

### Risks
- The outline is painted from `marley_spans` at paint time; a block whose first row scrolled
  off still has rows on screen and keeps its outline over them, which is right.
- `ctrl-up`'s meaning changes for the golden scenarios that press the block keys (#473's);
  Test reruns the set and reads their checks.
- A right-click on a text selection keeps Zed's `select_word_at_event_position` and its Copy;
  the Block section joins the same menu, so nothing is lost.
- The menu hook's context holds a weak view; a view released between the click and the item's
  click makes the item a no-op.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
