# Warp session sidebar & tab-row styling — Notes

- **Forge ticket:** #219 (6de7c89d-96b2-4f5d-9cc7-c875c1dad801)
- **AAR:** 1d75b4a5-6514-4ab5-bce9-cc38ac424ddd
- **Local ticket doc:** docs/planning/tickets/open/TICKET-219-warp-sidebar-styling.md
- **Pipeline spec:** warp-sidebar-styling.spec.md

## Phase 1 — Plan
- **Request:** match Warp's left session list. Auto-approved (/work 195-222).
- **Classification / tier:** work pipeline, small/bounded. Systems: app.rs left-rail render (shim) + a
  small pure highlight helper (cov/MSI 100). Pure model `tabs::rail_rows`/`RailRow`/`RailLevel` unchanged.
- **Forge recall (§18.3):** #194 added the `contrast_ratio` WCAG helper (reuse for the distinct-from-surface
  guard); #191 established the accent focus idiom (active PANE = 4-side accent border) — the active TAB
  highlight should rhyme with it. The generic mutation traps apply (f32/hsla literals aren't mutated →
  exact-value asserts). AAR opened.
- **Discovery (measured, the edit surface for Design):**
  - The rail renders inside `dock_panel(DockSide::Left)` — app.rs:364 fills it `bg(colors.surface)`.
  - The pure model: `rail_rows(&ws) -> Vec<RailRow{level,label,active,project,tab,pane}>`,
    `RailLevel{Workspace,Project,Tab,Pane}` (tabs.rs ~415-490; cov/MSI 100). No color/style in the model.
  - Render loop app.rs ~3721-3945: Workspace header (px_2 py_1 text 11 muted uppercased); Project row
    (pl16; active→foreground else muted; name·branch + ×); Tab row (pl28 py2 text13; the #167 agent glyph
    BEFORE the label only for agent tabs; label + ×; **active → `entry.bg(colors.surface).text_color(foreground)`
    else muted**); Pane row (pl40 py1 text12; **active → `entry.bg(colors.surface).text_color(foreground)`
    else muted**).
  - **THE BUG:** active fill `colors.surface` == the dock bg `colors.surface` → the highlight is INVISIBLE.
    In `219-marley-sidebar.png` the active tab ('terminal 4') shows only brighter text, no box. Warp
    (`219-warp-sidebar.png`) = a distinct rounded lighter selected box + row hover highlight.
- **Bounded deltas:** D-A active Tab/Pane highlight → a DISTINCT rounded elevated fill (recommend
  accent-tinted); D-B a subtle row hover highlight; pure `rail_active_highlight` helper guarded
  distinct-from-surface (pins the fix, cov/MSI 100).
- **Deferred (documented):** per-row leading terminal icon (the rail is an indent-based TREE + already
  glyphs agent tabs — a larger separate call); 2-line subtitles (data plumbing); row height/indent + the
  header (already fine).
- **Decisions:** D1 distinctness is the fix + guarded by a pure test; D2 pure helper is the single source
  (cov/MSI 100), render is the shim (capture-validated); D3 accent-tinted recommended (design confirms the
  Hsla alpha/lightness API + value; tokens only); D4 auto-approved, document with captures.
- **Open questions for Design:** (1) the exact highlight — accent at reduced alpha (e.g. `accent.opacity(α)`)
  vs a solid lightness-step above surface; does gpui `Hsla` expose `.opacity()`/an alpha field cleanly? (2)
  the helper HOME so it's pure-testable — `ThemeColors` is a gpui-typed struct (ui_components); the helper
  can take `&ThemeColors` and live in app.rs with a `#[cfg(test)]` test (gpui types are fine in a unit test),
  OR take the raw `surface`/`accent` `Hsla` and live in tabs.rs. (3) the distinct-from-surface guard metric
  + threshold (contrast_ratio ≥ X, or |L_highlight − L_surface| ≥ δ). (4) does the hover bg also route
  through the helper (same color, lower intensity) or a separate value?

## Phase 2 — Design

