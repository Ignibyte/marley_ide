# TICKET-350 — Language-gate selection-ladder + sticky headers (the #340 M1 class, back-filled)

- **Forge ticket:** #350 `1c88f88a-41b4-4e67-8071-3d1647ad8f26` (bug, M22) · duplicate #351 closed
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `34b5f8ea-5b31-4d24-a034-cbd598d0c256`
- **Pipeline doc:** ../../pipeline/active/350-structural-language-gate.spec.md
- **Source ticket:** the M22 integrity-five shelf — ../../design-notes/integrity-five-shelf.md
- **Status:** closed

## Summary

`step_selection_ladder` (⌃W/⌃⇧W expand-selection, #329) and `refresh_sticky_headers` (#330) build a
throwaway **Rust** `HighlightSession` unconditionally — neither checks the active file's language. On
a `.json`/`.py`/`.toml` file, ⌃W walks a Rust error-recovery tree's node ancestry (plausible-but-wrong
rungs) and sticky headers can pin garbage if the Rust grammar hallucinates a `function_item`-shaped
node. This is exactly the #340 inspect M1 class — bracket-match got its caller-side gate at inspect;
these two shipped before the rule existed. #315's implementer saw the gap and correctly scoped past
it, leaving a "stay Rust-PARSED, pre-existing behavior" comment at both sites. This ticket back-fills
the gate, mirroring `refresh_bracket_match` verbatim, and closes the M1 class across every structural
consumer.

## Acceptance

⌃W/⌃⇧W and sticky headers stay ACTIVE on a `.rs` file and go INERT on the same content opened as a
non-Rust file — no ladder built, no sticky header pinned, an existing pinned cache dropped in one
repaint on a Rust→non-Rust switch. Bracket-match, folding, and file-symbols are byte-identical (they
share no touched code). Full EARS criteria (REQ-001 … REQ-006) live in the pipeline spec.
