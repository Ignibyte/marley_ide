# text

> Per-crate reference (Marley round 2 — Zed granular). Crate dir: `crates/text`. Zed
> (github.com/zed-industries/zed) is the EDITOR reference for Marley's editing surface; Marley reimplements
> each capability **in its own code**. Zed source is cloned only into session scratch — never committed. This
> doc is Marley's own description of the crate's design and API, not a copy of it.

| | |
|---|---|
| Subsystem | [02 — text / buffer / rope / anchors](../subsystems/02-text-buffer-anchors.md) |
| License | **GPL-3.0-or-later** (`license = "GPL-3.0-or-later"`; per-crate `LICENSE-GPL` symlink) |
| Internal deps | 5 — `clock`, `sum_tree`, `rope`, `collections`, `util` |
| Used by | 28 (Zed tree); the primary consumer is `language` (adds tree-sitter / LSP / diagnostics on top) |
| Provenance | `[Zed-derived / GPL]` CRDT rendition built over `[public]` anchor + fractional-index concepts |

## Purpose

`text` is the **CRDT text buffer** at the bottom of Zed's editing stack — the layer that stores characters and
answers positional questions *correctly across edits and across replicas*. It is deliberately **not** "a string
with a cursor." A `Buffer` here is a **persistent, versioned, operation-based CRDT** in which every inserted run
of text has a **stable identity** (a `clock::Lamport` timestamp + a dense `Locator`) that is independent of its
current numeric offset. That single property is what makes three otherwise-hard features fall out:

1. **Anchors** — a logical position that survives arbitrary edits (the prerequisite for async LSP, multi-cursor,
   marks, folds, diagnostics, decorations).
2. **Convergent concurrency** — edits from multiple replicas (a human, a remote collaborator, an **agent**) can
   arrive out of order and every replica converges to the same text.
3. **Syncable, operation-based undo** — undo is itself a first-class CRDT operation, not string surgery.

Everything Zed builds above this — `language::Buffer` (tree-sitter, LSP, diagnostics), `MultiBuffer`,
`editor::Editor`, multi-cursor, vim — resolves down to the primitives in this crate. It sits directly on top of
`rope` (the text store) and `clock` (the logical clock), and uses `sum_tree` for every index it keeps.

## Key types, modules & public API

The crate is one large module (`src/text.rs`, ~3.6k lines) plus focused siblings. Types are grouped by file.

### `text.rs` — the buffer, history, and operations

- **`Buffer`** `[Zed-derived]` — the mutable CRDT buffer. Holds a `BufferSnapshot`, a `History`, a
  `deferred_ops: OperationQueue<Operation>` (out-of-order ops awaiting their causal dependencies), a
  `lamport_clock: clock::Lamport`, subscription `Topic`, and async waiter maps. Key API:
  - construction / identity: `new(replica_id, remote_id: BufferId, base_text)`, `new_normalized(...)`,
    `replica_id()`, `remote_id()`, `version() -> clock::Global`, `snapshot() -> &BufferSnapshot`,
    `branch()` (a `LOCAL_BRANCH` replica sharing history).
  - editing: `edit<R,I,S,T>(edits) -> Operation` (the core mutation; takes `(range, new_text)` pairs, returns
    the `EditOperation` it generated), `set_line_ending(...)`.
  - replication: `apply_ops(ops)` (apply remote `Operation`s; defers any whose dependencies are unmet),
    `has_deferred_ops()`, `deferred_ops_len()`.
  - transactions (undo grouping): `start_transaction()` / `end_transaction()` (nestable via a depth counter;
    empty transactions are discarded), `..._at(now)` variants, `finalize_last_transaction()` (pins a boundary),
    `group_until_transaction(id)`, `push_transaction(...)`, `merge_transactions(...)`,
    `forget_transaction(...)`, `get_transaction(id)`.
  - undo/redo: `undo() -> Option<(TransactionId, Operation)>`, `redo()`, `undo_transaction(id)`,
    `undo_to_transaction(id)`, `redo_to_transaction(id)`, `undo_operations(counts)`. Undo emits an
    `Operation::Undo`, never a string diff.
  - async waiters: `wait_for_edits(edit_ids)`, `wait_for_anchors(anchors)`, `wait_for_version(version)`,
    `give_up_waiting()` — futures that resolve once the buffer has observed a given edit/anchor/version (the
    primitive an async LSP or collab flow awaits before resolving a stale position).
  - subscriptions: `subscribe() -> Subscription<usize>` (a stream of coalesced `Patch`es of edits).
