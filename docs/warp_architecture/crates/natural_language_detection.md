# natural_language_detection

> Per-crate reference (Marley round 2) — crate dir `crates/natural_language_detection`. Marley is forked from Warp (warpdotdev/warp).
> Provenance: [Warp-derived/AGPL] over [permissive/public: word-lists + rust_stemmers] · Marley status: gap — reimplement the scorer from the public concept; trivial, not the brain.

| | |
|---|---|
| **Subsystem** | [agent-ai-mcp](../subsystems/04-agent-ai-mcp.md) |
| **License** | AGPL v3 |
| **Internal deps** | 0 |
| **Used by** | 2 (`input_classifier`, `warp`) |

## Purpose

A small, leaf-level lexical heuristic library that scores how "natural-language-like" a span of user input is. It is the cheap, dependency-free first stage of Warp's "is the user typing a shell command or talking to the AI agent?" decision. It owns no model and no async machinery — just dictionaries, a stemmer, and a scoring function. The crate exists so the (heavier) `input_classifier` and the main `warp` app can share the exact same word-level signal without dragging in ONNX, tokenizers, or the completion engine.

## Key types, modules & public API

Single-module crate; everything public lives in `src/lib.rs` (with the embedded dictionaries in the private `src/word_list.rs`).

- **`pub enum WordDb`** — selects which embedded dictionary to test against: `English`, `StackOverflow`, `Command`.
- **`pub fn is_word(word: &str, db: WordDb) -> bool`** — membership test against `WORD_LIST` / `STACK_OVERFLOW_LIST` / `COMMAND_LIST` (the three lists baked into `word_list.rs`).
- **`pub fn natural_language_words_score(words: Vec<Cow<str>>, is_first_token_command: bool) -> usize`** — the core entry point. Counts natural-language tokens minus tokens carrying shell syntax (clamped at 0). Skips a leading token if it's a known command, lowercases/expands contractions, stems with `rust_stemmers` (`Algorithm::English`), and penalizes unquoted tokens that contain shell metacharacters.
- **`pub fn check_if_token_has_shell_syntax(word: &str) -> bool`** — true if a single token contains any of `$ = { } [ ] > < * ~ & ( ) | / -` (from the Bash special-characters list).

Internals worth knowing: `CONTRACTION_REGEX` (reduces `he's`→`he`, `mustn't`→`must`, with `can't`→`can` special-cased), `RESERVED_KEYWORDS = ["what"]`, and `token_preprocessing` (lowercase + contraction expansion).

## Depends on (internal)

- None. This is a leaf crate (zero internal deps), depending only on external crates `rust-stemmers`, `lazy_static`, and `regex`.

## Used by (internal dependents)

- [`input_classifier`](./input_classifier.md) — its `HeuristicClassifier` calls `natural_language_words_score` to decide AI-vs-shell.
- `warp` — the top-level app binary (no separate crate doc; consumes the scorer directly).

## Related crates

- [`input_classifier`](./input_classifier.md) — the orchestrating consumer that wraps this scorer with the ONNX model and one-off allowlists.
- [`warp_completer`](./warp_completer.md) — supplies the *other* signal (whether tokens are describable by the completion engine) that `input_classifier` weighs against this one.

## Marley relevance

**Classification: KEEP.**

This crate is pure offline lexical logic with no Warp branding, no auth, no network, and no model files — it is exactly the kind of self-contained heuristic Marley wants to retain. It directly serves **goal (2) session spawn/write/read** by powering the AI-vs-shell decision on what the user types into a Marley session, and it has zero coupling to **goal (3) de-auth** or **goal (4) de-Warp rebrand** (no `warp_` in the package name, no telemetry). If Marley simplifies the classifier stack, this is the piece to keep as the baseline even if the ONNX path is stubbed out. The only naming touch-up: none required — the package name is already neutral.

## Notes / gotchas

- Dictionaries are compiled-in via `word_list.rs` (large generated `const`/`lazy_static` lists); changing vocabulary means regenerating that file, not editing JSON at runtime.
- English-only: the stemmer and word lists are hard-coded to English; non-English shell sessions get weaker signal.
- The scoring is intentionally lossy and "tunable" — it is a heuristic, not a classifier, and is meant to be combined with other signals rather than used alone.
- `version = "0.1.0"` / `edition = "2021"` (older edition than `input_classifier`'s 2024), so it's a stable, rarely-touched leaf.
