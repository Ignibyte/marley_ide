# gpui

> Per-crate reference (Marley round 2) — crate dir `crates/gpui` in the Zed clone
> (github.com/zed-industries/zed). The API-surface catalog for Marley's **#1 adopt
> target**. Unlike the rest of the Zed reference, **this crate is not GPL** — see the
> license row. The strategic view + adoption sequencing lives in the subsystem doc
> ([01 — gpui UI Framework](../subsystems/01-gpui-ui-framework.md)); this doc is the
> concrete public-API catalog + the 1:1 Marley-hand-rolled→gpui-native map.

| Field | Value |
|-------|-------|
| Subsystem | [01 — gpui: UI Framework, Rendering, Text & Input](../subsystems/01-gpui-ui-framework.md) |
| License | **Apache-2.0** — `license = "Apache-2.0"`, `publish = true`, `version = "0.2.2"`, `homepage = "https://gpui.rs"` in `crates/gpui/Cargo.toml`. **NOT part of Zed's GPL-3.0 surface.** It is a standalone publishable crate Zed authors, and **Marley's own dependency** (`gpui = "0.2.2"`). Adopting any API below carries **zero copyleft obligation.** |
| Internal deps (Zed workspace) | `gpui_macros`, `gpui_shared_string`, `gpui_util`, `collections`, `http_client`, `refineable`, `scheduler`, `sum_tree` (+ externals: `taffy =0.10.1`, `resvg`/`usvg`, `lyon`, `etagere`, `ttf-parser`, `image`; macOS: `metal`/`cocoa`/`core-text`/`objc`) |
| Used by (in Marley) | **3** direct dependents — `marley_app`, `ui_components`, `marley_visual_harness` (all `gpui = "0.2.2"`). In Zed itself: virtually every UI crate. |

## Purpose

`gpui` is Zed's **GPU-accelerated, immediate-mode (retained-scene) UI framework** — the
`App`/`Entity`/`Context` ownership model, the `Element`/`Render` tree, taffy layout, a
subpixel text-shaping system with a cross-frame cache, an IME-grade input bridge,
key-context dispatch, and a `Scene` → native-Metal/wgpu paint pipeline. It is a single
publishable Apache-2.0 crate (`path = "src/gpui.rs"`).

For Marley it is unique among the Zed crates: **Marley already links it directly**, so this
is not a "reimplement the capability" analysis (like `editor`/`text`/`language`) but a
**"stop hand-rolling what the dependency already gives us"** analysis. Marley today
hand-rolls render + input on gpui's *lowest* primitives — one `div` per line
(un-virtualized), a hand-written `apply_key`, a two-`div` `split_at_caret`, hand chord logic
in `keymap.rs`, hand scroll math — while the higher-level gpui APIs cataloged here
(`uniform_list`, `EntityInputHandler`, `ShapedLine::split_at`, `KeyContext`, `ScrollHandle`,
`render_to_image`) are the intended replacements. **Provenance:** everything here is
`[gpui Apache-2.0]` unless tagged `[Marley-original]` (Marley's hand-rolled code being
replaced) — there is no GPL surface in this crate.

## Key types, modules & public API

The crate root (`src/gpui.rs`) re-exports every module flat (`pub use app::*; pub use
element::*; …`) and ships a `prelude` (`Context`, `Element`, `ParentElement`, `Render`,
`RenderOnce`, `Styled`, `InteractiveElement`, `StatefulInteractiveElement`, `IntoElement`,
`VisualContext`, `AppContext as _`, `FluentBuilder`). Grouped by responsibility:

### 1. Ownership & reactivity — `app.rs`, `app/context.rs`, `app/entity_map.rs`
- **`App`** (`app.rs:679`) — the single global owner of every entity; `Application::new()`
  boots it. `App::open_window<V: Render>(options, |win, cx| -> Entity<V>) -> Result<WindowHandle<V>>`
  (`app.rs:1200`); `quit()` (`:981`), `refresh_windows()` (`:1008`).
