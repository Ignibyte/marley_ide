---
pipeline_id: 7a3ead7a-453e-4c2a-9ee8-20ca8ffdf9ea
ticket: forge#255 (fc52ec37-67ce-46a1-a976-57dd7ab90069)
aar_id: dac6c569-dc59-4832-a831-dc658eb44b85
---

# Editor selection (#255) — pipeline notes

## Phase 1 — Plan

**Intent.** The editor has a single caret (#251 types, #254 clicks). #255 makes it a live SELECTION: shift+arrow
extends (anchor fixed, head moves), a mouse-drag selects, the #250 renderer highlights the span, typing/backspace
replaces it.

**Classification.** Work pipeline, feature, LARGE — a selection subsystem: a model change (an `anchor` per file) +
shift-extend + drag + highlight + edit-replace. The pure seam (the extend/collapse fn + `row_selection_cols`)
stays tight, so PREFER one v1 slice; design may recommend a #255a/b split only if the shim grows unwieldy.

**Discovery (confirmed):**
- `marley_editor::Selection { anchor, head }` EXISTS (selection.rs:9) + new/caret/anchor()/head(); a caret =
  zero-width (anchor==head). #255 uses it (normalize anchor..caret).
- `input.rs`: `Key` enum (:17), `apply_key(buffer, caret, key) -> KeyOutcome` (:58, the prompt engine),
  `apply_editor_key` (:~120, the #251 editor variant). ⚠️ RISK (D3): whether `Key` carries a shift bit is unknown
  — design must RE-READ `Key`/`apply_editor_key` + the app.rs on_key_down editor branch to see where shift is
  available (a shift+arrow has shift=true; the extend logic needs it). If `Key` doesn't carry shift, add a
  variant / a shift param / compute the extend in the app handler.
- The app holds `active_caret: CharOffset` per OpenFile (#249), moved by #251/#254 via
  `active_buffer_and_caret_mut`. #255 adds `anchor: Option<CharOffset>` alongside.
- The #254 click uses the render loop var for the row; a drag's `on_mouse_move` needs a y→row map (code-area top
  + cell.h + scroll) — the new shim piece. The #254 canvas records `origin.x`; #255 also needs `origin.y` (the
  code-area top).
- The #250 render loop draws each row; the highlight adds a tint rect over the selected columns (via
  `col_of_offset`, the #250 map).

**§20 (FILLED).** The universal monospace-editor selection convention (Zed = the editor-experience reference; NO
Zed source read — clean-room). Shift-extend + drag-select + highlight + replace-on-type, over Marley's own
`Selection` type + the #250 grid (the highlight draws on the same cell grid as the caret — observed capture
250-warp-monospace-grid-caret.png).

**Locked-in decisions:** D1 `anchor: Option<CharOffset>` per file (None = bare caret). D2 the shift-extend is a
pure fn reusing movement. D3 the shift bit must reach the editor (design resolves how — the key risk). D4
drag-select via a `dragging_selection` flag + y→row. D5 the highlight is a pure per-row span + a render rect. D6
edit-over-selection replaces the range then collapses. See the spec.

**EARS AC:** REQ-001 shift-extend / unshifted-collapse · REQ-002 drag selects the range · REQ-003 highlight the
span · REQ-004 type/backspace replaces the selection · REQ-005 the model is pure cov/MSI 100.

**Risks:** (1) D3 — threading the shift bit to the editor (RE-READ the key path; if Key lacks shift, extend it).
(2) the drag y→row math (the new shim; the pure offset is reused). (3) the highlight span across multibyte/tabs
(reuse `col_of_offset`, test tabs+multibyte). (4) edit-over-selection must normalize start/end (not edit
anchor..caret raw when caret < anchor). (5) mutants::skip detach if a fn moves near a masked shim.

**Gate note:** the #253 real-PTY flake (`workspace_two_real_sessions_are_independent` wedges under llvm-cov) — if
the gate runs >10 min, `ps -o pid,pcpu,etime`, kill + re-run (`PR-claude-gate-hang-diagnose-via-per-proc-cpu-253`).
The #254 harness note: a fresh app instance may have a launch keyboard-focus flake — relaunch if typing doesn't
reach the editor.

