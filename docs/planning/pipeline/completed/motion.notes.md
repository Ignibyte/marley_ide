---
pipeline_id: ed244310-92fb-4533-8eb5-3b755d9fa6aa
ticket: forge#257 (73daf3cd-5cc6-41a8-8013-7a53700ba122)
aar_id: e218ff82-b653-4edd-b813-0e381b9d79b5
---

# Editor motion parity (#257) — pipeline notes

## Phase 1 — Plan

**Intent.** Standard mac editor caret motion in the editor — Home/End, ⌥←→ (word), ⌘←→ (line), ⌘↑↓ (doc),
Up/Down (vertical) — each with a Shift-extend variant (#255), reusing marley_editor::movement.

**Classification.** Work pipeline, feature, LARGE — a motion subsystem, but the pure seam is TIGHT (two small
pure fns: `move_up`/`move_down` + `extend_or_go`); everything else reuses the existing movement fns + the #251/
#255/#256 branch idioms. Prefer one v1 slice.

**Discovery (confirmed):**
- movement.rs HAS move_char_left/right, move_word_left/right, move_line_home/end, extend_or_move (#255,
  Left/Right). MISSING vertical. buffer.rs has line_col/line_start/line_text/len_lines/len_chars — all the
  vertical helpers.
- input.rs Key: Char/Backspace/Enter/Left/Right/WordLeft/WordRight/Home/End/DeleteForward/Other — MISSING Up/Down.
  apply_editor_key acts on Char/Backspace/Left/Right; the MOTIONS are handled in the #251 BRANCH (like #255's
  Left/Right), not apply_editor_key.
