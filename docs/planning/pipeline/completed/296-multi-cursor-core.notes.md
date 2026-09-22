# Multi-cursor core — Notes

- **Forge ticket:** #296 `2fb2b40a-d84c-4846-9a20-8265b0532ef8` — THE M19 KEYSTONE
- **AAR:** `d68a1852-579a-4110-90ce-7683688f7c11`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-296-multi-cursor-core.md
- **Pipeline spec:** 296-multi-cursor-core.spec.md

## Phase 1 — Plan
- **Request:** land the multi-member `SelectionSet` + the N-caret edit + one undo unit.
- **Classification / tier:** work pipeline slice, `feature`. Crates: `marley_editor` (pure —
  selection.rs, buffer.rs), `marley_app` (shim — N-caret render + typing route). KEYSTONE:
  #297/#298/#299/#300/#303 all depend on it.

### The gap (verbatim in-tree)
`crates/editor/src/selection.rs`: *"M1.A always holds exactly one member (`single`); the
multi-member sort/merge constructor (`from_selections`) is deferred to M1.B."* The `Vec` is
there; only ever one member in it. This ticket IS that M1.B.

### KEY discoveries from grounding (they CHANGED the design)
1. **`find::replace_all` is the already-shipped N-edit-on-one-buffer pattern**, and its comment
   states the rule outright: *"Bracket the back-to-front sweep into one undo group … offset-stable
   under back-to-front application (later replaces sit above it)."*
   ```rust
   buffer.begin_undo_group(sel);
   for (start, end) in matches.iter().rev() { buffer.edit(*start..*end, repl, Human); }
   buffer.end_undo_group(sel);
   ```
   → **D1: apply the N edits BACK-TO-FRONT; NO rebasing is needed at all.** The ticket's original
   "rebase each later offset by the cumulative delta" framing was the harder, needless path.
2. **The one genuinely NEW computation is the RESULTING selections.** `replace_all` never needed
   them (the find bar has no caret — its comment says so), so nothing in-tree computes "where do
   N carets land after an N-caret edit". That forward cumulative shift is the pure seam (D2) — and
   the thing most likely to carry an off-by-one, so it is what the fuzzer must attack.
3. **One undo unit is FREE** — `begin_undo_group`/`end_undo_group` shipped in #282 precisely so
   "a whole block indent / replace-all undoes in a single step" (D3).
4. `rebase_offset(offset, bias, delta)` (anchor.rs) + `edits_since` exist and are the ORACLE for
   the differential fuzzer (an independent front-to-back path), not the production path.

### The validate mandate (forge recall — PR-claude-fuzz-the-equivalence-claim, from #285/#288)
The N-caret edit is an **equivalence invariant**, and the recalled rule is explicit: *for a subtle
equivalence claim, a DIFFERENTIAL FUZZER is the authority; inline traces + hand-picked corpora miss
whole classes.* That rule was earned by a CRITICAL bug the #285 fuzzer caught (and it caught the
first two attempted fixes as still-wrong). So REQ-004 mandates a deterministic seeded-LCG fuzzer with
TWO independent implementations:
- **Production:** back-to-front sweep + the FORWARD-computed selections (pure math, no rebasing).
- **Oracle:** front-to-back, one edit at a time, rebasing every remaining selection through the REAL
  `rebase_offset` + the actual `BufferDelta` returned by `Buffer::edit`.
Assert byte-identical final text AND identical final `SelectionSet`. ≥50k steps, 0 divergences.

- **Scope decision / honest limitation:** #296 has NO gesture to create a 2-caret set (that is #297),
  so it CANNOT be driven through the UI. Validation = pure units + the fuzzer + a HEADLESS test that
  sets the set programmatically and types. The live N-caret PIXEL drive lands with #297 — stated in
  the spec, not silently skipped.
- **Risk:** moderate-high for a pure crate — the forward-shift math is exactly the kind of off-by-one
  that unit fixtures rubber-stamp and a fuzzer catches. Mitigated by REQ-004. The merge (`from_selections`)
  is standard interval-merge but the ADJACENT-vs-overlapping boundary is a classic `<` vs `<=` trap →
  trace the real `cargo mutants --list` set (never guess the operators).

## Phase 2 — Design

### Architecture / approach
**#296 is a PURE `crates/editor` ticket. No app changes.** (D6 — see below.) Four additions, all in
the gpui-free editor crate; the app keeps its single-caret path untouched until #297.

