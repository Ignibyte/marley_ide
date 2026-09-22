---
pipeline_id: 5fc35972-2c52-4b83-9205-db698b76b18d
ticket: forge#220 (6172b7fc-2430-425b-af71-3b0b7cea493d) · local docs/planning/tickets/open/TICKET-220-warp-cursor-styling.md
aar_id: 79dfdb78-0b12-4a7f-af7c-653ba2c48791
status: Phase 5 — Complete PASS (Plan · Design · Implement · Inspect · Validate all PASS; SPLIT capture confirms REQ-001/002, selection kept REQ-003)
title: Warp visual parity — focused-vs-unfocused block cursor (+ selection tint evaluate)
type: feature
milestone: M12.2
references: []
---

## Title
Match Warp's terminal cursor focus behavior + assess the selection tint. NOTE the ticket
description is STALE ("the caret is a 2px accent bar") — #218 already replaced the prompt caret
with a solid block cursor, so the block SHAPE is done. The genuine remaining deltas (confirmed at
discovery — app.rs cursor render ~4655 + `selection_bg` ~4357; `is_focused` at ~4191):
Marley's block cursor is ALWAYS solid accent, even on an unfocused pane — so in a split, an
unfocused pane's cursor looks identical to the focused one. Warp draws a solid filled block on the
FOCUSED pane and a HOLLOW outline on unfocused panes. The drag-selection tint is `accent @ 0.3`.

## Scope
### In
- **D-A (primary)** — make the prompt block cursor FOCUS-AWARE (app.rs ~4655, inside the pane loop
  where `is_focused` is in scope): FOCUSED → the current solid filled block (`bg(accent)`, caret char
  reverse-video `fg=background`); UNFOCUSED → a HOLLOW block (transparent bg + a 1px `accent` border,
  the caret char in its normal `foreground`).
- **D-B (evaluate → calibrate-or-keep)** — the drag-selection tint (`accent @ 0.3`). Capture a real
  drag-selection, compare to Warp's neutral selection. KEEP + document if acceptable; else ONE bounded
  calibration via a pure `selection_tint(colors) -> Hsla` helper (exact-value tested).

### Out (explicitly deferred)
- The block SHAPE / caret grapheme isolation (done in #218).
- Cursor BLINK (Warp's cursor blinks; Marley's is static — a timer-driven follow-up, not a parity
  blocker at rest).
- The find-match tint (`success @ 0.35`) — unrelated, unchanged.
- The M13 editor caret (#208) — a separate ticket (this composes with it later).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — the unfocused cursor is HOLLOW (a 1px `accent` outline, no fill), the classic terminal
  unfocused-cursor look; the caret char shows in `foreground` (not reverse-video — no fill behind it).
- **D2** — reuse the existing `is_focused` (`pane_id == focused`, #191) at the cursor render site — no
  new focus plumbing.
- **D3** — D-A is shim-only (a focus branch on the cursor render inside the `mutants::skip` render),
  validated by a driven SPLIT capture (focused solid, unfocused hollow) — like #217's precedent. D-B
  gets a pure helper + exact-value test ONLY if it calibrates.
- **D4** — tokens only (`accent`/`background`/`border`/`foreground`) — no new hardcoded hsla beyond an
  alpha on an existing token; clean-room §20.
- **D5** — auto-approved (/work 195–222): document with the Warp-vs-Marley captures.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a terminal pane is focused, its prompt block cursor shall render as a solid filled accent block (the caret char reverse-videoed). | Driven capture (focused pane) + review |
| REQ-002 | WHEN a terminal pane is NOT focused, its prompt block cursor shall render as a hollow accent outline with no fill (the caret char in the normal foreground color). | Driven capture (a SPLIT — focused solid, unfocused hollow) + review |
| REQ-003 | The drag-selection tint shall read as a Warp-like selection over output text (evaluated against Warp; calibrated or kept with rationale). | Driven capture (a real drag-selection) + review |
| REQ-004 | IF the selection tint is calibrated, its value shall come from a pure helper returning a tokens-derived tint (exact-value, cov/MSI 100). | Unit test (only if D-B calibrates; else N/A + documented) |
| REQ-005 | The change shall not alter the caret POSITION, the focused-cursor look, the labels, or the find-match tint. | Review + capture |

## Phase Plan
- **P2 Design** — confirm the gpui hollow-block idiom (`border_1` + `border_color` + transparent bg on
  the cursor div; the char color branch); capture + judge the selection tint (calibrate-or-keep); the
  exact render change; file manifest + test plan.
- **P3 Implement** — the focus branch on the cursor render (+ the pure `selection_tint` if calibrating).
- **P3.5 Inspect** — critics: the hollow cursor doesn't break the #88 gapless caret line / the caret
  position; the focused look is unchanged; no regression to alt-screen / the find tint; clean-room.
- **P4 Validate** — driven SPLIT capture (focused solid / unfocused hollow) + selection capture; the
  `selection_tint` unit test if calibrated; gate.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close.
