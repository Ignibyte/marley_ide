# rope

> Per-crate reference (Marley **Zed** round 2 — the granular counterpart to
> [subsystem 02 §2.2](../subsystems/02-text-buffer-anchors.md)). Crate dir: `crates/rope`. Zed is the EDITOR
> reference for Marley's editing surface; Zed source is cloned only into session scratch, never committed. This
> doc is Marley's own description of Zed's design.

| | |
|---|---|
| Subsystem | [02 — Text / Buffer / Rope / Anchors](../subsystems/02-text-buffer-anchors.md) |
| License | **GPL-3.0-or-later** (own `LICENSE-GPL`) — `[Zed-derived]`; may live only in Marley's editor/terminal layer, **never the brain** |
| Internal deps | 2 — [`sum_tree`](./sum_tree.md) `[Apache-2.0]`, `util` `[Zed]` (+ external `heapless`, `rayon`, `unicode-segmentation`, `ztracing`) |
| Used by | 13 Zed crates (incl. `text`, `editor`, `multi_buffer`, `git`, `buffer_diff`, `fs`, `outline`, `agent_ui`, `zed`) |

## Purpose

`rope` is Zed's **text store**: a `Rope` is a **`SumTree<Chunk>`** — a chunked, copy-on-write B+ tree of small
UTF-8 leaves whose summary carries **every coordinate system Zed's editor needs at once**. It answers, in
`O(log n)`, "what byte / char / UTF-16 unit / (row, column) is at position X" and converts freely between all of
them, plus grapheme-safe **clipping** of arbitrary positions to valid boundaries. It is the layer directly under
the CRDT `text::Buffer` and everything above it (multi-buffer, editor, diff, LSP position mapping). Concept-wise
it is a **chunked rope**; the *specific rendition* — SIMD-style per-chunk bitmaps and the exact `TextSummary` —
is Zed's, and the crate is **GPL**.

Where [`sum_tree`](./sum_tree.md) is the generic Apache engine, `rope` is the GPL text *instantiation* of it: it
picks `Chunk` as the `Item` and `TextSummary` as the `Summary`, and exposes a text-shaped API over the resulting
tree.

## Key types, modules & public API

`crates/rope/src/rope.rs` (+ `chunk.rs`, `point.rs`, `point_utf16.rs`, `offset_utf16.rs`, `unclipped.rs`).

