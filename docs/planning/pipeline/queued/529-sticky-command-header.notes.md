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

### Design
- **Which block** (pure, `marley_terminal::anchored`): `sticky_block(spans, display_offset,
  alt_screen) -> Option<usize>`: none on the alternate screen or at the live screen; otherwise the
  span whose rows start at row 0 and that does not start in view. The jump's offset:
  `screen_top - start`, the block's prompt line or its output's first, clamped to the history,
  as `block_scroll` clamps.
- **The hook** (Zed crate `terminal_view`): `MarleyStickyHeader(Arc<dyn Fn(&Entity<Terminal>,
  &mut Window, &mut App) -> Option<AnyElement>>)` beside `MarleyTerminalSuggestion`; in
  `prepaint`, the element asks it after the spans, lays the answer out as a root at the element's
  top-left, the element's width and one line high, and stores it in `LayoutState`; `paint` paints
  it after the blocks and the suggestion, so nothing is drawn over it.
- **The header** (`crates/marley_workbench/src/blocks.rs`, which holds the block keys): the hook
  set at `init`; it returns none when `MarleySettings::sticky_command_header` is false or
  `sticky_block` finds none; otherwise a row with the terminal's background and a bottom border,
  the command in the buffer font, cut to fit, the state (`running`, a check, `exit N`) and an
  `ArrowUp` icon, with the pointing-hand cursor. `on_mouse_down(Left)` stops propagation;
  `on_click` syncs the terminal and scrolls it by the jump's offset, as `scroll_to_block` does.
- **The setting:** `MarleySettingsContent::sticky_command_header: Option<bool>`, `true` in
  `default.json`'s `marley` block, `MarleySettings` resolving it, and a Terminal section on the
  Marley page with the toggle.
- **File manifest.**
  - Marley crates: `crates/marley_terminal/src/anchored.rs` (`sticky_block`),
    `crates/marley_workbench/src/blocks.rs` (the header), `marley_workbench.rs` (the setting);
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
