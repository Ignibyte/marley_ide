---
spec_id: input-classifier
component: marley_input_classifier
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Shell-vs-AI input classification for the unified prompt
goal: Decide, from what the user has typed into Marley's single prompt field, whether the line is a shell command or a natural-language query for the AI agent, so one text field can serve both the terminal and Agent Mode.
reuses: [rust-stemmers, regex, anyhow]
spec_source: "behavior-only — observable I/O: classify an input line as a shell command vs a natural-language prompt via token-count + natural-language-ratio heuristics, returning the class + confidence. No fork module/type/static names."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_input_classifier` is the decision engine behind Marley's unified prompt: given the tokens the user has typed and the current prompt state, it answers "is this a **shell command** or a **natural-language query** to the AI agent?" so a single text field can route to either the terminal or Agent Mode. This spec covers the **heuristic-first** implementation: a model-free `HeuristicClassifier` that weighs a lexical natural-language score against token-count thresholds and hand-tuned one-off allowlists, plus a stubbed ML path that always degrades to the heuristic so the prompt never blocks or panics. The optional on-device model (candle/ort) is deferred; the offline build embeds no model bytes. Per seam-contracts §3.5a the crate parses its **own** in-crate `Tokens` at M1 and takes **no** dependency on `marley_completer`.

## Public surface (the contract)
All in `crates/marley_input_classifier/src/` unless noted. All identifiers are Marley-original; no fork-internal regex/keyword-table static name appears in the surface (those remain private implementation details — see Clean-room provenance).

- `pub enum InputType { Shell, Ai }` (`src/input_type.rs`) — the classification result kind; `Shell` is `Default`. Implements `Display`, `FromStr`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Debug`. `pub const fn is_ai(self) -> bool`.
- `pub enum DecisionSource` (`src/lib.rs`) — provenance of a decision: `Heuristic`, `FallbackCurrentInput`, `NaturalLanguageOneOffAllowlist`, `ShellCommandAllowList`, `ModelStub`. `Clone`, `Copy`, `PartialEq`, `Eq`, `Debug`.
- `pub struct Context { pub current_input_type: InputType, pub is_agent_follow_up: bool }` — the prompt state at classification time.
- `pub struct InputClassificationResult { pub input_type: InputType, pub source: DecisionSource }`.
- `pub struct ClassificationResult { p_shell: f32, p_ai: f32, source: DecisionSource }` — probability-bearing result.
  - `pub fn pure_shell(source: DecisionSource) -> Self` (`p_shell = 1.0`), `pub fn pure_ai(source: DecisionSource) -> Self` (`p_ai = 1.0`).
  - `pub fn confidence(&self) -> f32` (`max(p_shell, p_ai)`), `pub fn to_input_type(&self) -> InputType` (`Ai` iff `p_ai > p_shell`, else `Shell`), `pub fn source(&self) -> DecisionSource`.
- `pub trait InputClassifier: 'static + Send + Sync` — the core abstraction:
  - `fn detect_input_type(&self, tokens: &Tokens, cx: &Context) -> InputClassificationResult`
  - `fn classify_input(&self, tokens: &Tokens, cx: &Context) -> anyhow::Result<ClassificationResult>`
- `pub struct Tokens` — the parsed input passed to the classifier, parsed **in-crate** (seam-contracts §3.5a): `pub fn parse(query: &str) -> Tokens` (in `src/parser.rs`), exposing `pub fn is_empty(&self) -> bool`, `pub fn len(&self) -> usize`, `pub fn first(&self) -> Option<&str>`, and `pub fn words(&self) -> Vec<Cow<'_, str>>`.
- `pub struct HeuristicClassifier` — the model-free `InputClassifier`; `pub fn new() -> Self`. Constants (seam-contracts §9): `DETECT_AS_NATURAL_LANGUAGE_THRESHOLD = 0.6` (high-token regime), `LOW_TOKEN_THRESHOLD = 0.8` (low-token regime), `LOW_TOKEN_COUNT = 2`.
- `pub struct StubModelClassifier` — the deferred-ML stand-in; `pub fn new() -> Self`. Its `classify_input` always returns `Err(ClassifierError::ModelUnavailable)` mapped to the heuristic at the call site; its `detect_input_type` delegates to an inner `HeuristicClassifier` and stamps `DecisionSource::ModelStub`.
- `pub mod scorer` — the lexical signal:
  - `pub fn natural_language_words_score(words: Vec<Cow<'_, str>>, is_first_token_command: bool) -> usize`
  - `pub fn check_if_token_has_shell_syntax(word: &str) -> bool`
  - `pub fn is_word(word: &str, db: WordDb) -> bool`, `pub enum WordDb { English, StackOverflow, Command }`.