1. **`Selection::start()` / `end()` / `len()` / `is_caret()`** (selection.rs) — `Selection` today exposes
   only `anchor()` / `head()`, with NO ordering helpers. `start = min(anchor, head)`, `end = max(...)`.
   Every merge/shift below needs them.
2. **`SelectionSet::from_selections(Vec<Selection>) -> SelectionSet`** — sort by `start()`, then a single
   forward pass merging while `next.start() <= cur.end()`. The `<=` (not `<`) is deliberate and is the
   whole trap: it covers OVERLAPPING **and** ADJACENT/touching ranges **and** collapses same-offset carets.
   Union = `Selection::new(min_start, max_end)` (anchor=start, head=end — a canonical forward orientation).
   A merge of two carets at the same offset yields that caret. Never empty (≥1 in ⇒ ≥1 out).
3. **`selections_after_multi_edit(set, repl_len) -> SelectionSet`** — the ONE genuinely new computation
   (nothing in-tree does it: `replace_all` never needed resulting carets). For selection *i* ascending:
   `new_caret_i = start_i + repl_len + delta_before_i`, where
   `delta_before_i = Σ_{j<i} (repl_len as isize − len_j as isize)`.
   **The running delta MUST be signed (`isize`)** — a DELETE (`repl_len < len_j`) makes it negative, and a
   `usize` accumulator underflow-panics. The final position is provably ≥ 0. Each result is a zero-width
   CARET (typing over a selection collapses it — D5).
4. **`Buffer::edit_at_selections(&mut self, set, replacement, origin) -> SelectionSet`** — the thin driver:
   `begin_undo_group(SelSnapshot{primary})` → for each selection **in REVERSE** (`.iter().rev()`):
   `self.edit(sel.start()..sel.end(), replacement, origin)` → `end_undo_group(SelSnapshot{new primary})` →
   `let new = selections_after_multi_edit(set, repl_len)` → `self.set_selection(new.clone())` → return it.
   **Back-to-front needs NO rebasing** — this is the shipped `find::replace_all` idiom verbatim (D1).

**§14:** all pure, total (no panics — the signed delta removes the only underflow); no IO; no unsafe.

