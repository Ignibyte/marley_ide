# The syntax theme system — a per-theme SyntaxPalette, contrast-proven by test — Notes

- **Forge ticket:** #316 7f9d185e-ca50-4026-b548-5546864113a1
- **AAR:** 71a05e24-d2f5-4372-a8c7-4c87cbcbc1bb
- **Local ticket doc:** docs/planning/tickets/open/TICKET-316-syntax-themes.md
- **Pipeline spec:** 316-syntax-themes.spec.md
- **pipeline_id:** 6d0401a3-0ec4-4ce7-bc47-4d63db0e052f

## Phase 1 — Plan

- **Request:** Promote the pre-authored spec (the Fable method). Give every theme its own `SyntaxPalette` over the #315 10-slot taxonomy, tuned per built-in + contrast-proven ≥AA by test; retire `token_color`'s hardcoded hsla literals. EIGHTH of the goal `/work 300,302,303,304,305,314,315,316,317,259` (#315 `28c063e` shipped the taxonomy this themes). Autonomous-through-commit.
- **Classification / tier:** work pipeline, ONE slice — a type (`SyntaxPalette` + `color_for`), two tuned palettes, a derived-default generator, the AA-matrix gate tests, and the `token_color` reroute. Not a larger feature (the recon dissolved the cache half; see below).
- **Forge recall (§18.3):** NO bulletins. knowledge-search surfaced my just-recorded #315 records (the sweep AD, the verify-adopted-grammar PR) + the exhaustive-match discipline — no #316-specific blockers. The #316 knowledge is IN the well-authored spec + [[marley-m20-language-intelligence]].

### Seam re-verification against `main` @ `28c063e` (#315 just landed + moved token_color)

