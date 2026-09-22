# warp_search_core

> Per-crate reference (Marley round 2) — crate dir `crates/warp_search_core`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL over permissive tantivy]` — universal-search framework + telemetry; **not** what `marley_search_core` is (that is a `nucleo` fuzzy matcher for palette/file-open). Gap. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (workspace `AGPL-3.0-only`) |
| **Internal deps** | 3 (`string-offset`, `warp_core`, `warpui_core`) |
| **Used by** | 1 (`warp`) |

## Purpose

`warp_search_core` is the reusable engine behind every **command-palette / fuzzy-finder style search surface** in the app (the inline command-search menu, repos picker, slash-commands, conversations, models, prompts, profiles, etc. — all live under `app/src/terminal/input/*`). It provides two layered capabilities:

1. **A result-mixing framework** (`mixer`, `data_source`, `item`, `result_renderer`) — an entity that fans a typed `Query` out to many registered *data sources* (sync or async), collects scored `QueryResult<T>`s, dedupes/orders them by priority tier and score, tracks per-source loading/error state, and renders rows. Generic over the action type `T: Action + Clone` that a row emits when accepted/executed.
2. **A full-text search backend** (`searcher`, `macros`) built on **Tantivy** (`tantivy = "0.26"`), with a schema-definition macro DSL so each consumer can declare a typed document/id schema and get a `SimpleFullTextSearcher` / `AsyncSearcher` with insert/delete/search-id/search-full-doc operations, custom tokenization, fuzzy + phrase-prefix matching, and score normalization.

It solves the "many heterogeneous searchable things, one ranked list" problem and isolates the heavy Tantivy dependency behind a typed API.

## Key types, modules & public API

**`mixer`** — the orchestrator.
- `SearchMixer<T: Action + Clone>` (a `warpui_core` entity / `ModelContext`) — `new()`, `reset()`, `add_sync_source()`, `add_async_source()` (with `AddAsyncSourceOptions`), `run_query(query, ctx)`, `results() -> &Vec<QueryResult<T>>`, `is_loading()`, `registered_filters()`, `first_data_source_error()`.
- `SearchMixerEvent` — emitted entity event. `DataSourceId::new()` identifies a registered source.
- Traits `SyncDataSource` and `AsyncDataSource` (the latter `Send + Sync`, returns `BoxFuture`) — implement these to plug a new source in. `DataSourceRunError` / `DataSourceRunErrorWrapper` carry per-source failures.

**`data_source`** — query model + filter taxonomy.
- `Query`, `QueryResult<T>` (`score()`, `priority_tier()`, `accept_result()`, `execute_result()`, `render_item()`, `render_icon()`, `detail_data()`, `accessibility_label()`…).
- `QueryFilter` enum + `FilterAtom` (e.g. `history:`/`h:`, `workflows:`/`w:`, `notebooks:`/`n:`, `actions:`, `drive:`, `sessions:`, `commands:`, `blocks:`/`b:`, `#` natural-language) — `filter_atom()`, `display_name()`, `placeholder_text()`, `icon_svg_path()`, `QueryFilter::all()`.
- `DataSourceSearchError`.

**`item`** — `SearchItem` trait (the renderable unit; `Send + Sync`), `SearchItemDetail`, `IconLocation`.

**`result_renderer`** — `QueryResultRenderer<T>` (`new`, `render`, `render_details`), `QueryResultRendererStyles`, `ItemHighlightState` (fills for selected/hovered rows via `warp_core::ui::appearance`), `OnQueryResultClickedFn<T>`.

**`searcher`** (gated `#![cfg(not(target_family = "wasm"))]` — Tantivy is unavailable on wasm).
- `SimpleFullTextSearcher<C: SearchSchemaConfig>` — `new()`, `build_index()`, `insert_document()`, `delete_document()`, `search_id()`, `search_full_doc()`, `get_all_documents()`, `clear_search_index()`.
- `AsyncSearcher<C>` — `*_async` variants, emits `SearcherEvent`.
- `FullTextSearchSchema<C>` — `add_search_field(name, weight)`, `add_id_field(name, type)`, `create_searcher()` / `create_async_searcher()`.
- Value model: `FullTextSearchFieldValue` / `FullTextSearchFieldTypes` (Str/U64/I64/F64/Bool), `FullTextSearchMatch`, traits `SearchSchemaConfig`, `SearchDocumentEntry`, `SearchIdentifyingEntry`, `ToFieldType`, `FromOwnedValue`. `CustomTokenizer` / `CustomTokenStream` implement Tantivy tokenization. Tuning consts: `DEFAULT_MEMORY_BUDGET` (50 MB), `MIN_MEMORY_BUDGET` (15 MB), `SCORE_BOOST_FACTOR`, `FUZZY_SCORE_PENALIZED_FACTOR`.

**`macros`** — `#[macro_export]` DSL: `type_to_field_type!`, `data_from_owned_value!`, `get_factor_or_default!`, and the headline schema-definition macro that generates the `SearchSchemaConfig` + doc/id structs from a declarative field list. The crate re-exports `paste` and `tantivy` from `lib.rs` for these macros to use.

## Depends on (internal)

- [string-offset](./string-offset.md) — `ByteOffset` for highlight/snippet ranges in matched text.
- [warp_core](./warp_core.md) — `features::FeatureFlag`, and `ui::{appearance::Appearance, icons::Icon, theme::Fill}` for rendering rows/icons consistently with app theming.
- [warpui_core](./warpui_core.md) — the UI/entity runtime: `Action`, `AppContext`, `Element`, `Entity`, `ModelHandle`/`ModelContext`, and `async::{Background, Timer, BoxFuture, block_on}` used to drive async data sources and searchers.

## Used by (internal dependents)

- [warp](./warp.md) — the top-level app crate (dir `app/`). Concretely consumed across `app/src/terminal/input/*` (`inline_menu`, `repos`, `slash_commands`, `conversations`, `models`, `prompts`, `profiles`, `cloud_mode_v2_history_menu`, `user_query`, …), each registering data sources into a `SearchMixer`.

## Related crates

- [markdown_parser](./markdown_parser.md), [warp_editor](./warp_editor.md) — produce/consume text that search surfaces index and highlight.
- [warp_core](./warp_core.md) — owns the underlying domains (history, workflows, sessions, blocks) that data sources query.
- `tantivy` (out-of-repo crate) — the embedded search-index backend.

## Marley relevance

**Classify: KEEP (likely EXTEND).** This is the natural home for Marley goal (1) *expand the UI surface with a custom panel* if that panel is search-driven: a Marley command/result panel would add its own `AsyncDataSource` (or a new `QueryFilter` variant + `FilterAtom`, e.g. `marley:`) and reuse `SearchMixer` + `QueryResultRenderer` rather than reimplementing fuzzy search. It is also relevant to goal (2) *session spawn/write/read* — a "sessions:" data source already exists, so a Marley session-launcher panel can be a thin new source over it.

The crate is **internal-only and de-Warp-rebrand light**: there is no user-facing "Warp" string here; rename touches are limited to the package name `warp_search_core` → `marley_search_core`. It has only one dependent (`warp`), so a rename is low-risk and can be done in the first rebrand pass. No auth/login coupling, so goal (3) does not touch it.

## Notes / gotchas

- **Tantivy is excluded on wasm** (`#![cfg(not(target_family = "wasm"))]` on `searcher.rs`, and the `tantivy` dep is gated `cfg(not(target_family = "wasm"))` in `Cargo.toml`). The mixer framework still compiles on wasm; only the full-text backend is dropped. Keep this gate if Marley ever targets wasm.
- Heavy **macro-generated code**: consumers' schemas are produced by the schema macro in `macros.rs`; `paste` and `tantivy` are re-exported from `lib.rs` specifically so those macros resolve. Don't remove those re-exports.
- Default index memory budget is 50 MB per searcher (`DEFAULT_MEMORY_BUDGET`); multiple live searchers add up. `MAX_THREADS_PER_INDEX_WRITER = 2`.
- Scoring has Warp-tuned magic constants (`SCORE_CONVERSION_FACTOR`, etc.) with a `TODO` about removing blended scoring — treat ranking as tuned, not principled, if you retune for Marley.
- `edition = "2024"`.
