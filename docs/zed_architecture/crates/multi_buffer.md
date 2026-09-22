# multi_buffer

> Per-crate reference (Marley **Zed** round 2 — the granular counterpart to
> [subsystem 03 §6](../subsystems/03-editor-multibuffer.md), "the multibuffer — the capstone"). Crate dir:
> `crates/multi_buffer`. Zed is the EDITOR reference for Marley's editing surface; Zed source is cloned only
> into session scratch, never committed. This doc is Marley's own description of Zed's design.

| | |
|---|---|
| Subsystem | [03 — Editor & Multibuffer](../subsystems/03-editor-multibuffer.md) |
| License | **GPL-3.0-or-later** (`LICENSE-GPL` symlink, © Zed Industries) — `[Zed-derived]`, copyleft; **editor-layer only, never the brain** |
| Internal deps | 12 Zed crates: `sum_tree` (Apache), `text`, `rope`, `language`, `buffer_diff`, `clock`, `collections`, `gpui`, `settings`, `theme`, `util`, `ztracing` (+ external `tree-sitter`, `smallvec`, `itertools`, `serde`, `rand`, `parking_lot`, `futures-lite`) |
| Used by | 14 Zed crates — **`editor`** (the consumer), `search`, `vim`, `git_ui`, `agent_ui`, `acp_thread`, `repl`, `go_to_line`, `edit_prediction_ui`, `picker_preview`, `svg_preview`, `collab` (+ benchmarks) |

## Purpose

`multi_buffer` is Zed's **signature capstone**: it stitches **anchored excerpts from N separate `language::Buffer`s
into one editable, summarized text sequence**, where editing an excerpt **writes straight through to the real source
buffer (and thus the file on disk)**. It backs every "many files as one view" surface — multi-file **search
results**, **find-all-references**, project-wide **diagnostics**, and the inline **git / agent-diff review** — each a
single `Editor` over slices of many buffers.

The one-line thesis: **a `SumTree` of `Excerpt`s (each a `Range<text::Anchor>` into a source buffer) presented as a
single rope-shaped, offset-seekable, edit-stable surface — and the *same type*, in `singleton` mode, IS the plain
one-file editor.** That last clause is the masterstroke: `MultiBuffer::singleton(buffer)` is one buffer + one excerpt
spanning the whole file, so `Editor`, `DisplayMap`, `SelectionsCollection`, and `EditorElement` have **exactly one
code path** — a scratch buffer and a 20-file references view render through identical machinery. There is no separate
"single file" editor in Zed; there is only the degenerate multibuffer. In the clone, `Editor::singleton` /
`for_buffer` / `auto_height` and friends wrap their buffer in `MultiBuffer::singleton` at **83 call-sites** in the
`editor` crate alone.

The multibuffer holds **no text of its own.** Its `MultiBufferSnapshot` is a *derived* view; every mutation is routed
back to the owning `Entity<Buffer>` and the snapshot is lazily rebuilt on the next read.

## Key types, modules & public API

`crates/multi_buffer/src/multi_buffer.rs` (~8.3 kLOC — the densest crate in the editor stack) + `anchor.rs`,
`path_key.rs`, `transaction.rs` (+ a 6.3 kLOC `multi_buffer_tests.rs`).

### The two top-level structures

- **`MultiBuffer`** — a `gpui::Entity`, `EventEmitter<Event>`. The mutable owner:
  - `snapshot: RefCell<MultiBufferSnapshot>` — the derived, lazily-synced view.
  - `buffers: BTreeMap<BufferId, BufferState>` — the **real source buffers** (`BufferState { buffer: Entity<Buffer>,
    _subscriptions: [Subscription; 2] }`; it observes + subscribes to each so edits in the underlying buffer
    invalidate the multibuffer).
  - `diffs: HashMap<BufferId, DiffState>` — per-buffer git/base diff (for the review view).
  - `singleton: bool` — the unification flag (§ Purpose).
  - `history: History` — **cross-buffer** undo/redo (see `transaction.rs`).
  - `subscriptions: Topic<MultiBufferOffset>`, `title: Option<String>`, `capability: Capability` (read-only vs
    read-write), `buffer_changed_since_sync`.
