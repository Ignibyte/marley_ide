---
pipeline_id: e7707db6-26b5-46c4-9bcb-725bf5fc4934
aar_id: b492f0f1-9df8-4ff2-bcc0-8bb1335c7631
---

# marley_visual_harness text baselines — pipeline notes

## Phase 1 — Plan (2026-06-29)

**Intent:** the M1 enabler (sprint M1.A seq 1/5) — text-tolerant screenshot baselines so every M1 UI
crate can be visually gate-tested. Closes `AD-claude-headed-visual-baseline-text-tolerance-deferred-001`.
Existing-crate enhancement (marley_visual_harness, forge #10), not a new crate.

**Confirmed-present primitives (image_diff.rs):** `Tolerance` (+ `gate15_default`), `ChannelSet`
(+ `rgb`), `RegionMask` (line 78), `compare(a,b,tol)`, `diff(a,b)`, `perceptual_distance`. So the
additions BUILD ON these — minimal new surface.

## Carry to Design (Phase 2 does these)

1. **READ FIRST** (exact current behavior): `crates/marley_visual_harness/src/image_diff.rs` —
   `Tolerance` fields + `gate15_default` (what thresholds?), `RegionMask` (how does it represent a
   region — rects? a bool grid? does `compare`/`diff` ALREADY accept/honor a mask, or is it unused?),
   `compare`/`diff`/`perceptual_distance` (how the per-pixel + perceptual decision combine);
   `baseline.rs` `evaluate_baseline` (does it take a Tolerance? a mask?); `capture.rs`
   `assert_matches_baseline_in` (does it pass a Tolerance/mask, or hardcode gate15_default?).
2. **Design the minimal additions:**
   - `RegionMask::exclude_top_band(px)` (or similar) — exclude a titlebar band. Decide the px height
     (the AX size includes ~32pt titlebar = ~64px@2x; mask a bit more to cover the whole chrome).
     Region math PURE + testable.
   - `Tolerance::gate15_text()` (or a builder on Tolerance) — a looser perceptual/SSIM threshold for
     text. Decide the threshold (calibrate against the text fixture's run-to-run jitter). Keep it
     STRICT enough to reject a real content change (AC-2/AC-3 both-sides test).
   - Thread the mask through `compare`/`evaluate_baseline` so masked pixels are excluded from BOTH the
     per-pixel count AND the perceptual distance. Decide the cleanest signature (a mask field on
     Tolerance? a `compare_masked(a,b,tol,mask)`? — pick the one that keeps the pure layer testable +
     `assert_matches_baseline_in` able to pass it).
   - `assert_matches_baseline_in` / a new `assert_matches_baseline_masked` so the headed test can
     supply the text tolerance + titlebar mask.
3. **The text fixture** — a `TextFixture` (parallel to `TwoElementFixture` in selftest.rs): a gpui
   window rendering a known header line + body line on a fixed bg. A new runner bin (like
   marley_harness_selftest) OR extend the existing selftest runner with a mode. Decide which keeps the
   shim minimal. It's ACCEPTED-UNTESTABLE (mutants::skip + rust_cov exclude — extend the existing
   regex).
4. **Mutation map** — the region-bound math (exclude_top_band) risks EQUIVALENT/loop-bound mutants
   (the 005/007 lesson: use a fixture where the band boundary is load-bearing — a diff pixel exactly
   at band_height-1 vs band_height). The threshold comparison (`<=` vs `<`) needs both-sides tests.
5. **Headed determinism** — the AC-4 proof: capture the text fixture 3× and confirm the masked+text
   compare Matches each time. (This is what marley_spike couldn't do; here it's the deliverable.)

**Phase 1 status:** PASS (autonomous-through-commit per chad's /goal). → Phase 2 Design.

## Phase 2 — Design (2026-06-29)

### Read findings (image_diff.rs / selftest.rs)
- `Tolerance { per_channel:u8, max_changed_fraction:f32, perceptual_mean:f32 }`; `gate15_default = (4,
  0.0, 1.0)`. **`max_changed_fraction = 0.0` is the text-killer** — ZERO pixels may differ beyond
  per_channel=4, so any subpixel-AA jitter = a changed pixel = fail. (`perceptual_mean=1.0` is the max
  → effectively unbounded; the per-pixel fraction dominates.)
- `RegionMask { w, h, inside:Vec<bool> }` + `contains_px(x,y)` already exist — but only `state_delta`
  (R18) consumes a mask. **`compare(a,b,tol)` does NOT take a mask** — it walks all pixels via
  `measure`. `perceptual_distance` is `1 - SSIM` (image_compare rgba_hybrid) over the WHOLE image.
- `selftest.rs` `TwoElementFixture` **already renders TEXT** ("UPPER"/"LOWER") + its strict baseline
  (`harness_selftest.png`, 800×664 @2x) passes. So strict tolerance is BORDERLINE-ok for that small
  centered fixture but failed marley_spike's larger top-left text 3/3 (env focus/AA). The robust fix
  generalizes it for all M1 windows.

### Architecture — mask-out-then-compare (minimal, additive; NO change to compare/measure/state_delta)
Three PURE additions in `image_diff.rs` + one testable wrapper in `capture.rs`. Keep `gate15_default`
+ the existing self-test untouched.
1. `RegionMask::titlebar_band(w:u32, h:u32, band_px:u32) -> RegionMask` — `inside[y*w+x] = y < band_px`
   (the top band = the focus-variable titlebar chrome). PURE region math.
2. `Tolerance::gate15_text() -> Tolerance` — looser for text: `per_channel` ~8, `max_changed_fraction`
   ~0.02 (allow the AA-jittered edge pixels), `perceptual_mean` ~0.05 (the real gate — SSIM catches a
   content change). EXACT values calibrated at validate against the fixture's run-to-run jitter, with a
   BOTH-SIDES test (must still reject a real content change — not floor-lowering, it's a new
   purpose-built tolerance proven to reject changes).
3. `mask_out(img:&Image, mask:&RegionMask, fill:[u8;4]) -> Image` — return a copy with `inside` pixels
   set to `fill`. Applied to BOTH baseline + capture → the titlebar is identical there → excluded from
   BOTH the per-pixel count AND the whole-image SSIM (a uniform region contributes ~0 diff). PURE.
4. `capture.rs` `Screenshot::assert_matches_baseline_masked(crate_root, target_dir, name, tol, mask)`
   — load baseline, `mask_out` both, run the EXISTING `evaluate_baseline(masked_base, masked_cap, tol)`
   + `resolve_approval` flow. Testable wrapper (like `assert_matches_baseline_in`, via `put_baseline`).

**SIMPLIFICATION vs the plan:** REUSE `TwoElementFixture` + `marley_harness_selftest` bin +
`harness_selftest.png` for the headed proof — NO new fixture/bin/baseline. The masked headed test
asserts the existing fixture via the new path: it masks the titlebar (so the focus-variable traffic
lights can't fail it) and uses gate15_text — proving the path runs headed + is deterministic. The
MECHANISM rigor (mask excludes a band; tolerance accepts jitter but rejects a change) is in the PURE
unit tests, not the headed lane. → no new shim file, no gates.sh rust_cov change.

### File manifest (edits only)
- `crates/marley_visual_harness/src/image_diff.rs` — `RegionMask::titlebar_band`, `mask_out`,
  `Tolerance::gate15_text` (PURE; the existing test module gets their tests).
- `crates/marley_visual_harness/src/capture.rs` — `assert_matches_baseline_masked` (testable wrapper).
- `crates/marley_visual_harness/src/lib.rs` — re-export the new pure items if the pattern re-exports.
- `crates/marley_visual_harness/tests/headed_text.rs` — NEW `#[ignore]` headed test: launch
  `marley_harness_selftest`, capture, `assert_matches_baseline_masked(crate_root, target/, "harness_selftest",
  gate15_text(), RegionMask::titlebar_band(w, h, 70))` — assert deterministic ×3.
- CHANGELOG.md + docs/marley_architecture/marley_visual_harness.md (§21).
- NO new bin, NO new baseline PNG, NO scripts/gates.sh change (no new shim/coverage surface).

### Regression Test Plan (one row per AC)
| AC | Test (in-crate `#[cfg(test)]` unless noted) | Kills |
|---|---|---|
| AC-1 | `titlebar_band_masks_top_rows` — `titlebar_band(3,4,2)`: assert `contains_px(_,0)`+`(_,1)` true, `(_,2)`+`(_,3)` false (band boundary load-bearing; kills `<`→`<=` / off-by-one — multi-row) | bound mutant |
| AC-1 | `mask_out_blanks_inside_only` — 1×2 image, mask row 0: assert row0==fill, row1==original | masked-branch |
| AC-2 | `gate15_text_accepts_aa_jitter_rejects_content` — base vs base+small-jitter (≤fraction) → Match; base vs base+big-block-change → Mismatch (BOTH sides; exercises the gate15_text thresholds via compare) | threshold `<=`/`<` |
| AC-3 | `masked_compare_ignores_masked_diff` — a big diff confined to a masked region → mask_out both → compare Match; the SAME diff unmasked → Mismatch | mask exclusion |
| AC-3 | `assert_matches_baseline_masked_wrapper` — `put_baseline` a known PNG, capture = baseline with a titlebar-band difference → masked assert Pass (covers the wrapper, like the _in test) | wrapper |
| AC-4 | `tests/headed_text.rs` (`#[ignore]`, headed lane) — the existing fixture via the masked+text path, deterministic ×3 | headed proof |
| AC-5 | FULL gate green; pure additions cov 100 / MSI 100 | — |

### Risks / decisions
1. **gate15_text calibration** — pick per_channel/fraction/perceptual at implement; PROVE both-sides at
   validate (accept the measured jitter, reject a synthesized content change). If the whole-image SSIM
   is too lenient after masking (the masked region inflates similarity), tighten perceptual_mean or
   crop perceptual to the unmasked bbox — decide only if the AC-3 reject side fails.
2. **band_px=70** is retina-2x-calibrated (the ~64px@2x titlebar + margin) — fine for this env; a real
   cross-display harness would compute it from capture-vs-content height (out of scope for M1.A).
3. **Equivalent-mutant guard** on `titlebar_band`'s `y < band_px` — the AC-1 multi-row fixture with the
   boundary at row 2 makes `<`↔`<=` observable (the 005/007 lesson).

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-06-29)

Added (3 files, +88/−3): `image_diff.rs` — `Tolerance::gate15_text` (per_channel 8 / max_changed_fraction
0.02 / perceptual_mean 0.05), `RegionMask::titlebar_band(w,h,band_px)` (top `band_px` rows inside),
`mask_out(img, mask, fill) -> Image` (copy with masked pixels filled); `capture.rs` —
`Screenshot::assert_matches_baseline_masked(crate_root, target_dir, name, tol, mask)`; `lib.rs` —
re-export `mask_out`. All 3 image_diff fns PURE, NO `mutants::skip`. check + clippy(-D warnings) + fmt
clean. Nothing existing changed (gate15_default / compare / measure / state_delta / self-test untouched).

**Deviation (correct — the literal sketch didn't type-check):** `evaluate_baseline` takes
`Option<&Image>` (its first param) and `load_image_png` returns `Option<Image>`, so the masked baseline
is threaded preserving the Option (and thus the fail-closed `Missing` case):
`let mb = baseline.as_ref().map(|b| mask_out(b, mask, [0,0,0,255])); let mc = mask_out(&self.img, mask,
[0,0,0,255]); evaluate_baseline(mb.as_ref(), &mc, tol)`. `WriteBaseline` saves the UNMASKED `self.img`;
`FailWritePending`/`write_pending` pass the UNMASKED images (pending shows reality). Otherwise identical
to `assert_matches_baseline_in`.

**For validate (from the subagent):** `evaluate_baseline(baseline: Option<&Image>, capture: &Image, tol:
Tolerance) -> BaselineDecision{Match,Mismatch(PixelDiff),Missing}`. Reusable test scaffolding in
capture.rs's test mod: `tempfile::tempdir()`, `put_baseline(root,name,&img)`, `Screenshot::from_image`,
`solid(w,h,color)`, `ApproveGuard` (+ `#[serial]` on env-touching tests). Mirror
`baseline_matching_passes_without_panic` (Pass) + `baseline_mismatch_fails_closed`
(`#[should_panic(expected="wrote pending")]`) for the masked wrapper tests.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 1 critic (pure-layer correctness + mutation-readiness)

**Verdict:** production code CORRECT — NO source bugs. Verified: `titlebar_band` index math (row-major,
no OOB); `mask_out` safe even if mask dims ≠ img dims (writes into img's own rgba, contains_px
bounds-checks); `assert_matches_baseline_masked` preserves fail-closed `Missing` (None→map→None→Missing),
saves the UNMASKED `self.img` on WriteBaseline, passes unmasked images to write_pending, no undocumented
panic; ALL `Default::default()` return mutants UNVIABLE (no Default on Tolerance/RegionMask/Image/
Screenshot → gate15_text has 0 viable mutants); capture.rs + image_diff.rs ARE coverage-counted (need
the unit tests). Two findings — both TEST-DESIGN guidance for validate, no source edits:

| # | Sev | Finding | Resolution |
|---|---|---|---|
| 1 | HIGH | The Phase-2 `mask_out` kill-fixture (1×2, mask ROW 0 = the ORIGIN) is DEGENERATE: index 0 is a fixed point of `*`/`/`/`+`/`-`, so 4 index mutants on image_diff.rs:283 survive → MSI<100. (Sharper 005/007 lesson.) | Use a NON-ORIGIN interior fixture (kill map below). Forge `BF-…-origin-collapse-fixture-…-001` + `PR-claude-nonorigin-interior-fixture-for-index-arithmetic-mutants-001`. |
| 2 | MED | `gate15_text` accepts a SMALL localized content change (<2% px → fraction<0.02 AND whole-image-mean SSIM<0.05). Inherent to any whole-image-mean threshold; masking dilutes both gates ~uniformly 10% (NOT a targeted blind spot). | Mechanism SOUND — NO bbox crop. Validate: lean on `max_changed_fraction=0.02` as the binding gate for GROSS changes (the M1 purpose — layout breaks ≫2%); ADD a small-but-real content-change reject test to pin + DOCUMENT the guaranteed-catch floor; confirm the 8/0.02/0.05 values empirically (tighten perceptual_mean, don't crop, if sub-2% must be caught). |

### MUTATION KILL MAP → carry to VALIDATE (the critic's corrected fixtures)
- **`titlebar_band`** (viable: `*→+`/`*→/` on `y*w`, `+→-`/`+→*` on `+x`, `<→==`/`<→>`/`<→<=` on `y<band_px`):
  `let m = RegionMask::titlebar_band(3, 4, 2);` (w≠h, multi-row, band 0<2<4) → assert `m.contains_px(0,0)
  && m.contains_px(2,0) && m.contains_px(1,1)` (rows 0,1 inside) AND `!m.contains_px(0,2) && !m.contains_px(2,2)
  && !m.contains_px(1,3)` (rows 2,3 outside — kills `<→<=`; the index collapse + underflow kill the rest).
- **`mask_out`** (viable: `*→+`/`*→/` on `y*w` + on `*4`, `+→-`/`+→*` on `+x`, `+→-`/`+→*` on `i+4`):
  `let orig = solid(2,3,[10,20,30,40]); let mask = RegionMask::titlebar_band(2,3,2); let out =
  mask_out(&orig, &mask, [99,99,99,99]);` → `assert_eq!(&out.rgba[0..16], &[99u8;16])` (rows 0-1 incl (1,1)
  filled) AND `assert_eq!(&out.rgba[16..24], &[10,20,30,40,10,20,30,40])` (row 2 untouched) AND
  `assert_eq!((out.w,out.h),(2,3))`. NON-ORIGIN masked pixel (1,1), w≥2, unmasked row 2 remains.
- **`gate15_text`** (0 viable mutants; line-cov + value proof, observably ≠ gate15_default):
  base=solid(10,10,[60,60,60,255]); jitter=base with rgba[0]=66 (ΔR=6: >default's 4, ≤text's 8) →
  `compare(base,jitter,gate15_default())==Mismatch` (default REJECTS) AND `compare(base,jitter,gate15_text())==Match`
  (text ACCEPTS); content=base with 40 px changed by +160 → `compare(base,content,gate15_text())==Mismatch`
  (text still REJECTS gross). PLUS finding-2's SMALL-real-change reject test.
- **AC-3 masked compare**: a big diff confined to a masked region → mask_out both → `compare` Match; the
  SAME diff unmasked → Mismatch.
- **`assert_matches_baseline_masked`** (viable: `==→!=` on the approve check; + full line-cov): 6 tests
  mirroring the `_in` suite (tempdir/put_baseline/ApproveGuard/#[serial]/solid): masked-Pass (capture
  differs ONLY in the titlebar band → Pass, proves band excluded); Missing→FailWritePending
  `should_panic("wrote pending")` (kills `==→!=`); content-Mismatch→FailWritePending; corrupt-read
  `should_panic("baseline read failed")`; approve→WriteBaseline saves UNMASKED (re-load == self.img);
  approve+unwritable→`should_panic("baseline write failed")`.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-06-29)

**Tests written** (per the inspect kill map, fixtures verbatim): in-crate `#[cfg(test)]` — image_diff:
`titlebar_band_boundary_and_index` (3×4 band-2, w≠h multi-row), `mask_out_fills_inside_only`
(NON-ORIGIN solid(2,3)+band(2,3,2), masked rows==fill + row2==original), `gate15_text` both-sides
(ΔR=6 jitter: default Mismatch + text Match; gross +160 content: text Mismatch) + the small-real-change
reject (the documented catch-floor: **>2% of pixels = the binding `max_changed_fraction=0.02` gate**),
AC-3 masked-compare (band diff masked→Match, unmasked→Mismatch); capture: 6 `assert_matches_baseline_masked`
tests mirroring the `_in` suite (Pass / Missing→pending / content-Mismatch→pending / corrupt-read /
approve-writes-UNMASKED / write-failure), reusing tempdir/put_baseline/ApproveGuard/#[serial].
`tests/headed_text.rs` — the `#[ignore]` headed proof.

**Results:** `cargo nextest -p marley_visual_harness` = **158 passed, 2 skipped** (the #[ignore] headed
tests). Focused mutation (`--file image_diff.rs --file capture.rs`) = **63 caught / 0 missed / 15
unviable** (the `Default::default()` returns unviable as predicted → gate15_text 0 viable). fmt + clippy
clean.

**Headed proof + empirical gate15_text calibration:** `headed_text.rs` (reusing the existing
TwoElementFixture / marley_harness_selftest / harness_selftest.png via the masked+text path) passed
**3/3 deterministically** — the titlebar focus is masked out, the text content matches under
gate15_text(8/0.02/0.05). The values are validated for real text; no new baseline committed.

**FULL gate (008):** `GATE GREEN [diff]` (21:10:00) — **15 passed, 0 failed**. Coverage **100% lines**
whole-workspace; mutation of the harness change (`--in-diff`) **16 caught / 0 missed → MSI 100.0%** (the
new pure fns; `Default::default()` mutants unviable). Commit receipt written (41 b).

**Phase 4 status:** PASS.
