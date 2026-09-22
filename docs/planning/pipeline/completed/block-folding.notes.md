---
pipeline_id: d023ffc5-1242-4669-baae-6812efc4fcfd
ticket: forge#184 (eaa3b75a-ede5-43c3-a5c2-c447f867bcb4)
aar_id: 6935d5e5-fe5d-44e6-a0ea-fb8c98f75c7a
---

# Notes — M12 #184 block folding

## Phase 1 — Plan / Phase 2 — Design (folded)
**Approach.** A folded block's OUTPUT rows genuinely disappear (Warp behavior + the ticket's "fold_visible_rows
-> the rows to emit") — so the render, content_rows, and content_row_texts all walk via the SAME fold-aware
sequence, keeping Selection/viewport row indices aligned (the #175/#47 invariant). Pure model in nav.rs
(alongside block_at_row/#52 boundaries — same block_line_counts shape). Per-pane FoldState on TerminalPane
(beside viewport/selection).

**Pure surface.**
- `enum RowKind { Header(usize), Output(usize, usize) }` (block index; block+line index) — what fold_visible_rows emits.
- `FoldState { folded: BTreeSet<usize> }`: `toggle`, `is_folded`, `retain_below(len)`, `fold_visible_rows(counts, &self)`.
  Header ALWAYS pushed; Output(bi, li) pushed for li in 0..count ONLY when !is_folded(bi).

**File manifest.**
- `nav.rs` — add RowKind + FoldState + fold_visible_rows (+ tests in P4).
- `workspace.rs` — TerminalPane gains `pub folds: FoldState` (default empty); init in new().
- `app.rs` (SHIM) — content_rows/content_row_texts walk the fold (a folded block contributes header only);
  the block render draws the ▾/▸ chevron (a click toggles state.folds + notify) and skips a folded block's
  output rows; retain_below(blocks().len()) called in refresh so stale folds reset on reshape.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `fold_toggle_membership` | REQ-001 — toggle adds then removes; is_folded reflects it |
| `fold_visible_rows_hides_output` | REQ-002 — headers always; a folded block emits header only; others full (kills the membership + header-always) |
| `fold_retain_below_resets` | REQ-003 — retain_below drops >= len, keeps < len |
| driven | REQ-004 — chevron click hides output, click restores |
| gate --diff (staged) | REQ-005 |

**Risk.** The viewport/selection row indices must be computed over the SAME folded sequence the render walks —
route content_rows + content_row_texts through fold_visible_rows so all three agree (else copy/selection mis-map
when a block is folded). retain_below timing: apply it where refresh_agent_statuses runs (every pump) so a
re-run's reshaped block list clears folds for vanished indices. BTreeSet for a deterministic, mutation-stable set.

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean (only the pre-existing `block v0.1.6` warning).
- `nav.rs` (PURE) — `RowKind { Header(usize), Output(usize,usize) }`; `FoldState { folded: BTreeSet<usize> }`
  with `new`/`is_folded`/`toggle` (remove-or-insert)/`retain_below(len)`; `fold_visible_rows(counts, &folds)`
  emitting Header always + Output only when unfolded.
- `workspace.rs` — `TerminalPane.folds: FoldState` (+ import), init `FoldState::new()` in new().
- `app.rs` (SHIM) — imported `fold_visible_rows`; new `block_line_counts(state)` helper; `content_rows` now
  `fold_visible_rows(&counts, &folds).len() + 1` (the ONE pure source of row count); `content_row_texts` skips
  a folded block's output (same `is_folded` gate); the block render draws the ▾/▸ chevron as the header's first
  child (a click → `toggle_block_fold(pane, i, cx)` + notify) and iterates an empty slice for a folded block's
  output so `row` doesn't advance (render/content_rows/content_row_texts in lockstep); `retain_below(blocks
  ().len())` called in the pump's Ok-events arm so a reshaped block list drops stale folds.

**Deviation:** the folded-output loop uses `&[][..]` (an empty StyledLine slice) rather than naming the type —
`output_styled() -> &[StyledLine]` and the empty-slice branch infers cleanly from the else. Behaviorally: a
folded block emits its header only, `row` advances by exactly 1, matching the pure fold_visible_rows.
## Inspect (Phase 3.5)
2 parallel general-purpose critics (row-index-invariant/correctness + fold-model-mutation/state). Both ran the
build + `cargo mutants --list`; both landed hard, high-value findings I'd missed. Findings + verdicts:

- **[CRITICAL] `content_rows` lost its `mutants::skip`** — my `block_line_counts` insertion landed BETWEEN the
  pre-existing skip attribute and `content_rows`, so the attr (+ a stale doc + a duplicate skip) all shielded
  `block_line_counts`, leaving `content_rows` unskipped → 4 unkillable mutants → MSI RED. REAL (mutation critic,
  `cargo mutants --list` confirmed). FIXED: reordered so each fn owns its doc + skip; verified both now carry it.
  → failure-record + prevention rule (the attribute-detaches-on-insertion class).
- **[HIGH] Right-click block menu targets the WRONG block after a fold** — line 4065 fed a fold-aware `row` to
  `block_at_row` over RAW counts → wrong block (or a phantom block on the prompt row). REAL (row critic, proof-
  executed; I'd flagged it too). FIXED: added pure `fold_block_at_row(counts, folds, row)` (indexes
  fold_visible_rows, reads the block off RowKind); the menu calls it now.
- **[MED] ⌘↑/↓ jump lands wrong after a fold** — line 967 used raw `block_boundary_rows` vs a fold-aware
  viewport `top`. REAL (both; I'd flagged it). FIXED: added pure `fold_boundary_rows(counts, folds)` (the Header
  row positions in fold_visible_rows); the jump calls it.
- **[MED] Fold leaked into agent OBSERVATION** — `content_row_texts` (now fold-aware) feeds `agent_tail`/
  `agent_last_line`, so a human folding a block in an agent's pane blanked its Fleet line / cockpit tail (a view
  toggle corrupting observation — the memory flags agent-observe as load-bearing). REAL (row critic). FIXED:
  added `observed_row_texts` (raw, fold-INDEPENDENT, all blocks' output) and repointed both observation reads to
  it; copy/find/selection keep the fold-aware `content_row_texts` (they follow visible rows). → prevention rule.
- **[MED] "single source" overclaim / drift** — fold_visible_rows was only walked by content_rows; the render +
  content_row_texts mirror the `is_folded` gate inline. FIXED (partial): fold_block_at_row/fold_boundary_rows now
  ALSO derive from fold_visible_rows (3 pure consumers), and the doc was corrected to state content_rows/the two
  fold-aware fns derive from it while the render+copy mirror the gate. Full RowKind-match refactor of the render
  is a bigger change than warranted; the row-invariant is now protected by the tested fold-aware helpers.
- Superseded `block_at_row`/`block_boundary_rows` (raw) + their tests REMOVED — the fold-aware versions are
  strict generalizations (empty FoldState == the old behavior); their unfolded cases move into the #184 fold
  tests (Phase 4). Clippy `-D warnings` (dead code) confirmed the removal was needed.
- **[MED — Phase 4] Pure fold code has no tests yet** — deferred to validate (correct). The mutation critic
  enumerated the exact kill-tests: is_folded true+false; toggle with the LOAD-BEARING intermediate assert (one
  toggle flips false→true — a round-trip-only test does NOT kill the no-op mutant, the #181 family); retain_below
  with an index EXACTLY == len as the `<`-vs-`<=` witness; fold_visible_rows with one folded + one unfolded
  block (both count ≥ 1). Recorded verbatim for Phase 4.

Lenses: row-index invariant (every block-row walker), correctness/edge, mutation-killability (full enumeration +
equivalent-mutant check), state integrity (BTreeSet/Default/persistence/construction), agent-observe coupling,
simplification/drift, clippy/fmt/secrets. Post-fix: `cargo check --workspace` clean (no warnings),
`cargo mutants --list` surface all killable (no equivalent traps).
## Phase 4 — Validate
**Tests added** (nav.rs, the mutation critic's exact kill-fixtures): fold_is_folded_reflects_membership;
fold_toggle_flips_and_round_trips (the LOAD-BEARING intermediate assert — one toggle flips false→true, kills
the no-op mutants the #181-family round-trip-only test would miss); fold_retain_below_resets_stale (index ==
len as the `<`-vs-`<=` witness); fold_visible_rows_hides_folded_output; rowkind_block_reads_index;
fold_block_at_row_maps_folded_and_unfolded (absorbs the old block_at_row cases + the folded regression the
inspect critic proved); fold_boundary_rows_tracks_folds.
`cargo mutants -f nav.rs` → **28 mutants, 27 caught, 1 unviable, 0 missed** (MSI 100). No equivalent-mutant traps.
**cargo nextest run --workspace**: 758 passed, 5 skipped (in-transcript).

**Driven live-app capture (REQ-004)** — minted 2 blocks (`echo aaa184` / `echo bbb184`, each `▾ ✓` + output).
Clicked block 1's ▾ chevron → it became **`▸ ✓ echo aaa184` with its output row HIDDEN** (folded, header + ✓
stay). Clicked the same block's ▸ chevron again → **`▾ ✓ echo aaa184` with `aaa184` output RESTORED** — a clean
fold→unfold round-trip on one block (/tmp/mly184_after.png → _unfold.png). Chevron renders on every header.
(Harness note: repeated imprecise clicks grazed the ↻ run affordance and added stray rerun blocks — a self-test
precision limit, NOT a code fault; the fold render + toggle were correct in every capture.)

**Gate**: `scripts/gates.sh --diff` (staged, so the new nav.rs lines are mutation-tested) → **GATE GREEN [diff]**,
15/15. gate:4 coverage 100%; gate:5 mutation 18/18 → MSI 100.0%. First run RED on gate:1 (rustfmt — a fold-test
comment's indentation drifted after my post-fmt edits); fixed with `cargo fmt`, re-ran green. (Streamlined-flow
reminder: run `cargo fmt` immediately before staging.)

**Pre-existing exclusions**: none. (Restored ~/.marley/config/settings.toml from the pre-test backup.)
## Phase 5 — Complete
(pending)