- **`AppContext`** trait (`gpui.rs:166`, the `cx` verbs; prelude-imported as `AppContext as _`):
  `new(build) -> Entity<T>`, `reserve_entity`/`insert_entity`, `update_entity`, `read_entity`,
  `read_global::<G>`, `background_spawn(future) -> Task<R>`, `update_window`/`with_window`.
- **`Entity<T>`** — a refcounted slotmap handle (`entity_map.rs`); `read(&App) -> &T` (`:464`),
  `read_with` (`:470`), `update(cx, |t, cx| …) -> R` (`:476`), `update_in(&VisualContext)`
  (`:502`), `entity_id()` (`:443`), `downgrade() -> WeakEntity<T>` (`:449`). **`WeakEntity<T>`**:
  `upgrade() -> Option`, `update(...) -> Result` (`:777`).
- **`Context<'a, T>`** (`app/context.rs:20`) — `{ app: &mut App, … }` that `Deref`s to `App`;
  the reactive verbs: `notify()` (invalidate this entity), `observe(&other, cb)`,
  `subscribe(&other, cb)` (typed, `other: EventEmitter<E>`), `emit(event)`, `spawn(async cb)
  -> Task`, `spawn_in(&window, …)`, `listener(method) -> handler`, `focus_handle()`.
- **`VisualContext`** (`gpui.rs:254`) — the window-bearing ctx: `focus(&entity)`,
  `new_window_entity`, `replace_root_view`.
- **`Global`** (`global.rs`) marker + `set_global`/`update_global`/`read_global` — app-wide
  singletons by type. **`Subscription`** (`subscription.rs`) — `.detach()` or hold-to-cancel.
- **Executors** (`executor.rs`): `BackgroundExecutor` (Send pool) / `ForegroundExecutor`
  (main thread), both → `Task<R>` (cancel-on-drop). `AsyncApp`/`AsyncWindowContext`.

### 2. Elements, render & layout — `element.rs`, `view.rs`, `styled.rs`, `elements/div.rs`, `taffy.rs`
- **`Element`** trait (`element.rs:51`) — the per-frame `request_layout → prepaint → paint`
  lifecycle with two carried state types; `prepaint` is where hitboxes/input-handlers register
  and virtualization happens. **`IntoElement`** (`:145`), **`ParentElement`** (`.child`/
  `.children`, `:188`).
- **`Render`** (`element.rs:163`) — `fn render(&mut self, &mut Window, &mut Context<Self>) ->
  impl IntoElement`. An `Entity<T: Render>` is a *view*. **`RenderOnce`** (`:179`) — a
  stateless, by-value component (pair with `#[derive(IntoElement)]`). **`View`** (`view.rs:175`)
  + **`AnyView`**/`AnyWeakView` unify them; `Entity::cached(style)` (`view.rs:225`) freezes a
  subtree until `notify()`. **`ViewElement`** (`view.rs:233`) is the reactive/caching boundary.
- **`div()` → `Div`** (`elements/div.rs:1593`) — the workhorse container; `.id(…)` upgrades to
  **`Stateful`** (`:3691`).
- **`Styled`** trait (`styled.rs:22`) — Tailwind-ish fluent API on every element: `.flex()`
  (`:45`), `.flex_col()`/`.flex_row()`, `.w_full()`/`.h_full()`, `.p_2()`/`.gap_1()`,
  `.bg(fill)` (`:490`), `.text_color(hsla)` (`:514`), `.font_family(name)` (`:708`),
  `.line_height(len)` (`:740`), `.rounded_md()`, `.border_1()`, `.absolute()`,
  `.overflow_y_scroll()`. `Style` (`style.rs:182`) is a flat web-ish struct; `StyleRefinement`
  is the hover/focus/group "patch". Layout is **taffy** (`taffy.rs`, flexbox/grid) — gpui does
  not hand-roll layout.

