---
pipeline_id: 277fa58a-b83d-489b-aa6e-9c0c8ffca4a8
ticket: forge#251 (b97191e2-eff7-4db8-878a-90e7c1709fdc)
aar_id: 9269d495-736b-41dc-8db0-466860c4cb4c
---

# Notes — Editor input intercept (forge#251)

## Plan (Phase 1)

**Classification:** work pipeline, feature, MEDIUM (a tiny pure fn + a nuanced on_key_down routing branch).
AUTONOMOUS (chad's `/goal /work 249 to 258`). The marquee interaction — the first WRITE path to the editor
buffer; #250 (render) + #249 (model) make the edit VISIBLE.

**Intent:** typing does nothing in the editor today — keys leak to a hidden terminal (on_key_down routes to
workspace()'s first terminal; no editor-focus concept). #251 captures keys for the editor tab → the buffer.

**Discovery (read the code — anchors):**
- **`input::apply_key`** (input.rs:58) — the tested reuse core: `Key::Char(c)` inserts + caret+1; `Backspace`
  deletes-left (no-op at 0); `Enter` → `submit_line` → `KeyOutcome::Submit(line)` (NO buffer mutation — the
  prompt's behavior); `Left`/`Right` → `move_char_left`/`right` (char ±1, clamped); also WordLeft/Right/Home/End/
  DeleteForward. `enum Key` (input.rs:17): Char/Backspace/Enter/Left/Right/WordLeft/WordRight/Home/End/
  DeleteForward/Other. `enum KeyOutcome` (input.rs:44): Edited/Submit(String)/Ignored. **PURE, gpui-free.**
- **THE Enter DIVERGENCE:** apply_key's Enter SUBMITS (right for the prompt); the editor wants Enter⇒insert
  `\n`. Since `apply_key(Char('\n'))` inserts a newline + advances (the Char path), `apply_editor_key` = "map
  Enter→Char('\n'), else apply_key" — a 2-line wrapper, MAXIMAL reuse of the tested path (D1).
- **`on_key_down`** (app.rs:3953, `cx.listener(|view, event: &KeyDownEvent, _window, cx|)`) — the shell router.
  It handles overlays FIRST, each with an early `return` + `cx.notify()`: `naming_workflow` (3957, `!modifiers.
  platform && !modifiers.control` for printables + Enter/Esc/Backspace/Space), `renaming_tab` (4016), and
  (further down) finder/palette/etc., THEN the terminal/prompt routing. The #251 editor branch sits AFTER the
  overlay guards (they own the keyboard first) and BEFORE the terminal route.
- **The keystroke→Key parse:** the naming_workflow guard shows the idiom — `event.keystroke.key.as_str()` for
  named keys ("escape"/"enter"/"backspace"/"space"), a 1-char `key` with `!modifiers.platform &&
  !modifiers.control` for printables (via `keystroke.key_char`). The terminal/prompt route (further in
  on_key_down) builds a `Key` — the editor branch reuses/mirrors that parse. Design traces the exact prompt
  parse to reuse.
- **`active_buffer_mut()`/`active_caret_mut()`** (editor_surface.rs, #249) — the write handles the editor branch
  targets. `active_tab().editor()` → the surface (#237). #250's render already redraws buffer edits + the caret.

**THE CAPTURE CONTRACT (D2 — the one nuance):** the editor OWNS plain keys (no ⌘/ctrl) — printable inserts,
Backspace/Left/Right/Enter act, and an unhandled plain key (Up/Down for now) is still CAPTURED (swallowed, no
leak). ⌘/ctrl chords (⌘W/⌘P/⌘D) FALL THROUGH to the keymap (shortcuts must keep working — DON'T swallow them).
This mirrors the naming_workflow guard's `!modifiers.platform && !modifiers.control`.

**Decisions:** D1 apply_editor_key = Enter→Char('\n') else apply_key · D2 capture plain keys, ⌘/ctrl fall
through, overlays first · D3 reuse the prompt's Keystroke→Key parse · D4 no new focus flag (active-editor-tab +
no-overlay == focused).

**Risks / load-bearing:**
- **The Enter divergence** — the prompt's Enter⇒Submit MUST stay unchanged (only the editor branch maps Enter→
  `\n`). The pure test pins both (apply_key Enter still Submit; apply_editor_key Enter inserts `\n`).
- **⌘/ctrl fall-through** — if the editor branch swallows ⌘-chords, every shortcut breaks. The modifier guard is
  load-bearing; the driven proof types a char AND fires ⌘W/⌘P to confirm both.
- **The terminal path unchanged** — the editor branch is ADDED before the terminal route (an `if editor {…
  return }`); with a terminal tab active it's skipped → the terminal path is byte-unaffected.
- **Multi-line Left/Right** — `move_char_left`/`right` are char-offset ±1 (clamped to [0,len]) → they cross line
  boundaries naturally (Left at a line start → the previous `\n`); #250's line_col renders the caret at the
  right (row,col). So Left/Right are multi-line-correct without new movement code (Home/End/word ARE line-aware
  → deferred to #257).

**Test plan (finalized at design):** REQ-001/002/003/004 pure units on `apply_editor_key` (Char inserts +
caret+1 + Edited [NOT Submit]; Backspace deletes / no-op at 0; Left/Right clamp; Enter inserts `\n` + advances)
+ a guard that `apply_key` Enter STILL Submits (the prompt unchanged). cov/MSI 100. REQ-005 DRIVEN (type into the
editor → text + caret advance; ⌘W/⌘P still work; a terminal tab still types to the terminal) + review.

**Reference (§20):** Warp's focus routing (the focused surface owns the keystroke) + the #250 grid/caret capture;
the multi-line Enter⇒`\n` is a Marley file-editor concern (noted). Clean-room.

**AAR:** 9269d495-736b-41dc-8db0-466860c4cb4c (opened).

**Phase 1 status: Plan PASS — autonomous (M15 /goal). Ready for Phase 2 — Design.**

## Design (Phase 2)

**## Reference (§20) confirmed:** Warp's focus routing (the focused surface owns the keystroke). This design
MATCHES it — an active editor tab captures plain keys into its file buffer (like Warp's input owning its keys),
reusing the in-repo `input::apply_key` + `key_from_keystroke`; the caret advances one cell per char (the #250
grid/caret capture). Clean-room: observed routing behavior, no Warp source.

**Architecture.** A tiny PURE `apply_editor_key` (input.rs) + a routing branch in the app.rs `on_key_down` shim.
NO editor_surface/buffer change (`active_buffer_mut`/`active_caret_mut` exist from #249; the #250 render already
redraws edits + the caret).

### THE PURE apply_editor_key — RESOLVED (input.rs, D1 refined)
```
pub fn apply_editor_key(buffer: &mut Buffer, caret: &mut CharOffset, key: Key) -> KeyOutcome {
    match key {
        Key::Enter => apply_key(buffer, caret, Key::Char('\n')),           // multi-line: Enter ⇒ '\n', not Submit
        Key::Char(_) | Key::Backspace | Key::Left | Key::Right => apply_key(buffer, caret, key),  // the loop
        _ => KeyOutcome::Ignored,   // Home/End/word/Up/Down/DeleteForward: line-aware movement is #257 — a
    }                               // NO-OP here (NOT apply_key's single-line move_line_home on a multi-line buffer)
}
```
- Verified: `apply_key(Key::Char('\n'))` inserts "\n" + caret+1 + `Edited` (the Char arm, input.rs:60) → Enter⇒
  `\n` via the ONE tested path. REFINEMENT from plan D1: NOT a blanket `else apply_key` — Home/End/word delegate
  to `apply_key`'s SINGLE-LINE `move_line_home/end`/`move_word_*` which are WRONG on a multi-line file (jump to
  buffer-start/end), so #251 IGNORES them (deferred to #257). Only the 5 scoped keys act.
- `apply_key` is UNCHANGED — the prompt still calls it directly (Enter⇒Submit intact).

### THE on_key_down EDITOR BRANCH — RESOLVED (app.rs shim, D2/D3)
Placement: **immediately after the cockpit-chord dispatch (app.rs:4266-4276 `binding_from_keystroke` →
`keymap.action_for` → dispatch+return) and BEFORE the interactive/raw-PTY route (4277)**. This is exactly right:
- ⌘/ctrl cockpit chords (⌘W/⌘P/⌘D) are dispatched by the keymap FIRST (4266-4276, return) → shortcuts WORK.
- cmd-C/cmd-V (4230/4241, require `.platform`) already returned → the editor never sees them; a PLAIN c/v →
  the editor inserts Char('c')/('v') (correct).
- The editor branch sits BEFORE the raw-PTY route (4277) + Tab-complete (4304) + scroll (4312) + history ↑/↓
  (4335) + the cooked-prompt `apply_key` (4355) → when an editor tab is active, NONE of the terminal-prompt
  logic runs; plain keys are captured (no leak).
```
// #251: an active editor tab OWNS plain keys — edit its file buffer, don't leak to a terminal.
if view.shell.active_project().active_tab().editor().is_some()
    && !event.keystroke.modifiers.platform
    && !event.keystroke.modifiers.control
{
    let key = key_from_keystroke(&event.keystroke);   // REUSE the prompt's parser (app.rs:1439)
    if let Some(surface) = view.shell.active_project_mut().active_tab_mut().editor_mut() {
        apply_editor_key(surface.active_buffer_mut(), surface.active_caret_mut(), key);
    }
    cx.notify();
    return;
}
```
- **Guard `!platform && !control`** (load-bearing): the editor captures ONLY plain keys; ⌘/ctrl keys skip the
  branch → fall through (chords already handled above; ctrl keys → the raw route, pre-existing — ctrl-editing/
  copy-paste in the editor is #256, noted). This is the REQ-005 nuance.
- **The parse is REUSED:** `key_from_keystroke(&Keystroke) -> Key` (app.rs:1439) already exists (the prompt's
  parser) — NO extraction/refactor needed (lower risk; the editor branch just calls it). An unmapped key →
  `Key::Other` → `apply_editor_key` Ignores it (swallowed).
- **Borrow-dance** (mirror the #250 scroll fix): `active_tab().editor().is_some()` (immutable) + `key_from_
  keystroke` (no view borrow) → then `active_project_mut().active_tab_mut().editor_mut()` (mutable). Sequential,
  NLL-fine.

### File Manifest
| File | Change |
|---|---|
| crates/marley_app/src/input.rs | ADD `pub fn apply_editor_key` (Enter⇒Char('\n'); Char/Backspace/Left/Right⇒apply_key; else Ignored). PURE, cov/MSI 100. Tests-phase adds T1-T6. |
| crates/marley_app/src/app.rs | ADD the on_key_down editor branch (after the chord dispatch ~4276, before the raw route ~4277) — reuses `key_from_keystroke`, calls `apply_editor_key` on the surface. SHIM (the on_key_down listener is already coverage-excluded/masked). |

### Regression Test Plan
| # | Test (input.rs `#[cfg(test)]`) | Proves |
|---|---|---|
| T1 | `apply_editor_key(Enter)` on "ab" caret 2 → text "ab\n", caret 3, `Edited` (NOT Submit) | REQ-004 |
| T2 | `apply_editor_key(Char('x'))` caret 1 → inserts 'x' + caret 2 + `Edited` (delegates) | REQ-001 |
| T3 | `apply_editor_key(Backspace)` → deletes left; at offset 0 → `Ignored`, no change (delegates) | REQ-002 |
| T4 | `apply_editor_key(Left)`/`(Right)` → caret ±1 clamped to [0,len] (delegates) | REQ-003 |
| T5 | `apply_editor_key(Home)`/`(End)`/`(DeleteForward)`/`(WordLeft)`/`(Other)` → `Ignored`, buffer UNCHANGED (deferred keys are no-ops, NOT single-line motion) | REQ-004/005 (scope) |
| T6 | GUARD: `apply_key(Enter)` on "ab" STILL returns `Submit("ab")` (the prompt path unchanged) | REQ-005 |
| — | `cargo mutants --list -f input.rs` (post-impl) — the apply_editor_key match arms + the Char('\n') literal; kill with T1-T5. | MSI 100 |
| REQ-005 | DRIVEN + `cargo check` — editor captures plain keys (type→text+caret); ⌘W/⌘P still work; a terminal tab still types to the terminal. | driven + review |

**Uncoverable / shim:** the on_key_down editor branch (gpui listener, `Keystroke`) is shim — DRIVEN-validated
(type into the editor; boot restores an editor tab so no finder needed — the #250 finder-open quirk is moot).

**Risks:** (1) the Enter divergence — T1 + T6 pin both sides. (2) the ⌘/ctrl fall-through — the modifier guard;
driven fires ⌘W/⌘P to confirm chords still work. (3) placement before the raw route — confirmed by the 4266-4277
structure (chords first, then the editor, then raw/terminal). (4) ctrl-key-in-editor (copy/paste/ctrl-editing)
falls through for now → #256 (noted, out of scope).

**Phase 2 status: Design PASS — apply_editor_key (Enter⇒\n, 5 keys act, rest Ignored) + the on_key_down branch
placement (after chords 4276, before raw 4277, guarded !platform&&!control, reusing key_from_keystroke) + T1-T6
locked. Ready for Phase 3 — Implement.**

## Implement (Phase 3)

**Built (to the manifest):**
- **input.rs** — `pub fn apply_editor_key(buffer, caret, key) -> KeyOutcome` exactly as designed: `Key::Enter =>
  apply_key(.., Key::Char('\n'))`; `Char/Backspace/Left/Right => apply_key(.., key)`; `_ => Ignored`. `apply_key`
  UNCHANGED (the prompt's Enter⇒Submit intact).
- **app.rs** — the `on_key_down` editor branch inserted AFTER the cockpit-chord dispatch (`view.keymap.
  action_for` block) and BEFORE the `// Interactive/raw mode (R40)` route, guarded `editor().is_some() &&
  !modifiers.platform && !modifiers.control`; reuses `key_from_keystroke` (app.rs:1439) then `apply_editor_key`,
  `cx.notify()`, `return`. Import: added `apply_editor_key` to `use crate::input::{…}` (app.rs:71).

**Deviation from design (with reason):** the caller `apply_editor_key(surface.active_buffer_mut(),
surface.active_caret_mut(), key)` FAILED to compile — **E0499 double-mutable-borrow** (two `&mut self` method
calls on `surface` in one expression). The prompt route avoids this by borrowing two FIELDS
(`&mut state.buffer, &mut state.caret` — disjoint), which a method pair can't express. **FIX:** added
`EditorSurface::active_buffer_and_caret_mut(&mut self) -> (&mut Buffer, &mut CharOffset)` (a disjoint-field
borrow of the active `OpenFile`: `let f = &mut self.files[self.active]; (&mut f.buffer, &mut f.caret)`); the
caller does `let (buffer, caret) = surface.active_buffer_and_caret_mut(); apply_editor_key(buffer, caret, key)`.
A small new EditorSurface accessor (needs its own cov/MSI test at validate — mirror the #250 active_buffer
test). LESSON: two `&mut self` accessors can't be called in one expression (unlike two struct FIELDS) — an
input layer that needs buffer+caret together needs a combined `(&mut, &mut)` accessor.

**Compile:** `cargo check -p marley --all-targets` clean (the borrow-dance — immutable `editor().is_some()` →
`key_from_keystroke` (no view borrow) → mutable `editor_mut()` — is NLL-fine). Do NOT expand tests here (Phase 4
adds T1-T6 + the active_buffer_and_caret_mut test).

**Phase 3 status: Implement PASS — apply_editor_key + the on_key_down branch + active_buffer_and_caret_mut (the
double-borrow fix) all compile clean. Ready for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5)

2 parallel general-purpose critics (C1 = correctness + the key-routing contract; C2 = regression + reuse +
clean-room) + self-review. **0 CODE findings — the diff is correct** (both critics confirmed every claim). **1
HIGH = a coverage gap (validate's job, not an inspect code fix).** `cargo check` clean; `cargo nextest -p marley
input::` 12/12; clippy clean.

**Load-bearing checks — CONFIRMED CLEAN:**
- **C1 the routing contract:** `apply_editor_key` is total + correct (Enter→`apply_key(Char('\n'))` inserts a
  newline + `Edited`, NOT Submit; Char/Backspace/Left/Right delegate; Home/End/word/Up/Down/DeleteForward/Other
  → Ignored no-op — never `apply_key`'s motion). The branch PLACEMENT is right (AFTER the chord dispatch → ⌘W/
  ⌘P/⌘D still work; BEFORE the raw-PTY/Tab-complete/scroll/history/cooked-apply_key → no leak when an editor tab
  is active; cmd-C/V require `.platform` + returned earlier → a plain c/v inserts). The GUARD `editor().is_some()
  && !platform && !control` captures only plain keys; no plain-key GLOBAL is wrongly swallowed (plain Esc has no
  app-global; everything after the branch is terminal-prompt-only). The BORROW (`active_buffer_and_caret_mut` =
  disjoint fields; the immutable-is_some→mutable-editor_mut dance) is sound + compiles. The LIVE LOOP: the write
  targets `files[active].buffer`/`.caret` — the SAME fields the #250 render reads (`active_buffer()`/
  `active_caret()`) → typing shows new text + the caret advances (REQ-001). C1 also verified `move_char_left/
  right` are flat-char-offset (multi-line-safe, cross newlines).
- **C2 no regression + reuse:** `apply_key` UNCHANGED (the prompt's Enter⇒Submit intact — diff is purely
  additive after it); the terminal route (`focused_terminal_mut` + `apply_key` at ~4404) UNCHANGED (the editor
  branch only early-returns when an editor tab is active → a terminal tab skips it); the overlay guards
  (naming_workflow/renaming_tab/completion/palette/find) all still return FIRST (untouched). REUSE clean —
  `key_from_keystroke` reused (not duplicated), `apply_editor_key` reuses `apply_key`, `active_buffer_and_caret_
  mut` is the MINIMAL fix for the E0499 (the terminal writes two struct FIELDS; the editor's are behind
  `files[active]`, so a combined `(&mut,&mut)` accessor is idiomatic — inlining impossible, `.unwrap()` worse).
  No panic/unwrap; clean-room (in-repo apply_key + gpui + observed Warp focus-routing, no Warp source); clippy
  clean (gate:2 passes); the import is ordered + used.

**Finding folded (1 HIGH — deferred to VALIDATE, its job):**
- **F1 [HIGH] the two new #251 fns are uncovered → 3 viable mutants** (C2). `apply_editor_key` (input.rs) +
  `active_buffer_and_caret_mut` (editor_surface.rs) have NO test yet → the `--diff` gate would go MSI-RED +
  line-cov-RED. Viable mutants: `editor_surface.rs` → `(Box::leak(Box::new(Default::default())), …)` (Buffer +
  CharOffset both derive Default — VIABLE); `input.rs` → delete the `Key::Enter` arm; delete the
  `Char|Backspace|Left|Right` arm. (The `input.rs` body → `Default::default()` is UNVIABLE — `KeyOutcome` has no
  Default — the #203 rule.) **VERDICT real coverage-gap; NOT a code bug.** **FIX = validate deliverable
  (inspect does not write tests):** the design's T1-T6 for `apply_editor_key` (Enter→`\n`+Edited-not-Submit;
  Char/Backspace/Left/Right edit; Home/End/Other→Ignored-no-mutation) kill the input.rs mutants; ADD an
  `editor_surface` test that edits through `active_buffer_and_caret_mut`'s returned refs then asserts the REAL
  `active_buffer().text()`/`active_caret()` changed (kills the leaked-default mutant). ON THE VALIDATE RADAR.

**INFO (no action):** a spurious `cx.notify()` on a no-op editor key (Up/Down) — trivial repaint, harmless. The
outer `editor().is_some()` + inner `editor_mut()` re-check is a micro-nit (correct as-is; not folded — changing
correct code for a nit risks a bug). ctrl-key-in-editor (copy/paste/ctrl-editing) falls through → #256.

**No forge failure-record** — no shipped runtime bug (F1 is a caught-in-pipeline coverage gap validate closes).
Lenses covered: correctness, routing/capture contract, regression (prompt/overlay/keymap), reuse, clean-room,
coverage/mutation.

**Phase 3.5 status: Inspect PASS — 0 code findings (the diff is correct); 1 HIGH coverage-gap flagged for
validate (T1-T6 + the accessor test kill the 3 viable mutants). Ready for Phase 4 — Validate.**

## Validate (Phase 4)

**Tests added (close the inspect coverage gap):**
- **input.rs** (4 new `apply_editor_key` tests; T6 already existed): `editor_enter_inserts_newline_not_submit`
  (Enter → "ab\n", caret 3, `Edited` — NOT Submit), `editor_char_and_backspace_delegate`,
  `editor_left_right_move_and_clamp`, `editor_deferred_keys_are_ignored_noops` (Home/End/word/DeleteForward/
  Other → `Ignored`, buffer+caret UNCHANGED). T6 = the pre-existing `enter_submits_the_trimmed_line_without_
  mutating` (proves `apply_key`'s Enter STILL Submits — the prompt path untouched).
- **editor_surface.rs** (1): `active_buffer_and_caret_mut_edits_the_real_active_file` (edit BOTH refs → the
  REAL `active_buffer().text()`/`active_caret()` change — kills the leaked-default mutant).

**Runs (actual):** `cargo nextest run -p marley input:: editor_surface::` → all pass (incl the 5 new);
`cargo nextest run -p marley` → **354 passed, 2 skipped** (no regression).

**Mutation (`cargo mutants --list`):** the 3 VIABLE mutants confirmed + killed — input.rs `delete Key::Enter arm`
(killed by editor_enter), `delete Char|Backspace|Left|Right arm` (killed by editor_char), editor_surface.rs
`active_buffer_and_caret_mut -> (Box::leak(Default), Box::leak(Default))` (killed by the accessor test). The
`apply_editor_key -> Default::default()` body mutant is UNVIABLE (`KeyOutcome` has no Default — the #203 rule).
Gate ran the real mutants → **MSI ≥ 100%**.

**DRIVEN — REAL CAPTURE (mac unlocked; the app booted INTO an editor tab via session restore — no finder
needed):** THE MARQUEE INTERACTION, proven live:
1. **Type "HELLO"** (`drive.swift focus type:HELLO`) → line 1 became `HELLO//! PURE …` — the chars INSERTED at
   the offset-0 caret into the file buffer, and the cyan caret bar ADVANCED to after "HELLO" (col 5). The keys
   were CAPTURED by the editor (not leaked to a terminal), applied via `apply_editor_key`, and the #250 render
   redrew it live. **REQ-001 end-to-end.**
2. **Enter** → the line SPLIT: line 1 "HELLO", line 2 "//! PURE …" (pushed down), the caret dropped to line 2
   col 0. A newline insertion, NOT a submit. **REQ-004.**
3. **⌘P** (with the editor active) → the finder overlay OPENED (top match `.cargo/audit.toml`) → ⌘/ctrl chords
   fall through to the keymap. **REQ-005** (shortcuts still work). REQ-002/003 (Backspace/Left/Right) carried by
   the units. (The typing edited selection.rs's in-memory BUFFER only — unsaved/transient, #249's model doesn't
   persist buffer content, so the on-disk file is untouched; save is #252.)

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** 15/15 (cov ≥ 100% lines, MSI ≥ 100%, miri, visual).
The receipt for `/commit` is written. cargo-mutants at `--jobs 2` (the memory fix), no freeze.

**Pre-existing (not in scope):** the `block v0.1.6` future-incompat (transitive dep).

**Phase 4 status: Validate PASS — 5 new unit tests (cov/MSI 100 on apply_editor_key + active_buffer_and_caret_
mut), 354 marley tests green, THE MARQUEE (type into the editor + Enter⇒\n + ⌘P-still-works) driven-proven live,
GATE GREEN [diff]. Ready for Phase 5 — Complete.**
