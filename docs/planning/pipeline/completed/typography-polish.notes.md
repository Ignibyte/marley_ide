# typography + focus/hover (M1.E finale) — Notes

- **Forge ticket:** #39 `85e6b003-36f3-4dbf-8f4f-d8885d84b5f6`
- **AAR:** `af2f8cc1-6a56-4ecb-aee0-8bedd4533c40`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-039-typography-polish.md
- **Pipeline spec:** typography-polish.spec.md

## Phase 1 — Plan
- **Request:** forge #39 (M1.E "The Warp Look" seq-6 FINALE, auto-approved) — the cohesion pass.
- **Classification / tier:** work pipeline, `feature`, TWO pure surfaces (`muted` data in
  ui_components + `type_scale`/`weight_value` logic in a new gpui-free marley_app module) + a SHIM
  render-tuning application. Two crates.
- **Discovery (§18):**
  - The render has ad-hoc `FontWeight::BOLD`/`MEDIUM` (app.rs:780/810/826) + `text_sm` (108, the #38
    dock header) scattered — `type_scale` consolidates them.
  - `is_focused` accent border already drawn (app.rs:722); the palette highlights the active row
    (#25) — the hover vocabulary extends these.
  - `ThemeColors` (7 color + 3 metric fields, #35) — adding `muted` ripples ONLY to `default_for`
    (both arms) + the field-equality tests (the widget `dark()` fixtures call `default_for`) — the
    same contained ripple as #35's `success`.
  - gpui `FontWeight(pub f32)`: NORMAL 400 / MEDIUM 500 / BOLD 700 → `weight_value` returns these as
    `f32`, the shim wraps `gpui::FontWeight(weight_value(w))` (keeps typography.rs gpui-free).
  - marley_app modules end at `workspace` → add `typography`.
  - Deps #34–#38 all DONE.
- **Decisions:** D1–D4 in the spec (gpui-free typography; muted new role; 3-role scale; pure
  data+logic, shim application).
- **Open questions for Design:** the exact `muted` dark/light `hsla` (a mid-gray between fg and
  border); the scale values (14/14/12 sizes, Medium/Normal/Normal weights — a starting point);
  whether `weight_value` returns f32 (yes — gpui-free) vs a gpui FontWeight; the hover treatment
  (a `.hover()` bg tint on Block/palette rows).
- **AAR id:** `af2f8cc1-6a56-4ecb-aee0-8bedd4533c40`.
- **NOTE:** this is the LAST M1.E ticket — Phase 5 also CLOSES forge sprint #5.

## Phase 2 — Design

### PURE — `crates/ui_components/src/lib.rs` (`muted` on ThemeColors)
`ThemeColors` gains `muted: Hsla` (after `success`). Values (a mid-gray between fg and border):
- Dark: `hsla(0.62, 0.08, 0.60)` (dimmer than fg L0.90, brighter than border L0.22).
- Light: `hsla(0.62, 0.10, 0.45)` (a gray caption on the L0.97 bg, distinct from the dark fg).
Both distinct from `foreground`/`background`/`border`. Ripple: struct + `default_for` (both arms) +
the field-equality tests (the widget `dark()` fixtures call `default_for` — no ripple, as #35).

### PURE — `crates/marley_app/src/typography.rs` (NEW, gpui-FREE)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role { Command, Output, Caption }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWeight { Normal, Medium, Bold }
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle { pub size: f32, pub weight: TextWeight }

/// The type scale (R45): one place for the command/output/caption size + weight.
pub fn type_scale(role: Role) -> TextStyle {
    match role {
        Role::Command => TextStyle { size: 14.0, weight: TextWeight::Medium },
        Role::Output  => TextStyle { size: 14.0, weight: TextWeight::Normal },
        Role::Caption => TextStyle { size: 12.0, weight: TextWeight::Normal },
    }
}
/// The CSS numeric weight for a `TextWeight` — the shim wraps it in `gpui::FontWeight(..)` so this
/// module stays gpui-free.
pub fn weight_value(weight: TextWeight) -> f32 {
    match weight { TextWeight::Normal => 400.0, TextWeight::Medium => 500.0, TextWeight::Bold => 700.0 }
}
```

### SHIM — `crates/marley_app/src/app.rs`
- `mod typography;` in lib.rs. Import `typography::{type_scale, weight_value, Role}`.
- Command header (#36 block_status header): apply `let s = type_scale(Role::Command);
  .text_size(px(s.size)).font_weight(FontWeight(weight_value(s.weight)))`; add a hover affordance
  `.hover(|d| d.bg(colors.surface))` on the header row.
- Dock caption (`dock_panel`, #38): `let c = type_scale(Role::Caption); .text_size(px(c.size))
  .text_color(colors.muted)` (replacing the `text_sm` + `foreground`).
- Output rows are the `Output` base (already 14pt terminal font) — the scale documents it; the ANSI
  bold stays driven by the terminal `Flags` (#31).

### File manifest
- M `crates/ui_components/src/lib.rs` — `muted` field + `default_for` values + field-equality tests.
- A `crates/marley_app/src/typography.rs` — the pure module + tests.
- M `crates/marley_app/src/lib.rs` — `mod typography;`.
- M `crates/marley_app/src/app.rs` — apply the scale to the header (+ hover) + the dock caption (muted).
- M `docs/specs/SPEC-ui-components.spec.md` (muted) + `docs/specs/SPEC-app-shell.spec.md` (R45 + MT).
  CHANGELOG; arch docs.

### Mutation Targets
- `muted` — a value (hsla literal, not mutated); the field-equality test pins it (regression guard).
- `type_scale` — the 3 role arms (whole-fn → `Default` UNVIABLE — TextStyle has no Default; arm-delete
  → non-exhaustive → unviable; likely 0 viable, coverage-driven — like #35). Killed/covered by the
  per-role exact-`TextStyle` asserts.
- `weight_value` — whole-fn → `0.0`/`1.0` (VIABLE, killed by `Normal→400.0` etc.). Covered by the 3
  per-weight asserts.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `muted_present_and_distinct` — `default_for(Dark).muted == hsla(0.62,0.08,0.60)`, `(Light).muted == hsla(0.62,0.10,0.45)`; `muted != foreground/background/border` both appearances | unit (ui_components) |
| REQ-002 | `type_scale_by_role` — `Command→(14,Medium)`, `Output→(14,Normal)`, `Caption→(12,Normal)` | unit (typography) |
| REQ-003 | `weight_value_by_weight` — `Normal→400.0`, `Medium→500.0`, `Bold→700.0` | unit (typography) |
| REQ-004 | the render application (header scale + hover; dock caption muted) | shim + masked full-cockpit visual — chad's final sign-off |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs render application (shim exclude; needs a live window).

### Risks / decisions
- `typography.rs` gpui-FREE (weight as `f32`) → fully unit-testable without a window; the shim wraps
  `FontWeight`. D-2.1.
- `type_scale` may yield 0 viable mutants (struct returns, no Default) — coverage carries it (like #35);
  the exact-value asserts are the regression guard. `weight_value`'s `0.0`/`1.0` mutants ARE viable +
  killed. No equivalent-mutant risk (distinct values per arm).
- Adding `muted` is the same contained ThemeColors ripple as #35's `success` — the field-equality
  tests + the two default_for arms; no widget/destructure breakage (additive field).
- Contrast: `muted` is a caption tone (lower contrast BY DESIGN) — not held to AA body-text; chad
  eyeballs. Distinct-from-fg/bg/border is asserted so it's never invisible against them.

## Phase 3 — Implement
- **Built (per manifest):** ui_components/lib.rs — `ThemeColors.muted` + both `default_for` values
  (dark `0.62,0.08,0.60` / light `0.62,0.10,0.45`); `typography.rs` (NEW, gpui-free) —
  `Role{Command,Output,Caption}` + `TextWeight{Normal,Medium,Bold}` + `TextStyle{size,weight}` +
  `type_scale` (14/14/12, Medium/Normal/Normal) + `weight_value` (400/500/700); `lib.rs` `mod
  typography;`; `app.rs` — the Command header uses `type_scale(Command)` + a `.hover(bg surface)`
  tint; the dock caption uses `type_scale(Caption)` in `colors.muted`; the output rows use
  `type_scale(Output)` size; ALL ANSI-bold sites (cooked output + alt-screen grid) route through
  `FontWeight(weight_value(TextWeight::Bold))`. SPEC-ui-components (`muted`) + SPEC-app-shell (R45 +
  row 45 + Mutation-Targets); CHANGELOG.
- **Deviation from design (positive):** the design applied only Command + Caption; a `dead_code` error
  on the unused `Role::Output`/`TextWeight::Bold` surfaced that the consolidation was incomplete — so
  I routed EVERY weight/role site through the scale (output size via `Output`, all ANSI-bold via
  `Bold`). This is the fuller cohesion the ticket intends (no ad-hoc `FontWeight::*` left in the
  render) and makes all variants live.
- **Verification at this phase:** `cargo check -p marley` 0 errors; fmt; clippy `-D warnings` 0
  (marley + ui_components); docs gate 0; ui_components 28 lib tests pass. The muted + typography unit
  suites are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (real scoped cargo-mutants on typography.rs [`-j2` fresh target dir] + muted
  distinctness/clean-room + a full consolidation grep + workspace check). Verdict: **DIFF IS CLEAN,
  no blocker.**
- **Findings table:**
  | # | Sev | Finding | Verdict | Carry-forward |
  |---|---|---|---|---|
  | F1 | MED (Phase-4 guardrail) | `type_scale` has 0 viable mutants (whole-fn→`Default` UNVIABLE — no Default; no per-arm mutants), so cov+MSI 100 is reachable with a WEAK assert: a size-only assert leaves `Command(14,Medium)`↔`Output(14,Normal)` indistinguishable yet green. `weight_value`'s 3 viable mutants (→0.0/1.0/-1.0) force only ONE discriminating assert, not pinning 500/700. | REAL (the gate won't enforce it) | P4 asserts the FULL `TextStyle` per role (size AND weight, via `PartialEq`) ×3 + each weight's exact value ×3 — the design's REQ-002/003, now known load-bearing not just thorough. |
  | F2 | LOW | The #35 `theme_colors_carries_all_roles_and_metrics` comment said "7 roles" (now 9 with success+muted). | REAL (doc) | Fixed → "9 roles". |
  | F3 | LOW | task's `grep -c gpui` = 4 — all doc-comment prose ("gpui-free"); purity holds (no `use`, no gpui type). | REJECTED (false positive) | none. |
- **Verified CLEAN by the critic:**
  - **Consolidation COMPLETE** (the ticket's point): 0 `FontWeight::BOLD/MEDIUM/NORMAL` + 0
    `.text_sm/xs/lg()` left in the render; all 3 `.font_weight` sites route through
    `FontWeight(weight_value(..))`, all size sites through `type_scale(..).size`. `TERMINAL_FONT_SIZE`
    (#34 mono metric) correctly left as a separate concern (and cohesive — == Output size 14).
  - **muted** distinct from fg/bg/border in both appearances; a generic mid-gray, NOT a branded lift;
    `success` (#35) still pinned — no palette regression.
  - gpui-free purity real; ripple complete (`cargo check --workspace` 0 errors — a missing `muted`
    literal = hard error; only `default_for` constructs; no exhaustive destructure); both fns total,
    no panic.
- **No code fix** beyond the F2 comment. F1 is the critical Phase-4 instruction.
- **Lesson:** `PR-claude-no-default-struct-return-needs-full-value-assert` — a pure fn returning a
  struct that derives NO `Default` yields ZERO viable cargo-mutants (whole-fn→Default is unviable, and
  literals/enum-arm bodies aren't mutated), so cov+MSI 100 does NOT force the test to pin the returned
  VALUES — a weak assert (one field, or one arm) passes green while sibling arms stay indistinguishable.
  Assert the FULL returned value for EVERY arm; the gate can't catch a weak assert here.

## Phase 4 — Validate
- **Tests added:** `typography.rs` — `type_scale_by_role` (the FULL `TextStyle` per role — size AND
  weight — the critic's MED guardrail, so Command(14,Medium)/Output(14,Normal) can't collapse) +
  `weight_value_by_weight` (each 400/500/700). `lib.rs` — `muted_present_and_distinct` (exact dark/
  light values + `!= foreground/background/border` both appearances) + the F2 comment fix (7→9 roles).
- **Runs (actual):** `cargo nextest run -p marley -p marley_ui_components` → 130 passed (all 3 new
  PASS); `cargo nextest run --workspace` → 507 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **3 caught /
  0 missed → MSI 100.0%** (weight_value's 3 viable whole-fn mutants; type_scale has 0 viable [no
  Default] — carried by coverage + the full-value asserts; muted is hsla data pinned by the golden).
  Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` (type-scale bullet, M1.E
  CLOSED) + `ui_components.md` (`muted`). SPEC-ui-components + SPEC-app-shell R45 at implement.
- **Knowledge captured:** `PR-…-no-default-struct-return-needs-full-value-assert` (a struct return
  with no Default → 0 viable mutants → assert the FULL value per arm; MSI can't enforce it — the
  struct-return cousin of the hsla/self-oracle traps). aar-submit `completed` (score 5). Win: the
  dead_code error on `Role::Output`/`TextWeight::Bold` was a GIFT — it revealed the consolidation was
  half-done, so the finale routes EVERY weight/size through the scale (no ad-hoc `FontWeight::*` left).
- **Ticket:** forge #39 → done; local doc → closed/; pipeline pair archived. **6 of 6 in M1.E.**
- **SPRINT:** forge sprint #5 "M1.E — The Warp Look" CLOSED. `cargo run -p marley` reads as a finished
  Warp-style terminal — mono font, designed palette, carded Blocks w/ status, cwd/git prompt, dock
  panels, one type scale.
