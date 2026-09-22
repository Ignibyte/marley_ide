# sum_tree

> Per-crate reference (Marley **Zed** round 2 — the granular counterpart to
> [subsystem 02 §2.1](../subsystems/02-text-buffer-anchors.md)). Crate dir: `crates/sum_tree`. Zed is the
> EDITOR reference for Marley's editing surface; Zed source is cloned only into session scratch, never
> committed. This doc is Marley's own description of Zed's design.

| | |
|---|---|
| Subsystem | [02 — Text / Buffer / Rope / Anchors](../subsystems/02-text-buffer-anchors.md) |
| License | **Apache-2.0** (own `LICENSE-APACHE`, © Zed Industries) — `[sum_tree Apache-2.0]`, permissively licensed, adoptable into *any* Marley layer |
| Internal deps | 0 (external only: `heapless`, `rayon`, `log`, `ztracing`/`tracing`, `proptest` opt.) |
| Used by | 13 Zed crates (incl. `rope`, `text`, `editor`, `language`, `multi_buffer`, `project`, `git`, `gpui`, `worktree`) |

## Purpose

`sum_tree` is Zed's foundational, dependency-free data-structure crate — a **copy-on-write, concurrency-friendly
B+ tree** (`Cargo.toml` describes it verbatim as *"a sum tree data structure, a concurrency-friendly B-tree"*).
Every leaf item contributes a **`Summary`** that aggregates monotonically up the tree, so any number of derived
coordinates ("**dimensions**" — byte count, char count, line count, UTF-16 units, …) can be **seeked in
`O(log n)`** and the tree edited **persistently via structural sharing** (`Arc<Node<T>>`). It is the engine under
Zed's `rope`, the CRDT `text::Buffer`, the multi-buffer, worktree entries, and git state — anywhere Zed needs a
balanced, summarizable, cheaply-snapshotted sequence. It is the permissively-licensed *half* of the text stack:
the tree is Apache-2.0; the `rope`/`text`/`clock` built on top are GPL.

The one-line thesis: **an order-statistic / augmented B-tree whose augmentation set is open.** You define what to
aggregate; the tree gives you `O(log n)` seek on each aggregate for free, forever, as items are inserted/removed.

## Key types, modules & public API

`crates/sum_tree/src/sum_tree.rs` (+ `cursor.rs`, `tree_map.rs`, `property_test.rs`).

- **`SumTree<T: Item>(Arc<Node<T>>)`** — the persistent B+ tree. Max items/node = `TREE_BASE * 2`; `TREE_BASE = 6`
  in release, `2` under `#[cfg(test)]` to force deep trees. `Clone` is `O(1)` (Arc bump); snapshots are
  structurally shared.
  - Build: `new(cx)`, `from_summary`, `from_item`, `from_iter`, **`from_par_iter`** / **`par_extend`** (rayon
    parallel bulk-build), `extend`, `push`, `append`.
  - Query: `summary()`, `extent::<D>()`, `first`/`last`/`last_summary`, `is_empty`, `items(cx)`, `iter()`,
    `find`/`find_exact`/`find_with_prev`.
  - Update: `update_first`, `update_last`.
  - Keyed (ordered-map) ops on `KeyedItem`: `insert_or_replace`, `remove`, **`edit(&mut [Edit<T>], cx)`** (batch
    apply), `get`.
- **`trait Item: Clone { type Summary: Summary; fn summary(&self, cx) -> Self::Summary; }`** — a stored leaf and
  how to summarize it.
- **`trait KeyedItem: Item { type Key: Dimension + Ord; fn key() -> Self::Key; }`** — adds an ordered key for
  map/`edit` semantics.
- **`trait Summary: Clone { type Context<'a>: Copy; fn zero(cx); fn add_summary(&mut self, &Self, cx); }`** — the
  aggregate. **The `Context` associated type is the key evolution over Warp's fork:** summarization can fold in
  external, borrowed data (e.g. a language/tab config) instead of being a pure `AddAssign`. Monoid-shaped:
  `zero` is the identity, `add_summary` the associative combine.
