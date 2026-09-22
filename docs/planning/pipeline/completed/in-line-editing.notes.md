# in-line editing — Notes

- **Forge ticket:** #28 `1a5d1fa1-01e1-417c-b216-7d787df93adf`
- **AAR:** `7e6ede9d-b180-4655-9e1f-241ff73808b4`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-028-in-line-editing.md
- **Pipeline spec:** in-line-editing.spec.md

## Phase 1 — Plan
- **Request:** forge #28 (M1.D "The Daily Driver" seq-1, auto-approved) — make the prompt a real
  editable line. The small enabler for seq-2 (history) + seq-6 (raw mode).
- **Classification / tier:** work pipeline, `feature` — one small slice (input.rs pure additions +
  the app.rs shim mapping/render). marley_app only.
- **Forge recall (§18.3) + discovery (the load-bearing facts):**
  - `marley_editor::movement` ALREADY EXPORTS all six motions (verified `crates/editor/src/lib.rs`
    21-24): `move_char_left`, `move_char_right`, `move_word_left`, `move_word_right`,
    `move_line_home`, `move_line_end` — each `(&Buffer, CharOffset) -> CharOffset`, cov-100/MSI-100
    from M1.A. So #28 WIRES them; it does NOT reimplement motion (D1).
  - `input.rs`: `Key::{Char,Backspace,Enter,Other}` + `apply_key` — Char inserts at caret
    (advancing), Backspace deletes left, Enter submits. Insert/backspace ALREADY work at an
    interior caret (they edit at `caret`), just never exercised there — a test pins it.
  - `Buffer` exposes `len_chars()`, `text()`, `text_in_range()`, `char_to_byte()` — enough for
    DeleteForward's `caret < len` guard + the split.
  - **Render gap (app.rs:517):** `format!("\u{258f}{}", state.buffer.text())` draws the `▏` caret
    at the START, before the text — so even with motion, the cursor won't visually move. #28 needs
    a shim render fix + a PURE `split_at_caret` helper (D3) to paint `before ▏ after`.
- **Decisions:** D1–D4 in the spec (reuse movement; motions → Edited, no Moved variant; the
  split helper; preserve the in-range-caret contract).
- **Open questions for Design:** where `split_at_caret` lives (input.rs vs terminal_view.rs);
  whether to add a `Moved` KeyOutcome now or defer to seq-2's text-compare (leaning defer, D2);
  the exact gpui keystroke names for the modifiers (alt-arrow for word, fn+delete for
  DeleteForward — confirm at implement via cargo-check against the existing "backspace"/"enter"
  precedent); whether Home/End should be line-home/end (single-line prompt → whole-line) — yes,
  the prompt is one line.
- **AAR id:** `7e6ede9d-b180-4655-9e1f-241ff73808b4`.

## Phase 2 — Design

### Architecture / approach
PURE (input.rs):
```rust
pub enum Key { Char(char), Backspace, Enter,
               Left, Right, WordLeft, WordRight, Home, End, DeleteForward,  // NEW
               Other }

// apply_key new arms — thin delegations to the (tested, internally-clamping) movement fns:
Key::Left      => { *caret = move_char_left(buffer, *caret);  Edited }
Key::Right     => { *caret = move_char_right(buffer, *caret); Edited }
Key::WordLeft  => { *caret = move_word_left(buffer, *caret);  Edited }
Key::WordRight => { *caret = move_word_right(buffer, *caret); Edited }
Key::Home      => { *caret = move_line_home(buffer, *caret);  Edited }
Key::End       => { *caret = move_line_end(buffer, *caret);   Edited }
Key::DeleteForward => {
    let at = caret.as_usize();
    if at < buffer.len_chars() {
        buffer.edit(*caret..CharOffset::from(at + 1), "", EditOrigin::Human);  // caret unchanged
        Edited
    } else { Ignored }
}
// Char/Backspace arms UNCHANGED — they already edit at `caret` (interior works); tests pin it.

pub fn split_at_caret(text: &str, caret: CharOffset) -> (String, String) {
    let n = caret.as_usize();
    (text.chars().take(n).collect(), text.chars().skip(n).collect())  // multibyte-safe
}
```
Every motion fn clamps internally (verified movement.rs: `saturating_sub` / `.min(len_chars)` /
loop-bounded by 0 and len), so `apply_key` adds NO bounds logic — the mutation surface is "the
RIGHT fn per key + `Edited`" and DeleteForward's guard + range.