### 3. Text — `elements/text.rs`, `text_system.rs`, `text_system/{line,line_layout,line_wrapper}.rs`
- **`StyledText`** (`text.rs:391`) — `new(text)` (`:401`) + `.with_highlights(hl)` (`:433`, lazy,
  inherits window style) or `.with_default_highlights(&style, hl)` (`:418`, eager). Highlights
  are **sparse** `(Range<usize>, HighlightStyle)` that auto-fill gaps; a **`TextRun`**
  (`text_system.rs:987`) = `{ len, font, color, background, underline, strikethrough }` tiling
  the string. `.layout() -> &TextLayout` (`:412`).
- **`TextLayout`** (`text.rs:614`, shared `Rc<RefCell>`) — the pixel↔byte hit-testing Marley
  lacks: `index_for_position(pt) -> Result<usize, usize>` (`:808`, `Ok`=inside glyph,
  `Err`=clamped-nearest-slot), `position_for_index(byte) -> Option<Point<Pixels>>` (`:842`),
  `line_layout_for_index`. **`InteractiveText`** (`text.rs:959`) adds `.on_click(ranges, cb)`/
  `.on_hover`/`.tooltip` over a `StyledText`.
- **`ShapedLine::split_at(byte) -> (ShapedLine, ShapedLine)`** (`text_system/line.rs`) — splits a
  shaped line at a byte offset, rebasing suffix glyph positions; the exact 1:1 for
  `split_at_caret`.
- **`WindowTextSystem`** shaping entry points: `shape_line(text, size, runs, force_width) ->
  ShapedLine` (`force_width: Some(cell_width)` snaps to a monospace grid — the terminal
  primitive), `shape_text(…, wrap_width, …)` (soft-wrap), `layout_line_by_hash` (keys the cache
  on a `u64` without materializing the string). **`LineLayoutCache`** keeps prev/current-frame
  maps so an unchanged row costs an `Arc` clone, not a reshape.

### 4. Input & IME — `input.rs`, `platform.rs`, `window.rs`
- **`EntityInputHandler`** trait (`input.rs:10`) — implemented by the *editor entity itself*;
  the methods (all take `&mut Context<Self>`): `text_for_range`, `selected_text_range ->
  Option<UTF16Selection>`, `marked_text_range -> Option<Range>` (`Some` == IME composing),
  `unmark_text`, **`replace_text_in_range`** (the real "type a char" commit),
  **`replace_and_mark_text_in_range`** (set preedit), `bounds_for_range -> Option<Bounds>`
  (caret/IME rect), `character_index_for_point` (click→offset), + defaulted `set_selected_text_range`,
  `text_length_utf16`, `accepts_text_input`.
- **`ElementInputHandler<V>`** (`input.rs:100`) — `new(bounds, entity)`; forwards each call into
  `entity.update(...)`. Built and registered in the element's `paint`.
- **`UTF16Selection { range, reversed }`** (`platform.rs:1511`) — all offsets at this boundary are
  **UTF-16 code units** (every OS text API speaks UTF-16); the handler owns the conversion.
- **Registration:** `window.handle_input(&focus_handle, ElementInputHandler::new(bounds, entity),
  cx)` (`window.rs:4365`) — **focus-gated, rebuilt every frame; never cached across focus.**

### 5. Lists & virtualization — `elements/uniform_list.rs`, `elements/list.rs`
- **`uniform_list(id, item_count, |Range, win, app| -> Vec<R>) -> UniformList`**
  (`uniform_list.rs:22`) — fixed row height (→ terminal grid). Measures one row, realizes only
  the visible index range; cost O(visible), independent of `item_count`.
  **`UniformListScrollHandle`** (`:80`): `scroll_to_item(ix, ScrollStrategy)` (`:150`;
  `Top|Center|Bottom|Nearest`, `:84`), `scroll_to_bottom()` (`:252`), `is_scrolled_to_end() ->
  Option<bool>` (`:241`), `y_flipped()` (`:216`). Attach via `.track_scroll(&h)` (`:683`),
  `.with_sizing_behavior`, `.with_decoration`.
