---
pipeline_id: c8752af9-80a6-4a3a-9184-f5a301af3376
ticket: forge#276 (2af791c8-03bb-45b6-93e2-3fa91e3443f8) · local docs/planning/tickets/open/TICKET-276-auto-indent.md
aar_id: 9cfbf20a-0cdd-49a2-85b5-428da6918264
status: Phase 5 — Complete PASS
title: Auto-indent on Enter + Tab/⇧Tab indent-dedent
type: feature
milestone: M17
references:
  - docs/marley_architecture/editor.md
---

## Title
The v1 (non-tree-sitter) indentation layer that makes code typing feel
right: Enter clones the current line's leading whitespace (caret-aware),
Tab with a selection / ⇧Tab always run line indent/dedent ops with the
selection following, and a bare-caret Tab inserts spaces to the next
tab stop. Tab/⇧Tab become ROUTER-handled editor keys (the #267 table —
today Tab reaches the buffer as a literal \t via the IME fallback and
⇧Tab does nothing).

## Scope
### In
- Pure (marley_editor or code_view — decide at design; leans
  marley_editor::indent for buffer ops + a code_view tab-stop helper):
  `indent_for_newline(line, caret_col) -> String` (leading whitespace
  clone, clipped at the caret when it sits inside the indent);
  `indent_lines`/`dedent_lines` over the selection's line span (spaces
  per tab_width; dedent strips up to ONE stop of leading
  spaces-or-tab, per line) returning the edits + rebased
  (anchor, caret); `spaces_to_next_tab_stop(col, tab_width)`.
- app.rs editor key arm: Enter → auto-indent insert (replaces the bare
  apply_editor_key '\n' path for the editor); Tab/⇧Tab rows (selection
  → indent/dedent ops; bare-caret Tab → the stop-padding insert;
  ⇧Tab bare-caret → dedent the current line); all router-handled with
  stop_propagation; `clear_marked` per #267.
- key_from_keystroke: "tab" mapping visibility (today Tab is Char('\t')
  via key_char; ⇧Tab arrives as "tab"+shift with NO key_char) — the
  editor arm branches BEFORE the Char claim.
- Terminal untouched: the #89 cooked-prompt Tab completion and the raw
  Tab/BackTab streaming (#41) keep their arms (they sit in later ladder
  positions / the terminal route).

### Out
- tree-sitter indent queries (language-aware — the recorded B3 slice).
- A hard-\t insert setting (spaces-only v1, matching the render).
- Re-indent-on-paste; block-comment continuation.

## Reference (§20)
N/A — the universal editor indent convention (clone-previous-indent +
tab-stop ops); Marley-original implementation over Marley's own
buffer/selection seams. No copyleft source consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Enter clones ONLY whitespace left of the caret when the caret
  sits inside the indent (the universal behavior); full leading
  whitespace otherwise.
- D2 — Indent unit = spaces to tab_width (the render's expansion);
  dedent strips min(one stop of spaces, or one \t) per line.
- D3 — Selection/caret REBASE through the line ops keeps the same
  TEXT selected (grow/shrink by the per-line deltas before each
  endpoint).
- D4 — Tab with NO selection inserts stop-padding at the caret; ⇧Tab
  with no selection dedents the caret's line (caret follows).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Enter after an indented line shall start the new line at the same indentation (whitespace clone), and Enter with the caret INSIDE the leading whitespace shall clone only up to the caret. | pure units + headless |
| REQ-002 | Tab with a multi-line selection shall indent every touched line by one stop with the selection covering the same text after; ⇧Tab shall dedent likewise (lines with less than one stop lose what they have). | pure units (rebase math) + headless |
| REQ-003 | A bare-caret Tab shall insert spaces to the next tab stop (column-aware); a bare-caret ⇧Tab shall dedent the current line. | pure units + headless |
| REQ-004 | Tab/⇧Tab on the editor shall be router-handled (stop propagation — no literal \t via the IME fallback, no double effect); the terminal's Tab completion and raw Tab/BackTab streaming shall be byte-identical. | headless both surfaces |
| REQ-005 | Every indent op shall be undo-recorded: Enter (incl. over a selection) shall restore in ONE ⌘Z; multi-line Tab/⇧Tab shall unwind per-line back-to-front with the full sequence restoring the exact pre-op text (the #272 replace-all v1 undo convention — grouped undo is a recorded follow-up). | headless undo asserts |

## Phase Plan
- P2 exact op signatures + rebase arithmetic + the arm diff; P3
  implement; P3.5 critic (rebase fenceposts, mixed tabs/spaces, the
  routing regression matrix); P4 units+headless+driven+gate; P5 docs.
