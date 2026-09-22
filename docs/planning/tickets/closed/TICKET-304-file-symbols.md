# TICKET-304 — Go to Symbol in File (⌘⇧O): a tags-query picker over the current file

- **Forge ticket:** #304 4b10e6e2-0a24-4835-bcaf-a8be11a432ab (feature, M19)
- **Owner:** claude (this session — the /work 300…259 goal)
- **AAR:** 044a3850-e2e3-47f8-92c2-2c47780d1574
- **Pipeline doc:** ../../pipeline/active/304-file-symbols.spec.md (promoted 2026-07-18; pipeline_id 4687b05f-744c-468a-8a86-ff6e51c0e3cc)
- **Source ticket:** the IDE-MVP shelf ([ide-mvp-shelf.md](../../design-notes/ide-mvp-shelf.md)) — the FOURTH of `/work 300,302,303,304,305,314,315,316,317,259`
- **Status:** closed

## Summary
⌘⇧O opens a fuzzy picker of THIS file's symbols (fns/structs/enums/traits/impl methods/mods/macros); Enter
jumps to the chosen one, centered, pushing the NavStack (⌃- returns). The extractor is a pure `marley_syntax`
`file_symbols(src) -> Vec<Symbol>` — a thin adapter over tree-sitter-rust's OWN tags query (`TAGS_QUERY`),
NOT a hand-rolled node walk. The picker mirrors the #325 `FinderState` shape but ranks locally (the list is in
hand) with `marley_search_core::fuzzy_rank`. Rust-only v1, caller-gated on `Language::Rust` (the #340 M1 lesson).

## Acceptance
⌘⇧O on a Rust editor tab lists its symbols in document order (empty query) or fuzzy-ranked (typed); Enter
jumps + centers + pushes NavStack; a non-Rust/empty file shows "(no symbols)" with no walk; ⌘⇧O off an editor
still opens a remote (the shadow). Full EARS in the spec.
**Load-bearing claim under Phase-1 verification:** tree-sitter-rust ships `TAGS_QUERY` with usable
`@definition.*`/`@name` captures. If absent/different, the extraction approach reshapes (see the Phase 1 ledger).
