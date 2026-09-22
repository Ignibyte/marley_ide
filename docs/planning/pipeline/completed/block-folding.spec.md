---
pipeline_id: d023ffc5-1242-4669-baae-6812efc4fcfd
ticket: forge#184 (eaa3b75a-ede5-43c3-a5c2-c447f867bcb4) · local docs/planning/tickets/open/TICKET-184-block-folding.md
aar_id: 6935d5e5-fe5d-44e6-a0ea-fb8c98f75c7a
status: Phase 5 — Complete PASS
title: M12 — block folding: collapse/expand a command block's output
type: feature
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/nav.rs (PURE: FoldState + fold_visible_rows)
  - crates/marley_app/src/workspace.rs (TerminalPane gains a fold field)
  - crates/marley_app/src/app.rs (SHIM: the chevron + header-click toggle + the row-skip render)
---

## Title
Warp fidelity — fold a command block's output to its header. A chevron on each block header (▾ open / ▸
folded) toggles the block's OUTPUT rows hidden; the header + exit glyph stay. Per-block, keyed by block index.

## Scope
### In
- PURE `nav.rs`: `FoldState { folded: BTreeSet<usize> }` with `toggle(i)`, `is_folded(i)`, and
  `retain_below(len)` (drop indices >= a new block count, so fold state resets as the block list reshapes);
  `fold_visible_rows(block_line_counts, &folded) -> Vec<RowKind>` — the ordered rows the render emits: each
  block's Header ALWAYS, then its output rows ONLY when that block is unfolded (Prompt handled by the shim).
- SHIM `app.rs`: each block's per-pane fold is read from a new `TerminalPane.folds: FoldState`; the header
  draws a ▾/▸ chevron whose click toggles that block (and repaints); the block render + `content_rows`/
  `content_row_texts` consult the fold so a folded block's output rows vanish (the viewport shrinks with them).
- `retain_below` is applied each pump/refresh so a re-run / cleared blocks reset stale folds.

### Out
- ⌘. fold-focused-block (the ticket marks it optional — a follow-up; the chevron click is the core).
- Persisting fold state across a reboot; folding the prompt; a fold-all affordance.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `FoldState` toggle/is_folded shall add/remove a block index from the fold set (membership). | unit + mutation |
| REQ-002 | `fold_visible_rows` shall emit every block's header, and a block's output rows ONLY when it is unfolded. | unit + mutation |
| REQ-003 | `retain_below(len)` shall drop fold indices >= len (reset on a reshaped block list). | unit + mutation |
| REQ-004 (visual) | WHEN a block header's chevron is clicked, that block's output rows shall hide (header stays); clicking again restores them. | driven capture |
| REQ-005 | gate GREEN; the pure model cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the pure model + the shim (fold field + chevron + fold-aware row walk). P3.5 1-2 critics (the
row-emit filter vs #175 block_at_row consistency; the viewport/selection row-index alignment when folded; the
retain_below reset timing; membership mutation). P4 unit + driven + gate (--diff, staged). P5 docs.
