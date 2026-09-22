---
pipeline_id: e989f753-f5b7-4fe0-9abd-ba21f3c5acc2
ticket: forge#300 (ed4f3307-e982-4c03-a256-dfd2570186b6) · local docs/planning/tickets/open/TICKET-300-move-and-duplicate-lines.md
aar_id: 422cffde-0ca1-48fa-a671-2def254a4441
status: Phase 2 — Design PASS; ready for Phase 3 — Implement
title: ⌥↑/⌥↓ move lines · ⇧⌥↑/⇧⌥↓ duplicate lines — multi-cursor aware, one undo unit
type: feature
milestone: M19
references: [docs/planning/pipeline/completed/299-toggle-line-comment.spec.md, docs/planning/pipeline/completed/297-multi-cursor-gestures-and-shim.spec.md]
---

## Title
**⌥↑/⌥↓ move the lines. ⇧⌥↑/⇧⌥↓ duplicate them.** The universal reordering ops — absent today, so moving code
means cut-and-paste. Multi-cursor aware, one undo unit per press, and the cursors ride their text.

## Scope

### In
- **`move_lines(buffer, rows, dir)`** and **`duplicate_lines(buffer, rows, dir)`** (pure, `crates/editor`) —
  the edits **and the carried cursors** (see the constraint below; the cursors cannot be rebased).
