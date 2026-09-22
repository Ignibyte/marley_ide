# asset_macro

> Per-crate reference (Marley round 2). Dir: `crates/asset_macro`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — rebrand]` — bundled/remote asset-classification macros; not ported. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `license = "AGPL-3.0-only"`) |
| Internal deps | 1 |
| Used by | 1 |

## Purpose

A `proc-macro` crate that resolves **compile-time asset references** into `AssetSource` values. It exists so app code can reference bundled, remote, or split (native-bundled / web-remote) assets by relative path while the build verifies the file exists and, for remote assets, embeds a content hash into the fetch URL. This is what makes the asset pipeline type-safe and prevents shipping dangling references.

The crate documents three asset classes (from its module docs):
- **Bundled** — always in the app bundle (`app/assets/bundled`).
- **Remote** — always fetched by name + content hash (`app/assets/remote`).
- **Bundled-on-native / remote-on-web** — large assets split out to keep the WASM build small (`app/assets/async`).

## Key types, modules & public API

Single file: `crates/asset_macro/src/lib.rs`. It is `proc-macro = true`, so its public surface is macros, not types.

- `bundled_asset!("path", [folder])` — `#[proc_macro] pub fn bundled_asset`. Verifies the file exists under `app/assets/<folder>` (default `bundled`) and expands to `::warpui_core::assets::asset_cache::AssetSource::Bundled { path }`.
- `remote_asset!("path", [folder])` — `#[proc_macro] pub fn remote_asset`. Reads the file at build time, SHA-256-hashes it (`sha2::Digest`), and expands to `::asset_cache::url_source(::warp_util::assets::make_absolute_url(<hashed url>))` via `warp_util::assets::hashed_asset_path` / `hashed_asset_url`.
- `bundled_or_fetched_asset!("path")` — `#[proc_macro] pub fn bundled_or_fetched_asset`. Emits a `cfg`-gated block: `bundled_asset!` on `not(target_family = "wasm")`, `remote_asset!` on `wasm`, both pinned to `ASYNC_ASSETS_DIR`. Works around the fact that proc macros can't see their consumer's compile target.
- Internal helpers: `MacroArgs` (the `syn::parse::Parse` impl accepting one or two `LitStr`s), `full_asset_path` (resolves relative to `CARGO_MANIFEST_DIR` since the proc-macro CWD is undefined), and `format_error` (expands to `compile_error!` with the absolute path).

## Depends on (internal)

- [`./warp_util.md`](./warp_util.md) — uses `warp_util::assets` constants (`ASSETS_DIR`, `ASYNC_ASSETS_DIR`, `BUNDLED_ASSETS_DIR`, `REMOTE_ASSETS_DIR`) and the URL/hash helpers (`hashed_asset_path`, `hashed_asset_url`, `make_absolute_url`) to locate files and build hashed remote URLs.

## Used by (internal dependents)

- [`./warp.md`](./warp.md) — the top-level app crate is the only direct dependent; it invokes the macros wherever assets are referenced. The expansions also name `warpui_core::assets::asset_cache` and `asset_cache`, so those crates are runtime collaborators even though they are not Cargo deps of `asset_macro`.

## Related crates

- [`./warp_assets.md`](./warp_assets.md) — the runtime embedder (`rust-embed`) that actually carries the bytes for `Bundled` sources this macro emits.
- [`./warp_util.md`](./warp_util.md) — owns `warp_util::assets`, the shared source of truth for asset directory names and URL construction.
- `warpui_core` — defines `AssetSource` and `asset_cache`, the targets of the macro expansion.

## Marley relevance

**Classify: KEEP (likely RENAME-deferred).** This is build infrastructure with no Warp branding in its logic — it resolves paths and hashes. None of the four Marley goals touch it directly: it is not UI, not session, not auth. It matters indirectly to **goal 4 (de-Warp rebrand)** only because remote assets are fetched from a Warp-hosted URL via `make_absolute_url` (defined in `warp_util::assets`) — so if Marley replaces or stubs the remote-asset CDN, the change happens in `warp_util`, not here. Keep the macro as-is; if we standardize the crate prefix to `marley_*`, renaming `asset_macro` is low-cost (1 dependent: `warp`) but cosmetic, so defer behind higher-value renames.

## Notes / gotchas

- **Build-time filesystem access.** `bundled_asset!` calls `Path::exists()` and `remote_asset!` calls `std::fs::read` during compilation. A missing asset is a hard `compile_error!`, not a runtime failure — moving/renaming files under `app/assets` will break the build.
- **Paths are anchored to `CARGO_MANIFEST_DIR`**, not the working directory, and the embed folder is `../../app/assets` relative to the crate (mirrored in `warp_assets`). The macro and the embedder must stay in sync on the `bundled`/`async`/`remote` directory split.
- **Target-blindness workaround.** `bundled_or_fetched_asset!` cannot know its consumer's target, so it emits both branches under `cfg`; references upstream Rust issue rust-lang/cargo#10714.
- The macro hard-codes fully-qualified paths like `::asset_cache::url_source` and `::warpui_core::assets::asset_cache::AssetSource`, so any consumer must have those crates in scope even though `asset_macro` doesn't depend on them.
