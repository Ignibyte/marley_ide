# clock

> Per-crate reference (Marley round 2 — Zed granular). Crate dir: `crates/clock`. Zed
> (github.com/zed-industries/zed) is the EDITOR reference for Marley's editing surface; Marley reimplements
> each capability **in its own code**. Zed source is cloned only into session scratch — never committed. This
> doc is Marley's own description of the crate's design and API, not a copy of it.

| | |
|---|---|
| Subsystem | [02 — text / buffer / rope / anchors](../subsystems/02-text-buffer-anchors.md) (§6 concurrency) |
| License | **GPL-3.0-or-later** (`license = "GPL-3.0-or-later"`; per-crate `LICENSE-GPL` symlink) |
| Internal deps | 0 (external only: `serde`, `smallvec`, optional `parking_lot`) |
| Used by | 21 (Zed tree); the foundational consumer is `text` (every edit/undo is Lamport-stamped) |
| Provenance | `[Zed-derived / GPL]` rendition of `[public]` Lamport-timestamp + version-vector CS |

## Purpose

`clock` is a tiny, dependency-free crate (~230 lines of real code) that provides the **logical clock** the whole
editing stack orders itself by. It is *not* a wall-clock time crate for the editor's causality — that is the
whole point. In a system where a human, a remote collaborator, and an **agent** can all edit the same buffer and
messages can arrive out of order, you cannot order events by wall time; you order them by a **Lamport
timestamp** (causal logical clock) and track "what each replica has seen" with a **version vector**. This crate
is those two primitives plus a small `ReplicaId` newtype, and (separately) a trivial `SystemClock` trait for the
places that *do* want real time.

Concretely it answers: *"which of two concurrent edits wins the tie?"* (Lamport total order), *"has this
snapshot observed the edit this anchor points into?"* (`Global::observed`), and *"what are the causal
dependencies this operation must wait for?"* (version comparison). `text::Buffer` stamps every `Operation` with
a `Lamport` and identifies every `BufferSnapshot` by a `Global`; nothing above it (anchors, undo, deferred-op
reordering, collab) works without these two types.

## Key types, modules & public API

### `clock.rs` — the logical clock

- **`ReplicaId(u16)`** `[Zed-derived reservations over a [public] idea]` — a unique id per distributed node.
  Reserves well-known slots: `LOCAL (0)`, `REMOTE_SERVER (1)`, **`AGENT (2)`**, `LOCAL_BRANCH (3)`,
  `FIRST_COLLAB_ID (8)` (anything ≥ 8 is a live collaborator). `is_remote()`, `new(u16)`, `as_u16()`. **Note
  Zed already models an agent as a first-class replica** — the clearest signal that Zed's design anticipates
  *concurrent* (not merely serialized) human+agent editing.
- **`Seq = u32`** — a Lamport sequence number.
- **`Lamport { value: Seq, replica_id: ReplicaId }`** `[public: Lamport timestamps]` — the logical clock.
  - `tick(&mut self) -> Self` — bump on a local event, returning the pre-bump stamp.
  - `observe(&mut self, other)` — advance past a seen remote event (`value = max(self, other) + 1`).
  - `Ord` is `value` **then `replica_id`** — so two genuinely concurrent events (same `value`) get a
    deterministic, total tie-break by replica, giving every replica the same order.
  - `MIN` / `MAX` sentinels (used by `text` for `Anchor::min/max`), `new(replica_id)` (starts at `value = 1`),
    `as_u64()` (packs into a sortable `u64`).
- **`Global`** `[public: version vector]` — a version vector: `SmallVec<[u32; 4]>`, one high-water `Seq` per
  replica (4 inline slots cover all the non-collab reserved ids for free). This is what a `BufferSnapshot` *is*
  identified by. The standard version-vector algebra:
  - `observe(timestamp)` / `join(&other)` — take the pointwise max (merge what two replicas have seen).
  - `meet(&other)` — pointwise min over commonly-observed replicas (the greatest common causal past).
  - `observed(timestamp) -> bool` — has this version seen that Lamport? (the check `Anchor` resolution and
    `can_apply_op` rely on).
  - `observed_all` / `observed_any` / `changed_since(&other)` / `most_recent()` / `get(replica_id)` / `iter()`.
  - hand-written `Clone` (a `from_slice` copy, faster than `SmallVec::clone` for `Copy` `u32`s) — a micro-opt
    worth noting only because it recurs in `Locator` too.

### `system_clock.rs` — real time (orthogonal)

- **`trait SystemClock: Send + Sync { fn utc_now(&self) -> Instant }`**, **`RealSystemClock`** (delegates to
  `Instant::now()`), and a test-only **`FakeSystemClock`** (`set_now` / `advance`, behind a `parking_lot`
  mutex for deterministic tests). This is the *wall-clock* seam — deliberately separate from the Lamport logical
  clock above; used where the app wants injectable real time, not causal ordering.

## Depends on (internal)

- **None.** The crate is a leaf: external deps are only `serde` (wire serialization of timestamps/versions),
  `smallvec` (the inline version-vector storage), and optional `parking_lot` (test clock).

## Used by (internal dependents)

21 crates in the Zed tree. The foundational consumer is **`text`** ([text.md](./text.md)) — every `Operation`,
`Anchor`, `Transaction`, and `BufferSnapshot` carries a `Lamport` or a `Global`. From there it is pulled by
`language`, `multi_buffer`, `editor`, `project`, `collab`, `worktree`, and the agent-edit crates
(`agent`, `action_log`) — anywhere buffer versions or replica identity are compared.

