---
pipeline_id: 2294c06a-a90a-49ae-a6cf-3b5b2110f3f5
ticket: forge#300 (ed4f3307-e982-4c03-a256-dfd2570186b6) · local docs/planning/tickets/open/TICKET-300-move-and-duplicate-lines.md
aar_id: 422cffde-0ca1-48fa-a671-2def254a4441
status: Phase 5 — Complete PASS
title: Move lines (⌥↑/⌥↓) + duplicate lines (⇧⌥↑/⇧⌥↓) — multi-cursor blocks, cursors carried
type: feature
milestone: M19
references: [the #299 toggle-comment idiom (app.rs:6676 — Vec<LineEdit> + begin/end_undo_group + .rev() apply), indent::touched_rows/LineEdit (indent.rs:66/:14), THE CLAMP TRAP: rebase_selections collapses a cursor inside a removed span (indent.rs:131 — a move's removed span IS the cursor's row), the ropey phantom last row ("a\nb\n".len_lines()==3, buffer.rs:1165), all four chords verified FREE (recon 2026-07-17), scroll_editor_to_row (app.rs:11029)]
---

## Title
⌥↑/⌥↓ move the line — or the whole contiguous block of lines the cursors touch — up/down one row, carrying
the carets and selections with the text, so a held ⌥↓ walks code down the file. ⇧⌥↑/⇧⌥↓ duplicate the
block (the copy lands on the pressed side; the cursors follow the copy). One undo unit per press. The
universal reordering ops; today moving code means cut-and-paste.

**A prior plan pass already ran** (sprint #32; the local TICKET-300 doc survives) and its three findings
are binding inputs, not things to rediscover — the central one: **the cursors CANNOT be rebased.**

## Scope
### In
- **The pure seams (crates/editor, cov/MSI 100):**
  `move_lines(buffer, set, dir) -> Option<(Vec<LineEdit>, SelectionSet)>` and
  `duplicate_lines(buffer, set, dir) -> (Vec<LineEdit>, SelectionSet)` — each returns the ascending edit
  list AND the **carried** cursor set. **D-CARRY-NOT-REBASE (the prior plan's F1):**
  `indent::rebase_selections` CLAMPS a position inside a removed span to that edit's start
  (indent.rs:131-133), and a move's removed span IS the row the cursor sits on — rebasing would collapse
  every cursor to a line start. The seam computes the carried positions itself (offset arithmetic: the
  cursor's char-delta within its block is invariant; the block's start shifts by the swapped gap-line's
  length). `None`/identity at the edge (block already at the first/last line) — a no-op, cursors intact.
- **Block grouping:** the rows any cursor/selection touches (`indent::touched_rows` union), coalesced into
  maximal contiguous runs; each run moves independently. **The prior plan's F2, kept:** two blocks can
  never collide (proven exhaustively then — runs are separated by ≥1 gap row and a move consumes exactly
  one gap row on the moving side), so no collision handling exists to get wrong.
- **The trailing-newline edge (the prior plan's F3, kept):** operate on the line list INCLUDING ropey's
  phantom empty last row and re-join — only the BOUND needs the phantom rule (an empty last row exists iff
  the file ends with `\n`; verified: `"a\nb\n".len_lines()==3` with `line_text(2)==""`, buffer.rs:1165).
  The no-trailing-newline last line must round-trip byte-identically through a move (the classic eaten-`\n`
  bug, pinned by test).
- **The apply idiom = the shipped #299/Tab pattern** (app.rs:6676): snapshot `before`, build the edits,
  `begin_undo_group(before)` → raw `buffer.edit()` calls BACK-TO-FRONT → `end_undo_group(carried)` →
  `set_selection(carried)`. One press = one undo unit; a subsequent typed char is its OWN unit (the #338
  redo lesson — never let the group leak).
- **Duplicate semantics (observed):** ⇧⌥↓ copies the block below with the cursors on the LOWER copy; ⇧⌥↑
  copies above with the cursors staying on the UPPER copy — both directions leave the caret on the copy at
  the pressed side. No edge no-op for duplicate (duplicating at the last line is fine).
- **The chords:** ⌥↑ `(F,F,T,F,"up")`, ⌥↓, ⇧⌥↑ `(F,F,T,T,"up")`, ⇧⌥↓ — **all four verified FREE** (the
  only alt-arrow rows are ⌘⌥ focus/add-cursor). Editor-scoped; roster 67→71, scoped 22→26 (assert each
  chord individually FIRST — the #337 discipline). After a move, `follow_editor_caret()` keeps the primary
  in view (the shared primitive centers; a held ⌥↓ tracks).
### Out (explicitly)
- Move/duplicate by SELECTION-shape (column blocks); swap-with-fold interplay (#305 lands separately —
  when both exist, moving across a fold is that ticket's regression row, noted for its promotion);
  copy-line-without-selection variants (⌘⇧D-style); cross-file drags.

## Reference (§20)
VS Code / Zed / JetBrains = OBSERVED (⌥↑↓ move, ⇧⌥↑↓ duplicate, block coalescing, cursors ride, one undo
step). The seams and the carry math are Marley-original over shipped buffer primitives.

### Prior art
1. **Behavior maps / observed** — the semantics above, incl. the copy-lands-on-pressed-side detail.
2. **Published material** — none needed (a pure text-reorder op).
3. **OUR OWN CODE — the sweep found the trap AND the idiom:** `indent::rebase_selections` exists and is
   the WRONG tool here (the clamp — its doc says so; the prior plan hit it); the #299 toggle-comment apply
   sequence (app.rs:6676) is the RIGHT idiom copied whole (`touched_rows` + `LineEdit` + grouped raw edits
   back-to-front). ropey (permissive) owns the line model; its phantom-last-row behavior is pinned by an
   existing buffer test we extend, not fight. No new deps.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-CARRY-NOT-REBASE** — the seam RETURNS carried cursors; rebase_selections is never called on a move.
- **D-MAX-CONTIGUOUS-BLOCKS** — runs move independently; the no-collision proof stands (re-verify at
  promotion, don't re-prove).
- **D-PHANTOM-ROW-INCLUSIVE** — line-list-including-phantom + re-join; the bound rule is the only edge.
- **D-ONE-UNDO-UNIT** — the #299 group idiom; the group never leaks into the next keystroke.
- **D-EDGE-NOOP** — a block at the buffer's first (up) / last (down) line: nothing changes, cursors intact.
- **D-COPY-ON-PRESSED-SIDE** — duplicate's cursor destination, pinned by table.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | move a single line down/up one row with the caret riding its text (round-trip byte-identical) | pure table |
| REQ-002 | move the whole contiguous block a multi-line SELECTION touches, selection carried | pure table |
| REQ-003 | move each DISJOINT cursor block independently in one press, one undo unit | pure + headless |
| REQ-004 | no-op at the first line (up) / last line (down), cursors untouched | pure |
| REQ-005 | round-trip the no-trailing-newline last line byte-identically through a move (no eaten/invented `\n`) | pure — the F3 pin |
| REQ-006 | duplicate the block with the copy and the cursors on the pressed side (both directions) | pure table |
| REQ-007 | revert one press with ONE ⌘Z (and a following typed char with its own ⌘Z — the group does not leak) | headless — the #338 lesson row |
| REQ-008 | keep the primary caret in view through a held ⌥↓ walk (follow via the shared primitive) | headless |
| REQ-009 | merge two cursors on ADJACENT lines into ONE moving block (the #296 semantics) | pure |

## Phase Plan
P2 confirm the carry math on the gap-line-length shift (a table over blocks × directions × multibyte gap
lines — the #336/#339 unit discipline: char-deltas, not byte guesses) + re-verify the no-collision proof +
the TICKET-300/AAR reuse; P3 the pure seams first (their truth tables), then the four chords + dispatch +
follow; P3.5 critics on the carry offsets (multibyte lines, a selection spanning block edges, the phantom
row), the undo-group leak, the edge no-ops; P4 tables + headless drives (incl. the held-walk + one-⌘Z
rows) + gate; P5 docs. Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