- **`BufferSnapshot`** `[Zed-derived]` — a cheap (`Arc`-shared, copy-on-write) immutable view; the unit of
  off-thread work. Holds `visible_text: Rope`, `deleted_text: Rope` (**deletes are tombstoned, not dropped** —
  needed so a concurrent op that references deleted text still resolves), `fragments: SumTree<Fragment>`,
  `insertions: SumTree<InsertionFragment>` (the identity index), an `UndoMap`, and a `version: clock::Global`.
  Its API is the whole read surface:
  - anchors: `anchor_at(pos, Bias)`, `anchor_before(pos)`, `anchor_after(pos)`, `anchor_range_inside/outside`,
    `offset_for_anchor(&Anchor) -> usize`, `summary_for_anchor::<D>(&Anchor) -> D`.
  - coordinate conversions (one `O(log n)` seek each, any-to-any): `point_to_offset`, `offset_to_point`,
    `offset_to_offset_utf16`, `offset_utf16_to_offset`, `point_utf16_to_offset`, `point_to_point_utf16`, …plus
    `clip_offset`/`clip_point`/`clip_offset_utf16`/`clip_point_utf16` (snap a possibly-invalid position to a
    boundary — the LSP robustness detail).
  - text access: `text()`, `text_for_range(range)`, `chars_at(pos)`, `reversed_chars_at`, `line_len(row)`,
    `line_indent_for_row`, `text_summary()`, `text_summary_for_range::<D>(range)`, `max_point()`.
  - diffing: `edits_since::<D>(since: &Global)` and `edits_since_in_range(...)` — iterate the `Edit<D>`s between
    two versions (drives incremental re-highlight / re-layout); `anchored_edits_since(...)` variants.
- **`Anchor` resolution path** (the mechanism, `[Zed-derived]`): `offset_for_anchor` looks the insertion up by
  `timestamp` in `insertions` → gets the current `Locator` → seeks `fragments` to that Locator → adds the
  intra-insertion `offset`. It `debug_assert!`s `version.observed(anchor.timestamp())`: an anchor is only
  meaningful in a snapshot new enough to contain the edit it points into.
- **`History` / `HistoryEntry` / `Transaction`** `[Zed-derived]` — undo/redo stacks of `HistoryEntry`
  (`first_edit_at`/`last_edit_at`/`suppress_grouping`). `Transaction { id, edit_ids, start }`. `group()` merges
  adjacent entries whose time gap is under `group_interval` (**300 ms** in prod, `Duration::ZERO` in tests), so
  a fast typed run coalesces into one ⌘Z but a pause starts a fresh step. Nestable transactions collapse a
  multi-cursor / find-replace-all edit into a single undo step.
- **`Operation`** `[Zed-derived]` — `Edit(EditOperation)` | `Undo(UndoOperation)`, the syncable unit.
  - `EditOperation { timestamp: Lamport, version: Global, ranges: Vec<Range<FullOffset>>, new_text: Vec<Arc<str>> }`.
  - `UndoOperation { timestamp, version, counts: HashMap<Lamport, u32> }` — records *which edit timestamps to
    flip how many times*; undo/redo is toggling fragment visibility via the `UndoMap`, not re-deriving text.
- **CRDT internals (private)** `[Zed-derived]` — `Fragment { id: Locator, timestamp, insertion_offset, len,
  visible, deletions, max_undos }` (a contiguous run from one insertion; `visible`+`deletions` implement
  tombstoning), `FragmentSummary`, `InsertionFragment` / `InsertionFragmentKey` (the timestamp→Locator index),
  `FullOffset`, `VersionedFullOffset` (a `SeekTarget` that resolves a position *as of* a given version).
