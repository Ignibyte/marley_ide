# languages

> Per-crate reference (Marley Zed-architecture map). Crate dir: `crates/languages`. Marley's editor
> surface is modeled on **Zed** (zed-industries/zed); this file is *architecture analysis* — the Zed
> clone lives only in session scratch, never committed — not vendored code. (Zed's `languages` and
> Warp's `languages` are unrelated crates that happen to share a name; this doc covers Zed's.)

| | |
|---|---|
| Subsystem | [04 — Language / Syntax / Tree-sitter](../subsystems/04-language-syntax-treesitter.md) |
| Zed license | `GPL-3.0-or-later` (explicit in `Cargo.toml`) |
| Provenance | **[Zed-derived]** wiring + LSP adapters, over **[permissive]** grammars/`.scm` assets (in the sibling `grammars` crate) |
| Internal deps | `grammars`, `language`, `lsp`, `project`, `terminal`, `node_runtime`, `task`, `snippet`, `theme`, `pet*`, `json_schema_store`, … |
| Used by (in Zed) | 9 (`zed`, `editor`, `agent_ui`, `markdown`, `repl`, `remote_server`, `edit_prediction`, `edit_prediction_cli`, `eval_cli`) |
| Marley target | a `marley_languages` init module (native-grammar registration; LSP adapters deferred) |

## Purpose

`languages` is the **built-in language registration + LSP-adapter layer** — the crate that, at
startup, wires every shipped language into the `LanguageRegistry`: which grammar, which `.scm`
queries, which LSP server(s), which task/context provider, which toolchain lister, which manifest
file. It is the *composition root* of the syntax/LSP stack.

Crucially, **`languages` does not contain the grammar assets** — those live in the sibling `grammars`
crate (embedded `config.toml` + `.scm` files + the compiled-in tree-sitter grammar list). `languages`
*consumes* `grammars::native_grammars()` / `load_config(name)` / `load_queries(name)` and adds the
orchestration on top: the `init(...)` wiring plus one Rust module per language (`rust.rs`, `python.rs`,
`go.rs`, …) holding that language's LSP adapter(s), context/task providers, toolchains, and manifest
providers.

## Key types, modules & public API

### `lib.rs` — the composition root
- **`pub fn init(languages: Arc<LanguageRegistry>, fs, node: NodeRuntime, cx: &mut App)`** — the entry
  point. It:
  1. `languages.register_native_grammars(grammars::native_grammars())` (gated on the `load-grammars`
     feature) — registers the compiled-in `tree_sitter::Language`s.
  2. Constructs each language's LSP adapters / providers (`RustLspAdapter`, `PyrightLspAdapter`,
     `GoLspAdapter`, `RustContextProvider`, `PythonToolchainProvider`, …).
  3. Iterates a `built_in_languages: [LanguageInfo; ~24]` table and calls `register_language(...)` per
     entry.
  4. Registers *available* (opt-in) LSP adapters (`tailwindcss-language-server`, `eslint`, `vtsls`,
     `typescript-language-server`) and default-attaches Tailwind/ESLint to a fixed language list.
  5. Registers manifest providers (`CargoManifestProvider`, `PyprojectTomlManifestProvider`) and a
     settings-sync task.
- **`struct LanguageInfo { name, adapters: Vec<Arc<dyn LspAdapter>>, context: Option<Arc<dyn
  ContextProvider>>, toolchain: Option<Arc<dyn ToolchainLister>>, manifest_name, semantic_token_rules }`**
  — the per-language registration record.
- **`fn register_language(...)`** — the funnel: `load_config(name)` → push semantic-token rules into
  the `SettingsStore` → register each LSP adapter → `languages.register_language(name, grammar, matcher,
  hidden, manifest, load_closure)` where the **`load` closure returns `LoadedLanguage { config:
  grammars::load_config(name), queries: grammars::load_queries(name), context_provider, toolchain_provider,
  manifest_name }`**. This is the lazy hook: the `.scm` files are only read + compiled when the language
  is first needed.
- **`pub use language::*`** — the crate re-exports the entire `language` public surface, so `languages`
  is a superset façade for consumers.
