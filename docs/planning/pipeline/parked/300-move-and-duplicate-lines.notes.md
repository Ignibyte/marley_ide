# ⌥↑↓ move lines · ⇧⌥↑↓ duplicate lines — Notes

- **Forge ticket:** #300 `ed4f3307-e982-4c03-a256-dfd2570186b6`
- **AAR:** `422cffde-0ca1-48fa-a671-2def254a4441`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-300-move-and-duplicate-lines.md
- **Pipeline spec:** 300-move-and-duplicate-lines.spec.md
- **Depends on:** #296 (`135b439`) · #297 (`edb62d4`) · #298 (`bc27c70`) · #299 (`9b43c04`) — all SHIPPED + PUSHED

## Phase 1 — Plan

### The one thing that makes this ticket different from #299
⌘/ and Tab are **line-PREFIX** edits: text is inserted or stripped at the front of a row, the row stays where it
is, and `indent::rebase_selections` carries the cursors through. **A move is a REORDER.** Reaching for
`rebase_selections` out of habit would silently destroy every cursor — `rebase_through` CLAMPS a position
*inside a removed span* to that edit's start, and a move's removed span **is the row the cursor is sitting on**.
So the seam must **return** the carried cursors. Found by reading the code at pre-flight, not at the drive.

What rescues it: **a move preserves its span's character count** (it is a permutation of the same lines). So
offsets outside the span never shift, every cursor is inside some block (that is what `touched_rows` means), and
every block shifts by `dir`. Each cursor's row therefore shifts by `dir` (or 0 if its block clamped) and its
**column is unchanged**. Duplicate *does* change the length, so its shift is a prefix-sum of block lengths —
Phase 2 pins that.

### Two things PROVEN at plan time (ran them; results, not intentions)

**1. Two blocks can never collide → no collision handling.** Blocks are *maximal* contiguous runs, so any two
are separated by ≥1 gap row; a move rewrites exactly `[s, e+1]` (down) or `[s-1, e]` (up), consuming precisely
ONE gap row on the moving side; the next block starts at `s' ≥ e+2 > e+1`. Verified **exhaustively over every
touched-row subset of a 7-row buffer, both directions — zero collisions.** So each block emits one independent
edit, ascending, back-to-front. That is a real simplification, and it was worth ten minutes to know it rather
than to guess it.

**2. The trailing-newline edge dissolves if the line model is right.** Ropey gives a `\n`-terminated file a
PHANTOM empty last row that `len_lines()` counts (`"a\nb\nc\n"` → 4 rows, the last `""`). Operate on the line
list **including** the phantom, reorder, re-join — and the newline takes care of itself. Only the BOUND needs
the phantom rule, and it is provable rather than heuristic: **an empty last row can only exist if the file ends
with `\n`**, so `n > 1 && line_text(n-1).is_empty()` ⇒ the last real row is `n-2`. It correctly keeps a genuine
trailing blank line (`"a\n\n"` → rows `["a","",""]` → last real = 1, so row 1 IS movable).

Verified on all four shapes, every row, both directions — every round trip byte-identical. The forge ticket
calls this "the nasty edge to pin"; it is only nasty if you special-case the reconstruction instead of fixing
the line model.

**And the trap inside it:** `touched_rows` CAN RETURN THE PHANTOM — a caret at end-of-file sits on it. The op
must drop it, or it will try to move a row that does not exist.

### Grounding (verified, not guessed)
- **The keymap is clear.** ⌥↑/⌥↓/⇧⌥↑/⇧⌥↓ have no row today, and the keymap check (app.rs:6564) runs BEFORE the
  editor's arrow handling (app.rs:6784) — so binding them intercepts cleanly. Roster 51 → 55 rows, 10 → 14
  scoped.
