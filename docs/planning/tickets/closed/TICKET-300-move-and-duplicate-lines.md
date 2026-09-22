# TICKET-300 — ⌥↑/⌥↓ move lines · ⇧⌥↑/⇧⌥↓ duplicate lines (multi-cursor aware)

- **Forge ticket:** #300 `ed4f3307-e982-4c03-a256-dfd2570186b6` (feature, M19)
- **Owner:** autonomous /goal run (sprint #32 — M19 Editor power tools)
- **AAR:** `422cffde-0ca1-48fa-a671-2def254a4441`
- **Pipeline doc:** ../../pipeline/active/300-move-dup-lines.spec.md (promoted 2026-07-17; pipeline_id 2294c06a-a90a-49ae-a6cf-3b5b2110f3f5)
- **Depends on:** #296 (`135b439`) · #297 (`edb62d4`) · #298 (`bc27c70`) · #299 (`9b43c04`) — all SHIPPED + PUSHED
- **Status:** closed (SHIPPED — gate GREEN [diff], local commit; the FIRST of /work 300,302,303,304,305,314,315,316,317,259)

## Summary
The universal reordering ops, absent today — moving code currently means cut-and-paste. **⌥↑/⌥↓** move the rows
the cursors touch; **⇧⌥↑/⇧⌥↓** duplicate them. Multi-cursor aware (each disjoint block moves independently), one
undo unit per press, and the cursors ride their text.

Reuses #299's `indent::touched_rows` (the deduped N-cursor row union) and `indent::LineEdit`. The genuinely new
parts are the block grouping, the reorder edits, and — the important one — **carrying the cursors**.

## Acceptance
Caret on a line → ⌥↓ → the line moves down and the caret rides it → ⌥↑ → byte-identical → ⇧⌥↓ → duplicated.
Then three cursors → ⌥↓ → all three rows move together → type a char → **one ⌘Z reverts only the char**.
**Proven on live pixels.** Full EARS in the spec.

## Three things surfaced at plan time
- **The cursors CANNOT be rebased.** `indent::rebase_selections` — the seam Tab and ⌘/ use — CLAMPS a position
  inside a removed span to that edit's start, and a move's removed span **is the row the cursor sits on**.
  Reaching for it out of habit would collapse every cursor to a line start. The seam must **return** the carried
  cursors.
- **Two blocks can never collide — proven exhaustively**, so no collision handling is needed. Maximal contiguous
  runs are separated by ≥1 gap row, and a move consumes exactly one gap row on the moving side.
- **The "nasty" trailing-newline edge dissolves** if you operate on the line list *including* ropey's phantom
  empty last row and re-join. Only the BOUND needs the phantom rule — and that rule is provable: an empty last
  row can only exist if the file ends with `\n`.
