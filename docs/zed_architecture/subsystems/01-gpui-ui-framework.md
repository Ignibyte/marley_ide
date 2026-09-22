# Subsystem 01 — gpui: UI Framework, Rendering, Text & Input

> Part of the Marley **Zed** architecture reference. Zed is the *editor* reference
> for Marley's editing surface (the counterpart to `../../warp_architecture/`, the
> terminal/cockpit reference). Unlike the rest of Zed, **the subject of this
> document is not GPL** — see the license note immediately below.

## Scope

This document deconstructs **gpui**, Zed's GPU-accelerated, immediate-mode UI
framework, as the reference for Marley's own rendering/editing substrate. It is
the single most important subsystem for Marley's editor work because **Marley
already depends on gpui directly** (`gpui = "0.2.2"` in `crates/marley_app/Cargo.toml`)
— so this is not a "reimplement the capability in our own code" analysis like the
other Zed subsystems. It is a **"stop hand-rolling what the dependency already
gives us"** analysis: Marley currently hand-rolls render + input on top of gpui's
lowest-level primitives (one `div` per line, un-virtualized; a hand-written
`apply_key`; a two-`div` `split_at_caret`; no IME; hand scroll-offset math at ~4
sites). Every section below maps a hand-rolled Marley mechanism onto the gpui
capability that should replace it, with sequencing.

### License posture — the KEY point `[gpui Apache-2.0]`

- **gpui is Apache-2.0**, declared explicitly in `crates/gpui/Cargo.toml`
  (`license = "Apache-2.0"`, `publish = true`, `homepage = "https://gpui.rs"`). It
  is a standalone, publishable crate that Zed happens to author — **it is NOT part
  of Zed's GPL-3.0 copyleft surface.**
- **Implication for Marley:** everything in this document tagged `[gpui Apache-2.0]`
  is freely usable by Marley *as a normal permissive dependency*, including in the
  proprietary "brain" layer. Adopting `StyledText`, `uniform_list`,
  `EntityInputHandler`, `KeyContext`, `ScrollHandle`, etc. does **not** pull any
  GPL obligation — Marley already links gpui and calls these APIs. This is
  categorically different from the editor/terminal *logic* Marley derives from
  Zed's GPL crates (`editor`, `text`, `language`, …), which stay in the GPL layer.
- Provenance tags used below:
  - `[gpui Apache-2.0]` — a gpui capability, freely adoptable (the vast majority).
  - `[Marley-original]` — Marley's own hand-rolled code (the thing being replaced),
    or a Marley-specific integration decision.
  - `[Zed-derived]` — a *pattern* observed in Zed's GPL consumer crates (e.g. how
    Zed's `editor` crate wires an `EntityInputHandler`); the pattern is instructive
    but any code Marley writes imitating Zed's *editor logic* is GPL-derived and
    stays in the GPL layer. The gpui *API* it calls is still Apache.

### The gpui crate family (`crates/gpui*`)

| Crate | Role |
|-------|------|
| `gpui` | The framework proper: `App`/`Entity`/`Context` ownership, `Element`/`Render` tree, `Window`, taffy layout, `text_system`, `input`, `key_dispatch`/`keymap`, `scene`, `elements/` (div, text, uniform_list, list, anchored, deferred, img, canvas, svg…). `path = "src/gpui.rs"`. |
| `gpui_macos` | macOS backend: **native Metal** renderer (`metal_renderer.rs`, `shaders.metal`), `metal_atlas.rs`, Cocoa window/events, `NSTextInputClient` IME bridge (`window.rs`), CoreText text system. |
| `gpui_wgpu` | Cross-platform **wgpu** renderer (`wgpu_renderer.rs`, `shaders.wgsl` + `shaders_subpixel.wgsl`), `wgpu_atlas.rs`, `cosmic_text_system.rs` (Linux/Windows/Web GPU + text). |
| `gpui_linux` / `gpui_windows` / `gpui_web` | Per-OS platform glue (windowing, dispatch, clipboard) atop `gpui_wgpu`. |
| `gpui_platform` | The `application()` entry point that selects the concrete platform. |
| `gpui_macros` | `#[derive(IntoElement)]`, `actions!`/`impl_actions!`, `Render` helpers, the `#[inspector]` feature. |
| `gpui_shared_string` / `gpui_util` / `gpui_tokio` | `SharedString` (cheap cloneable string), assorted utils (`FluentBuilder`, `Deferred`), Tokio interop. |

> **No blade.** Older gpui used `blade-graphics`; **this 0.2.x line does not** — the
> manifests contain zero `blade` references. The backends are **native Metal on
> macOS** (`metal` + `objc2-metal` crates) and **wgpu everywhere else**, with
> `etagere` (shelf bin-packing) as the sprite-atlas allocator on every backend.

---

## The mental model

gpui borrows the same lineage as WarpUI (Flutter/React-ish) but has since
converged on a **single unified entity model**. Three ideas:

