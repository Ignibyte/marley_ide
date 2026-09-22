# Subsystem 01 — UI Framework & GPU Rendering (WarpUI)

> Part of the Marley architecture docs. **Round 3 re-review (@ Marley M15)** of the Jun-27
> deconstruction. Marley is forked from Warp (warpdotdev/warp); Warp ships under **AGPL-3.0**.
>
> **Re-review headline — READ FIRST.** The crates deconstructed below (`warpui_core`, `warpui`,
> `warpui_extras`, and Warp's AGPL `ui_components`) are **Warp's in-house UI stack. Marley never
> adopted any of them.** Marley renders **gpui-native** — `gpui = "0.2.2"` (Apache-2.0), the same
> standalone crate Zed authors (see
> [`../../zed_architecture/subsystems/01-gpui-ui-framework.md`](../../zed_architecture/subsystems/01-gpui-ui-framework.md)).
> A workspace-wide grep finds **zero** `warpui` / `warp_core` references in Marley's tree. This
> document is therefore a **reference for a road not taken** — kept for the WarpUI/GPUI mental model
> and the MIT-vs-AGPL boundary analysis, *not* as a description of Marley's shipping UI. The two
> sections added this round — **§ Marley status @ M15** and **§ Provenance & licensing** (immediately
> below) — carry the current truth; the older "Marley relevance" / "Open questions" sections retain
> the (now-stale) WarpUI-adoption framing and are flagged inline.

## Scope

This document covers the **MIT-licensed**, in-house GPU UI framework that the
entire Warp/Marley desktop client is built on. It is a Flutter-inspired,
immediate-mode-render / retained-scene-paint hybrid with its own entity tree,
layout engine, and cross-platform GPU backends (Metal on macOS, wgpu →
Vulkan/DX12/Metal/GL/WebGL elsewhere).

Crates surveyed (all under `crates/`):

| Crate | License | Role |
|-------|---------|------|
| `warpui_core` | **MIT** | Platform-agnostic core: entity/App model, `View`/`Element` traits, layout/paint (`Presenter`), the retained `Scene`, the built-in element + `ui_components` widget libraries, keymap, fonts, text layout, async executor abstractions. No GPU code. |
| `warpui` | **MIT** | Platform layer + GPU renderers. Re-exports all of `warpui_core` (`pub use warpui_core::*`) and adds the macOS Cocoa/Metal backend, the winit + wgpu backend (Linux/Windows/Wasm), a headless backend, font rasterization, and the `AppBuilder` entry point. |
| `warpui_extras` | **MIT** | Small platform-service add-ons on top of `warpui_core`: `secure_storage` (Keychain / secret-service / Windows cred) and `user_preferences` (NSUserDefaults / file / toml / localStorage). Not UI rendering per se. |
| `ui_components` | inherits workspace `AGPL-3.0-only` | Higher-level, **themeable** widget library (`Component` trait: button, dialog, switch, tooltip, lightbox, keyboard_shortcut). Depends on `warpui_core` **and** `warp_core` (for `warp_core::ui::appearance::Appearance`), which is why it is *not* MIT. |

### License boundary (critical for Marley)

- `warpui`, `warpui_core`, `warpui_extras` declare `license = "MIT"` explicitly
  in their `Cargo.toml`. They have **no dependency on `warp_core`** and are the
  reusable, permissively-licensed substrate.
- The workspace default (`Cargo.toml` `[workspace.package] license = "AGPL-3.0-only"`)
  applies to crates that use `license.workspace = true`. `ui_components` does
  this and also depends on `warp_core` → it is effectively **AGPL**. Repo root
  ships both `LICENSE-MIT` and `LICENSE-AGPL`.
- **Implication:** A new Ignibyte UI surface written purely against
  `warpui`/`warpui_core` *can* remain MIT. The moment it links `warp_core`
  (e.g. to read terminal/session/appearance state) or `ui_components`, the
  derived work falls under AGPL. Decide this deliberately when building the
  Marley agentic panel.

---

## Marley status @ M15

> Added in the round-3 re-review (Marley at M15). **Bottom line: Marley does not use WarpUI.** The
> subsystem above documents Warp's own framework; Marley's UI is built directly on **gpui 0.2.2
> (Apache-2.0)** — the Zed-authored, standalone renderer — with two Marley-original crates on top:
> `crates/marley_app` (the app shell / `RootView`) and `crates/ui_components` (package
> `marley_ui_components`, a *different* crate from Warp's AGPL `ui_components` despite the name — it
> depends on **gpui only**, and owns `Appearance` + `ThemeColors` + five cockpit widgets). Marley's
> workspace license is **`MIT OR Apache-2.0`** (permissive), not AGPL.

**What Marley matches / exceeds vs the WarpUI-era plan.** The Jun-27 "Marley relevance" here imagined a
single custom panel slotted into WarpUI. Marley has shipped far past that — on gpui, not WarpUI:

- A full multi-pane **cockpit**: 3-region workspace (left dock · tiled terminal center · right dock),
  the `PaneGroup` split/close/neighbor algebra, drag-resize, directional focus.
- A fuzzy **command palette** (`nucleo`), a **chord keymap** layer, and **themes** with a dark default
  and *WCAG-proven* contrast (`relative_luminance` / `contrast_ratio` in `marley_ui_components`).
- **M13** workspace-centric shell: a launcher / landing page, a Workspace→Project→Tab→Pane rail
  (collapsible, focus-highlighted), a top-bar workspace switcher, editor-surface tabs.
- **M15** editable editor: a buffer-backed doc model (`OpenFile { Buffer, caret, saved_version, anchor }`),
  a faithful buffer-driven renderer, a key-input intercept that makes the editor typeable, and selection
  anchors — net-new surface WarpUI never provided to Marley.

Warp's *equivalent* app features (workspace, command blocks, palette) live in its **AGPL `warp_core`** (a
different subsystem), so at the framework layer the honest comparison is: **Marley reimplemented the
cockpit clean-room on gpui** rather than deriving it from WarpUI + `warp_core`.

