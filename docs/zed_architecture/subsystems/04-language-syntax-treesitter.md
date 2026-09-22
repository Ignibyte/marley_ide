# Subsystem 04 — Language / Syntax / Tree-sitter

Part of the Marley **Zed** architecture docs (the editor reference; counterpart to
`../../warp_architecture/` which maps the terminal/cockpit). Zed is GPL-3.0; this file is
*architecture analysis* — Marley's own description of how the capability works and how Marley
rebuilds it. The Zed source lives only in session scratch, never committed.

> Scope: the crates that turn source text into a **tree-sitter parse tree** and then read a
> whole family of editor features off that one tree — `language` (the engine: `Buffer`
> feature methods, `SyntaxMap`, `LanguageRegistry`), `language_core` (the `Grammar` + query
> config + highlight map), `grammars` (embedded `config.toml` + `.scm` assets),
> `languages` (built-in language registration + LSP adapters), `language_extension` (the
> wasm-extension registration bridge), `outline` (the go-to-symbol modal), and
> `language_selector` (the manual language picker). This is the layer Marley must understand
> to replace its per-line hand lexer with a real syntax engine.

---

## 1. Purpose & big picture — one dependency, a family of features

The thesis of this subsystem: **you add a single dependency (tree-sitter), parse each file
once into a persistent tree, and then every structural editor feature is a small query or a
small AST walk over that one tree.** Highlighting, auto-indent, bracket matching, outline,
breadcrumbs, injected languages, runnable/test gutters, and syntax-aware selection are not
eight subsystems — they are eight *readers* of the same `SyntaxSnapshot`.

The crates split cleanly into four layers:

- **`language_core`** `[Zed-derived]` — the pure data layer. Defines `Grammar` (a
  `tree_sitter::Language` plus a bundle of per-feature query configs), the `LanguageConfig`
  (declarative brackets/comments/overrides), `LanguageQueries` (the raw `.scm` sources), and
  `HighlightMap`/`HighlightId` (capture-index → theme-style index). No parsing happens here;
  it is types + capture-index resolution. Depends only on `tree-sitter`, `lsp` types, serde.
- **`language`** `[Zed-derived]` — the engine. `SyntaxMap`/`SyntaxSnapshot` (the incremental,
  multi-layer parser), `LanguageRegistry` (lazy async language/grammar loading), and the big
  `Buffer`/`BufferSnapshot` whose methods are the feature interpreters (`chunks`,
  `suggested_indents`, `bracket_ranges`, `outline`, `syntax_ancestor`, `runnable_ranges`, …).
- **`grammars` + `languages`** `[mixed]` — the data + registration. `grammars` embeds each
  language's `config.toml` and `*.scm` files (`rust_embed`) and lists the compiled-in
  tree-sitter grammar functions; `languages` wires them into the registry alongside LSP
  adapters, task/context providers, and toolchains.
- **`language_extension`, `outline`, `language_selector`** `[Zed-derived]` — the edges.
  `language_extension` lets wasm extensions register new grammars/languages through the same
  registry API; `outline`/`language_selector` are gpui modal UIs that consume the tree.

Crucially, `language` does **not** paint anything. Per its own crate doc: it "only maps
ranges in a buffer to colors." The pixels are drawn by the `editor`/gpui layer — which for
Marley means the highlight spans feed straight into gpui's `StyledText::with_highlights`
(§9). And a single file is **not** one language: HTML-with-JS, Markdown-with-code-fences,
Rust-with-doc-comments are all modeled as a *stack of syntax layers* (§4.3).

---

## 2. Provenance & licensing posture (READ THIS FIRST)

This subsystem is the cleanest case in the whole Zed map for the open-core boundary, because
**the load-bearing engine — tree-sitter and the grammars — is permissive, not GPL.**

- **`[permissive/public]` tree-sitter itself = MIT.** The parser runtime, the incremental
  parse algorithm, `Query`, `QueryCursor`, `InputEdit`, `Tree`, `Node`, `TreeCursor` — all
  MIT. Marley adopts the `tree-sitter` crate directly, same as Zed and Warp do.
- **`[permissive/public]` the grammars = per-language MIT/Apache.** `tree-sitter-rust`,
  `tree-sitter-python`, `tree-sitter-typescript`, etc. are independent crates by the grammar
  authors. Marley adds them directly.
- **`[permissive/public]` the `.scm` query files = the grammar authors', not Zed's.** The
  highlight/indent/bracket/outline/injection queries are overwhelmingly upstream tree-sitter
  query files (MIT/Apache), the same corpus every tree-sitter editor ships. A handful are
  Zed-authored refinements; those specific files, if adopted verbatim, carry Zed's license —
  but the capability does **not** depend on Zed's specific `.scm` text, and Marley can vendor
  upstream queries or write its own. **The queries are data, not Zed's engine.**
- **`[Zed-derived]` the integration glue = GPL.** What is genuinely Zed's invention here is
  the *wiring*: the `SyntaxMap` incremental multi-layer re-parse algorithm, the `Grammar`
  struct that binds one `Query` to resolved capture-index fields, `HighlightMap`, the
  `LanguageRegistry` lazy-load state machine, and every `Buffer` feature method. This is the
  copyleft surface. Marley reimplements these in its own code (clean-room from this
  description), which is exactly what the Marley model already does for the terminal layer.

