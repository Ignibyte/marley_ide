---
pipeline_id: d1e44841-558a-45a7-b6a2-8fe46e61fb61
ticket: forge#267 (7f1f13d4-b1fb-4334-bb1a-bba77a0a8eca) · local docs/planning/tickets/open/TICKET-267-b2-input.md
aar_id: 669ad392-f54b-41d7-86f8-7d670061de95
status: Phase 5 — Complete PASS
title: B2 input — EntityInputHandler (real OS/IME text input for the editor)
type: feature
milestone: M16
references:
  - docs/zed_architecture/crates/gpui.md
  - docs/zed_architecture/subsystems/01-gpui-ui-framework.md
---

## Title
Give the editable editor REAL OS text input: implement gpui's
`EntityInputHandler` on `RootView` (routing to the active editor surface)
and register an `ElementInputHandler` during the editor branch's paint via
`Window::handle_input` — so plain characters, dead keys (⌥E → ´ pending →
é), and CJK/IME composition arrive through the platform's
NSTextInputClient path (`replace_text_in_range` /
`replace_and_mark_text_in_range`) instead of the hand-rolled
`Key::Char` arm. The dispatch contract is verified in the vendored gpui
0.2.2: key events route through the on_key_down ladder FIRST; the input
handler receives text only when propagation continues (test platform
`dispatch_keystroke` codifies it; mac window.rs wires the real
`insertText:`/`setMarkedText:` selectors). The editor's char-insert arm is
therefore REMOVED (handler-only insertion — no double-insert), the
remaining editor key arms stop propagation, and a new pure UTF-16↔char
seam (ropey `try_char_to_utf16_cu`/`try_utf16_cu_to_char`) plus a pure IME
op layer in `marley_editor` keep every conversion and composition edit
testable without gpui.

## Scope
### In
- `marley_editor`: a new `ime` module — pure ops over `(Buffer, caret,
  anchor, marked)`: `replace_text` (range-or-selection replace + clear
  mark), `replace_and_mark` (replace + set marked + composition
  selection), `unmark`, `text_for_range`, plus the UTF-16 seam on `Buffer`
  (`char_to_utf16` / `utf16_to_char`, `try_`-based, clamping, multibyte-
  exact). All CharOffset-pure inside; UTF-16 only at the seam edges.
- `EditorSurface`/`OpenFile`: a per-file `marked: Option<(CharOffset,
  CharOffset)>` (+ accessors), cleared on every non-IME edit path that
  already collapses selections.
- app.rs: `impl EntityInputHandler for RootView` (8 methods → the active
  editor surface through the pure ops; `None`/no-ops when no editor tab is
  active); registration inside the #266 editor-branch canvas paint
  (`window.handle_input(&focus_handle, ElementInputHandler::new(bounds,
  entity), cx)`) — paint-scoped, so ONLY an active+focused editor tab ever
  registers (the terminal/prompt path is untouched by construction);
  `bounds_for_range` = the caret cell rect from the monospace cell math;
  `character_index_for_point` via the existing click mapping.
- app.rs on_key_down editor arm: the plain-char insert arm REMOVED (chars
  now arrive via the handler); Enter/Backspace/Delete/motions/shift-arrows
  stay and now `cx.stop_propagation()` (Enter carries key_char "\n" — the
  fallback would double-insert otherwise).
- Composition visual v1: the composition selection (from
  `new_selected_range`) renders through the EXISTING selection tint — no
  new render channel; the dedicated marked-text underline is deferred.
- Driven + headless verification (typing via the handler, dead-key ⌥E→E
  composition live, marked-state asserts).

### Out (explicitly deferred)
- IME for the terminal prompt / PTY tabs (separate ticket — the prompt
  keeps the raw `Key` path; no handler ever registers on terminal tabs).
