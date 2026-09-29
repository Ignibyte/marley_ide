# A long block's command stays in view — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-529-sticky-command-header.md
- **Pipeline spec:** 529-sticky-command-header.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** Chad approved all seven items of the Warp once-over on 2026-09-25. Item 5's second
  half, as asked: a long block's command pinned at the top of the pane while you scroll its
  output; a click jumps to the block's start. The filter, item 5's first half, is #528.
- **Classification:** feature, size S. Marley crates `marley_terminal` (which block to pin, pure)
  and `marley_workbench` (the header, its click, the setting's reading); Zed crates
  `terminal_view` (the hook), `settings_content`, `settings_ui` and `assets/settings/default.json`
  (the toggle).
- **Recall (§18.3):**
  - AD-claude-470-stage-one-draws-blocks-over-zeds-rows-001: decorations over Zed's rows, no change
    to the row model; its Rejected list names "a sticky pill for a block whose first row scrolled
    away (later)", which this ticket is.
  - AD-claude-473-the-block-keys-scroll-and-select-nothing-001: scroll so a block's first line is at
    the top; the handler syncs the terminal before it reads.
  - AD-claude-476-the-content-sits-on-the-bottom-edge-by-moving-the-grids-origin-001: the grid's
    origin moves down by `bottom_shift` rows while the screen has room; the header is placed at the
    element's top, not the moved grid's.
  - AD-claude-484-autosuggestions-read-the-typed-command-from-the-grid-001: the element asks a
    Marley hook in `prepaint` and paints its answer.
  - L-claude-474-an-occluding-child-ends-its-groups-hover-001: stop a press in a wrapper rather than
    occlude; the header's own press guard follows it.
  - PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: showing a frame's field
    needs no nonce; the header only shows the command.
- **Discovery (the seams, checked 2026-09-25):**
  - `crates/terminal_view/src/terminal_element.rs`: `LayoutState`'s Marley fields (52-57); in
    `prepaint`, the spans (`let marley_spans`, 1619) and the suggestion asked of the
    `MarleyTerminalSuggestion` hook (1663-1664); in `paint`, the suggestion painted (1834) after
    the blocks. The header's hook call and paint go beside those.
  - `crates/terminal_view/src/terminal_view.rs:150` `MarleyTerminalSuggestion`, the pattern for the
    new hook's global.
  - `crates/marley_terminal/src/anchored.rs`: `BlockSpan::starts_in_view` (226), `visible_spans`
    (240), `block_scroll` (278).
  - `crates/marley_workbench/src/blocks.rs:31` `scroll_to_block`: `terminal.sync` (44), then
    `scroll_to_bottom` and `scroll_up_by(offset)` (56).
  - `crates/editor/src/element/header.rs`: `sticky_headers` (65), `paint_sticky_headers` (322),
    whose left press scrolls the scope to the top and stops propagation (404);
    `crates/editor/src/editor_settings.rs:98` `StickyScroll`; `assets/settings/default.json:787-790`
    `sticky_scroll.enabled: false`.
  - `crates/settings_content/src/marley.rs`, `assets/settings/default.json`'s `marley` block and the
    Marley page, `crates/settings_ui/src/marley_page.rs` (#515, #516 extending both tonight).
  - Zed's `assets/keymaps/default-linux.json:1329`: `shift-pageup` scrolls the terminal a page,
    which the scenario uses.
- **Decisions:** D1 to D5 in the spec.
- **Promotion, 2026-09-29:** every seam re-read. Since the draft: #554's selection outline,
  #559's scope outline and ticks after the block elements in `paint`; #555's last-row chip, an
  element pushed onto `marley_blocks` in `prepaint` (`terminal_element.rs:1690-1722`), the model
  for the header; `MarleyTerminalSuggestion` at `terminal_view.rs:264`, its call at
  `terminal_element.rs:1724`; the Marley page's Terminal section exists (#503,
  `marley_page.rs:284`, one item, `terminal_links`), so the toggle joins it; `blocks::reveal`
  (`blocks.rs:140`) already scrolls a block's first line to the top. Brain (consultation a10e0fc4): nothing on this seam.

### Design
- **Which block** (pure, `marley_terminal::anchored`): `sticky_block(spans, display_offset) ->
  Option<usize>`: none at the live screen; otherwise the first span when its rows start at row 0
  and it does not start in view. The alternate screen needs no check: the element's spans are
  empty there.
- **The hook** (Zed crate `terminal_view`): `MarleyStickyHeader(Arc<dyn Fn(&Entity<TerminalView>,
  &Entity<Terminal>, usize, &App) -> Option<AnyElement>>)`, `MarleyBlockChip`'s shape; in
  `prepaint`, after #555's last-row chip, the element asks it for `sticky_block`'s block, lays the
  answer out as a root at row 0's origin, the element's width and one line high, and pushes it
  onto `marley_blocks`, painted last among them.
- **The header** (`crates/marley_workbench/src/sticky_header.rs`, new): the hook set at `init`;
  none when `MarleySettings::sticky_command_header` is false; otherwise a row with the terminal's
  background and a bottom border, the command in the buffer font, cut to fit, the state
  (`running`, a check, `exit N`) and an `ArrowUp` icon, with the pointing-hand cursor.
  `on_mouse_down(Left)` stops propagation, so the terminal starts no selection; `on_click` runs
  `blocks::reveal` (now `pub(crate)`) and notifies the view.
- **The setting:** `MarleySettingsContent::sticky_command_header: Option<bool>`, `true` in
  `default.json`'s `marley` block, `MarleySettings` resolving it, and a Terminal section on the
  Marley page with the toggle.
- **File manifest.**
  - Marley crates: `crates/marley_terminal/src/anchored.rs` (`sticky_block`),
    `crates/marley_workbench/src/sticky_header.rs` (new, the header), `blocks.rs` (`reveal`
    `pub(crate)`), `marley_workbench.rs` (the module, its init, the setting);
    `script/e2e/529-sticky-command-header.sh` (Test).
  - Zed crates: `crates/terminal_view/src/terminal_view.rs` (the global),
    `crates/terminal_view/src/terminal_element.rs` (the call, the layout, the paint),
    `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
    `assets/settings/default.json`.
- **Ledger rows:** extend the rows of the five Zed paths above with the #529 hunks.

### E2E plan
`script/e2e/529-sticky-command-header.sh`, `compositor sway`. Setup: a scratch HOME with
`PS1='$ '`; a 500-line file for `less`.

| REQ | Step | Shot |
|---|---|---|
| REQ-002 | `seq 1 400`; `echo after` | `529-01-live` |
| REQ-001 | Shift+PageUp twice | `529-02-pinned` |
| REQ-003 | Shift+PageUp until `$ seq 1 400` shows | `529-03-start-in-view` |
| REQ-004 | Shift+PageDown twice; a click on the header | `529-04-jumped` |
| REQ-005 | Shift+End; the counting loop; Shift+PageUp while it runs | `529-05-running` |
| REQ-006 | after the loop, `less` on the file; Shift+PageUp | `529-06-alternate` |
| REQ-007 | `q`; the setting set to false in the profile copy's `settings.json`; Shift+PageUp twice | `529-07-off` |
| REQ-008 | `marley: open settings` | `529-08-setting` |
| REQ-009 | `just gate-diff` | the gate's exit |

### Risks
- **Scrolling while a block runs:** if Zed's terminal jumps to the bottom on new output while
  scrolled back, `529-05-running` shows no header; the shot says which, and a scroll-lock is not
  this ticket's.
- **The top row under the header** is hidden while it shows; one Shift+PageUp more shows it. Warp
  has the same trade.
- **The bottom shift (#476):** placed at the element's top, the header sits over the padding rows
  when the content is drawn down, which is harmless; the shots check it.

## Phase 2 — Code
- **Built:**
  - `marley_terminal/src/anchored.rs`: `sticky_block(spans, display_offset)`, re-exported.
  - `terminal_view.rs`: the `MarleyStickyHeader` global, `MarleyBlockChip`'s shape.
    `terminal_element.rs`: in `prepaint`, after #555's last-row chip, the hook asked for
    `sticky_block`'s block, laid out at row 0's origin (the moved grid's, so #476's shift holds),
    the element's width and a line high, and pushed last onto `marley_blocks`.
  - `marley_workbench/src/sticky_header.rs` (new): the hook; none while
    `MarleySettings::sticky_command_header` is off; the row with the terminal's background and a
    bottom border, `$ ` and the command's first line (an ellipsis after it for more) in the buffer
    font, truncated, the state (`running` for a pending or running block, a check, `exit N`) and
    `ArrowUp`. `blocks::reveal` is `pub(crate)` for the jump.
  - The setting: `sticky_command_header` in `MarleySettingsContent`, `true` in `default.json`'s
    `marley` block, `MarleySettings` resolving it (default true), and a Sticky Command Header
    toggle in the Marley page's Terminal section (now three items).
- **Deviations:** the header jumps on the press, not the click, as Zed's editor sticky headers do,
  and stops the release too: with the stop on the press and `on_click` on one element, the
  release still reached the terminal, whose plain-click listener (#579) could open a link menu for
  a URL on the row beneath.
- **Review of the diff:** the hook reads the terminal, never the view; the element's spans are
  empty on the alternate screen, so `sticky_block` needs no check of its own; a block whose first
  row is row 0 starts in view and gets no header; `reveal` clamps the offset to the history.
- **Gate:** run 1 red: rustfmt and a `too_long_first_doc_paragraph` on the module doc, both in
  `sticky_header.rs`. Run 2: GATE GREEN [diff], with the scenario in the tree. Its warnings
  (`settings_content`'s `language.rs:74` and `merge_from.rs`, `terminal_panel.rs:327`,
  `terminal_view.rs:1666`, `:2106`) are Zed's own code: pre-existing, not in scope.

## Phase 3 — Test
- **Scenario:** `script/e2e/529-sticky-command-header.sh` under `compositor sway`.
- **Run 1, one red, the scenario's:** `529-05-running` showed `seq 1 400`'s header, not the
  loop's: four seconds in, the loop had printed about 35 lines, so one page up still showed its
  start. The header was right for what was on screen. The scenario now waits eight seconds. The
  run also showed the view holding its place while output arrives, so a running block can be
  pinned at all.
- **Run 2:** exit 0; every shot read:
  - `529-01-live` (REQ-002): `echo after` and the prompt on the live screen, no header.
  - `529-02-pinned` (REQ-001): two pages up, 267 to 311 of `seq`'s output; over the top row
    `$ seq 1 400`, its check and the up arrow, a border under it.
  - `529-03-start-in-view` (REQ-003): at the scrollback's top, the block's own first row and
    pill, no header.
  - `529-04a-before-click`, `529-04-jumped` (REQ-004): the header over 139 to 183; after the
    click `$ seq 1 400` on the top row with 1 to 45 below, no header, no selection (the hover row
    is the pointer's, which stays over the block).
  - `529-05-running` (REQ-005): over 69 to 113, `$ for i in $(seq 1 400); do echo $i; sleep
    0.05; done` with `running` and the arrow.
  - `529-06-alternate` (REQ-006): `less long.txt` after two page ups, no header.
  - `529-07-off` (REQ-007): the setting set off in the profile copy; two pages up, 265 to 310,
    no header.
  - `529-08a-settings`, `529-08-setting` (REQ-008): the Marley page; scrolled, the Terminal
    section with Terminal Links and Sticky Command Header, its toggle off with the reset mark,
    as the run set it.
- **Gate:** the scenario changed after run 2's green, so run 3 of `just gate-diff`: GATE GREEN [diff].

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: a long block's command stays in view);
  `docs/marley/three-prong-plan.md` T1; `docs/marley_architecture/marley_workbench.md` (the
  sticky command header) and `terminal_blocks.md` (`sticky_block`); the touchpoint rows of the
  five Zed paths describe what shipped.
- **Knowledge:** `AD-claude-529-the-sticky-header-is-drawn-over-row-zero-while-scrolled-back-001`,
  `L-claude-529-an-element-over-the-terminal-stops-the-release-too-001`. No `F-…` block: the
  release gap was caught in review before any run, and the run's red was the scenario's timing.
  Brain: `decisions/marleys-sticky-command-header-is-drawn-over-row-0-only-while-scrolled-back` on
  consultation a10e0fc4, follow-up by 2026-10-29.
- **Closed** TICKET-529, archived the pair.
