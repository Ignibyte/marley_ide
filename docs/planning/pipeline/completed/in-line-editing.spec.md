---
pipeline_id: 5c244525-3d4c-4ef8-804f-8dc207c360ed
ticket: forge#28 (1a5d1fa1-01e1-417c-b216-7d787df93adf) · local docs/planning/tickets/open/TICKET-028-in-line-editing.md
aar_id: 7e6ede9d-b180-4655-9e1f-241ff73808b4
status: Phase 5 — Complete PASS
title: in-line editing — cursor movement + edit within the prompt line
type: feature
milestone: M1.D
references:
  - crates/marley_app/src/input.rs (Key / apply_key — the pure prompt seam)
  - crates/editor/src/movement.rs (move_char/word/line_* — REUSED, already tested)
  - docs/specs/SPEC-app-shell.spec.md (gains the in-line-editing clause)
---

## Title
Make the input prompt a real editable line. Today `apply_key` handles only Char/Backspace/Enter,
and arrows → `Key::Other` → Ignored, so the caret only moves by insert/backspace and edits land at
the end. Extend `Key` + `apply_key` to move the caret (←/→, word, home/end) and edit at ANY
interior position — WIRING the already-shipped, already-tested `marley_editor::movement` fns, not
reimplementing motion. Plus the render fix so the caret is actually drawn at its position.

## Scope
### In
- `crates/marley_app/src/input.rs` — extend `Key` with `Left`, `Right`, `WordLeft`, `WordRight`,
  `Home`, `End`, `DeleteForward`; grow `apply_key`:
  - motion arms delegate to `marley_editor::{move_char_left, move_char_right, move_word_left,
    move_word_right, move_line_home, move_line_end}` — set `*caret` to the result, return `Edited`.
  - `DeleteForward` deletes the char AT the caret (`buffer.edit(caret..caret+1, "")` when
    `caret < len_chars`, caret unchanged; else `Ignored`).
  - insert-at-interior + backspace-at-interior already work (edit at caret) — a test PINS them.
- `crates/marley_app/src/input.rs` (or terminal_view.rs) — a PURE `split_at_caret(text, caret)
  -> (String, String)` that splits the prompt text at the caret's CHAR offset (multibyte-safe:
  `chars().take/skip`), so the caret renders in place.
- `crates/marley_app/src/app.rs` (shim) — `key_from_keystroke` maps the new keys ("left"/"right"/
  "home"/"end", alt-arrow → word, fn+delete → DeleteForward); the prompt render draws
  `before ▏ after` via `split_at_caret` instead of `▏ + text` (caret always at start today).
- SPEC-app-shell: the in-line-editing clause (motion + interior edit + the caret-split render) +
  AC/Test-Plan/Mutation-Targets. CHANGELOG + arch doc.

### Out (explicitly deferred)
- Command history (seq-2), selection/multi-cursor (SPEC-editor depth / a later sprint),
  kill/yank, mouse caret placement, undo/redo. Raw-mode key encoding is seq-6.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — REUSE `marley_editor::movement` (the six motions are public + cov-100/MSI-100 from M1.A).
  `apply_key`'s new arms are thin delegations; the mutation surface is "the RIGHT motion fn is
  called + the Edited outcome" + DeleteForward's range/guard — NOT re-tested motion internals.
- D2 — Motions return `KeyOutcome::Edited` (a caret move re-renders). No new `Moved` variant this
  ticket; seq-2 (history) will distinguish edit-vs-move by buffer-text comparison, not the outcome
  (kept robust). (Revisit only if a consumer needs it.)
- D3 — The caret-position render needs `split_at_caret` (the render draws `▏` at the start today);
  the split is a PURE tested helper (char-boundary, multibyte-safe), the paint is the shim.
- D4 — `apply_key`'s in-range-caret contract (caller keeps caret in `[0, len]`) is preserved — the
  motion fns clamp, and DeleteForward guards `caret < len`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a motion `Key` (`Left`/`Right`/`WordLeft`/`WordRight`/`Home`/`End`) is applied, `apply_key` shall set the caret to the corresponding `marley_editor::movement` result and return `Edited`, leaving the buffer text unchanged. | unit tests (each motion → the expected caret over a fixture; buffer text stable) |
| REQ-002 | WHEN `DeleteForward` is applied with the caret before the last char, `apply_key` shall delete the char at the caret (caret unchanged); WHEN the caret is at the end, it shall return `Ignored`. | unit tests (interior + end-of-line) |
| REQ-003 | WHEN a `Char` or `Backspace` is applied with the caret at an INTERIOR position, `apply_key` shall insert/delete at that position (not the end). | unit tests (insert mid-line; backspace mid-line) |
| REQ-004 | WHEN `split_at_caret(text, caret)` is called, it shall return the text split at the caret's char offset — `(chars[..caret], chars[caret..])` — correct for a multibyte string and at both ends. | unit tests (ascii, multibyte, caret 0, caret len) |
| REQ-005 | WHEN `scripts/gates.sh --diff` runs, every gate shall be GREEN with coverage 100%/MSI 100% on the touched pure surface. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact `Key` variants + arm delegations, `split_at_caret` shape + home, the
  keystroke mapping (alt/fn modifiers), the render change, the SPEC clause + mutation targets.
- **P3 Implement** — input.rs + app.rs + spec + CHANGELOG.
- **P3.5 Inspect** — critics: motion delegation correctness (right fn per key), DeleteForward
  bounds, the multibyte split, the render.
- **P4 Validate** — the unit tests + gate GREEN [--diff].
- **P5 Complete** — docs, AAR, archive, close #28.
