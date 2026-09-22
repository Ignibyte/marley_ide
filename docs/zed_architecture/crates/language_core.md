# language_core

> Per-crate reference (Marley Zed-architecture map). Crate dir: `crates/language_core`. Marley's
> editor surface is modeled on **Zed** (zed-industries/zed); this file is *architecture analysis* —
> the Zed clone lives only in session scratch, never committed — not vendored code. Read alongside
> the subsystem overview it belongs to.

| | |
|---|---|
| Subsystem | [04 — Language / Syntax / Tree-sitter](../subsystems/04-language-syntax-treesitter.md) |
| Zed license | GPL-3.0 (crate carries a `LICENSE-GPL` symlink; `Cargo.toml` sets `publish = false`, no explicit `license =` field) |
| Provenance | **[Zed-derived]** — the glue over **[permissive/public: tree-sitter MIT]** primitives |
| Edition | 2024 |
| Internal deps | 5 (`collections`, `lsp`, `util`, `gpui_shared_string`, plus `tree-sitter`) |
| Used by (in Zed) | 2 direct (`grammars`, `language`) — the whole feature stack sits on top of `language` |
| Marley target | a new `marley_grammar` crate (the pure data layer) |

## Purpose

`language_core` is the **pure data layer** of the syntax engine: the types that describe *what a
language is* and *what its tree-sitter queries mean*, with **no parsing, no rendering, no async, and
no gpui runtime.** It is the smallest, most portable piece of the stack — it depends on the
`tree-sitter` crate for `Query`/`Language` and on `gpui_shared_string` for `SharedString`, but not on
`gpui`, the buffer/`text` crate, or any executor.

Its single most important job is `Grammar` + `populate_capture_indices`: it takes a compiled
tree-sitter `Query` (from a `.scm` source) and **resolves each `@capture` name to a numeric capture
index**, producing a per-feature "dispatch table" (`IndentConfig`, `OutlineConfig`, …). That
resolution is why the rest of the engine can, at highlight time, do a bare array index by capture id
instead of string comparison. Everything downstream (`language`'s `SyntaxMap`, the `Buffer` feature
methods) consumes these resolved configs.

## Key types, modules & public API

Root re-exports live in `src/language_core.rs`; the substance is in these modules.

### `grammar.rs` — the query/grammar core (the heart)
- **`struct Grammar`** — one `tree_sitter::Language` plus a bundle of `Option<*Config>` (one per
  feature) and a `Mutex<HighlightMap>`. Built by `Grammar::new(ts_language)` (which also compiles the
  built-in `"(ERROR) @error"` query into `error_query`), then hydrated by `with_queries(queries,
  &mut config)`.
- **The `*Config` structs** — each is a compiled `Query` + resolved capture indices:
  `HighlightsConfig { query, identifier_capture_indices }`, `IndentConfig { query,
  indent_capture_ix, start/end/outdent_capture_ix, suffixed_start_captures }`, `OutlineConfig
  { item/name/context/extra_context/open/close/annotation_capture_ix }`, `BracketsConfig { open/close
  + Vec<BracketsPatternConfig{newline_only, rainbow_exclude}> }`, `InjectionConfig { content/language
  capture + Vec<InjectionPatternConfig{language, combined}> }`, `RunnableConfig { extra_captures:
  Vec<RunnableCapture>, supports_grouped_runnables }`, `OverrideConfig { values: HashMap<u32,
  OverrideEntry> }`, `RedactionConfig`, `TextObjectConfig { text_objects_by_capture_ix }`,
  `DebugVariablesConfig`.
- **`enum RunnableCapture` = `Run | RunItem | Named(SharedString)`** and **`enum TextObject`**
  (`InsideFunction`/`AroundFunction`/`InsideClass`/…) / **`enum DebuggerTextObject`** — the capture
  vocabularies, resolved from capture names by `from_capture_name`.
- **`fn populate_capture_indices(query, name, query_type, expected_prefixes, &mut [Capture])`** — the
  shared resolver. Walks `query.capture_names()`, binds `Capture::Required("open", &mut ix)` /
  `Optional(...)` fields, **warns** on an unrecognized capture name (unless it starts with `_` or an
  allowed prefix like `"start."`) and **errors** if a `Required` capture is missing (returning `false`
  so that config is left `None`). This is the contract enforcement: a `.scm` must speak the capture
  vocabulary the engine expects.
- **`with_<feature>_query(source, name)`** methods — one funnel per feature; each does
  `Query::new(&ts_language, source)?` then `populate_capture_indices`. Some also scan
  `query.property_settings(ix)` for pattern-level flags (`newline.only`, `rainbow.exclude`,
  `injection.language`, `injection.combined`) or handle a `.inclusive` capture-name suffix (overrides).
- **`GrammarId`** (atomic counter) and `NEXT_GRAMMAR_ID`.