- `pub mod util` — one-off allowlists and head-token signals: `pub fn is_one_off_shell_command_keyword(word: &str) -> bool`, `pub fn is_one_off_natural_language_word_or_prefix(word: &str) -> bool`, `pub fn is_agent_follow_up_input(tokens: &Tokens, cx: &Context) -> bool`, `pub fn is_likely_shell_command(tokens: &Tokens) -> bool` (inspects the head token; the derivation basis for `is_first_token_command`, §9).

## EARS Requirements
R1. The system shall expose `InputType` with exactly two variants, `Shell` and `Ai`, where `InputType::default()` is `Shell`.

R2. WHEN `InputType::is_ai()` is called, the system shall return `true` for `Ai` and `false` for `Shell`.

R3. WHEN an `InputType` is rendered via `Display` and then parsed back via `FromStr`, the system shall yield the original variant; and IF the input string matches no variant, THEN `FromStr` shall return `Err`.

R4. WHEN `Tokens::parse(query)` is called, the system shall split `query` into whitespace-delimited tokens such that `len()` equals the token count and `first()` is the leading token, and `is_empty()` is `true` iff the trimmed query contains no tokens.

R5. WHEN `detect_input_type` is called with empty `Tokens`, the system shall return `InputClassificationResult { input_type: cx.current_input_type, source: DecisionSource::FallbackCurrentInput }`.

R6. WHEN `detect_input_type` is called and the first token satisfies `util::is_one_off_shell_command_keyword`, the system shall return `input_type: Shell` with `source: DecisionSource::ShellCommandAllowList`, regardless of the natural-language score.

R7. The system shall classify the leading tokens `claude`, `codex`, `gemini`, and `sudo` as one-off shell command keywords via `util::is_one_off_shell_command_keyword`.

R8. WHEN `detect_input_type` is called, the one-off shell allowlist (R6) does not match, and the first token satisfies `util::is_one_off_natural_language_word_or_prefix`, the system shall return `input_type: Ai` with `source: DecisionSource::NaturalLanguageOneOffAllowlist`.

R9. WHILE `cx.is_agent_follow_up` is `true`, WHEN `detect_input_type` is called and no shell allowlist keyword (R6) matches, the system shall return `input_type: Ai` with `source: DecisionSource::NaturalLanguageOneOffAllowlist`.

R10. WHEN `scorer::natural_language_words_score(words, is_first_token_command)` is called, the system shall return the count of natural-language tokens minus tokens carrying shell syntax, clamped at `0`, after **skipping the leading (head) token when `is_first_token_command` is `true`**, lowercasing, expanding contractions, and stemming each remaining token with the English stemmer.

R11. WHEN `scorer::check_if_token_has_shell_syntax(word)` is called, the system shall return `true` iff `word` contains at least one of the following explicit shell-metacharacters — `$`, `{`, `}`, `[`, `]`, `>`, `<`, `*`, `~`, `&`, `(`, `)`, `|`, `/`, `-` — and `false` otherwise. (The set is exactly these fifteen literal characters; `=` is **not** a member.)

R12. WHILE `words.len() > LOW_TOKEN_COUNT`, WHEN `HeuristicClassifier::detect_input_type` reaches the scoring path (no empty-input, allowlist, or follow-up rule fired), the system shall derive `is_first_token_command = util::is_likely_shell_command(tokens)` (evaluated on the head token), compute `ratio = natural_language_words_score(words, is_first_token_command) as f32 / words.len() as f32` — where the numerator (R10) skips the head token when `is_first_token_command` is `true` while the denominator is the full `words.len()`, so a leading command token lowers the ratio (the off-by-one is intentional and defined here) — and classify as `Ai` iff `ratio >= DETECT_AS_NATURAL_LANGUAGE_THRESHOLD` (`0.6`), otherwise `Shell`, stamping `source: DecisionSource::Heuristic`.