- The `built_in_languages` set (~24 entries): `bash, c, cpp, css, diff, go, gomod, gowork, json, jsonc,
  markdown, markdown-inline, python, rust, tsx, typescript, javascript, jsdoc, regex, yaml, gitcommit,
  zed-keybind-context` (plus available-only Tailwind/ESLint/vtsls adapters attached to more languages).

### Per-language modules — the LSP/task substance
Each file is that language's integration, not its grammar. The heavyweights: **`python.rs` (~123 k),
`rust.rs` (~89 k), `go.rs` (~72 k), `typescript.rs` (~65 k), `eslint.rs` (~40 k)**; plus `c.rs`,
`cpp.rs`, `css.rs`, `bash.rs`, `json.rs`, `yaml.rs`, `tailwind.rs`, `tailwindcss.rs`, `vtsls.rs`,
`package_json.rs`. A typical module contains:
- **`impl LspAdapter`** — binary discovery (`check_if_user_installed`, cached path), download/update
  (via `http_client` GitHub releases + `async-tar`/`async-compression` extraction to
  `node_runtime`-managed dirs), `initialization_options`, `workspace_configuration`,
  `label_for_completion`/`label_for_symbol` (building `CodeLabel`s), `code_action_kinds`,
  `disk_based_diagnostic_sources`.
- **`impl ContextProvider`** (e.g. `RustContextProvider`, `GoContextProvider`, `PythonContextProvider`)
  — resolves runnable captures + cwd/env into concrete task variables (`$ZED_*`-style), the bridge from
  a `runnable_ranges` node to a shell command.
- **`impl ToolchainLister`** (Python venvs via the `pet*` crates: conda/poetry/virtualenv discovery) and
  **`impl ManifestProvider`** (`CargoManifestProvider`, `PyprojectTomlManifestProvider`).
- Language-specific extras: `semantic_token_rules()` (rust/go/cpp/python), JSX/TSX auto-close config,
  `package_json` parsing for JS tooling.

### Features
- **`load-grammars`** (implied by `test-support`) — gates whether grammars are compiled in + the
  `tree-sitter`/`tree-sitter-gitcommit` deps; lets lighter builds skip the grammar payload. `load_config`
  falls back to `load_config_for_feature(name, grammars_loaded)` when grammars are absent.

## Depends on

**The asset crate:** `grammars` (sibling; the embedded `.scm`/`config.toml` + `native_grammars()`).
**The engine:** `language` (registry + `LoadedLanguage` + `LspAdapter`/`ContextProvider` traits).
**LSP/tooling:** `lsp`, `node_runtime` (manages downloaded servers/npm), `http_client`
(`github-download`), `async-fs`/`async-tar`/`async-compression` (fetch+extract servers), `project`,
`terminal` (task spawning), `task`, `snippet`, `json_schema_store`, `theme`, `settings`.
**Python env discovery:** `pet`, `pet-conda`, `pet-core`, `pet-fs`, `pet-poetry`, `pet-reporter`,
`pet-virtualenv`.
**Grammars (dev/test):** direct `tree-sitter-*` crates **[permissive: MIT/Apache]**.

## Used by (internal dependents)

9 crates — the top-level `zed` app (calls `languages::init`), `editor`, `agent_ui`, `markdown`, `repl`,
`remote_server`, and the edit-prediction/eval CLIs. It sits near the top of the stack: almost nothing
depends *on* `languages`; `languages` depends on nearly everything language-related.

## Related crates

- [language](./language.md) — the engine + registry this crate populates.
- [language_core](./language_core.md) — the `LanguageConfig`/`LanguageQueries`/`LspAdapter`-support types.
- `grammars` (sibling) — where the actual grammars + `.scm` + `config.toml` live **[permissive assets +
  Zed-derived loader]**. `languages` is the wiring; `grammars` is the data.
- [lsp](./lsp.md) — the LSP client each adapter drives; `node_runtime`, `project`, `terminal` — the
  server-management + task machinery.

## Provenance