- The marked-text UNDERLINE style (needs a third highlight channel
  through styled_slices — folds into the #268 highlight generalization).
- `TextLayout`-based caret/click geometry (stays deferred with #266's D3/
  D4 — the cell math is exact on the mono grid).
- Key-repeat/press-and-hold accent popover tuning
  (`apple_press_and_hold_enabled` stays default).

## Reference (§20)
**Zed (the editor reference)** — behavior matched: the editor-as-
NSTextInputClient model (text insertion, marked/composition text, and IME
geometry served by the view through gpui's input-handler element, with
key chords resolved by the key dispatch ladder first) as described
behaviorally in `docs/zed_architecture/crates/gpui.md` +
`subsystems/01-gpui-ui-framework.md`. Clean-room: the MECHANISM is gpui's
own Apache-2.0 public `EntityInputHandler`/`ElementInputHandler` API
consumed as shipped; no GPL editor source is read — Marley's handler
routing, UTF-16 seam, IME ops, and marked-state model are original.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Insertion becomes HANDLER-ONLY on the editor tab: the `Key::Char`
  arm is deleted, not gated (two live insert paths = the double-insert
  class; the #266 F-ledger idiom "one mechanism per behavior").
- D2 — The handler methods are THIN: every conversion/edit decision lives
  in the pure `ime` ops (cov/MSI 100); the impl block is shim-skipped like
  the rest of app.rs.
- D3 — UTF-16 offsets are window-facing ONLY; the editor model stays
  CharOffset-pure (same seam discipline as #266's byte ranges).
- D4 — Registration is paint-scoped to the editor branch (no global
  registry, no per-tab bookkeeping — the frame that doesn't paint an
  editor simply registers nothing).
- D5 — Composition selection reuses the existing selection channel
  (anchor/caret) — no parallel selection state.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `Buffer::char_to_utf16`/`utf16_to_char` shall convert exactly on ASCII/multibyte (é=1cu, 日=1cu, 😀=2cu) and clamp out-of-range input (never panic). | pure unit tests, mutants killed |
| REQ-002 | `ime::replace_text` shall replace the given UTF-16 range (or the selection/caret when `None`), position the caret after the inserted text, clear the marked range, and record a Human edit (undoable). | pure unit tests + undo round-trip |
| REQ-003 | `ime::replace_and_mark` shall replace the marked range (falling back to selection/caret), set the new marked span over the inserted text, and set the composition selection from `new_selected_range` (relative UTF-16 → absolute chars). | pure unit tests (dead-key + CJK sequences scripted) |
| REQ-004 | With an editor tab focused, a typed printable shall insert through the input handler (the char arm gone): headless `simulate_keystrokes("x")` mutates the buffer; on a terminal tab the prompt path is byte-identical. | headless lane + existing prompt tests |
| REQ-005 | Enter/Backspace/motion keys on the editor shall behave exactly as before AND stop propagation (no handler double-fire; Enter inserts exactly one `\n`). | headless keystroke asserts |
| REQ-006 | A live dead-key composition (⌥E then E) shall produce é in the editor buffer through marked-then-commit. | driven capture + buffer state readback |
| REQ-007 | `selected_text_range`/`text_for_range`/`marked_text_range` shall report UTF-16-correct values over multibyte content (the IME's view of the buffer is consistent). | direct trait-method tests through the window |

## Phase Plan
- **P2 Design** — exact ime op signatures, the marked-state placement, the
  handler impl skeleton, canvas registration mechanics, the key-arm diff,
  test plan incl. the headless + driven matrix.
- **P3 Implement** — seam + ops + surface state + handler + registration +
  key-arm cut.
- **P3.5 Inspect** — critics (utf16 fenceposts, composition state machine,
  propagation/double-insert, terminal-path regression, §20).
- **P4 Validate** — units + mutants; headless lane; driven dead-key
  capture; gate --diff.
- **P5 Complete** — CHANGELOG, editor.md/app_shell.md, AAR, archive,
  close.