- `indent::touched_rows` (#299) and `indent::LineEdit` are the reusable halves. `contiguous_runs` is the one
  genuinely new grouping step.

### The #299 contract, inherited for free
`Buffer::begin_undo_group`'s default is `cursor_anchored: false`, which is exactly what a LINE-anchored op
needs. Use it, apply raw `buffer.edit()` back-to-front, and **never nest `edit_at_selections`** inside the group
(it self-brackets, and `begin_group` overwrites an open group and discards its records). On #299 a line-anchored
group absorbed the next typed character and **⌘Z deleted the user's code**, unrecoverably.

**REQ-007 is invisible to mutation.** `cargo-mutants` does not mutate call ARGUMENTS, so a wrong
`cursor_anchored` would sail through MSI 100. Only a behavior test catches it: N cursors → move → type a char →
ONE ⌘Z must revert ONLY the char.

### Risks
- **The duplicate row arithmetic in the N-block case** is the fiddliest thing here (each block's insert shifts
  the blocks below it, and the cursors follow the COPY, which sits on a different side per direction). It is the
  most likely place for a real bug. Inspect gets a lens on it.
- **The cursor carry for `move` looks trivial** (row ± 1, same column) — which is exactly the kind of thing that
  is *almost* right. A clamped block, a cursor on the phantom, a selection whose anchor and head are on
  different rows: each is a chance to be off by one.
- `undo.rs` / `buffer.rs` are freshly in the diff again if the dispatch touches them — a green MSI only covers
  files IN THE DIFF.

## Phase 2 — Design

### THE CENTRAL DECISION — the cursors are carried by a CONSTANT CHAR DELTA per block
Phase 1 ruled out `rebase_selections`. The design offered two mechanisms — (A) the seam builds a shadow
line-start table, (B) the shim does `(row, col)` bookkeeping. **Both are worse than what the arithmetic actually
allows**, and simulating it first is what surfaced the third option:

> **A move (or a duplicate) TRANSLATES a contiguous span of text. Every cursor inside a block therefore shifts
> by the SAME constant CHAR delta.** No shadow table, no `(row, col)` round-trip, no bookkeeping in the shim —
> just offset arithmetic, entirely inside the pure crate under cov/MSI 100.

**MOVE** — verified against rebuilt buffers over every touched-row subset of three fixture shapes
(`\n`-terminated, non-terminated, containing a blank line), both directions: **0 mismatches**.
```
delta(block [s,e]) = 0                              if the block CLAMPED
                   = + (len_chars(line[e+1]) + 1)   moving DOWN   (the row it jumps over)
                   = − (len_chars(line[s-1]) + 1)   moving UP
```

**DUPLICATE** — verified the same way, **0 mismatches**. With `ins(Bj)` = the chars that block's copy inserts:
```
delta(block Bi) = Σ ins(Bj) for j ≤ i     ⇧⌥↓  (own block INCLUDED — the cursor jumps to its copy BELOW)
                = Σ ins(Bj) for j < i     ⇧⌥↑  (STRICTLY above — the copy occupies the block's ORIGINAL rows,
                                                so within its own block the cursor does not move)
```

**Why the anchor and the head are never in different blocks:** `line_span` gives the contiguous row range a
selection touches, and `touched_rows` unions them — so a selection's rows are contiguous and all present, hence
all in ONE block. Both endpoints therefore take the same delta. (A gap row between two blocks is by definition
*untouched*, so no selection can straddle it.)

**A cursor NOT in any block shifts by 0.** The only way that happens is a caret on the PHANTOM row (dropped
from the movable set) — and a move preserves total length, so it correctly stays put. This falls out; it is not
a special case.

### Architecture
`crates/editor/src/lines.rs` (**NEW**). Not `indent.rs`: that module is *whitespace* ops (Enter's indent clone,
Tab/⇧Tab, tab stops), and a reorder is not one. A separate module also keeps the new mutant/coverage surface
isolated. It imports `indent::{LineEdit, touched_rows}` — no cycle.

**§20 confirmed.** Reference = Zed / VS Code / JetBrains move-line + copy-line. Matched behaviors: the
caret/selection rides the moved text; each disjoint block moves and clamps independently; the duplicate lands on
the pressed side and the selection follows the COPY. Observed behavior only — no Zed or VS Code source read.

### File manifest
| file | change |
|---|---|
| `crates/editor/src/lines.rs` | **NEW.** `contiguous_runs`, `move_lines`, `duplicate_lines`, `last_content_row`. |
| `crates/editor/src/lib.rs` | `pub mod lines;`. |
| `crates/marley_app/src/keymap.rs` | FOUR new Editor-scoped rows (⌥↑/⌥↓/⇧⌥↑/⇧⌥↓). Roster **51→55** rows, **10→14** scoped. |
| `crates/marley_app/src/app.rs` | The four dispatch arms — ⌘/'s shape (which was Tab's). |

