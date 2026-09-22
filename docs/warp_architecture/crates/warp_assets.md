# warp_assets

> Per-crate reference (Marley round 2). Dir: `crates/warp_assets`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — rebrand]` — `rust_embed` provider for branded imagery/icons; rebrand touch point, not ported. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `license = "AGPL-3.0-only"`) |
| Internal deps | 1 |
| Used by | 1 |

## Purpose

The **runtime asset embedder**. Where [`asset_macro`](./asset_macro.md) produces compile-time references, `warp_assets` carries the actual bytes: it uses `rust-embed` to bake the contents of `app/assets` directly into the binary so bundled assets are available with no filesystem dependency at runtime (essential for WASM, useful for portable native builds). It is the concrete `AssetProvider` the rest of the app reads from.

## Key types, modules & public API

Single file: `crates/warp_assets/src/lib.rs`.

- `pub struct Assets` — a zero-sized `#[derive(Clone, Copy, RustEmbed)]` type. The `#[folder = "../../app/assets"]` attribute embeds the directory, with `#[include = "bundled/**"]` and `#[include = "async/**"]` selecting the two embedded trees (kept in sync with `warp_util::assets::BUNDLED_ASSETS_DIR` / `ASYNC_ASSETS_DIR`).
- `impl warpui_core::AssetProvider for Assets` — `fn get(&self, path: &str) -> anyhow::Result<Cow<'_, [u8]>>`. Delegates to `<Assets as RustEmbed>::get(path)`, returning the embedded bytes or `anyhow!("no asset exists at path {}", path)`.

Conditional embedding (from the `RustEmbed` attributes):
- `#[cfg_attr(target_family = "wasm", exclude = "async/**")]` — WASM excludes the large `async/` assets (they are fetched remotely instead).
- `#[cfg_attr(feature = "standalone", exclude = "async/**")]` — the `standalone` feature does the same for the headless `oz` CLI tarball to keep the binary small.

## Depends on (internal)

- [`./warpui_core.md`](./warpui_core.md) — provides the `AssetProvider` trait that `Assets` implements; this is how the UI layer consumes embedded bytes without knowing the embed mechanism.

## Used by (internal dependents)

- [`./warp.md`](./warp.md) — the top-level app crate constructs `Assets` and hands it to the UI/asset-cache layer as the `AssetProvider`.

## Related crates

- [`./asset_macro.md`](./asset_macro.md) — emits the `AssetSource::Bundled { path }` references that resolve against this provider's `get`; the two must agree on the `bundled`/`async` directory split.
- [`./warp_util.md`](./warp_util.md) — `warp_util::assets` is the canonical source for the directory-name constants the embed include/exclude globs mirror.
- `warpui_core` — owns `AssetProvider` and the asset cache that calls it.

## Marley relevance

**Classify: KEEP (rebrand its payload, not its code).** The Rust here is mechanical (`rust-embed` glue) and goal-agnostic, but the *embedded directory* `app/assets` is the single largest carrier of **Warp branding — goal 4**: logos, onboarding imagery, themes. Marley's de-Warp work lands in the `app/assets` tree, not in this crate's source. Keep the crate; swap the asset payload. Two concrete levers already exist: the `standalone` feature and the `wasm` cfg both `exclude = "async/**"`, so a Marley headless/offline build can drop heavy onboarding art for free. Note `MIT` does **not** apply here (only `warpui*` crates are MIT); `warp_assets` is AGPL despite the "ui-adjacent" name. If we adopt a `marley_*` prefix, rename is trivial (1 dependent) but cosmetic — defer.

## Notes / gotchas

- **`#[folder = "../../app/assets"]` is a path relative to the crate dir**, evaluated at build time by `rust-embed`. The embed tree must exist at build time or compilation fails.
- **Debug builds normally read from disk**; the `wasm` target dependency pins `rust-embed` with `features = ["debug-embed"]` so the data is baked in even in debug (the filesystem isn't readable in the browser).
- **Excludes take precedence over includes** in `rust-embed`, which is why the `wasm`/`standalone` `async/**` excludes win over the `async/**` include.
- `version = "0.0.0"` and `edition = "2024"` — a deliberately unversioned leaf crate.
