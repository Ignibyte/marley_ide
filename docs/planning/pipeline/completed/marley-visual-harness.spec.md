---
pipeline_id: 61793ca7-bd8c-4e0a-b31b-b8c844d0706c
ticket: forge#10 (023b94b2-ca9a-486f-bc3d-86d228fdc980) · local docs/planning/tickets/open/TICKET-007-visual-harness.md
aar_id: 36c21c8d-5478-4516-a547-0e2224142ffa
status: Phase 5 — Complete PASS
title: marley_visual_harness — the fail-closed gate-15 contract (AXUIElement element-tree + screenshot baseline)
type: feature
milestone: M0
references:
  - ../../../pipeline/visual-testing.spec.md
  - ../../../specs/standards/quality-bar.spec.md
---

## Title

TICKET-007 — `marley_visual_harness`. The **single normative target for quality-bar
gate 15** — the dev/test-support crate that launches the app headed, asserts the macOS
**AXUIElement** element tree, and diffs screenshots against approved baselines. Every UI
spec's `visual_acceptance` clause (foundation-spike, ui-components, app-shell, …) binds to
*this* crate's API. Contract: [`visual-testing.spec.md`](../../../pipeline/visual-testing.spec.md)
(R1–R27, adopted verbatim as the acceptance bar).

