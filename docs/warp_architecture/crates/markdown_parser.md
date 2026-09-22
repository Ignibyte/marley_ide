# markdown_parser

> Per-crate reference (Marley round 2). Crate dir: `crates/markdown_parser`. Marley is forked from Warp (warpdotdev/warp).

| Field | Value |
| --- | --- |
| Subsystem | [editor-and-text](../subsystems/02-editor-and-text.md) |
| License | AGPL v3 (workspace `license = "AGPL-3.0-only"`; no per-crate LICENSE marker) |
| Internal deps | 0 |
| Used by | 5 |
| Provenance | **`[Warp-derived/AGPL]`** `FormattedText` model (markdown parsing itself is public). **Marley — not built** (no rich-text buffer yet). See [subsystem — Provenance & licensing](../subsystems/02-editor-and-text.md). |

## Purpose

`markdown_parser` turns markdown (and a subset of HTML) source into Warp's own
**`FormattedText`** model — a flat, line-oriented representation of styled text
that the terminal/editor renderer and block UI consume. It is the bridge between
raw markdown strings (LLM agent output, docs, notebook prose, table blocks) and
the structured, style-annotated lines the UI knows how to draw and diff. It does
**not** render; it only parses and models.

It exists because Warp needs a renderer-agnostic, diffable, round-trippable text
model that supports GFM tables, task lists, inline styling, hyperlinks (including
ones that dispatch in-app *actions*, not just URLs), images, and embedded YAML
blocks — none of which a generic markdown-to-HTML library would give in the right
shape. Parsing uses `nom` for markdown and `html5ever` + `markup5ever_rcdom` for
the HTML path.

## Key types, modules & public API

Crate root (`src/lib.rs`) defines the data model:

- **`FormattedText`** — `{ lines: VecDeque<FormattedTextLine> }`. Constructors
  `new`, `new_trimmed`, plus `raw_text()`, `append_line()`.
- **`FormattedTextLine`** (enum) — the line variants: `Heading`,
  `Line`, `OrderedList`, `UnorderedList`, `CodeBlock`, `TaskList`, `LineBreak`,
  `HorizontalRule`, `Embedded(Mapping)`, `Image`, `Table`. Methods: `raw_text()`,
  `set_weight()`, `hyperlinks(skip_raw_links)`, `is_empty_line()`.
- Line payload structs: `FormattedTextHeader`, `FormattedTaskList`,
  `FormattedIndentTextInline`, `OrderedFormattedIndentTextInline`, `CodeBlockText`
  (`{ lang, code }`), `FormattedImage` (`{ alt_text, source, title }`),
  **`FormattedTable`** (`{ headers, alignments, rows }`) with `from_internal_format`,
  `from_internal_format_with_alignments`, `to_internal_format`, `to_plain_text`,
  `normalize_shape`; `TableAlignment` (`Left`/`Center`/`Right`).
- Inline model: `FormattedTextInline = Vec<FormattedTextFragment>`;
  **`FormattedTextFragment`** (`{ text, styles }`) with constructors
  `plain_text`, `bold`, `italic`, `bold_italic`, `inline_code`, `strikethrough`,
  `underline`, `hyperlink`, `hyperlink_action`, `weighted`.
- **`FormattedTextStyles`** — `{ weight, italic, underline, strikethrough,
  inline_code, hyperlink }`.
- **`Hyperlink`** (enum) — `Url(String)` or `Action(Arc<dyn Action>)`. The local
  **`Action`** trait (with a blanket impl over `Any + Debug + Send + Sync`)
  deliberately shadows `warpui::Action` to avoid a dependency cycle (the UI crate
  depends on this crate, not vice versa).
- **`FormattedTextDelta`** + `compute_formatted_text_delta(old, new)` — a
  line-based diff (`common_prefix_lines`, `old_suffix_formatted_text_lines`,
  `new_suffix`) used to incrementally update rendered blocks (e.g. streaming
  agent output). Note the special case: `CodeBlock` lines compare only `code`,
  not `lang`, because the internal lang tag may differ from the parsed tag.