- **`MultiBufferSnapshot`** (`Clone`, `Default`) — the immutable per-frame capture the `DisplayMap` reads:
  - **`excerpts: SumTree<Excerpt>`** — the stitched slices, in a summarized tree `[sum_tree Apache-2.0]`.
  - `buffers: TreeMap<BufferId, BufferStateSnapshot>`, `path_keys: Arc<IndexSet<PathKey>>` (interned file
    identities), `diffs: SumTree<DiffStateSnapshot>`, `diff_transforms: SumTree<DiffTransform>`.
  - counters/flags: `edit_count`, `is_dirty`, `has_conflict`, `singleton`, `show_headers`,
    `all_diff_hunks_expanded`, `trailing_excerpt_update_count`, …

### The excerpt model

- **`Excerpt`** (`pub(crate)`) — *a slice of a `Buffer`*: `path_key: PathKey`, `path_key_index: PathKeyIndex`,
  `buffer_id: BufferId`, **`range: ExcerptRange<text::Anchor>`** (which slice), `max_buffer_row`, `text_summary:
  TextSummary`, `has_trailing_newline`. Its `impl sum_tree::Item` summarizes to `ExcerptSummary`.
- **`ExcerptRange<T> { context: Range<T>, primary: Range<T> }`** — `context` = the full window shown; `primary` = the
  sub-range to highlight (in a search excerpt, the matched text). `ExcerptRange::new(ctx)` sets `primary = context`.
  **Carry both** — search UIs highlight `primary`, render `context`.
- **`ExcerptSummary`** — the tree augmentation: `path_key`, `path_key_index`, `max_anchor: Option<text::Anchor>`,
  `widest_line_number`, `text: MBTextSummary`, `count`. Multiple `sum_tree::Dimension`/`SeekTarget` impls over it —
  `PathKey` (seek to a *file*), `MultiBufferOffset`, `ExcerptDimension<MBD>`, and `AnchorSeekTarget` — are what make
  offset→`(buffer, local offset)` and anchor→offset **O(log n)**, exactly like the rope one layer down.
- **`MBTextSummary`** — a multibuffer-flavored `text::TextSummary` (`len`, `chars`, `len_utf16`, `lines: Point`,
  `longest_row`, …) with `From`/`Into` the real `TextSummary`.

### Coordinates & conversions

- **`MultiBufferOffset(usize)`**, `MultiBufferOffsetUtf16`, **`MultiBufferRow(u32)`**, `MultiBufferPoint = Point` —
  positions in the *stitched* space. `BufferOffset(usize)` — a position in a *source* buffer. Internal newtypes
  `ExcerptDimension<T>` / `OutputDimension<T>` separate "input" (buffer) from "output" (post-diff-transform) space.
- **`trait ToOffset` / `trait ToPoint`** — resolve anything to multibuffer offset/point against a snapshot.
- **`trait MultiBufferDimension`** — a dimension foldable from an `MBTextSummary` (implemented for `Point`,
  `PointUtf16`, `MultiBufferOffset`, `MultiBufferOffsetUtf16`).

### The composed anchor (`anchor.rs`) — the edit-stability engine

