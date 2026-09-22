# marley_app (app-shell) — the runnable Marley window (M1.A finale)

**Status:** M1 · TICKET-012 · forge #16 · sprint M1.A seq 5/5 (THE FINALE). Contract:
[`SPEC-app-shell.spec.md`](../specs/SPEC-app-shell.spec.md). **M1.A-minimal cut** = the first runnable
Marley: one window + a terminal pane + the input. The full workspace shell (docks, pane-group algebra,
command palette, multi-theme) grows in **M1.B** — the **keymap** layer landed first (TICKET-018, #18).
**`visual_acceptance` is SET** — gate-15 applies.

## Purpose

`marley_app` is the top-of-graph app shell: `run()` boots one gpui window titled "Marley" whose
`RootView` mounts a live [`marley_terminal`](terminal_blocks.md) session + the
[`marley_editor`](editor.md) input, painted in the Dark theme from
[`marley_ui_components`](ui_components.md). Type a command, press Enter, and it runs in a real `zsh` —
segmented into per-command **Blocks** by a bundled shell-integration. **`cargo run -p marley` opens a
usable bare terminal.** Since **#380** `run()` also registers a minimal macOS application menu — a
`Quit` gpui action + a `cmd-q` binding + a "Marley" menu with "Quit Marley" — so ⌘Q quits the app
(gpui derives the menu item's key-equivalent from its own keymap at `set_menus` time, so `bind_keys`
must precede `set_menus`). The handler is `cx.quit()`, which routes through the same `[NSApp
terminate:]` path as the AppleEvent quit → the RootView-drop teardown (mcp_host/#375 discovery-file;
the #376 `fleet_subscription` join rode here until the forge transport retired at #411) runs
identically; never `process::exit`. From **#381** until the scrap-forge rip (#409/#411) the boot forge
client (#64/#69) resolved its `.mcp.json` via the pure `mcp_config::mcp_json_path` — the restored
ACTIVE project's root first, the launch cwd as fallback — evaluated AFTER the shell restore (the #376
placement), so a Finder/`open` launch (cwd `/`) still found the project's file; the same single string
fed #376's live-wire bearer decision; that per-project forge `.mcp.json` (retired #411) was distinct
from `marley_core::marley_mcp_config_file_path` (`<config>/mcp.json`, the #371 marley_mcp server
registry — which survives the rip).

## Crate layout (resolves three constraints, zero spec edits)
- **dir `crates/marley_app`** — gate-15's `visual_g` reads `component: marley_app` from the spec then
  checks `[ -d crates/marley_app ]`; a different dir would **silently skip** the UI component (the
  blank-shipped-green trap the gate exists to prevent).
- **`[package] name = "marley"`** — so `cargo run -p marley` works (`-p` is by package).
- **`[lib] name = "marley_app"`** — the frozen `marley_app::{run, RootView, ThemeRegistry}` surface M2
  panels bind to. **`[[bin]] name = "marley"`** — `src/bin/marley.rs` → `marley_app::run()`.

## Shape — the pure / shim seam
**PURE (cov 100 / MSI 100 — gpui-free, unit-tested):**
- `themes.rs` — `Theme` + `ThemeRegistry` (`builtin`/`by_name`/`default_for`/`dark_default`) wrapping
  `marley_ui_components::{Appearance, ThemeColors}` (no local redeclaration, seam-contracts §6).
  **M12.2 (#194):** the DARK palette was recalibrated for the Warp look — `background` (terminal pane / code
  view / window root) → near-black L=0.05 and `surface` (docks / sidebar / Files / footer / prompt strip) → a
  distinct gray L=0.155 (separation ΔL 0.04→~0.10); `border` → L=0.26. Legibility is PROVEN, not eyeballed:
  `ui_components` gained pure `relative_luminance(Hsla)` + `contrast_ratio(Hsla, Hsla)` (WCAG relative luminance;
  cov/MSI 100) — dark foreground/background clears AAA (15.5:1); muted-on-surface (caption) is the canary at
  5.1:1 (AA). The light theme is unchanged.
- `input.rs` — `Key`/`KeyOutcome` + `apply_key(&mut Buffer, &mut CharOffset, Key)` (insert/backspace/
  submit-on-Enter over a `marley_editor` Buffer + a caret) + `submit_line` (trim; None if empty).
  **M1.D (#28):** a real editable line — `Key` gains `Left`/`Right`/`WordLeft`/`WordRight`/`Home`/
  `End`/`DeleteForward`, each motion arm DELEGATING to the already-tested `marley_editor::movement`
  fns (so the surface is the right-fn-per-key wiring, not re-tested motion); insert/backspace work
  at any interior caret; + `split_at_caret` (multibyte char-split) so the render paints the caret
  glyph AT its offset (`{before}▏{after}`) instead of always at the line start (R35); + `split_caret_char`
  (#218 — `{before}{Option<caret_char>}{after}`) that isolates the char under the caret for the Warp-style
  block cursor (see the prompt input-row entry).
- `history.rs` — **M1.D (#29):** `CommandHistory`, a pure per-pane bounded ring for ↑/↓ recall:
  `record` (dedup-last + capacity eviction, resets navigation), `recall_prev`/`recall_next`
  (walk older/newer with a draft-stash that restores the in-progress line past the newest, clamp
  at the oldest), `detach` (an edit leaves navigation). Held on `PaneState` (per-pane); the shim
  routes ↑/↓ only when the palette is closed, records on submit, detaches on an edit key (R36).
  **M12.2 (#200, inline ghost-text):** a pure `history::suggest(prefix: &str, history: &[&str]) ->
  Option<String>` (the most-recent entry that STRICTLY starts with `prefix` → its remaining suffix; `None`
  on empty/no-match/exact; byte-safe — `starts_with` guarantees `prefix.len()` bytes is a char boundary;
  cov/MSI 100, the matrix written from the REAL `cargo mutants --list` set) drives a fish/Warp-style inline
  ghost. The prompt render paints the muted suffix after the #218 block cursor ONLY when the caret is at EOL,
  on the FOCUSED pane ([[PR-claude-per-pane-render-affordance-must-gate-on-is-focused]] — a non-focused pane's
  ghost couldn't be accepted; mirrors the #186 focused-only find-highlight), and `!completion_open` (a
  `completion.is_some()` snapshot avoids the pane-loop E0502 borrow). The `Key::Right` dispatch intercepts at
  EOL+ghost to insert the suffix — reusing `buffer.edit` + `history.detach()` (the R36 edit contract — the
  inspect critic caught the missing detach). Display-only until accepted; both render + accept are the
  `mutants::skip` shim, `suggest` carries the mutation load.
- `workspace.rs` sizing — **M1.D (#30):** `CellSize` + `plan_resize(rect, cell, current) ->
  Option<(cols,rows)>` (pure, R37): floor-divide the pane `Rect` by the monospace cell per axis,
  clamp ≥1 (only guard `cell > 0.0`; the `u16` saturating cast + `f32::max`-ignores-NaN handle the
  rest — no inert clamp), and return `Some` only on a grid change (the unchanged-guard folded into
  the Option). `PaneState.pty_size` tracks the last-applied grid (init `(0,0)`); the app.rs shim
  reads the gpui cell metric (`em_advance` × `line_height`) and calls `TerminalSession::resize`
  (R17) on changed panes — advancing `pty_size` ONLY on a successful resize (a failed ioctl must
  not poison the guard). **M17 #287:** the shim feeds `inset_top(rect, PANE_TITLE_H)` (a pure
  top-edge carve mirroring `inset_right`) — the CONTENT rect the render actually draws into — not the
  full pane, so the grid is no longer ~1 row too tall (the old skew clipped an alt-screen TUI's top
  row off-screen AND offset #280's `pane_mouse_cell` clicks; correcting `pty_size` heals both, since
  the render (`.top(r.y+PANE_TITLE_H).h(r.h-PANE_TITLE_H)`) + the mouse mapping already trust it).
  Closes the 80×24 deferral. Cell-metric fidelity (proportional vs monospace font) is a tracked
  follow-up (`intake/terminal-monospace-font-and-cell-metric.md`).
- bottom-anchored terminal — **M1.H (#49, BUG FIX):** the pane content column is `justify_end`
  (bottom-anchored) so the prompt sits at the BOTTOM with output above it and empty space at the TOP —
  a normal-terminal feel (chad found the top-anchored version in live testing). It also keeps
  `overflow_hidden` from clipping the prompt (the oldest top row clips instead). Because the render's
  anchor flipped, its INVERSE — the click→cell hit-test — had to flip too: a pure
  `viewport::bottom_anchored_row(y_from_bottom, cell_h, start, end)` (M12 #179; cov/MSI 100) maps a
  pointer from the pane BOTTOM (`row = end-1-floor(y_from_bottom/cell_h)`, `None` above the content), and
  `pane_grid_pos` + the #175 menu hit-test compute `y_from_bottom = r.y+r.h-1 - pointer.y` and use it so
  drag-select/copy and the block menu hit the right rows even in a row's bottom sub-cell band (the inspect
  HIGH catch — a render transform + its inverse hit-test are a coupled pair; #179 fixed the title-bar +
  paint-remainder skew the M11 top-anchored `row_at`/`row_hit` left). Verified via window capture
  (`screencapture -l<windowid>` → prompt at
  the bottom). (R39/R47.)
  **M12.2 (#198):** the pane render now draws a SCROLLBAR THUMB on the right edge + a jump-to-bottom button.
  Pure geometry (viewport.rs, cov/MSI 100): `scrollbar_thumb(content, capacity, start) -> Option<(top_f,
  height_f)>` (fractions of the track; `None` when `content <= capacity`, so no scrollbar on a fitting pane;
  no `.min(1.0)` needed — the guard bounds `capacity/content < 1.0`) + `at_bottom(start, content, capacity)
  = start + capacity >= content`. The shim (in the cooked-Block branch) draws the thumb as an `.absolute()`
  `muted` bar (`top_f`/`height_f` × the pane content height `r.h - PANE_TITLE_H`, min 16px, clamped in-track)
  when `Some`, and a ▾ round `surface`/`border` button (`.absolute()` bottom-right, `.occlude()`) when
  `!at_bottom`, whose `on_mouse_down` reuses the tested `Viewport::scroll_down(content, content, capacity)`
  re-anchor (the same call the wheel-down makes). Display-only thumb — drag-to-scroll deferred. Two self-test
  lessons: a scroll/click drive must `focus` Marley FIRST or the synthetic events hit the host terminal
  (Warp) that covers it ([[PR-claude-selftest-focus-marley-before-driving-input]]); and a ~22px button click
  can miss because `screencapture -l` includes the drop shadow → verify a small button's BEHAVIOR via its
  mechanism, not a pixel-hunted click ([[PR-claude-selftest-screencapture-shadow-offsets-small-target-clicks]]).
- block nav + clear — **M1.G (#48, FINALE):** a pure `nav` module (cov/MSI 100) — `block_boundary_rows(
  line_counts)` gives each command block's header content-row (`1 + output_lines` accumulated, matching
  the render's row walk), `jump_target(boundaries, top, forward)` picks the nearest boundary above/below
  (the backward `.rev()` = nearest-previous, not first; no wrap at the ends). ⌘↑/⌘↓ (keymap →
  `jump_focused_block` → reuse #47's `scroll_focused_to_row`) jump between commands; cmd-K
  (`clear_focused`) sends the shell a `\x0c` Ctrl-L + snaps the viewport back to following — all
  app-shell shim. Inspect note: cargo-mutants doesn't mutate `.rev()`, so the backward-jump test with
  ≥2 boundaries below `top` is the direction guard, not MSI. (R52.)
- find in scrollback (cmd-F) — **M1.G (#47):** a pure `find` module (cov/MSI 100) — `find_matches(
  haystack, query)` returns every NON-overlapping, case-insensitive (`to_ascii_lowercase`, byte-length-
  preserving so the byte ranges are valid on the original) match; an empty query matches nothing;
  `match_navigation(len, current, forward)` is the wrap-around match cursor. cmd-F opens a find bar
  (`find_open`/`find_query`/`find_index` on RootView, mirroring the palette overlay): typing highlights
  matching rows (per-row `find_matches`), Enter/Shift-Enter cycle (`match_navigation`) + scroll the
  viewport to the match, Esc closes — all app-shell shim. Inspect note: `to_ascii_lowercase` (not
  `to_lowercase`) keeps ranges valid on the original; one `+`→`-` mutant is a real infinite loop the
  gate catches via timeout (the shipped loop always terminates: non-empty query ⇒ start increases).
  (R51.)
- copy (cmd-C) — **M1.G (#44):** completes copy/paste (#42 gave paste). `selected_text(rows, sel)`
  (pure, cov/MSI 100) walks the rows and reuses #43's `row_selection` to slice each selected span
  (char-safe via `row_slice`), `\n`-joined — the SAME geometry the highlight uses, so copy == what's
  tinted. `copy_payload(rows, selection)` returns `None` for an absent or empty selection (cmd-C with
  nothing selected never clobbers the clipboard with `""`). The cmd-C handler (a platform chord,
  distinct from the Ctrl-C that streams `\x03` to the PTY) builds the rows from the blocks and writes
  `ClipboardItem::new_string`. Inspect note: the highlight was extended to the command-header rows too
  (it had been output-only) so the tinted set matches the copied set exactly (R48).
- text selection — **M1.G (#43):** the pure `text_selection` module (cov/MSI 100) models a terminal
  selection — `Selection{anchor, head}` over content `GridPos{row, col}` (char cols, `Ord` = (row,col)
  so `normalized()` reads a backward drag forward) + `row_selection(sel, row, row_len)` (the per-row
  selected char span, or `None` outside). `PaneState` carries the per-pane `selection`; the app.rs
  shim maps mouse→`GridPos` (`pane_grid_pos`, the render's inverse), drags `head`, highlights the rows
  `row_selection` marks, and clears on a new command. The FOUNDATION for copy (#44) — `selected_text`
  (the `\n`-joined copy text) builds on the same geometry there (R47).
- type scale — **M1.E (#38... #39, FINALE):** the cohesion pass. A gpui-free `typography` module
  (cov/MSI 100) is the ONE place structural text size + weight live — `type_scale(Role)` (`Command→
  (14,Medium)`, `Output→(14,Normal)`, `Caption→(12,Normal)`) + `weight_value(TextWeight)→f32` (400/
  500/700, wrapped in `gpui::FontWeight` by the shim). Every ad-hoc `FontWeight::*`/`text_sm` in the
  render now routes through it (the command header = `Command` + a hover tint, output = `Output`,
  ANSI-bold = `Bold`, dock caption = `Caption` in the new `muted` color). NB: a struct return with no
  `Default` yields NO viable mutant, so the tests assert the FULL `TextStyle` per role — MSI can't
  enforce value correctness there ([[PR-claude-no-default-struct-return-needs-full-value-assert]]).
  **M12.2 (#195, Warp parity):** the scale was recalibrated to Warp's measured density — `Command`/`Output`
  → 13pt, `Caption` → 11pt (a ~1pt tightening from 14/12), and `app.rs TERMINAL_FONT_SIZE` → 13 (the mono
  cell metric derives from it, so columns stay aligned). **M13 #230:** added `Role::Nav` (12/Normal) — the
  left-sidebar/nav band — and calibrated the files/sidebar text to Warp: the files tree (which was UNSIZED →
  gpui's ~16px default) is now sized via the `files_panel` container so its rows INHERIT Nav(12) (gpui's
  `text_style_stack` cascade), and the rail Tab row (13) → Nav(12); the ~9 chrome literals at 12/11 now route
  through `type_scale` (Nav/Caption, no visual change); the `workspace.rs` `fallback_cell` default was refreshed
  14→13. **#223 CLOSED the remainder**: five more fixed chrome roles — `Panel`(13, the dock/git/diff/menu/hint
  band), `Chip`(10, fleet chips/meta), `Badge`(9), `Headline`(15, empty-state/browser-card), `Display`(22,
  the launcher title) — absorbed the last 28 hardcoded `text_size(px(N))` sites (6 landed on the existing
  Caption/Nav for free), pinned by full-value table tests; a negative grep holds the literal count at ZERO.
  No calibration — values preserved exactly (routing, not a density tune).
  `cargo run -p marley` now reads as a finished Warp-style terminal — M1.E CLOSED (R45).
  **M22 #337 — the scale became a FUNCTION of the user's setting, and `TERMINAL_FONT_SIZE` is GONE.**
  `type_scale(Role)` → **`type_scale(Role, font_size)`**: `Command`/`Output` return the live
  `appearance.font_size`; `Caption`(11)/`Nav`(12) — and since #223 `Panel`/`Chip`/`Badge`/`Headline`/`Display` — stay FIXED — **content zooms, chrome does not**, which is
  what makes ⌘= mean "the text I read gets bigger" rather than "the whole UI rescales". Note what this bullet
  used to claim: that `typography` was "the ONE place" text size lives while `app.rs TERMINAL_FONT_SIZE`
  *also* held 13. **Both were true and that was the bug** — two sources agreeing only because #195 set both
  to 13.0, so scaling either alone half-zoomed the terminal. #337 deleted the const; the ONE home for the
  numbers is now `font_zoom.rs` (`FONT_SIZE_DEFAULT`, `LINE_HEIGHT_RATIO`, and the total `clamp_font_size`
  into [8,32]), which `fallback_cell`'s guard and the `define_setting!` default both source rather than
  re-typing (that literal had gone stale TWICE). The "no viable mutant for a `Default`-less struct return"
  note above still holds — and the signature change added zero mutants, so the full-`TextStyle` asserts
  remain the only thing enforcing per-role values.
- dock panels — **M1.E (#38):** the left/right docks render as PANELS via a shared `dock_panel(side,
  x, w, h, &colors)` shim — the `surface` elevation, a header (`dock_title(side)` via the shared
  `caption_header` shim, muted Caption + `border_b` divider), a DIVIDER border on the dock's inner edge toward
  the center (`border_r` Left / `border_l` Right), and a padded content area. The label is the PURE
  `dock_title(DockSide)` (layout.rs, cov/MSI 100 — **Left→"Workspace"** (M12.1 #190 — the left dock is the
  Workspace→Project→Tab navigator since #152, NOT the file tree), Right→"Details"); the panel layout is shim.
  The `files_panel` file browser wears its own **"Files"** `caption_header` (#190) so it's distinguishable from
  the Workspace dock. The root already paints `background`, so docks/center/native-"Marley"-titlebar read as one
  cohesive surface (R44).
- prompt input row — **M1.E (#37):** the prompt is an input ROW — a `surface` strip with a `❯`
  (U+276F) `accent` marker, the cwd/git context segments, then the buffer split at the caret with a
  styled `accent` caret bar (replacing the bare `▏`). The context comes from the PURE `prompt` module
  (cov/MSI 100): `prompt_segments(info)` (a `Cwd` segment iff `pwd`, a `Git` segment iff `git_branch`,
  order `[Cwd, Git]`) + `pwd_label(pwd)` (the last path component). The `PromptInfo` is
  `TerminalSession::current_prompt()` — the precmd-staged LIVE context (terminal_blocks R27); `None`
  → the row shows just `❯` + buffer. First cut = pwd + git (virtual_env/node deferred); the input-row
  layout is the app.rs shim (R43). The full shell-ZLE prompt is the deferred #33 intake. **M12.1 (#193):**
  the input row is HIDDEN while the pane's session has a foreground command running — an agent (or any
  command) owns stdin, so Marley's `❯` buffer beside the program's own prompt was a redundant double prompt
  (Warp hides its prompt the same way). It returns when the command exits; the scrollback blocks stay
  visible throughout. The row's visibility is ONE predicate (`!is_command_running`) shared by the render
  gate AND the viewport count (see `content_rows`), so render and count can't skew.
  **M12.2 (#218, Warp parity):** the input row now matches Warp's bare prompt — the `surface` strip +
  `corner_radius` rounding are DROPPED (the prompt sits directly on the pane `background`), the `❯` marker
  and the `Cwd` segment are `muted` (a dim chevron + breadcrumb; the `Git` segment stays `accent`), and the
  `accent` caret BAR is replaced by a Warp-style BLOCK cursor — `bg = accent`, glyph `text_color = background`
  (reverse-video). The block is fed by a new PURE seam `split_caret_char(text, caret) -> (before,
  Option<caret_char>, after)` (input.rs, beside `split_at_caret` which stays as the distinct
  insertion-halves primitive; cov/MSI 100): the render draws `before | block(caret_char) | after`, and a
  standalone one-cell block (a `" "` child) at end-of-line where `caret_char` is `None`. The #88 gapless
  inner flex is preserved so the block sits flush. NB: this removed `corner_radius`'s last render consumer at
  the time (#219 then re-consumed it for the rail rows; #221 applied it to the 10 overlay cards — #224 closed).
  The self-test
  driver gained plain `left`/`right` caret-move verbs (keycodes 123/124) for the block-cursor captures.
  **M12.2 (#220, Warp parity):** the block cursor is now FOCUS-AWARE — solid `bg(accent)` reverse-video on
  the FOCUSED pane (unchanged from #218), a DIM `bg(colors.accent.opacity(0.4))` fill with the caret char in
  `foreground` on UNFOCUSED panes (reuses the `is_focused = pane_id == focused` flag already at the render
  site). In a split you now read the active pane from its bright cursor vs the dim ones. It's a dim FILL, not
  an outline, ON PURPOSE: gpui has NO box-sizing, so a `.border_1()` on the auto-sized one-cell cursor would
  inflate the box by 2px (jitter on focus change / regress the one-cell #88 flush) — a fill keeps exactly one
  cell ([[PR-claude-gpui-no-box-sizing-border-inflates]]). Shim-only (capture-validated). The drag-selection
  tint (`accent @ 0.3`) was evaluated against Warp + KEPT (can't measure Warp's selection clean-room →
  keeping the measured-original beats inventing a value).
- block chrome — **M1.E (#36):** each command Block renders a styled HEADER row — an exit-status
  indicator (a distinct icon AND color, so it reads in grayscale: a dot/running/`border`, a
  check/success/`success`, a cross/failure/`danger`) + the command in MEDIUM weight, with a `border_t`
  separator above the block; output rows below unchanged (#31). The classification is the PURE
  `block_status` module (cov/MSI 100): `exit_status_kind(state, exit)` (state-first — `Pending`/
  `Running`→`Running`; `Finished`+`Some(0)`→`Success`; `Finished`+`Some(nonzero)`|`None`→`Failure`) +
  `status_indicator(kind, &ThemeColors)` (using #35's `success`/`danger`/`border`); the header layout
  is the app.rs shim. **M13 (#232):** `status_indicator` now returns an `Icon` (not a text glyph) — the
  shim paints it as `svg().path(icon_path(icon)).text_color(gcolor)` (the #137 pattern), so the ○/✓/✗
  dingbats became clean-room SVG icons (`dot`/`check`/`cross`); both header rows gained `.items_center()`
  to center the icon box against the command text. See [`icon-audit.md`](icon-audit.md) for the full
  emoji→icon audit + brand-icon list + the follow-up plan. A HEADER (not a wrapping card) respects the
  #32 per-row viewport windowing (R42).
- clickable links — **M12.2 (#196):** URLs + file paths in block output are clickable. PURE `links.rs`
  (cov/MSI 100): `scan_links(line)` → ordered, non-overlapping `Link{range, LinkTarget::{Url|File}}`
  (per-token: an `http(s)://` URL with the linkify trim rule — trailing punctuation + UNBALANCED closing
  brackets stripped — else a file-path heuristic: contains `/` or a known code extension, `:line[:col]`
  stripped); `split_run_by_links(run, links)` slices each #31 styled run at link boundaries so the overlay
  underlines/clicks links WITHOUT changing character content (the colors + text-selection geometry are
  untouched — `row_len` is summed from the runs, not the spans). The render (app.rs shim) paints a link
  segment with an underline + an `on_click` (release, not mouse-down, and no `stop_propagation` — so a drag
  starting on a link still selects). A URL opens via `marley_command::open_url` (the http/https-guarded,
  single-argv, no-shell spawn seam); a File via `open_file_in_viewer` — which now `stat`s BEFORE reading
  (rejects a non-regular file / >2 MB up front, so a clicked `/dev/zero`/fifo from attacker-influenced output
  can't freeze the UI thread — inspect MED).
  **M12.2 (#214) — OSC 8 explicit hyperlinks now honored:** the render composes a line's links via the pure
  `links::line_links(runs: &[(&str, Option<&str>)])` — each run carrying a non-empty OSC 8 hyperlink URI yields
  an AUTHORITATIVE `LinkTarget::Url` over its byte range, then `scan_links` adds heuristic links EXCEPT any
  overlapping an explicit run (`ranges_overlap`); the merged, ordered `Vec<Link>` feeds the UNCHANGED
  `split_run_by_links` + click path. The URI is carried from the grid cell: `terminal_blocks::StyledRun` gained
  `hyperlink: Option<String>`, `coalesce_row` breaks a run on a hyperlink change (so `output_text`/R20b stays
  byte-identical — a hyperlink split shifts run boundaries only, never the joined text), and both session
  grid-reads (`term_to_styled_rows` live-grid + `full_term_to_styled_rows` command-finish) extract
  `cell.hyperlink().map(|h| h.uri().to_string())`. Back-compatible: all-`None` runs → exactly `scan_links` (#196).
  The codec + composer are pure (cov/MSI 100); the render swap is the app.rs shim.
  **M18 (#212) — the terminal↔editor fusion FOUNDATION: a `file:line:col` ref opens the editor AT that line.**
  `LinkTarget::File` gained `{ line, col }` (the pure `parse_line_col` captures the trailing `:line[:col]`
  #196 already stripped-and-discarded — the LAST digit group is the line, a preceding one the col; a non-digit
  or empty group stops the scan, caught by `parse::<usize>()`); the clickable range now covers the WHOLE
  `path:line:col` ref (inspect F1 — clicking the `:12` must jump too), unchanged for a location-less ref. The
  `open_link_target` File arm calls `open_file_at` → `open_file_in_viewer` + places the caret via the pure
  `code_view::caret_for_line_col(buffer, line, col)` (1-based → 0-based, col past EOL clamps to the line end,
  line past EOF to the last line). Four M18 fusion tickets (diagnostics gutter, jump-to-failure, trace frames,
  context menu) reuse this parser + open-at-line. Parser + caret math pure (cov/MSI 100).
  **M18 (#213) — the run→fix loop: a failed block gains a "Jump to Failure" action.** When a command
  block FAILS (`exit_status_kind == Failure`) and its output carries a `file:line` ref, the #175 block
  context menu shows a 7th (leading) "Jump to Failure" row — gated on a `has_failure` bool on
  `MenuKind::Block` computed at menu-open (`items_for` returns `BLOCK_MENU_ITEMS_FAILED[7]` vs the 6-row
  table). The pure `links::first_failure_ref(output)` picks the primary ref (the first output line carrying
  a line); dispatching reuses #212's `open_file_at`. `first_failure_ref` + `items_for` pure (cov/MSI 100).
  **M18 (#289) — errors show up IN the editor: a diagnostics gutter.** When a command block fails and its
  output references lines in the OPEN editor file, those rows' gutter line-numbers tint `danger`. The pure
  `links::diagnostics_for_file(output, open_path, root)` maps each `File{line}` ref resolving (#190, against
  `self.project_root` — the SAME root that built the stored `open_path`, inspect F1) to the open file into a
  0-based row, sorted + deduped; `open_file_diagnostic_rows()` unions each terminal pane's LAST failed
  block's rows — across EVERY terminal grid in the active project (M22 #295: the pure
  `Project::terminal_grids()`, not just the active-or-first grid `workspace()` resolves, so a failure in ANY
  terminal tab lights the gutter, not only the first) — so they self-clear on a new command, and the render
  captures the set like `efind` (binary-searched per row). `diagnostics_for_file` + `terminal_grids()` pure
  (cov/MSI 100); the shim stays `mutants::skip`.
  **M18 (#290) — F8 / ⇧F8 navigate the diagnostics.** Two Editor-scoped keymap rows (`f8`/`shift-f8`, the
  #265 KeyContext) → a `dispatch_action` arm that walks the caret through `open_file_diagnostic_rows()` via
  the pure `code_view::next_diagnostic` (first row strictly after the caret, wrapping) / `prev_diagnostic`
  (strictly before), sets the caret to `line_start(row)`, and `follow_editor_caret()`s (#270). Pure nav at
  cov/MSI 100; F8 routes because the keymap lookup precedes the editor key branch and `binding_from_keystroke`
  passes `keystroke.key` verbatim.
  **M18 (#291) — multi-frame trace frames fold into the gutter (esp. Python).** The rust panic / backtrace /
  node frames are `path:line`-shaped so #289's `scan_links` already catches them; the Python
  `File "<path>", line <N>, in <fn>` shape is the gap (the quoted path + the separated line defeat the
  `path:line` scanner). The pure `links::parse_trace_frames(output)` parses every frame in order — per line
  `python_frame` (a `split_once("File \"")`→`split_once('"')`→`split_once(", line ")`→`take_while(is_ascii_digit)`
  chain, no index arithmetic) else the first `scan_links` ref via the extracted primitive
  `first_file_ref_on_line` (also now the body of #213's `first_failure_ref`). The pure `links::trace_diagnostic_rows`
  is the twin of `diagnostics_for_file` over that frame source; `open_file_diagnostic_rows()` unions BOTH (neither
  subsumes the other — `scan_links` catches multiple refs/line, `parse_trace_frames` catches the Python shape),
  the trailing sort+dedup collapsing the overlap. No new UI — the #289 gutter + #290 F8 carry it. Proven live:
  a Python-traceback failure → `trace_rows=[2] diag_rows=[]` → the gutter line lit danger-red.
  Parser + `trace_diagnostic_rows` pure (cov/MSI 100). *Known scope limit (follow-up): `open_file_diagnostic_rows`
  reads only `workspace()`'s active-or-first terminal grid (via `terminal_grid_index`), so a failed block in a
  DIFFERENT terminal tab is not scanned — surfaced by the live drive, filed as a follow-up to iterate all grids.*
  **M18 (#292) — close the run→fix loop: re-run the last failed command.** A pure
  `block_status::last_failure_block_index(kinds)` (`rposition == Failure`) finds the MOST RECENT failure —
  distinct from `rerun-last` (⌘⇧R, the most recent FINISHED command via `focused_terminal`). A cockpit palette
  command "Re-run Last Failed Command" (`CommandId(11)` → the `rerun-last-failed` verb) scans `workspace().states()`
  (a HashMap → collect + `sort_by_key(id.0)` for a deterministic lowest-PaneId pick; skip `is_command_running`
  panes so a busy one can't shadow an idle failed one) and re-runs it via the #175 `rerun_block`. It works with
  the EDITOR focused (no focused terminal) — the whole point vs `rerun-last`. Clear-on-green is inherent:
  `rerun_block` writes a NEW last block, and #289's Failure-only gutter drops a green last block (no new code).
  `last_failure_block_index` pure (cov/MSI 100). Live-driven (palette → re-run). The cross-pane "most-recent
  failure" selection + the per-block "re-run to verify" affordance are follow-ups (#295 covers the multi-terminal
  aggregation shared with #289).
  **M18 (#293) — file-ref right-click context menu.** A RIGHT-click on a `LinkTarget::File` link in block
  output opens a 4-item menu (Open in Editor #212 / Open in Split Right #246 / Reveal in File Tree / Copy Path).
  `context_menu.rs` gains `MenuKind::FileRef` (a payload-less marker — `MenuKind` is `Copy`, a ref carries a
  `PathBuf`, so the shim holds the target in `file_ref_menu_target`, set at menu-open) + `FILE_REF_MENU_ITEMS`
  + the `items_for` branch. `marley_project` gains the pure `FileTree::reveal(path)` — expand every ancestor
  dir (`expand_parts`), then the visible-row ordinal via the tested `path_at` scan; `None` when the path isn't
  in the tree. The link span's `on_mouse_down(Right)` calls `open_file_ref_menu` + `cx.stop_propagation()` so
  the pane's #175 block/split right-click can't ALSO fire (proven in the live drive: the FileRef menu appears,
  not the block menu). The 4 dispatch arms in `run_context_menu_action` `take()` the target and reuse the
  shipped `open_file_at` / `split_file_pane` / `reveal`→`files_scroll` / `write_to_clipboard`. `items_for(FileRef)`
  + `reveal`/`expand_parts` pure (cov/MSI 100). Follow-ups: a persistent reveal highlight, disabled items, a
  modifier-click, a URL-link menu.
  **M18 (#294) — cwd↔editor link (closes the M18 fusion train).** Two cockpit palette commands
  (`CommandId(12/13)` → `open-terminal-here` / `cd-terminal-here`) tie the surfaces' cwds. A new pure
  `cwd_link.rs`: `dir_of(path)` = the file's parent (`None` for a bare/empty path) + `cd_command(dir)` =
  a POSIX single-quote-wrapped `cd '<dir>'` (`'`→`'\''`, injection-safe). "Open Terminal Here" spawns a
  new terminal rooted at `valid_dir_or(dir_of(active editor file), root)` (#281) via the extracted
  `spawn_terminal_tab_in(cwd)` (the shared spawn+add-tab tail of `new_terminal_pane`); "cd Terminal to
  Editor Dir" `write_command(cd_command(dir))` to the focused terminal (idle-guarded; from the editor tab
  it targets the first terminal grid via `terminal_grid_index`). `dir_of` + `cd_command` pure (cov/MSI 100);
  live-driven (open a file → "Open Terminal Here" → the new terminal's `pwd` is the file's dir). A per-tab/
  file-tree affordance + a keybinding are follow-ups.
- terminal font — **M1.E (#34):** terminal content (Block command/output, prompt, alt-screen grid)
  renders in a monospace font (`TERMINAL_FONT="Menlo"` @ 14pt, clean-room) applied to the pane
  content container; the cockpit chrome keeps the UI font. The pane cell metric is read from that
  font — width from `em_advance`, height from the pure `fallback_cell(font_size)` (`0.6·size` /
  `1.2·size`, `>0` guard→14.0; cov/MSI 100) which the terminal rows ALSO render as their
  line-height, so the metric (feeding #30/#32/#33) and the drawn glyphs agree. The scroll-wheel
  pixel→rows divisor uses the same `fallback.h` (R41). Closes the monospace-font intake.
  **M22 #337 — both literals above are now SETTINGS, and the guard sources its default.** The size is
  `appearance.font_size` (default 13, clamped to [8,32], live via ⌘=/⌘−/⌘0) and the family is
  `appearance.font_family` (hand-edited; empty ⇒ the built-in `TERMINAL_FONT`). **M22 #344 — an unresolvable
  family no longer degrades silently:** `new_in` probes it at boot via `font_resolves` (gpui's PUBLIC
  `TextSystem::all_font_names()` membership — NOT the private `font_id`, NOT the panic-prone `resolve_font`) and,
  on a non-resolve, the pure `settings::resolve_font_family` resets it to `""` (the built-in mono, so the grid
  stays aligned) AND raises a `status_flash` naming the family. **M22 #361 extends this to a resolvable-but-
  PROPORTIONAL family** (e.g. `Helvetica`) — which #344's resolve check accepts but which still breaks the grid:
  `new_in` also probes monospace-ness via `font_is_monospace` (a shim comparing gpui's per-char `TextSystem::advance`
  of `i` vs `m`), gated on `font_resolves` first (`resolve_font` panics on a missing font). The pure decision is
  `settings::is_monospace_advance` (an i/m advance ratio within a tolerant `MONOSPACE_EPSILON` = 5%, size-independent)
  feeding a fourth `resolve_font_family` arm (`resolves && !monospace` → built-in + a "is not monospace" flash). The
  decision is unit-tested (cov/MSI 100 in settings.rs); the gpui metric read is a boot shim (`mutants::skip`, like
  `font_resolves`) — the resolvable case is not headless-testable (`#[gpui::test]`'s `NoopTextSystem` resolves no
  real font). `fallback_cell`'s `>0`
  guard no longer answers **14.0** — it reads `font_zoom::FONT_SIZE_DEFAULT`, and the `1.2` is
  `font_zoom::LINE_HEIGHT_RATIO`. That guard's literal had **gone stale twice** (it still said 14 after #195
  moved the app to 13, and this very sentence said so too), which is the whole argument for sourcing it: a
  second copy of a number is a second chance to drift. The `0.6·size` fallback WIDTH is now used only as a
  fallback — #337 pointed the terminal hit-test at the **measured** `em_advance`, since the derived 0.6 ratio
  disagrees with Menlo's real 0.60205 em by ~0.27 cells per 80 columns (scale-INVARIANT: it is wrong at 13pt
  from ~column 146, not something zoom introduced).
- interactive/raw mode — **M1.D (#33):** the shim maps a gpui `Keystroke` → `marley_terminal`'s
  gpui-free `KeyInput` (`key_input_from_keystroke`, filtering ⌘/platform chords), and routes via
  `input_route(session.is_alt_screen(), ctrl)`: `Raw` → `write_bytes(encode_key(..))` streams to the
  PTY (BEFORE scroll/history so alt-screen arrows reach the program), `Cooked` → the local
  `apply_key`/Blocks path. The render switches on `is_alt_screen()` — the live colored grid
  (`grid_styled_rows`) for full-screen programs vs the Block+viewport view. Makes vim/top/less +
  Ctrl-C work (R40). Prompt tab-completion is deferred (intake `prompt-shell-line-editing-model.md`).
- TUI mouse reporting — **M17 (#280):** when a program requests tracking (the DECSET flags via
  `session.mouse_modes()` — a gpui-free TermMode snapshot), the grid's left clicks/drags/wheel
  encode through the pure `marley_terminal::mouse_report` (SGR 1006 / legacy X10 / the
  alt-scroll arrow fallback, the exact ctlseqs bit matrix) and stream to the PTY instead of
  driving Marley's selection/scrollback; `pane_mouse_cell` maps to the VISIBLE pty cell
  (bottom-anchored, 1-based) — distinct from `pane_grid_pos`'s scrollback rows. Drags report
  once per cell (`last_mouse_cell`); ⇧ bypasses everything (the local copy-out hatch); tracking
  off = byte-identical local behavior. Left-only v1 (right = Marley's menu). Known follow-ups:
  DECCKM (#286) and the M5-era pty_size title-bar row skew (#287).
- click selection — **M17 (#279):** double-click = the same-class run under the pointer (the
  pure `click_selection` decision over `word_bounds_at`'s three-class table — Word keeps
  `./-_~:@?&#%` so paths/URLs select whole; `()[]{}<>"',;=`+backtick are their own runs; iTerm's
  same-class-run convention), triple-click = the whole row, ⇧-click extends the head (anchor
  sacred, any count). One pure fn feeds the #43/#44 Selection/copy pipeline — highlight == copy
  by construction. The row text is fetched ONLY for the double/triple arms (a plain/⇧ click is
  zero-alloc); ⌘ folds to the single seed (the #196 link chord untouched). Word-snap drag after
  a double-click is the recorded follow-up (plain cell-extend today).
- cwd inheritance — **M17 (#281):** new splits (⌘⇧L/⌘⇧J + menu) and new terminals (⌘T/⌘D/"+")
  spawn in the FOCUSED pane's live prompt pwd (the #201 shell-integration source) when it is a
  real absolute directory, else the project root (#160's behavior as the fallback). ONE
  validation authority `valid_dir_or` (non-empty ∧ absolute ∧ is_dir); the #205 restore's
  `cwd_or_root` delegates (strengthened: relative persisted cwds no longer resolve against the
  app process cwd). `launch_agent` stays at the project root deliberately; ⌘T from an editor tab
  inherits the most-recent working terminal's cwd (a read-only inherit — safe, unlike the #278
  hidden-mutation class). The cwd is an exec-cwd `PathBuf`, never shell-interpolated.
- readline keys at the cooked prompt — **M17 (#278):** ⌃A/⌃E/⌃K/⌃U/⌃W/⌃Y over Marley's own prompt
  buffer — the pure `input::ReadlineOp` + `op_for_ctrl_key` (a CLOSED 6-map; ⌃C/⌃D/⌃Z deliberately
  absent) + `apply_readline` (a ONE-slot kill on `TerminalPane.kill`; an EMPTY kill never clobbers
  the slot; ⌃W kills to the #257 word boundary — bash's ⌥⌫, not unix-word-rubout; consecutive kills
  do NOT coalesce — the append + the full ring are recorded polish). The arm claims its six chords
  BEFORE `input_route` (which Raw-routes EVERY ctrl), gated `ctrl && !alt-screen && !running &&
  !⌘ && !⌥` AND `active_tab().grid().is_some()` — the inspect-HIGH: without the terminal-tab gate
  the chords edited the HIDDEN pane's prompt on editor/cockpit tabs (`focused_terminal` falls back
  — the #71/#283 class). A matched chord is OWNED even when it no-ops (an empty ⌃K must not stream
  0x0B); everything unmatched falls to the route byte-identically.
- `viewport.rs` — **M1.D (#32):** the scrollback `Viewport` (pure, cov/MSI 100) — a following-vs-held
  scroll state + `visible(content, capacity) -> (start, end)` (bottom `capacity` rows while
  following; a held `top`-window that HOLDS as content grows otherwise), `scroll_up` (materialise at
  the bottom, no-op when all fits), `scroll_down` (re-anchor at the bottom), + `scroll_steps` (a
  fractional-row accumulator so trackpad sub-row deltas aren't lost). Held on `PaneState` beside
  `scroll_remainder`; the shim counts content rows, reuses `pty_size` rows as capacity, slices the
  render to the window, and wires wheel + PageUp/Down + Shift+Arrows (R39). `Workspace::state_mut`
  lets the wheel target the hovered pane.
- `color.rs` — **M1.D (#31):** the ANSI color mapping (marley_app holds gpui, so the `Color → Hsla`
  step lives here, not in the gpui-free `marley_terminal`). `AnsiPalette::from_theme` (an original
  16-color table + the theme's fg/bg), `ansi_color_to_hsla` (Named table / Indexed 6×6×6 cube +
  grayscale ramp / Spec passthrough — pure, cov/MSI 100), `hsla_from_rgb` (direct `Rgba` channels —
  no bit-pack, to avoid an equivalent mutant), `run_paint` (INVERSE swaps fg/bg, BOLD → weight). The
  render paints one colored span per `Block::output_styled()` run. (Superseded the old
  `terminal_view.rs` `block_row` plain-text path.)
- `shell_integration.rs` — `marley_zsh_init()` (the Marley-original zsh rc) + `write_shell_integration_in`
  + `session_env`.
- `keymap.rs` (M1.B, #18) — `KeyBinding` (modifiers + base key) + `Keymap` (`default_bindings` /
  `action_for`): the chord→action map (cmd-shift-p→open-command-palette, cmd-d→new-terminal,
  cmd-w→close-pane; `None` for an unbound chord). gpui-free. **M12.2 (#197):** ⌘⇧L→`"split-right"` /
  ⌘⇧J→`"split-down"` (vim l/j) give the tile-split keyboard access (was right-click-only, dispatching to
  `split_focused_pane`); ⌘D's action was renamed `"split-pane"`→`"new-terminal"` (honest — ⌘D makes a new
  terminal, #135). **M16 (#265) — context-scoped:** each binding may carry a `KeyContext`
  (`Terminal`/`Editor`; `None` = global); the active tab publishes its stack (`Tab::key_context()`:
  terminal→`[Terminal]`, editor→`[Editor]`, cockpit→`[]`) and `action_for(chord, stack)` resolves
  deepest-context-first with global as the fallback — so ⌘D is `"add-next-occurrence"` on the editor
  (the pure `marley_editor::add_next_occurrence` — M19 #298 ADDS the match as a new cursor) while staying `"new-terminal"`
  everywhere else, and ⌘F (`"open-find"`, the scrollback bar) is Terminal-only (was a hardcoded
  context-blind arm). `chords_unique_scoped()` + a `debug_assert` in `default_bindings` guard a
  duplicate `(chord, context)` pair — the same chord under disjoint contexts is legal (the model's
  point), and resolution never ties (semantics per the gpui Apache-2.0 KeyContext model, Marley-original
  code — see docs/zed_architecture/subsystems/09-vim-keymap-contexts.md §A.7).
- `palette.rs` (M1.B #19, M1.C seq-4 #25) — `Command` / `CommandId` / `ScoredCommand` +
  `filter_commands` (the `nucleo` subsequence ranking over a static `&[Command]`: empty query →
  registration order; non-empty → ranked, non-matches excluded, MAX field score, descending,
  reg-order ties) + `PaletteState` (the selection model: ↑/↓ clamp at both ends, reset-on-edit,
  `activate` → the SELECTED `CommandId` or `None` on an empty list) + `action_for_command`
  (CommandId → verb, routed through the ONE `dispatch_action` the chords use — so Enter runs the
  command; a consistency test pins every listed command to a real verb). The M1 LOCAL static-list
  wrapper; migrates onto `marley_search_core::SearchMixer` in M2. gpui-free (nucleo only).
  **M12.2 (#222, Warp parity — the final #216-222 warp-parity ticket):** the palette command-row renders
  each command's shortcut as bordered rounded KEYCAP CHIPS via
  `KeyboardShortcut::parse(&binding.display()).render(&colors)` (was plain `.keys().join(" ")` text), right-
  aligned by giving the title a `flex_1` cell so the chips sit at the row's right edge. The `KeyboardShortcut`
  render shim (ui_components render/keyboard_shortcut.rs, `mutants::skip` ACCEPTED-UNTESTABLE) was upgraded
  from a bare `bg(surface)` to a real keycap — per-chip `border_1`+`border_color(border)`+`rounded(
  corner_radius)`+`px_1`+`muted` text, a `gap_1` between chips. The PURE `KeyboardShortcut::parse`/`keys`
  (cov/MSI 100) are untouched. (Deferred micro-polish: the chip fill is `surface` = the card bg, so only the
  border outlines it; a `background`-inset fill would read as a proper keycap — a trivial follow-up.) The
  widgets-gallery bin (the other `render` consumer) inherits the keycap. This CLOSES the #215 Warp-parity
  thread — #216 (density) · #217 (block hover) · #218 (prompt) · #219 (sidebar) · #220 (cursor) · #221
  (radii/borders) · #222 (palette chips) all shipped.
  **M22 (#306, shadowed-chord honesty):** the keycap chip is now resolved PER ROW against the FOCUSED surface —
  a pure `palette::displayed_binding(command, keymap, stack)` returns a command's chord only when
  `keymap.action_for(chord, stack)` still equals that command's own `action_for_command(id)`, else `None` (no
  chip). So a context-scoped row that SHADOWS a chord (⌘⇧L → `select-all-occurrences` on the editor, #298) no
  longer leaves the palette advertising the global "Split Right" chord where it will not fire; an Editor-scoped
  chord (⌘⇧\ go-to-bracket) likewise shows only on the editor. Consumer-only over the existing `action_for`
  engine; the chip decision matches what the keydown handler dispatches (the same `active_tab().key_context()`).
  **M12.2 (#199, live theme picker):** the palette now registers a DYNAMIC "Theme: <name>" command per
  `ThemeRegistry::builtin()` theme in a `THEME_BASE` (2000) id range — mirroring the #87 `CONNECT_BASE`
  saved-host pattern (the id ranges are disjoint: statics <9 · connect 1000+i · theme 2000+i). The pure
  `palette::dynamic_command_index(id, base, count) -> Option<usize>` (cov/MSI 100; renamed from
  `theme_pick_index` by #204 now that it serves two ranges) resolves a range id to its index;
  `handle_palette_key`'s dispatch (a chained `else if` after the connect branch) calls the existing
  `set_theme` (applies the colors live — the render reads `self.theme.colors` each frame — AND persists via
  `persist_theme`, the tested settings round-trip). `ThemeRegistry::builtin()` is a deterministic `vec![Light,
  Dark]`, so `THEME_BASE+i` aligns at registration + dispatch. The registration loop + the dispatch are the
  `mutants::skip` shim; the whole mutation burden sits on the pure `dynamic_command_index`.
  **M12.2 (#204, command-palette workflows):** a new pure `workflows.rs` — `Workflow{name,command,params}`
  (serde; `#[serde(default)] params` so a hand-edited `[[workflows]]` missing the auto-derived `params` key
  loads empty, not a wipe-all) + `substitute(template, &BTreeMap) -> Result<String, MissingParam>` (a
  `str::find` scan — `{{name}}`→`args[name]`, a missing token→Err, single/unterminated braces literal,
  byte-safe; cov/MSI 100, the real 16-mutant set killed by the T1–T10 matrix incl T10 for the dangling-`{{`
  else branch) + `params_of` (distinct trimmed names). Persisted via a `Workflows: Vec<Workflow>` setting
  mirroring #87 `RemoteHosts` + `persist_workflows`. The masked shim: `dynamic_command_index` (above) is now
  SHARED by the `THEME_BASE` and the new `WORKFLOW_BASE` (3000) ranges; the palette lists saved workflows as
  "Workflow: <name>" commands + a static "Save Command as Workflow…" (`SAVE_WORKFLOW_ID = CommandId(9)`,
  dispatched specially by id). Invoking a NO-param workflow inserts the substituted command into the cooked
  prompt buffer (the #200 insert idiom — no auto-run). **#227 shipped the follow-ups:** a `{{param}}`
  workflow now opens the GUIDED param-prompt modal (pure `workflows::ParamPrompt` — one value per param,
  Enter advances, Done zips → `substitute` → the factored `insert_at_prompt`; the modal renders via the
  Recipe-C `naming_overlay_card`, React-approved at the POC's `ParamPromptCard` first; a hand-desynced
  settings entry flashes rather than silently inserting nothing) instead of dumping the raw template.
  Saving captures the focused terminal's last history command into a `naming_workflow` inline draft (mirrors
  the #177 `renaming_tab` modal) → append the `Workflow` + `persist_workflows` + the ONE builder
  `rebuild_workflow_commands` (retain `[3000,4000)` + re-push — #227 converted the old append, honoring
  the #204 R1 note: the ids are positional, so save AND the new "Delete Workflow…" picker
  (`DELETE_WORKFLOW_ID = CommandId(32)` → a finder-family picker; Enter removes + persists + rebuilds)
  both re-mint the whole block; append-drift is structurally dead). Both #227 overlays are #415
  roster/blocked/snapshot members (the overlay inventory is 28 states). Still out (recorded): workflow
  rename/reorder/template-edit — the settings file remains their surface.
- `layout.rs` (M1.B, #20) — the `PaneGroup` tree algebra (single / split[Before/After + axis] /
  close[collapse to survivor + `LastPane`/`PaneNotFound`] / panes[depth-first] / neighbor[the R12
  boundary-adjacency rule via a recursive `find_neighbor`, incl nested 2×2] / equal-ratio renorm) +
  `DockSide` / `DockState`. A frozen M2-facing contract. gpui-free. (`PaneId` gained `Hash` in
  M1.C.)
- `workspace.rs` (M1.C, #23) — the multi-pane workspace model (R27–R30): `pane_rects` (depth-first
  rect tiling — axis partition × ratios, origin accumulation, area-exact; no remainder arm per the
  D-2.2 equivalent-mutant finding) + `inset_right(rect, inset)` (M12.1 #189 — a pure right-gutter
  shave: `w−inset` clamped ≥0, x/y/h unchanged; the shim feeds the terminal tiling band through it)
  + `Workspace<S>` (the per-pane {session, `Buffer`, caret}
  registry GENERIC over the session handle behind a spawn-per-call seam — mock-tested at cov
  100/MSI 100) + the focused-pane model (`split_focused` spawns FIRST then focuses the new pane;
  `close(pane)`/`close_focused` RETURN the removed state so the CALLER owns the blocking teardown;
  focus-after-close = depth-first predecessor-or-successor; registry keys == `group.panes()` after
  every op). gpui-free.
- `region_widths` in `layout.rs` (M1.C, #24) — the 3-region width partition (R31): Open dock →
  the dock width, Closed → exactly zero, center = the remainder CLAMPED AT ZERO (an unclamped
  remainder reversed the layout below 440pt — inspect-probed). The render sizes both dock panels
  AND insets the center pane tiling from this ONE computation; the keymap's cmd-b / cmd-shift-b
  chords toggle the docks (R19 amended to five chords).
- `settings.rs` (M1.C, #26) — the settings schema (`appearance.theme` / `docks.left`/`right` /
  `terminal.cols`/`rows` via `define_setting!`) + `applied_from` (reads a loaded
  `marley_settings::SettingsManager`, VALIDATES the saved theme against the registry — unknown →
  Dark) + `applied_defaults` + the `*_in(dir)` IO seam (`settings_file_in` / `load_manager_in`) +
  `persist_theme`/`persist_dock_*` (R33/R34). `RootView` boots from `marley_config_dir()` and
  persists on theme/dock change, so the theme + dock state survive a relaunch — the M1.C
  end-to-end proof. marley_app is `marley_settings`' first consumer (+ a direct `marley_core` dep
  for the config path). gpui-free.

**ACCEPTED-UNTESTABLE (the ONLY shim — `app.rs` + `bin/marley.rs`, `mutants::skip` + rust_cov-excluded,
asserted by the headed visual harness):** the gpui `RootView` Render — since #23 (M1.C) a MULTI-PANE
render: one absolutely-positioned div per `pane_rects` rect, the focused pane bordered in the theme's
`accent` (unfocused: `border`), per-pane click-to-focus listeners, and a `.occlude()`d palette overlay.
Since M12.1 (#189) the terminal-tab tiling band is `inset_right`-shaved by `PANE_GUTTER` (8pt) before
`pane_rects`, so the rightmost pane ends inside the window with a right margin (split or single) instead of
flush against the edge; cockpit/code tabs are one full-screen panel and keep the un-inset `center_bounds`.
(clicks on the open palette cannot refocus panes beneath it) — the `run()` event loop, the **pump-ALL
timer** (pumps EVERY pane's session each 16 ms frame; panes whose shell exited are auto-closed unless
last, their state dropped on a REAPER THREAD because a session drop blocks in the child reap), the
`Keystroke` → `Key`/`KeyBinding` translations + the `on_key_down` **keymap chord interception** routing
prompt input to the FOCUSED pane (cmd-shift-p opens the palette overlay [#19]; cmd-d/cmd-w
`dispatch_action` split/close the FOCUSED pane via the workspace [#23]), and the `&BlockList` iteration
(the Block content types are `pub(crate)`-constructed, unreachable to a unit test). Every pane/focus/
layout DECISION lives in the pure `workspace` module.

The pump-ALL timer also drives the **action-confirmation flash** (#77, M2.D finale): a pure
`flash::Flash { message, remaining }` counts down one pump tick per 16 ms frame (`FLASH_TICKS ≈ 2 s` —
`Date::now` is unavailable, so the steady pump IS the clock); the pump `tick()`s it (clearing to `None` at
0) while a control/write action (copy / send / broadcast / claim / comment) sets a fresh flash. A passive
bottom-center toast renders it. The `Flash` logic is the tested pure surface; the render + the countdown
wiring are the masked shim.

## Shell integration (chosen over a hollow blocks-only shell)
`marley_terminal` segments output into Blocks **only** from shell DCS hooks (R8), and a bare shell emits
none. So `marley_app` ships a **Marley-original** zsh rc (`marley_zsh_init`) that registers `preexec`/
`precmd` hooks emitting Marley's own DCS frames — `init;id=1`, `preexec;command=…`, `precmd;exit=…;pwd=…`
(framed `ESC P q <payload> ESC\`, **AnsiCQuoted** on every frame since TICKET-022) — the EMIT counterpart
to `marley_terminal::decode_hook`. The rc's `__marley_quote` (a no-fork `__MARLEY_REPLY` zsh idiom)
escapes each dynamic value — `\` `;` ESC `\n` `\t` `\r`, backslash first — and the decoder splits on
UNESCAPED separators before un-escaping (SPEC-terminal-blocks R24), so a command containing `;`/`=`/quotes
(`ls; pwd`, `--flag=value`) round-trips into its Block EXACTLY and a `;`-containing `$PWD` cannot inject
phantom prompt fields (R25/R26). `precmd` captures `$?` before calling the quote fn (a zsh fn call resets
`$?`). It is written to a per-session `ZDOTDIR` (`session_env`) so zsh sources it on startup — proven by
headless real-zsh integration tests incl. a `;`/`=`/quotes torture round-trip. (Remaining M1 limitation:
raw non-UTF-8 command bytes drop the hook — `UndecodablePayload`.)

## M4 — The Code Panel (read-only code + diff viewer)
A read-only viewer for reading code and reviewing the diffs agents produce. All logic is PURE (cov/MSI 100)
behind the `app.rs` shim (the overlays + the `git diff` spawn are masked):
- **`code_view.rs`** — `code_lines` (split · tab-stop-expand · char-truncate) → `CodeViewState`; `gutter_width`/
  `gutter_label` (#100 numbers); `visible_range`/`jump_to` (#101 scroll window); `viewer_open_path` (#98 dir
  guard); `parse_file_ref` (#105 `path:line` tokens); `is_probably_binary` (NUL-in-8000) + `viewer_size_ok`
  (≤ `VIEWER_MAX_BYTES` 2 MiB) (#106 guards). The viewer overlay opens from the ⌘P finder (⌘↵) or the tree
  (⌘-click); ↑/↓/⌘↑/⌘↓ scroll; Esc closes.
- **`code_syntax.rs`** — a clean-room single-line lexer (`language_of` + `highlight_line`) for Rust/TOML/JSON/
  Shell via a `LangSpec` (keywords/line-comment/quotes); Markdown/Plain pass through. No tree-sitter/syntect (§20).
- **`git_diff.rs`** — `parse_diff` (unified `git diff` → `FileDiff`/`Hunk`/`DiffLine`); `diff_rows` (flatten to
  colored `DiffRow`s); `agent_diff_summary` ("N files · +A −R"). The `git diff` spawn is a READ-ONLY masked
  adapter routed through `marley_command::blocking` (§14 — `std::process::Command` is disallowed). ⌘⇧D opens
  the working-tree diff overlay; ⌘-click an agent (Agents dock section) opens it too; a diff file header
  clicks through to the file (#105).
- **`code.tab_width`** (settings) — a read-only config knob (mirrors `terminal.cols`) the viewer honors.

## M5 — The Warp Workspace (the IDE cockpit layout)
Reshapes the shell toward Warp's IDE: a sessions sidebar on the left, panels beside the terminal, a top
search bar, and a git panel. All logic PURE (cov/MSI 100) behind the `app.rs` shim:
- **`workspace.rs`** — `PaneKind{Terminal,FileTree,CodeView,Git}` on each `PaneState` (#107, the typed-pane
  foundation); `pane_title`/`pane_icon` (#108, the per-pane title bar with an × close).
- **`sessions.rs`** — the left sidebar: `session_rows` (#109) + `group_sessions` (#110 workspace groups) +
  `session_row_icon`/two-line rows (#111) + `filter_sessions` (#112 "Search tabs" fuzzy filter). Promotes the
  #68 Fleet overlay into persistent chrome; a row click focuses its pane.
- **`file_tree_view.rs`** — `file_icon` (extension → glyph) + `entry_is_dimmed` (dot-files) (#113, the
  explorer look on the left-dock Files tree).
- **`code_view.rs::viewer_split`** (#114) — the code viewer is a right-side panel beside the terminal (the
  terminal shrinks), not a modal.
- **`git_diff.rs`** — `parse_status`/`change_summary` (#115, the ⌘⇧C source-control panel) + `commit_enabled`
  (#116). **#116 is Marley's FIRST git write** — a confined adapter (`git add`/`restore --staged`/`commit -m`
  via `marley_command`, argv-not-shell, NO push/force/rewrite; `AD-claude-git-write-confined-to-add-restore-commit-001`).
- **`command_bar.rs::search_everything`** (#117) — the top "Search sessions, agents, files…" bar: one fuzzy
  search across sessions + files + palette actions, kind-tagged.
- **`settings.rs::serialize_layout`/`restore_layout`** + `workspace.layout` (#118 finale) — the git-panel
  state persists across launches.

## M6 — The Warp Layout (typed panes)
Reshapes the shell into Warp's real pane grid (per chad's 3 Warp screenshots: sessions left, titled
tileable panes for terminal/files/code/git, cockpit tabs at the top, no right rail). The structural
enabler is **#120** (`workspace.rs`): `PaneState<S>` no longer hard-codes a terminal — it holds a typed
`PaneContent<S> { Terminal(TerminalPane<S>) | FileTree | CodeView(CodeViewState) | Git }`. The terminal
fields moved into `TerminalPane<S>`; `PaneState` exposes `kind()` (derived from the variant),
`terminal()/terminal_mut()`, and `code_view()`. `Workspace` gained `open_pane(axis, dir, content)` (a
session-less split, sharing `insert_focused_split` with `split_focused`) plus `terminal(id)` /
`terminal_mut(id)` / `focused_terminal[_mut]()` narrows so the app.rs shim's ~120 terminal-field accesses
route through `Option`-returning accessors (no panic when a pane is session-less). The
split/close/focus/neighbor algebra is unchanged (pure `PaneGroup` geometry + `PaneId`).

**Landed (#120–131, sprint #17 closed).** The three former right-side surfaces are now real grid panes: the
file explorer (#123), the read-only code viewer (#124), and the git source-control panel (#125) — each
retired its M4/M5 side-panel + that panel's persisted flag (the obsolete #118 `workspace.layout` mechanism
went with the git panel). The grid itself persists via `grid_layout` (`serialize_grid`/`restore_grid` +
`pub default_grid`, the single-Terminal first-run default) behind the `workspace.grid` setting (#122).
**#205 — per-terminal cwd:** the grid blob now optionally carries each terminal's working directory — a
Terminal leaf `t` → `t=<cwd>` when a framing-safe cwd is known (a new `breaks_grid_framing` excludes the grid
delimiters `,`/`:`/`=`, the shell framing `\t\n\r`, and the #177 `\x1f` title separator — a superset of
`breaks_framing`, which guards the root; an unrepresentable cwd is dropped → plain `t` → restores to root, the
D2 stance). `GridLayout` gained a parallel `cwds: Vec<Option<String>>`; `serialize_grid`/`flatten` take a
`cwds: &HashMap<PaneId,String>`; `restore_grid`/`parse_leaf` build the cwds in lockstep; an old bare-`t` blob
restores `None` (back-compat). The masked shim (`build_shell_layout`/`persist_grid`) captures each terminal's
live `current_prompt().pwd` (the #201 source); on restore, `cwd_or_root(cwd, root)` (`Path::is_dir`) respawns
the base + each split pane in its saved cwd when it still exists, else the project root — the launch project's
first terminal reuses the pre-spawned boot PTY ONLY when its saved cwd resolves back to the launch root (a
`cd`'d subdir spawns fresh THERE — inspect fixed a silent no-op for that common case). cov/MSI 100 on the pure
codec (36 viable mutants killed by the T1–T5 round-trip); the capture/spawn is the `mutants::skip` shim. Panes
are user-openable through the shared `Workspace::open_or_focus(kind) -> OpenAction` (#128) — a "Files" palette
command (`CommandId(6)`), ⌘-click for code, ⌘⇧C for git — with `first_pane_of_kind` / `panes_of_kind` (the
sidebar lists only real terminal sessions, #129). The Details/Agents/Forge cockpit moved from a right rail to
a **top tab strip** (`top_tabs`, #126); the right dock defaults closed so the pane grid gets the full width
(#127). Panes are drag-resizable (`layout::resize_split` + a divider handle, #130) and focus moves by
direction (`focus_neighbor` reusing the tree `neighbor`, ⌘⌥-arrow) with an accent focus affordance on the
focused pane — a left+top edge in #131, widened to a **full 4-side border** in M12.1 #191 (the pure
`focus_border_rects(content, thickness) -> [Rect; 4]` returns the four edge rects, right/bottom inset so the
border stays inside the pane / in-window with #189's gutter; the shim draws four thin accent bars, not a filled
frame, so the pane interior stays clickable). All pure surfaces are cov/MSI 100; the mouse-drag / key-nav gestures are shim-masked
(synthetic input is unavailable in the headless self-test). **M13 #228:** the focus border is drawn LAST among the
pane-chrome — after the #130 divider loop (so a divider no longer occludes its shared-edge bar) but before the
palette/overlay modals (so those still occlude it). The render hoists the focused pane's content rect into an
`Option<Rect>` during the pane loop and drains it after the divider loop; the border bars are non-`.occlude()` so
the mouse still reaches the divider underneath (the drag is preserved).

## M7 — The Warp Top Bar & Sessions (#132–136, sprint #18 closed)
A real **top bar** replaces the floating overlays. The window now reserves a band at the top as well as the
bottom: `layout::content_band(window_h, top_bar_h, status_bar_h) -> (content_top, content_h)` (the vertical
analogue of `region_widths`) — the docks + pane grid start at `content_top`, below `TOP_BAR_H` (#132). This
fixed two reported bugs at once: the global search stopped floating over the terminal, and the pane title
bars (with their close ×) stopped being occluded by the search/tabs overlays that used to sit on top of them.
The bar holds, left-to-right: a 📁 file-explorer icon (→ `open_files_pane`, #133), a `+` (→ `new_terminal_pane`,
a fresh PTY terminal via `split_focused` + `spawn_session`; `⌘D`/`split-pane` now shares it, #135), a 🧠
agent-launch icon (→ the tested `new-agent` dispatch, #136), the centered search, and the Details/Agents/Forge
cockpit as a compact icon cluster (`right_dock::section_icon`; the active section marked with a background
pill since emoji glyphs ignore `text_color`, #134). New pure surfaces (`content_band`, `section_icon`) are
cov/MSI 100; the top-bar button *clicks* are shim-masked (synthetic mouse input is unavailable headless), but
each reuses an already-tested path.

## M8 — Warp Chrome & Fidelity (#137–144, sprint #19 closed)
Chrome fit-and-finish, driven by chad's post-M7 feedback + a verification sweep.
- **Real icons** (#137) — clean-room monochrome SVGs under `crates/marley_app/assets/icons/`, served by an
  `AssetSource` (`Assets`) + `Application::with_assets`; `gpui::svg().path(icon_path(Icon)).text_color(c)`
  tints them, so emoji are gone and the active-section *bg pill* (M7 #134) is retired — SVG respects
  `text_color`. Pure `icons::icon_path` + `right_dock::section_icon → Icon`; `Assets::load/list` are
  `mutants::skip` (asset I/O).
- **Unified titlebar** (#138) — `TitlebarOptions { appears_transparent: true, traffic_light_position, title:
  None }` drops the OS title bar so the top bar shares the traffic-light row (Warp-style). Pure
  `layout::topbar_icon_x(slot, inset, gap)` insets the left icon cluster past the lights.
- **Deterministic pane placement** (#139) — non-terminal panes (files/code/git) open at the RIGHTMOST leaf:
  pure `Workspace::rightmost_pane` (DFS-last) + `open_pane_rightmost` (shared `insert_split_at`), so order no
  longer depends on focus.
- **Synthetic-input harness** (#140) — corrects the M7 "clicks unavailable headless" claim: the app IS
  drivable (`AXIsProcessTrusted`; `scripts/selftest/drive.swift` posts `clickat:`/`cmd:`/`type:`/`drag:`/
  `cmdopt:` CGEvents). A `check` action + a fail-loud AX preflight were added. Interaction tickets now DRIVE.
- **Search keyboard nav** (#141) — pure `command_bar::move_selection` (clamped index) + a selected/highlighted
  row + ↑/↓/↵; every hit activates (`activate_search_hit`: file→viewer, session→focus).
- **cwd + branch label** (#142) — pure `titlebar.rs` (`abbreviate_path`, `branch_from_git_head`,
  `titlebar_label`) renders `~/…/Marley · main`, read from `$HOME` + `{project_root}/.git/HEAD`. **M12.1 #192**
  moved its render from the top-right titlebar to the RIGHT of the always-on footer status bar (a `flex_1` spacer
  after the `cockpit_status` segments) — chad wanted it "at the bottom"; the pure `titlebar_label` is unchanged.
- **Session-model spike** (#143) — Warp uses a sidebar session-list + tiled panes (Marley's model already);
  NO top tab strip. See `docs/planning/design-notes/session-tabs-vs-sidebar.md`; follow-ups #145–148.
- **Interaction verification** (#144) — drove the shipped gestures; FOUND + fixed a real bug: pane focus-nav
  was bound to ⌘⇧-arrow, not the documented ⌘⌥-arrow (a shift/alt swap in the positional `KeyBinding::chord`).
  Drag-resize re-verify deferred to #149.

## M9 — Workspace / Project / Tab model (in progress, sprint #20)
Re-architecting the shell from a single tiled grid into a hierarchy of full-screen tabs (chad 2026-07-07):
**Workspace** (a multi-project container) → **Project** (a repo root + its tabs) → **Tab** (full-screen:
a terminal or a cockpit section). Splitting is opt-in *within* a terminal tab; the Files panel expands from
the left, scoped to the active project; the cockpit becomes tabs (the right dock retires).
- **seq-1 (#150) — the pure model** (`tabs.rs`): `Workspace<S>`/`Project<S>`/`Tab<S>`/`TabContent<S>` +
  the add/close/switch algebra with never-empty / active-follows-close / last-item invariants (mirroring the
  M6 pane-grid rules R28–R30), generic over the session handle `S`, cov/MSI 100. A terminal tab owns a
  `PaneGrid<S>` — the former `Workspace<S>` grid, renamed — so all M6 tiling/split/resize survives inside a
  tab. No render yet (seq-2 wires it into the app view).
- **seq-2 (#151) — full-screen tab render + `+`-adds-a-tab.** `RootView`'s direct `workspace: PaneGrid` field
  becomes `shell: Workspace<TerminalSession>` (boot: one project named after the repo dir + one terminal tab
  wrapping the restored grid). Two accessors — `workspace()`/`workspace_mut()` → the active tab's grid — let
  the ~69 existing grid call-sites migrate mechanically (pre-seq-4 the active tab is always a terminal, so the
  accessor `expect`s it; seq-4 adds a cockpit-active guard). `+` (`new_terminal_pane`) now ADDS a terminal tab
  instead of splitting (retire default tiling); `⌘D` still splits the active tab's grid (opt-in). `⌘]` cycles
  tabs (pure `tabs::next_index` wrap; temporary until the seq-3 rail). Verified live: `+` opens a fresh tab
  full-screen (the prior multi-pane tab hidden), `⌘]` swaps the view back. Known forward-risk filed (#158):
  each tab's grid restarts `PaneId(0)`, so the global agents/remotes maps can alias across tabs — fix in seq-4.
- **seq-3 (#152) — the rail.** The left dock's flat session list is replaced by the Workspace → Project → Tab
  tree: pure `tabs::rail_rows(&Workspace<S>) -> Vec<RailRow>` (per project a Project header + its Tab rows, with
  `active` flags; the top-level Workspace-header row was later removed as redundant — M14 #239) at cov/MSI 100;
  the shim renders the rows indented by level, the
  active tab highlighted, and a Tab-row click routes `switch_project(p)` → `switch_tab(t)`. The old
  `sessions.rs` projection (`group_sessions`/`SessionGroup`) is fully retired + deleted. The agent-status icon
  and the branch subtitle it carried are deferred to the seq-4 cockpit / seq-8 live-titles work.
  **M12.2 (#219, Warp parity):** the active-row highlight was `bg(colors.surface)` — but the rail's
  `dock_panel` bg IS `surface` (#194), so the highlight was surface-on-surface = INVISIBLE (the active tab
  read only via brighter text). It now uses a pure `rail_highlight(colors, active) -> Hsla`
  (`accent.opacity(0.22)` active / `.opacity(0.10)` hover) — a distinct accent WASH — to draw a rounded
  highlight box on the active Tab/Pane row + a subtler hover wash on ALL clickable rows (Project/Tab/Pane).
  `rail_highlight` is the pure seam (a free fn near `dock_panel`, so it's mutation-tested — cov/MSI 100 — and
  guarded by a test asserting it stays perceptibly distinct from `surface`, pinning the invisibility bug);
  the row render is the shim. The active TEXT stays `foreground`; the selection BOX is reserved for the leaf
  Tab/Pane rows (an active Project stays bright-text — the grouping level above them). The rounding reuses
  `colors.corner_radius`.
  **M12.2 (#221, Warp parity — closes #224):** all 10 floating overlays (command palette, agent launcher,
  file + history finders, forge + fleet overlays, the diff/find/completion overlays, and the right-click
  context menu) now render `.rounded(colors.corner_radius).overflow_hidden()` → Warp-style rounded cards; the
  6 borderless listing overlays also gained `.border_1().border_color(colors.border)` so the rounded edge
  reads (a rounded corner needs an edge). `overflow_hidden` clips the selected-row `bg(accent)` to the rounded
  card shape. `corner_radius` now has 13 render consumers (the 3 rail rows + the 10 cards) — the token frames
  the floating cards, its intended use, so the #218-era orphan (#224) is RESOLVED. Borders/separators (uniform
  1px muted) + the #130 pane divider already read Warp-like → unchanged. Shim-only (capture-validated).
- **seq-4 (#153) — cockpit as full-screen tabs.** Details/Agents/Forge become `TabContent::Cockpit` tabs; the
  render moved out of the retired right dock into `cockpit_body`, drawn full-screen in the center when the
  active tab is a cockpit tab (an empty `rect_list` suppresses the grid). Pure helpers (cov/MSI 100):
  `Tab::cockpit_section`, `Project::terminal_grid_index` (so `workspace()` targets the active tab's grid, else
  the first terminal tab — a cockpit tab inspects the focused terminal, so terminal actions still work + never
  panic), `tab_grid`/`tab_grid_mut`, `open_or_switch_cockpit`. The top-right icons + `⌘⇧B` open a cockpit tab;
  the right dock is unopenable (regions.right ≡ 0). Forward-safety filed (#159): guard the last-terminal-tab
  close before any tab-close affordance lands.
- **seq-5 (#154) — Files as a left panel + open-file-as-a-tab.** The file tree becomes a LEFT panel
  (`files_panel`, toggled by 📁 / `open-files`, width `FILES_PANEL_W`, shrinking `center_bounds` from the left)
  and opening a file opens a full-screen `TabContent::CodeView(CodeViewState)` tab (`code_view_body`), retiring
  the tiled FileTree/CodeView panes (`open_files_pane`/`open_code_pane` deleted; boot restore drops those pane
  kinds). Pure (cov/MSI 100): `Tab::code`/`code_view`/`code_view_mut`, `grid`/`cockpit_section` → None for
  CodeView, `Project::open_or_switch_code` (one code tab reused: replace + switch, else append). The center
  render is now a 3-way branch — cockpit / code full-screen, or the terminal grid.
  - **M12.1 #190 — file paths resolve under the project root.** `FileTree::path_at` / `list_files_in` yield paths
    RELATIVE to the project root; `open_file_in_viewer` (and the boot code-tab restore) must join them onto
    `project_root` before `std::fs::read`, because a bundled app launched via LaunchServices runs with CWD `/` —
    a bare relative read silently failed (`Err(_)` swallowed) so a file click did nothing. The pure
    `marley_project::resolve_under_root(root, path)` (relative→`root.join`, absolute→unchanged; cov/MSI 100)
    routes every `open_file_in_viewer` caller, so opens are CWD-independent and every entry point (tree click,
    ⌘P-finder ⌘↵, search hit, diff header) yields the same absolute `CodeViewState.path` (so `open_or_switch_code`
    dedups instead of making duplicate tabs).
- **seq-6 (#155) — right-click → Split + nested pane rows.** Splitting is opt-in per terminal tab: a
  right-click (`MouseButton::Right`) on a terminal pane runs `split_focused_pane` (reuse M6 `split_focused` +
  persist), and `rail_rows` emits a nested `RailLevel::Pane` row per pane when a terminal tab's grid has >1
  pane (the focused pane marked active; `RailRow.pane` carries the 0-based index for click→focus). Single-pane
  tabs nest nothing. Pure at cov/MSI 100; the right-click + rail Pane arm masked. Added a `rightclickat` verb
  to the self-test harness.
- **seq-7 (#156) — multi-project.** A workspace holds multiple projects. `⌘O` → `open_project_picker` (the
  native directory picker via `prompt_for_paths` + `cx.spawn`) → `open_project_path` (a terminal spawned in the
  folder via `spawn_session_in` + `add_project`). The rail `Project` row is clickable → `switch_project`. Both
  paths call `sync_active_project`, which rebuilds `project_root` / `project_files` / `file_tree` from the
  active project's root — so the ~15 readers (git cwd, titlebar, branch, Files, ⌘P) follow automatically, no
  per-site change. The pure algebra (seq-1) is cov/MSI 100; the shim is coverage-excluded. Follow-up #160:
  subsequent tabs/splits/agents in a secondary project should also spawn in its root.
- **seq-8 (#157) — live rail titles + real branch (M9 FINALE).** A terminal tab row shows its running command
  via pure `titlebar::rail_tab_title(command, fallback)` (the first whitespace token, else the static title,
  cov/MSI 100) fed by the shim `live_tab_title` (the tab's focused terminal's latest block command; code/cockpit
  tabs keep their file/section title). The rail project row shows `name · branch` (reusing pure
  `branch_from_git_head` on `root/.git/HEAD`; name-only for a detached HEAD / non-repo). **M9 complete** — the
  shell is Workspace → Project → Tab, with full-screen tabs, a Workspace→Project→Tab→Pane rail, cockpit + code
  as tabs, Files as a left panel, opt-in right-click split, multi-project, and live titles.
- **M13 #233 — workspace cycling + terminology foundation** (the workspace-centric re-arch spine). The M13
  vocabulary recast: chad's "workspace" = a Project (a repo), promoted to the top-level focusable unit; the
  singleton container becomes the "Session". #233 PINS this terminology — the code rename is DEFERRED
  (a `tabs::Project → Workspace` rename collides with the existing `marley_project::Project` + ~75 refs, so
  it's a dedicated later pass) — and adds the first switch algebra: pure `Workspace::cycle_project` (next/prev
  focused workspace, wrapping, cov/MSI 100, reusing `next_index`/`prev_index`) bound to ⌘⇧] / ⌘⇧[ (a level up
  from the ⌘] / ⌘[ tab chords). A workspace switch calls `sync_active_project`, so Files/⌘P/git/titlebar follow.
  The never-empties + always-focused invariants are UNCHANGED (the launcher's empty state + guard-relaxing are
  #234). See `docs/planning/pipeline/completed/workspace-foundation.*`.
- **M13 #234 — the launcher / landing page** (supersedes #202). Relaxing the M10 never-empties guards
  (`close_project` drops its `LastProject` refusal; `adjust_active` guards the `new_len==0` underflow)
  makes a zero-workspace state reachable, and `RootView::render` branches at its VERY TOP on pure
  `launcher::should_show_launcher(project_count()==0)` to `render_launcher` (a centered "Open a workspace"
  card: recent workspaces + "Open Folder…") instead of the shell — so the shell (and its ~90 `workspace()`
  /`active_project()` sites) only renders at ≥1. The zero-state is panic-safe: the render branches at the
  top (before the find-snapshot prologue), and the two OFF-render-path accessors are guarded — the 16ms
  PTY pump early-returns at 0, and `persist_grid` persists only the 0-safe shell layout. Recents = a
  persisted `Recents: Vec<String>` setting (pure `launcher::push_recent` MRU, front-dedup'd + capped),
  recorded on every open (incl the boot workspace). See `docs/planning/pipeline/completed/workspace-launcher.*`.
- **M13 #235 — scope-driven top bar** (chad #6+#7). Shipped a focused-workspace INDICATOR (pure
  `titlebar::focused_workspace_indicator` — `name · branch`, truncated) at slot 3 + moved the cockpit
  tabs (Details/Agents/Forge) from `.right(16)` to a LEFT x after it, grouping the workspace-scoped
  actions. **REMOVED by M16 #260** (chad: "we opt in for the left" — the left Workspace rail is the
  single home): the indicator + its pure fn are gone; the cockpit-tab relocation SURVIVES, now anchored
  AT slot-3 x (which also cleared a live occlusion — at the 1024px default width the tabs previously sat
  under the centered search bar). See `docs/planning/pipeline/completed/scope-driven-topbar.*` +
  `260-topbar-workspace-removal.*`.
- **M13 #236 — collapsible rail + focused-workspace highlight** (chad #8 + the rail-highlight ask). The
  pure `tabs::rail_rows` gained a `collapsed: &HashSet<usize>` param + a `RailRow.collapsed` flag — a
  collapsed project's Tab/Pane rows are OMITTED (the disclosure tree). The shim
  (`RootView.collapsed_projects`, remapped on a project close via the pure
  `tabs::remap_indices_after_remove` — index-keyed state must be remapped on a Vec removal) renders a ▸/▾
  chevron on the project row (stop_propagation toggle → `toggle_project_collapse`) + a highlight
  background on the ACTIVE project row (the focused workspace). See
  `docs/planning/pipeline/completed/collapsible-rail.*`.
- **M13 #237 — the editor surface** (chad #9, anti-clutter; the FINAL ticket of the #228–#237 train).
  Recasts the per-file code tab (#154/#164) into a MULTI-FILE surface so N open files = ONE rail row, not
  N. New pure `editor_surface::EditorSurface { files: Vec<CodeViewState>, active }` (cov/MSI 100) — `open`
  (dedupe by path → switch+refresh, else append+activate), `close(idx)` (remove + clamp active; `false`
  when it would drain the last file → the shim drops the tab), `activate`, `active_file[_mut]`, `files`,
  `active_index`. `TabContent::CodeView(CodeViewState)` → `CodeView(EditorSurface)`; `code_view()`/
  `code_view_mut()` now return `surface.active_file[_mut]()` (render + persistence unchanged); new
  `editor()`/`editor_mut()`; `Project::open_or_switch_code` routes into the project's ONE editor tab
  (`position(|t| t.editor().is_some())` → `surface.open`, else create). The shim draws a horizontal file-
  tab STRIP (name · × `close_editor_file(i)` · click `activate(i)`) above `code_view_body`; closing the
  last file calls `close_tab_at`. Read-only v1 (reuses `CodeViewState`); editing (`marley_editor::Buffer`),
  terminal-split-to-a-file, and multi-file persistence (only the active file persists via `code_view()`)
  are deferred. Supersedes #164's "a code tab per file". See
  `docs/planning/pipeline/completed/editor-surface-tabs.*`.

## M14 — The Editable Editor + cockpit polish (in progress)
- **M14 #239 — remove the session-header rail row** (chad live #1). `tabs::rail_rows` no longer emits the
  top-of-tree `RailLevel::Workspace` row (the variant + its render arm removed); the rail now begins with the
  Project row — redundant with the "Workspace" dock panel title (`dock_title`, layout.rs — unaffected), the
  status bar, and the tabs. Pure subtractive removal, cov/MSI 100 (the retained rail_rows mutants stay killed by
  the unchanged tests). See `docs/planning/pipeline/completed/rail-remove-session-header.*`.
- **M26 #385 — the rail groups tabs into three fixed-order type sections (Editor / Terminal / Browser).**
  `tabs::rail_rows` now emits, per non-collapsed project, each section header then the tabs that file under
  it: a pure `RailSection { Editor, Terminal, Browser }` (+ `ALL` = the fixed order) + a total
  `TabContent::rail_section()` (CodeView→Editor / Terminal→Terminal / Cockpit→Browser — the cockpit/Forge
  tabs are the Browser section's transitional home until the Phase-E embedded webview) + a new
  `RailLevel::Section` render arm (a muted `Role::Caption` at `pl(18)`, between the project (8) and tab (28)
  indents). DISPLAY-ONLY: each Tab row keeps its original `Project.tabs()` index (the switch target), so
  storage order, the active index, and the shell/grid codec (`grid_layout.rs`) are byte-identical — the
  sections exist only in the row stream (the map's flagged storage-reorder hazard, avoided by construction).
  Empty sections still render a header (the fixed skeleton); a split terminal's panes still nest under their
  tab inside the Terminal section. Pure seam cov/MSI 100; the render arm is the masked app shim,
  driven-validated. Section interaction (active-section highlight + click-collapse-persist) is #386;
  per-section ＋ new-actions #387; the real Browser content (Forge-in-webview) is Phase E / #389.
- **M26 #386 — the rail sections become interactive (highlight · collapse-persist · auto-reveal).**
  The section header holding the active tab (of the active project) carries the #219 accent-wash
  (`rail_rows` computes `active_tab_section` = the active tab's `rail_section`); a ▸/▾ chevron on each
  header toggles `AppView.collapsed_sections: HashSet<(usize, RailSection)>` (`RailSection` gained
  `Hash`), whose members' tab rows are omitted — EXCEPT the active section, which is force-expanded at
  render (`contains(&(i, section)) && !is_active_section`), so its active tab is never hidden (the #305
  auto-reveal invariant, by construction — no activation-site hooking). The set persists by project
  root + section label (`rail.collapsed_sections`) via pure `collapsed_section_keys` /
  `collapsed_sections_from_keys` / `RailSection::from_label` (mirroring #245's `collapsed_roots` /
  `collapsed_indices`) and index-remaps on project close via `remap_section_indices_after_remove` (the
  #236 lesson). The footer `focus_label` is a documented no-op (a cockpit tab reads "cockpit" — the
  footer names the surface's nature, the section names its category). Pure seams cov/MSI 100; the
  chevron+click+wash render arm is masked, driven-validated. Per-section ＋ new-actions are #387.
- **M26 #387 — the sections gain a hover-revealed ＋ that creates under that section.** Each header
  carries a ＋ (hidden at rest, `group_hover` opacity-0→1 — the #217 idiom) that dispatches an EXISTING
  verb only, via a pure total `section_action(RailSection) -> SectionAction` table: Editor→OpenFile (the
  ⌘P finder `start_file_finder`, extracted so the `open-file-finder` command and the ＋ share it),
  Terminal→NewTerminal (`new_terminal_pane`), Browser→OpenCockpit(Forge) (`open_or_switch_cockpit`; Agents since #411 retired Forge). The
  masked `dispatch_section_action(p, section)` shim mirrors the #174 cross-project idiom (activate the
  row's project first), and the ＋'s own `on_mouse_down` calls `cx.stop_propagation()` so it never also
  fires #386's collapse toggle on the parent header — the hit-target separation, sound because gpui
  dispatches mouse listeners child-first (reverse-bubble) and breaks on stop (window.rs:3705/3708). The
  never-empties guard is refactored into a pure, exhaustively-tested `close_tab_refusal(tab_count, idx,
  target_is_terminal, terminal_count)` truth table that `close_tab` delegates to (behaviour-identical);
  it settled the D4 question — `LastTab` (sole tab, checked first) and `LastTerminal` (>1 tab, exactly one
  terminal) are DISTINCT and both reachable, so the "subsumption" hypothesis is FALSE. Pure seams cov/MSI
  100; the ＋ render + dispatch is the masked shim (render confirmed by capture, input env-blocked →
  units + the byte-identical #174/#217 idioms).
- **M27 #390 — the rail's 4th section, Panes, dynamically collects split views (the un-nest).** The rail
  is now `Editor · Terminal · Panes · Browser` (`RailSection::ALL[4]`). Panes is a **derived** section —
  `rail_section()` never yields it (no `TabContent` files there); instead `rail_rows` special-cases it: a
  per-project counter emits one **`RailLevel::Arrangement`** row `"PANE n"` per tab whose grid has ≥2
  cells, with that tab's pane cells nested beneath. This **un-nests** the #155 nested-`Pane` block from
  under the Terminal `Tab` row (it moved to the Panes block; a split tab now shows only its Tab row under
  Terminal). The origin tab dual-lists (model A — the Panes row is a view of it). `section_action` gains a
  `Panes => SplitFocused` arm → `split_focused_pane(Horizontal)` (reuse-only). The Arrangement render arm
  mirrors the Tab arm's #174 cross-project click; the `RailLevel::Pane` cell arm is unchanged (keyed on
  (project,tab,pane), agnostic to parent). Panes is never the *active section* (no tab files there) and
  its collapse key `(i, Panes)` round-trips through the #386 helpers (back-compat: old blobs lack it).
  Display-first on (tab,pane) coordinates — the #388 model's DISPLAY half, ahead of the #394 `ContentId`
  registry. Pure seams cov/MSI 100 (57 tabs tests); the render was **confirmed live** via a pre-seeded
  `workspace.shell = …T=H:t,t` restore (a split-terminal session the app reconstructs on boot — the way to
  driven-verify a render when synthetic input is env-blocked).
- **M27 #393 — the section ＋ becomes a MENU (the extension point), folded into the ONE #166 machinery.**
  The #387 single-verb ＋ now opens a small anchored menu of the section's create-verbs. The DECISIVE design:
  the section menu is a **peer `MenuKind`** — `context_menu.rs` gains `MenuKind::Section { p, section }` +
  `MenuAction::Section(SectionAction)` + a pure `section_items(RailSection) -> &'static [(MenuAction, &str)]`
  table (the `items_for` const-slice idiom), so the existing `menu.items()` render loop, ↑/↓/↵ keyboard nav,
  esc/click-away, `menu_origin` clamp, and one-modal ALL apply for free (zero new render code — the D3
  "one menu system"). The `p` rides the KIND (the #175 target-rides-the-kind pattern) so a mid-menu project
  switch can't retarget. Rows (item 0 = the #387 default, muscle memory): Terminal[New Terminal ·
  New Terminal at Root], Editor[Open File… · Open in Split #246], Browser[Forge · Agents · Details — Agents · Details · Browser since #411 dropped the Forge row],
  Panes[Split Right · Split Down] — all reuse-only; the future items (SSH #87, workflows #204, URL/Phase-E,
  New File) are documented-not-built. The masked `open_section_menu` / per-item `dispatch_section_verb`
  shims mirror the #174 activate-first + #387-F1 persist rule uniformly. `section_action`/
  `dispatch_section_action` (the #387/#390 single-verb path) are REMOVED. Pure `section_items` cov/MSI 100;
  the render/dispatch is masked (mechanism-verified — byte-identical to the proven Split/Block/FileRef loop;
  a pixel capture awaits `render_to_image`, #264).
- **The one-modal discipline for MOUSE-opened modals — `close_transient_overlays()` (M27 #393).** A modal
  opened by a mouse click (the #181 agent launcher; the #393 section ＋ menu) bypasses the key router's
  overlay guards, so it must close every interactive overlay whose key-arm precedes it in the router — or it
  opens keyboard-dead (keys go to the still-open picker; a text draft even captures them). The launcher
  accreted this fix across **8** separate inspect findings (#229 naming_workflow, #312 def_picker, #317
  references, #323 code_action, #324 signature, #325 symbols, #326 search, #327 problems); #393 factored the
  13 closers into ONE `close_transient_overlays` helper both call — the single source of truth (add a new
  overlay's field there once). Realism-verified: the pickers are single centered cards (no full-window scrim),
  so the left-rail ＋ is not occluded and the co-open is reachable. **Membership (#415):** every transient
  modal/picker with a key-arm joins the roster — including the launcher ITSELF (extracted from its closers
  at #393, it never closed itself, so other mouse-opened modals stacked over a lingering launcher — the
  #318 smoke observation); persistent docks (fleet/files) are deliberately exempt — fleet is toggle-scoped.
  #415 also swept the two menus that PREDATED the helper (the #175 file-ref menu, the terminal right-click)
  off their fossilized hand-copied trio and onto the roster
  (F-claude-415-roster-extraction-left-preexisting-hand-closers-unswept-001).
- **M27 #394 — the ContentId registry (`content_registry.rs`, the #388 spine).** The id-keyed
  content-ownership generalization of the existing `agents`/`remotes`/`panes: HashMap<PaneId, PaneState>`
  side-tables: a `ContentId` newtype + a pure, gpui-free, **generic `ContentRegistry<C>`** owning content
  once per id with an explicit **view-count** and **drop-on-last-close** (`release_view` returns the owned
  content on the last view, for the caller's off-thread reap). One instance, many views (a tab row + a
  split cell hold the same id) — what gpui's `Entity<T>` gives from a cloned handle, hand-rolled so the
  model layer stays gpui-free (`AD-claude-pane-content-id-registry-001`; the refcount is the binding catch,
  `PR-…-refcount-…-001`). **Wired but UNUSED this slice** — the #371 `orchestration_for` idiom (`mod` +
  crate-root `pub use` → lib-API-reachable, not dead-code under `-D warnings`). The **D5 split**:
  #394 = the pure lifecycle (proven at cov/MSI 100 with a test double, no live PTY); **#396 wires terminals
  onto it** (the `Content` enum + the ownership/reaper/persistence migration — its own reviewed ticket).
- **M27 #391 — the app can render + persist a NO-TERMINAL project (the `try_workspace` contract).** The
  `workspace()`/`workspace_mut()` accessors `.expect()` a terminal grid (`terminal_grid_index()` → the
  active tab's grid, else the first terminal) — the code reason the M10 never-empties guards forbid closing
  the last terminal (#202's ~90 dependent sites). #391 adds the total twins **`try_workspace()`/
  `try_workspace_mut() -> Option<&(mut) PaneGrid>`** (`None` when no terminal) and moves the ALWAYS-RUN
  paths (render focused, resync, PTY-resize, `cockpit_body` Details, `persist_grid`'s legacy grid block) to
  them; the panicking accessors stay for the terminal-GATED sites (a terminal is guaranteed by their
  trigger). **Behaviour-neutral** — the guards remain, so a no-terminal project is test-only reachable and
  the full suite is byte-identical. This is the **always-run slice** of dissolving the never-empties
  invariant (a D4 split: the ~30 USER-TRIGGERED handlers + the empty-workspace UI move to **#392**, which
  removes the guards). The full audit ledger (always-run vs user-triggered) lives in the #391 pipeline
  notes.
- **M27 #392 — the app is total over a ZERO-TAB project too (the `try_active_tab` twin; the totality
  substrate completed).** #391 totalled the always-run paths over a *no-terminal* project; #392 totals the
  rest over BOTH empty states. `active_tab()`/`active_tab_mut()` are bare indexes
  (`&self.tabs[self.active]`) that panic on a *zero-tab* project — reachable once #395 dissolves the
  `LastTab` guard. #392 adds the total twins **`try_active_tab()`/`try_active_tab_mut() -> Option<&(mut)
  Tab<S>>`** (`self.tabs.get(self.active)`), hardens `terminal_grid_index` to `.get()`, and routes the ~40
  zero-tab / no-terminal-reachable sites through the `try_active_tab`/`try_workspace` twins: the render
  center dispatch (a leading `tab_count()==0 → blank center` guard, so no bare `active_tab()` runs before
  it — #395 fills the blank with hints), the status bar, both `key_context()` reads (an empty project → no
  key context → the chord no-ops), the terminal raw-key router, the pump tick, and every keystroke-reachable
  handler. The panicking accessors stay for the tab/terminal-GATED sites. **Behaviour-neutral** — the guards
  remain, so both empty states are test-only reachable and the full suite is byte-identical; two adversarial
  critics (byte-identity + an exhaustive accessor-reachability audit) confirmed it, the audit closing one
  latent accessor (`activate_search_hit`) that was safe only by a cross-function invariant. **This is the
  crash-safe substrate; #395 flips both never-empties guards + ships the empty-workspace UI on top of it —
  where the M10 never-empties era (a project always holds ≥1 terminal tab) formally ends.** The pure
  `try_active_tab*`/codec seams carry coverage + mutation 100%; the app.rs render/handler shims stay in the
  ACCEPTED-UNTESTABLE exclude (byte-identity-proven).
- **M27 #395 — the M10 never-empties invariant is DISSOLVED (a workspace may be fully empty).** The capstone
  of "terminate the last terminal", flipping the guards onto #392's crash-safe substrate. `close_tab_refusal`
  shrinks to **IndexOutOfRange-only**; `TabError` loses `LastTab`/`LastTerminal` (single-variant now) and its
  sole consumer (the "can't close the last terminal" status flash) is gone; the #387-pinned guard tests are
  deliberately rewritten (D3). An emptied project renders the **empty-center hint panel** (fills #392's
  `tab_count()==0` blank branch: *Nothing open in this workspace* + ⌘T/⌘P/＋) with the rail's four empty
  section headers — **NOT the launcher** (#247 stays zero-WORKSPACE; an open-but-empty workspace stays open,
  D2). Persistence: a persisted empty project **restores empty** — the boot restore loop no longer force-seeds
  a terminal (`Project::empty`; the `workspace()` ≥1-terminal invariant the force-seed upheld dissolved in
  #391/#392, so the boot honors the persisted empty state; a NEW workspace still default-seeds — D4). The PTY
  reaper on close is unchanged. Two critics confirmed the empty state is TOTAL (no panic, no cascade to
  `close_project`); the empty-workspace UX was driven-captured (pixel-verified). **This is where the M10
  #161/#159 "a project always holds ≥1 terminal tab" invariant formally ends** — the #202-parked "welcome
  state" fork resolved as a workspace-launcher (the launcher) + this empty-center hint state.
- **M26 #389 (SPIKE) — the embedded browser** ("Forge opens in the Browser section", opens Phase E).
  Designed in [embedded-browser-model.md](embedded-browser-model.md): the substrate is **wry-as-child
  WKWebView** (feasible via gpui's exposed `raw-window-handle`; the native-child z-order-vs-gpui-overlays
  collision is the load-bearing risk, CEF off-screen-render via gpui's `paint_surface` the named
  revisit), the Forge URL derived from #381's `.mcp.json` seam *(derivation ripped at #410 — the
  scrap-forge pivot; the generic pane survives with no origin source)* (bearer never in the URL, pinned-origin
  nav, native cockpit coexists), a `TabContent::Browser` + a framing-safe `b`-marker codec (URL
  re-derived on restore), the CDP fork answered with two substrates (WKWebView pane + a separate
  headless-Chromium agent lane), and the Phase-E follow-up train (proof-of-embed first).
- **M26 #388 (SPIKE) — the four-section pane-composition model** (chad's 2026-07-22 refinement). The
  rail becomes FOUR per-workspace sections `Editor · Terminals · Panes · Browser`: the Panes section
  dynamically lists every SPLIT VIEW (the split's origin stays under Editor/Terminals; a Pane is a VIEW
  referencing it), superseding #385/#386's pane-under-tab nesting. Designed in
  [pane-composition-model.md](pane-composition-model.md): a Marley-owned `ContentId` registry (NOT gpui
  `Entity` — the model stays gpui-free), one-instance-many-views (kills the #259 two-Buffers dup), the
  migration ripple, nameable arrangements, and the **M27 refactor train** that implements it (the
  un-nesting into Panes is a train slice, not a #385/#386 amendment). A global cross-workspace Panes
  section is designed, build-gated on the multi-workspace model.
- **M14 #240 — editor rail row → a stable "Editor" label** (chad live #2). The editor surface tab is created with
  a `pub(crate) EDITOR_TAB_TITLE = "Editor"` instead of the first file's name — used by BOTH creation paths
  (`Project::open_or_switch_code` and the session-restore `TabLayout::Code` arm, so a restored tab keeps "Editor").
  The rail label reaches the row via `live_tab_title`'s fallback: an editor tab has no terminal → command/cwd
  None → `display_title(None,None,None,"Editor")` returns the fallback "Editor". The file-tab strip
  (`surface.files()`) stays the source of truth for WHICH file; a #177 rename still overrides. Pure seam cov/MSI
  100. See `docs/planning/pipeline/completed/editor-rail-label.*`.
- **M14 #241 — disambiguate identical tab labels** (chad live, alongside #2). Idle terminals fall back to their
  cwd basename (`live_tab_title`), so N terminals in one dir all read "Marley". The pure
  `titlebar::disambiguate_labels(&[String])` appends `" {n}"` to the 2nd-and-later occurrence of a duplicate
  (first bare, per-LABEL counter); the app.rs render builds a per-project `(project,tab)→label` map over the
  FULL `rail_row_list` (scroll-safe — computed BEFORE the `skip(rail_skip)` loop, so a scrolled/filtered rail
  numbers correctly) which the `RailLevel::Tab` arm + the #112 "Search tabs" filter look up. Uniques
  (Details/Agents/Forge, the #240 Editor) are untouched. Pure seam cov/MSI 100. Known LOW (doc-noted): a sibling
  literally named "{base} {n}" can re-collide (cosmetic; click routing is by index). See
  `docs/planning/pipeline/completed/disambiguate-tab-labels.*`.
- **M14 #243 — persist all open editor files** (a #237 follow-on). `TabLayout::Code` now carries the full open
  path list + active index (wire `V=<active>\x1f<path…>`; back-compat the old single `V=<path>`; a framing/`\x1f`
  path is dropped + active re-clamped). Pure `serialize_code_paths` / `parse_code_paths` + a shared
  `reclamp_active` — used by BOTH serialize AND the session-restore, so a dropped/unreadable file before the
  active one shifts active onto a valid neighbour — + `EditorSurface::from_files`. **`open_file_in_viewer` now
  calls `persist_grid`**: previously it mutated the editor tab in-memory WITHOUT persisting, so the tab was never
  saved (the codec was correct but the save was never triggered — caught only by the driven quit→relaunch, not
  the unit tests). Pure codec cov/MSI 100. See `docs/planning/pipeline/completed/persist-editor-files.*`.
- **M32 #427 — Multibuffer tabs are TRANSIENT.** The persist writer's FIRST dropping arm:
  `TabContent::Multibuffer(_) => return None` — no `TabLayout` variant, no reader arm; a materialized
  result set re-materializes from a fresh ⌘⇧F only. Because the persisted list now differs from the live
  one, `active_tab` is re-derived by counting SURVIVORS before the live active index (clamped to the
  persisted list) — restore never focuses a shifted neighbour (PR-claude-427-c; the drive
  `multibuffer_close_releases_and_restore_drops_headless` pins both halves).
- **M14 #245 — persist the rail collapse-state** (a #236 follow-on). The per-project collapse set (#236
  `collapsed_projects`) persists via a new `CollapsedProjects` `Vec<String>` setting (`rail.collapsed`) mirroring
  Recents, keyed by the STABLE project ROOT (not the shifting index). Pure `tabs::collapsed_roots` (indices→
  roots, save) / `collapsed_indices` (roots→indices, restore); `persist_collapsed_state` is called on
  `toggle_project_collapse` + the project-close remap (the persist is TRIGGERED — the #243 lesson, verified via
  the driven `[rail] collapsed=[root]` write); boot restores `collapsed_projects` via `collapsed_indices` before
  the RootView struct is built. Known LOW (doc-noted): two projects sharing a root both restore collapsed. Pure
  seam cov/MSI 100. See `docs/planning/pipeline/completed/persist-rail-collapse.*`.
- **M14 #246 — a terminal pane can split-right to a read-only file view** (chad's ask; a #237 deferred item). The
  pane model was ALREADY content-agnostic (`PaneContent::CodeView` + the session-less `open_pane` [no PTY] +
  `code_view_body` + PaneId-keyed resize/focus/close) — so v1 is SMALL: revive the CodeView render arm (the
  session-less `else` in the pane loop) + `split_file_pane` (the shared `load_code_view_state` guard ladder →
  `open_pane(Horizontal, After, CodeView)`, NO PTY) + a "Split Right → File" palette command (`CommandId(10)`)
  that opens the ⌘P finder in a `finder_split` mode (its Enter splits instead of opening a tab) + a pure
  `pane_display_name` (a CodeView pane titles by filename). The `open_file_in_viewer` guard ladder was extracted
  into the shared `load_code_view_state` (open_file_in_viewer behavior byte-preserved; the split path reuses the
  same #190/#196/#106 guards + gains the same reject status-flash). **v1 NON-PERSISTED** — a split file pane
  drops on restart: the restore's ADDITIVE rebuild (a flat kinds list; the `FileTree|CodeView => {}` drop arm)
  drops a CodeView leaf as a no-op → proven no-crash, no malformed grid; a follow-up adds `c=<path>` to the grid
  codec + a restore arm. Pure seam cov/MSI 100 on `pane_display_name`. Inspect caught a recurring `mutants::skip`
  DETACH (the refactor relocated the skip off `open_file_in_viewer` → `PR-claude-recheck-mutants-skip-after-
  refactoring-shims-001`). Driven env-blocked (locked screen) → the no-crash boot + shipped-reuse mechanism. See
  `docs/planning/pipeline/completed/terminal-split-to-file.*`.
- **M14 #244 — the top-bar workspace indicator is a click-to-switch popover** (a #235 follow-on). Shipped
  the indicator-as-toggle: a pure `titlebar::workspace_switcher_rows -> Vec<SwitcherRow>` display model +
  a `workspace_switcher_open` bool + a popover mirroring the #166 backdrop, switching via the #233
  rail-click idiom with the #229 close-all on open. **REMOVED by M16 #260** together with the #235
  indicator (the popover, its pure fn/type, the bool, and their tests — the #233/#236 rail is the single
  switch affordance; nothing else referenced the switcher, so the removal was purely subtractive). See
  `docs/planning/pipeline/completed/topbar-workspace-switcher.*` + `260-topbar-workspace-removal.*`.
- **M14 #247 — boot to the launcher when empty + a "New empty workspace" action** (a #234 follow-on). Boot
  builds a zero-project workspace (a new pure `Workspace::empty` — the #234/#247 launcher state) when
  `restored.is_none()` AND the legacy #205 grid is empty (`applied.grid.trim().is_empty()`); otherwise the
  legacy-grid seed arm is UNCHANGED (a pre-#163 user still restores — NO data loss). `persist_grid`'s empty
  branch now also CLEARS the legacy grid key, so closing the last workspace → the launcher on next boot (not a
  resurrected stale grid). The boot recents-fold is guarded — the ONLY new empty-boot accessor: the #234
  relax-always-≥1-invariant audit (`PR-claude-relax-nonempty-invariant-audit-all-accessors-001`), confirmed by
  two independent critics — the render early-return (`should_show_launcher` → `render_launcher`) never mounts the
  shell element tree at count 0, so its ~141 `active_project()`/`workspace()` sites are unreachable, and the 16ms
  pump is fenced. A "New empty workspace" launcher button → `new_empty_workspace` roots at `$HOME` (a real dir —
  the PTY needs a live cwd, the #205 lesson) via the pure `launcher::first_existing_dir` ($HOME→cwd→temp) + the
  shipped `open_project_path`. cov/MSI 100 on `Workspace::empty` + `first_existing_dir`. Driven env-blocked
  (locked screen) → carried by the no-crash empty-boot (process stayed alive) + the byte-identical settings
  restore + the audit. Single-Workspace launcher behavior — not the multi-workspace re-arch. See
  `docs/planning/pipeline/completed/boot-to-launcher.*`.

## M15 — The Editable Editor (in progress)
- **M15 #249 — the editor doc model** (the FOUNDATION of the editable editor; chad's `/work 249-258` train).
  The #237 editor surface stored `files: Vec<CodeViewState>` (a read-only, lossy render doc). #249 backs each
  open file with a real editable model: `editor_surface::OpenFile { view: CodeViewState, buffer:
  marley_editor::Buffer, caret: CharOffset, saved_version: BufferVersion }`, so `EditorSurface` now holds
  `Vec<OpenFile>`. The `buffer` is seeded from the file's **RAW text** (not `view.lines`, which are tab-expanded
  + truncated — lossy); `saved_version = buffer.version()` at load (clean). `active_file()`/`active_file_mut()`
  keep the same signatures (return `&[mut] files[active].view`), so `code_view_body` render, the #243 grid-path
  persistence, and the file-tab strip are UNCHANGED; new accessors `active_buffer_mut` / `active_caret[_mut]` /
  `active_saved_version` / `active_mark_saved` / `active_is_dirty` feed #250-252. Dirty ⇔ `buffer.version() !=
  saved_version`. `open` on an already-open path now switches (preserving the buffer — no disk-clobber of unsaved
  edits) instead of the old refresh. `files()` returns an `impl ExactSizeIterator<Item = &CodeViewState>` (the
  internal `Vec<OpenFile>` exposes only views). `EditorSurface` DROPPED its `Debug/Clone/PartialEq/Eq` derives
  (`Buffer` derives none, and `TabContent<S>`/`Tab<S>` carry no derives, so nothing needs them — Fork B); the
  ctors thread the raw text (`load_code_view_state` returns `(CodeViewState, String)`; `from_files` takes
  `Vec<(CodeViewState, String)>` pairs so `OpenFile` stays private). A `debug_assert!(!files.is_empty())` locks
  the ≥1-file invariant the `active_*` accessors depend on. cov/MSI 100 on the pure surface; the model is
  invisible until #250 renders from the buffer. Reference §20: Warp's command-input buffer+caret editing model
  (behavior; the observed capture lands at #250). See `docs/planning/pipeline/completed/editor-doc-model.*`.
- **M15 #250 — the faithful renderer** (THE make-or-break of the editor). The editor tab now draws each visible
  line FROM the `Buffer` (`active_buffer().line_text(row)`), not the lossy tab-expanded/200-col-truncated
  `CodeViewState.lines`, **un-truncated** — so on-screen columns equal char offsets. New pure
  `code_view::line_layout(line, tab_width) -> LineLayout { display, col_starts }` builds the tab-stop display
  string AND the char→column map in ONE pass (`col_of_offset(char_idx) -> display col`; each non-tab char = 1
  col in v1, tab → next stop); `expand_tabs` is reimplemented as `line_layout(..).display` (the single source
  of truth, so the render + the caret map can't diverge). New `Buffer::line_text(row)` (ropey line minus the
  trailing `\n`, out-of-range → "") + `Buffer::line_col(c) -> (row, char-in-line)` (a CHAR index — NOT
  `Point.column`, which is bytes) + `EditorSurface::active_buffer()`. `code_view_body` gained an
  `Option<EditorDraw>` arm: `Some` = the editor path (buffer lines, un-truncated, `.text_size`+`.line_height`
  pinned to the measured cell, + a caret bar at `col × em_advance` — the terminal's measured cell — computed via
  `line_col` + `col_of_offset`); `None` = the #246 read-only split pane's `cv.lines` path (BYTE-IDENTICAL,
  unchanged). The wheel scroll clamps against `buffer.len_lines()` (the render's line-count source), not the
  frozen `cv.lines`. The editor uses ropey's line model throughout (render + caret self-consistent). cov/MSI 100
  on the pure map + buffer helpers; the caret + faithful render are driven-proven (a caret bar at line 1 col 0).
  The caret is static at offset 0 (a bar; #251 adds movement/typing + a block/blink); `offset_of_col` (the
  column→offset inverse) lands with #254 (mouse click). See `docs/planning/pipeline/completed/faithful-renderer.*`.
- **M15 #251 — editor input intercept (the MARQUEE — the editor is now editable).** The shell `on_key_down`
  router (app.rs) gained an editor branch placed AFTER the cockpit-chord dispatch (`keymap.action_for` →
  dispatch+return) and BEFORE the interactive/raw-PTY route: when `active_tab().editor().is_some() && !platform
  && !control` (a plain key, an editor tab active), it reuses the prompt's `key_from_keystroke` parser + calls
  the new PURE `input::apply_editor_key(&mut Buffer, &mut CharOffset, Key)` on the active file's buffer+caret,
  then `cx.notify()` + returns (no leak to a hidden terminal). `apply_editor_key`: `Key::Enter =>
  apply_key(.., Char('\n'))` (multi-line — a newline, NOT the prompt's Submit); `Char/Backspace/Left/Right =>
  apply_key(.., key)` (delegate — maximal reuse); `_ => Ignored` (Home/End/word-wise/vertical/DeleteForward are
  line-aware or out-of-scope → NO-OP here, deferred to #257, so `apply_key`'s single-line motion never runs on a
  multi-line buffer). `apply_key` (the terminal prompt) is UNCHANGED (Enter⇒Submit intact). New
  `EditorSurface::active_buffer_and_caret_mut() -> (&mut Buffer, &mut CharOffset)` — the input layer needs both,
  and two `&mut self` accessors can't be called in one expression (E0499), so a combined disjoint-field-borrow
  accessor. The #250 render redraws the buffer edit + the caret for free. Driven-proven LIVE: typed "HELLO" →
  it inserted + the caret advanced; Enter → the line split; ⌘P → the finder still opened (chords work). Up/Down/
  Home/End/word-wise = #257; ctrl-editing + copy/paste = #256. cov/MSI 100 on `apply_editor_key` +
  `active_buffer_and_caret_mut`. See `docs/planning/pipeline/completed/editor-input-intercept.*`.
- **M15 #252 — editor save (⌘S) + the dirty ●.** `EditorSurface::file_dirty_flags()` (pure, per-file
  `OpenFile::is_dirty()` in tab order) drives a ● in the #237 tab strip — zipped with `files()`, the "●"
  (U+25CF, a mono primitive, accent-tinted) prepended when dirty (per-file, not just the active one).
  `RootView::save_active` (app.rs shim, `#[cfg_attr(test, mutants::skip)]`): reads OWNED `(active_file().path,
  active_buffer().text())`, `std::fs::write(&path, text)?` then `active_mark_saved()` — the `?` is BEFORE the
  mark, so a failed write leaves the file dirty (mark ONLY on Ok); a None editor tab → `Ok(())` no-op (⌘S on a
  terminal is safe). ⌘S is bound via the pure keymap `(chord(true,false,false,false,"s"), "save")` + a
  `dispatch_action` `"save"` arm (on `Err` → a `status_flash`). Driven: the ● appears per-file on type (the
  ⌘S-write itself is critic-verified — right bytes to the right absolute path — since the finder wouldn't open a
  scratch temp file and data-safety forbids ⌘S on a real repo file). Save-as / save-all / external-change =
  deferred. ⚠️ The `mutants::skip` DETACH TRAP recurred here (inserting `save_active` above `dispatch_action`
  stranded its skip → 11 live mutants); fixed by regrouping — RUN `cargo mutants --list` after inserting a fn
  adjacent to a masked shim (a doc/attr binds to the NEXT item). See
  `docs/planning/pipeline/completed/editor-save.*`.
- **M20 #354 — the save machine is instance-addressed (origin-targeted format-on-save).** `save_active`'s tail
  is extracted to `write_and_mark(id: ContentId)` — fs write → `didSave` to the instance's **owning root** host
  (`EditorInstance::root()`; `LspHost::did_save` doc-guards, so mis-routing was a silent no-op and owning-root
  is strictly better) → `mark_saved`/`set_disk` → the #284 racing-length cross-check — and `save_active` keeps
  only the interactive head (active external check + the #275/#284 arm flow). A parked format-on-save completes
  through `save_editor_by_id(id, cause)`: an ACTIVE origin delegates to `do_plain_save` verbatim (flash order,
  arm flow — unswitched behavior byte-identical), a BACKGROUND origin runs the #275 decision table on the
  instance with strict consent (`Changed` → banner set, NO write, NO arm — D-ARMED-BACKGROUND-HOLDS;
  `CleanReload` → skip, activation owns the reload per D4; `Deleted` → recreate), and a dead id (origin closed
  mid-format — `ContentId` is monotonic, never reused) drops silently. `begin_format_on_save` SETTLES a
  different-origin parked latch before re-parking (inspect F1: ⌘S on B must not discard A's consented save);
  `reload_active_from_disk` clears `formatting_request` (the #401 version-epoch-collision class — inspect F2);
  the latchless ⌥⇧F response resolution is pinned to the responding root (inspect F3). `active_is_origin` and
  the surface-level `has_path`/`active_mark_saved` delegates are RETIRED (id-granular identity replaced them).
  See `docs/planning/pipeline/completed/354-format-save-origin.*`.
- **M15 #253 — editor undo/redo (coalesced ⌘Z / ⌘⇧Z).** `marley_editor::undo::UndoHistory` — a pure two-`Vec`
  push/coalesce/pop stack of `EditRecord { at, removed, inserted, origin }` (gpui-free, no rope; the Buffer owns
  one and drives the apply). `Buffer::edit` now captures `removed` PRE-mutation, then delegates to a private
  `apply_raw` (the old edit body: rope remove/insert + version bump, builds the identical `EditResult`, but does
  NOT record) and records the step. `Buffer::undo` / `redo` → `Option<CharOffset>`: pop a record, re-apply the
  inverse via `apply_raw` (so the inverse never re-records → the stack DRAINS, never loops), move the record to
  the other stack, and return the caret site (end of the affected span). Coalesce = a 5-guard `&&` (last is an
  insert-run + new is a pure 1-char insert + same origin + contiguous `last.at + last.inserted.chars == rec.at`);
  `record` clears the redo stack unconditionally (before the coalesce branch). All offset math is
  `.chars().count()` (multibyte-safe). ⌘Z/⌘⇧Z bound via the pure keymap `(chord(⌘,z), "undo")` /
  `(chord(⌘,⇧,z), "redo")` (roster guard bumped to 41) + guarded `dispatch_action` arms that call the #251
  `active_buffer_and_caret_mut` combined accessor and sync the caret; a terminal tab → no editor surface → no-op.
  The arms are shim (`dispatch_action` stays `#[cfg_attr(test, mutants::skip)]` — the arms live INSIDE the body;
  the mutants::skip detach trap was re-checked `== 0` at implement AND inspect). Undo bumps `version`, so it does
  NOT re-clean the #252 dirty ● (content-may-match-disk-yet-dirty; a saved-checkpoint model is a future ticket).
  Pure seam cov/MSI 100 (15 targeted tests; the `Some(Default::default())` mutants are auto-unviable — no Default
  on EditRecord/EditResult). REQ-005 driven-proven live (⌘Z removed an 8-char coalesced run in one step, ⌘⇧Z
  restored it). See `docs/planning/pipeline/completed/editor-undo.*`.
- **M15 #254 — click-to-place-caret in the editor.** The pure inverse of #250's map: `LineLayout::offset_of_col(
  col)` (code_view.rs) scans the SAME `col_starts` `col_of_offset` reads for the nearest boundary (`<=` so a tie
  goes to the LATER index — a click on a char's right half lands after it), clamped to `[0, n_chars]`; because
  `col_starts` is strictly increasing the round-trip `offset_of_col(col_of_offset(i))==i` is exact.
  `offset_for_click(line, col, tab_width)` = `line_layout(...).offset_of_col(col)` (the in-line char offset), and
  `Buffer::line_start(row)` = `line_to_char(row.min(len_lines-1))` (the absolute char offset of a row, clamped —
  no panic). The app composes `line_start(row) + offset_for_click(line_text(row), col, tw)` = the exact inverse
  of #250's forward `line_col` caret map (the shared-risk closed by construction). Shim (in `code_view_body`,
  `#[cfg_attr(test, mutants::skip)]`): a `canvas` element records the text-column left pixel (`bounds.origin.x`)
  into `editor_text_x0: Rc<Cell<f32>>`; each editor `code_row` (now `.w_full()`) has an `on_mouse_down(Left)`
  computing `col = round((click.x − x0) / cell.w)` — the ROW is the render loop var (so NO y/scroll math), the
  handler is only built in the editor branch (a terminal / #246 read-only `None` pane is unaffected), and it
  sets the active file's caret via the #251 `active_buffer_and_caret_mut` accessor. `code_view_body` gained a
  `cx: &mut Context<Self>` param (threaded from its 2 render call sites) for `cx.listener`. Inspect: 2 critics,
  0 code defects (the inverse round-trips, tie→later confirmed, E0499-free, `code_view_body`/`dispatch_action`
  mutants 0) + a defensive `cell_w.max(EPSILON)` (a degenerate 0 em-advance can't divide-by-zero). cov/MSI 100
  on the 3 pure fns. REQ-002/005 driven-proven live (click mid-word → the caret lands there → typing inserts at
  the click). Drag-select / word-select deferred (#255+). See `docs/planning/pipeline/completed/click-to-caret.*`.
- **M15 #255 — text selection (shift+arrows + mouse-drag).** The MODEL: an `anchor: Option<CharOffset>` per
  OpenFile (None = a bare caret; Some(a) = the selection a..caret) + `active_selection()` (the normalized
  `(min, max)`, None for a zero-width caret) + the E0499-safe combined `active_buffer_caret_anchor_mut`. Pure
  seam (cov/MSI 100): `movement::extend_or_move(anchor, caret, buffer, right, shift)` (shift pins the anchor +
  moves the head; unshifted collapses a real selection to its edge, else moves; `.filter(|a| *a != caret)` treats
  anchor==caret as no selection) + `code_view::row_selection_cols(sel_start, sel_end, row_start, row_nchars,
  layout)` (the selected display columns via `col_of_offset` — the #250 map, so the highlight aligns with the
  caret across tabs/multibyte) + `input::selection_replacement(key)` (Char→Some(c) / Backspace→Some("") / else
  None). Shim (render on_key_down + code_view_body, both `mutants::skip`): the key route reads
  `event.keystroke.modifiers.shift` (`key_from_keystroke` maps a shifted arrow to Key::Left/Right — the D3
  route), routes Left/Right→extend_or_move, and a printable/backspace over a non-empty selection →
  `buffer.edit(start..end, repl)` (start..end from `active_selection` = NORMALIZED, so a backwards selection
  edits min..max) + collapse; the drag = the #254 per-row on_mouse_down seeds anchor=caret + `dragging_selection`,
  a per-row on_mouse_move (gated on the flag; row = the render loop var → NO y math) extends the head, on_mouse_up
  clears it, PLUS a self-heal in on_mouse_move (`if event.pressed_button != Some(Left) { dragging=false; return }`)
  because gpui's on_mouse_up is BOUNDS-GATED and an off-row release would otherwise leak the flag ([[BF-claude-
  drag-flag-leaks-on-bounds-gated-mouse-up-255]] / [[PR-claude-drag-flag-needs-buttonless-move-selfheal-001]]);
  the highlight = a per-row accent-alpha rect drawn BEHIND the text. Inspect: 3 critics, 1 HIGH (the drag-flag
  leak, FIXED) — the pure logic + the edit-over-selection normalization + the shift-threading all CONFIRMED.
  REQ-002/003/004 driven-proven live (drag → highlight → type replaces the range). Deferred: multi-cursor,
  shift+word/Home/End (#257), copy/paste (#256), double/triple-click. See
  `docs/planning/pipeline/completed/selection.*`.
- **M15 #256 — editor copy / cut / paste (⌘C / ⌘X / ⌘V).** An `on_key_down` editor-clipboard branch
  (`platform && !shift && key ∈ {c,x,v} && active_tab().editor().is_some()`), placed BEFORE the terminal
  cmd-C/cmd-V and AFTER the ⌘⇧C git-pane block, `return`ing unconditionally on entry — so an editor tab's
  ⌘C/⌘X/⌘V never leak to the terminal handlers, and a terminal tab (editor() None) skips the branch → the
  existing terminal copy/paste run unchanged (REQ-004). ⌘C copies `active_selection()`'s `text_in_range` to the
  clipboard (no selection → no-op); ⌘X copies + `edit(s..e, "")` + collapse; ⌘V reads the clipboard → the pure
  `code_view::paste_edit(selection, caret, clip_chars) -> (start, end, new_caret)` (replace the NORMALIZED
  selection (#255) else insert at the caret; caret = start + clip_chars, CHAR-indexed) → `edit(start..end, clip)`
  + caret + collapse. All edits go through `Buffer::edit` → #253's undo records them (one ⌘Z reverts a cut/paste),
  and the #252 dirty ● updates. Inspect: 2 critics, 0 code defects — the cut's COPIED span == the DELETED span on
  the normalized `(min,max)` range (a backwards selection cuts the right span, no reverse-range panic); the
  editor-vs-terminal guard; E0499-free; `c`/`x`/`v` are NOT keymap actions → no double-handling with the keymap
  dispatch. cov/MSI 100 on `paste_edit`. REQ-001/002/003 driven-proven live (⌘X cut deleted the selection; a
  ⌘X→⌘V round-trip restored the line — the clipboard carried the text). Deferred: copy-line, multi-cursor,
  ctrl-editing (#257). See `docs/planning/pipeline/completed/clipboard.*`.
- **M15 #257 — editor caret-motion keymap parity (Home/End, ⌥←→ word, ⌘←→ line, ⌘↑↓ doc, Up/Down vertical).**
  TWO pure additions to `marley_editor::movement`: `move_up`/`move_down` (the same visual column on the adjacent
  row via `line_col`/`line_start`/`line_text`, the char-column CLAMPED to the target line's char count; the
  last/first row stays via `.min(last)`/`.saturating_sub(1)`; multibyte-correct since `col` and
  `line_text().chars().count()` are both char units) + `extend_or_go(anchor, caret, target, shift)` (generalizes
  #255's shift-extend to any PRE-COMPUTED target: `shift` → pin the anchor [`unwrap_or` the old caret] + move to
  target, else clear the anchor + move — the char-arrows KEEP `extend_or_move`'s collapse-to-edge nuance). TWO
  shim branches in `on_key_down`: (a) the #251 editor branch gained a `Key::Home|End|WordLeft|WordRight|Up|Down`
  arm (each → its movement fn → `extend_or_go`); (b) a NEW platform ⌘-motion branch
  (`platform && !alt && !control && key ∈ {left,right,up,down} && editor().is_some()` → ⌘←=`move_line_home` /
  ⌘→=`move_line_end` / ⌘↑=`CharOffset::zero()` / ⌘↓=`len_chars` → `extend_or_go` → `return`), placed BEFORE the
  #256 clipboard branch and `!alt`-guarded so ⌘⌥-arrow stays the #131 pane-nav. `Key::Up`/`Key::Down` were added
  to the input `Key` enum + `key_from_keystroke`; a terminal tab's up/down is intercepted earlier (history
  recall) so scrollback is unchanged, and `apply_key`'s `Up|Down → Ignored` is byte-equivalent to the old
  `Other → Ignored` (no prompt regression). Inspect: 2 critics, 0 code defects (vertical clamps, multibyte,
  `extend_or_go`, #255-unchanged, ⌘⌥-arrow pane-nav survival, no prompt ↑/↓ regression — all confirmed). cov/MSI
  100 on the 3 pure fns; the app.rs mutant count is unchanged at 6 (the branches are in the render-skip'd
  `on_key_down`). REQ-002 driven-proven live (click line 21 → Down×5 → line 26 / Up×5 → line 16, the column held,
  `selection.rs` pristine). Deferred: goal-column memory, page-up/down, smart-home. See
  `docs/planning/pipeline/completed/motion.*`.
- **M15 #258 — split-file panes persist across restart (`c=<path>` grid codec; the #246 follow-on).** The grid
  codec (`grid_layout.rs`) gains a `c=<path>` leaf token — a STRICT sibling of #205's `t=<cwd>`: `serialize_leaf`
  `(CodeView, Some(path)) → "c={path}"`, `parse_leaf` `c=<path> → (CodeView, Some(path))` (a bare `c` →
  `(CodeView, None)`, back-compat), and `flatten` gains a parallel `paths: &HashMap<PaneId, String>` param — a
  Terminal reads `cwds`, a CodeView reads `paths` (no cross-talk; both framing-filtered via `breaks_grid_framing`
  so a path holding `,`/`:`/`=`/`\x1f` drops to a bare `c`). The path rides `GridLayout.cwds` (the leaf's generic
  detail slot; the field keeps its #205 name). The SHIM (`app.rs`, coverage-excluded): `persist_grid` +
  `build_shell_layout` each capture a `paths` map from `state().code_view()`'s `cv.path`; the `restore_panes`
  CodeView arm (was a `{}` drop per #154) re-reads via the #243 editor-tab restore ladder (`resolve_under_root` +
  `fs::read` + `viewer_size_ok` + `!is_probably_binary` + `CodeViewState::new(applied.code_tab_width)`) →
  `open_pane(CodeView)` — the ONE closure serves BOTH restore paths (whole-shell + legacy single-grid).
  `split_file_pane` already triggered `persist_grid` (#246, anticipating #258). SCOPE: the EDITABLE split pane was
  split out to a fast-follow (#259) — a `PaneContent` model change rewiring 16+ `active_tab().editor()` sites,
  measured at plan (evidence-based split); the pane stays read-only here so #259 inherits a persisted surface.
  Inspect: 2 critics, 0 logic defects (codec round-trip / framing / back-compat / no-cross-talk + the restore
  no-panic + both-restore-paths + no-third-pane-rebuild-path all confirmed) + 2 doc-rot fixes. cov 100 / MSI 100
  on the codec (T1/T2 kill the 2 flatten CodeView mutants the #205 tests missed — they always passed an empty
  `paths` map); app.rs mutants unchanged at 6. REQ-001/002/003 driven-proven live (split → `c=<path>` landed in
  settings.toml before relaunch → quit/relaunch, no crash → the live restored workspace re-serialized `c=<path>`,
  proving the restore arm re-created the pane — a stable round-trip). See
  `docs/planning/pipeline/completed/split-pane-persist.*`.
- **#259 — editable split pane: the editor stops being a singleton.** The #258 fast-follow makes a FOCUSED
  split code pane a real editing surface (type/save/caret/selection/motion/undo). The plan recon corrected the
  spec's "16-site" premise to **~52 editing sites** and split it to slice 1 (core editable pane) + slice 2
  (find/fold/⌘⇧O/LSP-in-pane, follow-up). Four shipped single-slot assumptions broke, each with a bounded fix:
  (1) the pane had no `Buffer` → `PaneContent::CodeView(CodeViewState)` became `CodeView(EditorSurface)` (the
  #237 recast pattern — `PaneState::code_view()` reads through `surface.active_file()`, so the render + the
  #258 `c=<path>` codec are unchanged; the cascade was ~8 sites, not a rewrite); (2) one app-wide
  `FocusHandle` + `PaneGrid.focused` → the NEW `PaneGrid::focused_editable_surface()/_mut()` +
  `focused_pane_is_editable()` seams (the `focused_terminal()` twins, cov/MSI 100); (3) per-tab `key_context()`
  → focus-aware (a terminal tab whose focused pane is a CodeView publishes `[Editor]`); (4) one `editor_geom`
  Cell + one `handle_input` registration → written ONLY by the focused editable surface's render frame (the
  slot stays single, its writer chosen by focus — macOS's one-focused-text-input model). **The spine is the
  accessor:** `active_editor()/_mut()` became focus-aware (resolving the focused pane, return type unchanged),
  so ~75 through-accessor sites were untouched and only the ~43 inline `active_tab().editor()` bypassers moved
  onto them (grep-gated, REQ-009). **D-OWN-BUFFER-V1:** the same file in tab + pane = two independent Buffers;
  the shipped #275 conflict machinery is the no-silent-clobber net. Inspect caught a real HIGH: a shared
  `'static` render row closure re-read `active_tab().editor()` (= None for a split pane's terminal host tab) →
  the focused pane rendered BLANK; fixed to the focus-aware `active_editor()`, drive-proven. GATE GREEN [diff]
  15/15, coverage 100 (workspace.rs + tabs.rs seams) + MSI 100 (14/14). See
  `docs/planning/pipeline/completed/259-editable-split-pane.*`.
  **M22 (#356) — an UNFOCUSED editable split pane re-syncs its read-only lines from the live Buffer.** #259's
  focused pane renders from its Buffer, but an unfocused one falls to `CodeViewState.lines` (set at file-open,
  never re-synced), so a focused edit "vanished" on click-away. A version-memoized `CodeViewState::sync_lines_from`
  (reusing the tested `code_lines`) re-syncs each open file's lines from its Buffer in a `&mut` pre-pass at the top
  of `render` (O(1) unless the buffer advanced; the FOCUSED pane is skipped — it renders from the Buffer). A reload
  resets the memo — a fresh Buffer restarts at version 0, the #268 `(nonce, version)` trap (inspect F1).
  **M22 (#357) — the #275 external-change banner renders on a focused editable split pane too.** The Keep-mine/
  Reload (Changed) / Dismiss (Deleted) card was inline in the editor-tab block; extracted to a reusable
  `ext_conflict_banner` builder and rendered on the focused split pane's body (`.absolute` top-right; `active_editor()`
  resolves the pane, so its buttons act on it). Byte-equivalent extraction — the editor-tab banner is unchanged.
  **M22 (#355) — slice-2 editor-feature parity in the focused split pane is VERIFIED.** #259's focus-aware
  `active_editor()`/`active_editor_mut()` + `editor_geom` (written for the focused pane) + the top-level
  focus-aware overlays already deliver find/fold/⌘⇧O + the LSP/nav caret features on a focused split pane;
  three headless drives (⌘⇧O/⌘F/fold) prove + guard it. The recon's D-CORE-EDITING-ONLY-V1 tab-side scoping is
  superseded — the parity fell out of the accessor + the overlays, not a per-feature port.

## M10 — Warp polish + shell hardening (in progress)
- **#161 — tab close.** Each rail Tab row carries an × (`close_tab_at`; the × handler `stop_propagation`s so
  the row's switch never co-fires); `⌘W` branches — a multi-pane terminal closes the focused PANE (now also
  persisting), else the active TAB closes. `Project::close_tab` gained the `TabError::LastTerminal` guard
  (absorbing #159): a multi-tab project refuses closing its only terminal, so `workspace()`'s ≥1-terminal
  invariant is enforced at the source. A closed terminal tab conservatively clears `last_agent` (PaneIds alias
  across tabs — the full per-tab map scoping is #167/#158). `Workspace::project_mut(idx)` added.
- **#162 — project close.** Each rail Project row carries the same × (`close_project_at` → the tested
  `Workspace::close_project`, refusing the last project with a flash). The whole removed project — every
  tab's grid and PTYs, an all-by-value ownership chain with no main-thread-affine Drop — is dropped on a
  spawned thread; `sync_active_project` then resyncs Files/branch/cwd to the survivor and the grid persists.
  Stale agents/remotes map entries from the dead project remain until #167's per-tab scoping (commented there).
- **#359 — project close also drops the LSP host.** `close_project_at` now removes the closed project's entry
  from `lsp_hosts` so `LspHost::drop` shuts down + reaps its rust-analyzer instead of leaking it until app quit
  (the map was insert-only before this; #321 made host creation open-document-state-derived on the pump, so a
  restored multi-project session had one rust-analyzer per root to reclaim). The drop is LAST-ONE-OUT, gated on
  the pure `Workspace::has_project_with_root`: two open projects can share a root and thus one host (opening the
  same directory twice is not deduped), so the host is dropped only when no surviving project shares the root.
  The map is keyed by the raw project root (= the insert key at host creation), so the guard's `Path` equality
  and the map key's `Path` equality are the same relation on the same field. Dropping a host on the last
  editor-tab close is a separate deferred (#321) decision.
- **#164 — a code tab per file.** `open_or_switch_code` keys by `state.path`: a tab already viewing that file
  is refreshed (fresh read) + switched to; any other file appends its own code tab. Replaces #154's
  single-reused-tab decision now that tabs are closable (#161). Pure-only; PathBuf equality is exact — a
  symlink/`..` alias would open a separate tab (cosmetic; all callers use the same canonical sources).
- **#171 — the right-dock vestige sweep.** The #153 cockpit-tabs retirement left dead plumbing: removed the
  `DockRight` setting, `persist_dock_right`, `AppliedSettings.right` (write-only — the docks init already
  hardcoded `Closed`), and the caller-less `toggle_dock()`; `toggle_dock_state` collapsed to the left-only
  `toggle_left_dock`. Kept as the honest general primitive: `DockSide` + `dock()` + `region_widths(left,
  right)` with `docks[1]` a documented permanently-Closed slot (region_widths yields right=0). An old
  `docks.right` key is tolerated-and-ignored on load. Zero behavior change.
- **#183 — command + history Tab-completion.** Extends the #89/#96/#178 engine: at the COMMAND position (a
  slash-free FIRST word) Tab completes `$PATH` executables + session history instead of the cwd listing. Pure
  `complete.rs`: `is_command_position(line, word_start)` = `line.chars().take(word_start).all(char::is_whitespace)`;
  `merge_candidates(path_names, history_words, prefix)` = deduped history hits first (given/most-recent-first
  order) then sorted PATH hits (excluding history-seen), prefix-filtered (a `BTreeSet` seen). Shim (app.rs,
  masked): `complete_at_prompt` branches on `is_command_position(&line, start) && !word.contains('/')` (a
  path-form command like `./x` or `/usr/bin/x` falls through to dir completion, like every shell — inspect MED);
  the command candidates = merge_candidates over a cached `path_commands` ($PATH basenames, read once via
  `read_path_commands`) + the focused pane's history first-words (slash-free); the popup pool = the empty-prefix
  merge for #178 refilter. Everything downstream (common_prefix, the #96 popup, refilter) is unchanged. NOTE:
  inserting `path_command_names` next to `complete_at_prompt` detached the latter's `mutants::skip` — a
  RECURRENCE of the #184 skip-detach class (`PR-verify-skip-attr-still-attached-after-inserting-fn`); caught by
  the inspect critics + fixed by reordering. Run `cargo mutants --list` after inserting a fn near a skipped shim.
- **#186 — scrollback find next/prev + "n of m".** Extends the #51 find bar (which already had Enter/⇧Enter →
  `match_navigation` wrap + scroll). Pure `find.rs`: `scrollback_matches(rows, query) -> Vec<(row, Range)>` (the
  row-flatten core `find_match_rows` now delegates to, reusing the tested `find_matches` per row) +
  `match_label(current, total)` (the 1-based "n of m", "" at 0). Shim (app.rs, masked): the find bar shows
  `match_label(find_index, count)`; the render tints the ACTIVE match's row (`find_match_rows[find_index].row`)
  with a brighter `active_find_bg` (success @ 0.6 vs find_bg @ 0.35). Inspect fixes: the active tint is a bare
  INDEX compare, so it's gated on `is_focused` (else it phantom-tints a non-focused pane's same-index row — see
  `PR-claude-index-highlight-needs-owner-guard`); the find highlight was added to the HEADER render too (a
  command-line match is counted + navigable, so it must light up); the counter clamps `find_index.min(count-1)`
  so a fold shrinking the match set can't render "n of m" with n > m.
- **#185 — sticky command header (Warp/devtools sticky-scroll).** Pure `nav::sticky_block(counts, folds,
  viewport_top) -> Option<usize>`: over the #184 fold-aware rows, `Some(block)` when the top visible row is that
  block's OUTPUT (its real header scrolled above), `None` at a HEADER row (already visible — no double) or the
  prompt/past-end (elegantly, the prompt's row index == `fold_visible_rows.len()`, so "at prompt" and "out of
  bounds" are the same `None`). Shim (app.rs, masked): after `(start,end)`, when `sticky_block(…, start)` is
  Some, render an `.absolute().top(0).w_full()` overlay row (the block's status glyph + command) pinned at the
  pane's top edge over the scrolled output. Uses `.block_mouse_except_scroll()` (NOT `.occlude()`) — the sticky
  is a NON-MODAL overlay on a scroll area, so it must block click/selection fall-through but pass scroll through
  (else the top band becomes a scroll dead-zone; inspect HIGH — see
  `PR-claude-block-mouse-except-scroll-for-nonmodal-scroll-overlays`).
- **#184 — block folding (Warp fidelity).** Pure `nav.rs`: `FoldState { folded: BTreeSet<usize> }`
  (toggle/is_folded/retain_below — resets folds >= the block count when the list reshapes); `RowKind
  { Header(i), Output(i,l) }`; `fold_visible_rows(counts, folds)` (Header always + Output only when unfolded);
  and its fold-aware twins `fold_block_at_row`/`fold_boundary_rows` that SUPERSEDED the raw `block_at_row`/
  `block_boundary_rows` (a folded block shifts every row index, so the #175 right-click menu + ⌘↑/↓ jump had to
  become fold-aware or they'd target the wrong block). Per-pane `TerminalPane.folds`. Shim (app.rs, masked): the
  block header draws a ▾/▸ chevron (a click → `toggle_block_fold` + notify); `content_rows` derives its count
  from `fold_visible_rows`, and the render + `content_row_texts` (copy/find/selection) mirror the same
  `is_folded` gate so a folded block emits only its header and every row index stays aligned (#47/#175);
  **#193:** `content_rows` wraps that fold-aware length in the PURE `nav::content_row_count(visible_rows, prompt_visible)`
  (cov/MSI 100) with `prompt_visible = !is_command_running`, so the trailing `❯` prompt row is counted only when
  it's actually painted — hiding it while a command runs drops it from the count too (no phantom-row viewport skew);
  `retain_below` runs in the pump. **Agent observation is fold-INDEPENDENT**: `observed_row_texts` (raw, all
  blocks) feeds `agent_tail`/`agent_last_line` (#78/#180) so a human folding a block in an agent's pane can't
  blank its Fleet line — folds are a per-pane VIEW toggle, not a change to what the brain observes (inspect fix;
  see `PR-claude-view-toggles-must-not-corrupt-observation-reads`).
- **#181 — the agent launch picker (the "brain controls agents" slice).** Pure `agent_launcher.rs`:
  `launchable_kinds() -> &[Claude, Codex]` + `AgentLauncherState { selected, prompt }` mirroring PaletteState,
  but the kind list is FIXED so move WRAPS (branch form `if sel == 0 { n-1 } else { sel-1 }` — NOT modulo: at
  n=2 a `-1` ≡ `+1` mod 2 leaves the modulo form an equivalent mutant, MSI < 100; see
  `PR-claude-branch-form-wrap-not-modulo-for-small-fixed-lists`), and typing feeds a separate prompt buffer
  (no reset-on-edit). Shim (app.rs, masked): a `agent_launcher: Option<AgentLauncherState>` field; the
  "new-agent" verb OPENS the picker (both ⌘⇧A and the 🧠 icon; it clears other overlays first so the 🧠 mouse
  click can't stack); `handle_launcher_key` (esc/enter-take/↑↓/backspace/type); the overlay render; and a
  generalized `launch_agent(kind, prompt)`. **Initial-prompt delivery:** a same-frame send after the launch
  line is DROPPED by the agent TUI's startup input-flush (validated), so `launch_agent` stashes the prompt in a
  `pending_agent_send` field and the pump delivers it via the #72/#174 across-grids send path once the pane is
  `is_command_running` (past the flush), then clears it. Validated live: "hi" + "probe181here" landed in their
  claude's input. **M13 #229:** the picker now has a full-screen `.inset_0().occlude()` click-away backdrop
  (Left + Right `on_mouse_down` → `agent_launcher = None`) mirroring the #166 context-menu dismiss — an OUTSIDE
  click dismisses, an inside click is swallowed by the box's own `.occlude()`. The open guard now also clears the
  #204 `naming_workflow` draft (a full-screen dismiss backdrop HIDES, not just occludes, any overlay rendered
  before it — inspect F1).
- **#188 — a live fleet badge + click-to-observe.** Pure `agent_view::fleet_badge(agents) -> Option<String>`:
  the count of RUNNING agents (`Working | Waiting` — the #167 aggregate's two live states; Idle + Exited excluded),
  or `None` at zero so the badge hides. Shim (app.rs, masked): the top-bar cockpit tab loop overlays the badge
  (an accent `on_accent` pill, `.relative()` icon + `.absolute()` corner child) on the Agents tab when Some; the
  footer's agent segment (index 1 of the pure `cockpit_status` then; index 0 since #411 dropped the sprint segment — the agents-first footer) is now a clickable div. Both it and the tab icons
  call a new shared `open_cockpit_section(section, cx)` helper (right_section + open_or_switch_cockpit + persist +
  notify) so the affordances can't drift. Note: the badge counts Working+Waiting while the footer text says
  "n working" (Working only) — distinct-by-design (running vs actively-producing). MUTATION NOTE: cargo-mutants
  does not mutate a `matches!` arm and coverage is line-based, so the badge's status SET is pinned only by the
  hand-written per-status unit asserts (see `PR-claude-match-arm-filter-needs-hand-asserts-not-mutation`).
- **#382 — the footer `focus:` label reflects the FOCUSED pane, not a hardcoded "terminal".** The shim
  builds a `FocusTab` by an exhaustive `match` on the active tab's `TabContent` (editor→`editor`,
  cockpit→`cockpit`, terminal→the focused pane's agent/remote/kind via `grid.state(focused).kind()`), and
  the pure `status_bar::focus_label` maps it to the word (a `CodeView` pane reads `editor`, not
  `PaneKind::label()`'s `code`). Both matches are exhaustive over their closed enums (a new pane OR tab
  kind fails the build into the label); the non-terminal arms never read the grid/agents, so the old
  background-agent-label leak is unrepresentable. `cockpit_status` is unchanged.
- **#187 — agent lifecycle: exit code + run duration + a finish flash.** `AgentRun` (in `marley_agent`) gains
  `run_ticks: u32` (lifetime, +1 per pump tick while alive, frozen at exit) + `exit_code: Option<i32>` (None =
  live). Pure `agent_view`: `AGENT_TICKS_PER_SEC = 62` (the shared tick→sec rate `quiet_age` also uses now),
  `fmt_duration(secs)` ("45s"/"2m14s"/"1h03m" at the 60/3600 boundaries), `agent_finish_flash(label, code,
  secs)`, `run_secs(&AgentRun)` (the single duration source, so flash + row can't drift), and `AgentRow.exit:
  Option<(i32,u64)>` — `agent_rows` picks ✕ for a non-zero exit (else the status glyph, ✓ once Exited) and
  `agent_row_text` appends " · exit N · DUR". Shim: the pump's `dead: Vec<(PaneId, Option<i32>)>` carries
  ChildExited's `ExitCode(Option<i32>)`; an exiting AGENT is marked Exited + exit_code + the flash fires ONCE
  (guard `exit_code.is_some()`, idempotent against a kept last-pane re-emitting ChildExited every frame), and
  its row is KEPT — the auto-reap dropped its `agents.remove` so a finished agent persists marked done (a signal
  / pump-Err with no code → -1 → ✕). `refresh_agent_statuses` `continue`s on an exited agent (freezes the
  duration + Exited status against `is_command_running=false`) and now carries `#[cfg_attr(test, mutants::skip)]`
  like its 79 shim siblings — a `--diff` gate mutation-tests a shim fn the moment you touch it, and its pure
  decisions (`agent_status_from`, `run_secs`) are the mutation-covered surface. Manual closes (⌘W / close-tab /
  close-project) still drop the agent; only the automatic reap keeps an exited one (rows accumulate for the
  session — a "clear finished" affordance is a follow-up).
- **#180 — the Agents cockpit output tail (the observe core).** Pure `agent_view::agent_tail(output, n)`:
  strip trailing blank lines, take the last `n`, `trim_end` each (indentation kept). The Agents cockpit
  rows stack a muted `n=6` tail (`agent_tail_lines(pane, n)` reads the agent's pane across all grids —
  `grids().find_map(terminal)` — kept fresh by the #173 pump) below each `agent_row_text`. Watch an agent's
  recent output without focusing its pane — the first slice of the chad-flagged M3/M5 observe thread. (A
  per-agent selection model for a fuller focused tail is a follow-up; the cockpit has no row selection yet.)
- **#182 — the #174 agent follow-ups.** `Workspace::pane_project_root(pane)` (pure, over `locate_pane`)
  resolves an agent's pane → its project root; the ⌘-click agent diff calls the extracted
  `git_working_diff_in(root)` with THAT root (fallback: the active `project_root`) so a cross-project agent
  shows its own repo's diff. The send-to-agent arm gained an else: on a not-sent (target missing or a dead
  kept-pane write fail) it flashes "{label} has exited" and keeps the composed line (#72). `git_working_diff`
  now delegates to `git_working_diff_in(&self.project_root)` — the git-panel + ⌘⇧D diffs are unchanged
  (active-project, correct).
- **#179 — the pane hit-test maps from the bottom.** The render bottom-anchors rows (`justify_end`); the
  click→row inverse (`viewport::bottom_anchored_row`, replacing the top-anchored `row_at`/`row_hit`) now maps
  from the pane bottom (`end-1-floor(y_from_bottom/cell_h)`, `None` above content) so selection + the #175
  menu land on the right block even in a row's bottom sub-cell band — fixing the title-bar + paint-remainder
  skew. `pane_grid_pos` + the menu hit-test compute `y_from_bottom = r.y+r.h-1 - pointer.y`.
- **#178 — the completion popup live-filters.** `CompletionState` gained an `entries` SNAPSHOT (the dir
  listing from open time — never re-read) + `refilter(word) -> keep_open`: recompute `complete_word(word,
  entries)`, clamp `selected`, return `len >= 2`. The popup key `_` arm no longer dismisses on a plain
  printable/backspace — it falls through so the buffer edit types the char, then a post-`apply_key` refilter
  recomputes `word = buffer[start..caret]` and narrows/widens, closing below 2 (the #89 inline Tab finishes a
  lone match). Space/modified/other keys still dismiss.
- **#177 — tab rename + persisted titles.** `Tab.custom_title: Option<String>` (None keeps every #157
  behavior); `titlebar::display_title` gives the precedence custom(non-blank) > command > fallback. The
  shell codec's `TabLayout::Terminal` became `{ blob, title }` on the wire `T=<title>\x1f<blob>` — no `\x1f`
  = the pre-#177 shape, so old blobs restore byte-identically; `sanitize_title` (strip framing + `\x1f`,
  trim, cap 60) guards at WRITE (the #163 D2 stance). Double-click a rail TERMINAL row → the inline editor
  (`renaming_tab` state; the rename key branch is first among the modals — Enter commits+persists, an empty
  draft clears the custom title; the row cell renders draft+caret). Closes dismiss the rename (indices
  shift); the right-click/double-click clear the sibling modals (the #175/#96 exclusivity rule). Non-terminal
  rows refuse the rename (their wire carries no title). Harness gained `dblclickat` (a real down/up×2 with
  `mouseEventClickState`).
- **#201 — cwd-aware tab titles.** `titlebar::display_title(custom, command, cwd, fallback)` gained a `cwd`
  param + a CWD-basename tier, so the precedence is now custom(non-blank) > running-command program token >
  **cwd basename** > static fallback ("terminal N"). The cwd tier reuses `prompt::pwd_label` (the last
  non-empty path segment); a `None`/empty cwd skips it. Implemented as "Option B" — it composes the UNCHANGED
  `rail_tab_title` with `cwd_or_fallback = cwd.map(pwd_label).filter(non-empty).unwrap_or(fallback)` (the cwd
  basename IS the command tier's fallback, with "terminal N" behind it), so `rail_tab_title` stays live + DRY
  (no `command_program` helper, no dead code). The shim `live_tab_title` (masked) now reads the LIVE
  `session.current_prompt().pwd` — the staged cwd that tracks `cd`, the same source `complete_at_prompt` uses,
  NOT the last block's stale `prompt.pwd` — and gates the command on `session.blocks().current()` (the last
  block iff still `Running`), so an idle tab reverts to its cwd on a command's exit. This refines #157's
  "latest command" → "running command; idle → cwd" (Warp parity). cov/MSI 100 on `display_title` (the real
  4-mutant set: the body pair + two `delete !` guards — the `if let`/`if !` form yields no true/false guard
  mutants, unlike the pre-#201 `match`-arm form); the shim carries no mutation load. The rename SEED (#177)
  now WYSIWYG-seeds the cwd basename for an idle tab.
- **#176 — the chrome persists.** `files.open` (both toggle sites persist; boot applies) +
  `window.{x,y,w,h}` (i32/u16 — the Eq-integral rule). The render SETTLE-persists the geometry from gpui's
  reopen-purposed `window_bounds()` — a write only when the bounds stopped changing for a frame AND differ
  from the store (a live drag writes zero times; note: the final write needs one post-resize render — any
  interaction or output provides it). Boot: `run()` pre-reads the saved geometry (a lightweight second
  settings load, masked) and the PURE `sanitize_window_bounds` gates it against `cx.displays()` — min
  400×300, the title STRIP must overlap some display (exact-touch abutment is NOT overlap — the strict
  inequalities are mutation-pinned), `w == 0` = never saved; any failure → centered 1024×768.
- **#175 — the block context menu.** The #166 menu is now kind-parametrized: `MenuKind::Split` (the 3 split
  rows) vs `MenuKind::Block { pane, block }` (Copy Command / Copy Output / Rerun + the split rows) — the
  TARGET rides the kind (the #96 pane-binding pattern), so a #173 pump auto-close shifting focus mid-menu
  can never retarget an action. The right-click hit-test maps from the pane bottom (#179's
  `bottom_anchored_row`; before #179 this was `row_hit`, the non-clamping `row_at` sibling —
  the empty band above bottom-anchored content and past-end are MISSES → the split menu, never a phantom
  block 0) → `block_at_row` (pure, beside `block_boundary_rows`); alt-screen panes always get the split menu
  (their cooked history is invisible). The R50 header buttons now share `copy_block_text`/`rerun_block` with
  the menu (deduped). **M12.2 (#217, Warp parity):** those R50 header buttons (⧉ copy-cmd / ⧉ copy-out / ↻
  rerun) used to render always-visible; they now start `.opacity(0.0)` and reveal via gpui group-hover — the
  block header is a `.group("block-actions")` and each affordance carries
  `.group_hover("block-actions", |s| s.opacity(1.0))`, so a block's actions fade in only when THAT header is
  hovered (group_hover scopes to the nearest ancestor group → no cross-block bleed). The right-click block
  menu is the always-on path (unchanged); glyph/command/separator untouched. A pure-shim render change (no
  new logic), validated by driven at-rest/hover captures. Known debt → #179: the bottom-anchor remainder skew
  in `pane_grid_pos` (inherited from R47, shared with selection — scoped to its own ticket).
- **#174 — agent rows jump; agent delivery is grid-global.** `jump_to_pane` (locate_pane → switch_project
  [+ sync only on a project CHANGE, preserving `files_scroll` otherwise] → switch_tab → the owning grid's
  focus → persist) backs the Agents-tab + Fleet rows (a gone pane flashes). ⌘⇧S and the broadcast resolve
  their targets via `grids_mut().find_map(terminal_mut)` — a background agent is reachable, and the
  broadcast no longer ticket-tags agents it silently skipped. The rail Tab/Pane rows gained the same
  cross-project sync gate. Known follow-ups: the ⌘-click agent diff still runs in the ACTIVE project's
  root; a dead-but-kept agent pane fails ⌘⇧S without a flash.
- **#173 — the pump drains EVERY grid.** `Workspace::grids()/grids_mut()` (all terminal tabs across all
  projects) + `locate_pane(PaneId) -> Option<(project, tab)>` (exact under #167's unique ids; #174 reuses
  it). The 16ms pump iterates every grid — the background PTY-fill stall is gone, agent glyphs/quiet-ticks
  and live titles stay truthful across tabs (the #167/#157 documented limits close) — while RENDER stays
  active-only (gpui coalesces the notify). Dead panes close in their OWNING grid (agents/remotes dropped,
  a stale `last_agent` cleared, the layout persisted; a grid's last pane stays visibly dead). Perf: idle
  +0.2–0.3% CPU for 4 extra PTYs (the WouldBlock fast-path); the mid-burst 8ms/pane retry budget now
  applies per tab — staggering deferred until measured hot.
- **#203 — background-command completion badge.** A new pure `notify::should_notify(pane_focused, status:
  StatusKind, elapsed_secs: u64, threshold_secs: u64) -> Notify{No,Succeeded,Failed}` — no badge when the pane
  is focused or the command ran below the threshold; else the outcome; AT the threshold notifies. cov/MSI 100,
  but note the only VIABLE cargo-mutants are the three `<`-swaps — the whole-body `Default::default()` mutant
  is UNVIABLE (`Notify` has no `Default` derive → excluded), so the T4/T5/T6 tests (the Failure/Running arms +
  the focus short-circuit) carry zero mutation pressure yet must be kept (they're the only guard on that
  behavior). The masked shim in the #173 pump keeps a per-pane `notify_ticks: HashMap<PaneId,u32>` counting
  16ms frames while `is_command_running()` (`elapsed_secs = ticks × PUMP_INTERVAL_MS ÷ 1000` — `Instant` is
  banned in pure code, the shim converts; the tick counter also avoids touching the serialized `PaneState`).
  On the running→idle edge it reads the last block's `exit_status_kind` + `pane_focused = Some(id) ==
  active_foreground_pane` (the active tab's focused pane, snapshotted BEFORE the `grids_mut()` borrow — a
  disjoint `RootView` field, so the in-loop `notify_ticks` access is legal; `None` when the active tab is
  cockpit/code → every terminal finish counts as background), calls `should_notify`, and — two-phase after the
  loop, via `locate_pane` (which re-borrows `shell`) — sets a per-tab `tab_flashes: HashMap<(proj,tab),Notify>`
  badge, rendered as a ● (`colors.success`/`colors.danger`) on the rail Tab row beside the #167 agent glyph.
  The badge persists until the tab is VIEWED: each pump tick prunes the active tab's badge (clear-on-view), and
  that prune sets `dirty` so it repaints on an otherwise-idle frame (inspect F1 —
  `PR-claude-pump-state-change-must-set-dirty-to-repaint`). *(#398 re-keyed `notify_ticks` to `ContentId` with
  the release-tail scrub; the "hardening on manual close" follow-up died there.)* **#226 shipped the OS half +
  the badge-map remap:** pure `notify::os_note(window_active, Notify, command) -> Option<OsNote>` (fires only
  while the WINDOW is inactive — `last_window_active`, the render focus-edge flag; body = the command at
  ≤120 chars) feeds `marley_command::blocking::notify_macos(program, title, body)` — the osascript-argv
  adapter (constant `-e` script lines, payload as opaque trailing run-handler args = injection-free by
  construction; program-injected for tests per the `open_url_with` seam pattern; a detached thread REAPS the
  child — `PR-claude-fire-and-forget-spawns-still-reap-001`). And `tab_flashes`' positional keys now REMAP on
  close (`remap_flashes_after_tab_close`/`_project_close`, the #236 close-shift discipline) so a surviving
  unviewed badge never shows on the wrong tab.
- **#96 — the completion popup.** The #89 many/prefix-not-longer no-op now opens
  `completion: Option<(PaneId, CompletionState)>` — PANE-BOUND (#167's unique ids make the staleness guard
  exact). Pure `complete.rs`: `CompletionState` (wrap-cycle via the reused `next_index`/`prev_index`) +
  `popup_window` (the ≤max slice ending at the selection). The root key closure's FIRST branch drives it
  (plain-modifier nav only — ⌘/⌃/⌥ chords fall through and dismiss; shift-Tab cycles backward); any other
  key dismisses AND types through. Accept replaces `start..caret` with the full re-prefixed candidate
  (+ space unless a `/`-dir). Render-LAST: an occluding box above the opening pane's input row (≤8 rows,
  short panes shrink it, one honest "… n more" tail, rows click-accept), self-dismissing if the pane isn't
  on screen. One modal at a time: the #166 right-click menu dismisses the popup on open.
- **#160 — cwd follows the project everywhere.** The 3 user-facing spawn sites (⌘T tab, ⌘D/menu split, the
  new-agent split) pass the ACTIVE project's root to `spawn_session_in` (the root cloned before the
  `&mut`/closure use); only the 2 boot seeds keep the cwd-less `spawn_session` — they ARE the launch-cwd
  project. A pane spawns in the PROJECT root, not its sibling's live `cd` (intended: panes follow the
  project, like Files/branch). Proven by lsof: a ⌘T in project B's tab started its zsh in B's root.
- **#163 — the shell persists.** `workspace.shell` stores `serialize_shell`'s blob: line 1 = the active
  project; per project `root \t active_tab \t T=<grid-kinds-blob> | C=<section> | V=<path>` (the embedded
  #122 grid blob's alphabet can't break the framing; a root/path containing `\t`/`\n`/`\r` is dropped at
  SERIALIZE time — restore can't detect the damage, serialize can). `persist_grid` piggybacks the shell save,
  so every existing persist site + all 6 tab/project switch sites cover it. Boot: a non-empty blob rebuilds
  everything (missing roots/files skipped; the launch-cwd project's first terminal reuses the boot PTY via
  `take_if` — block 0, no cwd straddle; other grids `spawn_session_in(root)` on fresh #167 blocks and the
  FIELD `pane_blocks` seeds from the boot counter so post-boot ⌘T can never re-mint a restored block; an
  emptied project gets one terminal; actives clamp) — an empty blob boots the legacy #122 single-grid path.
  Known limits (documented): tab titles regenerate; an all-roots-missing boot falls back to legacy and the
  first mutation persists the trimmed shape. *(Coda — M29 #408: the third original limit — "a dropped tab
  shifts the saved active index (clamped, valid)" — is retired. Every drop now reclamps the sibling active
  index over the SURVIVORS through the shared #243 `reclamp_active`, at all four sites: `serialize_shell`
  pre-scans surviving projects/tabs and writes reclamped actives (covering the pre-existing `V=`-drop drift
  and the empty-root line the READER unconditionally skips — the writer's survivor model mirrors the
  reader's drops); the app.rs restore loop collects surviving original indices as it pushes (failed `T=`
  spawn, all-unreadable `V=`, vanished root) and maps `active_tab`/`active_project` through them instead of
  the old bare `.min(last)` bounds. The terminal grid blob — the one writer input that rode unguarded — now
  drops its tab on an entry-framing hazard (`breaks_entry_framing`: `\t\n\r` + `\x1f`, the predicate now
  shared with the #243 path filter), closing a laundering hole where a hand-mangled multi-`\x1f` `T=` entry
  would re-emit as a forged frame on the next save. No-drop layouts serialize byte-identically; degenerate
  out-of-range actives are deliberately clamped. Pure codec seams at cov/MSI 100; the restore wiring is
  proven by three headless drives seeded through the real codec.)*
- **#167 — rail agent icons + globally-unique PaneIds (closes #158).** Every non-boot grid mints its ids from
  a fresh `PANE_ID_BLOCK` (1 << 32) via `PaneGrid::new_with_base` (the boot grid keeps base 0; ids are never
  persisted), so a `PaneId` names exactly one pane ever — the #158 cross-tab aliasing class is structurally
  dead and the `PaneId`-keyed agents/remotes maps stay byte-identical at all 26 call sites. Closes
  (⌘W / the pane ×, tab ×, project ×) now remove exactly their grids' map entries + a targeted `last_agent`
  clear. The rail Tab rows show `agent_status_glyph(fleet_status_for(pane_ids, agents))` — the aggregate
  (Working > Waiting > Idle > Exited), accent while Working. Known limit (pre-existing): the pump refreshes
  only the ACTIVE grid, so a background tab's glyph can go stale until revisited (the M3/M5 observe work). The rail + the Files tree gained wheel handlers: row-skip offsets
  (`rail_scroll`/`files_scroll` + #165-style remainders) clamped by the REUSED `code_view::scroll_code` (zero
  new pure surface), re-clamped at render so a shrunk list never blanks. `.enumerate().skip(n)` preserves the
  ORIGINAL tree indices (`toggle`/`path_at` key on them); `sync_active_project` resets the files offset.
- **#168 — Files panel polish.** Opening the panel calls `sync_active_project` (a fresh walk — new files
  appear); the width became the drag-resizable `files_panel_w` backed by the `files.width` setting (pure
  `clamp_files_width` 160–480 / NaN→240, applied on load AND persist; a 6px edge grab strip + a root
  move/up pair, the #130 divider pattern). The FILES_PANEL_W const retired — the default lives in
  `FilesWidth`'s `define_setting!`.
- **#170 — tab chords.** ⌘T → `new-tab` (`new_terminal_pane`); ⌘[ → `prev-tab` via the pure
  `tabs::prev_index` (the mirror of `next_index`, cov/MSI 100); ⌘1–⌘9 → `switch-tab-N`, parsed in
  `dispatch_action`'s tail arm (`switch_tab(N-1)`, out-of-range = the guard's no-op). All collision-checked.
- **#166 — the split context menu.** Right-click → a pure `context_menu.rs` overlay (MenuAction / MENU_ITEMS /
  `ContextMenuState` wrap-nav / `menu_origin` clamp, cov/MSI 100) rendered LAST at the pointer: Split Right /
  Split Down (`split_focused_pane` gained the axis) / Close Pane (the #161 `close-pane` path). The backdrop
  `.occlude()`s (the modal convention — the dismissing click must not start a selection beneath); Esc/↑↓/Enter
  route ahead of everything while open.
- **#165 — code-tab wheel-scroll (regression fix).** #154's render move orphaned `cv.scroll` (read-only —
  nothing mutated it once the tiled pane died). The code-branch wrapper now takes `.on_scroll_wheel`: pixel
  deltas → rows (the same `fallback_cell` height the lines draw with) → the shared `scroll_steps` remainder →
  the pure `code_view::scroll_code` clamp (`[0, total−height]`) written through `code_view_mut`. The terminal's
  sign convention; one app-level `code_scroll_remainder` (CodeViewState derives `Eq`, so no f32 member). The
  self-test harness gained a `scrollat:fx,fy,clicks` verb. *(Coda — M16 #266: the EDITOR tab's wheel handler
  and `code_scroll_remainder` are gone — the editor renders in a `uniform_list` that scrolls natively through
  `RootView.editor_scroll: UniformListScrollHandle`, each row ONE `StyledText::with_highlights`;
  `scroll_code`/`cv.scroll` still serve the #246 read-only split pane and the rail/Files-tree row-skip
  paths.)*

## The overlay-card recipe (M20 #318)

The floating overlay cards share ONE extracted chrome: `overlay_card_chrome(colors)` (app.rs, beside
`icon_label`) is the 8-call core every card repeats — `occlude / flex / flex_col / bg(surface) /
rounded(corner_radius) / overflow_hidden / border_1 / border_color(border)`. It is **modal** chrome;
its NON-modal sibling `overlay_card_chrome_over_scroll` (#414) swaps `.occlude()` for gpui's
`.block_mouse_except_scroll()` — clicks/hover still blocked, the WHEEL passes — for members over
scrollable content per `PR-claude-block-mouse-except-scroll-…`: the editor completion popup (whose
scrolled-out-of-viewport guard IS its scroll-dismiss — dead under occlude, working now); hover keeps
its occlude on purpose (the shipped #318-D3 tension). THREE positioning recipes ride the chrome.
**Recipe A (anchored):** hover, def-picker, and the six anchored pickers (references, code_action,
file_symbols, symbols, search, problems) keep caller-side fixed `W`/`MAX_H` + `menu_origin` + mono
type — caller-side ON PURPOSE (#414 recorded): per-site values differ and the fonts need `&self`, so
an `anchored_overlay_card` recipe fn is a NAMED OPTION (as a `&self` method) if a 7th anchored picker
lands, not shipped shape-enforcement. **Recipe B (centered-quarter):**
`quarter_overlay_card(colors, win_w, win_h)` assembles chrome + the pure
`context_menu::overlay_quarter_geometry` (`left = w/4, top = h/6, width = w/2` — containment holds by
CONSTRUCTION at every window size, proven by unit tests incl. an exhaustive-f32 argument; height stays
CONTENT-driven, bottom overflow deliberate — the settled contract `OverlayShell.tsx` measures) + the
family `text_color`, for the five members: palette, file finder, history search, agent launcher, fleet.
**Recipe C (centered input, #414):** `naming_overlay_card(colors, win_w, win_h)` = chrome + the pure
`context_menu::naming_card_geometry` (`0.3·w / h/4 / 0.4·w`, unit-proven like its quarter sibling) +
the trio's shared `.p_3()` + `text_color`, for save-as-workflow (#204), name-an-arrangement (#399),
the fleet dispatch draft (#378). At extraction the chain was verbatim at **17 sites**; 7 converted at
#318 (zero visual delta — the before/after capture protocol + the per-site chain-identity review at
inspect) and the remaining 10 at #414 (the same chain-identity review; the sweep's negative grep pins
the core to exactly the two helper bodies workspace-wide) — **zero verbatim sites remain**. #414's one
deliberate behavior delta is the completion popup's hitbox (wheel pass-through, proven at gpui source);
its live wheel confirmation rides the #417 battery. Selection stays split ON PURPOSE: palette/finder/history CLAMP,
def-picker WRAPS — wrap is structurally coupled to def-picker's selection-windowed rows; the
unwindowed `.take(20)` overlays would wrap the selection onto an unrendered row (the #312 open
question, closed at #318 plan). `recipe_b_overlays_open_and_draw_headless` drives all five open-verbs
through a real draw (writing it surfaced that `close_transient_overlays` omitted the agent launcher —
closed at #415: the launcher joined the roster and the smoke tightened to strict one-hot equality per
iteration, with a verb-table tripwire, a direct roster-clears-all cleanup assert, the fleet
toggle-scope exemption pinned before its toggle-close, and a post-loop all-closed tail). The React
POC's two recorded drifts — `useOverlaySelection` wrapped and its `maxHeight` capped — were
corrected POC-side at #416 (clamp token-equivalent to `palette.rs`/`finder.rs`; content-driven
height with deliberate bottom overflow, DOM-proven at a short window; MARLEY-PARITY.md's overlay
bullets updated to the clamp contract). Three further pre-existing POC divergences surfaced by the
#416 inspect (render-cap `.take(20)`, Enter-on-empty, re-anchor keying) are queued in
`docs/planning/intake/poc-overlay-parity-nits.md`.

- **M20 (#422) — divider drag routes to the grabbed boundary (nested included).** The pure
  `divider_rects(group, bounds, hit)` walk (workspace.rs) emits one descriptor per Split —
  {path, axis, band-inheriting seam-centered strip, extent}; `PaneGroup::resize_at(path, …)`
  (layout.rs) replaced the top-level-only `resize_boundary` (retired), reusing `resize_split`'s
  clamp unchanged; the shim's `dragging_divider` holds a `DividerDrag` and the move handler
  normalizes Δ-along-axis by the OWNING split's extent, ending on gpui's `!dragging()`
  (missed-release hardening) with a total `try_workspace_mut` (no mid-drag panic). The two
  inspect HIGHs were call-site world-mismatches (bounds inset + gate fallback) — the AGREEMENT
  unit (descriptor centers ≡ `pane_rects` edges over identical inputs) pins that class
  permanently. Zero boundary-index math survives (the L TICKET-020 hazard designed out).

- **M31 (#421) — the rail's add-project door (the M31 batch's last port slice).** The Workspace
  dock header carries the #418 accent ＋ → `MenuKind::AddProject` through the one menu machinery
  (const table, keyboard-generic, one-modal via `close_transient_overlays`) → the two shipped
  verbs (`open_project_picker` / `new_empty_workspace`); the post-add tail unchanged
  (`open_project_path`: append + activate `len-1` + Files sync + recent + persist).
  `caption_header`/`dock_panel` host an optional trailing header-action slot. The F-#236 class
  is pinned at the seam: add never re-keys index-keyed view state (remap stays close-only).
  Proven: the table/routing/keyboard-wrap unit, the append-without-rekeying pin, the headless
  menu smoke, and a live end-to-end drive (a real $HOME project appended, activated, Files
  re-walked, PTY spawned, closed clean).

- **M31 (#420) — per-file editor rows in the rail (the #237/#240 projection reversed).** A
  CodeView tab projects as one `RailLevel::File` row per open view — surface order, disambiguated
  basenames, `(editor-tab, view, cid)` coordinates — and the frozen "Editor" row is gone (the
  one-surface model + `EDITOR_TAB_TITLE` survive; zero persistence change). `RailSelection::File`
  makes the active VIEW's row the one selection; a non-active project's active view Rings
  (background-active); pane mounts stay on CrossRef (model A). The × is the (p,t)-addressed
  `close_editor_view_at` honoring `CloseOutcome` — and BOTH close paths now persist the shrunk
  open set (a pre-existing resurrect-on-relaunch gap). The #398 content match hosts the File
  emission as its CodeView arm (no unreachable arms). Proven: the projection suite (order /
  disambiguation / coordinates / coexistence / the 3 background-Ring conjuncts), the re-keyed
  STRUCTURAL collapse pin, a live drive (tree-open → three rows, click, ×), MSI 100 on the diff.

- **M31 (#419) — the rail's single selection + presence dots SHIPPED in Rust (the first #418
  port).** `rail_rows` derives ONE internal `RailSelection { None, Tab, Cell }` coordinate — the
  PR-…-derived-selector-001 shape: coordinates are unique per row and the variants exclusive, so
  two selected rows are unrepresentable; `RailRow.active` → `selected`, `pane_marked` → `dot:
  RailDot { None, Filled, Ring }` (Filled = the #398 foreign-host/cross-ref mount signal,
  unchanged as input; Ring = container-of-focus on the focused cell's Tab + Arrangement rows, and
  background-active on a non-active project's own active tab; Filled outranks Ring; cells and
  structural rows never dot). Structural rows (Project / Section / Arrangement) and CrossRef rows
  NEVER select: the #236 active-project background + bright text and the #386 header fill retired
  with no substitute emphasis (the #418 grammar). The #386 force-expand gate survives
  byte-identically with ONE deliberate correction: the #398 "focused CodeView cell ⇒ Editor is
  the active section" override is now gated on MULTI-cell grids — a 1-cell CodeView grid selects
  its own Tab row, so the force-expand protection must stay on that row's section (the #419
  inspect find; the multi-cell case keeps Editor force-expanded for the cross-link breadcrumb,
  and a user-collapsed host section legally hides the selection, ring surviving — DD-419-COLLAPSE).
  Render: all six arms route the fill from `selected` only (`rail_highlight` untouched, hover 0.10
  as shipped); a 12px leading dot slot on Tab/CrossRef/Arrangement rows keeps labels
  byte-positioned (pl 28 → 16 + slot); the ⊞ render sites died (the glyph survives only in
  history). The #418 indent-ladder/type changes are NOT this slice (#420+). Proven: the
  every-RailLevel ≤1 sweep + truth-table units (2131 green, MSI 100 on the diff), a live drive
  (fill moves alone through tab-create and ⌘⇧L split; containers ring), and the POC parity pair.

- **M31 (#418) — the simple rail, designed React-first (the Zone A rail inversion).** Chad's call
  (2026-08-12): the ancestry-lit four-level rail dies; a ChatGPT-style single-selection rail
  replaces it — designed, built and settled in the marley-web POC BEFORE any Rust (grammar source:
  the observed beautifului.dev captures in `docs/warp_architecture/observed/`). The settled model,
  now the PORT CONTRACT for #419–#421 (geometry sheet + captures 34–38 in
  `marley-web/docs/MARLEY-PARITY.md`): at most ONE row ever carries `--rail-active` (the row whose
  content the center presents — ancestors NEVER light); a 5px left-edge dot carries presence
  (filled = mounted in a pane cell, the #398 ⊞ relocated; ring = the section's remembered-active
  row when not selected; filled ∩ ring → filled); sections become small-caps tracked headers
  (10px/500/0.08em) that never fill — label click focuses the remembered CHILD (empty = no-op),
  ＋/chevron hover-reveal (▸ persists collapsed); the indent ladder flattens 4→3 levels (labels
  x=10/22/34 at 1×); row ×s hover-reveal; add-project is a persistent accent ＋ in the top band
  (#421 wires the real verbs). **The selection is ONE derived selector** mirroring the center
  router (agent-id ⊃ section, project-2 ⊃ both, incoherent → home) — the #418 inspect proved
  per-row booleans break under foreign writers (4 HIGH findings: stale `activeAgentId` double-fill,
  unreset `paneCellFocus`, palette-stranded `activeProject`, the center split-surface writing cell
  ORDINALS into `activePane`); PR-claude-single-selection-is-a-derived-selector-not-scattered-
  booleans-001 binds the Rust port to the selector shape. MARLEY-PARITY.md now carries the one
  sanctioned Zone A inversion: for the rail, the POC is the design source and **Marley is the bug
  until #419–#421 land** (rail_rows' six per-level actives tabs.rs:1047-:1241 + `rail_highlight`
  → #419; the frozen "Editor" row → per-file rows #420; add-project wiring #421).

## Verification
The pure surface — themes/input/terminal_view/shell_integration — is cov 100 / MSI 100. The shim
(`app.rs`) is ACCEPTED-UNTESTABLE, asserted by a headed visual test (AX window "Marley" + a masked,
text-tolerant `shell_dark` baseline in the headed lane) + a headless `#[serial]` real-zsh integration
test (`write_command("echo hi")` → a Finished Block). gate-15 in CI = the `crates/marley_app` dir makes
the component asserted + the harness headless tests stay green.

## Deferred (M1.C seq-4+ / M2)
Settings load/persist (seq-5); real dock
panel CONTENT (the M2 panel system — the docks render titled placeholders); PTY resize-to-rect;
scrollback; drag-resize ratios; the extra screenshot baselines (the headed lane needs a desktop
session — see the forge #23/#24 follow-up comments). SHIPPED since this doc's M1.A origin: the
3-region docks (#24), the `PaneGroup` algebra + real multi-pane render (#20/#23), the palette
overlay + `filter_commands` + selection/dispatch (#19/#25), the `Keymap` (#18/#24), the
AnsiCQuoted shell integration (#22).

## See also
- [crate-map.md](crate-map.md) · [SPEC-app-shell.spec.md](../specs/SPEC-app-shell.spec.md) ·
  the M0 viability spike's gpui boot it promotes (the spike crate was deleted once M1 shipped) ·
  [terminal_blocks.md](terminal_blocks.md) · [editor.md](editor.md) · [ui_components.md](ui_components.md).
