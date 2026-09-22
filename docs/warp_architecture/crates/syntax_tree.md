# syntax_tree

> Per-crate reference (Marley round 2). Crate dir: `crates/syntax_tree`. Marley is forked from Warp (warpdotdev/warp).

| | |
|---|---|
| Subsystem | [editor-and-text](../subsystems/02-editor-and-text.md) |
| License | AGPL v3 (`AGPL-3.0-only`, inherited from workspace; no per-crate `LICENSE`) |
| Internal deps | 6 |
| Used by | 2 |
| Provenance | **`[Warp-derived/AGPL]`** incremental-highlight runtime over **`[permissive]`** tree-sitter (MIT). **Marley frontier — not built** (no syntax highlighting yet); mapped in [Zed 04](../../zed_architecture/subsystems/04-language-syntax-treesitter.md). See [subsystem — Provenance & licensing](../subsystems/02-editor-and-text.md). |

## Purpose

`syntax_tree` is the **incremental syntax-highlighting and auto-indent runtime**. It owns the live tree-sitter `Tree` for an editor buffer, re-parses incrementally as the buffer is edited (translating buffer deltas into tree-sitter `InputEdit`s), runs the highlight and indent queries from a `languages::Language`, and produces per-range colors (`RangeMap<CharOffset, ColorU>`) plus indentation deltas. It glues the `languages` grammar data, the `warp_editor` buffer model, and the `warpui_core` rendering/entity system together so the UI can paint syntax-colored, correctly-indented code.

## Key types, modules & public API

`crates/syntax_tree/src/lib.rs` plus `queries/` (`highlight_query`, `indent_query`, mod).

- **`struct SyntaxTreeState`** — the central entity (a `warpui_core` model). Constructed with `new(...)`; methods:
  - `set_language(Arc<Language>)`, `has_supported_highlighting()`, `indent_unit() -> Option<IndentUnit>`, `bracket_pairs() -> Option<&[(char,char)]>`, `comment_prefix() -> Option<&str>`.
  - `highlights_in_ranges(...)` — compute/return cached highlight colors for buffer ranges.
  - `indentation_at_point(point, &AppContext) -> Option<IndentDelta>` — auto-indent decision for a position.
  - `invalidate_highlight_cache_for_version(BufferVersion)`, `set_color_map(ColorMap)`.
- **`enum DecorationStateEvent`** — `DecorationUpdated { version }`, emitted when highlights are recomputed.
- **`pub use queries::highlight_query::{ColorMap, TextSlice}`** — the theme color mapping and a text-slice helper.
- Internal: `queries::highlight_query::HighlightQuery`, `queries::indent_query::{indentation_delta, IndentDelta}`, `LanguageQueries`, single-entry `HighlightCache`/`HighlightCacheKey`.
- Parsing infra: a `thread_local!` `PARSER: RefCell<Parser>` (arborium/tree-sitter), `MAX_SYNTAX_TREES = 3`, and `MAX_PARSE_BYTES = 2 MB` (buffers larger than this are skipped to avoid tree-sitter's super-linear memory growth). Uses `AbortHandle` (futures) to cancel in-flight parses and `parking_lot::Mutex` for shared state.

## Depends on (internal)

- [languages](./languages.md) — supplies the `Language` (grammar + queries) being run.
- [warp_editor](./warp_editor.md) — the `Buffer`/`BufferSnapshot`, `PreciseDelta`, `BufferVersion`, `IndentUnit`, and `DecorationLayer` it parses and decorates.
- [string-offset](./string-offset.md) — `ByteOffset`/`CharOffset` keying the highlight `RangeMap`/`RangeSet`.
- [warpui_core](./warpui_core.md) — entity system (`AppContext`, `Entity`, `ModelContext`, `WeakModelHandle`), `color::ColorU`, `text::point::Point`. *(MIT.)*
- [warpui](./warpui.md) — UI layer (test-utils used in dev-deps; pulled for the rendering surface). *(MIT.)*
- [warp_util](./warp_util.md) — shared utilities.

## Used by (internal dependents)

- [ai](./ai.md) — uses parsed syntax info for AI features.
- [warp](./warp.md) — the application wires highlighting into the editor UI.

(2 dependents total.)

## Related crates

- [languages](./languages.md) — its data source; read them together.
- [warp_editor](./warp_editor.md) — the buffer/decoration model it decorates (`decoration::DecorationLayer`).
- `arborium` (external/vendored tree-sitter) + `streaming-iterator` — the parsing substrate.

## Marley relevance

**KEEP.** Pure rendering/intelligence layer with no auth, network, or Warp-cloud surface, so de-auth (goal 3) and de-Warp rebrand (goal 3/4) don't touch it beyond an eventual package-name sweep. **Goal 1 (custom panel):** any Marley panel that shows highlighted code/source reuses `SyntaxTreeState` directly (`set_language` + `highlights_in_ranges`); do not reinvent highlighting. No reason to STUB — it degrades gracefully (`has_supported_highlighting()` is `false` / `None` for unknown languages and big buffers are skipped via `MAX_PARSE_BYTES`), so it is safe in an offline/minimal boot. Rename to `marley_syntax` only as part of the batched de-Warp pass (only 2 dependents, low cost, but no urgency).

## Notes / gotchas

- **2 MB parse cap (`MAX_PARSE_BYTES`)** and **`MAX_SYNTAX_TREES = 3`** are deliberate guards against tree-sitter memory blow-up — see the linked tree-sitter issue in the source comment. Large files render *unhighlighted* by design.
- Uses a **`thread_local!` tree-sitter `Parser`** (`RefCell`) — parsing is per-thread; be careful moving work across threads.
- Single-entry `HighlightCache` (one key) — only the most recent (version, ranges, language) result is memoized; rapid alternation between regions re-parses.
- In-flight parses are cancelled via `futures::stream::AbortHandle`; relies on the `warpui_core` async/entity runtime being present.
- Tightly coupled to the `arborium` tree-sitter wrapper API (`InputEdit`, `Parser`, `Tree`, `Language`) — a vendored, out-of-batch dependency.
