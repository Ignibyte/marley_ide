---
ticket: TICKET-007
forge: forge#10 (023b94b2-ca9a-486f-bc3d-86d228fdc980)
status: closed
type: feature
milestone: M0
branch: ticket-007-visual-harness
pipeline: docs/planning/pipeline/active/marley-visual-harness.spec.md
spec: docs/pipeline/visual-testing.spec.md
aar: 36c21c8d-5478-4516-a547-0e2224142ffa
---

# TICKET-007 — marley_visual_harness (gate-15 harness)

Build the **single normative target for quality-bar gate 15** — the dev/test crate that
launches the app headed, asserts the macOS AXUIElement element tree, and diffs screenshots
against approved baselines. Contract: `docs/pipeline/visual-testing.spec.md` (R1–R27).

**Un-deferred from M1** (was forge#10 "lands with the first UI crate"): this session's de-risk
proved headed GUI works in this env (Warp has Accessibility + Screen Recording; screencapture +
osascript-AX both functional). Built harness-first per chad's goal, so 006 (foundation-spike) can
assert its visual_acceptance.

## Acceptance
- The full pure decision surface (frame predicates R10–R14, `compare`/`state_delta`/`diff`
  R17–R19, `evaluate_baseline`/`resolve_approval` R20–R24, the `AxSnapshot`/`AxQuery` model +
  `find`/`assert_*`/`assert_count` R7–R15) — **100% coverage + mutation MSI 100**, headless.
- The thin display-IO shim (`mount`/`HeadedSession`/`capture`/AX-snapshot via osascript, R1–R7/R16)
  covered + mutation-killed by a **real headed self-test** (`TwoElementFixture`) — NOT `mutants::skip`.
- `deny.toml` allows MPL-2.0 (gpui tree); FULL `scripts/gates.sh` → `GATE GREEN [full]` (15 gates).
- §21: CHANGELOG + `docs/marley_architecture/marley_visual_harness.md`.

## Deferred (follow-up tickets, documented)
- **R27 SyntheticInput** (synthetic pointer/clock) — only ui-components needs it (M1).
- **R25 `cargo xtask visual review/approve` CLI** — the logic ships + is tested; the CLI wrapper follows.

## Design spike (Phase 2, FIRST)
Does a gpui window expose inner elements in its AX tree (→ R9/R10 via `assert_above`) or only the
window (→ R9/R10 inner content via the screenshot baseline)? Decides AC-7.

See the pipeline spec + notes for the full plan, locked decisions, and the §0 mutation strategy.
