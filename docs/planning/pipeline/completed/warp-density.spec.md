---
pipeline_id: 91dd49f0-f72b-4da5-89f6-a1cdd7680b64
ticket: forge#216 (54be0f67-0332-41e5-a9d6-31268f1ce23c) · local docs/planning/tickets/open/TICKET-216-warp-density.md
aar_id: 2a540c15-0767-4908-af34-f015a9c9f458
status: Phase 5 — Complete PASS
title: Warp visual parity — spacing & density calibration
type: feature
milestone: M12.2
references: []
---

## Title
Calibrate Marley's load-bearing spacing/density drivers toward Warp's compact
density (chad: "match warp"). The spacing half of the Warp-look review; colors
(#194) + fonts (#195) shipped.

## Scope
### In
- The load-bearing density DRIVERS (the handful that most define the feel):
  `PANE_TITLE_H`, the dock/panel header padding (`caption_header` px_3/py_2), the
  rail/sidebar + file-tree row density, `STATUS_BAR_H`, `TOP_BAR_H`, `DOCK_WIDTH` —
  as the DESIGN measurement finds them differing from Warp.
- Measured against the captured Warp reference (`195-warp-ref.png`) + current Marley.

### Out (explicitly deferred)
- A total spacing rewrite (every `px_`/`gap_` call). BOUNDED to the drivers that matter.
- Routing chrome sizes through `type_scale` (#223).
- Block styling (#217), sidebar row STYLING beyond density (#219) — separate.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — BOUNDED: tune only the density drivers that genuinely differ from Warp;
  if a surface already matches within tolerance, LEAVE it (no manufactured churn).
- **D2** — measure vs the captured Warp reference; post-#195 (13pt) the chrome was
  sized for 14pt text and can tighten ~1–2px.
- **D3 — pure seam:** any density value that's a pure layout constant/ratio is pinned
  by an exact-value assert (f32 consts aren't mutated); the render `px_`/`py_` are shim.
- **D4** — auto-approved (/work 195–222): make defensible measured calls, document with
  the Warp-vs-Marley captures; batch the push.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The load-bearing density drivers shall be calibrated toward Warp's compact density (or confirmed already-matching within tolerance). | Warp-vs-Marley driven capture + exact-value assert on any changed pure const |
| REQ-002 | The calibration shall not break layout (no clipped/overlapping chrome; the terminal + panes still render fully). | Driven capture (the shell renders intact) |
| REQ-003 | Any changed pure layout constant shall be Marley-original (measured, not copied). | Review (§20) |

## Phase Plan
- **P2 Design** — capture current Marley; measure vs Warp; identify the concrete
  density deltas (the specific consts/px_ to change); manifest + test plan.
- **P3 Implement** — apply the measured density values.
- **P3.5 Inspect** — critic: no layout break, no over-tightening below legibility/touch,
  clean-room.
- **P4 Validate** — exact-value asserts on changed consts; gate; driven capture next to Warp.
- **P5 Complete** — CHANGELOG + doc; AAR; close.
