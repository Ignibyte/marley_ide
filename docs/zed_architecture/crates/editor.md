# editor

> Per-crate reference (Marley, Zed round 2 — the granular per-crate map). Crate dir: `crates/editor`
> (session Zed clone `…/scratchpad/zed-src/crates/editor`). Zed is the EDITOR reference for Marley's editing
> surface. **This is the most GPL-sensitive, most reimplementation-critical crate in the whole reference.**
>
> The strategic view — the road from Marley's single-cursor code view to a Zed-class editor — is
> [`../subsystems/03-editor-multibuffer.md`](../subsystems/03-editor-multibuffer.md). **This doc is the crate
> reference:** the module-by-module catalog of what each of `editor/src`'s ~54 modules owns, the concrete
> Marley reimplementation mapping, and per-module provenance.

| | |
|---|---|
| Subsystem | [03 — Editor & Multibuffer](../subsystems/03-editor-multibuffer.md) |
| License | **GPL-3.0-or-later** (explicit per-crate `license` in `Cargo.toml`; symlinks `LICENSE-GPL`) |
| Builds on | `gpui` (Apache-2.0), `sum_tree` (Apache-2.0), `rope`/`text`/`language`/`multi_buffer` (GPL) |
| Internal deps | ~41 Zed workspace crates |
| Used by | 51 crates (incl. `vim`, `search`, `project_panel`, `terminal_view`, `agent`, `git_ui`, `zed`) |
| Provenance | `[Zed-derived]` editing logic · `[gpui Apache-2.0]` primitives · `[public: CS/Unicode]` patterns |

## Purpose

`editor` is **the editing element** of Zed: it turns one or more text buffers into an on-screen, editable,
multi-cursor, syntax-highlighted, LSP-decorated view. It owns the `Editor` composer (a gpui `Entity`, not
itself an `Element` — it *produces* an `EditorElement`), the anchor-array selection model, the six-layer
`DisplayMap` coordinate-transform stack, display-space motions, the keystroke→per-selection-edit input path,
the gpui paint element, and a very large periphery of LSP/git/task/inlay features hung off that core. The same
type serves every editing surface in the app — the file pane, the single-line command input, the auto-height
chat box, the minimap, the multi-file search/diagnostics view — parameterized by an `EditorMode` enum and (for
multi-file) by sitting on a `MultiBuffer` (see [`multi_buffer.md`](./multi_buffer.md), the capstone dependency).

At ~456 KB `editor.rs` + ~494 KB `element.rs` + a 1.28 MB `editor_tests.rs`, this is Zed's single largest and
most load-bearing crate. Marley reimplements its **decomposition** (how the pieces are cut apart), not its
code.

## Key types, modules & public API

The lib root is `src/editor.rs` (`[lib] path = "src/editor.rs"`), which declares ~50 modules. Grouped by
role; the first group is the reimplementation-critical core.

### A. The core editing pipeline `[Zed-derived]` + `[gpui Apache-2.0]`

- **`editor.rs` — the `Editor` composer (the god-object).** `pub struct Editor` is a single ~230-field struct
  that owns the whole session and wires the pipeline: `buffer: Entity<MultiBuffer>`, `display_map:
  Entity<DisplayMap>`, `selections: SelectionsCollection`, `scroll_manager: ScrollManager`, `mode: EditorMode`,
  plus the three multi-cursor gesture state machines (`columnar_selection_state`, `add_selections_state`,
  `select_next_state`/`select_prev_state`), `selection_history`, `autoclose_regions`, `snippet_stack`,
  `ime_transaction`, `blink_manager`, and a long tail of LSP/git/diagnostics/edit-prediction state. Key public
  enums: **`EditorMode`** (`SingleLine` | `AutoHeight { min_lines, max_lines }` | `Full { … }` | `Minimap {
  parent }}` — the one type-serves-every-surface knob), **`SoftWrap`** (`None` | `EditorWidth` | `Bounded(u32)`
  | `GitDiff`), **`SelectMode`** (`Character | Word(Range<Anchor>) | Line(Range<Anchor>) | All`),
  **`EditorStyle`**. Everything the `Editor` owns is a **handle to a sub-entity** (`Entity<T>`) it observes and
  re-renders on — this is what lets soft-wrap recompute off the main update and two editors share one buffer.
  `Editor` never draws: `render` returns `EditorElement::new(…)`. ~253 `pub fn` methods; input arrives as gpui
  Actions registered in `register_actions` plus raw text via `EntityInputHandler` (see `input.rs`).