**Net:** Marley gets ~90% of the value (the parser + grammars + queries) for free under
permissive licenses, and only has to *re-author* the orchestration layer — a few thousand
lines of well-understood glue — rather than invent an algorithm. Marley's own integration
into its `editor`/gpui code is `[Marley-original]`.

---

## 3. The `Grammar` + query model (`language_core`)

`crates/language_core/src/grammar.rs` is the heart of the query/grammar handling. `[Zed-derived]`

### 3.1 `Grammar` = one tree-sitter language + a bundle of feature configs

```
pub struct Grammar {
    id: GrammarId,
    pub ts_language: tree_sitter::Language,     // the compiled grammar
    pub error_query: Option<Query>,             // "(ERROR) @error" — built in new()
    pub highlights_config:  Option<HighlightsConfig>,
    pub brackets_config:    Option<BracketsConfig>,
    pub indents_config:     Option<IndentConfig>,
    pub outline_config:     Option<OutlineConfig>,
    pub injection_config:   Option<InjectionConfig>,
    pub override_config:    Option<OverrideConfig>,
    pub redactions_config:  Option<RedactionConfig>,
    pub runnable_config:    Option<RunnableConfig>,
    pub text_object_config: Option<TextObjectConfig>,
    pub debug_variables_config: Option<DebugVariablesConfig>,
    pub highlight_map: Mutex<HighlightMap>,
}
```

Each `*Config` is the same shape: a **compiled `tree_sitter::Query`** plus a set of
**resolved capture indices** telling the consumer what each `@capture` means. For example
`IndentConfig { query, indent_capture_ix, start_capture_ix, end_capture_ix,
outdent_capture_ix, suffixed_start_captures }` and `OutlineConfig { query, item_capture_ix,
name_capture_ix, context_capture_ix, open_capture_ix, close_capture_ix,
annotation_capture_ix, … }`. So the grammar is a *dispatch table*: "capture index 3 in the
indents query means `@outdent`."

### 3.2 Loading a query: `with_<feature>_query` + `populate_capture_indices`

`Grammar::with_queries(queries: LanguageQueries, config)` is the funnel: for each present
`.scm` source it calls the matching `with_*_query(source, name)`, which compiles
`Query::new(&ts_language, source)` and then resolves capture names to indices. The shared
helper `populate_capture_indices(query, name, query_type, expected_prefixes, captures)`
walks `query.capture_names()` and, for each config, binds `Required("open", &mut ix)` /
`Optional("context", &mut Option<ix>)` fields; it **warns** on an unrecognized capture name
(unless it starts with `_` or an allowed prefix) and **errors** if a required capture is
missing. This is why a language's `.scm` files must use the capture-name vocabulary Zed
expects (`@item`, `@name`, `@indent`, `@run`, `@injection.content`, …) — the names are the
API between the query author and the engine.

Some configs read **pattern-level properties** too. Brackets scan
`query.property_settings(ix)` for `newline.only` / `rainbow.exclude`; injections read
`injection.language` / `injection.combined`; overrides handle a `.inclusive` suffix and
cross-check against the declarative `LanguageConfig.overrides`.

### 3.3 `LanguageQueries` and the filename→field map

`crates/language_core/src/queries.rs` defines the raw-source struct and the filename
convention:

```
pub struct LanguageQueries {
  highlights, brackets, indents, outline, injections,
  overrides, redactions, runnables, text_objects, debugger : Option<Cow<'static,str>>
}
pub const QUERY_FILENAME_PREFIXES: &[(&str, QueryFieldAccessor)]  // "highlights" -> |q| &mut q.highlights, …
```

Note the exhaustive list: **there is no `folds` entry.** Zed does *not* drive code folding
from a `folds.scm` query (see §6.4) — an important correction to the naïve "every feature has
its own `.scm`" model. The ten query kinds above are the complete set.

### 3.4 `HighlightMap` — capture index → theme color

