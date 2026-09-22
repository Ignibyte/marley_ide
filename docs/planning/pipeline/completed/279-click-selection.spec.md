---
pipeline_id: 84ff971f-2aaf-437d-871e-fbcba315e1df
ticket: forge#279 (520339d5-a2e1-4d9f-b36b-6051cb10b500) · local docs/planning/tickets/open/TICKET-279-click-selection.md
aar_id: 50f7723b-92bb-4873-b34e-e591d0e815d7
status: Phase 5 — Complete PASS
title: Double-click word / triple-click line / ⇧-click extend in the terminal
type: feature
milestone: M17
references:
  - docs/marley_architecture/app_shell.md
---

## Title
Selection ergonomics on the terminal grid: double-click selects the
word under the pointer (a terminal word class that keeps paths/URLs
whole), triple-click selects the row, ⇧-click extends the existing
selection. All three feed the EXISTING #43/#44 Selection/copy pipeline
— what highlights is exactly what copies (the paired PR rule).

## Scope
### In
- Pure (text_selection.rs): `CharClass { Word, Space, Punct }` +
  `classify` (separators = whitespace + `()[]{}<>"',;=` + backtick;
  path/URL-friendly `./-_~:@?&#%` stay Word so `/path/to/file.rs`
  and `https://…` select whole); `word_bounds_at(row, col) ->
  Option<(usize, usize)>` — the run of the SAME class around col
  (iTerm-style: a click on punctuation selects the punct run; on
  whitespace the space run); col ≥ len → None; `row_bounds(row) ->
  (usize, usize)` = (0, chars) — the full row; `extend_selection
  (existing: Option<Selection>, pos) -> Selection` (keep the anchor;
  no existing → seed at pos).
- Shim (the grid left-mouse-down): shift → extend (before any seed);
  click_count == 2 → word bounds over the row text at pos (the
  `content_row_texts` source — the SAME rows copy uses); ≥ 3 → row
  bounds; plain single-click keeps the exact #43 seed. ⌘-click keeps
  the #196 link-open precedence UNTOUCHED (the link handler lives on
  the row spans — a ⌘ mouse-down must not clobber the selection
  before the link click fires: gate the new branches AND the seed on
  `!platform`... verify the current behavior first at implement and
  keep it byte-identical for ⌘).
- Word/line DRAG-extend: implement only if it drops out of the
  existing drag path cheaply; else RECORD as the follow-up (the
  ticket pre-authorizes).

### Out
- Grapheme clusters in the word class (chars only, like the grid).
- Multi-row double-click word wrap-around (a word split across rows
  selects its row's part only — grid semantics).

## Reference (§20)
The universal terminal selection convention (xterm/iTerm/every
terminal — public behavior class). Marley-original word class over
Marley's own grid Selection. No copyleft source consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Same-class-run semantics (word OR punct-run OR space-run) —
  matches iTerm; a click on `=` in `a=b` selects `=` alone.
- D2 — Path chars `./-_~:@?&#%` are Word: the daily case is grabbing
  a path or URL whole.
- D3 — Triple-click = the WHOLE row (incl. trailing spaces the grid
  holds) — copy trims nothing the highlight shows.
- D4 — ⇧-click extends the HEAD only (the anchor is sacred); with no
  selection it seeds.
- D5 — Multibyte: bounds are CHAR indices over the row text (the
  Selection columns are grid cells = chars in these rows).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `word_bounds_at` shall return the same-class run for Word/Punct/Space clicks with the D2 class (paths + URLs whole; `()[]{}<>"',;=` separate; multibyte exact) and None past EOL. | pure units (kill list) |
| REQ-002 | Double-click shall select exactly the word run under the pointer; triple-click the full row; both highlighting AND copying identically (copy_payload over the same Selection). | headless state asserts + the copy pairing |
| REQ-003 | ⇧-click shall extend the existing selection's head to the clicked cell keeping the anchor; with no selection it shall seed at the cell. | pure units + headless |
| REQ-004 | A plain single click shall behave byte-identically to pre-#279 (seed anchor=head at the cell); ⌘-click link-open precedence shall be unchanged. | existing tests + headless |

## Phase Plan
- P1+P2 combined; P3 implement; P3.5 critic (class-table completeness,
  the ⌘/⇧ precedence matrix, drag interplay); P4 units + headless +
  gate; P5 docs. Drag-extend recorded if not cheap.