**§20 (clean-room) — CONFIRMED.** Reference = Zed / VS Code multi-cursor semantics: an ordered, disjoint
selection set; merge-on-overlap; one edit at every cursor; the whole gesture is ONE undo unit. Marley
matches with its own `SelectionSet` + the shipped `Buffer`/undo-group machinery (#282). Observed behavior
only — no Zed source read or translated.

### File manifest
| File | Change |
|------|--------|
| `crates/editor/src/selection.rs` | + `Selection::{start,end,len,is_caret}`; + `SelectionSet::from_selections` (sort+merge); + `selections_after_multi_edit` (the signed forward shift). Tests in P4. |
| `crates/editor/src/buffer.rs` | + `Buffer::edit_at_selections` (undo-group + back-to-front sweep + set_selection). Tests in P4. |
| *(no `marley_app` changes)* | The app-side N-cursor surface state / N-caret render / typing route is **#297** (D6). |

### Regression Test Plan
| # | Test | AC | Asserts |
|---|---|---|---|
| T1 | `selection::from_selections_merges_to_ordered_disjoint` | REQ-001 | The merge truth table: same-offset carets COLLAPSE; OVERLAPPING union; **ADJACENT/touching union** (the `<=` boundary — the classic `<` vs `<=` mutant); already-disjoint passthrough; UNSORTED input comes out sorted; a single member passes through unchanged. |
| T2 | `selection::selections_after_multi_edit_shifts_forward` | REQ-002 | Fixtures: 2 carets + a 1-char insert; 3 carets; a caret + a RANGE selection mixed; a **DELETE** (`repl_len=0 < len_j`) proving the signed delta (a `usize` accumulator would panic here); an insert at offset 0. |
| T3 | `buffer::edit_at_selections_is_one_undo_unit` | REQ-003 | A 3-caret edit → `undo()` ONCE → text is byte-identical to the original AND the selection is restored; `redo()` → back to the edited text. |
| T4 | **`t296_multi_edit_differential_fuzz`** | REQ-004 | **THE AUTHORITY.** Seeded-LCG (the `t288_differential_fuzz` shape). Random buffer text, random N (1–6) disjoint selections (carets AND ranges), random replacement (incl. `""` = delete). **Production** = back-to-front sweep + the forward-computed selections. **ORACLE** = an INDEPENDENT front-to-back path: apply one edit at a time, and after each, rebase every REMAINING selection's start/end through the REAL `rebase_offset(off, bias, &delta)` using the ACTUAL `BufferDelta` from `Buffer::edit`. Assert **byte-identical final text AND identical final `SelectionSet`**. ≥50k steps, 0 divergences. |
| T5 | gate `scripts/gates.sh --diff` | REQ-005 | 100% line cov + MSI 100 on all four new fns (trace the REAL `cargo mutants --list -f selection.rs -f buffer.rs` set — never guess). |

**Uncoverable: NONE.** #296 is a pure crate with no UI surface, so there is no live-drive gap — the
N-caret PIXEL drive belongs to #297 (which introduces the gesture that can create a second cursor).
This is precisely why the shim was moved out (D6), rather than shipping an un-drivable render change.

### Risks / decisions
- **D6 — the app-side shim moves to #297 (SCOPE CORRECTION at design).** `EditorSurface`/`OpenFile` holds
  its own `caret` + `anchor` (NOT the crate's `SelectionSet`), with ~61 dependent call sites. Converting it
  is a real refactor, and #296 has no gesture to create a 2nd cursor — so the shim would be un-drivable.
  #296 = pure core (fuzz-proven); #297 = the app integration + the gestures that make it live-drivable.
- **D7 — the merge boundary is `<=`, not `<`** (adjacent/touching ranges MUST union, else two ⌘D
  selections that abut would stay separate and double-insert at the seam). This is exactly the
  `<`→`<=` mutant class; T1 pins BOTH sides of the boundary.
- **D8 — the running delta is SIGNED (`isize`).** A delete (`repl_len < len_j`) drives it negative; a
  `usize` accumulator would underflow-panic. T2's delete fixture is the guard.
- **D9 — the fuzzer's oracle must be genuinely INDEPENDENT** (front-to-back + the real `rebase_offset`
  + real `BufferDelta`s), not a re-statement of the production formula — otherwise it proves nothing.
  This is the #285/#288 lesson (`PR-claude-fuzz-the-equivalence-claim`), where the fuzzer caught a
  CRITICAL bug *and* rejected the first two attempted fixes.

## Phase 3 — Implement
- **`crates/editor/src/selection.rs`**
  - `Selection::{start, end, len, is_caret}` — `start = min(anchor,head)`, `end = max(...)`; a selection
    can be dragged BACKWARDS (`head < anchor`), so every merge/edit op works in `start..end` terms.
  - `SelectionSet::from_selections(Vec<Selection>)` — `sort_by_key(|s| s.start())` then ONE forward
    merge pass widening the running member while `sel.start() <= cur.end()` (the `<=` covers
    overlapping + adjacent + same-offset collapse, D7). A merged member takes the canonical forward
    orientation (`anchor=start, head=end`); a NON-merged member KEEPS its original orientation, so a
    backwards drag still extends correctly.
  - `selections_after_multi_edit(set, repl_len)` — the signed (`isize`) running delta (D8).
  - The `SelectionSet` doc comment that declared the deferral ("…deferred to M1.B") is now OBSOLETE →
    rewritten to state the landed invariant (ordered, disjoint, never empty).
- **`crates/editor/src/buffer.rs`**
  - `Buffer::edit_at_selections(set, replacement, origin) -> SelectionSet` — compute `after` FIRST
    (from the pre-edit set), `begin_undo_group(snapshot_of(set))`, sweep `.iter().rev()` calling the
    shipped `edit()` per member (BACK-TO-FRONT ⇒ no rebasing, D1), `end_undo_group(snapshot_of(&after))`,
    `set_selection(after)`, return it.
  - `Buffer::snapshot_of(set) -> SelSnapshot` — the PRIMARY (first) member. **Documented limitation:**
    `SelSnapshot` carries ONE `(anchor, caret)`, so undo reverts all N edits to the TEXT (one group) but
    restores only the PRIMARY cursor; restoring the whole set would need `SelSnapshot` to carry a
    `SelectionSet` — a follow-up. The `None` arm (an empty set) is unreachable but present so the fn is
    total (§14, no panics) — it must be COVERED in P4 via an in-crate `from_members(vec![])`.
- **Deviations:** none of substance. `sort_by_key(Selection::start)` did not compile (E0631 — `start`
  takes `self`, `sort_by_key` wants `&T`) → a closure. `cargo check` clean (crate + workspace); the
  115 existing editor tests still pass (no regression from the doc/API additions).
- **Tests deferred to P4** — incl. the differential fuzzer (REQ-004, the authority).

## Inspect (Phase 3.5)
Rigorous self-review + the REAL mutant list + a brute-force proof harness. (A correctness critic on
the offset math was spawned and runs on; its findings fold via a re-entry if real — the #292 flow.)
**TWO REAL FIXES applied.**

### F1 [MED→fixed] `pos as usize` was a silent-wrap landmine
The original used a SIGNED running delta and cast the result: `CharOffset::from(pos as usize)` where
`pos: isize`. Correct *today* — for an ORDERED+DISJOINT set the members before `i` all lie within
`[0, start_i)`, so `Σ_{j<i} len_j ≤ start_i` and `pos ≥ 0` (proved). **But if that invariant ever
broke, a negative `isize` cast to `usize` WRAPS to a huge offset SILENTLY** — no panic, just a caret
in hyperspace. Unacceptable in a foundation five tickets depend on.
**FIX:** reformulated in pure `usize` with two monotonically-increasing accumulators —
`pos = start_i.saturating_sub(removed_before) + inserted_before + repl_len`. Mathematically identical
for disjoint sets, the cast is GONE, and a hypothetical bad set stays bounded instead of wrapping.
(D8 is thereby superseded: the delta is now unsigned by construction, not "signed with care".)

### F2 [MED→fixed] `edit_at_selections` self-brackets — a nesting footgun for #299/#300/#303
`UndoHistory::begin_group` does `self.open = Some(..)`, which **OVERWRITES an already-open group and
silently DISCARDS its records** (verified in undo.rs). `edit_at_selections` opens its own group, so
calling it *inside* another `begin/end_undo_group` would lose the outer group's edits. The three
downstream ops that need several edits per cursor (comment toggle, line move, delete-word) will reach
for exactly that pattern.
**FIX:** the contract is now documented loudly on the fn — never nest it; an op needing multiple edits
per cursor must bracket its OWN group around raw `edit()` calls instead.

### Verified clean (my own traces + a brute-force harness)
- **Brute-forced 4,620 disjoint sets × replacement lengths** (standalone `rustc` harness) asserting
  the three claims the design rests on: (1) NO wrap — every position is a small real `usize`;
  (2) STRICTLY INCREASING — the N carets never collide (so `from_members` is safe *and* non-masking:
  a collision would survive to be caught, rather than being silently merged away by `from_selections`);
  (3) the last caret never exceeds the new text length — therefore `set_selection`'s clamp is a NO-OP
  and **the returned `after` cannot diverge from what the buffer holds** (the divergence I most suspected).
- **Merge (`from_selections`):** `<=` is required (ADJACENT/touching ranges MUST union, else two abutting
  ⌘D selections would double-insert at the seam). A fully-CONTAINED member cannot shrink the running one
  (`cur.end().max(sel.end())`). An equal-start pair (`[3..3]` caret + `[3..7]` range) yields `[3..7]` in
  BOTH input orders (traced) — no sort tiebreak needed. A NON-merged member KEEPS its backwards
  orientation (correct — the drag direction must survive for shift+arrow extend); a MERGED member takes
  the canonical forward orientation (direction is ambiguous after a union) — documented.
- **`after` computed pre-edit** is correct: the formula reads only the PRE-edit set (mapping pre→post
  offsets is its whole job); it never touches the buffer.
- **Undo group before/after** matches undo.rs's contract (`sel_before` restored on undo, `sel_after` on
  redo; an EMPTY group is dropped — unreachable here since N≥1 members each record one edit).
- **`snapshot_of`'s `None` arm** is unreachable but must be COVERED or the 100%-line gate fails —
  `SelectionSet::from_members(vec![])` is `pub(crate)`, so an in-crate test can reach it. Pinned for P4.
- **Real mutant set: 24** (via `--list`, not guessed). Notable: cargo-mutants generates `<=`→`>` but
  **NO `<=`→`<`** — so the adjacent-union boundary is NOT mutation-covered and must be tested for
  CORRECTNESS regardless (D7). All 24 are killable by the P4 plan.

**Ledger:** 2 real findings, both FIXED at source; the rest verified clean by trace + brute force.
Lenses: offset-math correctness, integer-safety (the wrap landmine), merge boundaries, undo semantics,
state-divergence (returned set vs buffer), mutation reachability, downstream-footgun. `cargo check`
clean; the 115 existing editor tests still pass.

---

## Inspect re-entry (Phase 3.5, second pass) — the critic came back with FIVE more

The correctness critic spawned in the first pass returned AFTER the gate had already gone green. It
found **five real findings the green gate could not see — three of them in code the 50k-step
differential fuzzer had been hammering all along.** All five reproduced with failing tests first, then
fixed. **This is the ticket's central lesson: the fuzzer proved the offset MATH and was blind to the
INVARIANT ENFORCEMENT around it, because both the production path and the oracle were fed sets that
had already been canonicalized by the very constructor whose enforcement was missing.**

### F3 [HIGH→fixed] `set_selection`'s clamp broke the ORDERED+DISJOINT invariant
`set_selection` clamped each member to the rope, then rebuilt with the raw in-crate `from_members` —
**no re-merge**. But **clamping is not injective**: two distinct out-of-bounds cursors collapse onto
the SAME offset and stayed in the set as DUPLICATES. `edit_at_selections` then edited at both.
Reproduced: `"abc"` + carets at 10 and 20 + `"X"` → **`"abcXX"`** — one visible cursor, two inserts.
**FIX:** `set_selection` re-canonicalizes → `SelectionSet::from_selections(clamped)`. Regression test
`clamped_duplicate_cursors_collapse_to_one`.

### F4 [HIGH→fixed] a stale post-undo selection reached ropey → PANIC
`edit_at_selections` writes `self.selection`, but `undo`/`redo` restore only the TEXT — they do not
maintain the set. So after a multi-caret edit + ⌘Z, `buffer.selection()` holds offsets past EOF
(reproduced: `[3, 7]` in a 3-char rope), and feeding those straight back in handed ropey an
out-of-bounds range → **panic at `rope.rs:952` "end is out of bounds"**. `edit_at_selections` is the
only `pub fn` that takes offsets it did not itself produce and forwards them to the rope, while its
sibling `set_selection` already clamped defensively — an inconsistency that read as an oversight and was.
**FIX:** `edit_at_selections` CLAMPS + canonicalizes its INCOMING set before use. Regression test
`stale_selection_after_undo_is_clamped_not_a_panic`.

### F5 [MED→fixed] the "never empty" invariant was fiction
`from_selections(vec![])` publicly returned an EMPTY set — a buffer with ZERO cursors, on which every
downstream `selections()[0]` panics. D4 ("the set never empties") and `snapshot_of`'s "unreachable"
arm were **documenting an invariant nothing enforced**.
**FIX:** enforced where it is claimed — empty input yields `single(caret(0))`. The doc now says
ENFORCED, not asserted, and `snapshot_of`'s empty arm is honestly described (still reachable via the
raw in-crate `from_members`, hence still covered).

