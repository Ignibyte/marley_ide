---
spec_id: markdown-render
component: marley_markdown
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Markdown/GFM → formatted-text model for blocks (tables, task lists, links)
goal: Parse markdown (and a GFM subset) into a flat, diffable line-oriented formatted-text model that the block UI renders, without performing any rendering itself.
reuses: [pulldown-cmark, html5ever]
spec_source: "behavior-only — observable I/O: parse GFM markdown (headings, emphasis, lists, task lists, tables, code spans/blocks, links) into a formatted-text model of styled line/inline runs for block rendering. No fork module/type/file/static names."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
---

## Purpose
`marley_markdown` converts markdown source (LLM agent output, docs, notebook prose, table blocks) into a renderer-agnostic **`FormattedText`** model: a flat sequence of style-annotated lines that the block/editor UI draws and diffs. It parses and models only — it never renders pixels. The model's line and inline vocabularies are **derived directly from the GFM/CommonMark block and inline features** the requirements below exercise (paragraph, ATX heading, bullet/numbered/task list item, fenced code block, thematic break, blank line, pipe table, image, and a Marley-reserved embedded-data fence), plus the inline spans (emphasis, strong, code, strikethrough, link/auto-link, image). It exists so the UI has one diffable, round-trippable text model that a generic markdown-to-HTML library would not produce. Markdown is parsed with `pulldown-cmark`; the HTML subset is parsed with `html5ever`.

## Public surface (the contract)
All identifiers below are Marley-original and named for the **observable GFM feature** they carry, not for any upstream module/type taxonomy.

Data model (`src/lib.rs`):
- `FormattedText { lines: VecDeque<FormattedLine> }` — `new()`, `new_trimmed()`, `raw_text() -> String`, `push_line(line)`.
- `FormattedLine` (enum) — one variant per GFM block feature in the requirements:
  `Paragraph(StyledText)`, `Heading(HeadingLine)`, `BulletItem(ListItemLine)`,
  `NumberedItem(NumberedItemLine)`, `TaskItem(TaskItemLine)`, `CodeBlock(CodeBlockLine)`,
  `BlankLine`, `ThematicBreak`, `Embedded(serde_yaml::Mapping)`, `Image(ImageLine)`,
  `Table(MarkdownTable)`.
  Methods: `raw_text() -> String`, `set_weight(TextWeight)`,
  `hyperlinks(skip_raw_links: bool) -> Vec<Hyperlink>`, `is_blank() -> bool`.
- Block payloads:
  - `HeadingLine { level: HeadingLevel, content: StyledText }`;
    `HeadingLevel { H1, H2, H3, H4, H5, H6 }`.
  - `ListItemLine { indent: u16, content: StyledText }`,
    `NumberedItemLine { indent: u16, number: u64, content: StyledText }`.
  - `TaskItemLine { checked: bool, content: StyledText }`.
  - `CodeBlockLine { lang: Option<String>, code: String }`.
  - `ImageLine { alt_text: String, source: String, title: Option<String> }`.
  - `MarkdownTable { headers, alignments, rows }` with
    `from_markdown_source(&str) -> Result<MarkdownTable, MarkdownError>`,
    `to_markdown_source() -> String`, `to_plain_text() -> String`, `normalize_shape()`;
    `TableAlignment { None, Left, Center, Right }` (`None` = GFM default `---`, no colon).
- Inline model:
  `StyledText = Vec<TextRun>`; `TextRun { text: String, styles: RunStyles }` with constructors
  `plain_text`, `bold`, `italic`, `bold_italic`, `inline_code`, `strikethrough`, `underline`,
  `hyperlink`, `hyperlink_action`, `weighted`.
  `RunStyles { weight: TextWeight, italic: bool, underline: bool, strikethrough: bool, inline_code: bool, hyperlink: Option<Hyperlink> }`.
- `Hyperlink` (enum): `Url(String)` | `Action(Arc<dyn LinkAction>)`; crate-local `LinkAction`
  trait with a blanket impl over `Any + Debug + Send + Sync`.
- `FormattedTextDelta { common_prefix_lines, old_suffix_lines, new_suffix_lines }` +
  `compute_formatted_text_delta(old: &FormattedText, new: &FormattedText) -> FormattedTextDelta`.
- `LineCount` trait (`num_lines() -> usize`); `weight::TextWeight` (8 weights `Thin`..`Black`)
  with `is_at_least_bold() -> bool` and `heavier_of(a: TextWeight, b: TextWeight) -> TextWeight`.
