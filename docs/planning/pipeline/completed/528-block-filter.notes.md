# Filter a block's output — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-528-block-filter.md
- **Pipeline spec:** 528-block-filter.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** Chad approved all seven items of the Warp once-over on 2026-09-25. Item 5's first
  half, as asked: show only a block's lines that match text or a regex (case, invert, context
  lines), deleting nothing. The sticky command header, item 5's second half, is #529.
- **Classification:** feature, size S for the overlay (the once-over: "S each, M to filter in
  place"). Marley crates `marley_terminal` (the pure filter) and `marley_workbench` (the panel,
  the action); Zed crate `terminal_view` (two hooks and the button).
- **Recall (§18.3):**
  - AD-claude-470-stage-one-draws-blocks-over-zeds-rows-001: stage one never changes Zed's row
    model; header rows and hidden rows are stage two (T5). Hence the overlay (D1).
  - AD-claude-474-hover-actions-live-on-one-element-per-block-001,
    L-claude-474-an-occluding-child-ends-its-groups-hover-001: the buttons live on one element per
    block with a hover group; a press on one is stopped in a wrapper, never by occluding.
  - AD-claude-473-the-block-keys-scroll-and-select-nothing-001: no selected block in stage one, so
    the key picks the newest block in view (D4).
  - F-claude-481-the-rich-input-dropped-every-typed-character-on-linux-001 and
    L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001: an editor inside the
    terminal view must stop only the keys the terminal maps (chords, keys that type nothing), or
    Linux drops every character; the panel's container copies the rich input's.
  - L-claude-490-a-view-that-forwards-keys-sees-its-editors-keys-001: a view that forwards keys
    sees its child editors' keys too.
  - AD-claude-477-a-footer-hook-in-zeds-terminal-view-and-the-bar-in-marleys-crate-001: the hook
    pattern; the footer takes rows from the grid, which is why the panel is an overlay and not a
    footer (a footer would resize the PTY while the panel is open).