- **`actions.rs` — the action vocabulary.** ~243 gpui `Action` structs (`MoveLeft`, `SelectNext`,
  `AddSelectionAbove`, `DeleteToNextWordEnd`, `Fold`, `ToggleComments`, …) — the declarative surface the keymap
  binds to and `Editor` methods handle. `[gpui Apache-2.0]` for the `Action` machinery; `[Zed-derived]` for the
  specific verb set.

- **`selections_collection.rs` — the multi-cursor model.** `pub struct SelectionsCollection { next_selection_id,
  line_mode, disjoint: Arc<[Selection<Anchor>]>, pending: Option<PendingSelection>, select_mode, is_extending }`
  — the committed cursors are a **sorted, non-overlapping array of anchor-based selections** plus one `pending`
  (the drag). Concrete coordinates are resolved on demand: `all::<D>(&snapshot)` folds the anchor array into any
  coordinate space `D`. The write side, **`MutableSelectionsCollection`**, holds the invariant: `select` /
  `select_ranges` / `select_anchor_ranges` / `select_display_ranges` / `insert_range` / `delete` /
  `build_columnar_selection`, and the motion fan-outs `move_with` / `move_offsets_with` / `move_heads_with` /
  `move_cursors_with` / `replace_cursors_with` — each re-sorts and merges overlaps.

- **`selection.rs` — the multi-cursor *gestures*** (on `Editor`). The three state machines: **columnar / block**
  (`add_selection_above/below`, `AddSelectionsState`), **⌘D select-next** (`select_next` /
  `select_next_match_internal` / `select_all_matches`, backed by a `SelectNextState { query: AhoCorasick,
  wordwise, done }` — first ⌘D selects the surrounding word and builds an aho-corasick automaton, each next ⌘D
  streams it over the buffer to add a cursor at the next match), and **tree-sitter node grow/shrink**
  (`select_larger_syntax_node`, `SelectSyntaxNodeHistory`). Once N cursors exist, every edit and motion just
  maps over the array — that is the entire trick.

- **`display_map.rs` + `display_map/` — the six-layer transform stack.** `pub struct DisplayMap` chains the
  layers in order: `inlay_map: InlayMap` → `fold_map: FoldMap` → `tab_map: TabMap` → `wrap_map:
  Entity<WrapMap>` → `block_map: BlockMap` → highlights (`text_highlights`, `inlay_highlights`,
  `semantic_token_highlights`) + `crease_map`. The immutable per-frame `DisplaySnapshot` nests the layer
  snapshots (`block_snapshot` at the top) so any coordinate converts through the whole stack. The top
  coordinate is `pub struct DisplayPoint(BlockPoint)` / `DisplayRow(pub u32)`. Each layer follows **one uniform
  contract**: a `Transform` enum (pass-through `Isomorphic` vs replace/inject) stored in a `SumTree`, a
  `TransformSummary { input: TextSummary, output: TextSummary }` (folding these up the tree gives O(log n)
  *bidirectional* coordinate conversion), a `Snapshot`, and a `sync(snapshot, edits) -> (snapshot, edits')`
  that translates lower-layer edits into its own coordinates so an edit only recomputes the touched region.
  The submodules and the coordinate newtype each introduces:
  - `display_map/inlay_map.rs` — inject inlay hints not in the file → `InlayOffset(MultiBufferOffset)`,
    `InlayPoint(Point)`.
  - `display_map/fold_map.rs` — hide folded regions behind a placeholder → `FoldPoint(Point)`,
    `FoldOffset(MultiBufferOffset)`.
  - `display_map/tab_map.rs` — expand hard tabs to N columns → `TabPoint(Point)`.
  - `display_map/wrap_map.rs` — soft-wrap long lines (consults gpui line widths) → `WrapPoint(Point)`,
    `WrapRow(u32)`. The only layer that is itself an `Entity` (wrapping recompute is async).
  - `display_map/block_map.rs` — insert full-width block widgets (diagnostics rows, **excerpt headers**,
    deleted-diff hunks) between lines → `BlockPoint(Point)`, `BlockRow(u32)`. (200 KB — the largest layer.)
  - `display_map/crease_map.rs` — user/LSP-defined foldable ranges (creases) that supersede indent-based folds.
  - `display_map/custom_highlights.rs`, `invisibles.rs`, `dimensions.rs` — highlight overlays, whitespace
    rendering, shared dimension helpers.

