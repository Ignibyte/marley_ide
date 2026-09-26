---
pipeline_id: ba518e99-6a91-4404-bcd9-f067d19b21df
ticket: docs/planning/tickets/open/TICKET-554-block-selection-and-menu.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Block selection and the block menu"
type: feature
slice: prong 1 T1 (the bar's item 3: select a block, copy command); the Warp blocks note's recommendation 1
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md, docs/planning/pipeline/queued/528-block-filter.spec.md, docs/planning/pipeline/queued/529-sticky-command-header.spec.md]
---

## Title
A selected block per terminal view, chosen with the keyboard (`ctrl-up` takes the newest block
and moves up from there, `up` and `down` move, Escape or a key typed to the shell drops it) and
drawn as an outline over its rows; and a Block section in the terminal's right-click menu when
the click lands on a block: Copy Command, Copy Output, Copy Both, Copy as Markdown, Reinput
and Reinput with sudo. Reinput types the command at the prompt without running it, under
Rerun's rules: a verified command, the shell at its prompt.

## Scope
### In
- `crates/marley_workbench/src/block_selection.rs` (new): the actions, the selection's
  bookkeeping, the clear on `Event::SelectionsChanged`, the menu's section and its items, and
  the copies.
- **The selection.** A `MarleyBlockSelection` global in `terminal_view` (a map from the view's
  entity id to a block index, as rich input keeps its editors), written by the workbench and
  read by Zed's element and view. `marley::PreviousBlock` and `NextBlock` (`ctrl-up`,
  `ctrl-down`) select as they scroll: with no selection `ctrl-up` selects the newest block and
  scrolls its first row into view; with one it moves to the block before, `ctrl-down` to the
  block after, and past the last block clears. While a block is selected the view's key context
  carries `MarleyBlockSelected` (a `// Marley:` hunk in `TerminalView::dispatch_context`), and
  the Marley keymap binds `up`, `down`, `escape` and `ctrl-shift-i` in
  `Terminal && MarleyBlockSelected`. Any other key reaches the terminal as before, and since
  `Terminal::input` queues `SetSelection(None)`, whose arm emits `SelectionsChanged`, the
  selection clears on that event with no hunk. On the alternate screen the keys propagate.
- **The outline.** `terminal_element.rs` paints `outline(marley_rows_bounds(..))` in the
  accent color over the selected block's rows in the pass that paints the gutter bars (a
  `// Marley:` hunk beside the existing ones).
- **The menu.** `TerminalView::deploy_context_menu` calls a `MarleyBlockMenu` global (a hook,
  as `MarleyTerminalFooter` is) with the view, its terminal, the click's position and the
  builder; the workbench finds the block under the click through `visible_spans` and the
  content's bounds, selects it, and appends a separator, a `Block` header and the items. Copy
  Output joins the menu; the hover buttons stay.
- **The copies.** Copy Command (`AnchoredBlock::command`), Copy Output
  (`Terminal::block_output`), Copy Both (command, newline, output), Copy as Markdown: a fenced
  block with `$ <command>` and the output, then one line `exit <code> · <duration> · <cwd>
  (<branch>)`, from a pure `marley_terminal::block_markdown` that #555 reuses; output no longer
  in the scrollback says so inside the fence.
