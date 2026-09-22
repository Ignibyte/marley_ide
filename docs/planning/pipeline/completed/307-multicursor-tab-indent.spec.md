---
pipeline_id: 8a3145b2-2832-4bf2-af22-53e6bd2e3a9c
ticket: forge#307 (c8a3a909-6ad3-4360-bfb9-ee225ccf9a2a) · local docs/planning/tickets/open/TICKET-307-multicursor-tab-indent.md
aar_id: 7cb51412-b1db-4a6a-b593-a6cfddd3588c
status: Phase 5 — Complete PASS
title: Multi-cursor Tab / ⇧Tab — both branches of the indent arm learn about the whole set
type: bug
milestone: M19
references: [the Tab dispatch arm's TWO branches (app.rs:14195-14260 — block: line_span(PRIMARY) → indent_edits/dedent_edits over a contiguous (first,last); bare-caret: ONE pad at the PRIMARY via spaces_to_next_tab_stop over the #250 layout col), has_selection = active_selection() = PRIMARY-range-only (editor_surface.rs:262-265), touched_rows — the union seam, tested for dedupe/multi-row/col-0-carve (indent.rs:66-80 + its 4 tests), indent_edits skips .is_empty() lines (indent.rs:84-92), dedent_edits strips ≤1 stop or 1 tab (indent.rs:95-113), the pair's ONLY production caller is the Tab arm (grep-verified), rebase_selections carries every member (the c11 tests), the D6 raw-edit/self-bracketing undo contract (the arm's own comment), the #282 empty-group drop semantics, the comment-toggle pipeline as the shipped shape to mirror (app.rs:7199-7225)]
---

## Title
With three cursors on three blocks, Tab indents only the block under the PRIMARY. #297's inspect
surfaced it, #299's design scoped it out and built the exact seam it needs (`touched_rows` — the
deduped ascending union of every member's `line_span`), and the arm's own comment names this ticket.
The recon adds a finding the ticket missed: **its own verify step (⌘⌥↓ ×2 → Tab → all three rows
indent) exercises the BARE-CARET branch, which its "The work" section never touches** — ⌘⌥↓ produces
carets, not ranges, so a rows-only fix would fail the ticket's own drive. Both branches of the arm
learn about the whole set: ranged/⇧Tab → the touched-rows union; all-bare-carets Tab → a pad at EVERY
caret. One undo unit either way.

## Scope
### In
- **Widen the edit builders in place** — `indent_edits` / `dedent_edits` go from a contiguous
  `(first, last)` to `rows: &[usize]` (ascending, deduped — exactly what `touched_rows` emits; the
  precondition is documented and the ONLY production caller is the Tab arm, grep-verified). The edits
  stay ascending in `at`, so `rebase_through`'s contract holds untouched. Tests t4/t5 rewrite to
  row-lists; new discontiguous cases land beside them.
- **The block branch goes whole-set** — the branch predicate becomes "ANY member has a range, or ⇧"
  (today's `has_selection` reads only the PRIMARY's range — editor_surface.rs:262-265 — so a mixed
  set with a caret primary takes the WRONG branch today; a second latent bug the predicate change
  fixes). The branch feeds `touched_rows(buffer, &before)`: caret members contribute their line,
  range members their span, two cursors on ONE row indent it ONCE (the dedupe), and the existing
  begin/end_undo_group + back-to-front apply + `rebase_selections` machinery is byte-identical —
  it already carries every member (the #297 fix).
- **The bare-caret branch goes per-caret** — ALL members carets: each caret gets its own pad,
  `spaces_to_next_tab_stop` over ITS OWN #250 layout column, expressed as an ascending per-caret edit
  list riding the SAME rebase the block branch uses (the branch already builds a one-element list for
  exactly this reason — the shape generalizes). >1 edit brackets one undo group so a single ⌘Z
  reverts every pad; the single-caret case stays byte-identical (see the P2 confirm on grouping).
- **⇧Tab with bare carets** — already a line op (dedent has no caret-pad analog); it joins the block
  branch's touched-rows union, so N carets dedent N lines (today: the primary's line only).
- **The empty-line rule stays** — `indent_edits` skips `.is_empty()` lines, deliberately LOOSER than
  comment-toggle's `.trim().is_empty()` blank rule (the ticket's own item-4 rationale: an extra
  invisible space is harmless, a dangling `//` is not). Locked, not re-derived.
### Out (explicitly)
- Any change to `comment_edits` / the #299 rules; elastic tab stops; routing through
  `edit_at_selections_with` (the D6 contract in the arm's own comment forbids it — the engine
  self-brackets, and `begin_group` inside an open group silently discards records); tab-width
  settings work; the terminal's Tab (Editor-context routing untouched).

## Reference (§20)
VS Code / Zed = OBSERVED: any non-empty selection (or ⇧) → indent/dedent the LINES of every cursor;
all bare carets → insert at every caret; either way one undo step reverts the lot. Marley matches
that behavior over its own shipped seams. One documented divergence: same-row twin carets get pads
sized in the PRE-edit space (the single-pass rebase model), where the references apply sequentially.
Each cursor still lands directly after its OWN padding, but the later one may not finish on a
post-edit tab stop (`"ab\n"`, carets at 1 and 2, tab_width 4 → `"a   b  \n"`, cursors at 4 and 7,
stops at 4 and 8). Carets on DIFFERENT rows — the common case — never interact and always land on a
stop. Recorded so it is a decision and not an accident.

### Prior art
1. **OUR OWN CODE (the whole shape):** the ⌘/ comment-toggle pipeline (app.rs:7199-7225) is the
   shipped twin — `touched_rows` → pure edit builder → begin_group → back-to-front raw edits →
   `rebase_selections` → end_group. The block branch becomes its mirror; the widened builders slot
   into the identical call pattern. `touched_rows` already carries the col-0 carve and dedupe tests.
2. **Behavior maps / observed** — the VS Code/Zed rules above (behavior only).
3. Checked gpui/ropey — no owner (a pure line-edit computation; ropey is already under `Buffer`).

## Locked-In Decisions
- **D1-ROWS-IN-PLACE** — widen the two builders' signatures (sole caller); no range-form wrappers
  left behind (dead API is a liability the mutation gate then pays for). **AMENDED at Phase 1 (F5):
  both builders ENFORCE the ascending+deduped postcondition internally** (`sort_unstable` + `dedup`),
  matching their shipped sibling `comment_edits` — whose own comment names #307 as the second caller
  and spells out the failure mode (back-to-front application corrupts on mis-order; a duplicate row
  double-indents). Two `pub` fns in one crate with the same `rows: &[usize]` shape must not disagree
  on whether the precondition is trusted or enforced.
- **D2-ANY-RANGE-GOES-BLOCK** — the branch predicate is any-member-has-a-range (or ⇧); all-carets
  Tab is the only pad path. Fixes the mixed-set misroute as a rider, with its own test.
- **D3-PER-CARET-PAD** — each caret's pad from its own display column; ascending edit list; one
  group when >1 edit. Same-row pre-edit sizing divergence documented (see Reference).
  **MEASURED, not stylistic — via the mechanism corrected at inspect (F9):** a following keystroke at
  ONE cursor opens no group, so it reaches `UndoHistory::record`, whose coalesce guard is
  `group.sel_before.is_none() && records.len() == 1`. Grouping the pad sets `sel_before: Some(..)` and
  blocks that coalesce, splitting one ⌘Z into two. (Phase 1 originally cited
  `coalesces_into`/`cursor_anchored`; that governs GROUP-into-GROUP absorption and never fires here —
  right conclusion, wrong function.) N==1 must stay ungrouped (REQ-008); `cursor_anchored` remains the
  independent reason an N≥2 group cannot be absorbed either.
- **D4-KEEP-IS-EMPTY** — indent's looser empty-line skip stays (ticket item 4, decided).
- **D5-ONE-UNDO-UNIT** — N cursors, one ⌘Z; the #282 empty-group drop semantics (a no-op ⇧Tab
  commits nothing and cannot clobber redo) must hold at N — a truth-table row, not an assumption.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | build indent edits for a DISCONTIGUOUS row list (gaps untouched, empty lines skipped, out-of-range clamped) | pure truth table on the widened `indent_edits` (cov/MSI 100) |