- **`movement.rs` — motions over display coordinates.** Pure functions `(&DisplaySnapshot, DisplayPoint) ->
  DisplayPoint`, **in display space** so "down" moves one *visual* row (respecting wrap/folds): `left`/`right`
  (+ `saturating_*`), `up`/`down` (carry a `SelectionGoal` for the desired column over ragged/wrapped lines),
  `line_beginning`/`line_end`/`indented_line_beginning`, `previous_word_start`/`next_word_end`, the
  camelCase-aware `previous_subword_start`/`next_subword_end` (+ `is_subword_start`/`is_subword_end`),
  `start_of_paragraph`/`end_of_paragraph`, `start_of_excerpt`/`end_of_excerpt` (multibuffer-aware), and the
  generic `find_boundary` / `find_preceding_boundary` that walk chars through a **`CharClassifier`** (imported
  from the `language` crate — word/punct/whitespace classes, language-configurable) — the single primitive
  under every word/subword motion.

- **`input.rs` — keystrokes → edits.** `Editor::handle_input(text, window, cx)` is the core: gather all N
  cursors in buffer coords, **loop building one edit per selection** while handling autoclose (`(`→`()`),
  auto-surround (wrap a selection), bracket-pair detection via the language scope, and linked edits, then apply
  all edits in one transaction and recompute selections. `impl EntityInputHandler for Editor` provides the
  platform text-input/IME protocol — `replace_text_in_range` (commit) and `replace_and_mark_text_in_range`
  (marked/composing text); an `ime_transaction` groups IME edits for clean undo. `[gpui Apache-2.0]` for the
  `EntityInputHandler` trait + action dispatch; `[Zed-derived]` for the fan-out and autoclose logic.

- **`element.rs` + `element/` — the gpui paint element.** `pub struct EditorElement` `impl Element for
  EditorElement`: `request_layout` (reserve space) → `prepaint` (take a `DisplaySnapshot`, compute the visible
  `DisplayRow` range from scroll, shape each line via the gpui `WindowTextSystem` into a `LineWithInvisibles`,
  build a `PositionMap` for hit-testing, size the gutter) → `paint` (draw text runs, selections, N cursors via
  `CursorLayout`, gutter, block widgets; register mouse/scroll handlers). `EditorLayout` caches the frame's
  geometry; `PointForPosition` maps a pixel to a `DisplayPoint`. `element/header.rs` renders multibuffer
  excerpt headers; `element/mouse.rs` the mouse hit-testing/drag. The `Element` trait, `LineLayout`, shaping,
  and hit-testing are `[gpui Apache-2.0]`; the editor-specific layout/paint algorithm is `[Zed-derived]`.

