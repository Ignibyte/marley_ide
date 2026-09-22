# Fix the #285 shrinking-edit windowing regression — Notes

- **Forge ticket:** #288 `ce37cfba-3d91-45da-8514-d6f3dd91a4f0`
- **AAR:** `ec411627-3f7c-4507-bfe4-e28543c03f74`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-288-shrinking-edit-window-fix.md
- **Pipeline spec:** 288-shrinking-edit-window-fix.spec.md

## Phase 1 — Plan
- **Request:** fix the CRITICAL #285 regression — a shrinking edit (pure-deletion
  tail-trim, or a prefix re-tokenizing shorter) leaves the surviving token outside
  the damage window and unqueried → `splice_spans` drops the stale cached span →
  highlight vanishes.
- **Classification / tier:** work pipeline, one slice, `bug` (CRITICAL). Single
  crate `marley_syntax` (lib.rs): one new pure fn + one wiring line + tests. No UI
  surface (library) — the equivalence corpus + the differential fuzzer are the
  acceptance.
- **Root cause (confirmed via the #285 inspect differential-fuzz critic + the
  marley_syntax API):** `damage_window` unions `changed_ranges` with the edit span.
  For a pure deletion `new_end == start` the edit span is EMPTY `[p,p)`; when the
  deleted bytes were a surviving token's TAIL, `changed_ranges` is ALSO empty (the
  node's extent shrank but its bytes are textually unchanged) → `window = [p,p)`.
  `spans_in_window(set_byte_range([p,p)))` returns `[]` (tree-sitter needs a node
  strictly straddling `p`; the shrunk token ends EXACTLY at `p`). The grow-loop
  (`window_extent(window, [])`) can't widen. `splice_spans` drops the stale cached
  span. Gap. Same shape for `"42"→"4"` (a non-empty but incomplete `changed_ranges`
  missing the surviving prefix byte).
- **The flaw:** #285 assumed `changed_ranges ∪ edit-span ⊇ every stale span`. FALSE
  for SHRINKING edits.
- **Fix (D1/D2):** a pure `cover_edited_cached(window, cached_old, start, old_end,
  new_end)` widening the window to the surviving new-coord footprint of every cached
  span TOUCHING `[start, old_end)` (`s < old_end && e > start` — empty-window-proof,
  unlike a window-overlap test). Verified by hand on both repros:
  - `"// abcdef"→"// abc"` (start=6, old_end=9, new_end=6): comment `(0,9)` touches
    `[6,9)` → `lo=min(6,0)=0`, `e=9>=9` → `hi=max(6, 9-9+6)=6` → window `[0,6)`. Fresh
    over `[0,6)` returns the new comment `(0,6)`; the `fn` keyword `(10,12)` shifts to
    `(7,9)` and is KEPT by splice. `inc == fresh`.
  - `"42"→"4"` (start=1, old_end=8, new_end=2, changed `[1,3]`): int `(0,2)` touches
    `[1,8)` → `lo=0`, `e=2<8` → `hi=max(3, new_end=2)=3` → `[0,3)`. Fresh returns
    `(0,1)`. `inc == fresh`.
- **Durable fix (D4):** a bounded in-test differential fuzzer — a seeded LCG (the
  crate bans `Math.random`/`Date`) drives random insert/delete/replace chains over a
  seed corpus, asserting `inc == highlight_lines(new)` each step. This is what would
  have caught #285.
- **Forge recall:** the #285 AAR (435358c3) + AD `f62331c9` + PR `c0e30a1e` (the
  windowed-splice equivalence invariant — now shown INCOMPLETE for shrinking edits;
  #288 amends it). Lesson carried: fuzz an equivalence claim, don't hand-pick the
  corpus.
- **Discovery (Design surface):** `crates/syntax/src/lib.rs` — `damage_window`
  (128), `window_extent` (144), `splice_spans` (161), `highlight_incremental` (the
  grow-loop at ~363). Add `cover_edited_cached` next to the other window seams; wire
  one line after `damage_window` in the loop setup. Corpus + perf test in the test
  module (~626 / ~726).