**`last_content_row` lives in `lines.rs`, not on `Buffer`.** Putting it on `Buffer` would drag `buffer.rs` back
into the diff (65 mutants) for one helper that only this module needs. It is a pure fn over `&Buffer`.

### The seams
```rust
/// Maximal contiguous blocks of an ASCENDING, DEDUPED row list. Two adjacent rows are ONE block.
pub fn contiguous_runs(rows: &[usize]) -> Vec<(usize, usize)>

/// The LAST row that is a real line. A `\n`-terminated file has a PHANTOM empty last row that `len_lines()`
/// counts; it is not a line you can move. PROVABLE, not heuristic: an empty last row can ONLY exist if the
/// file ends with `\n`.
pub fn last_content_row(buffer: &Buffer) -> usize      // n > 1 && line_text(n-1).is_empty() ? n-2 : n-1

/// ⌥↑/⌥↓ · ⇧⌥↑/⇧⌥↓ — the edits AND the carried cursors (post-edit offsets).
pub fn move_lines(buffer: &Buffer, set: &SelectionSet, dir: VDir) -> (Vec<LineEdit>, SelectionSet)
pub fn duplicate_lines(buffer: &Buffer, set: &SelectionSet, dir: VDir) -> (Vec<LineEdit>, SelectionSet)
```
Taking the `set` (not a row list) lets the seam do the whole job — `touched_rows` → drop the phantom →
`contiguous_runs` → edits + the per-block delta → the carried `SelectionSet`. The shim stays a shim.
Reuses the shipped `VDir` (#297).

**The edit for one block** — built on the LINE LIST INCLUDING THE PHANTOM, then re-joined, which is what makes
the trailing newline correct for free (Phase 1):
- MOVE down `[s,e]`: replace the char span of rows `s..=e+1` with `line[e+1] ⧺ block`, re-joined.
- MOVE up `[s,e]`: replace rows `s-1..=e` with `block ⧺ line[s-1]`.
- DUP down: insert a copy of the block after row `e`. DUP up: insert it before row `s`.
Emitted ASCENDING in `at`; the caller applies them BACK-TO-FRONT — and **the blocks provably never collide**
(Phase 1), so no ordering hazard exists between them.

### Edge clamping
- **MOVE**: ⌥↑ with a block at row 0 → that block emits NO edit and its cursors take delta 0. ⌥↓ with a block
  ending at `last_content_row` → likewise. **The OTHER blocks still move** (reference behavior, and the
  no-collision proof means it is safe).
- **DUPLICATE NEVER CLAMPS.** ⇧⌥↓ on the last row just extends the file; ⇧⌥↑ on row 0 just prepends. There is
  no edge to guard — **and adding a guard anyway would be dead code**, which #298 taught us costs a coverage
  hole (its unreachable `matches.is_empty()` guard was the single uncovered line and had to be deleted).
- **Every block clamped ⇒ NO edits.** `end_group` DROPS an empty group: no undo step, no redo clobber, no
  version bump — a no-op ⌥↑ at the top of the file leaves the buffer CLEAN.

### The dispatch arm — ⌘/'s shape, which was Tab's
Read the set → call the seam → **ONE `begin_undo_group`** around raw `buffer.edit()` calls applied
**BACK-TO-FRONT** → `set_selection(carried)`. **Never `edit_at_selections`** (it self-brackets, and
`begin_group` OVERWRITES an open group and silently discards its records — #296 D6).

**`cursor_anchored` is `false` by default** (`begin_undo_group`), which is exactly right: these records are
LINE-anchored. On #299 a line-anchored group absorbed the next typed char and **⌘Z deleted the user's code**.
We inherit the fix for free — *and it is invisible to mutation* (cargo-mutants does not mutate call arguments),
so **REQ-007 must be a behavior test**.

**`follow_editor_caret()` — YES, unlike ⌘/.** A move CHANGES the cursor's row, so holding ⌥↓ walks a block down
the file and the viewport must keep up. (⌘/ deliberately did not follow, because it never changes a row.)

### Regression Test Plan

| # | Test | Where | Proves |
|---|---|---|---|
| T1 | ⌥↓ / ⌥↑ single caret: the row moves, the caret rides it at the **same COLUMN**; assert the exact text AND the caret offset | `lines.rs` | REQ-001/002 |
| T2 | **Round trip byte-identical** on THREE fixtures: `\n`-terminated, NON-terminated, and one with a genuine trailing blank line | `lines.rs` | REQ-002/004 |
| T3 | ⌥↑ at row 0 and ⌥↓ at `last_content_row` → **NO-OP**: assert the buffer string is UNCHANGED and no edits are emitted (on both a `\n`-terminated and a non-terminated fixture — the phantom must not be swapped into) | `lines.rs` | REQ-004 |
| T4 | A multi-row SELECTION moves as a block, with **anchor AND head** both carried | `lines.rs` | REQ-003 |
| T5 | DISCONTIGUOUS blocks: carets on rows 1/5/9 → THREE blocks, each moves; **one block at the edge clamps while the others still move** | `lines.rs` | REQ-005 |
| T6 | Two carets on ADJACENT rows → ONE block, moving together | `lines.rs` | REQ-005 |
| T7 | `contiguous_runs` directly: `[1,5,9]`→3, `[1,2]`→1, `[1,2,3,7]`→2, `[]`→0, single | `lines.rs` | REQ-005 |
| T8 | DUPLICATE both directions: the copy lands on the pressed side and **the cursors are on the COPY** — assert their offsets, not just the text | `lines.rs` | REQ-006 |
| T9 | Duplicate at BOTH edges (row 0 up, last row down) — never a no-op, never corrupts the ending | `lines.rs` | REQ-006 |
| T10 | A caret at END-OF-FILE (on the phantom row) — the buffer is not corrupted and the caret stays put | `lines.rs` | REQ-008 |
| T11 | `last_content_row` on all four shapes (`"a\nb\nc\n"`→2, `"a\nb\nc"`→2, `"a\n\n"`→1, `"a"`→0) | `lines.rs` | the phantom rule |
| T12 | **THE #299 CONTRACT**: N cursors → move → type a char → **ONE ⌘Z reverts ONLY the char**; a second reverts the move. **Mutation CANNOT see this** — `cargo-mutants` does not mutate call ARGUMENTS, so a wrong `cursor_anchored` sails through MSI 100. | `buffer.rs`-style behavior test | REQ-007 |
| T13 | Keymap resolution: the four chords → their actions on Editor, **None** elsewhere. Roster 55 rows / 14 scoped. | `keymap.rs` | shim |
| T14 | Headless: ⌥↓ on a real file moves the row; ONE ⌘Z reverts it. Read → `reap_sessions` → THEN assert (a panic before reap HANGS on the PTY drop chain). | `headless_drive.rs` | end-to-end |
| T15 | The TRACED `cargo mutants --list` kill set (never guessed operators) | gate | REQ-009 |
| — | **LIVE DRIVE**: ⌥↓ walks a line down · ⌥↑ back byte-identical · ⇧⌥↓ duplicates · three cursors → ⌥↓ → all three move → type → **ONE ⌘Z reverts only the char**. Never put `focus` between a gesture and its assertion. | drive | all |

Uncoverable: none. Every seam is pure; the shim is covered headlessly and on pixels.

### Risks
- **The duplicate prefix-sum is the fiddliest thing here** — but it is now *verified*, not derived: brute-forced
  at both the row level and the char level, over every touched-row subset of a 7-row buffer, both directions,
  three fixture shapes. Inspect still gets a lens on it.
- **The move carry looks trivial** (row ± 1, same column) — which is exactly the kind of thing that is *almost*
  right. The clamped-block case, the phantom-row cursor, and a selection whose anchor and head sit on different
  rows are each a chance to be off by one. All three are pinned above.
- `lines.rs` is new, so its whole surface is in the diff. `keymap.rs`/`app.rs` return to the diff too.
