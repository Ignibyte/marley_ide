---
pipeline_id: 2b88f960-2379-4da2-9ba6-e48e532c2158
aar_id: 2946f859-4356-4651-9bbc-25322c038e0c
---

# marley_ui_components widgets — pipeline notes

## Phase 1 — Plan (2026-07-01)

**Intent:** add the 5 SPEC-ui-components widgets (button/switch/dialog/tooltip/keyboard-shortcut, R1-R16)
to the existing marley_ui_components crate. M1.B Cockpit seq 1/5 — the primitives the palette (#19) +
settings (#21) render with. Each = a long-lived struct + persistent interaction state + a `&ThemeColors`
render + PURE decision accessors.

**Decision B (hand-roll, no gpui-component):** gpui-component 0.5.1 resolves cleanly with gpui 0.2.2
(declares gpui ^0.2.2, crates.io, no git-pin conflict) but pulls 32 non-optional deps for 5 widgets. So
hand-roll the renders on raw gpui (app.rs pattern) + a Marley-owned `IconName`. Spec-delta: reuses→[gpui]
+ IconName Marley-owned (apply in the §21 arch/spec update).

## Carry to Design (Phase 2) — the analysis (verified against the spec + the existing crate)

### Module manifest (crates/ui_components/src/)
- **PURE (cov 100/MSI 100 — gpui-Hsla/Pixels/SharedString only):** `button.rs`, `switch.rs`, `dialog.rs`,
  `tooltip.rs`, `keyboard_shortcut.rs`, `icon.rs` (Marley `IconName`). lib.rs (Appearance/ThemeColors)
  UNTOUCHED — add `pub mod` decls + re-exports + drop its "widgets deferred to M1.B" doc note.
- **SHIM (ACCEPTED-UNTESTABLE, mutants::skip per fn + rust_cov exclude `ui_components/src/render/|
  ui_components/src/bin/`):** `render/mod.rs` (shared chrome/card/scrim/chip helpers) + `render/{button,
  switch,dialog,tooltip,keyboard_shortcut}.rs` (a 2nd impl block per widget: `render(&mut self,
  &ThemeColors, props)->gpui::AnyElement` + on_hover/on_mouse_down/up handlers calling the pure latches)
  + `bin/marley_widgets_gallery.rs` (the headed fixture).
- Cargo.toml: NO gpui-component; add dev `marley_visual_harness`(path) + `[[bin]] name=
  "marley_widgets_gallery"`. gates.sh: extend rust_cov `--ignore-filename-regex` with
  `ui_components/src/render/|ui_components/src/bin/` + the comment.

### Per-widget PURE accessors (R#)
- **Button** `{hovered:bool, pressed:bool}` (Default→Rest, R16): `visual_state(props)` — precedence
  `disabled?Disabled (R8) : pressed?Pressed : hovered?Hovered (R6) : Rest`; `set_hover(bool)`/
  `set_press(bool)` latch writes (R15); `label_text(props)->Option<SharedString>` (R3: Label/IconLabel→
  Some, Icon→None); `content_order(props)` (R4: Label→LabelOnly, Icon→IconOnly, IconLabel→IconThenLabel);
  `button_colors(kind,&ThemeColors,state)->ButtonColors` (R2/R5 — per-(kind,state) table, EVERY cell an
  existing ThemeColors field [no literal], MUTUALLY DISTINCT per state so arm-swaps are killable; kind
  base: Primary→accent, Secondary→surface, Destructive→danger, Ghost→background; fg on_accent|foreground;
  R5 pinned `(Primary,Rest)→{background:accent, foreground:on_accent}`; `(_,Disabled)→{background:surface,
  foreground:border, border:border}`).
- **Switch** `{hovered:bool}`: `knob_position(on)` (R9: on→Trailing/off→Leading); `track_color(&,on)`
  (on→accent/off→surface); `knob_travel_x(on, progress:f32, track:Pixels, knob:Pixels)->Pixels` (lerp
  inset→target, target=on?track-knob-inset:inset, progress.clamp(0,1) — progress is an INPUT); set_hover.
- **Dialog** (unit): `content_summary(props)->DialogContent` (R11: has_scrim/card_has_title/body/confirm
  =true, card_has_cancel=props.cancel_label.is_some()); `outcome_for(DialogControl)->DialogOutcome` (R12:
  Confirm→Confirm, Cancel→Cancel).
- **Tooltip** `{visible:bool}` (default false, R16): `is_visible()->bool` (the latch, R13); `poll(&mut,
  hovering:bool, elapsed:Duration, delay:Duration)` (R13: `!hovering→visible=false` else `visible=elapsed
  >= delay` — elapsed is an INPUT, clock-free).
- **KeyboardShortcut** `{keys:Vec<SharedString>}`: `parse(&str)` (R14: split('+').map(trim).filter(!empty)
  .map(SharedString) → ordered keys, no empty entries — R14 uses `+` separator, authoritative);
  `keys()->&[SharedString]`.

### Mutation map (per accessor → the killing test)
visual_state: disabled+hover→Disabled(R8) / hover-not-press→Hover(R6) / press→Press / default→Rest(R16) +
the arm swaps. set_hover/press: set_hover(true)→same instance visual_state==Hover (R15). button_colors:
assert each cell EXACTLY [(Primary,Rest)→accent/on_accent (R5); (Destructive,_)→danger; (_,Disabled)→
surface/border] + distinct-per-state. label_text: Label→Some/IconLabel→Some/Icon→None (R3). content_order:
the 3 arms (R4). knob_position: on→Trailing/off→Leading. track_color: accent/surface. knob_travel_x:
progress=1,on→track-knob-inset / on=false→inset / progress=2 clamps / progress=0 start. content_summary:
all flags true + cancel=Some→true / None→false (R11). outcome_for: Confirm/Cancel (R12). poll/is_visible:
elapsed==delay→visible (kills `>`) / elapsed<delay→hidden / hovering=false→hidden (kills dropped reset) /
default→false (R16). parse: `"cmd+shift+p"`→[cmd,shift,p] / `"cmd++p"`→[cmd,p] (no empty) / order-sensitive.
keys: length+element equality. FIXTURES: a `dark()`=ThemeColors::default_for(Dark); a ButtonProps builder;
a Duration pair (delay=300ms, elapsed∈{299,300,301}ms) for the `>=` boundary.

### Headed-baseline plan (gate-15)
`src/bin/marley_widgets_gallery.rs` (#[cfg_attr(test,mutants::skip)] main; title "Marley Widgets", Dark
theme; renders ONE widget in ONE forced state selected by `MARLEY_WIDGET_FIXTURE` — latches PRE-SET
[set_hover(true)/poll past delay] for DETERMINISTIC captures, no OS pointer injection). `tests/
headed_widgets.rs` (#[ignore], per (widget,state): HeadedSession::launch(CARGO_BIN_EXE_marley_widgets_
gallery, "Marley Widgets", 15s) → AX assert [Button→AxRole::Button+title=label; Switch→CheckBox+AXValue;
Dialog→one centered Group card + confirm/cancel Buttons; Tooltip→one StaticText after delay; Shortcut→N
ordered chips] + shot.assert_matches_baseline_masked(manifest, target, name, Tolerance::gate15_text(),
&RegionMask::titlebar_band(w,h,70))). Baselines at tests/visual/baselines/: button_rest_dark/button_hover_
dark/button_disabled_dark/button_icon_label_dark/switch_off_dark/switch_on_dark/dialog_dark/tooltip_hidden_
dark/tooltip_shown_dark/shortcut_chips_dark (promote via MARLEY_VISUAL_APPROVE=1). gate-15 CI runs the
HARNESS's own tests, NOT this #[ignore] lane (needs a display).

### Risks
gpui-component (de-risked/moot for B). Persistent state: each widget is a field on the host Entity (gpui
re-renders the same instance — never reconstruct per frame). is_visible/knob_travel_x PURE (elapsed/
progress as inputs, the render owns the Instant). The R2-hover tension: button_colors field-equality-clean
+ the hover-bg overlay in the EXCLUDED render. The spec-delta (reuses→[gpui] + IconName Marley-owned) —
apply in §21. WRITE-FIRST for the render code (app.rs exemplar) — do NOT research gpui.

**Phase 1 status:** PASS (autonomous per chad's /goal). → Phase 2 Design.

## Phase 2 — Design (2026-07-01)

**Confirmed the Carry-to-Design against SPEC-ui-components' EXACT Public surface — it MATCHES.** The
authoritative types + signatures (implement writes THESE exactly):
- **Button:** `enum ButtonContent{Label(SharedString), Icon(IconName), IconLabel(IconName,SharedString)}`,
  `enum VisualState{Rest,Hovered,Pressed,Disabled}`, `enum ContentOrder{LabelOnly,IconOnly,IconThenLabel}`,
  `enum ButtonKind{Primary,Secondary,Destructive,Ghost}`, `struct ButtonColors{background,foreground,border:
  Hsla}`, `struct ButtonProps<'a>{content:ButtonContent, kind:ButtonKind, /* + disabled + label/icon */}`,
  `struct Button{/* hover/press latches */}`. `impl Button`: `render(&mut self,&ThemeColors,ButtonProps)->
  AnyElement`, `visual_state(&self,&ButtonProps)->VisualState`, **STATIC** `label_text(props:&ButtonProps)
  ->Option<SharedString>` + `content_order(props:&ButtonProps)->ContentOrder`. **FREE fn** `button_colors(
  kind:ButtonKind, colors:&ThemeColors, state:VisualState)->ButtonColors`.
- **Switch:** `enum KnobPosition{Leading,Trailing}`, `struct SwitchProps<'a>`, `struct Switch{/* hover
  latch */}`. `impl Switch`: `render`, `knob_position(&self,on:bool)->KnobPosition`, `track_color(&self,
  &ThemeColors,on:bool)->Hsla`.
- **Dialog:** `enum DialogOutcome{Confirm,Cancel}`, `enum DialogControl{Confirm,Cancel}`, `struct
  DialogContent{has_scrim,card_has_title,card_has_body,card_has_confirm,card_has_cancel:bool}`, `struct
  DialogProps<'a>{on_outcome:Box<dyn Fn(DialogOutcome,&mut Window,&mut App)+'a>, /* +title/body/labels */}`,
  `struct Dialog`. `impl Dialog`: `render`, **STATIC** `content_summary(props:&DialogProps)->DialogContent`
  + `outcome_for(control:DialogControl)->DialogOutcome`.
- **Tooltip:** `struct TooltipProps<'a>{target:AnyElement, text:SharedString, delay:Duration, offset:Pixels,
  _life:PhantomData}`, `struct Tooltip{/* hover-since + visibility latch */}`. `impl Tooltip`: `render`,
  `is_visible(&self)->bool`.
- **KeyboardShortcut:** `struct KeyboardShortcut{keys:Vec<SharedString> /* private */}`. `impl`: `parse(
  binding:&str)->Self`, `keys(&self)->&[SharedString]`, `render(&self,&ThemeColors)->AnyElement`.

**Refinements to the Carry-to-Design (spec-driven):**
1. `label_text`/`content_order`/`content_summary`/`outcome_for` are STATIC (take `props`/`control`, NO
   `&self`) — as the notes have them. `button_colors` is a FREE fn. Match exactly.
2. **Tooltip:** only `is_visible(&self)->bool` is PUBLIC. The threshold logic (`hovering && elapsed >=
   delay`) is a PURE `pub(crate) fn` helper (e.g. `visibility_for(hovering, elapsed, delay)->bool` OR
   `poll(&mut self, hovering, elapsed, delay)`) in tooltip.rs — unit-tested for cov/MSI 100 (the `>=`
   boundary); the EXCLUDED render/tooltip.rs owns the `Instant` (hover-since) + calls it. `is_visible`
   just reads the latch.
3. **DROP `knob_travel_x`** — NOT in the spec. The switch's pure surface = `knob_position` (Leading/
   Trailing) + `track_color`; the render (excluded) places the knob px at the end from `knob_position`
   (the switch_off/switch_on baselines capture the two ends — no animation-math needed).
4. `set_hover(&mut self,bool)` / `set_press(&mut self,bool)` (Button) + the hover setter (Switch) are
   `pub(crate)` design-additions — the EXCLUDED render's event handlers call them; unit tests call them to
   drive `visual_state` (proving R15 persistent-state on the SAME instance). Not in the public surface.
5. **`IconName` Marley-owned** (icon.rs) — the spec says "reused from gpui-component" but decision B makes
   it Marley-original (a `pub enum IconName{...}` or `struct IconName(SharedString)`). SPEC-DELTA for §21:
   amend `reuses: [gpui, gpui-component]`→`[gpui]` + the IconName-source line. Cleaner clean-room.
6. `ButtonProps`/`SwitchProps`/`DialogProps` exact fields: read the spec's full Public surface at implement
   (the grep truncated — ButtonProps has `disabled` [visual_state checks it], Dialog has title/body/labels,
   Switch has `on`). Match the spec's field names exactly.

**Manifest + seam + test plan + headed plan:** as in the Carry-to-Design (module split pure button/switch/
dialog/tooltip/keyboard_shortcut/icon.rs; SHIM render/ subdir + bin/marley_widgets_gallery.rs; rust_cov
exclude `ui_components/src/render/|ui_components/src/bin/`; the mutation map; the #[ignore] headed gallery
with MARLEY_WIDGET_FIXTURE deterministic states + per-widget AX + masked baselines). The R2-vs-hover
tension (button_colors field-equality-clean + the hover-bg overlay in the excluded render) CONFIRMED.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-07-01)

