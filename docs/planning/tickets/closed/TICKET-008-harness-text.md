---
ticket: TICKET-008
forge: forge#12 (809865b2-1314-41d3-a888-3a4fd8ac4352)
status: closed
type: feature
milestone: M1
sprint: M1.A — The Usable Terminal (seq 1/5)
branch: ticket-008-harness-text
pipeline: docs/planning/pipeline/active/harness-text-baselines.spec.md
aar: b492f0f1-9df8-4ff2-bcc0-8bb1335c7631
---

# TICKET-008 — marley_visual_harness: text-tolerant screenshot baselines

The M1 ENABLER (sprint M1.A seq 1/5). Extend the gate-15 harness so screenshot baselines work for
**text-rendering** windows — every M1 UI crate renders text, and the current `gate15_default` (tuned on
the static text-free `TwoElementFixture`) flakes on macOS titlebar focus + gpui subpixel text AA
(proven in marley_spike's validate, 3/3 reruns). Closes
`AD-claude-headed-visual-baseline-text-tolerance-deferred-001`.

## Acceptance
- A titlebar `RegionMask` (exclude the focus-variable titlebar band) + a text-tolerant `Tolerance`
  (`gate15_text`) + mask-aware `compare`/`evaluate_baseline` — the PURE logic at **100% cov + MSI 100**.
- A harness-owned gpui **text fixture** + a `#[ignore]` headed self-test that matches its committed
  baseline DETERMINISTICALLY across ≥3 runs (the proof marley_spike couldn't do). ACCEPTED-UNTESTABLE
  shim (existing precedent).
- FULL `scripts/gates.sh` → `GATE GREEN`; §21 CHANGELOG + arch doc.

## Notes
Additive only — `gate15_default` + the existing self-test stay untouched. See the pipeline spec/notes
for the full plan + the carry-to-design (READ image_diff/baseline/capture first).
