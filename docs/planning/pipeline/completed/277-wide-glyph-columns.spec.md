---
pipeline_id: 86a7842f-882d-4205-b1e8-008b165b228b
ticket: forge#277 (d656abeb-06e0-4038-90bc-7dd023feae68) · local docs/planning/tickets/open/TICKET-277-wide-glyph-columns.md
aar_id: b3d750a5-9201-4cee-a020-206f01cae745
status: Phase 5 — Complete PASS
title: Wide-glyph (CJK/emoji) column width in the editor
type: bug
milestone: M17
references:
  - docs/marley_architecture/editor.md
---

## Title
Real display columns for wide glyphs: the #250 `line_layout` gains a
UAX#11 char-width hook (unicode-width 0.2.2 — already in the lock;
wide/fullwidth = 2, zero-width/combining = 0, else 1) so on lines
containing 日/😀 the caret bar, click→caret mapping, selection tint,
find-mark bands, and the #268 syntax-span remap stop drifting left of
the text (the drift compounds per wide char). #267 made CJK input WORK;
this makes it ALIGN.

## Scope
### In
- `code_view::line_layout` — per-char width via the hook (tab stays
  positional: advance to the next stop over the ACCUMULATED col).
- **The premise correction (found at plan recon):** `cols_to_bytes` is
  display-CHAR-INDEX based (`char_indices().nth(col)`) — columns and
  display char indices coincide ONLY under all-width-1. It must become
  column-aware (a width-accumulating walk over `display`), or every
  consumer (the #266 selection highlight, the #272 mark bands via
  `raw_span_to_display_bytes`) maps wide-line highlights to the wrong
  bytes. The ticket's "downstream follows for free" holds for
  `col_of_offset`/`offset_of_col`/`offset_for_click`/
  `row_selection_cols` (col_starts-driven) but NOT for the two
  byte-mapping fns.
- Wide/zero-width test rows for every seam fn (the fixture family
  "aé日😀\tw" + a decomposed-combining case).
- The caret x (`ccol * cell.w`) and click col (`x / cell.w`) shims are
  already column-driven — headless asserts pin them on wide fixtures.

### Out
- TERMINAL columns (alacritty owns its grid widths).
- Grapheme clustering / emoji SEQUENCES: ZWJ families (👨‍👩‍👧) sum
  component widths — OVER-count (one ~2-cell glyph, model width 6);
  VS16/keycap sequences (❤️, #️⃣) UNDER-count (model 1, renders ~2;
  probe-verified on 0.2.2 — the STR-level `UnicodeWidthStr::width` is
  already sequence-correct, so a grapheme-cluster walk feeding it per
  cluster is the eventual fix). Both directions are the recorded
  follow-up; single-scalar emoji (😀=2) are in scope and correct.
- Font-fallback advance verification beyond what a capture can show
  (the mono font may render CJK at a non-2× advance; pin what IS true,
  note any residual).
- A goal-column memory for vertical motion (unchanged from #257).

## Reference (§20)
UAX#11 East-Asian-Width (the public Unicode spec, via the
unicode-width crate's standard table) — the universal terminal/editor
convention for display cell width: W/F → 2 cells, zero-width marks →
0, else 1. Marley-original wiring into Marley's own #250 layout seam.
No copyleft source consulted; the crate is MIT/Apache dual-licensed
and already lock-vetted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Columns stay a MAPPING: the `display` string is unchanged (no
  padding chars injected); `col_starts` simply advances by
  `char_width(ch)`.
- D2 — `char_width`: `\t` positional (pre-hook); unicode-width `None`
  (control chars) → 1 (every char stays caret-reachable); `Some(w)` →
  w. One shared fn used by BOTH the raw-line layout walk and the
  display-side col→byte walk.
- D3 — `cols_to_bytes` becomes width-aware over `display` (columns →
  display bytes by accumulating `char_width`); its signature keeps the
  same shape (display + cols) so callers only re-thread if the
  compiler says so. A col landing INSIDE a wide char maps to that
  char's start byte (band edges never split a glyph).
- D4 — Zero-width chars produce equal adjacent `col_starts`;
  `offset_of_col`'s later-wins tie rule stands (a click can never land
  between a base char and its combiner — desired), and the caret bar
  may draw at the same x for two adjacent offsets (accepted v1
  cosmetic, noted).
- D5 — unicode-width pinned at the lock's 0.2.2; added as a direct
  dep of `marley_app` only.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `line_layout` shall assign display columns by UAX#11 width — for "a日b😀c" col_starts = [0,1,3,4,6,7] (wide=2) and for a decomposed "e"+U+0301 the combiner shall occupy 0 columns. | pure units |
| REQ-002 | The caret column and click mapping shall align on wide lines: `col_of_offset`/`offset_of_col`/`offset_for_click` shall round-trip on the wide fixture family, with a click inside a wide glyph's right half landing after it (tie rule) and zero-width offsets never click-reachable mid-cluster. | pure units + headless caret asserts |
| REQ-003 | `cols_to_bytes` (and `raw_span_to_display_bytes` through it) shall map display COLUMNS — not display char indices — to bytes on wide lines: the selection/mark/syntax highlight byte ranges on "日x" for cols [2,3) shall be the bytes of "x" (not past-end). | pure units |
| REQ-004 | `row_selection_cols` shall return real display columns on wide lines (selecting 日 alone yields a width-2 band [c, c+2)). | pure units |
| REQ-005 | Tab stops shall compose with wide widths: in "日\tx" (tw 4) the tab shall advance from col 2 to col 4 (stop math over accumulated width), display "日  x". | pure units |
| REQ-006 | ASCII-only lines shall be byte-identical in behavior AND `col_starts` to pre-#277 (no regression on the entire existing suite). | full suite green |

## Phase Plan
- P2 exact fn diffs + the width-fn home + the display-side walk +
  test plan; P3 implement; P3.5 critic (the char-vs-col conflation
  hunt across ALL display consumers — incl. CodeViewState's max-cols
  truncation and the render's StyledText ranges; width-table edges);
  P4 units + headless + (driven if unlocked; else ENV-BLOCKED
  protocol) + gate; P5 docs.
