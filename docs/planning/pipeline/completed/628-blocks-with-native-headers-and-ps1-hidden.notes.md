# Blocks with native headers, PS1 hidden — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-628-blocks-with-native-headers-and-ps1-hidden.md
- **Pipeline spec:** 628-blocks-with-native-headers-and-ps1-hidden.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, third batch (#628 to #630, and #466): T5 stage two, and fish.
- **Recall (§18.3):**
  - The hooks send DCS frames, not OSC 133: a block's `prompt_line..output_start` holds PS1 and the command, with nothing marking where PS1 ends.
  - `display_offset` counts grid lines; a header that replaces rows one for one keeps the scroll arithmetic.
  - Zed's `block_below_cursor` with its pixel `scroll_top` is the precedent for rows outside the grid.
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-10-01)
- **Recall (§18.3):**
  - AD-claude-470: stage one draws over Zed's rows; stage two "needs a display-row map". Chad
    answered the plan's open decision 2 on 2026-09-30 with "Build stage two".
  - AD-claude-476: the bottom shift moves the grid's origin; anything placed from the origin
    follows it, so the header does too.
  - AD-claude-529 and L-claude-529-an-element-over-the-terminal-stops-the-release-too-001: an
    element over the terminal that stops the press must stop the release as well.
  - PR-claude-474: a frame is output until its nonce says otherwise, so only verified blocks hide.
  - Brain (consultation 4ac2a661424246ad8ce9724d02a2286e): nothing on this seam.
- **Re-binding:** the queued draft's map moved rows ("one for one or fewer"), which would touch
  all thirteen line-to-y sites and the four pixel-to-row ones. One for one touches none of them:
  the design keeps the draft's look (no PS1, a header with the command and pill) without moving a
  row.

### Design
- **`marley_terminal::anchored::prompt_rows(blocks, top, screen_lines) -> Vec<(usize, Range<usize>)>`**,
  pure: each block not pending, `command_verified`, with a `prompt_line`, and its
  `prompt_line..output_start` clipped to the viewport as rows; none empty.
- **`terminal_view.rs`:** `MarleyBlockHeader(pub Arc<dyn Fn(&Entity<TerminalView>, &Entity<Terminal>, usize, &App) -> Option<AnyElement>>)`.
- **`terminal_element.rs`:** a helper `marley_prompt_rows(content, blocks)` (none on the alternate
  screen, `top` as `marley_block_spans` computes it). In prepaint, after the spans: for each pair
  the hook gives an element for, lay it out at `origin + rows.start * line_height - scroll_top`,
  `rows.len()` rows tall, and collect the rows; then drop the text runs and background rects whose
  line is a hidden row, and the block-glyph rects whose cell row is one; the headers go into
  `marley_blocks` first, so pills and actions draw over them.
- **`marley_workbench::block_headers`** (new): the hook: `None` while `marley.block_headers` is off;
  else an `h_flex` the rows' height whose first row holds the command's first line in the buffer
  font (`…` when the command has more lines), starting at column 0, and which stops the press and
  the release.
- **Settings:** `block_headers: Option<bool>` in `MarleySettingsContent` (`Default: false`),
  `"block_headers": false` in `default.json`, `MarleySettings::block_headers`, and a Block Headers
  toggle in the Marley page's Terminal section.

### File manifest
- Marley: `crates/marley_terminal/src/anchored.rs`, `marley_terminal.rs` (re-export);
  `crates/marley_workbench/src/block_headers.rs` (new), `marley_workbench.rs`;
  `script/e2e/628-blocks-with-native-headers-and-ps1-hidden.sh`.
- Zed: `crates/terminal_view/src/terminal_view.rs`, `crates/terminal_view/src/terminal_element.rs`,
  `crates/settings_content/src/marley.rs`, `assets/settings/default.json`,
  `crates/settings_ui/src/marley_page.rs`; each row widened first.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-002 | bash, a two-line PS1, the setting on; `echo hi` then `ls` | `headers.png` |