| Seam | Spec claim | LIVE @ 28c063e | Verdict |
|---|---|---|---|
| `token_color` literals | "2 hardcoded hsla die (app.rs:801-806)" | **`token_color(kind, colors: &ThemeColors)` at app.rs:842 — SIX `gpui::hsla(...)` literals** (Str, Number, **Function, Type, Attribute, Property** — the last 4 from #315) + Punctuation→foreground + Keyword→accent + Comment/Hint→muted + Plain→foreground | **DRIFT — REQ-005 moves ~6, not 2.** Bigger but still bounded (one fn). |
| token_color callers | editor rows + hover fence + split pane (3) | **4 callers** — 5046 (editor rows, styled_slices HighlightStyle), 5425 (#246 code_view split pane, hand-lexer floor), 5516 + 12259 (HighlightStyle paths — hover fence #331 + editor) — ALL route through token_color | **REQ-006 HOLDS** (one seam; count is 4). |
| syntax cache | kinds not colors (`SyntaxLines`, app.rs:627) | **`SyntaxLines = Rc<Vec<Vec<(Range, TokenKind)>>>` (app.rs:652); `syntax_cache: Option<(u64, BufferVersion, SyntaxLines)>` (app.rs:492)** | **CONFIRMED — the dissolve holds; REQ-004 free (no invalidation).** |
| WCAG helpers | `contrast_ratio`/`relative_luminance` (ui_components:128/142, ≥7.0 tests :248) | **`relative_luminance(Hsla)->f32` (lib.rs:128) + `contrast_ratio(a,b:Hsla)->f32` (lib.rs:142); WCAG-known-value tests :248/:263** | **CONFIRMED — REQ-002 gate ready.** |
| theme registry | 2 built-ins (Marley Light/Dark, themes.rs:35) | **`ThemeRegistry::builtin()` → Marley Light + Marley Dark (themes.rs:35); test `builtin_has_a_light_and_a_dark_theme` :84** | **CONFIRMED (exactly 2).** |
| SyntaxPalette exists? | no (new) | grep-confirmed NONE | new type. |

### The load-bearing Phase-1 findings (for Design)

1. **CRATE SPLIT — and the refinement it enables.** `Theme` + `ThemeRegistry` live in **`marley_app/src/themes.rs`** (:18/:29), but `ThemeColors` + the WCAG helpers live in **`ui_components`**. `Theme` is `{ name, appearance, colors: ThemeColors }`, and each built-in's colors = `ThemeColors::default_for(appearance)`. **RECOMMENDED (design confirms): put `SyntaxPalette` AS A FIELD ON `ThemeColors`** (ui_components, populated by `default_for(appearance)`) rather than on `Theme`. Then `Theme` carries it transitively via `.colors`, and the **4 `token_color` call sites — which already pass `&ThemeColors` — reach `colors.syntax.color_for(kind)` with ZERO signature change**, so REQ-006 is free. This REFINES (does not violate) **D-PALETTE-ON-THEME** ("SyntaxPalette lives WITH ThemeColors" → literally a field on it). The alternative (palette on `Theme`) would force threading `Theme` through 4 render sites — avoid.
2. **THE HINT BOUNDARY.** The app `TokenKind` is 11 variants (9 syntax slots + `Hint` + `Plain`). The spec's `SyntaxPalette` covers the 9 syntax slots + Plain→foreground; `Hint` (#331, an inlay phantom → muted) is app-side, NOT a palette slot. **RECOMMENDED: `color_for` covers the 9 syntax slots and Plain→foreground; `token_color` KEEPS its `Hint→muted` arm OUTSIDE the palette** (so `token_color` stays exhaustive over 11, `SyntaxPalette` stays a clean 9-slot syntax concept). Design confirms the exact boundary.
3. **REQ-005 blast radius grew ~2→6** (all confirmed). The grep-gate must forbid `gpui::hsla(` inside `token_color` (or a broader token-color pattern) — the two tuned palettes + the derived default are the only place literals live. Design fixes the exact grep pattern.

### Prior art (§20 sweep — required)
1. **Behavior maps / observed** — every serious editor themes syntax alongside the chrome; illegible pale-on-light token colors are the observed failure class the AA gate exists to prevent (the spec's anti-goal: "one hardcoded look").
2. **Published material** — WCAG 2.x relative-luminance + contrast-ratio (the published formulae; the shipped `ui_components` fns already implement them, with known-value tests).
3. **OUR OWN CODE (highest-yield) — the sweep shrank the ticket twice, both CONFIRMED live:** (a) the WCAG helpers already ship with precedent tests (no reinvention — REQ-002 gates on them directly); (b) the cache-invalidation half DISSOLVED — `syntax_cache` stores `(Range, TokenKind)` KINDS, so a theme switch is free (REQ-004 needs no new wiring). `gpui::Hsla` is the value type throughout (no new color math beyond the shipped luminance). `ThemeColors::default_for(appearance)` is the per-appearance color source the palette rides. **No copyleft source read.**

§20: the ROLE taxonomy parallels TextMate/Zed SCOPE-theming as a published, ubiquitous CONCEPT; every value is Marley's own; WCAG is a published formula already in-repo. N/A for Warp (chrome, not code color). CONFIRMED.

**Phase 1 PASS.** The recon CONFIRMS the locked decisions hold, with two refinements for design (palette-on-ThemeColors; the Hint boundary) and one grown-but-bounded blast radius (REQ-005 ~6 literals). No scope reshape — one slice. Next: `/pipeline:design`.

## Phase 2 — Design

### 1. Architecture

**The crate-direction constraint (decides the shape):** `ui_components` is a LEAF (no dep on marley_syntax/marley_app — confirmed). So the palette CANNOT take `app::TokenKind` or `marley_syntax::TokenKind`; it needs its OWN vocabulary. The app bridges its `TokenKind` → the palette's slots at the ONE seam (`token_color`).

**(a) `SyntaxPalette` + `SyntaxSlot` (NEW, in `ui_components/src/lib.rs`, next to `ThemeColors`):**
- `pub enum SyntaxSlot { Keyword, Function, Type, Str, Number, Comment, Attribute, Punctuation, Property }` — the 9 theme-layer syntax ROLES (a THIRD mirror of the 9 syntax `TokenKind`s, justified by the crate boundary exactly as `marley_syntax::TokenKind`↔`app::TokenKind` are two mirrors bridged by `kind_from_syntax`; `SyntaxSlot` is the theming layer's vocabulary). `Copy, Eq`.
- `pub struct SyntaxPalette { keyword, function, type_, string, number, comment, attribute, punctuation, property: gpui::Hsla }` — `derive(Debug, Clone, Copy, PartialEq)` (Hsla is Copy).
- `pub fn color_for(&self, slot: SyntaxSlot) -> gpui::Hsla` — the TOTAL accessor (9-arm match → the field; no Option, no panic — D-TOTAL-COLOR-FOR). This is REQ-001's pure home, mutation+coverage-tested IN ui_components.
- `pub fn for_appearance(appearance: Appearance) -> SyntaxPalette` — the TWO tuned palettes (Light/Dark), each a deliberate per-slot Hsla pass.
- `pub fn derived_from(colors: &ThemeColors) -> SyntaxPalette` — the generator (insurance): an accent-anchored deterministic hue spread from `colors.accent`/`.foreground`, distinct slots, passing the SAME AA gate. v1 has no palette-less theme, so it is TEST-exercised not usage-exercised (documented insurance, sized as a fn + its spread table).

**(b) `SyntaxPalette` is a FIELD ON `ThemeColors`** (the Phase-1 refinement of D-PALETTE-ON-THEME): add `pub syntax: SyntaxPalette` to `ThemeColors`; the 2 `default_for` arms set `syntax: SyntaxPalette::for_appearance(appearance)`. `Theme` (marley_app) carries it transitively via `.colors` → the 4 `token_color` call sites (which pass `&ThemeColors`) reach `colors.syntax` with ZERO change → **REQ-006 free**. Only 2 `ThemeColors { }` literals exist (both in `default_for`), so the new field is compiler-forced at exactly 2 sites.

**(c) The tuned palettes (the load-bearing DATA — each slot ≥4.5:1 vs that theme's `background`):**
- **Dark** (bg `hsla(0.62,0.16,0.05)`, near-black L≈0.05 → slots must be LIGHT): start from #315's shipped hues (they were tuned for the dark default) — string `hsla(0.33,0.44,0.60)`, number `hsla(0.09,0.55,0.62)`, function `hsla(0.60,0.48,0.65)`, type `hsla(0.12,0.50,0.65)`, attribute `hsla(0.78,0.35,0.66)`, property `hsla(0.50,0.40,0.66)`, keyword ≈ the dark accent `hsla(0.52,0.58,0.55)`, comment ≈ the dark muted `hsla(0.62,0.08,0.60)`, punctuation ≈ the dark foreground `hsla(0.62,0.05,0.90)`. Each is VERIFIED ≥4.5 by the AA test (a near-black bg gives these mid-high-L colors ~5–8:1).
- **Light** (bg `hsla(0.10,0.10,0.97)`, near-white L≈0.97 → slots must be DARK): the #315 mid-L hues FAIL here (mid-L vs near-white ≈ 2–3:1), so Light needs its OWN DARK analogues (L≈0.30–0.45, saturated) — e.g. string a dark green, number a dark rust, function a dark blue, type a dark gold/brown, keyword ≈ the light accent `hsla(0.52,0.58,0.34)` (deep teal, ~6.5:1), comment ≈ the light muted (AT the 4.5 floor is OK for comments), punctuation ≈ the light foreground `hsla(0.62,0.14,0.16)`. **Implement TUNES these against the AA test** (iterate until every slot ≥4.5); the test is the arbiter, not the eye.

**(d) The `token_color` reroute (marley_app/src/app.rs — the 6 hsla literals DIE):** the 11-arm match becomes: the 9 syntax kinds → `colors.syntax.color_for(SyntaxSlot::X)`; `Hint => colors.muted` (the #331 inlay phantom stays app-side, NOT a palette slot); `Plain => colors.foreground` (the sibling foreground). Stays exhaustive over 11; NO hsla literals remain. The 4 callers unchanged. `token_color` is app.rs (coverage-excluded, mutation-included) — its whole-fn mutant is killed by the existing `token_color_maps_each_kind` (rerouted to assert palette-field equality); the per-arm reads carry no operator/literal mutation surface.

**(e) REQ-005 grep-gate (no color literal outside the palette):** a `#[cfg(test)]` unit that slurps `token_color`'s body (via `include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/src/app.rs"))`, slice between `fn token_color(` and the next `\nfn `) and asserts it contains NO `hsla(`. Keeps the guard in the nextest suite (no separate gate line); implement may instead add a `gate:13` SAST grep if cleaner.

**§14:** `color_for` is total (no Option/panic); `for_appearance`/`derived_from` are pure; no IO. §20: WCAG is a published formula already in-repo (`contrast_ratio`/`relative_luminance`); the role taxonomy parallels TextMate/Zed SCOPE-theming as a CONCEPT only; every Hsla is Marley's own; no copyleft source read. CONFIRMED.

### 2. File manifest
- **crates/ui_components/src/lib.rs** — `enum SyntaxSlot` (9); `struct SyntaxPalette` (9 Hsla fields) + `color_for` + `for_appearance` (2 tuned) + `derived_from` (generator); `pub syntax: SyntaxPalette` field on `ThemeColors`; wire `syntax: SyntaxPalette::for_appearance(appearance)` into both `default_for` arms; the tests (color_for total, AA matrix ×2 themes, derived-default AA ×2 appearances, REQ-007 light≠dark value table, min-hue-distance).
- **crates/marley_app/src/app.rs** — reroute `token_color` (9 syntax kinds → `colors.syntax.color_for(SyntaxSlot::X)`, Hint→muted, Plain→foreground); update `token_color_maps_each_kind` to assert palette-field equality; add the REQ-005 source-slurp grep unit.
- **(no new file)** — SyntaxPalette lives in ui_components/lib.rs beside ThemeColors (the AA test needs the WCAG helpers, same module).

### 3. Regression Test Plan (all in ui_components unless noted)
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `color_for` returns each slot's field for every `SyntaxSlot` (9 arms) + is total (no panic) | pure unit (ui_components) |
| REQ-002 | the AA matrix: `for_appearance(Light)` and `(Dark)` — every one of the 9 slots `contrast_ratio(slot, background) ≥ 4.5` (Comment may == 4.5) | pure unit (ui_components) — THE gate |
| REQ-003 | `derived_from(default_for(Light))` and `(Dark)` — 9 distinct slots + each ≥4.5 vs that bg (the SAME matrix) | pure unit |
| REQ-004 | `token_color(kind, &light.colors) != token_color(kind, &dark.colors)` for a slot that differs (Str) — proves per-theme resolution; + a note the cache is kind-keyed so a live switch recolors same-frame with no invalidation (structural, D-KINDS-NOT-COLORS) | pure unit (app) |
| REQ-005 | `token_color`'s body contains no `hsla(` literal (source-slurp) | unit/grep-gate (app) |
| REQ-006 | `token_color(kind, colors)` == `colors.syntax.color_for(SyntaxSlot::X)` for the 9 syntax kinds, `colors.muted` for Hint, `colors.foreground` for Plain (the rerouted `token_color_maps_each_kind`) | pure unit (app) |
| REQ-007 | Light≠Dark where it matters: a value table asserting `for_appearance(Light).string != for_appearance(Dark).string` (and number) — the anti-goal was one look | pure unit (ui_components) |
| edge | min-hue-distance / distinctness: no two of the 9 slots collide (a ratio test can't see two AA-passing slots that are visually identical) — assert pairwise the 9 slots are not equal, and a coarse hue/L separation for the 6 chromatic slots | pure unit (ui_components) |

LIVE drive: OFF-LIMITS (chad at machine) — REQ-004's pixel is deferred; the per-theme color + AA matrix are PURE (the state is fully proven headless). The #199 live-picker path needs NO change (colors resolve per frame).

### 4. Risks / decisions
- **The tuned Light palette values** (the real work): 9 dark, saturated, DISTINCT, AA-passing colors vs near-white — implement iterates against the AA test. The Dark palette rides #315's hues (verify ≥4.5). Recorded: the AA test is the arbiter, not the eye.
- **The generator's spread math** must be deterministic + AA-passing for BOTH appearances from just `ThemeColors` — an accent-anchored hue rotation with an L-clamp toward legibility; sized as insurance (test-exercised).
- **Min-hue-distance** (an inspect edge): two slots ≥AA but visually identical is a legibility bug the ratio can't catch → a distinctness test.
- **The THIRD mirror enum** (`SyntaxSlot`): justified by the leaf-crate boundary (same as the existing 2 TokenKind mirrors); the app→slot map is the 9-arm reroute in `token_color`.
- **`derived_from` dead-in-v1**: no palette-less theme exists, so it is test-only until a 3rd theme arrives — documented insurance (the spec sized it so), held to cov/MSI 100 by its own test.

**Design PASS.** Ready for Phase 3 — Implement.

## Phase 3 — Implement

**Built (per the manifest):**
- **`crates/ui_components/src/lib.rs`** — `pub enum SyntaxSlot` (9 roles) + `SyntaxSlot::ALL` (the 9 in field order, for the contrast matrix + iteration); `pub struct SyntaxPalette { keyword, function, type_, string, number, comment, attribute, punctuation, property: Hsla }` (`derive(Debug, Clone, Copy, PartialEq)`); `impl SyntaxPalette { color_for(&self, SyntaxSlot) -> Hsla` [total, exhaustive match, no `_`]`; for_appearance(Appearance)` [the 2 TUNED palettes]`; derived_from(&ThemeColors)` [the accent-anchored generator]`}`; `pub syntax: SyntaxPalette` field on `ThemeColors` (after `muted`); wired `syntax: SyntaxPalette::for_appearance(appearance)` into BOTH `default_for` arms (the only 2 `ThemeColors` literals); the `default_palettes_clear_aa` AA-matrix test (the tuning arbiter — Phase 4 extends).
- **`crates/marley_app/src/app.rs`** — `token_color` rerouted: the 9 syntax kinds → `colors.syntax.color_for(SyntaxSlot::X)`; `Hint => colors.muted`; `Plain => colors.foreground`. The SIX `gpui::hsla(...)` literals DIED. Exhaustive over 11, 4 callers unchanged. `token_color_maps_each_kind` updated to assert palette-field equality (REQ-006 — the exact-hsla asserts became `c.syntax.color_for(slot)` asserts).

**Deviations from design (with reason):**
1. **The Light palette passed AA on the FIRST values** — no tuning iteration needed. I picked conservatively dark, saturated hues (L≈0.28–0.44) and `default_palettes_clear_aa` went green immediately for all 9 slots × both tuned palettes AND the derived-default, both appearances. (The design flagged this as "the load-bearing tuning"; it turned out cheap because the near-white bg's threshold [relative-luminance ≤ ~0.17] is easy to clear with dark saturated colors.)
2. **Added `SyntaxSlot::ALL`** (a `pub const [SyntaxSlot; 9]`) — not explicitly in the manifest, but it's the clean way to iterate slots for the AA matrix (+ Phase 4's per-slot tests) rather than repeating the 9 literals; genuine API (enumerate the palette's slots).
3. **The AA-matrix test (`default_palettes_clear_aa`) landed in implement** (not deferred to Phase 4) — the design said "write it FIRST as the tuning arbiter", so it exists now; Phase 4 EXTENDS the suite (color_for total, REQ-007 light≠dark, min-hue-distance, the REQ-005 grep unit, the derived-default distinctness) but the core AA matrix is already green.
4. **`derived_from` anchors slot LIGHTNESS to `foreground.l`** (not a free hue-spread at arbitrary L) — because the foreground is AA vs background BY CONSTRUCTION (it's the text color), so slots at that L inherit the contrast, guaranteeing the generator passes AA for any theme without per-theme tuning. Hues spread evenly from `accent.h` (`(accent.h + i/9).fract()`) for distinctness.

**Compile/test state:** `cargo check -p marley_ui_components` + `-p marley --lib` clean; `cargo fmt` clean; **marley_ui_components 33/33** (incl `default_palettes_clear_aa` — all 9 slots ≥4.5 both themes + derived); **app `token_color_maps_each_kind` PASS** (the reroute returns `color_for` values). No new files (SyntaxPalette rides the existing ui_components/lib.rs; the reroute is in app.rs) → no git-add-N. ui_components is a leaf (no marley_syntax/marley_app import — SyntaxSlot is its own). token_color stays exhaustive over 11; its whole-fn mutant is killed by the rerouted test.

Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.

## Phase 3.5 — Inspect

**3 parallel general-purpose critics**, distinct lenses: (1) WCAG/AA-matrix + palette VALUES + slot distinctness, (2) the reroute + crate boundary + exhaustiveness, (3) simplification/hygiene + deviations. Each COMPUTED the actual `contrast_ratio` for all 18 (slot×theme) pairs. The reroute + crate boundary + exhaustiveness came back CLEAN; the defects were all in the tuned palette DATA — exactly the class the design flagged (an AA test is blind to slot distinctness). **The min-hue-distance edge the design predicted was real.**

### Findings + verdicts

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| C-1 | **HIGH** | Light `keyword` hsla(0.52,0.58,0.34) ≡ `property` hsla(0.52,0.50,0.34) — IDENTICAL hue+lightness → indistinguishable dark teal; both clear AA (4.51/4.81) so `default_palettes_clear_aa` is BLIND. Two of the most frequent kinds render the same. | **REAL** (all 3 critics; my suspicion exact) | Light `property` → a distinct violet `hsla(0.70,0.50,0.40)` (8.9:1). |
| C-2 | MEDIUM | Dark `type` hsla(0.12,0.50,0.65) ≈ `number` hsla(0.09,0.55,0.62) — near-identical amber (dH=0.03,dL=0.03; saturation doesn't rescue). Critic 1 caught what the hint missed. | **REAL** | Dark `number` → red-amber `hsla(0.04,0.60,0.62)` (6.7:1; dH from type 0.03→0.08). |
| C-3 | MEDIUM | Light `keyword` AA margin razor-thin (4.511; L 0.34→0.35 goes sub-AA) — brittle; and the AA-test comment misnamed the at-floor slot as Comment (Comment is 5.77). | **REAL** | keyword L 0.34→0.32 (→4.98, real headroom); corrected the test comment (names keyword as the thinnest, ~4.98 after the nudge). |
| C-4 | HIGH (hskp) | Throwaway `zzz_throwaway_316*.rs` diagnostic files left untracked (a critic race) — would pollute the nextest suite / a `git add -A`. | **REAL** | `rm` + verified `git status` clean of them (the critics' own cleanup had already removed them; re-confirmed). |
| C-5 | MED (P4) | `derived_from`'s hue-spread `(accent.h + i/9).fract()` has 2 mutants (`+`→`-`, `+`→`*`) that produce DISTINCT AA-passing hues → survive both the AA matrix AND a distinctness test. `cargo mutants --list` (critic 3, real). | **REAL — Phase 4** | Phase 4 pins the spread: assert `derived.keyword.h == accent.h` (kills `+`→`*`) + `derived.function.h == (accent.h + 1.0/9.0).fract()` (kills `+`→`-`). Noted in the Phase-4 plan below. |
| C-6 | LOW | `derived_from` doc overclaims "slots inherit that contrast" — WCAG contrast is luminance-driven, luminance ≠ HSL lightness; the L-anchor clears AA only for the built-ins' EXTREME-L foregrounds, not an arbitrary marginal theme. | **REAL** (doc) | Softened the doc: holds for extreme-L foregrounds; a moderate-L third-party theme would want per-slot L-clamping (a follow-up); distinctness is hue-only (colorblind-safe deferred). |
| C-7 | LOW | `token_color`'s doc forward-references `token_color_holds_no_hardcoded_literals`, a Phase-4 test that doesn't exist yet. | **REAL** (doc) | Softened to "(the REQ-005 grep-gate lands in Phase 4)". |
| C-8 | LOW→MED | The THIRD `SyntaxSlot` enum + `color_for` vs the lighter 9-pub-fields shape (token_color reads fields directly, no enum). | **REJECTED** (kept) | Both critics judged it DEFENSIBLE: the spec's D-TOTAL-COLOR-FOR wants a `color_for` accessor; `color_for` is production-reachable (the 4 render sites), mutation-tested (the AA test kills its body mutant), and `SyntaxSlot::ALL` drives the matrix. The redundant second match is minor; the leaf-crate boundary justifies an app-independent vocabulary. Kept — a judgment call, documented. |

**Rejected (with reason, no fix):** `type_` field name (idiomatic trailing-underscore, serde/std convention); Copy/size (ThemeColors is Clone-not-Copy, SyntaxPalette Copy 144B, all call sites take `&ThemeColors` → no perf hit); Function/Comment close hue+L in both palettes (saturation rescues — comment is near-gray S≈0.09 vs function saturated S≈0.5); `cargo doc`/`clippy` both crates CLEAN; REQ-006 the 4 callers unchanged (5047/5426/5517/12260 — editor rows / #246 split / code_view-with-syntax / #331 hover fence); the reroute mapping has ZERO transposition (Keyword→Keyword … Property→Property); exhaustive over 11, no `_`.

**Forge captured:** `BF-claude-aa-passing-palette-shipped-two-identical-slots-001` (the AA-blind collision) + `PR-claude-contrast-test-is-blind-to-slot-distinctness-001` (a contrast test proves legibility-vs-bg but is blind to slot-vs-slot distinctness — pair it with a pairwise distinctness gate + watch the AA margins).

**Phase 4 MUST (from inspect):** (a) keep `tuned_palettes_have_distinct_slots` (added at inspect — the collision gate); (b) pin `derived_from`'s spread arithmetic to kill the `+`→`-`/`+`→`*` mutants (C-5); (c) the REQ-005 grep unit named `token_color_holds_no_hardcoded_literals`; (d) `color_for` per-slot + REQ-007 light≠dark. Mutation note (critic 3, verified): `SyntaxPalette` has NO `Default` → the `for_appearance`/`derived_from` body-replace mutants are UNVIABLE (the #203/#205 lesson holds); `color_for`'s body mutant IS viable, killed by the AA test's dark-bg assert.

**Verify (inspect fixes):** `cargo nextest -p marley_ui_components --lib` — 4/4 pass incl `default_palettes_clear_aa` (AA matrix with the fixed values) + the NEW `tuned_palettes_have_distinct_slots` (no collisions). App `token_color` logic unchanged since its last passing run (only a doc comment softened).

**⚠️ BUILD-HANG HAZARD hit TWICE (cost ~45 min) — ROOT CAUSE + FIX found:** gpui-heavy workspace builds STALLED — rustc/clippy-driver `STAT S` at 0.0% CPU for 12-20 min (blocked, not compiling; memory 74% free, load 3.5 — NOT resource starvation), the "Checking …" log frozen, the SAME PIDs stuck. First on a ui_components test build, then IN the gate's `clippy --workspace` step. **ROOT CAUSE: a PARALLELISM deadlock** — after a long autonomous session (many concurrent background cargo runs + 3 critic agents each running cargo + several rustc killed mid-build), the default full-parallelism workspace build (N = all cores) deadlocks on the gpui-heavy crates. **THE FIX (proven): `CARGO_BUILD_JOBS=4`** — `clippy --workspace --all-targets` that had frozen for 12 min completed in **3m27s** with the job cap; the gate was then re-run as `CARGO_BUILD_JOBS=4 scripts/gates.sh --diff`. Recovery recipe: (1) a genuine stall = 0% CPU for MINUTES + frozen log + same PIDs (NOT slow progress — slow progress cycles PIDs / advances the log); (2) `pkill -9` ALL cargo/rustc/clippy-driver/nextest/mutants (kill the whole tree, leave NO orphaned 0%-CPU rustc — orphans block the next build); (3) verify the toolchain with a tiny crate (`cargo check -p marley_syntax`, ~0.2-8s); (4) re-run with **`CARGO_BUILD_JOBS=4`** (+ `CARGO_INCREMENTAL=0`). **The earlier #315 full gates ran fine at full parallelism — the deadlock is a LATE-SESSION accumulation effect, so cap jobs on any heavy build once a session has spawned many concurrent cargo runs.**

Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.

## Phase 4 — Validate

**Tests (all RUN + green):**
- **ui_components (`lib.rs`):** `color_for_returns_each_slots_field` (REQ-001 — each slot→its field, kills color_for's 9 delete-arm mutants); `default_palettes_clear_aa` (REQ-002 — the AA matrix, tuned+derived, both themes; impl+inspect); `derived_from_is_distinct_and_pins_the_spread` (REQ-003 + inspect C-5 — 9 distinct hues + PINS the spread: `keyword.h==accent.h` kills `+`→`*`, `function.h==(accent.h+1/9).fract()` kills `+`→`-`, distinctness kills `/`→`%`/`*`); `light_and_dark_palettes_differ` (REQ-007); `tuned_palettes_have_distinct_slots` (inspect distinctness gate).
- **app (`app.rs`):** `token_color_maps_each_kind` (REQ-006 — the reroute, every arm through the palette); `token_color_holds_no_hardcoded_literals` (REQ-005 — source-slurps token_color's body, no `hsla(` literal).
- **REQ-004** (live recolor on switch): the per-theme resolution is proven pure — `token_color(kind, &light.colors) != token_color(kind, &dark.colors)` follows from `light.syntax != dark.syntax` (REQ-007) + the reroute; the "same frame, no invalidation" is STRUCTURAL (D-KINDS-NOT-COLORS — the cache stores `TokenKind`, verified at Plan). LIVE pixel deferred (chad at machine).

**Test-run counts:** ui_components `--lib` 7/7 relevant pass (5 palette + 2 existing default_for); app `token_color_*` 2/2.

**Gate — TWO runs (a coverage red fixed at source, §0):**
- Run 1: `GATE RED` — only gate:4 coverage. **Mutation was already MSI 100% (6 caught / 0 missed)** — the in-diff set (color_for arms, derived_from spread, the AA/distinctness constants) all killed; the `for_appearance`/`derived_from` body-replace mutants UNVIABLE (no `Default` on SyntaxPalette — the #203/#205 lesson, as critic 3 predicted). The coverage miss = lib.rs:350/351, the `tuned_palettes_have_distinct_slots` assert's lazily-evaluated MESSAGE args (`slots[i].0`/`slots[j].0` on their own lines — only run on failure; the #315:811 trap AGAIN). FIX: bind `let (si, sj) = …` before the assert (runs every iteration → covered) + inline `{si:?}`/`{sj:?}` in the message. Re-verified lib.rs 100% (0 missed).
- Run 2 (`CARGO_BUILD_JOBS=4 scripts/gates.sh --diff`): **`GATE GREEN [diff]` — 15/15, 0 failed.** coverage 100% + mutation MSI 100% (6/6) + miri + visual/AX + deny + docs all green; receipt `ba69871c…` written + fingerprint-VALID (only docs edited since).

Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.

**Note:** the run used `CARGO_BUILD_JOBS=4` throughout — see the BUILD-HANG HAZARD in Phase 3.5 (the default full-parallelism gate DEADLOCKED late-session; the job cap resolved it).

## Phase 5 — Complete

**Docs (§21):** CHANGELOG.md #316 (per-theme SyntaxPalette + the 4 load-bearing parts incl the AA-blind distinctness catch); `crate-map.md` ui_components row (SyntaxPalette + SyntaxSlot + color_for, contrast-proven + distinct, leaf-crate); `editor.md` syntax section (token_color resolves through `ThemeColors.syntax`; kinds-not-colors → free switch). All drafted during the gate wait, verified present.

**Knowledge captured (forge):**
- AAR `71a05e24` submitted — outcome completed, effectiveness 4.
- Failure (inspect): `BF-claude-aa-passing-palette-shipped-two-identical-slots-001` (the AA-blind Light keyword≡property collision).
- Prevention rules: `PR-claude-contrast-test-is-blind-to-slot-distinctness-001` (a contrast test proves legibility-vs-bg but is blind to slot-vs-slot distinctness — add a pairwise gate + watch margins), `PR-claude-cap-cargo-jobs-late-session-to-avoid-build-deadlock-001` (the late-session full-parallelism build deadlock → `CARGO_BUILD_JOBS=4`).

**Durable lessons:**
1. **The recon DISSOLVED a locked worry** — the ticket feared a stale-color cache on theme switch; the syntax cache stores KINDS not colors, so the live switch is free with no design. Reading the code before designing beat the description.
2. **The Phase-1 refinement earned its keep** — SyntaxPalette on `ThemeColors` (which `Theme.colors` IS), not on `Theme`, made the 4 render sites zero-change (REQ-006 free). The spec's D-PALETTE-ON-THEME allowed both; the field-on-ThemeColors reading was strictly simpler.
3. **An AA/contrast test is BLIND to slot distinctness** — the design PREDICTED this edge (min-hue-distance) and the inspect confirmed it exactly (keyword≡property, both AA-passing yet identical). Two gates: AA-vs-background AND pairwise distinctness. And watch AA MARGINS (a slot at ~4.5 is brittle).
4. **A leaf crate needs its own vocabulary** — ui_components (gpui only) can't see the app/syntax `TokenKind`, so `SyntaxSlot` is a THIRD mirror; the app bridges `TokenKind → SyntaxSlot` at the one `token_color` seam. Justified by the boundary (both critics agreed).
5. **The #315:811 uncovered-line trap RECURRED** — a lazily-evaluated `assert!` MESSAGE arg on its own line is only run on failure → uncovered. Bind the value before the assert (runs every iteration) + inline-capture in the message.
6. **A late-session build-hang is real** — full-parallelism workspace builds deadlock after many concurrent cargo runs; `CARGO_BUILD_JOBS=4` is the fix (cost ~45 min to diagnose here; the PR above captures it).

Status: Phase 5 — Complete PASS.