### `highlight_map.rs` — capture index → theme style
- **`struct HighlightMap(Arc<[Option<HighlightId>]>)`** — indexed **by capture index**. `get(capture_id)`
  is the hot-path array lookup.
- **`struct HighlightId(NonZeroU32)`** — an index into the theme's syntax style table.
  `HighlightId::new(capture_id)` stores `capture_id + 1` (so id 0 is representable); reserves
  `TABSTOP_INSERT_ID`/`TABSTOP_REPLACE_ID` at the top of the range. *Note:* the map is `Default`
  (empty `Arc<[]>`) until a theme is applied — `language_core` defines the type; the actual
  `capture_name → theme style` build (`build_highlight_map`) lives in `language` because it needs the
  `theme` crate.

### `language_config.rs` — the declarative (non-query) config
- **`struct LanguageConfig`** — deserialized from a `config.toml`: `name: LanguageName`, `matcher:
  LanguageMatcher` (flattened), `brackets: BracketPairConfig`, `line_comments`, `block_comment`,
  `documentation_comment`, indent regexes (`increase/decrease_indent_pattern`,
  `decrease_indent_patterns`), `overrides`, `word_characters`, `tab_size`/`hard_tabs`, `soft_wrap`,
  `autoclose_before`, `jsx_tag_auto_close`, `hidden`, list-continuation configs, etc.
- **`struct LanguageMatcher { path_suffixes, first_line_pattern: Option<Regex>, modeline_aliases }`** —
  the file→language matching criteria (also `Ord`, for precedence).
- **`struct BracketPairConfig` / `BracketPair { start, end, close, surround, newline }`** — the
  declarative bracket pairs used for **auto-close on typing** (independent of the tree-sitter
  brackets query), plus `disabled_scopes_by_bracket_ix` (which scopes disable a pair).
- **`struct LanguageConfigOverride` + `enum Override<T> { Remove{remove}, Set(T) }`** — scope-local
  overrides (e.g. "inside a string, line-comments differ"). `Override::as_option` resolves an override
  against the base value.
- Serde helpers: `deserialize_regex`, `serialize_regex`, `regex_json_schema`, … (regexes as strings
  with `schemars` JSON-schema support for settings UIs).

### `queries.rs` — the raw `.scm` sources + filename convention
- **`struct LanguageQueries`** — ten `Option<Cow<'static,str>>` fields: `highlights, brackets, indents,
  outline, injections, overrides, redactions, runnables, text_objects, debugger`.
- **`const QUERY_FILENAME_PREFIXES: &[(&str, QueryFieldAccessor)]`** — maps a filename prefix
  (`"highlights"` → `highlights.scm`) to the struct field. **This is the complete set — there is no
  `folds` entry** (Zed does not drive folding from a query; see subsystem doc §6.4).

### Supporting modules
- **`code_label.rs`** — `CodeLabel { text, runs: Vec<(Range, HighlightId)>, filter_range }` +
  `CodeLabelBuilder` + `Symbol` — a syntax-highlighted label for LSP completion/symbol items.
- **`language_name.rs`** — `LanguageName(SharedString)` (with `lsp_id()`, proto conversions) +
  `LanguageId` (atomic counter, identity for a loaded language).
- **`lsp_adapter.rs`** — the small shared LSP types: `trait ToLspPosition`, `PromptResponseContext`,
  `enum BinaryStatus`, `enum ServerHealth`, `LanguageServerStatusUpdate`. (The heavy `LspAdapter`
  *trait* lives in `language`, not here.)
- **`toolchain.rs`** — `Toolchain`, `ToolchainList`, `ToolchainScope` (Global/Project/Subproject),
  `ToolchainMetadata` — a language's associated tools (Python venv, Rust toolchain).
- **`diagnostic.rs`** — `Diagnostic` (LSP diagnostic model over a buffer range) + `DiagnosticSourceKind`.
- **`manifest.rs`** — `ManifestName` (e.g. `Cargo.toml`, `pyproject.toml`).

## Depends on

**External / permissive:** `tree-sitter` **[permissive/public: MIT]** (`Query`, `Language`), `regex`,
`serde`/`serde_json`/`toml`, `schemars`, `anyhow`, `log`, `parking_lot`.
**Internal (Zed):** `gpui_shared_string` (just the `SharedString` type — *not* full gpui), `collections`
(`HashMap`/`HashSet`/`IndexSet`), `lsp` (LSP data types), `util`.

Notably absent: `gpui`, `text`, `theme`, `sum_tree`, any async executor. This crate is deliberately a
leaf — it is the part of the stack that could be lifted with the least friction.

## Used by (internal dependents)

- **`language`** — the engine; wraps `Grammar` into `Language`, builds `HighlightMap` from a theme,
  and runs the configs against parse trees.
- **`grammars`** — the asset crate; `load_config(name)` returns a `LanguageConfig`, `load_queries(name)`
  returns a `LanguageQueries`.

