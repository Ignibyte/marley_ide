# warpui_core

> Per-crate reference (Marley round 2) — crate dir `crates/warpui_core`. Marley is forked from Warp (warpdotdev/warp).
>
> Provenance: **[Warp-derived, MIT/permissive]** (explicit `license = "MIT"`; pins a warpdotdev `cosmic-text` fork) · Marley status: **not adopted** — Marley renders gpui-native (`gpui` 0.2.2, Apache-2.0); WarpUI is unused (0 refs in Marley's tree). See the subsystem doc's *Marley status @ M15*.

| | |
|---|---|
| Subsystem | [UI Framework & Rendering](../subsystems/01-ui-framework-rendering.md) |
| License | **MIT** (`license = "MIT"` in `Cargo.toml`) |
| Internal deps | 6 |
| Used by | 35 |

## Purpose

`warpui_core` is the **retained-mode GUI framework** at the heart of Warp/Marley — an in-house, GPUI-style immediate-data / retained-tree toolkit. It owns the application model: a single global `App` object is the sole owner of every view and model (collectively *entities*), which reference each other through cheap copyable *handles* rather than `Rc` cycles. It defines the element tree, layout/paint pipeline, keymap/action dispatch, font and text layout, the scene abstraction handed to a renderer, telemetry, and the platform/windowing **traits** (the concrete platform impls live in `warpui`).

The crate's `README.md` ("WarpUI — Whirlwind tour") is the canonical design narrative: `App` → entities → `ViewHandle`/`ModelHandle` → `AppContext`/`ViewContext` → `Element` → `Presenter` → `Scene`.

## Key types, modules & public API

`src/lib.rs` re-exports the whole surface; `src/core/mod.rs` (`pub use crate::core::*`) is the backbone.

- **`core::App`** (`src/core/app.rs:74`) — `pub struct App(Rc<RefCell<AppContext>>)`; `App::new(...)` is the bootstrap entry point. **`core::AppContext`** (`app.rs:572`) is the central mutable context threaded everywhere.
- **Entities & handles** — `core::Entity`, `core::View`, `core::ViewHandle`, `core::ModelHandle`, `ViewContext`, `ModelContext`, `Handle<T>` trait (`core/mod.rs:386`), `AnyView` trait, `SingletonEntity` / `GetSingletonModelHandle`, `EntityLocation`, `Effect`, `WindowInvalidation`, `AddWindowOptions`.
- **Elements** (`src/elements.rs` + `elements/`) — the layout primitives re-exported via `prelude`: `Element`, `Flex`, `Container`, `Align`, `Border`, `Padding`, `ConstrainedBox`, `Stack`, `Text`, `ChildView`, `Hoverable`, `MouseStateHandle`, `ParentElement`.
- **Presenter / paint** (`src/presenter.rs`) — `Presenter`, `LayoutContext`, `AfterLayoutContext`, `PaintContext`, `EventContext`, `SizeConstraint`.
- **Scene** (`src/scene.rs`) — `Scene`, `ClipBounds` — the renderer-agnostic display list.
- **Input** — `src/keymap.rs` (`keymap::*`), `src/actions.rs`, `core::TypedActionView`, `src/event.rs` (`Event`).
- **Text & fonts** — `src/fonts.rs`, `src/text.rs`, `src/text_layout.rs`, `src/image_cache.rs`.
- **Platform abstraction** — `src/platform/` defines `platform::Window` trait (`OptionalPlatformWindow = Option<Rc<dyn platform::Window>>`), `Clipboard`, `Cursor`; concrete backends are supplied by `warpui`.
- **`prelude`** (`src/prelude.rs`) — the single glob every consumer imports.
- **`ui_components`** module (`src/ui_components/`) — low-level component scaffolding (`components::Coords`), distinct from the higher-level `ui_components` crate.
- Color/geometry are re-exported as `color` (`pathfinder_color`) and `geometry` (`pathfinder_geometry`).
- Feature `tui` adds `runtime` + a `ratatui`-backed `core::app::tui` path for terminal rendering.

## Depends on (internal)

- [`command`](./command.md) — shell/command primitives used by core types.
- [`markdown_parser`](./markdown_parser.md) — markdown rendering inside UI text elements.
- [`settings_value`](./settings_value.md) — typed settings values (behind the `settings_value` feature).
- [`string-offset`](./string-offset.md) — UTF-8/UTF-16 offset conversions for text layout.
- [`sum_tree`](./sum_tree.md) — the B-tree rope/summary structure underpinning text buffers.
- [`warp_util`](./warp_util.md) — shared utilities.

## Used by (internal dependents)

35 crates depend on it — effectively the entire app. Notable: [`warpui`](./warpui.md) (platform layer), [`warpui_extras`](./warpui_extras.md), [`ui_components`](./ui_components.md), [`warp`](./warp.md) (the binary), [`warp_core`](./warp_core.md), [`warp_terminal`](./warp_terminal.md), [`warp_editor`](./warp_editor.md), [`onboarding`](./onboarding.md), [`settings`](./settings.md), [`ai`](./ai.md), [`lsp`](./lsp.md), [`vim`](./vim.md), [`integration`](./integration.md). Full list also includes `asset_cache`, `cloud_objects`, `computer_use`, `http_server`, `ipc`, `jsonrpc`, `remote_server`, `repo_metadata`, `simple_logger`, `syntax_tree`, `voice_input`, `warp_assets`, `warp_completer`, `warp_files`, `warp_managed_secrets`, `warp_search_core`, `warp_server_auth`, `warp_server_client`, `warp_tui`, `warp_util`, `watcher`.

## Related crates

- [`warpui`](./warpui.md) — the platform/renderer implementation that turns a `Scene` into pixels (winit/wgpu/Metal). Read together.
- [`ui_components`](./ui_components.md) — higher-level reusable widgets built on this crate's `Element`.
- [`warpui_extras`](./warpui_extras.md) — optional add-ons (secure storage, user prefs) registered into the `AppContext`.

## Marley relevance

**Classification: KEEP — and the eventual RENAME anchor (deferred).** This is the foundation of every Marley goal:
- **(1) Custom panel** — a new Marley panel is just a new `View`/`Element` tree presented through this crate's `Presenter`; no fork of core needed, only additive code.
- **(2) Session spawn/write/read** — sessions surface as entities/handles owned by `App`; reads/writes flow through `AppContext` effects.
- **(4) De-Warp rebrand** — the public crate name is `warpui_core` and the README/types say "WarpUI". A clean rebrand would `RENAME` it to e.g. `marleyui_core`, but with **35 internal dependents** this is the single most expensive rename in the workspace — **defer it** until late, or alias and keep the on-disk name. License is MIT, so re-licensing is not forced.

Do **not** STUB or REMOVE: nothing here is auth/network-coupled. De-auth work happens in `warpui_extras`/`warp_server_auth`, not here.

## Notes / gotchas

- Heavy **platform-conditional** `Cargo.toml`: macOS pulls `objc2*`/`cocoa`/`core-text`; non-macOS pulls `winit`/`wgpu`/`cosmic-text`; wasm pulls `gloo`/`web-sys`. Expect different code paths per OS.
- `cosmic-text` is pinned to a **warpdotdev fork** via git (`github.com/warpdotdev/cosmic-text`, rev `15198be…`) — an out-of-repo dependency to vendor/mirror for Marley reproducible builds.
- `version = "0.1.0"` (unlike most workspace crates at `0.0.0`).
- `tui` feature gates real API (`runtime`, `[[test]] tui_integration`, `tui_demo` example) — it is off by default.
- Telemetry: `log_named_telemetry_events` feature warns it must not ship in release without a PII audit — relevant to a de-telemetry Marley pass.
