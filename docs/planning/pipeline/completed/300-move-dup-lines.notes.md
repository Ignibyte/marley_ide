# Move/duplicate lines (#300) — Notes

- **Forge ticket:** #300 ed4f3307-e982-4c03-a256-dfd2570186b6
- **AAR:** 422cffde-0ca1-48fa-a671-2def254a4441 (REUSED — opened by the sprint-#32 prior plan pass, not re-opened)
- **Local ticket doc:** ../../tickets/open/TICKET-300-move-and-duplicate-lines.md
- **Pipeline spec:** 300-move-dup-lines.spec.md · pipeline_id 2294c06a-a90a-49ae-a6cf-3b5b2110f3f5

<!-- Working scratch. Each phase appends its entry. Excluded from gate:14 doc-todos. -->

## Phase 1 — Plan

Promoted `queued/300-move-dup-lines.spec.md` → `active/` (the FIRST of the goal /work
300,302,303,304,305,314,315,316,317,259). `active/` was empty. The spec was pre-authored by ME on Fable 5
with a live-code recon in hand; this pass re-verifies every cited seam on Opus against the current tree
(`main` @ `516d567`) — attacking my own confident sentences harder because a weaker model wrote them.

### THE VERIFICATION LEDGER — every cited seam CONFIRMED (no authorship error this run; several stronger than stated)

| Claim | Verdict |
|---|---|
| `rebase_through` CLAMPS a position inside a removed span to that edit's shifted start (indent.rs:123) | **VERIFIED** — `if p < at + remove { return CharOffset::from((at + delta).max(0)) }`. A move's removed span IS the moving row, so a carried cursor WOULD collapse → **D-CARRY-NOT-REBASE holds, load-bearing.** (Spec cited :131; actual :123-129 — minor line drift, not a defect.) |
| The #299 apply idiom at app.rs:6676 (before → touched_rows → edits → begin_undo_group → raw edit .rev() → end_undo_group → set_selection) | **VERIFIED EXACT** at app.rs:6676-6706, incl. the empty-edits-drops-the-group behavior. **One divergence to pin (F1 below).** |
| `LineEdit = (CharOffset, usize, String)` ascending (indent.rs:14); `touched_rows(buffer, set) -> Vec<usize>` (indent.rs:66) | **VERIFIED** — both signatures exact. |
| `begin_undo_group` overwrites/leaks — N edits need raw `edit()` bracketed, not a self-bracketing `edit_at_selections` inside | **VERIFIED + STRONGER (F2):** the group is **NOT cursor-anchored** (buffer.rs:677 doc) → it *cannot* absorb a following keystroke **by construction**. The #338 redo-corruption class is IMPOSSIBLE for a line op, not just avoided. |
| ropey phantom last row: `"a\nb\n".len_lines()==3`, `line_text(2)==""`; empty last row iff trailing `\n` | **VERIFIED** — len_lines = "breaks + 1" (buffer.rs:86); the test comment at :1167 confirms "a trailing `\n` adds a final empty line the render draws". The F3 bound rests on solid ground. |
| ⌥↑/⌥↓/⇧⌥↑/⇧⌥↓ = `chord(false,false,true,false/true,"up"/"down")` are FREE | **VERIFIED** — grep for those four exact chords → **0 hits**. The existing alt-arrow rows are `(true,false,true,false,…)` = **⌘⌥↑/↓** (add-cursor #297 Editor + focus global) — a DIFFERENT chord. This is exactly VS Code's layout (⌥↑ move-line sits beside ⌘⌥↑ add-cursor); no collision, intentional adjacency. |
| `move_lines`/`duplicate_lines` are NEW pure seams | **VERIFIED** — grep → none exist. |
| Roster 67, scoped 22 (spec's 67→71, 22→26 math) | **VERIFIED** — keymap.rs:1060 (`chords.len()==67`) + the scoped-22 assert. Adding 4 Editor-scoped rows → 71 / 26. |

### Findings (refinements, not defects — the spec stands)

**F1 [the ONE line #300 diverges from #299].** The #299 idiom's penultimate line is
`after = rebase_selections(&before, &edits)`. #300 must NOT call that (it would clamp-collapse the carried
cursors — the whole point of D-CARRY-NOT-REBASE). #300 substitutes the `SelectionSet` its pure
`move_lines`/`duplicate_lines` seam RETURNS. Same idiom, one substituted line. **Design must state this
explicitly so implement doesn't reach for `rebase_selections` out of habit** (the toggle-comment code is
right there as the template, and it calls rebase — the trap is copying it whole).

**F2 [the undo-group is safe BY CONSTRUCTION, strengthening D-ONE-UNDO-UNIT].** `begin_undo_group(before,
false)` opens a non-cursor-anchored group (buffer.rs:683 passes `false`). The buffer doc proves such a
group can never leak into the next keystroke. So the spec's "the group never leaks" worry is not a runtime
property to test carefully — it is a type-level guarantee of using `begin_undo_group` (not the
`edit_ranges_restoring` cursor-anchored path). REQ-007's one-⌘Z-then-own-⌘Z row still tested, but it can
only pass.

**F3 [the carry math is CHAR arithmetic — the #336/#339 unit trap, confirmed live].** Moving a block ±1 row
shifts every carried cursor by ± the swapped gap-line's length **in CHARS (incl. its `\n`)**, not bytes. On
a multibyte gap line a byte-based shift lands the cursor wrong; on ASCII the two coincide (the coincidence
that hid #336). The multibyte-gap row is therefore load-bearing (Design's test plan must include it — the
spec's P2 note already flags "char-deltas, not byte guesses"; this pins it as a REQ-worthy test, folded
under REQ-001/002).

### Confirmations
- **§20 + `### Prior art` HOLD** — VS Code/Zed/JetBrains = OBSERVED (the move/dup semantics, copy-on-pressed-
  side, block coalescing, one-undo-step); the seams + carry math are Marley-original over shipped buffer
  primitives; **NO new deps** (ropey owns the line model; the #299 idiom is the transaction shape) — re-
  confirmed against Cargo.toml (no add needed).
- **The EARS AC (REQ-001..009) STAND** — no finding reopens any; F1/F2/F3 sharpen the DESIGN, not the AC.
- **Locked decisions (D-CARRY-NOT-REBASE, D-MAX-CONTIGUOUS-BLOCKS, D-PHANTOM-ROW-INCLUSIVE, D-ONE-UNDO-UNIT,
  D-EDGE-NOOP, D-COPY-ON-PRESSED-SIDE) all HOLD.** D-MAX-CONTIGUOUS-BLOCKS' no-collision proof is asserted
  by the prior plan pass — Design re-verifies it (a table), does not re-prove from scratch.

### Standing context
Auto-approved, autonomous-through-commit (the goal is the directive). **chad is AT THE MACHINE → LIVE
synthetic-input drives OFF-LIMITS** → units + headless + mechanism, deferred-pixel note (the M21/M22
posture). Push UN-OK'd (LOCAL). Batch lessons: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md)
+ [ide-mvp-shelf.md](../../design-notes/ide-mvp-shelf.md).

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

### Architecture

A NEW pure module `crates/editor/src/line_move.rs` (move/duplicate are line REORDERING — distinct from
`indent.rs`'s indentation ops, but they REUSE `indent::{LineEdit, touched_rows}` + `movement::VDir`). All
offset math stays in the pure crate (§14); the fns are TOTAL (no panics — the edge no-op is a `None`, and
ropey line APIs clamp).

- **`move_lines(buffer, set, dir: VDir) -> Option<(Vec<LineEdit>, SelectionSet)>`**
  - Coalesce `touched_rows(buffer, set)` into MAXIMAL CONTIGUOUS blocks (a helper `coalesce_blocks(rows) ->
    Vec<(usize, usize)>` — runs where `row[i+1] == row[i]+1`).
  - **D-LEADING-BLOCK-GATES (the edge semantics, pinned):** the op is ALL-OR-NOTHING. The LEADING block
    (topmost `blocks[0]` for Up, bottommost `blocks[last]` for Down) is checked against the buffer edge —
    Up: `blocks[0].start == 0`; Down: `blocks[last].end == last_content_row(buffer)`. If the leading block
    can't move → **return `None`** (a clean no-op; the app arm never touches the buffer, REQ-004 "cursors
    untouched" is literal). Otherwise NO block is at the edge (blocks are ordered; the leading one is
    closest), so ALL blocks move by 1 — the prior plan's "all blocks move" model, and the no-collision
    proof is then TRIVIAL (all blocks move the same direction by 1, separated by ≥1 gap → mutually
    disjoint after). This is more conservative than VS Code's partial-move (cursors on rows 0 and 5 + ⌥↑ →
    nothing, not "move 5"), NAMED as a deliberate v1 simplification — it makes the carry math uniform and
    the proof one line. Partial-move is a recorded future refinement.
  - **The swap edit + carry (F3 — CHARS not bytes):** each moved block swaps with its adjacent GAP row (Up:
    the row above `blocks[i].start`; Down: the row below `blocks[i].end`). One `LineEdit` per block replaces
    the `[gap ∪ block]` span with the swapped concatenation (operating on the line list INCLUDING ropey's
    phantom last row + rejoin — D-PHANTOM-ROW-INCLUSIVE, so the trailing-`\n` invent/remove is handled at
    the bound only). Each cursor in the block shifts by **± the gap row's length in CHARS incl its `\n`**
    (`line_len_with_nl(gap_row)`); cursors NOT in a moving block are unchanged. The returned `SelectionSet`
    IS the carried set — the app substitutes it for `rebase_selections` (F1).
- **`duplicate_lines(buffer, set, dir: VDir) -> (Vec<LineEdit>, SelectionSet)`** (no Option — duplicate
  never edge-no-ops):
  - Per block, ONE `LineEdit` inserting a COPY of the block's text (+ a `\n` bridge) on the pressed side.
  - **D-COPY-ON-PRESSED-SIDE:** Down (⇧⌥↓) → the copy lands BELOW, cursors move to the LOWER copy (shift
    **+ block_len_chars**); Up (⇧⌥↑) → the copy lands ABOVE, cursors stay on the UPPER copy (which occupies
    the ORIGINAL offset — the copy is inserted above and pushes the original down, so the cursor offset is
    UNCHANGED). Pinned by the REQ-006 table (both directions).
- **The app shim (F1 + F2):** ONE dispatch helper `apply_line_reorder(dir, duplicate: bool)` mirroring the
  #299 idiom EXACTLY — `before = active_selections()`; call the pure seam; `begin_undo_group(before)` → raw
  `buffer.edit()` BACK-TO-FRONT → `end_undo_group(carried)` → `set_selection(carried)` — **substituting the
  seam's returned carried set for `rebase_selections` (F1: never call rebase_selections — the clamp
  collapses the carry).** A `None` from `move_lines` → early return, no group (REQ-004). Then
  `follow_editor_caret()` (REQ-008). `begin_undo_group` is non-cursor-anchored → cannot leak (F2). Shim
  = `#[cfg_attr(test, mutants::skip)]`; the pure seams carry the mutation surface.
- **4 verbs:** `move-line-up` / `move-line-down` / `duplicate-line-up` / `duplicate-line-down`, dispatched
  via `dir = VDir::{Up,Down}` + `duplicate = starts_with("duplicate")` (the add-cursor arm template).

### §20 — CONFIRMED (N/A-adjacent: OBSERVED behavior, no source)
VS Code / Zed / JetBrains = OBSERVED (move/dup, copy-on-pressed-side, block coalescing, one undo step). The
carry math + block reorder are Marley-original over the shipped `touched_rows`/`LineEdit`/#299-idiom seams.
NO new deps (ropey owns the line model — re-confirmed). Prior art (Phase 1) HOLDS.

### File manifest
| File | Change |
|---|---|
| `crates/editor/src/line_move.rs` | **NEW** — `move_lines`, `duplicate_lines`, `coalesce_blocks`, `line_len_with_nl`/`last_content_row` helpers; imports `indent::{LineEdit, touched_rows}`, `VDir`, `Buffer`, `Selection`/`SelectionSet`. cov/MSI 100. |
| `crates/editor/src/lib.rs` | `pub mod line_move;` + `pub use line_move::{move_lines, duplicate_lines};` |
| `crates/marley_app/src/keymap.rs` | 4 Editor-scoped rows (⌥↑/⌥↓/⇧⌥↑/⇧⌥↓ → the 4 verbs); roster **67→71**, scoped **22→26**; 4 individual `.contains` asserts BEFORE the count bump (#337 discipline). |
| `crates/marley_app/src/app.rs` | the `apply_line_reorder(dir, duplicate)` helper + the 4-verb dispatch arm (+ `follow_editor_caret`); `#[cfg_attr(test, mutants::skip)]`. |
| `crates/marley_app/src/headless_drive.rs` | the drives (below). |

**Exhaustive/roster sites:** the 2 keymap roster asserts (count 71 + scoped 26). NO MarkTier-style
exhaustive enum consumer (this adds no enum variant) — the type does not force any other site.

### Regression Test Plan
| REQ | Test | Where | Kind |
|---|---|---|---|
| REQ-001 | move a single line down/up, caret rides, round-trip byte-identical | line_move | pure table |
| REQ-002 | move the whole block a multi-line selection touches, selection carried | line_move | pure |
| REQ-003 | two DISJOINT cursor blocks move independently in one press (both shift, neither collides) | line_move + headless | pure + drive |
| REQ-004 | `move_lines` returns `None` (no-op) when the leading block is at the edge — single cursor on row 0 (up) / last content row (down); AND a multi-block set whose leading block is at the edge | line_move | pure — D-LEADING-BLOCK-GATES |
| REQ-005 | the no-trailing-`\n` LAST line round-trips byte-identically through a down-then-up (no eaten/invented `\n`) | line_move | pure — the F3 phantom-row edge |
| REQ-006 | duplicate places the copy + cursors on the pressed side: Down → lower copy (+block_len); Up → upper copy (offset unchanged) | line_move | pure table (both dirs) |
| REQ-007 | one press = ONE ⌘Z; a FOLLOWING typed char is its OWN ⌘Z (the group does not leak) | headless | drive — F2 |
| REQ-008 | a held ⌥↓ keeps the primary caret in view (follow the shared primitive) | headless | drive |
| REQ-009 | two carets on ADJACENT rows merge into ONE moving block (#296 semantics) | line_move | pure |
| **F3** | **the MULTIBYTE-GAP carry**: move a block past a gap line containing a multibyte char (é/😀) → the carried cursor shifts by the gap's CHAR count, NOT its byte count | line_move | pure — the ONLY carry proof |

**What a GREEN suite would NOT prove:** on an ASCII gap line bytes==chars, so every ASCII carry row passes
whether the shift is byte- or char-based (the #336/#339 coincidence). **The multibyte-gap row (F3) is the
sole separator** — without it a byte-based carry ships green. Pinned as a required row.

### Risks
1. **The carry offset arithmetic (F3)** — named, measured in chars; the multibyte row is the mutation-
   surface separator. LOW once the F3 row exists.
2. **The no-collision proof** — under D-LEADING-BLOCK-GATES it is trivial (all-or-nothing, uniform +1);
   Validate re-verifies via the REQ-003 disjoint-blocks table, does NOT re-prove from scratch.
3. **The phantom-row bound** (`last_content_row` + the swap's `\n` invent/remove) — the REQ-005 round-trip
   is the pin; the D-PHANTOM-ROW-INCLUSIVE line-list-and-rejoin approach contains it.
4. **The edge semantics choice (D-LEADING-BLOCK-GATES)** — more conservative than VS Code's partial-move;
   deliberate v1, documented, a named future refinement. Reversible.

### Live-pixel note
The move/dup is a STATE change (buffer text + the cursor SET) — **fully headless-provable** via
`editor_text` + `editor_selection`/`editor_caret` + the undo drive. The only deferred piece is the literal
paint (chad at the machine → live drives off-limits); the state asserts + the mechanism carry it, deferred-
not-skipped (the M21/M22 posture).

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement

### Built (to the manifest)
- **`crates/editor/src/line_move.rs`** (NEW, pure): `move_lines` / `duplicate_lines` + `coalesce_blocks`,
  `last_content_row`, `span_chars`, `move_block_edit`, `shift`. Reuses `indent::{rebase_through,
  touched_rows, LineEdit}` + `VDir`. D-LEADING-BLOCK-GATES (all-or-nothing edge gate); the swap edit
  reorders at the LINE level + rejoins (D-PHANTOM-ROW-INCLUSIVE); the carry is `± (gap chars + 1)` for move
  and rebase-plus/minus-own-copy for duplicate — all CHAR arithmetic (F3).
- **`crates/editor/src/lib.rs`** — `mod line_move;` + `pub use line_move::{duplicate_lines, move_lines};`.
  (Private module + selective re-export — only the two fns are public API; the helpers stay crate-internal
  and its tests reach them directly.)
- **`crates/marley_app/src/keymap.rs`** — 4 Editor-scoped rows (⌥↑=move-up, ⌥↓=move-down, ⇧⌥↑=duplicate-up,
  ⇧⌥↓=duplicate-down); roster 67→71, scoped 22→26; the 4 chords asserted INDIVIDUALLY (both `all_chords`
  and `scoped_entries`) before the count bumps.
- **`crates/marley_app/src/app.rs`** — `apply_line_reorder(dir, duplicate)` (mutants::skip) = the #299 idiom
  with the seam's carried set substituted for `rebase_selections` (F1); a `None` from `move_lines` →
  early-return no-op; then `follow_editor_caret`. The 4-verb dispatch arm derives `dir`/`duplicate` from the
  verb string.

### The carry math — WATCHED it work (a throwaway probe, run then DELETED; `grep zzz_probe` → clean)
Applied the returned edits to a real `Buffer` and printed text + caret for 9 cases. All correct:
- **move `"a\né\nc"@0` down → `"é\na\nc"` caret@2** — the caret lands at char offset 2 (`len("é")+1`), NOT 3
  (a byte shift would give 3, since é is 2 bytes). **This is the F3 proof that the carry is CHAR-based** — on
  an ASCII gap it would pass either way; only the multibyte gap separates them.
- move down/up (no trailing `\n`) round-trip; the two EDGE cases (`a` up, `b`-as-last down) → `None` no-op;
  the phantom-row case (`"a\nb\n"@2` down → no-op, because `b` is the last CONTENT row, the trailing `\n`
  making row 2 the phantom); duplicate down (caret on the lower copy) + duplicate up (caret stays on the
  upper copy at the original offset).
The probe validated the arithmetic ahead of the Phase 4 formal tables — those tables now have known-good
expected values to assert (the #339 "read the crate, don't guess twice" discipline).

### Deviations from design
None material. Module is a private `mod` (not `pub mod`) — only the two fns need to be public; the design
said `pub mod` but the tighter surface is better (helpers stay internal). A clippy `doc_lazy_continuation`
warning (a `//!` line starting with `+ [VDir]` read as a markdown bullet) — reworded to `and [VDir]`
(gate:2 `-D warnings` would have caught it; caught here).

### Verified
`cargo check --workspace` CLEAN; `cargo clippy -p marley_editor -p marley --all-targets` CLEAN;
`cargo nextest -p marley --lib -E 'test(/chord|roster|keymap/)'` → 20 passed (incl. count 71 + scoped 26 +
`all_chords_lists_every_binding`); `cargo fmt --all`; §20 CLEAN (0 zed/warp in the diff). Test hooks
(`*_for_test`) DEFERRED to Phase 4 (the #337 F2 rule — dead `#[cfg(test)]` trips gate:2).

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

Two parallel general-purpose critics — lens 1 the pure carry math, lens 2 the app wiring + undo group +
roster. Each verified concretely (throwaway Buffer tests applying the real edits, run then DELETED, tree
re-confirmed clean). **Critic 2 (app wiring) came back FULLY CLEAN** — F1 (no `rebase_selections` call),
the undo group (back-to-front, closed by construction, non-cursor-anchored → can't leak), the 4-verb
dispatch mapping, the roster (71/26, chords free + distinct from ⌘⌥ add-cursor), mutants discipline (0
app-shim / 100 pure), and lib re-export all verified with reproductions. **Critic 1 found 2 real, reproduced
defects in the CARRIED SELECTION** (the text edits were correct in every case — the bugs were purely in the
carry, i.e. exactly the D-CARRY-NOT-REBASE math under review).

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| H1 | **HIGH** | **A "full lines" selection's carved boundary head gets the wrong carry.** The universal "N full lines selected" shape is `anchor in the block, head at col 0 of the row PAST the block`; `indent::line_span` CARVES that col-0 row out (indent.rs:45), so the head sits on row `b1+1` — EXCLUDED from the block. The carry keyed the delta on the endpoint's raw `line_col(off).0`, so the boundary head got delta 0 while the anchor got the real delta → the selection distorts. Repro: move-down a 2-line selection `"a\nb\nc\nd"@(0,4)` → text correct `"c\na\nb\nd"` but carried `(2,4)` instead of `(2,6)` — the selection SHRANK. Same class on move-up (grew) and dup-up (doubled, spanning both copies). Breaks the flagship "select lines, move/duplicate" gesture (REQ-002/006). | **REAL — confirmed** with a fresh probe (mv-dn 2ln → `(2,4)` before). Reachable via the standard Home+⇧↓ selection; it COMPOUNDS (a shrunk selection fragments the next move). | Attribute an endpoint to its block by **OFFSET SPAN** (`span_chars` = `[line_start(b0), line_start(b1+1)]`) not raw row — a new `block_at_offset(buffer, blocks, off, inclusive_end)`. **INCLUSIVE** of the boundary for MOVE (it rides the block) and DUP-UP (it stays on the upper copy); **EXCLUSIVE** for DUP-DOWN, where `rebase_through` already carries the boundary onto the copy so it must not ALSO get the hop. Blocks are ≥1 gap apart → the spans are disjoint, attribution unambiguous. Probe-confirmed AFTER: mv-dn→`(2,6)`, mv-up→`(0,4)`, dup-up→`(0,4)`, dup-dn→`(4,8)` STILL correct. |
| L2 | **LOW** | **The phantom trailing row is movable UP, dropping the file's trailing `\n`.** The down-guard uses `last_content_row`, but the up-guard is only `blocks[0].0 == 0`, so a bare caret on ropey's phantom empty last row (the normal EOF position) + ⌥↑ swaps it: `"a\n"@2` → `"\na"` (trailing `\n` gone, leading blank appears). Asymmetric with the down-guard (`"a"` correctly refuses to move down). | **REAL — confirmed** (`"a\n"@2` up → `"\na"` before). Narrow (needs the caret on the invisible trailing line) + no panic, but flips the trailing-newline (shows in diffs / "no newline at EOF"). | `touched_content_rows` clamps every touched row to `last_content_row` before coalescing — a phantom-row caret then acts on the last CONTENT line (⌥↑ moves it, ⌥↓ is the edge no-op), symmetric with the down-guard. Probe-confirmed AFTER: `"a\n"@2` up → no-op `"a\n"`; `"a\nb\n"@4` (caret on phantom) up → `"b\na\n"` (trailing `\n` preserved). |

**Both fixes verified:** `cargo clippy -p marley_editor -p marley --all-targets` CLEAN; `cargo nextest -p
marley_editor -p marley --lib` → **843 passed**; the 8-case probe watched every fixed + regression case land
correctly (H1's four, L2's two, plus the multibyte + dup single-caret regressions); §20 clean; the probe
DELETED (`grep zzz_probe` → clean). **Phase 4 owes:** the full-line-selection carry as a formal pure row
(H1 — the offset-span attribution is the mutation surface), the phantom-up no-op row (L2), plus the multi-
block-different-gaps + dup-multi-block rows the critic verified.

**Lesson (for the AAR / capture):** **carry a selection endpoint through a line-block edit by OFFSET SPAN,
not raw row.** A "full lines" selection's head is not on a line the block contains — `line_span`'s col-0
carve puts it on the row PAST the block, at the block's exclusive-end offset. A row-based block lookup
misattributes it (delta 0), silently distorting the carried selection while the TEXT stays correct — so no
text test catches it; only a selection-shape assertion on a full-line selection does. The direction-
sensitivity (inclusive vs exclusive at the boundary) mirrors whether the op's own edit already moves that
offset (`rebase_through` does for a dup-down insert; a length-preserving move swap does not).

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**16 tests added (13 pure + 3 drives); GATE GREEN [diff] (15/15, coverage 100%, mutation MSI 100%). Receipt
valid (`a3df028c…`). One coverage red on the first run, fixed at source (§0).**

### Tests written
- **`crates/editor/src/line_move.rs mod tests` — 13 pure tests** (a `go`/`caret`/`multi` helper applies the
  edits back-to-front exactly as the app arm does, then asserts BOTH text AND the carried anchor/head):
  `req001` single-line move + round-trip; **`req002` the FULL-LINE-selection carry (the inspect H1) — a
  Home+⇧↓ selection whose head sits at the carved col-0 boundary rides the block by offset-span attribution
  (`(2,6)` not `(2,4)`); a row-based carry passes the TEXT and fails only this SELECTION-shape assert**;
  `req003` disjoint blocks (equal + DIFFERENT gap lengths, each caret riding its own gap); `req004` the edge
  no-op + **the L2 phantom-up (no `\n` drop)**; `req005` the no-trailing-`\n` last-line round-trip;
  **`req006` the MULTIBYTE gap carry (é / 😀 — the only char-vs-byte separator, the #336/#339 coincidence)**
  + duplicate pressed-side + the dup-up H1 selection (`(0,4)` not `(0,8)`) + the EOF dup-down branch; the
  multi-block dup-down; `req009` adjacent-rows coalesce; the helper tables (`coalesce_blocks`,
  `last_content_row`, `block_at_offset` inclusive-vs-exclusive at the boundary); totality (no panic on `""`,
  `"a"`, `"\n"`, `"\n\n"`).
- **`crates/marley_app/src/headless_drive.rs` — 3 drives** (via `dispatch_for_test` + the shipped
  `editor_text`/`editor_caret` — NO new `*_for_test` hook needed; a shared `open_editor_file` helper returns
  the `TempDir` so the config dir outlives the body): `move_line_down_rides_caret_and_undoes_headless`
  (REQ-008 caret rode + REQ-007 one-⌘Z whole-move revert); **`move_line_undo_group_does_not_leak_headless`
  (F2 — move, type `x`, ⌘Z reverts ONLY the char → `"a\nc\nb"`, the non-cursor-anchored group can't absorb
  the next keystroke)**; `duplicate_line_down_headless` (the `starts_with("duplicate")` dispatch branch).

### The first gate's coverage red — fixed at source
`line_move.rs:157` — the `if rows.is_empty() { return None }` guard was a dead-by-design line (a
`SelectionSet` is invariantly non-empty, so `touched_content_rows` never returns empty; the same class as
#340's redundant `count < 2`). `line_move.rs` is NOT coverage-excluded, so the unreachable `return None`
failed the 100% floor. Fixed by DROPPING the guard and making the block access total via `.first()?` /
`.last()?` — no dead line, still panic-free (an empty set would return `None`, unreachable). Mutation MSI
was already 100 (the removed guard also removed a now-moot mutant). Re-run → GATE GREEN.

### Live-pixel — DEFERRED, not skipped
Move/duplicate is a STATE change (buffer text + the cursor SET) — fully headless-proven via `editor_text` +
`editor_caret` + the undo drive. The literal paint is deferred (chad at the machine, live drives off-limits);
the state asserts + the mechanism carry it (the M21/M22 posture).

### Pre-existing / not in scope
None. `docs/README.md` + `detached-sessions.md` remain pre-existing unrelated changes (excluded).

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
