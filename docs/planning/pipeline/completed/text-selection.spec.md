---
pipeline_id: b3ccfd52-5345-4de3-a8b8-127bf9691daa
ticket: forge#43 (9a6582b5-76c3-4bde-9c2a-258ce5469d1c) · local docs/planning/tickets/open/TICKET-043-text-selection.md
aar_id: 98d605b0-1892-4622-9d1b-69280eba076d
status: Phase 5 — Complete PASS
title: terminal text selection model
type: feature
milestone: M1.G
references:
  - crates/marley_app/src/text_selection.rs (NEW — the pure selection model)
  - crates/marley_app/src/workspace.rs (PaneState gains `selection`)
  - crates/marley_app/src/app.rs (mouse handlers + highlight render — shim)
  - docs/specs/SPEC-app-shell.spec.md
---

## Title
There is NO text selection today — you cannot select terminal output at all (paste ships in #42, but
copy has nothing to copy). Add a pure selection model over the rendered content rows — the FOUNDATION
for copy (#44) and the M1.G copy/paste round.

## Scope
### In
- `crates/marley_app/src/text_selection.rs` (NEW, gpui-free PURE — cov/MSI 100):
  - `GridPos { row: usize, col: usize }` (col in CHARS) — derives `Ord` ((row, col) lexicographic).
  - `Selection { anchor: GridPos, head: GridPos }` + `normalized(self) -> (GridPos, GridPos)` (start ≤
    end, so a backward drag reads the same as forward).
  - `selected_text(rows: &[String], sel: Selection) -> String` — the char range: single row →
    `rows[r][anchor.col..head.col]`; multi-row → first row `anchor.col..`, middle rows whole, last row
    `..head.col`, joined with `\n`. Char-indexed + clamped per row + to `rows.len()` (an out-of-range
    drag yields the in-range text). A private `row_slice(s, from, to)` does the char-safe slice.
- `crates/marley_app/src/lib.rs` — `mod text_selection;`.
- `crates/marley_app/src/workspace.rs` — `PaneState<S>` gains `selection: Option<Selection>`
  (`PaneState::new` inits `None`).
- `crates/marley_app/src/app.rs` (SHIM) — mouse-down at a pane maps window x/y → `GridPos` (÷ the #34
  cell metric, offset by the #32 viewport top — the inverse of the render) → `selection =
  Some(Selection{anchor, head: anchor})`; drag extends `head`; the render paints the selected cells
  with a highlight bg (`accent` at low opacity); the selection clears on a click outside / a new
  command. The `Vec<String>` content is built in the shim from the blocks (header + `output_text`
  lines) — the same rows the render walks.
- SPEC-app-shell (the selection clause + Mutation-Targets). CHANGELOG + arch doc.

### Out (explicitly deferred)
- Copy itself (cmd-C → clipboard) — that's #44 (this only MODELS + highlights the selection). Word/line
  select (double/triple-click), column/box selection, selection auto-scroll while dragging past the
  edge (M2+). Selecting the alt-screen grid (vim) — the first cut selects the Block-list rows; the
  alt-screen selection is a later refinement. Copy-on-select.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `col` is a CHAR index (not byte) — `selected_text` slices via `chars()`, so multi-byte output
  (`café`, box-drawing) selects correctly; `GridPos` derives `Ord` for the normalization.
- D2 — `selected_text` CLAMPS (row ≥ len → empty; col past row → row end) so an out-of-range mouse
  drag never panics and yields the sensible in-range text.
- D3 — The selection is PER-PANE (`PaneState.selection`) — each terminal has its own; the mouse/
  highlight is SHIM (app.rs, masked). PURE surface = `selected_text` + `normalized` (cov/MSI 100).
- D4 — This is the TERMINAL-output selection, distinct from `editor::Selection` (#28, the prompt
  buffer's caret selection) — a separate module + type.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `Selection::normalized()` is called, it shall return `(start, end)` with `start ≤ end` by `(row, col)` order — a backward selection (anchor after head) normalizing to the same pair as its forward twin. | unit (forward + its reverse → same pair) |
| REQ-002 | WHEN `row_selection(sel, row, row_len)` is called for a row inside the selection, it shall return the selected char span `(from, to)` — the first row from `anchor.col`, the last to `head.col`, middle rows the whole `0..row_len`. | unit (multi-row: each row's span; single-row) |
| REQ-003 | WHEN `row` is outside `[start.row, end.row]`, `row_selection` shall return `None`; a `from`/`to` past `row_len` shall clamp; a backward selection shall give the same spans as its forward twin. | unit (out-of-range→None; clamp; backward) |
| REQ-004 | WHEN a drag selects terminal text, the render shall highlight the selected cells (via `row_selection` per row); the selection clears on a click outside or a new command. | shim + masked visual — chad-verified |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `row_selection` + `normalized`. | gate exit 0 + receipt |

NOTE: `selected_text(rows, sel)` (the `\n`-joined copy text, built on `row_selection`) moves to #44
(copy), where the cmd-C shim uses it — shipping it here would be dead-code (only tests would call it).

## Phase Plan
- **P2 Design** — the exact `text_selection.rs` shapes (GridPos/Selection/normalized/selected_text/
  row_slice), the PaneState field, the app.rs mouse→GridPos map + highlight, the SPEC clause + mutation
  targets.
- **P3 Implement** — text_selection.rs + mod + PaneState field + the app.rs mouse/highlight + spec +
  CHANGELOG.
- **P3.5 Inspect** — critics: normalization (backward), the row-span slicing (off-by-one first/last
  col, the middle-row range), char-boundary safety (byte vs char panic), clamp, no #32/#34 regression.
- **P4 Validate** — the selected_text/normalized unit tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #43.
