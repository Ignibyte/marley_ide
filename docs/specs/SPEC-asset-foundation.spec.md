---
spec_id: asset-foundation
component: marley_asset_core
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Synchronous asset cache foundation (never blocks a frame)
goal: Give every Marley UI surface a synchronous "ask for an asset, get a Loading/Loaded/Failed state back this instant" cache whose actual resolve+decode happens off the render thread, so a panel can request an image, font, or icon during paint and never stall a frame on network, disk, or decode work.
reuses: [futures, tokio]
spec_source: "behavior-level contract — a synchronous asset cache: a caller hands in a description of where an asset lives (a build-time bundled path, or an asynchronously-fetched source tagged by a kind marker) and receives, on the same call and without blocking, one of three states — still-loading, loaded-with-value, or failed-with-a-reason. The real fetch/read and the decode-from-bytes run on a background scheduler, never on the calling thread; repeated requests for the same described source are de-duplicated and served from one cache entry, and two sources that describe the same locator under different kind markers are kept apart. Observable I/O only; no fork module/type/static names, no private taxonomy transcribed. Foundation seam and ownership per standards/seam-contracts.md §10.1."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_asset_core` is the **synchronous asset-cache foundation** that every Marley UI crate renders against. A consumer calls `AssetCache::get::<T>(source)` during a paint and gets an `AssetState<T>` back **on the same call** — `Loading`, `Loaded(value)`, or `Failed(reason)` — never blocking the render thread on network, disk, or decode work. The actual byte resolution (a build-time **bundled** asset read through an embedder seam, or an **async** fetch tagged by a kind marker) and the `Asset::decode` step run on an injected off-render-thread scheduler; the cache de-duplicates repeated requests for the same described source into one entry and keeps two sources that name the same locator under **different** markers strictly apart.

Per **seam-contracts §10.1 (binding)** this crate is the **sole owner** of the asset foundation types `Asset`, `AsyncAssetType`, `AssetSource`, `AssetState`, `AssetError`, and `AssetCache`. It is an explicit **upstream dependency of `marley_assets`** and is **built before it in the M1 order**, so M1 is no longer gated by a crate whose base types are unspecified (review-r1 Blocker 7, foundation half). `marley_assets` depends on this crate and adds the URL/`data:` async sources, the embed macro, and the three concrete `AsyncAssetType` markers on top; this spec owns none of that — only the synchronous cache contract and its seams.

## Public surface (the contract)
All in `crates/marley_asset_core/src/lib.rs`. Every identifier is Marley-original (clean-room Posture A); no Warp-internal name appears.

```rust
/// A decodable asset kind: owned bytes in, a cloneable decoded value out, decoded off-thread.
pub trait Asset: 'static {
    type Output: Clone + 'static;
    fn decode(bytes: Cow<'static, [u8]>) -> DecodeFuture<Self::Output>;
}
pub type DecodeFuture<O> = Pin<Box<dyn Future<Output = Result<O, AssetError>> + Send + 'static>>;

/// A distinct compile-time marker per async source kind. Its *type identity* keys the cache,
/// so two sources with the same locator but different markers are never interchangeable.
pub trait AsyncAssetType: 'static {}

/// Where an asset lives. Constructed by consumers (e.g. `marley_assets`); never decoded here.
pub enum AssetSource {
    Bundled { path: &'static str },   // bytes resolved at runtime through the Embedder seam
    Async(AsyncSource),               // a marker's TypeId + a locator + a type-erased byte loader
}

/// The async arm of `AssetSource`. Carries the keying identity and the off-thread byte loader.
pub struct AsyncSource { /* private: marker TypeId, locator String, ByteLoader */ }
impl AsyncSource {
    pub fn new<M: AsyncAssetType>(locator: impl Into<String>, loader: ByteLoader) -> Self;
    pub fn marker_id(&self) -> TypeId;     // TypeId::of::<M>() captured at construction
    pub fn locator(&self) -> &str;
}
pub type ByteLoader = Arc<dyn Fn() -> FetchFuture + Send + Sync + 'static>;
pub type FetchFuture = Pin<Box<dyn Future<Output = Result<Cow<'static, [u8]>, AssetError>> + Send + 'static>>;

/// The three observable states a `get` can return. `T = <SomeAsset as Asset>::Output`.
pub enum AssetState<T> { Loading, Loaded(T), Failed(AssetError) }

/// Why a load failed. Exactly these three reasons are selectable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetError { Fetch, Decode, NotFound }

/// Resolves `Bundled { path }` bytes at runtime. Implemented by the consumer over `rust-embed`;
/// kept behind a seam so this crate carries no embedder dependency.
pub trait Embedder: Send + Sync + 'static {
    fn load(&self, path: &str) -> Option<Cow<'static, [u8]>>;
}

