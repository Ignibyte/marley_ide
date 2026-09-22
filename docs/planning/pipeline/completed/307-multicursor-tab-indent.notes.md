# Multi-cursor Tab / ⇧Tab — Notes

- **Forge ticket:** #307 `c8a3a909-6ad3-4360-bfb9-ee225ccf9a2a`
- **AAR:** `7cb51412-b1db-4a6a-b593-a6cfddd3588c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-307-multicursor-tab-indent.md
- **Pipeline spec:** 307-multicursor-tab-indent.spec.md
- **pipeline_id:** `8a3145b2-2832-4bf2-af22-53e6bd2e3a9c`

## Phase 1 — Plan

- **Request:** promote the pre-authored spec (`queued/` → `active/`), re-verify every cited seam by
  SYMBOL against live `main` @ `4cb6f9e`, record drift. FIRST of the goal
  `/work 307,321,350,345,348` (auto-approved, autonomous-through-commit).
- **Classification / tier:** work pipeline, one shippable slice. `type: bug` (it breaks a promise B5
  shipped — multi-cursor is advertised, Tab silently ignores all but the primary). Milestone M19
  (the ticket's own), executed in the M22 batch.
- **Forge recall (§18.3):** `bulletin-list` → **zero active bulletins**. `knowledge-search`
  (multi-cursor indent / undo group / rebase) returned 5 ADs + 4 prevention rules + 1 distilled
  lesson by id; the substantive ones are already carried in
  `docs/planning/design-notes/m22-editing-bar.md` (the #338 `cursor_anchored` CRITICAL, the #282
  empty-group semantics, the D6 self-bracketing contract, "run `cargo mutants --list` on the ACTUAL
  code", the uncovered-closure coverage trap) — no new constraint surfaced.

### Findings (all re-verified by symbol against live code, not by the spec's line numbers)

- **F1 — the blast radius is exactly as specced (CONFIRMED).** `indent_edits` / `dedent_edits` have
  **one** production caller between them: the editor Tab arm (app.rs:14223 / :14225). The only other
  workspace-wide hit is a doc-comment cross-reference in `comment.rs:19`. `D1-ROWS-IN-PLACE` (widen
  in place; no range-form wrappers left behind) is therefore safe — no dead API, no second caller to
  migrate.
- **F2 — the second `keystroke.key == "tab"` is out of scope (CONFIRMED not-a-site).** app.rs:14474
  is the TERMINAL cooked-prompt path completion (#89), a different handler in terminal context,
  gated on `!shift && !platform && !control`. The editor arm at app.rs:14198 is the only editing
  site. Recorded so inspect does not re-discover it as a missed call site.
- **F3 — THE COALESCING QUESTION IS ANSWERED; it decides the design's hardest open item.**
  ⚠️ **The MECHANISM recorded here was WRONG; inspect F9 corrected it. The CONCLUSION was right.**
  Originally written: "`undo::coalesces_into` requires `prev.cursor_anchored && group.cursor_anchored`,
  and `Buffer::begin_undo_group` pins that false, so grouping a lone pad would block coalescing."
  **That is the wrong path.** `coalesces_into` runs only from `end_group` — it governs GROUP-into-GROUP
  absorption. A following single keystroke never opens a group (`Buffer::edit_ranges_restoring_placing`
  takes its one-member ungrouped fast path), so the typed char reaches `UndoHistory::record`, whose
  guard is `group.sel_before.is_none() && group.records.len() == 1`; `cursor_anchored` is never
  consulted there. Grouping the pad sets `sel_before: Some(..)`, and THAT is what blocks the coalesce.
  → **Wrapping a SINGLE pad in a group would silently change ⌘Z granularity** (today: Tab-then-type is
  one undo unit; grouped: two) — same conclusion, correct reason.
  **DECISION (amends D3/D5, pins REQ-008): open an undo group ONLY when the edit list has >1
  element.** N==1 keeps the exact ungrouped path it has today, byte-identical; N>1 gets one group so
  a single ⌘Z reverts every pad. The resulting asymmetry is deliberate and documented, not an
  accident.
- **F4 — the ordering precondition holds for free (CONFIRMED).** `SelectionSet` is an *ordered* set:
  `sort_by_key(|sel| sel.start())` at construction (selection.rs:139), doc "The buffer's ordered set
  of cursors/Selections" / "The set's members, in order", pinned by
  `from_selections_merges_to_ordered_disjoint` ("UNSORTED input comes out sorted"). Mapping over
  `selections()` therefore yields an **ascending** per-caret edit list, satisfying
  `rebase_through`'s ascending precondition with no defensive sort. See F5 — we still enforce.
- **F5 — PRIOR-ART FIND THAT AMENDS A LOCKED DECISION (the sweep's payoff).** The spec's
  `D1-ROWS-IN-PLACE` said the ascending/deduped precondition would be *documented*. The shipped twin
  says otherwise: **`comment_edits(buffer, rows: &[usize], token)` (comment.rs:96) already has
  exactly the target signature AND enforces the postcondition internally** —
  `live.sort_unstable_by_key(...)` — with a rationale written *about this very ticket*:
  > "The only caller today passes `indent::touched_rows`, which is already sorted and deduped — but
  > this is a `pub` cross-crate fn and **forge #307 adds a second caller**. A mis-ordered `rows`
  > would emit edits out of order (the caller applies them BACK-TO-FRONT and would corrupt the
  > buffer); a duplicated row would prefix a line TWICE."
  That reasoning transfers verbatim to indent/dedent (same back-to-front application; a duplicated
  row would double-indent). **AMENDED DECISION: the widened `indent_edits`/`dedent_edits` ENFORCE
  sorted+deduped internally, matching their sibling.** Two `pub` fns in one crate taking `rows:
  &[usize]` where one enforces and one trusts is exactly the asymmetry that bites later. Cheap (a
  sort of a handful of rows) and it makes F4's reliance belt-and-braces rather than load-bearing.
- **F6 — the #338 corruption class cannot reach this ticket (recorded so inspect does not
  re-litigate).** That CRITICAL required a group asserting `cursor_anchored: true` which a consumer
  then trusted. Here the public wrapper pins it **false**, so `coalesces_into` short-circuits on the
  first conjunct — the per-caret pad group is structurally incapable of the same lie. (If a future
  change routes this arm through a `cursor_anchored: true` path, the analysis must be redone: a
  per-caret pad DOES end at the end of its own insert, but the block branch's line-start edits do
  NOT — the same shape that bit #299's ⌘/.)

### Prior-art sweep (§20, all three legs)

1. **Behavior maps / observed** — VS Code + Zed: any non-empty selection (or ⇧) indents/dedents the
   LINES of every cursor; all-bare-carets inserts at every caret; one undo step either way. Behavior
   only; no copyleft source read.
2. **Published material** — none needed (no protocol/spec surface).
3. **OUR PERMISSIVE DEPS (the high-yield leg):** **ropey 1.6.1** owns only single-position
   `insert(char_idx, &str)` / `remove(char_range)` (rope.rs:343/:542) — no batch, no line-start, no
   indent concept; it is already the substrate under `Buffer` and owns nothing here. **gpui 0.2.2**
   has no indent/text-edit seam (`fn indent` — no hits in `src`). **Verdict: no dep owns this
   seam.** The owner is our own `crates/editor` — and F5 above is the concrete adoption.

### Decisions confirmed / amended

- **CONFIRMED as written:** `D2-ANY-RANGE-GOES-BLOCK` (the branch predicate becomes any-member-has-a-
  range; today's `has_selection` reads only the PRIMARY — editor_surface.rs:262-265 — so a mixed set
  misroutes, the latent rider bug), `D3-PER-CARET-PAD` (each caret padded from its own display
  column), `D4-KEEP-IS-EMPTY` (indent's looser `.is_empty()` skip stays, deliberately unlike
  comment-toggle's `.trim().is_empty()`), `D5-ONE-UNDO-UNIT`.
- **AMENDED — `D1-ROWS-IN-PLACE`:** widen in place *and* **enforce** the ascending+deduped
  postcondition inside both builders (was: document it). Reason: F5, the shipped sibling's stance.
- **SHARPENED — `D3`/`D5`/REQ-008:** "one group when >1 edit" is now a *measured* requirement, not a
  stylistic choice — F3 shows grouping a lone pad changes ⌘Z granularity.
- **EARS AC:** all 9 REQs hold as written; no reopens. REQ-008's "byte-identical" is now backed by
  F3's mechanism rather than asserted.

### Discovery — the edit surface handed to Design

- `crates/editor/src/indent.rs` — `indent_edits` + `dedent_edits` signatures & bodies (+ their t4/t5
  tests rewritten to row-lists; new discontiguous cases).
- `crates/marley_app/src/app.rs` — the Tab arm (anchor: `event.keystroke.key == "tab"` @ ~14198):
  the branch predicate, the block branch's `line_span` → `touched_rows`, and the bare-caret branch's
  single pad → per-caret list + conditional group.
- No other file changes anticipated. `touched_rows`, `rebase_selections`, `rebase_through`,
  `spaces_to_next_tab_stop`, `line_layout` are all reused UNCHANGED.

**Status: Phase 1 — Plan PASS.**

## Phase 2 — Design

### Architecture / approach

The change lands entirely in the editor stack: the pure line-edit builders in `crates/editor`
(`marley_editor`) and the one gpui key-dispatch arm in `crates/marley_app/src/app.rs` that calls
them. Nothing touches the PTY/Blocks/forge layers. §14 holds throughout: the builders stay total
(no panic on any `rows` input — see D6a), no new error type is needed (the seam returns a `Vec`,
and "nothing to do" is the empty vec), and no IO is involved.

**The shape both branches converge on.** Today the arm has two structurally different tails: the
block branch builds an edit list, brackets an undo group, applies back-to-front, rebases; the
bare-caret branch does one direct `buffer.edit()` plus an ad-hoc one-element list built solely to
feed `rebase_selections`. After this change both branches produce a `Vec<LineEdit>` and share ONE
tail. That is the design's core simplification: the divergence collapses to *how the edit list is
built* and *whether a group is opened*.

```
let (edits, group) = if dedent || any_member_has_a_range {
    let rows = touched_rows(buffer, &before);          // the #299 union seam, unchanged
    (if dedent { dedent_edits(buffer, &rows, tab_width) }
     else      { indent_edits(buffer, &rows, tab_width) }, true)
} else {
    let e = /* per-caret pads, below */;
    let g = e.len() > 1;                                // D3a — see below
    (e, g)
};
if group { buffer.begin_undo_group(before.clone()); }
for (at, remove, text) in edits.iter().rev() {          // BACK-TO-FRONT (unchanged)
    let end = CharOffset::from(at.as_usize() + remove);
    buffer.edit(*at..end, text, EditOrigin::Human);
}
let after = rebase_selections(&before, &edits);
if group { buffer.end_undo_group(after.clone()); }
buffer.set_selection(after);
```

**D6 contract preserved (verified by reading the arm):** the tail keeps raw `buffer.edit()` calls
inside a group this site opens itself. It never routes through `edit_at_selections*`, which
self-brackets — and `begin_group` OVERWRITES an open group, silently discarding its records. The
arm's existing comment says exactly this; it stays true and stays put.

**D3a — a REFINEMENT of Phase-1 F3 (caught at design; F3 as written was over-general).** F3
concluded "open a group only when the edit list has >1 element". Applied globally that would be a
DEFECT: today's block branch groups *unconditionally*, including the zero-edit case, and the #282
semantics depend on that (an open-then-dropped empty group commits no undo step AND — since the
redo-clear fires only when a non-empty group commits — cannot clobber a pending redo). Making the
block branch conditional would delete that behavior. The correct, narrower rule:

| branch | grouping | why |
|---|---|---|
| block (`dedent` or any range) | **always** — unchanged | preserves the #282 empty-group drop + today's single-line behavior byte-identically |
| pad (all carets) | **iff `edits.len() > 1`** | F3: `begin_undo_group` pins `cursor_anchored: false`, so a group can never coalesce with following typing while today's ungrouped lone pad can. Grouping N==1 would silently change ⌘Z granularity (REQ-008) |

The pad branch cannot produce zero edits (the set is never empty and every caret yields a pad of
≥1 space — `spaces_to_next_tab_stop` returns a full `tab_width` at a stop, never 0), so `len() > 1`
is exactly "more than one cursor". No empty-group case arises there.

**The per-caret pad construction** (the ticket's blind spot). `before.selections()` is ascending by
construction (F4) and same-offset carets are impossible (Risk R2), so the produced list is STRICTLY
ascending in `at` — exactly `rebase_through`'s precondition, no sort needed:

```
before.selections().iter().map(|sel| {
    let caret = sel.head();
    let (row, cin) = buffer.line_col(caret);
    let col = line_layout(&buffer.line_text(row), tab_width).col_of_offset(cin);
    (caret, 0usize, " ".repeat(spaces_to_next_tab_stop(col, tab_width)))
}).collect()
```

Each caret is sized from ITS OWN display column via the #250 layout, so tabs earlier in that line
count correctly and two carets at different columns get different pads. **Documented divergence
(Reference §):** two carets on the SAME row are both sized in the PRE-edit coordinate space, where
VS Code/Zed apply sequentially — so the second caret's pad is computed as if the first had not
landed. Worked example (`"ab\n"`, carets at offsets 1 and 2, `tab_width` 4): pads of 3 and 2 →
`"a   b  \n"`, cursors at 4 and 7. Each cursor lands exactly after its OWN padding, which is the
part that matters; but the second is at column 7 with stops at 4 and 8, so it does NOT finish on a
post-edit tab stop the way a sequential editor would. Carets on DIFFERENT rows — the overwhelmingly
common multi-cursor case — never interact and always land on a true stop. Recorded as a decision,
with the honest consequence stated rather than waved off, and pinned by a test.

**Reference (§20) confirmed.** Unchanged from the spec and still accurate: VS Code/Zed are OBSERVED
behavior only (any selection or ⇧ → line indent/dedent at every cursor; all carets → insert at every
caret; one undo step). No copyleft source read. The implementation is our own seams (`touched_rows`,
`rebase_selections`, `line_layout`) plus the `comment_edits` idiom (F5). The prior-art sweep stands:
ropey 1.6.1 owns only single-position `insert`/`remove`, gpui 0.2.2 has no indent seam — **no shipped
dep owns this**; the owner is `crates/editor`.

### The widened builders — exact design

```rust
pub fn indent_edits(buffer: &Buffer, rows: &[usize], tab_width: usize) -> Vec<LineEdit>
pub fn dedent_edits(buffer: &Buffer, rows: &[usize], tab_width: usize) -> Vec<LineEdit>
```

- **D6a — enforcement, mirroring `comment_edits` (F5).** Each copies `rows` into a local `Vec`,
  `sort_unstable()` + `dedup()`, then proceeds. `comment_edits` uses `sort_unstable_by_key`/
  `dedup_by_key` because its rows are already paired with text; a bare `Vec<usize>` uses the plain
  forms — same stance, same rationale, and the doc comment says so and cross-references the sibling.
  This makes the contract *enforced* on both `pub` cross-crate fns instead of enforced on one and
  trusted on the other.
- **The clamp DISAPPEARS, and that is a simplification, not a gap.** The old
  `last.min(buffer.len_lines().saturating_sub(1))` existed because a RANGE could run past EOF. With a
  row list, totality comes from `Buffer::line_text`'s own contract: `row >= len_lines()` returns
  `String::new()` (buffer.rs:140-143). For indent, the existing `.filter(|row| !line_text.is_empty())`
  drops such a row *before* `line_start` is ever called; for dedent, an empty line yields `strip == 0`
  so the `if strip > 0` guard drops it, again before `line_start`. **`line_start` is therefore never
  called with an out-of-range row** — the thing that would have panicked cannot be reached. An
  explicit `row < len_lines()` guard would be a redundant cold branch (an uncovered arm and an
  equivalent mutant), so it is deliberately NOT added; the reliance is documented in both fns and
  pinned by a past-EOF test row.
- **Kept exactly:** indent's `.is_empty()` skip (D4 — deliberately looser than comment-toggle's
  `.trim().is_empty()`; extra invisible spaces are harmless, a dangling `//` is not) and dedent's
  "≤1 stop of leading spaces, or one leading `\t`" strip.
