# marley_ui_components — theming vocabulary + widgets (M1.A theme · M1.B widgets)

**Status:** M1 · theme vocabulary TICKET-009 / forge #13 (M1.A seq 2/5) · the 5 widgets TICKET-017 /
forge #17 (M1.B "The Cockpit" seq 1/5). Contract:
[`SPEC-ui-components.spec.md`](../specs/SPEC-ui-components.spec.md) (R1–R16 — the theming vocabulary + the
widget set). **Current @ M15:** the value + widget *API* is unchanged since M1.B — only the fallback
*palette* has been recalibrated (M1.E #35 `success` / #39 `muted`, M12.2 #194 terminal-black, M13 #231
darker panel surface). Clean-room `[Marley-original]` over `[gpui Apache-2.0]` — see [Provenance](#provenance-clean-room).

## Purpose

The workspace's single **theming value vocabulary** (seam-contracts §6): `Appearance` (the Light/Dark
discriminant) + `ThemeColors` (the `gpui::Hsla` color bundle + control metrics every Marley UI surface
renders from). It is the **lowest gpui-dependent crate** — the `Hsla`-bearing palette cannot live in the
gpui-free `marley_core`. `terminal_blocks`, `editor`, and `app_shell` render from a borrowed
`&ThemeColors` for a consistent look; `app_shell` (`marley_app`) wraps these value types in its theme
registry and supplies the real M1 palette.

## Shape

