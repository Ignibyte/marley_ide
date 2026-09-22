---
spec_id: completions
component: marley_completer
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Command-line completion engine: line+cursor → ranked suggestions
goal: Given a partial command line and a cursor position, the engine classifies the token under the cursor, looks up the relevant command signatures, and returns ranked fuzzy-matched suggestions to drive Marley's inline autocomplete dropdown.
reuses: [nucleo, clap]
spec_source: "behavior-only reference (observable I/O): given a partial command line plus a UTF-8 byte cursor position, return the role of the token under the cursor (command head, flag, flag-value, positional argument, path, or shell variable) and a ranked, fuzzy-matched list of completion candidates plus the byte span any chosen candidate would replace; candidates derive from a registered command signature, or from a configurable path-completion fallback when the command head is unknown. No fork file paths and no private module/type/static names."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_completer` is Marley's completions engine. Given a partial command line `line` and a
UTF-8 byte cursor position `pos`, it splits the line into spanned tokens, classifies the token
under the cursor (command head, flag, flag-value, positional argument, path, or shell
variable), looks up the matching command **signature**, and produces a ranked list of
fuzzy-matched suggestions to drive Marley's inline autocomplete dropdown. It is a fresh Rust
implementation that reuses the permissive crates `nucleo` (fuzzy match + ranking primitive)
and `clap` (command model parsed into a completion signature).

The engine is a stateful `Completer` value (seam-contracts §3.5a): `suggestions` and `describe`
take `&mut self` so the engine can hold a **private, in-crate** parse cache keyed on the input
text. That cache is an implementation detail — it is **not** exposed as a public type and is
**not** a cross-crate contract. The completer↔classifier shared-parse hand-off is deferred to
M2 (seam-contracts §3.5a); at M1 `marley_input_classifier` parses its own tokens and this crate
ships no classifier-consumer seam.

All completer spans are **byte** spans over `line`, expressed in
`marley_text_offsets::ByteOffset` (seam-contracts §1.1). `SuggestionResults::replaced_span` is a
`Range<ByteOffset>`. The prompt-input integration converts that span to a `Range<CharOffset>`
via `marley_editor::Buffer::byte_to_char` **before** calling `Buffer::edit`; `marley_completer`
imports no `marley_editor` type and vice versa. The shared currency is `marley_text_offsets`.

## Public surface (the contract)
All in `crates/marley_completer/`. Names are Marley-original; the surface describes observable
I/O only.

```rust
// --- spans (re-use the one workspace ByteOffset; seam-contracts §1.1) ---
pub struct Spanned<T> { pub value: T, pub span: Range<ByteOffset> }

// --- value vocabulary OWNED BY THIS CRATE (not marley_command, a spawn-only wrapper) ---

/// An immutable snapshot of shell environment variables, used to source `$VAR`
/// completion candidates and to resolve path prefixes.
pub struct EnvVars(/* private map */);
impl EnvVars {
    pub fn from_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Self;
    pub fn get(&self, key: &str) -> Option<&str>;
    pub fn names(&self) -> impl Iterator<Item = &str>;   // candidate source for Variable completion
}

