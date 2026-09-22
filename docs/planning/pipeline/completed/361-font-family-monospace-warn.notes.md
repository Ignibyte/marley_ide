# appearance.font_family monospace warn — Notes

- **Forge ticket:** #361 `df3f5b6d-ada1-4c8f-ba04-4e9f66953370`
- **AAR:** `8feaaf3b-a4dd-4053-87bc-65cc0f5c40a7`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-361-font-family-monospace-warn.md
- **Pipeline spec:** 361-font-family-monospace-warn.spec.md
- **pipeline_id:** 14933c7e-335e-40cb-abff-f44bfaa41c03
- **Live on:** `5d6028e` (SECOND of the goal /work 360,361,362,363,364)

## Phase 1 — Plan
- **Request:** warn when `appearance.font_family` resolves but is PROPORTIONAL (breaks the terminal grid) — the
  #344 scope boundary. A feature (a #344/#337 follow-up).
- **Classification / tier:** work pipeline; editor/settings/font; a pure-policy extension + an app-side gpui
  metric probe (mirrors #344 exactly).
- **Forge recall (§18.3):** bulletins none. `aar-open` → `8feaaf3b`. `knowledge-context` (Plan) logged 13
  surfacings — the #337/#344 font ADs (`481365bb`, `c10033d8`, `411bc001`) + PRs (`b8524342`, `5ee9347c` — the
  assertion-on-a-non-default + app-vs-pure cov split) + my own #360 PR (`bb5c5a00`).
- **★ Recon (on `5d6028e`) — the make-or-break gpui API is CONFIRMED, the #344 pattern is clear:**
  1. **The #344 seam:** `resolve_font_family(requested, resolves, builtin) -> FontResolution{applied,flash}`
     (settings.rs:260, a clean 3-arm pure policy: empty→built-in / resolves→apply / else→fallback+not-found-flash;
     `#[derive(Default)]` on FontResolution per the mutation-viability note); `font_resolves(cx, family) -> bool`
     (app.rs:2144, gpui PUBLIC `all_font_names()`); `TERMINAL_FONT="Menlo"` (:555); boot wiring :1902-1905.
  2. **★ The gpui metric API (CONFIRMED public + per-char):** `TextSystem::advance(font_id, font_size, ch) ->
     Result<Size<Pixels>>` (text_system.rs:197 — `glyph_for_char` + `platform.advance`/`units_per_em`);
     `resolve_font(&Font) -> FontId` (:150, PUBLIC but PANICS on a missing font). The app already does
     `window.text_system().resolve_font(&mono_font).em_advance(...)` (:4973/:15633) — the idiom.
  3. **The split:** the epsilon DECISION is pure (`resolve_font_family` + `is_monospace_advance`), the `advance`
     READ is app-side (cov-excluded, like `font_resolves`). Epsilon = a size-independent RATIO (both advances
     scale with size).
- **Decisions:** D1 fold `monospace: bool` into `resolve_font_family` (a 4th arm, not-found precedes
  not-monospace); D2 `monospace = !resolves || font_is_monospace(...)` (short-circuit → no resolve_font panic on a
  missing font); D3 epsilon = ratio, 0.05, documented heuristic; D4 `i` vs `m`, Err→conservative-true; D5 pure
  decision cov/MSI 100, app-side metric read cov-excluded.
- **Prior art:** gpui `advance`/`resolve_font` (Apache-2.0 adoption); #344 `resolve_font_family` + `font_resolves`
  shapes. §20 N/A.
- **EARS:** REQ-PROPORTIONAL-WARNS, REQ-MONO-SILENT, REQ-344-UNCHANGED, REQ-EPSILON-DECISION (see spec).

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture / approach.** Extends #344's font policy. Two pure seams in `settings.rs` (the DECISION — cov/MSI
100) + one app-side gpui-metric probe in `app.rs` (the READ — cov-excluded, mirrors `font_resolves`) + the boot
wire. §14: no panic on the path (the probe gates `resolve_font` — which panics on a missing font — behind
`!resolves ||`; `advance` Err → conservative `true`). §20 N/A (Marley's own settings/font policy; gpui `advance`
is Apache-2.0 adoption). Confirmed.

**★ Confirmed accessors (recon):**
- Boot call site (app.rs:1902, inside `new_in`): `resolve_font_family(&applied.font_family, font_resolves(cx,
  &applied.font_family), TERMINAL_FONT)`. `font_res.applied` → the `font_family` struct field (:2002);
  `font_res.flash` → `status_flash: font_res.flash.map(Flash::new)` (:2053).