**What Marley still lacks — but the gap is against gpui, not WarpUI.** Marley hand-rolls the low-level
render/input that gpui already productizes (grep-confirmed absent in `marley_app` + `marley_ui_components`):
**no** `uniform_list` / `list` virtualization (one `div` per line, O(n)); **no** `StyledText` / `TextLayout`
(hand `split_caret_char`, no pixel↔index hit-testing); **no** `EntityInputHandler` / IME (hand `apply_key` /
`apply_editor_key`); **no** `KeyContext` / `actions!` (a hand keymap chord table); **no** `anchored` /
`deferred` overlays (hand-placed absolute `div`s). These are exactly the adoption backlog catalogued in the
**Zed** gpui subsystem doc
([`01-gpui-ui-framework.md`](../../zed_architecture/subsystems/01-gpui-ui-framework.md) §Lists / §Text /
§Input / §Focus / §Overlays) — the productive "what to adopt next" reference for Marley's UI is that
document, **not** this WarpUI one.

**Stale claims flagged in this doc** (all rooted in the same wrong premise — that Marley extends WarpUI):

- *Scope → "License boundary (critical for Marley)"* — the MIT-vs-AGPL `warp_core`-linking calculus is
  **moot** for Marley: it links **neither** WarpUI **nor** `warp_core`. It survives only as Warp-reference.
- *"Minimal app … the **exact recipe for adding a new top-level Marley surface**"* — wrong API. Marley adds
  a surface via gpui's `Render` / `impl IntoElement` + `div()`, not WarpUI's
  `impl Entity + View (+ TypedActionView)` / `add_window` / `add_view`.
- *"## Marley relevance"* (UI-surface expansion / session bridge / de-auth / rebrand / license lever) —
  every bullet assumes a WarpUI `View` / `Element` / `ChildView` / `Presenter` that Marley does not use;
  the MIT "license lever" is irrelevant because Marley never took the MIT framework.
- *"## Open questions (round 2)" 1–6* — these ask where Warp's `WorkspaceView` / `AppBuilder::run` / dock
  system live. For Marley they are answered by Marley's own gpui shell
  (`crates/marley_app/src/app.rs` + `workspace.rs` / `tabs.rs` / `layout.rs`), not by locating them in
  `warp_core`.

## Provenance & licensing

> Round-3 tags, mirroring the style of `docs/zed_architecture/` (`[Warp-derived/AGPL]` ·
> `[permissive: gpui Apache-2.0 + other deps]` · `[Marley-original]`). **Scope note:** the first table
> classifies the **Warp reference crates** this subsystem documents — Marley ships **none** of them; the
> second records what Marley actually renders on.