- **`BufferId`** (`NonZeroU64`), **`LineEnding`** (`Unix`/`Windows` + normalization helpers), **`LineIndent`**,
  **`Edit<D> { old: Range<D>, new: Range<D> }`** (the generic diff item; `flatten`, `old_len`/`new_len`), and
  the coordinate traits **`ToOffset` / `ToPoint` / `ToOffsetUtf16` / `ToPointUtf16` / `FromAnchor`** (each
  implemented for `usize`, `Point`, `PointUtf16`, `OffsetUtf16`, and `Anchor` — so any API accepts any
  coordinate, including an anchor, and converts on demand).

### Sibling modules

- **`anchor.rs`** — **`Anchor`** `[Zed-derived rendition of a [public] concept]`: `{ timestamp_replica_id,
  timestamp_value, offset: u32, bias: Bias, buffer_id }` (the Lamport is stored split-into-fields to pack into
  padding and save 8 bytes). Not an offset — `timestamp` names the **insertion op**, `offset` is the byte
  offset *within that insertion*, `bias: Bias::{Left,Right}` decides which side of an exact-position insert the
  anchor sticks to. `min_for_buffer`/`max_for_buffer` sentinels, `cmp(other, &snapshot)` (orders by fragment
  `Locator` then offset then bias), `is_valid(&snapshot)` (does it still land in visible text?), `bias_left`/
  `bias_right`. Extension traits **`OffsetRangeExt`** (`Range<T: ToOffset>` → offset/point ranges) and
  **`AnchorRangeExt`** (`cmp`/`overlaps`/`contains_anchor` for `Range<Anchor>`).
- **`locator.rs`** — **`Locator(SmallVec<[u64;2]>)`** `[public: dense/fractional indexing, LSEQ/Logoot family]`:
  a dense identifier for a position in an ordered collection. `Locator::between(a, b)` mints a new key strictly
  between two neighbors **without renumbering** anything else (the `>> 48` shift keeps sequential typing at
  depth-1/2), which is exactly why an existing anchor's identity stays stable when text is inserted nearby.
  `min()`/`max()` sentinels; implements `sum_tree::{Item, KeyedItem, ContextLessSummary}`.
- **`undo_map.rs`** — **`UndoMap(SumTree<UndoMapEntry>)`** `[Zed-derived]`: maps an edit's `Lamport` to an undo
  count; `is_undone(edit_id)` = count is odd. `was_undone(edit_id, &version)` answers the question *as of* a
  version — the versioned read redo/collab needs.
- **`operation_queue.rs`** — **`OperationQueue<T: Operation>`** `[Zed-derived]`: a `SumTree`-backed, Lamport-
  sorted, deduped queue; the `deferred_ops` store for out-of-order operations. `trait Operation { fn
  lamport_timestamp() }`.
- **`patch.rs`** — **`Patch<T>(Vec<Edit<T>>)`** `[Zed-derived; the idea of composing deltas is [public]]`: a
  normalized, non-overlapping sequence of edits. `compose(other)` folds two patches into one, `invert()` swaps
  old↔new, `old_to_new(pos)` / `edit_for_old_position(pos)` **rebase a position through the patch** — this is
  the offset-rebasing primitive at the heart of the Marley plan below.
- **`selection.rs`** — **`Selection<T> { id, start, end, reversed, goal }`**, **`SelectionGoal`**. Generic over
  the coordinate `T`; `resolve::<D>(&snapshot)` turns a `Selection<Anchor>` into a `Selection<D>`. Zed's
  selections are stored as **`Selection<Anchor>`** so every caret survives every other caret's edit.
- **`subscription.rs`** — **`Topic<T>`** / **`Subscription<T>`**: publish coalesced `Patch<T>`es to observers;
  `consume()` drains the accumulated edits since last read.
- **`network.rs`** — **`Network<T, R: Rng>`**: an in-memory, randomized message-reordering harness used only to
  fuzz-test CRDT convergence (`broadcast`/`receive`/`disconnect_peer`/`replicate`). Test infrastructure, not
  runtime.

## Depends on (internal)

- **`clock`** — [clock.md](./clock.md); `Lamport` stamps every edit/undo, `Global` versions every snapshot.
- **`rope`** — the actual text store (`visible_text`/`deleted_text: Rope`) and all coordinate summaries
  (bytes/chars/UTF-16/lines carried in `rope::TextSummary`).