- **Risk:** low-and-gated. The fix only GROWS the window (never shrinks) → the #274
  corpus stays green (a bigger window is still equivalent; the fresh query + splice
  handle it). The perf pin: `cover_edited_cached` is one O(cached) scan (same order
  as splice); a shrinking edit's window grows to the shrunk token's extent (small).
  The differential fuzzer is the proof the fix is COMPLETE, not just repro-patching.

## Phase 2 — Design

### Approach
One pure fn + one wiring line + a durable fuzzer. `cover_edited_cached` widens the
damage window to the surviving new-coord footprint of every cached span the edit's
replaced region touches — the shrink candidates `changed_ranges` never reports.
Only GROWS the window (monotone), so the #274 equivalence is preserved (a bigger
window is still exact: `fresh` supplies the in-window tokens, `splice` keeps the
truly-outside ones). §20 Zed confirmed (same as #285; the fix restores the
byte-identical-to-full equivalence).

```rust
/// Widen `window` (new coords) to cover the surviving new-coord footprint of every cached span
/// (old coords) the edit's replaced region [start, old_end) touches (#288 — the shrinking-edit
/// fix): changed_ranges does NOT report a token whose EXTENT shrank while its surviving bytes are
/// textually unchanged, so without this a pure-deletion tail-trim leaves the shrunk token
/// unqueried and splice drops it. The touch-test `s < old_end && e > start` is empty-window-proof
/// (unlike a window-overlap test, which an empty [p,p) defeats).
fn cover_edited_cached(
    window: Range<usize>,
    cached_old: &[(usize, usize, TokenKind)],
    start: usize,
    old_end: usize,
    new_end: usize,
) -> Range<usize> {
    let mut lo = window.start;
    let mut hi = window.end;
    for &(s, e, _) in cached_old {
        if s < old_end && e > start {
            lo = lo.min(s);
            hi = hi.max(if e >= old_end { e - old_end + new_end } else { new_end });
        }
    }
    lo..hi
}
```

Wired in `highlight_incremental`, one line after `damage_window`:
```rust
let mut window = damage_window(&changed, e.start_byte..e.new_end_byte);
window = cover_edited_cached(window, &self.last_spans, e.start_byte, e.old_end_byte, e.new_end_byte);
let fresh = loop { /* unchanged: spans_in_window → window_extent → break/grow */ };
```
`splice_spans` and `window_extent` are UNCHANGED.

### The differential fuzzer (D4 — the durable fix)
A deterministic seeded LCG (`state = state.wrapping_mul(6364136223846793005).wrapping_add(1)`;
the crate bans `Math.random`/`Date`). For each of ~300 chains: pick a seed source
(a small Rust-ish corpus incl. comments `//`, `/* */`, strings, numbers,
keywords), run `highlight_full`, then ~12 steps — each step picks a char-boundary
position `p` and one of {insert a 1-3 char snippet from a code alphabet, delete
`[p, q)` to the next boundary, replace `[p, q)` with a snippet}, builds `new_src`,
derives `(start, old_end, new_len)`, calls `highlight_incremental`, and
`assert_eq!(inc, highlight_lines(&new_src))` (panicking with the `(src, edit)`
repro on mismatch). ASCII-dominant so every byte is a boundary; a few multibyte
chars seeded in the corpus exercise the boundary guard (which falls back to full —
still correct). Bounded to stay well under the test-time budget (small strings).
This is exactly the shape that found the #285 bug; it must now report ZERO
mismatches.

### File manifest
| File | Change |
|------|--------|
| `crates/syntax/src/lib.rs` | ADD pure `cover_edited_cached(window, cached_old, start, old_end, new_end)`; ADD one line in `highlight_incremental` applying it after `damage_window`. Tests: 3 shrink repros added to `t274_incremental_equals_full_corpus` (or a sibling), a `cover_edited_cached` unit truth-table, and `t288_differential_fuzz`. |

