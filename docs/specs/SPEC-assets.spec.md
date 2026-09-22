---
spec_id: assets
component: marley_assets (+ marley_asset_macro proc-macro)
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Async asset sources + compile-time bundled-embed macro
goal: Let Marley's UI reference images/fonts/icons by relative path (verified at build time and embedded) or by URL / data-URI (fetched off the render thread, optionally disk-cached) without ever blocking a frame or shipping a dangling asset reference.
reuses: [rust-embed, reqwest, sha2, base64, syn, quote, proc-macro2]
spec_source: "behavior-only — observable I/O of an async asset layer on a synchronous cache: reference a UI image/font/icon either by a relative path that is verified and embedded at build time, or by an HTTP URL / inline data:-URI that is fetched and decoded off the render thread (optionally persisted to a content-addressed cache file); every load returns a Loading/Loaded/Failed state synchronously and never blocks a frame on network or disk I/O. No fork file paths, no private module/type/static names. Foundation seam and scope per standards/seam-contracts.md §10."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_assets` adds **URL- and `data:`-URI-backed async asset sources** on top of the synchronous `AssetCache` foundation owned by `marley_asset_core` (seam-contracts §10.1), so panels can render remote thumbnails, avatars, and inline base64 images without blocking the render loop. It fetches remote bytes over HTTP (optionally persisting them to a content-addressed file in a cache dir for future hits) and decodes inline `data:` URIs under a hard size cap. Its companion proc-macro crate `marley_asset_macro` resolves **compile-time bundled asset references** into `AssetSource::Bundled` values: it verifies the referenced file exists under the assets directory at build time — so a moved or missing asset is a hard `compile_error!`, never a runtime surprise — and the bundled bytes are carried at runtime by the `rust-embed` embedder. Per seam-contracts §10.2 the M1 surface ships **only** the `bundled_asset!` embed macro plus `data:`-URI / `url_source` sources; the remote-CDN macro path (content-hashed fetch URLs, wasm bundled/remote splits) is out of scope and is **not** present.

## Public surface (the contract)
**Crate `marley_assets`** (`crates/marley_assets/src/lib.rs`) — **depends on `marley_asset_core`** for the foundation types `Asset`, `AssetCache`, `AssetSource`, `AssetState`, and `AsyncAssetType` (seam-contracts §10.1; `marley_asset_core` is built before `marley_assets` in the M1 order). This crate adds the async sources, the cache extension, and the distinct async markers:

- `pub trait AssetCacheExt` (impl'd for `marley_asset_core::AssetCache`) — `fn load_asset_from_url<T: Asset>(&self, url: &str, cache_dir: Option<&Path>) -> AssetState<T::Output>`. Primary entry point: persists when `cache_dir` is `Some`, memory-only when `None`.
- `pub fn url_source(url: &str) -> AssetSource` — async fetch to memory, no persistence (marker `UrlAssetWithoutPersistence`).
- `pub fn url_source_with_persistence(url: &str, cache_dir: &Path) -> AssetSource` — async fetch + write to a content-addressed file (marker `UrlAssetWithPersistence`).
- `pub fn data_uri_source(source: &str) -> Option<AssetSource>` — decode `data:[<mediatype>];base64,<payload>` (marker `DataUriAsset`); strips embedded whitespace; returns `None` for non-`data:`/malformed/oversize input.
- `pub const MAX_DATA_URI_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;`
- `pub fn data_uri_exceeds_limit(source: &str) -> bool` — true iff the base64 payload of `source` would decode to more than the cap, computed from the base64 text length without allocating the decoded buffer.
- Marker types implementing `marley_asset_core::AsyncAssetType`: `pub struct UrlAssetWithoutPersistence;`, `pub struct UrlAssetWithPersistence;`, `pub struct DataUriAsset;` — kept distinct.
- Internal: `cache_file_name(url) -> String` (lowercase-hex SHA-256 digest of the URL), `persist_bytes`, `fetch_asset(url, cache_dir)`.

**Crate `marley_asset_macro`** (`crates/marley_asset_macro/src/lib.rs`, `proc-macro = true`):

- `bundled_asset!("path"[, "folder"])` — `#[proc_macro]`; verifies the file exists under `app/assets/<folder>` (default `bundled`); expands to `::marley_assets::AssetSource::Bundled { path }`. This is the **only** macro M1 ships.

