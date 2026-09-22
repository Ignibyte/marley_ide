# Subsystem 02 — Editor & Text Engine (Warp)

Part of the Marley architecture docs. **Round 3 (refreshed @ M15)** — provenance-tagged and reconciled with
Marley's now-shipping editor. Marley is a **clean-room** reimplementation informed by Warp (warpdotdev/warp,
**AGPL-3.0**); the Warp source is a scratch reference, never committed. This doc deconstructs **Warp's** editor
engine. For Marley's own editor and the road ahead, read the **Marley status @ M15** section immediately below
and the Zed editor deconstruction it points to.

> Scope: Warp's SumTree-based rich-text editor and its supporting text primitives —
> `editor` (`warp_editor`), `sum_tree`, `string-offset`, `syntax_tree`, `languages`,
> `markdown_parser`, and `fuzzy_match`. In Warp this engine backs the command/prompt input and rich-text
> **display** blocks (markdown, AI output, notebooks) — it is a **command-input-shaped** editor, **not** a
> standalone file/code editor. **Marley built its own editor instead** (M15); Warp's engine is studied here as
> reference and contrast. The DEEP editor reference for *growing* Marley's editor is the **Zed** deconstruction
> ([02 — text/buffer/anchors](../../zed_architecture/subsystems/02-text-buffer-anchors.md),
> [03 — editor & multibuffer](../../zed_architecture/subsystems/03-editor-multibuffer.md)).

---

## Marley status @ M15