### F6 [LOW→fixed] a no-op sweep pushed a phantom undo step
An empty replacement over all-caret members removes nothing and inserts nothing, yet still recorded N
no-op edits in an undo group — a ⌘Z that visibly does nothing. (`find::replace_all` guards this.)
**FIX:** early-return the set unchanged when `repl_len == 0 && every member is a caret`. Regression
test `noop_sweep_records_no_undo_group`. Both mutants the new guard introduced (`&&`→`||`, `==`→`!=`)
die on the tests above — MSI stayed 100.

### F7 [docs→accepted, flagged for #298] `<=` over-merges ADJACENT ranges
KEPT: `<=` is REQUIRED, because same-offset carets satisfy `next.start == cur.end` and leaving them
unmerged double-inserts. The price is that two merely touching RANGES also union (`[0..2]+[2..4]` →
`[0..4]`), so ⌘D over `"aaaa"` for `"aa"` would give ONE cursor where the shipped `find::replace_all`
gives two. Losing a cursor cannot corrupt text; a duplicate cursor can — so this is the safe direction
to err. **Documented on the fn with the exact refinement #298 will need**
(`sel.start() < cur.end() || sel.is_caret() || cur.is_caret()`).

### Ledger
5 findings, 5 real (0 rejected) — 4 fixed at source, 1 accepted-and-documented with its successor
ticket named. Every fix landed a permanent regression test; a range-first-with-nonempty-replacement
fixture was added to `selections_after_multi_edit` (the commonest real multi-cursor op, and the shape
that makes both running totals load-bearing at once). Lenses: correctness/offset-math, state
integrity, panic-reachability, undo semantics, downstream-footgun, mutation reachability.
**Gate after the fixes: GATE GREEN [diff] — 15/15, coverage 100%, MSI 100%.**

