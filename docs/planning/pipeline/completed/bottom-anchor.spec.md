---
pipeline_id: ae755ef1-b704-4ac6-bf11-d44c94533a4c
ticket: forge#49 (859c30cf-3c09-4d16-a3e2-fafd3d5ab4d5) · local docs/planning/tickets/open/TICKET-049-bottom-anchor.md
aar_id: caab5473-939a-44e8-9f49-01181c607f24
status: Phase 5 — Complete PASS
title: terminal is bottom-anchored like a normal terminal
type: bug
milestone: M1.H
references:
  - crates/marley_app/src/app.rs (the pane content column — shim)
  - docs/specs/SPEC-app-shell.spec.md (R39 amend)
---

## Title
BUG (chad, live testing; confirmed via window capture): the terminal pane is TOP-anchored — when the
content is shorter than the pane, the blocks + prompt pile at the TOP with empty space BELOW, and the
prompt/latest output isn't kept at the bottom. It doesn't feel like a normal terminal (Terminal/iTerm/
Warp are bottom-anchored). Fix: bottom-anchor the pane content.

## Scope
### In
- `crates/marley_app/src/app.rs` (SHIM): add `.justify_end()` to the pane content `flex_col` (~982) so
  its children (block headers + output rows + the prompt `input_row`) pack at the BOTTOM — the prompt
  at the bottom, output right above it, empty space (short content) at the TOP.
- SPEC-app-shell R39 note (the pane is bottom-anchored). CHANGELOG + arch doc.

### Out (explicitly deferred)
- Any change to the viewport `visible()` logic (already correct — following shows the bottom
  `capacity` rows). Tab-completion (the #33 shell-line-editing fork — a separate ticket). A `.app`
  bundle + Accessibility permission for full self-driving AX input (a separate follow-up; capture-only
  verify works now). Any scroll-behavior redesign beyond the anchor.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — SHIM-ONLY: the fix is a single gpui layout call (`justify_end`), no pure-logic change. The
  viewport model (#32) stays as-is. So this is a VISUAL-ACCEPTANCE ticket — there is no new pure
  surface for cov/MSI; the acceptance is the window capture + the static gates.
- D2 — The alt-screen grid branch is unaffected: a full-screen program fills `capacity` rows, so the
  column is full and `justify_end` has no visible effect there. The change only bottom-anchors the
  short cooked-view case.
- D3 — VERIFY VIA WINDOW CAPTURE (the step chad requested): rebuild → `open` → `screencapture
  -l<windowid>` the empty prompt → the prompt renders at the BOTTOM of the pane (before/after). This
  establishes capture-based visual verification in the pipeline. (Driving synthetic input still needs
  Accessibility permission + a `.app` bundle — deferred.)

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the pane renders the cooked block view with content shorter than the pane, the prompt shall appear at the BOTTOM of the pane with any empty space ABOVE it (not the prompt at the top with empty space below). | window capture (before/after) — the AX-verify step |
| REQ-002 | WHEN a full-screen (alt-screen) program is active, the grid shall still fill the pane (no regression from the anchor change). | build + reasoning (grid = capacity rows) |
| REQ-003 | WHEN a pane is bottom-anchored and a pointer is `local_row` rows below the pane top, `row_at(local_row, start, end, capacity)` shall return the content row under it — mapping the empty top band to the first visible row and visible row `k` to `start + k` — so click/drag selection still hits the right rows (inspect HIGH: the hit-test must mirror the anchor). | unit (full pane; short content; empty-band; clamp) |
| REQ-004 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN — the static gates + coverage/MSI 100 on the NEW pure `row_at`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — confirm the exact `justify_end` placement + that it doesn't disturb the alt-screen/
  prompt/overflow; the capture-verify method; the SPEC note.
- **P3 Implement** — the `.justify_end()` line + spec + CHANGELOG.
- **P3.5 Inspect** — critic: does `justify_end` bottom-anchor correctly without clipping the LATEST
  content when overflowing (following shows the bottom rows — must stay visible), any alt-screen/
  scroll regression, the `overflow_hidden` interaction.
- **P4 Validate** — rebuild + `open` + capture the window → confirm the prompt is at the bottom; run
  the gate GREEN.
- **P5 Complete** — docs, AAR, archive, close #49.