- **`list(state, render_item) -> List`** (`list.rs:24`) + **`ListState::new(count, alignment,
  overdraw)`** (`:314`) — variable row height (→ editor / Warp blocks). A `SumTree<ListItem>`
  indexes by item **or** pixel offset in O(log n), lazy-measures on entry, and keeps a *logical*
  `ListOffset { item_ix, offset_in_item }` so edits above the viewport don't jump. `splice(range,
  count)`, `scroll_to_reveal_item`, `set_follow_mode(Tail)` (stick-to-bottom for streaming),
  `ListAlignment::{Top,Bottom}`.

### 6. Scrolling (non-virtualized) — `elements/div.rs`
- `div().overflow_scroll()` / `.overflow_x_scroll()` / `.overflow_y_scroll()` set `Overflow::Scroll`.
  **`ScrollHandle`** (`div.rs:3872`) owns the offset cell: `offset()` (`:3887`), `set_offset(pt)`
  (`:4023`), `scroll_to_item(ix)` (`:3945`), `scroll_to_bottom()` (`:4015`), `top_item()`
  (`:3897`), `bounds_for_item(ix)` (`:3940`); attach via `.track_scroll(&h)` (`:1416`). Wheel →
  pixel delta, clamp + content-mask clip happen next frame in prepaint. **`ScrollAnchor`**
  (`:3819`) scrolls a non-immediate descendant into view.

### 7. Focus, KeyContext, Keymap & Actions — `window.rs`, `key_dispatch.rs`, `keymap.rs`, `keymap/{context,binding}.rs`, `action.rs`, `elements/div.rs`
- **Focus:** `cx.focus_handle() -> FocusHandle` (mint once, store, return the same each frame);
  `.track_focus(&handle)` makes a div focusable; `handle.is_focused(window)` (`window.rs:354`),
  `contains_focused`; `window.focus(&handle, cx)` (`window.rs:1908`, also clears pending
  multi-key input). **`Focusable`** trait (`window.rs:557`).
- **`KeyContext`** (`keymap/context.rs:10`) — `parse(str)` (`:65`), `add(id)`/`set(k, v)`; a `div`
  attaches one via **`.key_context("Editor")`** (`div.rs:761`, `InteractiveElement`). The context
  **stack** for a focused leaf is `[Workspace, Pane, Editor]` root→leaf.
- **`KeyBindingContextPredicate`** (`context.rs:172`) — grammar `Editor`, `mode == full`,
  `Workspace > Editor` (descendant), `!vim`, `&&`, `||`; `parse` (`:249`), `depth_of(contexts) ->
  Option<usize>` (`:260`, deepest matching prefix → **deeper context wins**), `eval` (`:272`).
- **`Keymap`** (`keymap.rs:18`) — `new(bindings)`, `add_bindings(iter)` (`:63`),
  `bindings_for_input(input, stack)` (`:164`). **`KeyBinding::new(keystrokes, action,
  context: Option<&str>)`** (`binding.rs:33`; multi-stroke chords via space-separated
  keystrokes, e.g. `"cmd-k cmd-s"`).
- **`Action`** trait (`action.rs:117`, identity by `TypeId`). Declare with **`actions!(namespace,
  [SplitRight, CloseTab])`** (`action.rs:24`) for unit actions or **`#[derive(Action)]`** on a
  struct for data-carrying ones; auto-registered via `inventory`; loaded by name via
  `ActionRegistry::build_action(name, params)` (`:351`) — the TOML/JSON-keymap seam. `NoAction`/
  `Unbind` disable an inherited binding.
