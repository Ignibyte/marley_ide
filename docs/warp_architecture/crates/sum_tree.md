# sum_tree

> Per-crate reference (Marley round 2). Crate dir: `crates/sum_tree`. Marley is forked from Warp (warpdotdev/warp).

| | |
|---|---|
| Subsystem | [editor-and-text](../subsystems/02-editor-and-text.md) |
| License | AGPL v3 (`AGPL-3.0-only`, inherited from workspace; no per-crate `LICENSE`) |
| Internal deps | 0 |
| Used by | 5 |
| Provenance | Concept **`[permissive]`** (augmented/summarized B-tree; Zed's own `sum_tree` is Apache-2.0); Warp's copy carries the AGPL workspace license. **Marley uses `ropey` (MIT)** — no sum_tree. See [subsystem — Provenance & licensing](../subsystems/02-editor-and-text.md). |

## Purpose

`sum_tree` is a foundational, dependency-free data-structure crate: a copy-on-write B-tree (a "summed tree" / rope-like B+ tree) whose nodes carry user-defined **summaries**. Each leaf item contributes a `Summary` that aggregates up the tree, so any monotone dimension (byte count, character count, line count, etc.) can be seeked in `O(log n)` and edited persistently via structural sharing (`Arc<Node<T>>`). It is the engine underneath Warp's text buffer (`warp_editor`'s rope/`BufferText`) and any other place that needs balanced, summarizable sequences. This is a direct descendant of the Zed `sum_tree` design.

## Key types, modules & public API

All in `crates/sum_tree/src/lib.rs` plus the cursor module `cursor.rs`.

- **`SumTree<T: Item>`** — the persistent B-tree, wrapping `Arc<Node<T>>`. Entry points: `new()`, `from_item()`, `push()`, `push_tree()`, `extend()`, `insert()`, `edit(&mut [Edit<T>])`, `update_last()`, `summary()`, `extent::<D>()`, `first()`/`last()`, `is_empty()`.
- **`trait Item`** — items must produce a `Summary` (`type Summary: AddAssign + Default + Clone + Debug`) via `fn summary(&self)`.
- **`trait KeyedItem: Item`** — adds an ordered `Key` for `insert`/`edit` semantics.
- **`trait Dimension<'a, Summary>`** — a value extractable from a summary by folding (`add_summary`); the unit type `()` implements it as a no-op. Dimensions are the seek coordinates.
- **`enum SeekBias { Left, Right }`** — tie-breaking direction for cursor seeks.
- **`enum Node<T>`** — `Internal`/`Leaf` (public but mostly internal plumbing).
- **`enum Edit<T: KeyedItem>`** — batch insert/remove descriptor passed to `edit`.
- **`Cursor`, `FilterCursor`** (`pub use cursor::{Cursor, FilterCursor}`) — typed traversal: `cursor::<S, U>()` and `filter::<F, U>()` walk the tree tracking two dimensions while seeking.
- `const TREE_BASE` — node fan-out (6 in normal builds, 2 under the `test-util` feature to stress balancing).

## Depends on (internal)

- None. Only external crates (`arrayvec`, `log`).

## Used by (internal dependents)

- [warp_editor](./warp_editor.md) — the text buffer / rope is built on `SumTree`.
- [warpui](./warpui.md) and [warpui_core](./warpui_core.md) — summarizable sequences in the UI layer.
- [warp](./warp.md) — top-level app.
- [integration](./integration.md) — integration-test harness.

(5 dependents total.)

## Related crates

- [string-offset](./string-offset.md) — supplies the `CharOffset`/`ByteOffset` types that become `Dimension`s over text summaries.
- [warp_editor](./warp_editor.md) — primary consumer; see its `content::text::TextSummary`.

## Marley relevance

**KEEP (verbatim).** This is pure, generic, infrastructure-grade data-structure code with zero internal deps and no Warp branding, auth, or network surface. None of the four Marley goals (UI panel, session spawn/write/read, de-auth, de-Warp rebrand) touch it. Renaming the package buys nothing and would churn 5 dependents' manifests. Leave the package name `sum_tree` as-is. Only revisit if we ever need a new summary dimension for a custom Marley panel that renders buffer-derived data — even then we extend consumers, not this crate.

## Notes / gotchas

- The `test-util` feature deliberately shrinks `TREE_BASE` to 2 to force deep trees in tests; production fan-out is 6. Several dependents pull `sum_tree` with `features = ["test-util"]` only in `[dev-dependencies]`.
- Persistent / copy-on-write via `Arc`: clones are cheap and snapshots are structurally shared — important for the editor's versioned `BufferSnapshot`.
- `Node<T>` is `pub` but is implementation detail; consumers should drive everything through `SumTree`, `Cursor`, and the traits.