| REQ-003 | a drag over `hi`, Ctrl+Shift+C, Ctrl+Shift+V at the prompt | `selected.png` |
| REQ-004 | the setting off | `off.png` |
Since #627 the prompt editor takes the keys at the prompt; the scenario types there, and the live
prompt the shell drew stays visible above it.

### Risks and decisions
- Block glyphs merged across rows (a prompt drawn with `█`) are dropped by their first row only;
  rare in a PS1.
- A search match or a selection inside a hidden prompt draws its highlight over the header (#629).

## Phase 2 — Code (2026-10-01)
- **Built:** `marley_terminal::prompt_rows`; the `MarleyBlockHeader` hook (`terminal_view.rs`);
  in `terminal_element.rs`, `marley_prompt_rows` and the prepaint hunk that lays each header over
  its block's prompt rows, drops those rows' text runs, background rects and block-glyph rects,
  and puts the headers first of the block elements; `block_headers.rs` (the hook); the setting
  (`settings_content`, `default.json`, `MarleySettings::block_headers` as `BlockHeaders`, the
  Terminal section's Block Headers toggle).
- **Deviations:**
  - The hook takes the row height too, so a header over a two-row prompt puts the command on its
    first row, beside the pill.
  - `BlockHeaders` is an enum, and the push server's parse moved out of
    `MarleySettings::from_settings` into `push_settings`: clippy's bool and length limits.
  - #627 had left `PromptEditor` inside `EnglishHint`'s doc comment, which then documented the
    wrong type; both read as they should now.
- **Review:** no row moves, so the cursor, the selection, the mouse, the bottom shift and the
  pinned header keep their arithmetic; only the cells go. A pending block (the prompt the shell
  waits at) and an unverified one keep their rows. The header stops the press and the release, so
  a click on it starts no selection.
- **Clippy found:** `from_settings` past 100 lines, twice.
- **Gate:** GREEN, 17 PASS.

## Phase 3 — Test (2026-10-01)
- **Scenario:** `script/e2e/628-blocks-with-native-headers-and-ps1-hidden.sh` (sway): bash with
  `PS1='[the prompt]\n$ '`, `marley.block_headers` on, `terminal.copy_on_select` on. One run.
- **Deviation:** the copy is checked by pasting into the prompt editor, which holds the focus at
  the prompt since #627; `copy_on_select` puts the drag's text on the clipboard without moving the
  focus to the grid.
- **Shots:**
  - `selected` (REQ-003): `echo hi`'s header (`echo hi` and its check, with the hover actions,
    since the pointer is over the block), an empty row, then `hi` highlighted alone; the live
    `[the prompt]` and `$ ` below.
  - `pasted` (REQ-003): the editor holds `hi` (with #557's hint, as `hi` is no command).
  - `headers` (REQ-001, REQ-002): two blocks, `echo hi` and `ls`, each its command and check pill
    on its first prompt row, the second row empty, then the output (`hi`; `notes.txt  README.md`);
    no `[the prompt]` text in either; the live prompt `[the prompt]` and `$ ` as bash drew them.
  - `off` (REQ-004): the setting off: `[the prompt]`, `$ echo hi`, `hi`, `[the prompt]`, `$ ls`,
    the listing, the live prompt, with the pills, as before.

## Phase 4 — Complete (2026-10-01)
- **Documented:** `CHANGELOG.md`; the guide (Blocks); `marley_workbench.md` (Block headers);
  `terminal_blocks.md` (`prompt_rows`); the plan's T5 row; the ledger rows of `terminal_view.rs`,
  `terminal_element.rs`, `settings_content`, `default.json` and `marley_page.rs`.
- **Knowledge:** AD-claude-628-a-blocks-header-takes-its-prompts-rows-one-for-one-001,
  L-claude-628-check-a-copy-through-the-prompt-editor-with-copy-on-select-001,
  F-claude-628-an-enum-landed-inside-anothers-doc-comment-001.
- **Brain:** the consultation closed with `brain decide`.