- **`sum_tree`** — every index in the buffer (`fragments`, `insertions`, `UndoMap`, `OperationQueue`) is a
  `SumTree`. *(In the Zed tree `sum_tree` is **Apache-2.0** — the tree engine itself is permissively licensed;
  only the text/CRDT built on it is GPL. See the subsystem doc §2/§8.)*
- **`collections`**, **`util`** — hash maps / small helpers.

## Used by (internal dependents)

28 crates in the Zed tree. The load-bearing one is **`language`** (`language::Buffer` wraps `text::Buffer` and
adds tree-sitter, LSP, diagnostics, git). From there it fans out to `multi_buffer`, `editor`, `project`,
`collab`, `agent` / `action_log` / `acp_thread` (the agent-edit path), `buffer_diff`, and the rest of the app.

## Related crates

- [clock](./clock.md) — the logical-clock crate this one is built on (`Lamport`, `Global`, `ReplicaId`).
- **`rope`** and **`sum_tree`** — covered in the subsystem doc [02 §2 "The rope + SumTree"](../subsystems/02-text-buffer-anchors.md)
  (no dedicated per-crate doc yet). `rope` is GPL; `sum_tree` is Apache-2.0.
- **`language`** — the next layer up (out of scope here): grammar, LSP, diagnostics over `text::Buffer`.

---

## Marley mapping & reimplementation on our stack

### Marley baseline `[Marley-original]` (post-M15)