- **Handling:** `.on_action::<A>(cx.listener(Editor::method))` (`div.rs:1014`, bubble) /
  `.capture_action` (capture); `window.dispatch_action(action, cx)` fires without a keystroke;
  `window.available_actions(cx) -> Vec<Box<dyn Action>>` (`window.rs:5284`) is the context-aware
  command list. `window.dispatch_keystroke(keystroke, cx) -> bool` (`window.rs:4505`) drives the
  whole tree, including gpui's **built-in multi-stroke chord engine (1 s timeout + replay)**.
- ⚠️ **No `impl_actions!` in this revision** (grep-confirmed 0 occurrences) — data actions use
  `#[derive(Action)]`.

### 8. Overlays — `elements/anchored.rs`, `elements/deferred.rs`
- **`anchored()`** (`anchored.rs:27`) — `.anchor(Corner)`, `.position(pt)`, `.snap_to_window()`,
  `SwitchAnchor`/`SnapToWindow` fit modes; auto-avoids window overflow (menus/popovers/tooltips).
- **`deferred(child)`** (`deferred.rs:7`) — lay out in place but **paint after all ancestors**
  (`.with_priority(n)`) — correct z-order for overlays. **`ManagedView`** = `Focusable +
  EventEmitter<DismissEvent> + Render` (the modal contract; `emit(DismissEvent)` to close).

### 9. Platform, Scene & headless render — `scene.rs`, `platform.rs`, `window.rs`
- **`Scene`** (`scene.rs:41`) — the CPU→GPU boundary; `paint` pushes 8 `#[repr(C)]` primitive
  types (`Quad`, `Shadow`, `Path`, `Underline`, `MonochromeSprite` = glyphs/SVG,
  `SubpixelSprite`, `PolychromeSprite` = emoji/images, `PaintSurface` = video). A `BoundsTree`
  assigns draw order; `Scene::batches()` k-way-merges into one instanced draw call per batch.
- **Backend:** `Window::present` → `platform_window.draw(&scene)`. In **0.2.2 the macOS
  native-Metal + Cocoa backend links directly into this crate** (`metal`, `cocoa`, `core-text`,
  `objc` are macOS deps; `bindgen`/`cbindgen` build-deps); non-mac renderers come via the sibling
  `gpui_platform` umbrella. **There is zero `blade`** (grep-confirmed) — the "gpui uses blade"
  folklore is false for this line.
- **Headless / offscreen render** `[gpui Apache-2.0]` — **the deterministic alternative to
  Marley's `screencapture -l<winid>` test hazard**: `Window::render_to_image() ->
  Result<image::RgbaImage>` (`window.rs:2272`) renders the current scene offscreen with **no
  window compositor, no frontmost-window race, no shadow offsets**. Test harnesses expose it as
  `capture_screenshot(window)` on `HeadlessAppContext` (`headless_app_context.rs:168`) and
  `VisualTestContext` (`visual_test_context.rs:384`).

### 10. Test support — `app/{test_context,visual_test_context,headless_app_context}.rs` (feature `test-support`)
- `TestAppContext` / `VisualTestContext`: `simulate_keystrokes(win, "cmd-d")`
  (`test_context.rs:471`), `simulate_input(win, "abc")` (`:487`, drives text via the input
  handler), `dispatch_action(win, action)` (`:454`), `dispatch_keystroke` (`:496`),
  `run_until_parked()` (`:449`), `simulate_mouse_down`/`_up`/`_move`, `draw(...)` (`:866`), +
  `capture_screenshot`. **All input is injected straight into the entity tree** — no CGEvents,
  no "input hits the frontmost window" problem.

## Depends on (internal)

The `gpui_*` family + a few Zed workspace utility crates (from `Cargo.toml`):
- **`gpui_macros`** — `#[derive(IntoElement/Render/Action/AppContext/VisualContext)]`, the
  `#[register_action]` / `#[test]` / `bench` macros, the `inspector` feature.
