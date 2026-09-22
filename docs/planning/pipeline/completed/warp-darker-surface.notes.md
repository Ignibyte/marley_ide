# Darker, Warp-like gray for panel/dock surfaces — Notes

- **Forge ticket:** #231 (e8e48112-9bf6-420a-a832-920018691593)
- **AAR:** ae52900d-1266-4d1e-96a3-ae2b73faf7f6
- **Local ticket doc:** docs/planning/tickets/open/TICKET-231-warp-darker-surface.md
- **Pipeline spec:** warp-darker-surface.spec.md

## Phase 1 — Plan
- **Request:** darken the panel/dock surface to a Warp-like darker gray (chad live feedback #3). 4th ticket of
  the /work 228-237 M13 train.
- **Classification / tier:** work pipeline · chore · M13 · ONE tiny pure-seam slice (a theme value + its test).
- **Forge recall (§18.3):** #194 (the terminal-black + gray-panel recalibration that set the current surface),
  #35 (the Warp-matched palette), the M1.E palette-lightness rule (`palette-accent-lightness-must-clear-wcag`)
  — the contrast helper guards legibility. Clean-room §20.
- **Discovery (this turn):**
  - `crates/ui_components/src/lib.rs`: `ThemeColors` (:42, PURE, cov/MSI 100 exact-value pins :160-182). DARK
    theme (:96-109): `background hsla(0.62,0.16,0.05)` (near-black), **`surface hsla(0.62,0.09,0.155)`** (docks/
    sidebar/Files/panels/cards/inputs/overlays — the #231 target), `border hsla(0.62,0.10,0.26)`, `muted
    hsla(0.62,0.08,0.60)`, `foreground hsla(0.62,0.05,0.90)`. LIGHT surface `hsla(0.10,0.10,0.93)`.
  - `contrast_ratio(a,b)` (:142) + `relative_luminance` (:123) pure WCAG helpers + tests (:243+) → the legibility
    guard.
- **Decisions:** D1 darken the dark surface L 0.155→~0.10-0.11 (design picks); D2 leave the light surface; D3
  surface is shared → global consistent darken; D4 keep distinct from bg(0.05)+border(0.26); D5 contrast only
  improves (assert it).
- **Env note:** chad is ACTIVELY on a SHARED desktop → a driven capture would hijack his screen; the surface
  VALUE is unit-proven (exact-value pin) + observe-Warp-referenced; offer the visual vibe-check when free. A
  clean-room visual calibration — his eye is the final check.

## Phase 2 — Design

### Architecture / approach
A single pure theme-value change in `crates/ui_components/src/lib.rs` — the `default_for(Appearance::Dark)`
`surface` field. The render already reads `colors.surface` everywhere → one value change re-tones every panel/
dock/sidebar/Files/card/input/overlay consistently. No shim/app.rs change.

**THE VALUE:** dark `surface` `hsla(0.62, 0.09, 0.155)` → **`hsla(0.62, 0.09, 0.11)`** — a pure darken (hue 0.62
+ sat 0.09 kept, only L 0.155→0.11). Justification: Δ0.045 darker (a clear step, "more Warp like"); still Δ0.06
above the `background` L=0.05 → a visible raised-surface delta (Warp's panels sit subtly above the black
terminal — matched from the deskcheck captures, clean-room §20, our own value); Δ0.15 below the `border` L=0.26 →
the border stays clearly visible. LIGHT theme `surface` UNCHANGED (D2 — "darker" is a dark-mode ask).

**Legibility (only improves):** darkening `surface` (the darker member of each text/surface pair) RAISES the
contrast ratio. The existing canary `contrast_ratio(d.muted [L=0.60], d.surface) >= 4.5` (:290, was ≈5.08) goes
UP. Foreground (L=0.90) on the darker surface is AAA. The `background < surface` luminance assert (:285) still
holds (0.05 < 0.11).

### File manifest
| File | Kind | Change |
|---|---|---|
| `crates/ui_components/src/lib.rs` | PURE | dark `surface` L 0.155→0.11; update the `dark_194_shades_pinned_and_legible` surface exact-pin (:282); ADD a `contrast_ratio(d.foreground, d.surface) >= 7.0` assert (guard the darken's panel-text legibility) |

No app.rs/shim change. No new role.

### Regression Test Plan
| REQ | Test | Note |
|---|---|---|
| REQ-001 | update `dark_194_shades_pinned_and_legible` :282 → `assert_eq!(d.surface, gpui::hsla(0.62, 0.09, 0.11, 1.))` (the exact-value golden — hsla isn't mutated, so this is the typo guard + regression) | the value change |
| REQ-002 | :285 `relative_luminance(d.background) < relative_luminance(d.surface)` (0.05<0.11, holds) + `theme_colors_carries_all_roles_and_metrics` :182 `surface != border` | distinct-from-bg/border |
| REQ-003 | :290 `contrast_ratio(d.muted, d.surface) >= 4.5` (improves) + NEW `assert!(contrast_ratio(d.foreground, d.surface) >= 7.0)` (panel body text AAA on the darker surface) | legibility guard |
| REQ-004 | ADD `assert!(d.border.l > d.surface.l)` (0.26 > 0.11 — the border stays visible over the darker surface) | border visibility |

**Mutation:** `default_for` yields only `→Default::default()` (UNVIABLE — no Default on ThemeColors) → the
surface value has ZERO viable mutants; the change is coverage (the `surface:` line runs in every `default_for`
call) + the exact-pin golden. The `relative_luminance`/`contrast_ratio` mutants (op swaps, :129-145) are UNCHANGED
+ still killed by the WCAG tests. cov/MSI 100 maintained. **Uncoverable path:** the rendered panel color — the
VALUE is exact-pinned (unit); "looks Warp-like" is a clean-room visual judgment → chad's vibe-check (env-blocked
driven capture; offer when his screen's free).

### Risks / decisions
- **R1 — the exact L (0.11) is a picked value** — clean-room from Warp's panel gray; chad confirms the vibe
  (easy to nudge 0.10/0.12 if he wants darker/lighter). The exact-pin test makes any future change deliberate.
- **R2 — surface is shared** (docks + overlays) → the darken is global; that's the intent (D3). If chad later
  wants docks-vs-overlays to differ, that's a new role (out of #3's scope).
- D1 (0.11), D2 (light unchanged), D3 (global), D4 (deltas ok), D5 (contrast improves) per the spec.

## Phase 3 — Implement
**Built (manifest as designed) — `crates/ui_components/src/lib.rs` (marley_ui_components), PURE:**
- Dark theme `surface` `hsla(0.62, 0.09, 0.155)` → `hsla(0.62, 0.09, 0.11)` (:105) — the pure darken (hue+sat
  kept, L 0.155→0.11); updated the #194 comment (:97-100) to name the new value + the ΔL~0.06 raised delta.
- `dark_194_shades_pinned_and_legible` test: updated the surface exact-pin (:282 → 0.11); ADDED
  `assert!(contrast_ratio(d.foreground, d.surface) >= 7.0)` (panel body text AAA on the darker surface) +
  `assert!(d.border.l > d.surface.l)` (border visible over it); refreshed the muted-on-surface `≈5.08` annotation
  (the darker surface raises it). The other asserts (border pin, background<surface, fg-on-bg, accent) unchanged.

**Deviations from design:** none.

**Checks:** `cargo fmt` clean; `cargo check -p marley_ui_components --all-targets` ✓; `cargo clippy -p
marley_ui_components --all-targets -- -D warnings` exit 0; the 6 theme/contrast tests PASS — including the new
foreground-on-surface ≥7 + border.l>surface.l asserts (empirically confirming the darker surface stays legible +
the border stays visible).

## Phase 3.5 — Inspect
1 focused general-purpose critic (proportionate to a one-value theme change) + self-review. **CLEAN — no
findings.** Confirmed concretely (read the code + RAN the test + the mutants list):
- **Value:** dark `surface` = `hsla(0.62, 0.09, 0.11)` — pure darken (only L 0.155→0.11; hue/sat kept). Distinct
  from `background` (0.05 → Δ0.06 raised delta; `relative_luminance(bg)=0.0036 < surface=0.0109` holds) + below
  `border` (0.26 → the new `border.l > surface.l` holds, gap widened 0.105→0.15). Light surface + all other dark
  roles UNCHANGED.
- **Legibility:** `contrast_ratio(fg, surface) ≈ 13.68 ≥ 7` (panel body AAA); `contrast_ratio(muted, surface) ≈
  5.78 ≥ 4.5` — ROSE from ≈5.08 (surface is the darker/lo member; lowering L_lo raises the ratio). The stale
  "≈5.08" annotation was correctly dropped (applying the #230 stale-doc-number lesson). Test RUN → passes.
- **Mutation:** `default_for` yields ONLY `→Default::default()` (UNVIABLE — ThemeColors derives no Default) → the
  surface value has ZERO viable mutants; the exact-pin is coverage+regression. The `relative_luminance`/
  `contrast_ratio` op-swap mutants are unchanged + still killed by the WCAG tests. cov/MSI 100 maintained, no new
  obligations.
- **Clean-room §20:** 0.11 is our own value (a cool-tinted dark gray above the near-black bg, derived by lowering
  #194's L); the only remaining `0.155` is the intentional "was 0.155" history note; consumers bind the `surface`
  role (not a literal) so the darken propagates. No Warp source.
- **Non-blocking observation (no fix):** the terminal↔panel luminance gap narrowed (~43% smaller) — this IS chad's
  "a bit darker, more Warp-like" intent; the `background < surface` ordering assert guards they stay distinct.

No `failure-record`/`prevention-rule` — clean. Lenses covered: value/darken, distinct-from-bg/border, legibility
(both contrasts RUN), coverage/mutation, clean-room §20.

## Phase 4 — Validate
**Tests:** the surface exact-pin (→0.11) + the 2 new legibility asserts (foreground-on-surface≥7, border.l>
surface.l) were written at implement in `dark_194_shades_pinned_and_legible` — nothing to add (a one-value theme
change). Re-confirmed against the plan (REQ-001 pin, REQ-002 bg<surface + surface≠border, REQ-003 fg/muted-on-
surface contrasts, REQ-004 border.l>surface.l).

**RUN:** `cargo nextest run -p marley_ui_components` → **32 passed, 1 skipped** (incl the updated dark_194_shades
test + the WCAG contrast/luminance tests — the darker surface keeps everything legible).

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — cov ≥100% lines + MSI ≥100% on
ui_components/lib.rs (the surface value covered by default_for + the exact-pin; the contrast helpers' op-swap
mutants killed by the WCAG tests; default_for's only mutant [→Default] unviable). Receipt written for `/commit`.

**Driven capture (REQ-001) — DEFERRED (env-considerate), value unit-proven:** chad is STILL actively on a SHARED
desktop (Warp + Teams, mid-conversation with his agents, the desktop-share banner present) — a driven capture
would hijack his screen. Verified instead: the `surface` value is exact-pinned (`hsla(0.62,0.09,0.11)`) and the
render reads `colors.surface` everywhere (unchanged API) so every panel/dock/sidebar re-tones to the darker gray;
the contrast tests prove legibility (fg-on-surface AAA, muted-on-surface ≥4.5, both risen). **This is a clean-room
VISUAL calibration — chad's eye is the final vibe check; OFFER the visual confirm (panels now a darker Warp-like
gray) when his screen's free** — no separate ticket. gate-15 headless.

**Pre-existing failures:** none.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #231 in `[Unreleased] ### Changed` (above #230). ui_components.md — a #194/#231
note on the palette paragraph (dark surface 0.155→0.11).

**Forge capture (§19):** `aar-submit` ae52900d — completed, effectiveness 5. Lessons: (a) a SHARED theme role
(`surface`) means ONE value change re-tones every consumer (docks/sidebar/Files/cards/inputs/overlays) — the
leverage of a single-source palette; (b) DARKENING a surface RAISES light-text contrast (the darker/lo member
lowers L_lo → `(hi+.05)/(lo+.05)` goes UP) so legibility only improves — a darken is a "safe" direction (a
brighten needs the contrast re-checked); (c) hsla literals have NO cargo-mutants mutant + ThemeColors derives no
Default → `default_for` has ZERO viable mutants → the exact-pin is coverage+regression (theme-value golden;
RUN --list to confirm the only mutant [→Default] is unviable); (d) a value change should ADD the guard for what
it could break — added foreground-on-surface≥7 + border.l>surface.l asserts; (e) env-considerate: a clean-room
VISUAL calibration under a shared-desktop pixel block → exact-pinned value + contrast-proven, chad's eye is the
final vibe check. No failure-record/PR — inspect was clean.

**Env note:** a driven vibe-check (panels now a darker Warp-like gray) when chad's screen is free — no separate
ticket.

**Close + archive:** forge #231 → done. Local TICKET-231 → closed/. Pipeline doc pair → completed/. Spec status
→ Phase 5 — Complete PASS.