**Un-deferred** from M1 to *now* (harness-first, per chad's goal): this session's de-risk
proved headed GUI testing is feasible in this environment (Warp granted Accessibility +
Screen Recording → `screencapture` yields real 4K PNGs, osascript/`System Events` returns
live AX data). The harness is the long pole the foundation-spike (006) needs to assert its
visual_acceptance.

## Scope

### In (this ticket)
- **Pure decision surface** (100% cov + MSI 100, headless) — the geometry predicates
  `is_above`/`is_leading`/`contains`/`is_centered_in`/`covers` (R10–R14) + the `CENTER_EPS_PX`/
  `BOUNDS_EPS_PX` constants; `compare` (R17) + `state_delta` (R18) + `PixelDiff`/`diff` (R19);
  `evaluate_baseline` (R20–R22) + `resolve_approval` (R23/R24 — the fail-closed "no silent
  drift" core); the `AxSnapshot`/`AxQuery` in-memory model + `find`/`find_all`/`assert_count`
  cardinality (R8/R15) + the `assert_*` attribute/ordering methods (R9–R15); `baseline_path`/
  `pending_dir` layout (R20–R22).
- **Thin display-IO shim** (covered + mutation-killed by the headed self-test — NOT
  `mutants::skip`) — `mount`/`FixtureWindow`/`HeadedSession` headed gpui launch (R1–R3, R5/R6);
  `capture()` framebuffer grab (R16); the AXUIElement snapshot build (R7) via **osascript/
  `System Events`** (the de-risked, unsafe-free path), parsed by the pure layer; `Drop` teardown.
- **The headed self-test** — a `TwoElementFixture` (window "Harness Selftest", an upper +
  lower static-text) mounted, AX-asserted (`assert_above`), and screenshot-baseline-checked.
  This is the only place the shim executes under test; the de-risk proved this env runs it.
- **Gate wiring + deps** — allow **MPL-2.0** in `deny.toml` (+ justification; chad-approved;
  gpui's tree pulls cbindgen/dwrote/option-ext); confirm `visual_g` runs `cargo nextest -p
  marley_visual_harness` once the crate exists (skip-clean until a UI crate is built).
- **§21** — CHANGELOG + `docs/marley_architecture/marley_visual_harness.md`.

### Out (deferred — flagged for design to confirm, documented like 004b)
- **R27 `SyntheticInput`** (synthetic pointer move/enter/leave/press/release + `advance_clock`)
  — only ui-components hover/press/tooltip fixtures need it; it drives real synthetic events
  (display-bound, complex). Defer to when **ui-components** lands (M1) → a follow-up ticket.
- **R25 `cargo xtask visual review`/`approve` CLI** — the *logic* (`resolve_approval`,
  `evaluate_baseline`, `APPROVE_ENV`, pending/baseline paths) ships and is fully tested here;
  the thin xtask CLI wrapper is a follow-up (Marley has no xtask harness yet).
- Non-macOS snapshot capture; animation/transition assertions; cross-machine pixel-exactness
  (all per the spec's own Out-of-scope).

## Acceptance Criteria (EARS — adopt visual-testing.spec.md R1–R27)

The authoritative clauses are R1–R27 in [`visual-testing.spec.md`](../../../pipeline/visual-testing.spec.md);
this pipeline adopts them verbatim. The shippable subset this ticket must satisfy:

- **AC-1 (R10–R14)** — the system shall expose pure frame predicates such that `is_above(a,b)`
  is true iff `a.y+a.h ≤ b.y`; `is_leading` respects `ReadingDirection`; `contains` requires
  full inset on both axes; `is_centered_in`/`covers` hold within `CENTER_EPS_PX`/`BOUNDS_EPS_PX`
  — verified by unit tests at each boundary + `cargo mutants` MSI 100.
- **AC-2 (R17/R18/R19)** — `compare` shall classify a pixel as differing iff a channel |Δ| >
  `per_channel` and return `Match` iff fraction ≤ ceiling AND mean-perceptual ≤ ceiling
  (default 4/0.0/1.0); `state_delta` shall pass iff change is confined to `mask`+`channels`;
  `diff` shall report count/max-Δ/mean-perceptual/bbox — verified by unit tests + mutation.
- **AC-3 (R23/R24)** — `resolve_approval` shall yield `WriteBaseline` **only** when
  `approve == true` AND decision ≠ `Match`; every non-Match with `approve == false` shall
  resolve to `FailWritePending` (no baseline write) — verified by the full
  {Match,Mismatch,Missing}×{approve,deny} table + mutation (the security-critical fail-closed core).
- **AC-4 (R20–R22)** — `evaluate_baseline` + the baseline IO shall pass on a matching baseline
  (no write), fail-closed on mismatch (`.actual`/`.diff` to pending, committed baseline
  byte-unchanged), and fail-closed on missing (`.new` to pending) — verified over a `tempfile` dir.
- **AC-5 (R8/R9/R15)** — `AxQuery::find` shall return exactly one match (fail closed on 0 or >1
  with a candidate list); `assert_*` shall pass on equality else fail with expected-vs-actual;
  `assert_count(by,n)` shall be exact — verified over hand-built `AxSnapshot` fixtures + mutation.
- **AC-6 (R1–R3, R6, R16 — headed self-test)** — `mount(TwoElementFixture)` shall open exactly
  one headed window titled "Harness Selftest" at the fixture size; `window(t)` shall resolve the
  unique window; `capture()` shall return an RGBA8 image at the window's pixel size; `Drop` shall
  tear down idempotently — verified by the headed self-test (real launch + screencapture + AX),
  which this environment runs.
- **AC-7 (R7 + spike #1)** — the AXUIElement snapshot shall report each captured node's
  role/title/value/size/frame; **design spike #1 decides** whether a gpui window exposes inner
  elements (→ `assert_above` over the AX tree) or only the window (→ inner content asserted via
  the screenshot baseline). Either way the harness ships and the self-test passes.
- **AC-8 (gate)** — FULL `scripts/gates.sh` → `GATE GREEN [full]` (15 gates): cov 100 / MSI 100
  on the pure surface, the headed self-test green under gate:3, gate:8 (deny) green with the
  MPL allow, gate:13 SAST clean (osascript path → no `unsafe` unless spike #1 forces FFI).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **AX access = osascript/`System Events`** (shell-out + parse), NOT objc2 FFI — the de-risked,
   `unsafe`-free path (no gate:6 miri burden, no gate:13 SAFETY surface). FFI fallback
   (objc2-application-services / accessibility-sys) only if spike #1 proves a concrete gap.
2. **No `mutants::skip`, no exclusions (§0).** The display-IO shim is kept thin (one OS call per
   fn, zero decision logic) and behind injectable seams; its coverage AND its mutants are killed
   by the **headed self-test** (which runs under both `cargo llvm-cov nextest` and `cargo mutants`
   in this env). This supersedes the spec's `mutants::skip` ACCEPTED-UNTESTABLE note — the de-risk
   removed the "can't run it here" basis. **Design must confirm** the mutation run stays
   green+tractable (the headed self-test per mutant); if it proves intractable/flaky, design
   surfaces a scoped ACCEPTED-UNTESTABLE decision (documented, narrow), not a silent skip.
3. **Pure/shim seam** (the 004 pattern): every assertion operates on in-memory `AxSnapshot`/`Image`;
   the FFI/launch runs once per capture to populate them, then nothing display-bound is in the
   assertion path. Inject the OS-call seam (a trait/fn) so the pure layer tests with fixtures.
4. **MPL-2.0 allowed** in `deny.toml` (chad-approved charter amendment) — required for gpui's tree.
5. Deps (minimize + justify): `gpui` (the self-test fixture window + framebuffer + TestAppContext),
   `image` (PNG encode/decode), `image-compare` (SSIM perceptual; MIT — replaces AGPL `dssim`),
   `serde`/`serde_json` (parse osascript/AX output) or plist, `tempfile` (dev). Design finalizes.

## Phase Plan
- **P2 Design** — SPIKE #1 FIRST (minimal gpui window → launch → AX-query → does the tree have
  inner elements?). Then the module/file manifest, the pure/shim seam, the regression test plan
  (one row per R + the headed self-test set), the mutation map. Confirm the §0 mutation strategy
  (decision #2) is tractable. Confirm/finalize the deferral of R27 + the xtask CLI.
- **P3 Implement** — the pure surface + the shim + the self-test fixture + deny.toml MPL + Cargo.toml.
- **P3.5 Inspect** — adversarial critics (correctness of the predicates/fail-closed core; the
  osascript-parse robustness; clean-room; license/deny; the seam's mutation-readiness).
- **P4 Validate** — write the tests; FULL gate green (cov 100 / MSI 100 + headed self-test).
- **P5 Complete** — §21 (CHANGELOG + arch doc); capture knowledge; close #10; archive.