### Discovery verified (the open questions, answered)
- **(1) gpui `Hsla` API.** `gpui::Hsla { pub h, pub s, pub l, pub a }` — all fields public (color.rs).
  `Hsla::opacity(&self, factor) -> Self` (color.rs:549) = `{h,s,l, a: self.a * factor.clamp(0,1)}`.
  `gpui::hsla(h,s,l,a)` free fn + field access are ALREADY used in app.rs (line 2227 `gpui::hsla(m.h,m.s,m.l,0.5)`),
  so an accent wash is idiomatic + import-free. **Chosen highlight** = an accent wash:
  - active fill = `colors.accent.opacity(0.22)` → `hsla(0.52,0.58,0.55,0.22)` (accent.a is 1.0).
  - hover fill = `colors.accent.opacity(0.10)` → `hsla(0.52,0.58,0.55,0.10)`.
  Measured composite over `surface` (L=0.155): active L ≈ 0.155·0.78 + 0.55·0.22 ≈ **0.24** (+0.087 & a cyan
  hue-shift → a clearly-visible tinted rounded box); hover L ≈ **0.195** (+0.04 → a subtle wash). Ties to
  #191's accent focus identity; tokens only (derived from `accent`); §20 clean.
- **(2) Pure helper + home.** ONE small free fn in app.rs near `dock_panel`:
  `fn rail_highlight(colors: &ThemeColors, active: bool) -> gpui::Hsla { colors.accent.opacity(if active { 0.22 } else { 0.10 }) }`
  — `true` = the selected fill, `false` = the hover fill. Pure Hsla math (no gpui element calls). It's a FREE
  fn, NOT inside the `#[cfg_attr(test, mutants::skip)]` render, so it IS mutation-tested (the point). Home =
  app.rs (co-located with the render, takes the gpui-typed `&ThemeColors`; gpui types are fine in a
  `#[cfg(test)]` unit test). No `tabs.rs` (that stays the gpui-free pure MODEL; a color helper needing `Hsla`
  belongs beside the render).