- **`contiguous_runs`** — group the flat row list into maximal contiguous blocks (each moves independently).
- **Keymap**: four NEW Editor-scoped rows (⌥↑, ⌥↓, ⇧⌥↑, ⇧⌥↓) + the dispatch arms.
- REUSE `indent::touched_rows` (the deduped N-cursor row union, shipped by #299) and `indent::LineEdit`.

### Out (explicitly deferred)
- **Smart re-indent on move** — a moved line keeps its own indent. The reference does not re-indent, and doing
  so would need language knowledge we deliberately do not have here.
- Moving a line across a FOLD (#305 does not exist yet); drag-to-reorder.

## Reference (§20)
**Zed / VS Code / JetBrains** — *move line up/down* (⌥↑/⌥↓) and *copy line up/down* (⇧⌥↑/⇧⌥↓). Observed
behavior: the caret/selection rides the moved text; each disjoint block moves independently and clamps at the
buffer edges independently; the duplicate lands on the pressed side and the selection follows the **copy**.
Marley matches the observed BEHAVIOR on its own `SelectionSet` + the shipped `indent.rs` line machinery.
Clean-room §20 — observed behavior only; no Zed or VS Code source read or translated.

## THE CONSTRAINT THAT SHAPES EVERYTHING — the cursors CANNOT be rebased
`indent::rebase_selections` is how Tab and ⌘/ carry their cursors. **It is wrong here, and reaching for it out
of habit would silently destroy every cursor.** `rebase_through` does this:

```rust
if p < at + remove {
    return CharOffset::from((at as isize + delta).max(0) as usize);   // CLAMPS to the edit's start
}
```

A position **inside a removed span** clamps to that edit's start. A move's removed span **is the row the cursor
sits on** — so every cursor would collapse to a line start instead of riding its row. The seam must therefore
**RETURN** the carried cursors. (The forge ticket's own signature already says so: `-> (Vec<Edit>,
Vec<Selection>)`.)

**What makes this tractable** (derived at plan time): **a move PRESERVES its span's character count** — it is a
permutation of the same lines. So offsets outside the span are untouched, every cursor lies inside some block
(by definition of `touched_rows`), and every block shifts by `dir`. Therefore **each cursor's row shifts by
`dir` (or 0, if its block clamped) and its COLUMN is unchanged.** A duplicate *does* change the length, so its
shift is a prefix-sum of block lengths — Phase 2 pins the exact arithmetic for both directions.

## PROVEN AT PLAN TIME (ran it — these are results, not intentions)

### 1. Two blocks can NEVER collide → **no collision handling is needed**
Blocks are **maximal** contiguous runs, so any two are separated by ≥ 1 untouched gap row. A move rewrites
exactly `[s, e+1]` (down) or `[s-1, e]` (up) — it consumes precisely **one** gap row on the moving side. The
next block starts at `s' ≥ e+2 > e+1`, so the rewritten spans are disjoint. Verified **exhaustively over every
touched-row subset of a 7-row buffer, in both directions: zero collisions.** Each block emits one independent
edit, ascending, applied back-to-front — exactly the shipped `LineEdit` idiom.

### 2. The trailing-newline edge DISSOLVES if the line model is right
Ropey gives a `\n`-terminated file a **phantom empty last row** that `len_lines()` counts:
```
"a\nb\nc\n"  →  4 rows: ["a","b","c",""]      ← the phantom
"a\nb\nc"    →  3 rows: ["a","b","c"]         ← none
```
Operate on the **line list including the phantom**, reorder, and re-join — and the trailing newline takes care
of itself. **Only the BOUND needs the phantom rule**, and that rule is provable, not heuristic: an empty last
row can *only* exist if the file ends with `\n`, so `n > 1 && line_text(n-1).is_empty()` ⇒ the last **real**
row is `n-2`. Verified on all four shapes, every row, both directions — every round trip byte-identical:

| fixture | rows | last real | result |
|---|---|---|---|
| `"a\nb\nc\n"` | `["a","b","c",""]` | 2 | ⌥↓ on row 2 → **NO-OP** (does not eat the trailing `\n`) |
| `"a\nb\nc"` | `["a","b","c"]` | 2 | ⌥↓ on row 1 → `"a\nc\nb"` — **does not invent a `\n`** |
| `"a\n\n"` | `["a","",""]` | 1 | row 1 **IS** movable — the rule correctly keeps a genuine trailing blank line |
| `"a"` | `["a"]` | 0 | both directions no-op |

**`touched_rows` CAN RETURN THE PHANTOM** — a caret at end-of-file sits on it (`line_col(6)` in `"a\nb\nc\n"`
→ row 3). The op must drop it, or it will try to "move" a row that does not exist.

## Locked-In Decisions
- **D1 — the cursors are CARRIED, never rebased.** See the constraint above. `rebase_selections` would collapse
  them all to line starts.
- **D2 — each maximal contiguous block moves INDEPENDENTLY and clamps INDEPENDENTLY.** Two carets on adjacent
  rows form ONE block (they are contiguous) and move together; a block already at the edge no-ops while the
  others still move. That is the reference behavior, and (per the proof above) it needs no collision handling.
- **D3 — the phantom last row is not a line.** `⌥↓` on the last *real* row is a no-op; a cursor on the phantom
  contributes no movable row.
- **D4 — the duplicate lands on the PRESSED side, and the cursors follow the COPY.** ⇧⌥↓ → the copy goes
  BELOW, and the cursors move down onto it. ⇧⌥↑ → the copy goes ABOVE, occupying the block's original rows, so
  the cursors **stay where they are** (they are now on the copy). Phase 2 states the exact row arithmetic for
  the N-block case.
- **D5 — a moved line keeps its own indent.** No re-indent (out of scope, and the reference agrees).
- **D6 — the #299 UNDO CONTRACT.** Use `Buffer::begin_undo_group` (whose default is `cursor_anchored: false`);
  apply raw `buffer.edit()` calls **back-to-front**; **never nest `edit_at_selections`** inside the group (it
  self-brackets, and `begin_group` OVERWRITES an open group, silently discarding its records). Move/duplicate
  records are LINE-anchored, exactly like ⌘/'s — and on #299 a line-anchored group absorbed the next typed
  character and **⌘Z deleted the user's code**.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌥↓ is pressed, the system shall move the caret's row DOWN one, carrying the caret to the same COLUMN of the new row. | Pure unit + **live drive**. |
| REQ-002 | WHEN ⌥↑ is pressed, the system shall move the row UP one; ⌥↓ then ⌥↑ shall return the buffer BYTE-IDENTICAL. | Pure round-trip unit + **live drive**. |
| REQ-003 | WHEN a SELECTION spans rows, the system shall move the whole block and carry the selection with it. | Pure unit. |
| REQ-004 | ⌥↑ at row 0, and ⌥↓ at the last **real** row, shall be NO-OPs — never eating nor inventing a trailing `\n`. | Pure unit on BOTH a `\n`-terminated and a non-terminated fixture, **plus** a genuine trailing blank line (`"a\n\n"`). |
| REQ-005 | DISCONTIGUOUS cursors shall move as INDEPENDENT contiguous blocks, each clamping at the edges independently; two cursors on ADJACENT rows shall form ONE block and move together. | Pure unit (rows 1/5/9 → three blocks; rows 1/2 → one block; a block at row 0 no-ops while the others move). |
| REQ-006 | ⇧⌥↓ / ⇧⌥↑ shall DUPLICATE the block on the pressed side, with the cursors following the **COPY**. | Pure unit for BOTH directions + **live drive**. |
| REQ-007 | Each press shall be ONE undo unit, and the group shall NOT absorb a following keystroke: N cursors → move → type a char → ONE ⌘Z shall revert ONLY the char. | Behavior test (**mutation cannot see this** — `cargo-mutants` does not mutate call arguments) + **live drive**. |
| REQ-008 | A caret at END-OF-FILE (on the phantom row) shall not corrupt the buffer. | Pure unit. |
| REQ-009 | The new pure seams shall be at 100% line coverage and MSI 100. | `scripts/gates.sh --diff`. |

## Grounding (verified in-tree, not guessed)
- **The keymap is clear.** ⌥↑/⌥↓/⇧⌥↑/⇧⌥↓ have no row today, and the keymap check (app.rs:6564) runs **before**
  the editor's arrow handling (app.rs:6784, `Key::Up` → `move_all_vertical`) — so binding them intercepts
  cleanly. Arrows are already named `"up"`/`"down"` (#131's ⌘↑/↓, #297's ⌘⌥↑/↓). FOUR new Editor-scoped rows →
  the roster guard goes **51 → 55 rows, 10 → 14 scoped** (two assertions).
- `indent::touched_rows` (#299) is the deduped, ascending N-cursor row union. `indent::LineEdit = (CharOffset,
  usize, String)` — ascending, applied BACK-TO-FRONT.

## Phase Plan
- **P2 Design** — the two seams' signatures and **how the cursors are carried** (the central decision — D1 rules
  out the habitual path); `contiguous_runs`; the exact row arithmetic for duplicate in the N-block case (D4);
  the keymap rows + roster updates; the test plan.
- **P3 Implement** — the pure seams, then the shim.
- **P3.5 Inspect** — adversarial critics. **Spawn them and WAIT.** Four critics have found real HIGH bugs on
  EACH of the last four tickets — on #299 one that **deleted the user's code** behind 1100 green tests. Lenses:
  the phantom row and the trailing newline (can any sequence eat or invent one?); the cursor carry (does every
  cursor land on the same TEXT it was on — for move AND duplicate, up AND down?); the multi-block arithmetic
  (three blocks, one clamped — do the others still land right?); the #299 undo contract (move → type → ⌘Z);
  and the keymap interception (does ⌥↓ still reach the editor's arrow motion when unbound elsewhere?).
  **Snapshot the tree first; each critic gets its OWN scratch file, never `src/`.**
- **P4 Validate** — units + the TRACED `cargo mutants --list` kill set (never guessed operators) + the gate;
  then the **LIVE DRIVE**. Note REQ-007 is invisible to mutation — pin it with a behavior test.
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #300.
