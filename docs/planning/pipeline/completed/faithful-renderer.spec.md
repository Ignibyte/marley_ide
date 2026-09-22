---
pipeline_id: 0de52da1-37d2-4c8b-9400-78858153e8dd
ticket: forge#250 (da40fb6e-a4b2-4100-b567-2811776095d7) · local docs/planning/tickets/open/TICKET-250-faithful-renderer.md
aar_id: 786bca71-dafc-48e1-b418-e54f0c30fe7c
status: Phase 5 — Complete PASS
title: Faithful editor renderer — draw from the Buffer with a visible caret (exact offset↔column)
type: feature
milestone: M15
references: [forge#249, forge#251, forge#254, marley_editor]
---

## Title
Render the active editor file FROM the `marley_editor::Buffer` (not the lossy pre-rendered `CodeViewState.lines`),
un-truncated, so on-screen columns line up EXACTLY with character offsets — and draw a VISIBLE caret at the
active file's caret position. Expose the offset↔column mapping as a tested PURE seam that the caret (this ticket),
mouse-click→offset (#254), and selection (#255) all reuse. THE make-or-break of the M15 editor: if the
offset↔column map is wrong, the caret mis-places and a save (#252) can corrupt.

## Scope
### In
- **A pure offset↔column mapping** (in `code_view.rs`, the existing pure seam): for a line + `tab_width`,
  `char_offset → display column` (tab advances to the next tab-stop `col + (tab_width − col % tab_width)`; every
  other char = 1 column in v1) and `display column → nearest char offset` (the inverse, for #254). ONE source of
  truth — the existing `expand_tabs` is reimplemented on it (or derived from it) so the render + the map can
  never disagree. cov/MSI 100.
- **Render the editor from the Buffer** — the editor-surface tab draws each visible line's CONTENT from the
  active file's `Buffer` (via a new `Buffer::line_text(row)` + an immutable `EditorSurface::active_buffer()`),
  **un-truncated** (the caret must be able to address every column — no `…` cap on the editor).
- **A visible caret** — draw a block/bar at the active file's caret (`active_caret()` from #249): x = gutter +
  `column × measured monospace cell width` (reuse the terminal's `em_advance` measurement), y =
  `(caret_row − scroll) × line_height`; shown when the editor tab is active.
- Tabs expand to tab-stops in the render, consistent with the mapping (REQ-003).

### Out (explicitly deferred)
- **Caret MOVEMENT + text input** (#251 — focus + `apply_key`, Enter⇒`\n`); the #250 caret is static at
  `active_caret()` (which #249 seeds at offset 0) until #251 moves it.
- **Mouse click → CharOffset** (#254) — but the `column → offset` half of the map is BORN here for #254 to reuse.
- **Selection highlight** (#255) — needs per-column rects; #250 draws only the caret.
- **Horizontal scroll** + true **wide/CJK/grapheme width** — v1 is char-width-1 + no h-scroll (long lines
  overflow-clip at the pane edge). The mapping fn is SHAPED to grow (a `char_width` hook defaulting to 1).
- **The #246 read-only split-pane** rendering from a buffer — it has NO buffer (`PaneContent::CodeView`); it
  keeps its lossy `CodeViewState.lines` render (editable split pane is #258). #250 must NOT break it.

## Reference (§20)
**Warp — the monospace cell grid + block caret.** Captured non-invasively from a running Warp window →
`docs/warp_architecture/observed/250-warp-monospace-grid-caret.png` (+ the note in `observed/README.md`). Warp
renders all text on a **fixed-width monospace cell grid**: every `char` occupies one cell, so a character's
offset maps **1:1** to its display column (indentation, tree glyphs, bullets all land on identical columns);
tabs advance to the next tab-stop; the caret is a **solid block occupying one whole cell** on a column boundary.
#250 reproduces exactly this: render from the real text on a monospace grid where column == offset, and place a
block caret at `column × cell-width`. A terminal is the canonical monospace grid, so Warp (the §20 terminal
reference) is the right source; the editor-experience polish reference (Zed) begins in a later train. Clean-room:
observed rendering behavior only — no Warp source read; reuse `gpui` + `ropey` freely.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the offset↔column map is the durable PURE seam** (`code_view.rs`), reused by the render + the #250
  caret + #254 mouse + #255 selection. `expand_tabs` is reimplemented on / derived from it (single source of
  truth — the display string and the column map come from the same pass). Design picks the exact shape (likely a
  `line_columns(line, tab_width) -> {display: String, col_of_offset: Vec<usize>|fn, offset_of_col: fn}`).
- **D2 — v1 column model = char-width-1 + tab-stops.** Each non-tab `char` is one column; tabs go to the next
  tab-stop. Wide/CJK/grapheme width is DEFERRED but the fn is shaped so a `char_width(char)->usize` (default 1)
  can slot in without changing callers.
- **D3 — render from the Buffer via new plumbing:** `Buffer::line_text(row) -> String` (marley_editor, pure,
  its own cov/MSI 100 — wraps ropey `rope.line(row)`) + `EditorSurface::active_buffer(&self) -> &Buffer`
  (immutable mirror of `active_buffer_mut`). The editor render sources line text from the buffer; the split pane
  is untouched (keeps `CodeViewState.lines`).
- **D4 — un-truncate the editor render** (drop `CODE_MAX_COLS` for the editor path; the caret needs full lines).
  Long lines overflow-clip for now (h-scroll deferred). The split pane KEEPS `CODE_MAX_COLS`.
- **D5 — the caret is an overlaid, absolutely-positioned block** at `(gutter + col×em_advance, (row−scroll)×
  line_height)`, reusing the terminal's `em_advance`/`fallback_cell` measurement (app.rs — proven in-codebase).
  This avoids a full per-column measured-grid line-render rewrite (deferred until selection rects need it, #255);
  the monospace flexbox text lands on column boundaries so the overlaid caret aligns. Design confirms the
  measurement reuse + the caret geometry.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | A pure fn shall map each char offset in a line to its display column (tab → next tab-stop; non-tab char = 1 col), exact for tab / empty / ascii lines. (The inverse — display column → char offset — is added with the #254 mouse click that consumes it; shipping it unused in #250 would be dead code under `-D warnings`.) | pure unit (cov/MSI 100) |
| REQ-002 | The editor surface shall render the active file's line CONTENT from its `Buffer` (not `CodeViewState.lines`), un-truncated, so rendered columns equal char offsets. | driven capture + review |
| REQ-003 | Tabs shall expand to tab-stops in the editor render, consistent with the REQ-001 mapping (one source of truth). | pure unit + driven |
| REQ-004 | The editor shall draw a visible caret at the active file's caret offset, positioned at `column × measured monospace cell width`, when the editor tab is active. | driven capture |
| REQ-005 | The #246 read-only split pane, the #243 persistence, and the file-tab strip shall be UNAFFECTED (compile + behavior). | `cargo check --workspace` + driven + review |

## Phase Plan
- **P2 Design** — resolve D1-D5 concretely: the `line_columns`/map fn signature + how `expand_tabs` derives from
  it; `Buffer::line_text` + `active_buffer()`; the `code_view_body` fork (editor draws from buffer + caret;
  split pane unchanged) + the caret overlay geometry (em_advance reuse); the pure test matrix (tabs, empty,
  ascii, offset↔column round-trip, boundary cols); `cargo mutants --list`. Confirm the §20 match.
- **P3 Implement** — the map (+ expand_tabs reuse) in code_view.rs; `line_text`/`active_buffer`; the render +
  caret shim; `cargo check --workspace` clean.
- **P3.5 Inspect** — critics: offset↔column EXACTNESS (tabs, empty, trailing, ascii; the inverse round-trips);
  the caret aligns with the monospace text (measurement correct); NO split-pane / persistence / tab-strip
  regression; un-truncation doesn't break gutter/scroll; clean-room.
- **P4 Validate** — pure units cov/MSI 100 (the map + line_text) + `cargo check --workspace`; DRIVEN (mac
  unlocked) — open a file with TABS + a >200-col line → prove the editor renders faithfully (tabs on stops, NO
  `…` truncation) with the caret block at the top-left (offset 0), vs the Warp observed reference. Gate green.
- **P5 Complete** — CHANGELOG + app_shell/editor doc; AAR; close #250; archive.
