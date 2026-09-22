# warpui

> Per-crate reference (Marley round 2) — crate dir `crates/warpui`. Marley is forked from Warp (warpdotdev/warp).
>
> Provenance: **[Warp-derived, MIT/permissive]** (explicit `license = "MIT"`; re-exports `warpui_core::*`) · Marley status: **not adopted** — Marley's `Scene → native-Metal` stack comes from gpui 0.2.2 (Apache-2.0), not `warpui`. See the subsystem doc's *Marley status @ M15*.

| | |
|---|---|
| Subsystem | [UI Framework & Rendering](../subsystems/01-ui-framework-rendering.md) |
| License | **MIT** (`license = "MIT"` in `Cargo.toml`) |
| Internal deps | 6 |
| Used by | 8 |

## Purpose

`warpui` is the **platform & rendering backend** for the `warpui_core` framework. Where `warpui_core` is platform-agnostic (traits, scene, layout), `warpui` provides the concrete OS integration that actually creates windows, loads system fonts, talks to the GPU, and rasterizes a `Scene` into pixels. Its `lib.rs` re-exports all of `warpui_core` (`pub use warpui_core::*`), so downstream crates depend on `warpui` to get *both* the framework and a working backend in one import.

It is the seam where Warp's GUI meets winit/wgpu (Linux/Windows/wasm), AppKit/Metal (macOS), and the browser (wasm/`gloo`).

## Key types, modules & public API

`src/lib.rs` exposes five platform modules plus the full `warpui_core` re-export:

```rust
pub mod browser;
pub mod fonts;
pub mod platform;
pub mod rendering;
pub mod windowing;
pub use warpui_core::*;
```

- **`windowing`** — the windowing backends. `windowing::winit` holds `AppDelegate` (`winit/delegate.rs:181`, `AppDelegate::new(event_loop_proxy)`), `DispatchDelegate`, `IntegrationTestDelegate`, the `TextLayoutSystem` + `FontDB` (`winit/fonts.rs`), the platform `Window` impl (`winit/window.rs`, `get_os_window_manager_name()`, titlebar metrics), and `open_url_in_system(url)`. This is where `winit::CustomEvent` and the event loop live.
- **`rendering`** — `rendering::wgpu::renderer::Renderer` (`rendering/wgpu/renderer.rs:24`) is the wgpu pipeline that consumes a `warpui_core::Scene`; glyph atlas + texture cache support it. macOS has a separate Metal path (objc2-metal); `experimental-wgpu-renderer` feature can force wgpu even there.
- **`fonts`** — system font discovery/loading (`font-kit` on macOS, `fontdb`/`owned_ttf_parser`/`cosmic-text` elsewhere).
- **`platform`** — platform predicates/impls; e.g. `platform::is_mobile_device()`.
- **`browser`** — wasm/web entry path.

Consumers typically `use warpui::prelude::*` and construct the windowing delegate + renderer at startup.

## Depends on (internal)

- [`warpui_core`](./warpui_core.md) — the framework it implements a backend for and re-exports.
- [`asset_cache`](./asset_cache.md) — cached asset/image loading for rendering.
- [`command`](./command.md) — command primitives (non-macOS targets).
- [`markdown_parser`](./markdown_parser.md) — markdown surfaced through text rendering.
- [`sum_tree`](./sum_tree.md) — text buffer summary structure.
- [`virtual-fs`](./virtual-fs.md) — virtual filesystem used by font/asset loading paths.

## Used by (internal dependents)

8 crates: [`warp`](./warp.md) (the binary), [`ui_components`](./ui_components.md), [`warp_editor`](./warp_editor.md), [`onboarding`](./onboarding.md), [`integration`](./integration.md), [`mcp`](./mcp.md), [`repo_metadata`](./repo_metadata.md), [`syntax_tree`](./syntax_tree.md).

## Related crates

- [`warpui_core`](./warpui_core.md) — the framework; `warpui` is meaningless without it. Read first.
- [`ui_components`](./ui_components.md) — widgets that render through this backend.
- [`asset_cache`](./asset_cache.md) — feeds textures/fonts into the renderer.

## Marley relevance

**Classification: KEEP (RENAME deferred).** This is the rendering substrate for **goal (1) — the custom Marley panel**: a new panel needs a real window + GPU surface, which only `warpui` provides. No de-auth or de-Warp logic lives here, so it stays as-is functionally.
- **Rebrand (goal 4):** package name `warpui` and module-level "WarpUI" naming would ideally `RENAME` to `marleyui`, but it sits directly under 8 dependents (and transitively the whole app), so **defer**; an `[package] name` alias keeps churn down. MIT license means no relicensing obligation.
- **Risk to watch:** because `warpui` re-exports `warpui_core::*`, renaming either crate must be coordinated — a Marley rebrand touches both at once.

## Notes / gotchas

- Massive **per-target dependency matrix** in `Cargo.toml`: macOS = `objc2-app-kit`/`objc2-metal`/`cocoa`/`core-text`/`core-graphics`; non-macOS = `winit`/`wgpu`/`cosmic-text`/`resvg`/`fontdb`; wasm = `gloo`/`js-sys`/`web-sys`/`wasm-bindgen`; non-macOS desktop also pulls `global-hotkey`.
- `cosmic-text` is a **git-pinned warpdotdev fork** (rev `15198be…`) — vendor it for Marley.
- Feature `enable-metal-frame-capture` writes Metal shaders to a temp dir at first-renderer init (slows first window) — a debug-only workaround for a Metal frame-capture bug.
- `takecell`, `core-graphics 0.25`, `core-text 21.0`, `dispatch 0.2` are direct (non-workspace) version pins.
- `traces`, `integration_tests`, `defer_scene_build`, `schema_gen`, `test-util` features all forward to `warpui_core`'s same-named features.