- key_from_keystroke maps alt+left/right→Word, home/end→Home/End, left/right→Left/Right; control||platform →
  Key::Other; up/down UNMAPPED. The #251 branch guards !platform → ⌘-chords need a separate platform-chord branch
  (like #256).

**§20 (FILLED).** The universal macOS caret-motion convention (Cocoa text system; Warp follows it; no source
read). ⌘←→ line / ⌥←→ word / Home-End line / ⌘↑↓ doc / ↑↓ vertical / Shift+motion extend. Marley's own impl over
movement + #255; the vertical col reuses #250's line_col/grid.

**Locked-in decisions:** D1 move_up/move_down + extend_or_go in movement.rs; reuse the rest. D2 vertical = same col
on the adjacent row (clamped, current col, no goal-column memory). D3 extend_or_go(anchor,caret,target,shift) —
shift extends, else move+collapse; char-arrows keep extend_or_move. D4 the !platform motions route through the
#251 branch (target + extend_or_go); apply_editor_key unchanged. D5 the platform ⌘-motions get a new platform-
chord branch. See the spec.

**EARS AC:** REQ-001 Home/End/⌥word move · REQ-002 Up/Down vertical (clamped) · REQ-003 ⌘←→ line / ⌘↑↓ doc ·
REQ-004 Shift+motion extends / unshifted collapses · REQ-005 move_up/down + extend_or_go pure cov/MSI 100.

**Risks:** (1) the vertical col-clamp (up on row 0 / down on the last row / a shorter target line / multibyte —
the col is a CHAR index via line_col). (2) extend_or_move ↔ extend_or_go reconciliation (must NOT change #255's
char-arrow collapse-to-edge behavior — re-run the #255 tests). (3) the platform-chord branch guard (⌘-arrows only
for an editor tab; a terminal tab keeps its behavior). (4) key_from_keystroke up/down mapping (must not break the
terminal's up/down which needs the raw keycode — CONFIRM the editor Key mapping is separate from the PTY keycode
map). (5) mutants::skip on the branch shims.

**Gate note:** the #253 real-PTY flake (wedge under llvm-cov — ps -o pcpu,etime + kill + re-run). The #254/#255
harness note: a fresh instance may have a launch-focus flake (relaunch); a `focus` after a drag clears the
selection → drag+op in one drive call.

**Phase 1 status: Plan PASS — ready for Phase 2 — Design.**

---

## Phase 2 — Design

### Architecture / approach
Three small pure fns (movement.rs: `move_up`, `move_down`, `extend_or_go`) + shim (input.rs Key variants,
app.rs key mapping + two branch changes). Confirms §20 (the mac Cocoa caret-motion convention; clean-room).
Confirmed by reading: the #256 clipboard branch is app.rs:4424, the #255 motion arm app.rs:4546
(`Key::Left|Right → extend_or_move`), the #131 ⌘⌥-arrow pane-nav is separate, `key_from_keystroke`'s alt-arrows
at 1465.

**D1/D2 — move_up / move_down (movement.rs, PURE):**
```rust
pub fn move_down(buffer: &Buffer, off: CharOffset) -> CharOffset {
    let (row, col) = buffer.line_col(off);
    let last = buffer.len_lines().saturating_sub(1);
    let target = (row + 1).min(last);
    let target_len = buffer.line_text(target).chars().count();
    CharOffset::from(buffer.line_start(target).as_usize() + col.min(target_len))
}
pub fn move_up(buffer: &Buffer, off: CharOffset) -> CharOffset {
    let (row, col) = buffer.line_col(off);
    let target = row.saturating_sub(1);
    let target_len = buffer.line_text(target).chars().count();
    CharOffset::from(buffer.line_start(target).as_usize() + col.min(target_len))
}
```
Down on the LAST row → target=last=row → stays (col clamped to its own line); up on row 0 → target=0=row →
stays; a SHORTER target line → col clamps to `target_len` (caret at the target line end). `col` is a CHAR index
(line_col) + `chars().count()` → multibyte-correct. Reuses ropey's line model (consistent with #250/#254).

**D3 — extend_or_go (movement.rs, PURE):**
```rust
pub fn extend_or_go(anchor: Option<CharOffset>, caret: CharOffset, target: CharOffset, shift: bool)
    -> (Option<CharOffset>, CharOffset) {
    if shift { (Some(anchor.unwrap_or(caret)), target) } else { (None, target) }
}
```
Reconciliation (DECISION (a)): `extend_or_move` (#255) is LEFT UNCHANGED — it keeps its collapse-to-EDGE nuance
for an unshifted char-arrow on a selection. `extend_or_go` is ADDITIVE (the new motions go straight to the
computed target). The #255 char-arrow tests stay green (no touch).

**D4 — the #251 branch motion arm (app.rs:4546, after `Key::Left|Right`):**
```rust
Key::Home | Key::End | Key::WordLeft | Key::WordRight | Key::Up | Key::Down => {
    let target = match key {
        Key::Home => move_line_home(buffer, *caret),
        Key::End => move_line_end(buffer, *caret),
        Key::WordLeft => move_word_left(buffer, *caret),
        Key::WordRight => move_word_right(buffer, *caret),
        Key::Up => move_up(buffer, *caret),
        Key::Down => move_down(buffer, *caret),
        _ => *caret,
    };
    let (na, nc) = extend_or_go(*anchor, *caret, target, shift);
    *anchor = na;
    *caret = nc;
}
```
`shift` is already in scope (#255). Imports: add `move_up, move_down, extend_or_go` (move_line_home/end/word_*
already imported? — CONFIRM at implement; add if missing). `key_from_keystroke` gets `"up" => Key::Up`,
`"down" => Key::Down` (after the home/end arms in the !platform match). Key::Up/Down are NEW input.rs variants.
apply_editor_key UNCHANGED (the branch intercepts the motions before it).

**KEY-INDEPENDENCE (D4 risk):** `key_from_keystroke` → the editor `Key` enum, consumed ONLY by the #251 editor
branch (guarded `editor().is_some()`). The PTY keycode map (KeyCode::Up/Down ~app.rs:1410, for the terminal
scrollback/history) is SEPARATE. So adding Key::Up/Down affects ONLY the editor path — a terminal tab skips the
#251 branch (editor() None) → its up/down (scrollback) is unchanged.

**D5 — the platform ⌘-motion branch (app.rs, near the #256 clipboard branch, shim):**
```rust
if event.keystroke.modifiers.platform
    && !event.keystroke.modifiers.alt      // ⌘⌥-arrow stays the #131 pane-nav
    && !event.keystroke.modifiers.control
    && matches!(event.keystroke.key.as_str(), "left" | "right" | "up" | "down")
    && view.shell.active_project().active_tab().editor().is_some()
{
    let shift = event.keystroke.modifiers.shift;
    if let Some(surface) = view.shell.active_project_mut().active_tab_mut().editor_mut() {
        let (buffer, caret, anchor) = surface.active_buffer_caret_anchor_mut();
        let target = match event.keystroke.key.as_str() {
            "left" => move_line_home(buffer, *caret),
            "right" => move_line_end(buffer, *caret),
            "up" => CharOffset::zero(),
            "down" => CharOffset::from(buffer.len_chars()),
            _ => *caret,
        };
        let (na, nc) = extend_or_go(*anchor, *caret, target, shift);
        *anchor = na;
        *caret = nc;
        cx.notify();
    }
    return;
}
```
`!alt` keeps ⌘⌥-arrow as the #131 pane-nav (grep confirms plain ⌘-arrows are NOT keymap actions + the pane-nav
is ⌘⌥). Editor-guarded → a terminal tab keeps its ⌘-arrow behavior. `CharOffset::zero()` = doc start;
`len_chars()` = doc end. shift extends.

### File manifest
| File | Change |
|---|---|
| crates/editor/src/movement.rs | ADD `move_up`, `move_down`, `extend_or_go` (pure). |
| crates/editor/src/lib.rs | export `move_up, move_down, extend_or_go`. |
| crates/marley_app/src/input.rs | ADD `Key::Up`, `Key::Down` variants. |
| crates/marley_app/src/app.rs | `key_from_keystroke` up/down; the #251-branch motion arm; the platform ⌘-motion branch. SHIM (render skip'd). |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | movement `move_down`/`move_up` on "abc\ndefgh\nij": down (row0 col2 → co6); down onto a SHORT line (row1 col4 → co12, clamped to line 2 end); down on the LAST row → stays (co11 → co11); up (row1 col2 → co2); up on row 0 → stays; a MULTIBYTE fixture (col is a char index) | REQ-002 |
| T2 | movement `extend_or_go`: `(None, co5, co2, true)` → `(Some(co5), co2)`; `(Some(co3), co5, co8, true)` → `(Some(co3), co8)` [anchor pinned]; `(None, co5, co2, false)` → `(None, co2)`; `(Some(co3), co5, co8, false)` → `(None, co8)` [collapse] | REQ-004 |
| T3 | the #255 char-arrow tests (extend_or_move) still pass — extend_or_move UNCHANGED | regression |
| T4 (driven) | Home/End move to line start/end; ⌥←→ jump by word; Up/Down move vertically; Shift+motion highlights a range; ⌘←→ jump to line start/end | REQ-001/002/003/004 |

**Uncoverable/shim:** the key mapping + the two branches are shim (render skip'd; driven T4 + review). move_up/
move_down/extend_or_go are the cov/MSI-100 seam.

**Risks:** (1) the vertical col-clamp edges (row 0 / last / short line / multibyte) — T1. (2) extend_or_move
UNCHANGED (#255) — T3. (3) the ⌘-motion `!alt` guard (pane-nav survives) + editor-guard (terminal ⌘-arrow
survives) — review + driven. (4) key_from_keystroke Up/Down independent from the PTY keycode — review. (5)
mutants::skip on the branches (render skip'd).

**Phase 2 status: Design PASS — move_up/move_down/extend_or_go + the key mapping + the #251 motion arm + the
platform ⌘-motion branch (!alt) + T1-T4 locked; extend_or_move UNCHANGED. Ready for Phase 3 — Implement.**

---

## Phase 3 — Implement (PASS)

Built to the manifest; ONE deviation (an exhaustive-match arm the design didn't flag):

- **crates/editor/src/movement.rs** — `move_down`/`move_up` (same col on the adjacent row via line_col/line_start/
  line_text, clamped to `[0,len_lines-1]` + the target line's char count) + `extend_or_go(anchor, caret, target,
  shift)` (shift → pin+target; else None+target). Exported via lib.rs. `extend_or_move` (#255) UNCHANGED.
- **crates/marley_app/src/input.rs** — `Key::Up`/`Key::Down` variants.
  - DEVIATION: `apply_key` (the prompt engine) matches `Key` EXHAUSTIVELY (no wildcard) → the new Up/Down broke
    it (E0004). ADDED `Key::Up | Key::Down => KeyOutcome::Ignored` (vertical is meaningless in the single-line
    prompt; the editor handles Up/Down in the #251 branch, not apply_key). `apply_editor_key` has a `_` wildcard
    → needed no change.
- **crates/marley_app/src/app.rs (SHIM — render skip'd):** `use marley_editor::{… move_down, move_line_end,
  move_line_home, move_up, move_word_left, move_word_right, extend_or_go, extend_or_move …}` (added the motion
  fns). `key_from_keystroke`: `"up"→Key::Up`, `"down"→Key::Down` (the !platform match; only the #251 editor branch
  consumes them → a terminal tab's up/down [scrollback] is untouched). The #251 branch: a NEW arm
  `Key::Home|End|WordLeft|WordRight|Up|Down` → compute the target via the movement fn → `extend_or_go`
  (Left/Right keep #255's `extend_or_move`). A NEW platform ⌘-motion branch (before the #256 clipboard branch):
  `platform && !alt && !control && key∈{left,right,up,down} && editor()` → ⌘←=line_home / ⌘→=line_end /
  ⌘↑=CharOffset::zero / ⌘↓=len_chars → `extend_or_go` → return. `!alt` keeps ⌘⌥-arrow as the #131 pane-nav; a
  terminal tab is unaffected (editor-guarded).

**Checks:** `cargo fmt` clean; `cargo check -p marley_editor` + `-p marley --all-targets` clean. **skips 0 + app.rs
total mutant count UNCHANGED at 6** (the branches are in `render`, skip'd → 0 new live app.rs mutants). Pure-seam
mutants for Phase 4: move_down {Default, `+`→`-`/`*` on the `row+1` and the `line_start+col`}, move_up {Default,
`+`→`-`/`*` on `line_start+col`}, extend_or_go {tuple Default pair}. **The #255 char-arrow tests
(`extend_or_move`) stay GREEN — extend_or_move UNCHANGED.** No unwrap.

**Phase 3 status: Implement PASS — ready for Phase 3.5 — Inspect.**

---

## Inspect (Phase 3.5) — PASS (0 code fixes)

2 critics (pure movement · branches/guards). **Both: ZERO code defects — the implementation is correct.** No
failure-record (no bug).

### Critic 1 — pure movement (all CONFIRMED)
- (a) move_down: row-clamp (last row → stays) + col-clamp (short target line → its end) traced on "abc\ndefgh\nij".
  (b) move_up: row 0 → stays (saturating_sub) + col-clamp. (c) MULTIBYTE char-correct (line_col's col + line_text
  .chars().count() are CHAR units; "aé\nb😀c" move preserves the char column). (d) extend_or_go: shift pins the
  anchor (unwrap_or the caret) + target; else None+target. (e) extend_or_move (#255) UNCHANGED — the diff is
  purely additive; the 2 #255 char-arrow tests pass; app.rs still routes Left/Right through extend_or_move.
- **VALIDATE TRAPS (the #256 lesson, MANDATORY):** the pure mutants are 10 (move_down/up Default + `+`→`-`/`*`;
  extend_or_go tuple Default). To kill NON-degenerately: (1) `+`→`*` on `line_start+col` — pick a target line with
  `line_start ≥ 4` (NOT 0 or 2), so `line_start+col ≠ line_start*col` [move_down(co(8))→co(12): 10+2≠10*2].
  (2) `row+1`→`-` — start move_down on row≥1 (a row-0 start catches it only by underflow PANIC, unclean)
  [move_down(co(8)), row1]. (3) move_up `+`→`-` — use a move_up whose TARGET line ≠ 0 (line_start≠0) [move_up
  from row 2 → target row 1]. (4) extend_or_go — an EXPLICIT `(Some(co3),co5,co8,true)==(Some(co3),co8)`
  assertion (the anchor-pin's `unwrap_or` is a method call → UNMUTATED → not guarded by a mutant; #255 analog).
- LOW: the anchor-pin unguarded by mutation → the explicit assertion above. INFO: no tests yet (validate).

### Critic 2 — branches + guards (all PASS)
- (f) the #251 motion arm: each key → the right movement fn; shift extends via extend_or_go; Left/Right (#255
  extend_or_move) + the `_` edit arm UNCHANGED; E0499-free. (g) the ⌘-motion branch: **⌘⌥-arrow pane-nav SURVIVES**
  (`!alt` → it falls to the keymap `chord(platform,alt,"left")→"focus-left"`); editor-guarded (terminal skips);
  disjoint from the #256 clipboard (arrows vs c/x/v); ⌘↑=zero / ⌘↓=len_chars; no unwrap. (h) **key_from_keystroke
  Up/Down independence + NO PROMPT REGRESSION:** the terminal/prompt up/down is intercepted at app.rs:4689
  (history recall) + returns BEFORE key_from_keystroke (4709) → prompt ↑/↓ history UNCHANGED; the editor path
  consumes them at the #251 branch; apply_key Up|Down→Ignored is equivalent to the old Other→Ignored. (i) render
  skip → mutants 6 UNCHANGED, grep 0; no unwrap; imports used; clean-room OK.
- LOW-1 (premise correction, NO code change): key_from_keystroke has 2 callers (4570 editor + 4709 prompt), not
  1; safety holds (up/down never reaches 4709 — intercepted at 4689). Informational.

**Phase 3.5 status: Inspect PASS — 0 code fixes. Ready for Phase 4 — Validate (the pure-seam tests with the
non-degenerate trap values + the explicit anchor-pin assertion per critic 1).**

---

## Phase 4 — Validate — PASS

### Tests added (pure seam — movement.rs, cov/MSI 100)
Three `#[cfg(test)]` unit tests on the fixture `F="abc\nzé😀w\ndef"` (l0 "abc" start 0; l1 "zé😀w" chars 4..=7
start 4; l2 "def" chars 9..=11 start 9), with critic-1's NON-DEGENERATE trap values:
- **`move_down_holds_the_column_clamps_short_lines_and_stays_on_the_last_row`** — move_down(co(2))→co(6)
  [row0→1, multibyte target line]; move_down(co(6))→co(11) [row1 start, line_start 9 → the CLEAN row≥1 value-kill:
  `row+1`→`-` gives 0≠11 no panic, `+`→`*` gives 18≠11]; move_down(co(11))→co(11) [last-row clamp stays];
  move_down(co(7))→co(12) [long col onto short line → its end].
- **`move_up_holds_the_column_and_stays_on_row_zero`** — move_up(co(11))→co(6) [target l1 line_start 4≠0 → `+`→`-`
  gives 2≠6 no underflow]; move_up(co(2))→co(2) [row 0 saturating stays]; move_up(co(7))→co(3) [clamp].
- **`extend_or_go_pins_the_anchor_on_shift_and_clears_it_otherwise`** — (None,co5,co2,true)→(Some(co5),co2);
  (Some(co3),co5,co8,true)→(Some(co3),co8) [EXPLICIT anchor-pin, co3≠caret co5 — the `unwrap_or` method-call guard];
  (None,co5,co2,false)→(None,co2); (Some(co3),co5,co8,false)→(None,co8).

### Coverage fix (gate:4 caught it — §0 fix-at-source, no suppression)
First `--diff` run went RED on gate:4 only (coverage 99.62% on input.rs): the new `apply_key`
`Key::Up | Key::Down => Ignored` arm was uncovered (no test drove Up/Down through the single-line prompt path).
FIX: extended the `other_key_is_ignored` test →
`other_and_vertical_keys_are_ignored_in_the_single_line_prompt`, looping `[Key::Other, Key::Up, Key::Down]` and
asserting each → `KeyOutcome::Ignored` with the buffer/caret untouched (byte-equivalent to the pre-#257
Other→Ignored, per critic-2's (h) equivalence). Re-ran → 100%.

### Test run (actual)
- 3 new movement.rs fns: `3 tests run: 3 passed`.
- #255 `extend_or_move` (UNCHANGED): `2 tests run: 2 passed`.
- Full `marley_editor`: `49 tests run: 49 passed`. Full `marley`: `364 tests run: 364 passed, 2 skipped`.
- Mutation list: movement.rs = the 10 expected pure mutants (5 move_down, 3 move_up, 2 extend_or_go), all killed
  by T1/T2/T3. app.rs: `code_view_body|dispatch_action` count = 0, total = 6 (UNCHANGED from #256 — the two
  branches live in the render-skip'd on_key_down).

### Driven live proof (REQ-002 vertical, the most visible new behavior)
Bundled + launched the fresh binary (window 1771×808, machine UNLOCKED). The #205 session restored an editor tab
(`selection.rs`). drive.swift inventory: it has bare `up`/`down`/`left`/`right` (kc 126/125/123/124) + `cmdopt:`
(⌘⌥ pane-nav) but NO bare ⌘-arrow / ⌥-arrow / shift-arrow / Home / End verbs — so the drivable NEW motion is the
plain vertical Up/Down. Clicked `clickat:0.36,0.47` → the caret landed at LINE 21 col 0 (a clear vertical bar left
of `pub fn caret`), editor focused. Then, each in ONE drive call (focus held — the harness `focus`/`clickat`
gotcha):
- **`clickat:0.36,0.47 down down down down down` → caret at LINE 26** (21+5). READ the capture: caret bar left of
  the closing `}`. ✓
- **`clickat:0.36,0.47 up up up up up` → caret at LINE 16** (21−5). READ the capture: caret bar at `pub fn new`. ✓
The column held down the left edge; ±5 exactly from the click reference. REQ-001 (Home/End/⌥←→), REQ-003 (⌘←→
line, ⌘↑↓ doc), REQ-004 (shift-extend) are NOT drivable (drive.swift lacks those verbs) → carried by the pure
units (move_line_home/end/word_left/right are #255-tested; move_up/down + extend_or_go are the new units above) +
critic-2's mechanism trace (each key → the right fn → extend_or_go; the ⌘-branch `!alt`/editor guards). DATA-SAFE:
motion is read-only (no buffer edit, no ⌘S); `git status` confirmed `selection.rs` PRISTINE after; only the 4
intended #257 files (movement.rs, lib.rs, app.rs, input.rs) show as modified.

### Gate
`scripts/gates.sh --diff` → **GATE GREEN [diff]** — 15 passed, 0 failed (incl gate:4 coverage 100% on movement.rs
+ input.rs, gate:5 mutation MSI 100%, gate:15 visual/AX). No real-PTY wedge this run. No pre-existing failures.

**Phase 4 status: Validate PASS — gate green. Ready for Phase 5 — Complete.**

---

## Phase 5 — Complete — PASS

- **Docs (§21):** CHANGELOG.md — added the #257 `### Added` entry above #256. docs/marley_architecture/app_shell.md
  — added the M15 #257 bullet after #256 (the caret-motion keymap: the 2 pure fns + the 2 shim branches + the
  guards + the driven proof).
- **Knowledge (forge wired):** aar-submit (aar_id e218ff82, outcome completed, effectiveness 5). LESSON reinforced
  (the #256 coverage-trap analog): adding a NEW arm to an exhaustive match [Key::Up|Down→Ignored in apply_key]
  introduces an uncovered PURE line even when the arm is a NO-OP — gate:4 caught it RED at 99.62%, fixed at source
  by looping the existing no-op test over [Other, Up, Down]. NO new failure/prevention record — the coverage miss
  was a validate-phase GATE CATCH (the gate is the guardrail), not a code defect; inspect was clean (0 code bugs).
  What worked: the tight pure seam (2 vertical-column fns + 1 shift-extend generalization) reusing #255's word/line
  fns + the #251/#256 branch idioms; critic-1's non-degenerate trap values [line_start≥4 so `+`≠`*`, a row≥1 start
  so `row+1`→`-` value-diverges without an underflow panic] made cov/MSI 100 first-try on the pure seam; the
  highest-value critic checks (⌘⌥-arrow #131 pane-nav survives via `!alt`; no prompt ↑/↓ regression — the prompt
  up/down is intercepted at the history-recall handler BEFORE key_from_keystroke) both confirmed by trace.
- **Ticket:** TICKET-257 open→closed; forge ticket-close 73daf3cd.
- **Archive:** motion.{spec,notes}.md active/→completed/.

**Phase 5 status: Complete PASS. Ready for /commit.**
