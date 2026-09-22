# languages

> Per-crate reference (Marley round 2). Crate dir: `crates/languages`. Marley is forked from Warp (warpdotdev/warp).

| | |
|---|---|
| Subsystem | [editor-and-text](../subsystems/02-editor-and-text.md) |
| License | AGPL v3 (`AGPL-3.0-only`, inherited from workspace; no per-crate `LICENSE`) |
| Internal deps | 2 |
| Used by | 3 |
| Provenance | Registry code **`[Warp-derived/AGPL]`**; the embedded grammars are third-party **`[permissive]`** (tree-sitter). **Marley frontier — not built.** See [subsystem — Provenance & licensing](../subsystems/02-editor-and-text.md). |

## Purpose

`languages` is the **programming-language registry**: it maps language names and filenames to a fully loaded `Language` bundle — tree-sitter grammar, highlight query, indent query, symbol query, indent unit, comment prefix, and bracket pairs. Grammar source and `.scm` query files are **embedded into the binary** (`rust_embed` over the `grammars/` folder), so the running app needs no on-disk grammar files. It is the data layer that `syntax_tree` consumes to actually highlight and indent buffers.

## Key types, modules & public API

All in `crates/languages/src/lib.rs`.

- **`pub const SUPPORTED_LANGUAGES: [&str; 34]`** — the authoritative list (rust, golang, yaml, python, javascript/jsx, typescript/tsx, java, cpp/c, shell, csharp, html, css, json, jq, hcl, lua, ruby, php, toml, swift, kotlin, scala, powershell, elixir, sql, starlark, objective-c, xml, vue, dockerfile, nix).
- **`struct Language`** — public fields: `grammar: ParserGrammar` (tree-sitter `Language`), `highlight_query: Query`, `indents_query: Option<Query>`, `indent_unit: IndentUnit` (from `warp_editor`), `comment_prefix: Option<String>`, `bracket_pairs: Vec<(char,char)>`, `symbols_query: Option<Query>`, `display_name: String`; method `display_name()`.
- **`struct LanguageRegistry`** — lazily caches `Arc<Language>` per name (`Mutex<HashMap<String, Arc<Language>>>`); `language_by_name(name) -> Option<Arc<Language>>` (returns `None` for unsupported names, otherwise loads + memoizes). A process-global `LANGUAGE_REGISTRY` is held via `lazy_static!`.
- **Free functions:** `language_by_name(name)`, `language_by_filename(&StandardizedPath)`, `language_by_local_filename(&Path)` — resolve a language from a name or a file path (extension/filename mapping).
- **`#[derive(RustEmbed)] struct Grammars`** (`#[folder = "grammars"]`) — embeds grammar `.so`/wasm + `.scm` queries; private loaders (`load_language`, query getters) read from it via `<Grammars as RustEmbed>::get(path)`.
- Internal config structs (`LanguageConfig`, `BracketPair`) deserialized from embedded YAML.

## Depends on (internal)

- [warp_editor](./warp_editor.md) — uses `content::text::IndentUnit` in the `Language` model (declared as a path dep `../editor`).
- [warp_util](./warp_util.md) — `standardized_path::StandardizedPath` for filename-based lookup.

## Used by (internal dependents)

- [syntax_tree](./syntax_tree.md) — its primary consumer; turns a `Language` into parsed/highlighted trees.
- [ai](./ai.md) — language detection / metadata for AI features.
- [warp](./warp.md) — the application.

(3 dependents total.)

## Related crates

- [syntax_tree](./syntax_tree.md) — the runtime that drives parsing/highlighting from these grammars.
- [warp_editor](./warp_editor.md) — supplies `IndentUnit` and the buffers being highlighted.
- `arborium` (external/vendored tree-sitter wrapper) — provides `ParserGrammar`, `Query`.

## Marley relevance

**KEEP.** Self-contained data/registry crate, no Warp branding in its API, no auth or network surface; none of the four Marley goals require changing it. For **goal 1 (custom panel)**, if a Marley panel shows code with syntax highlighting it consumes this registry as-is via `language_by_name` / `language_by_filename`. Optional later **EXTEND**: trim or grow `SUPPORTED_LANGUAGES` and the embedded `grammars/` set to shrink binary size or add languages — but that is a size/scope optimization, not required for the fork. Do not rename the package early; it is referenced by name in three dependents and carries no brand baggage.

## Notes / gotchas

- **Grammars are embedded at build time** from the `grammars/` directory via `rust_embed` — that folder must be present and populated or languages silently fail to load (`get()` returns `None`). This is a build-data dependency, not just code.
- `SUPPORTED_LANGUAGES` is a hard-coded length-34 array; adding a language means updating the const **and** dropping grammar + `.scm` files into `grammars/`.
- The registry uses `std::sync::Mutex` and `.expect("Mutex should not be poisoned")` — a panic in a loader while holding the lock would poison it process-wide.
- Depends directly on `warp_editor` only for the `IndentUnit` type — a lightweight coupling worth remembering if the editor's text module is refactored.
