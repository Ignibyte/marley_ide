---
pipeline_id: f12c94d4-dd92-4454-b7fa-99b918a66ab3
ticket: forge#231 (e8e48112-9bf6-420a-a832-920018691593) · local docs/planning/tickets/open/TICKET-231-warp-darker-surface.md
aar_id: ae52900d-1266-4d1e-96a3-ae2b73faf7f6
status: Phase 5 — Complete PASS
title: Darker, Warp-like gray for panel/dock surfaces
type: chore
milestone: M13
references: []
---

## Title
The dark theme's `surface` (docks / sidebar / Files / panels / cards / inputs / overlays) is `hsla(0.62,
0.09, 0.155)` — a mid gray that reads lighter than Warp's panels. Darken it to a Warp-like darker gray
(chad feedback #3). A single pure theme-value change in `ui_components/lib.rs` re-tones every surface
consistently.

## Scope
### In
- `crates/ui_components/src/lib.rs` (PURE) — lower the DARK theme `surface` lightness (0.155 → the design's
  Warp-calibrated ~0.10-0.11); update its exact-value pin test + a contrast assertion.

### Out (explicitly deferred)
- The LIGHT theme `surface` (L=0.93) — "darker gray" is a dark-mode ask; unchanged.
- Every other role (background/foreground/accent/border/…), unless design finds the border needs a nudge.
- Any new role (splitting docks-vs-overlays surfaces — over-reads #3).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — darken the DARK `surface` lightness** from 0.155 to a Warp-calibrated darker gray (design picks the
  exact L, ~0.10-0.11; observe Warp, pick OUR value — clean-room §20).
- **D2 — leave the LIGHT `surface` unchanged** ("darker" doesn't map to light mode).
- **D3 — surface is a SHARED role** → the darken re-tones ALL surfaces (docks/sidebar/Files + cards/inputs/
  palette/menu/launcher overlays) consistently. That's the intent (chad said panels/docks; they share `surface`).
- **D4 — keep surface DISTINCT from background** (0.05) — a visible raised-surface delta — and **border** (0.26)
  above it. Design confirms border needs no change.
- **D5 — legibility only improves** — darkening `surface` raises the contrast for the light foreground/muted text
  on it; asserted via `contrast_ratio`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The dark theme `surface` shall be a darker gray — lower lightness than the current 0.155 (Warp-calibrated). | ui_components exact-value unit (the new surface `hsla`) |
| REQ-002 | The dark `surface` shall remain distinct from `background` (a visible raised-surface delta). | the existing `surface != background` test |
| REQ-003 | The dark `foreground` and `muted` text shall clear the WCAG contrast floor on the darker `surface`. | a `contrast_ratio(foreground/muted, surface)` assertion ≥ the floor |
| REQ-004 | The `border` shall stay visible against the darker `surface` (border lightness > surface lightness). | unit (`border.l > surface.l`) |

## Phase Plan
- **P2 Design** — pick the exact Warp-calibrated dark `surface` L (observe the deskcheck Warp captures); confirm
  the distinct-from-bg/border deltas + the contrast floor; the exact-value + contrast test updates; `cargo mutants
  --list -f lib.rs` (the theme fns' viable set).
- **P3 Implement** — the one `surface` value change + the test updates.
- **P3.5 Inspect** — critic: the value, the deltas, the contrast, cov/MSI, clean-room.
- **P4 Validate** — the exact-value + contrast tests; gate green [diff] cov/MSI 100; driven-or-env-considerate capture.
- **P5 Complete** — CHANGELOG + the theme doc; AAR; close #231; archive.
