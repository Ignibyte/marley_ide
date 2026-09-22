---
pipeline_id: a6709898-394f-44d5-a586-d8ceaff6e765
ticket: forge#254 (fb9e2682-6515-4acc-9514-bc88215eda20)
aar_id: 41875b20-6243-4151-b507-11650a559ece
---

# Click-to-place-caret (#254) — pipeline notes

## Phase 1 — Plan

**Intent.** The editor caret starts at offset 0 and only moves by typing (#251). A left-click in the editor body
does nothing today. #254 makes a click place the caret at the `char` nearest the click, by inverting #250's
char↔column map. The shared risk with #250: a wrong map puts the caret on the wrong char → a later save/selection
corrupts. Mitigation: `offset_of_col` inverts the SAME `col_starts`, round-tripped against `col_of_offset` in
tests.

**Classification.** Work pipeline, feature, MEDIUM. Two pure pieces (the `col→offset` inverse + the
`(row,col)→CharOffset` composition) + one app.rs mouse shim (pixel→row/col + set caret). The #250 offset↔column
reuse de-risks the hard part.

**Discovery (confirmed by reading the code):**
- `code_view.rs`: `LineLayout { display: String, col_starts: Vec<usize> }`. `col_starts[i]` = display column
  where char `i` begins; `len == n_chars + 1`; last = total width. `col_of_offset(char_idx)` (line 48) =
  `col_starts[char_idx.min(len-1)]`. The doc (code_view.rs:34-35) EXPLICITLY reserves the inverse for #254 "over
  the same col_starts". `line_layout(line, tab_width)` builds it (tab_width floored at 1).
- `app.rs`: `code_view_body` (~2548) draws the editor via `EditorDraw { buffer, caret: Option<(row,col)>, cell }`
  (`cell.h` = line_height, `cell.w` = em_advance); the caret draws at `col × em_advance` (~2584). The editor body
  has NO caret-click handler — the `on_mouse_down` at app.rs:2688 is the FILE-TREE (opens files), NOT the editor.
- `buffer.rs`: #250 added `line_text(row)` + `line_col(offset) -> (row, char-in-line)`. A line-START char offset
  (`line_to_char`, ropey) may need a thin pure wrapper for the composition — design confirms.
- Focus: from #253's driven run, typing reaches the active editor tab via the #251 `on_key_down` branch even
  though the status bar reads "focus: terminal" — so the click's job is to SET THE CARET; design confirms whether
  an explicit focus call is also needed.

**§20 (FILLED).** Warp's command-input click-to-place-caret = the universal monospace-editor convention (click →
the nearest cell → the caret between two chars). Shared substrate = #250's observed grid capture
(`docs/warp_architecture/observed/250-warp-monospace-grid-caret.png`); #254 is that grid inverted. Marley's own
pure inverse over `col_starts` (`AD-claude-editor-offset-column-model-001`). No fresh capture (clicking chad's
live Warp input would disturb his session; the #250 grid capture documents the geometry #254 inverts).

**Locked-in decisions:** D1 `offset_of_col` inverts the same `col_starts` (nearest-boundary, ties→later, clamp
`[0,n_chars]`). D2 the click→CharOffset math is pure/testable; the shim only does pixel→(row,col). D3 clamps
never panic (row→last line, col→line-end). D4 editor-tab-guarded. D5 the shim reuses the #250 gutter/scroll/cell
geometry (click-in and caret-out self-consistent). See the spec.

**EARS AC:** REQ-001 offset_of_col inverse+clamp (pure) · REQ-002 click sets the caret + redraws (driven) ·
REQ-003 clamps past-EOL / past-last-line, no panic (pure) · REQ-004 editor-tab-guarded (review+driven) · REQ-005
click-then-type inserts at the click (driven).

**Risks:** (1) the offset↔column round-trip exactness (tabs + multibyte — the #250 shared risk); mitigated by a
round-trip test vs `col_of_offset`. (2) the shim's pixel→(row,col): the code-area origin (gutter width) + scroll
must be subtracted correctly, or the caret lands off the click — design must find how gpui exposes the clicked
element's bounds (the #130 drag or a positioned child). (3) `mutants::skip` detach if a fn is inserted near a
masked shim (the #252/#253 lesson) — run `cargo mutants --list` after.

**Gate note:** the #253 diff gate flakily wedged on the real-PTY test `workspace_two_real_sessions_are_independent`
under llvm-cov — if the gate runs >10 min, `ps -o pid,pcpu,etime` the test procs; a binary at 0% CPU for minutes
is a deadlock → kill + re-run (`PR-claude-gate-hang-diagnose-via-per-proc-cpu-253`).

**Phase 1 status: Plan PASS — ready for Phase 2 — Design.**

---

## Phase 2 — Design

### Architecture / approach
Two pure seams (marley_app `code_view` + marley_editor `Buffer`) + one app.rs mouse shim. Confirms §20: the
click→cell→offset mapping is #250's grid INVERTED (observed capture 250-warp-monospace-grid-caret.png); pure
inverse over `col_starts` (`AD-claude-editor-offset-column-model-001`); clean-room, no Warp/Zed source.

**1. `LineLayout::offset_of_col(col) -> usize` (code_view.rs — pure seam #1, the col→offset INVERSE).**
`col_starts` is sorted ascending (len n_chars+1). Given a display `col`, return the char index `i ∈ [0,n_chars]`
minimizing `|col_starts[i] as isize - col as isize|`; on a TIE, the LATER (larger `i`) boundary wins (a click in
a char's right half lands after it — the editor feel). Impl: a single scan tracking the best `(dist, i)`, `<=`
on the distance compare so a later equal-distance index overwrites the earlier (→ ties go later). Clamp is
automatic (i never exceeds n_chars). Round-trip invariant: `offset_of_col(col_of_offset(i)) == i` for all i
(exact-column case). Tabs: for `"\ta"` (col_starts `[0,4,5]`) → offset_of_col(0)=0, (1)=0, (2)=1 [tie 0/4→later],
(3)=1, (4)=1, (5)=2, (6)=2 [clamp]. Multibyte: col_starts is per-CHAR (é/😀 = 1 col v1) → returns a CHAR index.

**2. `offset_for_click(line_text: &str, col: usize, tab_width: usize) -> usize` (code_view.rs — pure seam #2).**
Thin composer: `line_layout(line_text, tab_width).offset_of_col(col)`. The in-LINE char offset. (Kept a free fn
next to `line_layout` so the app doesn't rebuild the layout twice.)

**3. `Buffer::line_start(row: usize) -> CharOffset` (buffer.rs — pure seam #3).** `rope.line_to_char(row.min(
len_lines()-1))` wrapped in `CharOffset::from(..)` — the absolute char offset of line `row`'s start, row clamped
to the last line (never panics). `line_col`/`line_text` already use `line_to_char` internally (buffer.rs:112),
so this exposes the same, clamped. The app composes: `abs = line_start(row) + offset_for_click(line_text(row),
col, tab_width)`.

**4. THE SHIM (app.rs `code_view_body` EditorDraw branch — pixel→(row,col), mutants::skip/coverage-excluded).**
Marley has NO absolute click→element bounds precedent (the #130 divider is Δx-delta; forge rows use modifiers).
RESOLUTION — the ROW LOOP is the lever:
- **Row = the loop variable** (`code_view_body` already loops `for row in start..end`). Attach
  `.on_mouse_down(MouseButton::Left, cx.listener(move |view, event, _w, cx| …))` to EACH rendered `code_row`,
  capturing `row`. → NO y/scroll/line-height math, NO vertical bounds needed (gpui hit-tests the click to the
  row element; the row index is known). This sidesteps the hardest bounds problem.
- **Column** = `round((event.position.x - text_x0) / ed.cell.w)`, clamped `≥ 0`, where `text_x0` = the code
  column's left pixel (after the gutter+gap). Record `text_x0` via a `canvas` element placed at the code-area
  text origin whose prepaint writes the bounds' `origin.x` into a shared `Rc<Cell<f32>>` (`editor_text_x0`, a new
  RootView field cloned into the canvas + the row handlers). This is the standard gpui bounds-capture (an
  `Rc<Cell>` shared between the render-time canvas and the event handler) — clean-room, no Zed Element code.
  (Confirm `gpui::canvas` imports at implement; FALLBACK if not: a per-row handler that records the row's own
  origin.x on first paint, same Rc<Cell>.)
- **Set the caret:** `let abs = view.<buffer>.line_start(row).as_usize() + offset_for_click(&line_text, col,
  tab_width); let (_b, c) = surface.active_buffer_and_caret_mut(); *c = CharOffset::from(abs);` then `cx.notify()`
  (#250 redraws the caret). GUARD: the handler is only built inside the `EditorDraw`/editor branch (a terminal
  tab / the #246 read-only `None` split-pane branch has no handler) → REQ-004 by construction.
- KEEP any adjacent `mutants::skip` (the #252/#253 detach trap): `code_view_body` is a render method; after
  implement RUN `cargo mutants --list -f crates/marley_app/src/app.rs` — the new pixel arithmetic must be
  skip'd/excluded (app.rs is coverage-excluded; confirm no new live mutant on the arithmetic, else the fn needs
  the skip).

**5. FOCUS (D4 — confirmed).** #251's `on_key_down` editor branch routes keys to the active editor tab (guard
`active_tab().editor().is_some()`), independent of any click-focus — #253's driven run typed into the active
editor with the status bar reading "focus: terminal". So the click ONLY sets the caret; NO explicit focus call.

### File manifest
| File | Change |
|---|---|
| crates/marley_app/src/code_view.rs | ADD `LineLayout::offset_of_col(col)->usize` (nearest-boundary inverse over col_starts) + free fn `offset_for_click(line_text,col,tab_width)->usize`. Pure. |
| crates/editor/src/buffer.rs | ADD `pub fn line_start(&self, row: usize) -> CharOffset` (clamped `line_to_char`). Pure. |
| crates/marley_app/src/app.rs | ADD RootView field `editor_text_x0: Rc<Cell<f32>>` (+ init) + a `canvas` recorder in the EditorDraw branch + a per-`code_row` `on_mouse_down` computing (row=loop, col=(x−x0)/cell.w) → the pure fns → set active caret + notify. SHIM (coverage-excluded; keep/verify mutants::skip). |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | code_view: `offset_of_col` round-trips `col_of_offset` — for "hello" (7 cols), `offset_of_col(col_of_offset(i))==i` for i in 0..=5 | REQ-001 |
| T2 | code_view: nearest-boundary + TIE on a tab line "\ta" (col_starts [0,4,5]): offset_of_col at cols 0..=6 → [0,0,1,1,1,2,2] (kills the min-scan `<`/`<=` + the tie-goes-later branch) | REQ-001 |
| T3 | code_view: clamp — `offset_of_col(999)` on "abc" → 3 (n_chars) | REQ-001/003 |
| T4 | code_view: `offset_for_click` multibyte "éx😀" at cols 0..=3 → 0,1,2,3 (CHAR indices) + past-EOL "ab" col 99 → 2 | REQ-001/003 |
| T5 | buffer: `line_start` on "a\nbb\nccc" → line_start(0)=0, (1)=2, (2)=5, (99)=5 (clamp to last line); multibyte "é\n😀" → line_start(1)=2 (chars, after "é\n") | REQ-002/003 |
| T6 (driven) | click a spot mid-line in the editor → the caret bar jumps there (not offset 0) → type a char → it inserts at the click point; data-safe (no ⌘S) | REQ-002/004/005 |

**Uncoverable/shim:** the app.rs pixel→(row,col) + the canvas bounds recorder are shim (coverage-excluded;
driven T6 + review). The 3 pure fns are the cov/MSI-100 seam.

**Risks:** (1) offset↔col round-trip exactness (tabs/multibyte) — T1/T2/T4 vs col_of_offset. (2) the shim's
`text_x0` capture (the canvas Rc<Cell> pattern is new to Marley) — driven-validated (T6); FALLBACK per-row origin
capture noted. (3) the tie rule at a tab boundary — T2 pins it. (4) mutants::skip detach — `cargo mutants --list`
after implement. (5) `gpui::canvas` availability — confirm at implement, fallback noted.

**Phase 2 status: Design PASS — the pure inverse (offset_of_col nearest-boundary) + offset_for_click + line_start
+ the per-row-loop click shim (row from the loop, col from x−x0 via a canvas recorder) + T1-T6 locked. Ready for
Phase 3 — Implement.**

---

## Phase 3 — Implement (PASS)

Built to the manifest; ONE design-improving deviation (threading `cx`, below):

- **crates/marley_app/src/code_view.rs** — `LineLayout::offset_of_col(col) -> usize` (nearest-boundary inverse:
  a single `col_starts` scan tracking the min `abs_diff`, `<=` so ties go to the LATER index; exact matches are
  unique → the round-trip `offset_of_col(col_of_offset(i))==i` holds) + free fn `offset_for_click(line, col,
  tab_width) -> usize` (= `line_layout(...).offset_of_col(col)`). Both pure/pub.
- **crates/editor/src/buffer.rs** — `pub fn line_start(&self, row) -> CharOffset` (`rope.line_to_char(row.min(
  len_lines-1))`, clamped; mirrors line_col's line_to_char use). Pure.
- **crates/marley_app/src/app.rs** — the click shim (all inside `code_view_body`, which is already
  `#[cfg_attr(test, mutants::skip)]`):
  - `use gpui::canvas` added; RootView field `editor_text_x0: Rc<Cell<f32>>` + init `Rc::new(Cell::new(0.0))`.
  - In `code_view_body`'s EditorDraw loop: on the FIRST visible row, a zero-size `.absolute()` `canvas` child of
    `code_row` records `f32::from(bounds.origin.x)` (the text-column left, window coords) into `editor_text_x0`.
  - Each row div gained `.w_full()` + `.on_mouse_down(Left, cx.listener(...))`: `col = round((f32::from(event
    .position.x) - x0.get()).max(0) / cell.w)`, then `editor_mut()` guard → `line_start(row) + offset_for_click(
    line_text(row), col, tab_width)` → `active_buffer_and_caret_mut().1 = CharOffset::from(abs)` → `cx.notify()`.
    `row` is the loop var (no y/scroll math); the guard makes it editor-only (REQ-004 by construction).
  - **DEVIATION (design-improving):** `code_view_body` had NO `cx` param (it's `&self`), so `cx.listener` was
    unavailable. Added `cx: &mut Context<Self>` to its signature + threaded `cx` at BOTH call sites (the editor
    tab at ~5026 + the #246 read-only pane at ~5856, both inside `render(…, cx)`). The `None`/read-only branch
    is unaffected (no handler built there).
  - **FALLBACK NOT NEEDED:** `gpui::canvas` IS exported (gpui-0.2.2/src/elements/canvas.rs) — used directly.

**Checks:** `cargo fmt` clean; `cargo check -p marley_editor` + `cargo check -p marley --all-targets` clean (only
the pre-existing `block v0.1.6` dep warning). **`cargo mutants --list -f app.rs | grep -c dispatch_action` == 0
AND `grep -c code_view_body` == 0** — the whole pixel shim sits in the mutants::skip'd `code_view_body`, so no
new live mutants (no #252/#253 detach). Pure-seam mutants for Phase 4 to kill: code_view.rs `offset_of_col`
{→0, →1, `<=`→`>`} + `offset_for_click` {→0, →1}; buffer.rs `line_start` {→Default::default()=CharOffset(0),
viable}. `abs_diff` is a method call → UNMUTATED (needs coverage, not a kill). All T1-T5 targets.

**Phase 3 status: Implement PASS — ready for Phase 3.5 — Inspect.**

---

## Inspect (Phase 3.5) — PASS (1 defensive fix)

2 independent critics (pure inverse correctness · shim/cx-threading/regression/clean-room). **Both: SHIP, zero
HIGH/MED — the implementation is correct.** Every check CONFIRMED.

### Critic 1 — pure inverse (all CONFIRMED)
- (a) ROUND-TRIP holds for ALL lines: `col_starts` is STRICTLY increasing (every non-tab char +1 col, a `\t`
  +≥1) → exact matches are unique → `offset_of_col(col_of_offset(i))==i`, the `<=` can't corrupt it (once dist
  hits 0 at j=i, no later j overwrites). The shared risk with #250 is CLOSED. Bonus: the composition
  `line_start(row) + offset_for_click(...)` exactly inverts #250's forward `line_col` map.
- (b) TIE→later: "\ta" (col_starts [0,4,5]) → offset_of_col cols 0..=6 = [0,0,1,1,1,2,2]; col-2 tie resolves to
  the LATER index via `<=`. (c) MULTIBYTE: char-indexed (line_layout iterates `.chars()`); "éx😀" cols 0-3 →
  0,1,2,3. (d) past-EOL clamps to n_chars. (e) line_start clamps `row.min(len_lines-1)` → no ropey panic; char-
  indexed.
- Killing-value table for every pure mutant confirmed (offset_of_col {→0,→1,`<=`→`>`}, offset_for_click {→0,→1},
  line_start {→Default=CharOffset(0)}); `line_layout→Default` UNVIABLE (no Default derive).

### Critic 2 — the shim (all CONFIRMED)
- (f) cx threaded into code_view_body + BOTH call sites (editor tab + #246 read-only pane, both inside
  `render(…,cx)`); the read-only/None branch builds NO handler → REQ-004. No 3rd caller. (g) click math correct
  (window-coord subtraction, .max(0) gutter-clamp, round). (h) E0499-free (sequential `s.active_buffer()`
  borrows end before the `_and_caret_mut`), no unwrap/panic (editor_mut None → skip), abs ≤ buffer len (can't
  corrupt), event-time re-fetch correct. (i) the canvas records the SAME origin the #250 caret uses (consistent).
  (j) `mutants::skip` directly above `fn code_view_body`; `grep -c code_view_body|dispatch_action` = 0/0; the
  `.w_full()` doesn't shift the #250 layout (text stays flex-start; only extends the hit area → REQ-003); the
  #250 caret draw + #246 None-branch are byte-unchanged; clean-room OK (gpui canvas Rc<Cell> is standard, not
  Zed Element code).

### Findings + verdicts
- **[LOW-1, critic 2 — FIXED IN CODE]** `cell_w = ed.cell.w` could be 0 on a degenerate font → `rel/cell_w` =
  NaN/inf (non-corrupting: the offset clamps, but a code smell). FIX: `ed.cell.w.max(f32::EPSILON)` — the
  arithmetic is now provably finite. (Inside code_view_body, still mutants::skip'd — re-verified 0.)
- **[LOW, critic 1 — VALIDATE REQUIREMENT, no code change]** cargo-mutants only generates `<=`→`>` for the tie
  compare (not `<=`→`<`), and all offset_of_col mutants die to ANY non-zero result → MSI can reach 100% WITHOUT
  testing a tie. → **Phase 4 MUST assert the tie explicitly** (`offset_for_click("\ta", 2, 4) == 1`) as a
  first-class AC test, independent of mutation. (T2 already targets this — keep it.)
- **[LOW-2, critic 2 — acknowledged, no fix]** first-frame x0=0.0 race: self-correcting (a click before the
  first prepaint maps to a huge col → clamps to line-end), and in gpui's frame model the handler is only
  registered during the paint that follows the x0-setting prepaint → never observed. Cosmetic edge.
- **[LOW, critic 1 — out of #254 scope, noted]** the round-trip depends on `char_width ≥ 1`; a FUTURE zero-width
  char hook (CJK/combining) would duplicate col_starts and break single-valued inversion. v1 has no zero-width
  chars → not a live bug. Guard when the hook lands (skip a zero-width col_starts entry).

**No `failure-record`** — no real bug (the cell_w guard is defensive hardening, not a defect fix). **Phase 3.5
status: Inspect PASS — ready for Phase 4 — Validate.**

---

## Phase 4 — Validate (PASS — GATE GREEN [diff])

### Tests added (cov/MSI 100 on the pure seam)
**crates/marley_app/src/code_view.rs** (3 tests):
- `offset_of_col_round_trips_col_of_offset` (REQ-001) — "hello", `offset_of_col(col_of_offset(i))==i` for i 0..=5
  (kills →0/→1 via non-zero i, and `<=`→`>` which returns 0 always).
- `offset_of_col_nearest_boundary_ties_go_later` (REQ-001, critic-1's requirement) — "\ta" (col_starts [0,4,5])
  → offset_of_col cols 0..=6 == [0,0,1,1,1,2,2]; the col-2 TIE → 1 (later) is the load-bearing assertion (a `<`
  gives 0) — NOT left to mutation (cargo-mutants only emits `<=`→`>`). + the clamp (col 6 → 2 = n_chars).
- `offset_for_click_char_indexed_and_clamps` (REQ-001/003) — "éx😀" cols 0..=3 → [0,1,2,3] (CHAR indices,
  multibyte) + "ab" col 99 → 2 (past-EOL → line-end).

**crates/editor/src/buffer.rs** (1 test): `line_start_is_the_char_offset_of_the_row_and_clamps` (REQ-002/003) —
"a\nbb\nccc" → line_start 0/1/2/99 = 0/2/5/5 (clamp to last line, no panic); "é\n😀" → line_start(1)=2 (chars).

### Test runs (actual)
- `cargo nextest run -p marley_editor` → **44 passed** (+1 line_start). `code_view::tests::offset*` → **3 passed**.
- `cargo nextest run -p marley` → **360 passed, 2 skipped** (+3 code_view).
- `cargo mutants --list`: the 6 pure-seam mutants (offset_of_col {→0,→1,`<=`→`>`}, offset_for_click {→0,→1},
  line_start {→Default}) are exactly T1-T4's targets; `line_layout→Default` UNVIABLE. app.rs
  `code_view_body`+`dispatch_action` mutants == 0 (skips held).

### Driven live proof (REQ-002/005 — mac unlocked, DATA-SAFE)
Bundled + `open target/Marley.app`; the #205 session restored the 4-file editor surface (selection.rs active).
**REQ-002:** clicked mid-line at x-frac 0.45 on line 10 (`anchor: CharOffset,`) → **the caret JUMPED from line 3
(restored) to line 10 AT the click column** (a zoom shows it landed inside "CharOffset"). **REQ-005:** clicked
line 10 + typed `X` → **`anchor: CharOffX│set,`** — the X inserted EXACTLY at the click point (mid-"CharOffset"),
NOT at offset 0; the column accuracy is spot-on (click on "CharOffset" → caret in "CharOffset"). The tab showed
the #252 dirty ● on type. **DATA-SAFETY honored: NO ⌘S ever sent → `git status` confirms selection.rs (typed
into) is PRISTINE on disk;** only the #254 source I authored is modified. (Note: the FIRST app instance had a
per-launch keyboard-focus flake — typing didn't reach the editor — but a fresh relaunch typed fine, and the click
handler itself worked on BOTH; a harness launch-focus quirk, NOT a #254 defect. Captures: m254_00/01/05/06.)

### Gate
`scripts/gates.sh --diff` → **GATE GREEN [diff] — 15 passed, 0 failed** (gate:4 coverage ≥100%, gate:5 mutation
MSI ≥100% on the pure seam, gate:1 fmt, gate:6 miri, gate:15 visual). Finished ~180s — NO real-PTY wedge this
run (the #253-flake watchdog was armed but not needed). Receipt `9a1da473…` matches the current `.rs` state → the
commit gate will pass.

**Phase 4 status: Validate PASS — GATE GREEN [diff]. Ready for Phase 5 — Complete.**