- **`struct Rope { chunks: SumTree<Chunk> }`** — the store.
  - Build / edit: `new`, `append(Rope)`, `push(&str)`, `push_front`, **`replace(Range<usize>, &str)`**,
    **`slice(Range<usize>) -> Rope`** (COW sub-rope, `O(log n)`), `slice_rows(Range<u32>)`.
  - Metrics: `len()` (bytes), `is_empty`, **`summary() -> TextSummary`**, `max_point()`, `max_point_utf16()`,
    `line_len(row) -> u32`.
  - Iterate: `chars`/`chars_at`/`reversed_chars_at`, `bytes_in_range`, `chunks`/`chunks_in_range`/
    `reversed_chunks_in_range`, `cursor(offset) -> Cursor`.
  - Boundaries: `is_char_boundary`, `floor_char_boundary`, `ceil_char_boundary`, `starts_with`/`ends_with`.
  - **Coordinate conversions — the LSP/render bridge (the crate's real value):** `offset_to_point`,
    `point_to_offset`, `offset_to_offset_utf16`, `offset_utf16_to_offset`, `offset_to_point_utf16`,
    `point_to_point_utf16`, `point_utf16_to_point`, `point_utf16_to_offset`, `point_to_offset_utf16`,
    `point_utf16_to_offset_utf16`, `unclipped_point_utf16_to_offset`, `unclipped_point_utf16_to_point`.
  - **Clipping (snap an arbitrary position to a legal boundary, with a `Bias`):** `clip_offset`, `clip_point`,
    `clip_offset_utf16`, `clip_point_utf16`.
- **`struct TextSummary`** — the `sum_tree` summary, carrying **all** coordinates simultaneously: `len` (bytes),
  `chars`, `len_utf16: OffsetUtf16`, `lines: Point` (row + last-line byte column), `first_line_chars`,
  `last_line_chars`, `last_line_len_utf16`, and **`longest_row` / `longest_row_chars`** (which power "widest line"
  for the horizontal scrollbar — a coordinate a fixed rope can't cheaply expose). `From<&str>` computes it;
  `AddAssign` folds two, correctly stitching the boundary line (`last_line_chars + other.first_line_chars`). Impls
  `ContextLessSummary` (so its `sum_tree::Summary::Context = ()`).
- **`struct Chunk`** (`chunk.rs`, `[Zed-derived]` — the clever, GPL-specific part) — a fixed-capacity leaf,
  `text: ArrayString<MAX_BASE>` with `MAX_BASE = 128` (= `u128::BITS`; `16` in test). Alongside the bytes it
  keeps four **bitmaps** (`u128`) — `chars` (UTF-8 boundary bits), `chars_utf16` (UTF-16 code-unit bits),
  `newlines`, `tabs` — so per-chunk positional queries are **bit-ops / popcount, not byte scans**. `ChunkSlice`
  is a borrowed view; `Chunk::new/push_str/append/prepend/slice` maintain the bitmaps.
- **`trait TextDimension`** — anything derivable from a `TextSummary` / `ChunkSlice`: implemented by `usize` (byte
  offset), `OffsetUtf16`, `Point`, `PointUtf16`, `TextSummary` itself, and `Dimensions<D1,D2>` pairs. Makes
  `Cursor::summary::<D>(end)` and the conversions generic over "which coordinate do you want out."
- **Coordinate newtypes** — all impl `sum_tree::Dimension<ChunkSummary>`:
  - **`Point { row: u32, column: u32 }`** — (row, **byte** column).
  - **`PointUtf16 { row, column }`** — (row, **UTF-16** column) — the LSP position shape.
  - **`OffsetUtf16(usize)`** — a flat UTF-16 code-unit offset.
  - **`Unclipped<T>`** — a coordinate that may land mid-grapheme / out of bounds (e.g. a raw LSP position) and
    must be run through a `clip_*` before use.
- **`struct Cursor<'a>`** (rope's own, distinct from `sum_tree::Cursor`) — `new(rope, offset)`, `seek_forward`,
  **`slice(end) -> Rope`**, **`summary::<D: TextDimension>(end) -> D`**, `suffix`, `offset`.
- **Iterators** — `Chunks<'a>` (`peek`, `next_line`/`prev_line`, `seek`, `lines()`, `equals_str`, bitmap-
  accelerated `peek_with_bitmaps`), `Bytes<'a>`, `Lines<'a>`.

## Depends on (internal)

- [sum_tree](./sum_tree.md) `[Apache-2.0]` — `Rope` **is** a `SumTree<Chunk>`; `TextSummary`/`Point`/… are its
  `Summary`/`Dimension`s.
- `util` `[Zed]` — small shared helpers (`debug_panic`, `is_utf8_char_boundary`).
- External: `heapless` (the fixed-cap `ArrayString`/`ArrayVec` leaves), `rayon`, `unicode-segmentation`
  (`GraphemeCursor`, for grapheme-correct clipping), `ztracing`/`tracing`. Dev: `gpui`, `criterion` (benchmark).

## Used by (internal dependents)

13 Zed crates — most directly **`text`** (the CRDT `Buffer` stores a `Rope` + an anchor tree), then **`editor`**,
**`multi_buffer`**, **`buffer_diff`**, **`streaming_diff`**, **`git`**, **`fs`**, **`outline`**, `go_to_line`,
`languages`, `agent_ui`, `picker_preview`, `zed`.

## Related crates

- [sum_tree](./sum_tree.md) — the Apache engine; read it first. `rope` is what makes the abstract "open summary"
  concrete for text.
- [subsystem 02](../subsystems/02-text-buffer-anchors.md) — situates `rope` under `clock` + `text` (the CRDT
  buffer, anchors, undo) and against Marley's ropey baseline.

## Reimplementation on our stack (Marley)

**Current baseline.** `marley_editor::Buffer` wraps **`ropey::Rope` (MIT)** directly (`type Rope = ropey::Rope`),
**char-indexed** via `CharOffset`, using ropey's `char_to_line` / `line_to_char` / `char_to_byte`. ropey is
itself a chunked-B-tree rope with `O(log n)` byte/char/line conversions — it already delivers Zed-`rope`'s
*properties* (chunked, balanced, snapshot-cheap) for the coordinate set it ships. Crucially, **ropey 1.6.1
already provides `char_to_utf16_cu` / `utf16_cu_to_char` / `len_utf16_cu`**, so the LSP UTF-16 bridge is
**additive** on top of ropey, not a reason to replace it.

**Decision — KEEP ropey; do NOT port Zed's `rope`.** Two independent reasons:

1. **License.** Zed's `rope` is **GPL-3.0** `[Zed-derived]`. Even setting engineering aside, it cannot be copied
   — and it must never touch the proprietary brain. ropey (MIT) has no such constraint.
2. **Redundancy.** ropey covers the common path (bytes/chars/lines/utf16 in `O(log n)`) that `marley_editor`
   already drives. Re-deriving it buys nothing today.

**What Zed's `rope` has that ropey does not** — and how Marley closes each gap *without* the GPL crate (build it
as **`[Marley-original]`** helpers over ropey, or over Apache `sum_tree` if it must be structural):

- **An open `TextSummary`** (`longest_row`, first/last-line metrics, and — the real superpower — *arbitrary custom
  dimensions* aggregated up the tree). ropey answers a **fixed** coordinate set. → For scalar extras (longest
  row for the h-scrollbar) compute incrementally alongside edits. For a genuinely *open, seekable* aggregate
  (fold regions, diagnostics-per-line), that is the trigger to adopt **Apache `sum_tree`** and build a
  Marley-original rope on it — see [sum_tree.md](./sum_tree.md).
- **Native `Point` / `PointUtf16` (row+column) coordinates + `Unclipped` + grapheme-safe `clip_*`.** ropey is
  offset/line-oriented; Marley currently derives row/col itself and is char-indexed. → Add a **Marley-original
  coordinate layer** — `Point`/`PointUtf16` newtypes + `clip`/`unclipped` helpers built on ropey's
  `char_to_line`/`line_to_char` and its `*_utf16_cu` conversions (+ a grapheme crate for clipping). These are
  precisely the primitives [subsystem 02](../subsystems/02-text-buffer-anchors.md) flags as prerequisites for
  LSP, multi-cursor, and marks.

**Bottom line (matches the text-subsystem conclusion):** ropey stays the store; the LSP/point/clip machinery is
**additive Marley-original code over ropey**; Zed's GPL `rope` is **study-only** (understand the `TextSummary`
coordinate set and the clip/bias semantics, reimplement from the public chunked-rope concept). `sum_tree`
(Apache) is the escape hatch **only if** an open, seekable custom summary becomes load-bearing.

## Provenance & licensing

- **`[Zed-derived]`** — the `rope` crate is **GPL-3.0-or-later**. Study the design; **do not copy** the source,
  and keep any rope-derived code out of the brain.
- **`[permissive/public: augmented-B-tree textbook DS]`** — the *concept* (a chunked rope over a summarizing B+
  tree) is standard CS literature. The SIMD-bitmap `Chunk` and the exact `TextSummary` shape are Zed's specific,
  GPL rendition — reimplement, don't lift.
- **`[permissive/public: ropey MIT]`** — Marley's actual store; MIT, unrestricted. Already ships the UTF-16
  conversions the LSP bridge needs.
- Its one internal engine dep, [`sum_tree`](./sum_tree.md), is **`[Apache-2.0]`** — the permissive half. The GPL
  boundary sits at `rope`, not at the tree beneath it.

## Notes / gotchas

- `MAX_BASE` (chunk capacity) is tied to the bitmap width: `u128` → 128 bytes/chunk in release, `u16` → 16 under
  test. Any reimplementation that mimics the bitmap trick inherits this "chunk size == bitmap bits" coupling.
- The bitmaps make per-chunk char/utf16/newline/tab queries `O(1)` via shifts + popcount — that's why Zed's
  `rope` outperforms a naive scan-the-chunk rope. ropey does *not* do this; if Marley ever profiles a hot
  coordinate conversion, this is the technique to study (concept is public).
- `Unclipped<T>` is a type-level reminder that a raw external position (e.g. from an LSP client) is **not yet
  safe** — it must pass through `clip_*` before indexing. Marley's coordinate layer should encode the same
  invariant so a mid-grapheme LSP position can't silently panic or corrupt an edit.
- `TextSummary::AddAssign` carefully stitches the boundary line when concatenating; a hand-rolled equivalent must
  reproduce the `last_line + other.first_line` merge or line/column math drifts across chunk seams.
