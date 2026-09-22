# TICKET-305 — Code folding: chevrons, ⌥⌘[/⌥⌘], and the first visible↔buffer row projection

- **Forge ticket:** #305 13b140d5-ca11-44a6-80c3-29bc6dbac0f3 (feature, M19)
- **Owner:** claude (this session — the /work 300…259 goal)
- **AAR:** 9bd998a9-ccfc-48f8-953d-2428ca72a3db
- **Pipeline doc:** ../../pipeline/active/305-code-folding.spec.md (promoted 2026-07-18; pipeline_id 2a6b5293-2778-43e6-ab6d-deb975146f79)
- **Source ticket:** the IDE-MVP shelf ([ide-mvp-shelf.md](../../design-notes/ide-mvp-shelf.md)) — the FIFTH of `/work 300,302,303,304,305,314,315,316,317,259`
- **Status:** closed

## Summary
Fold a function/impl/mod/trait/struct/enum/match body to one line — a gutter chevron (▾/▸), ⌥⌘[ folds the
innermost region at the caret, ⌥⌘] unfolds, a "⋯ N lines" tail on the header. The REAL work is the first
projection between buffer rows and rendered rows: a pure `marley_syntax::fold_regions` + a pure `FoldProjection`
(`visible_count`/`buffer_row(slot)`/`slot_of(row)`) that converts ONCE at the `uniform_list` rim
(D-PROJECT-AT-THE-BOUNDARY) so the ~17 interior visible==buffer sites stay untouched. Fold state rides
`anchor.rs` (its first production consumer); auto-reveal on any caret placement into a hidden row; Rust-only,
caller-gated.

## Acceptance
⌥⌘[ folds the innermost region at the caret (its rows hide, a "⋯ N lines" tail shows), ⌥⌘] unfolds exactly;
a gutter-chevron click toggles without moving the caret; a fold survives text typed above it (anchors); any
navigation into a hidden row auto-reveals; scroll/sticky convert buffer↔slot correctly; a non-Rust file shows
no chevrons. Full EARS in the spec.
**Load-bearing premises under Phase-1 verification:** the ~17-site blast radius + the "project once at the
boundary" seam viability + anchor.rs being a shipped, unused rebase capability. If the seam isn't clean or the
radius is much larger, the ticket may split (see the Phase 1 ledger).
