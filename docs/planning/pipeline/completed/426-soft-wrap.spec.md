---
pipeline_id: e95a3df6-2b61-4022-816e-e0d042da76a6
ticket: docs/planning/tickets/open/TICKET-426-soft-wrap.md
status: Phase 5 — Complete PASS
title: Soft wrap — one buffer line → N display rows at the pane's cell width
type: feature
milestone: M32
references:
  - docs/planning/design-notes/display-map-shelf.md
  - docs/planning/pipeline/completed/425-display-map-foundation.spec.md
  - docs/zed_architecture/subsystems/03-editor-multibuffer.md
  - docs/marley_architecture/editor.md
---

## Title

The B-c chain's step 2 — the highest-value visual feature of the display map. With
`editor.soft_wrap` ON, a buffer line wider than the code viewport renders as N stacked
display rows (word-boundary breaks, continuation indent), the gutter numbers only the
first row, caret/click/selection/decorations work per segment, ↑/↓ move by display row,
and the whole h-scroll apparatus goes inert (nothing clips, so nothing scrolls
horizontally). OFF (the default) is byte-identical to today. Wrap is the display map's
SECOND layer — it inserts inside the #425 facade, composing fold∘wrap, and the crossing
sites do not change count.

## Scope

### In
- A pure wrap seam (break-point computation in display CELLS over the whole-line
  `LineLayout`; segment slicing that clips-and-re-bases the display string + all
  byte-range producers).
- The facade layer: `DisplayMap` grows the wrap stage; `visible_count`/`buffer_row`/
  `slot_of`/`viewport_offset` compose folds∘wrap; a segment-aware row model at the rim
  ((buffer row, segment) per slot).
- The rim render under wrap: per-segment StyledText, caret bars, selection/find/bracket/
  squiggle bands (split at boundaries), gutter first-row-only numbering, git-lane/tint
  policy (design pins), the ⋯ N lines phantom on the header's LAST segment, sticky band
  correctness.
- ↑/↓ display-row motion with goal-column preservation (`move_vertical_goal`'s
  display-row twin; the char-vs-cell goal domain reconciled at design).
- Jump/reveal/anchor correctness: `scroll_editor_to` targets the first segment;
  the five overlay cards + both IME sites anchor at the caret's segment.
- `editor.soft_wrap` (bool, default OFF) end-to-end (the six-edit settings pattern) +
  "Toggle Soft Wrap" palette command (next free CommandId) + live flip (memo nulled,
  scroll_x reset).
- The h-scroll inert set under wrap-ON: wheel-x, `content_px` write, clamp, both follow
  shims, the thumb — each gated; `scroll_x` pinned 0.
- React-first: wrap designed in the POC `EditorView.tsx` first (width from the code
  box's rect + a ResizeObserver; a `wrap.ts` mirroring the Rust rules; gutter + caret
  per segment), screenshot-inspected, then ported 1:1.

