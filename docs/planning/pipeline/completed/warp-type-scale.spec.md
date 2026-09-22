---
pipeline_id: 4ec7481e-4edf-4dd7-a619-01a8ca5af9f3
ticket: forge#195 (9377f953-94ed-41de-80af-307095b55bb8) · local docs/planning/tickets/open/TICKET-195-warp-type-scale.md
aar_id: 0ef796a3-e28e-4b95-bb78-283b96c83c5e
status: Phase 5 — Complete PASS
title: Warp visual parity — font-size calibration (type scale)
type: feature
milestone: M12.2
references: []
---

## Title
Calibrate Marley's type scale to MATCH Warp's rendered font proportions/density
(chad: "match warp is the end goal"). The FONT half of the Warp-look review; the
color half shipped in #194.

## Scope
### In
- `marley_app/src/typography.rs` `type_scale(Role)` — the Command / Output / Caption
  sizes (+ weights) recalibrated toward Warp's proportions.
- `marley_app/src/app.rs` `TERMINAL_FONT_SIZE` — the terminal cell text size. The
  monospace cell metric DERIVES from this (`fallback_cell(TERMINAL_FONT_SIZE)` / the
  em_advance read), so it auto-tracks a size change (columns stay aligned).
- A captured **Warp reference** (Warp.app is the host terminal) + current Marley, to
  measure the target proportions rather than guess.

### Out (explicitly deferred)
- The MONO FONT choice (Menlo shipped in #34 — unchanged; clean-room, no Warp font).
- Spacing/density (#216), and every other Warp-parity surface (#217–222) — separate.
- The light theme / colors (#194).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — recalibrate `type_scale` (Command/Output/Caption) + `TERMINAL_FONT_SIZE`;
  the cell metric is derived from `TERMINAL_FONT_SIZE`, so no separate metric edit.
- **D2** — measure against a CAPTURED Warp reference (screencapture Warp's host window,
  the same mechanism the selftest uses for Marley) — the enabler for genuinely matching
  vs guessing.
- **D3 — pure seam:** the `type_scale` table is guarded by EXACT-value regression
  asserts (cargo-mutants doesn't mutate f32 size literals → a full-`TextStyle` assert is
  the typo guard, like #194's palette pins + the existing `type_scale_by_role`).
- **D4** — legibility floor: terminal text ≥ 12pt (don't shrink below readable).
- **D5** — auto-approved (`/work 195–222`): make a defensible Warp-matching call,
  document it with the Warp-vs-Marley captures; hold the push (batch).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The terminal text size + the terminal→command→caption size relationship shall be calibrated toward Warp's rendered proportions. | Warp-vs-Marley driven capture + exact-value `type_scale`/`TERMINAL_FONT_SIZE` assert |
| REQ-002 | WHEN the terminal text size changes, the monospace cell metric shall track it so output columns stay aligned. | Driven capture (`ls -l`/a table aligns) — the metric derives from `TERMINAL_FONT_SIZE` |
| REQ-003 | Terminal body text shall remain legible (≥ 12pt). | Unit assert on the size + capture |
| REQ-004 | The chosen sizes shall be Marley-original (observed, not copied from Warp). | Review (§20) |

## Phase Plan
- **P2 Design** — capture Warp (host) + current Marley; measure Warp's terminal +
  chrome text sizes/relationship; propose the exact `type_scale` + `TERMINAL_FONT_SIZE`
  values; the manifest + the test plan.
- **P3 Implement** — set the values in typography.rs + app.rs.
- **P3.5 Inspect** — critics: legibility floor, cell-metric consistency, no consumer
  reads a stale size, clean-room.
- **P4 Validate** — the exact-value asserts; the gate; driven capture of Marley (columns
  align, sizes read Warp-like) next to the Warp reference.
- **P5 Complete** — CHANGELOG + app_shell typography doc; AAR; close.