`crates/language_core/src/highlight_map.rs` `[Zed-derived]`. A `HighlightMap` is
`Arc<[Option<HighlightId>]>` indexed **by capture index**; `HighlightId(NonZeroU32)` is an
index into the theme's syntax style table. It is built by `build_highlight_map(capture_names,
theme)` (`language.rs`) = for each capture name, `theme.highlight_id(name).map(HighlightId::new)`,
stored positionally. So the resolution "capture name `@keyword` → theme style" happens **once
per (language, theme)** at `Language::set_theme`, and at highlight time the hot path is a bare
array index by capture id — no string work. On a theme change the registry rebuilds every
grammar's map.

---

## 4. The syntax map — incremental, multi-layer, off-main-thread (`language/src/syntax_map.rs`)

This ~2 k-line file is the crown jewel and the primary `[Zed-derived]` invention Marley must
re-author. It maintains, for one buffer, a **stack of tree-sitter trees** and keeps them
current across edits without re-parsing the whole file or blocking the UI.

### 4.1 The layer tree

```
pub struct SyntaxSnapshot {
    layers: SumTree<SyntaxLayerEntry>,   // ordered by (depth, range, language)
    parsed_version:  clock::Global,      // buffer version this tree reflects
    interpolated_version: clock::Global, // version after cheap edit-shifting
    language_registry_version: usize,
    update_count: usize,
}
```

Each `SyntaxLayerEntry { depth, range: Range<Anchor>, content }` is one parsed region.
`content` is `Parsed { tree, language, included_sub_ranges }` or `Pending { language_name }`
(an injection whose grammar has not loaded yet). Depth 0 is the root language; depth 1+ are
injections (a code fence, a JS block in HTML), and injections can nest. Layers are stored in a
`SumTree` (Marley's `editor` crate already has the equivalent rope/sum-tree) keyed on a
`SyntaxLayerSummary` so the engine can seek to "all layers overlapping byte range R at depth
D" in O(log n). `SyntaxSnapshot` is cheap to `Clone` (Arc-backed SumTree + Arc trees) — that
clone is what gets handed to the background thread.

### 4.2 The two-phase update: `interpolate` then `reparse`

The key performance idea is that a keystroke does **not** immediately re-parse. Two phases:

1. **`interpolate(text)` — synchronous, main-thread, cheap.** For every buffer edit since
   `interpolated_version`, it applies a `tree_sitter::InputEdit` (byte + row/col deltas) to
   each affected layer's tree via `tree.edit(&input_edit)`. This *shifts node positions* so
   existing highlight/outline captures stay approximately correct, but does **not** re-parse.
   It walks the layer SumTree with a cursor, preserving layers before the first edit verbatim
   and only touching layers the edit intersects. This is what keeps the screen coherent in the
   window between a keystroke and the async reparse finishing.

2. **`reparse(text, registry, root_language)` — the real parse, run in the background.** It
   collects the changed byte-ranges since `parsed_version` and runs a **depth-ordered
   `ParseStep` queue** (a binary heap). For each step it: reuses the existing layer if its
   range/language is unchanged and it doesn't intersect a `ChangedRegion`; otherwise calls
   `parse_text(grammar, rope, start, included_ranges, Some(old_tree), budget)` — a tree-sitter
   parse that **passes the old tree** so tree-sitter reuses unchanged subtrees (true
   incremental parsing). Discarded layers insert a `ChangedRegion` at `depth+1` so their
   injected children are re-evaluated. After parsing a layer it calls `get_injections(...)`
   with the grammar's `injection_config` to discover `@injection.content` captures and pushes
   child `ParseStep`s for each, resolving the target language from the registry (or queuing a
   `Pending` layer if that grammar isn't loaded yet).

### 4.3 Injections (embedded languages)

Injections are how "one file = many languages" is modeled. The `injection_config` query
captures a content range (`@injection.content`) and a language name (either a static
`(#set! injection.language "javascript")` property or a dynamic `@injection.language`
capture). `get_injections` turns each into a child `ParseStep`. Two flavors:

- **Simple injection** — each captured range becomes its own child layer/tree.
- **Combined injection** (`injection.combined`) — *all* captured ranges of one pattern feed a
  **single** child tree with multiple `included_ranges` (this is how, e.g., all the code lines
  of one heredoc or all fragments of a templating language parse as one coherent tree).
  `splice_included_ranges` + `insert_newlines_between_ranges` maintain the combined range set
  incrementally as edits land.

When the registry gains a language later (an extension loads), `reparse` notices
`language_registry_version` changed, finds `Pending` layers with `contains_unknown_injections`,
and promotes them.

### 4.4 Off-main-thread scheduling (in `Buffer::reparse`, `language/src/buffer.rs`)

The async wiring lives on `Buffer`:

- `reparse(cx, may_block)` locks the syntax map, calls `interpolate` (cheap, main thread),
  clones the interpolated `SyntaxSnapshot` and a `BufferSnapshot`, sets
  `ParseStatus::Parsing`, and `cx.background_spawn`s a task that calls
  `snapshot.reparse_with_timeout(text, registry, root_language, budget)`.
- **`reparse_with_timeout` returns `Err(ParseTimeout)`** if it exhausts its time budget —
  cooperative yielding so a huge file never freezes a frame. On timeout the buffer keeps the
  interpolated (shifted) tree serving reads and spawns a follow-up task that waits for the full
  parse to land.
- On completion, `did_finish_parsing` calls `syntax_map.did_parse(snapshot)` on the main
  thread and flips `ParseStatus` back to `Idle`. A `watch::channel<ParseStatus>` (`Idle` /
  `Parsing`) lets consumers await a settled tree.
- Deep tree-sitter `Tree` drops are slow, so `SyntaxSnapshot::drop` **ships the SumTree to a
  dedicated background drop thread** rather than deallocating on the main thread.

### 4.5 Shared parser/cursor pools

`language.rs` keeps `static PARSERS: Mutex<Vec<Parser>>` and `static QUERY_CURSORS:
Mutex<Vec<QueryCursor>>`. `with_parser(f)` pops or creates a `Parser` (each carrying a
`wasmtime` WASM store for wasm grammars), `reset`s it (cancelled parses leave state), runs the
closure, and returns it to the pool. This amortizes parser allocation across the many
background parses. `with_query_cursor(f)` does the same for `QueryCursor`.

### 4.6 Reading captures across layers

Every feature (§6) consumes one of these `SyntaxSnapshot` iterators, which **merge captures
from all overlapping layers by position** so a consumer sees one flat, ordered stream even
across injections:

- `captures(range, buffer, |grammar| Option<&Query>)` → `SyntaxMapCaptures` (one capture at a
  time; used by highlighting).
- `matches(range, buffer, selector)` / `matches_with_options(…, TreeSitterOptions)` →
  `SyntaxMapMatches` (whole pattern matches with all their captures; used by indent, brackets,
  outline, runnables, text-objects). `TreeSitterOptions::max_start_depth` and
  `MAX_BYTES_TO_QUERY (16 KiB)` bound the work.