### Out (explicitly deferred)
- The read-only viewer twin (#246 pane; fully disjoint path, already truncates — stays).
- Home/End/word/page/document motions: Home/End stay buffer-line this ticket (recorded
  deferral — the display-row form is a follow-up polish; PageUp/Down do not exist in the
  editor at all); ⌘↑/⌘↓ and word motions unchanged.
- Any change to fold semantics, `FoldProjection`, or the column model's #331 invariants.
- The POC `SplitTerminalView` file cells (a pre-existing POC simplification with
  hard-coded metrics; the parity surface is `EditorView`).
- Wrap-width modes (wrap-at-N-columns à la Zed's preferred_line_length) — v1 wraps at
  the pane width only.

## Reference (§20)

**Zed (the editor reference — same-gpui-stack).** Behavior matched: soft wrap breaks a
too-wide line at the viewport's wrap width with word-boundary preference; vertical
motion operates in DISPLAY space ("move down moves down one visible line" —
`docs/zed_architecture/subsystems/03-editor-multibuffer.md` §4), the wrap layer sits in
the display-map stack between folds and blocks (§3's WrapMap row), and the gutter
numbers logical lines (continuation rows blank — the Zed/VS Code shared convention).
Cited research: `03-editor-multibuffer.md` §3 (the WrapMap layer + uniform contract)
and §4 (display-space movement); `docs/zed_architecture/crates/editor.md:254-260`
(TabMap→WrapMap first in the recommended Marley order). Clean-room: behavior docs +
permissive-dep source only; no Zed source read.

### Prior art

1. **Behavior maps** — the two Zed chapters above; `crates/language.md:147`
   (soft-wrap settings live in language settings — Marley v1 keeps one global bool).
2. **Published** — the greedy word-wrap algorithm is textbook; VS Code's `wordWrap`
   off-default + first-row-only numbering corroborate the convention.
3. **Our permissive deps (swept 2026-08-14, registry source):**
   - **gpui 0.2.2 `LineWrapper::wrap_line`** (`text_system/line_wrapper.rs:6-110`) —
     READ in full. Its wrap RULES are adopted: greedy accumulate; a word char after a
     space marks the last break candidate; ANY non-word char is a candidate (the CJK
     clause — "CJK may not be space separated"); on overflow break at the last
     candidate, else hard-break at the overflowing char; continuation rows carry the
     line's leading-whitespace indent (capped — gpui caps at 256). Its MACHINERY is
     deliberately NOT adopted: it measures px via `PlatformTextSystem` + per-char font
     caches because gpui text is proportional — Marley's editor is a mono CELL grid, so
     the same rules re-express as pure integer arithmetic over
     `code_view::char_width`/`LineLayout` display cells (the ONE column authority,
     #277/#336). A locked "adopt LineWrapper?" question DISSOLVED into "adopt its rules,
     keep our substrate" — recorded as the sweep's win.
   - **In-tree**: `LineLayout.display`/`col_starts`/`col_ends`/`hint_spans` +
     `display_cols()` (#331/#336) already carry every cell/byte mapping the slicer
     needs; `cols_to_bytes` is the existing cell→byte clipper; the #425 facade is the
     insertion point built for exactly this layer.
   - ropey/regex/tree-sitter: no wrap ownership (checked).

## React-first (parity)

**UI-AFFECTING — Zone A, the editor surface row.** React files (per MARLEY-PARITY.md):
`components/EditorView.tsx` (+ a new `utils/wrap.ts`; `utils/docShare.ts` gains one long fixture line so the wrap is demonstrable — amended at inspect).
Implement builds the wrap in the POC FIRST: measure the code box width (its rect is
already probed at `EditorView.tsx:136-154` — today `.width` is discarded) + a
ResizeObserver, compute segments with the same rules as the Rust seam, render
continuation rows with first-row-only gutter numbers and the continuation indent, keep
caret/click segment-aware; `pnpm --filter @workspace/marley-ide run dev` →
localhost:5173, screenshot + READ the PNG at ON and OFF; then port 1:1. Validate
captures the React↔Marley parity pair at the same state. The `SplitTerminalView` file
cells stay out (pre-existing POC simplification — recorded in Scope/Out).

## Locked-In Decisions

- **D1 — Wrap is a facade layer, not a render hack.** The #425 `DisplayMap` grows the
  wrap stage internally (folds∘wrap); the rim consumes a segment-aware row model; the
  14 crossing sites keep their shapes.
- **D2 — Cell-domain computation.** Break points are computed in display cells over the
  whole-line `LineLayout` (phantoms occupy real cells — a trailing hint wraps like
  text); the wrap width is `floor(code_w / cell_w)` cells from the frame geom (the
  previous-frame `content_px` precedent; a resize re-wraps next frame). gpui's
  LineWrapper rules, Marley's grid substrate (see Prior art).
- **D3 — Clip and re-base, never re-lex.** One whole-line `LineLayout` per line;
  segments are cell-range slices of its display string; syntax/selection/find/bracket/
  squiggle byte ranges are clipped and re-based per segment. No per-segment re-parse,
  no second layout pass.
- **D4 — Scope: the editable list only.** The editor tab + the focused editable split
  pane (one code path). The read-only viewer keeps its truncation. Home/End stay
  buffer-line (deferred, recorded); ↑/↓ get the display-row twin of
  `move_vertical_goal` — the ONE motion function that carries the goal column.
- **D5 — OFF default; ON inerts h-scroll.** `editor.soft_wrap = false` default. ON pins
  `scroll_x` to 0 (reset at toggle + at the park/restore door) and gates the enumerated
  consumer set (wheel-x, content_px write, clamp, follow shims, thumb). The #336
  clipper/shift structure REMAINS for the OFF path — wrap-ON simply never shifts
  (revisiting, not removing, the shift sites per the AD).
- **D6 — Goal column reconciliation is pinned at design.** Vertical motion under wrap
  moves in display-row space; `Selection.goal_col` (a char column today) stays the
  storage; design specifies the char↔cell reconciliation and its zero-width/wide-glyph
  behavior (PR: probe zero-width boundaries).
- **D7 — Fold composes above wrap.** Hidden lines produce no segments; the ⋯ phantom is
  part of the header line's layout and lands on its LAST segment; sticky uses the first
  visible segment's buffer row (the partially-scrolled-header edge pinned by test).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `editor.soft_wrap` is ON and a line's display width exceeds the viewport's cell width, the editor shall render it as N stacked display rows whose concatenated cells equal the unwrapped display string, with no horizontal clipping. | Pure-seam units; headless visible-count/render tests; live drive + READ the PNG. |
| REQ-002 | WHEN `editor.soft_wrap` is OFF, behavior shall be byte-identical to pre-426 (rendering, geometry, motion, h-scroll). | Full suite unchanged; identity units incl. zero-width boundary probes. |
| REQ-003 | The wrap layer shall break at the last word-boundary candidate at or before the limit (space→word transitions; any non-word char a candidate), hard-break only when a segment has no candidate, and indent continuation rows by the line's leading-whitespace cells (capped). | Unit table: ASCII/space runs, CJK, tabs, zero-width, trailing-phantom, long-token hard-break, indent cap. |
| REQ-004 | A wrapped line's gutter number shall render on its FIRST display row only; continuation rows shall render an empty gutter cell of identical width. | Row-model unit + headless render assert. |
| REQ-005 | Clicking any cell of any segment shall place the caret at the char whose display cell contains the click, and the caret bar shall render on the caret's segment at its within-segment column. | Geometry units (segment inverse) + headless click drives. |
| REQ-006 | Selection, find, bracket, and squiggle ranges crossing a wrap boundary shall paint on every segment they cover, clipped and re-based. | Slice units + headless. |
| REQ-007 | WHEN wrap is ON, ↑/↓ shall move by one DISPLAY row preserving the goal column across segments; WHEN OFF, vertical motion is unchanged. | Movement units + headless drives (incl. segment→segment, segment→next-line, goal through short lines). |
| REQ-008 | Jumps (goto/def/find/problems) shall land the target's FIRST segment in view with fold auto-reveal intact, and overlay cards + IME shall anchor at the caret's SEGMENT viewport offset. | Headless jump + seeded-geom anchor tests (the W-1 verbs). |
| REQ-009 | WHEN wrap is ON, `scroll_x` shall pin to 0, the h-thumb shall not render, wheel-x and the caret-follow shims shall be inert; toggling ON shall reset any parked scroll_x. | Unit gates + headless toggle test. |
| REQ-010 | "Toggle Soft Wrap" (palette) shall flip the behavior live and persist `editor.soft_wrap`; boot shall seed from the setting. | Headless toggle + settings round-trip unit; the command-table guard test. |
| REQ-011 | WHEN folds and wrap are both active, hidden lines shall produce no display rows and the fold ⋯ marker shall render on the header's LAST segment. | fold∘wrap composition units. |
| REQ-012 | The new pure surface shall hold 100% line coverage and 100% MSI; the gate shall be GREEN [diff]. | gate:4/5 exit codes; `scripts/gates.sh --diff`. |

## Phase Plan

- **P2 Design** — the wrap seam's exact types (segment model, break fn signature, the
  facade's slot→(row,segment) shape); the goal-column reconciliation (D6); the gutter/
  git-lane/tint per-segment policy; the h-scroll gate placement; the settings/palette
  edits; the POC build plan; the regression test table per REQ.
- **P3 Implement** — POC FIRST (build + screenshot + read), then Rust to the manifest.
- **P3.5 Inspect** — critics on: boundary math (off-by-one at break points), the
  re-base correctness (byte vs cell vs char domains), OFF-path identity, PR-#305 reveal
  paths, provenance.
- **P4 Validate** — the REQ table's tests written + run; parity pair captured (React ↔
  live Marley, same state, pixels sampled); `scripts/gates.sh --diff` green.
- **P5 Complete** — CHANGELOG + editor.md/roadmap; MARLEY-PARITY.md editor-surface row
  re-baselined for wrap; ledger capture; close + archive.
