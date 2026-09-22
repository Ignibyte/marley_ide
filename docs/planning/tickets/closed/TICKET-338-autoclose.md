# TICKET-338 — Auto-close brackets + quotes: pair / type-over / backspace-pair / wrap-selection, at N cursors

- **Forge ticket:** #338 19428bae-8f16-4bdb-b3c8-c9850e54dd27 (feature, M22)
- **Owner:** claude (this session)
- **AAR:** acd1a989-2571-43db-82d1-78a7a467c007
- **Pipeline doc:** ../../pipeline/completed/338-autoclose.spec.md
- **Source ticket:** M22 "The editing bar" batch (#336–340 + the 11 that follow) — the THIRD of the batch
  ([m22-editing-bar.md](../../design-notes/m22-editing-bar.md) · [roadmap.md](../../../marley_architecture/roadmap.md))
- **Status:** closed

## Summary
Typing `(` gives you `(`. Every editor a developer has used in the last decade gives `()` with the caret
inside — plus type-over on the closer, pair-delete on backspace, and wrap-on-type around a selection. Its
absence is felt within seconds, and no ticket for it existed (`auto_close` has zero hits repo-wide). The
behavior is one pure decision table — `pair_action(typed, prev, next, has_selection) -> Action` — mapped onto
the shipped M19 #296/#297 multi-cursor edit engine.

**Phase 1 found the spec's central bullet false**, which reshapes the ticket: the engine does the TEXT half,
but its post-state is always **bare carets at the end of each insert** (`selections_after_multi_edit`,
selection.rs:253-255), so InsertPair lands after the `)`, Wrap destroys the selection it is supposed to
preserve, and TypeOver is not an edit at all. **The SELECTION half is this ticket's own work**, and it must be
done without breaking the undo-coalescing precondition that same placement encodes. The good news Phase 1 also
found: the hook site is `ime::replace_text` branch 1 (crates/editor/src/ime.rs:100) — inside the **editor
crate**, unit-testable, not the `mutants::skip` app shim.

## Acceptance
Typing an opener at a bare caret inserts the pair with the caret between it; typing a closer adjacent to that
same closer steps over it instead of inserting; backspace between an adjacent pair deletes both; typing an
opener with a selection wraps it, selection preserved, at each of N cursors; `'` does not pair after `&` or an
identifier char; every action is ONE undo unit and ⌘Z restores text + all carets exactly; `editor.auto_close`
off is byte-identical to today. Full EARS REQ-001..009 in the pipeline spec.