- **`trait ContextLessSummary { fn zero(); fn add_summary(&mut self, &Self); }`** — convenience for summaries that
  need no context; blanket-impls `Summary` with `Context = ()`. (`rope`'s `TextSummary`/`ChunkSummary` use this.)
- **`struct NoSummary`** — a placeholder `Summary` for trees that don't need one (deliberately a named type, not
  `()`, to dodge blanket-impl collisions with the `()` fill-in dimension).
- **`trait Dimension<'a, S: Summary>: Clone`** — a coordinate *extracted from* a summary by folding
  (`zero` + `add_summary`). Any `S: Summary` is a `Dimension` over itself; `()` is the no-op fill-in dimension.
  Dimensions are the seek coordinates.
- **`struct Dimensions<D1, D2, D3 = ()>(pub D1, pub D2, pub D3)`** — seek/track **several coordinates at once** in
  one traversal (itself a `Dimension`).
- **`trait SeekTarget<'a, S, D>`** — comparator that tells a cursor "am I before/at/after this position?"; any
  `Ord` dimension auto-implements it.
- **`enum Bias { Left, Right }`** (+ `invert()`) — tie-break direction when a position is ambiguous
  (Zed's replacement for Warp's `SeekBias`; the same concept that later disambiguates anchors and folds).
- **`enum Node<T> { Internal{…}, Leaf{…} }`** — the Arc'd node; `pub` but implementation detail.
- **`enum Edit<T: KeyedItem> { Insert(T), Remove(Key) }`** — batch descriptor for `SumTree::edit`.

**Cursors** (`cursor.rs`, re-exported): **`Cursor<'a,'b,T,D>`** — stateful traversal tracking a running dimension
`D`: `seek(pos,bias)`, `seek_forward`, **`slice(end,bias) -> SumTree<T>`** (extract a sub-tree, `O(log n)`),
`suffix()`, **`summary::<Target,Output>(end,bias)`** (aggregate a range into any dimension), `next`/`prev`,
`search_forward`/`search_backward`, `item`/`item_summary`/`next_item`/`prev_item`, `start`/`end`.
**`FilterCursor`** — the same over a predicate-pruned view (skips whole subtrees whose summary fails the filter).
**`Iter`** — a plain forward item iterator.

**Ordered map/set** (`tree_map.rs`, re-exported): **`TreeMap<K,V>`** (a `SumTree<MapEntry<K,V>>`) and
**`TreeSet<K>`** — a **persistent, cheaply-cloneable ordered map/set**: `insert`/`insert_or_replace`, `get`,
`contains_key`, `remove`, `remove_range`, `iter`/`iter_from`, `closest`, `retain`, `update`, `first`/`last`,
`insert_tree`. Plus **`trait MapSeekTarget<K>`**. This is a cherry-pickable Apache-licensed persistent B-tree map,
independent of the rope.

## Depends on (internal)

- **None.** Only external crates: `heapless` (fixed-capacity `ArrayVec` for node children), `rayon` (parallel
  build), `log`, `ztracing`/`tracing`, `proptest` (optional, `test-support`).

## Used by (internal dependents)

13 Zed crates — the entire text/project spine: **`rope`** and **`text`** (the CRDT buffer), **`editor`**,
**`language`**, **`multi_buffer`**, **`buffer_diff`**, **`project`**, **`worktree`**, **`git`**, **`gpui`**,
`copilot`, `markdown`, `notifications`.

## Related crates

- [rope](./rope.md) — `Rope = SumTree<Chunk>`; the primary text consumer, and the reason `sum_tree` exists in the
  editor stack. `rope`'s `TextSummary`/`Point`/`PointUtf16`/`OffsetUtf16` are the canonical `Dimension`s.