| REQ-002 | build dedent edits per row with the ≤1-stop/1-tab strip rules over a discontiguous list | pure truth table on the widened `dedent_edits` |
| REQ-003 | indent a row exactly ONCE when two cursors share it | pure (the union seam) + a headless drive |
| REQ-004 | indent all three lines on ⌘⌥↓ ×2 → Tab (bare carets at col 0) and revert ALL THREE with one ⌘Z, cursors carried | headless drive (the ticket's own verify, now on the right branch) |
| REQ-005 | dedent every touched line on ⇧Tab at N cursors (carets and ranges alike) | headless drive |
| REQ-006 | route a MIXED set (caret primary + ranged secondary) down the block path | headless (the D2 rider's pin) |
| REQ-007 | pad each caret to ITS OWN next tab stop when all members are carets at differing columns | headless: two carets, different cols, distinct pad widths asserted |
| REQ-008 | keep single-caret Tab and single-range Tab byte-identical to today (pad + rebase; span + group) | existing behavior drives stay green |
| REQ-009 | commit no undo step and preserve pending redo on a no-op ⇧Tab at N cursors (all flush-left) | headless (the #282 semantics at N) |

## Phase Plan
P2 confirm `selections()` ordering (ascending? the per-caret edit list needs it — sort defensively if
not contractual), the grouping-vs-coalescing question for the now-grouped single pad (does a raw
`edit()` pad coalesce with subsequent typing today? if yes, grouping changes ⌘Z granularity — decide
with the #338 cursor-anchored contract in hand), and re-verify the arm's line numbers by symbol.
P3 widen the builders + rewrite t4/t5 first (pure, compile-green), then the arm's two branches.
P3.5 critics on: the predicate flip's blast radius (any OTHER reader of `active_selection()` assuming
primary-only?), the per-caret list's ascending invariant under a caret-at-same-offset degenerate set,
undo-group emptiness at N, and the D6 contract (no `edit_at_selections` sneaking in). P4 the truth
tables + the five drives + `cargo mutants --list` on the ACTUAL widened fns (the syntactic-form rule
— run it, never guess) + gate `--diff`. P5 docs (editor.md's indent paragraph) + AAR; the authoring
lesson — a ticket whose VERIFY exercises a branch its WORK section never names — goes to the shelf
note. Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md); **builds
job-capped**.
