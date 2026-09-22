---
pipeline_id: f1b7aeb1-f026-4913-a641-6a9a00b861d9
ticket: forge#272 (ef9424f3-45d6-4d67-8d12-c3488cb136d6) · local docs/planning/tickets/open/TICKET-272-editor-find.md
aar_id: d9e23b8d-57ea-4f12-8114-49362f47835c
status: Phase 5 — Complete PASS
title: Editor find & replace (⌘F Editor context) + ⌘A select-all
type: feature
milestone: M17
references:
  - docs/marley_architecture/editor.md
---

## Title
The editor's missing table-stakes basics: an Editor-context ⌘F
find/replace bar (the editor has NO find — #265 scoped ⌘F to the
Terminal context) and ⌘A select-all. Live match highlights render
through a GENERALIZED #266 highlight channel (N match ranges with a
distinct tint + a brighter current match — the same shape the deferred
marked-text underline wants); Enter/⇧Enter cycle wrapping via the
shipped `find::next_occurrence`; the current match centers via the
#273 `scroll_editor_to_row` mechanism; Replace One / Replace All edit
through `Buffer::edit` (undo-recorded).

## Scope
### In
- keymap: Editor-context rows `⌘F → "open-editor-find"` and
  `⌘A → "select-all"` (the #265 scoped-row idiom; terminal ⌘F/⌘A
  untouched).
- RootView editor-find state (`efind_open/query/replace/index/focus`
  — mirroring the #47 terminal find-bar state family) + a bar overlay
  rendered on the editor tab (input row, match count "n of N", replace
  row, the #47 render idiom).
- Key routing: while the bar is open, keys drive it (typed chars edit
  the focused field, Tab toggles find↔replace focus, Enter next /
  ⇧Enter prev, ⌘Enter or a Replace-All chord = replace-all, Esc
  closes back to the buffer). The arm STOPS propagation (the #267
  ladder rule) and joins `text_input_blocked()`.
- Pure seams: `find::find_all(buffer, needle) -> Vec<(CharOffset,
  CharOffset)>` (non-overlapping, ASCII-case-sensitive v1 — decide
  fold at design vs the terminal's ASCII-fold precedent);
  `replace_all_edits`/apply arithmetic (back-to-front so offsets stay
  valid); the styled_slices GENERALIZATION carrying N match ranges +
  the selected range with per-kind flags.
- Current-match behavior: selecting a match sets selection
  (anchor..caret) + `scroll_editor_to_row` centers it (two-frame
  constraint honored — the bar acts on an already-owned file).
- ⌘A: anchor=0, caret=len_chars (dispatch verb; no-op on non-editor).

### Out (explicitly deferred)
- Regex / whole-word / case-toggle options (v2 chrome).
- Project-wide search (B7 multibuffer).
- The marked-text underline itself (consumes the generalized channel
  later).
- Search-in-#246-pane (read-only pane keeps no find).

## Reference (§20)
N/A — the universal editor find-bar convention (behavior generic
across every editor); implementation Marley-original, reusing
Marley's own #47 bar idiom, #265 find.rs, #266 render seams, #273
scroll mechanism. No copyleft source consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — The bar is an EDITOR-TAB overlay (state on RootView like every
  #47-family bar), not a new widget crate.
- D2 — Highlight channel: generalize `styled_slices` to N ranges with
  kinds (match / current-match / selection) in ONE pass — designed
  once for find + the future marked underline.
- D3 — Replace All applies back-to-front in ONE loop of Buffer::edit
  calls (per-edit undo steps are acceptable v1 if coalescing is not
  free — decide at design and record).
- D4 — Matching is ASCII-case-insensitive (the #47 terminal find's
  ASCII-fold precedent) unless design finds char-count hazards; exact
  decision recorded at design.
- D5 — ⌘F while the bar is open refocuses/reseeds it from the current
  selection (the universal convention) — cheap; confirm at design.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `find_all` shall return every non-overlapping match range in order (multibyte-safe, empty needle → none), and the bar shall show "i of N" live as the query edits. | pure units + headless |
| REQ-002 | Enter/⇧Enter shall cycle the current match forward/backward WRAPPING, set the selection to the match, and center it via the shared scroll mechanism when off-screen. | headless (selection + scroll asserts) |
| REQ-003 | Match highlighting shall tint every match distinctly from the selection tint, with the CURRENT match brighter, coexisting with syntax tints per row. | pure slices units + driven capture |
| REQ-004 | Replace One shall replace the current match and advance to the next; Replace All shall replace every match in one action with offsets staying correct (back-to-front), both undo-recorded. | pure arithmetic units + headless undo assert |
| REQ-005 | While the bar is open, typing shall edit the BAR (never the buffer), the arm shall stop propagation (no IME fallback leak), and Esc shall close and return typing to the buffer. | headless (the #267 overlay-leak pattern test) |
| REQ-006 | ⌘A on an editor tab shall select the whole buffer (anchor 0, caret len); on a terminal tab ⌘A shall behave exactly as before. | headless both surfaces |
| REQ-007 | The terminal ⌘F scrollback bar shall be byte-identical (untouched). | existing tests + capture |

## Phase Plan
- **P2 Design** — the slices generalization shape, find_all/replace
  arithmetic, bar state + key table, keymap rows, test plan.
- **P3 Implement** — find.rs additions; code_view slices; app state +
  bar + routing; keymap rows.
- **P3.5 Inspect** — critics (offset arithmetic under replace; overlay
  routing vs #267 rules; channel regression on #266 rows).
- **P4 Validate** — units + headless + driven; gate --diff.
- **P5 Complete** — CHANGELOG, editor.md, AAR, archive, close.
