---
spec_id: syntax-highlight
component: marley_syntax
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Incremental tree-sitter highlight + auto-indent runtime
goal: Keep an editor buffer's syntax colors and auto-indent decisions live and correct by re-parsing only the edited region with tree-sitter.
reuses: [tree-sitter, tree-sitter-highlight, tree-sitter-rust, tree-sitter-python, tree-sitter-javascript, tree-sitter-typescript, tree-sitter-json, tree-sitter-toml-ng, tree-sitter-go, tree-sitter-bash]
spec_source: "behavior-only — observable I/O: an incremental tree-sitter-backed highlight + auto-indent runtime that yields highlight spans (id→color via a registered theme with fallback) and indent suggestions, emitting decoration-updated events. No fork module/type/static names. Seam types per standards/seam-contracts.md §3."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A — this crate emits highlight spans + indent deltas as data; on-screen painting is asserted by the editor-panel spec that consumes them.
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_syntax` is the incremental syntax-highlighting and auto-indent runtime for an editor buffer. It keeps a small least-recently-used set of recent parsed tree-sitter `Tree`s — the current tree plus up to `MAX_SYNTAX_TREES - 1` prior-version snapshots that back incremental reuse and version-keyed highlight queries — and translates buffer edits into tree-sitter `InputEdit`s so re-parsing touches only the changed region. It runs a language's highlight and indent queries, emits per-range highlight identifiers (resolvable to theme colors through a registered color map) plus per-line indent deltas, and announces completed recomputations to subscribers. A sibling registry, `marley_languages`, resolves a language name or filename to a loaded grammar + query bundle and owns the `IndentUnit` vocabulary. Together they let any Marley panel paint syntax-colored, correctly-indented code without re-implementing parsing.

The buffer-core seam types this crate binds to are owned upstream and imported, never re-declared here: `Rope`, `BufferDelta`, `BufferVersion`, and `Point` come from `marley_editor` (seam-contract §3.1–§3.4); `IndentUnit` comes from `marley_languages` (§3.6); `CharOffset` comes from `marley_text_offsets` (§1). `apply_edit` derives the tree-sitter `InputEdit` from a `BufferDelta` per §3.1 — `start_byte = byte_range.start`, `old_end_byte = byte_range.end`, `new_end_byte = byte_range.start + new_byte_len`, with the corresponding `Point` row/column positions (§3.3).

## Public surface (the contract)
`marley_languages`:
- `pub const SUPPORTED_LANGUAGES: &[&str]` — authoritative name list (v1 trimmed set).
- `pub struct Language { grammar, highlight_query, indents_query: Option<Query>, indent_unit: IndentUnit, comment_prefix: Option<String>, bracket_pairs: Vec<(char, char)>, display_name: String }` — `indent_unit: IndentUnit` is the `marley_languages`-owned type (§3.6).
- `pub struct LanguageRegistry` with `fn language_by_name(&self, name: &str) -> Option<Arc<Language>>`.
- `pub fn language_by_filename(path: &Path) -> Option<Arc<Language>>`.

`marley_syntax`:
- `pub struct SyntaxHighlighter` — central state. `fn new(language: Option<Arc<Language>>) -> Self`.
- `fn set_language(&mut self, language: Option<Arc<Language>>)`.
- `fn has_highlighting(&self) -> bool` — defined as `loaded AND last_parsed_size <= MAX_PARSE_BYTES`; the size guard wins over `loaded` (an oversized buffer disables highlighting even with a language set).
- `fn indent_unit(&self) -> Option<IndentUnit>` (`marley_languages::IndentUnit`); `fn bracket_pairs(&self) -> Option<&[(char, char)]>`; `fn comment_prefix(&self) -> Option<&str>`.
- `fn current_version(&self) -> BufferVersion` (`marley_editor::BufferVersion`) — the shadow version this runtime advances in lockstep with the editor buffer.
- `fn apply_edit(&mut self, text: &Rope, edit: &BufferDelta) -> Result<(), SyntaxError>` — `Rope`/`BufferDelta` from `marley_editor`; derives the tree-sitter `InputEdit` from the delta (§3.1) and incrementally re-parses, advancing `current_version()`.
- `fn highlights_in_range(&mut self, text: &Rope, range: Range<CharOffset>) -> Vec<HighlightSpan>` where `HighlightSpan { range: Range<CharOffset>, id: HighlightId }` (`CharOffset` from `marley_text_offsets`). Spans carry ids only; color resolution is via `color_for`.
- `fn indentation_at(&self, text: &Rope, point: Point) -> Option<IndentDelta>` (`Point` from `marley_editor`).
- `fn set_color_map(&mut self, map: ColorMap)` — registers the `HighlightId -> ColorU` table together with its fallback color.
- `fn color_for(&self, id: HighlightId) -> ColorU` — resolves a highlight id to its color, or the registered fallback when the id is absent from the map.
- `pub struct ColorMap { /* HighlightId -> ColorU entries + a registered `fallback: ColorU` */ }` with `fn new(entries: impl IntoIterator<Item = (HighlightId, ColorU)>, fallback: ColorU) -> Self`.
- `fn invalidate_cache_for_version(&mut self, version: BufferVersion)`.
- `fn retained_tree_count(&self) -> usize` — number of parsed `Tree`s currently retained (`<= MAX_SYNTAX_TREES`); makes the cap/eviction black-box testable.
- `fn subscribe(&mut self, f: impl Fn(&DecorationEvent) + 'static) -> DecorationSubscription` — emission sink for recompute notifications (mirrors `marley_settings::subscribe`); dropping the returned `DecorationSubscription` unsubscribes.
- `pub enum DecorationEvent { Updated { version: BufferVersion } }` — `version` is `marley_editor::BufferVersion` (§3.2), never a bare `u64`.
- `pub struct DecorationSubscription` — RAII handle; on drop the callback stops receiving events.
- `pub enum SyntaxError { /// A delta byte or char offset fell outside the current text length. OffsetOutOfBounds }`.
- `pub const MAX_PARSE_BYTES: usize = 2 * 1024 * 1024;` `pub const MAX_SYNTAX_TREES: usize = 3;`

## EARS Requirements
R1. The system shall expose `SyntaxHighlighter` that, given a `Language`, produces highlight spans and indent deltas for an editor buffer.
R2. WHEN `set_language` is called with a language whose grammar and highlight query load successfully, the system shall load the grammar and make `has_highlighting()` return `true` while the last-parsed buffer size is `<= MAX_PARSE_BYTES`.
R3. IF `language_by_name` is called with a name not in `SUPPORTED_LANGUAGES`, THEN the system shall return `None`.
R4. WHEN `set_language(None)` is called, the system shall make `has_highlighting()` return `false` and `highlights_in_range` return an empty vector.
R5. WHEN `apply_edit(text, edit)` is called with highlighting enabled and an in-bounds `BufferDelta`, the system shall derive the tree-sitter `InputEdit` from the delta (per §3.1) and re-parse incrementally, such that `Tree::changed_ranges(previous, current)` reports only ranges intersecting the edited region (no whole-buffer reparse).
R6. WHEN `highlights_in_range(text, range)` is called with highlighting enabled, the system shall return `HighlightSpan`s whose `range` fields are non-overlapping, sorted ascending, and each fully contained within the requested `range`.
R7. The system shall, for identical `(text, language, range)` inputs, return byte-for-byte identical `HighlightSpan` output across repeated calls.
R8. WHILE the highlight cache holds a result for the current `(version, range, language)` key, WHEN `highlights_in_range` is called with that same key, the system shall return the cached result without invoking the highlight query, as observed by a `#[cfg(test)]` query-invocation counter that does not advance.
R9. WHEN `invalidate_cache_for_version(version)` is called, the system shall discard any cached highlights keyed to that `version`.
R10. IF a parse is attempted on a buffer whose byte length exceeds `MAX_PARSE_BYTES`, THEN the system shall skip parsing, record the oversized size so that `has_highlighting()` returns `false` (the size guard taking precedence over a loaded language), and `highlights_in_range` returns an empty vector.
R11. The system shall retain at most `MAX_SYNTAX_TREES` parsed `Tree`s at any time, evicting the least-recently-used tree when a new parse would exceed the cap, such that `retained_tree_count()` never exceeds `MAX_SYNTAX_TREES`.
R12. WHEN a highlight recomputation completes for the current buffer version (the highlight query was run, not served from cache), the system shall deliver `DecorationEvent::Updated { version }` to every live subscriber, where `version == current_version()`.
R13. WHEN `indentation_at(text, point)` is called and the active language has an indent query, the system shall return `Some(IndentDelta)` computed from the syntax node enclosing `point`.
R14. IF the active language has no indent query (`indents_query` is `None`), THEN `indentation_at` shall return `None`.
R15. WHEN `color_for(id)` is called after `set_color_map(map)`, the system shall return the `ColorU` mapped to `id` when `id` is present in `map`, and the map's registered fallback color when `id` is absent.
R16. IF tree-sitter parsing produces an error node, THEN the system shall return highlight spans for the successfully parsed regions and shall not panic.
R17. The system shall return `indent_unit()`, `bracket_pairs()`, and `comment_prefix()` equal to the corresponding fields of the active `Language`, and `None` when no language is set.
R18. IF `apply_edit` is called with a `BufferDelta` whose `byte_range` or `char_range` falls outside the current text length, THEN the system shall return `Err(SyntaxError::OffsetOutOfBounds)` and leave the retained `Tree`(s) and `current_version()` unchanged.
R19. WHEN `apply_edit` returns `Ok(())`, the system shall advance `current_version()` to its successor (`BufferVersion::next`).
R20. WHEN a `DecorationSubscription` returned by `subscribe` is dropped, the system shall stop invoking its callback for subsequent `DecorationEvent`s.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `SyntaxHighlighter::new` + `highlights_in_range` yields spans for a known language (R1) | planned |
| 2 | `set_language` with a supported language ⇒ `has_highlighting()==true` (R2) | planned |
| 3 | `language_by_name("not-a-lang")==None` (R3) | planned |
| 4 | `set_language(None)` ⇒ `has_highlighting()==false`, empty spans (R4) | planned |
| 5 | `apply_edit` re-parses incrementally; `Tree::changed_ranges` is local to the edit (R5) | planned |
| 6 | spans are non-overlapping, sorted, contained in range (R6) | planned |
| 7 | identical inputs ⇒ identical output bytes (R7) | planned |
| 8 | second same-key call hits cache; query-invocation counter does not advance (R8) | planned |
| 9 | `invalidate_cache_for_version` drops that version's cache (R9) | planned |
| 10 | buffer > 2 MiB ⇒ no parse, `has_highlighting()==false`, empty spans (R10) | planned |
| 11 | `retained_tree_count() <= MAX_SYNTAX_TREES`; LRU eviction at the cap (R11) | planned |
| 12 | recompute delivers `DecorationEvent::Updated{version}` to subscribers, `version==current_version()` (R12) | planned |
| 13 | `indentation_at` returns delta from enclosing node (R13) | planned |
| 14 | no indent query ⇒ `indentation_at==None` (R14) | planned |
| 15 | `color_for` returns the mapped color; absent id ⇒ registered fallback (R15) | planned |
| 16 | malformed source ⇒ partial spans, no panic (R16) | planned |
| 17 | metadata accessors mirror `Language`; `None` when unset (R17) | planned |
| 18 | out-of-bounds delta ⇒ `Err(OffsetOutOfBounds)`, tree + version unchanged (R18) | planned |
| 19 | successful `apply_edit` advances `current_version()` by one (R19) | planned |
| 20 | dropping a `DecorationSubscription` stops callbacks (R20) | planned |

## Visual / Behavioral Acceptance
N/A. `marley_syntax` emits highlight spans and indent deltas as plain data; the on-screen painting and AXUIElement/screenshot assertions belong to the editor-panel spec that consumes this runtime. Output correctness here is asserted by deterministic snapshot tests over the `HighlightSpan` vectors (R7). Because `visual_acceptance` is `N/A`, quality-bar gate 15 is satisfied with no UI clause to assert.

## Test Plan
- **Unit:** one test per requirement — `r1_spans_for_known_language`, `r2_set_language_enables`, `r3_unknown_name_none`, `r4_no_language_empty`, `r5_incremental_changed_ranges_local`, `r6_spans_sorted_contained`, `r7_deterministic_output`, `r8_cache_hit_no_query_invocation`, `r9_invalidate_drops_version`, `r10_oversized_buffer_disables_highlighting`, `r11_lru_evicts_at_cap`, `r12_recompute_delivers_decoration_updated`, `r13_indent_from_node`, `r14_no_indent_query_none`, `r15_color_for_maps_and_fallback`, `r16_error_node_partial_no_panic`, `r17_metadata_accessors`, `r18_out_of_bounds_err_no_mutation`, `r19_version_advances_on_apply_edit`, `r20_dropped_subscription_stops_callbacks`. 100% coverage on touched lines. R8 uses a `#[cfg(test)]` query-invocation counter; R12/R20 use a counter-backed subscriber to assert the exact delivery count (once per live recompute; zero after drop).
- **Integration:** `marley_languages::language_by_name`/`language_by_filename` → `SyntaxHighlighter::set_language` → `highlights_in_range` round trip across the v1 grammar set (rust, python, json, **toml via `tree-sitter-toml-ng`**); edit-then-rehighlight seam exercising `apply_edit` (real `BufferDelta` from `marley_editor`) + cache invalidation + `DecorationEvent` delivery. A CI check asserts every grammar crate resolves against a single tree-sitter core minor (no mixed-ABI grammar).
- **Visual:** none (see Visual / Behavioral Acceptance).
- **Regression:** golden snapshot files of highlight spans for fixture sources per v1 language must stay byte-stable across grammar/query bumps; the full suite green before merge.

## Mutation Targets
`cargo-mutants` (MSI 100% on the testable surface) must kill mutants in: the delta→`InputEdit` offset arithmetic (`start_byte`/`old_end_byte`/`new_end_byte` — R5/R18 boundary math); the span sort/containment filter (R6); the cache-key equality and `MAX_PARSE_BYTES` comparison (R8/R10 — flip `<`/`<=`/`>`); the `has_highlighting()` `loaded && size <= MAX_PARSE_BYTES` composition (R2/R10 precedence); the `MAX_SYNTAX_TREES` LRU eviction count and `retained_tree_count()` cap (R11); the `BufferVersion::next` advance in `apply_edit` (R19); the indent-delta sign/magnitude computation (R13); the `color_for` lookup-vs-fallback branch (R15); and the `DecorationEvent` delivery + `DecorationSubscription`-drop unsubscribe path (R12/R20). No ACCEPTED-UNTESTABLE lines are expected: every per-language grammar binds through the **safe `LanguageFn` API** (tree-sitter ≥ 0.22 grammar crates), so this crate carries no `unsafe` and needs no miri target (gate 6 N/A); the tree-sitter C FFI lives entirely inside the REUSE grammar/runtime crates and is exercised only through their safe API.

## Dependencies
- REUSE (permissive): `tree-sitter`, `tree-sitter-highlight` (MIT); per-language grammar crates `tree-sitter-{rust,python,javascript,typescript,json,go,bash}` and `tree-sitter-toml-ng` (0.7.0, MIT — replaces `tree-sitter-toml` 0.20.0, whose old 0.20 ABI cannot link against the tree-sitter core minor this crate uses); Helix/Zed `.scm` highlight + indent query sets (MIT/Apache, vendored as data). All grammar crates are pinned to versions (≥ 0.22) that expose the safe `LanguageFn` binding, and CI asserts they all resolve against **one** tree-sitter core minor.
- Marley components:
  - `marley_editor` (seam-contract §3) — `Rope` (§3.4), `BufferDelta` (§3.1), `BufferVersion` (§3.2), `Point` (§3.3). `apply_edit` consumes a `BufferDelta` and derives the tree-sitter `InputEdit` per §3.1.
  - `marley_languages` (§3.6) — `IndentUnit` and the grammar + query bundles surfaced through `Language`/`LanguageRegistry`.
  - `marley_text_offsets` (§1) — `CharOffset` (the `HighlightSpan`/range currency; private-field, non-interchangeable).
  - the theming layer — supplies the `ColorMap` (`HighlightId -> ColorU`) and its registered fallback; `ColorU` is the shared RGBA color value type.
  - The consuming editor-panel spec depends on this crate (it owns the on-screen painting and calls `color_for`).

## Out of scope / deferred
- The full 34-language grammar set — v1 ships a trimmed set; growing it is a later EXTEND (data-only) pass.
- Symbol/outline queries (`symbols_query`), code folding, and injection grammars (e.g. JS-in-HTML) — deferred to M2+.
- Async/cancellable background parsing (in-flight parse abort) — v1 parses synchronously under the `MAX_PARSE_BYTES` guard; cancellation is deferred to the rendering-runtime integration in M2.
- On-screen rendering, theming UI, and bracket-match highlighting — owned by the editor-panel spec.

## Clean-room provenance
Spec'd from `docs/specs/fork-reference/syntax-highlight.behavior.md` — a behavior-only reference (observable inputs/outputs: edit→reparse, range→highlight ids, point→indent delta) authored without reading or translating any AGPL/fork source, and carrying no private module/type/static names or fork file paths. Per seam-contract §11 the `clean_room` line is held at "behavior-derived from a fork-reference doc; IP-counsel sign-off pending" until the behavioral wall + counsel sign-off land (open item in `clean-build-plan.md`). The implementation is a fresh Rust crate built on the permissive `tree-sitter`/`tree-sitter-highlight` runtime crates and public MIT/Apache grammar + query sets (Helix/Zed); it does not reuse or translate the fork's AGPL syntax-tree code, and its public surface uses only Marley-original identifiers (Posture A).