/// The path-separator policy of the target host (observable; e.g. `/` on unix,
/// `/` + `\` on Windows). Drives where a path token's directory prefix ends.
pub struct PathSeparators { pub primary: char, pub alternates: &'static [char] }

/// One entry returned by a directory listing, used to build Path candidates.
pub struct PathEntry { pub name: String, pub is_dir: bool }

/// The classified role of a single token, plus the token it describes.
pub struct Description {
    pub role: SuggestionType,
    pub token: Spanned<String>,
    pub token_index: usize,
}

// --- the stateful engine (seam-contracts §3.5a) ---
pub struct Completer { /* PRIVATE in-crate parse cache; no public snapshot */ }
impl Completer {
    pub fn new() -> Self;
    /// Primary entry point. Returns `None` only when `pos` is not addressable (see R14);
    /// otherwise always returns `Some` (possibly with an empty candidate list — R13).
    pub async fn suggestions<C: CompletionContext>(
        &mut self,
        line: &str,
        pos: ByteOffset,
        env: &EnvVars,
        options: &CompleterOptions,
        ctx: &C,
    ) -> Option<SuggestionResults>;
    /// Classify the token under `pos`. `None` when no token is under the cursor (R4).
    pub fn describe(&mut self, line: &str, pos: ByteOffset) -> Option<Description>;
    /// Pure classification of an already-extracted token at a known index (no parsing).
    pub fn describe_given_token(&self, token: &Spanned<String>, token_index: usize) -> Description;
}

pub trait CompletionContext {
    fn cwd(&self) -> &Path;
    fn env(&self) -> &EnvVars;
    fn path_separators(&self) -> PathSeparators;
    async fn list_dir(&self, dir: &Path) -> Vec<PathEntry>;
}

pub struct SuggestionResults {
    pub suggestions: Vec<MatchedSuggestion>,
    pub replaced_span: Range<ByteOffset>,
}
pub struct Suggestion {
    pub insert_text: String,
    pub display_text: String,
    pub kind: SuggestionType,
    pub priority: Priority,
    pub description: Option<String>,
}
pub struct MatchedSuggestion {
    pub suggestion: Suggestion,
    // private `mat: Match { score: u32, indices: Vec<u32> }` — ranking detail, not public
}
impl MatchedSuggestion {
    /// Observable highlight positions (the matched char indices) for the dropdown.
    pub fn highlight_indices(&self) -> &[u32];
}
pub enum SuggestionType { Subcommand, Flag, FlagValue, Argument, Path, Variable, Alias }

pub struct Priority(pub i32);
pub struct CompleterOptions { pub max_results: usize, pub fallback: CompletionsFallbackStrategy }
pub enum CompletionsFallbackStrategy { None, Paths }

// --- command signatures ---
pub struct SignatureRegistry { /* private */ }
impl SignatureRegistry {
    pub fn lookup(&self, command: &str) -> Option<&CommandSignature>;
    /// Derive a completion signature from a clap command model.
    pub fn from_clap(cmd: &clap::Command) -> CommandSignature;
}
pub struct CommandSignature { /* flags + positional value sets */ }
```

> **Removed for clean-room posture A (seam-contracts §11.1):** the `parsers` module and
> `classify_command`/`LiteCommand`/`ParsedToken`/`ParsedExpression`/`ClassifiedCommand`, the
> `matchers`/`MatchStrategy`/`MatchType` ranking taxonomy, and all "legacy vs v2" engine-split
> language. The private `Match { score, indices }` ranking type is an implementation detail and
> is not part of the public surface. The cross-crate `ParsedTokensSnapshot` is removed
> (deferred to M2 — seam-contracts §3.5a).

## EARS Requirements
R1. WHEN `Completer::suggestions(line, pos, …)` is called, the system shall split `line` into
tokens each occupying a byte range `[start, end)` over `line` such that the spans are
non-overlapping, appear in left-to-right source order, and are addressable by a zero-based
index increasing by exactly one (observable per token via `describe`'s
`Description::token.span` and `token_index`).

R2. WHEN a token's `span` is sliced out of `line` via its byte range, the system shall yield
exactly that token's `value` string (the `Description::token.value`).

R3. WHEN `line` contains pipe (`|`) separators, the system shall treat the first token of each
pipeline segment as a command-head position, so that `describe` classifies the token
immediately following a `|` separator as `SuggestionType::Subcommand`, identically to how it
classifies the first token of the whole line.

R4. WHEN `describe(line, pos)` is called, the system shall return the `Description` of the
single token whose span contains `pos` — treating a `pos` on a token's end boundary as
belonging to that token — and shall return `None` when no token is under the cursor: both when
`line` contains no tokens and when `pos` falls in inter-token whitespace (between two tokens, on
no token's span).

R5. WHEN the token under the cursor is the first token of its pipeline segment (command-head
position), the system shall classify it as `SuggestionType::Subcommand` regardless of any
leading sigil, so a first-position `-x` or `$FOO` classifies as `Subcommand` rather than `Flag`
or `Variable` (command-head position takes precedence over the sigil rules of R6).

R6. WHEN the token under the cursor is **not** in command-head position and begins with `-`, the
system shall classify it as `SuggestionType::Flag`; and WHEN it is **not** in command-head
position and begins with `$`, the system shall classify it as `SuggestionType::Variable`.
Exactly one classification fires for any token (R5 over R6).

R7. WHEN the command head resolves to a known signature in the `SignatureRegistry` and the
cursor token is a flag, the system shall draw candidate suggestions from that signature's
declared flags; and WHEN the cursor token is a positional argument, the system shall draw
candidates from that argument position's declared value set.

R8. IF the command head does not resolve to any signature in the registry, THEN the system shall
apply the configured `CompletionsFallbackStrategy`: returning no command-derived suggestions
when `None`, and returning path-completion candidates from the cursor token's directory prefix
(listed via `CompletionContext::list_dir`, split on `PathSeparators`) when `Paths`.

R9. WHEN candidate suggestions are matched against the cursor token's text, the system shall
rank them with the `nucleo`-backed matcher, retaining only candidates that fuzzy-match the typed
prefix and recording each kept candidate's match score and matched indices (the indices being
observable via `MatchedSuggestion::highlight_indices`).

R10. WHEN ranked results are ordered, the system shall sort by descending `Suggestion::priority`
first, then by descending match score, then by ascending case-insensitive `display_text` as a
deterministic final tiebreak.

R11. WHEN `CompleterOptions::max_results` is `n`, the system shall return at most `n` suggestions
in `SuggestionResults::suggestions`, keeping the top-`n` by the ordering of R10.

R12. WHEN a `SuggestionResults` is produced, the system shall set `replaced_span` to the byte
span of the cursor token that the chosen `insert_text` would replace, such that applying any
returned suggestion edits exactly that byte range of `line`.

R13. IF no candidate fuzzy-matches the cursor token and the fallback strategy yields nothing,
THEN the system shall return `Some(SuggestionResults)` with an empty `suggestions` vector rather
than `None`.

R14. WHEN `suggestions` is called with `pos` greater than `line.len()` or not on a UTF-8
character boundary, the system shall return `None` without panicking.

R15. WHEN both `describe(line, pos)` and `suggestions(line, pos, …)` are evaluated for the same
`(line, pos)`, the system shall classify the cursor token identically in both — i.e. the role in
the returned `Description` matches the role the suggestions were drawn for — because both run off
one in-crate parse/classification path.

R16. WHILE successive `suggestions`/`describe` calls on the same `Completer` instance are made
with a `line` byte-identical to the immediately preceding call's input, the system shall reuse
its private in-crate cached parse rather than re-splitting the line.

R17. WHEN a `clap::Command` is registered via `SignatureRegistry::from_clap`, the system shall
expose each of its long flags, short flags, and subcommands as completion candidates of the
corresponding `SuggestionType`.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | Tokens (via `describe`) have non-overlapping, source-ordered byte spans with `token_index` increasing by one from 0 (R1) | planned |
| 2 | Slicing a token's span out of `line` reproduces its `value` (R2) | planned |
| 3 | Token after a `|` classifies as `Subcommand`, like the line's first token (R3) | planned |
| 4 | `describe` returns the cursor token's role; end-boundary belongs to the token; empty line → `None`; inter-token whitespace → `None` (R4) | planned |
| 5 | First token of a segment classifies as `Subcommand` even for `-x`/`$FOO` (R5) | planned |
| 6 | Non-head `-`-prefixed → `Flag`; non-head `$`-prefixed → `Variable`; exactly one fires (R6) | planned |
| 7 | Known signature drives flag/positional candidates (R7) | planned |
| 8 | Unknown command applies fallback: `None`→empty, `Paths`→path candidates (R8) | planned |
| 9 | Candidates ranked via `nucleo`; non-matches dropped; highlight indices recorded (R9) | planned |
| 10 | Ordering = priority desc, score desc, case-insensitive display asc (R10) | planned |
| 11 | At most `max_results` returned, top-n by R10 (R11) | planned |
| 12 | `replaced_span` equals the cursor token's byte span (R12) | planned |
| 13 | No match + empty fallback → `Some` with empty `suggestions` (R13) | planned |
| 14 | `pos` past end or off char boundary → `None`, no panic (R14) | planned |
| 15 | `describe` and `suggestions` classify the cursor token identically (R15) | planned |
| 16 | Byte-identical `line` reuses the in-crate cached parse, no re-split (R16) | planned |
| 17 | `from_clap` exposes long/short flags + subcommands as typed candidates (R17) | planned |

## Visual / Behavioral Acceptance
N/A — non-UI engine. It returns a suggestion model; the autocomplete dropdown rendering and its
AXUIElement/screenshot assertions belong to the M1+ completions-panel UI spec, not here.

## Test Plan
- **Unit:** one `#[test]` per EARS clause, named for the requirement:
  - `r1_spans_nonoverlapping_ordered_indexed`, `r2_span_slice_equals_value`,
    `r3_token_after_pipe_is_subcommand`, `r4_describe_cursor_boundary_empty_and_whitespace`,
    `r5_first_token_is_subcommand_even_with_sigil`, `r6_nonhead_dash_is_flag_dollar_is_variable`,
    `r7_known_signature_flags_and_positionals`, `r8_unknown_command_fallback_none_and_paths`,
    `r9_nucleo_ranks_drops_nonmatches_records_indices`, `r10_ordering_priority_score_display`,
    `r11_max_results_top_n`, `r12_replaced_span_is_cursor_token`,
    `r13_no_match_returns_empty_results`, `r14_pos_out_of_range_or_off_boundary_none`,
    `r15_describe_and_suggestions_agree`, `r16_identical_line_reuses_in_crate_cache`,
    `r17_from_clap_exposes_flags_and_subcommands`.
  - R4 asserts `None` for three positions on one fixture: an empty line, an end-of-token boundary
    (which must be `Some`), and a byte landing in the space between two tokens (which must be
    `None`).
  - R5/R6 share a fixture pinning precedence: a `-x` and a `$FOO` in head position both yield
    `Subcommand`; the same tokens in non-head position yield `Flag`/`Variable` — proving exactly
    one classification fires and command-head wins.
  - R10 ordering uses a fixture with deliberate priority/score/display ties to pin every tiebreak
    level.
  - R16 cache-reuse is verified by a parse-call-counting test double (or instrumented splitter)
    asserting the second byte-identical call on the same `Completer` performs zero re-split
    passes, and that a changed `line` invalidates the cache.
  - R14 uses a multi-byte UTF-8 line and asserts `None` for an interior-byte `pos`.
  - 100% line coverage on every touched line; there is no feature-gated alternative engine to
    exclude.
- **Integration:** a seam test feeds a real `clap::Command` (built in-test) through
  `SignatureRegistry::from_clap` → `Completer::suggestions` and asserts the dropdown candidate
  set and types. (No completer↔classifier hand-off seam test at M1 — the shared parse snapshot is
  deferred to M2 per seam-contracts §3.5a; the classifier parses its own tokens.)
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full `cargo nextest` suite stays green; the `from_clap` candidate set and
  the R10 ordering are golden-fixture-pinned so a ranking or signature-extraction change is
  caught.

## Mutation Targets
`cargo mutants` must kill every viable mutant on the tokenizer, classifier, and ranker:
- span-arithmetic mutants in the tokenizer (off-by-one start/end, `<`↔`<=` in the
  cursor-containment test) — killed by R1/R2/R4/R12.
- classification branch mutants (`Subcommand`↔`Argument`; dropping the `-`/`$` sigil checks;
  **dropping the command-head precedence guard** so a first-position `-x`/`$FOO` wrongly becomes
  `Flag`/`Variable`) — killed by R5/R6.
- whitespace/no-token mutant (returning the nearest token instead of `None` for an inter-token
  `pos`) — killed by R4.
- signature-lookup and fallback-branch mutants (`Some`↔`None` swap, `None`↔`Paths` arm swap) —
  killed by R7/R8/R13.
- comparator mutants in the R10 sort key (swapping the priority/score/display order,
  `asc`↔`desc`) — killed by R10.
- `max_results` boundary mutant (`<`↔`<=`, dropping the truncation) — killed by R11.
- in-crate cache-guard mutant (always-miss / always-hit on the `line` byte-equality) — killed by
  R16.
- describe/suggestions divergence mutant (classifying the cursor token on a second, separate code
  path) — killed by R15.
- MSI target: **100%** on the testable surface.

## Dependencies
- REUSE (permissive, MIT/Apache): `nucleo` (fuzzy match + ranking primitive behind the private
  `Match`), `clap` (command model parsed for `from_clap` signature extraction).
- Marley components: `marley_text_offsets` (the one workspace `ByteOffset`; `Spanned` span math —
  seam-contracts §1.1).
- This crate **owns** `EnvVars`, `PathSeparators`, `PathEntry`, and `Description`; it does **not**
  source them from `marley_command` (a non-PTY spawn wrapper with no env/cwd/listing model —
  seam-contracts §5). The execution context (cwd/env/path-separators/dir-listing) is supplied by
  the caller through the `CompletionContext` trait.
- No dependency on `marley_input_classifier`: the M1 completer exposes no cross-crate parse
  snapshot, so there is no consumer edge (seam-contracts §3.5a).

## Out of scope / deferred
- The completer↔classifier shared parse-snapshot hand-off — **deferred to M2** (seam-contracts
  §3.5a). At M1 `marley_input_classifier` parses its own tokens; this crate keeps only a private
  in-crate cache and ships no cross-crate snapshot type or seam test. The shared-snapshot
  unification is the named M2 item in both specs and the build order.
- Any alternative JS-driven completion engine — out of scope at M1 (a separate later spec if ever
  pursued); M1 computes all suggestions through this Rust engine with no JS runtime dependency.
- The autocomplete dropdown UI (rendering, keyboard navigation, accept/dismiss, AXUIElement
  assertions) — owned by the M1+ completions-panel UI spec.
- The bundled command-signature data set (vendoring embedded signature dictionaries) — a
  data/packaging task; this spec defines the registry interface and the clap-derived path, not
  the shipped corpus.

## Clean-room provenance
Spec'd from a behavior-only reference (observable I/O of a command-line completion engine — see
`spec_source`); no AGPL/fork source read, no fork file paths, and no private module/type/static
names in the public surface (seam-contracts §11; quality-bar gate 16). The Warp-internal
`parsers` module, `classify_command`/`LiteCommand`/`ParsedToken`/`ParsedExpression`/
`ClassifiedCommand`, the `matchers` ranking taxonomy, and the legacy/`v2` engine split are
absent by construction. REUSE crates (`nucleo`, `clap`) are MIT/Apache. The package is named
`marley_completer`. IP-counsel sign-off on the behavioral wall is pending (open item in
`clean-build-plan.md`), hence `clean_room: "behavior-derived from a fork-reference doc;
IP-counsel sign-off pending"`.
