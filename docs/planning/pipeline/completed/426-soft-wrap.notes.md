# 426-soft-wrap — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-426-soft-wrap.md
- **Pipeline spec:** 426-soft-wrap.spec.md

## Phase 1 — Plan

- **Request:** TICKET-426 — soft wrap, second of the M32 B-c sprint (auto-approved
  directive; shelf `display-map-shelf.md`). Systems: editor render + display map
  (`marley_app`), movement (`marley_editor`), settings, the React POC.
- **Classification / tier:** feature, L (the sprint's hardest). ONE pipeline — splitting
  render from motion would ship a broken feel between them.
- **Recall (§18.3):** the 425-era pins all bind (AD-305 half-open runs; AD-331 two
  boundary maps — columns stay in `LineLayout`; AD-336 h-scroll clipper/shift + "wrap
  must revisit the shift site, not layer on it"; PR reveal-hook — every jump path;
  F-#352 row-vs-slot, now compile-checked; PR zero-width boundary probes; F-#386 cargo
  check --tests; PR-137 sizer/closure same instance; the MSI-hollowness family).
  #277 (wide-glyph columns) is the cell-width recall: `char_width`/UAX#11 is the one
  authority; clicks are float-domain (#277).
- **Discovery (Explore sweep, 2026-08-14, HEAD a68f816 — full report in the agent
  transcript; the load-bearing facts):**
  - **Motion:** ↑/↓ → `marley_editor::move_all_vertical` (app.rs:18521-18535); the goal
    column lives on `Selection.goal_col` (selection.rs:27, a CHAR column) carried ONLY
    by vertical moves (multi_cursor.rs:376-393); `move_vertical_goal`
    (movement.rs:130-138) is THE single primitive needing a display-row twin.
    `add_cursor_vertical` also carries goal (multi_cursor.rs:67-95). Home/End are
    buffer-line (movement.rs:95-111) — kept, deferral recorded. PageUp/Down: the editor
    has none (terminal-only) — nothing to do.
  - **Render:** the rim loop's items 3–13 (layout, syntax, hints, selection, find,
    bracket, merge, StyledText, geom canvas, carets, squiggles) ALL index ONE
    whole-line `LineLayout` display byte space (app.rs:6727-7227 enumerated per-item in
    the report) → the minimum-blast-radius shape is a pure slicer: (LineLayout + break
    cols) → per-segment (byte range, cell offset), clip-and-re-base every producer.
  - **The EOL ⋯ phantom** is an `Inlay{char_idx: n_chars}` merged into the ONE layout
    (app.rs:6666-6681) → lands naturally on the LAST segment; the #331 EOL asymmetry
    (`col_starts` stops before an EOL phantom; `display_cols` counts it) must be pinned
    in the break math.
  - **Sticky band:** pure `sticky_rows` takes a buffer row; the band renders ONE row
    per header at fixed cell_h with its own shift site (app.rs:7303-7409). Under wrap:
    pass the first visible segment's buffer row; the partially-scrolled-header edge
    needs a pinned test; band shift is 0 when wrap ON (scroll_x pinned).
  - **h-scroll inert set** (each currently ungated): wheel-x (app.rs:6601-6620),
    content_px write (:6698-6719), h_scroll_clamp (:16505), follow_editor_caret_x/_to
    (:16518-16550), thumb (:7234-7268), park/restore (:16799-16821). Shift sites:
    :6922 (code row) + :7402 (sticky).
  - **Settings pattern** (six edits, no registration): define_setting! →
    AppliedSettings field → defaults arm → from arm → persist fn → RootView field+seed;
    toggle precedent `toggle_sticky_header` (app.rs:4919-4927, nulls its cache +
    persists); palette: CommandId(33) is next free; `action_for_command` table row +
    `dispatch_action` verb; the command-table guard test exists (app.rs:22567-22580).
    Settings are boot-loaded, write-through, NO file watcher — in-app toggle is the
    live path (the #337 model: field + persist + cx.notify; render reads the field).
  - **The chicken/egg:** wrap width = floor(geom.code_w / cell_w) reads the PREVIOUS
    frame's probe (exactly content_px's standing pattern) — a resize re-wraps next
    frame; documented, not a defect.
  - **Read-only viewer** (app.rs:7270-7293): fully disjoint (no projection/scroll_x/
    geom; pre-truncated via `code_lines`) — OUT stands.
  - **POC:** EditorView.tsx one-div-per-line; `whitespace-pre` + `overflow-x-auto` =
    the Rust posture; the code box rect is probed at :136-154 with `.width` DISCARDED —
    the wrap hook; effect deps miss resize → needs ResizeObserver; caret math
    `caret.row * cell.h` (:282-291) and click inverse (:160-175) are the
    one-row-per-line twins to make segment-aware; no cell-metric util exists →
    `utils/wrap.ts` is the natural home. SplitTerminalView file cells are a second,
    metric-hard-coded copy — OUT with reason.
- **Prior art:** gpui `LineWrapper` read (registry; Apache) — RULES adopted (greedy,
  word-boundary candidates, the CJK any-break clause, hard-break fallback, capped
  continuation indent), MACHINERY rejected (proportional-px vs Marley's mono cells) —
  the "adopt LineWrapper?" fork dissolved. In-tree: `LineLayout`/`display_cols`/
  `cols_to_bytes` carry every mapping the slicer needs.
- **Decisions:** D1–D7 in the spec.
- **Auto-approval:** sprint directive; Phase-1 confirmation covered (recorded per §15).

## Phase 2 — Design

### Architecture (§20 confirmed — Zed behavior at Marley's substrate; no Zed source)

**The pure wrap seam — NEW `crates/marley_app/src/wrap.rs`** (the code_view/h_scroll
precedent; cov/MSI 100):
- `wrap_breaks(layout: &LineLayout, wrap_cols: usize) -> LineSegs` — the gpui-rules
  break computation in display CELLS: greedy accumulate over the display string's cells;
  a word char after a space marks the last candidate; ANY non-word char is a candidate
  (CJK clause); overflow → break at last candidate else hard-break; `wrap_cols == 0` or
  fits → one segment. `LineSegs { starts: Vec<u32> /* cell starts, [0, …] */, indent: u32 }`
  — continuation capacity = `wrap_cols - indent` where
  `indent = min(leading_ws_cells, wrap_cols.saturating_sub(MIN_TAIL))`, `MIN_TAIL = 8`
  (a continuation always keeps ≥8 content cells; the cap is Marley's, gpui's 256 noted).
  The #331 EOL asymmetry pinned HERE: breaks are computed over `display_cols()` space
  (phantoms wrap like text).
- Segment slicing helpers over (LineLayout, LineSegs): cell-range → display BYTE range
  (delegating to the existing `cols_to_bytes`), cell → segment index, per-segment
  clip+re-base of `(start_byte, end_byte)` producer ranges, char↔(seg, content-cell)
  both ways (via `col_starts` partition_point). Pure, exhaustively unit-tested.
- `WrapLayout { wrap_cols: usize, lines: Vec<LineSegs> }` for the WHOLE buffer, with the
  arithmetic pre-filter: a line with `2*len_chars + phantom_cells(row) <= wrap_cols`
  skips the walk (1 segment, no LineLayout built) — wide glyphs are ≤2 cells/char, so
  the bound is sound; only genuinely-long lines pay the walk.

**The facade — `display_map.rs` grows the wrap stage:**
- `DisplayMap { folds, wrap: Option<Arc<WrapLayout>>, slots: Option<Arc<SlotIndex>> }`;
  `SlotIndex` = cumulative segment counts over VISIBLE (unfolded) lines — binary-search
  `locate(slot) -> (BufferRow, u16 seg)`; `slot_of(row)` = the line's FIRST segment
  (wrap-aware); NEW `slot_at(row, cell: u32)` = the segment containing that cell; NEW
  `viewport_offset_at(row, cell, first, last)` (the caret/anchor-segment twin of
  `viewport_offset`, which keeps first-segment semantics). Arc-cloned per call — the
  memo stays cheap. OFF (`wrap: None`): every method degenerates to the 425 shapes
  (REQ-002 structurally).
- **The builder `display_map()`** grows the memo key to
  `(nonce, version, anchors, wrap_cols, phantom_cells)` where `wrap_cols =
  floor(geom.code_w / cell_w)` (0 on the first unprobed frame → unwrapped frame one,
  wraps frame two — the `content_px` frame-late precedent, pinned by test) and
  `phantom_cells: Vec<(row, cells)>` comes from ONE shared assembly helper (below).
  `soft_wrap == false` → wrap stage skipped entirely, memo key unchanged from 425.
- **Phantom single-assembly (kills the drift class):** extract the render pre-block's
  einlay construction into `fn phantom_inlays_for_active(&self, map_folds) ->
  HashMap<usize, Vec<Inlay>>` used by BOTH the render (layout building, unchanged
  behavior) and the builder (summed cell widths per row for the pre-filter + break
  input). One source; no builder-vs-render disagreement possible.

**The rim under wrap** (app.rs, the one render path — editor tab + focused editable
pane): `let (row, seg) = map.locate(slot)`; build the whole-line layout as today; when
`segs.len() == 1` the path IS today's path (identity by construction); else slice: the
StyledText renders `display[b0..b1]` with highlights clipped/re-based; continuation
rows prepend `indent` blank cells (a `div` spacer — cheaper than padding the string);
caret bars render on the caret's segment at `indent + (ccol - seg_start)`; selection/
find/bracket/squiggle ranges clip per segment (a shared `clip_to_segment` helper);
gutter label on seg 0 only (empty same-width cell after); fold chevron seg 0 only; git
lane + diagnostic tint on ALL segments (the VS Code convention); the ⋯ phantom falls on
the LAST segment naturally (it lives at the display tail). Click/drag inverse: `rel`
→ content cell = `seg_start + (cell - indent).clamp(0, seg_span)` → `offset_for_click`
on the whole line with the segment's cell offset added — indent-zone clicks clamp to
segment start. Geom recorder/`first`/`last` unchanged (slot domain — 425's typing
makes every consumer correct through the facade automatically).

**Motion (D6 resolved):** `marley_editor::multi_cursor` gains
`move_all_vertical_by(set, buffer, extend, step: impl Fn(CharOffset, Option<usize>) ->
(CharOffset, usize))` — the generic carry: per selection `let (target, goal') =
step(caret, sel.goal_col())`, then the existing `as_pair`/`extend_or_go`/`from_pair`
`.with_goal(Some(goal'))`/`from_selections` mechanics (the multi-cursor invariant doc
carries over verbatim). `move_all_vertical` re-expresses over it with the classic step
(behavioral no-op, suite-proven). The APP owns the display step (pure fn in wrap.rs
given buffer + WrapLayout + layout access): goal = CONTENT cell within segment;
up/down = previous/next display row (prev/next segment, else adjacent line's last/first
segment); target char = the char whose cell contains `seg_start + goal` clamped to the
segment. The Key::Up/Down arm branches: wrap-ON → `move_all_vertical_by(display_step)`,
OFF → the classic call (byte-identical). `toggle_soft_wrap` rebuilds the selection set
through `from_pair` (goal reset — a stale char-goal must not be read as a cell-goal;
one-keystroke artifact prevented, pinned by test).

**Follow vs jump:** `scroll_editor_to_row` (the caret-FOLLOW path via
`follow_editor_caret`) becomes segment-aware — `slot_at(row, caret_cell)` — so typing
at the end of a 6-segment line keeps the CARET visible; the named JUMP helpers
(`reveal_and_scroll_to_row` and the goto/def/problems verbs) keep first-segment
targeting (REQ-008's "lands the FIRST segment"). Auto-reveal (PR-#305) is untouched —
`reveal_caret_row` runs before any scroll either way.

**h-scroll inert set (D5):** one predicate `fn wrap_active(&self) -> bool` (field ON +
an editor present). Gates: the wheel-x arm early-returns; the `content_px` write is
skipped (and `editor_content_w` set = `code_w` once at toggle so `h_thumb` returns
`None` structurally); `follow_editor_caret_x/_to` early-return; `h_scroll_clamp`
returns 0; park/restore parks 0 when ON; `toggle_soft_wrap` sets `scroll_x = 0`. The
#336 clipper/shift STRUCTURE stays for OFF (the AD's "revisit, not remove": with wrap
ON the shift is always 0 and `whitespace_nowrap` is retained per-segment — a segment
never exceeds the viewport by construction, so nothing clips).

**Settings/palette:** the six-edit pattern — `define_setting!(SoftWrap: bool = false,
"editor.soft_wrap")`; `AppliedSettings.soft_wrap`; defaults + from arms;
`persist_soft_wrap`; `RootView.soft_wrap` seeded at boot; `fn toggle_soft_wrap` (flip,
persist, null the display-map memo, `scroll_x = 0`, goal-reset the selection set,
`cx.notify` at the dispatch site); `CommandId(33)` "Toggle Soft Wrap" (palette-only,
`binding: None`) + `action_for_command` row + `dispatch_action` verb — the command-table
guard test covers the wiring.

### File manifest

**POC first (implement order):**
1. `marley-web/artifacts/marley-ide/src/utils/wrap.ts` — ADD: `wrapBreaks(line, cols)`
   (the same rules; ASCII-width fixtures) + segment helpers.
2. `marley-web/artifacts/marley-ide/src/components/EditorView.tsx` — MOD: measure
   `box.width` (kept today, discarded) + a ResizeObserver; `softWrap` state (default ON
   for the demo); segment rendering (gutter number on seg 0 only, continuation indent);
   caret/click/anchors display-row aware. Typecheck green; screenshot ON + OFF; READ.

**Rust:**
3. `crates/marley_app/src/wrap.rs` — ADD (the pure seam + WrapLayout + helpers).
4. `crates/marley_app/src/display_map.rs` — MOD (wrap stage, SlotIndex, locate/slot_at/
   viewport_offset_at; tests extended).
5. `crates/marley_app/src/app.rs` — MOD (builder + memo key + phantom single-assembly;
   the rim; h-scroll gates; toggle + field + seed; the five cards + IME →
   `viewport_offset_at`; follow/jump split; Key::Up/Down branch; CommandId(33) wiring).
6. `crates/marley_app/src/settings.rs` — MOD (the six-edit SoftWrap pattern).
7. `crates/editor/src/multi_cursor.rs` — MOD (`move_all_vertical_by` + re-expression).
8. `crates/marley_app/src/lib.rs` — MOD (`mod wrap;`).

### Regression Test Plan (≥1 row per REQ)

| REQ | Test | Where |
|---|---|---|
| REQ-001 | `wrap_breaks` unit table (fits/exact-fit/one-over; concat-equals-display invariant) + headless: visible_count grows for a long line; rendered segments concat == unwrapped display | wrap.rs units + headless_drive |
| REQ-002 | OFF: full suite unchanged; `display_map()` memo/identity units; the seg-count-1 render path is today's (structural) + zero-width probe on the slice helpers | suite + wrap.rs units |
| REQ-003 | Break-policy table: space-separated words, CJK any-break, tabs, zero-width joins, trailing ⋯ phantom wraps, long-token hard-break, indent cap (MIN_TAIL), indent shrinks continuation capacity | wrap.rs units |
| REQ-004 | Row model: gutter label only at seg 0 (unit on the rim's row descriptor) + headless render assert (label strings per slot) | units + headless |
| REQ-005 | char↔(seg, cell) round-trip units incl. wide glyphs at boundaries; headless click drives on segment 2 + indent-zone click clamps | units + headless |
| REQ-006 | clip_to_segment units (span inside / crossing / covering); headless: a selection across a wrap boundary paints both segments | units + headless |
| REQ-007 | display step units (seg→seg, seg→next line, goal through short lines/segments, goal reset on toggle); headless ↑/↓ drives wrap ON + OFF-unchanged | units + headless |
| REQ-008 | headless: goto into a wrapped/folded line lands seg 0 + reveals; typing at a long line's tail keeps caret visible (follow → slot_at); seeded-geom card anchors at the caret's segment (`viewport_offset_at` units + the 5-card sites) | units + headless |
| REQ-009 | Gate units (wheel/clamp/follow/thumb inert when ON); headless toggle: scroll_x pinned 0, thumb absent | units + headless |
| REQ-010 | settings round-trip (persist + boot seed); headless palette toggle flips live; the command-table guard | units + headless |
| REQ-011 | fold∘wrap composition: hidden lines contribute 0 segments; ⋯ on the header's LAST segment; SlotIndex counts | display_map units |
| REQ-012 | `scripts/gates.sh --diff` green | gate |
| parity | React capture (ON, wrapped fixture) ↔ live Marley capture, SAME state; pixels sampled per MARLEY-PARITY.md; OFF pair too | validate |

Uncoverable: none new — app.rs wiring is the standing excluded shim; all new mechanisms
are pure files.

### Risks / decisions

- R1 — break-math off-by-ones at cell/char/byte junctions (+ the #331 EOL asymmetry):
  the largest unit table in the sprint; PR zero-width probes mandatory.
- R2 — OFF identity: structurally the same code path at seg-count 1; suite is the net.
- R3 — perf: the `2*chars + phantoms <= cols` pre-filter skips the walk for nearly all
  lines; memo keyed on (nonce, version, anchors, wrap_cols, phantom_cells); Arc'd
  index. A large-file wrap smoke at validate (assert no per-frame rebuild via the memo).
- R4 — goal-domain mix on toggle: goals reset at toggle (pinned by test).
- R5 — first-frame `wrap_cols == 0` renders unwrapped once: accepted (content_px
  precedent), pinned by test so it never silently changes.
- R6 — phantom drift between builder and render: killed by the single-assembly helper.
- R7 — `move_all_vertical_by` regression risk on the OFF path: `move_all_vertical`
  re-expressed over it; the existing multi-cursor suite is the net (F-#386:
  `cargo check --tests` after the signature lands).

## Phase 3 — Implement

**React-first (done FIRST):** dev server up; `utils/wrap.ts` (the break rules, ASCII
cells) + `EditorView.tsx` (box-width kept + ResizeObserver; display-row view; gutter
number on seg 0 only; caret/click/hover/rename segment-aware; `min-w-0` +
`overflow-hidden` when wrapping) + a 173-char fixture line seeded into
`docShare.ts` `code_syntax.rs`. Typecheck green. Screenshot READ
(`scratchpad/426-poc-wrap-final.png`): line 4 wraps at a word boundary onto a
continuation row with a BLANK gutter cell; renumbering correct. TWO POC findings that
shaped the Rust: (1) dropping `overflow-x-auto` alone let the flex column size to its
CONTENT (width 1448 at an 1100 window → cols always ≥ longest line → wrap never fired):
the box needs `min-w-0` — exactly the Rust clipper's `min_w_0`, independently
rediscovered; (2) the POC's persisted session shadows re-seeded docs (localStorage
merge) — reset before capture.

**Rust, to the manifest:** `wrap.rs` (LineSegs + wrap_breaks with the gpui rules on
cells, seg slicing/clip_rebase/cell_to_char/inlay_cells, display_step with injected
accessors); `display_map.rs` (WrapIndex Arc + cum over fold-visible lines; locate /
slot_at / segs_of_row / viewport_offset_at — `viewport_offset` DELETED: every consumer
moved to the `_at` form, cell 0 = the first-segment #352 semantics; tests re-pointed);
settings six-edit (`SoftWrap` default OFF) + `persist_soft_wrap`; `move_all_vertical_by`
(injected step; `move_all_vertical` re-expressed over it; `goal_of` inlined away);
app.rs — the split builder (`fold_projection_inner` + `wrap_index_for` with the
2×chars+phantoms pre-filter and the (nonce, version, anchors, wrap_cols, phantoms)
memo), `editor_phantoms` single-assembly (render + builder share it), the rim
(locate → segment slice: StyledText on `display[seg_bytes]`, clipped/re-based
syntax/selection/find/bracket, per-segment carets/squiggles, indent spacer, gutter
first-seg-only, chevron seg 0, git lane all segs), h-scroll inert set (wheel gate,
clamp→0, follow shims early-return, park/restore pins 0, content_w zeroed ON),
`toggle_soft_wrap` (memo null + scroll_x 0 + GOAL RESET via re-built SelectionSet +
persist) + CommandId(33)/verb wiring, the anchors (`anchor_slot_and_col` helper at
hover/completion/signature/rename/IME + the dwell and `character_index_for_point`
segment inverses), and `scroll_editor_to`'s follow-the-caret's-segment rule (bare row
jumps center seg 0).

**Deviations from design (with reasons):**
1. `DisplayMap::viewport_offset` deleted rather than kept alongside `_at` — after the
   anchor helper landed, it had NO production caller; keeping it would be dead code
   under -D warnings. `_at(cell=0)` IS the first-segment form; the 425 tests re-pointed.
2. `goal_of` (multi_cursor) inlined into the classic step — the generic carry left it
   without a caller.
3. The soft-wrap test hook deleted at implement per the #338/#337-F2 precedent
   (re-added at validate WITH its consuming drive).
4. `scroll_editor_to` resolves the followed caret's segment itself (design said split
   follow vs jump at the callers) — the callers all funnel here, and the
   caret-sits-on-row check makes the split automatic (a jump that placed its caret
   lands the caret's segment; a bare preview row lands seg 0). Smaller diff, same
   contract.

`cargo check --workspace --tests` clean, zero warnings; fmt applied. Tests are Phase 4.

## Inspect (Phase 3.5)

Four critics (break-math · facade/index · rim/gates · simplification+provenance+parity),
each with concrete hand-traces; plus the lead's own pre-check (which independently
flagged the tab pre-filter before the reports landed).

| # | Finding | Severity | Verdict | Fix |
|---|---|---|---|---|
| 1 | `wrap_breaks` dropped gpui's first-non-whitespace candidacy gate — an indented line broke AT its first word (bare-indent first row + over-capacity carry) | HIGH | REAL (hand-traced `"    indented …"` @12 → `[0,4,…]`) | `seen_nonspace` gate, the gpui shape |
| 2 | No re-break after a candidate rollback — carried cells + next char can exceed the SHRUNKEN continuation capacity (segment wider than the wrap width; unreachable cells since content_w=0 under ON) | MEDIUM | REAL | overflow `if` → `loop` (recompute capacity; 2nd pass hard-breaks; ≤3 iterations, total) |
| 3 | Pre-filter `2×chars` bound unsound for tabs (a char is up to `tab_width` cells) — tab-heavy long lines skipped the walk and rendered clipped with NO h-scroll (the stranded-tail class #336 killed, reintroduced) | HIGH | REAL (three critics + lead, numeric traces) | bound = `chars × max(2, tab_width)`; comment rewritten honestly |
| 4 | Unicode `is_alphanumeric` classified CJK as word chars, defeating the any-break clause | MED-LOW | REAL | ASCII word class (`is_ascii_alphanumeric() \|\| '_'`), the gpui posture |
| 5 | OFF-path regression: the `de.min(seg_c1)` squiggle clip undid the `.max(ds+1)` EOL widening — the zero-width "expected `;`" bar past EOL vanished (default config!) | HIGH | REAL | right-edge clip only on non-LAST segments |
| 6 | Phantom-BLIND columns fed phantom-AWARE segment math: the caret bar (EditorDraw), ⌘K hover writer, completion/signature/rename/IME anchors — wrong DISPLAY ROW whenever a hint precedes the anchor on a wrapped line (the F-#352 class on the column axis) | MEDIUM | REAL | carets now carry (row, CHAR) and the rim maps with ITS aware layout; `aware_display_col` helper at the five sites; `HoverCard.anchor_col` now single-domain (aware) |
| 7 | dwell + `character_index_for_point` missing the click path's segment-width clamp — pointing right of a short segment resolved the NEXT display row's chars | MEDIUM | REAL | width clamps added at both |
| 8 | `tab_width` missing from the WrapCache key (latent — no live settings reload today) + cells-only phantom key served stale breaks on anchor-moving refreshes | LOW | REAL (latent) | key = (nonce, version, anchors, wrap_cols, tab_width, (row, char_idx, cells)); residual (same-width-same-anchor text change) recorded |
| 9 | Stale comments (display_map module doc "one layer today"; rim `buffer_row` mention; three "identity when nothing folded" claims; sticky) + fmt failure + dead `too_many_arguments` armor + dead `seg_bytes` work per unwrapped row | LOW/MED | REAL | all fixed; `seg_bytes` moved into the wrapped arm |
| 10 | POC parity: wrap.ts must mirror rules 1/2/4 | — | REQUIRED | mirrored (seen-nonspace gate + re-break loop; WORD regex already ASCII); doc phrasing fixed; spec's React-first file list amended (docShare.ts fixture) |
| Accepted (recorded, not fixed) | phantom assembly runs on memo HIT + per display_map() call under ON (bounded by one file's hints; OFF pays zero) — deliberate perf debt; ⌘⌥↑/↓ add-cursor stays buffer-row under ON (REQ-007 scope is ↑/↓); goal-domain mix ⌘⌥ then ↑ steady-state (self-corrects on any horizontal); `segs_of_row` per-row clone; `seg_cell_range` past-end convention (latent, unreachable); the gpui-faithful near-empty continuation artifact (parity-identical both sides) | — | — | noted for validate + the ledger |

**Dissolved by critics (verified sound):** the WrapIndex/locate/cum math incl. fold
composition (hand-traced), saturation semantics, `Selection::new` preserves reversed
selections (the toggle's goal reset is safe), zero-width discipline end-to-end
(candidacy + cols_to_bytes + cell_to_char ties), wide glyphs never split, the h-scroll
writer set fully gated (all six writers enumerated), Up/Down OFF-path identical, POC
break parity EXECUTED over 9 cases (all match), provenance clean (rules-not-code
verified structurally; no Cargo delta).

Post-fix: `cargo check --workspace --tests` clean; facade tests 4/4; POC typecheck green;
fmt green.

## Phase 4 — Validate

**Tests added:** `wrap.rs` unit module (18 tests: the REQ-003 break-policy table incl.
the punctuation-breaks-BEFORE convention; the inspect fixes each pinned —
first-nonspace `[0,12,19,24]`, rollback-capacity sweep, CJK any-break, zero-width
candidacy theft, tabs-by-cells, trailing-phantom wrap, MIN_TAIL cap; the REQ-001
segments-tile-exactly concat invariant; seg/clip/inlay_cells conventions; the
`display_step` motion table — down-walks-segments, up-enters-last-segment,
goal-springs-back, up-within-line + segment-relative goal + width clamp, document-edge
clamps). `display_map.rs` (+2): the fold∘wrap composition hand-trace as a test
(cum/locate/slot_of/slot_at/viewport_offset_at incl. hidden-row snap and past-EOF), and
the facade edges (wrapped buffer_row, OFF slot_at, empty wrapped map, wrapped-last-line
saturation, window-first-slot Some(0)). `headless_drive.rs` (+2): the palette-verb
toggle (flag + write-through persist read at the NON-default + scroll pin) and the
display-row ↓ walk (OFF crosses lines; ON walks segments with the goal springing back —
REAL keystrokes; geometry re-seeded before each per the seed hook's no-op-draw doc).

**Runs (actual):** wrap+display_map 24/24; drives 2/2; full workspace
`cargo nextest run` → **2193/2193 passed, 7 skipped** (mid-phase) and the final gate's
gate:3 re-run green; doctests green.

**Gate:** first run 12/15 (docs: ONE brand word in a settings doc comment — reworded;
coverage: 4 uncovered facade/step arms — covered by the edge tests; mutation: 12 missed)
→ second 14/15 (mutation: 1) → **GATE GREEN [diff] 15/15**. The 12 missed mutants: 10
killed by discriminating tests (segment-relative goal signs, width clamps, up-branch
arithmetic, viewport `<`-vs-`<=` at the window start, locate's `total - 1` via a
wrapped LAST line); 2 PROVEN equivalent and re-expressed per AD-claude-305 (the
`indent()` len guard — construction invariant; the candidacy `cell > seg_start` —
implied by `seen_nonspace` + the carry theorem, now documented in-line).

**Live drive (pixels):** bundled app, window at 980×680, palette → "Toggle Soft Wrap"
(config write-through verified: `soft_wrap = true` then restored `false`). READ
`scratchpad/426-live-state.png`: the mutation-log's long lines wrap at word boundaries
onto continuation rows with BLANK gutter cells (pixel-sampled: continuation gutter ==
background rgb(14,15,17), numbered cell = the muted glyph rgb(111,115,125)); nothing
clips; line 28's toolchain path wraps twice. OFF capture + the ON→OFF live flip both
exercised through the real palette.

**Parity pair:** React `scratchpad/426-poc-wrap-final.png` (the approved design) ↔ live
`426-live-state.png`, both READ + pixel-sampled (POC continuation gutter == bg
rgb(11,12,15)). The four grammar points match 1:1: first-row-only numbering with
same-width blank continuation cells, word-boundary breaks, continuation indent, no
horizontal clipping. The POC's committed `SOFT_WRAP` const set to the SHIPPED default
(false) at close; the ON capture stands as the design record.

Pre-existing exclusions: none touched. app.rs remains the standing excluded shim.

## Phase 5 — Complete

- **§21 docs:** CHANGELOG `[Unreleased] Added` entry (the full wrap story + the inspect
  catches); `editor.md` — the "no soft wrap" paragraph replaced with the shipped
  architecture (facade layer, segment rim, motion, gates, the aware-column discipline);
  `roadmap.md` — B-c item 2 marked SHIPPED. Parity sync: `MARLEY-PARITY.md`'s editor-
  surface row records the both-sides wrap + the committed OFF default; the POC's
  `SOFT_WRAP` const ships at the Marley default with the ON capture as the design record.
- **Ledger appends:** `AD-claude-426-soft-wrap-is-a-facade-layer-on-cell-arithmetic-001`;
  `L-claude-426-live-drives-need-a-click-and-reseeded-geometry-001`; at inspect:
  `F-claude-426-adopted-rules-drift-three-ways-before-first-test-001`,
  `F-claude-426-cheap-bound-inverted-tabs-strand-the-tail-001`,
  `F-claude-426-a-clip-undid-a-widening-the-off-path-depended-on-001`,
  `PR-claude-426-segment-math-eats-only-phantom-aware-columns-001`.
- **Accepted debts (recorded):** phantom assembly on memo HIT + per display_map() call
  under ON (bounded; OFF pays zero); ⌘⌥↑/↓ add-cursor stays buffer-row under ON;
  the ⌘⌥-then-↑ goal-domain mix (self-corrects); Home/End buffer-line (deferral);
  the same-width-same-anchor phantom-text cache residual.
- **Ticket:** closed → `tickets/closed/`; BACKLOG swept (427 now tops the queue).
- **Archive:** pair → `pipeline/completed/`.