`marley_editor::Buffer` wraps **`ropey::Rope` (MIT)** directly. Positions are **`CharOffset`** (char-indexed);
versioning is a single monotonic **`BufferVersion(u64)`** (`+1` per edit); each edit emits a **`BufferDelta`**
(char + byte ranges, new lengths); undo is a **two-`Vec` invert-stack** with contiguity coalescing (M15 #253).
There are **no anchors** — `Selection { anchor: CharOffset, head: CharOffset }` stores raw offsets that go stale
on any edit. Single-writer (the agent's writes are serialized through the same buffer via `EditOrigin::Agent`).

### The mapping — what to take, what to leave

| Zed `text` capability | Marley today | Action |
|---|---|---|
| CRDT identity (`Fragment`/`InsertionFragment`/`Locator`) | none (raw ropey offsets) | **Do not port.** Full CRDT is GPL and overkill for single-writer. Defer to a real multi-writer milestone. |
| `Anchor { timestamp, offset, bias }` | `Selection` holds raw `CharOffset`s | **Reimplement as a delta-log anchor** (below) — `[Marley-original]`. |
| `Patch::compose` / `old_to_new` (rebase a position through edits) | `BufferDelta` emitted but unconsumed | **Adopt the *idea*** — a bounded log of `BufferDelta`s + a `resolve` that rebases. `[Marley-original]`. |
| `clock::Lamport` / `Global` version vector | `BufferVersion(u64)` | **Keep `u64`** as a staleness detector for single-writer + async. Version-vector deferred with the CRDT (see [clock.md](./clock.md)). |
| `Transaction` / nestable undo grouping | contiguity-coalesced invert-stack | **Add a `Transaction` wrapper** over the delta-log so a multi-cursor / replace-all edit is one undo step. `[Marley-original]`. |
| Operation-based undo (`UndoOperation`, `UndoMap`) | string-inverse undo | **Keep string-inverse** for single-writer; op-based undo is a Tier-B/CRDT concern — defer. |
| UTF-16 coordinate + `clip_*` | char/byte only | Additive ropey wrapper (`len_utf16_cu` etc.), landed **with** LSP — independent of anchors. |

### The recommended first step: a delta-log anchor `[Marley-original]`, from the `[public]` bias concept

Because Marley is single-writer, it does **not** need Zed's Lamport-stamped CRDT to get stable positions. The
pragmatic anchor is:

```
Anchor { version: BufferVersion, offset: CharOffset, bias: Bias }
```

Resolve it by **replaying the accumulated `BufferDelta`s** from the anchor's `version` up to the buffer's
current version — for each intervening edit: shift `offset` if the edit was strictly before it, clamp if the
edit deleted across it, and use `bias` to break the tie when an insert lands exactly on it. Marley already emits
one `BufferDelta` per edit; keep a **bounded ring** of them. This is functionally Zed's `Patch::old_to_new`
(rebase a position through composed edits) **without** the CRDT — it delivers ~90% of anchors' value (stable
positions + version resolution for single-writer and async LSP) at a fraction of the cost and stays **clean of
GPL**. `bias` and "resolve a position through an edit" are public concepts; the delta-log is Marley's own.

Then: convert `Selection` to hold two `Anchor`s → real multi-cursor (`SelectionSet` whose members survive each
other's edits) → a `Transaction` wrapper for one-undo-step grouping. **Anchors come before LSP and before
multi-cursor** — both are unbuildable-correctly without them.

### What is deferred (the full `[Zed-derived / GPL]` model)

Lamport-stamped insertions + `Locator` fragments + tombstoned deletes + operation-based undo + a `deferred_ops`
queue — the whole CRDT — is adopted **only** when Marley needs true multi-writer concurrency (live collab, or a
human and agent editing the *same region* with automatic convergence rather than serialized turns). When that
day comes it **subsumes** the delta-log anchor (anchors become CRDT anchors) and must be **reimplemented from the
CRDT / fractional-indexing literature, not copied** from this GPL crate. See the subsystem doc for the full
staged plan and licensing map.

## Provenance & the GPL boundary

- **The crate is `GPL-3.0-or-later`.** Its *specific rendition* — the `Anchor` struct layout, the
  `Fragment`/`InsertionFragment`/`Locator` CRDT, `History`/`Transaction`/`UndoMap`, the apply/defer path — is
  **`[Zed-derived / GPL]`**: study it, but reimplement from concepts; it may live only in Marley's
  editor/terminal layer, **never in the brain/agent layer** (the sold product must stay copyleft-clean).
- **`[public / permissive]`** underneath the rendition: the **anchor concept** (every serious editor/CRDT has
  one), **Lamport timestamps + version vectors** (distributed-systems CS — see [clock.md](./clock.md)),
  **dense/fractional indexing** (`Locator` is the LSEQ/Logoot family), **`Bias`** semantics, **UTF-16 position
  semantics** (LSP spec), and the **`sum_tree` engine itself (Apache-2.0)**. Marley may implement all of these
  freely.
- **`[Marley-original]`**: the delta-log anchor (`Anchor{version,offset,bias}` + bounded `BufferDelta` ring +
  `resolve`), the `Transaction` wrapper, and the whole current `marley_editor` baseline.
- **The seam that protects the boundary:** the brain touches text only through Marley's own public API
  (`Buffer::edit(range, replacement, EditOrigin::Agent)`, `Anchor::resolve`, coordinate conversions) — all
  `[Marley-original]`. `EditOrigin::Agent` keeps agent writes on the clean side.

## Notes / gotchas

- **Deletes are tombstoned, not removed** (`deleted_text: Rope` + `Fragment.visible`) — a subtlety that only
  matters once you have concurrent ops referencing already-deleted text. Marley's single-writer model does not
  need it yet; do **not** import it prematurely.
- **`group_interval` is 300 ms in prod, `ZERO` in tests** — Zed disables time-grouping under test because it is
  a footgun in deterministic tests. Mirror that if Marley adds time-based grouping.
- **`MAX_INSERTION_LEN`** caps a fragment so intra-insertion offsets fit in `u32` (16 under test to force
  splits). An implementation detail of the CRDT, irrelevant to the delta-log plan.
- **Anchors assert version observation** (`offset_for_anchor` debug-asserts `version.observed(timestamp)`).
  Marley's delta-log analogue must define its eviction policy: what happens when an anchor's `version` has
  fallen off the bounded ring — clamp, or force a resolve-at-eviction that "bakes" the anchor forward. (Open
  question #1 in the subsystem doc.)
- **`ReplicaId::AGENT` already exists in Zed** (see [clock.md](./clock.md)) — Zed models an agent as a
  first-class concurrent replica. Whether Marley ever needs that (vs. serialized agent turns) is the single
  question that decides if the deferred GPL milestone is ever built.
