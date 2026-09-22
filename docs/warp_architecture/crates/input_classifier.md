# input_classifier

> Per-crate reference (Marley round 2) — crate dir `crates/input_classifier`. Marley is forked from Warp (warpdotdev/warp).
> Provenance: [Warp-derived/AGPL] over [permissive/public: ONNX + heuristics] · Marley status: gap — reimplement the shell-vs-NL gate from concept; a front-door utility, NOT part of the sold brain's derivation chain.

| | |
|---|---|
| **Subsystem** | [agent-ai-mcp](../subsystems/04-agent-ai-mcp.md) |
| **License** | AGPL v3 |
| **Internal deps** | 3 (`natural_language_detection`, `warp_completer`, `warp_features`) |
| **Used by** | 1 (`warp`) |

## Purpose

The decision engine behind Warp's input prompt mode-switching: given what the user has typed, is it a **shell command** or a **natural-language query to the AI agent**? This is the core of the "smart prompt" that lets a single text field serve both the terminal and Agent Mode. It combines three signals — a baked-in **BERT-tiny ONNX classifier**, the lexical **`natural_language_detection`** scorer, and the **`warp_completer`** completion engine (can the tokens be described as a real command?) — plus hand-tuned one-off allowlists, and falls back gracefully (heuristic, then "keep current mode") if the model panics or errors.

## Key types, modules & public API

Public surface in `src/lib.rs`:

- **`pub trait InputClassifier: 'static + Send + Sync`** — the core abstraction. Two async methods, both taking a [`warp_completer::ParsedTokensSnapshot`](./warp_completer.md) and a `&Context`:
  - `detect_input_type(...) -> InputClassificationResult`
  - `classify_input(...) -> anyhow::Result<ClassificationResult>`
  - `#[async_trait]` on native, `#[async_trait(?Send)]` on wasm.
- **`pub enum InputType`** (`src/input_type.rs`) — `Shell` (default) or `AI`; implements `FromStr` and `Display`; `is_ai()`.
- **`pub struct InputClassificationResult`** — `{ input_type, source }`.
- **`pub enum InputClassifierDecisionSource`** — provenance of a decision: `InputClassifier`, `InputClassifierFallbackHeuristic`, `InputClassifierFallbackCurrentInput`, `NaturalLanguageOneOffAllowlist`, `ShellCommandAllowList`, `ShellHeuristic`.
- **`pub struct ClassificationResult`** — `p_shell`/`p_ai` probabilities + `source`; `confidence()`, `to_input_type()`, constructors `pure_ai`/`pure_shell`.
- **`pub struct Context`** — `{ current_input_type, is_agent_follow_up }`.
- **`pub struct HeuristicClassifier`** (`src/heuristic_classifier/mod.rs`) — the model-free implementation; weighs `natural_language_words_score` against token thresholds (`DETECT_AS_NATURAL_LANGUAGE_THRESHOLD = 0.6`, low-token `0.8`).
- **`pub OnnxModel` / `pub OnnxClassifier`** (`src/onnx/mod.rs`, behind the `onnx` feature) — `Model::{BertTinyV1,V2,V3}` selected via `rust-embed` from `models/onnx/`; runs inference through a `candle` or `ort` backend with panic-catching fallback (`HasPanicked`).
- **`pub mod util`** — one-off allowlists: `is_one_off_shell_command_keyword` (e.g. `claude`, `codex`, `gemini`, `sudo` — deliberately forced to shell), `is_one_off_natural_language_word_or_prefix`, `is_agent_follow_up_input`, `is_likely_shell_command`.
- **`pub mod test_utils`**, plus a `src/parser.rs` `parse_query_into_tokens(query)` helper and an `evaluate` binary (`src/bin/evaluate.rs`) for offline model evaluation.

## Depends on (internal)

- [`natural_language_detection`](./natural_language_detection.md) — supplies `natural_language_words_score`, the lexical AI-vs-shell signal used by `HeuristicClassifier`.
- [`warp_completer`](./warp_completer.md) — supplies `ParsedTokensSnapshot` (the parsed input) and the "is this describable as a command?" signal.
- `warp_features` — feature/flag gating (also a dev-dependency); decides which classifier variant is live.

## Used by (internal dependents)

- `warp` — the top-level application; wires the classifier into the input prompt to flip between Shell and AI modes. (Only dependent; no separate crate doc.)

## Related crates

- [`natural_language_detection`](./natural_language_detection.md) — the lexical scorer this crate orchestrates.
- [`warp_completer`](./warp_completer.md) — the completion engine providing the command-describability signal and the `ParsedTokensSnapshot` input type.
- `warp_features` — flag plumbing that selects classifier/heuristic versions.

## Marley relevance

**Classification: KEEP (likely STUB the ONNX path for first boot).**

This is the brain of the unified prompt and is squarely on **goal (1) expand the UI surface** and **goal (2) session spawn/write/read**: any Marley panel that routes a typed line to either a shell session or the agent will call `InputClassifier::detect_input_type`. Recommended path:

- **KEEP** the trait, `InputType`, `Context`, and `HeuristicClassifier` — they're offline, neutral, and self-contained.
- **STUB / defer the ONNX classifier** for the initial offline build: don't enable any `nld_classifier_v*` feature (no model bytes embedded), so the app uses `HeuristicClassifier` only. This avoids shipping ~MB BERT-tiny weights and the heavyweight `candle`/`ort`/`tokenizers` dep tree before Marley needs them.
- **De-Warp rebrand (goal 4):** the package name `input_classifier` is already vendor-neutral, but `util.rs` hard-codes a Warp-product behavior — `claude`, `codex`, `gemini` are force-classified as shell "because users think we're pushing them into Agent Mode." Marley should revisit that allowlist to match its own agent UX. The `InputClassifierDecisionSource::InputClassifier` naming is internal only.
- No auth coupling, so **goal (3)** is unaffected.

## Notes / gotchas

- `edition = "2024"` — newer than most sibling crates; needs a recent toolchain.
- **Feature minefield:** `onnx` is the base; `onnx_candle` vs `onnx_ort` pick the inference backend; `nld_classifier_v1/v2/v3` select *which* embedded `.onnx` bytes (enable exactly one); `nld_heuristic_v1/v2` select shell-heuristic behavior (v2 wins if both). Easy to misconfigure.
- Model + tokenizer (`bert_tiny_*.onnx`, `bert_tiny_tokenizer.json`) are embedded from `models/onnx/` via `rust-embed` at compile time — they must exist on disk to build with `onnx`.
- The ONNX runner catches panics and degrades through `InputClassifierFallbackHeuristic` → `InputClassifierFallbackCurrentInput`, so a broken model won't crash the prompt.
- wasm-aware: trait uses `?Send` async on wasm; `warp_completer`'s wasm path drops `embed-signatures`.
