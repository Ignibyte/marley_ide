---
pipeline_id: 92a3cfc4-0a00-4fae-8278-71d534786a5f
ticket: docs/planning/tickets/open/TICKET-432-multibuffer-input-completeness.md
status: Phase 5 — Complete PASS
title: Multibuffer input completeness — click-column caret · ⌘V paste · IME composition
type: feature
milestone: M33
references:
  - docs/planning/design-notes/m33-tail-and-wedge-shelf.md
---

## Title

The #428 editable multibuffer closes its three recorded v1 input seams. (a) A click on an
excerpt line places the caret at the CLICKED COLUMN — the x offset maps through the measured
mono advance and the editor's own `LineLayout` inversion (`mb_place_caret` today parks at
line end; the click handler at `app.rs:6631` discards the event entirely). (b) ⌘V pastes
through the ONE insert mechanism (`mb_edit_transient` → `edit_at_selections` →
`mb_after_edit` — the exact chassis `mb_newline` rides): a single-line clip writes through
at the caret; a multi-line clip GROWS the window exactly as Enter does (the #428 anchor
arithmetic, no new growth machinery) — and the ⌘V ladder stops misdirecting an mb-tab paste
into the invisible workspace terminal (today's fall-through at `app.rs:20745`). (c) IME
composition (dead keys, marked text) works end-to-end on the mb's own `handle_input` canvas
(#428 inspect C1): the `EntityInputHandler` methods that are still editor-only —
`replace_and_mark_text_in_range`, `marked_text_range`, `selected_text_range`,
`text_for_range`, `unmark_text`, `bounds_for_range` — gain mb arms over the standing
`marley_editor::ime` fns and the already-shipped `mb_marked` span. All three are small arms
on shipped infra (AD-claude-428-excerpt-editing-rides-existing-machinery-001); no new
machinery.

## Scope

### In
- **Click-column caret**: the mb Line-row click maps `event.position.x` through a recorded
  mb frame geometry (the `editor_geom` zero-size-canvas idiom — the #428 design's named
  "geom hook") into a FLOAT column (`(x − x0) / cell_w`, unrounded —
  BF-claude-click-col-quantized-before-nearest-scan) and inverts it with the editor's own
  `code_view::offset_for_click` (tab-aware, wide-glyph midpoint, multibyte-correct);
  `mb_place_caret` grows a column parameter. Clicks left of the text cell clamp to col 0;
  past line end clamp to line end (the shipped v1 behavior becomes the clamp arm).
- **Caret-bar/column agreement**: the caret bar's paint column moves from raw char-col ×
  cell_w (`app.rs:6619`) onto the SAME map (`LineLayout::col_of_offset`) so
  click → caret → bar round-trips exactly in pixel space — one column domain, one
  conversion path.
- **⌘V paste**: an mb arm in the key ladder (placed with the #256 editor ⌘C/⌘X/⌘V block,
  BEFORE the terminal cmd-V arm — PR-claude-new-chord-shadowed-by-hardcoded-key-001 audit):
  clipboard text → `mb_marked = None` (paste ends composition, the #267 rule) → ONE
  `edit_at_selections` on the transient caret set (PR-claude-composite-type-over-one-edit-001)
  → `mb_after_edit` (one journal entry, touched, caret after the insert). Multi-line is the
  same single edit; the live rebuild grows the window like Enter.
- **Fail-closed surface routing**: WHILE a multibuffer tab is active, the plain-⌘C/X/V
  family never reaches the workspace terminal arms (`focused_terminal*` resolves the hidden
  grid pane regardless of tab kind — today ⌘V types the clipboard into the unseen prompt/PTY).
  ⌘V with no caret (or an empty/non-text clip) is a consumed no-op. ⌘C/⌘X stay functionally
  out (the mb has no selection model) but are consumed, not misdirected.
- **IME composition**: mb arms in every still-editor-only `EntityInputHandler` method —
  `replace_and_mark` through `mb_edit_transient` with `&mut self.mb_marked` (the closure
  param shipped at #428 for exactly this); the read-side queries (`marked_text_range`,
  `selected_text_range`, `text_for_range`) answered from the caret ANCHOR + `mb_marked`,
  never from the buffer's live `SelectionSet` (F-claude-428-b /
  PR-claude-428-b — the live set belongs to the file's own views); `bounds_for_range` /
  `character_index_for_point` from the recorded mb geometry (the editor's pure-arithmetic
  shape, `None` while off-viewport). Preedit is buffer text (write-through, visible in the
  file's tab mid-composition — the editor's own #267 semantics); commit replaces the marked
  span; every gesture that drops the caret keeps dropping the composition.
- **React-first**: the POC MultibufferView gains the paste interaction FIRST (its
  click-column math already ships — the standing parity reference).

### Out (explicitly deferred)
- Rich clipboard formats (only `ClipboardItem::text()` — the editor's own posture),
  column-select paste, cross-file multi-paste.
- ⌘C/⌘X *function* on the mb (needs a selection model; only the fail-closed consume lands).
- An mb selection model / drag-select; multi-cursor in excerpts.
- Marked-text underline/styling in mb rows (the #428 recorded deferral stands: composition
  renders as plain buffer text; the editor tab has the same v1 face).
- Tab-expanded mb ROW RENDERING: the row keeps painting raw text (`StyledText`); the column
  MAP is tab_width-aware, so click→bar stays self-consistent on tab-bearing lines even
  where painted glyph advances diverge — the pre-existing #427 render caveat, recorded.
- #431 DisplayMap excerpt unification (the shelf's prior tail item) — this spec is authored
  against the post-#430 tree and touches input arms, not the projection stack; if #431
  lands first, Phase 2 re-checks only the geometry-recording home.

## Reference (§20)

**Zed (the editor reference — same-gpui-stack).** Behavior matched: a click in any excerpt
places a precise caret (mono column at the pointer); paste is the same per-selection insert
every other edit uses; IME rides `EntityInputHandler` — "input is compute one edit per
selection, apply atomically", with `replace_text_in_range` (commit) and
`replace_and_mark_text_in_range` (marked/composing) as the platform protocol and an
`ime_transaction` grouping composition edits for clean undo
(`docs/zed_architecture/subsystems/03-editor-multibuffer.md`, "The input path"). Adopt the
CONTRACT, not the container — Marley's #428 transient-caret + journal chassis already
implements the grouping/undo half. Clean-room wall holds: deconstruction docs only, no Zed
source read.

### Prior art

1. **Behavior maps** — `03-editor-multibuffer.md` "Two entry points converge on the buffer"
   (bound keys → actions; raw text/IME → `EntityInputHandler`; paste as the same fan-out;
   the doc's own reimplementation note: "IME through gpui's input-handler trait — adopt
   `replace_text_in_range`/`replace_and_mark_text_in_range` verbatim as the design; back
   them with `Buffer::edit`"). No multibuffer-SPECIFIC input chapter exists beyond this —
   the excerpt surface inherits the editor contract, which is exactly this ticket's shape.
2. **Published** — gpui's `InputHandler` documents itself as "a 1:1 exposure of the
   NSTextInputClient API" (gpui-0.2.2 `src/platform.rs`, linking Apple's protocol):
   ranges are UTF-16 code units over the whole document; `selected_text_range` ↔
   `selectedRange`, `marked_text_range` ↔ `markedRange`, `replace_and_mark` ↔
   `setMarkedText:`, `bounds_for_range` ↔ `firstRectForCharacterRange` (the candidate
   window anchor), `character_index_for_point` ↔ `characterIndexForPoint`. Nothing further
   needed — `marley_editor::ime` already encodes the protocol's target resolution
   (explicit range → marked → selection → caret, clamped) and the de-facto
   document-absolute range semantics (its own recorded WebKit/Chromium/Firefox note).
3. **Permissive deps / in-tree (the highest-yield leg)** — gpui (Apache-2.0, crates.io
   0.2.2) IS the seam: `EntityInputHandler` (`src/input.rs:10`) + `ElementInputHandler`
   registered per-frame by a paint-scoped canvas — the mb already registers one
   (`app.rs:6684-6697`, #428 inspect C1), so typing/commits ALREADY route; only the marked
   half and the geometry answers are missing. In-tree, everything else exists: the ONE
   insert chassis (`mb_edit_prepare`/`mb_edit_transient`/`mb_after_edit`,
   `app.rs:13236/13320/13253` — transient set saved+restored, journal real-edit-only +
   depth-keyed); `mb_marked` (`app.rs:571`) already threaded through the transient closure
   and cleared by Esc/arrows/undo/redo/place-caret; `marley_editor::ime`
   (`replace_and_mark` `ime.rs:167`, `marked_utf16`/`selected_utf16`/`text_for_range`,
   `utf16_ix_to_char_ix`); the editor's click recipe (`app.rs:7680-7736`: recorded
   `editor_geom` x0 → float col → `offset_for_click` `code_view.rs:248` →
   `offset_of_col_f`); the geometry recorder idiom (`EditorFrameGeom` canvas,
   `app.rs:7458-7494`; test seed `seed_editor_geom_for_test` `app.rs:19023`); the editor
   ⌘V shape (`app.rs:20683-20698`: clear composition + one `edit_at_selections`,
   `EditOrigin::Human`); the mb cell_w measurement already in `multibuffer_body`
   (`app.rs:6427-6434`, the same `em_advance` at the Command size); window growth on
   multi-line insert proven by `mb_newline` (`app.rs:13520`) + `sync_multibuffer_live`
   (`app.rs:14037`) over the #428 window anchors (grow-at-edges bias arithmetic,
   `multibuffer.rs:119-122`).

## React-first (parity)

**UI-AFFECTING — zone B (the POC is the multibuffer surface's design source;
MARLEY-PARITY port row: `views/MultibufferView.tsx` ↔ `multibuffer.rs`/`app.rs`).**
Honest split per seam: **click-column** — the POC ALREADY ships it
(`MultibufferView.tsx:222-231`: `(clientX − box.left − GUTTER_W) / CELL_W`, clamped
[0, len]; bar at `col × CELL_W`) and MARLEY-PARITY records the standing gap "POC mono-col
click vs Rust line-end caret" — this ticket closes it FROM THE RUST SIDE against the POC
reference (pull the POC up, verify, re-capture; the POC's `Math.round` is a stand-in — the
Rust port uses the real float-midpoint inversion, behavior-equivalent at cell scale).
**⌘V paste** — a real POC delta to BUILD FIRST: a paste arm in
`views/MultibufferView.tsx`'s key/clipboard handling that splices the clip through the
existing `writeThrough` (single-line in-row; multi-line splits rows and renumbers exactly
like its Enter arm — the window grows). Implement builds + screenshots the POC paste
FIRST (dev server, READ the PNG), then ports 1:1; validate captures the parity pair.
**IME** — Rust-only, recorded (like the POC's recorded no-undo gap): composition is the
platform NSTextInputClient seam; the POC's window-keydown splicing surface has no marked-
text path to prototype (no editable element hosts a browser composition), so no POC twin
is built and the MARLEY-PARITY row records IME as Rust-only at complete.

## Locked-In Decisions

- **D1 — One column map, the editor's own.** Click inversion = `offset_for_click`
  (`line_layout` + `offset_of_col_f`, FLOAT until the final compare); caret-bar paint =
  `col_of_offset` over the same layout. No new conversion path, no `Math.round` port. Both
  consumers are CARET-map classifications (AD-claude-two-boundary-maps-for-phantom-text-001
  — trivially satisfied today since mb rows render no phantoms; the classification is
  recorded so a future mb inlay cannot silently break it).
- **D2 — The geom hook is the `editor_geom` idiom.** A per-frame mb frame geometry (text
  x0, y0, first/last slots, cell_w, cell_h) recorded by a zero-size canvas in the paint
  pass; click, `bounds_for_range`, and `character_index_for_point` all read the SAME
  recorded frame (pure arithmetic, `None`/no-op before first paint). A test seed sibling of
  `seed_editor_geom_for_test` serves headless drives (L-claude-426: the seed is the LAST
  update before each probe).
- **D3 — Paste rides the ONE insert mechanism.** `mb_edit_prepare` → clear `mb_marked` →
  `mb_edit_transient` + ONE `edit_at_selections(set, &clip, Human)` → `mb_after_edit`.
  Single vs multi-line is NOT special-cased at the edit: a clip containing `\n` is the same
  one edit, and the window GROWS via the standing anchor arithmetic + live rebuild —
  exactly Enter's shape, one undo group, one journal entry.
- **D4 — IME rides the standing `ime` fns + `mb_marked`.** Write side through
  `mb_edit_transient` (the marked param exists since #428); read side answers from the
  caret ANCHOR + `mb_marked`, never the buffer's live `SelectionSet`
  (PR-claude-428-b). v1 point-selection report: a composition's internal selection range
  collapses to the caret head in `selected_text_range` (the mb caret is one anchor) —
  recorded, candidate-window navigation unaffected in the common path.
- **D5 — Fail-closed ladder.** The mb ⌘V arm consumes; on an mb tab no plain-⌘C/X/V falls
  through to `focused_terminal*` (the hidden-surface misdirection is a bug being FIXED, not
  preserved). `cx.stop_propagation()` on every consuming arm
  (BF-claude-overlay-keys-leak-keychar-to-ime-fallback).
- **D6 — No render machinery.** Row render (raw `StyledText`) unchanged; boundary guards
  (⌫/⌦ window edges) unchanged; the journal/undo/save chassis unchanged — paste and
  composition enter through the same `mb_after_edit` tail that already guards no-op edits
  and stale entries (F-claude-428-c).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks a multibuffer Line row, the caret shall land at the clicked COLUMN via the measured mono advance and the editor's `LineLayout` inversion — exact on plain ASCII and on MULTIBYTE lines (fixture where char count ≠ byte count, e.g. `aébc`; a wide glyph flips at its midpoint, #277), clamped to [0, line end] (gutter clicks → col 0; past-end clicks → line end); the caret bar shall re-render at that same column (`col_of_offset` round-trip). | Pure units on the mapping fn + headless drive (seeded mb geom, L-claude-426) + live drive click at a measured x. |
| REQ-002 | WHEN ⌘V fires with an armed mb caret and a single-line text clip, the clip shall insert at the caret through the one insert mechanism — visible in the file's own editor tab (write-through), caret after the inserted text, target touched+dirty, ONE journal entry — and one mb ⌘Z shall revert the entire paste (redo restores). | Headless drive (paste → read target buffer + journal len → ⌘Z → pre-text). |
| REQ-003 | WHEN the clip contains newlines, the paste shall be that same ONE edit and the caret file's window shall GROW by the pasted rows (Enter's growth semantics: rows below renumber, window edges re-resolve, hairlines/washes intact), one ⌘Z reverting the whole paste. | Headless drive asserting rendered rows + `mb_window_edges` before/after + single-undo; unit rows in the boundary table if any new pure fn lands. |
| REQ-004 | WHEN an IME composition runs on the mb surface (dead key ⌥E then `e`, and a multi-update marked sequence), the preedit shall land in the target buffer at the caret (visible in the excerpt AND the file's tab), `marked_text_range`/`selected_text_range`/`text_for_range` shall answer from the mb caret + `mb_marked` (never the buffer's live selection set), the commit shall REPLACE the marked span (exactly one `é`, no doubled glyphs), and `bounds_for_range` shall anchor the candidate window at the caret's cell from the recorded mb geometry (`None` off-viewport). | Headless drives calling the real `EntityInputHandler` methods (compose→update→commit; compose→Esc) + geometry asserts on seeded geom + LIVE drive with a real typed dead-key compose (PR-claude-428-a: a live composed char, not only direct calls). |
| REQ-005 | WHILE a multibuffer tab is active, plain ⌘V/⌘C/⌘X shall NEVER reach the workspace terminal's prompt/PTY (today's fall-through); ⌘V with no caret, or an empty/non-text clipboard, shall be a consumed no-op; a paste shall end any pending composition first; and every gesture that drops the caret (Esc, ↑/↓, click, undo/redo) shall keep dropping the composition. | Headless drives (mb tab + live terminal: ⌘V → terminal prompt buffer unchanged; no-caret ⌘V → no edit, no PTY write) + guard units. |
| REQ-006 | Every new pure surface shall hold 100% coverage / 100% MSI; `scripts/gates.sh --diff` shall be GREEN. | gate:4/5; `scripts/gates.sh --diff`. |

## Phase Plan

- **P2 Design** — the mb geometry cell's shape + recording home (first-rendered Line row's
  text cell vs list-level canvas; interaction with the uniform row pitch) and its test
  seed; the ⌘V arm's exact ladder placement + the tab-kind fail-closed gate (audit every
  hardcoded `key == "c"|"v"` intercept — PR-claude-new-chord-shadowed-by-hardcoded-key-001);
  the method-by-method mb-arm table for `EntityInputHandler` (which reuse `mb_edit_transient`,
  which answer read-only from anchor+`mb_marked`, UTF-16 conversion seams); the caret-bar
  `col_of_offset` switch; `character_index_for_point`'s mb arm (same geometry, slot →
  `locate` → col); the POC paste build plan; the test table per REQ incl. the multibyte/
  wide-glyph fixtures and the terminal-misdirection guard drive.
- **P3 Implement** — POC FIRST (paste arm built + screenshotted + READ; click-column
  verified against the standing POC reference), then Rust to the design manifest.
- **P3.5 Inspect** — critics on: ladder ordering + fail-closed routing (the terminal
  fall-through class); borrowed-state discipline in the IME read arms (PR-claude-428-b);
  journal soundness for paste entries (F-claude-428-c — no-op clips, coalescing);
  column-map classification (AD-claude-two-boundary-maps); geometry staleness
  (off-viewport, pre-first-paint, #431-adjacent seams); provenance.
- **P4 Validate** — the REQ drives; live drive: click a mid-line char at a measured x →
  caret bar at that column → ⌘V a two-line clip → window grows + file tab shows it →
  ⌥E e → é composes live (L-claude-426 click-first + clearmods recipe); parity pair
  (POC paste ↔ live paste; the click gap re-captured CLOSED); gate GREEN.
- **P5 Complete** — CHANGELOG; editor.md's deferred list loses all three seams;
  MARLEY-PARITY multibuffer row updated (mono-col-click gap closed; paste paired; IME
  recorded Rust-only); ledger; ticket closed; pipeline archived.
