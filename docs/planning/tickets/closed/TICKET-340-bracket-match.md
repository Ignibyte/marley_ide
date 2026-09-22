# TICKET-340 — Bracket-match highlight: the caret's delimiter pair lit + ⌘⇧\ Go to Bracket

- **Forge ticket:** #340 b35af9e6-04c1-47a6-91af-f1614a2581fe (feature, M22)
- **Owner:** claude (this session)
- **AAR:** d6b81d92-45c1-4f2c-bf64-9473de807e98
- **Pipeline doc:** ../../pipeline/completed/340-bracket-match.spec.md
- **Source ticket:** M22 "The editing bar" batch (#336–340 + the 11 that follow) — the FIFTH and LAST of the
  goal `/work 336-340` ([m22-editing-bar.md](../../design-notes/m22-editing-bar.md) ·
  [roadmap.md](../../../marley_architecture/roadmap.md))
- **Status:** closed (SHIPPED — gate GREEN [diff], local commit; the LAST of `/work 336-340`)

## Summary
Put the caret on a `(` and light BOTH delimiters of the pair — the quiet affordance every editor has — plus
**⌘⇧\ Go to Bracket**. Rust-only v1, with the tree-sitter tree as the truth source so a `(` inside a string
or comment never false-positives.

**Phase 1 found the load-bearing cadence claim FALSE.** The spec says "recompute per caret move on the CACHED
tree, never a reparse per frame". There is **no cached tree** on the app side — the live tree lives only on
the syntax worker thread, and the #329/#330 node-range callers **reparse a throwaway `HighlightSession` (a
full whole-buffer `highlight_full`) on every caret move**. So following that route, bracket-match would
reparse the entire file on every arrow key. The public seams all exist and `matching_delimiters` is a viable
thin adapter over tree-sitter's node API — but the performance story has to be re-taken (Design Fork A).

## Acceptance
The caret on/adjacent-to a delimiter lights both halves of its pair (tree-derived, so no string/comment false
positives); ⌘⇧\ jumps between them; primary cursor only; recompute keyed on (nonce, version, caret); OFF/no-
tree is byte-identical. Full EARS REQ-001..008 in the pipeline spec — the **cadence** REQ is the one Phase 1
reshaped.