- **Mutant-surface note.** cargo-mutants does not mutate method calls, so `sort_unstable`/`dedup` add
  no mutants; the signature change adds none. The viable set should stay materially what it is today
  (the `.filter` negation, the arithmetic, the `strip > 0` comparison). **This is a prediction, not a
  fact** — Phase 4 runs `cargo mutants --list -f crates/editor/src/indent.rs` on the ACTUAL code and
  designs killers for whatever it really reports (the standing rule; guessing the operator set has
  bitten twice).

### File manifest

| File | Change |
|---|---|
| `crates/editor/src/indent.rs` | Widen `indent_edits` + `dedent_edits` to `rows: &[usize]`; add internal sort+dedup enforcement (D6a) with doc comments citing the `comment_edits` sibling; drop the now-unreachable range clamp and document why. Rewrite tests t4/t5 to row lists; add discontiguous/unsorted/duplicate/past-EOF/empty-list cases. |
| `crates/marley_app/src/app.rs` | The Tab arm (anchor `event.keystroke.key == "tab"`): predicate `has_selection` → any-member-has-a-range read off `before`; block branch `line_span(primary)` → `touched_rows`; bare-caret branch → per-caret pad list; the two tails merge into one shared tail with a `group` flag. |
| `crates/marley_app/src/headless_drive.rs` | New multi-cursor Tab/⇧Tab drives (REQ-003 e2e, 004–007, 009) reusing `seed_one_project`/`boot`/`set_editor_caret`/`editor_text`/`reap_sessions`. |