/// Runs a load future off the render thread. Injectable so transitions are deterministically
/// testable; the production impl bridges to Tokio.
pub trait LoadScheduler: Send + Sync + 'static {
    fn schedule(&self, task: Pin<Box<dyn Future<Output = ()> + Send + 'static>>);
}
pub struct TokioScheduler(tokio::runtime::Handle);   // schedule() == handle.spawn(task)

/// The synchronous, frame-safe cache. `get` never blocks; resolve+decode happen on the scheduler.
pub struct AssetCache { /* embedder + scheduler + RwLock<HashMap<AssetKey, Entry>> */ }
impl AssetCache {
    pub fn new(embedder: Arc<dyn Embedder>, scheduler: Arc<dyn LoadScheduler>) -> Self;
    pub fn with_tokio(embedder: Arc<dyn Embedder>, handle: tokio::runtime::Handle) -> Self;
    /// Returns the entry's state captured AT CALL TIME (Loading for a fresh source), synchronously,
    /// then schedules resolve+decode off the render thread. Never blocks a frame.
    pub fn get<T: Asset>(&self, source: &AssetSource) -> AssetState<T::Output>;
}
```

The cache identity (internal, behavior-observable through `get` de-duplication) is the pair derived from `AssetSource`:

```rust
// crate-internal — the keying branch the mutation gate guards:
enum AssetKey { Bundled(&'static str), Async { marker: TypeId, locator: String } } // Eq + Hash
```

`AssetState` is generic over the decoded `Output`, not over the `Asset` type, so a consumer reads `AssetState<image::RgbaImage>` etc.

## EARS Requirements
R1. WHEN `get::<T>(source)` is called for a source whose cache key is not yet present, the system shall insert a `Loading` entry, schedule the resolve-and-decode work on the `LoadScheduler`, and return `AssetState::Loading` captured **before** the scheduled work can mutate the entry — performing no network I/O, no disk I/O, and no `Asset::decode` on the calling thread.

R2. WHILE a source's entry is `Loading`, WHEN `get::<T>` is called again for the same cache key, the system shall return `AssetState::Loading` and shall schedule the resolve-and-decode work **exactly once** across those calls (it shall not re-schedule on each call).

R3. WHEN a `Bundled { path }` source is resolved, the system shall obtain its bytes by calling `Embedder::load(path)` and shall pass exactly those bytes to `Asset::decode`.

R4. IF `Embedder::load(path)` returns `None` for a `Bundled` source, THEN the system shall transition that entry to `AssetState::Failed(AssetError::NotFound)` and shall not call `Asset::decode`.

R5. WHEN an `Async(source)` is resolved, the system shall obtain its bytes by invoking the source's `ByteLoader` future, and IF that future yields `Err`, THEN the system shall transition the entry to `AssetState::Failed(AssetError::Fetch)` and shall not call `Asset::decode`.

R6. IF bytes are obtained (bundled or async) but `Asset::decode(bytes)` yields `Err`, THEN the system shall transition that entry to `AssetState::Failed(AssetError::Decode)`.

R7. WHEN the resolve-and-decode work for a source completes successfully with decoded value `v`, the system shall transition that entry from `Loading` to `Loaded(v)`.

R8. WHILE an entry is `Loaded` or `Failed`, WHEN `get::<T>` is called again for the same cache key, the system shall return a clone of that terminal state and shall not schedule any further resolve-and-decode work.

R9. WHEN `get::<T>` returns `Loaded`, the system shall return `AssetState::Loaded` carrying a **clone** of the cached `T::Output`, leaving the cached value in place for subsequent calls.

R10. WHEN two `get` calls supply sources with equal cache identity, the system shall serve both from the **same** cache entry; the cache identity of a `Bundled` source is its `path`, and of an `Async` source is the pair (`AsyncSource::marker_id()`, `AsyncSource::locator()`).

R11. The system shall treat two `Async` sources that share a locator but carry **distinct** `AsyncAssetType` markers as **different** cache entries, such that a value resolved under one marker is never returned to a `get` keyed by the other (the markers stay non-interchangeable).

R12. WHEN any resolve, byte-load, or `Asset::decode` is performed, the system shall perform it only via the injected `LoadScheduler` (off the render thread), such that the calling thread of `get` is never blocked on that work.

R13. The system shall expose exactly three `AssetError` reasons — `Fetch`, `Decode`, `NotFound` — and select among them per R4 (`NotFound`), R5 (`Fetch`), and R6 (`Decode`), with each reason distinguishable by value (`PartialEq`).

R14. WHEN `AssetCache::get` is called concurrently from multiple threads for the same fresh source, the system shall create at most one `Loading` entry and schedule the load at most once (the entry insert is atomic under the cache's interior lock).

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | Fresh `get` returns `Loading` synchronously, schedules off-thread, does zero I/O/decode on the caller (R1) | planned |
| 2 | Repeated `get` while loading: still `Loading`, scheduled exactly once (R2) | planned |
| 3 | `Bundled` bytes come from `Embedder::load(path)` and are the exact bytes decoded (R3) | planned |
| 4 | `Embedder::load` → `None` ⇒ `Failed(NotFound)`, no decode (R4) | planned |
| 5 | Async `ByteLoader` `Err` ⇒ `Failed(Fetch)`, no decode (R5) | planned |
| 6 | `Asset::decode` `Err` ⇒ `Failed(Decode)` (R6) | planned |
| 7 | Successful resolve+decode ⇒ `Loading`→`Loaded(v)` (R7) | planned |
| 8 | Terminal entry returns clone, never re-schedules (R8) | planned |
| 9 | `Loaded` returns a clone; cached value stays (R9) | planned |
| 10 | Equal identity (`Bundled` path / `Async` marker+locator) shares one entry (R10) | planned |
| 11 | Same locator + distinct markers ⇒ separate entries, never cross-served (R11) | planned |
| 12 | All resolve/fetch/decode routed through `LoadScheduler`; caller never blocks (R12) | planned |
| 13 | Exactly `{Fetch, Decode, NotFound}`; reason selected per R4/R5/R6, value-distinct (R13) | planned |
| 14 | Concurrent fresh `get` ⇒ at most one entry, scheduled at most once (R14) | planned |

## Visual / Behavioral Acceptance
N/A — headless cache and decode plumbing; no window, pane, or AXUIElement surface. `browser_testable: no`, no `visual_acceptance` clause, so quality-bar **gate 15 does not apply** to this spec. The first consuming panel that paints a cached asset asserts its own visual clause in its own M1 spec; that assertion is named here only for traceability.

## Test Plan
- **Unit:** one `#[test]` per EARS clause, named for the requirement, run with a deterministic **`ImmediateScheduler`** test double (a `LoadScheduler` that drives the scheduled future to completion synchronously on a current-thread runtime). Because `get` captures and returns the entry state **before** invoking the scheduler (R1), the immediate scheduler makes every transition observable on the *next* `get` without any timing/sleep:
  - `r1_fresh_get_returns_loading_sync_no_caller_io` — a `CountingEmbedder` + a `ByteLoader` wired to a flag assert neither ran on the calling thread before `get` returns, and the return value is `Loading`.
  - `r2_loading_get_reschedules_once` — a scheduler whose `schedule` increments an `AtomicUsize`; two `get`s while loading ⇒ count `== 1`, both return `Loading`.
  - `r3_bundled_decodes_embedder_bytes` — `Embedder::load` returns a known byte fixture; assert the value passed to `Asset::decode` equals it (captured by a recording test `Asset`).
  - `r4_bundled_missing_is_not_found` — embedder returns `None` ⇒ second `get` is `Failed(NotFound)`; the recording `Asset` asserts `decode` was never called.
  - `r5_async_fetch_err_is_fetch` — `ByteLoader` future yields `Err(AssetError::Fetch)`'s source error ⇒ `Failed(Fetch)`, no decode.
  - `r6_decode_err_is_decode` — bytes resolve but the test `Asset::decode` returns `Err` ⇒ `Failed(Decode)`.
  - `r7_success_transitions_to_loaded` — embedder/loader + decode succeed ⇒ second `get` is `Loaded(v)` with the expected `v`.
  - `r8_terminal_does_not_reschedule` — after reaching `Loaded` (and separately `Failed`), further `get`s leave the schedule counter unchanged and return the terminal state.
  - `r9_loaded_returns_clone_value_persists` — a `T::Output` whose `Clone` bumps a counter; two `get`s after load each yield an equal value and the cached entry remains `Loaded` (a third `get` still loads).
  - `r10_equal_identity_shares_entry` — two distinct `AssetSource` values with equal identity (same `Bundled` path; and separately same marker+locator) cause exactly one `ByteLoader`/embedder invocation (counter `== 1`).
  - `r11_distinct_markers_not_interchangeable` — two `AsyncSource`s with the same locator but markers `MarkerA`/`MarkerB` (`struct MarkerA; impl AsyncAssetType for MarkerA {}`, likewise `MarkerB`) ⇒ two `ByteLoader` invocations, two entries; the value loaded under `MarkerA` is never returned for a `get` keyed by `MarkerB`. A direct sibling asserts `AssetKey` of the two differs (`marker` field), and that two same-marker+same-locator keys are `Eq`.
  - `r12_all_work_routed_through_scheduler` — a scheduler that records it was invoked and a flag set only inside the scheduled future prove resolve/fetch/decode ran under `schedule`, not inline; the caller-thread id captured before `get` returns differs from the work-thread id under a real two-thread scheduler variant.
  - `r13_error_reasons_exact_and_distinct` — `AssetError` has exactly the three variants (compile-time exhaustiveness in the test `match`), and `Fetch != Decode != NotFound` by `PartialEq`; the three reasons are produced by R4/R5/R6 fixtures respectively.
  - `r14_concurrent_fresh_get_single_entry` — N threads call `get` on the same fresh source through a gate barrier; assert the embedder/loader ran once and the schedule counter is `1` (entry insert atomic under the cache lock).
  - 100% line coverage on every touched line of `lib.rs` except the single ACCEPTED-UNTESTABLE Tokio-bridge line (below).
- **Integration:** `marley_assets` (downstream) supplies the seam test that drives a real `AsyncSource` (its `UrlAssetWithPersistence` marker) through `AssetCache::get` and asserts the `Loading`→`Loaded` transition end-to-end; named here for traceability, owned by `SPEC-assets`. Within this crate, one integration test wires the **`TokioScheduler`** to a multi-thread Tokio runtime and a `127.0.0.1` byte loader to confirm `get` returns `Loading` immediately and the value becomes `Loaded` after the runtime drains — exercising the bridge behaviorally.
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full suite stays green; the foundation type signatures in seam-contracts §10.1 must remain exact (a `static_assertions`-style trait-bound test pins `Asset::Output: Clone + 'static`, `AsyncAssetType: 'static`, and the `AssetState`/`AssetError` variant sets); `marley_assets` continues to compile against this crate's surface unchanged.

## Mutation Targets
`cargo mutants` must kill every viable mutant on the state-transition and cache-key surface:
- **State-transition branches** — swapping the `Loading`→`Loaded` arm for `Loading`→`Failed` (or vice versa), dropping the terminal-state guard in R8 so a `Loaded`/`Failed` entry re-schedules, or returning the captured pre-schedule state where the live entry state is required — killed by R1/R2/R7/R8.
- **`AssetError` selection** — replacing `NotFound`↔`Fetch`↔`Decode` at the three selection sites, or deleting the "no decode after a fetch/not-found failure" early return — killed by R4/R5/R6/R13.
- **Cache-key branches** — collapsing the `AssetKey::{Bundled, Async}` arms, dropping the `marker: TypeId` from the `Async` key (which would make distinct markers collide), or dropping the `locator` from the key — killed by R10/R11.
- **De-dup / schedule-once guard** — flipping the "entry absent?" condition so the load schedules on every `get`, or the R14 atomic insert race window — killed by R2/R8/R14.
- **Clone-on-read (R9)** — returning the stored value by move/take instead of clone (would leave the entry empty) — killed by R9's value-persists assertion.
- **MSI target: 100%** on the testable surface. **ACCEPTED-UNTESTABLE (one line):** `TokioScheduler::schedule`'s `self.0.spawn(task)` — the off-render-thread/Tokio bridge — has no deterministic unit harness (it hands the future to a live multi-thread runtime); it is exercised behaviorally by the Tokio integration test and is the single declared carve-out. Every transition and keying branch is testable through the injected `LoadScheduler`/`Embedder` seams, so nothing else is excluded. This is a closed decision, recorded in Forge RLM, not a pending gap.

## Dependencies
- REUSE (permissive, MIT/Apache): `futures` (`BoxFuture`/`FutureExt` for the boxed `DecodeFuture`/`FetchFuture` and the immediate-drive test double), `tokio` (runtime `Handle` for the default `TokioScheduler` off-thread bridge only). Both pass `cargo deny`'s MIT/Apache allowlist.
- Marley components: **none upstream** — this is the M1 leaf foundation for the asset stack. It is the **upstream dependency of `marley_assets`** (seam-contracts §10.1) and is built **before** `marley_assets` in the M1 order, closing review-r1 Blocker 7 (foundation half) so M1 is no longer gated by a crate whose base types are unspecified.

## Out of scope / deferred
- **Concrete async sources** — URL fetch (`url_source`/`url_source_with_persistence`), `data:`-URI decode, content-addressed disk persistence, and the three concrete `AsyncAssetType` markers (`UrlAssetWithoutPersistence`, `UrlAssetWithPersistence`, `DataUriAsset`) — owned by `SPEC-assets` (`marley_assets`), built on this foundation.
- **The `bundled_asset!` embed macro and the `rust-embed` binding** — owned by `marley_assets`/`marley_asset_macro`; this crate defines only the `Embedder` seam the consumer implements, carrying no `rust-embed` dependency.
- **The remote-CDN macro path** (`remote_asset!`, `bundled_or_fetched_asset!`, `make_absolute_url`) — dropped at M1 per seam-contracts §10.2; not reachable from this foundation.
- **Cache eviction / capacity / TTL** — the M1 cache is grow-only by source identity with manual lifetime (the cache lives as long as the app shell); an eviction policy is a later spec.
- **WASM-target scheduling** — out of scope for M1 (native cockpit only); the `TokioScheduler` is the native bridge.

## Clean-room provenance
Behavior-derived from a fork-reference doc (the observable I/O of a synchronous asset cache: ask-for-a-source → get a `Loading`/`Loaded`/`Failed` state on the same call, with resolve+decode off the render thread and identity-keyed de-duplication) — no AGPL/fork source read, no private module/type/static names reproduced, IP-counsel sign-off pending (open item in `clean-build-plan.md`). Seam ownership conforms to `standards/seam-contracts.md` §10.1: `marley_asset_core` owns `Asset`, `AsyncAssetType`, `AssetSource`, `AssetState`, `AssetError`, and `AssetCache`, and is the upstream dependency of `marley_assets` built before it. Public-surface identifiers are Marley-original (clean-room Posture A): the `Embedder` and `LoadScheduler` seams, the `AsyncSource` keying type, and the `AssetKey` identity are our own names describing observable behavior, not a transcription of any private taxonomy. REUSE crates (`futures`, `tokio`) are MIT/Apache. The package is named `marley_asset_core`; no Warp source was consulted.
