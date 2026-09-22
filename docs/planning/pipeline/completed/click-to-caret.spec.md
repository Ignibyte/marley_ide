---
pipeline_id: a6709898-394f-44d5-a586-d8ceaff6e765
ticket: forge#254 (fb9e2682-6515-4acc-9514-bc88215eda20) · local docs/planning/tickets/open/TICKET-254-click-to-caret.md
aar_id: 41875b20-6243-4151-b507-11650a559ece
title: Mouse click-to-place-caret in the editor
type: feature
milestone: M15
references: [forge#249, forge#250, forge#251, marley_editor]
status: Phase 5 — Complete PASS
---

## Title
A left-click in the editor body places the caret at the `char` offset nearest the click, by inverting #250's
exact char↔column map (`col_starts`) — so clicking mid-line moves the caret there and typing (#251) inserts at
the click point, not at offset 0.

## Scope
### In
- **`LineLayout::offset_of_col(col) -> usize`** (crates/marley_app/src/code_view.rs) — the column→offset INVERSE
  over the SAME `col_starts` that `col_of_offset` reads (the #250 doc reserved it). Nearest-boundary rounding
  (a click in a char's right half snaps to the next boundary); clamped to `[0, n_chars]`.
- **A pure click→CharOffset composition** — given `(row, display_col)`: clamp `row` to `[0, len_lines-1]`, take
  that line's text → `line_layout(tab_width)` → `offset_of_col(col)` → char-in-line, then add the line-start char
  offset → the absolute `CharOffset`. (A small pure `Buffer::line_start(row) -> CharOffset` helper if ropey's
  `line_to_char` isn't already exposed.)
- **The editor-body `on_mouse_down(Left)` shim** (app.rs) — `MouseDownEvent.position` minus the code-area origin
  → `(row = (y_rel + scroll_px) / cell.h, col = round(x_rel / cell.w))` → the pure fn → set the active file's
  caret (`active_buffer_and_caret_mut`) + `cx.notify()`. Guarded to an editor tab.

### Out (explicitly deferred)
- Drag-to-select / shift-click range (#255); double-click-word / triple-click-line; a caret in the read-only
  split-file pane (#246/#258); a hover text-cursor; horizontal-scroll click math (v1 assumes no h-scroll — the
  #250 renderer is un-truncated + wraps within the pane width).

## Reference (§20)
**Warp — its command-input click-to-place-caret (the universal monospace-editor convention).** Clicking in a
monospace text input places the caret at the character cell nearest the click; the caret then sits between two
chars and typing inserts there. Marley mirrors this BEHAVIOR for the file buffer. The shared substrate is #250's
observed monospace cell grid (`docs/warp_architecture/observed/250-warp-monospace-grid-caret.png`) — #254 is
simply that grid inverted (pixel → cell → `char` offset). The mapping is Marley's own pure inverse over
`col_starts` (`AD-claude-editor-offset-column-model-001`); clean-room — observe the click BEHAVIOR, read NO
Warp/Zed source. (No fresh capture: clicking chad's live Warp input to observe would disturb his session; the
#250 grid capture already documents the cell geometry #254 inverts.)

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — `offset_of_col` inverts the SAME `col_starts`** as `col_of_offset` (one source of truth; the #250 AD).
  Nearest-boundary rounding: pick the `col_starts` entry closest to `col` (ties → the later boundary), clamped
  to `[0, n_chars]`. Pure, cov/MSI 100 (round-trip vs `col_of_offset`, past-EOL clamp, a tab line, multibyte).
- **D2 — the click→CharOffset math is pure + testable** (`(row, col, tab_width, buffer/line-text) → CharOffset`);
  the app.rs shim does ONLY the pixel→`(row, col)` conversion (cell metrics + scroll + gutter), then delegates.
- **D3 — clamps, never panics:** `row` past the last line → the last line; `col` past a line's end → the
  line-end offset (`n_chars`). A click in the gutter or empty area resolves to a valid offset.
- **D4 — editor-tab-guarded.** The handler no-ops unless an editor tab is active (a terminal tab / read-only
  split pane is unaffected). Focus: the #251 `on_key_down` editor branch already routes keys to the active editor
  tab (proven in #253's driven run) — so the click SETS THE CARET; design confirms whether an explicit focus
  call is also needed.
- **D5 — the shim reuses #250 render geometry** — the same gutter width + `cv.scroll` + `cell` (em_advance /
  line_height) the #250 caret DRAW uses, so click-in and caret-out are self-consistent (a mismatch = the caret
  lands off the click).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `LineLayout::offset_of_col(col)` shall return the `char` offset nearest `col` over `col_starts` (the inverse of `col_of_offset`), clamped to `[0, n_chars]`. | pure unit |
| REQ-002 | WHEN the editor is active and the body is left-clicked, the system shall set the active file's caret to the `CharOffset` nearest the click (row from y+scroll, col from x) and #250 shall redraw the caret there. | driven + review |
| REQ-003 | A click past a line's end shall clamp to the line-end offset; a click below the last line shall clamp to the last line (no panic). | pure unit |
| REQ-004 | Click-to-place-caret shall apply only when an editor tab is active; a terminal tab / read-only pane shall be unaffected. | review + driven |
| REQ-005 | WHEN the editor is clicked and then typed into, the inserted text shall land at the clicked offset (not offset 0). | driven |

## Phase Plan
- **P2 Design** — the `offset_of_col` rounding rule (nearest-boundary over `col_starts`) + the pure
  click→CharOffset fn signature (+ a `Buffer::line_start` helper if needed) + the app.rs shim's pixel→(row,col)
  (bounds-capture pattern for the code-area origin/gutter/scroll) + the editor-tab guard + the pure test matrix +
  `cargo mutants --list`. Confirm §20.
- **P3 Implement** — `offset_of_col` + the pure click fn (+ line-start helper) + the `on_mouse_down` shim; KEEP
  any adjacent `mutants::skip` (the #252/#253 detach-trap lesson); `cargo check`.
- **P3.5 Inspect** — critics: `offset_of_col` round-trips `col_of_offset` exactly (incl. tabs/multibyte); the
  clamps hold (past-EOL, past-last-line, gutter); the shim guards to an editor tab + subtracts the right
  origin/scroll; re-run `cargo mutants --list` if a fn moved near a masked shim.
- **P4 Validate** — pure units cov/MSI 100 (offset_of_col + the click math) + DRIVEN (click a spot in the editor
  → the caret bar jumps there → type → inserts at the click point; data-safe, no ⌘S). Gate green.
- **P5 Complete** — CHANGELOG + app_shell/editor doc; AAR; close #254; archive.