- `layers_for_range(range, buffer, include_hidden)` → raw `SyntaxLayer`s (each exposes a
  `tree_sitter::Tree` + offset) for the AST-walk features (selection expand, §6.8).

The selector closure `fn(&Grammar) -> Option<&Query>` is the uniform way a caller says "run
*this* feature's query" — e.g. `|g| g.outline_config.as_ref().map(|c| &c.query)`.

---

## 5. The language registry / extension model (`language/src/language_registry.rs`)

`[Zed-derived]`. The registry is a lazy, async, versioned, theme-aware catalog.

### 5.1 State

```
pub struct LanguageRegistry { state: RwLock<LanguageRegistryState>, executor, … }
struct LanguageRegistryState {
    languages: Vec<Arc<Language>>,                 // fully-loaded
    available_languages: Vec<AvailableLanguage>,   // registered but not yet loaded
    grammars: HashMap<Arc<str>, AvailableGrammar>, // by grammar name
    lsp_adapters, available_lsp_adapters, loading_languages,
    theme, version, reload_count, …
}
```

- **`AvailableLanguage`** is a *lazy* entry: name, grammar name, `LanguageMatcher`
  (path suffixes / first-line pattern / modeline aliases), and a
  **`load: Arc<dyn Fn() -> Result<LoadedLanguage>>`** closure. Nothing is parsed until a file
  actually needs the language.
- **`AvailableGrammar`** is `Native(tree_sitter::Language)` (compiled in) or
  `Unloaded(PathBuf)` / `Loading` / `Loaded` / `LoadFailed` for **wasm grammars loaded from
  disk at runtime** (the extension path).
- **`LoadedLanguage { config, queries, context_provider, toolchain_provider, manifest_name }`**
  is what a `load` closure returns; the registry then builds `Language::new(config,
  ts_language).with_queries(queries)` — i.e. it compiles the `.scm` sources into the grammar's
  `*Config`s at first use.

### 5.2 Resolution & lifecycle

- **Selection:** `language_for_file` / `language_for_file_path` match by `LanguageMatcher`
  (extension, first-line regex, modeline) with a precedence ladder
  (user-configured > path/content). `language_for_name_or_extension` handles injection targets.
- **Async load:** `load_language_for_file_path` returns a future; concurrent requests for the
  same language coalesce via `loading_languages` oneshot senders. Grammar wasm files load on
  the background executor.
- **Versioning:** every registration bumps `version` / `reload_count` and pings a
  `watch` subscription. `SyntaxSnapshot` stores the `language_registry_version` it parsed
  against so it knows when to retry `Pending` injections (§4.3). `reload()` clears and reloads
  everything (used when extensions change).
- **Theme:** `set_theme(theme)` stores it and rebuilds every grammar's `HighlightMap` (§3.4).

### 5.3 Built-in languages: the `grammars` + `languages` crates

- **`grammars`** `[mixed]` embeds the assets: `#[derive(RustEmbed)] #[folder = "src/"]` over
  per-language folders each holding a `config.toml` and `*.scm` files. `load_config(name)`
  parses `<name>/config.toml` → `LanguageConfig`; `load_queries(name)` iterates the embedded
  files and, per `QUERY_FILENAME_PREFIXES`, reads `<name>/highlights.scm` etc. into
  `LanguageQueries` (concatenating multiple same-prefix files). `native_grammars()` returns the
  compiled-in list — `("rust", tree_sitter_rust::LANGUAGE.into())`, `("python", …)`, tsx,
  json, markdown + markdown-inline, yaml, bash, c/cpp, css, go/gomod, regex, jsdoc, gitcommit.
  The `config.toml` + `.scm` payloads are `[permissive/public]` (upstream); the loader code is
  `[Zed-derived]`.
- **`languages`** `[Zed-derived]` `init(registry, fs, node, cx)` calls
  `register_native_grammars(grammars::native_grammars())` then, per language, a
  `register_language(name, adapters, context, toolchain, manifest, …)` that installs a `load`
  closure returning `LoadedLanguage { config: grammars::load_config(name), queries:
  grammars::load_queries(name), … }` — and separately registers **LSP adapters** (rust-analyzer,
  pyright/ty/ruff, gopls, clangd, ts/vtsls, eslint, tailwind, yaml), **context/task providers**
  (which is where runnables get resolved into shell commands, §6.7), and **toolchain listers**.

### 5.4 Extensions (`language_extension`)