R13. WHILE `words.len() <= LOW_TOKEN_COUNT`, WHEN the scoring path runs (no empty-input, allowlist, or follow-up rule fired), the system shall derive `is_first_token_command` and `ratio` exactly as in R12 and classify as `Ai` iff `ratio >= LOW_TOKEN_THRESHOLD` (the stricter `0.8`), otherwise `Shell`, stamping `source: DecisionSource::Heuristic`. This is the **sole** scoring rule for the `words.len() <= LOW_TOKEN_COUNT` regime; R12 governs the disjoint `words.len() > LOW_TOKEN_COUNT` regime, so no `(words.len(), ratio)` pair satisfies both clauses (seam-contracts §9).

R14. WHEN `HeuristicClassifier::classify_input` is called on a non-empty input, the system shall return `Ok(ClassificationResult)` whose `to_input_type()` equals the `input_type` returned by `detect_input_type` for the same `Tokens` and `Context`.

R15. WHEN `ClassificationResult::confidence()` is called, the system shall return `max(p_shell, p_ai)`; and `to_input_type()` shall return `Ai` iff `p_ai > p_shell`, else `Shell`.

R16. WHEN `ClassificationResult::pure_shell(src)` is constructed, the system shall set `p_shell = 1.0`, `p_ai = 0.0`, `source = src`; and `pure_ai(src)` shall set `p_ai = 1.0`, `p_shell = 0.0`, `source = src`.

R17. IF the ML model path is invoked via `StubModelClassifier::classify_input`, THEN the system shall return `Err(ClassifierError::ModelUnavailable)` without panicking and without performing any model inference.

R18. WHEN `StubModelClassifier::detect_input_type` is called, the system shall delegate to an inner `HeuristicClassifier`, returning that classifier's `input_type` but overriding `source` to `DecisionSource::ModelStub`.

R19. The system shall embed no ONNX/model bytes and link no `candle`/`ort`/`tokenizers` dependency in the offline build, such that the crate compiles and all classification runs through the heuristic path with no model file present on disk.

R20. WHEN `util::is_agent_follow_up_input(tokens, cx)` is called, the system shall return `true` iff `cx.is_agent_follow_up` is `true` and the first token is not a one-off shell command keyword (R6).

R21. The system shall make `detect_input_type` deterministic: WHEN called twice with equal `Tokens` and equal `Context`, the system shall return identical `InputClassificationResult` values.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `InputType` has exactly `Shell`/`Ai`; default is `Shell` (R1) | planned |
| 2 | `is_ai()` true for `Ai`, false for `Shell` (R2) | planned |
| 3 | `Display`→`FromStr` round-trips; unknown string errors (R3) | planned |
| 4 | `Tokens::parse` splits on whitespace; `len`/`first`/`is_empty` correct (R4) | planned |
| 5 | Empty tokens → current input type, `FallbackCurrentInput` (R5) | planned |
| 6 | Shell-keyword first token → `Shell`, `ShellCommandAllowList`, score ignored (R6) | planned |
| 7 | `claude`/`codex`/`gemini`/`sudo` recognized as shell keywords (R7) | planned |
| 8 | NL-allowlist first token → `Ai`, `NaturalLanguageOneOffAllowlist` (R8) | planned |
| 9 | `is_agent_follow_up` + no shell keyword → `Ai` (R9) | planned |
| 10 | `natural_language_words_score` counts NL minus shell-syntax, clamp 0, skip head when command, stem (R10) | planned |
| 11 | `check_if_token_has_shell_syntax` true iff one of the 15 explicit metachars present; `=` not a member (R11) | planned |
| 12 | High-token regime (`> LOW_TOKEN_COUNT`): `is_first_token_command` derived via `is_likely_shell_command`; ratio = score/`words.len()`; `0.6` threshold; `Heuristic` source (R12) | planned |
| 13 | Low-token regime (`<= LOW_TOKEN_COUNT`): same derivation; stricter `0.8` threshold; sole rule, disjoint from R12 (R13) | planned |
| 14 | `classify_input` agrees with `detect_input_type` (R14) | planned |
| 15 | `confidence` = max; `to_input_type` = `Ai` iff `p_ai > p_shell` (R15) | planned |
| 16 | `pure_shell`/`pure_ai` set probabilities + source (R16) | planned |
| 17 | Stub model returns `Err(ModelUnavailable)`, no panic, no inference (R17) | planned |
| 18 | Stub `detect_input_type` delegates to heuristic, stamps `ModelStub` (R18) | planned |
| 19 | No model bytes/candle/ort/tokenizers; builds + runs heuristic-only (R19) | planned |
| 20 | `is_agent_follow_up_input` true iff follow-up and not shell keyword (R20) | planned |
| 21 | `detect_input_type` deterministic on equal inputs (R21) | planned |

