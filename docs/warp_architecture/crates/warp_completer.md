# warp_completer

> Per-crate reference (Marley round 2) — crate dir `crates/warp_completer`. Marley is forked from Warp (warpdotdev/warp).
> Provenance: [Warp-derived/AGPL] · Marley status: gap — terminal-side shell completion (feeds the input classifier), NOT the brain; belongs to the terminal layer, reimplement there.

| | |
|---|---|
| **Subsystem** | [agent-ai-mcp](../subsystems/04-agent-ai-mcp.md) |
| **License** | AGPL v3 |
| **Internal deps** | 8 (`command`, `fuzzy_match`, `string-offset`, `warp_cli`, `warp_core`, `warp_js`, `warp_util`, `warpui_core`) |
| **Used by** | 3 (`input_classifier`, `warp`, `warp_terminal`) |

## Purpose

The **completions engine** (the crate's own `description` says so): given a partial command line and cursor position, it parses the line into commands/tokens, looks up the relevant command **signatures**, and produces ranked, fuzzy-matched suggestions (flags, arguments, paths, variables, subcommands, aliases). It is what powers Warp's inline autocomplete dropdown. As a side product it exposes a structured parse of the input (`ParsedTokensSnapshot`), which is exactly what [`input_classifier`](./input_classifier.md) consumes to judge "is this a real command?". The engine ships in two generations — a `legacy` Rust path and an optional JS-driven **v2** (`feature = "v2"`, completions implemented in JS via `rquickjs`/`warp_js`).

## Key types, modules & public API

Top-level modules in `src/lib.rs`: `completer`, `meta`, `parsers`, `signatures`, `util`.

- **`pub struct ParsedTokensSnapshot`** (`src/lib.rs`) — `{ buffer_text: String, parsed_tokens: Vec<ParsedTokenData> }`; the cached parse of a buffer. This is the public hand-off type to `input_classifier`.
- **`pub struct ParsedTokenData`** — `{ token: meta::Spanned<String>, token_index: usize, token_description: Option<completer::Description> }`.
- **`completer` module** — the engine entry points and result types:
  - **`pub async fn completer::suggestions<T: CompletionContext>(line, pos, session_env_vars, options, ctx) -> Option<SuggestionResults>`** — the primary entry point a consumer calls.
  - **`pub trait CompletionContext`** (+ `GeneratorContext`, `PathCompletionContext`, `PathSeparators`, `CommandOutput`, `CommandExitStatus`) — host-supplied context (cwd, env, path semantics, command execution).
  - **`pub fn describe` / `describe_given_token` → `Description`** — classify a single token's role.
  - **`SuggestionResults`, `Suggestion`, `MatchedSuggestion`, `SuggestionType`, `SuggestionTypeName`, `Priority`, `CompleterOptions`, `CompletionsFallbackStrategy`** — suggestion output model.
  - **`Match`, `MatchStrategy`, `MatchType`** (re-exported from `matchers`) — fuzzy-match results.
  - JS path (feature `v2`): `JsExecutionContext`, `JsExecutionError`.
- **`parsers` module** — `classify_command`, `LiteCommand`, `LitePipeline`, `LiteGroup`, `LiteRootNode`, `ClassifiedCommand`, `ParsedToken`, `ParsedExpression`; the `simple` lexer/parser and an `hir`. (See `src/parsers/README.md`.)
- **`signatures` module** — command-signature registry; backend swapped by feature: `signatures/legacy` vs `signatures/v2` (path-cfg'd as `imp`), plus `signatures::clap` for clap-derived signatures. Spec data comes from the external `warp-command-signatures` crate (`embed-signatures` on non-wasm).
- **`meta` module** — `Spanned<T>` and span/offset utilities (uses `string-offset`).

## Depends on (internal)

- [`command`](./command.md) — non-wasm dependency providing the command/process model the engine parses and describes (also a `test-util` dev-dep).
- [`fuzzy_match`](./fuzzy_match.md) — fuzzy matching/ranking of suggestions against the typed prefix.
- [`string-offset`](./string-offset.md) — byte/char/UTF-16 offset math behind `Spanned` and span tracking.
- [`warp_cli`](./warp_cli.md) — CLI/command surface shared with the rest of Warp.
- [`warp_core`](./warp_core.md) — core shared types and runtime the completer integrates with.
- [`warp_js`](./warp_js.md) — *optional* (feature `v2`); JS runtime hosting for the JS completion engine.
- [`warp_util`](./warp_util.md) — shared utilities.
- [`warpui_core`](./warpui_core.md) — UI-core types (MIT-licensed) used to shape suggestions for display.

## Used by (internal dependents)

- [`input_classifier`](./input_classifier.md) — consumes `ParsedTokensSnapshot` + describability as a shell-vs-AI signal.
- `warp` — top-level app, drives the autocomplete UI.
- `warp_terminal` — terminal surface that renders/feeds completions.

## Related crates

- [`input_classifier`](./input_classifier.md) — the main non-UI consumer of this crate's parse output.
- `warp-command-signatures` — out-of-repo signature data embedded at build time (the dictionaries this engine completes against).
- [`warp_js`](./warp_js.md) — backs the optional `v2` JS completion engine.
- [`fuzzy_match`](./fuzzy_match.md) — the ranking primitive.

## Marley relevance

**Classification: KEEP, RENAME (deferred).**

The completion engine is genuinely useful and on **goal (1) expand the UI surface** (a Marley completions/suggestions panel) and **goal (2) session spawn/write/read** (it parses live session input). Keep the engine.

- **RENAME (deferred):** the package name carries the `warp_` prefix, which conflicts with **goal (4) de-Warp rebrand** (e.g. `marley_completer`). But it has **3 internal dependents** and pulls in 8 internal deps including the heavily-shared `warp_core`/`warp_cli`/`warpui_core`, so renaming should be batched with the broader `warp_*` → `marley_*` rename rather than done in isolation.
- **STUB the `v2` JS engine for first boot:** leave `feature = "v2"` off so `warp_js`/`rquickjs` and the JS completion runtime aren't compiled — the `legacy` Rust path gives working completions with a far smaller dependency surface during the offline-boot bring-up.
- **No auth coupling** — neutral to **goal (3) de-auth**; nothing here logs in or phones home, though signature *content* updates in stock Warp come from Warp's distribution (see gotchas).

## Notes / gotchas

- **Out-of-repo data dependency:** command signatures are supplied by the `warp-command-signatures` workspace crate and embedded via the `embed-signatures` feature on non-wasm targets; wasm builds deliberately drop it. Marley must ensure that crate is vendored or the engine completes against nothing.
- **wasm split:** the `[target.'cfg(target_family = "wasm")']` block omits the `command` dep and `embed-signatures`; the non-wasm block adds `command` and (optionally) `rquickjs`. Build behavior differs by target.
- **Two-generation codebase:** nearly every submodule has `legacy.rs` + `v2.rs` (parsers, suggest, signatures, engine argument/flag) selected by feature/path-cfg — expect duplicated logic and read the active path for the build you're targeting.
- `test-util` is a self-referential dev feature (`warp_completer = { path = ".", features = ["test-util"] }`) and forwards to `warp_js?/test-util`.
- `edition = "2021"`, `version = "0.1.0"`.