### Regression Test Plan
| # | Test (lib.rs `#[cfg(test)]`) | Proves |
|---|---|---|
| T1 | `cover_edited_cached` truth-table: a comment straddler `(0,9)` under (6,9,6) → window `[6,6)`→`[0,6)`; an int `(0,2)` under (1,8,2) with window `[1,3)`→`[0,3)`; a span ending INSIDE the replacement (`e < old_end`) → `hi` uses `new_end`; a span NOT touching → window unchanged; a span after `old_end` → shifted `hi`. | REQ-005 |
| T2 | Corpus rows added: `"// abcdef\nfn f(){}\n"`→`"// abc\nfn f(){}\n"` (comment tail-trim); a string tail-trim; `"let x = 42;\n"`→`"let x = 4;\n"` (int re-tokenize). Assert `inc == highlight_lines(new)`. | REQ-001/002 |
| T3 | `t288_differential_fuzz` — deterministic LCG, ~300 chains × ~12 steps, `assert_eq!(inc, fresh)` each step, panic-with-repro on mismatch. | REQ-003 |
| T4 | Existing `t274_incremental_equals_full_corpus` + `t274_perf_ratio_pins` stay green. | REQ-004 |
| — | cov/MSI 100 on `cover_edited_cached` via `--diff`. | REQ-005 |