## Visual / Behavioral Acceptance
N/A — non-UI decision engine with no window, pane, or AXUIElement surface. Its output drives the prompt's mode flip, which is asserted in the M1 prompt-routing integration spec, not here.

## Test Plan
- **Unit:** one `#[test]` per EARS clause, named for the requirement:
  - `r1_inputtype_variants_default_shell`, `r2_is_ai`, `r3_display_fromstr_roundtrip` (+ `r3_fromstr_unknown_errs`), `r4_tokens_parse_split`, `r5_empty_falls_back_to_current`, `r6_shell_keyword_forces_shell`, `r7_known_shell_keywords` (table over `claude`/`codex`/`gemini`/`sudo`), `r8_nl_allowlist_forces_ai`, `r9_agent_follow_up_forces_ai`, `r10_nl_score_counts_skips_head_and_stems`, `r11_shell_syntax_metachars` (table over the 15 explicit metachars + negatives, including a `=`-only token asserting `false`), `r12_high_token_threshold_0_6`, `r13_low_token_stricter_0_8`, `r14_classify_matches_detect`, `r15_confidence_and_to_input_type`, `r16_pure_shell_pure_ai`, `r17_stub_model_errs_no_panic`, `r18_stub_detect_delegates_stamps_modelstub`, `r19_no_model_bytes_heuristic_only`, `r20_is_agent_follow_up_input`, `r21_detect_is_deterministic`.
  - R7/R11 are table-driven over their full keyword/character sets, asserting both positive and negative cases so a shrunk set is caught; R11's negatives include `=` (must be `false`) to pin that `=` is not a member.
  - R12 fixtures use **> 2 tokens** (high-token regime) and straddle the `0.6` ratio (exactly-at-threshold, just-below, just-above); at least one fixture has a leading shell-command head so `is_first_token_command` is exercised and the numerator-skips-head / denominator-`words.len()` off-by-one is pinned. R13 fixtures use **1–2 tokens** (low-token regime) and straddle the `0.8` ratio. The disjoint-regime invariant (no `(len, ratio)` satisfies both) is asserted by a fixture with `len ∈ {1,2}` and `ratio ∈ [0.6, 0.8)` resolving to `Shell` under R13 only.
  - R19 is a build-level assertion: a `compile-time` test (`#[cfg(not(feature = "onnx"))]` default) plus a `cargo metadata` check in CI confirming `candle`/`ort`/`tokenizers` are absent from the dependency tree; a runtime test confirms classification succeeds with no model file on disk.
  - 100% line coverage on every touched line of the crate.
- **Integration:** the M1 unified-prompt routing seam — `detect_input_type` output flips a live session between Shell and Agent Mode — is named here for traceability and owned by the prompt-routing spec; this spec verifies the classifier contract in isolation with a fake `Context`. Per seam-contracts §3.5a there is **no** completer-snapshot seam test at M1.
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full suite plus the scorer fixture corpus (a frozen set of `(query, expected InputType, expected DecisionSource)` rows) must stay green; any threshold or allowlist change must update the corpus deliberately.