(Only 2 direct dependents — but everything that uses `language` uses these types transitively; the
`language` crate re-exports most of `language_core`'s public surface.)

## Related crates

- [language](./language.md) — the engine that consumes these types; read them together.
- [languages](./languages.md) — the registration layer that produces `LanguageConfig`/`LanguageQueries`
  (via the sibling `grammars` crate) and feeds them to the registry.
- [lsp](./lsp.md) — the LSP data types this crate re-uses (`ToLspPosition`, `Diagnostic`, status enums).
- `grammars` (sibling, `[permissive assets + Zed-derived loader]`) — embeds the `config.toml` + `.scm`
  files and the compiled-in tree-sitter grammar list.

## Provenance

| Element | Provenance | Marley action |
|---|---|---|
| `tree_sitter::Query`, `Language`, capture-name convention (`@keyword`, `@item`, `@injection.content`, …) | **[permissive/public: tree-sitter MIT + grammar-authors' `.scm` vocabulary]** | Adopt the crate + the query vocabulary directly |
| `Grammar` struct + all `*Config` shapes + `populate_capture_indices` + `with_*_query` | **[Zed-derived: GPL]** | Reimplement in `marley_grammar` |
| `HighlightMap` / `HighlightId` (capture-index → theme id) design | **[Zed-derived: GPL]** | Reimplement |
| `LanguageConfig` / `LanguageMatcher` / `BracketPairConfig` / override schema | **[Zed-derived: GPL]** (the *schema*; the `config.toml` *data* is grammar-authored, permissive) | Reimplement the structs; the toml payloads are adoptable |
| `LanguageQueries` + `QUERY_FILENAME_PREFIXES` (the 10 query kinds) | **[Zed-derived: GPL]** | Reimplement (trivial) |
| `CodeLabel`, `Toolchain*`, `Diagnostic`, `LanguageName`, LSP status enums | **[Zed-derived: GPL]** (LSP types themselves are **[permissive]**) | Reimplement the wrappers Marley needs |

**Net:** the *primitives* (tree-sitter, the capture vocabulary, the `.scm`/`config.toml` data) are
permissive and adoptable; the *glue* (how a `Query` becomes a resolved dispatch table) is the GPL
surface Marley re-authors. It is a few hundred lines and mechanical — no algorithm to invent.

## Reimplementation on Marley's stack

Marley's target is a `marley_grammar` crate mirroring this one — the pure, gpui-free data layer of the
new syntax engine (see subsystem doc §9.1).

- **Adopt directly:** the `tree-sitter` crate and each grammar crate; the upstream `.scm` query files
  and their capture-name vocabulary; the `config.toml` payloads (declarative brackets/comments) where
  Marley wants them.
- **Re-author (clean-room from this description):** a `Grammar { ts_language, highlights_config,
  indents_config, brackets_config, outline_config, injection_config, runnable_config, … }` where each
  config is `Query` + resolved capture indices, built by a `populate_capture_indices`-style resolver;
  a `HighlightMap`/`HighlightId` pair; a `LanguageQueries` struct + filename map. Keep this crate free
  of gpui and async, exactly as Zed does — it makes the data layer trivially testable (pure seams,
  cov/MSI 100 per the pipeline norms).
- **Marley theme bridge `[Marley-original]`:** Zed resolves `capture_name → theme style` via
  `theme.highlight_id(name)`. Marley's theme model today maps ~5 `TokenKind`s; before this crate pays
  off, the Marley theme needs a **syntax-style sub-table** keyed by the ~40 tree-sitter capture names
  (open question #2 in the subsystem doc). Build `HighlightMap` once per `(language, theme)`, index by
  capture id at highlight time.
- **Scope:** start with the three configs that carry the most value — `highlights_config` (Seq 1),
  then `indents_config`/`brackets_config` and `outline_config` (later seqs). `override_config`,
  `redactions_config`, `debug_variables_config` are optional for v1.

## Notes / gotchas

- **Capture names are the API.** A `.scm` file that uses a capture name outside the expected
  vocabulary is silently *warned* (not fatal) and that capture is ignored; a missing `Required`
  capture leaves the whole config `None` (the feature silently disappears). When adopting an upstream
  query, verify its capture names match the resolver's expectations.
- **`with_injection_query` guards against ambiguity:** it accepts *either* `@content`/`@language` *or*
  `@injection.content`/`@injection.language`, and hard-errors (`anyhow::bail!`) if both spellings are
  present — a real trap when merging queries from different upstreams.
- **`HighlightId` is offset by +1** and stored `NonZeroU32`; `usize::from(HighlightId)` subtracts the 1
  back. Don't treat the stored value as a raw capture id.
- **No parsing here.** This crate compiles queries but never runs a parse — nothing needs a `Parser`,
  a buffer, or a thread. That is by design and worth preserving in `marley_grammar`.
- **`edition = "2024"` and `publish = false`** — it is an internal crate; the GPL license comes from
  the workspace `LICENSE-GPL` symlink, not a `Cargo.toml` field.
