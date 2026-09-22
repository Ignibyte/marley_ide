# Subsystem 02 — Text / Buffer / Rope / Anchors

Part of the Marley **Zed** architecture docs (round 1). Zed is the EDITOR reference for Marley's editing
surface; Marley reimplements each capability **in its own code**. Zed's source is cloned only into session
scratch — never committed. This doc is Marley's own description of Zed's design.

> Scope: the lowest layer of Zed's editor — the **text model** that stores characters and answers positional
> questions. Four crates: `sum_tree` (the summarized B-tree), `rope` (text on top of it), `clock` (the
> distributed logical clock), and `text` (the CRDT `Buffer` + `Anchor` + undo `History`). Everything Zed does
> above this — `language::Buffer`, `MultiBuffer`, `editor::Editor`, LSP, syntax, multi-cursor — resolves down
> to these primitives. This is the layer Marley must understand before it can grow anchors, LSP, and
> multi-cursor.

## Provenance & the GPL boundary (read first)

Per [`../README.md`](../README.md), every claim is tagged so the eventual legal review can audit the copyleft
surface. The tags used here:

- **`[Zed-derived]`** — Zed's *specific rendition* of a design, under **GPL-3.0**. Marley may study it but must
  **reimplement from the public concept**, not copy. GPL-derived code may live only in Marley's editor/terminal
  layer, **never in the brain/agent layer** (which is the sold product and must stay copyleft-clean).
