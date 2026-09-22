---
spec_id: ui-components
component: marley_ui_components
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Reusable widget set (button/switch/dialog/tooltip/shortcut) on gpui + the theming value vocabulary
goal: Provide a stateful, theme-driven widget toolbox (button, switch, dialog, tooltip, keyboard-shortcut chip) plus the shared `Appearance`/`ThemeColors` value vocabulary every widget renders from, so the Marley panel composes widgets instead of hand-rolling element trees and the whole set restyles from one supplied color palette.
reuses: [gpui]  # decision B (#17): hand-rolled on raw gpui — gpui-component resolves with gpui 0.2.2 but drags 32 non-optional deps for 5 trivial widgets; IconName is Marley-owned (icon.rs)
spec_source: "behavior-only — observable I/O of a reusable gpui widget set (button, switch, dialog, tooltip, keyboard-shortcut chip) that renders entirely from a supplied light/dark color palette: visible label/icon content and their order, hover/press/disabled visual states, switch knob side + track color, dialog scrim+card composition and confirm/cancel outcome, delayed tooltip visibility, and keycap-chip ordering; plus a typed light-vs-dark palette value type the whole set keys on. No fork module/type/static names, no fork file paths. Theming + widget seam ownership pinned in standards/seam-contracts.md §6 + §6.1."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: "Each widget renders from one supplied `ThemeColors` palette and matches its captured Marley baseline screenshot within tolerance — Button padding/corner-radius/accent background, Switch knob travel and on/off track colors, Dialog centered card over a dimmed full-window scrim, Tooltip floating card offset from its target, KeyboardShortcut as ordered keycap chips — all asserted via the macOS AXUIElement + screenshot harness (../pipeline/visual-testing.spec.md)."
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose

`marley_ui_components` is the reusable widget library that sits one layer above gpui's raw element
primitives, and it is the workspace's single **theming value vocabulary** (seam-contracts §6): it
owns `Appearance` (the Light/Dark discriminant) and `ThemeColors` (the `gpui::Hsla` color bundle every
widget renders from). It lives at the lowest crate that depends on `gpui`, because the `Hsla`-bearing
palette cannot live in the gpui-free `marley_core`. Each widget is a Marley-original long-lived struct
that holds its own persistent interaction state (hover/press latches, tooltip hover-since) and is
stored as a field on a `Render` host — never recreated per frame — so intra-frame hover and tooltip
state stay correct. A widget renders purely from a borrowed `&ThemeColors` (plus `Appearance` only
where a built-in default must branch Light vs Dark) and exposes **pure decision accessors** alongside
its `render` entry, so all of its logic — chosen colors, visual state, knob side, tooltip visibility,
content order, dialog outcome — is headless-testable without a GPU. It owns the button, switch, dialog,
tooltip, and keyboard-shortcut chip; the panel composes these rather than assembling ad-hoc elements.
The element/render substrate and the underlying styled primitives are supplied by the reused `gpui`
and `gpui-component`.

## Public surface (the contract)

```rust
// ── Theming value vocabulary (owned here — seam-contracts §6) ────────────────
pub enum Appearance { Light, Dark }     // the discriminant ONLY

pub struct ThemeColors {                // the shared bundle every widget renders from
    pub background: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub accent: gpui::Hsla,
    pub on_accent: gpui::Hsla,
    pub surface: gpui::Hsla,
    pub border: gpui::Hsla,
    pub danger: gpui::Hsla,
    pub success: gpui::Hsla,               // M1.E #35 — the positive/OK semantic (exit 0)
    pub muted: gpui::Hsla,                 // M1.E #39 — secondary/caption text, dimmer than foreground
    // metrics:
    pub corner_radius: gpui::Pixels,
    pub control_height: gpui::Pixels,
    pub control_padding: gpui::Edges<gpui::Pixels>,
}
impl ThemeColors {
    /// Built-in fallback palette; branches on `Appearance` (used only where no palette is supplied).
    pub fn default_for(appearance: Appearance) -> ThemeColors;
}
```