- `MarkdownError` (enum): `InvalidEmbeddedYaml { line: usize }` | `Html(String)` —
  the closed set of conditions under which parsing returns `Err`.
- Embedded-fence discriminator: `EMBEDDED_FENCE_TAG: &str` — the reserved info-string
  (`"embedded"`) that designates a fenced block as a structured-data block rather than a code
  block. Public so consumers and tests share one source of truth.

Parsing entry points:
- `parse_markdown(&str) -> Result<FormattedText, MarkdownError>`,
  `parse_markdown_with_tables(&str) -> Result<FormattedText, MarkdownError>`.
- `parse_markdown_to_raw_text(&str) -> Result<String, MarkdownError>`.
- `parse_inline(&str) -> StyledText`.
- `parse_image_line(&str) -> Option<ImageLine>`, `parse_image_prefix(&str) -> Option<(ImageLine, &str)>`.
- `parse_html(&str) -> Result<FormattedText, MarkdownError>`.

## EARS Requirements
- **R1.** WHEN `parse_markdown` receives a string of N markdown paragraphs separated by blank lines, the system shall return a `FormattedText` whose lines preserve the source paragraph order.
- **R2.** WHEN `parse_markdown` receives an ATX heading with 1 to 6 leading `#` characters, the system shall emit a `FormattedLine::Heading` whose `level` equals the leading-`#` count (`H1`..`H6`); a line opening with 7 or more `#` characters is not a heading and the system shall emit a `FormattedLine::Paragraph` (matching `pulldown-cmark` / CommonMark, which treats 7+ `#` as paragraph text).
- **R3.** WHEN `parse_markdown` receives a GFM unordered task-list item (`- [ ]` or `- [x]`), the system shall emit a `FormattedLine::TaskItem` whose `checked` flag is `true` for `[x]`/`[X]` and `false` for `[ ]`.
- **R4.** WHEN `parse_markdown_with_tables` receives a GFM pipe table with a delimiter row, the system shall emit one `FormattedLine::Table` whose `headers` length, every `row` length, and `alignments` length are all equal to the table's column count, and every `alignments` entry shall hold a defined `TableAlignment` value.
- **R5.** WHEN a GFM table delimiter cell specifies alignment, the system shall set the matching `MarkdownTable::alignments` entry to `TableAlignment::Left` for `:---`, `TableAlignment::Center` for `:---:`, `TableAlignment::Right` for `---:`, and `TableAlignment::None` for a colon-less `---` (GFM default).
- **R6.** WHEN `parse_inline` receives text containing `**bold**`, `*italic*`, `` `code` ``, `~~strike~~`, and `[label](url)` spans, the system shall produce `TextRun`s whose `styles` carry exactly the marker's style (`weight >= bold`, `italic`, `inline_code`, `strikethrough`, or `hyperlink = Url`) and whose `text` excludes the markup delimiters.
- **R7.** WHEN `parse_markdown` receives a fenced code block whose info string is NOT the reserved `EMBEDDED_FENCE_TAG`, the system shall emit a `FormattedLine::CodeBlock` whose `CodeBlockLine.lang` equals the info string (or `None` when absent) and whose `code` equals the verbatim fence contents with inline markdown left unparsed.
- **R8.** WHEN `parse_image_line` receives `![alt](source "title")`, the system shall emit an `ImageLine` with `alt_text = "alt"`, `source = "source"`, and `title = Some("title")`.
- **R9.** WHEN `FormattedLine::hyperlinks(true)` is called, the system shall return every `Hyperlink` carried by the line's runs while omitting hyperlinks whose displayed text equals their URL (raw/auto links).
- **R10.** WHEN `MarkdownTable::to_markdown_source` is applied to a table and the result is parsed back with `MarkdownTable::from_markdown_source`, the system shall return a table equal to the original in `headers`, `alignments`, and `rows` (GFM-pipe-table round-trip identity).
- **R11.** WHEN `compute_formatted_text_delta(old, new)` is given two `FormattedText` values sharing a leading run of identical lines, the system shall set `common_prefix_lines` to the length of that shared run and set `new_suffix_lines` to exactly the trailing lines of `new` that differ.
- **R12.** WHEN `compute_formatted_text_delta` compares two `CodeBlock` lines, the system shall treat them as equal whenever their `code` fields are equal, regardless of any difference in their `lang` fields.
- **R13.** WHEN `parse_html` receives an HTML fragment containing `<b>`, `<i>`, and `<a href>` elements, the system shall emit runs whose styles map `<b>` to bold weight, `<i>` to italic, and `<a href="u">` to `Hyperlink::Url("u")`.
- **R14.** WHEN `parse_markdown` receives a fenced block whose info string equals `EMBEDDED_FENCE_TAG` and whose body is a valid YAML mapping, the system shall emit a `FormattedLine::Embedded` carrying the parsed `serde_yaml::Mapping`.
- **R15.** The system shall implement `Hyperlink`'s `PartialEq` such that two `Url` variants compare by string equality and any comparison involving an `Action` variant returns `false`.
- **R16.** WHEN `heavier_of(a, b)` is called, the system shall return the heavier of the two `TextWeight` values on the `Thin`..`Black` scale.
- **R17.** IF `parse_markdown` receives a fenced block tagged `EMBEDDED_FENCE_TAG` whose body is not a valid YAML mapping, THEN the system shall return `Err(MarkdownError::InvalidEmbeddedYaml { .. })` without panicking. (This is the single realistic failure source: CommonMark/GFM parsing is otherwise total, so all other inputs return `Ok`.)
- **R18.** WHEN `FormattedText::new_trimmed` is constructed from lines with leading and trailing `BlankLine` lines, the system shall produce a `FormattedText` with no leading or trailing blank lines.
- **R19.** WHEN `MarkdownTable::normalize_shape` is applied to a table whose rows are shorter or longer than its header count, the system shall pad short rows with empty cells and truncate long rows so every row length equals the header count.
- **R20.** WHEN `num_lines()` is called on a `FormattedText`, the system shall return a count equal to `lines.len()`.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | Paragraph order preserved across N paragraphs (R1) | planned |
| 2 | Heading `level` = leading-`#` count for 1..6 (`H1`..`H6`); 7+ `#` → `Paragraph` per pulldown-cmark (R2) | planned |
| 3 | Task-item checked flag parsed from `[x]`/`[ ]` (R3) | planned |
| 4 | Table headers/rows/alignments all = column count; every alignment defined (R4) | planned |
| 5 | Delimiter alignment → Left/Center/Right; colon-less `---` → None (R5) | planned |
| 6 | Inline styles bold/italic/code/strike/link; delimiters stripped (R6) | planned |
| 7 | Non-embedded fence → CodeBlock; lang = info string; body verbatim (R7) | planned |
| 8 | Image alt/source/title parsed (R8) | planned |
| 9 | `hyperlinks(true)` omits raw/auto links (R9) | planned |
| 10 | Table GFM-source round-trip identity (R10) | planned |
| 11 | Delta common prefix + new suffix correct (R11) | planned |
| 12 | Code-block delta ignores `lang` (R12) | planned |
| 13 | HTML `<b>`/`<i>`/`<a>` → styles/Url (R13) | planned |
| 14 | Embedded fence (`EMBEDDED_FENCE_TAG`) + valid YAML → `Mapping` (R14) | planned |
| 15 | `Hyperlink` PartialEq: Url by value, Action always ne (R15) | planned |
| 16 | `heavier_of` returns heavier weight (R16) | planned |
| 17 | Embedded fence with invalid YAML body → `Err(InvalidEmbeddedYaml)`, no panic (R17) | planned |
| 18 | `new_trimmed` drops edge blank lines (R18) | planned |
| 19 | `normalize_shape` pads/truncates rows to header count (R19) | planned |
| 20 | `num_lines() == lines.len()` (R20) | planned |