- **`scroll.rs` + `scroll/` — the viewport.** `ScrollManager`, `ScrollAnchor` (scroll position as an
  edit-stable anchor + offset), and `scroll/autoscroll.rs` (`enum Autoscroll` — the "keep the newest cursor
  visible" strategies: `Fit`, `Center`, `Newest`, …), `scroll/scroll_amount.rs` (`enum ScrollAmount`),
  `scroll/actions.rs`.

- **`clipboard.rs` — multi-cursor copy/paste.** `ClipboardSelection` records per-cursor clipboard entries so a
  paste distributes one entry per cursor (or the whole clipboard at each).

- **`blink_manager.rs` — cursor blink.** A small `Entity<BlinkManager>` toggling cursor visibility on a timer.

### B. Editing features layered on the pipeline `[Zed-derived]`

- `fold.rs` / `folding_ranges.rs` — code folding actions + LSP/indent fold-range computation.
- `rewrap.rs` — hard rewrap (reflow) of paragraphs/comments to a column.
- `indent_guides.rs` — the vertical indent guide lines.
- `highlight_matching_bracket.rs` — highlight the bracket matching the one at the cursor.
- `bracket_colorization.rs` — rainbow bracket colors (73 KB).
- `jsx_tag_auto_close.rs` — auto-close JSX tags.
- `linked_editing_ranges.rs` — edit rename-linked ranges (e.g. open/close tag) together (`LinkedEdits`).
- `inlays.rs` + `inlays/inlay_hints.rs` — the `Inlay` value + LSP inlay-hint management (209 KB — huge).

### C. LSP / language-intelligence surface `[Zed-derived]` (logic) over `[LSP spec: public]`

Mostly **defer for Marley** (needs the LSP subsystem). `completions.rs` + `code_context_menus.rs` (completion
& code-action popups) · `code_actions.rs` · `code_lens.rs` · `diagnostics.rs` (inline + gutter diagnostics) ·
`hover_popover.rs` + `hover_links.rs` (hover docs, ⌘-click go-to) · `signature_help.rs` · `semantic_tokens.rs`
(101 KB — LSP semantic highlighting) · `document_colors.rs` / `document_links.rs` / `document_symbols.rs` ·
`edit_prediction.rs` (98 KB — AI/Copilot inline "ghost text" prediction; **brain-adjacent but GPL here — keep
Marley's equivalent out of the brain**) · `lsp_ext.rs` / `clangd_ext.rs` / `rust_analyzer_ext.rs`
(server-specific extensions).

### D. Git `[Zed-derived]`

- `git.rs` (112 KB) + `git/` — diff hunks in the gutter, inline blame, the split diff-review overlay,
  `DiffHunkDelegate`/`ResolvedDiffHunk`, `BlameRenderer`.

### E. Workspace / item integration `[Zed-derived]` + `[gpui Apache-2.0]`

- `items.rs` (114 KB) — `impl Item for Editor`: tabs, save/save-as, tab titles (`MAX_TAB_TITLE_LEN`), the
  breadcrumb bar; the glue that makes an `Editor` a pane item.
- `persistence.rs` — serialize an editor's state (path, scroll, selections) to SQLite for session restore.
- `navigation.rs` (90 KB) — go-to-line, nav history (back/forward jump list).
- `bookmarks.rs` — per-line bookmarks.
- `split.rs` (200 KB) + `split_editor_view.rs` — the split diff view (`SplittableEditor`, `SplitEditorView`).
- `mouse_context_menu.rs` — the right-click context menu.

### F. Runnables / tasks + config

- `runnables.rs` / `tasks.rs` / `markdown_actions.rs` — gutter "run" indicators for tasks/tests.
- `editor_settings.rs` (`EditorSettings`, `CurrentLineHighlight`, `ScrollbarAxes`, `ShowMinimap`, …) +
  `config.rs` — the settings surface.

### G. Tests

`test.rs` (harness) + `editor_tests.rs` (**1.28 MB**) + `editor_tests/` + `editor_block_comment_tests.rs` +
`code_completion_tests.rs` + `edit_prediction_tests.rs`. Enabled by the `test-support` feature; use `proptest`
for buffer/anchor invariants.

## Depends on (internal)

~41 Zed workspace crates. The load-bearing ones: **`gpui`** (Apache-2.0 — `Element`, `Entity`, `FocusHandle`,
`EntityInputHandler`, `WindowTextSystem`, actions), **`multi_buffer`** (the text model — `MultiBuffer`,
`Anchor`, `Excerpt`, `MultiBufferSnapshot`), **`text`** + **`rope`** (the buffer/anchor primitives, GPL),
**`sum_tree`** (Apache-2.0 — the summarized B-tree under every transform layer), **`language`**
(`CharClassifier`, syntax, `language_settings`), **`lsp`** + **`project`** (LSP + fs/project model),
**`workspace`** (pane/item host), **`theme`**/`theme_settings`/`ui`/`ui_input` (styling & widgets),
**`settings`**, **`snippet`**, **`git`**/`buffer_diff`, **`task`**, **`fuzzy`**, **`markdown`**, `menu`,
`breadcrumbs`, `edit_prediction_types`, `vim_mode_setting`, `zed_actions`, plus external `aho-corasick`
(⌘D), `unicode-segmentation`/`unicode-script`, `regex`, `tree-sitter-*`.

## Used by (internal dependents)

**51 crates** — one of the most depended-on crates in Zed. Notable: `zed` (the app), `vim` (modal editing over
the editor), `search`, `project_panel`, `outline_panel`, `outline`, `terminal_view`, `agent` + `agent_ui` +
`edit_prediction_ui` (the AI layer), `git_ui`, `diagnostics`, `debugger_ui`, `repl`, `command_palette`,
`file_finder`, `tab_switcher`, `go_to_line`, `markdown_preview`, plus every "selector" (theme/language/
encoding/line-ending/toolchain).

## Related crates

- [`multi_buffer.md`](./multi_buffer.md) — the capstone dependency; the text model even for a single file
  (`singleton: true` = one buffer, one excerpt). The `DisplayMap` sits directly on its snapshot.
- [`gpui.md`](./gpui.md) — the Apache-2.0 render/input framework the element paints and receives input through.
- [`sum_tree.md`](./sum_tree.md) — the Apache-2.0 summarized B-tree under every transform layer.
- `text` / `rope` / `language` — the GPL buffer, anchor, and syntax layer below.
- [`vim.md`](./vim.md) — modal editing implemented as a layer *over* the editor's actions.

## Reimplementation on our stack (Marley)

**This crate is `[Zed-derived]` in full — reimplement from concepts, keep inside Marley's intended-GPL
editor/terminal layer, never let it touch the sellable brain.** The primitives it stands on are already
Apache-2.0 (`gpui`, which Marley depends on) so those are adopted as-designed; the value to re-derive is the
*decomposition*.

**Marley baseline (post-M15).** The pure model is `marley_editor` (`crates/editor/src/`): `buffer.rs`
(ropey-backed `Buffer` with `edit → EditResult` carrying a `BufferDelta`, `undo.rs` history, `BufferVersion`),
`selection.rs` (`Selection { anchor, head: CharOffset }`, `SelectionSet` **capped at one member** — `single()`
is the only public ctor, `from_members` is `pub(crate)`, the sort/merge ctor deferred), `movement.rs` (pure
single-cursor motions over *buffer* offsets: `move_char/word_left/right`, `move_up/down`, `move_line_home/end`,
`extend_or_move`/`extend_or_go`), `types.rs` (`EditOrigin { Human, Agent }`, `BufferVersion`, `BufferDelta`).
The **render half** is in `marley_app`: `code_view.rs` / `editor_surface.rs` / `input.rs` (the gpui element that
maps offset↔column and draws one caret). No composer type, no `EditorMode` enum, no `DisplayMap`, no anchors,
no multi-cursor.

The mapping, in dependency order (full sequencing in subsystem 03 §7):

1. **`Editor` composer + `EditorMode`.** Introduce a gpui-aware `Editor` shell (in `marley_app` or a new
   `marley_editor_view` crate) owning `buffer: Buffer` (unchanged, stays pure), `selections`, a `DisplayMap`, a
   scroll state, and `mode: EditorMode { SingleLine, AutoHeight, Full }`. Fold today's duplicate prompt/file
   input onto one parameterized element. *Prereq for everything; pure refactor value.* `[Zed-derived]` shape,
   `[gpui Apache-2.0]` `Entity`/`FocusHandle`/`Context`.

2. **Anchors (the hard prerequisite).** Marley has no free anchors on ropey. Build a lightweight anchor by
   **replaying the `BufferDelta` Marley already emits on every `EditResult`** — a `(BufferVersion, CharOffset)`
   re-mapped across edits — so N cursors survive an edit without manual fix-up. `[Marley-original]` mechanism,
   `[Zed-derived]` intent.

3. **`SelectionSet` → the real `SelectionsCollection`.** Flip `from_members` public; keep members **sorted +
   non-overlapping** (merge on insert); store `Selection { anchor: Anchor, head: Anchor, goal }`; add `pending`
   for drag; resolve via `set.resolve(&buffer) -> Vec<Selection<CharOffset>>` (the analogue of Zed's `all::<D>`).
   Gestures cheapest-first: **columnar** `add_cursor_above/below` (pure geometry over M15's offset↔column map) →
   **⌘D** via the `aho-corasick` crate (MIT, a near-verbatim port of Zed's automaton approach, no Zed code) →
   node-grow (defer to tree-sitter).

4. **`DisplayMap` as a Marley layer stack.** Reproduce the **uniform layer contract** (Transform +
   input/output summary + Snapshot + `sync`) as a Marley `trait DisplayLayer` over ropey; back each layer with
   a small Marley summary-tree or (early, few transforms) a sorted `Vec` + binary search — *the contract is
   what matters, not the container.* Add layers in terminal-first value order: **TabMap** (correct columns,
   trivial) → **WrapMap** (highest-value for a narrow pane; consults the gpui `WindowTextSystem` widths Marley
   already calls) → **highlight overlay** (promote the hand lexer + search/selection backgrounds) → **BlockMap**
   (gate for diagnostics rows & multibuffer excerpt headers) → FoldMap → InlayMap (needs LSP). Each layer is
   independently shippable. The specific stack is `[Zed-derived]`; the summarized-tree + dual-dimension pattern
   is `[public: rope/CS literature]`.

5. **`movement` → display space.** Once the stack lands, re-base the pure motions onto
   `(&DisplaySnapshot, DisplayPoint)` (mechanical lift). Introduce a Marley **`CharClassifier`** as the single
   word/subword primitive and route word motions through `find_boundary`; upgrade grapheme/subword stepping via
   the **`unicode-segmentation` crate** (MIT — the same crate Zed uses; the algorithm is public UAX#29/#14, not
   Zed code). Add `SelectionSet::move_with(|sel| …)` so motion maps over all cursors for free. Note Zed's
   `CharClassifier` lives in the GPL `language` crate — reimplement the class table; do not copy.

6. **`handle_input` fan-out + IME.** On the composer: `for sel in selections { edits.push(edit_for(sel, text)) }`
   applied as one grouped undo transaction (extend `undo.rs` with a group boundary). Autoclose/auto-surround as
   a small bracket-pair table (independent, pure). IME through gpui's `EntityInputHandler`
   (`replace_text_in_range` / `replace_and_mark_text_in_range`) adopted verbatim — an available Apache-2.0
   primitive, not a port — backed by `Buffer::edit`. Keep `EditOrigin::{Human, Agent}` on every path (already
   true — the exact analogue of Zed's edit source; the seam the brain writes through).

7. **`EditorElement`.** Marley's M15 element already does the plain version over gpui; it grows a
   `DisplaySnapshot` input (4) and multi-cursor painting (3), not a new architecture. `[gpui Apache-2.0]` for
   `Element`/`LineLayout`/shaping/hit-testing; `[Zed-derived]` for the row→y layout and multi-cursor paint.

**Defer wholesale:** the LSP surface (group C), git (D), split-diff (E), inlay hints, edit-prediction —
they trail the LSP/tree-sitter subsystems and the multibuffer (subsystem 03 §6). **Never port into the brain:**
`edit_prediction.rs` is AI-adjacent but GPL here; Marley's prediction/agent logic must be an independent
`[Marley-original]` module that talks to the editor across the clean `Buffer::edit(EditOrigin::Agent)` /
`text_in_range` interface — it must never link the editor's GPL internals.

## Provenance map

| Piece | Provenance |
|---|---|
| `Editor` composer decomposition, `EditorMode`, "produces an element not is one" | `[Zed-derived]` |
| `FocusHandle`, `Entity`/`Context` observation, action dispatch, `EntityInputHandler`/IME, `WindowTextSystem`/`LineLayout` | `[gpui Apache-2.0]` — adopt as-designed |
| Anchor-array `SelectionsCollection` + disjoint invariant + 3 gesture state machines | `[Zed-derived]` |
| ⌘D automaton | `[public: aho-corasick MIT]`; whole-word matching is standard |
| The specific 6-layer `DisplayMap` stack + order | `[Zed-derived]` |
| Persistent summarized-tree + dual input/output dimension coordinate mapping | `[public: rope/piece-table CS, Zed-published]` |
| Display-space motions, `CharClassifier`/`find_boundary` factoring, `move_with` fan-out | `[Zed-derived]` |
| Grapheme / word / subword segmentation *algorithms* | `[public: Unicode UAX#29/#14]` via `unicode-segmentation` (MIT) |
| Per-selection input fan-out, autoclose/surround/linked-edit, IME-transaction undo grouping | `[Zed-derived]` |
| `EditorElement` layout/paint algorithm | `[Zed-derived]` over `[gpui Apache-2.0]` primitives |
| LSP/git/edit-prediction/split feature logic | `[Zed-derived]` (over the public LSP spec) |
| Marley `Buffer`/`SelectionSet`/`movement`/`undo`/`EditOrigin`, the anchor delta-replay mechanism | `[Marley-original]` (modeled on the design, independently written; rope is MIT `ropey`) |

## Notes / gotchas

- **`edition = "2024"`** (`edition.workspace = true`) and a very heavy dependency graph (~41 internal crates) —
  the editor pulls in essentially the whole app. It cannot be built or studied in isolation from `gpui`, `text`,
  `language`, and `multi_buffer`.
- **The god-object is deliberate but not aspirational.** `Editor`'s ~230 fields are Zed's accumulated feature
  history; Marley should re-derive the *core* decomposition (buffer + display + selections + scroll + mode +
  the gesture state machines) and add the periphery incrementally, not clone the field list.
- **Everything is an `Entity` handle.** The buffer, display_map, wrap_map (inside display_map), and
  blink_manager are all independently-updatable observable cells — this is what makes async soft-wrap recompute
  and buffer-sharing (splits, multibuffer) work. Marley already has gpui `Entity`; use it the same way.
- **`DisplayPoint != buffer point` the moment any layer exists.** Marley's current trivial identity holds only
  because there is no display stack. Motions, selections, and the element must all switch to display
  coordinates in lockstep when WrapMap lands (subsystem 03 §3–§4).
- **The `*_tests.rs` siblings are enormous and load-bearing** (`editor_tests.rs` alone is 1.28 MB). They pin
  buffer/anchor/undo/display invariants — the same role Marley's per-module `#[cfg(test)]` + `proptest` blocks
  play. When re-deriving a layer, port the *invariants they assert*, not the code.
- **Legal boundary (the hard rule):** this crate is the most GPL-sensitive in the project. Keep all
  `[Zed-derived]` material inside Marley's intended-GPL editor/terminal layer; the brain talks to it only across
  the clean `Buffer::edit`/`text_in_range` interface with `EditOrigin::Agent`. Expert + legal review before
  release, per [`../README.md`](../README.md).
