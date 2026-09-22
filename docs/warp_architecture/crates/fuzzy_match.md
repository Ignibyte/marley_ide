# fuzzy_match

> Per-crate reference (Marley round 2). Crate dir: `crates/fuzzy_match`. Marley is forked from Warp (warpdotdev/warp).

| Field | Value |
| --- | --- |
| Subsystem | [editor-and-text](../subsystems/02-editor-and-text.md) |
| License | AGPL v3 (workspace `license = "AGPL-3.0-only"`; no per-crate LICENSE marker) |
| Internal deps | 0 |
| Used by | 2 |
| Provenance | Core **`[permissive]`** (`fuzzy-matcher` / `SkimMatcherV2`, MIT); the thin wrapper carries the AGPL workspace license (`[Warp-derived/AGPL]`). Marley's palette/finder (`marley_app::{palette, finder, command_bar}`) do their own matching — **`[Marley-original]`**. See [subsystem — Provenance & licensing](../subsystems/02-editor-and-text.md). |

## Purpose

`fuzzy_match` is a tiny, dependency-light string-matching library. Its
Cargo description: *"Performs fuzzy match with text against query."* It provides
two families of matching:

1. **Traditional fuzzy matching** — wraps `fuzzy-matcher`'s `SkimMatcherV2`
   (the Skim/fzf algorithm) to score how well a query fuzzily matches a string,
   returning both a score and the matched character indices (for highlight
   rendering).
2. **Glob-style wildcard matching** (`*`, `?`) — a hand-rolled, regex-free
   recursive matcher optimized for **file-path search**, with fast paths for
   common patterns (`*.rs`, `src/*`), substring matching anywhere in a path,
   and "progressive typing" support (`*.r` matches `.rs`/`.rb` as the user types).

It exists so that command palettes, file finders, completion menus, and symbol
search can rank and highlight candidates consistently without each call site
re-implementing matching or pulling in regex.

## Key types, modules & public API

The whole crate is `src/lib.rs` (tests in `src/fuzzy_tests.rs`).

- **`FuzzyMatchResult`** — `{ score: i64, matched_indices: Vec<usize> }`
  (char indices). `FuzzyMatchResult::no_match()` returns a zero-score, empty
  result for representing an unmatched-but-present item.
- Fuzzy functions (all `-> Option<FuzzyMatchResult>`):
  - `match_indices(text, query)` — smart-case (case-sensitive only if the query
    contains an uppercase letter).
  - `match_indices_case_insensitive(text, query)`.
  - `match_indices_case_insensitive_ignore_spaces(text, query)` — strips spaces
    from the query (useful for symbol-name search), but returns indices into the
    original text.
- Wildcard functions:
  - `contains_wildcards(query) -> bool` — does the query contain `*` or `?`.
  - `match_wildcard_pattern(text, pattern)` — glob match; scoring tiers
    (exact 2000, prefix/suffix/substring 1000, partial-suffix 800).
  - `match_wildcard_pattern_case_insensitive(text, pattern)`.

Internals (private): `match_internal` (the `SkimMatcherV2` adapter),
`is_glob_match` / `is_glob_match_recursive` (byte-level), `is_glob_match_chars*`
(char-level), `find_substring_glob_match`, `find_partial_suffix_match`,
`is_glob_match_at_position`, `find_glob_match_end`.

## Depends on (internal)

None. Zero internal dependencies — the only dependency is the third-party
`fuzzy-matcher = "0.3.7"` crate.

## Used by (internal dependents)

- [warp](./warp.md) — the main app crate (command palette / general search).
- [warp_completer](./warp_completer.md) — completion ranking and filtering.

Total: 2 dependents.

## Related crates

- [warp_completer](./warp_completer.md) — primary consumer; pairs fuzzy scoring
  with completion candidate generation.
- [markdown_parser](./markdown_parser.md) — sibling leaf crate in the same
  subsystem (both zero-internal-dep utilities).

## Marley relevance

**Classify: KEEP.**

A pure algorithm crate with no Warp branding, no network, no auth. Keep as-is.
It directly serves Marley goal (1) *expand the UI surface with a custom panel*:
any new Marley panel that filters lists (sessions, commands, files) should reuse
`match_indices` for scoring + `matched_indices` for highlight spans rather than
rolling its own. Nothing here relates to session lifecycle (2), login/de-auth
(3), or rebrand (4) beyond a possible eventual package rename — and the name
`fuzzy_match` is already generic, so even that is unnecessary. This is a
zero-risk crate to leave untouched.

## Notes / gotchas

- **Index units:** `match_indices*` return **char** indices (the
  `fuzzy_indices` API already yields char indices, per the inline comment), and
  the wildcard matchers also operate in char space — but several private helpers
  do byte-level matching internally (`is_glob_match` over `&[u8]`). Consumers
  should treat all returned indices as char offsets.
- **Empty pattern:** `match_wildcard_pattern("", ...)` returns
  `Some(no_match())` (a present-but-unmatched result), not `None` — guard for
  this at call sites that distinguish "no query" from "no match."
- **Recursive glob matcher** is unbounded in the general case; the code adds
  fast paths and a non-recursive branch specifically to avoid pathological
  recursion on complex patterns. Be cautious adding new wildcard syntax.
- `match_wildcard_pattern_case_insensitive` lowercases both inputs and assumes
  char positions stay aligned — this can drift for locale/multi-codepoint
  case mappings; it defensively `retain`s only in-range indices.