| Crate (as documented) | Provenance | Manifest license | In Marley? |
|---|---|---|---|
| `warpui_core` | **[Warp-derived, MIT/permissive]** — Warp-authored GPUI-style framework; pins a warpdotdev `cosmic-text` fork | `license = "MIT"` (explicit) | **No** — never linked |
| `warpui` | **[Warp-derived, MIT/permissive]** — platform/GPU backend; re-exports `warpui_core` | `license = "MIT"` (explicit) | **No** |
| `warpui_extras` | **[Warp-derived — MIT-intended, AGPL-in-manifest]** — the doc's own gotcha: `license.workspace = true`, no per-crate override | `license.workspace = true` ⇒ **AGPL-3.0-only** | **No** |
| Warp's `ui_components` | **[Warp-derived/AGPL]** — themeable widgets; links AGPL `warp_core` for `Appearance` | `license.workspace = true` ⇒ **AGPL-3.0-only** | **No** — superseded by Marley's own `marley_ui_components` (name collision, different crate) |

**What Marley actually renders on (the real subsystem-01 surface):**

| Crate / dep | Provenance | License |
|---|---|---|
| `gpui` 0.2.2 | **[permissive: gpui Apache-2.0]** — standalone Zed-authored crate; not GPL/AGPL. Marley's whole `Scene → native-Metal` stack. | Apache-2.0 |
| `crates/marley_app` | **[Marley-original]** — the `RootView` shell, the pure/shim seam, all cockpit + editor logic | MIT OR Apache-2.0 (workspace) |
| `crates/ui_components` (`marley_ui_components`) | **[Marley-original]** on **[gpui Apache-2.0]** — `Appearance` + `ThemeColors` + button/switch/dialog/tooltip/keyboard-shortcut; depends on **gpui only** (no `warp_core`, no WarpUI) | MIT OR Apache-2.0 (workspace) |

**Net:** the AGPL exposure this doc worries about (linking `warp_core` via `ui_components`, or the
`warpui_extras` manifest discrepancy) **does not reach Marley's UI** — Marley's rendering subsystem is
clean-room `[Marley-original]` over `[gpui Apache-2.0]`, carrying no Warp code. The AGPL posture in
Marley's product plan applies to derived editor/terminal *logic* in other subsystems, not to this substrate.

---

## The mental model (from `crates/warpui_core/README.md`)

WarpUI borrows GPUI/Flutter ideas. Three concept layers:

1. **Entities** = `View`s and `Model`s. The single global `App` owns *all* of
   them; nobody else holds them directly. You reference an entity through a
   **handle** (`ViewHandle<T>` / `ModelHandle<T>`), which is "a glorified
   identifier" (an `EntityId`) plus a refcount. A handle is dereferenced to the
   real entity only when you hold an `&AppContext` (or a typed `ViewContext` /
   `ModelContext`), which the framework hands you at specific callbacks.
2. **Views** implement `render(&self, &AppContext) -> Box<dyn Element>` — like a
   React component. Long-lived; re-rendered when their state is invalidated.
3. **Elements** are throwaway, single-frame layout/paint primitives (Flutter-style).
   The element tree returned by `render` is laid out, painted into a `Scene`,
   and discarded/recycled each frame.

This is the "immediate + retained hybrid": the **element tree is rebuilt
immediately each frame** from view state, but it paints into a **retained
`Scene`** (a flat list of GPU draw primitives with a spatial hit-test index)
that the platform renderer consumes.

---

## Key types & entry points

### Entity / App core — `crates/warpui_core/src/core/`

- `entity.rs`
  - `EntityId(usize)` — globally-unique, atomic counter. Shared namespace for
    views and models.
  - `trait Entity: 'static { type Event; }` — base for both views and models.
  - `trait SingletonEntity` — global/singleton models fetched by type
    (`Self::as_ref(ctx)`), used for app-wide state.
- `app.rs` (≈4900 lines — the heart of the framework)
  - `pub struct App(Rc<RefCell<AppContext>>)` — cloneable handle to the one true
    owner. `App::new(platform_delegate, window_manager, font_db, asset_provider)`.
  - `pub struct AppContext` — the actual state container. Holds windows, the
    entity maps, refcounts, executors, action registry, font cache, platform
    delegate, etc. **This is the `&AppContext` / `&mut AppContext` threaded
    through every render/layout/paint/event call.**
  - Window & entity creation (the surface Marley expands):
    - `add_window<T,F>(style, build_root_view) -> WindowId` and
      `add_window_with_bounds(...)` — creates an OS window whose **root view**
      is the `View` returned by `build_root_view`.
    - `add_view<T,F>(window_id, build_view) -> ViewHandle<T>` — create a child
      view inside a window.
    - `add_model<T,F>(build_model) -> ModelHandle<T>` — create a model.
    - `root_view<T>(window_id)` / `root_view_id(window_id)`,
      `view_descendants(window_id, root_view_id)`,
      `transfer_view_tree_to_window(...)` (multi-window view migration).
  - Actions (the command/event-dispatch system):
    - `add_action(name, handler)` / `add_global_action(name, handler)`,
      `dispatch_action(window_id, responder_chain, name, arg)`,
      `dispatch_global_action`. Actions bubble up a responder chain of
      `EntityId`s; a handler returns `bool` (handled / keep propagating).
  - `update<T,F: FnOnce(&mut AppContext)->T>(callback)` — the mutate-then-flush
    boundary that re-runs effects and triggers re-render.
