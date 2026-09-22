# asset_cache

> Per-crate reference (Marley round 2) — crate dir `crates/asset_cache`. Marley is Ignibyte's fork of Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[permissive: standard deps]` — URL-keyed `reqwest` asset cache; N/A for the offline build (no remote asset delivery). See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (no in-crate LICENSE marker; inherits workspace `license`) |
| Internal deps | 1 (`warpui_core`) |
| Used by | 4 (`ui_components`, `warp`, `warp_editor`, `warpui`) |

## Purpose

Adds **URL- and `data:`-URI-backed async asset sources** on top of `warpui_core`'s built-in `AssetCache`. It fetches remote bytes over HTTP (optionally persisting them to a hashed file in a cache dir for future hits) and decodes inline base64 `data:` URIs, so the UI can render images/fonts/etc. referenced by URL without blocking the render loop. It exists to bridge `warpui_core`'s synchronous asset abstraction to real network/disk I/O.

## Key types, modules & public API

Single-file crate (`src/lib.rs`, tests in `lib_tests.rs`). Builds on `warpui_core::assets::asset_cache::{Asset, AssetCache, AssetSource, AssetState, AsyncAssetId, AsyncAssetType}`.

- `pub trait AssetCacheExt` (impl'd for `warpui_core`'s `AssetCache`) — `load_asset_from_url<T: Asset>(&self, url: &str, cache_dir: Option<&Path>) -> AssetState<T>`. The primary entry point: persists when `cache_dir` is `Some`, memory-only when `None`.
- Source constructors returning `AssetSource::Async`:
  - `pub fn url_source(url) -> AssetSource` — fetch to memory, no persistence (`UrlAssetWithoutPersistence`).
  - `pub fn url_source_with_persistence(url, cache_dir: &Path) -> AssetSource` — fetch + write to a hashed file (`UrlAssetWithPersistence`).
  - `pub fn data_uri_source(source: &str) -> Option<AssetSource>` — decode `data:[mediatype];base64,<payload>` (`DataUriAsset`); strips embedded whitespace, rejects oversize.
- Marker types implementing `AsyncAssetType`: `UrlAssetWithoutPersistence`, `UrlAssetWithPersistence`, `DataUriAsset` (kept distinct so a previously-unpersisted URL still gets persisted on a later persisted load).
- `pub const MAX_DATA_URI_PAYLOAD_BYTES: usize = 16 * 1024 * 1024` and `pub fn data_uri_exceeds_limit(source: &str) -> bool` — guard against oversized untrusted `data:` payloads.
- Internal I/O: `fetch_file_to_memory` (wraps `reqwest::get` in `async_compat::Compat` off-wasm so reqwest gets a Tokio context), `get_file_path_for_asset` (hashes the URL → hex filename, deliberately not base64 to stay case-insensitive-FS safe), `persist_bytes`, `fetch_asset_from_url` (reads cached file if present and non-empty, else fetches).

## Depends on (internal)

- [`warpui_core`](./warpui_core.md) — the `AssetCache` / `AssetSource` / `AssetState` types this crate extends; `asset_cache` plugs async network sources into that framework.

## Used by (internal dependents)

- [`warpui`](./warpui.md) — UI runtime that loads assets.
- [`ui_components`](./ui_components.md) — shared components rendering images/icons.
- [`warp_editor`](./warp_editor.md) — editor surfaces needing remote/data-URI assets.
- [`warp`](./warp.md) — top-level app.

## Related crates

- [`warpui_core`](./warpui_core.md) — owns the synchronous `AssetCache` this crate makes async. **MIT-licensed**, unlike `asset_cache` itself (AGPL).
- [`watcher`](./watcher.md) — sibling `warpui_core`-based infra crate.
- [`node_runtime`](./node_runtime.md) — shares the "fetch over HTTP, persist to a hashed path in the data dir" pattern.

## Marley relevance

**Classification: KEEP.** Generic, brand-free image/asset plumbing with no auth coupling. It is directly useful for Marley goal (1) expand the UI surface with a custom panel: a Marley panel rendering remote thumbnails, avatars, or inline `data:` images uses `AssetCacheExt::load_asset_from_url` as-is. No de-Warp work needed (no Warp strings, no Warp endpoints — caller supplies the URL). Note the **license boundary**: this crate is AGPL but sits directly on the MIT `warpui_core`; keep that split intact when redistributing. KEEP unchanged.

## Notes / gotchas

- **Security:** `data:` URIs are treated as untrusted — `data_uri_exceeds_limit` rejects base64 payloads over 16 MiB *before* decoding/cloning. Respect this if extending.
- **Runtime bridging:** off-wasm, `reqwest::get` is wrapped in `async_compat::Compat` because reqwest assumes a Tokio runtime; on wasm it calls reqwest directly. Persistence is a no-op on wasm (`persist_bytes` just logs).
- **Cache key:** files are named by a `DefaultHasher` digest of the URL rendered as hex (not base64) to stay safe on case-insensitive filesystems like macOS. Empty cached files are treated as a miss and re-fetched.
- `edition = "2024"`, `version = "0.0.0"`.