- **(3) Distinct-from-surface guard** (encodes the exact invisibility bug so it can't regress). The test
  asserts, for BOTH themes: `hl.a < 1.0` (a translucent wash — regressing to `bg(colors.surface)` gives
  a=1.0 → FAILS), `hl != colors.surface` (the literal bug — the fill must differ from the dock bg), and
  `hl.h == colors.accent.h` (it's an accent wash, hue 0.52 ≠ surface hue 0.62). No `contrast_ratio` needed
  (kept available, pub). Plus exact-value asserts pin the formula + kill mutants (below).
- **(4) Rounding.** `.rounded(colors.corner_radius)` (px(6)) on the Tab/Pane entry → the Warp rounded-box
  look. **Bonus:** this RE-CONSUMES `colors.corner_radius`, partially addressing the #224 orphan (the rail now
  uses it) — but #224 (palette/dialog/find-bar cards) stays open for those surfaces; note the partial re-consume.

### Architecture / approach
- **PURE seam** (app.rs free fn): `rail_highlight` — gpui-free math, cov/MSI 100 (exact-value both bools/both
  themes + the distinct-from-surface guard). §14 clean (no IO, no panic).
- **SHIM** (app.rs rail render, inside `#[cfg_attr(test, mutants::skip)] fn render`): the Tab row (~3870) +
  Pane row (~3926) highlight. Per row:
  ```
  entry = entry.rounded(colors.corner_radius);
  entry = if row.active {
      entry.bg(rail_highlight(colors, true)).text_color(colors.foreground)
  } else {
      let hover = rail_highlight(colors, false);
      entry.text_color(colors.muted).hover(move |s| s.bg(hover))
  };
  ```
  **The `.hover` is ONLY on the non-active arm** — else hovering the active row would let gpui's hover-bg
  override the stronger active fill (a regression). Active rows are always highlighted; non-active rows
  reveal a subtler wash on hover. `.hover` needs no `.id()` (InteractiveElement, like the existing
  `.on_mouse_down`); the closure captures the `Copy` `Hsla` by `move`.
- Purely visual — the pure model (`rail_rows`/labels/active flag), the #167 agent glyph, the close ×, the
  #177 rename, and the active TEXT color (`foreground`) are all UNCHANGED.

### File manifest
- `crates/marley_app/src/app.rs` — (a) ADD `fn rail_highlight(colors: &ThemeColors, active: bool) -> gpui::Hsla`
  near `dock_panel` (~347) [its `#[cfg(test)]` test lands at validate]; (b) the Tab row (~3870-3874) +
  (c) the Pane row (~3926-3930): `.rounded(colors.corner_radius)` + active `bg(colors.surface)` →
  `bg(rail_highlight(colors, true))` + the non-active arm gains `.hover(|s| s.bg(rail_highlight(colors, false)))`.
- No `ui_components` change (`contrast_ratio` already `pub`; the guard is self-contained asserts). No new imports.

### Regression Test Plan
| REQ | test |
|---|---|
| REQ-004 (helper) | app.rs `#[cfg(test)]` `rail_highlight_is_a_distinct_accent_wash` — for `ThemeColors::default_for(Dark)` AND `default_for(Light)`: exact-value `rail_highlight(&c,true) == c.accent.opacity(0.22)` and `rail_highlight(&c,false) == c.accent.opacity(0.10)` (kills the `if active`→`true`/`false` mutants — each bool arm has a distinct value — and the body→Default mutant); guard `hl.a < 1.0`, `hl != c.surface`, `hl.h == c.accent.h`. One absolute Dark pin: `rail_highlight(&dark,true) == gpui::hsla(0.52,0.58,0.55,0.22)`. |
| REQ-001/002 | driven capture — the active tab shows a VISIBLE rounded accent-tinted highlight box distinct from the sidebar bg (was invisible). |
| REQ-003 | driven capture — hovering a non-active Tab/Pane row shows a subtle wash. |
| REQ-005 | review + capture — active text stays `foreground`; labels / close × / agent glyph unchanged. |
- **Uncoverable by unit test:** the shim render (`mutants::skip`) — validated by the driven captures (like
  #216/#217/#218). The pure `rail_highlight` carries the mutation load.

### Risks / decisions
- **R1 — hover must not downgrade the active fill.** Fixed: `.hover` is only on the non-active arm.
- **R2 — alpha compositing.** The wash composites over `surface` (gpui blends); the guard reasons on the RAW
  `Hsla` (a<1, hue=accent, ≠surface) — theme-agnostic + encodes the bug. Visible result validated by capture.
- **R3 — rounding a `w_full` row** rounds at the panel edge (small 6px radius — acceptable). Optional `mx_1`
  inset (Warp insets its box) DEFERRED; add only if the capture reads poorly at the edges.
- **R4 — `corner_radius` re-consume** partially addresses #224 (the rail now uses it); #224 stays open for the
  palette/dialog/find-bar cards — noted.
- **R5 — accent-vs-surface hue.** Both built-in themes have accent hue 0.52 ≠ surface hue 0.62, and the wash
  always shifts L over surface → distinct in both. A pathological theme with accent==surface is out of scope.

## Phase 3 — Implement
- **app.rs** — (a) added `fn rail_highlight(colors: &ThemeColors, active: bool) -> gpui::Hsla`
  (`colors.accent.opacity(if active { 0.22 } else { 0.10 })`) right after `dock_panel` (~377). (b) Tab row
  (~3877) + (c) Pane row (~3942): added `entry = entry.rounded(colors.corner_radius);` then the active arm
  → `entry.bg(rail_highlight(&colors, true)).text_color(colors.foreground)` and the non-active arm →
  `let hover = rail_highlight(&colors, false); entry.text_color(colors.muted).hover(move |s| s.bg(hover))`.
  A `#219 (Warp parity)` comment on each.
- **Verified before coding:** `.hover(move |d| d.bg(hover_bg))` is the established working idiom (app.rs:4389,
  4831) → `.hover(move |s| s.bg(hover))` is correct; `.hover` needs no `.id()` (the entry already has
  `.on_mouse_down`). `colors` in the render is OWNED (`self.theme.colors.clone()` at 3214) → pass `&colors`
  to the helper. `gpui::Hsla`/`gpui::hsla` are in scope (used at ~2227); `ThemeColors` imported (line 104).
- **Deviations from design:** none. (One helper with a bool, home = app.rs beside `dock_panel`, as designed.)
- `cargo fmt` + `cargo check -p marley` clean (only the pre-existing transitive `block v0.1.6` note).

## Inspect (Phase 3.5)
Two parallel critics over the diff (correctness; clean-room/consistency/simplification) + my own
independent verification (numeric composite, clippy, greps). One MED fixed; two LOWs resolved.

**Critic 1 — Correctness (returned: "No correctness defects; verified A, B"):**
- Helper value math correct both bools/both themes; **distinct from surface** (h 0.52≠0.62, a 0.22≠1.0 →
  the bug is fixed; composites as a ~22% accent wash, lighter+bluer than surface L0.155). MSI 100 — `Hsla`
  derives `Default` so exactly 3 mutants (body→Default, active→true, active→false) all killed by the
  two-bool exact-value asserts; the `0.22`/`0.10` f32 literals emit NO mutant (no float-literal operator).
  `.hover` only on the non-active arm (no active-fill downgrade); `.hover`/`.rounded(Pixels)` compile
  (InteractiveElement; `From<Pixels> for AbsoluteLength`); `&colors` borrow fine; no clipping; text/×/glyph/
  rename all unchanged. REAL — verified, no defect. (Noted harmless line-drift in my brief vs actual.)

**Critic 2 — Clean-room / Consistency / Simplification:**
- **[MED — judgment call] Project-row hover asymmetry → FIXED.** The Project row (app.rs ~3782) is also
  clickable (`switch_project`), but the diff left it inert while its Tab/Pane siblings gained a hover
  highlight → reads as unfinished. **Fix applied:** added `.rounded(colors.corner_radius)` +
  `.hover(|s| s.bg(rail_highlight(&colors, false)))` to the Project row's non-active arm — all clickable
  rail rows now give hover feedback. Kept the Project ACTIVE state as bright text (NO box): the selection
  box stays reserved for the leaf Tab/Pane rows, so the Project reads as the grouping/context level above
  them (a defensible hierarchy, and Warp's workspace-name section rows aren't boxed either). `cargo check`
  clean.
- **[LOW] 8-line Tab/Pane highlight duplication → REJECTED (extraction would break the mutation gate).** A
  `fn rail_row_fill(entry: Div, colors, active) -> Div` extraction exists, BUT it would be a FREE fn with an
  `if active` branch returning a gpui `Div` — cargo-mutants would emit the branch/Default mutants and there
  is NO way to assert a `Div`'s applied styles in a unit test (that's exactly why the render is
  `#[cfg_attr(test, mutants::skip)]`) → the mutants would SURVIVE → MSI RED. The established Marley pattern:
  only PURE logic extracts + gets mutation-tested (`rail_highlight` — the color); shim `Div` glue STAYS
  inline in the skip'd render. The ~8-line duplication across 2 (now 3, with the Project row) sites is the
  correct price of that boundary. Rejected with reason.
- **[LOW] `.opacity()` vs `.alpha()` → KEPT `.opacity()`.** Critic 2 itself found gpui's `opacity` docstring
  (color.rs:546-547) names it "suitable for an element's hover or selected state" — i.e. `.opacity()` is the
  DOCUMENTED idiom for exactly this. Both shipped accents have a=1.0 so `.opacity(0.22)==.alpha(0.22)` today.
  No change.
- VERIFIED clean: clean-room (tokens-only, no new hsla/hex — derives from `accent`); dead-code (clippy clean,
  `rail_highlight` used ×4/×6-after-fix, both `let hover` consumed, no shadow); `corner_radius` partial
  re-consume of #224 (the rail now uses it; palette/dialog cards stay open — #224 remains); alpha math
  reasonable on both themes.

**My independent verification (corroborating, evidence-backed):** numeric composite — active wash L 0.155→0.242
(ΔL +0.087 + cyan hue-shift), hover L→0.195 (ΔL +0.040); `cargo clippy -p marley --all-targets` clean
(only the pre-existing `block v0.1.6` note); `grep 'let hover'` = only my 2 bindings (now 3), disjoint
scopes, no shadow; diff has no new color literal.

**Verdict:** 1 MED fixed (Project-row hover), 2 LOWs resolved (extraction rejected w/ the mutation-gate
reason; `.opacity()` kept as gpui's idiom). No forge `failure-record` — no real bug (the MED was a
consistency polish, not a defect). Lenses covered: correctness, panic-safety, mutation (MSI-100 plan),
render-purity, clean-room, consistency, dead-code, simplification, legibility.

## Phase 4 — Validate
- **Unit test (REQ-004):** added `rail_highlight_is_a_distinct_accent_wash` in app.rs `#[cfg(test)] mod tests`
  — for BOTH `Appearance::Dark` + `Light`: exact-value (`rail_highlight(&c,true)==c.accent.opacity(0.22)`,
  `false→0.10`) + the distinct-from-surface guard (`a<1.0`, `!=surface`, `h==accent.h`) + an absolute Dark
  pin (`==hsla(0.52,0.58,0.55,0.22)`). `cargo nextest run -p marley` = **301 passed, 2 skipped** (300 + this);
  isolated run confirms `app::tests::rail_highlight_is_a_distinct_accent_wash` PASS.
- **Driven captures (live app, RE-BUNDLED after the Project-row fix; stale instance quit + relaunched):**
  - `scratchpad/219-atrest-after.png` (+ `-crop`): **REQ-001/002** — the active tab "terminal 4" now shows a
    VISIBLE accent-tinted ROUNDED highlight box (a teal wash, distinct from the surface sidebar) vs the
    pre-#219 `219-marley-sidebar.png` where it was only brighter text with NO box. **REQ-005** — the active
    text stays bright `foreground`, the close × + labels unchanged.
  - `scratchpad/219-hover3.png` (+ `-crop`): **REQ-003** — hovering the "ls" tab highlights ONLY that row
    with a subtle rounded wash (the 0.10 alpha — visibly lighter than the 0.22 active box); its neighbors
    stay unhighlighted → hover tracks the mouse + is single-row. (An earlier `219-hover.png` parked the
    pointer at the terminal-3/pane-1 boundary and showed both — a boundary-parking artifact, not a bleed;
    each row has an independent `.hover`, confirmed by the clean single-row `hover3`.)
- **Gate:** `git add -A` + `scripts/gates.sh --diff` — GREEN (below).
- **Pre-existing exclusions:** none. (The transitive `block v0.1.6` future-incompat note is upstream.)

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — #219 entry under [Unreleased]/Changed (above #218). app_shell.md — the
  seq-3 (#152) rail entry updated (M12.2 #219: the invisible surface-on-surface highlight → the
  `rail_highlight` accent wash + rounded box + hover on all clickable rows; the pure seam; corner_radius
  partial re-consume).
- **Knowledge (forge):** `aar-submit` 1d75b4a5 (completed, effectiveness 5). No `failure-record` (inspect
  found no real bug — the MED was a consistency polish). **Prevention rule recorded** →
  `PR-claude-selection-bg-distinct-from-container-001` (0fe7bdf9): a selection/active bg equal to its
  container bg is an invisible highlight — derive it from a DIFFERENT token + guard distinct-from-container
  with a pure test (exactly the #194→#152 regression this ticket fixed). `#224` ticket-comment: #219
  re-consumed corner_radius (partial); #224 narrows to the palette/dialog/find-bar cards.
- **Lessons:** (1) a token recalibration (#194) can silently invalidate an unrelated surface's highlight
  (#152) — a pure distinct-from-container guard now pins it (→ the prevention rule). (2) `.hover` must go
  ONLY on the non-active arm, else gpui's hover-bg overrides the stronger active fill (inspect confirmed).
  (3) shim Div glue can't extract to a free fn without breaking the mutation gate (Divs aren't
  unit-assertable) — only PURE color/logic extracts + gets mutation-tested; the ~8-line duplication is the
  correct price. (4) the drive harness has no pure hover verb — `scrollat:x,y,0` moves the pointer without a
  click/scroll → a usable hover capture (single-row when mid-row; parks at a boundary otherwise).
- **Close/archive:** TICKET-219 open→closed; forge ticket-close #219 done; pipeline pair → completed/.