---

## Phase 4 — Validate

### Tests added

| Test | File | Proves |
|---|---|---|
| `from_selections_merges_to_ordered_disjoint` | selection.rs | **REQ-001** — the merge truth table: EMPTY input → one caret (the F5 enforcement); same-offset carets COLLAPSE; overlapping UNION; adjacent/touching UNION (the `<=` boundary); a fully-CONTAINED member does not shrink the running one; already-disjoint passthrough; unsorted input comes out sorted; an equal-start pair in BOTH orders agrees; a lone member keeps a BACKWARDS orientation. |
| `selections_after_multi_edit_shifts_forward` | selection.rs | **REQ-002** — the forward shift: 2 carets + insert; 3 carets; caret+range mixed; **range-FIRST with a non-empty replacement** (the commonest real op — typing over N selections — and the shape that makes BOTH running totals load-bearing at once); a DELETE (`repl_len 0 < len`, the fixture that proves the unsigned reformulation); an insert at offset 0. Plus `Selection::{start,end,len_chars,is_caret}`. |
| `edit_at_selections_is_one_undo_unit` | buffer.rs | **REQ-003** — a 3-caret edit → ONE `undo()` restores the original text byte-for-byte; `redo()` re-applies all three. Also asserts `buffer.selection() == the returned set`. |
| `snapshot_of_primary_and_the_unreachable_empty_arm` | buffer.rs | `snapshot_of` reads the PRIMARY member (kills the `Default::default()` mutant) + covers the total-function empty arm for the 100%-line floor. |
| **`t296_multi_edit_differential_fuzz`** | buffer.rs | **REQ-004, THE AUTHORITY** — seeded LCG, 5 seeds × 10,000 chains ≈ **50k steps**, 0 divergences. PRODUCTION (back-to-front sweep + the forward-computed cursors) vs an **INDEPENDENT** front-to-back ORACLE that applies one edit at a time and rebases every REMAINING selection through the REAL `rebase_offset(off, Bias::Right, &delta)` with the ACTUAL `BufferDelta`. Asserts identical final TEXT **and** identical final cursors **and** `buf.selection() == the returned set`. |
| `clamped_duplicate_cursors_collapse_to_one` | buffer.rs | **Inspect F3** — clamping is not injective; the duplicates must collapse or one cursor types twice. |
| `stale_selection_after_undo_is_clamped_not_a_panic` | buffer.rs | **Inspect F4** — a post-undo set holding offsets past EOF is clamped, not fed to ropey. |
| `noop_sweep_records_no_undo_group` | buffer.rs | **Inspect F6** — a no-op sweep pushes no phantom undo step. (Also kills both mutants the new guard introduced.) |