- `Appearance { Light, Dark }` — the discriminant (`Copy`).
- `ThemeColors { background, foreground, accent, on_accent, surface, border, danger, success: gpui::Hsla;
  corner_radius, control_height: gpui::Pixels; control_padding: gpui::Edges<gpui::Pixels> }` — the shared
  bundle; no `Default` (so consumers must supply a palette, and the cargo-mutants `Default::default()`
  return mutant is unviable). `success` (M1.E #35) is the positive/OK semantic paired with `danger`
  (the #36 exit-status indicator's green-vs-red); `muted` (M1.E #39) is the secondary/caption text
  color (dimmer than `foreground`) — the dock header + metadata tone.
- `ThemeColors::default_for(Appearance) -> ThemeColors` — the built-in Light/Dark palettes (R17). As
  of **M1.E #35** these are the ORIGINAL Warp-matched palette (clean-room §20 — the observable Warp
  aesthetic derived into original `hsla`, NOT lifted tokens): a deep cool near-black dark (tinted,
  sat > 0 — not the old flat gray), a warm off-white light, a Marley teal-cyan `accent` (`hsla(0.52,
  …)`), a warm `danger` red, a `success` green. The light `accent` is a DEEP teal (`L 0.34`) so the
  white `on_accent` clears WCAG AA on button labels ([[PR-claude-palette-accent-lightness-must-clear-wcag-vs-on-accent]]).
  The metric tokens are unchanged; the two appearances differ field-wise; the whole cockpit re-skins
  from the swap (everything renders from the bundle). **M12.2 #194** recalibrated the dark theme to
  terminal-black panes (`background` L=0.05) + a distinct gray panel `surface`; **M13 #231** darkened that
  `surface` to L=0.11 for a more Warp-like panel gray (still raised above the near-black bg, the `border`
  still visible, light-text contrast improved — asserted by `contrast_ratio` in `dark_194_shades_pinned_and_legible`).
- `relative_luminance(gpui::Hsla) -> f32` + `contrast_ratio(a, b) -> f32` — the free WCAG helpers (M12.2
  #194): a palette can PROVE its text/ANSI colors stay legible on a background (≥ 4.5 AA, ≥ 7 AAA) rather
  than eyeballing it. `relative_luminance` linearizes sRGB through the WCAG gamma piecewise + Rec. 709 luma
  weights; both are pure, order-independent, alpha-IGNORED (composite a translucent tint over its background
  first). These drive the dark-recalibration legibility asserts.

The theme vocabulary (above) + every widget's **pure decision accessors** are the testable surface —
**cov 100 / MSI 100** (many whole-fn `Default::default()` mutants are unviable because the enums/structs
derive no `Default`, so the viable surface is tight + fully killed). The widgets' gpui **render** (the
`render/` subdir) + the widget-gallery fixture bin (`src/bin/marley_widgets_gallery.rs`) are
ACCEPTED-UNTESTABLE — `#[cfg_attr(test, mutants::skip)]` on every fn **and** a `rust_cov`
`--ignore-filename-regex` exclude, since coverage-exclusion and mutation-exclusion are *separate*
mechanisms (`PR-claude-shim-needs-both-cov-exclude-and-mutants-skip-001`) — asserted by the `#[ignore]`
headed widget-gallery visual test (`tests/headed_widgets.rs`).

## Widgets (M1.B — TICKET-017)

The 5 reusable widgets (SPEC-ui-components R1–R16), each a **long-lived struct** holding persistent
interaction state (never recreated per render — gpui re-renders the same host-owned instance), rendering
from a borrowed `&ThemeColors`, with **pure decision accessors** alongside the gpui render:

- **`button`** — `visual_state` (Rest/Hovered/Pressed/Disabled precedence) · `label_text` ·
  `content_order` · the free `button_colors(kind, &ThemeColors, state)` (every color a `ThemeColors` role).
- **`switch`** — `knob_position` (Leading/Trailing) · `track_color`.
- **`dialog`** — `content_summary` (scrim + card composition) · `outcome_for`.
- **`tooltip`** — `is_visible` + `poll(hovering, elapsed, delay)` (the delay threshold; `elapsed` is a
  pure input — the render owns the `Instant`).
- **`keyboard_shortcut`** — `parse` (a `+`-chord → ordered keycaps) · `keys`. Plus a Marley-owned
  `icon::IconName`.

**Persistent state (R15):** the latches (`hovered`/`pressed`/`visible`) are driven by the **public**
`set_hover` / `set_press` / `poll` setters — the host wires its pointer events to them from its own
`cx.listener` (a `render(&mut self) -> AnyElement` cannot install live gpui handlers without a `Context`).
The three widget handler fields are all `Option` (`Button.on_click`, `Switch.on_toggle`, `Dialog.on_outcome`).

**Decision B — hand-rolled on raw gpui, NO `gpui-component`.** `gpui-component` 0.5.1 *does* resolve with
gpui 0.2.2 (it declares `gpui ^0.2.2` from crates.io) but pulls **32 non-optional deps** (markdown /
html5ever / lsp-types / tree-sitter / …) for 5 trivial widgets; the pure accessors need none of it, and
`IconName` is a small Marley-original enum. The widget renders use the raw-gpui `div()/bg/text_color/child`
pattern (the `marley_app` exemplar).

## Provenance (clean-room)

`[Marley-original]` on `[gpui Apache-2.0]`. Every type here (`Appearance`, `ThemeColors`, the five widgets,
`IconName`) is hand-written Marley code; the only dependency is **gpui 0.2.2** (`Hsla` / `Pixels` /
`Edges` / `SharedString` / `AnyElement`) — Apache-2.0, the Zed-authored standalone renderer, carrying **zero
copyleft**. There is **no** `warp_core`, **no** WarpUI (`warpui_core` / `warpui` / `warpui_extras`), and **no**
`gpui-component` in the dependency set (Decision B, above). The workspace license is **`MIT OR Apache-2.0`**.

**Name-collision caveat:** `marley_ui_components` is a *different crate* from Warp's own AGPL `ui_components`
(which links `warp_core` for its `Appearance`). Marley never adopted the Warp widget stack; a workspace-wide
grep finds zero `warpui` / `warp_core` references. The Warp round-3 UI re-review confirms this — Marley's UI
subsystem is "clean-room `[Marley-original]` over `[gpui Apache-2.0]`, carrying no Warp code" (see
[`warp_architecture/subsystems/01-ui-framework-rendering.md`](../warp_architecture/subsystems/01-ui-framework-rendering.md)
§ "Marley status @ M15" / § "Provenance & licensing"). The Warp-matched palette is the *observable* Warp
aesthetic derived into original `hsla` (clean-room §20), **not** lifted tokens.

## See also

- [crate-map.md](crate-map.md) — the Marley ↔ Warp lineage (ui_components = REIMPLEMENT M).
- [SPEC-ui-components.spec.md](../specs/SPEC-ui-components.spec.md) — the full widget+theme contract.
- [`warp_architecture/subsystems/01-ui-framework-rendering.md`](../warp_architecture/subsystems/01-ui-framework-rendering.md)
  — the Warp UI reference; confirms Marley renders gpui-native, not on WarpUI.
- [`zed_architecture/crates/gpui.md`](../zed_architecture/crates/gpui.md) — the gpui 0.2.2 (Apache-2.0)
  API catalog these widgets render on.