- **`gpui_shared_string`** — `SharedString` (cheap cloneable string) + `ArcCow`.
- **`gpui_util`** — `FluentBuilder`, `Timeout`, `FutureExt`, `arc_cow`.
- **`collections`** — `HashMap`/`TypeIdHashMap` (FxHash) used throughout.
- **`scheduler`** — the executor/dispatcher substrate for `Background`/`Foreground` executors.
- **`sum_tree`** — the `SumTree` powering `list`/`ListState` virtualization.
- **`refineable`** — the `Refineable` derive behind `StyleRefinement`.
- **`http_client`** — asset/image fetching (`pub use http_client`).
- Externals: `taffy =0.10.1` (layout), `resvg`/`usvg` (SVG), `lyon` (path tessellation),
  `etagere` (sprite-atlas shelf allocator), `ttf-parser`, `image`, `parking_lot`, `smallvec`,
  `slotmap`; macOS-only `metal`/`cocoa`/`core-text`/`core-graphics`/`objc`/`font-kit`.

## Used by (internal dependents)

- **In Marley (3):** `marley_app` (`Cargo.toml:16`, the app shell/editor/terminal render + input),
  `ui_components` (`Cargo.toml:14`, the M1.B cockpit widgets — button/switch/dialog/tooltip/
  shortcut), `marley_visual_harness` (`Cargo.toml:8`, the driven-capture harness). All pinned to
  `gpui = "0.2.2"`.
- **In Zed:** effectively every UI-bearing crate (`editor`, `workspace`, `terminal_view`, `ui`,
  `gpui_component`, …) — gpui is the framework everything renders on.

## Related crates

- [`editor`](../subsystems/03-editor-multibuffer.md), [`text`](../subsystems/02-text-buffer-anchors.md)
  — Zed's GPL consumer crates whose *patterns* (`EntityInputHandler` wiring, custom-Element editor)
  are `[Zed-derived]`; the gpui API they call is Apache but any editor **logic** Marley imitates is
  GPL-derived and stays in Marley's GPL layer.
- `gpui_macros` / `gpui_platform` — the sibling crates in the gpui family (macro surface; non-mac
  platform umbrella).
- Round-1 subsystem doc [01 — gpui UI Framework](../subsystems/01-gpui-ui-framework.md) — the
  full strategic deconstruction (mental model, reactive loop, Scene/GPU pipeline, sequencing).

## Adoption on our stack (Marley)

**Classify: ADOPT (incrementally).** gpui is already linked (`gpui = "0.2.2"` in three crates),
so every row below is a *net-code-deletion* — replace a hand-rolled Marley mechanism with the
gpui-native API it shadows. **Zero license cost** (`[gpui Apache-2.0]`). The 1:1 map (verified
against the current Marley tree):

| Marley hand-rolled today `[Marley-original]` | gpui-native replacement `[gpui Apache-2.0]` | What it retires |
|---|---|---|
| one `div` per line, un-virtualized (`code_view.rs`; **0** `uniform_list` uses in Marley) | **`uniform_list(id, count, closure)`** + `UniformListScrollHandle` | O(n) live elements → O(visible); deletes the per-line loop |
| `visible_range` (`code_view.rs:187`) + `scroll_code` (`:196`) + the #198 `scrollbar_thumb`/`at_bottom` math | `UniformListScrollHandle::{scroll_to_bottom, is_scrolled_to_end}` / `ScrollHandle::{offset,set_offset,scroll_to_item}` | ~4 bespoke offset sites → 1–3 handles; stick/jump-to-bottom for free |
| `split_at_caret` / `split_caret_char` (`input.rs:153/166`, two-`div` cursor) | **`ShapedLine::split_at(byte)`** + `TextLayout::position_for_index` for the caret quad | glyph-accurate caret (ligatures/combining marks) vs. string split |
| `apply_key` (`input.rs:62`, hand text mutation; **0** `EntityInputHandler` uses) | **`EntityInputHandler` + `window.handle_input`** (focus-gated in `paint`) | the text path *and* net-new IME (dead keys, CJK) + UTF-16 caret correctness |
| `keymap.rs` hand chord table (`KeyBinding::chord`, `Keymap::default_bindings`; single-stroke only, no context) | **`actions!` / `#[derive(Action)]` + `KeyBinding::new(…, ctx)` + `Keymap`**; `window.dispatch_keystroke` | gpui's built-in **multi-stroke chord engine** (1 s timeout + replay) |
| manual active-pane flag + per-pane `⌘D`/`⌘W` branching (**0** `key_context` uses in Marley) | **`.key_context("Terminal"/"Editor")` + predicate bindings** (`Some("Editor")`); `handle.is_focused(window)` | same-key/different-action falls out of the dispatch tree; retires the #220/#186 hand-tracked flag |
| hand-maintained command table for the palette | **`window.available_actions(cx)`** + `window.dispatch_action` | context-aware command list, for free |
| `screencapture -l<winid>` driven capture (frontmost-window race; shadow offsets — see MEMORY self-test PRs) | **`Window::render_to_image()`** / `VisualTestContext::{simulate_keystrokes, simulate_input, dispatch_action, capture_screenshot}` | deterministic offscreen render + input-into-entity-tree; no compositor, no frontmost race |