**Palette values (M1.E #35 — an ORIGINAL Warp-matched palette, clean-room §20: the observable Warp
aesthetic derived into original `hsla`, NOT lifted from Warp's source tokens).** Dark: a deep cool
near-black `background` `hsla(0.62,0.14,0.09)` (tinted, not flat gray), soft near-white `foreground`,
a Marley teal-cyan `accent` `hsla(0.52,0.58,0.55)`, dark `on_accent`, elevated `surface`/`border`,
warm `danger` red `hsla(0.99,0.62,0.62)`, `success` green `hsla(0.40,0.50,0.55)`. Light: warm
off-white `background`, dark `foreground`, the accent darkened for contrast, white `on_accent`, the
danger/success darkened. The metric tokens (`corner_radius` px6, `control_height` px28,
`control_padding`) are unchanged. The two appearances differ; `success` is distinct from `danger`/
`accent`/`background`.

```

// ── Button ───────────────────────────────────────────────────────────────────
pub mod button {
    use super::{Appearance, ThemeColors};
    // `IconName` is Marley-owned (icon.rs) — decision B (#17); NOT reused from gpui-component.
    pub enum ButtonContent { Label(gpui::SharedString), Icon(IconName), IconLabel(IconName, gpui::SharedString) }
    pub enum VisualState { Rest, Hovered, Pressed, Disabled }
    pub enum ContentOrder { LabelOnly, IconOnly, IconThenLabel }   // R4 descriptor (icon = inline-start)
    pub enum ButtonKind { Primary, Secondary, Destructive, Ghost } // replaces the round-1 Theme trait + themes::* structs
    pub struct ButtonColors { pub background: gpui::Hsla, pub foreground: gpui::Hsla, pub border: gpui::Hsla }

    pub struct ButtonProps<'a> {
        pub content: ButtonContent,
        pub kind: ButtonKind,
        pub disabled: bool,
        pub on_click: Option<Box<dyn Fn(&mut gpui::Window, &mut gpui::App) + 'a>>,
    }

    #[derive(Default)]
    pub struct Button { /* persistent hover/press latches */ }
    impl Button {
        /// Render purely from `colors` + `props`; `&mut self` carries persistent interaction state.
        pub fn render(&mut self, colors: &ThemeColors, props: ButtonProps<'_>) -> gpui::AnyElement;
        /// Pure decision seam (R6/R8/R15/R16): Rest/Hovered/Pressed/Disabled from latches + `props.disabled`.
        pub fn visual_state(&self, props: &ButtonProps) -> VisualState;
        /// Pure decision seam (R3): the visible label this content yields, if any.
        pub fn label_text(props: &ButtonProps) -> Option<gpui::SharedString>;
        /// Pure decision seam (R4): icon-vs-label ordering for `props.content`.
        pub fn content_order(props: &ButtonProps) -> ContentOrder;
    }
    /// Pure color decision seam (R2/R5): maps (kind, palette, state) → concrete colors.
    pub fn button_colors(kind: ButtonKind, colors: &ThemeColors, state: VisualState) -> ButtonColors;
}

// ── Switch ───────────────────────────────────────────────────────────────────
pub mod switch {
    use super::ThemeColors;
    pub enum KnobPosition { Leading, Trailing }
    pub struct SwitchProps<'a> {
        pub on: bool,
        pub disabled: bool,
        pub on_toggle: Option<Box<dyn Fn(bool, &mut gpui::Window, &mut gpui::App) + 'a>>,
    }
    #[derive(Default)]
    pub struct Switch { /* persistent hover latch */ }
    impl Switch {
        pub fn render(&mut self, colors: &ThemeColors, props: SwitchProps<'_>) -> gpui::AnyElement;
        /// Pure decision seam (R9): Trailing WHILE `on`, Leading otherwise.
        pub fn knob_position(&self, on: bool) -> KnobPosition;
        /// Pure decision seam (R9/R2): the track color for the given `on` value, sourced from `colors`.
        pub fn track_color(&self, colors: &ThemeColors, on: bool) -> gpui::Hsla;
    }
}

// ── Dialog ───────────────────────────────────────────────────────────────────
pub mod dialog {
    use super::ThemeColors;
    pub enum DialogOutcome { Confirm, Cancel }
    pub enum DialogControl { Confirm, Cancel }
    pub struct DialogContent { pub has_scrim: bool, pub card_has_title: bool, pub card_has_body: bool, pub card_has_confirm: bool, pub card_has_cancel: bool }
    pub struct DialogProps<'a> {
        pub title: gpui::SharedString,
        pub body: gpui::AnyElement,
        pub confirm_label: gpui::SharedString,
        pub cancel_label: Option<gpui::SharedString>,
        pub on_outcome: Option<Box<dyn Fn(DialogOutcome, &mut gpui::Window, &mut gpui::App) + 'a>>, // Option (#17) — aligns with Button.on_click + Switch.on_toggle; the R15 latch setters set_hover/set_press/poll are pub (host-driven)
    }
    #[derive(Default)]
    pub struct Dialog { /* — */ }
    impl Dialog {
        pub fn render(&mut self, colors: &ThemeColors, props: DialogProps<'_>) -> gpui::AnyElement;
        /// Pure decision seam (R11): the scrim/card composition this props yields (membership, not pixels).
        pub fn content_summary(props: &DialogProps) -> DialogContent;
        /// Pure decision seam (R12): which outcome a given control maps to.
        pub fn outcome_for(control: DialogControl) -> DialogOutcome;
    }
}

// ── Tooltip ──────────────────────────────────────────────────────────────────
pub mod tooltip {
    use super::ThemeColors;
    pub struct TooltipProps<'a> { pub target: gpui::AnyElement, pub text: gpui::SharedString, pub delay: std::time::Duration, pub offset: gpui::Pixels, pub _life: std::marker::PhantomData<&'a ()> }
    #[derive(Default)]
    pub struct Tooltip { /* persistent hover-since + visibility latch */ }
    impl Tooltip {
        pub fn render(&mut self, colors: &ThemeColors, props: TooltipProps<'_>) -> gpui::AnyElement;
        /// Pure decision seam (R13/R15/R16): is the tooltip card currently shown?
        pub fn is_visible(&self) -> bool;
    }
}

// ── Keyboard-shortcut chip ─────────────────────────────────────────────────────
pub mod keyboard_shortcut {
    use super::ThemeColors;
    pub struct KeyboardShortcut { keys: Vec<gpui::SharedString> }   // private field; constructed via parse
    impl KeyboardShortcut {
        pub fn parse(binding: &str) -> Self;
        /// Pure decision seam (R14): the ordered key labels, one per chip, no separator entry.
        pub fn keys(&self) -> &[gpui::SharedString];
        pub fn render(&self, colors: &ThemeColors) -> gpui::AnyElement;
    }
}

pub use keyboard_shortcut::KeyboardShortcut;
```

## EARS Requirements

- **R1.** The system shall render every widget purely as a function of the supplied `ThemeColors`, the widget's `Props`, and the widget's own persistent state, reading no process-global or thread-local state.
- **R2.** The system shall source every color a widget paints from a field of the supplied `ThemeColors` (via `button_colors`, `Switch::track_color`, the dialog/tooltip color paths), containing no hard-coded color literal.
- **R3.** WHEN `Button::label_text` is evaluated for `ButtonContent::Label(text)` (or `IconLabel(_, text)`), the system shall report `Some(text)`; for `ButtonContent::Icon(_)` it shall report `None`.
- **R4.** WHERE `ButtonContent::IconLabel(icon, text)` is supplied, the system shall report `ContentOrder::IconThenLabel` from `Button::content_order`, placing the icon to the leading (inline-start) side of the label.
- **R5.** WHEN `button_colors(ButtonKind::Primary, colors, VisualState::Rest)` is called, the system shall return `ButtonColors.background == colors.accent` and `ButtonColors.foreground == colors.on_accent`.
- **R6.** WHILE a button's pointer is inside its bounds and the button is enabled, the system shall report `VisualState::Hovered` from `Button::visual_state` rather than `VisualState::Rest`.
- **R7.** WHEN an enabled button receives a pointer press followed by a release inside its bounds, the system shall invoke its `on_click` handler exactly once.
- **R8.** WHILE `ButtonProps.disabled` is `true`, the system shall report `VisualState::Disabled` from `Button::visual_state` regardless of pointer state and shall not invoke `on_click` on press/release.
- **R9.** WHEN a switch is rendered, the system shall report `KnobPosition::Trailing` and `Switch::track_color(colors, true) == colors.accent` WHILE `SwitchProps.on` is `true`, and `KnobPosition::Leading` and `Switch::track_color(colors, false) == colors.surface` WHILE `SwitchProps.on` is `false`.
- **R10.** WHEN an enabled switch is activated by a pointer release inside its bounds, the system shall invoke `on_toggle` exactly once with the boolean negation of the current `SwitchProps.on` value.
- **R11.** WHEN `Dialog::content_summary` is evaluated, the system shall report `has_scrim == true` and a card carrying the title, body, and confirm control (`card_has_title && card_has_body && card_has_confirm`), with `card_has_cancel` equal to whether `cancel_label` is present.
- **R12.** WHEN `Dialog::outcome_for(DialogControl::Confirm)` is evaluated the system shall return `DialogOutcome::Confirm`, and WHEN `Dialog::outcome_for(DialogControl::Cancel)` is evaluated it shall return `DialogOutcome::Cancel`; activating the matching rendered control shall invoke `on_outcome` with that outcome.
- **R13.** WHILE a tooltip's pointer has hovered its target continuously for at least `TooltipProps.delay`, the system shall report `Tooltip::is_visible() == true`; WHEN the pointer leaves the target, the system shall report `Tooltip::is_visible() == false`.
- **R14.** WHEN `KeyboardShortcut::parse` is given a `+`-joined binding string, the system shall expose from `keys()` one entry per key in left-to-right binding order, with no entry for the `+` separator.
- **R15.** WHEN a stored widget instance is rendered, mutated by an interaction, and queried again via its decision accessor from the same instance, the system shall preserve its interaction state across the two renders (button `visual_state`, tooltip `is_visible`).
- **R16.** IF a fresh, default-constructed widget instance is queried, THEN the system shall report its rest state — `Button::visual_state == VisualState::Rest` (given an enabled, non-pressed props) and `Tooltip::is_visible() == false`.
- **R17.** WHERE no explicit palette is supplied, `ThemeColors::default_for(appearance)` shall return the Light fallback palette WHILE `appearance == Appearance::Light` and the Dark fallback palette WHILE `appearance == Appearance::Dark`, and the two palettes shall differ.

## Acceptance Criteria

| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | A widget render given identical `ThemeColors`+`Props`+state yields an identical element regardless of any ambient state (R1) | planned |
| 2 | Every painted color traces to a `ThemeColors` field via the color seams; no literal colors (R2) | planned |
| 3 | `Button::label_text` reports `"Run"` for `Label("Run")` and `None` for `Icon(_)` (R3) | planned |
| 4 | `content_order(IconLabel(..))` == `IconThenLabel` (icon leading) (R4) | planned |
| 5 | `button_colors(Primary, colors, Rest)` background==accent, foreground==on_accent (R5) | planned |
| 6 | Pointer-inside enabled button → `visual_state()==Hovered`; outside → `Rest` (R6) | planned |
| 7 | Press-then-release inside bounds fires `on_click` once (R7) | planned |
| 8 | Disabled button → `visual_state()==Disabled` and never fires `on_click` (R8) | planned |
| 9 | `on=true` → knob Trailing + track==accent; `on=false` → knob Leading + track==surface (R9) | planned |
| 10 | Toggling an `on=true` switch calls `on_toggle(false)` once (R10) | planned |
| 11 | `content_summary` reports scrim + card with title/body/confirm; cancel iff `cancel_label` present (R11) | planned |
| 12 | `outcome_for(Confirm)`==Confirm, `outcome_for(Cancel)`==Cancel; controls fire matching outcome (R12) | planned |
| 13 | Hover ≥ delay → `is_visible()==true`; leave → `is_visible()==false` (R13) | planned |
| 14 | `parse("cmd+shift+p").keys()` == `[cmd, shift, p]` with no separator entry (R14) | planned |
| 15 | Reused instance keeps hover/press + tooltip visibility across two renders (R15) | planned |
| 16 | A default-constructed instance → `visual_state==Rest`, `is_visible()==false` (R16) | planned |
| 17 | `default_for(Light)` ≠ `default_for(Dark)`, each the corresponding palette (R17) | planned |

## Visual / Behavioral Acceptance

Routed to the macOS accessibility (AXUIElement) + screenshot harness defined in
[../pipeline/visual-testing.spec.md](../pipeline/visual-testing.spec.md). The harness mounts a fixture
window with one of each widget rendered from a known `ThemeColors` palette; this section asserts the
**pixel/GPU-bound** halves that the headless decision seams above do not cover:

- **Button** — AX role `AXButton`, AXTitle equals `label_text`; rest padding, corner radius, and accent
  background match the captured Marley baseline within the screenshot tolerance; the hovered screenshot
  differs from the rest screenshot only in background color; the icon sits on the leading side of the
  label (the pixel realization of R4).
- **Switch** — AX role `AXCheckBox` with AXValue reflecting `on`; the `on` and `off` screenshots show
  the knob at opposite ends (the pixel travel of R9) with the on/off track colors.
- **Dialog** — exactly one **centered** card in the AX tree above a **full-window** dimmed scrim (the
  centering + full-bounds pixels of R11); confirm and (when present) cancel buttons are AX descendants
  of the card.
- **Tooltip** — after the hover delay, one floating static-text card carrying the text appears offset
  from the target by `offset` (the pixel offset of R13); it is absent before the delay and after
  pointer-leave.
- **KeyboardShortcut** — N ordered keycap chips, one per key, matching the captured Marley chip baseline.
- Screenshot regression baselines captured per widget and per visual state per the harness's baseline
  storage/approval policy; tolerance and headed-launch driver are defined by the harness, not here.

These cover the paint/GPU-bound halves of R3–R6, R9, R11, R13, R14 that have no headless harness; the
**decision** halves of those same clauses are unit-tested below.

## Test Plan

- **Unit (100% coverage on this component's touched lines; headless `gpui::TestAppContext` + pure
  decision accessors — no GPU required):**
  - `render_is_pure_function_of_inputs` → R1 (render twice from identical palette/props/state; compare structure)
  - `colors_trace_to_theme_colors_fields` → R2 (every value from `button_colors`/`track_color`/dialog/tooltip equals a `ThemeColors` field; no literal)
  - `button_label_text_from_content` → R3 (`Label`→`Some`, `IconLabel`→`Some(text)`, `Icon`→`None`)
  - `icon_label_orders_icon_before_label` → R4 (`content_order(IconLabel)==IconThenLabel`)
  - `primary_rest_colors_from_palette` → R5 (`button_colors(Primary,_,Rest)` background==accent, fg==on_accent)
  - `button_hover_selects_hovered_state` → R6 (drive synthetic pointer-enter/leave; assert `visual_state`)
  - `button_press_release_fires_on_click_once` → R7 (click counter == 1)
  - `disabled_button_state_and_no_click` → R8 (`visual_state==Disabled`; counter == 0)
  - `switch_knob_side_and_track_color` → R9 (both `on` values: `knob_position` + `track_color`)
  - `switch_toggle_invokes_with_negation_once` → R10
  - `dialog_content_summary_scrim_and_card` → R11 (scrim + title/body/confirm; cancel iff label present)
  - `dialog_outcome_mapping_and_controls` → R12 (`outcome_for` both arms + control activation)
  - `tooltip_visible_after_delay_hidden_on_leave` → R13 (advance the test clock past/under `delay`; assert `is_visible`)
  - `keyboard_shortcut_parse_orders_keys` → R14 (`keys()` order, no separator entry)
  - `reused_instance_preserves_state` → R15 (`visual_state`/`is_visible` survive a second render of the same instance)
  - `default_instance_is_rest_state` → R16 (`visual_state==Rest`, `is_visible()==false`)
  - `default_for_branches_light_dark` → R17 (`default_for(Light) != default_for(Dark)`)
- **Integration (cross-crate seam with `gpui-component`):** mount all five widgets in one host `Render`
  view from a single `ThemeColors`, drive a scripted pointer/clock sequence, and assert the handler
  callbacks (`on_click`/`on_toggle`/`on_outcome`) fire in order with the expected payloads — exercising
  the per-widget `render(&ThemeColors, props)` + decision-accessor seam end-to-end against real gpui
  element dispatch (no `Component`/`Params`/`Options` trait triad).
- **Visual (headed launch):** the AXUIElement + screenshot assertions in the section above, run through
  [../pipeline/visual-testing.spec.md](../pipeline/visual-testing.spec.md).
- **Regression:** the full `marley_ui_components` unit + integration suite stays green and the per-widget
  screenshot baselines match before any merge; theming stays data-driven (R1/R2/R17 keep passing) after
  any `gpui`/`gpui-component` bump.

## Mutation Targets

`cargo mutants` must kill every viable mutant on: `button_colors` (kind × `VisualState` arm mapping —
swap any arm or color field), `Button::visual_state` state machine and the `on_click`
once-and-only-once gate, `Button::content_order`/`label_text`, `Switch::knob_position` +
`Switch::track_color` selection and the toggle-negation in `on_toggle`, `Dialog::content_summary`
membership flags and `Dialog::outcome_for` confirm/cancel mapping, the tooltip `delay` comparison and
`is_visible` latch (flip `>=`, drop the leave reset), `KeyboardShortcut::parse`/`keys()` split+order,
`ThemeColors::default_for` Light/Dark branch, and the instance state-persistence path (R15). MSI 100%
on this decision surface.

ACCEPTED-UNTESTABLE: the actual GPU paint of element trees and the OS pointer/event-loop delivery —
exercised only by the headed AXUIElement + screenshot harness — are excluded from `cargo-mutants` via
`mutants::skip` with `// justification: headed GPU/display path, covered by the visual harness`. All
decision logic (color selection, visual state, knob/track choice, content order, dialog composition +
outcome, tooltip visibility, parse, palette default) is exposed as a pure accessor, is unit-testable,
and is NOT excluded.

## Dependencies

- REUSE (permissive): `gpui` (Apache-2.0 — element/`Render` model, styling, pointer events, `Hsla`,
  `Pixels`, `Edges`, `SharedString`, `TestAppContext`), `gpui-component` (Apache-2.0 — the underlying
  styled button/switch/overlay primitives the widgets compose, and `IconName`). Both MIT/Apache —
  supply-chain clean per gate 8.
- Marley components: **none upstream** — this crate is the owner of the theming value vocabulary
  (`Appearance`, `ThemeColors`) and the five widgets (seam-contracts §6). Downstream, **`marley_app`
  depends on this crate** for `Appearance`/`ThemeColors` (it wraps them in `Theme`/`ThemeRegistry` and
  supplies the M1 palette) and the Marley panel/workspace UI composes these widgets. `marley_app`
  declares no `Appearance`/`ThemeColors` of its own (no cycle; natural direction).

## Out of scope / deferred

- The concrete brand palettes / theme registry and any brand-string/asset sourcing — `marley_app` owns
  `Theme`/`ThemeRegistry` and the M1 palette (this crate ships only the value types + a `default_for`
  fallback).
- Lightbox and segmented/control-cluster widgets and any non-listed widgets (later widget-set spec).
- Asset-cache-backed icon loading internals (icons consumed via a handle; `marley_asset_core`/
  `marley_assets` own caching — seam-contracts §10).
- Keyboard focus traversal / full keymap-driven activation beyond the explicit confirm/cancel and
  pointer paths above (M2 input-routing spec).
- Animation/transition curves for hover, knob travel, and dialog entry (later motion spec); M1 asserts
  end states only.

## Clean-room provenance

Behavior-derived from a fork-reference doc describing observable widget I/O only — visible label/icon
content and order, hover/press/disabled visual states, switch knob side + track color, dialog
scrim/card composition and confirm/cancel outcome, delayed tooltip visibility, keycap-chip ordering,
and a typed light-vs-dark color palette — with no private module/type/static names and no fork file
paths. Per seam-contracts §11, the `clean_room` line is downgraded to
`behavior-derived from a fork-reference doc; IP-counsel sign-off pending` until the behavioral wall +
sign-off land (open item in `clean-build-plan.md`). The public surface uses **Marley-original**
identifiers throughout: the round-1 `struct Appearance { accent, surface, … }` is renamed to
`ThemeColors` and the bare `Appearance` is now the Light/Dark enum (both owned here, seam-contracts §6);
the `Component`/`Params`/`Options` trait triad is struck entirely and replaced by per-widget structs
exposing `render(&ThemeColors, props)` plus pure decision accessors (seam-contracts §6.1). No
Warp-internal name appears. REUSE crates (`gpui`, `gpui-component`) are Apache-2.0.
