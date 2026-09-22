# TICKET-329 — Expand/shrink selection: tree-sitter node ancestry + marley_syntax's first node-range API

- **Forge ticket:** #329 78e4ae26-a46d-4fdf-815e-229d738faede (feature, M21)
- **Owner:** claude (this session)
- **AAR:** 4cb6a9a2-12a7-4dd8-8259-c5446654dd4f
- **Pipeline doc:** ../../pipeline/active/329-expand-selection.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331)
- **Status:** closed

## Summary
Grow the selection identifier→call→statement→block→fn with one chord; shrink walks back to the ORIGINAL. The
heart is a pure tree-sitter ancestry walk, but it doubles as the FOUNDATION: no tree is reachable today
(marley_syntax exposes highlight spans only; the `tree_sitter::Tree` is private to `HighlightSession` on the
worker thread). This ships `enclosing_ranges(&HighlightSession, byte_range) -> Vec<Range<usize>>` — the pure
ancestry ladder (smallest named node → root, deduped) that #305 fold + #330 sticky header reuse — plus a pure
`SelectionLadder` (grow up / shrink to the remembered anchor, invalidated by edit/caret-move) and an
Editor-scoped chord (⌃W/⌃⇧W leading). Delivery is a hybrid: large files round-trip the worker (parked one
tick, the #313 precedent); small files (which keep no retained tree) build a throwaway session synchronously.
Byte↔char through the rope (emoji fixture mandatory).

## Acceptance
`enclosing_ranges` at cov/MSI 100 (identifier→call→statement→block→fn on real Rust; identical-range dedupe;
root top-out; no-tree→empty; empty-selection→smallest node). Byte↔char exact across an emoji line.
`SelectionLadder` grow/shrink-to-original/invalidate table. Headless: chord→(park→worker | sync)→grow through 3
rungs; shrink returns EXACTLY to the original; an edit mid-ladder invalidates. Multi-cursor v1: primary grows,
others collapse. Live drive env-blocked (screen locked) → units+mechanism. Full EARS in the pipeline spec.