- **`enum Anchor { Min, Excerpt(ExcerptAnchor), Max }`** — `Min`/`Max` always resolve to the multibuffer's
  start/end; the middle case wraps **`ExcerptAnchor { text_anchor: text::Anchor, path: PathKeyIndex, diff_base_anchor:
  Option<text::Anchor> }`** = *a per-buffer edit-stable anchor + which excerpt/file it lives in* (+ an optional anchor
  into the diff **base** text, so a caret can sit on a deleted-hunk row that isn't in the live buffer).
- **`Anchor::cmp(&self, other, snapshot)`** — composes the ordering: `Min`/`Max` short-circuit; otherwise order by
  `path_key`, then `buffer_id`, then the underlying `text::Anchor::cmp` *within that buffer's snapshot*, then the
  diff-base anchor. This is why a selection spanning two files sorts correctly and survives edits in either.
- `bias_left`/`bias_right`, `is_valid`, `summary::<D>()` (→ offset/point via `summary_for_anchor`),
  `text_anchor_in(buffer)`, `diff_base_anchor()`, `opaque_id()`.
- **`enum AnchorSeekTarget { Missing | Excerpt | Empty }`** — implements `sum_tree::SeekTarget<ExcerptSummary>`, so an
  anchor seeks the excerpt tree directly.
- **`trait AnchorRangeExt for Range<Anchor>`** — `cmp` / `includes` / `overlaps` / `to_offset` / `to_point`.
- **Notable evolution — there is no `ExcerptId` in this Zed revision** (0 occurrences). The classic
  `add_excerpts → ExcerptId` API has been **replaced by a `PathKey` model**: excerpts are addressed by a *sortable
  file identity*, not an opaque handle (see next).

### File identity (`path_key.rs`) & the excerpt-mutation API

- **`PathKey { sort_prefix: Option<u64>, path: Arc<RelPath> }`** — the stable, **sortable** slot identity of a
  buffer in the multibuffer (from `worktree_id + rel path`, or the entity id for buffers with no file). `min()`,
  `sorted(prefix)`, `with_sort_prefix`, `for_buffer`. **`PathKeyIndex(u64)`** interns it into
  `snapshot.path_keys`; anchors carry the cheap *index*, not the `PathKey`.
- **`set_excerpts_for_buffer` / `set_excerpts_for_path(path, buffer, ranges: Point-ranges, context_line_count, cx)
  -> bool`** — the primary entry point (search / diagnostics / references call this). `build_excerpt_ranges` pads each
  match by `context_line_count` lines (`excerpt_context_lines`, default 2); `merge_excerpt_ranges` coalesces
  overlaps. Replaces all excerpts for that buffer/path.
- **`update_excerpts_for_path`** (grow to cover existing overlaps), **`set_anchored_excerpts_for_path`** (async —
  background-computes ranges for large result sets, then applies), **`expand_excerpts(anchors, line_count,
  Up|Down|UpAndDown)`** (grow context around anchors — the "expand" affordance), **`remove_excerpts(path)`** /
  **`remove_excerpts_for_buffer`** / **`clear`**.
- Under them: **`set_merged_excerpt_ranges_for_path` → `update_path_excerpts`** — the engine. It anchors each range
  (`anchor_before` / `anchor_after`), walks the old excerpt list against the new with a `SumTree` cursor keyed on
  `Dimensions<PathKey, ExcerptOffset>`, and emits a **`Patch<MultiBufferOffset>`** of insert/remove edits so the
  `DisplayMap` re-syncs only the touched rows (plus fiddly `has_trailing_newline` boundary bookkeeping between
  adjacent excerpts).

### Editing — routing edits back to source buffers

- **`edit<I, S: ToOffset, T: Into<Arc<str>>>(edits, autoindent_mode, cx)`** / `edit_non_coalesce` → `edit_internal` →
  **`convert_edits_to_buffer_edits(edits, snapshot, indents) -> HashMap<BufferId, Vec<BufferEdit>>`**: seeks the
  excerpt tree (`cursor::<MultiBufferOffset, BufferOffset>`), maps each multibuffer-offset range to the owning
  excerpt's *source-buffer* offset range (skipping excerpt boundaries and non-main-buffer regions like deleted-diff
  rows), then **for each `buffer_id` calls `buffer.update(cx, |buffer, cx| buffer.edit(...))`** — the edit lands in
  the real `language::Buffer`. Adjacent edits coalesce; autoindent columns thread through.

### Cross-buffer undo/redo (`transaction.rs`)