- `mod.rs`
  - `trait AnyView` — object-safe erasure of `View` (the framework stores
    `Box<dyn AnyView>`); blanket-impl `impl<T: View> AnyView for T`.
  - `trait Handle<T> { fn id(&self) -> EntityId; fn location(&self) -> EntityLocation; }`
  - `enum EntityLocation { Model(EntityId), View(WindowId, EntityId) }`
  - `enum Effect`, `struct WindowInvalidation` — the invalidation/effect queue
    that schedules repaints.

### The View interface — `crates/warpui_core/src/core/view/mod.rs`

```rust
pub trait View: Entity {
    fn ui_name() -> &'static str;
    fn render(&self, app: &AppContext) -> Box<dyn Element>;   // <-- builds element tree
    fn on_focus(&mut self, _: &FocusContext, _: &mut ViewContext<Self>) {}
    fn on_blur(&mut self, _: &BlurContext, _: &mut ViewContext<Self>) {}
    fn child_view_ids(&self, _: &AppContext) -> Vec<EntityId> { vec![] } // ownership graph
    fn keymap_context(&self, _: &AppContext) -> keymap::Context { ... }
    fn accessibility_data(&self, _: &mut ViewContext<Self>) -> Option<AccessibilityData> { None }
    // + on_window_closed/transferred, a11y, cursor position hooks
}
```

- `trait TypedActionView { type Action; fn handle_action(&mut self, &Self::Action, &mut ViewContext<Self>); }`
  — strongly-typed action handling per view (the root view of a window is added
  via `add_typed_action_view`).
- `ViewContext<'a, T>` (`core/view/context.rs`) is the mutation API a view gets
  inside its callbacks. Key methods for **state flow**:
  - `notify()` — mark this view dirty → schedules re-render.
  - `observe(handle, cb)` / `subscribe_to_model` / `subscribe_to_view` — react
    to another entity's changes/events.
  - `emit(payload: T::Event)` — emit a typed event to subscribers.
  - `focus(handle)` / `focus_self()`.
  - `add_view(build_view)` — spawn a child view, get a `ViewHandle`.
  - `spawn(future, cb)` / `spawn_abortable` / `spawn_stream_local` — run async
    work on the foreground executor and fold the result back into view state
    (this is how a panel would consume streaming subprocess output).
  - Derefs to `&mut AppContext`, and impls `ReadModel/UpdateModel/ReadView/UpdateView`.