Uncoverable: none. Library crate — no live-drive (the vanished-highlight bug IS visible in the app, but the corpus + fuzzer prove the per-line output; a driven capture would need a pre-existing tracked file opened + a backspace-at-comment-end, env-blocked per #275 — the units are the authority and STRONGER than one capture).

### Risks / decisions
- **R1 (fuzzer budget/flake):** the fuzzer must stay deterministic (seeded LCG, no wall-clock) and bounded (small strings, ~300×12) so it's fast + never flaky. If a mismatch is found, it panics with the exact `(src, start, old_end, new_len)` for a permanent regression row.
- **R2 (window over-grow):** `cover_edited_cached` can only GROW the window → still equivalent (never drops a needed token); worst case a bigger fresh query (bounded by the touched tokens' extents). The perf pin re-confirms.
- **D-amend:** PR `c0e30a1e` (the #285 equivalence invariant) is amended — the window must ALSO cover the surviving footprint of edited cached spans, not just `changed_ranges ∪ edit`. Captured in Phase 5.

## Phase 3 — Implement
- **Built to the manifest, no deviations.** Added pure `cover_edited_cached(window, cached_old, start, old_end, new_end)` (after `splice_spans`); wired one line in `highlight_incremental` after `damage_window` (`window = cover_edited_cached(window, last_spans, e.start_byte, e.old_end_byte, e.new_end_byte)`). `splice_spans`/`window_extent`/`damage_window` UNCHANGED.
- **PROOF the fix is complete (ran the #285 critic's own harness against the new code — the tool that FOUND the bug):**
  - `minimal` repro binary: ALL cases now **MATCH** — the previously-DIFFERING BUG 1 (`"// abcdef"`→`"// abc"` comment tail-trim) and BUG 2 (`"42 /*{i42"`→`"4X2"`) + string tail-trims + doc-comment boundary edits.
  - `fuzzcrit` full differential fuzzer (previously **40+ mismatches / 300 seeds** across random_chain / array-of-literals / nested-block-comment / oscillating-cascade / doc-comment-boundary strategies): now **"ALL OK — no incremental/full divergence found."**
- Existing `marley_syntax` tests 21/21 green (the #274 corpus + perf pin unaffected — the fix only GROWS the window). `cargo check` clean; `cargo fmt --check` clean; `cargo clippy --all-targets` clean.
- Phase 4 will re-create the fuzzer as a PERMANENT in-test regression (`t288_differential_fuzz`) + the 3 minimal repro rows (the scratchpad harness is ephemeral).

## Phase 3.5 — Inspect
A background adversarial critic is extending the fuzzer with new strategies (start-inside-replacement, merges, oscillating chains, multibyte tails) — **reconciled before /commit** (per #285's lesson that a fuzzer catches what a trace misses). Primary = the empirical fuzzer result + an inline trace. **No defects.**

| Angle | Verdict | Evidence |
|---|---|---|
| **Empirical (the strongest check)** | **SAFE** | The #285 critic's OWN fuzzer — the tool that found the bug, 40+ mismatches / 300 seeds across 5 strategies — now reports "ALL OK — no divergence." All minimal repros MATCH. |
| **Touch-test completeness** | **SAFE** | `s < old_end && e > start` = the span intersects `[start, old_end)` = exactly "touches the replaced region" = every stale cached span. A span NOT touching it is unchanged content, correctly rebased by `splice`. |
| **Footprint arithmetic** | **SAFE** | Prefix survives at `s` → `lo=min(lo,s)`; suffix (`e >= old_end`) shifts to `e-old_end+new_end` → `hi` covers it; a span ending INSIDE the replacement (`e < old_end`) → `hi=new_end` (its prefix abuts the new end); a span starting inside (`s >= start`) → `lo` unchanged (`lo<=start<=s`). Traced a straddler `(2,12)` under `(5,8,6)` → footprint `[2,10)` = the reformed token's extent. |
| **Token-grow vs shrink** | **SAFE** | `cover_edited_cached` covers SHRINKS (footprint ≤ old extent); a token that GROWS beyond it is a fresh straddler caught by the UNCHANGED `window_extent` grow-loop. The two are complementary and together complete. |
| **Merge (deletion joins two tokens)** | **SAFE** | Both merged tokens touch the replaced region (the boundary between them was replaced) → both covered → `lo=min` of both starts covers the merged left edge. |
| **Run-once sufficiency** | **SAFE** | `cover_edited_cached` depends on the FIXED `[start,old_end)`, not the window, so growing the window later never exposes a new "touching" span — one pass before the grow-loop suffices. |
| **No new bug / #274 regression** | **SAFE** | Only GROWS the window → never drops a needed token; the #274 corpus + perf pin stay green (a bigger window is still equivalent, bounded). |
| **MSI** | **Phase-4-gated** | Viable mutants = the touch-test `<`/`>`, the `e >= old_end` branch, the `e-old_end+new_end` arithmetic (`min`/`max` are unmutated method calls). T1 truth-table will kill each at the boundaries (`s==old_end`, `e==start`, `e==old_end`). |

## Phase 4 — Validate
**The fuzzer earned its keep — it drove the fix to COMPLETENESS through two refinements the inline trace + the first critic missed (the #285 lesson, applied and vindicated):**
- Writing `t288_differential_fuzz` immediately caught that the Phase-3 fix (overlap-only `cover_edited_cached`) was INCOMPLETE: a block-comment `*/` deletion makes the following text swallow into the comment; the new integer at `changed_ranges.end` (exclusive) sat just outside the byte-tight window. **Diagnosed** (scratchpad `diag288`, comparing trees): `TREES MATCH` → a window bug, not tree divergence. Fix 1: the touch-test → CLOSED interval `s <= old_end && e >= start` (covers ADJACENT re-tokenization).
- A beefed-up scratch fuzz (120k steps) then found 2 more: one Class-A (a same-line token re-tokenizing at `changed_ranges.end`) and one Class-B (`TREES DIFFER` — tree-sitter's incremental parser genuinely diverging from a fresh parse on error input). Fix 2: **`snap_to_lines`** — snap the window to whole-line bounds (covers same-line boundary re-tokenization).
- **Final proof:** the tree-guarded scratch fuzz — **120,000 steps → 0 Class-A window bugs, 1 Class-B tree-divergence** (tree-sitter's, not this crate's). The background adversarial critic independently confirmed (~50k comparisons / 30 runs, incl. 5 new strategies: deep-deletion, merge/split, multibyte-tail, insertion-split, shrink-biased-chains): all clean against the current code; it reproduced the SAME block-comment class against the ORIGINAL and verified `snap_to_lines` fixes it.
- **Class B (tree divergence) is scoped OUT, correctly:** this crate's windowing contract is "SAME tree ⇒ same spans"; tree-sitter's incremental parser can itself differ from a fresh parse on error-recovery inputs (rare, ~1/100k on random garbage, pre-existing since #274). So `t288_differential_fuzz` asserts equivalence ONLY when the incremental tree matches a fresh parse — making it non-flaky AND correctly-scoped to MY code — plus a sanity assert that ≥90% of steps ARE tree-matched (the guard isn't hiding everything).
- **Tests added:** `t288_cover_edited_cached_widens_to_shrunk_footprint` (T1, overlap+abut+strictly-outside boundaries), `t288_snap_to_lines_extends_to_line_bounds` (T1b — DIRECT range asserts; an off-by-one just widens the window → identical output, so only exact bounds kill the `i`/`i+1` mutants), 3 shrink corpus rows (T2), `t288_differential_fuzz` (T3, tree-guarded, ~2880 steps).
- `cargo nextest run -p marley_syntax`: **24 passed**; the #274 corpus + perf pin stay green (the wider window is bounded → pin holds).
- **`scripts/gates.sh --diff` → `GATE GREEN [diff]`** — 15/15 incl. **coverage 100%** + **MSI 100%** (`cover_edited_cached` touch-test/arithmetic + `snap_to_lines` bounds all killed). Two source-fixes to hit the gate (§0, no suppressions): the `if e >= old_end` branch → `e.saturating_sub(old_end) + new_end` (branchless, kills the equivalent `>=`/`>` mutant); the tree-guard flattened to line-covered statements (a multi-line `assert_eq!` inside an `if` leaves the block's `}` in a cold macro region).
- Library crate — no live-drive (the vanished-highlight IS app-visible, but the equivalence corpus + the tree-guarded fuzzer prove the per-line output far more thoroughly than one capture could; env-blocked per #275).

## Inspect (Phase 3.5) — AMENDED
The Phase-3.5 inline trace + first critic pass judged the (overlap-only) fix complete. **The Phase-4 fuzzer proved that judgment WRONG — twice** (adjacent re-tokenization, then same-line boundary). Honest correction: for a subtle equivalence invariant, a DIFFERENTIAL FUZZER is the authority, not a trace — recorded as the #288 prevention rule. The fix was completed in-phase (validate fixes gate reds at source); the final fuzzer + critic + gate are all green.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Fixed (#288); docs/marley_architecture/editor.md incremental-highlight section extended with the M17 #288 shrinking-edit fix (cover_edited_cached overlap+abut, snap_to_lines, the tree-match-scoped equivalence, the differential fuzzer).
- **Knowledge (forge):** failure `BF-shrinking-edit-window-drops-token-001` (`028f56d0`, runtime/critical — the #285 regression + why it shipped + how it was found); prevention rule `PR-claude-fuzz-the-equivalence-claim-001` (`bd416c08`, high — the durable meta-lesson: fuzz an equivalence claim, an inline trace + hand-picked corpus miss whole classes; guard on the shared precondition when the reference is non-deterministic). AAR `ec411627` submitted: completed, effectiveness 5. This AMENDS the #285 PR `c0e30a1e` (the windowed-splice invariant) — the window must ALSO cover overlapping+abutting cached spans and snap to lines, and equivalence holds only per-tree.
- **Ticket** TICKET-288 → closed/ (status closed) + forge ticket-close. **Pipeline** spec+notes → completed/.
- **Result:** the CRITICAL #285 regression is fixed; incremental highlighting is byte-identical to a full walk whenever the incremental tree matches a fresh parse, proven by a permanent tree-guarded differential fuzzer (0 same-tree divergences / 120k adversarial steps) + the independent critic (~50k comparisons). cov/MSI 100, GATE GREEN [diff]. LOCAL commit only (push un-OK'd).
