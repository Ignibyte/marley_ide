---
pipeline_id: 14662b89-19bb-4ef6-9093-3ea2c887233c
ticket: forge#285 (e6210a61-79e6-4e65-90fa-2748332fa89e) · local docs/planning/tickets/open/TICKET-285-query-walk-windowing.md
aar_id: 435358c3-3dd6-4467-b0b1-16739213870b
status: Phase 5 — Complete PASS
title: Window the tree-sitter highlight query walk to the damage span
type: feature
milestone: M17
references: []
---

## Title
Window the highlight query walk to the damage span. Today the incremental
highlight path re-parses the tree cheaply (~1.0ms) but then runs the SAME
full-tree `QueryCursor` walk (`spans_from_tree`) at ~5.7ms on every keystroke —
flooring end-to-end incremental at 0.46 of a full highlight. Window that walk to
the edit's damage region and splice the fresh in-window spans into the cached
out-of-window spans, staying byte-identical to a fresh full highlight.

## Scope
### In
- `marley_syntax`: a pure `damage_window` (merge `changed_ranges` ∪ edit span) +
  a pure `splice_spans` (rebase cached out-of-window spans by the edit delta,
  drop window-overlapping cached spans, union the fresh in-window spans).
- A windowed query pass in `parse.rs` (`QueryCursor::set_byte_range`).
- `HighlightSession` caches the previous disjoint span set; `highlight_incremental`
  uses `changed_ranges` + the windowed pass + the splice instead of the full walk.
- Tighten the #274 perf pin: end-to-end incremental < 1/3 of a full highlight.

### Out (explicitly deferred)
- Windowing the `span→lines` tail (reuse cached per-line vectors). The measured
  tail is ~0.3ms, so span-windowing alone should clear <1/3; add line-windowing
  ONLY if the pin fails (D1).
- Other grammars, multi-file sessions, the async/threading path (unchanged from #274).

## Reference (§20)
Zed (the editor) — incremental tree-sitter syntax highlighting. Observed
behavior: as you type, highlighting updates without a visible full-file re-flash
and stays consistent with what a full re-highlight would produce. Marley matches
that BEHAVIOR — the windowed incremental result is byte-identical to a fresh full
highlight (REQ-001) — using tree-sitter's PUBLIC API (`Tree::changed_ranges`,
`QueryCursor::set_byte_range`). Clean-room §20: behavior observed only; the damage
merge + splice/rebase arithmetic is Marley's own; Zed's GPL source is never read
or translated.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — Splice at the SPAN level; keep `lines_from_spans` on the full spliced set.**
  Identical span multiset ⇒ identical per-line output, so line equivalence is
  guaranteed by construction and ALL equivalence risk concentrates in the span
  splice (gated hard by the corpus). Windowing the `span→lines` tail is deferred
  (Out) unless the perf pin needs it.
- **D2 — Damage window (NEW-tree byte coords) = `min(Δ.start, edit.start) ..
  max(Δ.end, edit.new_end)`** over `Δ = changed_ranges`. Unioning the edit span
  guarantees the window covers the literal edit even when the grammar reports no
  structural change; an empty `Δ` ⇒ the window is just the edit span.
- **D3 — Out-of-window cached spans are rebased by the edit delta** (before
  `edit.start`: unchanged; at/after `edit.old_end`: shifted by `new_end-old_end`);
  spans OVERLAPPING the window are dropped (the fresh query supplies them).
  Sound because `changed_ranges ⊆ window`, so every out-of-window span covers
  unchanged content.
- **D4 — `set_byte_range` returns matches INTERSECTING the window** (tree-sitter
  contract). A token straddling the window edge comes from the fresh query with
  its FULL extent; its rebased cached twin overlaps the window and is dropped — no
  duplication, no truncation.
- **D5 — The cascade case** (block-comment open at BOF → `changed_ranges` spans
  ~the whole file) makes the window ≈ full-file ⇒ the windowed query IS the full
  query ⇒ trivially equivalent, no perf win but no lost regions.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a single-delta incremental highlight runs through the windowed path, the system shall produce per-line highlights byte-identical to a fresh full highlight of the same source. | The #274 equivalence corpus `t274_incremental_equals_full_corpus` (expanded with window-boundary + straddle cases), GATE GREEN. |
| REQ-002 | WHERE the damage window is a strict sub-range of the file, the system shall execute the highlight query over only that window (not the whole tree). | Unit test on `damage_window`/`splice_spans` + the perf pin (a full-file walk cannot meet <1/3). |
| REQ-003 | WHEN an edit makes `changed_ranges` span most of the file (block-comment cascade), the windowing shall degrade to a ~full-file query and remain exactly equivalent. | The corpus cascade cases (block-comment open at BOF + the 10-step chain). |
| REQ-004 | The end-to-end incremental highlight shall complete in under 1/3 of a full highlight on the ~8k-line fixture. | `t274_perf_ratio_pins` tightened to `inc_e2e < full_e2e / 3`. |
| REQ-005 | The damage-window merge and the span splice/rebase shall be pure functions at 100% line coverage and MSI 100. | `scripts/gates.sh --diff` (coverage + mutation on changed files). |

## Phase Plan
- **P2 Design** — the `damage_window` + `splice_spans` signatures + the `parse.rs`
  windowed pass + the `HighlightSession.last_spans` cache; the regression test plan
  (corpus expansion + splice unit truth-table + the tightened perf pin).
- **P3 Implement** — the two pure fns, the windowed query pass, wire
  `highlight_incremental`.
- **P3.5 Inspect** — critics/inline vs the diff: straddle boundaries, the cascade,
  span rebasing across the edit (#269), the sweep at the window edge.
- **P4 Validate** — expand + RUN the corpus, the splice truth-table, tighten +
  RUN the perf pin; gate green [diff].
- **P5 Complete** — CHANGELOG + editor.md, AAR capture, close #285.