1. **Entities** — *everything* stateful (what older gpui and WarpUI split into
   "models" and "views") is now a single `Entity<T>`. The one global `App` owns
   every entity; you hold an `Entity<T>` **handle** (a slotmap id + refcount, "an
   inert identifier plus a compile-time type tag"), and you can only touch the
   real `T` while you have an `&App`/`&mut App` (or a typed `Context<T>`), which
   the framework hands you at callbacks. `[gpui Apache-2.0]`
2. **Views are just entities that render.** An entity is a *view* when its type
   implements `Render` (`fn render(&mut self, &mut Window, &mut Context<Self>) -> impl IntoElement`).
   There is no separate `View<T>` type anymore — a "view" is `Entity<T> where T: Render`.
3. **Elements are throwaway, per-frame layout/paint primitives.** `render` returns
   an element tree; it is laid out by **taffy** (web flexbox/grid) and painted into
   a **retained `Scene`**, then the entire element tree + its registered callbacks
   are dropped before the next frame. "Immediate-mode render, retained-scene paint."

The reactive loop: reading an entity's state during render **auto-tracks** a
dependency; calling `cx.notify()` on an entity marks it dirty; on the next frame
the dirty entity's subtree is re-rendered while unchanged subtrees are **reused**
(cached element + reused shaped text + reused paint range). This is what makes an
un-virtualized "div per line" *tolerable* today but not *scalable* — see §Lists.

Key naming vs. the older WarpUI doc (so the two references line up):

| Concept | WarpUI / old gpui | **this gpui (0.2.x)** |
|---|---|---|
| Global owner | `AppContext` | **`App`** |
| Entity handle | `ViewHandle`/`ModelHandle` | **`Entity<T>` / `WeakEntity<T>`** |
| Per-entity ctx | `ViewContext`/`ModelContext` | **`Context<'a, T>`** (derefs to `App`) |
| Render trait | `View::render -> Box<dyn Element>` | **`Render::render -> impl IntoElement`** |
| Layout | custom constraint solve | **taffy** (flexbox/grid) |
| Window | implicit in ctx | **explicit `&mut Window`** threaded everywhere |

---

## 1. Ownership & reactivity — `App` / `Entity<T>` / `Context<T>` `[gpui Apache-2.0]`

Files: `crates/gpui/src/app.rs` (2886 lines), `app/context.rs`, `app/entity_map.rs`,
`subscription.rs`, `global.rs`, and the excellent prose in `_ownership_and_data_flow.rs`.

**The one owner.** `EntityId` is a `slotmap` key (`entity_map.rs:29`). The
`EntityMap` (`entity_map.rs:56`) holds `SecondaryMap<EntityId, Box<dyn Any>>` plus
atomic refcounts. The `App` owns the `EntityMap`; an `Entity<T>` handle is a
refcounted id (like `Rc`) that *only* yields access to the `T` when an `&App` is
present. `WeakEntity<T>` is the non-owning variant (`upgrade() -> Option<Entity<T>>`).

**Creating & accessing entities** (the `AppContext` trait, imported as `AppContext as _` in the prelude):
```rust
let counter: Entity<Counter> = cx.new(|cx: &mut Context<Counter>| Counter { count: 0 }); // app.rs:2540
counter.update(cx, |c: &mut Counter, cx: &mut Context<Counter>| { c.count += 1; cx.notify(); });
let n = counter.read(cx).count;                        // &T while you hold &App
```
`cx.new` / `reserve_entity` / `insert_entity` (`app.rs:2540/2556/2560`) are the
constructors; `Entity::update`/`read` are the accessors.

**`Context<'a, T>`** (`app/context.rs:20`) is `{ app: &mut App, entity_state: WeakEntity<T> }`
and `Deref`s to `App` — so a `Context<T>` is "an `App` that also knows which entity
you are." The reactive API lives here:
- `cx.notify()` (`context.rs:229`) — mark this entity dirty → schedule re-render of
  its subtree. This is *the* invalidation primitive.
- `cx.observe(&other, |self, other_handle, cx| …)` (`context.rs:63`) — run a callback
  whenever `other` calls `notify()`. Returns a `Subscription`.
- `cx.subscribe(&other, |self, other, &Event, cx| …)` (`context.rs:98`) — typed
  events; the emitter must `impl EventEmitter<Event>` (a marker trait, `gpui.rs:290`).
- `cx.emit(event)` (`context.rs:765`) — emit a typed event to subscribers.
- `cx.spawn(async move |cx| …) -> Task<R>` (`context.rs:237`) / `cx.spawn_in(&window, …)`
  (`context.rs:676`) — run a future on the foreground executor and fold the result
  back into entity state (this is how a panel consumes streaming subprocess output).
- `cx.observe_release`, `cx.on_next_frame` (`context.rs:151/292`).

**`Subscription`** (`subscription.rs`) — returned by every `observe`/`subscribe`;
`.detach()` keeps it alive forever, or hold it in a field and drop it to cancel.

**Globals** (`global.rs:22`): `pub trait Global: 'static {}`. `cx.global::<G>()`,
`cx.global_mut::<G>()`, `cx.set_global(g)`, `cx.observe_global::<G>(…)`
(`app.rs:1851/1867/1889/1914`) — app-wide singletons fetched by type (settings,
theme, registries).

**Async executors** (`executor.rs`): `BackgroundExecutor` (`:14`, thread pool,
`spawn(Send future)`) and `ForegroundExecutor` (`:22`, main thread, `spawn(!Send future)`),
both yielding `Task<R>` (a cancel-on-drop future). `.timer(duration)` for delays.

> **Marley adoption `[Marley-original]` → `[gpui Apache-2.0]`:** Marley already uses
> this model (its app/editor are entities; it uses `cx.notify()`, `.track_focus`).
> Nothing to replace here — this is the substrate the *other* adoptions plug into.
> The one thing to lean on harder is the **auto-invalidation + subtree reuse**: put
> editor/terminal state on entities and `notify()` precisely, so that when
> virtualization lands (see §Lists) gpui reuses the shaped text + paint of
> unchanged rows for free. Model streaming session output as an entity the pane
> `observe`s, and consume it via `cx.spawn`.

---

## 2. The Element lifecycle & layout (taffy) `[gpui Apache-2.0]`

Files: `element.rs` (760 lines), `taffy.rs` (781), `style.rs` (1525), `styled.rs`,
`elements/div.rs`, `view.rs`.

### 2a. The `Element` trait — a 3-phase frame lifecycle

Every visual primitive implements `Element` (`element.rs:51`). The lifecycle is
**request_layout → prepaint → paint**, with two associated state types carried
between phases:
```rust
pub trait Element: 'static + IntoElement {
    type RequestLayoutState: 'static;
    type PrepaintState: 'static;
    fn id(&self) -> Option<ElementId>;
    fn source_location(&self) -> Option<&'static panic::Location<'static>>;
    fn request_layout(&mut self, id, inspector_id, &mut Window, &mut App)
        -> (LayoutId, Self::RequestLayoutState);            // ask taffy for a box
    fn prepaint(&mut self, id, inspector_id, bounds, &mut req_state, &mut Window, &mut App)
        -> Self::PrepaintState;                              // commit bounds, hitboxes, input handlers
    fn paint(&mut self, id, inspector_id, bounds, &mut req_state, &mut prepaint_state, &mut Window, &mut App);
}
```
- **`request_layout`** returns a taffy `LayoutId`. Leaf content that measures itself
  (text) uses `window.request_measured_layout(style, measure_fn)` (`window.rs:4212`)
  so taffy calls back to shape/measure only when space changes.
- **`prepaint`** runs *after* taffy has computed absolute bounds. This is where an
  element commits its `Hitbox` (`window.insert_hitbox`, `window.rs:4269`), registers
  its input handler (`window.handle_input`, §Input), and — crucially — where
  **virtualization happens** (compute the visible index range, realize only those
  children).
- **`paint`** records draw primitives into the `Scene` via `window.paint_quad`,
  `paint_glyph`, `paint_path`, `paint_underline`, `paint_image`, etc. (§Platform),
  optionally inside `window.with_content_mask(bounds, …)` for clipping and
  `window.paint_layer(...)` for z-order.

The `Drawable<E>` wrapper (`element.rs:254`) drives an element through the phases
(`Start → RequestLayout → LayoutComputed → Prepaint → Painted`). Elements are
allocated in a **per-frame arena** (`ElementArenaScope`, `window.rs:2647`) and freed
en masse at end of frame.

### 2b. `Render` / `RenderOnce` / `IntoElement` / `View`

- **`Render`** (`element.rs:163`): `fn render(&mut self, &mut Window, &mut Context<Self>) -> impl IntoElement`.
  An `Entity<T: Render>` is a "view."
- **`RenderOnce`** (`element.rs:179`): `fn render(self, &mut Window, &mut App) -> impl IntoElement`.
  A *stateless component* — consumed by value; `#[derive(IntoElement)]` turns any
  `RenderOnce` into an element (the "components out of plain data" pattern).
- **`IntoElement`** (`element.rs:145`) / **`ParentElement`** (`.child`/`.children`,
  `element.rs:188`) — the fluent builder surface.
- **`View`** (`view.rs:175`) unifies them: `entity_id() -> Option<EntityId>` (identity)
  + `render(self, …)`. Blanket impls make `Entity<T: Render>` (id = the entity) and
  any `T: RenderOnce` (id = None) both `View`. **`ViewElement`** (`view.rs:233`) is
  the element that hosts a view in the tree — and it is the **reactive boundary**.

### 2c. The reactive boundary + caching (why re-render is cheap)

`ViewElement::request_layout`/`prepaint` (`view.rs:307/355`) wrap the view's render
in `window.with_rendered_view(entity_id, …)` and key a cache on
`(bounds, content_mask, text_style)` plus `window.dirty_views.contains(entity_id)`.
If the entity is **not** dirty and geometry is unchanged, it **reuses the prior
frame's prepaint/paint ranges** (`reuse_prepaint`/`reuse_paint`) instead of
re-rendering (`view.rs:379-394`). `cx.detect_accessed_entities` records which
entities were read during render, so reads auto-establish the invalidation set.
`Entity::cached(style)` / `AnyView::cached` (`view.rs:225/39`) opt a subtree into
full caching (frozen until `notify()`).

> **This is the single fact that makes Marley's current un-virtualized render
> survive:** unchanged lines' views are cache-reused. But reuse still walks O(n)
> elements per frame and holds O(n) live elements — virtualization (§Lists) removes
> the O(n), and cross-frame shaped-text reuse (§Text) removes the reshape cost.

### 2d. Layout = taffy (web flexbox/grid) + `Style` / `Styled`

gpui does **not** hand-roll layout — it embeds **taffy** (`taffy.rs:34`,
`TaffyLayoutEngine { taffy: TaffyTree<NodeContext> }`). `Style::to_taffy(rem_size, scale)`
(`taffy.rs:72`) converts a gpui `Style` to a taffy style; `request_layout` builds a
taffy node (leaf or with children); `request_measured_layout` builds a
self-measuring leaf (`taffy.rs:88`) — the text path. Rounding is disabled and gpui
does its own device-pixel rounding.

`Style` (`style.rs:182`) is a flat struct of web-ish fields: `display`, `overflow:
Point<Overflow>`, `position`, `inset`, `size`, `margin`, `padding`, `border_widths`,
`gap`, `flex_direction`/`flex_wrap`/`flex_basis`/`flex_grow`/`flex_shrink`,
`background: Option<Fill>`, `border_color`, `corner_radii`, plus a `TextStyle`.
`StyleRefinement` (via the `Refineable` derive) is the "patch" type used by hover/
focus/group styles.

**`Styled`** (`styled.rs:22`) is the Tailwind-like fluent trait every element gets:
`.flex()`, `.flex_col()`, `.w_full()`, `.p_2()`, `.gap_1()`, `.bg(color)`,
`.rounded_md()`, `.border_1()`, `.absolute()`, `.overflow_y_scroll()`, etc. **`div()`**
(`elements/div.rs`) is the workhorse container; `InteractiveElement` /
`StatefulInteractiveElement` (`div.rs:699/1213`) add `.id(…)` (→ `Stateful`),
`.track_focus(&h)`, `.key_context(…)`, `.hover(…)`, `.on_mouse_down(…)`,
`.on_click(…)`, `.on_action::<A>(…)`, `.track_scroll(&h)`, `.on_scroll_wheel(…)`.

> **Marley adoption `[Marley-original]`:** Marley already builds its UI from `div()`
> + `Styled` (confirmed: hundreds of `div().flex().flex_row()` sites in
> `crates/marley_app/src/app.rs`). Correct usage; no change. The lifecycle detail
> that matters is that **`prepaint` is where to virtualize** and where input
> handlers/hitboxes register — Marley's future editor element should be a real
> `impl Element` (like Zed's editor) that owns its `prepaint` to place the caret,
> register the `EntityInputHandler`, and realize only visible rows.

---

## 3. Text — `StyledText`, `.with_highlights`, `TextLayout`, shaping cache `[gpui Apache-2.0]`

Files: `elements/text.rs`, `text_system.rs`, `text_system/{line,line_layout,line_wrapper}.rs`.
This is the **headline replacement** for Marley's hand-rolled line rendering and its
`split_at_caret` cursor. Four layers:

### 3a. `StyledText` + the highlight API

`StyledText` (`text.rs:391`) = `SharedString` + `Vec<TextRun>` + a shared `TextLayout`.
A **`TextRun`** (`text_system.rs:987`) is `{ len: usize /*utf-8 bytes*/, font: Font,
color, background_color, underline, strikethrough }` — runs must tile the string
exactly; a run boundary is any font-or-decoration change.

Two highlight entry points (mutually exclusive):
```rust
StyledText::new(text)
  .with_default_highlights(&default_style, highlights)  // text.rs:418  eager, explicit base style
  .with_highlights(highlights)                          // text.rs:433  lazy, inherits window text style
// highlights: impl IntoIterator<Item = (Range<usize>, HighlightStyle)>
```
Both funnel through `compute_runs` (`text.rs:453`): walk the **sorted, non-overlapping**
highlight ranges; fill gaps with the default style; emit
`default_style.clone().highlight(hl).to_run(range.len())` for each span. So
**highlights are sparse** and gaps auto-fill. `HighlightStyle` (`style.rs:576`) is an
all-`Option` overlay (`color` alpha-blends, others override). `TextStyle::to_run(len)`
(`style.rs:555`) flattens a style into a run.

**`InteractiveText`** (`text.rs:959`) wraps a `StyledText` with `.on_click(ranges, cb)`
(`text.rs:1000`), `.on_hover(cb)` (`:1019`), `.tooltip(builder)` (`:1028`) — all driven
by hit-testing pixel→byte via `TextLayout::index_for_position`. This is the machinery
for clickable paths/URLs/error spans in terminal output.

### 3b. `TextLayout` — the pixel↔index hit-testing Marley lacks entirely

`TextLayout` (`text.rs:614`) is a cheap `Rc<RefCell<Option<TextLayoutInner>>>` handle
shared by the element and every hit-tester, so an event handler sees the *same*
geometry that was painted. The three load-bearing methods:
```rust
pub fn index_for_position(&self, position: Point<Pixels>) -> Result<usize, usize>  // text.rs:808
pub fn position_for_index(&self, index: usize)            -> Option<Point<Pixels>>  // text.rs:842
pub fn line_layout_for_index(&self, index: usize)         -> Option<Arc<WrappedLineLayout>> // text.rs:873
```
- `index_for_position` returns **`Ok(byte)`** when the point lands inside the glyphs,
  **`Err(byte)`** when it clamps to the nearest slot (above → `Err(0)`, past a line's
  end → clamped, below all → `Err(last)`). Use `.ok()` for "real hit only"
  (hover/tooltip); accept the `Err` value for "always give me the nearest caret slot"
  (click-to-place-caret).
- Per-visual-line geometry lives on `WrappedLineLayout`/`LineLayout`
  (`line_layout.rs`): `index_for_x` (`:58`), `closest_index_for_x` (`:75`, the ↑/↓
  column-preserving primitive), `x_for_index` (`:105`), `font_id_for_index` (`:117`,
  which fallback font actually drew a char). All in **byte indices**, glyph-accurate
  (proportional fonts, ligatures, combining marks) — things Marley's column
  arithmetic can't represent.

### 3c. `ShapedLine::split_at` — the exact 1:1 replacement for `split_at_caret`

`ShapedLine::split_at(byte_index) -> (ShapedLine, ShapedLine)` (`line.rs:141`) splits
a shaped line at a byte offset (`x` from `x_for_index`), rebasing the suffix's glyph
positions and byte indices to 0 and dividing straddling decoration runs. Widths sum
exactly. **Marley's two-`div` `split_at_caret`/`split_caret_char` maps onto: shape the
row once → `split_at(caret_byte)` → paint prefix, block cursor at `x_for_index(caret)`
(width = `x_for_index(caret+1) - x_for_index(caret)` or one cell), rebased suffix.**
Glyph-accurate where the string split is not.

### 3d. Shaping pipeline + the cross-frame `LineLayoutCache`

- **`TextSystem`** (`text_system.rs:51`, process-wide `Arc`) owns font resolution
  (`resolve_font`, `:148`, with a family fallback stack), metrics, glyph raster-bounds
  cache, and a `LineWrapper` pool. **`WindowTextSystem`** (`:364`, per window) adds
  the frame-scoped `LineLayoutCache`.
- Three shaping entry points on `WindowTextSystem`:
  - `shape_line(text, size, runs, force_width) -> ShapedLine` (`:397`) — one line, no
    wrap. **`force_width: Some(cell_width)` snaps glyphs to a monospace cell grid**
    (`apply_force_width_to_layout`, `line_layout.rs:787`, correct for zero-advance
    combining marks) — the terminal-grid primitive.
  - `shape_text(text, size, runs, wrap_width, line_clamp) -> SmallVec<[WrappedLine;1]>`
    (`:509`) — multi-line, soft-wrap. Used by `StyledText`.
  - `layout_line(...) -> Arc<LineLayout>` (`:645`) plus hash-keyed variants
    (`layout_line_by_hash`, `:787`) that key the cache on a `u64` content hash and
    only materialize the string on miss — ideal for a terminal grid rope.
- **`LineLayoutCache`** (`line_layout.rs:392`) keeps `previous_frame` + `current_frame`
  maps keyed on `(text, font_size, runs, wrap_width, force_width)`. A line shaped last
  frame is reused this frame by an `Arc` clone. `layout_index()` + `reuse_layouts(range)`
  bulk-promote a contiguous span of last frame's shaped lines (wired into the window's
  subtree reuse: `window.rs:3093/3116`). **Unchanged rows cost an Arc clone, not a reshape.**
- **Glyph→GPU:** `paint_line` (`line.rs:334`) walks glyphs and calls
  `window.paint_glyph`/`paint_emoji`; `Window::paint_glyph` (`window.rs:3871`) quantizes
  to subpixel bins, fetches/rasterizes into the sprite atlas on miss, and pushes a
  `MonochromeSprite`/`SubpixelSprite` into the scene (§Platform). Subpixel, atlas-cached,
  for free.

### 3e. Line wrapping

`LineWrapper` (`line_wrapper.rs:17`, width-budget, pre-shape, `wrap_line`/`truncate_line`)
and `LineLayout::compute_wrap_boundaries` (`line_layout.rs:128`, glyph-accurate,
post-shape) produce `WrapBoundary { run_ix, glyph_ix }` (`:225`) marking where a visual
row starts. Hit-testing and paint both understand wrap boundaries, so soft-wrap "just
works" via `shape_text(.., Some(wrap_width), ..)`; a hard-wrapped terminal passes
`wrap_width: None` to skip wrapping.

> **Marley adoption sequence `[Marley-original]` → `[gpui Apache-2.0]`:**
> 1. Render editor/terminal lines through `StyledText` + `TextLayout` (build `TextRun`s
>    from existing style spans via the sparse-highlight pattern) — correct shaping,
>    ligatures, per-glyph fallback, atlas glyphs. (Keep div-per-line at first.)
> 2. Wire **click-to-place-caret** (`index_for_position`, accept `Err` clamp) and
>    **caret geometry** (`position_for_index` + `x_for_index`) — capability Marley has
>    *none* of today.
> 3. Replace the two-`div` `split_at_caret` with `ShapedLine::split_at`.
> 4. Route terminal rows through `shape_line(.., force_width: Some(cell_width))`.
> 5. Adopt cross-frame `LineLayoutCache` reuse (+ real virtualization, §Lists);
>    `layout_line_by_hash` keys on a grid-line hash without materializing strings.
> Caveat: `TextLayout` accessors panic before prepaint and it's `Rc<RefCell>`
> (single-thread, per-window) — hit-test inside element event handlers, as
> `InteractiveText` does.

---

## 4. Input & IME — `EntityInputHandler` / `Window::handle_input` / `UTF16Selection` `[gpui Apache-2.0]`

Files: `input.rs`, `platform.rs` (`InputHandler`/`PlatformInputHandler`/`UTF16Selection`),
`window.rs` (`handle_input`), `gpui_macos/src/window.rs` (the `NSTextInputClient` bridge),
`examples/input.rs` (the canonical editor template). This replaces Marley's hand-rolled
`apply_key` and adds the IME Marley wholly lacks.

### 4a. The three-layer bridge

1. **`EntityInputHandler`** (`input.rs:10`) — implemented by the *editor entity itself*;
   methods take `&mut Context<Self>`.
2. **`ElementInputHandler<V>`** (`input.rs:100`) = `{ view: Entity<V>, element_bounds }`;
   forwards each call into `view.update(cx, |v, cx| v.method(...))`. Built in `paint`.
3. **`InputHandler`** (`platform.rs:1524`, "1:1 with NSTextInputClient") + the
   OS-facing **`PlatformInputHandler`** (`platform.rs:1298`, holds an
   `AsyncWindowContext`) which the platform's `extern "C"` IME callbacks invoke.

**Registration (per frame, focus-gated):** the editor element's `paint` calls
```rust
window.handle_input(&focus_handle, ElementInputHandler::new(bounds, entity.clone()), cx); // window.rs:4365
```
`handle_input` gates on `focus_handle.is_focused()` and stashes the handler; at end of
frame gpui installs the last one via `platform_window.set_input_handler(...)`. The
handler is **rebuilt every frame and only while focused** — never cache it across focus.

### 4b. `EntityInputHandler` — the methods

```rust
fn text_for_range(range, &mut adjusted_range, win, cx) -> Option<String>          // substring for a UTF-16 range
fn selected_text_range(ignore_disabled, win, cx)       -> Option<UTF16Selection>  // current selection/caret
fn marked_text_range(win, cx)                          -> Option<Range<usize>>    // IME preedit span (Some == composing)
fn unmark_text(win, cx)                                                           // cancel composition
fn replace_text_in_range(range, text, win, cx)                                    // COMMIT text (the real "type a char")
fn replace_and_mark_text_in_range(range, new_text, new_sel, win, cx)              // set/update preedit
fn bounds_for_range(range_utf16, element_bounds, win, cx) -> Option<Bounds<Pixels>> // caret/preedit rect (IME candidate window)
fn character_index_for_point(point, win, cx)             -> Option<usize>         // hit-test point -> UTF-16 offset
// + defaulted: set_selected_text_range, text_length_utf16, accepts_text_input
```
(`input.rs:12-95`.) Each maps 1:1 to an `NSTextInputClient` selector
(`gpui_macos/src/window.rs:242-292` / `2681-2863`).

### 4c. `UTF16Selection` and why UTF-16

`UTF16Selection { range: Range<usize>, reversed: bool }` (`platform.rs:1511`). **All
offsets at this boundary are UTF-16 code units** because every OS text API speaks
UTF-16 (`NSRange`, Windows TSF/IMM, mobile `UITextInput`) — gpui keeps the boundary
UTF-16 so platform code is a pass-through and pushes conversion into the handler, which
knows the buffer's real encoding. `reversed` says which end is the caret (a `Range`
can't). The handler owns `offset_from_utf16`/`offset_to_utf16` + grapheme-boundary
movement (`examples/input.rs:210-261`).

### 4d. Marked text (composition) + caret geometry

`bounds_for_range(caret..caret)` returns the caret rectangle in element coords, used
both to place the OS IME candidate palette (`firstRectForCharacterRange:`,
`gpui_macos/src/window.rs:2697`) **and** to draw the app's own caret — so it *also*
replaces `split_at_caret`'s geometry. The composition lifecycle: first keystroke →
`[inputContext handleEvent:]` → `setMarkedText:` → `replace_and_mark_text_in_range`
(insert provisional text, mark it, render underlined) → `insertText:` →
`replace_text_in_range` (commit) → mark cleared. `marked_text_range().is_some()` is
consulted on every keystroke to decide IME-first vs keybinding-first;
`prefers_ime_for_printable_keys` is the per-mode knob (true for a text editor pane,
false for a terminal that wants raw bytes).

> **Marley adoption sequence `[Marley-original]` → `[gpui Apache-2.0]`, `[Zed-derived]` pattern:**
> Model the editor entity on `examples/input.rs` (`content: Rope`, `selected_range:
> Range<usize>` in bytes, `selection_reversed`, `marked_range: Option<Range<usize>>`).
> 1. `impl EntityInputHandler` for the editor entity; register via `window.handle_input`
>    in `paint`, focus-gated.
> 2. Route real typing through `replace_text_in_range`; **delete `apply_key`'s text
>    path**. Keep only command keys (arrows, chords, cmd-w) on the action layer
>    (§Focus) — un-consumed keys re-dispatch via `doCommandBySelector:`.
> 3. Port UTF-16 conversion + grapheme movement — this *also fixes latent multibyte
>    caret bugs* Marley has even before IME.
> 4. `bounds_for_range` from the shaped line → one caret `PaintQuad` (retires
>    `split_at_caret`) and correct IME rects.
> 5. `character_index_for_point` for mouse click-to-caret.
> 6. The composition trio (`replace_and_mark_text_in_range` + `marked_text_range` +
>    `unmark_text`) + underline the preedit → real IME (dead keys, CJK, accents) —
>    net-new. `prefers_ime_for_printable_keys` per pane mode.
> Each stage is testable via `editor.handle_input("…")` without a live IME. The
> *shape* of this wiring is `[Zed-derived]` (Zed's `editor` does exactly this); the
> gpui API is Apache.

---

## 5. Lists & virtualization — `uniform_list` and `list` + `ListState` `[gpui Apache-2.0]`

Files: `elements/uniform_list.rs`, `elements/list.rs`. **Virtualization is not a
widget, it's a discipline done in `prepaint`: compute the visible index range, then
realize only that window of children.** gpui ships two tools that map onto Marley's two
surfaces:

### 5a. `uniform_list` — fixed row height (→ the terminal grid)

```rust
pub fn uniform_list<R: IntoElement>(
    id: impl Into<ElementId>, item_count: usize,
    f: impl 'static + Fn(Range<usize>, &mut Window, &mut App) -> Vec<R>,
) -> UniformList                                                    // uniform_list.rs:22
```
You pass a total count and a closure rendering **only a requested index range**. It
**measures one item** (`measure_item`, `:658`) to get the row height, sets
`content_size.height = item_height * item_count`, computes
`visible = floor(-offset/h) .. ceil((-offset+height)/h)`, and realizes only those rows,
clipped with `with_content_mask`. Cost O(visible rows), independent of `item_count`.
**`UniformListScrollHandle`** (`:80`): `scroll_to_item(ix, ScrollStrategy)` (`:150`,
`Top|Center|Bottom|Nearest`), `scroll_to_bottom()` (`:252`), `is_scrolled_to_end()`
(`:241`). Attach via `.track_scroll(&handle)` (`:683`); `.y_flipped(true)` for
chat/log order. `UniformListDecoration` (`:580`) overlays (indent guides) spanning the
virtual content.

### 5b. `list` + `ListState` — variable row height (→ the editor / Warp blocks)

```rust
pub fn list(state: ListState, render_item: impl FnMut(usize, &mut Window, &mut App) -> AnyElement + 'static) -> List // list.rs:24
ListState::new(item_count, alignment: ListAlignment, overdraw: Pixels)                                              // list.rs:314
```
For rows whose height isn't known until measured (wrapped/folded editor lines, variable
command blocks). The virtualization index is a **`SumTree<ListItem>`** (`list.rs:65`);
each `ListItem` is `Unmeasured|Measured` and contributes a `ListItemSummary`
(`{count, height, has_unknown_height, …}`), so the tree root knows total count **and**
total pixel height, and you can **seek by item index OR by pixel offset in O(log n)**
(`Count`/`Height` dimensions, `:1668/:1678`). Items are measured lazily as they enter
the viewport (+`overdraw`). Scroll position is a **logical** `ListOffset { item_ix,
offset_in_item }` (`:1420`) so inserting/removing rows above the viewport doesn't jump.
API: `splice(old_range, count)` (`:503`, incremental SumTree patch on each edit),
`scroll_to_reveal_item` (`:664`), `logical_scroll_top` (`:560`), `set_follow_mode(Tail)`
(`:617`, auto-stick-to-bottom for streaming), `ListAlignment::{Top,Bottom}` (`:164`),
scrollbar integration (`set_offset_from_scrollbar`, `:753`).

**Contrast:** `uniform_list` = one measure + arithmetic, lightweight handle; `list` =
lazy measure + persistent `ListState` you own on the view + `splice` discipline, but
handles varying heights, bottom-alignment, tail-follow, jump-free offsets. Pick per
surface by whether row height is constant.

> **Marley adoption `[Marley-original]` → `[gpui Apache-2.0]`:** Marley renders one
> live `div` per line **un-virtualized** (O(n) elements) with hand `visible_range` /
> `scroll_code` integer math (`crates/marley_app/src/code_view.rs`).
> - **Terminal grid → `uniform_list`** (uniform `line_height`, closure fed from the
>   alacritty grid). Deletes the per-line loop *and* the #198 stick/jump-to-bottom
>   hand-rolling (`scroll_to_bottom()` + `is_scrolled_to_end()`).
> - **Editor → `list` + `ListState`** (wrapped/folded → variable height). `splice` on
>   each edit; `logical_scroll_top` as the persisted, jump-free scroll position.
> - **Warp-block model (if pursued):** `list` of blocks (`Bottom` + `Tail`) nesting
>   `uniform_list` rows per block.
> Known escape hatch `[Zed-derived]`: Zed's *production* editor outgrew `list` and
> hand-writes a custom `Element` (same prepaint-only-visible discipline, sub-line
> precision) — so `list` is the right **baseline**, "graduate to a custom Element"
> the known next step.

---

## 6. Scrolling — `ScrollHandle` / `overflow_y_scroll` `[gpui Apache-2.0]`

For **non-virtualized** scroll regions (sidebars, palette, dialogs). `div()` becomes
scrollable via `.overflow_scroll()` / `.overflow_x_scroll()` / `.overflow_y_scroll()`
(`div.rs:1397/1404/1410`) — each just sets `Overflow::Scroll`. The scroll offset is an
`Rc<RefCell<Point<Pixels>>>` owned either by a tracked **`ScrollHandle`** (`div.rs:3872`,
via `.track_scroll(&h)`, `:1416`) or by per-element window state. Wheel handling
(`paint_scroll_listener`, `:3035`) converts wheel delta → pixels, mutates the shared
offset, `cx.notify()`; **clamping** to `[-scroll_max, 0]` and content-mask clipping
happen next frame in prepaint (`:2209-2249`). `ScrollHandle` API: `offset()`/`set_offset()`
(`:3887/:4023`), `scroll_to_item(ix)` (`:3945`), `scroll_to_bottom()` (`:4015`),
`bounds()`/`bounds_for_item()`. `ScrollAnchor` (`:3819`) scrolls a non-immediate
descendant into view.

> **Marley adoption:** Marley tracks pixel offsets manually at ~4 sites with no
> reusable abstraction. Adopt **one `ScrollHandle` per chrome scroll region** (owns
> the offset cell, wheel, clamp, clip) + `UniformListScrollHandle` (terminal) +
> `ListState` (editor). The 4 bespoke offset sites collapse to these three handles.

---

## 7. Overlays — `anchored` and `deferred` `[gpui Apache-2.0]`

Files: `elements/anchored.rs`, `elements/deferred.rs`.

- **`anchored()`** (`anchored.rs:27`) — position a child (menu/popover/tooltip) at a
  point and **auto-avoid overflowing the window**: `.anchor(Corner)`, `.position(pt)`,
  `.offset(pt)`, `.snap_to_window()` / `.snap_to_window_with_margin(edges)`, fit mode
  `SwitchAnchor` (flip corner) vs `SnapToWindow`. Children should have no margin.
- **`deferred(child)`** (`deferred.rs:7`) — **lay the child out in place but paint it
  *after* all ancestors**, so it draws on top; `.with_priority(n)` orders deferred
  elements. This is how popovers/menus/drag images escape their parent's paint order
  without leaving the layout tree.

Related: `ManagedView` (`window.rs`) = `Focusable + EventEmitter<DismissEvent> + Render`
— the modal/popover/menu contract; emit `DismissEvent` to close.

> **Marley adoption:** the command palette, completion popup, ghost-text menus, and
> tooltips should be `anchored()` + `deferred()` (auto-fit + correct z-order) rather
> than hand-placed absolute `div`s. Marley's overlay cards (#221) and palette are the
> natural first users.

---

## 8. Focus, KeyContext & Actions — the editor-vs-terminal ⌘D fix `[gpui Apache-2.0]`

Files: `key_dispatch.rs` (1195 lines), `keymap.rs`, `keymap/{context,binding}.rs`,
`action.rs`, `window.rs`, `elements/div.rs`, `app.rs`, `app/context.rs`.

**The mental model.** Every frame, during *paint*, gpui builds a **`DispatchTree`** — a
flat `Vec<DispatchNode>` (`key_dispatch.rs:82`) where each node optionally carries a
`KeyContext`, a `FocusId`, and lists of action/key listeners. A `div` contributes its
context via `.key_context(...)`, focusability via `.track_focus(&handle)`, handlers via
`.on_action(...)`. On a keystroke: compute the **dispatch path** (root → focused node),
collect the **stack of `KeyContext`s** along it, ask the **`Keymap`** which binding
matches *this keystroke under this context stack*, and dispatch the resulting `Action`
back along the path to the focused pane's listener. Same physical key → different action
purely as a function of *what is focused* and *what `key_context` its path carries*.

### 8a. Focus

- **`FocusId`** — a `slotmap` key, globally unique per focusable element
  (`window.rs:265`). **`FocusHandle`** (`window.rs:383`) = `{ id: FocusId, handles:
  Arc<FocusMap>, tab_index, tab_stop }`, refcounted (clone bumps the count).
- `cx.focus_handle()` (`app.rs:2444`) mints one; a view stores it and returns the *same*
  handle every frame. `.track_focus(&handle)` (`div.rs:719`) makes a div focusable +
  applies focus styles; during prepaint it registers the `FocusId → DispatchNodeId`
  mapping. **`Focusable`** (`window.rs:557`): `fn focus_handle(&self, &App) -> FocusHandle`.
- Query: `handle.is_focused(window)` (`window.rs:354`), `contains_focused` /
  `within_focused` (`window.rs:360/368`). Move: `window.focus(&handle, cx)`
  (`window.rs:1908`, also **clears pending multi-key input**), `blur`, `focus_next`/`focus_prev`.
- **Focus path** = `DispatchTree::focus_path(focus_id)` (`key_dispatch.rs:574`), the
  `[root … leaf]` chain. Each frame gpui diffs previous vs current path
  (`window.rs:2698-2728`) and fires `on_focus_in` (`window.rs:4446`) / `on_focus_out`
  (`window.rs:4466`).

### 8b. KeyContext — the cmd-D disambiguation

`KeyContext(Vec<ContextEntry>)` (`keymap/context.rs:10`); each `ContextEntry` is a bare
identifier (`Editor`) or a key-value (`mode = full`). `.key_context("Editor")`
(`div.rs:761`) attaches one (via `&str: TryInto<KeyContext>` → `KeyContext::parse`). Nodes
form a parent chain, so the **context stack** for a focused leaf is `[Workspace{},
Pane{}, Editor{}]` in root→leaf order (`window.rs:5273`, `key_dispatch.rs:453`).

The predicate grammar — **`KeyBindingContextPredicate`** (`keymap/context.rs:172`):
`Identifier` (`Editor`), `Equal`/`NotEqual` (`mode == full`), `Descendant` (`Workspace >
Editor`), `Not` (`!vim_mode`), `And` (`&&`), `Or` (`||`). Evaluation:
`predicate.depth_of(contexts) -> Option<usize>` (`keymap/context.rs:260`) returns the
**deepest** prefix depth at which the predicate holds. `Keymap::bindings_for_input(input,
context_stack)` (`keymap.rs:164`) returns matching bindings, and `binding_enabled`
(`keymap.rs:245`) is `predicate.depth_of(...)` or `Some(stack.len())` for a **predicate-less
(global) binding**. The precedence sort (`keymap.rs:187`) is
`depth_b.cmp(depth_a).then(ix_b.cmp(ix_a))` — **deeper context wins**, ties break to the
later-added (user) keymap. `NoAction`/`Unbind` entries let a deeper/later scope *disable*
an inherited binding.

**Worked ⌘D example (three panes):**
```rust
div().key_context("Workspace")
  .child(editor_div)    // .track_focus(&editor.focus).key_context("Editor").on_action(cx.listener(Editor::duplicate_line))
  .child(terminal_div)  // .track_focus(&term.focus).key_context("Terminal").on_action(cx.listener(Terminal::duplicate_input))
cx.bind_keys([                                                          // app.rs:2069
  KeyBinding::new("cmd-d", DuplicateLine,  Some("Editor")),
  KeyBinding::new("cmd-d", DuplicateInput, Some("Terminal")),
  KeyBinding::new("cmd-shift-p", ToggleCommandPalette, None),          // context-less ⇒ fires everywhere
]);
```
- **Editor focused** → stack `[Workspace, Editor]`; `Editor` predicate → `depth 2`,
  `Terminal` → `None` (filtered). Result = `DuplicateLine`.
- **Terminal focused** → `Terminal` → `depth 2`, `Editor` → `None`. Result =
  `DuplicateInput`. Same key, different action, **zero manual branching.**
- The predicate-less `cmd-shift-p` matches at `depth = stack.len()` regardless of focus.

### 8c. Actions

`Action` (`action.rs:117`) — identity is by `TypeId`; listeners keyed by `TypeId`.
Declared via `actions!(marley, [SplitRight, CloseTab, DuplicateLine])` (`action.rs:24`)
for unit actions, or `#[derive(Action)] #[action(namespace = editor)]` on a struct for
**data-carrying** actions. Registration is automatic via `inventory`; keymaps load
actions **by name** via `ActionRegistry::build_action(name, params)` (`action.rs:351`) →
this is how a TOML/JSON keymap file binds. Handle with `.on_action::<A>(|a, win, cx|
…)` (`div.rs:1014`, bubble phase) or `.capture_action` (capture phase), almost always
wrapped `cx.listener(...)` (`app/context.rs:252`) to get `&mut self` view state:
`.on_action(cx.listener(Editor::duplicate_line))`. Dispatch without a keystroke:
`window.dispatch_action(action, cx)`; enumerate reachable actions:
`window.available_actions(cx)` (`window.rs:5284`).

> ⚠️ **Correction vs. the task brief: this gpui revision has NO `impl_actions!` macro**
> (grepped `crates/gpui/src` — only `actions!` exists). Data-carrying actions use
> `#[derive(Action)]`. Plan around the derive, not `impl_actions!`.

### 8d. Dispatch flow

`Window::dispatch_keystroke` (`window.rs:4505`) → `dispatch_key_event` (`window.rs:4736`):
locate focus node + `dispatch_path`; match via `dispatch_tree.dispatch_key(...)`
(`key_dispatch.rs:483`) which collects the context stack and calls `bindings_for_input`;
**multi-stroke chords** (a prefix like `cmd-k …`) are stashed with a **1-second timeout +
replay** (`window.rs:4815-4866`) — gpui's built-in chord engine; dispatch matched
bindings, stopping at the first that consumes; else fall through to raw `on_key_down`
listeners and finally the IME/input handler. Action propagation
(`dispatch_action_on_node_inner`, `window.rs:5061`) runs **capture** root→leaf then
**bubble** leaf→root, and a bubble handler stops propagation by default
(`cx.propagate_event = false`, `window.rs:5129`) — so the innermost (leaf-most) pane
wins.

> **Marley adoption `[Marley-original]` → `[gpui Apache-2.0]` (the payoff — retires
> `apply_key` + the hand-rolled chord/disambiguation logic):**
> 1. **FocusHandle per leaf pane** — store `cx.focus_handle()`, add
>    `.track_focus(&handle)`, route pane activation through `window.focus(&handle, cx)`.
>    Immediately repoint the dim-on-unfocused (#220) / per-pane-render (#186) gates at
>    `handle.is_focused(window)` and delete the hand-tracked active-pane flag.
> 2. **`actions!(marley, [...])`** for existing verbs (additive).
> 3. **`.key_context("Terminal"/"Editor"/"CommandPalette")`** per pane (+ outer
>    `"Workspace"`).
> 4. **Register `KeyBinding`s with predicates via `cx.bind_keys`, delete `apply_key`.**
>    Global chords stay context-less; per-pane chords get `Some("Editor")` etc. ⌘D/⌘W
>    disambiguation falls out of the dispatch tree. **Remove Marley's hand-rolled chord
>    state machine — gpui already does multi-stroke + 1 s timeout + replay**; just feed
>    keystrokes through `dispatch_keystroke`.
> 5. **Command palette → `window.available_actions(cx)`** (context-aware list, for free)
>    + `window.dispatch_action(action, cx)` instead of a hand-maintained command table.
> 6. **(Optional) user keymaps** via `ActionRegistry::build_action(name, params)` fed
>    from Marley's TOML settings layer; `NoAction`/`Unbind` for overrides.
> The *shape* of this wiring (context names, which action per pane) is a Marley design
> choice `[Marley-original]`; the gpui mechanism is Apache.

---

## 9. The platform & GPU layer — Scene → native Metal / D3D11 / wgpu `[gpui Apache-2.0]`

Files: `scene.rs` (915 lines), `bounds_tree.rs`, `platform.rs` (2723 lines),
`gpui_platform/src/gpui_platform.rs`, `gpui_macos/src/{metal_renderer,metal_atlas}.rs`
+ `shaders.metal`, `gpui_wgpu/src/{wgpu_renderer,wgpu_atlas,cosmic_text_system}.rs`.

> **Headline finding (verified by grep across the whole workspace + `Cargo.lock`):
> there is NO `blade` / `blade-graphics` anywhere.** The "gpui uses blade" folklore is
> **false for 0.2.2.** gpui ships **three separate native renderers**, one `Scene`
> contract, chosen per-OS.

### 9a. The `Scene` — the clean CPU→GPU boundary

`paint` never touches the GPU; it pushes primitives into a flat, retained `Scene`
(`scene.rs:41`) of parallel `#[repr(C)]` vecs the renderer memcpy's into GPU instance
buffers. **Eight primitive types** (`Primitive`, `scene.rs:222`):

| Primitive | Use | scene.rs |
|---|---|---|
| `Quad` | rects w/ background, border, corner radii — the universal box | `:501` |
| `Shadow` | blurred drop/inset shadows | `:540` |
| `Path<ScaledPixels>` | filled vector paths (quadratic béziers, Loop-Blinn, CPU-tessellated) | `:755` |
| `Underline` | straight or `wavy` underline/strikethrough | `:521` |
| `MonochromeSprite` | atlas tile as alpha mask, tinted — **glyphs (grayscale AA)** + SVGs | `:677` |
| `SubpixelSprite` | glyphs w/ LCD subpixel AA — **`unreachable!()` on macOS** | `:696` |
| `PolychromeSprite` | full-color atlas tile — **emoji + images** | `:715` |
| `PaintSurface` | platform video surface (`CVPixelBuffer` on macOS, YUV→RGB in-shader) | `:734` |

`Window::paint_quad/paint_shadows/paint_path/paint_underline/paint_glyph/paint_emoji/
paint_svg/paint_image/paint_surface` (`window.rs:3763-4148`) each build one primitive and
call `Scene::insert_primitive` (`scene.rs:87`), which **clips to the content mask**, drops
if empty, assigns a **draw order**, and appends to the matching typed vec + a replay log
(`Scene::replay`, `:141`, re-emits a cached subtree's primitives without re-running the
element — the basis of frame caching). The Scene is double-buffered (`rendered_frame`/
`next_frame`, `window.rs:2700`).

**Z-order + batching (why it's cheap):** draw order comes from a `BoundsTree`
(`bounds_tree.rs`, an R-tree variant); `insert(bounds) -> u32` (`:120`) returns `1 +
max(order of already-inserted bounds that intersect)`, so **non-overlapping primitives
get independent orders (batchable regardless of paint sequence)** while overlapping ones
keep painter's order. `Scene::finish()` (`:151`) sorts each vec by `order` (sprites by
`(order, tile_id)` to cluster same-texture tiles); `Scene::batches()` (`:172`) is a
**k-way merge** yielding `PrimitiveBatch`es (`:477`), each of which becomes **exactly one
instanced GPU draw call**. A full screen of one-font text collapses to a handful of draws.

### 9b. The handoff + backend selection

`Window::present` (`window.rs:2774`) calls `self.platform_window.draw(&self.rendered_frame.scene)`
(`platform.rs:759`) — the single CPU→GPU line, forwarded to the per-OS renderer. Backend
choice lives in the thin `gpui_platform` umbrella: `current_platform(headless)`
(`gpui_platform.rs:35`) is a `#[cfg]` ladder → `gpui_macos::MacPlatform` /
`gpui_windows::WindowsPlatform` / `gpui_linux::current_platform` (which then picks
Wayland/X11/Headless at **runtime** via `guess_compositor`) / `gpui_web::WebPlatform`.

| Platform | Crate | GPU API | Shaders | Text |
|---|---|---|---|---|
| **macOS** (Marley's live path) | `gpui_macos` | **native Metal** (`metal`) | `shaders.metal` → `.metallib` | CoreText |
| **Windows** | `gpui_windows` | **native Direct3D 11** (`windows`) | `shaders.hlsl` (ClearType) | DirectWrite |
| **Linux/FreeBSD** | `gpui_linux`→`gpui_wgpu` | **wgpu 29.0.4** | `shaders.wgsl` + `shaders_subpixel.wgsl` | cosmic-text/swash |
| **Web/wasm** | `gpui_web`→`gpui_wgpu` | **wgpu 29.0.4** | same | same |

Core platform traits (`platform.rs`): `Platform` (`:125`, app-wide OS handle:
executors, `text_system`, `open_window`, clipboard, menus, keychain), `PlatformWindow`
(`:717`: `draw(&Scene)`, `sprite_atlas`, `is_subpixel_rendering_supported`,
`on_request_frame` [vsync], `set_input_handler`), `PlatformDisplay` (`:282`),
**`PlatformDispatcher`** (`:912`, the **executor bridge** — `dispatch` → background pool,
`dispatch_on_main_thread` → OS run-loop; on macOS bridges to GCD), `PlatformTextSystem`
(`:942`: `font_id`/`rasterize_glyph`/`layout_line`), `PlatformAtlas` (`:1194`).

### 9c. macOS Metal renderer + the glyph→screen pipeline

`MetalRenderer` (`metal_renderer.rs:111`) holds one `RenderPipelineState` **per primitive
kind** and `include_bytes!`'s the precompiled `shaders.metal`. `draw(&Scene)` (`:446`)
loops `for batch in scene.batches()` and issues one instanced **unit-quad** draw per
batch (`quad_vertex` etc. expand a shared quad from `[[vertex_id]]`). **Paths are a
special two-pass, 4× MSAA path** (`:961`/`:1169`) — expensive, and each `Paths` batch
tears down/rebuilds the encoder. Sprite **atlas** = `etagere::BucketedAtlasAllocator`
(`metal_atlas.rs:205`): textures start 1024², grow to fit, cap 16384²; **Monochrome =
`A8Unorm` (1 B/px), Polychrome = `BGRA8Unorm`** — **Subpixel is `unreachable!()`**
(`:72`). One glyph's journey: `layout_line` → `paint_glyph` (quantize to subpixel bins →
`RenderGlyphParams`) → `sprite_atlas.get_or_insert_with(key, || rasterize_glyph(...))`
(cache miss → CoreText raster → etagere tile → upload) → push `MonochromeSprite{tile}` →
`finish()` sorts by `(order, tile_id)` → `batches()` groups same-texture sprites → one
instanced draw sampling A8 coverage × color.

> **Marley adoption `[Marley-original]` (adopt as-is — this is substrate):** Marley's
> `gpui = "0.2.2"` dependency already pulls the entire `Scene → native-Metal` stack; on
> macOS the live renderer is `gpui_macos`' **native Metal** with an etagere atlas — **no
> wgpu, no blade** on Marley's platform (they only matter if Marley ships Linux/Web).
> Key insights:
> - **The GPU/Scene layer is not the bottleneck and needs nothing from Marley** — it is
>   ruthlessly batched (BoundsTree-ordered, k-way-merged, instanced, one draw per
>   same-texture run). **But `insert_primitive` + `finish()` still run per-primitive on
>   the CPU every frame.** An un-virtualized 10k-line buffer walks/paints/sorts 10k
>   lines' primitives every frame though ~50 are visible. **The entire win is at the
>   element layer** (don't emit off-screen lines — §Lists) **and shaped-text reuse**
>   (§Text); the Scene layer is downstream of both.
> - **Headless rendering is the blessed path for Marley's visual/AX self-tests.**
>   `PlatformHeadlessRenderer::render_scene_to_image(scene, size) -> RgbaImage`
>   (`platform.rs:878`) and `PlatformWindow::render_to_image(&Scene)` (`platform.rs:869`,
>   behind `test-support`; macOS `MetalHeadlessRenderer`) rasterize a `Scene` to pixels
>   **without a visible window** — cleaner than the current `screencapture`-by-CGWindowID
>   approach and immune to the "locked screen blocks capture" / "Warp shadows the window"
>   hazards in Marley's memory (it captures gpui's Scene, not the OS compositor result).
> - **`PaintSurface` is the zero-copy video door** for the embedded-browser (CDP)
>   pillar: `window.paint_surface(bounds, CVPixelBuffer)` (`window.rs:4148`) hands a
>   buffer straight to Metal with no CPU copy.
> - **Two facts to file:** (1) **macOS never uses subpixel AA** (glyphs are grayscale
>   `MonochromeSprite`; the subpixel path is `unreachable!()` on Metal) — a pixel-diff
>   test tuned to mac grayscale edges won't transfer to a future Linux build. (2) Prefer
>   `Quad` + `corner_radii` over filled `Path`s for rounded rects (as the cockpit widgets
>   already do) — paths cost the two-pass MSAA detour.

---

## Data / control flow (one frame)

```
OS event (Cocoa / winit) ─► App ─► Window::dispatch_event
   │                                   └─ hitbox test ─► focus path ─► KeyContext stack
   │                                        ─► KeyBinding predicate ─► Action ─► .on_action handler
   │                                        │                     └─ IME: NSTextInputClient ─► EntityInputHandler
   ▼                                        ▼
cx.notify() / entity update / cx.spawn result ─► dirty_views
   │
Window::draw(cx):                                          (window.rs:2639)
   1. invalidate_entities; clear accessed set; enter element arena
   2. draw_roots ─► for each view element (ViewElement reactive boundary):
        request_layout ─► taffy solve ─► prepaint (bounds, hitboxes, handle_input, VIRTUALIZE)
        ─► paint ─► records Quad/Glyph/Path/Sprite into next_frame.scene
        (unchanged entity subtree ⇒ reuse_prepaint + reuse_paint + reuse shaped-text Arc)
   3. install EntityInputHandler with the platform window
   4. text_system.finish_frame() (swap line-layout caches); taffy.clear()
   5. swap rendered_frame ⇄ next_frame; diff focus path ─► focus in/out events
        │
        ▼
   platform Renderer (Metal / wgpu): batch Scene primitives ─► atlas sample ─► GPU ─► present
```

State flows **down** via `render` reading entity state (auto-tracked), **up** via
`Element` events → `Action`s / `Context` mutations → `notify()` → re-render. Async
(session/subprocess streaming) enters via `cx.spawn`.

---

## Dependencies on other subsystems

- **gpui is self-contained** (Apache-2.0): depends only on third-party crates
  (`taffy`, `etagere`, `wgpu`/`metal`, `cosmic-text`, `font-kit` [zed fork],
  `accesskit`, `smallvec`, `slotmap`, `parking_lot`, `sum_tree`, …) — **no dependency
  on any GPL Zed crate.** This is what keeps it freely adoptable.
- Zed's GPL crates (`editor`, `text`, `language`, `workspace`, `terminal_view`) are
  *consumers*: they define concrete `Entity<T: Render>` views and `impl
  EntityInputHandler`. Those patterns are `[Zed-derived]` and instructive, but code
  Marley writes imitating that *logic* is GPL-derived → stays in Marley's GPL
  editor/terminal layer. The gpui APIs they call remain Apache.
- **Marley** consumes gpui directly (`gpui = "0.2.2"`). `crates/marley_app` is the
  primary consumer; `crates/ui_components` builds Marley widgets on gpui elements.

---

## Marley relevance

**Classification: ADOPT (deepen an existing dependency).** gpui is already Marley's
UI framework; the entire value of this document is to **retire hand-rolled code in
favor of gpui capabilities Marley isn't yet using.** The through-line: Marley's M15
editor works on a *hand-rolled substrate* (one div/line, `apply_key`, two-div caret,
hand scroll math, no IME, no KeyContext). Every subsystem above is the productized
version of one of those.

**The adoption backlog, in dependency order (each independently shippable):**

| # | Replace (Marley hand-rolled `[Marley-original]`) | With (gpui `[gpui Apache-2.0]`) | Payoff |
|---|---|---|---|
| 1 | per-line `div` loop, un-virtualized (`app.rs`) | `uniform_list` (terminal) | O(visible) not O(n); free stick/jump-to-bottom |
| 2 | hand `visible_range`/`scroll_code` (`code_view.rs`), 4 offset sites | `UniformListScrollHandle` / `ScrollHandle` / `ListState` | one reusable handle each |
| 3 | generic div text per line | `StyledText` + `TextRun`s + `TextLayout` | shaping, ligatures, per-glyph fallback, atlas glyphs |
| 4 | *nothing* (no pixel↔index) | `index_for_position` / `position_for_index` | click-to-place-caret, caret geometry |
| 5 | two-div `split_at_caret` / `split_caret_char` | `ShapedLine::split_at` | glyph-accurate block cursor |
| 6 | `apply_key` text path | `EntityInputHandler` + `handle_input` | OS text input, fixes multibyte caret bugs |
| 7 | *nothing* (no IME) | marked-text trio + `bounds_for_range` | dead keys, CJK, accents |
| 8 | hand chord disambiguation | `KeyContext` + `actions!` + `KeyBinding` predicates | ⌘D routes to focused pane; keymap-as-data |
| 9 | wrapped/folded editor rows | `list` + `ListState` (+ custom Element later) | variable-height virtualization |
| 10 | hand-placed absolute overlays | `anchored()` + `deferred()` | auto-fit, correct z-order |

**Recommended sequencing:** (1)(2) first — highest ROI, lowest risk (terminal grid is
textbook `uniform_list`), and they de-risk the shaped-text work. Then (3)(4)(5) as the
text/caret block. Then (6)(7)(8) as the input/keymap block (the net-new IME + the ⌘D
fix). (9) when the editor needs soft-wrap/folds; (10) opportunistically. Nothing here
touches the GPL boundary — all Apache.

---

## Key files (cite these)

- `crates/gpui/Cargo.toml` — `license = "Apache-2.0"` (the license lever).
- `crates/gpui/src/gpui.rs` — public surface / module map.
- `crates/gpui/src/app.rs`, `app/context.rs`, `app/entity_map.rs` — `App`/`Entity`/`Context`, ownership.
- `crates/gpui/src/_ownership_and_data_flow.rs` — gpui's own prose on the model.
- `crates/gpui/src/element.rs` — `Element` (request_layout/prepaint/paint), `IntoElement`/`Render`/`RenderOnce`.
- `crates/gpui/src/view.rs` — `View`/`ViewElement` (the reactive boundary + caching).
- `crates/gpui/src/taffy.rs`, `style.rs`, `styled.rs`, `elements/div.rs` — layout + the styled/interactive div.
- `crates/gpui/src/elements/text.rs` — `StyledText`, `.with_highlights`, `TextLayout`, `InteractiveText`.
- `crates/gpui/src/text_system.rs`, `text_system/{line,line_layout,line_wrapper}.rs` — shaping, cache, `ShapedLine::split_at`, wrapping.
- `crates/gpui/src/input.rs`, `platform.rs` — `EntityInputHandler`, `ElementInputHandler`, `UTF16Selection`, `InputHandler`.
- `crates/gpui/src/window.rs` — `Window::{draw, handle_input, paint_glyph, paint_quad, request_measured_layout, insert_hitbox}`.
- `crates/gpui/src/elements/{uniform_list,list}.rs` — virtualized lists + `ListState`/`UniformListScrollHandle`.
- `crates/gpui/src/elements/{anchored,deferred}.rs` — overlays.
- `crates/gpui/src/key_dispatch.rs`, `keymap.rs`, `keymap/{context,binding}.rs`, `action.rs` — focus/KeyContext/actions.
- `crates/gpui/src/scene.rs` — the retained `Scene` (CPU→GPU boundary).
- `crates/gpui_macos/src/{metal_renderer,metal_atlas}.rs` + `shaders.metal` — macOS Metal backend.
- `crates/gpui_wgpu/src/{wgpu_renderer,wgpu_atlas}.rs` + `shaders.wgsl` — cross-platform wgpu backend.
- `crates/gpui/examples/input.rs` — the canonical `EntityInputHandler` editor template.

## Notes / gotchas (load-bearing corrections)

- **No `blade`.** gpui 0.2.2 uses **native Metal (mac) / native D3D11 (Windows) / wgpu
  29.0.4 (Linux/Web)** — three native renderers behind one `Scene`. Correct any prior
  "gpui uses blade" assumption.
- **No `impl_actions!` macro** in this revision — data-carrying actions use
  `#[derive(Action)] #[action(namespace = …)]`. Only `actions!` (unit actions) exists.
- **gpui has a built-in multi-stroke chord engine** (1 s timeout + replay,
  `window.rs:4815`) — Marley's hand-rolled chord state machine should be **deleted**, not
  ported; feed keystrokes through `dispatch_keystroke`.
- **This is the unified-entity gpui:** `App` (not `AppContext`), `Entity<T>` (not
  `Model`/`View` split), `Context<T>` (not `ModelContext`/`ViewContext`), `Render` (not
  `View::render -> Box<dyn Element>`), explicit `&mut Window` threaded everywhere. The
  Marley memory / older WarpUI naming maps per the table in §Mental model.
- **`TextLayout` accessors panic before prepaint** and it's `Rc<RefCell>` (single-thread,
  per-window) — hit-test only inside element event handlers against a painted layout
  (the `InteractiveText` pattern).
- **macOS never uses subpixel AA** — glyphs are grayscale `MonochromeSprite`; the
  `SubpixelSprite` path is `unreachable!()` on Metal. Pixel-diff tests tuned to mac
  grayscale glyph edges won't transfer to a Linux/wgpu build.
- **Filled `Path`s are the expensive primitive** (two-pass 4× MSAA, encoder rebuild per
  batch) — prefer `Quad` + `corner_radii` for rounded rects.
- **Headless `render_scene_to_image` (behind `test-support`) rasterizes a `Scene` to an
  `RgbaImage` with no visible window** — the sanctioned capture path for Marley's visual/
  AX gate, avoiding the `screencapture` locked-screen / window-shadowing hazards (it
  captures gpui's Scene, not the OS compositor output).
- **`gpui = "0.2.2"` is a `publish = true`, Apache-2.0 crate** — adopting anything in this
  document carries **no GPL obligation**; it is a normal permissive dependency Marley
  already links.

## Open questions (round 2)

1. Does Marley's editor become a single custom `impl Element` (Zed-style, owning its
   `prepaint`) or stay a `list` of per-line views? The custom-Element path is required
   for sub-line carets, block decorations, and inlays; the `list` path is faster to
   ship. (Affects sequencing of items 5/9 above.)
2. Terminal: `uniform_list` (fixed cell) vs a custom grid Element that batches the
   whole viewport into one `shape_line(force_width)` per row — which wins on a
   fast-scrolling full-screen `cat`?
3. What is Marley's keymap *data* format, and can pane actions register without a core
   enum edit (so third-party/agent actions can bind)? (§Focus, item 8.)
4. IME on the terminal pane: does Marley want `prefers_ime_for_printable_keys=false`
   (raw bytes, like Zed's terminal) always, or a mode where the terminal composes?
5. Is `cosmic-text` (wgpu backend) vs CoreText (mac) shaping parity a concern for
   cross-platform glyph metrics in the terminal grid?
6. Accessibility (`accesskit` is wired into gpui elements): does Marley want to opt its
   editor/terminal into the a11y tree now, or defer?
