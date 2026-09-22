# ui_components

> Per-crate reference (Marley round 2) — crate dir `crates/ui_components`. Marley is forked from Warp (warpdotdev/warp).
>
> Provenance: **[Warp-derived/AGPL]** (`license.workspace = true` ⇒ AGPL-3.0-only; links `warp_core` for `Appearance`) · Marley status: **superseded** — Marley ships its own gpui-only `marley_ui_components` (Marley's `crates/ui_components`, a *different* crate despite the name collision — `[Marley-original]` on `[gpui Apache-2.0]`, no `warp_core`). See the subsystem doc's *Marley status @ M15*.

| | |
|---|---|
| Subsystem | [UI Framework & Rendering](../subsystems/01-ui-framework-rendering.md) |
| License | **AGPL v3** (`license.workspace = true` → `AGPL-3.0-only`) |
| Internal deps | 4 |
| Used by | 2 |

## Purpose

`ui_components` is the **reusable, higher-level widget library** for Warp/Marley — buttons, switches, dialogs, tooltips, lightboxes, and keyboard-shortcut chips. It sits one layer above `warpui_core`'s raw `Element` primitives and codifies a single component contract: a long-lived component **struct** holds persistent state (mouse hover handles, tooltip state), a **`Params`** struct carries required + optional render inputs, and an **`Options`** struct supplies appearance-derived defaults. Components are stored as fields on a `View` (not recreated each render) so intra-frame state stays correct.

## Key types, modules & public API

`src/lib.rs` defines the core traits and re-exports widgets:

- **`pub trait Component: Default`** — `type Params<'a>: Params;` and `fn render<'a>(&self, appearance: &warp_core::ui::appearance::Appearance, params: Self::Params<'a>) -> Box<dyn warpui_core::Element>`.
- **`pub trait Params`** — links a params struct to its `type Options<'a>: Options`.
- **`pub trait Options`** — `fn default(appearance: &Appearance) -> Self;` (appearance-driven defaults).
- **`pub use keyboard_shortcut::KeyboardShortcut`**.

Widget modules:
- **`button`** (`src/button.rs` + `button/params.rs`, `button/themes.rs`) — `button::Button`, `button::Params`, `button::Options`, `button::Content` (e.g. `Content::Label`), and `button::themes::Primary` and friends.
- **`dialog`** (`src/dialog.rs`), **`switch`** (`src/switch.rs`), **`tooltip`** (`src/tooltip.rs`), **`lightbox`** (`src/lightbox.rs`), **`keyboard_shortcut`** (`src/keyboard_shortcut.rs`).

Usage pattern (from the crate's own rustdoc example): store the component as a view field, then call `self.my_button.render(appearance, button::Params { content, theme, options })`.

## Depends on (internal)

- [`warpui_core`](./warpui_core.md) — the `Element` trait every component returns, plus `prelude` primitives (`MouseStateHandle`, `Empty`, etc.).
- [`warpui`](./warpui.md) — the platform/renderer (and full re-export of core) used at the widget layer.
- [`warp_core`](./warp_core.md) — provides `warp_core::ui::appearance::Appearance`, the theming/appearance input to every `render`.
- [`asset_cache`](./asset_cache.md) — cached icons/images for components (dev + asset paths).

## Used by (internal dependents)

2 crates: [`warp`](./warp.md) (the main binary) and [`onboarding`](./onboarding.md).

## Related crates

- [`warpui_core`](./warpui_core.md) — the lower-level `Element`/`elements` primitives these widgets compose. Note: core also has an internal `ui_components` *module* (`components::Coords`) — distinct from this crate.
- [`warp_core`](./warp_core.md) — supplies `Appearance`/theming.
- [`onboarding`](./onboarding.md) — a heavy consumer; good reference for assembling these widgets into a flow.

## Marley relevance

**Classification: KEEP + EXTEND.** This is the **widget toolbox for goal (1) — the custom Marley panel.** Building a new panel means composing `button::Button`, `switch`, `dialog`, `tooltip` rather than hand-rolling `Element` trees, so we lean on this crate directly.
- **EXTEND:** add Marley-specific components here (following the `Component`/`Params`/`Options` pattern) rather than scattering ad-hoc elements in `warp`. New widgets inherit appearance defaults for free.
- **Rebrand (goal 4):** widgets render purely from `Appearance` (no embedded "Warp" branding logic), so rebrand here is cosmetic — theme/string changes flow in via `warp_core` appearance, not code edits here.
- **No auth/session coupling**, so nothing to stub or remove for goals (2)/(3).
- **License caveat:** unlike `warpui*` (MIT), this crate is **AGPL v3**. Any Marley distribution that links it inherits AGPL obligations — keep that boundary in mind when deciding what ships in a closed component vs. the AGPL app.

## Notes / gotchas

- **Edition 2024** (newer than the `warpui*` crates' 2021) — requires a recent toolchain.
- Components are **stateful and must be reused** across frames; the rustdoc explicitly warns that creating them fresh each render corrupts hover/tooltip state. This is a correctness constraint for any new panel code.
- Dev-dependencies pull `warpui`, `rust-embed`, `rustls`, and `warp_core` with `test-util` — tests render real components, so they need the full UI stack.
- Despite the name collision, this crate is **not** `warpui_core::ui_components`; consumers import the top-level `ui_components` crate for `Button`, `Switch`, etc.