### Actual results
- `cargo nextest run --workspace` → **1075 tests run: 1075 passed, 5 skipped**.
- `cargo test --workspace --doc` → ok, 0 failed.
- The 8 #296 tests, by name → **8 passed**.
- `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15 passed, 0 failed**, including **gate:4 coverage ≥ 100% lines** and **gate:5 mutation MSI ≥ 100%** on every changed file (the real `cargo mutants --list` set was traced, not guessed — the two new mutants from the F6 guard, `&&`→`||` and `==`→`!=`, both die on the tests above).
- No pre-existing failures; nothing excluded.

### REQ-005
Met: the three pure fns (`from_selections`, `selections_after_multi_edit`, `edit_at_selections`) plus the `Selection` helpers are at 100% line coverage and MSI 100.

### NO LIVE DRIVE — stated explicitly, not silently skipped
**#296 ships the PURE crate only (D6).** The entire app-side shim — the N-cursor `EditorSurface` state, the N-caret render, the typing route — moved to **#297**, because #296 introduces **no gesture that can create a second cursor**: there is nothing a human (or `drive.swift`) can press to reach a multi-cursor state, so there are no pixels to capture. Driving the app today would only re-prove the single-caret behavior #253 already owns. The N-caret pixel drive (two carets visible, a typed char landing at both, ⌘Z reverting both) is **#297's acceptance**, where the gesture and the render land together and can actually be proven. `marley_editor` is a library crate with no UI surface of its own; gate:15 (visual/AX) is green.