| Element | Provenance | Marley action |
|---|---|---|
| Tree-sitter grammars (`tree-sitter-rust`/`-python`/`-go`/…) | **[permissive/public: MIT/Apache]** | Add the crates directly |
| The `.scm` query files + `config.toml` (in the `grammars` crate) | **[permissive/public: grammar-authors', mostly MIT/Apache]** | Vendor upstream or author own; not Zed-dependent |
| `grammars` `rust_embed` loader + `native_grammars()`/`load_config`/`load_queries` | **[Zed-derived: GPL]** (assets permissive) | Reimplement the loader; reuse the `rust_embed` *pattern* |
| `init` / `register_language` wiring + `LanguageInfo` table | **[Zed-derived: GPL]** | Reimplement (Marley's own init) |
| Per-language `LspAdapter` impls (binary discovery/download, init options, completion labels) | **[Zed-derived: GPL]** (the LSP *protocol* is **[permissive]**) | Reimplement per-language — a later LSP milestone, not the syntax engine |
| `ContextProvider`/task providers (runnable → shell command) | **[Zed-derived: GPL]** | Reimplement for the terminal-fusion feature |
| Toolchain/manifest providers (`pet*` Python discovery, Cargo/pyproject) | **[Zed-derived: GPL]** (`pet*` crates themselves are **[permissive]**) | Reimplement only what Marley needs |

## Reimplementation on Marley's stack

Marley needs a *thin* version of this crate — mostly the native-grammar registration, deferring the
whole LSP-adapter/download apparatus to a much later milestone.

- **v1 = registration only.** A `marley_languages::init(registry)` that mirrors the `grammars` embed
  pattern (`rust_embed` over a `grammars/` folder of `config.toml` + `.scm`) + a `native_grammars()`
  list + a `register_language` funnel installing a lazy `load` closure returning `{config, queries}`.
  **No LSP adapters, no `node_runtime`, no `http_client`, no `pet*`.** That strips the crate from ~30
  deps to a handful.
- **Grammar set (open question #4).** Start with the languages Marley already hand-lexes today —
  `rust`, `toml`, `json`, `shell/bash`, `markdown` — plus whatever the cockpit itself needs. Each added
  language is a grammar crate + a folder of `.scm`/`config.toml`, not bespoke Rust (contrast the hand
  lexer, where every language is new code). Binary size is the only cost knob.
- **The terminal-fusion seam is the payoff (later milestone).** The `ContextProvider` → task →
  `terminal` path is the built-in bridge from a syntax node (`runnable_ranges`) to a spawned command —
  exactly Marley's differentiator. When Marley builds the runnable gutter, this is the module to
  re-author (resolve `@run` captures + cwd/env into a cockpit terminal block). Confirm the exact
  cockpit spawn API + whether a `ContextProvider`-style resolver already exists (open question #3).
- **Skip:** the LSP-server download/update machinery, Python `pet*` toolchain discovery, semantic-token
  rules, Prettier/ESLint/Tailwind adapters — all deferrable until Marley pursues LSP intelligence
  (subsystem 05), which is a separate, later track from the tree-sitter syntax engine.

## Notes / gotchas

- **`languages` ≠ `grammars`.** The name suggests it holds the language data; it does not. Assets live
  in `grammars`; `languages` is orchestration + LSP. Keep this split in Marley — a pure asset crate
  (embeddable, testable, `[permissive]` payloads) separate from the wiring.
- **The `load` closure is where `.scm` compilation happens**, lazily, at first use of a language — not
  at `init`. `init` only *registers* the closure. Preserve that laziness (boot stays fast; unused
  grammars never compile).
- **Most of the crate's bulk is LSP, not syntax.** `python.rs`/`rust.rs`/`go.rs` are huge because of
  server management and completion labeling, none of which the tree-sitter syntax engine needs. Do not
  let the size of this crate scare the syntax-engine scope — the syntax part is the tiny
  `register_language` funnel.
- **`register_native_grammars` is feature-gated** (`load-grammars`); a build without it registers
  languages whose `load` will find no grammar. Marley's minimal build should either always compile
  grammars in or degrade to the `PLAIN_TEXT` fallback gracefully.
- **~24 built-in languages, but only ~21 grammars** — some entries (`gitcommit`, `zed-keybind-context`)
  and injections (`markdown-inline`, `jsdoc`) exist for embedding/tooling rather than as user-visible
  top-level languages (`config.hidden`).
