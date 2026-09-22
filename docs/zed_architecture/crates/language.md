# language

> Per-crate reference (Marley Zed-architecture map). Crate dir: `crates/language`. Marley's editor
> surface is modeled on **Zed** (zed-industries/zed); this file is *architecture analysis* — the Zed
> clone lives only in session scratch, never committed — not vendored code.

| | |
|---|---|
| Subsystem | [04 — Language / Syntax / Tree-sitter](../subsystems/04-language-syntax-treesitter.md) |
| Zed license | `GPL-3.0-or-later` (explicit in `Cargo.toml`) |
| Provenance | **[Zed-derived]** — the incremental engine + registry + feature interpreters, over **[permissive]** tree-sitter/gpui |
| Internal deps | ~30 (`language_core`, `gpui`, `lsp`, `text`, `sum_tree`, `clock`, `theme`, `fuzzy_nucleo`, `task`, `watch`, `settings`, `fs`, `http_client`, `rpc`, …) |
| Used by (in Zed) | **74 crates** — this is the foundational language crate the whole editor sits on (`editor`, `project`, `multi_buffer`, `vim`, `outline`, every LSP/AI consumer, `zed`) |
| Marley target | new `marley_syntax` (the `SyntaxMap`/registry engine) + `Buffer` feature methods on `crates/editor` |

## Purpose

`language` is the **engine**: it owns the live tree-sitter parse state for a buffer, re-parses
incrementally and off the main thread as edits land, lazily loads languages/grammars, and exposes the
whole family of structural editor features (highlight, indent, brackets, outline, injections,
runnables, selection-expand, text-objects) as methods on `Buffer`/`BufferSnapshot`. Per its own crate
doc, it "does **not** perform highlighting by itself — it only maps ranges in a buffer to colors";
the pixels are painted by the `editor`/gpui layer.

Three responsibilities, three cores:
1. **`SyntaxMap`/`SyntaxSnapshot`** (`syntax_map.rs`, ~2 k lines) — the incremental, multi-layer
   parser. The crown jewel and the primary invention Marley must re-author.
2. **`LanguageRegistry`** (`language_registry.rs`) — a lazy, async, versioned, theme-aware catalog of
   languages and grammars (native + wasm), with LSP-adapter registration.
3. **`Buffer` feature methods** (`buffer.rs`, ~230 k of source incl. tests) — one query-selector +
   capture-index interpreter per feature, plus the async `reparse` scheduling.

Around those sit the `Language`/`LanguageScope` model, the parser/cursor pools, per-language editor
settings, modeline parsing, diagnostics, outline/runnable value types, text-diff, and the collab
`proto` serialization.

## Key types, modules & public API