## Mutation Targets
`cargo mutants` must kill every viable mutant on the testable surface (MSI 100%):
- comparison-operator and constant mutants on the thresholds (`>=`↔`>`, `0.6`↔other, `0.8`↔other) and the **regime guard** (`LOW_TOKEN_COUNT` off-by-one, `>`↔`<=` swap between R12/R13) — killed by R12/R13 boundary fixtures and the disjoint-regime invariant test.
- the `is_first_token_command` derivation (replacing `util::is_likely_shell_command(tokens)` with a constant `true`/`false`) and the numerator-skips-head / denominator-`words.len()` basis — killed by the R12 leading-command-head fixture.
- the clamp-at-0 and subtraction in `natural_language_words_score` (`-`↔`+`, dropping the clamp, dropping the leading-head skip guard) — killed by R10.
- the metachar membership set in `check_if_token_has_shell_syntax` (dropping any of the 15 characters, adding `=`, negating the test) — killed by R11's full-set + `=`-negative table.
- short-circuit ordering of the allowlist/follow-up branches (swapping R6 before/after R8/R9, returning the wrong `DecisionSource`) — killed by R5/R6/R8/R9/R18 source assertions.
- `to_input_type` / `confidence` operator swaps (`>`↔`>=`, `max`↔`min`) — killed by R15.
- the stub model's `Err` arm replaced by `Ok` — killed by R17.
- MSI target: **100%** on the testable surface. ACCEPTED-UNTESTABLE: none in the offline build. The deferred `candle`/`ort` inference body is not present in this crate (stubbed), so there are no GPU/IO lines to exclude.

## Dependencies
- REUSE (permissive, MIT/Apache): `rust-stemmers` (English stemming for R10), `regex` (private contraction-expansion pattern used by R10), and `anyhow` (the `classify_input` error channel). No `heuristics` crate is used (it does not exist); the structured `reuses` field lists only the real crates so the `deny.toml` allowlist generates correctly.
- Marley components: **none** at M1. Per seam-contracts §3.5a the parser/`Tokens` type is provided **in-crate** (`Tokens::parse`) to avoid a hard dependency on the completer; the crate lists **no** dependency on `marley_completer` and does not accept `ParsedTokensSnapshot`. The shared-snapshot unification is a named **M2** item in this spec, `SPEC-completions`, and the build order.

## Out of scope / deferred
- The on-device ONNX model path (BERT-tiny via candle/ort, tokenizer embedding, panic-catching inference fallback) — deferred; this spec ships only `StubModelClassifier`. Picked up in a later milestone (M3+) behind an `onnx` feature.
- The completer's "is this describable as a real command?" signal and a shared `ParsedTokensSnapshot` input type — deferred to **M2** (seam-contracts §3.5a); M1 uses the in-crate `Tokens` parser and takes no `marley_completer` dependency.
- Non-English word lists / stemming and any model-evaluation binary — deferred.
- The prompt-UI mode-flip rendering and keybindings — owned by the M1 unified-prompt routing spec.

## Clean-room provenance
Behavior-derived from a fork-reference doc (`docs/specs/behavior/input-classifier.behavior.md`) describing observable I/O only — no AGPL/fork source read, no private module/type/static names, no fork file paths. The public surface uses Marley-original identifiers; the contraction-expansion regex and the reserved-keyword table are **private implementation details** and do **not** appear in the public surface (per seam-contracts §11.1, naming map: no `CONTRACTION_REGEX`/`RESERVED_KEYWORDS` on the contract). Threshold regimes and the `is_first_token_command` derivation conform to seam-contracts §9; the in-crate `Tokens`/no-completer-dep posture conforms to §3.5a. Per seam-contracts §11 the `clean_room` line is downgraded to `behavior-derived from a fork-reference doc; IP-counsel sign-off pending` until the behavioral wall + sign-off land (open item in `clean-build-plan.md`). REUSE crates (`rust-stemmers`, `regex`, `anyhow`) are MIT/Apache. The package is named `marley_input_classifier`. The `claude`/`codex`/`gemini`/`sudo` force-to-shell allowlist is retained as documented behavior but flagged for Marley's own agent-UX review.