- **Discovery (the seams, checked 2026-09-25):**
  - `crates/terminal_view/src/terminal_element.rs`: `marley_keep_from_terminal` (2311);
    `marley_block` (2320), which builds Copy (2332) and Rerun (2343) and lays out the actions row
    shown on hover (debug selectors `marley-block-copy-{index}` at 2371). The Filter button joins
    that row. `TerminalElement` holds `terminal_view` (404), which the button needs.
  - `crates/terminal_view/src/terminal_view.rs`: `MarleyFooterContext` (130) and
    `MarleyTerminalFooter` (141), called in `render` (1397); the grid's container
    `terminal-view-container` (1465), where the overlay goes as an absolute child; Zed's
    `SearchableItem for TerminalView` (2029), `supported_options` regex only (2032-2036).
  - `crates/terminal/src/terminal.rs:1859` `block_output`: the block's text from the grid, `None`
    once its first line left the scrollback.
  - `crates/marley_terminal/src/anchored.rs` `visible_spans`: the blocks in view, from which the key
    takes the newest.
  - `crates/marley_workbench/src/rich_input.rs:140` `element`: the `MarleyRichInput` key context,
    its actions, and the container's `on_key_down` that stops the keys that type no text; the
    `Prompts` global keyed by the view's entity id (`:32`), the model for keeping each terminal's
    filter.
  - `crates/marley_workbench/src/blocks.rs:63` `focused_terminal`.
  - `crates/marley_workbench/keymap.json:17` the `Terminal` bindings; `:27` the rich input's
    editor context, the model for `MarleyBlockFilter > Editor`. Zed's
    `assets/keymaps/default-linux.json` binds `ctrl-shift-f` in `Terminal` to
    `buffer_search::Deploy` (1320) and nothing to `alt-shift-f`.
  - `crates/editor/src/editor.rs`: `single_line` (1800), `multi_line` (1810),
    `set_placeholder_text` (3299), `set_read_only` (3417), `set_text` (8931),
    `highlight_background` (9519).
  - `Cargo.toml:817` `regex = "1.5"`; `crates/marley_mcp/Cargo.toml:14` takes it (#516);
    `marley_terminal` does not yet.
- **Decisions:** D1 to D7 in the spec.

- **Promotion (2026-09-29), the seams re-verified:** `terminal_element.rs` `marley_block`
  (`:2387`, now with #555's `chip` argument), `marley_keep_from_terminal`, the block elements'
  map in `prepaint` (`:1664`); `terminal_view.rs` `MarleyFooterContext` (`:130`), the footer's
  call in `render`, `MarleyBlockSelection` (#554) for the key's block; `rich_input.rs`'s
  `element` and `Prompts`; `blocks.rs`'s `focused_terminal`; `terminal.rs` `block_output`
  (`:1929`); the Marley keymap's `Terminal` block. Changes: D4 takes #554's selected block first;
  the overlay hook is called from the view's render, so it reads only what its context hands it
  (PR-claude-a-render-hook-reads-its-context-not-its-view-001); the filter's panel is an entity
  of its own, which the hook returns as its element.
- **Brain:** consultation e5f0c4b08b7e45f7911ba03663f72885, asked at promotion.

### Design
- **The pure filter** (`crates/marley_terminal/src/filter.rs`, new): `FilterQuery { text, regex,
  case_sensitive, invert, context }`; `filter_lines(text, &query) -> Result<Filtered,
  FilterError>` with `Filtered { lines: Vec<Kept>, total }`, each kept line carrying its index in
  the block, whether it matched, and its match ranges, and a `Gap` between groups that do not
  touch. A plain query goes through `regex::escape`; `RegexBuilder::case_insensitive(!case)`; an
  empty query keeps every line.
- **The hooks** (Zed crate `terminal_view`): `MarleyTerminalOverlay(Arc<dyn Fn(&MarleyFooterContext,
  &mut Window, &mut App) -> Option<AnyElement>>)`, called in `render` beside the footer and drawn
  as an absolute child of `terminal-view-container`, covering the grid; `MarleyBlockFilter(Arc<dyn
  Fn(WeakEntity<TerminalView>, usize, &mut Window, &mut App)>)`, whose presence adds the Filter
  button to `marley_block` (wrapped in `marley_keep_from_terminal`, selector
  `marley-block-filter-{index}`), which calls it with the view and the block's index.
- **The panel** (`crates/marley_workbench/src/block_filter.rs`, new): a global `BlockFilters`
  keyed by the view's entity id, each with the block index, the three editors (query, context,
  list), the toggles, the last `Filtered` and an error. The overlay renders it while open: a
  header row (the command, "N of M lines", a close button), the controls, and the list editor.
  The container has key context `MarleyBlockFilter` and the rich input's `on_key_down`. Edits to
  the query or context field and a toggle filter again in `cx.background_spawn`, with the text
  from `block_output`; for a running block, a subscription to the terminal's events filters
  again, throttled to 250 ms. The list's text is the kept lines (with `--` rows between groups);
  the matches get `highlight_background`.
- **The actions** (`marley_workbench.rs`, `keymap.json`): `marley::FilterBlock` (Alt+Shift+F in
  `Terminal` and in `MarleyBlockFilter`) toggles the panel on the focused terminal's newest block
  in view; `marley::CloseBlockFilter` (Escape in `MarleyBlockFilter > Editor`); Enter bound there
  too so it never reaches the terminal. Closing gives the focus back to the terminal.
- **File manifest.**
  - Marley crates: `crates/marley_terminal/src/filter.rs` (new), `marley_terminal.rs` (the
    module), `crates/marley_terminal/Cargo.toml` (`regex`); `crates/marley_workbench/src/block_filter.rs`
    (new), `marley_workbench.rs` (the actions, the init), `keymap.json`;
    `script/e2e/528-block-filter.sh` (Test).
  - Zed crates: `crates/terminal_view/src/terminal_view.rs` (the overlay hook and its call),
    `crates/terminal_view/src/terminal_element.rs` (the Filter button through the hook).
- **Ledger rows:** extend the rows of `crates/terminal_view/src/terminal_view.rs` and
  `crates/terminal_view/src/terminal_element.rs` with the #528 hunks.

### Visual check plan
`script/e2e/528-block-filter.sh`, `compositor sway`. Setup: a scratch HOME with `PS1='$ '`;
`log.txt` in the scratch repository: 40 lines of `INFO`, `WARN` and `ERROR` entries, some in lower
case, some `WARN disk 9N%` lines.

| REQ | Step | Shot |
|---|---|---|
| REQ-001 | `seq 1 300`; `cat log.txt`; Alt+Shift+F | `528-01-open` |
| REQ-003 | `error` typed | `528-02-text` |
| REQ-004 | a click on the case toggle | `528-03-case` |
| REQ-005 | case off, the regex toggle, `^WARN .* 9[0-9]%$` | `528-04-regex` |
| REQ-005 | the query set to `(` | `528-05-invalid` |
| REQ-006 | a valid query, the invert toggle | `528-06-invert` |
| REQ-007 | invert off, context 1 | `528-07-context` |
| REQ-008, REQ-009 | Escape | `528-08-closed`: the grid as before, the prompt line empty |
| REQ-010 | Alt+Shift+F | `528-09-reopened` |
| REQ-002 | Escape; the pointer on `seq 1 300`'s block; its Filter button | `528-10-button` |
| REQ-011 | Escape; `for i in $(seq 1 60); do echo line $i; sleep 0.2; done`; Alt+Shift+F; `5`; four seconds | `528-11-running-a`, `528-11-running-b` |
| REQ-012 | `just gate-diff` | the gate's exit |

### Risks
- **Keys leaking to the PTY** from the panel's editors (F-claude-481): the container stops the
  keys the terminal maps, the panel's context binds Enter and Escape, and `528-08-closed`'s empty
  prompt line checks it.
- **A very long block** (a shell keeps 10,000 lines by default and up to 100,000,
  `crates/terminal/src/terminal.rs:961-962`, which a task terminal always keeps, L-claude-464):
  `block_output` builds the text on the main thread under the terminal's lock; measure at Test, and
  read it in pieces if a frame stalls.
- **Alt+Shift+F** is taken from programs in the terminal, as Ctrl-Up and Ctrl-Down are for the
  block keys (AD-claude-473).
- **Plain text only:** the kept lines lose the block's colors, a known difference from Warp until
  the in-place filter.

## Phase 2 — Code
- **Built:**
  - `marley_terminal/src/filter.rs` (new, pure): `FilterQuery`, `filter_lines` (a plain query
    escaped, case ignored unless asked, invert, context with a `Gap` between groups when context
    is on), `Filtered { rows, picked, total }`, `FilterError`; `regex` added to the crate.
  - `terminal_view.rs`: `MarleyTerminalOverlay`, asked in `render` and drawn inside
    `terminal-view-container`, made `relative` only while an overlay shows; `MarleyBlockFilter`,
    the Filter button's action. `terminal_element.rs`: the Filter button first among a block's
    hover actions while `MarleyBlockFilter` is set; `marley_block` takes the view.
  - `marley_workbench/src/block_filter.rs` (new): a `FilterPanel` entity per terminal view, kept
    for the session (`Filters`), with the query and context fields, the list (a read-only
    multi-line editor, no gutter), the `Toggles`, the count and the error. `refilter` reads
    `block_output` and filters off the main thread (`futures::future::lazy`); `follow` filters a
    running block again at most every 250 ms on its terminal's `Wakeup`; `fill_list` writes the
    rows (`--` for a gap) into the list's buffer and highlights the matches
    (`HighlightKey::BufferSearchHighlights`, the theme's search-match color). The overlay hook
    returns the open panel; `FilterBlock` opens it on the selected block or the newest in view,
    and in the panel closes it, as Escape does.
  - `marley_workbench.rs`: the module, its init, `FilterBlock` and `CloseBlockFilter`. The keymap:
    `alt-shift-f` in `Terminal`, `escape` in `MarleyBlockFilter > Editor`.
  - The module line in `marley_terminal.rs` and the `regex` dependency were written through a
    shell command a moment before this phase was opened; they are this phase's.
- **Deviations:** the Filter button is checked on a short block (`printf` of three lines) rather
  than on `seq 1 300`: the hover buttons live on a block's first row, which that block scrolls
  off. The panel is an entity the overlay hook returns, not state in the hook.
- **Review of the diff:** the overlay hook reads only the view's id and the panel (the view is
  leased in its render); the panel's container stops every key that types no text, as the rich
  input does, and Escape and Alt+Shift+F are the panel's own actions, so no key reaches the PTY;
  the grid is never written: the list is a separate buffer. `open` and the key's handler touch no
  workspace (the key's handler runs while it is leased). Two `unused_qualifications` warnings from
  `cargo check` fixed.
- **Gate:** run 1 red: `similar_names` (`matches` beside `matcher`) and
  `too_long_first_doc_paragraph` in `filter.rs`, and Zed's `async_block_without_await` on the
  filter's `background_spawn` (now `futures::future::lazy`, the crate's idiom). Run 2 red once
  `marley_terminal` compiled: `single_match_else` and `option_if_let_else` on the panel lookup,
  `struct_excessive_bools` (the toggles are a `Toggles` struct now), two
  `semicolon_if_nothing_returned`, four `needless_pass_by_ref_mut`. Run 3: GATE GREEN [diff],
  with the scenario in the tree. Run 4, after the scenario's coordinates changed: GATE GREEN
  [diff].

## Phase 3 — Test
- **The scenario:** `script/e2e/528-block-filter.sh`, `compositor sway`: `seq 1 300`, then
  `cat log.txt` (40 entries: INFO, `error:` and `ERROR`, `WARN disk 9N%`, `warn cache`), then a
  short `printf` block and a running loop. Run 1: every step ran and the prompt check passed, but
  the toggles sat further right than the plan's guesses, so the case click landed on the query
  field and the regex click on Case; the controls were measured from its shots (the row at 122;
  Case 1171, Regex 1197, Invert 1232, Context 1325; the Filter button 1262 on the block's first
  row). Run 2: exit 0, no `panicked` in any log; the focus report: nothing on Hyprland, the run's
  sway stopped.
- **The shots (run 2):**
  - 528-01-open: Alt+Shift+F: "Filter · cat log.txt", "40 of 40 lines", the query field focused,
    Case, Regex, Invert and Context 0, the 40 lines listed (REQ-001).
  - 528-02-text: `error`: "10 of 40 lines", `error:` and `ERROR` lines, the matches highlighted
    (REQ-003).
  - 528-03-case: Case on: "5 of 40", only the `error:` lines (REQ-004).
  - 528-04-regex: Case off, Regex on, `^WARN .* 9[0-9]%$`: the five `WARN disk 9N%` lines, each
    matched whole (REQ-005).
  - 528-05-invalid: `(`: "Not a valid regular expression: regex parse error: … unclosed group",
    and the five lines kept (REQ-005).
  - 528-06-invert: `WARN`, Regex off, Invert on: "30 of 40", no `WARN` or `warn` line (REQ-006).
  - 528-07-context: Invert off, Context 1: the ten `WARN`/`warn` lines with a line either side,
    `--` between the five groups (REQ-007).
  - 528-08-closed: Escape: the terminal's rows as they were, the prompt line empty, which the
    stand-in's `terminal-screen` confirms (REQ-008, REQ-009).
  - 528-09-reopened: Alt+Shift+F: `WARN`, Context 1, the same list (REQ-010).
  - 528-10a-hover / 528-10-button: the short block's hover row shows Filter before Copy and
    Rerun; its click opens "Filter · printf …", "0 of 3 lines" under the kept `WARN` (REQ-002).
  - 528-11-running-a / 528-11-running-b: the loop filtered on `5`: "3 of 26 lines", then five
    seconds later "8 of 52 lines" (REQ-011).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #528); `marley_workbench.md` (a section for `block_filter.rs`);
  `terminal_blocks.md` (`filter.rs`); the touchpoints rows for `terminal_view.rs` and
  `terminal_element.rs` (written before the code); the plan's T1 row.
- **Knowledge:** AD-claude-528-a-blocks-filter-is-a-panel-over-the-grid-001. The gate's
  `async_block_without_await` was already a lesson twice over
  (L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001,
  L-claude-507-sync-work-for-the-background-executor-goes-in-future-lazy-001): recall missed it,
  so no new block.
- **Brain:** consultation e5f0c4b08b7e45f7911ba03663f72885 closed with a decision (follow-up
  2026-10-29).
- **Closed:** TICKET-528 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **These notes restored (2026-09-29):** the Phase 2 write looked for a `## Phase 2 — Code`
  heading the queued notes did not have, and a slice from `find`'s -1 cut the file to its first
  line; it went into a8f4d1594e so. Rebuilt from the queued notes and the entries as written.