Built (I took over after an implement subagent research-stalled on gpui — the #16 lesson recurred; wrote
the pure modules myself, then the render write-then-check against app.rs → 1 compiler iteration).

**Files:** `crates/ui_components/src/` — PURE: `icon.rs` (Marley `IconName`), `button.rs`, `switch.rs`,
`dialog.rs`, `tooltip.rs`, `keyboard_shortcut.rs`; SHIM (mutants::skip + cov-excluded): `render/{mod,
button,switch,dialog,tooltip,keyboard_shortcut}.rs` + `bin/marley_widgets_gallery.rs`. `lib.rs` (mod
decls + `pub use IconName, KeyboardShortcut`; dropped the "deferred to M1.B" note). `Cargo.toml` ([[bin]]
+ dev `marley_visual_harness`). `scripts/gates.sh` (rust_cov exclude `ui_components/src/render/|.../bin/`
+ the documented comment). **check --all-targets 0 err, clippy -D 0, fmt clean, rustdoc -D clean, no
`unsafe`.**

**Deviations / spec-deltas (all sound):**
1. **Decision B applied** — hand-rolled on raw gpui, NO gpui-component; `IconName` Marley-owned
   (`icon.rs`). SPEC-DELTA (apply in §21): `reuses: [gpui, gpui-component]`→`[gpui]`; the "IconName reused
   from gpui-component" comment → Marley-owned.