## EARS Requirements
R1. WHEN `url_source(url)` is called, the system shall return an `AssetSource::Async` carrying the `UrlAssetWithoutPersistence` marker and the given `url`, with no cache directory recorded.

R2. WHEN `url_source_with_persistence(url, cache_dir)` is called, the system shall return an `AssetSource::Async` carrying the `UrlAssetWithPersistence` marker, the given `url`, and the given `cache_dir`.

R3. WHEN `data_uri_source(source)` is called with `source` matching `data:[<mediatype>];base64,<payload>`, the system shall return `Some(AssetSource::Async)` carrying the `DataUriAsset` marker whose bytes equal the base64-decoded payload.

R4. IF `data_uri_source(source)` is called with `source` that does not start with `data:` or lacks the `;base64,` separator or whose payload is not valid base64, THEN the system shall return `None`.

R5. WHEN `data_uri_source(source)` decodes a payload, the system shall strip ASCII whitespace embedded in the base64 text before decoding.

R6. IF `data_uri_exceeds_limit(source)` is true, THEN `data_uri_source(source)` shall return `None` and shall not decode the payload.

R7. WHEN `data_uri_exceeds_limit(source)` is called, the system shall return `true` iff the base64 payload in `source` would decode to more than `MAX_DATA_URI_PAYLOAD_BYTES` bytes, computing the decoded length from the base64 text length and shall not allocate the decoded buffer.

R8. The system shall define `MAX_DATA_URI_PAYLOAD_BYTES` equal to `16 * 1024 * 1024`.

R9. WHEN `load_asset_from_url(url, None)` is called, the system shall fetch into memory only and shall not create any file on disk.

R10. WHEN `load_asset_from_url(url, Some(dir))` resolves bytes for `url`, the system shall persist those bytes to a file named `cache_file_name(url)` inside `dir`.

R11. WHEN `cache_file_name(url)` is computed, the system shall return the **lowercase-hexadecimal SHA-256 digest of `url`**, never base64, so the name is safe on case-insensitive filesystems.

R12. WHILE a non-empty file named `cache_file_name(url)` exists in the cache dir, WHEN that persisted URL asset is loaded, the system shall read the bytes from that file and shall not issue a network request.

R13. IF the cache file for `url` exists but is zero bytes, THEN the system shall treat it as a miss and re-fetch over the network.

R14. The system shall define `UrlAssetWithoutPersistence`, `UrlAssetWithPersistence`, and `DataUriAsset` as three distinct `AsyncAssetType` markers, such that a URL previously loaded without persistence is still fetched and persisted when later loaded via `url_source_with_persistence`.

R15. WHEN `load_asset_from_url` is called, the system shall return synchronously with `AssetState::Loading` and perform the fetch/decode off the calling render thread, never blocking the caller on network or disk I/O.

R16. WHEN `bundled_asset!("path")` is invoked and the file exists under `app/assets/bundled`, the system shall expand to `::marley_assets::AssetSource::Bundled { path: "path" }`.

R17. WHEN `bundled_asset!("path", "folder")` is invoked, the system shall resolve and verify the file under `app/assets/<folder>` instead of the default `bundled` folder.

R18. IF the file referenced by `bundled_asset!` does not exist at build time, THEN the macro shall expand to `compile_error!` naming the unresolved asset path **relative to `CARGO_MANIFEST_DIR`** (a deterministic, machine-independent path).

R19. The system shall resolve every `bundled_asset!` asset path relative to the consumer's `CARGO_MANIFEST_DIR`, not the process working directory.

R20. IF `bundled_asset!` is invoked with anything other than one or two string-literal arguments, THEN the macro shall fail to compile with a parse error.