- `History` records `Transaction { id, buffer_transactions: HashMap<BufferId, text::TransactionId>, first/last_edit_at
  }`. **`start_transaction` / `end_transaction[_with_source]` / `undo` / `redo` / `undo_transaction` /
  `forget_transaction` / `merge_transactions` / `group_until_transaction` / `finalize_last_transaction`** replay each
  source buffer's own transaction; grouping honors `group_interval` (300 ms default). **The `singleton` fast path
  short-circuits every transaction/undo op straight to the sole underlying `Buffer`'s history** (`as_singleton()` →
  `buffer.update`).

### Diff / review layer (`buffer_diff` dep)

- `add_diff(Entity<BufferDiff>)`, `expand_diff_hunks`, `set_all_diff_hunks_expanded`,
  `expand_or_collapse_diff_hunks`. **`diff_transforms: SumTree<DiffTransform>`** (`BufferContent` vs `DeletedHunk`)
  synthesizes deleted-hunk rows into the *output* text so an inline diff shows removed lines absent from the live
  buffer. `MultiBufferDiffHunk`, `RowInfo.diff_status`. Powers the git panel and the agent-diff review pane.

### Read / render surface (what the `DisplayMap` + `EditorElement` consume)

- `snapshot(cx)` / `read(cx)` (sync + borrow), `len`, `is_empty`, `is_dirty`, `text()`, `text_for_range`, **`chunks(range,
  language_aware)`** → `MultiBufferChunks` (syntax-highlight runs), **`row_infos(start_row)`** → `MultiBufferRows`
  (per-row `RowInfo { buffer_id, buffer_row, diff_status, expand_info }`), **`excerpt_boundaries_in_range`** →
  `ExcerptBoundary` / `ExcerptBoundaryInfo` (`starts_new_buffer()` — the data an excerpt-**header** block draws from),
  `text_summary`, `max_point`, `buffer_line_for_row`, `excerpt_containing`, `excerpts_for_buffer`,
  `buffers_with_paths`.
- Anchors: `anchor_before` / `anchor_after` / `anchor_at(pos, bias)`, `anchor_in_excerpt(text_anchor)`,
  `buffer_anchor_range_to_anchor_range`, `text_anchor_for_position`.
- Lifecycle: `as_singleton()` / `is_singleton()`, `title` / `set_title` / `with_title`, `capability` / `read_only`,
  `subscribe()` → `Subscription<MultiBufferOffset>`, and `EventEmitter<Event>` — **`Event`** = `BuffersEdited`,
  `Edited`, `BufferRangesUpdated`, `BuffersRemoved`, `DiffHunksToggled`, `Reparsed`, `LanguageChanged`,
  `DiagnosticsUpdated`, `Reloaded`, `Saved`, `DirtyChanged`, `BufferDiffChanged`.

## Depends on (internal)

- **[sum_tree](./sum_tree.md)** (**Apache-2.0**) — `SumTree<Excerpt>`, `TreeMap`, `Dimension`/`SeekTarget`. The
  permissive container the whole crate stands on.
- **[text](./text.md)** (GPL) — `text::Anchor` (the per-buffer edit-stable position), `TextSummary`, `BufferId`,
  `Patch`, the edit subscription `Topic`.
- **[rope](./rope.md)** (GPL) — `Point` / `PointUtf16` / `OffsetUtf16` coordinate types.
- **`language`** (GPL) — `Buffer` (the source), `BufferSnapshot`, diagnostics, outline, `CharClassifier`,
  tree-sitter styling.
- **`buffer_diff`** (GPL) — `BufferDiff` / diff hunks for the review layer.
- **`gpui`** (**Apache-2.0**) — `Entity` / `Context` / `App`, subscriptions, `EventEmitter`.
- `clock`, `collections` (`BTreeMap`/`IndexSet`/`HashMap`), `settings`, `theme`, `util` (`RelPath`), `ztracing`.

## Used by (internal dependents)