SHIM (app.rs, existing exclude): `key_from_keystroke` maps the new gpui keys — "left"/"right" →
Left/Right (Word* when `alt`), "home"/"end" → Home/End, "delete" (forward-delete / fn+delete) →
DeleteForward. (The existing arm returns `Key::Other` for cmd/ctrl chords — unchanged; the motion
keys carry no cmd/ctrl.) Render (app.rs:517): replace `format!("▏{}", text)` with
`let (before, after) = split_at_caret(&text, caret); … .child(before).child("▏").child(after)` so
the caret paints in place.

### Decisions (+ the Phase-1 opens resolved)
- D-2.1 `split_at_caret` lives in input.rs (prompt-view logic, beside `apply_key`).
- D-2.2 Motions → `Edited`; NO `Moved` variant (D2 stands) — seq-2 will compare buffer text.
- D-2.3 Home/End = line-home/line-end (the prompt is single-line, so whole-line) — `move_line_*`
  already handle multi-line correctly if a paste ever adds a newline.
- D-2.4 DeleteForward maps from the "delete" key (macOS forward-delete / fn+Delete); the plain
  Backspace key stays "backspace". Confirm the exact gpui key strings at implement (write-then-
  cargo-check against the "backspace"/"enter" precedent).

### File manifest
- M `crates/marley_app/src/input.rs` — the 7 new `Key` variants, the 7 `apply_key` arms,
  `split_at_caret`, + unit tests.
- M `crates/marley_app/src/app.rs` — `key_from_keystroke` maps the new keys; the prompt render
  uses `split_at_caret` for the in-place caret.
- M `crates/marley_app/src/lib.rs` — export `split_at_caret` if a test/consumer needs it (else
  keep crate-private; the tests are in-module).
- M `docs/specs/SPEC-app-shell.spec.md` — the in-line-editing clause (R35: motion + interior edit
  + caret-split render) + AC row + Test-Plan + Mutation-Targets. CHANGELOG; arch doc at complete.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `apply_key_motions_delegate_to_the_right_movement` — over an ASYMMETRIC fixture ("foo bar", caret 5): Left→4, Right→6, WordLeft→4, WordRight→7, Home→0, End→7; EACH returns `Edited` and leaves `buffer.text()` unchanged (a per-key caret assertion where left≠right kills the delegation swaps) | unit |
| REQ-002 | `delete_forward_deletes_at_caret_and_guards_end` — "abc" caret 1 → text "ac", caret 1, `Edited`; caret 3 (end) → `Ignored`, text "abc" (kills the `<`→`<=`/`==` guard mutant + the range) | unit |
| REQ-003 | `char_and_backspace_edit_at_interior_caret` — "ac" caret 1 + Char('b') → "abc" caret 2; "abc" caret 2 + Backspace → "ac" caret 1 (interior, not end) | unit |
| REQ-004 | `split_at_caret_ascii_multibyte_and_ends` — ("abc",1)→("a","bc"); a multibyte string ("héllo",2)→("hé","llo") [char not byte]; caret 0 → ("","abc"); caret len → ("abc","") (kills take/skip off-by-one + char-vs-byte) | unit |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the `key_from_keystroke` mapping + the render (app.rs — existing exclude).

### Risks
- gpui forward-delete key string: macOS "delete" is backspace; forward-delete may be "delete" with
  a fn modifier or a distinct key — resolved at implement by cargo-check + the keystroke precedent
  (worst case DeleteForward is unmapped until confirmed; the pure arm is still tested).
- `buffer.edit` with `EditOrigin::Human` on DeleteForward — matches the existing insert/backspace
  origin (consistent; seq-2/agent-injection cares about origin, human typing is Human).

## Phase 3 — Implement
- **Built (per manifest):** input.rs — 7 new `Key` variants + the 7 `apply_key` arms (motion
  delegating to `marley_editor::{move_char_left,move_char_right,move_word_left,move_word_right,
  move_line_home,move_line_end}`, each → `Edited`; `DeleteForward` guards `caret < len_chars`,
  edits `caret..caret+1`) + `split_at_caret` (chars take/skip); app.rs — `key_from_keystroke` maps
  backspace/enter/delete + left/right (alt→Word*) + home/end (the match returns early per arm; the
  cmd/ctrl→Other + key_char fallthrough unchanged), and the prompt render now paints
  `{before}▏{after}` via `split_at_caret`; lib.rs exports `split_at_caret`. SPEC-app-shell R35 +
  AC row 35 + Test-Plan + Mutation-Targets; CHANGELOG.
