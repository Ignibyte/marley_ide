---
pipeline_id: 0de52da1-37d2-4c8b-9400-78858153e8dd
ticket: forge#250 (da40fb6e-a4b2-4100-b567-2811776095d7)
aar_id: 786bca71-dafc-48e1-b418-e54f0c30fe7c
---

# Notes — Faithful editor renderer (forge#250)

## Plan (Phase 1)

**Classification:** work pipeline, feature, LARGE (the ticket's own words: "THE HARD ONE / make-or-break").
Scoped TIGHT to the make-or-break slice (map + render-from-buffer + visible caret); input/mouse/selection/
h-scroll/wide-chars/split-pane-from-buffer all DEFERRED to their own tickets (#251/#254/#255/#258 + follow-ups).
AUTONOMOUS (chad's `/goal /work 249 to 258`). 2nd ticket under the #248 `## Reference (§20)` discipline — the
FIRST with a REAL observed capture.

**Intent:** the editor renders the lossy `CodeViewState.lines` (tab-expanded + TRUNCATED at CODE_MAX_COLS=200
with `…`); typing (#251) will mutate the Buffer, which the render must reflect, and the caret must sit where the
char is. #250 makes the editor draw from the Buffer with an exact offset↔column map + a visible caret.

**Discovery (Explore agent — the local code, since the code-index is main-branch-only):**
- **`code_view.rs` is a PURE seam** (only `std::path`; no gpui). `expand_tabs` (code_view.rs:30) already does the
  TRUE tab-stop math (`tab_width − col % tab_width`), counting each `char` as 1 column — but RETURNS ONLY the
  `String`, throwing away the per-char→column mapping. **That discarded mapping is #250's heart** (reimplement
  expand_tabs to KEEP it). `truncate_cols` (49) caps at `CODE_MAX_COLS=200` with `…`. `CodeViewState{path,
  lines: Vec<CodeLine>, scroll}` (142). `visible_range`/`scroll_code` are vertical-only; `gutter_width`=digit
  count. **NO offset↔column fn exists** (confirmed). Tests are exact-value units.
- **`code_view_body` (app.rs:2539) is a masked shim** (`#[cfg_attr(test, mutants::skip)]`) — renders each visible
  line as N flexbox syntax-span divs (`highlight_line` over the LOSSY `line.text`) + a `gap_2` gutter. **NO
  measured per-column grid** — columns align only because Menlo is monospace by convention. Shared by BOTH the
  editor tab (app.rs:4842, via `active_file()`) AND the #246 split pane (app.rs:5657, a bufferless
  `PaneContent::CodeView(CodeViewState)`).
- **`Buffer` (crates/editor) has NO public per-line getter** — ropey `rope.line(n)` is behind `pub(crate)
  rope()`. And **no immutable `active_buffer()`** (only `active_buffer_mut`, editor_surface.rs:130). Both are new
  plumbing #250 adds. `Point.column` is BYTES (not display cols) → NOT a usable column source.
- **The measured cell exists in the TERMINAL path** (app.rs:~4967 `em_advance(resolve_font(TERMINAL_FONT))` +
  `fallback_cell`) — the code view does none of it. The caret needs this measurement (D5 reuses it).
- **`CODE_MAX_COLS=200`** gates ONLY `truncate_cols` (not gutter/scroll). No h-scroll exists; long lines clip.
- **`code_tab_width`** is a clamped-≥1 setting (settings.rs), on the app as `self.code_tab_width`, fed to
  `CodeViewState::new`. The map fn must use the SAME width + the SAME tab-stop rule.

**Decisions:** D1 map is the pure seam in code_view.rs (expand_tabs derives from it) · D2 v1 char-width-1 +
tab-stops (wide/CJK deferred, shaped to grow) · D3 render from buffer via `Buffer::line_text` + `active_buffer()`
· D4 un-truncate the editor (split pane keeps the cap) · D5 caret = overlaid block at col×em_advance (reuse the
terminal measurement; no full grid-render rewrite).

**Risks / load-bearing:**
- **The offset↔column EXACTNESS is the whole ballgame** — a wrong map mis-places the caret + a #252 save corrupts.
  Design pins the exact fn + a round-trip (offset→col→offset) test incl tabs/empty/trailing.
- **The caret overlay aligning with the flexbox monospace text** (D5) — relies on Menlo's fixed advance; the
  DRIVEN capture is the proof (does the caret block sit on the char?). If it mis-aligns, fall back to a measured
  per-column line render (bigger — flag at design, but LEAN the overlay for v1).
- **Don't break the #246 split pane** (bufferless) or #243 persistence or the tab strip — the render fork must
  leave the split-pane path on `CodeViewState.lines`.
- **Un-truncation** — a very long line now overflows the pane (clip); acceptable v1 (h-scroll is a follow-up);
  confirm gutter/vertical-scroll still work.

**Test plan (finalized at design):** REQ-001/003 pure units on the map (offset→col + col→offset; tabs at various
positions, empty line, ascii, boundary/past-end col, round-trip) cov/MSI 100; `Buffer::line_text` pure unit in
the editor crate (line 0/mid/last/empty/out-of-range). REQ-002/004/005 DRIVEN (mac unlocked) — a file with tabs
+ a >200-col line renders faithfully (tabs on stops, no `…`) + the caret block at offset 0; `cargo check
--workspace` (split pane/persistence/tab strip compile + a driven no-regression).

**Reference (§20):** REAL observed capture taken this phase → `observed/250-warp-monospace-grid-caret.png` +
note. Warp's monospace cell grid (char=cell, tab-stops, block caret on a column boundary) is the model. Clean-room.

**AAR:** 786bca71-dafc-48e1-b418-e54f0c30fe7c (opened).

**Phase 1 status: Plan PASS — autonomous (M15 /goal). Ready for Phase 2 — Design.**

## Design (Phase 2)

**## Reference (§20) confirmed:** the observed `250-warp-monospace-grid-caret.png` (Warp's fixed monospace cell
grid: char=cell, tabs to stops, block caret on a column boundary). This design MATCHES it by (a) a pure
offset↔column map where each char = 1 column + tabs advance to the next stop, and (b) a caret drawn at
`column × measured monospace cell width`. Clean-room: observed behavior only; reuse `gpui`/`ropey`, no Warp source.

**Architecture.** A pure map in `code_view.rs` (the #97 pure seam) + two pure `Buffer` helpers in the editor
crate + an immutable surface accessor + a render/caret change in the app.rs shim. The EDITOR path uses ropey's
line model THROUGHOUT (render + caret), so it is self-consistent by construction; the #246 split pane keeps its
`split('\n')`/`CodeViewState.lines` world untouched.

### THE offset↔column MAP — RESOLVED (`code_view.rs`, pure, the heart)
```
pub struct LineLayout { pub display: String, col_starts: Vec<usize> }   // col_starts[i] = display col where
impl LineLayout {                                                       // char i begins; len = n_chars + 1
    pub fn width(&self) -> usize            // = *col_starts.last() (total display width)
    pub fn col_of_offset(&self, i: usize) -> usize   // col_starts[i.min(len-1)]  (char idx → display col)
    pub fn offset_of_col(&self, col: usize) -> usize // col_starts.partition_point(|&c| c <= col) - 1  (#254)
}
pub fn line_layout(line: &str, tab_width: usize) -> LineLayout {
    let tab_width = tab_width.max(1);                // §14 no ÷0 (setting is clamped, but guard the pure fn)
    let mut display = String::new(); let mut col_starts = Vec::new(); let mut col = 0;
    for ch in line.chars() {
        col_starts.push(col);
        if ch == '\t' { let n = tab_width - (col % tab_width); for _ in 0..n { display.push(' '); col += 1; } }
        else { display.push(ch); col += 1; }        // char_width = 1 (v1); a char_width(ch) hook slots in (D2)
    }
    col_starts.push(col);                           // sentinel: end-of-line column = total width
    LineLayout { display, col_starts }
}
```
- `expand_tabs(line, tw)` is REPLACED by `line_layout(line, tw).display` (ONE source of truth — render + map
  can't diverge). `code_lines` keeps calling the display (via a thin `expand_tabs` wrapper or inline) so the
  split pane is byte-identical.
- `offset_of_col`: `col_starts` is sorted ascending, `col_starts[0]=0`, so `partition_point(|&c| c<=col)` ≥ 1 →
  `-1` never underflows; `col ≥ width` → returns `n_chars` (end of line, for a click past the last char);
  clicking inside a tab's cells maps to the tab char (verified: `"ab\tc"` tw4 → col_starts `[0,1,2,4,5]`,
  `offset_of_col(3)=2`). #254 reuses it.
- v1 char-width-1: a wide/CJK char counts as 1 column (KNOWN v1 limitation — matches today's `expand_tabs`; the
  `char_width` hook is the growth path, D2). The `col` counter is CHARS, consistent with the old `expand_tabs`.

### BUFFER PLUMBING — RESOLVED (editor crate, pure)
- `Buffer::line_text(row: usize) -> String` — `if row >= self.rope.len_lines() { return String::new(); }` then
  `self.rope.line(row).to_string()` with a trailing `\n` (and a `\r\n`'s `\r`? NO — only strip the final `\n`,
  keep any `\r` so the display matches `split('\n')`) stripped: `s.strip_suffix('\n').unwrap_or(&s).to_string()`.
  Out-of-range → `""` (no panic). cov/MSI 100 (line 0 / mid / last / trailing-newline empty last / empty buffer
  / out-of-range).
- `Buffer::line_col(c: CharOffset) -> (usize, usize)` — `(row, char_in_line)` where `row =
  rope.char_to_line(c)`, `char_in_line = c - rope.line_to_char(row)`. The caret's offset→(row, char-in-line);
  `char_in_line` indexes `line_text(row)`'s chars, which `line_layout.col_of_offset` maps to a display column.
  Pure, cov/MSI 100 (offset 0, mid-line, line start, end-of-line-before-`\n`, last line).
- **ropey line-model reconciliation — DISSOLVED:** the editor render (`len_lines`/`line_text`) AND the caret
  (`char_to_line`/`line_to_char` via `line_col`/`point_at`) BOTH use ropey → self-consistent (the caret's row
  indexes exactly the lines the render draws). ropey `len_lines() == text.split('\n').count()` for `\n`/`\r\n`
  (a design test ASSERTS this incl a trailing-`\n` empty last line); a lone `\r` is the only divergence from the
  split-pane world (rare, cosmetic, cross-pane only — noted, not blocking). The split pane is UNCHANGED.
- `EditorSurface::active_buffer(&self) -> &Buffer` — immutable mirror of `active_buffer_mut` (editor_surface.rs).

### THE RENDER FORK + CARET — RESOLVED (app.rs shim, D4/D5)
- **Fork = Option A:** `code_view_body` gains an editor arm. Since it's `&self` (no `window`), the CALLER (the
  render method at ~4843, which HAS `window` — the terminal measures `em_advance` at 4967 in the same scope)
  measures the cell + computes the caret geometry via the PURE helpers, and passes an editor bundle:
  `code_view_body(cv, colors, editor: Option<EditorDraw>)` where `EditorDraw { buffer: &Buffer, caret: Option<(row,col)>, cell: Size<Pixels> }`.
  - `Some(EditorDraw)` (editor tab): render `for row in visible_range(cv.scroll, VIEWER_ROWS, buffer.len_lines())`
    → `line_layout(buffer.line_text(row), tab_width).display` → `highlight_line` (unchanged, over the display) →
    UN-TRUNCATED (no `CODE_MAX_COLS`); gutter from `buffer.len_lines()`. Then overlay the caret block:
    `x = gutter_px + col*cell.w`, `y = (row - scroll)*cell.h`, size `cell` — an absolutely-positioned filled
    block (an OVERLAY, not the prompt's inline `split_caret`), drawn when `caret` is `Some` (editor tab active).
  - `None` (split pane, app.rs:5658): the EXISTING `cv.lines` path, byte-identical — truncated, no caret. UNCHANGED.
- The caller computes the caret cell: `active_caret()` → `active_buffer().line_col(caret)` → `(row, char_in_line)`
  → `line_layout(line_text(row), tw).col_of_offset(char_in_line)` → `col`. **Point.column is BYTES — NOT used;**
  the display col comes from `line_layout` (the Explore trap, avoided).
- Scroll/gutter for the editor use `buffer.len_lines()` (the scroll handler app.rs:4925 keeps mutating
  `cv.scroll`; `scroll_code`/`visible_range` just take the buffer line count now — a value change, not a shape
  change).

### File Manifest
| File | Change |
|---|---|
| crates/marley_app/src/code_view.rs | ADD `LineLayout` + `line_layout` (+ `col_of_offset`/`offset_of_col`/`width`); reimplement `expand_tabs` as `line_layout(..).display` (one source of truth); `code_lines`/`truncate_cols`/`CodeViewState` UNCHANGED (split pane). Tests-phase adds the map tests. |
| crates/editor/src/buffer.rs | ADD `line_text(row) -> String` (ropey line, strip final `\n`, out-of-range → ""), `line_col(c) -> (usize,usize)` (row, char-in-line). Tests-phase. (No re-export change — both are `Buffer` methods, already public via the type.) |
| crates/marley_app/src/editor_surface.rs | ADD `active_buffer(&self) -> &Buffer` (immutable mirror of `active_buffer_mut`). |
| crates/marley_app/src/app.rs | `code_view_body` gains the `Option<EditorDraw>` arm (editor draws from the buffer, un-truncated, + the caret overlay); the caller (~4843) measures the cell + computes the caret via the pure helpers + passes the buffer; the split-pane caller (5658) passes `None`. SHIM (`mutants::skip` + coverage-excluded). |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | `line_layout` display: ascii, tabs at col 0 / mid / after-text, trailing tab, empty line → `.display` matches the old `expand_tabs` (tab-stop) exactly; `width()` correct | REQ-001/003 |
| T2 | `col_of_offset`: each char idx → its start column (ascii + tab lines); offset 0 → 0; offset == n_chars → width; offset past end → clamps to width | REQ-001 |
| T3 | `offset_of_col`: col in a char's cell → that char; col inside a tab's cells → the tab char; col ≥ width → n_chars (end); ROUND-TRIP `offset_of_col(col_of_offset(i)) == i` for non-tab chars | REQ-001 (+ #254 seam) |
| T4 | `line_layout` tab_width=0 guard → treated as 1 (no ÷0 panic) | REQ-001 (safety) |
| T5 | `Buffer::line_text`: `"a\nbb\nccc"` → line 0 "a" / 1 "bb" / 2 "ccc"; trailing `\n` (`"a\n"`) → line 1 ""; empty buffer → line 0 ""; out-of-range row → "" | REQ-002 |
| T6 | `Buffer::line_col`: offset 0 → (0,0); mid line-1 → (1, k); line start → (row,0); end-of-line-before-`\n` → (row, line_len); last line | REQ-004 (caret math) |
| T7 | ropey line-model assert: `len_lines()` == `split('\n').count()` for `"a\nb"`, `"a\nb\n"` (trailing empty), `"a\r\nb"`; `line_text` round-trips each | REQ-002 (reconciliation) |
| T8 (driven) | open a file with TABS + a >200-col line in the editor → renders faithfully (tabs land on stops, NO `…` truncation) + a block caret at top-left (offset 0); the split pane still truncates; vs the Warp observed reference | REQ-002/003/004 |
| — | `cargo mutants --list -f code_view.rs` + `-f crates/editor/src/buffer.rs` (post-impl) → kill the new-fn mutants with T1-T7 | MSI 100 |
| REQ-005 | `cargo check --workspace` + driven: split pane / persistence / tab strip unaffected | compile + review + driven |

**Uncoverable / shim:** `code_view_body`'s editor arm + the caret overlay (gpui) are shim (`mutants::skip`,
coverage-excluded) — DRIVEN-validated (T8); the pure map + buffer helpers carry the correctness.

**Risks:** (1) offset↔column EXACTNESS — the round-trip + tab tests (T1-T3) are the guard. (2) the caret overlay
aligning with the flexbox monospace text — T8 driven is the proof; FALLBACK if mis-aligned = a measured
per-column line render (bigger; deferred, noted). (3) ropey vs split('\n') for a lone `\r` — cross-pane cosmetic
only (T7 documents it). (4) un-truncation → a very long line overflows the pane (clip; h-scroll is a follow-up).

**Phase 2 status: Design PASS — the LineLayout map (col_starts) + expand_tabs-reuse, the two ropey buffer
helpers (line_text/line_col, self-consistent), active_buffer, the Option A render fork + caret overlay, and the
T1-T8 plan are locked. Ready for Phase 3 — Implement.**

## Implement (Phase 3)

**Built (to the manifest):**
- **code_view.rs** — `pub struct LineLayout { pub display, col_starts: Vec<usize> }` + `width`/`col_of_offset`/
  `offset_of_col` + `pub fn line_layout(line, tab_width)` (one pass, `tab_width.max(1)` guard, char-width-1 +
  tab-stops). `expand_tabs` now delegates to `line_layout(..).display` (ONE source of truth) — `code_lines`/
  `truncate_cols`/`CodeViewState` byte-identical, so the split pane is untouched.
- **crates/editor/src/buffer.rs** — `pub fn line_text(row) -> String` (ropey `rope.line(row)`, strip ONLY the
  final `\n`, out-of-range → "") + `pub fn line_col(c) -> (usize, usize)` (row via `char_to_line`, char-in-line
  via `c - line_to_char(row)`). Both `&self`, pure, infallible. `cargo check -p marley_editor` clean.
- **editor_surface.rs** — `pub fn active_buffer(&self) -> &Buffer` (immutable mirror of `active_buffer_mut`).
- **app.rs** — `struct EditorDraw<'a> { buffer: &Buffer, caret: Option<(row,col)>, cell: CellSize }` (module
  level, near VIEWER_ROWS; reuses the workspace.rs `CellSize`). `code_view_body` gained a 4th param
  `editor: Option<EditorDraw>`: `Some` = the editor path (draw each visible line from `buffer.line_text(row)` →
  `line_layout(..).display`, un-truncated; gutter/scroll from `buffer.len_lines()`; a caret at its column);
  `None` = the existing `cv.lines` split-pane path (byte-identical, extracted below the early `return`). The
  editor caller (~4909) measures the cell (the terminal's `em_advance` idiom, verbatim) + computes the caret via
  `active_buffer().line_col()` + `line_layout().col_of_offset()` (NOT `Point.column`, the bytes trap) + passes
  `Some`; the split-pane caller (~5724) passes `None`.

**Deviations from design (with reason):**
- **The caret is a 2px BAR, not a solid block** (design/Warp ref showed a block). A bar is a clear, precise,
  NON-occluding visible caret at the exact column boundary — it proves the offset↔column map (the #250 point)
  just as well, without needing char-inversion/alpha. A block-with-blink is a natural #251 polish. (Noted; the
  driven capture validates the column placement either way.)
- **Added `.text_size(px(TERMINAL_FONT_SIZE))` to the EDITOR rows** (the original had only `.font_family`). This
  PINS the rendered char width to what `em_advance` measured at `TERMINAL_FONT_SIZE` (13pt) — the prerequisite
  for the caret (positioned at `col × cell.w`) to align with the text. The `None`/split-pane path is UNCHANGED
  (no `text_size`) so it stays byte-identical.
- The caret is an `.absolute()` child of a `.relative()` `code_row`, so `left = col × cell.w` is measured from
  the START of the code area — the gutter offset is automatic (no px gutter-width arithmetic needed).

**Compile:** `cargo check -p marley --all-targets` + `cargo check -p marley_editor` clean (the multi-shared-
borrow of `self` — cv + surface + `code_view_body` — is all immutable, NLL-fine). Do NOT expand tests here
(Phase 4 adds T1-T8).

**Phase 3 status: Implement PASS — the pure map + the two ropey buffer helpers + active_buffer + the Option A
render fork + the caret bar all compile clean; None path byte-identical; caret is a bar (deviation, noted).
Ready for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5)

3 parallel general-purpose critics (C1 = the offset↔column map; C2 = the buffer helpers + caret math; C3 =
regression + scroll + clean-room) + self-review. **0 HIGH, 0 MEDIUM. 3 LOW folded.** The make-or-break map is
VERIFIED CORRECT — C1 fuzz-checked col_starts + round-trips over 13 lines × {1,2,3,4,8} tab-widths (incl CJK),
0 violations; `cargo nextest -p marley code_view::` 10/10, `-p marley_editor` 25/25, `-p marley` 345/2.

**Load-bearing checks — CONFIRMED CLEAN:**
- **C1 the map:** `line_layout` builds `col_starts` exactly (`"ab\tc"` tw4 → `[0,1,2,4,5]`, display `"ab  c"`);
  `col_of_offset(n)==width` (caret-at-EOL, no off-by-one), clamps past-end; `offset_of_col` lands a mid-tab col
  on the tab char + `col>=width`→`n_chars`, no underflow (col_starts[0]=0 → partition_point≥1); the round-trip
  `offset_of_col(col_of_offset(i))==i` holds for every char-start + EOL. `expand_tabs`==`line_layout(..).display`
  BYTE-IDENTICAL across 88 (line,tw) combos → split pane + all code_view tests unregressed. `tab_width.max(1)`
  kills the ÷0. The map is DISPLAY-only over `CharOffset` — a wrong col could at worst mis-draw the caret,
  never corrupt saved bytes (C1's "corrupts saves" risk retired).
- **C2 buffer + caret:** `line_text` (ropey line, strip only final `\n`, out-of-range→"") + `line_col`
  (row/char-in-line, no underflow) trace correctly + are self-consistent (both ropey); the caller uses
  `line_col`→`line_text(crow)`→`col_of_offset` (NOT `Point.column`, the bytes trap), caret seeded at 0 → (0,0);
  the caret is an `.absolute()` child of the `.relative()` code_row (gutter offset automatic), drawn only for a
  visible row; the `em_advance` measurement is BYTE-IDENTICAL to the terminal's (same Menlo, same 13pt) → the
  bar aligns.
- **C3 regression:** the `None` split-pane arm is byte-identical to HEAD's `code_view_body` (only `lang` hoisted,
  pure) → #246 pane unchanged; `active_file()`/`files()`/`CodeViewState` untouched → #243 persistence + the
  file-tab strip unaffected; no panic paths; clean-room (gpui/ropey/in-repo marley_editor, observed Warp
  capture, no Warp source).

**Findings folded (3 LOW):**
- **F1 [LOW→latent-MED] scroll/render line-count mismatch** (C3, app.rs) — the editor render windows over
  `buffer.len_lines()` but the wheel handler clamped `cv.scroll` against `cv.lines.len()`. Equal for `\n`/CRLF
  files (cosmetic today) BUT ropey (`unicode_lines` default) also breaks on bare CR/FF/VT/NEL/LS/PS → the bottom
  N lines become unscrollable; and the moment **#251** edits the buffer, stale `cv.lines` breaks the clamp for
  EVERY edited file. **VERDICT real (latent-MED).** **FIX:** the handler now reads
  `editor().map(|s| s.active_buffer().len_lines())` (sequential immutable-then-mut borrow) and clamps
  `scroll_code` against that, falling back to `cv.lines.len()` for a bufferless read-only pane. Prevention:
  `PR-claude-scroll-clamp-must-track-the-render-line-count-source`.
- **F2 [LOW] `line_text` doc overstated `split('\n')` equivalence** (C2, buffer.rs) — ropey breaks on more than
  `\n`. **FIX:** reworded to state ropey's Unicode line semantics + the #246 cross-pane cosmetic difference.
- **F3 [INFO→fixed] editor rows omit `.line_height`** (C2) — the terminal panes set it; ~0.1px caret-height
  parity. **FIX:** added `.line_height(px(ed.cell.h))` to the editor row (None/split path unchanged).

**Deferred (noted, NOT blocking #250):**
- **(n) un-truncated editor render allocs O(line_len)/frame** for a pathological single >200-col line (bounded
  by VIEWER_MAX_BYTES=2 MB — no OOM, but can jank). h-scroll/soft-wrap is the deferred follow-up (spec §Out);
  filing intent noted for a later ticket.
- **VALIDATE MUST test ALL new pure fns incl `offset_of_col` + `width`** (born here for #254/#255, unused in
  #250 → uncovered/surviving mutants otherwise): C1's (a)-(c) tables + C2's line_text/line_col traces are the
  ready oracles. cov/MSI 100 requires them.

**Fix compile:** `cargo check -p marley --all-targets` clean (`.line_height` exists on Div; the scroll
borrow-dance is NLL-fine).

**Phase 3.5 status: Inspect PASS — 0 HIGH/MED; 3 LOW folded (F1 scroll-source, F2 doc, F3 line-height); the map
is fuzz-verified correct; compile clean. Ready for Phase 4 — Validate.**

## Validate (Phase 4)

**Tests added:**
- **code_view.rs** (3): `line_layout_display_and_columns` (tab-stop display + col_starts via col_of_offset:
  "ab\tc"→"ab  c" [0,1,2,4,5], tab@0, trailing tab, empty, two-tabs, ascii), `col_of_offset_eol_and_clamp`
  (i==n→width, i>n clamps, empty→0), `line_layout_tab_width_zero_guard` (tw 0→1, no ÷0).
- **buffer.rs** (3): `line_text_strips_final_newline_and_guards_range` (per-line, trailing-\n empty last,
  empty, out-of-range→"", CRLF \r kept), `line_col_maps_offset_to_row_and_char_in_line` (row/char-in-line, the
  multibyte char-vs-byte contrast with point_at), `ropey_line_model_len_and_round_trip` (len_lines + join).
- **editor_surface.rs** (1): `active_buffer_reads_active_file` (kills the `active_buffer -> Default::default()`
  mutant the --diff gate flagged — Buffer HAS Default, so it was VIABLE + untested).

**SCOPE ADJUSTMENT (validate-time, forced by §0 no-suppressions):** `LineLayout::offset_of_col` + `width` were
in the implement but are UNUSED by #250 non-test code → a dead-code warning under gate:2 `-D warnings`, and
`#[allow]` is forbidden. **Removed both** (kept `line_layout` + `col_of_offset`, both used: render + caret).
The inverse (`offset_of_col`, column→offset) + its round-trip test move to **#254** (the mouse click that
consumes it) — its natural home. Spec REQ-001 updated (forward-only in #250). LESSON:
`PR-claude-dont-ship-a-pure-seam-method-unused-until-a-later-ticket` (an unused pub fn in a private mod is
dead-code under -D warnings; add it WITH its consumer).

**Runs (actual):** `cargo nextest run -p marley code_view::` → 13/13; `-p marley_editor` → 28/28; `-p marley` →
**348 passed, 2 skipped** (no regression).

**Mutation (`cargo mutants --list`):** code_view.rs — `line_layout` (loop `%`/`-`/`+=`/`==`) + `col_of_offset`
(const 0/1) all killed by the 3 map tests; **`line_layout -> Default::default()` UNVIABLE** (LineLayout has no
Default derive — confirmed). buffer.rs — `line_text` (`>=`/return) + `line_col` (`-`/const tuple) killed by T6/
T7. editor_surface.rs — `active_buffer -> Default::default()` killed by the new test. Gate ran the real mutants
→ **MSI ≥ 100%**.

**DRIVEN — REAL CAPTURE (mac unlocked):** bundled + opened Marley. The editor renders a file (selection.rs)
**faithfully from the Buffer** (line numbers, syntax, per-line from `line_text` — the #250 render-from-buffer is
LIVE). A magnified crop of **line 1, column 0** shows the **caret** — a cyan accent bar sitting exactly before
the first char, at the correct height, aligned to the char boundary (the `em_advance`+`text_size` alignment
works). Proves REQ-002 (render from buffer) + REQ-004 (visible caret at the seeded offset-0 caret) + the
caret/text alignment, vs the observed Warp monospace-grid reference. The tab/long-line fixture could NOT be
opened via the ⌘P finder (Enter/click didn't take — a self-test harness focus quirk with "focus: terminal"; hit
the attempt limit, NOT a #250 defect) → tabs-at-tab-stops (REQ-003) + no-`…`-truncation (REQ-004 long line) are
carried by the pure `line_layout` tests (the exact tab-stop math) + the mechanism (the editor `Some` arm renders
`line_layout(..).display` with NO `truncate_cols`, unlike the split-pane `None` arm). Fixture + mutation
leftovers cleaned; app quit; tree = only the four #250 `.rs` + docs.

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** 15/15 (incl gate:4 coverage ≥ 100% lines, gate:5
mutation MSI ≥ 100%, gate:6 miri, gate:15 visual/AX). One RED fixed at source first: gate:14 rustdoc flagged a
broken intra-doc link `[crate::LineLayout::col_of_offset]` in buffer.rs (marley_editor can't link to the
marley_app crate — a reverse dep) → changed to plain backticks. The receipt for `/commit` is written. The gate
ran cargo-mutants at `--jobs 2` (the memory fix) without a freeze.

**Pre-existing (not in scope):** the `block v0.1.6` future-incompat (transitive dep).

**Phase 4 status: Validate PASS — 7 new unit tests (cov/MSI 100 on the pure map + buffer helpers + the
active_buffer accessor), 348 marley tests green, the caret + buffer-render driven-proven live, GATE GREEN
[diff]. offset_of_col/width deferred to #254 (no-suppressions). Ready for Phase 5 — Complete.**