## Visual / Behavioral Acceptance
N/A — this crate parses and models only; it produces no on-screen surface. Rendering of the `FormattedText` model is owned by the block-UI spec and its visual/AX acceptance is asserted there.

## Test Plan
- **Unit:** one test per clause — `r1_paragraph_order`, `r2_heading_level_and_overflow`, `r3_task_item_checked`, `r4_table_shape_all_defined`, `r5_alignment_from_delimiter_incl_default_none`, `r6_inline_styles_strip_delimiters`, `r7_codeblock_lang_and_verbatim_non_embedded`, `r8_image_alt_source_title`, `r9_hyperlinks_skip_raw`, `r10_table_gfm_source_roundtrip`, `r11_delta_prefix_suffix`, `r12_delta_codeblock_ignores_lang`, `r13_html_b_i_a`, `r14_embedded_fence_valid_yaml_mapping`, `r15_hyperlink_partial_eq`, `r16_heavier_of`, `r17_embedded_invalid_yaml_returns_err`, `r18_new_trimmed_edges`, `r19_normalize_shape_pad_truncate`, `r20_num_lines`. 100% coverage on touched lines.
  - **R2 fixtures (library-behavior):** `#`→`H1`, `######`→`H6`, `#######` (7 hashes)→`Paragraph` — proving 7+ `#` is paragraph text, not a clamped heading.
  - **R7 / R14 non-overlapping fixtures (discriminator):** R7 uses a `` ```rust `` fence (and a bare `` ```yaml `` fence) → both `CodeBlock` with verbatim body; R14 uses a `` ```embedded `` fence (`EMBEDDED_FENCE_TAG`) with a YAML mapping body → `Embedded`. No fixture is ambiguous across the two clauses: the info string is the sole discriminator.
  - **R5 fixtures:** delimiter row `| :--- | :---: | ---: | --- |` → `[Left, Center, Right, None]`.
- **Integration:** `compute_formatted_text_delta` over two successive `parse_markdown` outputs of a growing (streaming) document — asserts the incremental-update seam used by the block UI. `parse_html` → same `FormattedText` shape as `parse_markdown` for an equivalent fragment.
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full per-clause suite stays green; round-trip (R10) and delta (R11/R12) invariants are property-tested with generated tables/line sequences so future parser changes cannot silently break them.

## Mutation Targets
`cargo-mutants` must kill mutants on: heading-level arithmetic and the 1..6-vs-7+ paragraph boundary (R2), the task-item `checked` boolean (R3), the table column-count equality and alignment mapping including the colon-less `---`→`None` arm (R4/R5), delimiter-stripping and style-flag assignment in inline parsing (R6), the embedded-vs-codeblock discriminator branch on `EMBEDDED_FENCE_TAG` (R7/R14), the `hyperlinks` skip-raw predicate (R9), the table GFM-source round-trip and `normalize_shape` pad/truncate bounds (R10/R19), the delta `common_prefix` boundary and the code-block `lang`-ignoring comparison (R11/R12), the `MarkdownError::InvalidEmbeddedYaml` error path (R17), `Hyperlink` PartialEq branches (R15), and `heavier_of` ordering (R16). MSI 100% on the testable surface. ACCEPTED-UNTESTABLE: none — the entire surface is pure and harnessable (no GPU/IO path).

## Dependencies
- REUSE (permissive): `pulldown-cmark` (markdown + GFM tables/task lists/strikethrough), `html5ever` (HTML subset). Model/serde support: `serde_yaml` for `Embedded` payloads.
- Marley components: none (leaf crate — zero internal dependencies, consumed by the block-UI and editor specs).

## Out of scope / deferred
- Pixel/glyph rendering of `FormattedText` — owned by the block-UI spec (M2).
- `Hyperlink::Action` *construction* from markdown — the parser provably never emits action hyperlinks; action wiring is an editor/UI concern (M2+). This spec only models the `Action` variant and its PartialEq contract (R15).
- Notebook (`.ipynb`) cell extraction — a separate parser spec feeds its markdown cells into `parse_markdown`.

## Clean-room provenance
Spec'd from `docs/specs/behavior/markdown-render.behavior.md` — a behavior-only description (observable I/O: the GFM/CommonMark features above and the model they map to), no AGPL/fork source read, no private upstream module/type/static names. The line and inline model names (`Paragraph`, `Heading`/`HeadingLevel`, `BulletItem`/`NumberedItem`, `TaskItem`, `CodeBlock`, `BlankLine`, `ThematicBreak`, `Embedded`, `Image`, `Table`; `StyledText`/`TextRun`/`RunStyles`) are derived from the GFM features the requirements exercise, and the table serialization round-trip (R10) is expressed in the public **GFM pipe-table source** form (`to_markdown_source`/`from_markdown_source`) — no opaque internal tag-string format. The fresh implementation reuses `pulldown-cmark` and `html5ever` (both MIT/Apache-2.0); all REUSE crates are MIT/Apache. Per seam-contracts §11, `clean_room` is downgraded to "behavior-derived from a fork-reference doc; IP-counsel sign-off pending" until the behavioral wall + sign-off land (open item in `clean-build-plan.md`).