No other files. `touched_rows`, `rebase_selections`, `rebase_through`, `spaces_to_next_tab_stop`,
`line_layout`, `line_span`, `Buffer`, `SelectionSet` are reused UNCHANGED.

### Mutation / coverage homes (verified, not assumed)

- **`crates/editor/src/indent.rs` — PURE, fully in scope.** Coverage 100% + MSI 100% required. All
  new logic carrying a mutation surface lives here; this is where the test weight goes.
- **`crates/marley_app/src/app.rs` — coverage-EXCLUDED and mutation-SKIPPED for this arm.** Verified:
  the Tab arm (~14198) sits inside `impl Render for RootView :: fn render` (app.rs:13471), which
  carries `#[cfg_attr(test, mutants::skip)]` (app.rs:13470); app.rs is on gates.sh's documented
  ACCEPTED-UNTESTABLE coverage exclude list (gates.sh:189). **Consequence: the app-side changes carry
  NO live mutants and no coverage obligation — they are pinned by the headless drives alone.** That is
  why the plan puts a drive behind every app-side REQ.
- No new file is added, so the `mutants::skip` detach trap (inserting a fn above a skipped shim
  rebinds its attribute) is not in play; Phase 4 still re-runs `--list` to confirm.

### Regression Test Plan

| REQ | The behavior | Test | Where |
|---|---|---|---|
| REQ-001 | indent edits over a DISCONTIGUOUS row list — gaps untouched, empty lines skipped, past-EOF rows contribute nothing | `t4_indent_edits_over_row_lists` (rewrite of t4): rows `[0,2,4]` in a 5-line buffer with line 2 empty and a row past EOF → exact edit list | pure, indent.rs |
| REQ-002 | dedent edits per row over a discontiguous list — ≤1 stop of spaces or one `\t`; a flush-left row contributes nothing | `t5_dedent_edits_over_row_lists` (rewrite of t5) | pure, indent.rs |
| REQ-003 | a row appearing TWICE is indented ONCE (the D6a enforcement) | `t4b_row_lists_are_sorted_and_deduped_internally`: unsorted `[4,0,2,0]` → the same ascending list as `[0,2,4]`; asserts ascending `at` order explicitly (the back-to-front precondition) | pure, indent.rs |
| REQ-003 | …and end-to-end: two cursors on ONE row indent it once | drive: two carets on row 0 at different columns → Tab (via a range so it takes the block path) → row 0 gains exactly one pad | headless |
| REQ-004 | ⌘⌥↓ ×2 → Tab indents all three lines; one ⌘Z reverts all three with cursors carried | `tab_pads_every_caret_headless` — the ticket's own verify, on the branch it actually reaches. Asserts text after Tab, text after ⌘Z == original, and the restored selection SET | headless |
| REQ-005 | ⇧Tab dedents every touched line at N cursors (carets and ranges alike) | `shift_tab_dedents_every_cursors_line_headless` | headless |
| REQ-006 | a MIXED set (caret primary + ranged secondary) takes the BLOCK path | `mixed_set_takes_the_block_path_headless`: caret on row 0 + range on rows 2–3 → Tab indents rows 0,2,3 (today: row 0 only) | headless |
| REQ-007 | each bare caret pads to ITS OWN next tab stop (differing columns → differing pad widths) | drive with carets at differing columns → differing pads; plus the same-row twin-caret pre-edit-sizing divergence asserted explicitly as documented behavior | headless |
| REQ-008 | single-caret Tab and single-range Tab byte-identical — INCLUDING ⌘Z granularity | the shipped `editor_tab_indent_dedent_flows_headless` (#276) stays green UNCHANGED (block indent, ⇧Tab dedent, ⌘Z, and a bare-caret pad); **plus** a new granularity row: Tab → type → ONE ⌘Z reverts both (proves the lone pad stayed ungrouped and still coalesces) | headless |
| REQ-009 | a no-op ⇧Tab at N cursors commits no undo step and preserves a pending redo (#282 at N) | `noop_shift_tab_at_n_preserves_redo_headless`: edit → ⌘Z (redo pending) → ⇧Tab on all-flush-left rows → redo still restores the original edit | headless |

**Pure vs headless split:** REQ-001/002/003(unit) are pure truth tables in `indent.rs` and carry the
cov/MSI-100 obligation. REQ-003(e2e)/004/005/006/007/008/009 are headless drives — the ONLY proof for
the app-side arm, which is coverage-excluded and mutation-skipped.

**LIVE drives are OFF-LIMITS this session** (chad is at the machine; synthetic input would hit his
frontmost window). The ticket's own verify asks for a live ⌘⌥↓ ×2 → Tab → ⌘Z. REQ-004's headless
drive carries it instead: it injects the identical keystrokes through the #264 `TestAppContext` lane
against the real `RootView` and asserts the same three facts (all rows indented, one ⌘Z reverts,
cursors carried). Deferred-not-skipped; re-runnable live in ~30 s when the machine is free, no ticket.

### Risks / decisions

- **R1 — the predicate flip's blast radius is ZERO (measured).** `active_selection()` has exactly ONE
  other production reader: app.rs:7056 in `"open-editor-find"`, seeding the ⌘F query from a
  single-line selection. This design does **not** change `active_selection()` — it stops the Tab arm
  *calling* it and reads the whole set off the already-cloned `before`. That reader is untouched. (All
  other hits are tests.)
- **R2 — the degenerate twin-caret case CANNOT arise (measured).** `SelectionSet::from_selections`
  collapses same-offset carets (`members(vec![car(3), car(3)]) == vec![car(3)]` — "they ARE one
  logical cursor, and leaving them separate would make it insert its text twice") and the set is never
  empty (empty input → `vec![car(0)]`). So the per-caret list is STRICTLY ascending with no duplicate
  `at`. Two carets on the same ROW at different columns is the real case — REQ-007's second half.
- **R3 — #282 empty-group semantics at N.** The block branch keeps its unconditional group, so a no-op
  ⇧Tab still opens-then-drops and cannot clobber a pending redo. REQ-009 pins it at N rather than
  trusting that N=1 generalizes.
- **R4 — the builders' mutant set is a PREDICTION.** Phase 4 must run `cargo mutants --list -f
  crates/editor/src/indent.rs` and design killers for the real set, not the guessed one.
- **R5 — the shared tail is a refactor of working code.** The block branch must come out
  byte-identical for the single-range case; REQ-008 leans on the *unchanged* #276 drive to prove it
  rather than on new assertions written by the same hand that moved the code.

**Status: Phase 2 — Design PASS.**

## Phase 3 — Implement

Built in two compiling-green groups per the manifest. No test bodies expanded (Phase 4 owns them);
only the call-site fixes needed to compile.

### Group 1 — `crates/editor/src/indent.rs` (+ one comment in `comment.rs`)

- `indent_edits(buffer, rows: &[usize], tab_width)` and `dedent_edits(buffer, rows: &[usize],
  tab_width)`. Both copy `rows` into a local `Vec`, `sort_unstable()`, `dedup()`, then run their
  existing per-row logic unchanged.
- The `last.min(buffer.len_lines().saturating_sub(1))` clamp is GONE from both. Each fn's doc states
  why totality survives (`line_text` → `""` past EOF, so indent's empty-filter and dedent's
  `strip > 0` guard drop the row before `line_start` is consulted). No redundant bounds guard added.
- Kept exactly: indent's `.is_empty()` skip (D4), dedent's ≤1-stop-of-spaces / one-`\t` strip, and
  both `tab_width.max(1)` floors.
- Doc comments cross-reference `comment::comment_edits` as the sibling taking the same stance.
- **`comment.rs`'s enforcement comment updated (in-scope staleness, not scope creep):** it read as
  prose about a *future* caller ("forge #307 adds a second caller"). That caller now exists, so it
  states the shared contract in the present tense and points back at `indent_edits`/`dedent_edits`.
- **`touched_rows`' doc comment was also stale** — it explained that Tab could NOT use it because the
  builders took a contiguous range. Rewritten to describe the shipped state, keeping the old behavior
  as the parenthetical history that explains the old bug.
- Compile-only test fixes: t4/t5 calls converted to row lists; t7's round trip passes
  `&(first..=last).collect::<Vec<_>>()` (it tests the apply/rebase round trip, not the row-list shape,
  so materializing the range at the call keeps its intent). t4 renamed `…_skip_empty_and_clamp` →
  `…_skip_empty_and_past_eof` with its "clamp" wording corrected, since clamping is no longer the
  mechanism. One further stale comment fixed on the C11 test ("Tab acts on the primary cursor's line
  span").
- `cargo check -p marley_editor --all-targets`: **green**.

### Group 2 — `crates/marley_app/src/app.rs` (the Tab arm)

- `has_selection` (primary-only) → `any_range = before.selections().iter().any(|sel| !sel.is_caret())`,
  computed off the already-cloned `before`, so the borrow order is unchanged.
- Block branch: `line_span(primary)` → `touched_rows(buffer, &before)`, feeding the widened builders.
- Pad branch: maps every member to `(caret, 0, pad)`, each pad sized from that caret's own
  `line_layout` column.
- **The two tails merged into one:** `if group { begin }` → back-to-front apply →
  `rebase_selections` → `if group { end }` → `set_selection`. `group` is unconditionally `true` on
  the block branch (D3a — preserves the #282 empty-group drop) and `edits.len() > 1` on the pad
  branch (F3 — a lone pad must stay ungrouped or ⌘Z granularity changes).
- Dead locals removed: `primary`, `caret`, `anchor` — the arm no longer reads the primary at all.
  `touched_rows` added to the import list and **`line_span` removed from it**: the arm was its only
  app-side consumer, and the leftover import is a `-D warnings` clippy failure (caught by the check,
  fixed at source rather than allowed).
- Comments rewritten: both paragraphs asserting Tab acts on the primary's span (one naming #307 as a
  future ticket) are gone; the D6 raw-edit/self-bracketing contract comment stays verbatim; new
  comments record (a) the whole-set predicate and why primary-only was wrong, (b) the #282
  always-group rule on the block branch, and (c) the lone-pad-stays-ungrouped rule *with* its
  `cursor_anchored`/`coalesces_into` mechanism — the one a future "simplification" would otherwise
  undo silently.
- `cargo check -p marley --all-targets`: **green**, warning-free after the import fix; `cargo fmt
  --all` applied.

### Deviations from design

None material. Two incidental extras, both in-scope staleness the design implied but did not
enumerate: `comment.rs`'s enforcement comment and `touched_rows`' doc comment (plus one test
comment), all of which asserted the pre-#307 world as current fact. Leaving them would have left the
crate documenting behavior it no longer has — the "a false doc is well-formed Rust" trap, which cost
a full cycle on #337.

**Status: Phase 3 — Implement PASS.**

## Phase 3.5 — Inspect

Three critics were dispatched in parallel over the diff (lens 1 shared-tail behavior preservation,
lens 2 pure-builder totality + the deleted clamp, lens 3 multi-cursor/undo semantics). **Process note,
recorded honestly (§15):** two of the three were still mid-`cargo` build when this phase closed — the
app crate takes 5–8 minutes and one critic was compiling `-p marley` to run the shipped indent drive.
Rather than block the pipeline indefinitely I re-ran **all three lenses myself** against live code;
every finding below carries the evidence I read or ran, not a critic's summary. One critic's
throwaway probe did land before it stalled, and its evidence is credited in F3.

### Findings

| # | Severity | Finding | Verdict | Resolution |
|---|---|---|---|---|
| F1 | MEDIUM | `indent_edits`' doc justified the deleted bounds clamp by saying the filter drops out-of-range rows "before `line_start` is ever asked about it" — implying `line_start` is the hazard | **REAL (doc accuracy)** | `Buffer::line_start` (buffer.rs) opens with `let row = row.min(self.len_lines().saturating_sub(1));` — it CLAMPS, it does not panic. So the filter is not preventing a crash; it is preventing something quieter and worse: an out-of-range row would resolve to the LAST line's start and silently indent/dedent the wrong line. Both fns' docs rewritten to state the actual hazard. Code unchanged — it was already correct. |
| F2 | MEDIUM | The design notes called the same-row twin-caret divergence "invisible in practice (both still reach a tab stop relative to the pre-edit text)" | **REAL (false justification)** | Traced `"ab\n"`, carets at 1 and 2, `tab_width` 4: pads 3 and 2 → `"a   b  \n"`, cursors at 4 and 7. Each cursor lands correctly after its OWN padding, but cursor 2 sits at column 7 with stops at 4 and 8 — it does NOT finish on a post-edit stop. The decision stands; the stated reason overclaimed. Notes + the app.rs comment rewritten to state what actually happens, and to note that carets on different rows (the common case) never interact. This is the `CHECK THE REASON, NOT JUST THE FIX` class from the M22 ledger. |
| F3 | — | Does deleting the clamp change behavior for ropey's phantom trailing row, or panic on a far-past-EOF row? | **NO CHANGE (verified)** | A critic's probe (since removed) ran: `"a\nb\n"` has `len_lines()==3`, and the OLD clamp DID include phantom row 2; `indent_edits(&b, &old_rows, 4) == indent_edits(&b, &[0,1,2,99], 4)` and likewise for dedent; `indent_edits(&b, &[usize::MAX], 4) == []` with no panic, same for dedent and for `&[3,4,500,usize::MAX]`. Old and new agree; totality holds at the extreme. |
| F4 | — | `clear_marked()` now runs BEFORE the branch predicate is computed (it used to run after `has_selection` was read) | **NOT A DEFECT (verified)** | `EditorSurface::clear_marked` (editor_surface.rs:326) is `self.files[self.active].marked = None;` — it touches the IME marked span only, never the selection set. The reorder is inert. Recorded because a reviewer will ask. |
| F5 | — | Could the pad branch's edit list contain two edits at the SAME offset (breaking `rebase_through`, which `break`s on `p < at`)? | **IMPOSSIBLE (verified from the implementation, not its tests)** | `SelectionSet::from_selections` sorts then merges on `overlaps`, whose caret arm is `next.start() <= cur.end()` — same-offset carets collapse to one. Empty input returns `single(caret(0))`, so the set is never empty. The `selections` field is PRIVATE; the only constructors are `single`/`from_selections`; and `indent::rebase_selections` rebuilds its result THROUGH `from_selections`, so the post-rebase set is re-normalized too. The strictly-ascending precondition is guaranteed by construction, not by convention. |
| F6 | — | Does the block branch's now-conditional `if group` still preserve the #282 no-op invariant? | **YES (verified)** | `undo::end_group` returns early when `group.records.is_empty()` — **before** `self.redone.clear()`. An empty group is dropped AND a pending redo survives. The block branch passes `group = true` unconditionally, so its behavior is identical to the old unconditional bracketing. |
| F7 | — | Can a multi-pad group now coalesce with following typing (the #338 corruption shape)? | **NO (verified)** | `Buffer::begin_undo_group` (buffer.rs:708) calls `begin_group(sel_before, /*cursor_anchored*/ false)`, and `undo::coalesces_into`'s first conjunct is `prev.cursor_anchored && group.cursor_anchored` — false for any group opened this way, so it short-circuits. Nothing in this diff routes the arm through a `cursor_anchored: true` path. The single-pad case remains ungrouped and still coalesces via `record()` exactly as before. |
| F9 | **MEDIUM** | The lone-pad comment (and Phase-1 F3, and D3's spec note) justified "don't group a single pad" via `coalesces_into`/`cursor_anchored` — the WRONG mechanism | **REAL (false reasoning, correct decision)** | Verified against the code: `coalesces_into` is called only from `undo::end_group`, i.e. GROUP-into-GROUP. A following single keystroke never opens a group — `Buffer::edit_ranges_restoring_placing` takes its one-member ungrouped fast path — so the typed char reaches `UndoHistory::record`, whose guard is `group.sel_before.is_none() && group.records.len() == 1`. `cursor_anchored` is never consulted on that path. Grouping the pad sets `sel_before: Some(..)`, and that is what would block the coalesce. The shipped precedent states it correctly at `buffer.rs:613-617` and I cited the wrong sibling. **Why it matters beyond pedantry:** if someone later changes `begin_undo_group` to pass `cursor_anchored: true` (a plausible cleanup), my comment reads as a licence to group the lone pad and ⌘Z granularity silently regresses. Fixed in app.rs, the notes' F3, and the spec's D3; `cursor_anchored` now appears only on the N≥2 half where it genuinely applies. |
| F10 | LOW | `comment.rs`'s doc linked `[`crate::indent_edits`]`, which resolves nowhere (the fn is at `crate::indent::indent_edits`) | **REAL (pre-existing, adjacent)** | Invisible to gate:14 (it omits `--document-private-items` and the link sits on a private fn), so not a gate red — but the diff's whole thesis is that these two siblings cross-reference each other, and this particular cross-ref was broken. One-word fix applied. |
| F11 | LOW | `indent_edits`' doc said "a block indent must not seed whitespace-only lines", which inverts on a plain reading | **REAL (ambiguous doc)** | The filter is `!is_empty()`, so a line that is ALREADY whitespace-only IS indented; only zero-length lines are skipped. The intended reading ("don't CREATE a whitespace-only line from an empty one") is correct, but it is the one sentence here a reader can invert — and `comment.rs` needed a whole paragraph to undo exactly that confusion. Reworded to state both halves explicitly. Also fixed "a second, unreachable spelling" → "redundant": a `row < len_lines()` guard would be perfectly reachable, just subsumed by the emptiness filter. |
| F12 | **MEDIUM** | `EditorSurface::active_selection`'s doc still listed "Tab's 'is there a selection'" among the consumers that legitimately want the PRIMARY only | **REAL** | Tab no longer calls it — that is this ticket's headline fix. Worse than staleness: the sentence justifies the fn being primary-only by citing as its lead example the exact call site #307 just proved was WRONG to be primary-only. Rewritten to drop Tab, record why it was wrong ("is there a selection anywhere in the set" is not a one-range question), and warn the next person to check before adding a consumer. |
| F13 | LOW | The shared tail's comment said `rebase_selections` over an empty list "is the identity" | **REAL (overclaim)** | It rebuilds every member through `Selection::new(...)`, which deliberately resets the sticky goal column to `None` (selection.rs:31-37). OFFSETS are unchanged — which is the property the #282 no-op invariant actually needs — but "identity" is wrong. Reworded to say offsets are unchanged and note the goal-column reset happens on any edit. |
| F14 | LOW | The spec still carried F2's retracted "invisible in practice" justification | **REAL (incomplete propagation)** | I fixed the notes and the app.rs comment at inspect but not `spec.md:57`, so the artifact I had explicitly classified as a false justification survived in the document a reader reaches first. This is the M22 ledger's own rule — *when one recorded fix proves unapplied, AUDIT the rest* — recurring on my own fix. Spec now carries the worked example and the honest consequence. |
| F15 | LOW | The per-caret pad list TRUSTS the ascending precondition that D1 makes both row builders ENFORCE | **REAL (consistency)** | The precondition genuinely holds (verified three ways: private field, no struct literal outside `selection.rs`, `overlaps`' `<=` caret arm, and `rebase_selections` rebuilding through `from_selections`) — but it now rests on a guarantee from another file, asserted in the one file with no unit tests and no mutation coverage. Added a `debug_assert!` that the pads ascend strictly, so a future change to `SelectionSet`'s invariant fails loudly in test/debug rather than corrupting a buffer in release. Free at runtime; consistent with D1's own rationale. |
| F16 | INFO | ⇧Tab can now MERGE two cursors that sit inside the same row's indentation | **REAL but correct (documented)** | Two carets at cols 1 and 3 inside a 4-space indent both clamp to the shifted line start and `from_selections` merges them: 2 cursors → 1. Correct by the crate's law that one offset is one cursor, and the common case is safe (three carets on three indented rows keep all three). Pre-#307 this could only happen on the primary's row; now any row. Recorded rather than changed, because this repo carries a scar for SILENT cursor loss (#297 inspect C1) and the next person deserves to find this written down. |
| F8 | LOW | Two critics left artifacts in the tree (`crates/editor/src/zz_throwaway_critic.rs` + a `mod` line in `lib.rs`; `crates/editor/tests/zz_critic_307.rs`) | **REAL (hygiene)** | Removed; `lib.rs` restored to unmodified (`git diff --stat` empty for it); `cargo fmt --all --check` clean; `git status` shows exactly the manifest. This is the #259 F6 class recurring — critics must clean up after themselves; the instruction was given and still not honored, so it is worth a standing rule rather than a per-ticket reminder. |

**Rejected / not findings:** no `unwrap`/`expect` was added on any input-reachable path; no secrets, no process-spawn, no PTY surface touched; §20 clean (no Zed/Warp brand names in `crates/`, and no copyleft source was read — the only external source read was ropey/gpui at Phase 1, which is permissive adoption); the predicate flip's blast radius is zero (`active_selection()`'s one other production reader, the ⌘F reseed at app.rs:7056, is untouched — this diff stops the Tab arm CALLING it rather than changing it).

### Verification run after the fixes

- `cargo test -p marley_editor`: **228 passed, 0 failed** (+ 2 in the second target) — the pre-existing
  indent/comment/undo suites pass unchanged against the widened signatures.
- `cargo check -p marley_editor --all-targets` and `-p marley --all-targets`: green, warning-free.
  (One warning WAS produced and fixed at source during Phase 3: `line_span` became an unused import in
  app.rs once the arm stopped using it — `-D warnings` would have failed the gate.)
- `cargo doc -p marley_editor --no-deps`: no diagnostics — the new intra-doc links resolve (gate:14
  runs `rustdoc -D warnings` and reports only the FIRST error, so this was checked deliberately).
- `cargo fmt --all --check`: clean.

**Status: Phase 3.5 — Inspect PASS.**

## Phase 4 — Validate

### The mutant set — MEASURED, not guessed

`cargo mutants --list -f crates/editor/src/indent.rs` (the standing rule: run it on the ACTUAL code).
**Phase 2's R4 prediction held exactly** — `to_vec`/`sort_unstable`/`dedup` are method calls, which
cargo-mutants does not mutate, so the widening added **zero** new mutants. The viable set on the two
changed fns is what it was before:

```
indent.rs:99:5   replace indent_edits -> Vec<LineEdit> with vec![]
indent.rs:99:5   replace indent_edits -> Vec<LineEdit> with vec![Default::default()]
indent.rs:104:23 delete ! in indent_edits            (the empty-line filter)
indent.rs:118:5  replace dedent_edits -> Vec<LineEdit> with vec![]
indent.rs:118:5  replace dedent_edits -> Vec<LineEdit> with vec![Default::default()]
indent.rs:130:36 replace == with != in dedent_edits  (the leading-space test)
indent.rs:133:18 replace > with ==/</>= in dedent_edits  (the strip > 0 guard)
```

Killers: t4/t4a (non-empty exact lists kill both body mutants + the `!`), t5 (the `== ' '` swap makes
`"    aa"` strip 0; the `>` swaps make the no-strip row emit a 0-width edit — the shape assert).
**Note for the gate:** because the enforcement lines produce no mutants, MSI can be 100 with the
sort/dedup claim entirely unproven — which is precisely why REQ-003's `t4b` exists as a behavioral
test rather than being left to mutation. Critic 1 flagged the same gap independently.

### Tests added

*Pure — `crates/editor/src/indent.rs` (carries the cov/MSI-100 obligation):*
- `t4_indent_edits_exact_lists_skip_empty_and_past_eof` (REQ-001) — extended with a `usize::MAX` row
  proving totality at the extreme (no panic, no edits).
- `t4a_indent_edits_over_a_discontiguous_row_list` (REQ-001, NEW) — rows `[0,2,4]` over five 1-char
  rows; asserts the exact edit list AND that the gap rows' line starts appear in no edit. This is the
  case a contiguous `(first,last)` range could not express at all.
- `t4b_row_lists_are_sorted_and_deduped_internally` (REQ-003, NEW) — `[4,0,2,0,4]` yields the same
  list as `[0,2,4]`; asserts STRICTLY ascending `at`s (the back-to-front precondition the enforcement
  exists for) and that the duplicate produced one edit, not two. Covers both builders. Mirrors the
  shipped twin `comment.rs::f3_an_unsorted_or_duplicated_rows_slice_…`.
- `t5_dedent_edits_strip_rules_and_shape` (REQ-002) — extended with a discontiguous `[0,5]` case and
  the `usize::MAX` totality twin.

*Headless drives — `crates/marley_app/src/headless_drive.rs` (the ONLY proof for the arm, which is
coverage-excluded and inside the `mutants::skip`'d `render`). Plus a shared `editor_selections`
helper — the multi-cursor twin of the existing primary-only `editor_selection`:*
- `tab_pads_every_caret_headless` (REQ-004) — the ticket's own verify, on the PAD branch it actually
  reaches: three carets → all three padded; ONE ⌘Z reverts all three; the restored set still has 3
  members.
- `shift_tab_dedents_every_cursors_line_headless` (REQ-005).
- `mixed_set_takes_the_block_path_headless` (REQ-006) — caret primary + ranged secondary; asserts the
  range's rows are no longer ignored. This is the latent bug the predicate flip fixes as a rider.
- `two_cursors_on_one_row_indent_it_once_headless` (REQ-003 e2e).
- `per_caret_pads_size_from_each_carets_own_column_headless` (REQ-007) — pins the documented
  divergence with exact numbers (`"ab\n"` carets 1,2 → `"a   b  \n"`, cursors 4 and 7), so it is a
  tested decision rather than an accident.
- `single_caret_tab_still_coalesces_with_following_typing_headless` (REQ-008) — Tab → type → ONE ⌘Z
  reverts both, proving the lone pad stayed ungrouped. This is the test that would fail if someone
  "simplified" the `edits.len() > 1` condition away.
- `noop_shift_tab_at_n_preserves_redo_headless` (REQ-009) — #282 proven at N, not assumed to
  generalize from N=1.
- The shipped `editor_tab_indent_dedent_flows_headless` (#276) is left **UNCHANGED** and must stay
  green — it is REQ-008's real baseline, written before this ticket existed.

### Test results — ACTUAL output

- **Pure (`cargo test -p marley_editor`): 230 passed, 0 failed** (+2 +1 in the other targets). Was 228
  before this ticket; the delta is `t4a` and `t4b`.
- **Headless (`cargo test -p marley --lib -- headless_drive`): 155 passed, 0 failed, 544 filtered.**
  All seven new drives green on their first run, and the shipped `editor_tab_indent_dedent_flows_headless`
  (#276, untouched) green alongside them:
  ```
  test headless_drive::tab_pads_every_caret_headless ... ok
  test headless_drive::shift_tab_dedents_every_cursors_line_headless ... ok
  test headless_drive::mixed_set_takes_the_block_path_headless ... ok
  test headless_drive::two_cursors_on_one_row_indent_it_once_headless ... ok
  test headless_drive::per_caret_pads_size_from_each_carets_own_column_headless ... ok
  test headless_drive::single_caret_tab_still_coalesces_with_following_typing_headless ... ok
  test headless_drive::noop_shift_tab_at_n_preserves_redo_headless ... ok
  test headless_drive::editor_tab_indent_dedent_flows_headless ... ok
  ```
- `cargo fmt --all --check` clean; `cargo doc -p marley_editor --no-deps` no diagnostics; clippy
  `-D warnings` green (gate:1 and gate:2 both passed on the gate run before it stalled).

### ✅ GATE GREEN [diff] — 15/15

```
mutation: 6 caught / 0 missed → MSI 100.0% (floor 100%)
     Summary [   6.266s] 1682 tests run: 1682 passed, 5 skipped     (coverage lane)
     Summary [   0.827s]  158 tests run:  158 passed, 2 skipped     (visual/AX lane)
  PASS gate:1 rustfmt · gate:2 clippy (-D warnings) · gate:3 tests (nextest + doctests)
  PASS gate:7 cargo-audit · gate:8 cargo-deny · gate:9 cargo-machete · gate:10 gitleaks
  PASS gate:11 shellcheck · gate:12 no-suppressions · gate:13 source-bans (SAST)
  PASS gate:14 docs (rustdoc -D warnings + doc-todos + brand-scrub)
  PASS gate:4 rust coverage (>= 100% lines) · gate:5 mutation (MSI >= 100%)
  PASS gate:6 miri (unsafe crates) · gate:15 visual / AX
  15 passed, 0 failed
GATE GREEN [diff]
```

Receipt written and verified against `gate_state_hash` (`c3d8984c…`), so `/commit` is unblocked.
The 6 mutants are exactly the diff-scoped set `cargo mutants --list` predicted, all caught — no
surviving mutant, and the enforcement lines' zero-mutant status is covered behaviorally by `t4b`
rather than being left to mutation (see the mutant-set section above).

### The OS fault that blocked the first two gate attempts (kept — it is evidence for #348)

The first two `--diff` runs stalled at **gate:3 (nextest)** and were diagnosed rather than retried:

1. `cargo-nextest` at 0% CPU, state `S`, holding 40 pipe FDs; children stuck in `--list --format
   terse` — no test had begun executing.
2. Running one of those binaries directly (`integration-…`, the `marley_terminal` real-PTY suite)
   hung emitting **zero bytes**, and hung on `--help` too — blocking at STARTUP, before libtest
   parsed an argument.
3. `sample` showed the whole stack as **`_dyld_start (in dyld)`** — stuck in the dynamic linker.
4. **A trivial freshly-compiled `fn main(){println!("ok")}` hung identically**, as did an
   ad-hoc-`codesign`ed copy, while `/usr/libexec/syspolicyd` burned a sustained ~15–29% CPU.

So the machine's Gatekeeper daemon was intermittently blocking execution of every newly-created
binary. Rejected explanations, each tested: a stale `serial_test` lock (no lock files), `ReportCrash`
holding the process (killed it, no change), a pipe-buffer deadlock in `--list` (the binary hangs with
no consumer at all), a leftover PTY child (none existed). The fault cleared on its own, recurred
briefly during doctests, and the third run completed clean — nothing in this diff was involved, and
nothing was worked around or suppressed to get green.

**Filed as evidence on #348** (ticket comment): that ticket's premise is `resize_real_pty_succeeds`
hanging under coverage, but this fault produces the IDENTICAL signature (0% CPU, no output, nextest
reporting SLOW forever) *without the test ever running* — nextest attributes the stall to the first
test in the binary. #348's Phase 1 must distinguish the two before building its second half; its
nextest terminate-ceiling half is correct either way, since it bounds "no progress" regardless of
cause.

`scripts/gates.sh --diff` was run TWICE and stalled both times at **gate:3 (nextest)**, after gate:1
(rustfmt) and gate:2 (clippy) passed. Diagnosed rather than retried:

1. `cargo-nextest` sat at 0% CPU in state `S`, holding 40 pipe FDs, with its children stuck in the
   `--list --format terse` discovery phase — no test had begun executing.
2. Running one of those binaries directly (`integration-…`, the `marley_terminal` real-PTY suite)
   hung producing **zero bytes** — and hung on `--help` too, so it was blocking at STARTUP, before
   libtest parsed a single argument.
3. `sample` on the hung process shows the entire stack as **`_dyld_start (in dyld)`** — stuck in the
   dynamic linker's entry point, before any user code.
4. **A trivial, freshly-compiled `fn main(){println!("ok")}` hangs identically**, and so does an
   ad-hoc-`codesign`ed copy of it. Meanwhile `/usr/libexec/syspolicyd` (Gatekeeper/notarization) is
   burning a sustained ~15–27% CPU.

**Conclusion: `syspolicyd` is wedged and blocking execution of every newly-created binary on this
machine.** That is why the gate dies precisely when nextest first execs freshly-built test binaries,
and why the same suites passed minutes earlier from already-validated binaries. Nothing in this diff
can cause or fix it. Rejected explanations, each tested: a stale `serial_test` lock (no lock files);
`ReportCrash` holding the process (killed it, no change); a pipe-buffer deadlock in `--list` (the
binary hangs with no consumer at all); a leftover PTY child (none exist).

**Not worked around, not faked.** The §0 rule is fix-at-source, and the source here is outside the
repo. Recovery needs an action only the machine's owner can take (restart `syspolicyd`, or reboot);
`sudo` was deliberately not attempted. The gate must be re-run to green before `/commit` — the
`enforce-commit-gate.sh` hook will block the commit until it is, which is the correct outcome.

**This is a live instance of the class #348 exists to fix** — a hung gate that reports nothing: no
red, no green, just silence. It also reframes #348's evidence: that ticket blamed an 11-hour zombie on
a real-PTY test hanging under coverage, but the same signature (0% CPU, no output, indefinite) is
produced by this OS fault, so #348's Phase 1 should re-check whether its original 39,240-second stall
was the PTY test at all or this same dyld/syspolicyd block. Recorded on the ticket.

### LIVE drive — deferred, not skipped

The ticket's verify step asks for a live ⌘⌥↓ ×2 → Tab → ⌘Z. **chad is at the machine, so synthetic
input is off-limits** (it lands in whatever window is frontmost). REQ-004's headless drive carries it:
the same keystrokes, injected through the #264 `TestAppContext` lane against the real `RootView`,
asserting the same three facts (all rows padded, one ⌘Z reverts, cursors carried). Re-runnable live in
~30 seconds when the machine is free; no follow-up ticket needed.

**Status: Phase 4 — Validate PASS.**

## Phase 5 — Complete

**Docs (§21):** `CHANGELOG.md` gained a `### Fixed` entry for #307 (the union/per-caret behavior, the
row-list builders + their enforced contract, the dropped clamp and the silent-wrong-line hazard it
really guards, the mixed-set rider, and the deliberate ⌘Z asymmetry). `docs/marley_architecture/editor.md`
gained a "Tab / ⇧Tab — line indentation at N cursors" section under the undo-group discussion, stating
the shared-tail shape, the enforcement stance, the totality argument, and — explicitly — the grouping
asymmetry WITH the correct mechanism (`UndoHistory::record`'s guard, not `coalesces_into`).
`docs/marley_architecture/roadmap.md` marks #307 SHIPPED and records that **B-a is now COMPLETE**;
with B-b already complete, the whole remaining M22 surface is the B-c display-map chain.

**Capture (forge):** AAR `7cb51412` submitted — outcome completed, effectiveness 4, 5 novel findings,
distillation/confidence-drift/pattern-emergence jobs enqueued. Two failures + three prevention rules
materialized across inspect and complete:

| Code | What it pins |
|---|---|
| `BF-claude-clamp-removal-doc-named-the-wrong-hazard-001` | The dropped clamp's doc named a panic that cannot happen; `line_start` clamps, so the real hazard is a silent wrong-line edit |
| `PR-claude-deleting-a-guard-name-the-hazard-you-measured-001` | When deleting a bounds check, read the callee and name the failure mode you MEASURED; a clamping callee is more dangerous than a panicking one |
| `PR-claude-inspect-critic-must-leave-the-tree-as-it-found-it-001` | Critics must delete probes AND revert registrations; the orchestrator re-verifies `git status` + `fmt --check` rather than trusting the instruction |
| `BF-claude-undo-granularity-comment-cited-the-wrong-coalesce-path-001` | The lone-pad justification cited `coalesces_into`/`cursor_anchored`, unreachable on the single-cursor path; the real guard is `record`'s `sel_before.is_none()` |
| `PR-claude-name-the-function-that-actually-runs-on-that-path-001` | Justify a grouping decision with the function that actually executes there, and match the shipped precedent that already says it correctly |

**Follow-up filed:** forge **#358** — "⇧Tab can merge two cursors that sit inside the same row's
indentation (2 cursors → 1)". Correct by the crate's one-offset-is-one-cursor law and ⌘Z restores the
set, but pre-#307 it could only happen on the primary's row; filed with three design options and a
verify plan because this repo carries the #297 C1 silent-cursor-loss scar.

**The lessons worth carrying forward:**
1. **The prior-art sweep changed a locked decision before any code existed.** `comment_edits` already
   had the exact target signature AND enforced the contract, with a comment naming #307 as its incoming
   second caller. D1 went from "document the precondition" to "enforce it" on that evidence alone.
2. **"Right decision, wrong reason" hit THREE times in one ticket** — the clamp hazard, the divergence
   justification, and the undo mechanism. I caught the first two myself; the third took two independent
   critics, one of which killed it with a counterfactual. A plausible reason terminates review before
   anyone measures it, which is precisely why it survives to the end.
3. **Mutation could not have covered the ticket's central claim.** `cargo mutants --list` showed the
   sort/dedup enforcement lines generate ZERO mutants (method calls are unmutated), so MSI 100 was
   reachable with the contract entirely unproven. `t4b` exists because of that, not in spite of it.
4. **A ticket's VERIFY step can contradict its WORK section.** #307's own verify (⌘⌥↓ ×2 → Tab) drove
   the bare-caret branch its work items never named. Check that consistency, not just the claims.

**Ticket + archive:** forge #307 closed done; `TICKET-307` moved to `tickets/closed/` with
`Status: closed`; this pair archived to `pipeline/completed/`.

**Status: Phase 5 — Complete PASS.**