- `autotracking` (`core/autotracking/`, re-exported as `Tracked`) — automatic
  dependency tracking so reads of tracked state auto-invalidate dependent views
  (the framework's reactive layer; see `examples/autotracking`).

### Models — `crates/warpui_core/src/core/model/`

- `trait AnyModel` (erasure), `ModelHandle<T>`, `ModelContext<T>`. Models are
  non-visual entities holding shared state; views subscribe/observe them. This
  is where an "AgentWorkflow" / "session registry" state object would naturally
  live, with the panel view observing it.

### Elements (the layout/paint tree) — `crates/warpui_core/src/elements/`

`elements.rs` → `mod gui` (always compiled) `+ mod tui` (feature `tui`,
ratatui-backed). The core trait, `crates/warpui_core/src/elements/gui/mod.rs:106`:

```rust
pub trait Element {
    fn layout(&mut self, constraint: SizeConstraint, ctx: &mut LayoutContext, app: &AppContext) -> Vector2F;
    fn after_layout(&mut self, _: &mut AfterLayoutContext, _: &AppContext);
    fn paint(&mut self, origin: Vector2F, ctx: &mut PaintContext, app: &AppContext);
    fn size(&self) -> Option<Vector2F>;
    fn origin(&self) -> Option<Point>;
    fn z_index(&self) -> Option<ZIndex> { ... }
    fn bounds(&self) -> Option<RectF> { ... }
    fn dispatch_event(&mut self, event: &DispatchedEvent, ctx: &mut EventContext, app: &AppContext) -> bool;
    fn finish(self) -> Box<dyn Element> where Self: Sized { Box::new(self) }   // builder terminator
}
```

- `trait ParentElement: Extend<Box<dyn Element>>` — `add_child` / `with_child` /
  `with_children` builder ergonomics; blanket-impl for any element that is
  `Extend<Box<dyn Element>>`.
- `Point { Vector2F + ZIndex }`, `enum Axis { Horizontal, Vertical }`,
  `enum DispatchEventResult { PropagateToParent, StopPropagation }`.
- **The Flutter-style stock element library** (each its own module, all
  re-exported): `Container`, `Flex` (+ `wrap`), `Stack`/`Overlay`/`Positioned`,
  `Align`, `ConstrainedBox`, `MinSize`, `Padding` (via container), `Rect`,
  `Text` / `FormattedTextElement`, `Image`, `Icon`, `List` / `UniformList` /
  `ViewportedList`, `Table`, `Scrollable` / `NewScrollable` / `ClippedScrollable`,
  `Clipped`, `Hoverable`, `EventHandler`, `Draggable` / `DropTarget` (drag/),
  `Resizable` / `DragResize`, `SelectableArea`, `ShimmeringText`, `ChildView`.
- `ChildView<T>` (`gui/child_view.rs`, impl at `presenter.rs:826`) is the bridge
  that **embeds one view's element tree inside another** — `ChildView::new(&handle)`
  delegates `layout/paint/dispatch_event` back into the presenter for that
  `view_id`. This is how a workspace view composes terminal/panel sub-views.

### Layout / paint engine — `crates/warpui_core/src/presenter.rs`

- `struct Presenter` — **one per window**. Owns `rendered_views: HashMap<EntityId, Box<dyn Element>>`,
  `LayoutCache` (text), `PositionCache` (cross-frame element rects keyed by id,
  used for hit-testing & drag/drop), and the last `Scene`.
- Frame pipeline (driven by `Presenter::build_scene` / the window invalidation
  callback in `core/app.rs`):
  1. `layout(window_size, parents, app)` — from the window's root view id, runs
     constraint-based layout (`SizeConstraint { min, max }`) top-down,
     populating `rendered_views` (each view's `render()` produces its element,
     which is laid out).
  2. `after_layout(app)` — second pass for elements needing post-layout info.
  3. `paint(scale_factor, window_size, max_texture_dimension_2d, app) -> Scene` —
     walks the tree calling `Element::paint`, which imperatively records draw
     primitives into a fresh `Scene`. Returns the scene plus an optional
     `repaint_at` (for animations) and pending asset handles.
- `dispatch_event(event, app) -> DispatchResult` — scales the event for zoom,
  dispatches from root view down the element tree (each element hit-tests and
  recurses), collecting `actions`, `notified` views, cursor updates, timer
  changes.
- Context structs (`LayoutContext`, `AfterLayoutContext`, `PaintContext`,
  `EventContext`) carry the mutable per-phase state (the `Scene`, font cache,
  caches, window size).

### The retained Scene — `crates/warpui_core/src/scene.rs`

- `struct Scene { scale_factor, rendering_config, layers: Vec1<Layer>, overlay_layers, active_layer_index_stack }`.
- `struct Layer { hit_map: RTree<…>, clip_bounds, rects: Vec<Rect>, images: Vec<Image>, glyphs: Vec<Glyph>, icons: Vec<Icon>, click_through }`
  — a flat bucket of GPU-ready primitives **plus an R-tree spatial index** for
  hit-testing.
- `enum ZIndex`, `enum ClipBounds { ActiveLayer, BoundedBy, BoundedByActiveLayerAnd, None }`.
- Imperative draw API (what custom elements call in `paint`):
  `start_layer(bounds)` / `stop_layer()` / `start_overlay_layer`,
  `draw_rect_with_hit_recording(rect) -> &mut Rect`,
  `draw_rect_without_hit_recording`, `draw_image`, `draw_icon`, `draw_glyph`,
  and chained styling `with_border`, `with_background`, `with_corner_radius`,
  `with_drop_shadow`, `with_fade`. Supporting style types: `Border`,
  `CornerRadius`/`Radius`, `DropShadow`, `Fill`, `Gradient` (re-exported from
  `warpui_core` root), `GlyphFade`, `Dash`.
- The `Scene` is the **clean hand-off boundary** between the platform-agnostic
  core and any GPU backend.

### GPU rendering backends — `crates/warpui/src/rendering/` + `crates/warpui/src/platform/*/rendering/`

- `warpui_core::rendering` (`rendering/mod.rs`, `gpu_info.rs`, `texture_cache.rs`)
  defines the backend-agnostic config & device descriptions: `rendering::Config`,
  `GPUBackend`, `GPUDeviceInfo`, `GPUDeviceType` (Integrated/Discrete),
  `GPUPowerPreference`, `OnGPUDeviceSelected`. No GPU API code here.
- `warpui::rendering` adds `atlas/` (glyph/image texture atlas: `manager`,
  `allocator`), `glyph_cache`, and the `wgpu/` backend (gated on the `wgpu`
  cfg-alias): `renderer.rs` (+ `renderer/{rect,glyph,image,frame,util}.rs`),
  `resources.rs` (`quad`, `uniforms`), `shader_types.rs`, and WGSL shaders
  `shaders/{rect,image,glyph}_shader.wgsl`.
- Platform-specific renderers under `crates/warpui/src/platform/`:
  - **macOS / Metal** (default): `platform/mac/rendering/metal/{renderer,renderer_manager,frame_capture}.rs`
    + `shaders/shaders.metal`. `trait Renderer { fn render(&mut self, &Scene, &WindowState, &fonts::Cache); fn resize(...); }`
    (`platform/mac/rendering/renderer.rs:21`). `enum Device { Metal(MetalDevice), WGPU(...) }`
    — macOS can use native Metal *or* the experimental wgpu path
    (`feature experimental-wgpu-renderer`).
  - **Linux/Windows/Wasm**: `crates/warpui/src/windowing/winit/` + the shared
    `wgpu` renderer (Vulkan/DX12/GL/WebGL via wgpu features).
  - **Headless**: `platform/headless/` (no-op `render_scene`) for tests/CI.
- A `Scene` reaches the GPU through the platform `Window`:
  `WindowContext::render_scene(&self, scene: Rc<Scene>)`
  (`crates/warpui_core/src/platform/mod.rs:483`; concrete impls at
  `platform/mac/window.rs:1049/1196` and `windowing/winit/window.rs:1725`).

### Platform abstraction & app entry — `crates/warpui/src/platform/`

- `platform/mod.rs` selects the concrete backend per-target via
  `pub mod current { cfg_if! { wasm | linux/freebsd | macos | windows | else test } }`.
  Re-exports `warpui_core::platform::*`.
- Core platform traits (`warpui_core/src/platform/mod.rs`): `Delegate`
  (app-level OS integration), `WindowManager`, `Window: WindowContext`
  (`minimize`, `toggle_fullscreen`, `graphics_backend`, `supported_backends`,
  `callbacks`), `WindowContext` (`size`, `backing_scale_factor`,
  `max_texture_dimension_2d`, `render_scene`, `request_redraw`,
  `request_frame_capture`), `FontDB`.
- **Entry point**: `platform::AppBuilder` (`platform/app.rs`).
  `AppBuilder::new(AppCallbacks, assets, test_driver)` (or `new_headless`),
  then `.run(|ctx: &mut AppContext| { ... })`. `enum AppBackend { CurrentPlatform, Headless }`.

#### Minimal app (from `crates/warpui/examples/example-black-background-box/`)

```rust
// main.rs
let app_builder = platform::AppBuilder::new(platform::AppCallbacks::default(), Box::new(ASSETS), None);
app_builder.run(move |ctx| {
    ctx.add_window(warpui::AddWindowOptions::default(), |_| root_view::RootView {});
});

// root_view.rs
impl Entity for RootView { type Event = (); }
impl View for RootView {
    fn ui_name() -> &'static str { "RootView" }
    fn render(&self, _: &AppContext) -> Box<dyn Element> {
        Rect::new().with_background_color(ColorU::black()).finish()
    }
}
impl TypedActionView for RootView { type Action = (); }
```

This is the **exact recipe for adding a new top-level Marley surface**: define a
struct, `impl Entity + View (+ TypedActionView)`, return an element tree from
`render`, and register it via `add_window`/`add_view`.

> ⚠️ **STALE @ M15.** This is the *WarpUI* recipe; Marley does not use it. On gpui, a Marley surface is
> a struct that `impl Render` (`fn render(&mut self, &mut Window, &mut Context<Self>) -> impl IntoElement`)
> built from `div()` + the `Styled` builder, mounted via `cx.new(...)` / `cx.open_window(...)`. See
> `crates/marley_app/src/app.rs` (`RootView`) and § Marley status @ M15.

### Widget libraries (two of them)

1. `crates/warpui_core/src/ui_components/` (MIT, no `warp_core` dep) — primitive,
   self-contained widgets returning `Element` trees: `button`, `checkbox`,
   `chip`, `link`, `list`, `progress_bar`, `radio_buttons`, `segmented_control`,
   `slider`, `switch`, `text`, `text_input`, `toggle_button`, `toggle_menu`,
   `tool_tip`, plus `components.rs` (shared helpers, `Coords`).
2. `crates/ui_components/` (AGPL via `warp_core`) — themeable higher-level
   `Component` trait:
   ```rust
   pub trait Component: Default {
       type Params<'a>;
       fn render<'a>(&self, appearance: &Appearance, params: Self::Params<'a>) -> Box<dyn Element>;
   }
   ```
   Components are **stored as fields on a view** (not recreated per frame) so
   internal state like `MouseStateHandle` hover persists. Members: `button`
   (+ `params`, `themes`), `dialog`, `switch`, `tooltip`, `lightbox`,
   `keyboard_shortcut`. Takes `warp_core::ui::appearance::Appearance` → this is
   where theming/branding colors enter.

---

## Data / control flow (one frame)

```
User input ──► platform event loop (mac Cocoa / winit) ──► AppContext
   │                                                          │
   │  Presenter::dispatch_event ◄── root View's Element tree  │
   │      └─ elements hit-test & handle, emit Actions ────────┤
   │                                                          ▼
   │   view.notify() / model change / async spawn result ─► invalidation queue
   ▼                                                          │
WindowInvalidation ──► Presenter::build_scene ───────────────┘
   1. layout (View::render → Box<dyn Element>, constraint solve)
   2. after_layout
   3. paint  → records Rect/Image/Glyph/Icon into Scene layers
        │
        ▼
   Window::render_scene(Rc<Scene>) ──► platform Renderer (Metal / wgpu)
        └─ atlas + glyph cache → GPU draw → swapchain present
```

State flows **down** via `View::render` reading view/model state, and **up**
via `Element::dispatch_event` → `Action`s → `ViewContext`/`ModelContext`
mutations → `notify()` → re-render. Async (subprocess/streaming) results enter
through `ViewContext::spawn*`.

---

## Dependencies on other subsystems

- `warpui_core` is **self-contained** (depends only on third-party crates:
  pathfinder geometry/color, sum_tree, markdown_parser, font-kit, tokio, etc.).
- `warpui` depends on `warpui_core` + OS/GPU crates (objc2-metal, cocoa,
  winit, wgpu, font-kit, cosmic-text, resvg).
- `ui_components` depends on `warpui_core` **and `warp_core`** (Subsystem: core
  app/terminal) for `Appearance` → the only UI crate coupled to the AGPL core.
- The real Warp app (the AGPL `warp_core` / workspace / terminal-pane views,
  and the `warp_cli` binary) is the *consumer*: it defines the concrete
  `View`s (workspace, terminal block list, command palette, etc.) and calls
  `AppBuilder`. Those live outside this subsystem but are built entirely on
  these traits.

---

## Marley relevance

> ⚠️ **STALE @ M15 — kept as Warp-reference only.** Every bullet below assumes Marley *extends WarpUI*
> (a `View`/`Element`/`ChildView`/`Presenter` panel that links `warp_core` for session/appearance state).
> Marley took **neither** WarpUI **nor** `warp_core`: it renders gpui-native and reimplemented the
> equivalent surfaces clean-room. The `warpui*`-is-MIT "license lever" therefore buys Marley nothing here.
> Read § Marley status @ M15 and § Provenance & licensing for the current picture.

- **UI-surface expansion (primary):** This subsystem *is* the surface. A custom
  Ignibyte panel = a new struct implementing `Entity + View` (`render` →
  `Box<dyn Element>`) and registered with `AppContext::add_window` (new window)
  or `AppContext::add_view` + a `ChildView` slot inside the existing workspace
  view (docked/side panel). The agentic-workflow visualization can be built
  either from stock elements (`Flex`/`Stack`/`Table`/`List`/`Rect`/`Text` +
  `ui_components` widgets) or, for bespoke graph/timeline drawing, a **custom
  `Element`** whose `paint` calls the imperative `Scene` API
  (`draw_rect_*`, `draw_glyph`, `with_border`, layers/z-index). Backing state
  goes in a `Model` (e.g. `AgentWorkflowModel`) that the panel `observe`s.
- **Session spawn / write / read:** The mechanism to spawn a PTY/session and
  stream output is **not in these crates** — it lives in `warp_core` / command
  / terminal subsystems (AGPL). But the *UI that triggers and visualizes it*
  belongs here: a panel view dispatches an `Action` (or calls a model method) to
  request a session, and consumes streamed output via
  `ViewContext::spawn_stream_local` → `notify()` → re-render. Wiring the panel
  to a terminal view is done with `ChildView<TerminalView>` /
  `transfer_view_tree_to_window`. Expect the panel→session bridge to cross the
  MIT↔AGPL line.
- **De-auth:** Login/auth is not implemented in WarpUI. These crates only
  provide the rendering of whatever auth UI exists (and `warpui_extras::secure_storage`
  for credential persistence). Removing the mandatory login and stubbing an
  Ignibyte login seam happens in the app/auth subsystem, then this layer just
  renders the replacement (or skips it).
- **Rebrand:** Window chrome (titlebar height, native decorations, transparency)
  is controlled via the `Window` trait here. Visual theming/colors come through
  `Appearance` into `ui_components` (AGPL) and the stock widget styles. Literal
  "Warp" strings are mostly in the app/asset layers, not in `warpui_core`'s
  trait definitions, but example asset folders and any default theme names
  should be audited. Keeping Warp credit while restyling is straightforward
  because branding is data (Appearance/assets), not baked into the framework.
- **License lever:** `warpui*` being clean MIT is a strategic asset — new
  framework-level capabilities (new elements, new panel infrastructure) can stay
  MIT as long as they don't pull in `warp_core`. Anything touching terminal/
  session/appearance state inherits AGPL.

---

## Key files (cite these)

- `crates/warpui_core/README.md` — authoritative "whirlwind tour" of the model.
- `crates/warpui_core/src/lib.rs` — public surface / re-exports.
- `crates/warpui_core/src/core/app.rs` — `App`, `AppContext`, window/view/model
  creation, action dispatch (the framework's spine).
- `crates/warpui_core/src/core/mod.rs` — `AnyView`, `Handle`, `EntityLocation`,
  `Effect`, `WindowInvalidation`.
- `crates/warpui_core/src/core/view/mod.rs` — `View` / `TypedActionView` traits.
- `crates/warpui_core/src/core/view/context.rs` — `ViewContext` (notify/observe/
  emit/spawn/add_view) = state-flow API.
- `crates/warpui_core/src/core/model/mod.rs`, `core/entity.rs` — models & `Entity`/`EntityId`.
- `crates/warpui_core/src/elements/gui/mod.rs` — `Element`, `ParentElement`,
  `Point`, `Axis`; gateway to the stock element library.
- `crates/warpui_core/src/elements/gui/child_view.rs` — view embedding bridge.
- `crates/warpui_core/src/presenter.rs` — layout/after_layout/paint/event pipeline.
- `crates/warpui_core/src/scene.rs` — retained `Scene` / `Layer` + imperative draw API.
- `crates/warpui_core/src/rendering/mod.rs` — backend-agnostic GPU config/device types.
- `crates/warpui/src/lib.rs` — re-export + platform/rendering modules.
- `crates/warpui/src/platform/app.rs` — `AppBuilder` entry point.
- `crates/warpui/src/platform/mod.rs` — per-target backend selection.
- `crates/warpui/src/platform/mac/rendering/renderer.rs` — `Renderer` trait, Metal `Device`.
- `crates/warpui/src/rendering/wgpu/` — cross-platform wgpu renderer + WGSL shaders.
- `crates/warpui_core/src/platform/mod.rs` — `Window`/`WindowContext`/`Delegate` traits.
- `crates/ui_components/src/lib.rs` — `Component` trait (AGPL, themeable widgets).
- `crates/warpui_core/src/ui_components/mod.rs` — MIT primitive widget set.
- `crates/warpui/examples/example-black-background-box/{main,root_view}.rs` — minimal app recipe.
- `crates/warpui_extras/src/lib.rs` — `secure_storage` + `user_preferences`.

## Open questions (round 2)

1. Where does the concrete top-level `WorkspaceView` / terminal-pane view tree
   live (presumably `warp_core` / a `workspace` crate), and what is the exact
   call site of `AppBuilder::run` in `warp_cli`? That is where a Marley panel
   must be slotted in.
2. What is the dock/split/panel layout system at the app level (does the
   workspace already support side panels, or must we add a split host element)?
3. How does a `TerminalView` consume PTY output today, and what model/handle
   would a workflow panel subscribe to in order to read session output and to
   write input — i.e. the exact MIT↔AGPL seam for session spawn/write/read.
4. Is the experimental wgpu-on-macOS renderer (`experimental-wgpu-renderer`)
   production-viable, or is Metal the only supported mac path? Affects how much
   GPU surface Marley can safely touch.
5. Where are literal "Warp" brand strings, default theme/`Appearance` values,
   and bundled assets defined, so rebrand can be scoped without touching the
   MIT framework traits?
6. How does `keymap` + the action registry map to a future Ignibyte command set,
   and can panel actions be registered without modifying core enums?