**Marley has its own editor now — clean-room, not a fork of `warp_editor`.** The M15 train (#249–258) shipped a
genuine, editable, **single-cursor plain-text** editor as a first-class *peer to the terminal*:
[`marley_editor::Buffer`](../../marley_architecture/editor.md) (backed by **`ropey` — MIT**), driven by
`marley_app::{editor_surface, code_view, input}` (see the
[M15 section of `app_shell.md`](../../marley_architecture/app_shell.md)). It is licensed **MIT OR Apache-2.0**
(Marley's workspace license), independently written from the *design*, and carries **none of Warp's AGPL
`warp_editor` code**. The AGPL engine deconstructed below is a **reference, not a dependency.**

What Marley M15 does — open a file → type → save:

- **Editable buffer** — `Buffer` over `ropey::Rope`: char/byte/line lengths, random-access `char_to_byte`/
  `byte_to_char`, `line_text`/`line_col`/`line_start`/`point_at`, and `edit(range, replacement, EditOrigin)`
  returning a `BufferDelta` + a monotonic `BufferVersion(u64)`.
- **Type / save / dirty** — `input::apply_editor_key` inserts/deletes at the caret; ⌘S writes `buffer.text()`
  to disk and clears the per-file dirty ● (dirty ⇔ `version != saved_version`); a faithful renderer draws each
  line straight from the buffer (un-truncated, exact column↔offset map via `code_view::LineLayout`).
- **Undo / redo** — `undo::UndoHistory`, a two-`Vec` invert-stack with contiguous same-origin coalescing (one
  ⌘Z undoes a typed run); ⌘Z / ⌘⇧Z.
- **Selection + clipboard** — a single `Selection { anchor, head }` (shift+arrows + mouse-drag, highlighted),
  ⌘C / ⌘X / ⌘V through the system clipboard, all routed through `Buffer::edit` (so one ⌘Z reverts a paste).
- **Caret motion** — char / word / line-home-end / document / vertical up-down (`marley_editor::movement`),
  bound to the standard mac keymap.
- **Persisted split panes** — file panes survive restart (`c=<path>` grid codec).
- **`EditOrigin::{Human, Agent}`** on every edit — the clean seam that keeps agent-driven writes distinguishable
  from human typing (and keeps the sellable brain on the far side of the licensing boundary).

**Marley now EXCEEDS Warp on the one axis that matters for an IDE-shaped product: a real editable *file*
editor.** Warp ships no standalone code editor — its `warp_editor` engine surfaces only as (a) the terminal's
**command-input line** and (b) rich-text **display** blocks. Marley's editor is a peer pane you open a file into
and edit, with save/undo/selection/clipboard/motions and persisted splits.

**But Warp's text *engine* is still richer than Marley's**, and closing that gap is the roadmap. Warp's `Buffer`
is a `SumTree` rich-text model with inline style markers, real **anchors**, **multi-cursor**, **vim**, async
**tree-sitter** highlighting, and markdown/ipynb/mermaid content. Marley's `Buffer` is single-cursor plain text:
its `Selection` stores **raw `CharOffset`s (not stable anchors)** that go stale on any edit, and there is **no
multi-cursor, no UTF-16/LSP, no tree-sitter, no rich text**. That frontier is mapped in depth against **Zed**
(the deep code-editor reference), not Warp:

| Capability Marley lacks | Deep reference |
|---|---|
| **Anchors** (positions that survive edits) — prerequisite for LSP, multi-cursor, marks | [Zed 02 — text/buffer/anchors](../../zed_architecture/subsystems/02-text-buffer-anchors.md) |
| **UTF-16 coordinate + LSP** position round-trip | [Zed 02 §4](../../zed_architecture/subsystems/02-text-buffer-anchors.md) |
| **Multi-cursor**, **DisplayMap** transform stack (soft-wrap/fold/inlay), **MultiBuffer** | [Zed 03 — editor & multibuffer](../../zed_architecture/subsystems/03-editor-multibuffer.md) |
| **tree-sitter** syntax highlighting + grammars | Warp `syntax_tree`/`languages` (§5 below) · [Zed 04](../../zed_architecture/subsystems/04-language-syntax-treesitter.md) |
| **Transactions / time-grouped undo** | [Zed 02 §5](../../zed_architecture/subsystems/02-text-buffer-anchors.md) |

> **Read the Zed editor docs for the road ahead.** The Zed deconstruction
> ([02](../../zed_architecture/subsystems/02-text-buffer-anchors.md),
> [03](../../zed_architecture/subsystems/03-editor-multibuffer.md)) is the DEEP editor reference now — it maps
> each capability above to a staged, licence-tagged Marley plan (anchors-first, delta-log before CRDT, UTF-16
> with LSP). Warp's editor, deconstructed below, is a command-input-shaped engine: study its `EditOrigin` split
> and its capture→submit path (§6), but grow the editor itself from the Zed plan.

---

## 1. Purpose & big picture

This subsystem is the **text model + editing engine** that Warp uses everywhere a user
types or where rich text is displayed/edited. It is deliberately split into a generic,
reusable lower stack and a concrete model that lives outside these crates:

- **`sum_tree`** — a persistent, copy-on-write B-tree (the "rope") that stores text as
  summarized items. Everything else is built on it.
- **`string-offset`** — newtypes for text positions (`CharOffset`, `ByteOffset`) so byte
  vs. char offsets are never confused.
- **`editor` (`warp_editor`)** — the heart: the `Buffer` content model (a
  `SumTree<BufferText>`), selections, anchors, undo, find, markdown/code awareness, and a
  rendering/layout model. Exposes the `CoreEditorModel` trait that concrete editors
  implement.
- **`markdown_parser`** — pure parser that turns markdown/HTML source into `FormattedText`,
  which `Buffer` ingests to build its styled content tree.
- **`languages`** — registry of 34 tree-sitter grammars (embedded via `rust-embed`),
  resolving a language by name or filename.
- **`syntax_tree`** — drives tree-sitter incrementally against a `Buffer` snapshot to
  produce syntax-highlight color maps and indent deltas asynchronously.
- **`fuzzy_match`** — fuzzy / wildcard string matching used by completion & file search
  surfaces (adjacent to, not part of, the core text model).

The **concrete editor model is NOT in these crates.** It lives in the `warp` app crate at
`app/src/code/editor/model.rs` as `CodeEditorModel`, which `impl`s the
`CoreEditorModel`/`PlainTextEditorModel` traits from `warp_editor`. This separation is the
key extension seam for Marley: the generic engine is reusable, and new surfaces plug in by
implementing the trait (the TUI input view does exactly this — see §6).

---

## 2. `sum_tree` — the rope

`crates/sum_tree/src/lib.rs` (+ `cursor.rs`). ~2k LOC, only deps are `arrayvec` + `log`.

- `SumTree<T: Item>(Arc<Node<T>>)` — a balanced B-tree where every node caches a
  `T::Summary`. `Node` is either `Internal` or `Leaf`; leaves hold up to `2 * TREE_BASE`
  items (`TREE_BASE = 6` in release, `2` under `test-util` to force deeper trees in tests).
  Wrapping the node in `Arc` makes the tree **cheaply cloneable / copy-on-write** — this is
  how `BufferSnapshot` (§4) takes an immutable point-in-time copy for off-thread parsing.
- Core traits:
  - `Item` — a value with an associative `Summary` (`fn summary(&self) -> Self::Summary`).
  - `Dimension<'a, Summary>` — a quantity that can be accumulated from summaries
    (`add_summary`). Multiple dimensions can be derived from one summary; this is how a
    single tree is indexed simultaneously by char offset, byte offset, line/`Point`, block
    count, link count, etc.
  - `KeyedItem` — items that additionally expose an `Ord` key.
- `Cursor` / `FilterCursor` (`cursor.rs`) — seekable iterators that walk the tree while
  accumulating two dimensions (`S`, `U`), using `SeekBias::{Left,Right}` to resolve which
  side of a boundary to land on. This is the primitive used to convert a `CharOffset` to a
  `Point`, find the leaf containing an offset, slice ranges, etc.
- Key ops: `push`, `push_tree`, `extend`, `extent::<D>()`, `summary()`, `cursor()`.

This is a textbook Zed-style sum tree — the *concept* is **`[permissive]`** (Zed's own `sum_tree` is Apache-2.0).
**Marley does not use this crate:** `marley_editor` is backed by **`ropey` (MIT)**, which already delivers the
cheap-clone / `O(log n)` byte↔char↔line rope Marley needs. A Marley SumTree becomes worth building only if a
future summary dimension ropey cannot carry forces it (see the Provenance section and
[Zed 02 §2.5](../../zed_architecture/subsystems/02-text-buffer-anchors.md)).

---

## 3. `string-offset` — position newtypes

`crates/string-offset/src/lib.rs` (~266 LOC).

- `CharOffset(usize)` and `ByteOffset(usize)` — `Copy` newtypes with full arithmetic
  (`Add/Sub/AddAssign`, `add_signed`, `empty_range`, `zero`, `range`) generated by the
  `impl_offset!` macro. Both are `Serialize`/`Deserialize` and `GetSize`.
- The entire editor API is typed in these offsets rather than raw `usize`, which prevents
  the classic byte-vs-char bug class. `CharOffset` is the user-facing currency (cursor
  positions, selection ranges, decoration ranges); `ByteOffset` is used at tree-sitter / raw
  buffer boundaries.

Both implement `sum_tree::Dimension<BufferSummary>` (in `editor`'s `text.rs`), so a tree
cursor can seek by either.

---

## 4. `editor` (`warp_editor`) — the engine

`crates/editor/` — ~54k LOC of `src` (largest crate in this subsystem). `lib.rs` exposes:
`content`, `decoration`, `editor`, `model`, `multiline`, `render`, `search`, `selection`.

### 4.1 `content` — the document model

`content/buffer.rs` (`Buffer`, ~6k LOC) is the central type:

```rust
pub struct Buffer {
    content: SumTree<BufferText>,   // the document
    internal_anchors: Anchors,      // stable positions across edits
    undo_stack: UndoStack,
    content_version: ContentVersion,// monotonic; user-edit identity
    version: BufferVersion,         // monotonic; any state change
    line_ending_mode: LineEnding,
    ...
}
```

- **`BufferText`** (`content/text.rs`) is the `sum_tree::Item` stored in the tree. It is NOT
  one-char-per-node; variants are `Text { fragment: ArrayString<N>, char_count }`,
  `Newline`, `Marker { BufferTextStyle, MarkerDir }` (bold/italic/underline/code/strike,
  paired start/end), `Link`, `Color`, `BlockItem`, `BlockMarker` (block style for a whole
  paragraph), and `Placeholder` (ghost/autosuggest text). So **rich-text styling is encoded
  inline as marker items in the same tree as the characters** — there is no separate span
  map for inline style.
- **`BufferSummary`** = `{ style: Option<Box<StyleSummary>>, text: TextSummary, block:
  BlockSummary }`. From it the tree derives dimensions for `CharOffset`, `ByteOffset`,
  `Point` (row/col), `LineCount`, `BlockCount`, `LinkCount`, `SyntaxColorId`. This is how an
  O(log n) seek converts any offset form to any other.
- **`BufferSnapshot`** — `{ content: SumTree<BufferText>, byte_len }`, a cheap immutable
  clone handed to `syntax_tree` for off-thread parsing.

Editing is **action-based, not direct mutation**:

- **`BufferEditAction<'a>`** (the write API) — `Insert { text, style, override_text_style }`,
  `InsertAtCharOffsetRanges`, `InsertForEachSelection`, `Enter { force_newline, style }`,
  `Backspace`, `Delete(Vec1<Range<CharOffset>>)`, `Style`/`Unstyle`, `Link`/`Unlink`,
  `StyleBlock`, `ReplaceWith(InitialBufferState)`, `Undo`, `Redo`, `Indent`, `VimEvent
  { text, insert_point, cursor_offset_len }`, embedding ops, etc.
- **`BufferSelectAction`** (selection-only) — `MoveLeft/Right`, `ExtendLeft/Right`,
  `SelectAll`, `AddCursorAt`, `SetSelectionOffsets`, multi-cursor variants on `Vec1`.
- **`EditOrigin`** — `UserTyped` | `UserInitiated` | `SystemEdit`. This three-way split is
  load-bearing: it controls undo grouping, autoscroll, and lets downstream code tell
  user typing apart from programmatic/streamed writes (e.g. an AI agent streaming output
  into a buffer is `SystemEdit`). **Highly relevant to Marley: a UI component that writes
  into an editor would use `SystemEdit`; a user typing is `UserTyped`.**
- Edits go through `Buffer::apply_edit(action, origin, selection_model, ctx)`, produce an
  `EditResult { undo_item, delta: Option<EditDelta>, anchor_updates }`, and emit
  `BufferEvent::{ContentChanged { delta, origin, buffer_version, ... }, SelectionChanged
  { active_text_styles, ... }}`. Consumers react to these events to re-render and to update
  formatting toolbars.

Other `content` modules: `anchor.rs` (`Anchor`/`Anchors` — positions that survive edits),
`selection.rs` + `selection_model.rs` (`BufferSelectionModel`, multi-cursor `Vec1`
selections), `undo.rs` (`UndoStack`, bounded to 30 entries by default), `find.rs`
(in-buffer find), `diff.rs` (imara-diff backed), `markdown.rs` (`BufferMarkdownParser`,
`BufferToFormattedText` — bridges the `markdown_parser` crate into the tree), `outline.rs`,
`segmentation.rs`, `mermaid_diagram.rs`, `hidden_lines_model.rs`, `version.rs`
(`BufferVersion`).

### 4.2 `model` — the `CoreEditorModel` trait (the plug-in seam)

`content/buffer.rs` holds raw data; `model.rs` defines the **trait a concrete editor
implements**:

- `CoreEditorModel: Entity` with associated `type T`, requiring handles to the sub-models:
  `content() -> ModelHandle<Buffer>`, `buffer_selection_model()`, `selection_model()`,
  `render_state() -> ModelHandle<RenderState>`, plus `active_text_style()`,
  `buffer_version()`, and the `on_buffer_version_updated` hook.
- `update_content(action, ctx)` is the single funnel for mutations: it wraps the buffer in a
  `BufferUpdateWrapper` (which exposes `apply_edit` / `apply_edit_with_autoscroll`) and
  guarantees `on_buffer_version_updated` fires synchronously after every change.
- Convenience methods built on top: `insert`, **`user_insert(text)`** (→ `Insert` with
  `EditOrigin::UserTyped`), `system_insert_autoscroll_vertical_only`, `truncate`,
  `clear_buffer`, `rebuild_layout`, `set_document_path`. `PlainTextEditorModel` is a
  sub-trait for plain-text surfaces.

The concrete implementor in the GUI is **`CodeEditorModel`** at
`app/src/code/editor/model.rs` (emits `CodeEditorModelEvent::{ContentChanged,
SelectionChanged, ViewportUpdated, SyntaxHighlightingUpdated, DiffUpdated, ...}`). It can be
constructed in a normal pixel layout mode or in **`LayoutMode::CharCell`** (terminal-cell
mode) — the latter is what makes the same editor usable inside a TUI.

### 4.3 `render` — layout & elements

`render/model/` builds the on-screen model from the buffer: `RenderState`, `viewport.rs`,
`offset_map.rs` / `table_offset_map.rs` (map buffer offsets ↔ rendered positions,
accounting for soft-wrap, hidden sections, tables), `location.rs`, `positioned.rs`,
`saved_positions.rs`, `bounds.rs`. `render/element/` is the rich-text element zoo —
`paragraph`, `header`, `table`, `image`, `mermaid`, `ordered_list`/`unordered_list`,
`task_list`, `horizontal_rule`, `runnable_command`, `lens_element`, etc. — plus `paint.rs`
and `layout.rs`. The `editor.rs` interop module defines `EditorView`,
`RunnableCommandModel`, `EmbeddedItemModel`, `TextDecoration`, and `NavigationKey`, the
traits a host view implements so blocks can render footers/borders and so syntax/search
decorations can be merged (`TextDecoration::to_paint_style_override` merges base color map +
override color map + underlines into a `PaintStyleOverride`).

`RunnableCommandModel::render_block_footer` is explicitly documented as being able to host a
**"button to insert a command into the terminal input"** — a built-in example of the
editor-to-session write path Marley wants to expand.

---

## 5. Syntax: `languages` + `syntax_tree`

- **`languages`** (`crates/languages/src/lib.rs`): a `lazy_static` `LanguageRegistry`
  caching `Arc<Language>` by name; grammars are **embedded into the binary** via
  `#[derive(RustEmbed)] #[folder = "grammars"]`. `SUPPORTED_LANGUAGES` lists 34 languages
  (rust, go, typescript/tsx, python, shell, etc.). Lookups: `language_by_name`,
  `language_by_filename(StandardizedPath)`. Built on `arborium` (Warp's tree-sitter
  wrapper).
- **`syntax_tree`** (`crates/syntax_tree/src/lib.rs`): `SyntaxTreeState` holds a thread-local
  `Parser`, a small `HashMap<BufferVersion, Tree>` cache (`MAX_SYNTAX_TREES = 3`), and a
  `WeakModelHandle<Buffer>`. It re-parses incrementally using tree-sitter `InputEdit`s
  derived from the buffer's `PreciseDelta`, **skipping files > `MAX_PARSE_BYTES` (2 MB)** to
  avoid tree-sitter's super-linear memory blowup. Parsing is async (`futures` +
  `AbortHandle`); when done it emits `DecorationStateEvent::DecorationUpdated { version }`,
  and the editor pulls a `RangeMap<CharOffset, ColorU>` highlight map (cached per
  `HighlightCacheKey` of version + ranges + language). It also computes indent deltas
  (`indent_query`). It depends directly on `warp_editor` (`Buffer`, `BufferSnapshot`,
  `DecorationLayer`).

This subsystem is purely cosmetic/assist (highlighting + indent) and does not gate input —
useful context for Marley's UI work but not on the critical path for session writes.

---

## 6. How text is captured and submitted (Warp's TUI input pattern)

> **Marley note (M15):** the framing below was written pre-M15 as "the Marley-critical path." It is now
> **Warp's** pattern, kept as reference. Marley shipped its own equivalent — `marley_app::input::apply_key` /
> `apply_editor_key` drive a `marley_editor::Buffer` + caret directly (insert / delete / motion); the prompt's
> `Enter` submits a trimmed `String`, while the editor's `Enter` inserts a newline. Warp's **`EditOrigin`**
> three-way split (§4.1) is the one idea Marley carried across in concept as `EditOrigin::{Human, Agent}`. See
> the **Marley status @ M15** section above.

The cleanest in-tree example of "a UI component captures typed text and submits it" is the
**TUI input view**, `crates/warp_tui/src/input/`:

1. **Hold a model.** `TuiInputView` (`view.rs`) owns a `ModelHandle<CodeEditorModel>`
   constructed in `LayoutMode::CharCell`, plus TUI-only state (a `KillBuffer`,
   `scroll_offset`, `max_visible_rows`). So the *generic editor engine is the input buffer.*
2. **Keystrokes → typed actions.** Key events are dispatched as a `TuiInputAction` enum
   (`InsertChar(char)`, `InsertNewline`, `Submit`, `Backspace`, `MoveLeft`,
   `DeleteWordBackward`, `KillToLineEnd`, `Yank`, `Undo`, …) — a readline-style keymap layered
   on top of the editor.
3. **Mutations go through the model.** e.g. character insert calls
   `model.update(ctx, |m, ctx| m.user_insert(&text, ctx))` →
   `CoreEditorModel::insert(text, EditOrigin::UserTyped, ctx)` →
   `BufferEditAction::Insert`. Deletions build a `BufferEditAction::Delete(vec1![range])`
   via `apply_edit`. Yank re-inserts killed text the same way.
4. **Submit.** `TuiInputView::submit` reads the current content with `self.plain_text(ctx)`,
   emits **`TuiInputViewEvent::Submitted(String)`**, then `m.clear_buffer(ctx)` and resets
   scroll. The owner of the view consumes `Submitted(String)` and is responsible for writing
   that string to the actual shell/PTY session (in `warp_terminal`'s shell layer).

So the canonical flow is: **typed chars → `BufferEditAction` on a `CodeEditorModel` → read
back as a plain `String` → emit a `Submitted` event → host writes to the session.** For
Marley's custom panel, the same recipe applies: instantiate a `CodeEditorModel`, drive it
with `BufferEditAction`s (or `user_insert` for human typing / `SystemEdit` inserts for
programmatic/agent text), read the buffer with `plain_text` / `text_in_range`, and route the
resulting string to whatever session-write API the terminal subsystem exposes.

Reading content back out: `Buffer::text_in_range(range)` and the view-level
`plain_text(ctx)` produce plain `String`s; `BufferSnapshot::bytes()` yields a `ByteOffset`
iterator for parsers.

---

## 7. `markdown_parser` & `fuzzy_match` (adjacent)

- **`markdown_parser`** (`crates/markdown_parser`, ~6.7k LOC): a dependency-light
  (`nom`, `html5ever`) parser producing `FormattedText` / `FormattedTextLine` /
  `FormattedTextFragment` / `FormattedTable` / `FormattedTaskList`, etc. Entry points:
  `parse_markdown`, `parse_markdown_with_gfm_tables`, `parse_inline_markdown`, `parse_html`.
  Also computes line-based diffs (`FormattedTextDelta`, `compute_formatted_text_delta`) so
  streamed markdown (AI output) can be applied incrementally. The `editor` crate consumes
  these types in `content/markdown.rs` to build the styled buffer tree. It defines its own
  `Action` trait to avoid a circular dep on `warpui_core`.
- **`fuzzy_match`** (`crates/fuzzy_match`, ~1k LOC): wraps `fuzzy-matcher`'s
  `SkimMatcherV2`. Provides `match_indices` (smart-case), `match_indices_case_insensitive`,
  and glob-style `match_wildcard_pattern[_case_insensitive]` plus `contains_wildcards`.
  Returns `FuzzyMatchResult { score, matched_indices }`. Used by command/file search and
  completion UIs — orthogonal to the text model but part of the "editor experience."

---

## 8. Dependency map (within this subsystem)

```
string-offset ──┐
sum_tree ───────┼──> editor (warp_editor) ──> syntax_tree
markdown_parser ┘                    ^            ^
languages ───────────────────────────┘            │
                              (languages also dep editor for IndentUnit)
fuzzy_match  (standalone)
                                   editor ─consumed by─> warp_tui (TUI input),
                                                         app/warp (CodeEditorModel), warpui
```

- `editor` depends on `sum_tree`, `string-offset`, `markdown_parser`, `warpui_core`,
  `warp_core`, `vim`, `ipynb_parser`, `imara-diff`, `line-ending`.
- `syntax_tree` depends on `editor`, `languages`, `arborium`, `string-offset`.
- `languages` depends on `editor` (for `IndentUnit`) and `arborium`.
- `markdown_parser`, `sum_tree`, `string-offset`, `fuzzy_match` are leaf-ish (no intra-subsystem deps except as shown).

---

## 9. Marley relevance

**Superseded by M15 — Marley built its own editor rather than reuse Warp's engine.** The round-1 plan here was
to instantiate Warp's `CodeEditorModel` and drive it with `BufferEditAction`s. That is moot: Marley wrote a
clean-room `marley_editor` (ropey / MIT) instead of forking the AGPL `warp_editor`. What remains relevant:

- **What Marley took (in concept, not code).** The **`EditOrigin` split** — Warp's `UserTyped` /
  `UserInitiated` / `SystemEdit` — is the idea Marley carried across as `EditOrigin::{Human, Agent}`: the seam
  that tells human typing apart from agent-issued writes and keeps agent writes on the clean side of the
  licensing boundary. The capture → read-back → submit shape (§6) is mirrored by `marley_app::input`
  (`apply_key` → `submit_line` for the prompt; `apply_editor_key` for the multi-line editor).
- **What Marley did NOT take.** The SumTree rich-text model, inline style markers, anchors, multi-cursor, vim,
  markdown/ipynb/mermaid content, and the `CoreEditorModel` / `RenderState` layout stack. Marley's editor is
  single-cursor plain text; the richer engine is the frontier (see Marley status @ M15 above).
- **The frontier reference is Zed, not Warp.** Growing Marley's editor toward anchors / UTF-16-LSP /
  multi-cursor / DisplayMap / MultiBuffer is planned against the **Zed** deconstruction
  ([02](../../zed_architecture/subsystems/02-text-buffer-anchors.md),
  [03](../../zed_architecture/subsystems/03-editor-multibuffer.md)), which carries the staged, licence-tagged
  plan. Warp's `syntax_tree` / `languages` (§5) remain a useful reference for the eventual tree-sitter layer.
- **De-auth / rebrand: N/A here.** Marley ships none of these AGPL crates, so there is nothing in this
  subsystem to de-auth or rename; the only obligation is to keep Warp's AGPL editor code *out* of Marley's tree
  (it is — Marley's editor is `marley_editor`, MIT/Apache). See **Provenance & licensing** below.

---

## Provenance & licensing

Marley is a **clean-room** project: it studies the Warp source (cloned only into session scratch, never
committed) and writes its own permissively-licensed code. **Warp is AGPL-3.0**; **Marley's workspace is
`MIT OR Apache-2.0`**. So the AGPL editor engine catalogued in this doc is a *reference*, and everything Marley
actually ships is either its own code or built on permissive deps. Tags mirror the
[Zed deconstruction](../../zed_architecture/README.md):

- **`[Warp-derived/AGPL]`** — Warp's *specific rendition*, under **AGPL-3.0**. Study the design; **reimplement
  from the public concept**, never copy. AGPL-derived design may inform only Marley's editor/terminal layer —
  **never the brain/agent layer** (the sold product must stay copyleft-clean).
- **`[permissive]`** — textbook CS, or a permissively-licensed dep Marley may use freely: the **summarized/
  augmented B-tree** (Zed's `sum_tree` is Apache-2.0; the concept is public), **ropey (MIT)** — Marley's text
  store, **tree-sitter (MIT)**, **`fuzzy-matcher` (MIT)**, and **char/byte offset newtypes** (trivial).
- **`[Marley-original: the M15 editor]`** — Marley's own clean-room code: `marley_editor` (the M15 editable
  editor) and `marley_text_offsets`, both `MIT OR Apache-2.0`.

| Warp crate (this subsystem) | Warp license | Provenance | Marley's stance |
|---|---|---|---|
| **`editor` (`warp_editor`)** — SumTree rich-text engine, anchors, multi-cursor, vim, render pipeline | AGPL-3.0 | **`[Warp-derived/AGPL]`** | **NOT forked.** Marley shipped its own **`marley_editor`** (**`[Marley-original: the M15 editor]`**, ropey/MIT). Warp's engine is a design reference; the deep reference is [Zed 02/03](../../zed_architecture/subsystems/03-editor-multibuffer.md). |
| **`sum_tree`** — copy-on-write summarized B-tree | AGPL-3.0 (Warp's copy) | concept **`[permissive]`**; Warp's copy `[Warp-derived/AGPL]` | Marley uses **`ropey` (MIT)** instead — no sum_tree. Build one only if a custom summary dimension forces it ([Zed 02 §2.5](../../zed_architecture/subsystems/02-text-buffer-anchors.md)), then from the Apache concept. |
| **`string-offset`** — `CharOffset`/`ByteOffset` newtypes | AGPL-3.0 | trivial → concept **`[permissive]`** | Reimplemented as **`marley_text_offsets`** (**`[Marley-original]`**, MIT/Apache). |
| **`syntax_tree`** — incremental tree-sitter highlight/indent runtime | AGPL-3.0 | **`[Warp-derived/AGPL]`** over `[permissive]` tree-sitter | **Frontier — not built.** Mapped in [Zed 04](../../zed_architecture/subsystems/04-language-syntax-treesitter.md). |
| **`languages`** — grammar registry (34 embedded grammars) | AGPL-3.0 (registry) | registry **`[Warp-derived/AGPL]`**; grammars are third-party `[permissive]` | **Frontier — not built.** |
| **`markdown_parser`** — `FormattedText` model | AGPL-3.0 | **`[Warp-derived/AGPL]`** (markdown parsing itself is public) | **Not built** — Marley has no rich-text buffer yet. |
| **`fuzzy_match`** — `SkimMatcherV2` wrapper + glob matcher | AGPL-3.0 (wrapper) | core **`[permissive]`** (`fuzzy-matcher`, MIT); thin wrapper `[Warp-derived/AGPL]` | Marley's palette/finder (`marley_app::{palette, finder, command_bar}`) do their own matching — **`[Marley-original]`**. |

**The boundary that protects the business model:** editor work may study AGPL design and stand on permissive
deps, but the **brain / agent layer must never import or derive from Warp's AGPL editor.** The brain touches text
only through Marley's own API — `Buffer::edit(range, replacement, EditOrigin::Agent)` — and `EditOrigin::Agent`
is the existing clean seam that keeps agent writes on the permissive side.

---

## 10. Key files

- `crates/sum_tree/src/lib.rs`, `crates/sum_tree/src/cursor.rs` — the rope.
- `crates/string-offset/src/lib.rs` — `CharOffset`/`ByteOffset`.
- `crates/editor/src/content/buffer.rs` — `Buffer`, `BufferEditAction`, `BufferSelectAction`,
  `EditOrigin`, `BufferEvent`, `BufferSnapshot`.
- `crates/editor/src/content/text.rs` — `BufferText` (the SumTree item) + `BufferSummary`
  dimensions.
- `crates/editor/src/model.rs` — `CoreEditorModel` trait, `user_insert`/`insert`/
  `clear_buffer`, `BufferUpdateWrapper`.
- `crates/editor/src/editor.rs` — `EditorView`, `RunnableCommandModel`, `TextDecoration`,
  `NavigationKey`.
- `crates/editor/src/render/model/mod.rs`, `.../offset_map.rs`, `.../viewport.rs` — layout.
- `crates/warp_tui/src/input/view.rs` — concrete capture→`BufferEditAction`→
  `Submitted(String)` example (the de-facto reference for Marley's write path).
- `app/src/code/editor/model.rs` — `CodeEditorModel`, the GUI's concrete `CoreEditorModel`.
- `crates/syntax_tree/src/lib.rs` — `SyntaxTreeState` (async tree-sitter highlighting).
- `crates/languages/src/lib.rs` — `LanguageRegistry`, embedded grammars.
- `crates/markdown_parser/src/lib.rs`, `.../markdown_parser.rs` — `FormattedText` parsing.
- `crates/fuzzy_match/src/lib.rs` — fuzzy/wildcard matching.

---

## 11. Open questions (round 3 — mostly resolved by M15)

1. **~~Reuse Warp's engine / `CodeEditorModel` construction cost / `arborium` provenance.~~** RESOLVED —
   Marley did **not** reuse Warp's engine; it built `marley_editor` clean-room (M15 #249–258). The
   construction-cost, injection-API, and `arborium`-provenance questions are moot for Marley's editor.
2. **Anchors.** Marley's `Selection` still holds raw `CharOffset`s that go stale on edits — the single largest
   gap. Sequencing (delta-log anchor first, CRDT deferred) is specified in
   [Zed 02 §3](../../zed_architecture/subsystems/02-text-buffer-anchors.md). *Prerequisite for LSP and
   multi-cursor.*
3. **UTF-16 / LSP.** No UTF-16 coordinate yet; needed before Marley can speak LSP. A cheap additive bridge over
   ropey — land it *with* LSP ([Zed 02 §4](../../zed_architecture/subsystems/02-text-buffer-anchors.md)).
4. **tree-sitter.** No syntax highlighting yet. Warp's `syntax_tree` / `languages` (§5) and
   [Zed 04](../../zed_architecture/subsystems/04-language-syntax-treesitter.md) are the references; tree-sitter
   itself is MIT (`[permissive]`).
5. **Rich text.** Warp's inline-marker `BufferText` gives markdown/ipynb/mermaid "for free"; Marley's plain-text
   buffer does not. Only worth building if a Marley surface needs rendered rich content in an *editable* buffer
   (agent/markdown output routes elsewhere today).