2. **State mutators are PUBLIC** — `Button::set_hover`/`set_press`, `Switch::set_hover`, `Tooltip::poll`
   are `pub` (the spec listed them implicitly under `/* latches */`). Rationale: a `render(&mut self, …)
   -> AnyElement` cannot install live gpui pointer handlers without a `Context`, so the HOST drives the
   latches from its own `cx.listener` (R15). They stay pure + tested. Added `Switch::is_hovered()`
   (`pub(crate)`) so `render/switch.rs` reads the hover latch (a hover knob-color treatment) — else the
   `hovered` field is write-only/dead.
3. **`button_colors`** — `Disabled`→`{surface, border, border}`; `Rest/Hovered/Pressed` share the kind
   fill/fg but differ in the BORDER role (`border`/`accent`/`foreground`) so all 4 states are distinct
   (arm-swaps killable). R5 pinned: `(Primary, Rest)`→`{accent, on_accent, border}`. The hover/press
   *background* pixel delta stays a render concern (excluded).
4. **Closure handler fields** factored into documented `type` aliases (`ButtonClick`/`SwitchToggle`/
   `DialogOutcomeHandler`) for clippy `type_complexity` — same types, aliased.
5. **`KeyboardShortcut::parse`** maps each segment via `SharedString::from(seg.to_string())` — the bare
   `SharedString::from(&str)` resolves to the `From<&'static str>` impl (zero-copy) and forces
   `binding: 'static` (E0521); the owned `String` conversion fixes it.