**Sequencing** (from round-1): (1) render lines through `StyledText`+`TextLayout` keeping
div-per-line; (2) wire click-to-caret + caret geometry; (3) `ShapedLine::split_at` for the
cursor; (4) route the terminal grid through `shape_line(force_width)`; (5) virtualize
(`uniform_list` terminal / `list`+`ListState` editor) + adopt `LineLayoutCache` reuse; in
parallel, migrate input to `EntityInputHandler` and the keymap to `actions!`+`KeyContext`. Each
stage is independently testable via the `test-support` harness (no live IME, no live window).

**Keep as-is (already correct usage `[Marley-original]`):** Marley's app/editor *are* entities;
it already uses `cx.notify()`, `.track_focus`, and builds UI from `div()`+`Styled`. Those are the
substrate the adoptions plug into — don't rewrite them; lean harder on **auto-invalidation +
subtree reuse** so virtualization reuses shaped text/paint of unchanged rows for free.

## Notes / gotchas

- **License is the headline:** Apache-2.0, `publish = true`, `gpui = "0.2.2"` — Marley's own
  permissive dependency. Nothing in this crate touches the GPL boundary. The *only* GPL exposure
  is if Marley copies Zed's **consumer-crate logic** (`editor`/`text`); the gpui API itself is clean.
- **The input handler is rebuilt every frame and only while focused** (`handle_input` gates on
  `focus_handle.is_focused()`) — never cache an `ElementInputHandler` across focus changes.
- **All input offsets at the OS boundary are UTF-16 code units** (`UTF16Selection`); the handler
  owns byte↔UTF-16 conversion + grapheme-boundary movement. Porting this *also fixes latent
  multibyte caret bugs* Marley's `CharOffset` math has even before IME.
- **`TextLayout` accessors panic before prepaint** and it is `Rc<RefCell>` (single-thread,
  per-window) — hit-test *inside* element event handlers (as `InteractiveText` does), never across
  threads/frames.
- **No `impl_actions!` macro** in 0.2.2 — use `#[derive(Action)]`. **No `blade`** — the backends
  are native Metal (macOS, in-crate) + wgpu (elsewhere, via `gpui_platform`).
- **`list` has a known ceiling:** Zed's *production* editor outgrew `list` and hand-writes a custom
  `Element` (same prepaint-only-visible discipline, sub-line precision). `list`+`ListState` is the
  right **baseline** for Marley's editor; "graduate to a custom `Element`" is the known next step.
- **`edition = 2024`.** Feature flags: `test-support` (the harness above; pulls `proptest`),
  `screen-capture` (`scap`), `inspector`, `wayland`/`x11` (Linux), `windows-manifest`. `doctest =
  false`. Many runnable `examples/` — `input.rs` is the canonical editor template, `uniform_list.rs`
  the virtualization template.