- [subsystem 02 §2](../subsystems/02-text-buffer-anchors.md) — the narrative of why a *summarized* tree beats a
  flat rope-of-strings (the summary is **open**: add a coordinate → all existing seeks still `O(log n)`).

## Reimplementation on our stack (Marley)

**Current baseline.** Marley has **no `SumTree`.** `marley_editor::Buffer` (`crates/editor`) wraps
**`ropey::Rope` (MIT)** directly (`type Rope = ropey::Rope`) and is **char-indexed** via `CharOffset`, driving
ropey's own `char_to_line` / `line_to_char` / `char_to_byte` conversions. ropey is *itself* a chunked-B-tree rope
with `O(log n)` byte/char/line seek — so Marley already has "property 1–3" (balanced, summarized, snapshot-cheap)
for the fixed coordinate set ropey ships.

**Decision — `sum_tree` is OPTIONAL / LAST (matches the text-subsystem conclusion: keep ropey).** Do **not**
adopt `sum_tree` now. It only pays for itself when Marley needs an **open summary dimension that ropey cannot
answer in `O(log n)`** — i.e. aggregating a *custom* per-row/per-span coordinate up the tree and seeking it:
diagnostics-per-line, fold/collapse regions, decoration or git-blame spans, agent annotations, "longest visible
row" for the horizontal scrollbar. ropey answers only bytes/chars/lines/utf16; the moment Marley wants to *seek
by a bespoke aggregate*, a `SumTree` with a custom `Summary` is the textbook fit and ropey is not.

**When we do reach for it, the license makes it easy.** `sum_tree` is **Apache-2.0**, so — unlike `rope`/`text`
(GPL) — it may be **adopted directly, verbatim, into any Marley layer including the proprietary brain**, with no
copyleft contamination. Two concrete on-ramps:

1. **Cherry-pick `TreeMap`/`TreeSet` alone** — a persistent, cheaply-cloneable ordered map is independently
   useful (anchor→metadata maps, versioned decoration sets) and needs none of the rope. Apache-clean.
2. **Build a Marley-original rope on `sum_tree`** if/when the open-summary superpower is required — adopt the
   Apache tree, define Marley's own `Summary`/`Dimension` set, and *study but do not copy* Zed's GPL `rope` for
   the shape. See [rope.md](./rope.md) for that boundary.

Until one of those triggers fires: **keep ropey, leave `sum_tree` on the shelf.**

## Provenance & licensing

- **`[sum_tree Apache-2.0]`** — permissively licensed; safe to vendor/adopt directly into editor *or* brain. The
  only Apache island in Zed's otherwise-GPL text stack.
- **`[permissive/public: augmented-B-tree textbook DS]`** — the underlying idea (an order-statistic / augmented
  B+ tree whose internal nodes cache a monoidal summary, seeked in `O(log n)`) is standard CS literature, not
  Zed-proprietary. Even a full Marley reimplementation stands on public ground; the Apache text merely removes
  any doubt.

## Notes / gotchas

- **`Context` on `Summary`** is the biggest delta from the Warp fork's `sum_tree` (which used a plain
  `AddAssign` summary). If Marley ports concepts across both references, note the two `Summary` shapes are not
  wire-compatible — Zed threads a borrowed `cx` through every fold.
- `TREE_BASE` shrinks to `2` under `#[cfg(test)]` to stress rebalancing; production fan-out is `6`
  (`2*TREE_BASE = 12` items/node max).
- `Node<T>` is `pub` but is plumbing — drive everything through `SumTree`, `Cursor`/`FilterCursor`, and the
  traits.
- Copy-on-write via `Arc<Node>`: clones and range-`slice`s are cheap and structurally shared — this is what makes
  Zed's versioned `BufferSnapshot`s viable, and is exactly the property Marley would want from any future
  summarized store.
- Ships `property_test.rs` + a `test-support`/`proptest` feature — the invariants (balance, summary correctness)
  are property-checked, worth preserving in any reimplementation.