R21. WHERE an `AssetSource::Bundled { path }` is resolved at runtime, the system shall return the bytes embedded by the `rust-embed` embedder for that relative `path`.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `url_source` → `Async`/`UrlAssetWithoutPersistence`, no cache dir (R1) | planned |
| 2 | `url_source_with_persistence` → `Async`/`UrlAssetWithPersistence` + dir (R2) | planned |
| 3 | Valid `data:` URI → `Some` with decoded bytes (R3) | planned |
| 4 | Non-`data:`/malformed/invalid-base64 → `None` (R4) | planned |
| 5 | Embedded whitespace stripped before decode (R5) | planned |
| 6 | Oversize `data:` rejected (`None`) without decoding (R6) | planned |
| 7 | `data_uri_exceeds_limit` exact at the boundary, no decoded-buffer alloc (R7) | planned |
| 8 | `MAX_DATA_URI_PAYLOAD_BYTES == 16 MiB` (R8) | planned |
| 9 | `cache_dir: None` writes no file (R9) | planned |
| 10 | `cache_dir: Some` persists to hashed file (R10) | planned |
| 11 | Cache file name is lowercase-hex SHA-256 of url, not base64 (R11) | planned |
| 12 | Non-empty cache file → read from disk, no network (R12) | planned |
| 13 | Empty cache file → miss, re-fetch (R13) | planned |
| 14 | Three distinct markers; unpersisted→persisted still persists (R14) | planned |
| 15 | `load_asset_from_url` returns `Loading` sync, I/O off-thread (R15) | planned |
| 16 | `bundled_asset!` default folder → `Bundled { path }` (R16) | planned |
| 17 | `bundled_asset!` honors explicit folder arg (R17) | planned |
| 18 | Missing file → `compile_error!` with manifest-relative path (R18) | planned |
| 19 | Paths anchored to `CARGO_MANIFEST_DIR` (R19) | planned |
| 20 | Non `1..=2` string-literal args → parse error (R20) | planned |
| 21 | `Bundled { path }` resolves via `rust-embed` bytes (R21) | planned |

## Visual / Behavioral Acceptance
N/A — headless asset plumbing and a build-time embed macro; no window, pane, or AXUIElement surface. The first consuming panel that renders a fetched image asserts its own visual clause in its own M1 spec; that assertion is named here only for traceability.

