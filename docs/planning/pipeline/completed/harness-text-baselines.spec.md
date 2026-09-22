---
pipeline_id: e7707db6-26b5-46c4-9bcb-725bf5fc4934
ticket: forge#12 (809865b2-1314-41d3-a888-3a4fd8ac4352) · local docs/planning/tickets/open/TICKET-008-harness-text.md
aar_id: b492f0f1-9df8-4ff2-bcc0-8bb1335c7631
sprint: M1.A — The Usable Terminal (aa46e22f) seq 1/5
status: Phase 5 — Complete PASS
title: marley_visual_harness — text-tolerant screenshot baselines (titlebar mask + text tolerance)
type: feature
milestone: M1
references:
  - ../../../marley_architecture/marley_visual_harness.md
  - ../../../marley_architecture/crate-map.md
---

## Title

TICKET-008 — extend `marley_visual_harness` (forge #10, the gate-15 harness) so screenshot baselines
work for **text-rendering** windows. The M1 enabler: every M1 UI crate (`ui_components`,
`terminal_blocks`, `editor`, `app_shell`) renders text, and the current `Tolerance::gate15_default`
was calibrated on the STATIC, text-free `TwoElementFixture` — a strict pixel-match flakes on (a) the
macOS titlebar focus state (traffic-light brightness frontmost-vs-not) and (b) gpui subpixel text AA
(proven: marley_spike's headed test failed 3/3 reruns at strict match). Closes
`AD-claude-headed-visual-baseline-text-tolerance-deferred-001`.

## Scope

### In
- **A titlebar/chrome `RegionMask`** — a constructor that excludes the top titlebar band (height in
  px, retina-aware) from the comparison, so the focus-variable traffic-lights can't fail a baseline.
  The region/exclusion math is PURE.
- **A text-tolerant `Tolerance`** — a new variant (e.g. `Tolerance::gate15_text()` or a builder) with
  a perceptual/SSIM threshold tuned for rendered text (accepts subpixel-AA jitter, rejects a real
  content change). `gate15_default` stays UNCHANGED; this is additive.
- **Mask-aware compare** — `compare`/`evaluate_baseline` honor a `RegionMask` (masked pixels excluded
  from both the per-pixel and the perceptual decision). PURE.
- **Validation:** a NEW gpui **text fixture** (a window rendering a known header + body line, parallel
  to `TwoElementFixture`/`selftest.rs`) + a `#[ignore]` headed self-test that captures it and matches
  its committed baseline (titlebar-mask + text tolerance) DETERMINISTICALLY across ≥3 runs.
- §21 — CHANGELOG + `docs/marley_architecture/marley_visual_harness.md`.

### Out / deferred
- No change to `gate15_default` or the existing `TwoElementFixture` self-test (stay green).
- No change to the AX path. No new OS dependencies. R27 synthetic-input still deferred.
- marley_spike is throwaway — NOT a dependency or a test target here.

## Acceptance Criteria (EARS)
- **AC-1 (titlebar mask)** — the harness *shall* provide a `RegionMask` constructor that excludes a
  top band of a given pixel height; a pixel inside the band that differs *shall not* affect the
  `compare`/`diff` outcome, while a differing pixel below the band *shall*. (Unit test.)
- **AC-2 (text tolerance)** — the harness *shall* provide a text-tolerant `Tolerance` distinct from
  `gate15_default`; under it, two images differing only by small sub-threshold (AA-scale) perturbation
  *shall* `Match`, and an image with a real content change *shall* `Differ`. (Unit test, synthesized
  images.)
- **AC-3 (mask-aware compare)** — `compare`/`evaluate_baseline` *shall* apply a supplied `RegionMask`
  so masked pixels are excluded from BOTH the per-pixel and the perceptual/SSIM decision. (Unit test:
  a large diff confined to the masked region → `Match`; the same diff unmasked → `Differ`.)
- **AC-4 (deterministic headed proof)** — a gpui text fixture launched headed and captured *shall*
  match its committed baseline under (titlebar-mask + text tolerance) on ≥3 consecutive runs with no
  approval. (`#[ignore]` headed self-test, run in the headed lane.)
- **AC-5 (gate)** — FULL `scripts/gates.sh` → `GATE GREEN [full/diff]`; the PURE mask/tolerance/
  exclusion logic at **cov 100 / MSI 100**; the gpui text-fixture + headed test ACCEPTED-UNTESTABLE
  (extend the existing `rust_cov` exclude + `mutants::skip`).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **Additive, not a rewrite** — keep `gate15_default` + the `TwoElementFixture` self-test untouched;
   ADD `gate15_text` + the titlebar `RegionMask` + mask-aware compare as new documented options.
2. **Pure/shim seam holds** — the mask region math, the text-tolerant threshold decision, and the
   masked-pixel exclusion in compare/diff are PURE (100/100). The gpui text fixture + the headed test
   are the ACCEPTED-UNTESTABLE shim (the existing `os_shim`/`launch`/`selftest` precedent + the
   documented `rust_cov` exclude + `mutants::skip`).
3. **Validate with a harness-owned text fixture**, NOT marley_spike (throwaway). Commit its baseline PNG.

## Phase Plan
- **P2 Design** — FIRST read `image_diff.rs` (Tolerance/ChannelSet/RegionMask/compare/diff/
  perceptual_distance), `baseline.rs` (evaluate_baseline/resolve_approval), `capture.rs`
  (assert_matches_baseline_in). Design the minimal additions + how compare threads the mask; the text
  fixture (selftest-style); the regression-test plan + mutation map (watch equivalent mutants on the
  region bounds — use the multi-row lesson).
- **P3 Implement** — the pure additions + the text fixture + the rust_cov/skip wiring.
- **P3.5 Inspect** — critics: the mask exclusion math (off-by-one on the band), the text threshold
  (does it still reject a real change?), the equivalent-mutant risk on region bounds.
- **P4 Validate** — unit tests (AC-1..3) + generate/commit the text baseline + the headed proof (AC-4);
  FULL gate.
- **P5 Complete** — §21; close #12; archive; → #13 ui_components.