`[Zed-derived]` a thin bridge: `init(...)` registers a `LanguageServerRegistryProxy` with the
extension host. `ExtensionGrammarProxy::register_grammars(Vec<(name, PathBuf)>)` →
`registry.register_wasm_grammars` (so a wasm-compiled grammar becomes an `Unloaded` entry);
`ExtensionLanguageProxy::register_language(name, grammar, matcher, hidden, load)` →
`registry.register_language` (the **same** API the built-ins use, but the `load` closure reads
the extension's files). `extension_lsp_adapter.rs` bridges extension-provided LSP servers.
**The extension model is just: an extension is another caller of `register_language` /
`register_wasm_grammars`.** For Marley this is optional — a v1 can ship only native grammars
and skip the wasm/extension machinery entirely.

---

## 6. The feature family — one tree, many readers

All feature methods live on `Buffer`/`BufferSnapshot` (`language/src/buffer.rs`) and follow
one pattern: pick a config's query with a selector closure, run `captures`/`matches` over the
`SyntaxSnapshot`, and interpret each capture by comparing `capture.index` to the config's
resolved `*_capture_ix` fields. The exceptions are the AST-walk features (§6.8), which take
raw `TreeCursor`s. `[all Zed-derived]` (the interpretation logic; the queries are public).

| Feature | `Buffer` entry | Query config | Capture vocabulary |
|---|---|---|---|
| Highlight | `chunks` → `get_highlights` | `highlights_config` | `@keyword`, `@string`, `@function`, … |
| Auto-indent | `suggested_indents` | `indents_config` | `@indent` `@start` `@end` `@outdent` |
| Bracket match | `enclosing_bracket_ranges` | `brackets_config` | `@open` `@close` |
| Outline / breadcrumbs | `outline`, `outline_items_containing` | `outline_config` | `@item` `@name` `@context` `@open` `@close` `@annotation` |
| Injections | `syntax_layers`, internal | `injection_config` | `@injection.content` `@injection.language` |
| Runnables (test gutter) | `runnable_ranges` | `runnable_config` | `@run` `@run_item` + named |
| Text objects | `text_object_ranges` | `text_object_config` | `function.inside/around`, `class.*`, `comment.*` |
| Redactions | `redacted_ranges` | `redactions_config` | `@redact` |
| Selection expand | `syntax_ancestor` (+ siblings) | *none — raw AST walk* | — |
| Folding | *editor layer* | *none — indent + LSP + creases* | — |

### 6.1 Semantic highlight → spans (the primary feature)

`chunks(range, LanguageAwareStyling)` returns a `BufferChunks` iterator. Under the hood
`get_highlights(range)` runs `syntax.captures(range, |g| &g.highlights_config.query)` and
collects one `HighlightMap` per participating grammar. `BufferChunks` merges three sorted
streams — rope text, highlight captures, and diagnostic endpoints — keeping a
`stack: Vec<(end_byte, HighlightId)>` of open captures; the **innermost** open capture's
`HighlightId` becomes each `Chunk`'s `syntax_highlight_id`. So the output is a sequence of

```
Chunk { text: &str, syntax_highlight_id: Option<HighlightId>, diagnostic_severity, … }
```

The editor turns each `HighlightId` into a concrete `HighlightStyle` via the theme's syntax
table and paints it. **This `(range, style)` list is exactly gpui's
`StyledText::with_highlights` input** (§9) — the whole highlight pipeline lands on one gpui
call.

### 6.2 Auto-indent

`suggested_indents(rows, single_indent_size)` → `suggest_autoindents(row_range)` runs
`matches_with_options(indents_config.query)` and reads: `@indent` (the node whose span defines
an indent region), `@start`/`@end` (open/close boundaries), `@outdent` (a position that
cancels the innermost indent, e.g. a closing `}`), plus `start.<suffix>` variants and an
`error_query` overlay and the declarative `increase/decrease_indent_pattern` regexes. It emits
per-row `IndentSuggestion { basis_row, delta: Ordering, within_error }`; `suggested_indents`
takes the basis row's indent and applies the delta to produce an `IndentSize`. On edit,
`compute_autoindents`/`apply_autoindents` run this on a background task so typing a newline
lands at the right column.

### 6.3 Bracket match + auto-close

`fetch_bracket_ranges` chunks the range and runs `matches_with_options(brackets_config.query)`
reading `@open`/`@close` per pattern, then *repairs* the grammar output by stacking opens and
matching closes per `pattern_index` (inferring a missing open when needed) into
`BracketMatch { open_range, close_range, newline_only, syntax_layer_depth, color_index }`.
`enclosing_bracket_ranges` keeps pairs that contain the cursor at the deepest layer;
`innermost_enclosing_bracket_ranges` picks the smallest. The editor consumes this in
`highlight_matching_bracket.rs` (match-highlight) and `bracket_colorization.rs` (rainbow
brackets). **Auto-close** on typing is driven by the *declarative*
`LanguageConfig.brackets` pairs (not the query) — `BracketPair { start, end, close, surround,
newline }` — so `(` inserts `)` even before a tree exists.

### 6.4 Code folding — NOT a `.scm` query (correction)

Zed has **no `folds.scm`**. Foldability is computed in the editor's `display_map.rs`/`fold.rs`
from three sources, in priority order: **LSP folding ranges** when available
(`use_lsp_folding_ranges`), otherwise **indentation** (`starts_indent(row)` — a row whose
following block is more-indented is foldable), plus explicit **`Crease`s** (regions features
register directly — diagnostics, runnable output, etc.). The one tree-sitter contribution is
`Buffer::function_body_fold_ranges`, which reuses the **text-objects** query
(`TextObject::InsideFunction`) to offer "fold this function body." Marley should model folding
as indent-derived + optional LSP + creases, and treat any doc that says "folds query" as
inaccurate for the Zed lineage.

### 6.5 Outline / breadcrumbs

`outline(theme)` = `Outline::new(outline_items_containing(0..len, …))`. `next_outline_item`
runs `matches(outline_config.query)` and reads `@item` (whole symbol), `@name`
(selection/display text), `@context`/`@context.extra` (modifiers/prefixes), `@open`/`@close`
(body range for fold-to-symbol), and `@annotation` (decorator rows attached to the next item).
Display text is built by iterating `chunks` over the captured sub-ranges and attaching
`HighlightStyle`s, so the outline is itself syntax-highlighted. Items are sorted and assigned
`depth` from a containment stack. `outline.rs` holds `Outline<T>` / `OutlineItem<T>` and an
async fuzzy `search` (via `fuzzy_nucleo`) with synthetic `Ancestor` rows so a match can show
its full path. `symbols_containing(position)` powers **breadcrumbs**.

### 6.6 Injections (consuming side)

Already parsed into layers (§4.3); consumers just read them. `syntax_layers_for_range`,
`smallest_syntax_layer_containing`, and `syntax_layer_at` (which checks `included_sub_ranges`
to confirm a point is really inside injected content) let features operate per-language. This
is what makes highlighting a JS block inside HTML, or a SQL string inside Rust, "just work" —
each layer contributes its own captures to the merged stream.

### 6.7 Runnables — the terminal-fusion hook

`runnable.rs` `[Zed-derived]`. `runnable_ranges(buffer, range)` runs
`matches(runnable_config.query)`; the `@run` capture marks the node that *is* the runnable
(the test-fn name, the subtest literal), `@run_item` marks a candidate inside a larger match
(so one match emits *multiple* runnables — Go table-test rows), and any other named capture is
exposed to a `RunnableResolver`. The output is

```
RunnableRange { buffer_id, run_range, full_range, runnable: Runnable{ tags, … }, extra_captures }
```

The editor renders a **run/debug icon in the gutter** on each `run_range`; clicking it resolves
the captures + the language's `ContextProvider` (registered in `languages`) into a concrete
shell command and **spawns it as a task in the terminal**. This is the single most Marley-
relevant seam: it is the built-in bridge from a syntax node to a terminal command — exactly
the "terminal-fusion" story Marley is built around. A `#[test] fn foo()` becomes a
`cargo test foo` block in the cockpit.

### 6.8 Selection expand/shrink — raw AST ancestry (no query)

`syntax_ancestor(range)` does **not** use a query. It calls `layers_for_range` and, per layer,
walks a `tree_sitter::TreeCursor` up the parents (`goto_node_enclosing_range`) until it finds
the smallest node that strictly contains the range; across layers it keeps the smallest. The
editor's `SelectLargerSyntaxNode` action loops this to grow a selection
identifier → call → statement → block → fn. `syntax_prev_sibling` / `syntax_next_sibling` add
structural motion. This is pure tree-sitter node ancestry — no `.scm` involved — and is the
cheapest high-value feature to add once a tree exists.

### 6.9 Text objects & redactions

`text_object_ranges` runs `matches(text_object_config.query)` and maps captures
(`function.inside`, `class.around`, `comment.*`) to a `TextObject` enum, merging co-captured
ranges — the substrate for vim `cif`/`daf`-style edits and function-body folds.
`redacted_ranges` runs `matches(redactions_config.query)` and yields each `@redact` node's
byte range (used to blur secrets in screen-share). Both are optional for Marley v1.

---

## 7. UI consumers (`outline`, `language_selector`)

- **`outline`** `[Zed-derived]` — the **go-to-symbol** modal. `OutlineView` is a `Picker`
  whose delegate calls `editor.buffer_outline_items(buffer_id)` → `language::Outline<Anchor>`,
  runs the async fuzzy `search`, and renders each row with `gpui::StyledText` + the item's
  `highlight_ranges`. Selecting a row scrolls/selects the symbol. The same `Outline` also
  backs breadcrumbs.
- **`language_selector`** `[Zed-derived]` — the **manual language picker** modal
  (`LanguageSelector`, a `Picker` over `registry.language_names()`) that sets a buffer's
  language when detection is wrong. `active_buffer_language.rs` is the status-bar indicator
  showing/toggling the current language.

Both are thin gpui `Picker` modals over the engine — good templates for how a Marley command-
palette entry ("Go to Symbol", "Set Language") would consume the tree.

---

## 8. Marley today (the hand lexer) + the gap

**Baseline (post-M15):** Marley's only syntax coloring is
`crates/marley_app/src/code_syntax.rs` `[Marley-original]` — a 388-line, gpui-free, per-**line**
hand lexer:

- `language_of(path) -> Language` picks by extension from a 6-variant enum
  (`Rust | Toml | Json | Shell | Markdown | Plain`).
- `highlight_line(line, lang) -> Vec<Span>` tokenizes **one line** into
  `Span { range, kind: TokenKind }` where `TokenKind ∈ {Keyword, Str, Comment, Number, Plain}`,
  by scanning characters with hand-written keyword/number/string/comment heuristics. The
  `app.rs` shim paints each span in a theme color.
- **Single-line only.** Block comments and strings that span lines are approximated per line;
  Markdown and Plain pass through as one span. There is **no** `tree-sitter` dependency
  anywhere in the workspace.

**The gap** is everything in §6 beyond crude coloring, plus correctness of the coloring itself:

- No parse tree ⇒ **no** auto-indent, bracket matching, folding, outline, breadcrumbs,
  injections, runnable gutter, or syntax-aware selection. These simply cannot exist on a
  per-line regex lexer.
- The coloring is **heuristic and per-line**, so it mis-colors multi-line strings/comments,
  can't tell a type from a variable, and needs a new hand-written lexer for every language
  (6 today; each addition is bespoke code).
- Marley already has the right *substrate*: an `editor` crate with a `Buffer` (SumTree
  lineage), `BufferVersion`, `BufferDelta`, `CharOffset` — i.e. the versioned, snapshot-able
  buffer a `SyntaxMap` needs to parse against. The missing piece is the parser + the glue.

---

## 9. The reimplementation — adopt tree-sitter as *one* query engine

The move is to **delete the hand lexer's role as the coloring source** and stand up a Marley
syntax engine that mirrors §3–§6 in Marley's own code, rendering through gpui. The engine is
`[Marley-original]` code implementing a `[Zed-derived]` design over `[permissive]` tree-sitter.

### 9.1 Target shape (Marley-original, mirroring Zed's roles)

- **`marley_grammar`** (mirrors `language_core`): a `Grammar { ts_language, highlights_config,
  indents_config, brackets_config, outline_config, injection_config, runnable_config, … }` where
  each config = a compiled `tree_sitter::Query` + resolved capture indices, built by a
  `populate_capture_indices`-style resolver. Plus `HighlightMap` (capture-index → Marley theme
  style id) and `LanguageQueries` (raw `.scm`).
- **`marley_syntax`** (mirrors `SyntaxMap`): `SyntaxSnapshot { layers: SumTree<Layer>, … }` with
  `interpolate(edit)` (apply `InputEdit`s, cheap) and `reparse(snapshot, registry, root)` (the
  depth-ordered `ParseStep` queue with incremental `parse` reusing the old tree + injection
  discovery). Reuse Marley's existing rope/sum-tree. Add the parser pool (`with_parser`) and the
  background drop thread.
- **Registry:** a lazy `LanguageRegistry` with native-grammar registration and `.scm`/`config`
  loading (embed with `rust_embed`, exactly like `grammars`). Skip wasm/extensions in v1.
- **`Buffer` feature methods:** `chunks`/`get_highlights`, `suggested_indents`, `bracket_ranges`,
  `outline`, `syntax_ancestor`, `runnable_ranges` — each a query-selector + capture-index
  interpreter, per §6.

### 9.2 Render: highlights → gpui `StyledText::with_highlights`

The render seam is trivial because gpui already speaks the right language. Marley's highlight
output — a list of `(Range<usize>, HighlightStyle)` where the style comes from mapping each
node's `HighlightId` through the active theme — feeds directly into
`gpui::StyledText::new(line_text).with_highlights(spans)` (signature confirmed in
`crates/gpui/src/elements/text.rs`: `with_highlights(highlights: impl IntoIterator<Item =
(Range<usize>, HighlightStyle)>)`). The existing `app.rs` code-viewer shim already loops lines
and paints spans; the change is to *source* those spans from `buffer.chunks(range)` instead of
`code_syntax::highlight_line`. The `TokenKind`→color mapping is replaced by
`HighlightId`→`theme.syntax_style` — a superset that distinguishes types/functions/properties
the hand lexer flattens to `Plain`.

### 9.3 Incremental + off-main-thread (mandatory, not optional)

Wire it the way §4.4 does: on a buffer edit, `interpolate` synchronously (keeps the screen
coherent), then `background_spawn` a `reparse_with_timeout` against a cloned `BufferSnapshot`,
publish a `ParseStatus` watch, and swap the new snapshot in via `did_parse` on the main thread.
This is what keeps Marley's cockpit at 120 fps while a 5 k-line file re-parses. The budget/
timeout path is what prevents a pathological file from dropping frames.

### 9.4 Sequencing (Marley-realistic, additive per M1.B pipeline norms)

1. **Seq 1 — dependency + one grammar + highlight parity.** Add `tree-sitter` +
   `tree-sitter-rust`; embed rust's `config.toml` + `highlights.scm`; stand up `Grammar` +
   `HighlightMap` + a *whole-file synchronous* parse; render via `StyledText::with_highlights`.
   Ship Rust highlight that already beats the hand lexer (types/functions/multi-line strings).
   Gate: pure `Grammar`/`HighlightMap` seams cov/MSI 100; shim renders the parsed spans.
2. **Seq 2 — the `SyntaxMap` + incremental + off-thread.** Add `interpolate`/`reparse`, the
   layer SumTree, the parser pool, background scheduling, `ParseStatus`. Now edits are cheap.
3. **Seq 3 — selection expand (`syntax_ancestor`).** Cheapest high-value non-highlight feature;
   pure AST walk, no new query. Wire `cmd-up`/`cmd-down` grow/shrink.
4. **Seq 4 — auto-indent + bracket match/auto-close.** `indents.scm` + `brackets.scm` + the
   declarative bracket pairs. Real editing ergonomics.
5. **Seq 5 — outline + breadcrumbs.** `outline.scm` → `Outline`; a "Go to Symbol" palette
   entry (mirror the `outline` crate's Picker) + a breadcrumb bar.
6. **Seq 6 — the runnable gutter (terminal fusion).** `runnables.scm` → `runnable_ranges` → a
   gutter run icon that spawns the resolved command as a cockpit terminal block. This is the
   flagship Marley-differentiating feature and should be its own milestone.
7. **Seq 7+ (optional).** Injections (embedded languages), folding (indent-derived + creases),
   text objects, more grammars, and only much later a wasm/extension registration path.

Folding (§6.4) is **not** a query step — schedule it as an editor-layer feature
(indent-derived + creases), not a `.scm` deliverable.

---

## 10. Provenance summary

| Element | Provenance | Marley action |
|---|---|---|
| `tree-sitter` runtime, `Query`, `InputEdit`, `Node`, `TreeCursor` | `[permissive/public: MIT]` | Adopt the crate directly |
| Grammars (`tree-sitter-rust`/`-python`/`-typescript`/…) | `[permissive/public: MIT/Apache]` | Add crates directly |
| `.scm` query files (highlights/indents/brackets/outline/injections/runnables/…) | `[permissive/public: grammar authors', mostly MIT/Apache]` | Vendor upstream or author own; not Zed-dependent |
| `Grammar` + `*Config` + capture-index resolution (`language_core`) | `[Zed-derived: GPL]` | Reimplement (`marley_grammar`) |
| `HighlightMap` / `HighlightId` design | `[Zed-derived: GPL]` | Reimplement |
| `SyntaxMap`/`SyntaxSnapshot` incremental multi-layer parse, injections, parser pool | `[Zed-derived: GPL]` | Reimplement (`marley_syntax`) — the core effort |
| `LanguageRegistry` lazy async loader + `grammars` embed + `languages` wiring | `[Zed-derived: GPL]` (assets permissive) | Reimplement loader; reuse `rust_embed` pattern |
| `Buffer` feature methods (`chunks`, `suggested_indents`, `outline`, `syntax_ancestor`, `runnable_ranges`, …) | `[Zed-derived: GPL]` | Reimplement as `Buffer` methods |
| `outline` / `language_selector` gpui modals | `[Zed-derived: GPL]` | Reimplement as Marley palette/modals |
| gpui `StyledText::with_highlights` render | `[permissive/public: Apache-2.0]` | Use directly (Marley already deps gpui) |
| `code_syntax.rs` hand lexer | `[Marley-original]` | Retire as the coloring source (keep as a no-grammar fallback if desired) |

**The boundary is clean:** the parser and its data are permissive; only the orchestration is
GPL and must be re-authored — and re-authoring it is bounded, well-understood work, not
research. Nothing in this subsystem touches the proprietary "brain" layer, so it sits entirely
inside Marley's intended-GPL editor tier.

---

## 11. Key files (in the Zed clone)

- `crates/language_core/src/grammar.rs` — `Grammar`, all `*Config`, `with_*_query`,
  `populate_capture_indices` (the query/grammar core).
- `crates/language_core/src/queries.rs` — `LanguageQueries`, `QUERY_FILENAME_PREFIXES`
  (the ten query kinds; note **no** folds).
- `crates/language_core/src/highlight_map.rs` — `HighlightMap`, `HighlightId`.
- `crates/language_core/src/language_config.rs` — declarative brackets/comments/overrides.
- `crates/language/src/syntax_map.rs` — `SyntaxMap`/`SyntaxSnapshot`, `interpolate`, `reparse`,
  injections, parser pools (the incremental engine).
- `crates/language/src/language.rs` — `Language`, `LanguageScope`, `with_parser`/
  `with_query_cursor`, `build_highlight_map`/`set_theme`.
- `crates/language/src/language_registry.rs` — `LanguageRegistry`, `AvailableLanguage`,
  `AvailableGrammar`, `LoadedLanguage`.
- `crates/language/src/buffer.rs` — the feature methods: `chunks`/`get_highlights`,
  `suggested_indents`, `bracket_ranges`/`enclosing_bracket_ranges`, `outline`/
  `outline_items_containing`, `syntax_ancestor`, `text_object_ranges`, `redacted_ranges`,
  `runnable_ranges`, and the async `reparse`/`did_finish_parsing` scheduling.
- `crates/language/src/outline.rs` — `Outline<T>`/`OutlineItem<T>` + fuzzy `search`.
- `crates/language/src/runnable.rs` — `RunnableRange`, `RunnableResolver` (terminal fusion).
- `crates/grammars/src/grammars.rs` — `rust_embed` asset loader, `native_grammars`,
  `load_config`, `load_queries`.
- `crates/languages/src/lib.rs` — built-in `init`/`register_language` wiring.
- `crates/language_extension/src/language_extension.rs` — the wasm-extension registration bridge.
- `crates/outline/src/outline.rs`, `crates/language_selector/src/language_selector.rs` — the
  gpui modal consumers.
- gpui render target: `crates/gpui/src/elements/text.rs` — `StyledText::with_highlights`.

Marley baseline: `crates/marley_app/src/code_syntax.rs` (the hand lexer to retire);
`crates/editor/src/buffer.rs` (the `Buffer` the syntax engine will parse against).

---

## 12. Open questions (for round 2)

1. **Marley's `SumTree` fit.** Does `crates/editor`'s buffer expose the anchor + `edits_since`/
   `anchored_edits_since` API `interpolate`/`reparse` need, or is a shim required to feed
   `InputEdit`s? Confirm before Seq 2.
2. **Theme syntax table.** Marley themes today map five `TokenKind`s. A tree-sitter highlight
   set is ~40 capture names — does the Marley theme model need a syntax-style sub-table
   (`theme.highlight_id(name)`) added before Seq 1 can show its full value?
3. **Runnable → terminal API.** What is the exact cockpit call to spawn a resolved command as a
   terminal block, and does a `ContextProvider`-style resolver (cwd, env, `$ZED_*`-equivalent
   vars) already exist on the Marley side? This determines Seq 6's shape.
4. **Grammar set + binary size.** Which languages ship native in v1 (rust/toml/json/shell to
   match today, + the cockpit's own needs)? Wasm/extension loading can be deferred indefinitely.
5. **Async executor.** Which Marley background executor hosts the reparse task, and how does
   `did_parse` marshal back to the gpui main thread (mirror `cx.background_spawn` +
   `ParseStatus` watch)?
