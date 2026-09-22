# 266-b2-render — Notes

- **Forge ticket:** #266 4f9c9b01-73c8-442d-9eb6-d64c3b7563fd
- **AAR:** 7f891f5a-8208-4474-889e-0c41264d288d
- **Local ticket doc:** docs/planning/tickets/open/TICKET-266-b2-render.md
- **Pipeline spec:** 266-b2-render.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch, ticket 8 of 10. B2 render half.
- **API ground truth (vendored 0.2.2 — the #264 lesson applied FIRST):**
  uniform_list (uniform_list.rs:22) + UniformListScrollHandle (:80) +
  StyledText (text.rs:148) + with_highlights (:188) + index_for_position
  (:483) + position_for_index (:517) + HighlightStyle.background_color
  (style.rs:507) — ALL present. No #264-style premise failure.
- **Render map (Explore agent — the full report is the design's ground):**
  the single fn `code_view_body` (app.rs:2656-2878) forks editor
  (Some branch, 2672-2847, ~176 lines) vs #246 read-only (None, 2848-2877);
  fixed VIEWER_ROWS=40 window over cv.scroll; syntax = highlight_line
  chunks → one colored div per span (2729-2738); selection = per-row
  absolute 28%-accent rect via row_selection_cols (2695-2705); caret = 2px
  overlay bar at ccol*cell.w (2743-2751); mouse = per-row handlers with
  stateless x0+cell math → offset_for_click (2779-2843); wheel =
  5507-5537 via scroll_code/scroll_steps; **split_at_caret is NOT an
  editor artifact** (zero non-test callers — dead; split_caret_char is the
  TERMINAL prompt's, one caller at 6051, untouched).
- **The 10 hazards** (from the map, all carried into design): HighlightStyle
  HAS background_color; caret stays an overlay; row height IS uniform
  (cell.h = 1.2×size — the ticket's "φ" quip was wrong); with_highlights
  ranges are BYTES vs the char-pure editor (debug panics on non-boundaries);
  tab expansion stays app-side (keep line_layout, feed `display`);
  click mapping becomes post-paint stateful (per-row TextLayout capture,
  Result<byte,byte> → char); shared fn with #246 (fork, don't rewrite);
  VIEWER_ROWS retirement changes visible-count assumptions; the list
  closure gets &mut App not Context (listeners re-plumb through the
  entity); scroll handle replaces the usize model (opens #270 cheaply —
  out of scope).
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Approach (§20 confirmed as planned — gpui public API, Marley-original glue)
**One design refinement over the plan (recorded as a spec-decision update):**
the caret bar + click/drag mapping KEEP the existing pure monospace cell
math (`ccol * cell.w`, `offset_for_click`) — for a monospace grid over the
tab-expanded `display` string it is exact, already unit-tested, and
stateless; `TextLayout::{position_for_index, index_for_position}` require
post-paint handle capture (the map's fiddliest hazard) and buy nothing
until a proportional font/ligatures land. Recorded as the deliberate
deviation; the REQs' observable behavior is unchanged.

**What swaps:**
1. **Windowing** — the editor branch's `visible_range(cv.scroll,
   VIEWER_ROWS=40, …)` for-loop → `uniform_list("editor-lines", total,
   closure)` + a new `RootView.editor_scroll: UniformListScrollHandle`
   (`.track_scroll`) — true viewport virtualization; the editor-tab wheel
   handler (app.rs:5507-5537) is DELETED (the list scrolls natively);
   `code_scroll_remainder`/`cv.scroll` remain for the untouched #246 pane.
2. **Row content** — the per-span colored divs → ONE
   `StyledText::new(display).with_highlights(&text_style, ranges)` per row.
   Ranges come from two NEW pure fns:
   - `code_syntax::highlight_ranges(line, lang) -> Vec<(Range<usize>
     /*bytes*/, TokenKind)>` — folds `highlight_line`'s sequential chunks
     into byte ranges (Plain dropped; concatenation invariant preserved).
   - `code_view::styled_slices(syntax: &[(Range, TokenKind)], sel:
     Option<Range<usize>>) -> Vec<(Range<usize>, TokenKind, bool)>` — the
     overlap SPLITTER (with_highlights needs disjoint sorted ranges): every
     boundary from both inputs cuts; each out-slice carries its token kind
     (Plain where none) + selected flag. Pure + exhaustively tested.
   - `code_view::cols_to_bytes(display, c0, c1) -> Range<usize>` — display
     char-cols → byte range (multibyte-safe), feeding sel from the existing
     `row_selection_cols`.
   The shim maps (kind, selected) → `HighlightStyle { color, background_color
   (28% accent when selected) }`.
3. **Listeners inside the list closure** — the closure signature is
   `Fn(Range<usize>, &mut Window, &mut App) -> Vec<R>`: capture
   `entity = cx.entity()` + the Rc x0 + cell/tab copies; handlers become raw
   `move |e, _w, app| entity.update(app, |view, cx| { …existing body… })`
   (the same logic, re-plumbed). The x0 canvas rides the first row of each
   returned batch (`row == range.start`).
4. **Caret** — the overlay bar unchanged, still `ccol * cell.w`.

### File manifest
| file | change |
|---|---|
| crates/marley_app/src/code_syntax.rs | + `highlight_ranges` + tests |
| crates/marley_app/src/code_view.rs | + `styled_slices` + `cols_to_bytes` + tests |
| crates/marley_app/src/app.rs | RootView.editor_scroll field+init; the editor branch of code_view_body → uniform_list closure; editor wheel handler deleted; imports |
| crates/marley_app/src/input.rs + lib.rs | `split_at_caret` dead-code removal (+ its tests) |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | highlight_ranges: coverage/order/Plain-dropped/multibyte("éx😀")/tab-display; mutants killed |
| — | styled_slices: disjoint pass-through; sel⊂span; span⊂sel; partial overlaps both sides; sel across spans; empty sel; boundary-equal edges (the classic `<`/`<=` mutant food) |
| — | cols_to_bytes: ascii, multibyte, c0==c1, EOL clamp |
| REQ-002 | headless: open a 120-line file (fixture), assert the editor tab active + `buffer.len_lines()==120`; driven capture scrolled past row 40 (wheel) shows rows >40 rendered |
| REQ-003 | headless_drive typing/undo asserts still green (the lane from #264 exercises the swapped render); captures |
| REQ-004/005/006 | driven captures: ⌘D word-select highlight; caret bar after a tabbed+multibyte prefix; click mid-word places the caret (state assert via headless where possible + pixels via drive) |
| REQ-007 | wheel scroll capture on the editor; the #246 pane capture byte-compared region-stable |
- Uncoverable: the uniform_list closure/listener plumbing lives in the
  documented app.rs shim exclude — asserted by the driven+headless lanes.

### Risks / decisions
- R1 with_highlights disjoint-sorted requirement → styled_slices is the
  guarantee (tested); a debug_assert in the shim guards ordering.
- R2 First-frame x0 not yet recorded → same as today (Rc<Cell> starts 0;
  the first click after paint is correct — unchanged behavior).
- R3 uniform_list + line_height: set text_size/line_height on the row div
  as today; the list measures row 0 (uniform cell.h — verified constant).
- R4 The #265 headless ⌘D test doubles as the regression harness for the
  swap (selection state), with captures for pixels.
- R5 The wheel-handler deletion must not orphan `scroll_steps` (check other
  callers — the #246 pane path) — implement verifies.

## Phase 3 — Implement
- **Built to manifest:** code_syntax::highlight_ranges (byte-range fold,
  Plain-dropped, ascending-disjoint by construction);
  code_view::{cols_to_bytes, styled_slices} (the boundary-cut splitter —
  with_highlights requires disjoint sorted); app.rs — editor_scroll handle
  field+init, the editor branch → the uniform_list closure ('static: fresh
  per-frame entity reads for line text/starts, captured colors/cell/caret/
  sel/tab_width/x0; one StyledText::with_highlights per row [syntax tint +
  28%-accent selected bg]; the x0 canvas rides each batch's first row; the
  caret overlay + the pure mouse mapping unchanged in substance, listeners
  re-plumbed as raw closures through entity.update), .track_scroll +
  flex_1; the editor wheel handler + wheel_cell_h DELETED (the list scrolls
  natively; code_scroll_remainder stays for the #246 pane); the dead
  split_at_caret fn + test + export removed (the render map proved it had
  zero production callers).
- **Deviations:** `let colors = colors.clone()` (ThemeColors isn't Copy —
  the design said capture owned; mechanism identical).
- **Verification:** fmt; check green; **full workspace 908/908** (−1 = the
  deleted dead test) — including the #264 headless lane, which now BOOTS
  AND DRIVES the swapped render (the ⌘D-split test's selection assert runs
  through uniform_list + styled slices end-to-end).

## Phase 3.5 — Inspect
- **Critics run:** 2 parallel — A (range math: a probe crate mounting the
  REAL sources via #[path]; 103,536 exhaustive chain cases + 30k fuzz iters
  over multibyte/tab/lang matrices; read gpui's compute_runs for the actual
  consumer contract; produced the full mutant kill list), B (closure
  semantics: gpui view.rs render-per-draw proof of no mixed staleness;
  line-by-line listener parity; x0 single-canvas-per-frame proof; scroll
  regression sweep; provenance).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| F1 | HIGH (gate) | dead `code_scroll_remainder` (its only reader was the deleted wheel handler) → clippy -D warnings RED | REAL | FIXED: field + init + doc removed |
| F2 | HIGH (MSI) | highlight_ranges' `end > at` guard = an EQUIVALENT mutant (`>`→`>=` unkillable — a non-Plain span is never empty; probe-verified over 60k lines) | REAL | FIXED: guard rephrased `!span.text.is_empty()` (its delete-! mutant IS killable); invariant documented |
| F3 | LOW-MED | a display line wider than the code area WRAPS onto a second visual line the next row paints over (slot pitch = one cell) | REAL | FIXED: `.whitespace_nowrap()` on the row (clean clip at the list edge) |
| F4 | MED | per-file scroll memory lost — ONE global handle vs the old per-file cv.scroll (a #237 behavior regression: file-tab switches now carry the offset) | REAL — ACCEPTED for this slice | recorded deliberately; folded into the scroll-UX follow-up with #270 (scroll_to_item makes per-file restore + caret-follow one change); CHANGELOG discloses |
| F5 | INFO | the read-only branch of code_view_body is UNREACHABLE from the tab caller (code_view() ⟺ editor() — same variant); the old handler's cv.lines fallback was already dead; the #246 PANE never scrolled (pre-existing) | verified | noted; branch left (out of scope; #268 coordinates) |
| F6 | INFO | closure staleness CLEARED (non-cached views re-render every draw; the closure lives within one draw; every mutating arm notifies — incl. the blanket post-dispatch notify) | verified | — |

- **Verified clean:** listener parity line-for-line (incl. the #255
  stuck-drag self-heal + notify placement); x0 benign (render_items runs
  once per frame, measure passes never prepaint → exactly one canvas;
  fixed-gw gutter ⇒ x0 constant across rows); styled_slices output meets
  compute_runs' real contract (ascending/disjoint/char-boundary — 133k
  cases, zero violations); provenance clean (the row idiom is Marley's own
  transplanted; Zed's editor doesn't even use uniform_list).
- **Phase-4 kill list (critic A, viability-traced):** highlight_ranges 8
  viable (T-hr-1 kills 7 + the rephrased guard's delete-!), styled_slices
  16 viable (T-ss-1 kills 14 incl. both `<`→`<=` boundary pairs; T-ss-2
  kills the last `>`→`>=`), cols_to_bytes ZERO viable (no Range Default —
  guard tests for coverage only). T-cb/T-ss-3/4 rows as listed by the
  critic.
- **Post-fix verify:** clippy -D warnings clean; 372/372.

## Phase 4 — Validate
- **Kill-list tests (critic A's viability-traced menu, all landed):** T-hr-1/2/3
  (highlight_ranges — the 8 viable mutants killed, incl. the rephrased guard's
  delete-!), T-ss-1/2/3/4 (styled_slices — all 16 viable killed, both `<`→`<=`
  boundary pairs + the last `>`→`>=` via the zero-width-selection case),
  T-cb-1 (cols_to_bytes — zero viable mutants [no `Range` Default]; guard
  tests pin multibyte/clamp/inverted behavior for coverage).
- **New headless (REQ-002 model half):**
  `editor_opens_a_120_line_file_whole_headless` — opens a 120-line fixture
  through `open_file_in_viewer`, asserts the editor tab active +
  `len_lines()==121` (ropey counts the final-\n tail line): the buffer holds
  the WHOLE file; the old `VIEWER_ROWS=40` window is not a truncation
  anywhere in the model. Runs green in the #264 lane.
- **Full suite:** `cargo nextest run --workspace` → **915/915 passed** (5
  skipped) — +1 for the new headless test.
- **Driven captures (bundled fresh binary, all PNGs read):**
  - `266-1`/`266-2` — the swapped render live: per-row StyledText syntax
    tints, the caret bar, and the #265 ⌘D word selection painted as the
    with_highlights 28%-accent background (REQ-003/004/005 pixels).
  - `266-4` — `crates/editor/src/movement.rs` (317 lines) opened from the
    file tree and wheel-scrolled to rows **152–198**: the 40-row cap is
    GONE and the real viewport picks the slice (REQ-002 pixels + the
    REQ-007 wheel half; three `scrollat` wheel batches drove the handle).
  - `266-5` — a click mid-line-168 placed the caret inside `co(5)` at the
    clicked glyph (REQ-006).
  - `266-7` — palette → "Split Right → File" → movement.rs: the #246
    read-only pane renders rows 1–40 through its untouched path (its
    pre-existing 40-row window + per-span divs), right of a live terminal
    (REQ-007 read-only half).
  - Drive lesson recorded: the ⌘P finder's plain **Enter INSERTS the path
    into the prompt** (the #65 semantic — two stray insertions are visible
    in the capture `266-6a`'s prompt line); **⌘↵** is the open-in-viewer
    chord, and drive.swift's `cmd:` verb takes single chars only — the
    deterministic driven route to open a file is a file-tree click.
- **Gate:** first `--diff` run RED — rustfmt (the fresh test block) +
  clippy `reversed_empty_ranges` on two DELIBERATE inverted-range test
  literals (`7..3`, `10..1`). Fixed at source (no suppression): the
  inverted ranges are the test DATA, so they're spelled as
  `std::ops::Range { start, end }` struct literals, which clippy correctly
  ignores. Re-run: **GATE GREEN [diff] — 15/15** (coverage 100, MSI 100,
  miri, visual all green; receipt written).

## Phase 5 — Complete
- (pending)
