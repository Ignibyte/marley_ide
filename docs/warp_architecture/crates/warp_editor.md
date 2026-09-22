# warp_editor

> Per-crate reference (Marley round 2). Crate dir: `crates/editor`. Marley is forked from Warp (warpdotdev/warp).

| | |
|---|---|
| Subsystem | [editor-and-text](../subsystems/02-editor-and-text.md) |
| License | AGPL v3 (`AGPL-3.0-only`, inherited from workspace; no per-crate `LICENSE`) |
| Internal deps | 10 |
| Used by | 4 |
| Provenance | **`[Warp-derived/AGPL]`** — Warp's SumTree rich-text engine. **Marley did NOT fork it**; Marley shipped its own clean-room `marley_editor` (ropey/MIT), **`[Marley-original: the M15 editor]`**. Warp's engine is a reference only. See [subsystem — Provenance & licensing](../subsystems/02-editor-and-text.md) + the deep [Zed editor reference](../../zed_architecture/subsystems/03-editor-multibuffer.md). |

## Purpose

`warp_editor` (package `warp_editor`, dir `crates/editor`) is the **text-editing core** of Warp: the rope-backed buffer, selection/cursor model, anchors, undo/redo, find, diffing, and the layout/render pipeline that turns buffer content into laid-out, decorated lines. It is the shared engine behind both the terminal's command input editor and rich-text blocks (markdown, ipynb notebooks, mermaid diagrams, embedded items). It owns the data model that `syntax_tree`, `languages`, vim mode, search, and the AI layer all build on top of.

## Key types, modules & public API

Top-level modules (`crates/editor/src/lib.rs`): `content`, `decoration`, `editor`, `model`, `multiline`, `render`, `search`, `selection` (plus private `parallel_util`).

- **`content`** (`content/mod.rs`) — the heart. Submodules `buffer`, `text`, `core`, `anchor`, `edit`, `diff`, `find`, `selection`, `selection_model`, `undo`, `version`, `outline`, `markdown`, `mermaid_diagram`, `hidden_lines_model`.
  - `content::buffer::Buffer` / `BufferSnapshot` — the mutable buffer and its cheap immutable snapshot (`Buffer::new(TabIndentation)`, `bytes()`, `byte_len()`, line/tab/line-ending config).
  - `content::buffer` event & action enums: `BufferEvent`, `BufferEditAction<'a>`, `BufferSelectAction`, `EditOrigin`, `AutoScrollBehavior`, `ShouldAutoscroll`, `SelectionOffsets`, `InitialBufferState<'a>` (`plain_text`/`markdown`/`ipynb` constructors), `ContentFormat`, `VimInsertPoint`.
  - `content::text` — the rope and rich-text model: `BufferText`, `TextSummary` (the `sum_tree` summary), `BufferBlockItem`, `BlockType`, `IndentUnit`, `IndentBehavior`, `TextStyles`/`TextStylesWithMetadata`, `ColorMarker`, `LinkMarker`, `CodeBlockType`.
  - `content::anchor` — `Anchor`, `AnchorSide`, `AnchorUpdate` (stable positions across edits).
  - `content::edit` — `PreciseDelta`, `EditDelta`, `LaidOutRenderDelta`, `TemporaryBlock`, `ParsedUrl`.
- **`editor`** (`editor.rs`) — higher-level glue: `TextDecoration<'a>` (→ `to_paint_style_override`), `NavigationKey`.
- **`model`** (`model.rs`) — `BufferUpdateWrapper<'a>` with `apply_edit` / `apply_edit_with_autoscroll` / `buffer()`; the mutation entry surface UI code uses.
- **`render`** (`render/mod.rs`, `render/layout.rs`) — line layout and painting.
- **`search`** (`search.rs`) — in-buffer find/replace (regex-automata backed).
- **`selection`** / `multiline` — multi-cursor selection logic.
- **`decoration`** — decoration layers consumed by `syntax_tree` for highlight injection.

Feature: `test-util` (enables test scaffolding; also re-pulls `sum_tree`/`warp_core`/`warpui` with their test utils). Ships a `buffer_bench` criterion benchmark.

## Depends on (internal)

- [string-offset](./string-offset.md) — `CharOffset`/`ByteOffset` for all positions.
- [sum_tree](./sum_tree.md) — the rope/buffer is a `SumTree` with `TextSummary`.
- [warp_core](./warp_core.md) — core app types/services.
- [warp_util](./warp_util.md) — shared utilities (paths, etc.).
- [warpui](./warpui.md) and [warpui_core](./warpui_core.md) — UI/rendering primitives (color, geometry, context) the render layer paints into. *(MIT-licensed crates.)*
- [vim](./vim.md) — vim-mode integration (`VimInsertPoint` etc.).
- [asset_cache](./asset_cache.md) — cached assets for embedded/rich content.
- [ipynb_parser](./ipynb_parser.md) — Jupyter notebook parsing for ipynb buffers.
- [markdown_parser](./markdown_parser.md) — markdown block parsing for rich-text buffers.

## Used by (internal dependents)

- [syntax_tree](./syntax_tree.md) — parses/highlights `Buffer`s.
- [languages](./languages.md) — uses editor's `IndentUnit` and content types.
- [warp](./warp.md) — the application.
- [warp_tui](./warp_tui.md) — terminal-UI surface.

(4 dependents total.)

## Related crates

- [syntax_tree](./syntax_tree.md) + [languages](./languages.md) — the highlighting/grammar layer that sits directly on top of `warp_editor` (note the mutual-ish coupling: `languages`/`syntax_tree` depend on `warp_editor` and `warp_editor` is in turn a dependent of them through the build graph — see Notes).
- [vim](./vim.md), [warp_completer](./warp_completer.md), [warp_search_core](./warp_search_core.md) — feature layers over the buffer.

## Marley relevance

**KEEP / EXTEND.** This is the load-bearing text engine; rewriting or replacing it is out of scope and it carries no auth/network/Warp-cloud surface, so the de-auth (goal 3) and de-Warp rebrand (goal 3/4) work barely touches it. Two reasons to engage it:
- **Goal 1 (custom panel):** a Marley side panel that displays or edits text (logs, notes, an AI scratch buffer) should *reuse* `content::buffer::Buffer` + `model::BufferUpdateWrapper` rather than invent a new editor — `InitialBufferState::plain_text`/`markdown` are the cheap entry points.
- **Goal 2 (session read/write):** rendering session output into a scrollback/edit surface flows through this crate's buffer + render pipeline.

**Rename: defer.** The `warp_editor` package name is internal and only `02-editor-and-text` consumers see it. A future `marley_editor` rename is mechanical but has cross-cutting reach (10 deps in, 4 out, threads through the whole UI); batch it with the global de-Warp pass, not piecemeal. Do **not** strip rich-content features (ipynb/mermaid/markdown) for v1 — they are inert if unused and removing them ripples through `content`.

## Notes / gotchas

- Large crate with extensive `*_tests.rs` siblings per module — keep them; they pin buffer/anchor/undo invariants.
- Pulls heavyweight external deps directly: `html5ever`/`markup5ever`, `mermaid_to_svg`, `imara-diff`, `regex-automata`, `rayon`, `pathfinder_color`. The rich-text/markdown path is non-trivial; an "offline minimal" Marley build still compiles all of it.
- `edition = "2024"` (newer than the 2021 siblings in this batch) — needs a recent toolchain.
- Build-graph coupling with `languages`/`syntax_tree` is layered (editor provides the buffer; those crates provide grammars/highlighting and depend back on the editor) — there is no literal Cargo cycle, but treat the three as one subsystem when refactoring.