### `language.rs` (root, ~70 k) — the `Language` model + shared machinery
- **`struct Language { id: LanguageId, config: LanguageConfig, grammar: Option<Arc<Grammar>>,
  context_provider, toolchain, manifest_name }`** — the loaded language. Built with
  `Language::new(config, Option<ts_language>)` then a builder chain: `with_queries(LanguageQueries)`
  (compiles all `.scm` into the grammar's configs), `with_highlights_query`, `with_context_provider`,
  `with_toolchain_lister`, `with_manifest`. `set_theme(&SyntaxTheme)` rebuilds the grammar's
  `HighlightMap`; `grammar()`, `config()`, `default_scope()`, `lsp_id()`, `path_suffixes()`.
- **`struct LanguageScope { language: Arc<Language>, override_id: Option<u32> }`** — a language *in a
  scope* (e.g. inside a string/comment), resolving `line_comment_prefixes`, `block_comment`,
  `brackets`, `word_characters` through the active `LanguageConfigOverride`. This is how "comments are
  different inside a JSX expression" works.
- **`fn build_highlight_map(capture_names, &SyntaxTheme) -> HighlightMap`** — positional map of
  `theme.highlight_id(name).map(HighlightId::new)`; the once-per-`(language, theme)` resolution.
- **Parser/cursor pools:** `static PARSERS: Mutex<Vec<Parser>>`, `static QUERY_CURSORS`, and
  `with_parser(f)` / `with_query_cursor(f)`. `with_parser` pops-or-creates a `Parser` (each carrying a
  `wasmtime` WASM store from a shared `WASM_ENGINE` `LazyLock`), **`reset`s it** (cancelled parses
  leave live state that would otherwise *resume*), runs the closure, and returns it to the pool —
  amortizing allocation across many background parses.
- **`trait LspAdapter` + `struct CachedLspAdapter`** — the (heavy) LSP-adapter interface (binary
  discovery/download, `initialization_options`, `label_for_completion`, workspace config, etc.);
  `CachedLspAdapter` memoizes the resolved binary/capabilities. `FakeLspAdapter` for tests.
- **`static PLAIN_TEXT: LazyLock<Arc<Language>>`** — the always-present grammarless fallback.

### `syntax_map.rs` — the incremental multi-layer parser **(crown jewel, [Zed-derived])**
- **`struct SyntaxMap { snapshot: SyntaxSnapshot, language_registry }`** and **`struct SyntaxSnapshot
  { layers: SumTree<SyntaxLayerEntry>, parsed_version: clock::Global, interpolated_version,
  language_registry_version, update_count }`** — a **stack of tree-sitter trees** for one buffer,
  cheap to `Clone` (Arc-backed `SumTree` + Arc trees) so it can be handed to a background thread.
- **`struct SyntaxLayerEntry { depth, range: Range<Anchor>, content }`** with **`enum
  SyntaxLayerContent { Parsed { tree, language, included_sub_ranges }, Pending { language_name } }`** —
  depth 0 is the root language; depth 1+ are injections (a JS block in HTML, a code fence in Markdown);
  `Pending` is an injection whose grammar has not loaded yet.
- **The two-phase update:**
  - **`interpolate(&BufferSnapshot)`** — synchronous, cheap, main-thread. Applies a
    `tree_sitter::InputEdit` (byte + row/col deltas) to each affected layer's tree via `tree.edit(...)`,
    *shifting* node positions so captures stay approximately correct. **Does not re-parse.** Keeps the
    screen coherent between a keystroke and the async reparse.
  - **`reparse(language, &BufferSnapshot)` / `reparse_with_timeout(...)`** — the real parse; runs a
    depth-ordered `ParseStep` binary-heap queue, reusing an existing layer when its range/language is
    unchanged, else calling tree-sitter with `Some(old_tree)` for true incremental reuse. Discovers
    injections (`get_injections` over `injection_config`) and pushes child `ParseStep`s.
    **`reparse_with_timeout` returns `Err(ParseTimeout)`** when it exhausts its budget — cooperative
    yielding so a huge file never freezes a frame.
- **Injections:** simple (each range → its own child layer) vs **combined** (`injection.combined`:
  all captured ranges of one pattern feed one child tree via `included_ranges`); `splice_included_ranges`
  keeps the combined set current across edits. When the registry gains a language later,
  `contains_unknown_injections` promotes `Pending` layers.
- **Reading captures across layers (merged by position):** `captures(range, buffer, selector) ->
  SyntaxMapCaptures` (one at a time; highlighting), `matches(...)` / `matches_with_options(...,
  TreeSitterOptions)` → `SyntaxMapMatches` (whole matches; indent/brackets/outline/runnables), and
  `layers_for_range(...)` → raw `SyntaxLayer`s (a `tree_sitter::Tree` + offset, for AST-walk features).
  The uniform selector is `fn(&Grammar) -> Option<&Query>`. `TreeSitterOptions::max_start_depth` and
  `MAX_BYTES_TO_QUERY` bound the work.
- **`impl Drop for SyntaxSnapshot`** ships the `SumTree` to a dedicated **background drop thread**
  (deep tree-sitter `Tree` drops are slow and would block the main thread).

### `buffer.rs` — `Buffer`/`BufferSnapshot` + the feature family
The feature methods, each = pick a config's query with a selector, run `captures`/`matches`, interpret
each capture by comparing `capture.index` to the config's resolved `*_capture_ix`:

| Feature | Method | Config |
|---|---|---|
| Highlight | `chunks(range, styling)` → `BufferChunks` (yields `Chunk { text, syntax_highlight_id, … }`) | `highlights_config` |
| Auto-indent | `suggested_indents(rows, size)` | `indents_config` (+ declarative indent regexes) |
| Bracket match | `enclosing_bracket_ranges`, `innermost_enclosing_bracket_ranges`, `bracket_ranges` | `brackets_config` |
| Outline/breadcrumbs | `outline(theme)`, `outline_items_containing`, `symbols_containing` | `outline_config` |
| Injections (read side) | `syntax_layers`, `smallest_syntax_layer_containing` | `injection_config` |
| Runnables (test gutter) | `runnable_ranges` | `runnable_config` |
| Text objects | `text_object_ranges` | `text_object_config` |
| Redactions | `redacted_ranges` | `redactions_config` |
| Selection expand | `syntax_ancestor`, `syntax_prev/next_sibling` | *raw AST walk — no query* |
| Function-body fold | `function_body_fold_ranges` | reuses `text_object_config` (`InsideFunction`) |

- **Async scheduling:** `reparse(cx, may_block)` locks the map, `interpolate`s on the main thread,
  clones the snapshot, sets `ParseStatus::Parsing`, and `background_spawn`s `reparse_with_timeout`; on
  completion `did_finish_parsing` calls `syntax_map.did_parse(snapshot)` on the main thread and flips
  `ParseStatus::Idle`. A **`watch::channel<ParseStatus>`** lets consumers await a settled tree.
- Note there is **no `folds` query** — folding is computed in the `editor` layer (indent-derived + LSP
  ranges + creases); the only tree-sitter contribution is `function_body_fold_ranges` (subsystem §6.4).

### `language_registry.rs` — the lazy async catalog
- **`struct LanguageRegistry { state: RwLock<LanguageRegistryState>, executor, … }`** holding
  `languages: Vec<Arc<Language>>` (loaded), `available_languages: Vec<AvailableLanguage>` (registered,
  not yet parsed), `grammars: HashMap<Arc<str>, AvailableGrammar>`, `lsp_adapters`, `loading_languages`,
  `theme`, `version`, `reload_count`.
- **`struct AvailableLanguage`** — a *lazy* entry: name, grammar name, `LanguageMatcher`, and a
  **`load: Arc<dyn Fn() -> Result<LoadedLanguage>>`** closure; nothing is compiled until a file needs it.
- **`enum AvailableGrammar { Native(tree_sitter::Language), Unloaded(PathBuf), Loading(…), Loaded(…),
  LoadFailed(…) }`** — native (compiled-in) vs **wasm grammars loaded from disk at runtime** (the
  extension path).
- **`struct LoadedLanguage { config, queries, context_provider, toolchain_provider, manifest_name }`** —
  what a `load` closure returns; the registry then builds `Language::new(config, ts).with_queries(queries)`.
- **Resolution:** `language_for_file` / `language_for_file_path` (by `LanguageMatcher` with a
  user-configured > path/content precedence ladder), `language_for_name_or_extension` (injection
  targets), `load_language_for_file_path` (async; concurrent requests coalesce via `loading_languages`
  oneshot senders). **Registration:** `register_language`, `register_native_grammars`,
  `register_wasm_grammars`, `register_lsp_adapter`, `register_available_lsp_adapter`. **Lifecycle:**
  `set_theme(theme)` rebuilds every grammar's `HighlightMap`; each registration bumps `version` and
  pings `subscribe()`; `reload()` clears+reloads (extensions changed).

### Supporting modules
- **`outline.rs`** — `Outline<T> { items: Vec<OutlineItem<T>>, candidates, leaf_offsets }`,
  `OutlineItem<T> { depth, range, selection_range, text, highlight_ranges: Vec<(Range, HighlightStyle)>,
  body_range, annotation_range, … }`, an async fuzzy `search` (via `fuzzy_nucleo`) with synthetic
  `Ancestor` rows, and `find_most_similar`. Backs both go-to-symbol and breadcrumbs.
- **`runnable.rs`** — `RunnableRange { buffer_id, run_range, full_range, runnable: Runnable,
  extra_captures }`, `trait RunnableResolver`, `RunnableMatchCapture`, `ResolvedRunnable`. The
  syntax-node → terminal-command seam (the Marley-relevant "terminal fusion" hook).
- **`language_settings.rs`** (~46 k) — the big per-language editor settings model (tab size, soft wrap,
  format-on-save, inlay hints, edit-prediction toggles, `AutoIndentMode`, `IndentGuideSettings`, …).
- **`modeline.rs`** — vim/emacs modeline parsing (`parse_modeline`) feeding `LanguageMatcher`.
- **`proto.rs`** — RPC serialization of buffers/operations for collab.
- **`diagnostic.rs` / `diagnostic_set.rs`** — `Diagnostic` + `DiagnosticEntry`/`DiagnosticGroup` sets
  over anchored ranges.
- **`file_content.rs`** — byte-content analysis / encoding detection (`analyze_byte_content`).
- **`task_context.rs`** — `trait ContextProvider` (resolves runnable captures + cwd/env into a task).
- **`text_diff.rs`** — diff algorithms (`text_diff`, `unified_diff`, `word_diff_ranges`, …).
- **`toolchain.rs`** — `LanguageToolchainStore`, `trait ToolchainLister`.

## Depends on

**External / permissive:** `tree-sitter` **[MIT]** + `wasmtime` (wasm grammars), `gpui` **[Apache-2.0]**,
`regex`, `futures`, `smallvec`, `serde`, `streaming-iterator`, `imara-diff`/`diffy`.
**Internal (Zed):** `language_core` (the data layer), `text` (`Buffer`/`BufferSnapshot`, `Anchor`,
`Rope`), `sum_tree` (the layer `SumTree`), `clock` (`Global` versions), `theme` (`SyntaxTheme`),
`lsp`, `fuzzy_nucleo` (outline search), `task` (runnable tags), `watch` (`ParseStatus`), `settings`,
`fs`, `http_client`, `rpc`, `collections`, `util`.

## Used by (internal dependents)

**74 workspace crates** — this is the foundation of the whole editor. Representative: `editor`,
`multi_buffer`, `project`, `vim`, `outline` / `outline_panel`, `file_finder`, `project_symbols`,
`diagnostics`, `command_palette`, every LSP consumer, the AI/edit-prediction crates, `languages`, and
the top-level `zed`.

## Related crates

- [language_core](./language_core.md) — the pure data layer it wraps; read first.
- [languages](./languages.md) — populates this registry with built-in languages + LSP adapters.
- `grammars` (sibling) — the embedded `.scm`/`config.toml` assets + native grammar list.
- [text](./text.md) / [sum_tree](./sum_tree.md) / [clock](./clock.md) — the buffer + anchor +
  versioning substrate the `SyntaxMap` parses against (Marley counterpart: `crates/editor` + a
  versioning/anchor shim).
- [gpui](./gpui.md) — the render target; the highlight spans feed `StyledText::with_highlights`.
- [lsp](./lsp.md) — the LSP client the `LspAdapter` trait drives (a later, separate track).

## Provenance

| Element | Provenance | Marley action |
|---|---|---|
| tree-sitter runtime, `Parser`, `Tree`, `Query`, `QueryCursor`, `InputEdit`, `Node`, `TreeCursor`, wasmtime | **[permissive/public: MIT / Apache]** | Adopt directly |
| `SyntaxMap`/`SyntaxSnapshot` incremental multi-layer parse (`interpolate`/`reparse`, `ParseStep`, injections, combined ranges) | **[Zed-derived: GPL]** | Reimplement (`marley_syntax`) — the core effort |
| Parser/cursor pools, background drop thread, `ParseStatus` scheduling | **[Zed-derived: GPL]** | Reimplement |
| `LanguageRegistry` lazy async loader + `AvailableLanguage`/`AvailableGrammar`/`LoadedLanguage` | **[Zed-derived: GPL]** | Reimplement (native-only for v1; skip wasm) |
| `Language`/`LanguageScope` model, `build_highlight_map`/`set_theme` | **[Zed-derived: GPL]** | Reimplement |
| `Buffer` feature methods (`chunks`, `suggested_indents`, `outline`, `syntax_ancestor`, `runnable_ranges`, …) | **[Zed-derived: GPL]** | Reimplement as Marley `Buffer` methods |
| `Outline`/`OutlineItem`, `RunnableRange`/`RunnableResolver` value types | **[Zed-derived: GPL]** | Reimplement |
| `LspAdapter` trait / `CachedLspAdapter` | **[Zed-derived: GPL]** (LSP *protocol* is **[permissive]**) | Out of scope for the syntax engine; part of a later LSP milestone |
| gpui render (`StyledText::with_highlights`) | **[permissive/public: Apache-2.0]** | Use directly |

## Reimplementation on Marley's stack

The engine is `[Marley-original]` code implementing this `[Zed-derived]` design over `[permissive]`
tree-sitter. Target `marley_syntax` (mirrors `SyntaxMap`) + feature methods on `crates/editor`'s
`Buffer` (subsystem doc §9).

- **Seq 1 — whole-file synchronous highlight.** Stand up `Grammar` + `HighlightMap` (in
  `marley_grammar`), parse a whole file once, run the highlights query, and feed the resulting
  `(Range, HighlightStyle)` spans into `gpui::StyledText::with_highlights` (already Marley's render
  path in the `app.rs` code viewer). This alone beats the hand lexer (types/functions, multi-line
  strings). Pure `Grammar`/`HighlightMap` seams → cov/MSI 100; a shim renders the parsed spans.
- **Seq 2 — the `SyntaxMap` + incremental + off-thread (mandatory).** Add `interpolate`/`reparse`, the
  layer `SumTree`, the parser pool, background scheduling, `ParseStatus`. This is what keeps the
  cockpit at 120 fps while a 5 k-line file re-parses.
- **Later seqs** — selection-expand (`syntax_ancestor`, cheapest high-value, no query), then
  indent+brackets, then outline/breadcrumbs, then the runnable gutter (the flagship terminal-fusion
  feature: a `#[test]` node → a gutter run icon → a spawned cockpit terminal block).

**The buffer-substrate gap (open question #1).** Zed's `SyntaxMap` is built on `text`'s anchored
`SumTree` buffer with `clock::Global` versions and `Anchor` ranges — `interpolate`/`reparse` translate
`edits_since` into `InputEdit`s and store layer ranges as anchors. **Marley's `crates/editor` `Buffer`
is a pure `ropey::Rope` with a monotonic `BufferVersion(u64)` and `BufferDelta`** — it has no anchors
and no `edits_since` API. So Marley must either (a) add an anchor + edit-diff layer to its buffer, or
(b) start with a simpler offset-based layer model (recompute `InputEdit`s from `BufferDelta`, store
layer ranges as byte offsets, and re-shift them on edit). Option (b) is the pragmatic v1 — full
anchor-tracked incrementality (Zed-parity) can follow. Confirm this before Seq 2.

- **Native-only registry.** Marley's v1 `LanguageRegistry` needs only the `AvailableGrammar::Native`
  path (compiled-in grammars) and the lazy `load` closure; the wasm/`Unloaded`/extension machinery
  (`register_wasm_grammars`, `WasmStore`, `wasmtime`) can be deferred indefinitely — which also drops
  the `wasmtime` dependency and lets `with_parser` skip the WASM store.
- **Async executor (open question #5).** Wire `reparse` onto Marley's gpui `background_spawn` and
  marshal `did_parse` back to the main thread, mirroring the `ParseStatus` watch.

## Notes / gotchas

- **`interpolate` is not a parse.** It only shifts node positions so highlights don't visibly jump
  before the async reparse lands; captures over freshly-typed text are approximate until `did_parse`.
  Skipping `interpolate` (parsing synchronously on every keystroke) is the classic way to drop frames.
- **The parser must be `reset()` after a cancelled parse.** Tree-sitter auto-resets after a *successful*
  parse, but a cancelled/timed-out parse leaves state that would *resume* on the next call — hence the
  explicit `parser.reset()` + `set_included_ranges(&[])` in `with_parser`. Reproduce this in the pool.
- **Deep `Tree` drops block the thread.** Zed offloads `SyntaxSnapshot` drop to a background thread; a
  naive Marley port that drops trees on the main thread will hitch on closing a large file.
- **Budget/timeout is load-bearing, not optional.** `reparse_with_timeout` is what prevents a
  pathological file from freezing a frame; ship it with the incremental path, not later.
- **`buffer.rs` is ~230 k of source** (heavily test-laden). The feature methods are individually small
  and uniform (selector + capture-index interpretation) — port them one at a time, each as its own
  pipeline seq, rather than as one big module.
- **No folds query.** Do not model folding as a `.scm` deliverable; it is an editor-layer feature
  (indent-derived + creases), with only `function_body_fold_ranges` sourced from the text-objects query.