- **Deviations from design:** none. (The render uses a single `format!("{before}▏{after}")` child
  rather than a 3-child flex-row — simplest, paints the caret in place; noted.)
- **Verification at this phase:** `cargo check -p marley` 0 errors; `cargo fmt`; 67 existing lib
  tests pass. The R35 unit suite is Phase 4.

## Phase 3.5 — Inspect
- **Critic run:** 1 correctness critic (56/56 real-`apply_key`/`split_at_caret`/movement probes +
  a read of gpui-0.2.2 mac/events.rs key names + code-traces).
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | MED | The refactored `key_from_keystroke` matched the nav keys (left/right/home/end/delete) and `return`ed BEFORE the `control||platform → Other` gate → `cmd+Left`/`cmd+Delete` triggered prompt motion instead of `Other`, contradicting the design invariant ("motion keys carry no cmd/ctrl") + the code comment. No crash (motions clamp), and cmd-letter chords still → Other (keymap intact), but a silent behavior change on a mutants::skip'd path. | REAL | Restructured: backspace/enter first (modifier-agnostic, as original) → then the `control||platform → Other` gate → THEN the nav/delete match (so `alt+Left`→WordLeft still works, `cmd/ctrl+nav`→Other). Comment corrected. |
  | F2 | LOW | The planned R35 delegation-test fixture `"foo bar"@5` collides (Left==WordLeft==4, WordRight==End==7) → couldn't kill a Left↔WordLeft / WordRight↔End swap (defense-in-depth only; NOT an MSI-gate threat — cargo-mutants doesn't synthesize fn-name swaps, and the real return-Ignored/drop-assignment mutants ARE killed since every motion moves off caret 5). | REAL (test-design) | SPEC test-plan fixture changed to ALL-DISTINCT `"xx aaaaa yy"@5` → L4/R6/WL3/WR8/H0/E11 (pairwise-distinct); the Phase 4 test uses it. |
- **Verified CORRECT (critic, all cleared):** multibyte DeleteForward is CHAR-based not byte-based
  (ropey char-indexed remove — a 😀 delete keeps valid UTF-8); motion delegation exact vs the
  direct movement-fn result (no double-move/off-by-one), each → Edited + text-stable;
  DeleteForward end-guard mutation-tight; split_at_caret ascii/multibyte/both-ends/caret>len-
  saturates (before+after==text always); interior insert/backspace regression clean; gpui key
  names confirmed (backspace/enter/left/right/home/end/delete + alt for word); plain char→Char,
  cmd-letter→Other (keymap intercept preserved). Provenance clean; the two collect()s in
  split_at_caret negligible for a prompt line.
- **Post-fix verification:** fmt/check clean; 67 lib tests pass; the F1 restructure keeps
  alt+Left→WordLeft while gating cmd/ctrl+nav→Other.

## Phase 4 — Validate
- **Tests added (input.rs, 4):** `apply_key_motions_delegate_to_the_right_movement` (the F2
  ALL-DISTINCT fixture `"xx aaaaa yy"@5` → L4/R6/WL3/WR8/H0/E11, each Edited + text-stable — kills
  every delegation swap), `delete_forward_deletes_at_caret_and_guards_end` (interior + end-guard +
  multibyte 😀 char-delete), `char_and_backspace_edit_at_interior_caret` (regression), and
  `split_at_caret_ascii_multibyte_and_ends` (é char-split, both ends, caret>len saturates).
- **Runs (actual):** `cargo nextest run -p marley` → the 4 new PASS (74 in marley); full
  workspace green; doctests green.
- **Visual (gate:15):** PASS headless (no new UI surface — the caret-split render rides the
  existing baseline).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15 first try**, 0 SLOW hangs (clean env
  — the stray marley from the demo was killed at pre-flight); coverage 100%; mutation **9 caught /
  0 missed → MSI 100.0%**; receipt written.
- **Pre-existing failures:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (done at implement); `app_shell.md` input.rs bullet
  extended (the motion delegation + split_at_caret + caret-in-place render). SPEC-app-shell R35
  landed at implement.
- **AAR capture:** `PR-claude-early-return-arms-before-a-guard-change-behavior-001` (the F1 lesson
  — a new early-return arm placed before an existing guard silently bypasses it; invisible on a
  mutants::skip'd shim); aar-submit `completed`. Win: the `marley_editor::movement` reuse made this
  a tiny wiring ticket (not a reimplement) — the recall-before-build paid off.
- **Ticket:** forge #28 → done; local doc → closed/; pipeline pair archived. First of the M1.D
  "Daily Driver" sprint (#28–33).
