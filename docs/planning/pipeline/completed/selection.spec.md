---
pipeline_id: 7a3ead7a-453e-4c2a-9ee8-20ca8ffdf9ea
ticket: forge#255 (fc52ec37-67ce-46a1-a976-57dd7ab90069) · local docs/planning/tickets/open/TICKET-255-selection.md
aar_id: dac6c569-dc59-4832-a831-dc658eb44b85
title: Editor text selection — shift+arrows + mouse-drag
type: feature
milestone: M15
references: [forge#249, forge#250, forge#251, forge#254, marley_editor]
status: Phase 5 — Complete PASS
---

## Title
Turn the editor's single caret into a live SELECTION: shift+arrows extend a range from the caret (anchor fixed,
head moves), a mouse-drag selects (down sets the anchor, move extends the head), the #250 renderer highlights the
selected span, and typing/backspace over a selection replaces it — reusing `marley_editor::Selection` and #254's
click→offset map.

## Scope
### In
- **The selection MODEL** — an `anchor: Option<CharOffset>` per open file (alongside #249's `caret`): `None` = a
  bare caret at `caret`; `Some(a)` = the selection `a..caret` (normalize `start = min(a, caret)`, `end = max`).
- **A pure shift-extend/collapse fn** — given `(anchor, caret, movement, shift)`: a SHIFTED arrow sets the anchor
  (to the old caret if `None`) and moves `head = caret` via #251's movement; an UNSHIFTED arrow COLLAPSES
  (anchor `None`) and moves the caret. Reuses `movement::move_char_left/right`.
- **Shift+Left / Shift+Right** wired through the editor key path (the shift bit reaches the extend logic).
- **Mouse drag-select** — mousedown sets `anchor = caret = the click offset` (#254); an `on_mouse_move` while
  the button is held extends `head = caret` to the moved offset (a `dragging_selection` state, mirror #130's
  `dragging_divider`); mouseup ends it. The move's row comes from a **y→row** map (the code-area top + `cell.h` +
  scroll — the new shim piece the #254 click avoided by using the render loop var).
- **The #250 highlight** — a pure `row_selection_cols(sel_start, sel_end, row_start, row_nchars) ->
  Option<(col_start, col_end)>` (which display columns of a row are selected, via `col_of_offset`; `None` off the
  selection) + a selection-tint rect drawn behind the row's text.
- **Edit-over-selection** — typing / backspace with a non-empty selection `edit(start..end, replacement)`
  (replaces the range) then collapses to a caret at the edit site.

### Out (explicitly deferred)
- Multi-cursor (SelectionSet stays single-member); block/column selection; double-click-word + triple-click-line
  (a follow-up); shift+Home/End/word-wise (#257); the actual clipboard copy/cut/paste (#256 — but the selection
  MODEL it needs lands here); select-all (⌘A).

## Reference (§20)
**The universal monospace-editor text-selection convention (Zed is the editor-experience reference-app; NO Zed
source read — GPL/clean-room).** Shift+arrow extends a selection from a fixed anchor; a mouse-drag selects
anchor→head; the selected span is highlighted; typing/backspace replaces the selection. Marley mirrors this
BEHAVIOR over its own `marley_editor::Selection { anchor, head }` type and the #250 monospace grid (the highlight
draws on the same cell grid as the caret — observed capture `docs/warp_architecture/observed/250-warp-monospace-
grid-caret.png`; the selected columns come from the same `col_of_offset` map #254 inverts). Clean-room: observe
the selection BEHAVIOR; reimplement over `ropey`/in-repo; read no Warp/Zed source.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the model is `anchor: Option<CharOffset>` on the open file** (the caret stays the head). A selection is
  `anchor..caret`, normalized start=min/end=max. A bare caret is `anchor == None` (NOT `anchor == caret` — an
  explicit None avoids a "zero-width selection vs no selection" ambiguity in the render/edit).
- **D2 — the shift-extend/collapse is a PURE fn** (`(anchor, caret, movement, shift) -> (anchor, caret)`),
  reusing `movement`. The app.rs key handler calls it; cov/MSI 100.
- **D3 — the shift bit must reach the editor** — DESIGN resolves how (a new `Key` variant, a shift param to
  `apply_editor_key`, or the on_key_down branch computes the extend before/around `apply_key`). RE-READ input.rs
  `Key`/`apply_editor_key` + the app.rs on_key_down editor branch (`!platform && !control`) — confirm shift is
  available there and route it.
- **D4 — drag-select** uses a `dragging_selection` flag + a y→row map (code-area top + cell.h + scroll); the
  down/move reuse #254's `offset_for_click` + `line_start`. The y→row is the new shim; the pure offset math is
  reused.
- **D5 — the highlight is a pure per-row span** (`row_selection_cols`) + a render rect; drawn BEHIND the text,
  self-consistent with the #250 caret (same `col_of_offset`).
- **D6 — edit-over-selection** replaces the `start..end` range then collapses (anchor None, caret at the edit
  site) — `edit()` already supports a range; `#253` undo records it as one step.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN shift+arrow is pressed the system shall extend the selection (anchor fixed, head moves); an unshifted arrow shall collapse it to a caret. | pure unit + driven |
| REQ-002 | WHEN the editor is drag-selected (down → move → up) the system shall select the range from the mousedown offset to the current offset. | driven + review |
| REQ-003 | The #250 renderer shall highlight the selected span (the selected display columns per row). | pure unit (span) + driven |
| REQ-004 | WHEN a printable key or backspace is pressed with a non-empty selection the system shall replace the selection and collapse to a caret. | pure unit + driven |
| REQ-005 | The selection model (shift-extend/collapse + `row_selection_cols`) shall be pure with cov/MSI 100. | gate |

## Phase Plan
- **P2 Design** — the `anchor` model + the pure shift-extend/collapse fn + how shift reaches the editor (D3, the
  key risk) + the drag state + y→row + `row_selection_cols` + edit-over-selection; the pure test matrix + `cargo
  mutants --list`. Confirm §20. (If design finds the surface too large for one slice, it may recommend a #255a/b
  split — but prefer one v1.)
- **P3 Implement** — the pure fns (marley_editor / code_view) + the app.rs shim (anchor field, the key-extend
  route, the drag handlers + y→row, the highlight render); KEEP adjacent `mutants::skip`; `cargo check`.
- **P3.5 Inspect** — critics: the shift-extend/collapse correctness (anchor set/clear, movement reuse); the
  drag y→row + offset; the highlight span (multibyte/tabs/partial rows); edit-over-selection replaces exactly +
  collapses; no E0499; re-run `cargo mutants --list` for any moved shim.
- **P4 Validate** — pure units cov/MSI 100 (extend/collapse + row_selection_cols + edit-over-selection) + DRIVEN
  (shift+arrow highlights; drag highlights a range; type replaces it; data-safe no ⌘S). Gate green.
- **P5 Complete** — CHANGELOG + app_shell/editor doc; AAR; close #255; archive.