**Phase 1 status: Plan PASS — ready for Phase 2 — Design.**

---

## Phase 2 — Design

### Architecture / approach
Three pure fns (marley_editor + code_view + input) + the app.rs shim (anchor field, the key route, the per-row
drag handlers, the highlight render). Confirms §20 (universal editor selection over the #250 grid; clean-room).
KEY simplification found in design: the DRAG reuses #254's per-row trick — a per-`code_row` `on_mouse_move` gives
the row for free (the loop var), so NO y→row / origin.y math is needed (the plan's D4 y→row concern is avoided).

**D1 — model (editor_surface.rs).** OpenFile (`{view, buffer, caret, saved_version}`) gains `anchor:
Option<CharOffset>` (None = a bare caret at `caret`; Some(a) = the selection a..caret). Accessors: a combined
`active_buffer_caret_anchor_mut() -> (&mut Buffer, &mut CharOffset, &mut Option<CharOffset>)` (three disjoint
fields → E0499-free, the #251 lesson) + a read `active_selection() -> Option<(CharOffset, CharOffset)>` (the
normalized `(min, max)` when `anchor` is Some AND `!= caret`; None for a bare/zero-width caret) for the render +
edit.

**D2/D3 — shift-extend/collapse (movement.rs, PURE) + the key route (app.rs shim).** A pure
`extend_or_move(anchor: Option<CharOffset>, caret: CharOffset, buffer: &Buffer, right: bool, shift: bool) ->
(Option<CharOffset>, CharOffset)`:
- `shift` → `new_anchor = Some(anchor.unwrap_or(caret))`; `new_caret = if right { move_char_right(buffer, caret)
  } else { move_char_left(buffer, caret) }` (reuse #251 movement). (Extend — the anchor is pinned, the head
  moves. A shift-arrow that shrinks to `anchor==caret` stays Some; the render/edit treat `anchor==caret` as no
  selection.)
- `!shift` → if `anchor` is Some and `!= caret` (a real selection): COLLAPSE to the edge — `new_caret = if right
  { max(anchor,caret) } else { min(anchor,caret) }` (Left→start, Right→end, WITHOUT moving further — the standard
  editor feel); else `new_caret = move_char_{right,left}(buffer, caret)` (a plain move). Either way `new_anchor =
  None`. (min/max via `.as_usize()` then `CharOffset::from`.)
cov/MSI 100. D3 route: the app.rs on_key_down editor branch (app.rs:4396, guard `editor().is_some() && !platform
&& !control`, `event.keystroke.modifiers.shift` IS in scope) — reads `shift`, gets
`active_buffer_caret_anchor_mut()`, and on `key_from_keystroke`:
- `Key::Left|Right` → `extend_or_move(...)` → set (anchor, caret).
- `Key::Char(_)|Backspace` → if `active_selection()` is Some (non-empty): `buffer.edit(start..end,
  selection_replacement(key))` then collapse (anchor=None, caret = start + replacement.chars); else
  `apply_editor_key(buffer, caret, key)` + `anchor = None` (a plain type collapses).
- else (Enter…) → `apply_editor_key` + `anchor = None`.

**D6 — selection_replacement (input.rs, PURE).** `selection_replacement(key: &Key) -> Option<String>`:
`Char(c)` → `Some(c.to_string())`, `Backspace` → `Some(String::new())` (delete the selection), else `None` (not
a replacing key). Pure, testable. The app uses it for the edit-over-selection replacement text.

**D4 — drag-select (app.rs shim, the per-row trick — NO y→row).** A `dragging_selection: bool` RootView field.
The #254 per-`code_row` `on_mouse_down` ALSO sets `anchor = Some(caret) = click offset` + `dragging_selection =
true` (a click STARTS a potential selection; a mouseup without a move leaves anchor==caret → a bare caret). ADD a
per-`code_row` `on_mouse_move` (mirror the on_mouse_down: `row` = the loop var, `col` from `(event.x - x0)/
cell.w`) gated on `dragging_selection` → set `caret = head` to the moved offset (anchor unchanged) → a live drag
selection. ADD an `on_mouse_up` (on `body` / the editor pane) → `dragging_selection = false`. A plain click (down
+ up, no move) → anchor==caret → `active_selection()` None → a bare caret (REQ collapses). NO y→row / origin.y —
the row comes from the loop var, exactly like #254.

**D5 — highlight (code_view.rs PURE + app.rs render).** `row_selection_cols(sel_start: usize, sel_end: usize,
row_start: usize, row_nchars: usize, layout: &LineLayout) -> Option<(usize, usize)>`: `let s =
sel_start.max(row_start); let e = sel_end.min(row_start + row_nchars); if s >= e { None } else { Some((
layout.col_of_offset(s - row_start), layout.col_of_offset(e - row_start))) }` — the selected DISPLAY columns of
the row (via #250's col_of_offset; multibyte/tabs handled). A whole-line-selected row → (0, width); a partial
row → the mid-columns; no overlap → None. v1 highlights the selected CHARS (not the trailing newline — a
cross-line selection shows each line's chars). cov/MSI 100. Render: in code_view_body's EditorDraw loop, when
`active_selection()` is Some, compute `row_selection_cols` per row and draw a tint rect (`colors.selection` if it
exists, else an accent-alpha) behind the text at `[col_start×cell.w, col_end×cell.w]` (an `.absolute()` child of
`code_row`, BEFORE the text spans so it sits behind — like the #250 caret bar's placement).

### File manifest
| File | Change |
|---|---|
| crates/editor/src/movement.rs | ADD `extend_or_move(anchor, caret, buffer, right, shift) -> (Option<CharOffset>, CharOffset)` (pure; reuses move_char_left/right). |
| crates/marley_app/src/input.rs | ADD `selection_replacement(key: &Key) -> Option<String>` (pure). |
| crates/marley_app/src/editor_surface.rs | OpenFile `anchor: Option<CharOffset>` field + `active_buffer_caret_anchor_mut` + `active_selection`. |
| crates/marley_app/src/code_view.rs | ADD `row_selection_cols(...) -> Option<(usize,usize)>` (pure). |
| crates/marley_app/src/app.rs | the on_key_down editor branch (shift route + edit-over-selection), the per-row on_mouse_down (set anchor + dragging), the per-row on_mouse_move (extend during drag) + on_mouse_up (clear), the highlight render. SHIM (keep mutants::skip). |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | movement: `extend_or_move` SHIFT-extend — (None, co(5), Right, shift)→(Some(5), co(6)); (Some(5), co(6), Right, shift)→(Some(5), co(7)); (None, co(5), Left, shift)→(Some(5), co(4)) | REQ-001 |
| T2 | movement: `extend_or_move` UNSHIFTED — (Some(2), co(5), Left, false)→(None, co(2)) [collapse to start]; (Some(2), co(5), Right, false)→(None, co(5)) [collapse to end]; (None, co(5), Left, false)→(None, co(4)) [plain move]; (None, co(5), Right, false)→(None, co(6)) | REQ-001 |
| T3 | code_view: `row_selection_cols` — partial-start ("hello" sel [2,4) on row_start 0 → Some((2,4))); whole-row (sel [0,5) → Some((0,5))); no-overlap (sel [10,12) on row_start 0 nchars 5 → None); a TAB row ("\ta" sel over the tab → cols via col_of_offset); multibyte ("éx😀" sel [1,3) → the é/x cols) | REQ-003/005 |
| T4 | input: `selection_replacement` — Char('x')→Some("x"); Backspace→Some(""); Left→None; Enter→None | REQ-004 |
| T5 (driven) | shift+Right×N highlights N chars; type a char replaces the highlighted range → one char; a drag (down→move→up) highlights a range; data-safe (no ⌘S) | REQ-001/002/003/004 |

**Uncoverable/shim:** the app.rs anchor field + the key route + the drag handlers + the highlight render are shim
(coverage-excluded; driven T5 + review). The 3 pure fns (extend_or_move, row_selection_cols, selection_replacement)
are the cov/MSI-100 seam.

**Risks:** (1) the collapse-to-edge nuance (T2 pins it). (2) the highlight span across multibyte/tabs/partial
rows (T3). (3) edit-over-selection must normalize start/end + collapse (the app route; driven T5). (4) the drag
per-row on_mouse_move must gate on dragging_selection (else a hover moves the caret) — driven. (5) mutants::skip
detach — `cargo mutants --list` after. (6) `colors.selection` may not exist — implement greps ThemeColors +
falls back to an accent-alpha.

**Phase 2 status: Design PASS — the anchor model + extend_or_move (shift-extend/collapse-to-edge) + the shift
route (D3 via the on_key_down branch's modifiers.shift) + the per-row drag (NO y→row) + row_selection_cols +
selection_replacement + T1-T5 locked. Ready for Phase 3 — Implement.**

---

## Phase 3 — Implement (PASS)

Built to the manifest; no design deviations. The key simplification (per-row drag → NO y→row) held.

- **crates/editor/src/movement.rs** — `extend_or_move(anchor, caret, buffer, right, shift) -> (Option<
  CharOffset>, CharOffset)` (pure): shift → pin anchor (`Some(anchor.unwrap_or(caret))`) + move the head; no-shift
  with a REAL selection (`anchor.filter(|a| *a != caret)`) → collapse to the edge (`right`→end/else start); else
  a plain move. Reuses move_char_left/right. Exported via lib.rs `pub use movement::{extend_or_move, …}`.
- **crates/marley_app/src/input.rs** — `selection_replacement(key: &Key) -> Option<String>` (pure): Char→Some(c),
  Backspace→Some(""), else None.
- **crates/marley_app/src/editor_surface.rs** — OpenFile `anchor: Option<CharOffset>` (init None in the ONE
  `OpenFile::new`) + `active_selection() -> Option<(CharOffset,CharOffset)>` (normalized (min,max), None for a
  bare/zero-width caret) + `active_buffer_caret_anchor_mut() -> (&mut Buffer, &mut CharOffset, &mut
  Option<CharOffset>)` (the E0499-safe combined accessor).
- **crates/marley_app/src/code_view.rs** — `row_selection_cols(sel_start, sel_end, row_start, row_nchars,
  layout) -> Option<(usize,usize)>` (pure): intersect the selection with the row's char range → col_of_offset of
  the endpoints; None off the selection.
- **crates/marley_app/src/app.rs (SHIM — all inside render / code_view_body, both `#[cfg_attr(test,
  mutants::skip)]`):** `dragging_selection: bool` field + init; imports (extend_or_move / selection_replacement /
  row_selection_cols). THE KEY ROUTE (render on_key_down editor branch): reads `event.keystroke.modifiers.shift`,
  reads `active_selection()` then `active_buffer_caret_anchor_mut()`, `match &key` → Left/Right → extend_or_move
  (set anchor+caret); else → if a non-empty selection + a replacing key: `buffer.edit(start..end, &repl)` +
  collapse (caret = start+repl.chars, anchor None); else apply_editor_key + anchor None. THE DRAG (code_view_body
  per-row): the #254 on_mouse_down now sets `anchor = Some(caret)` + `dragging_selection = true` (via
  active_buffer_caret_anchor_mut); a per-row `on_mouse_move` (gated on dragging_selection) extends the head
  (`row` = loop var → NO y math; a fresh `x0m` Rc clone); an `on_mouse_up` clears dragging_selection. THE
  HIGHLIGHT: `sel` read once before the loop (active_selection as usizes); per row a tint rect
  (`hsla(accent.h,s,l,0.28)` — ThemeColors has NO `selection` field, so an accent-alpha) drawn as an `.absolute()`
  child of code_row BEFORE the text spans (paints behind); reuses the row's `layout` + `line_start` + line_text
  char-count.
  - DEVIATION (minor): the loop top now binds `let text = …; let layout = line_layout(&text, …)` (was
    `let display = line_layout(…).display`) so the FULL layout (col_starts) feeds both the highlight and the text
    (`highlight_line(&layout.display, …)`) — one build, not two.

**Checks:** `cargo fmt` clean; `cargo check -p marley_editor` + `-p marley --all-targets` clean (only the
pre-existing block v0.1.6 dep warning). **`render` + `code_view_body` are BOTH `mutants::skip` → the entire #255
app.rs shim has 0 new live mutants** (app.rs total stays 6, all pre-existing; grep code_view_body/dispatch_action
== 0, none in the 4390-4480 key-route region). Pure-seam mutants for Phase 4: movement.rs extend_or_move
{`!=`→`==`, `<=`→`>`, tuple Default pair}, code_view.rs row_selection_cols {None/Some tuples, `+`→`-`/`*`, `>=`→
`<`, `-`→`+`/`/`}, input.rs selection_replacement {None/Some/arms}. All T1-T4 targets. ThemeColors selection
choice: accent-alpha (no `selection` field).

**Phase 3 status: Implement PASS — ready for Phase 3.5 — Inspect.**

---

## Inspect (Phase 3.5) — PASS (1 HIGH fixed)

3 critics (pure logic · key-route/edit · drag/highlight/regression). **The pure logic + the key route + the
shift-threading (D3) are all CONFIRMED correct; ONE real HIGH bug found + FIXED.**

### Critic 2 — key route + edit-over-selection (all CONFIRMED, 0 defects)
- (g) NORMALIZATION: `active_selection()` normalizes to `(min, max)`, so a BACKWARDS selection (caret<anchor)
  edits min..max, NOT a reversed range — `edit(3..8)` for anchor=8/caret=3. The caret lands at start+repl.chars;
  anchor=None collapses. (h) the shift bit reaches extend_or_move (orthogonal to the `!platform && !control`
  guard). (i) **CRITICAL — key_from_keystroke maps a SHIFTED arrow to Key::Left/Right (app.rs:1467, no shift
  guard on the arrow arm), NOT Other** → the extend actually fires (the D3 risk is CLOSED). Range valid, no
  unwrap/panic. By-design note: Enter over a selection inserts a newline + collapses (matches the AC "any other
  edit collapses"), not a defect.

### Critic 1 — pure logic (all CONFIRMED correct; findings are Phase-4 test gaps)
- (a)-(e) extend_or_move (shift-extend + collapse-to-edge, `filter(|a| *a != caret)` treats anchor==caret as no
  selection), row_selection_cols (intersect + col_of_offset, c0<c1 strict), active_selection (normalize, zero-
  width→None), selection_replacement (arms) all traced correct.
- **VALIDATE REQUIREMENTS (from the mutant inventory — the naïve trace table would miss these):**
  - [MED, gate-blocking] row_selection_cols `-`→`+` mutants (123/124) are EQUIVALENT at row_start=0 → MUST add a
    row_start≥1 case (an interior row of a multi-line selection).
  - [MED] extend_or_move anchor-pinning: `unwrap_or` is a method call (UNMUTATED) → assert BOTH `(Some5,co6,shift)
    →(Some5,co7)` [pins "existing anchor preserved"] AND `(None,co5,shift)→(Some5,co6)` [None→unwrap_or(caret)].
  - [LOW] add an `s==e` tie case (the `>=` `>`-direction is un-emitted, #254 analog); [LOW] assert
    `selection_replacement(Backspace)==Some("")` (else the delete-arm mutant survives); [LOW] keep the collapse
    anchor ≥2 chars from the caret so `!=`→`==` isn't masked.

### Critic 3 — drag + highlight + regression
- **[HIGH — FIXED IN CODE] Stuck `dragging_selection` on an off-row mouse-up.** gpui's `on_mouse_up` is
  bounds-gated (fires only when released over a row's hitbox). Releasing a drag in the empty area below the text
  / padding / the terminal pane / outside the window → NO row's on_mouse_up fires → the flag stays `true` → a
  subsequent plain HOVER passes the gate and reshuffles the caret (the exact bug the gate prevents, via a leaked
  flag). **FIX:** in `on_mouse_move`, a self-heal — `if event.pressed_button != Some(MouseButton::Left) {
  dragging_selection = false; return; }` (a button-less move means the drag ended elsewhere → clear + don't move
  the caret). Compiles; skips still 0. `BF-claude-drag-flag-leaks-on-bounds-gated-mouse-up`.
- (j)/(k)/(l) all else CONFIRMED: the hover-gate present; a plain click → bare caret; x0m/x0h distinct clones;
  the highlight rect is behind the text + `(c1-c0)` can't underflow (c0<c1 strict) + the layout built once; the
  #254 click no-regression (still places the caret; the added anchor=caret is inert); render+code_view_body both
  mutants::skip (0/6 app.rs mutants unchanged); no unwrap; clean-room OK.
- [LOW, no action] blank lines / the trailing newline render no tint in a multi-line selection (documented v1
  intent — highlights the selected chars).

**Phase 3.5 status: Inspect PASS — 1 HIGH fixed (the drag-flag self-heal). Ready for Phase 4 — Validate (which
MUST add the row_start≥1 + anchor-pinning + tie + Backspace tests per critic 1).**

---

## Phase 4 — Validate (PASS — GATE GREEN [diff])

### Tests added (cov/MSI 100 on the pure seam — incl. critic-1's MANDATORY additions)
- **movement.rs** (2): `extend_or_move_shift_extends_and_pins_the_anchor` (the anchor-PINNING assertion
  `(Some5,co6,shift)→(Some5,co7)` pins the unmutated `unwrap_or`) + `extend_or_move_unshifted_collapses_to_edge_
  or_moves` (collapse-to-edge, a BACKWARDS selection, and the anchor==caret→plain-move filter case).
- **input.rs** (1): `selection_replacement_maps_replacing_keys` (Char/Backspace→Some(""); Left/Enter→None — the
  explicit Backspace assertion kills the delete-arm mutant).
- **editor_surface.rs** (1): `active_selection_normalizes_and_none_when_bare` (None when bare / anchor==caret;
  normalized (min,max) forwards AND backwards).
- **code_view.rs** (1): `row_selection_cols_intersects_and_maps_columns` (partial / whole / no-overlap / the
  s==e tie → None / an **INTERIOR row row_start=3** [kills the `-`→`+` mutants, equivalent at row_start 0] / a
  tab row).

### Test runs (actual)
- `cargo nextest run -p marley_editor` → **46 passed** (+2 extend_or_move). `-p marley` → **363 passed, 2
  skipped** (+3: row_selection_cols / selection_replacement / active_selection, each confirmed by name).
- `cargo mutants --list`: the pure-seam mutants (extend_or_move `!=`/`<=`/tuple-Default; row_selection_cols
  None/Some/`+`/`>=`/`-`; selection_replacement arms; active_selection `==`/`<=`/None/Some) are all T1-T5 targets.
  app.rs `code_view_body`+`dispatch_action` == 0 (render + code_view_body both skip'd → the whole shim skipped).

### Driven live proof (REQ-002/003/004 — mac unlocked, DATA-SAFE)
Bundled + open; the #205 session restored selection.rs (active). **REQ-002/003:** `drag:0.42,0.26,0.52,0.26`
(a horizontal drag on line 10 `anchor: CharOffset,`) → **a cyan accent-alpha HIGHLIGHT span rendered over the
dragged range** (`harOffset,`), caret at the drag end (m255_01). **REQ-004:** drag + type `Z` in ONE drive call
(no `focus` between, so the selection persists) → **`anchor: CZ`** — the ~9-char selection was REPLACED by the
single `Z` (m255_03). **DATA-SAFETY: no ⌘S ever → `git status` confirms selection.rs PRISTINE on disk;** only the
#255 source I authored is modified. (REQ-001 shift-extend: drive.swift has no shift+arrow verb → unit-proven
T1/T2 + the key route critic-verified. HARNESS NOTE: a SEPARATE `focus type:Z` after the drag CLEARED the
selection — the `focus` activate does a click-to-activate that seeds a zero-width anchor; doing drag+type in one
call avoids it.)

### Gate
`scripts/gates.sh --diff` → **GATE GREEN [diff] — 15 passed, 0 failed** (gate:4 coverage ≥100%, gate:5 mutation
MSI ≥100% on the pure seam, gate:1 fmt, gate:6 miri, gate:15 visual). ~210s, no real-PTY wedge. Receipt
`f9771e97…` matches the current `.rs` state → the commit gate will pass.

**Phase 4 status: Validate PASS — GATE GREEN [diff]. Ready for Phase 5 — Complete.**