**Carry to Validate:** the pure surface (all accessors + `button_colors` + the latches/mutators +
`IconName`) needs cov 100/MSI 100 — use the Phase 2 mutation map (the `dark()=default_for(Dark)` fixture,
the ButtonProps builder, the tooltip `(299/300/301ms, delay 300ms)` `>=` boundary, `button_colors`
distinct-per-state cells, `parse("cmd++p")`→`[cmd,p]`). The `#[ignore]` headed widget-gallery test
(`tests/headed_widgets.rs`) uses `marley_visual_harness` (clears machete). render/ + bin excluded.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 2026-07-01

Inspected in-context (2 read-only critics stalled → the recurring gpui-subagent research-paralysis;
the `cargo mutants --list` output is the objective oracle, so I used it + direct greps instead).

**Findings:**
1. **[HIGH — FIXED] The gallery bin leaked 8 mutants** (`bin/marley_widgets_gallery.rs`: the 4 render
   match arms + `main` + 3 `WindowOptions`/`TitlebarOptions` field deletions). It's shim exercised ONLY
   by the `#[ignore]` headed test → those mutants would SURVIVE → MSI<100. ROOT: the rust_cov exclude
   (coverage) and `mutants::skip` (mutation) are SEPARATE mechanisms — excluding the bin from coverage
   does NOT exclude it from mutation. FIX: added `#[cfg_attr(test, mutants::skip)]` to `GalleryView::new`,
   `<impl Render>::render`, and `main` (the #16 `marley.rs` pattern). Re-listed: **0 bin mutants, 0
   render mutants, 24 pure-surface mutants.**
2. **[verified clean] Seam** — `mutants::skip` on EVERY render/ fn + the 3 bin fns, NONE in the 6 pure
   modules; the gates.sh regex matches render/ + bin only; the pure files stay in the 100% denominator.
3. **[verified clean] Clean-room (§20)** — `grep -rin warp crates/ui_components` = 0; `IconName` is a
   plain Marley-original enum (not gpui-component's); all names Marley-original.
4. **[verified clean] §14** — no `unwrap`/`expect`/`panic`/`unreachable` in the 6 pure modules; `parse`
   cannot panic on any `&str` (split/trim/filter are total). The bin's `env::var().unwrap_or_else` is a
   shim default.
5. **[verified — NOT a problem] `button_colors` "equal cells"** — MOOT for mutation: the whole-fn
   `Default::default()` mutant is UNVIABLE (`ButtonColors` derives no `Default`), and cargo-mutants emits
   no per-arm/field mutant for the two `match`es → `button_colors` has ZERO viable mutants (coverage-only:
   every kind+state arm must be executed). The cells ARE distinct anyway (Rest/Hovered/Pressed differ in
   border = `border`/`accent`/`foreground`; Disabled = surface/border/border; kinds differ in background =
   accent/surface/danger/background) — good for the explicit assertions.

**CARRY TO VALIDATE — the mutation kill map (24 mutants; the VIABLE ones + their killing test):**
- `label_text`: `→None` killed by `Label("Run")→Some("Run")`; `→Some(Default)` killed by asserting the
  EXACT `Some("Run")` (≠ `Some("")`); + `Icon(_)→None`. (`visual_state`/`content_order`/`button_colors`
  `→Default` are UNVIABLE — coverage-only: exercise disabled+hover→Disabled / hover→Hovered / press→
  Pressed / default→Rest; Label/Icon/IconLabel; button_colors for ALL 4 kinds × {Rest, Disabled}.)
- `set_hover`/`set_press` (button) `→()`: killed by `set_hover(true)` then `visual_state(&default_props)
  ==Hovered` (and `set_press(true)`→`Pressed`) on the SAME instance (R15).
- `track_color` `→Default`: killed by `track_color(&dark, true)==dark.accent` AND `(&dark,false)==
  dark.surface` (EXACT). (`knob_position→Default` unviable — coverage: on→Trailing, off→Leading.)
- `switch set_hover→()` + `is_hovered→true/→false`: default `is_hovered()==false`; after `set_hover(true)`
  `is_hovered()==true`.
- `content_summary`/`outcome_for` `→Default` UNVIABLE (coverage-only): `cancel_label=Some`→`card_has_cancel`
  true, `None`→false, + the 4 always-true flags; `outcome_for(Confirm)==Confirm`, `(Cancel)==Cancel`.
- `tooltip`: `is_visible→true` killed by default `==false`; `→false` killed by post-`poll(true,300,300)`
  `==true`; `poll→()` killed by that same; `poll &&→||` killed by `poll(false, 300ms, 300ms)`→hidden;
  `poll >=→<` killed by `poll(true, 300ms, 300ms)`→visible (the elapsed==delay boundary).
- `keyboard_shortcut`: `parse` `delete !` killed by `parse("cmd++p").keys()==["cmd","p"]` (no empty);
  `keys→leak(empty)`/`→leak([Default])` killed by asserting the EXACT `["cmd","shift","p"]`.
- `lib default_for→Default` UNVIABLE (`ThemeColors` no `Default`) — the #13 tests cover the branch.
- **Fixtures:** `dark() = ThemeColors::default_for(Appearance::Dark)`; a ButtonProps builder (`on_click:
  None`); the Duration boundary `(delay=300ms, elapsed∈{299,300,301}ms)`.
- **Props test-constructibility (VERIFY FIRST in validate):** `DialogProps.body` / `TooltipProps.target`
  are `gpui::AnyElement` — construct via `gpui::div().into_any_element()` (or `gpui::Empty` if div needs a
  cx) in a plain `#[test]` (no App). If neither builds headlessly, that's a HIGH — but element
  *construction* (not layout) needs no context, so it should work.
- **machete:** validate MUST add `tests/headed_widgets.rs` using `marley_visual_harness` (else the unused
  dev-dep → gate:9 RED). NO serial_test/tempfile (widgets spawn no process). `mutants` is used (skip attrs).

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-07-01)

Wrote the tests in-context (per the kill map; no subagent — the pure tests are deterministic).

**Tests added:** 22 in-crate `#[cfg(test)]` unit tests across the 5 pure modules (button 9, switch 3,
dialog 3, tooltip 5, keyboard_shortcut 3) + `tests/headed_widgets.rs` (#[ignore] — AX title + size +
blank-detector, models headed_shell.rs, consumes marley_visual_harness). `cargo nextest run -p
marley_ui_components` = **27 passed** (22 new + the 5 pre-existing #13 theme tests); the headed test is
#[ignore] (headed lane).

**The dialog `AnyElement`-in-test risk (inspect's HIGH) — RESOLVED:** `gpui::div().into_any_element()`
constructs fine in a plain `#[test]` (no App) — element CONSTRUCTION needs no gpui context; the
`content_summary` tests build `DialogProps` headlessly + pass.

**One validate fix (coverage):** the first gate run was RED on gate:4 — `dialog.rs` had 1 uncovered
function: the test's `on_outcome: Box::new(|_,_,_| {})` closure was never invoked (content_summary/
outcome_for don't call it, and it can't be called headlessly — it needs `&mut Window`/`&mut App`). FIX
(source, not a floor drop): changed `DialogProps.on_outcome` to `Option<DialogOutcomeHandler<'a>>` — which
ALIGNS it with `Button.on_click` + `Switch.on_toggle` (both already `Option` in the spec; the spec made
only `on_outcome` non-Option, an inconsistency); the test now passes `None`. render/dialog.rs doesn't
reference on_outcome (unaffected); the gallery bin's DialogProps → `None`. SPEC-DELTA for §21: on_outcome
is now Option. Re-ran → dialog.rs 100%.

**FULL gate (scripts/gates.sh --diff): `GATE GREEN [diff]` — 15/15**, cov 100% (pure surface; render/+bin
excluded), mutation MSI 100% (0 missed; the 24-listed minus unviable Default::default() whole-fn mutants),
machete green (marley_visual_harness now used by the headed test), gate-15 visual PASS (the harness's own
headless suite; the #[ignore] headed widget test is the headed lane). Receipt written.

**Phase 4 status:** PASS. → Phase 5 Complete.