- **Reinput.** Ctrl-U then the command, no return, only for a `command_verified` block and
  while the shell waits at its prompt (the last block Finished, Rerun's gate); Reinput with sudo
  the same with `sudo ` first. The two items show disabled otherwise.
- `script/e2e/554-block-selection-and-menu.sh`.

### Out (explicitly deferred)
- Several blocks selected (Shift-click, Select All Blocks): one block.
- Bookmarks, Find in Block, Scroll within block, Share, Filter Block (#528), the sticky header
  (#529), Save as Workflow (Chad: entries in `tasks.json`; its own ticket).
- Chords for the copies beyond the menu: `ctrl-shift-c` is Zed's terminal Copy,
  `ctrl-shift-a` its Select All, `ctrl-shift-f` its search; Warp's `ctrl-shift-alt-c` for Copy
  Output can come later.
- The block's rows as the terminal's text selection: `select_matches` only re-selects search
  matches, and a setter would be a Zed hunk that also writes the PRIMARY selection at every
  move; Zed's Copy, Inline Assist and Add to Agent Thread keep working on text selections.
- Stage two's header rows (T5), and a click elsewhere clearing the selection (a mouse hunk):
  Escape and typing clear it; a right-click moves it.

## Reference (§20)
- **Warp:** `Ctrl+↑` selects the newest block, the arrows move the selection, Esc clears it,
  and `preserve_input_focus_on_block_selection` keeps the input's focus; the block menu opens on
  right-click or from the kebab with Copy command (`Ctrl+Shift+C`), Copy output, Reinput
  (`Ctrl+Shift+I`), Reinput as root and the rest the blocks note lists
  (docs.warp.dev/terminal/blocks/, block-basics and block-actions;
  docs.warp.dev/getting-started/keyboard-shortcuts/;
  docs.warp.dev/terminal/settings/all-settings/). Marley takes the selection keys and the six
  items; Warp's copy chords collide with Zed's terminal keys, so the menu carries the copies and
  only Reinput keeps its chord, in the selected-block context where it is free.
- **Upstream Zed:** `TerminalView::deploy_context_menu` (`crates/terminal_view/src/terminal_view.rs:549`)
  and its items stay; the Block section is appended through a hook; `ui::ContextMenu`'s
  `header`, `separator`, `entry` and `action_disabled_when`
  (`crates/ui/src/components/context_menu.rs:450`, `:469`, `:488`, `:731`) draw it as Zed's
  menus are drawn.
- **Warp's blocks map:** `docs/warp_architecture/subsystems/03-terminal-session-core.md` §6:
  a block is the command's record with its exit code and prompt context, which the copies read.

### Prior art
- **Behavior maps and research.** The blocks note (the table of actions, "Block selection",
  "The block menu", open question 6, unanswered: Warp's binding is the default);
  AD-claude-473-the-block-keys-scroll-and-select-nothing-001 (a selected block rejected for
  then, and named for when hover actions need one); AD-claude-474 (Rerun's Ctrl-U, the verified
  command, a prompt only; "Rerun that types the command without running it" rejected because it
  changes Rerun, so Reinput is its own item);
  F-claude-474-rerun-would-have-run-a-command-that-output-printed-001 and the prevention rule at
  `prevention-rules.md:2218` (check the nonce before a block action turns a field into input);
  #528's D4 (`528-block-filter.spec.md:102`: it filters the newest block in view since stage one
  has no selected block; with one it can take the selected block, a note for #528's Plan) and
  #529's D3 (`529-sticky-command-header.spec.md:86`: its click selects nothing).
- **Published material.** Warp's pages above.
- **The code we already ship.** The decorations are drawn in `terminal_element.rs` with no
  hook: `marley_block_spans` (`:2182`, private), `marley_block` (`:2320`, the per-block element
  with Copy Output `:2336` and Rerun `:2350`, gated at `:1632` on the last block Finished),
  `marley_rows_bounds` (`:2199`), `marley_pill` (`:2262`), `marley_keep_from_terminal`
  (`:2311`), the paint of the block elements (`:1829`); `gpui::outline`
  (`crates/gpui/src/window.rs:7609`) paints a border. `terminal_view.rs`: `deploy_context_menu`
  (`:549`; the right-click at `:1443`, the call at `:1461`; Add to Agent Thread at `:590`),
  `dispatch_context` (`:1026`), the hook globals `MarleyTerminalFooter` (`:141`) and
  `MarleyTerminalSuggestion` (`:150`). `crates/terminal/src/terminal.rs`: `input` (`:2316`)
  calls `write_input` (`:2428`), which queues `SetSelection(None)`, whose arm emits
  `SelectionsChanged` unconditionally (`:1947`); `select_matches` (`:2185`) and `set_selection`
  (`:2204`, private); `blocks` (`:1845`), `block_output` (`:1859`), `block_output_kept`
  (`:1869`), `last_content` (`:2134`). `marley_terminal`: `AnchoredBlock`
  (`crates/marley_terminal/src/anchored.rs:38`: command, `command_verified` set at `:116`,
  state, exit code, prompt with pwd and branch), `BlockTimes` (`:63`), `at_prompt` (`:176`),
  `BlockSpan` (`:369`), `visible_spans` (`:389`), `block_scroll` (`:427`); `keys::ctrl_byte`
  (`crates/marley_terminal/src/keys.rs:78`). `blocks.rs` (`:15` init, `:31` `scroll_to_block`,
  `:63` `focused_terminal`); `rich_input.rs`'s per-view global (`:31`, released at `:88`). Keys:
  Zed's `Terminal` block (`assets/keymaps/default-linux.json:1295`): `ctrl-shift-c` Copy
  (`:1300`), `ctrl-shift-a` Select All (`:1318`), `ctrl-shift-f` search (`:1320`),
  `ctrl-shift-l` Clear (`:1321`); `ctrl-shift-b` is the outline panel in `Workspace` (`:709`);
  `ctrl-shift-i` is free in `Terminal` and `Workspace`; the Marley keymap's `Terminal` block
  (`crates/marley_workbench/keymap.json:17`). The clipboard:
  `cx.write_to_clipboard(ClipboardItem::new_string(..))` (`browser.rs:1959`,
  `terminal_element.rs:2337`). Does a crate we build own this seam? `terminal_view` and
  `terminal_element` own the menu and the drawing and gain three small hooks; the state, the
  items and the actions are Marley's.

## UI proof
UI-AFFECTING: the outline, the menu, the prompt, the clipboard.
`script/e2e/554-block-selection-and-menu.sh` (`compositor sway`: it right-clicks and clicks
menu items). Setup: a scratch repository; a HOME whose `.bashrc` is the scenario's
(`PS1='$ '`). Steps: `echo one`, `echo two`, `false`, each with Return; `ctrl-up`: the outline
on the `false` block (`554-01-selected`); `up`: on `echo two` (`554-02-moved`); `escape`: none
(`554-03-cleared`); `ctrl-up`, then `x` typed: no outline and `x` at the prompt
(`554-04-typed-clears`), `ctrl-u`. A right-click on the `echo two` block's first row: the menu
with the Block section, the block outlined (`554-05-menu`); click Copy Command; `ctrl-n` opens
a buffer, `ctrl-v`: `echo two` (`554-06-copy-command`); `ctrl-tab` back; right-click, Copy
Both; the buffer, `ctrl-a`, `ctrl-v`: `echo two` and `two` (`554-07-copy-both`); back,
right-click, Copy as Markdown; the buffer, `ctrl-a`, `ctrl-v`: the fence and the line
(`554-08-copy-markdown`). Right-click the `false` block, Reinput: `false` at the prompt and no
new block (`554-09-reinput`); `ctrl-u`; right-click, Reinput with sudo: `sudo false`
(`554-10-reinput-sudo`); `ctrl-u`. `ctrl-up`, `ctrl-shift-i`: `false` at the prompt again
(`554-12-reinput-key`); `ctrl-u`. `printf` a `precmd` and a `preexec` frame without the nonce,
as #474's test prints them, making an unverified block; right-click it: Reinput and Reinput
with sudo disabled (`554-11-unverified`).

## Locked-In Decisions
- D1: The block keys select and scroll (Warp's `Ctrl+↑`); open question 6 got no answer and
  Warp's binding is the default. `ctrl-up` with no selection takes the newest block, not the
  block above the viewport as today, since a selection wants the nearest block first;
  AD-claude-473's scroll (the first row at the top) stays for the blocks the selection moves
  through.
- D2: One selected block per view, in a global keyed by the view's entity id (rich input's
  pattern), released with the view; none survives a relaunch.
- D3: The selection ends on `SelectionsChanged`: typed input, a paste and a mouse selection all
  raise it; Escape clears by action; a right-click moves it to the block under the pointer. No
  mouse hunk.
- D4: An outline, not a text selection: `select_matches` cannot set an arbitrary range, and a
  setter would be a Zed hunk that also writes the PRIMARY selection at every move
  (`terminal.rs:1951`). The copies read the block by its anchors, as Copy Output does.
- D5: Three hooks in Zed's crates, one hunk each: the key context flag, the outline's paint,
  the menu's section; the state, the items and the actions live in `marley_workbench`.
- D6: Reinput follows Rerun's two rules: a verified command only (F-claude-474; the nonce) and a
  shell at its prompt only, Ctrl-U first and no return. The items show disabled otherwise, so
  the menu says what is possible.
- D7: Copy as Markdown is a fence with `$ <command>` and the output, then
  `exit <code> · <duration> · <cwd> (<branch>)`; a running block says `running` in place of the
  exit; evicted output says so inside the fence. The clipboard is not redacted: what Chad copies
  stays exact (AD-claude-516); #555 redacts what goes to an agent.
- D8: Keys beyond `up`, `down`, `escape` and `ctrl-shift-i` wait: Warp's copy chords collide
  with Zed's terminal keys, and the menu carries every item.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses `ctrl-up` with no block selected, the system shall outline the newest block and scroll its first row into view. | Shot `554-01-selected` |
| REQ-002 | WHILE a block is selected, `up` and `down` shall move the outline to the block before or after, and `ctrl-down` past the last block shall clear it. | Shot `554-02-moved` |
| REQ-003 | WHEN the user presses Escape or types to the shell, the system shall drop the selection and pass a typed key on. | Shots `554-03-cleared`, `554-04-typed-clears` |
| REQ-004 | WHEN the user right-clicks a block's rows, the system shall select it and show Zed's menu with a Block section: Copy Command, Copy Output, Copy Both, Copy as Markdown, Reinput, Reinput with sudo. | Shot `554-05-menu` |
| REQ-005 | WHEN the user picks a copy item, the clipboard shall hold the command, the output, both, or the Markdown form. | Shots `554-06-copy-command`, `554-07-copy-both`, `554-08-copy-markdown` |
| REQ-006 | WHEN the user picks Reinput on a verified block while the shell waits at its prompt, the system shall type Ctrl-U and the command with no return; Reinput with sudo the same with `sudo ` first. | Shots `554-09-reinput`, `554-10-reinput-sudo` (no new block) |
| REQ-007 | WHEN a block's command was not reported with the terminal's nonce, or the shell is not at its prompt, the menu shall show Reinput and Reinput with sudo disabled. | Shot `554-11-unverified` |
| REQ-008 | WHEN the user presses `ctrl-shift-i` with a block selected, the system shall reinput as the item does. | Shot `554-12-reinput-key` |
| REQ-009 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion re-verify
  the three seams in `terminal_view.rs` and `terminal_element.rs`, and #528's Plan reads D1.
- **P2 Code:** the ledger rows first (`crates/terminal_view/src/terminal_view.rs`,
  `crates/terminal_view/src/terminal_element.rs`); the global and the hooks; `block_selection.rs`;
  `block_markdown`; the keymap lines; the block keys' new behavior in `blocks.rs`. fmt and
  clippy clean; a review of the diff (the nonce check before both Reinput items; the keys
  propagate on the alternate screen; no PRIMARY write).
- **P3 Test:** write and run the scenario and read every shot; rerun the golden set, whose
  block scenarios press the block keys; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/terminal_blocks.md` and
  `marley_workbench.md`; the touchpoints rows checked; the plan's slice status; the ledger
  capture; close the ticket, archive, commit.
