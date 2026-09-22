# ipynb_parser

> Per-crate reference (Marley round 2) — crate dir `crates/ipynb_parser`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL]` — render-only ipynb→FormattedText; editor-surface feature, not ported. Gap. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (workspace `AGPL-3.0-only`) |
| **Internal deps** | 1 (`markdown_parser`) |
| **Used by** | 1 (`warp_editor`) |

## Purpose

`ipynb_parser` converts the JSON contents of a `.ipynb` (Jupyter) notebook into the editor's `FormattedText` rich-text model so notebooks can be **viewed** inside the editor. It is explicitly **render-only**: it produces a read-only view of existing markdown cells, code cells, and *saved* outputs (stream text, `execute_result`/`display_data` data, error tracebacks). It does **not** execute cells, run a kernel, or round-trip edits back to the file. Only **nbformat v4** is supported; anything else returns an error so the caller can fall back to showing the raw file verbatim (never a blank view). A single `lib.rs` (~400 lines) plus `lib_tests.rs`.

## Key types, modules & public API

Two public functions (the whole surface; notebook structs are private):

- **`pub fn ipynb_to_formatted_text(json: &str, gfm_tables: bool) -> Result<FormattedText, IpynbError>`** — the main entry point. Deserializes the notebook, requires `nbformat == 4`, and walks cells: markdown cells go through `markdown_parser::parse_markdown[_with_gfm_tables]`; code cells become `CodeBlockText` tagged with the notebook's language; outputs are appended. `gfm_tables` mirrors `Buffer::from_markdown`'s table-aware parsing (the caller passes the `MarkdownTables` feature-flag state).
- **`pub fn raw_fallback_formatted_text(content: &str) -> FormattedText`** — the verbatim fallback for when parsing fails; renders the raw bytes as plain `FormattedText`.
- **`pub enum IpynbError`** (`thiserror`) — `Parse(#[from] serde_json::Error)` and `UnsupportedFormat { nbformat: Option<i64> }`.

Private model (nbformat v4 subset, `serde::Deserialize`): `Notebook { nbformat, cells, metadata }` with `Notebook::language()` deriving a sanitized code-block language tag; `Metadata`/`LanguageInfo`/`Kernelspec`; `Cell { cell_type, source, outputs }`; `Output { output_type, text, data, traceback }`; and `Source` (an `#[serde(untagged)]` enum of `Lines(Vec<String>)` | `Text(String)`, since notebook source/text fields appear in both forms). Helpers: `value_to_text`, `sanitize_language` (ASCII-only, `MAX_LANGUAGE_TAG_CHARS = 32`), and an ANSI-escape stripper for output text (handles CSI/OSC sequences).

## Depends on (internal)

- [markdown_parser](./markdown_parser.md) — the target rich-text model and parser. Pulls in `FormattedText`, `FormattedTextLine`, `FormattedTextFragment`, `CodeBlockText`, `FormattedImage`, and `parse_markdown` / `parse_markdown_with_gfm_tables`; markdown cells are rendered through it so notebooks match normal markdown rendering.

## Used by (internal dependents)

- [warp_editor](./warp_editor.md) — the editor crate (dir `crates/editor`). Consumed in `crates/editor/src/content/buffer.rs`: `InitialBufferState::ipynb(text)` and `Buffer::from_markdown`/the `.ipynb` constructor surface `ipynb_parser::IpynbError`, building a read-only notebook `Buffer` from notebook JSON.

## Related crates

- [markdown_parser](./markdown_parser.md) — sibling/parent: produces the same `FormattedText` model for plain markdown; ipynb_parser is essentially a notebook-shaped front end to it.
- [warp_editor](./warp_editor.md) — owns the `Buffer` that displays the result.
- Out-of-repo: `serde` / `serde_json` for notebook deserialization, `thiserror` for the error enum.

## Marley relevance

**Classify: KEEP (rebrand-only).** This is a self-contained, offline, license-clean utility with no Warp branding, no auth, and no network — it touches none of the four Marley goals directly and needs no functional change. Keep as-is. It is mildly relevant to goal (1) *expand the UI surface*: if a Marley panel ever previews notebooks/files, this is the ready-made viewer to reuse via `ipynb_to_formatted_text` with a `raw_fallback_formatted_text` safety net.

De-Warp rebrand (goal 4) is cosmetic only: the package name `ipynb_parser` is already neutral; the `Cargo.toml` carries `authors = ["Warp Team <dev@warp.dev>"]` and the doc comments reference "Warp's rich-text/notebook renderer" — both are trivial string swaps. Single dependent (`warp_editor`), so any rename/move is low-risk.

## Notes / gotchas

- **nbformat v4 only** — `SUPPORTED_NBFORMAT = 4`; v3 or version-less notebooks return `IpynbError::UnsupportedFormat`. Callers *must* fall back to `raw_fallback_formatted_text` (the crate guarantees a non-blank view only if they do).
- **Security-minded sanitization**: notebook-declared languages are sanitized (`sanitize_language`, ASCII-alnum + a few symbols, ≤32 chars) so a malicious notebook can't inject a giant/garbage language tag into every code block; output text is run through an **ANSI-escape stripper** (CSI/OSC handling) so terminal control sequences in saved outputs don't leak into the rendered view. Preserve both if you refactor.
- `Source` uses `#[serde(untagged)]` because nbformat allows `source`/`text` as either a JSON string or an array of strings — don't "simplify" it to one variant.
- `edition = "2024"`, explicit `version = "0.1.0"`. No own LICENSE marker — inherits workspace AGPL v3 (note the `authors` field still says "Warp Team").