## Test Plan
- **Unit (`marley_assets`):** one `#[test]` per clause — `r1_url_source_no_persistence`, `r2_url_source_with_persistence`, `r3_data_uri_decodes_payload`, `r4_data_uri_rejects_malformed`, `r5_data_uri_strips_whitespace`, `r6_oversize_data_uri_returns_none`, `r7_exceeds_limit_boundary_no_decode_alloc`, `r8_max_payload_constant`, `r9_none_writes_no_file`, `r10_some_persists_hashed_file`, `r11_cache_name_is_lowercase_hex_sha256`, `r12_nonempty_cache_hits_disk_not_network`, `r13_empty_cache_refetches`, `r14_distinct_markers_unpersisted_then_persisted`, `r15_load_returns_loading_sync`. Network is exercised against a local in-process HTTP stub (`wiremock`/`tiny_http`) bound to `127.0.0.1`; disk against a `tempfile::TempDir`. R11 asserts the name is exactly the lowercase-hex SHA-256 of the url (matched against an independently computed digest, and proven not base64). 100% coverage on every touched line of `lib.rs`.
- **Allocation-counting harness (R6/R7):** `r7_exceeds_limit_boundary_no_decode_alloc` wraps the global allocator in a test-only counting allocator (a small `GlobalAlloc` shim holding an `AtomicUsize` of bytes allocated; no external crate, no `unsafe` beyond the documented `// SAFETY:` delegation to `System`) and asserts `data_uri_exceeds_limit` over an above-cap input performs **zero** decoded-buffer allocation while still returning the correct boundary verdict (exact at `MAX_DATA_URI_PAYLOAD_BYTES`, true one byte over). `r6_oversize_data_uri_returns_none` asserts the same counter sees no decode allocation on the `data_uri_source` reject path.
- **Unit (`marley_asset_macro`):** `trybuild` pass-cases for `r16_bundled_default_folder`, `r17_bundled_explicit_folder`, `r19_manifest_dir_anchored`; `trybuild` compile-fail cases with `.stderr` snapshots for `r18_missing_file_compile_error` and `r20_bad_args_parse_error`. The R18 `.stderr` snapshot embeds **only** the manifest-relative asset path (deterministic across machines); a normalized/substring matcher guards against any incidental absolute-path leakage so the snapshot is not flaky per checkout location.
- **Integration:** `r21_bundled_resolves_via_rust_embed` loads a `Bundled { path }` produced by `bundled_asset!` and asserts the bytes equal the fixture embedded by `rust-embed`; one round-trip seam test feeds a `url_source_with_persistence` result through `AssetCacheExt::load_asset_from_url` twice and asserts the second load hits disk (no second network request, via the stub's request counter).
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full suite plus all `trybuild` snapshots stay green; the three marker types must remain distinct (a parameterized test asserts their `TypeId`s differ).

## Mutation Targets
`cargo mutants` must kill every viable mutant on the testable surface:
- size-guard mutants in `data_uri_exceeds_limit` / R6 (`>`↔`>=`, dropping the cap check, constant swaps on `MAX_DATA_URI_PAYLOAD_BYTES`, replacing the length-arithmetic estimate with an allocating decode) — killed by R6/R7/R8 plus the allocation-counting assertion.
- branch mutants in `fetch_asset` cache-hit logic (`is_empty` negation, `Some`↔`None` cache-dir arm, skipping the disk read) — killed by R9/R10/R12/R13.
- the hex-vs-base64 rendering of `cache_file_name` (R11) and whitespace-strip removal (R5) — killed by R11/R5.
- macro mutants (default-folder swap, dropping the existence check, emitting an absolute instead of manifest-relative path in `compile_error!`) — killed by R16/R17/R18 snapshot tests.
- **MSI target: 100%** on the testable surface. ACCEPTED-UNTESTABLE: the `async_compat`/Tokio runtime-bridging line in `fetch_asset` is excluded — it has no unit harness on the macOS native target (real-socket-only) and is covered behaviorally by R12/R15 on the native path; this is a closed decision, recorded in Forge RLM, not a pending gap.

## Dependencies
- REUSE (permissive, MIT/Apache): `rust-embed` (carries bundled asset bytes for `Bundled` sources), `reqwest` (HTTP fetch), `sha2` (cache-key SHA-256 digest of the URL), `base64` (`data:` decode), `syn`/`quote`/`proc-macro2` (macro parsing + expansion).
- Marley components: **depends on `marley_asset_core`** (M1, REIMPLEMENT) for `Asset`, `AssetCache`, `AssetSource`, `AssetState`, and `AsyncAssetType` (seam-contracts §10.1). `marley_asset_core` is built **before** `marley_assets` in the M1 order, so M1 is no longer gated by a crate whose base types are unspecified. `marley_asset_macro` names `::marley_assets::*` paths in its expansion (runtime collaborator, not a Cargo dep).

## Out of scope / deferred
- The synchronous `AssetCache`/`AssetState`/`Asset`/`AssetSource`/`AsyncAssetType` foundation itself — owned by `marley_asset_core` (seam-contracts §10.1), an upstream M1 dependency built before this crate; this spec only adds the async URL/`data:` sources, the cache extension, and the embed macro on top.
- **The remote-CDN macro path is dropped at M1** (seam-contracts §10.2): `remote_asset!`, `bundled_or_fetched_asset!`, and any `make_absolute_url`/content-hashed fetch-URL helper are **not** part of this spec. They reintroduce the Warp-CDN/wasm asset machinery that crate-triage marked SKIP and contradict the local-first charter. M1 ships `bundled_asset!` + `data:`-URI / `url_source` sources only; a content-hashed remote-fetch endpoint, if ever needed, is a separate later spec with its own charter justification.
- HTTP caching semantics beyond presence/non-emptiness (ETag, `Cache-Control`, TTL expiry) — deferred; the cache is content-by-URL with manual invalidation only.
- WASM-target asset delivery and persistence — out of scope for M1 (native cockpit only).

## Clean-room provenance
Behavior-derived from a fork-reference doc (observable I/O of an async asset layer on a synchronous cache) — no AGPL/fork source read, no private module/type/static names reproduced, IP-counsel sign-off pending (open item in `clean-build-plan.md`). Seam ownership and scope conform to `standards/seam-contracts.md` §10: the `Asset`/`AssetCache`/`AssetSource`/`AssetState`/`AsyncAssetType` foundation is owned by `marley_asset_core` and depended upon here (§10.1), and the remote-CDN/wasm macro machinery is dropped per §10.2. Public-surface identifiers are Marley-original (clean-room Posture A): the M1 surface is `bundled_asset!` + the `url_source`/`url_source_with_persistence`/`data_uri_source` async sources and the three distinct `AsyncAssetType` markers. The packages are named `marley_assets` and `marley_asset_macro`; REUSE crates (`rust-embed`, `reqwest`, `sha2`, `base64`, `syn`, `quote`, `proc-macro2`) are MIT/Apache. No Warp source was consulted.