- **`[public/permissive]`** — a design that is textbook CS or ships under a permissive license, so Marley can
  implement it freely: **SumTree = an augmented (summarized) B-tree**, standard data-structure literature;
  **ropey = MIT** (already Marley's text store); **Lamport timestamps + version vectors** = public distributed-
  systems CS; **fractional indexing / dense ordering** (Zed's `Locator`) = the LSEQ/Logoot family, public;
  **UTF-16 position semantics** = the **LSP specification**, public; **gpui = Apache-2.0**.
- **`[Marley-original]`** — Marley's current code and the reimplementation choices recommended here.

> **A licensing finding worth flagging up front:** in the Zed tree, **`sum_tree` is licensed `Apache-2.0`**
> (it has its own `LICENSE-APACHE`), while **`rope`, `text`, and `clock` are `GPL-3.0-or-later`**. So the *tree
> data structure itself* is permissively licensed, but the *text/rope/CRDT built on it* is GPL. This sharpens
> the boundary: Marley can lean on the SumTree **concept** (and even the Apache crate) freely; the anchor/CRDT
> **rendition** is the GPL part to reimplement from concepts.

---

## 1. Purpose & big picture

Zed's editor is fast and correct because its text model is not "a string with a cursor." It is a **persistent,
summarized, versioned, CRDT rope**. Four properties stack up, each enabling the next:

1. **Summarized B-tree (`sum_tree`)** `[public]` — text is stored in a balanced tree where every node caches a
   `Summary`. Any *dimension* derivable from that summary (bytes, chars, UTF-16 units, lines) is answerable in
   `O(log n)`, and conversions between dimensions are a single tree seek.
2. **Rope (`rope`)** `[Zed-derived]` — a `SumTree<Chunk>` whose summary carries **all** the coordinate systems
   at once, so byte↔char↔UTF-16↔line is one seek in any direction.
3. **Immutable snapshots** — because the tree is `Arc`-shared and copy-on-write, a point-in-time
   `BufferSnapshot` is a cheap clone. Off-thread work (syntax parse, LSP, diff, search) runs against a frozen,
   consistent view while editing continues.
4. **CRDT identity (`text` + `clock`)** `[Zed-derived]` — every inserted run of text has a **stable identity**
   (a Lamport timestamp + a dense `Locator`), independent of its current numeric offset. This is what makes
   **anchors** possible (a position that survives edits), what makes **concurrent/async edits** converge, and
   what makes **undo** a first-class operation that can itself be synced.

The thesis for Marley: **ropey already gives Marley property 1–3 for the coordinate systems it supports.** What
ropey does *not* give — and what Marley will need for LSP, multi-cursor, marks, diagnostics, and agent co-
editing — is property 4 (stable identity / anchors) and the UTF-16 coordinate. The rest of this doc is a
component-by-component read of how Zed builds property 4, and a staged plan for Marley to add *just enough* of
it, in the right order, without inheriting the full GPL CRDT before it is needed.

### The layered stack

```
clock        Lamport timestamp + Global version vector        [Zed-derived rendition / public concept]
  │
sum_tree     SumTree<T: Item>  — Arc<Node>, Summary, Dimension, Cursor, Bias   [Apache-2.0 / public concept]
  │
rope         Rope = SumTree<Chunk>; TextSummary {bytes,chars,utf16,lines}      [Zed-derived / GPL]
  │
text         Buffer + BufferSnapshot (CRDT: Fragment/InsertionFragment),       [Zed-derived / GPL]
             Anchor, Locator, History/Transaction, Operation::{Edit,Undo}
  │
language     language::Buffer (adds tree-sitter, LSP, diagnostics) — out of scope for this doc
```

---

## 2. The rope + SumTree — why a summarized B-tree

### 2.1 `sum_tree` — the engine `[public concept; the crate is Apache-2.0]`

`crates/sum_tree/src/sum_tree.rs` (+ `cursor.rs`, `tree_map.rs`). The core is deliberately generic:

- **`SumTree<T: Item>(Arc<Node<T>>)`** — a B+ tree; leaves hold up to `2 * TREE_BASE` items (`TREE_BASE` is
  small under test to force deep trees). The `Arc` is the whole point: clone = pointer bump, and edits rebuild
  only the path from root to the changed leaf (**copy-on-write / structural sharing**). This is what makes
  snapshots free.
- **`trait Item { type Summary; fn summary(&self, cx) -> Summary }`** — every stored item contributes a
  summary.
- **`trait Summary { fn zero(cx); fn add_summary(&mut self, other, cx) }`** — summaries are a monoid
  (associative, with identity), so a parent node's summary is the fold of its children's. `Context` lets the
  fold depend on external data (Zed threads the buffer version through here for versioned seeks).
- **`trait Dimension<'a, S: Summary> { fn add_summary(&mut self, &S, cx) }`** — a *quantity you can read off a
  summary*. The killer feature: **many dimensions over one summary.** `Dimensions<D1, D2, D3>` seeks several at
  once. A single tree is thus indexed simultaneously by byte, char, UTF-16, and line.
- **`trait SeekTarget` + `enum Bias { Left, Right }`** — a `Cursor` walks the tree accumulating a dimension
  until it reaches a target, using `Bias` to decide which side of a boundary to land on (the same `Bias` that
  later parameterizes anchors, §3).

**Why a SumTree instead of a flat rope-of-strings?** Because the summary is *open*: adding a new coordinate
system (or a decoration count, or a diagnostic count) is "add a field to the summary + impl `Dimension`," and
every existing seek keeps working in `O(log n)`. A hand-rolled rope hard-codes its coordinate set.

### 2.2 `rope` — text on the tree `[Zed-derived / GPL]`

`crates/rope/src/rope.rs`. `Rope { chunks: SumTree<Chunk> }`.

- **`Chunk`** `[Zed-derived]` — a leaf holds an `ArrayString<128>` **plus SIMD bitmaps** (`u128` per chunk):
  one bit per byte marking char boundaries, UTF-16 boundaries, newlines, and tabs. Counting chars / UTF-16
  units / lines within a chunk is then a masked `popcount`, not a scan. This is a Zed-specific performance
  rendition — the *concept* (a chunked rope) is public; the bitmap trick is Zed's.
- **`TextSummary`** `[Zed-derived shape; the coordinates are universal]` — the summary carries **every
  coordinate at once**: `len` (bytes), `chars`, `len_utf16` (`OffsetUtf16`), `lines` (a `Point` = rows +
  last-line length), plus layout aids (`longest_row`, `first/last_line_chars`, `last_line_len_utf16`). Because
  all coordinates live in one summary, **any-to-any conversion is one seek** — byte→Point, char→UTF-16,
  Point→char all cost `O(log n)`.
- **`trait TextDimension`** — `usize` (bytes), `OffsetUtf16`, `Point`, `PointUtf16`, and `TextSummary` itself
  all implement it, so `rope.summary_for_range::<D>(range)` is generic over the coordinate you want out.

### 2.3 Marley today `[Marley-original]`

`crates/editor/src/{buffer.rs,types.rs}`. `marley_editor::Buffer` wraps **`ropey::Rope` (MIT)** directly:
`Rope = ropey::Rope` is a type alias. ropey is itself a chunked B-tree rope with `O(log n)` byte/char/line
seeks and cheap `clone` (also `Arc`-shared, copy-on-write) — so **Marley already has SumTree properties 1–3**
for bytes/chars/lines. `char_to_byte`/`byte_to_char` are random-access `O(log n)` ropey conversions;
`point_at` yields a `Point { row, column }` where **column is bytes**.

### 2.4 The gap

- ropey's summary set is **fixed** (bytes/chars/lines). Marley cannot add a UTF-16 dimension (§4) or an anchor/
  fragment dimension (§3) *inside* ropey's tree — those must be layered outside it.
- ropey stores **only text**, with **no per-run identity** — the prerequisite the CRDT (and thus anchors) needs
  (§3). ropey offsets are pure integers that go stale the instant text shifts.

### 2.5 Reimplementation & sequencing

**Keep ropey as the text store.** It already delivers the rope's job. Do **not** port Zed's `rope.rs`
(GPL, and the SIMD-bitmap `Chunk` is a micro-optimization Marley does not yet need). The two things ropey
lacks are addressed *above* it:

- **UTF-16** → additive wrapper over ropey's existing UTF-16 API (§4), no rope change.
- **Anchors** → a stable-identity layer *over* ropey offsets (§3), no rope change initially.

A Marley SumTree of its own becomes worth building **only** if/when Marley needs a custom summary dimension
ropey can't carry (e.g. decoration or diagnostic counts folded into the tree) **or** adopts the full CRDT. If
that day comes, build it **from the public augmented-B-tree concept** (or vendor the **Apache-2.0** `sum_tree`
crate) — this is the one piece of the stack that is *not* GPL. **Sequence: LAST / optional.**

---

## 3. Anchors — stable positions that survive edits

This is the heart of the subsystem and the single most important capability Marley lacks.

### 3.1 The problem anchors solve

A raw offset is a snapshot-relative integer. Insert one character at the top of the buffer and **every** offset
below it is now wrong. Anything that holds a position across an edit is therefore broken by construction:

- an **async LSP** response says "diagnostic at char 4200" but references the buffer *as it was when the request
  was sent* — the user has typed since;
- **multi-cursor**: an edit at cursor #1 must not corrupt cursors #2..N;
- **marks, folds, selections, decorations, breakpoints** all must ride along as the text around them changes;
- an **agent streaming edits** into a buffer the user is also editing needs positions that mean the same thing
  before and after each other's edits.

An **anchor** is a *logical* position that answers "where is this *now*?" for any version of the buffer. The
concept is `[public]` (every serious editor and CRDT has one); Zed's specific rendition is `[Zed-derived / GPL]`.

### 3.2 Zed's anchor design `[Zed-derived / GPL]`

`crates/text/src/anchor.rs`. An `Anchor` is **not** an offset:

```
Anchor { timestamp: Lamport, offset: u32, bias: Bias, buffer_id }
```

- **`timestamp` (Lamport)** identifies the **insertion operation** that created the run of text the anchor sits
  in — not a position in the current buffer, but *which edit* produced the character.
- **`offset`** is the byte offset *within that insertion's* text.
- **`bias: Bias::{Left,Right}`** decides behavior when text is inserted exactly at the anchor: `Left` stays
  before the new text (the anchor "sticks" to the character on its left), `Right` moves after it. This is how a
  selection's two ends behave correctly — a left end biases `Right`, a right end biases `Left`, so the selection
  grows/shrinks intuitively as you type at its edges.
- **`buffer_id`** scopes the anchor to its buffer.

**Resolution** (`offset_for_anchor`, `summary_for_anchor`): given a `BufferSnapshot`, Zed looks up the
insertion by `timestamp` in an `insertions: SumTree<InsertionFragment>` index → gets the **`Locator`** (a dense
fractional index, `[public concept]`, `crates/text/src/locator.rs`) of the current fragment → seeks the
`fragments: SumTree<Fragment>` tree to that Locator → adds the intra-fragment `offset` → out comes the current
numeric offset (or any `TextDimension`: `Point`, `PointUtf16`, …). Because the anchor references *identity*
(timestamp + Locator), not a number, it resolves correctly in **any** snapshot that has observed its timestamp.

**Version resolution** is explicit: `offset_for_anchor` asserts `snapshot.version.observed(anchor.timestamp())`.
An anchor is only meaningful in a snapshot new enough to contain the edit it points into; `Anchor::is_valid`
reports whether it still lands in visible (non-deleted) text.

### 3.3 The CRDT substrate anchors ride on `[Zed-derived / GPL]`

Anchors work because Zed's `Buffer` is a CRDT, not a string. `crates/text/src/text.rs`:

- **`BufferSnapshot`** holds `visible_text: Rope`, `deleted_text: Rope` (deletes are **tombstoned**, not
  dropped — required so a concurrent op referencing deleted text still resolves), a `fragments:
  SumTree<Fragment>`, an `insertions: SumTree<InsertionFragment>` index, an `undo_map`, and a `version:
  clock::Global`.
- **`Fragment { id: Locator, timestamp: Lamport, insertion_offset, len, visible, deletions, max_undos }`** — a
  contiguous run of text from one insertion. `visible` + `deletions` implement tombstoning; `max_undos` lets
  undo/redo flip visibility.
- Edits don't mutate a string; they **split fragments** and assign new Locators (`push_fragments_for_insertion`).
  A `Locator::between(a, b)` mints a new dense key between two neighbors, so insertion never renumbers existing
  fragments — which is exactly why an existing anchor's identity is stable.

### 3.4 Marley today `[Marley-original]`

`crates/editor/src/selection.rs`. Marley has **no anchors**. A `Selection { anchor: CharOffset, head:
CharOffset }` stores two **raw `CharOffset`s** — the field is *named* "anchor" but it is a plain integer that
goes stale on any edit. `set_selection` merely clamps into `[0, len_chars]`. There is exactly one selection
(`SelectionSet` holds a single member). Every edit returns a `BufferDelta` (char + byte ranges, new lengths)
but nothing consumes it to remap held positions.

### 3.5 The gap

Without anchors, Marley cannot correctly build **any** feature that holds a position across an edit: async LSP,
multi-cursor, marks, stable diagnostics/decorations, or human+agent co-editing. This is a **hard prerequisite**,
not a nice-to-have.

### 3.6 Reimplementation — two tiers, and which to pick

**Tier A — a delta-log anchor layer `[Marley-original]`, built from the `[public]` bias concept. RECOMMENDED
FIRST.** Marley is single-writer today (one human, plus an agent whose writes are serialized through the same
buffer). Full CRDT identity is overkill for that. A pragmatic anchor:

```
Anchor { version: BufferVersion, offset: CharOffset, bias: Bias }
```

Resolve by **replaying accumulated `BufferDelta`s** from the anchor's `version` to the buffer's current version
— for each intervening edit, shift the offset if the edit was strictly before it, clamp if the edit deleted
across it, and use `bias` to break the tie when an insert lands exactly on it. Marley already emits a
`BufferDelta` per edit; keep a bounded ring of them (mirrors Zed's `text/patch.rs`, which composes edit deltas —
but *without* the CRDT). This delivers ~90% of anchors' value (stable positions + version resolution for
single-writer and async edits) at a fraction of the cost, and stays **clean of GPL**. `bias` and "resolve a
position through an edit" are public concepts; the delta-log is Marley's own.

**Tier B — the full Zed-style CRDT `[Zed-derived / GPL]`. DEFER.** Lamport-stamped insertions + `Locator`
fragments + tombstoned deletes. Adopt this only when Marley needs **true multi-writer concurrency** — live
collaboration, or a human and agent editing the *same region* with automatic convergence/conflict resolution
rather than serialized turns. It is heavier and GPL, so it must live in the editor layer and be reimplemented
from the CRDT literature, not copied. When this lands it *subsumes* Tier A (anchors become CRDT anchors) and
also solves §6 concurrency and half of §4 (it can carry the UTF-16 dimension in its fragment summary).

### 3.7 Sequencing

**Anchors come BEFORE LSP and BEFORE multi-cursor** — both are unbuildable-correctly without them:

1. Add the Tier-A anchor layer to `marley_editor` (new `anchor.rs`: `Anchor` + a `Patch`/delta-log +
   `resolve`). Convert `Selection` to hold two `Anchor`s instead of two `CharOffset`s.
2. Only then: multi-cursor (`SelectionSet` becomes a genuine multi-member set whose members survive each
   other's edits) and LSP (§4).
3. Revisit Tier B (CRDT) if/when real concurrent multi-writer editing is on the roadmap.

---

## 4. Coordinate systems — byte / char / UTF-16 / Point, and why LSP forces UTF-16

### 4.1 The four coordinate systems and who demands each

| Coordinate | What it counts | Who needs it |
|---|---|---|
| **byte** (`usize`) | UTF-8 bytes | **tree-sitter** (parses bytes), file IO, rendering shaping |
| **char** (`CharOffset`) | Unicode scalar values | Marley's internal editing currency |
| **UTF-16** (`OffsetUtf16`) | UTF-16 code units | **LSP** (the protocol's default position encoding) |
| **Point / PointUtf16** | (row, column) | line-oriented APIs, viewport/layout, LSP `Position` |

**Why LSP needs UTF-16 specifically:** the Language Server Protocol defines a `Position` as `{ line, character }`
where `character` is, by default, an **offset in UTF-16 code units** from the line start (a historical
consequence of the protocol's VS Code origins). Servers *may* advertise `utf-8` or `utf-32` via the
`positionEncoding` capability, but a client must support **UTF-16** as the baseline. So the moment Marley talks
to any language server, a char-offset (or byte-offset) buffer must convert its positions to/from UTF-16 code
units for every request and every response — a `'é'` is 1 char / 2 bytes / **1** UTF-16 unit, a `'😀'` is 1
char / 4 bytes / **2** UTF-16 units, so none of the three coincide on non-ASCII text. Getting this wrong
misplaces every diagnostic, completion, and hover on any line with an emoji or accent. `[public: LSP spec]`

### 4.2 Zed's approach `[Zed-derived shape / universal coordinates]`

Zed carries **all four** in `TextSummary` (§2.2), so every conversion is one `O(log n)` seek and no coordinate
is privileged. `ToOffset`, `ToPoint`, `ToOffsetUtf16`, `ToPointUtf16` traits let any API accept any coordinate
and convert on demand; `unclipped.rs` distinguishes a raw (possibly mid-code-point) UTF-16 offset from a
clipped-to-a-boundary one — an LSP-driven robustness detail (a server can send an offset that lands inside a
surrogate pair).

### 4.3 Marley today `[Marley-original]`

`marley_text_offsets` defines exactly **two** newtypes: `CharOffset` and `ByteOffset` (each a private `usize`;
`CharCounter` is a forward-only byte→char converter). `Buffer` exposes `char_to_byte`/`byte_to_char` and a
`Point { row, column }` whose **column is bytes**. **There is no UTF-16 coordinate anywhere**, and no
`PointUtf16`.

### 4.4 The gap

No UTF-16 → **Marley cannot speak LSP correctly.** Any position exchanged with a language server on a line with
non-ASCII text would be wrong. `Point.column` being bytes is fine for tree-sitter/rendering but is a third
distinct thing from the char-column that caret math uses and the UTF-16-column LSP wants.

### 4.5 Reimplementation & sequencing

**This is cheaper than it looks, because ropey already does the hard part.** Verified: **ropey 1.6.1 (MIT)
ships `len_utf16_cu`, `char_to_utf16_cu`, and `utf16_cu_to_char`** on both `Rope` and `RopeSlice`. So the
bridge is *additive*, not a rope replacement:

1. Add a `Utf16Offset` newtype (and optionally `PointUtf16 { row, column_utf16 }`) to `marley_text_offsets`,
   mirroring the existing `CharOffset`/`ByteOffset` pattern.
2. Add `Buffer::char_to_utf16`/`utf16_to_char` (and point variants) as thin wrappers over ropey's methods —
   `O(log n)`, no new data structure.
3. Do a `clip`-to-boundary helper for inbound LSP offsets (mirrors Zed's `unclipped`), so a server offset that
   lands mid-surrogate is snapped to a valid char boundary rather than panicking.

**Sequence: land the UTF-16 bridge WITH the start of LSP work — not before (no consumer needs it until then),
and not after (LSP is wrong without it).** It is independent of anchors and can be built in parallel, but
**anchors are still the earlier prerequisite** because an LSP response is async and must be re-anchored to the
edited buffer (§3) *in addition* to being UTF-16-decoded.

---

## 5. Transactions & undo grouping

### 5.1 Zed's model `[Zed-derived / GPL; the "group by pause" idea is public]`

`crates/text/src/text.rs` — `History`, `Transaction`, `HistoryEntry`:

- **Undo is operation-based, not string-based.** An undo is itself an `Operation::Undo(UndoOperation { counts:
  HashMap<Lamport, u32> })` — it records *which insertion timestamps to flip how many times*, and it carries its
  own Lamport timestamp and `Global` version. Consequence: **undo is a CRDT op that can be synced** to other
  replicas exactly like an edit, and undo/redo is just toggling fragment visibility via the `undo_map` — it
  never re-derives text by string surgery.
- **Transactions group edits into one undo step.** `start_transaction` / `end_transaction` are **nestable**
  (a `transaction_depth` counter; only the outermost creates a `HistoryEntry`). A find-replace-all, or a
  multi-cursor edit at N carets, is *one* transaction → *one* ⌘Z. An empty transaction is discarded on `end`.
- **Time-based grouping.** `HistoryEntry` stamps `first_edit_at` / `last_edit_at`; `group()` merges adjacent
  entries whose gap is under a `group_interval` (**300ms** in production, `ZERO` in tests). So a fast typed run
  coalesces, but a pause starts a fresh undo step. `finalize_last_transaction` sets `suppress_grouping` to pin a
  boundary (e.g. don't merge across a save); a new transaction clears the redo stack.

### 5.2 Marley today `[Marley-original]`

`crates/editor/src/undo.rs` (M15 #253) — a **two-`Vec<EditRecord>` invert-stack** (`undone` + `redone`). Each
`EditRecord { at, removed, inserted, origin }` is a string-level inverse. `record` **coalesces** a contiguous
single-char insert into the previous run *of the same `EditOrigin`* (so one ⌘Z undoes a typed word), and clears
the redo stack. `undo`/`redo` re-apply through `apply_raw` (no re-recording, so the stack drains rather than
loops). It is simple, correct, and well-tested.

### 5.3 The gap

- **No transaction concept.** Marley cannot group a *multi-cursor* edit (N carets) or a *find-replace-all* into
  one undo step — each sub-edit would be its own record. This becomes a real defect the moment multi-cursor
  (§3.7) lands.
- **Grouping is contiguity-based, not time-based.** A run is coalesced only while chars are strictly adjacent
  and same-origin; there is no "pause ends the group" and no way to pin a boundary across a save.
- **Undo is string-based, not operation-based.** Fine for single-writer, but it cannot be synced and does not
  compose with anchors/CRDT if Tier B (§3.6) ever lands.

### 5.4 Reimplementation & sequencing

- **Add a `Transaction` wrapper `[Marley-original]`** around the existing delta-log: `begin()/commit()` collect
  every `BufferDelta` in the span into one undo entry (nestable via a depth counter, exactly Zed's shape but over
  Marley's `EditRecord`s). This is a small, GPL-free change and is **required by multi-cursor** — sequence it
  **with §3.7 step 2**.
- **Add time-based grouping** (a `last_edit_at` + a group interval) to complement the existing contiguity rule —
  low priority, a UX refinement, do it opportunistically.
- **Operation-based undo** is a **Tier-B concern** — only worthwhile alongside the full CRDT (§3.6 B) / real
  concurrency. Defer with it.

---

## 6. Concurrency & the clock/version model

### 6.1 Zed's model `[Zed-derived rendition / public concept]`

`crates/clock/src/clock.rs` + the apply path in `text.rs`:

- **`Lamport { value: Seq, replica_id: ReplicaId }`** `[public: Lamport timestamps]` — a logical clock;
  `tick()` bumps it on a local event, `observe()` advances it past a seen remote event. Ties between concurrent
  events break by `replica_id`, giving a total order. `ReplicaId` reserves well-known ids: `LOCAL`,
  `REMOTE_SERVER`, **`AGENT`**, `LOCAL_BRANCH`, then collaborators — note Zed already models an **agent** as a
  first-class replica.
- **`Global`** `[public: version vector]` — a version vector (`SmallVec<[u32; 4]>`, one seq per replica).
  `observe`/`join`/`meet`/`observed`/`changed_since` are the standard version-vector algebra. A `BufferSnapshot`
  *is* identified by its `Global` version.
- **Every mutation is an `Operation`** (`Edit` or `Undo`) carrying a Lamport `timestamp` and the `Global`
  `version` it was based on. `apply_ops` checks `can_apply_op` (are all the op's causal dependencies observed?);
  if not, the op waits in a **`deferred_ops` queue** and is retried when its dependencies arrive
  (`flush_deferred_ops`). This is what lets edits arrive **out of order** (network, async) and still converge to
  the same buffer on every replica.
- **Async waiters**: `wait_for_version` / `wait_for_edits` / `wait_for_anchors` return futures that resolve once
  the buffer has observed a given version/edit/anchor — the primitive an async LSP or collab flow awaits before
  resolving a stale position.

### 6.2 Marley today `[Marley-original]`

`BufferVersion(u64)` — a single monotonic counter, `+1` per edit, `initial() == 0`. One writer, synchronous
edits, no replica identity, no version vector, no operation log, no deferred/reorder handling.

### 6.3 The gap

- Adequate for the current model (one human; an agent whose writes are **serialized** through the same buffer
  via `EditOrigin::Agent`).
- **Async LSP** breaks it *partially even in single-writer*: a response computed against version *v* arrives
  after the buffer has advanced to *v+k*, so its positions must be rebased. Marley's `u64` version is enough to
  *detect* staleness (compare versions); rebasing then needs the **anchor delta-log (§3.6 A)** — which is the
  single-writer answer.
- **True multi-writer** (live collab, or human+agent editing the same region concurrently) needs replica
  identity + a version vector + operation reordering — none of which Marley has.

### 6.4 Reimplementation & sequencing

- **Single-writer + async (LSP, agent turns): no clock upgrade needed.** Keep `BufferVersion` as the staleness
  detector; **stamp anchors and edits with it** and rebase async results through the §3.6-A delta-log. This is
  already covered by the anchor work — **no separate step.**
- **Multi-writer: adopt Lamport + `Global` + a deferred-op queue** `[reimplement from public distributed-
  systems CS; Zed's rendition is GPL]`. This is the **same milestone as Tier-B CRDT anchors (§3.6 B)** — they are
  one body of work (the CRDT *is* the concurrency model). **Defer together, and reimplement from the Lamport/
  version-vector literature, not from `clock.rs`.**

---

## 7. The consolidated sequencing plan

The task's ordering constraint — **anchors before LSP and multi-cursor** — falls out of the dependencies above.
The recommended order for `marley_editor`:

| # | Capability | Provenance | Depends on | Why here |
|---|---|---|---|---|
| 1 | **Anchor layer (Tier A: delta-log)** — `Anchor{version,offset,bias}` + `Patch` + `resolve`; `Selection` holds anchors | `[Marley-original]` from `[public]` bias | delta-log (exists) | **Prerequisite for everything below.** Stable positions across edits, single-writer. |
| 2 | **Transactions** — `begin/commit` grouping over the delta-log | `[Marley-original]` | (1) | Needed so multi-cursor / replace-all is one undo step. |
| 3 | **Multi-cursor** — real multi-member `SelectionSet`, members survive each other's edits | `[Marley-original]` | (1),(2) | Each caret is an anchor; edits re-anchor the rest. |
| 4 | **UTF-16 bridge** — `Utf16Offset`/`PointUtf16` + ropey wrappers + clip | `[public: LSP spec]` over ropey `[MIT]` | ropey (exists) | Independent of 1–3; land it *with* LSP. |
| 5 | **LSP integration** — positions round-trip through UTF-16 *and* re-anchor async responses | `[public: LSP spec]` | (1),(4) | Needs both a UTF-16 codec **and** anchors for stale responses. |
| 6 | **Time-based undo grouping** | `[public]` | (2) | UX polish; opportunistic. |
| — | **Tier-B CRDT + Lamport/`Global` clock + operation-based undo + deferred ops** | `[Zed-derived / GPL]` → reimplement from `[public]` CRDT literature | (1) | **DEFER** until true multi-writer concurrency (live collab / concurrent human+agent co-edit). Subsumes anchors, §5 op-undo, §6 clock in one milestone. |
| — | **Own SumTree rope** | `[public]` / Apache `sum_tree` | — | **Optional / last.** Only if a custom summary dimension or the CRDT forces off ropey. |

**One-line rule of thumb:** *ropey stays; anchors (delta-log) come first and unlock LSP + multi-cursor; UTF-16
is a cheap additive bridge for LSP; the full CRDT/clock is a single deferred milestone gated on real multi-
writer editing.*

---

## 8. Provenance & licensing map

| Zed piece | License in tree | Concept status | Marley action |
|---|---|---|---|
| `sum_tree` (the tree, traits, cursor) | **Apache-2.0** | augmented B-tree — `[public]` | Reimplement freely, or vendor the Apache crate — only if needed (§2.5). |
| `rope::Rope` / `Chunk` / `TextSummary` | GPL-3.0 | chunked rope — `[public]`; SIMD-bitmap `Chunk` is `[Zed-derived]` | **Keep ropey (MIT).** Don't port. |
| `Anchor` + `Locator` + `Fragment` CRDT | GPL-3.0 | anchors + dense ordering — `[public]`; Zed's rendition — `[Zed-derived]` | Reimplement from concepts. Tier A now (Marley-original); Tier B deferred. |
| `clock::Lamport` / `Global` | GPL-3.0 | Lamport ts + version vector — `[public]` | Reimplement from CS literature **only** at multi-writer (deferred). |
| `History` / `Transaction` / `Operation::Undo` | GPL-3.0 | time-grouped + op-based undo — mixed | Transactions: Marley-original now. Op-based undo: deferred with CRDT. |
| UTF-16 position semantics | (LSP spec) | `[public]` | Implement over ropey's MIT UTF-16 API. |

**The boundary that protects the business model:** all of the above is **editor-layer** work and may carry GPL-
derived design. The **brain / agent layer must never import or derive from any of it** — it interacts with the
editor only through Marley's own public API (`Buffer::edit(range, replacement, EditOrigin::Agent)`,
`Anchor::resolve`, coordinate conversions), which is `[Marley-original]`. `EditOrigin::Agent` is the existing,
clean seam that keeps agent writes on the far side of that boundary.

---

## 9. Key files

**Zed reference (scratch clone — GPL unless noted; do not copy verbatim):**
- `crates/sum_tree/src/sum_tree.rs`, `.../cursor.rs` — **Apache-2.0** `SumTree`, `Item`/`Summary`/`Dimension`/
  `SeekTarget`/`Bias`.
- `crates/rope/src/rope.rs` — `Rope`, `TextSummary` (bytes/chars/UTF-16/lines), `TextDimension`; `chunk.rs`
  (SIMD-bitmap `Chunk`); `point.rs`, `point_utf16.rs`, `offset_utf16.rs`, `unclipped.rs` — the coordinate types.
- `crates/clock/src/clock.rs` — `Lamport`, `Global`, `ReplicaId` (incl. `AGENT`).
- `crates/text/src/text.rs` — `Buffer`, `BufferSnapshot`, `Fragment`/`InsertionFragment` (CRDT), `History`/
  `Transaction`, `Operation::{Edit,Undo}`, `apply_ops`/`deferred_ops`, `anchor_at`/`summary_for_anchor`.
- `crates/text/src/anchor.rs` — `Anchor { timestamp, offset, bias, buffer_id }`, resolution, `OffsetRangeExt`.
- `crates/text/src/locator.rs` — `Locator` (dense fractional index; `between`).
- `crates/text/src/patch.rs`, `undo_map.rs`, `operation_queue.rs`, `network.rs` — deltas, undo map, deferral.

**Marley baseline (`[Marley-original]`, the code this doc plans to grow):**
- `crates/editor/src/buffer.rs` — `Buffer` (ropey), `edit`/`apply_raw`, `char_to_byte`/`byte_to_char`,
  `point_at`, `undo`/`redo`.
- `crates/editor/src/types.rs` — `EditOrigin{Human,Agent}`, `BufferVersion(u64)`, `BufferDelta`, `EditResult`,
  `Point{row,column=bytes}`, `Rope = ropey::Rope`.
- `crates/editor/src/selection.rs` — `Selection{anchor,head: CharOffset}` (**raw offsets, not anchors**),
  `SelectionSet` (single member).
- `crates/editor/src/undo.rs` — `UndoHistory` (two-`Vec` invert-stack, contiguity coalescing).
- `crates/marley_text_offsets/src/lib.rs` — `CharOffset`, `ByteOffset` (**no UTF-16**), `CharCounter`.

---

## 10. Open questions (for round 2)

1. **Anchor storage granularity.** Tier A stores `Anchor{version,offset,bias}` and rebases through a delta-log.
   How long must the delta-log be retained (bounded ring vs. full history), and what is the policy when an
   anchor's `version` has been evicted — clamp, or force a resolve-at-eviction that "bakes" the anchor forward?
2. **`Point.column` unit.** Marley's `Point.column` is bytes (good for tree-sitter/rendering) but caret math is
   char-based and LSP is UTF-16. Should Marley keep one `Point` (bytes) + convert, or carry `Point`/`PointUtf16`
   explicitly like Zed? Decide when the UTF-16 bridge lands (§4.5).
3. **Agent co-editing model.** Is the agent always **serialized** through the buffer (Tier A suffices forever),
   or is *concurrent* human+agent editing of the same region a real goal (forces Tier B CRDT + clock)? This one
   answer decides whether an entire deferred milestone is ever needed. `ReplicaId::AGENT` shows Zed chose
   concurrent.
4. **LSP `positionEncoding` negotiation.** Will Marley advertise/prefer `utf-8` (cheaper — it already has bytes)
   where servers support it, falling back to UTF-16, or always UTF-16 for simplicity? Affects how much of §4.5
   is on the hot path.
5. **Own SumTree vs. ropey long-term.** Is there a concrete future summary dimension (decoration counts,
   diagnostic counts, fold state folded into the tree) that ropey cannot carry and that would justify a
   Marley/Apache `sum_tree` rope — or does an out-of-tree side index always suffice? Keep ropey until a real
   forcing function appears.