## Related crates

- [text](./text.md) — the direct consumer; `Anchor`/`Operation`/`BufferSnapshot` are all built on these types.
- **`sum_tree`** (Apache-2.0) — orthogonal, but note `Global`/`Lamport` are the *keys/summaries* stored in
  several of `text`'s `SumTree`s (`UndoMap`, `OperationQueue`, `insertions`).

---

## Marley mapping & reimplementation on our stack

### Marley baseline `[Marley-original]` (post-M15)

Marley versions a buffer with a single monotonic **`BufferVersion(u64)`** — `initial() == 0`, `+1` per edit.
There is **one writer** (one human; the agent's writes are *serialized* through the same buffer via
`EditOrigin::Agent`), edits are synchronous, and there is **no replica identity, no version vector, and no
operation log**. This is a scalar clock, not a logical one — which is exactly right for the current model.

### Verdict: DEFER the whole crate — it is only needed for concurrent multi-writer editing

`clock` buys nothing until Marley has **more than one concurrent writer to the same region**. For everything on
the current and near roadmap, the `u64` counter is sufficient:

| Scenario | Does Marley need `clock`? |
|---|---|
| Single human editing | **No** — `BufferVersion(u64)` monotonic counter suffices. |
| Agent editing, **serialized** through the buffer (`EditOrigin::Agent`) | **No** — still one logical writer; ordering is program order. |
| **Async LSP** (response computed at version *v*, arrives at *v+k*) | **No** — `u64` *detects* staleness (compare versions); *rebasing* the stale position is the [text.md](./text.md) **delta-log anchor**'s job, not a clock upgrade. |
| **True multi-writer**: live collab, or human + agent editing the **same region** concurrently with automatic convergence | **Yes** — this is the only scenario that forces Lamport + `Global` + a deferred-op queue. |

So the single product question — *is the agent always serialized, or is concurrent same-region co-editing a real
goal?* — decides whether this crate is ever reimplemented. `ReplicaId::AGENT` shows **Zed chose concurrent**;
Marley has not committed either way. Until it does, adding a logical clock is speculative complexity.

### If/when it lands: reimplement from the `[public]` literature

When true concurrency is on the roadmap, build (not copy) from the public CS:

1. **`ReplicaId`** — a small newtype with reserved slots mirroring Marley's actors (`Human`, `Agent`,
   collaborators). The *idea* is public; pick Marley's own reservations.
2. **`Lamport { value, replica_id }`** — `tick`/`observe`, `Ord = value then replica_id`. Textbook Lamport
   timestamps (Lamport 1978). Replace `BufferVersion(u64)` stamps on edits with `Lamport`.
3. **`Global` version vector** — `observe`/`join`/`meet`/`observed`, `SmallVec` inline storage. Textbook
   version vectors. A `BufferSnapshot` becomes identified by a `Global` instead of a `u64`.
4. Pair it with the **Tier-B CRDT** in [text.md](./text.md) — the clock and the CRDT are **one milestone**; the
   version vector is meaningless without the fragment CRDT it orders, and vice-versa.

`SystemClock` (real wall time) is a separate, trivial concern Marley can add independently whenever it wants an
injectable time source for the *app* (not for causal ordering); it carries none of the deferral logic above.

## Provenance & the GPL boundary

- **The crate is `GPL-3.0-or-later`**, but its content is a thin rendition of **`[public / permissive]`**
  distributed-systems CS: **Lamport logical clocks** and **version vectors** are decades-old public literature;
  Marley may implement them freely. What is **`[Zed-derived / GPL]`** is only the *specific rendition* — the
  exact packed field layout, the `ReplicaId` slot reservations, the `join`/`meet` implementations. Do not copy
  the code; reimplement from the CS.
- **`[Marley-original]`**: today's `BufferVersion(u64)`, and (later) Marley's own `ReplicaId` reservations and
  clock module built from the literature.
- **Boundary note:** if this crate is ever reimplemented it is **editor-layer** work and part of the deferred
  CRDT milestone. The brain/agent layer must never depend on it directly — it interacts with the buffer only
  through Marley's own public API, which is `[Marley-original]`. (That said, because Lamport clocks are public
  CS, a clean-room clock module is one of the *lower*-risk things to share across layers — the copyleft risk
  lives in the CRDT it serves, not in the clock itself.)

## Notes / gotchas

- **Lamport ≠ wall time.** `Lamport` orders *causality*, not chronology; two edits one hour apart on different
  replicas can be "concurrent." `SystemClock` is the separate wall-clock seam. Conflating them is the classic
  distributed-systems bug — keep them distinct if Marley ever adds the logical clock.
- **The `Ord` tie-break by `replica_id` is load-bearing.** It is what makes concurrent edits deterministically
  orderable *the same way on every replica* → convergence. Any Marley reimplementation must preserve it.
- **`Global` uses `SmallVec<[u32; 4]>`** — 4 inline slots chosen so the reserved non-collab ids
  (`LOCAL`/`REMOTE_SERVER`/`AGENT`/`LOCAL_BRANCH`) never heap-allocate. A sizing choice worth copying if the
  crate is ever built.
- **`observe` bumps to `max + 1`, `tick` returns pre-bump.** Off-by-one care: `tick` yields the stamp *for* the
  current event then advances; `observe` advances *past* a seen event. Easy to get backwards in a
  reimplementation.