- `LineCount` trait (`num_lines()`).

Parsing entry points:

- `markdown_parser::parse_markdown(&str) -> Result<FormattedText>` and
  `parse_markdown_with_gfm_tables(&str)` (re-exported at crate root).
- `parse_markdown_to_raw_text(&str) -> Result<String>`.
- `parse_inline_markdown(&str) -> Vec<FormattedTextFragment>` — inline styling only.
- `parse_image_run_line` / `parse_image_prefix` — image-line parsing.
- `html_parser::parse_html(&str) -> Result<FormattedText>` (re-exported as
  `parse_html`).
- `weight::CustomWeight` (8 weights `Thin`..`Black`) with `is_at_least_bold()`
  and `merge_weights()`.

## Depends on (internal)

None. This crate has zero internal dependencies — it sits at the leaf of the
editor-text dependency tree and pulls only third-party crates (`nom`,
`html5ever`, `markup5ever_rcdom`, `serde_yaml`, `itertools`, `enum-iterator`,
`anyhow`, `thiserror`).

## Used by (internal dependents)

- [ipynb_parser](./ipynb_parser.md) — renders notebook markdown cells.
- [warp](./warp.md) — the main app crate.
- [warp_editor](./warp_editor.md) — editor buffers that display formatted text.
- [warpui](./warpui.md) — UI rendering of formatted lines (MIT).
- [warpui_core](./warpui_core.md) — core UI primitives (MIT).

Total: 5 dependents.

## Related crates

- [warp_editor](./warp_editor.md) and [vim](./vim.md) — the rest of the
  editor-text subsystem that consumes this model.
- [ipynb_parser](./ipynb_parser.md) — a parallel parser for a different source
  format that also emits into the UI text model.

## Marley relevance

**Classify: KEEP (likely RENAME-deferred).**

This is pure, self-contained parsing/modeling logic with **no auth, no network,
no Warp branding** in its API surface. It is exactly the kind of crate Marley
keeps verbatim. For Marley goal (1) *expand the UI surface with a custom panel*,
this is the model you would feed: a Marley panel that renders agent/markdown
output reuses `parse_markdown` + `FormattedText` directly. For goal (2)
*session spawn/write/read*, streamed session output that is markdown can be
incrementally rendered via `compute_formatted_text_delta`.

Two rebrand touchpoints for goal (4) *de-Warp rebrand*:
- The internal table block tag string `warp-markdown-table` (referenced in the
  `FormattedTable::to_internal_format` doc comments) — cosmetic, rename to a
  Marley tag only if/when the buffer format is also changed in lockstep
  (round-trip safety).
- Package rename `markdown_parser` is trivially safe (the name is generic), but
  with 5 dependents it is still a defer-until-mass-rename item, not a day-one
  change. No login/de-auth work touches this crate (goal 3 = N/A).

## Notes / gotchas

- **Dependency-cycle workaround:** the crate-local `Action` trait shadows
  `warpui_core::Action` on purpose. Anything constructing `hyperlink_action`
  relies on the blanket impl; don't "fix" this by depending on `warpui_core`
  here — it would create a cycle.
- **`Hyperlink` PartialEq is a stub:** it compares only `Url` variants; two
  `Action` hyperlinks are never equal. This is only sound because markdown
  parsing provably never produces action hyperlinks (used for style
  consolidation). Preserve that invariant if you extend the parser.
- **Code-block diff quirk:** `compute_formatted_text_delta` ignores the `lang`
  field when comparing code blocks (internal lang tags like
  `"python path=… start=1"` won't match the bare parsed `"python"`).
- Out-of-repo deps include `markup5ever_rcdom = "0.35.0"` pinned directly (not
  via workspace). HTML parsing builds a full RC-DOM tree, so it is heavier than
  the markdown path.
- `edition = "2024"` for this crate (most siblings are 2021).