- `font_resolves(cx: &Context<Self>, family) -> bool` (:2144) uses `cx.text_system().all_font_names()`.
- The metric idiom (:4970-4977): `gpui::font(&fam)` → `text_system().resolve_font(&mono_font)` →
  `.em_advance(id, px(size))` → `.map(f32::from)`. So `advance(id, size, ch) -> Result<Size<Pixels>>`, and the
  Pixels→f32 is `f32::from(size.width)`.
- Test accessors: `mono_family_for_test()` (:2135, reads `self.font_family`, empty→built-in "Menlo") +
  `flash_message_for_test()` (used by the #344 drive :6149). The #344 drive
  `font_family_garbage_falls_back_to_builtin_headless` (:6137) is the exact template (persist_font_family + boot +
  assert mono_family + flash).

**File manifest (3 files):**
- `crates/marley_app/src/settings.rs` — (a) `resolve_font_family` gains a `monospace: bool` 4th param + a 4th arm
  `resolves && !monospace → { applied: "", flash: Some("Font \"<req>\" is not monospace — using <builtin>") }`
  (order: empty → !resolves → !monospace → apply); (b) a pure `is_monospace_advance(narrow_w: f32, wide_w: f32,
  epsilon: f32) -> bool` (`wide_w <= 0.0 → true`; else `(wide_w - narrow_w).abs() / wide_w <= epsilon`); (c) a
  `pub const MONOSPACE_EPSILON: f32 = 0.05`; (d) UPDATE the 3 t344 units (:1105-1123) for the new signature (pass
  `true` for monospace) + ADD a not-monospace unit + the is_monospace_advance units.
- `crates/marley_app/src/app.rs` — (a) `fn font_is_monospace(cx: &Context<Self>, family: &str) -> bool` (mirror
  `font_resolves`): `let ts = cx.text_system(); let id = ts.resolve_font(&gpui::font(family)); let size = px(16.0);
  match (ts.advance(id, size, 'i'), ts.advance(id, size, 'm')) { (Ok(i), Ok(m)) =>
  is_monospace_advance(f32::from(i.width), f32::from(m.width), MONOSPACE_EPSILON), _ => true }`; (b) the boot wire:
  `let resolves = Self::font_resolves(cx, &applied.font_family); let monospace = !resolves || Self::font_is_monospace(cx,
  &applied.font_family); let font_res = resolve_font_family(&applied.font_family, resolves, monospace, TERMINAL_FONT);`.
- `crates/marley_app/src/headless_drive.rs` — 2 drives (below).

**★ Test plan.**
| # | Proves | Test |
|---|---|---|
| U1-U4 | the policy (cov/MSI 100) | pure units: `resolve_font_family("", *, *, "Menlo")`→empty/no-flash; `("Nope", false, true, "Menlo")`→"" + not-found flash; `("Arial", true, false, "Menlo")`→"" + "is not monospace" flash; `("Fira Code", true, true, "Menlo")`→"Fira Code", no flash. |
| U5-U8 | the epsilon decision (cov/MSI 100) | `is_monospace_advance(8.0, 8.0, 0.05)`→true (mono); `(3.0, 8.0, 0.05)`→false (proportional, ~62%); `(7.7, 8.0, 0.05)`→true (~3.75%, near-mono within tolerance); `(1.0, 0.0, 0.05)`→true (bad measure). ★ boundary: `(7.6, 8.0, 0.05)`→exactly 5% → true (`<=`); `(7.59…, 8.0, 0.05)` just over → false — kills the `<=`→`<` mutant. |
| T1 | REQ-PROPORTIONAL-WARNS (wiring) | headless `font_family_proportional_falls_back_to_builtin_headless`: persist `"Helvetica"`, boot → `mono_family_for_test() == "Menlo"` AND `flash_message_for_test()` contains "Helvetica" + "not monospace". |
| T2 | REQ-MONO-SILENT (probe doesn't false-warn — end to end) | headless `font_family_monospace_applies_silently_headless`: persist `"Monaco"` (a macOS mono system font), boot → `mono_family_for_test() == "Monaco"` (APPLIED, not fallen back) AND `flash_message_for_test()` is None. ★ This is the negative-of-the-negative the pure units can't prove: a `font_is_monospace`-always-false bug passes every unit but breaks every real mono font — only T2 catches it. |

- **Uncoverable-honestly:** `font_is_monospace` (the `resolve_font`/`advance` metric read) is app-side
  (cov-excluded, integration-only, like `font_resolves`) — T1/T2 carry it end-to-end. The DECISION
  (`resolve_font_family` + `is_monospace_advance`) is the pure cov/MSI-100 surface.

**Risks / decisions.** (a) ★ `resolve_font` PANICS on a missing font → the `!resolves ||` short-circuit is
MANDATORY (D2) — inspect verifies it. (b) the epsilon is a size-independent RATIO (both advances scale with size)
+ tolerant (0.05; mono=0%, proportional=40-70%). (c) `advance` Err → conservative `true` (D4). (d) arm order:
not-found precedes not-monospace. (e) ★ Helvetica + Monaco are core macOS system fonts (present in
`all_font_names()` on any mac incl. the gate); if Helvetica somehow didn't resolve, T1's not-found arm would fire
and the "not monospace" assert would fail LOUDLY (not silently) — acceptable. (f) `f32::from(size.width)` is the
confirmed Pixels→f32 (mirrors :4976). §14; §20 N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

**Built (3 files, to the manifest):**
- `settings.rs` — `resolve_font_family` gained a `monospace: bool` 3rd param + the 4-arm body (empty → !resolves
  [not-found] → !monospace [not-monospace] → apply); a pure `is_monospace_advance(narrow_w, wide_w, epsilon) ->
  bool` (`wide_w <= 0.0 → true`; else `(wide_w - narrow_w).abs() / wide_w <= epsilon`); `pub const
  MONOSPACE_EPSILON: f32 = 0.05`. The 3 existing t344 units updated for the 4-arg signature (`true` for monospace).
- `app.rs` — `font_is_monospace(cx, family) -> bool` (after `font_resolves`): `resolve_font(&gpui::font(...))` →
  `advance(id, px(16), 'i'/'m')` → `is_monospace_advance(f32::from(i.width), f32::from(m.width), EPSILON)`; an
  `advance` Err → `true`. The boot wire (:1902): `resolves = font_resolves(...)`, `monospace = !resolves ||
  font_is_monospace(...)`, feeding `resolve_font_family(&family, resolves, monospace, TERMINAL_FONT)`.
- `headless_drive.rs` — `font_family_proportional_falls_back_to_builtin_headless` (Helvetica → "Menlo" + a
  "Helvetica … not monospace" flash) + `font_family_monospace_applies_silently_headless` (Monaco → "Monaco"
  applied, no flash).

**★ One compile fix (E0521 borrowed-data-escapes):** `gpui::font(family: &str)` would borrow the non-`'static`
`&str` into the `SharedString` (the family must outlive `'static`). Fixed by passing an OWNED `family.to_string()`
(`String: Into<SharedString>` needs no borrow) — matching the render-side idiom (:4969 passes a local `String`).
Not a design change; the design's `gpui::font(family)` sketch just needed the owned form.

**Checks:** `cargo fmt --all` clean; `cargo check -p marley --all-targets` CLEAN (after the E0521 fix); `cargo
clippy -p marley --all-targets -- -D warnings` **exit 0**. Diff = settings.rs + app.rs + headless_drive.rs.

**Deviations from design:** only the `family.to_string()` (E0521), noted above. Everything else byte-for-byte the
ratified bodies.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 critics (Agent general-purpose, parallel) + my own review. Verdict: 2 REAL findings fixed (1 HIGH, 1 MEDIUM);
the panic-gate / policy / provenance all CONFIRMED clean.**

**Critic 1 — panic-gate + policy + epsilon mutation set.**
- (a) ★ the `resolve_font` PANIC gate CONFIRMED safe: `let monospace = !resolves || Self::font_is_monospace(...)`
  is `||` (short-circuit), same family arg, sole caller → `resolve_font` never runs on a non-resolving font.
- (b) the 4-arm policy + order CONFIRMED (not-found before not-monospace; all fallbacks applied=""; flashes name
  family+builtin).
- (c) **[MEDIUM] REAL — FIXED.** `if wide_w <= 0.0 { return true }` yields an EQUIVALENT `<= → ==` mutant
  (diverges only at `wide_w < 0`, where the fall-through `(≥0)/(<0) ≤ 0 ≤ ε` ALSO returns true → survives → MSI
  < 100). **Fixed:** restructured to a POSITIVE `if wide_w > 0.0 { ratio } else { true }` — boundary at
  `wide_w == 0`, all `>`-mutants killed by the planned units `(1.0,0.0)` + `(3.0,8.0)`. (`!(wide_w > 0.0)` also
  kills it but trips clippy `neg_cmp_op_on_partial_ord`; the if/else form is mutant-complete AND clippy-clean.)
  Critic 1 also confirmed the rest of the `is_monospace_advance` mutant set is killed by the planned units (the
  `/`→`*` mutant needs the near-mono `(7.7,8.0)` case; the `<=`→`<` needs the boundary `(7.6,8.0)`), and that
  `.abs()` has no mutant (cargo-mutants leaves method calls unmutated) → not an MSI risk. `BF-…-guard-equivalent-mutant-001`.
- (d) provenance + the E0521 `family.to_string()` fix CONFIRMED (mirrors the render idiom; `f32::from(width)`;
  `advance` not `em_advance`; no unwrap on the probe).

**Critic 2 — the drives prove it end-to-end + the cov split.**
- (a)/(b)/(c) **[HIGH] REAL — FIXED.** The two headless drives CANNOT PASS: `#[gpui::test]` uses gpui's
  `NoopTextSystem`, under which `all_font_names()` reports nothing resolvable → "Helvetica"/"Monaco" read
  UNRESOLVABLE → the boot wire's `!resolves` short-circuit fires → `font_is_monospace` is never called → the
  not-monospace arm is UNREACHABLE headless (and Noop's synthetic advance reads every font as monospace anyway).
  The #344 garbage-drive comment already documents this ("The RESOLVABLE case is not headless-testable here (Noop
  resolves nothing) — the pure unit covers it"). **My design-level miss:** I read the #344 drive's CODE but not
  its COMMENT's rationale. **Fixed:** removed both drives; replaced with a comment documenting the Noop limitation
  + pointing at the pure units. The DECISION is proven by the pure units (settings.rs, cov-included); the
  app-side metric READ (`font_is_monospace`) is cov-excluded + headed-only, exactly like #344's `font_resolves`.
  `BF-…-headless-drives-unreachable-under-noop-text-system-001` + `PR-claude-headless-gpui-test-uses-noop-text-system-not-coretext-001`.
- (d) the app-vs-pure cov split CONFIRMED CORRECT: `settings.rs` (the decision) is cov-INCLUDED (not in
  gates.sh:217's ignore-regex — I verified independently); `app.rs` (the metric read) is cov-EXCLUDED. The pure
  units are the cov/MSI-100 surface.

**My own step-2 review (independent):** verified `settings.rs` cov-included / `app.rs` cov-excluded via the gate
regex; verified the `.abs()`-deletion mutant is killable by a `narrow > wide` input (moot — method calls
unmutated); read the #344 comment → confirmed Critic 2's Noop finding is real, not a false alarm.

**Findings acted on:** the MEDIUM (guard restructure) + the HIGH (drives removed) — both fixed at source, clippy
`-D warnings` exit 0 after. **The revised test lane: PURE UNITS ONLY** (the app metric-read is headed-only,
deferred like #344's resolvable case; a possible headed-lane end-to-end verification is a #344/#361-shared
follow-up, weighed at Phase 5).

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests WRITTEN (pure units in settings.rs — the cov/MSI-100 surface; NO headless drive, per P3.5):**
- `t361_resolve_font_family_proportional_falls_back_and_warns` — the not-monospace arm (`"Arial", true, false` →
  applied "" + "is not monospace" flash). (The empty/!resolves/apply arms are the 3 updated t344 units.)
- `is_monospace_advance` ×5: equal-widths `(8,8,.05)`→true; wide-spread `(3,8,.05)`→false; near-mono
  `(7.7,8,.05)`→true [kills `/`→`*`]; boundary `(7.5,8,.0625)`→true (exact `0.5/8=0.0625==ε`) + just-over
  `(7.0,8,.0625)`→false [kills the epsilon `<=`]; zero-width `(1,0,.05)`→true [kills the guard `>`→`>=`].
- ★ f32 boundary correction: the planned `(7.6,8,.05)` rounds to `0.05000001` (> 0.05) → would FAIL; switched to
  the EXACT `(7.5,8,.0625)` (`0.5/8=0.0625` is exact f32) for the true `ratio==ε` boundary (verified via a python
  f32 harness before writing).

**Tests RUN:**
- Units: `cargo nextest run -p marley -E 'test(t361) + test(t344)'` → **9 passed** (6 t361 + 3 t344).
- App regression: `cargo nextest run -p marley` → **742 passed, 2 skipped** (net vs #360's 736: −2 removed
  drives + the new units; no regression; the `search_open…` flake did not recur).

**Live drive:** NONE — stated deliberately. The change is settings/font policy; the pure units prove the
DECISION; the app-side gpui metric READ (`font_is_monospace`) is headed-only (needs real CoreText — off-limits,
chad may be at the machine) and cov/mutants-excluded as a shim, exactly like #344's `font_resolves`.

**Full `--diff` gate:** first run **GATE RED [diff]** — gate:5 mutation had 3 SURVIVORS, all in
`app.rs::font_is_monospace` (`→true`, `→false`, `delete match arm`). Root cause: coverage-excluded (llvm-cov
ignore-regex) ≠ mutants-excluded — cargo-mutants still mutates an app-side probe. **Fixed at source (§0, not a
suppression):** added `#[cfg_attr(test, mutants::skip)]` to `font_is_monospace` — the identical treatment its
sibling `font_resolves` already carries ("shim: a gpui font-name probe"); the DECISION it delegates to
(`is_monospace_advance`) is the unit-tested seam. Verified via `cargo mutants --list` that both probes now show
NO mutants and the `settings.rs` decision mutants remain (all killed by the units). Re-run → **GATE GREEN [diff]
— 15/15 PASS**, receipt `841e73e606ec0d92fb4d294041c78537364913ee`. gate:4 coverage ≥100% (settings.rs
cov-included, both pure fns fully exercised; app.rs excluded); gate:5 MSI ≥100% (16 settings mutants killed by
the units; the 2 app probes `mutants::skip`); gate:14 docs PASS. No pre-existing exclusions.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21).**
- **CHANGELOG.md** — a `### Added` entry: `appearance.font_family` warns when a resolvable font is not monospace
  (the i-vs-m advance probe; the pure decision + the shim metric read; not headless-testable under Noop).
- **docs/marley_architecture/app_shell.md** — extended the #344 font-probe paragraph to note #361's
  monospace-ness check (`font_is_monospace` → `is_monospace_advance`, the 4th `resolve_font_family` arm, the
  Noop-headless limitation). NON-`.rs` only → the Phase-4 receipt HOLDS (the settings.rs `define_setting!`
  FontFamily doc is accurate-as-is; the arch note covers the extension).

**Capture (forge wired).**
- `aar-submit` 8feaaf3b, outcome completed, effectiveness 3 (2 novel; a bumpier run — 2 inspect fixes + a
  gate-red). Headline lessons: (a) the gpui `advance` recon cleared the plan's central risk up front. (b) ★★ the
  BIG MISS — the 2 headless drives were unreachable under `#[gpui::test]`'s NoopTextSystem; I read the #344
  drive's code but not its comment's *why* (`PR-…-noop-text-system`). (c) the equivalent-mutant guard → positive
  `if wide_w > 0.0` gate. (d) ★★ coverage-excluded ≠ mutants-excluded — `font_is_monospace` (app.rs, cov-excluded)
  still got 3 mutation survivors → needed an explicit `#[cfg_attr(test, mutants::skip)]` (like `font_resolves`).
  (e) the f32 boundary correction (`(7.6,8)` rounds over → `(7.5,8,.0625)` exact).
- `prevention-rule-record` **PR-claude-cov-excluded-is-not-mutants-excluded-shim-needs-skip-001** (3a8085a8): a
  cov-excluded app-side shim is still mutated → needs an explicit fn-level `mutants::skip`; file-level cov
  exclusion and fn-level mutation exclusion are separate mechanisms. (Plus the inspect-recorded
  `PR-…-noop-text-system-not-coretext-001`.)
- `failure-record`: 3 total — the 2 inspect BFs (headless-drives-unreachable, guard-equivalent-mutant) +
  `BF-claude-361-cov-excluded-fn-still-mutated-msi-red-001` (8efe9d05, the gate-red).
- **Follow-up: forge #365** (`afb7aa3c`, editor/settings/font/test-lane/headed) — a headed/real-CoreText
  end-to-end drive proving BOTH #344's resolvable-apply AND #361's monospace paths (neither headless-testable
  under Noop; depends on the #264/#271 headed-lane deferral).

**Close + archive.** forge #361 → done; TICKET-361 → closed/ (`status: closed`); the spec/notes pair archived
active/ → completed/; spec `status: Phase 5 — Complete PASS`.

**Status: Phase 5 — Complete PASS.** Run `/commit` to deliver (LOCAL — push un-OK'd). SECOND of the goal /work
360…364 (2 of 5). ⚠️ P5 was docs-only (CHANGELOG + app_shell.md, NO `.rs` edit) → the Phase-4 receipt
`841e73e6` HOLDS.