14 crates — **`editor`** is *the* consumer (the buffer model behind every `Editor`, singleton or not). Then
`search` (results view), `vim`, `git_ui` (diff view), `agent_ui` + `acp_thread` (agent diff/review), `repl`,
`go_to_line`, `edit_prediction_ui`, `picker_preview`, `svg_preview`, `collab`, plus benchmarks.

## Related crates

- [sum_tree](./sum_tree.md) — the excerpt tree, its summaries, and `TreeMap` are all `sum_tree`; **only the
  multibuffer *design* is GPL, the container is Apache** (see Provenance).
- [text](./text.md) — the anchor model the composed `Anchor` is built from; read it for how `text::Anchor` survives
  edits (the property the multibuffer inherits and composes across files).
- [subsystem 03 §6](../subsystems/03-editor-multibuffer.md) — the narrative placement (the capstone that everything
  else in the editor subsystem builds toward) and the full editor pipeline this crate is the base of.

## Reimplementation on our stack (Marley)

**Current baseline (post-M15).** Marley has **no multibuffer.** `marley_editor::Buffer` (`crates/editor`) is one
`ropey::Rope` = one file. An editor tab shows **N *separate* files as a strip** (#237) — no unified, editable,
cross-file surface, no excerpt concept, no write-through. This is the single biggest gap between Marley's editor and a
Zed-class one, and — per [subsystem 03 §7](../subsystems/03-editor-multibuffer.md#7-roadmap--consolidated-sequencing)
— it is deliberately **scheduled LAST.**

**Decision — the capstone; gated, `singleton`-first.** Introduce **`marley_editor::MultiBuffer`** as a summarized
sequence of `Excerpt { buffer_id, range: Range<Anchor>, summary }` over Marley's source `Buffer`s. It **hard-depends**
on earlier stages and must not be attempted before them, or it gets rebuilt:

1. **A Marley `Anchor`** (subsystem 03 §2.1) — edit-stable positions. On `ropey` there are no free anchors; build one
   by **delta-replay over the `BufferDelta` Marley already emits on every edit**. Compose it the same way Zed does:
   `ExcerptAnchor { path_index, text_anchor }` so a cross-file selection stays valid and edits route to the right
   source. `[Marley-original]` mechanism, `[Zed-derived]` intent.
2. **A DisplayMap-style layer *including BlockMap*** (subsystem 03 §3) — **excerpt headers ("── src/foo.rs ──") *are*
   block widgets**, so BlockMap is the gate. `ExcerptBoundary::starts_new_buffer()` is the header data.
3. **Project search** — the primary *producer* of excerpts (`set_excerpts_for_path(path, buffer, match-ranges,
   context_lines)`); without it there is nothing to stitch. Reuse Marley's ripgrep surface.
4. The **§1 `Editor` composer** and **§5 input path** it sits under.

**Concrete mapping.**
- **Adopt the `PathKey` identity, not an opaque `ExcerptId`.** A sortable file key matches Marley's workspace/file
  model — the #237 strip already keys tabs by file — and gives free ordering + de-dup. Intern it (`PathKeyIndex`) so
  anchors stay cheap.
- **Back the excerpt sequence with a small Marley summary-tree or a sorted `Vec<Excerpt>` + binary search** — the
  *contract* (offset→`(buffer, local)` seek, an `input/output` summary) is what matters, not the container. (If/when
  the open-summary superpower is needed, `sum_tree` is Apache and adoptable directly — see
  [sum_tree.md](./sum_tree.md).)
- **Render as one `uniform_list`** with excerpt-header block rows between files (BlockMap), so the plain file editor
  and the multi-file view are one element.
- **Route edits back per source** via `Buffer::edit(range, text, EditOrigin)` — carrying Marley's existing
  `EditOrigin { Human, Agent }` so an agent-driven cross-file edit is provenance-tagged. This is the direct analogue
  of Zed's `convert_edits_to_buffer_edits` fan-out.
- **Ship the `singleton` unification first.** A normal file editor = a **one-excerpt multibuffer**, so every earlier
  stage (composer, selections, display, input, element) serves it unchanged and the multi-file case is purely
  additive. This de-risks the capstone: the unification lands before any multi-file UI exists.

**Payoff — this is what the brain actually wants.** The multibuffer is the substrate for **"review N files of proposed
agent edits in one editable pane,"** project-wide search results, and diagnostics — the exact
"custom panel that reads/writes across a whole workspace" Marley's agent layer is aimed at. But: it is a **GPL
`[Zed-derived]` design** — **clean-room from this analysis, keep it inside the intended-GPL editor layer, and never
let it (or `text::Anchor`-shaped code) leak into the sellable brain** (subsystem 03 §9).

## Provenance & licensing

- **`[Zed-derived]` (GPL-3.0-or-later)** — the whole crate's *design* is Zed's: the `MultiBuffer` / `Excerpt` model,
  the **composed `Anchor` enum** (`Min | Excerpt(ExcerptAnchor) | Max`), the `PathKey`/`PathKeyIndex` file-identity
  scheme, the edit-fan-out to source buffers, cross-buffer undo, the diff-transform (deleted-hunk) expansion, and the
  **`singleton` unification**. Re-derive it; do not copy the source. Editor/terminal layer only — copyleft-sensitive.
- **`[sum_tree Apache-2.0]`** — the `SumTree<Excerpt>` + `ExcerptSummary` + `Dimension`/`SeekTarget` + `TreeMap`
  machinery underneath is the *permissively-licensed* `sum_tree` crate. Marley may adopt that **container** directly;
  only the multibuffer *design* built on it is GPL. (See [sum_tree.md](./sum_tree.md).)
- **`[text GPL] / [public]`** — `text::Anchor`, the per-buffer edit-stable position the composed anchor wraps, lives
  in the GPL `text` crate, but the **anchor *concept* is public**; Marley builds its own (delta-replay over
  `BufferDelta`, subsystem 03 §2.1) rather than porting it.
- **`[public: augmented-B-tree + coordinate-mapping pattern]`** — "a summarized sequence you seek by any aggregate in
  O(log n)" is standard CS, the same pattern as the `DisplayMap` layers and the rope. The multibuffer is a *specific
  application* of it; the pattern itself is unencumbered.
- **`[Marley-original]`** — Marley's forthcoming `MultiBuffer`, its `Excerpt`/`Anchor` reimplementation, and the
  `singleton`-first strategy on `ropey`.

## Notes / gotchas

- **The multibuffer stores no text.** Its snapshot is *derived*; all mutation flows to source `Entity<Buffer>`s and
  `sync`/`sync_mut` (gated by `buffer_changed_since_sync`) lazily rebuilds the snapshot on read. A Marley port must
  keep this invariant — the multibuffer is a *view + router*, never a second copy of the text.
- **`singleton` is a pervasive fast path**, not a config flag bolted on: transactions, undo, `set_group_interval`,
  and title all delegate straight to the sole buffer when `singleton`. Decide singleton semantics **up front**.
- **No `ExcerptId` in this revision.** External write-ups (and older Zed) describe an `add_excerpts → ExcerptId` API;
  this clone uses `set_excerpts_for_path` keyed by `PathKey`. Prefer the `PathKey` model — it sorts files for free.
- **`ExcerptRange.primary` vs `context`** — the *match* vs the *shown window*. Model both from day one; retrofitting
  the highlight range later is painful.
- **`has_trailing_newline` boundary bookkeeping** between adjacent excerpts (the `update_path_excerpts` patch logic)
  is the fiddliest code in the crate and is heavily invariant-asserted — preserve those assertions in any port.
- **Diff-transform (deleted-hunk row synthesis) is a large secondary subsystem.** Ship the plain multibuffer first;
  add diff expansion later — it is an *additive* `SumTree<DiffTransform>` layer over the excerpts, not a rewrite.
- **~8.3 kLOC main + ~6.3 kLOC tests** — the most invariant-dense crate in the editor stack, pinned by randomized
  property tests (`randomly_edit_excerpts`, `build_random`). Any reimplementation wants the same fuzz harness.
