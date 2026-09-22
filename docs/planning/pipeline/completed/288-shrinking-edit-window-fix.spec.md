---
pipeline_id: 06c2566a-d919-4571-86fb-4af95d69bef1
ticket: forge#288 (ce37cfba-3d91-45da-8514-d6f3dd91a4f0) · local docs/planning/tickets/open/TICKET-288-shrinking-edit-window-fix.md
aar_id: ec411627-3f7c-4507-bfe4-e28543c03f74
status: Phase 5 — Complete PASS
title: Fix the #285 shrinking-edit windowing regression (cover the shrunk token)
type: bug
milestone: M17
references: []
---

## Title
Fix the #285 shrinking-edit windowing regression. A pure-deletion tail-trim of a
comment (or a token re-tokenizing shorter) leaves the surviving token OUTSIDE the
damage window and unqueried, so `splice_spans` drops the stale cached span and
nothing replaces it — the highlight vanishes. Widen the damage window to the
surviving new-coord footprint of every cached span the edit's replaced region
touches, and add a bounded differential fuzzer so the corpus gap can't recur.

## Scope
### In
- `marley_syntax`: a pure `cover_edited_cached(window, cached_old, start, old_end, new_end)` that
  widens the window to cover the surviving new-coord footprint of every cached span TOUCHING the
  replaced region `[start, old_end)` (the shrink candidates `changed_ranges` never reports).
- One line in `highlight_incremental`: apply it after `damage_window`, before the fresh grow-loop.
- Regression corpus: the fuzzer's minimal repros (comment tail-trim, string tail-trim,
  identifier→shorter-integer) + a bounded IN-TEST differential fuzzer.

### Out (explicitly deferred)
- `splice_spans` and `window_extent` are UNCHANGED (they were correct; only the window was too small).
- The line-cache O(damage) tail (still the #285 deferred follow-up).

## Reference (§20)
Zed (the editor) — incremental tree-sitter syntax highlighting, same as #285. The
fix RESTORES the behavior #285 targeted: the windowed incremental result is
byte-identical to a fresh full highlight (REQ-001), now including shrinking edits.
Clean-room §20: tree-sitter public API only; the window/footprint arithmetic is
Marley's own; Zed's GPL source is never read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — `cover_edited_cached` widens to the SURVIVING footprint.** For each cached
  span `(s, e)` with `s < old_end && e > start` (touches the replaced region):
  `lo = min(lo, s)` (the unchanged prefix survives at `s`); `hi = max(hi, e >=
  old_end ? e - old_end + new_end : new_end)` (the shifted suffix, or the edit
  point if the span ends inside the replacement). Computed ONCE — the touch-test is
  window-independent — before the fresh grow-loop.
- **D2 — the OLD-region touch-test, not window-overlap.** The ticket's first
  `extent_over_overlapping` sketch grew over spans OVERLAPPING the window; an EMPTY
  window `[p, p)` defeats that (nothing overlaps an empty range unless it strictly
  straddles `p`, and a token ending exactly at `p` — the shrunk tail — is missed).
  `s < old_end && e > start` is empty-window-proof.
- **D3 — `splice_spans` / `window_extent` UNCHANGED.** `splice` already drops a
  straddler of `[start, old_end)` (its `else continue`) and any window-overlapping
  span; `cover_edited_cached` only guarantees the window COVERS the shrunk token's
  replacement so the fresh query supplies it.
- **D4 — a deterministic in-test differential fuzzer.** A tiny seeded LCG (no
  `Math.random`/`Date` — the crate bans them) drives random insert/delete/replace
  edit chains over a seed corpus, asserting `inc == highlight_lines(new)` each step.
  Bounded to stay under the test budget; this is the durable fix for the #285 gap.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a pure-deletion tail-trim shrinks a comment or string token, the incremental highlight shall be byte-identical to a fresh full highlight. | Corpus rows: `"// abcdef"→"// abc"`, `"…\"abcdef\""→"…\"abc\""`. |
| REQ-002 | WHEN a token re-tokenizes to a shorter node keeping its prefix (`"42"→"4"`), the incremental highlight shall equal a fresh full highlight. | Corpus row. |
| REQ-003 | A bounded differential fuzzer over random edit chains shall find ZERO `inc != fresh` mismatches. | `t288_differential_fuzz` (deterministic LCG, ~hundreds of chains). |
| REQ-004 | The #274 equivalence corpus and the perf ratio pin shall remain green (no regression from the wider window). | `t274_incremental_equals_full_corpus` + `t274_perf_ratio_pins`. |
| REQ-005 | `cover_edited_cached` shall be a pure function at 100% line coverage and MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `cover_edited_cached` signature + the `highlight_incremental` wiring; the fuzzer harness; regression test plan.
- **P3 Implement** — the pure fn + the one-line wire.
- **P3.5 Inspect** — re-run the differential fuzzer adversarially (the fix must survive what broke #285); trace the footprint arithmetic + the growing-window interaction.
- **P4 Validate** — the repros + the fuzzer + the corpus + the perf pin; gate green [diff].
- **P5 Complete** — CHANGELOG + editor.md, AAR + a prevention rule (the shrinking-edit class + fuzz-the-equivalence rule), close #288.
