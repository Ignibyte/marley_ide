---
pipeline_id: 8529a539-8065-4bca-aadc-53d0fc0eec2d
ticket: forge#165 (fd1caabd-bb28-4b40-9eeb-be5ba4468e12) · local docs/planning/tickets/open/TICKET-165-code-scroll.md
aar_id: f2f0f603-6b77-4a5f-a354-28b885e3a7fb
status: Phase 5 — Complete PASS
title: M10 — code tab wheel-scroll (the #154 regression fix)
type: bug
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/code_view.rs (PURE: scroll_code clamp + tests)
  - crates/marley_app/src/app.rs (SHIM: code_scroll_remainder + the wheel handler on the code body)
  - scripts/selftest/drive.swift (a scrollat verb)
---

## Title
A full-screen code tab can scroll again — the #154 render move left `cv.scroll` read-only, so files longer
than the 40-row window were unreachable past the fold.

## Scope
### In
- PURE `code_view.rs`: `scroll_code(scroll, steps, height, total) -> usize` — clamp to
  `[0, total.saturating_sub(height)]` (a short file pins to 0).
- SHIM `app.rs`: a `code_scroll_remainder: f32` field (sub-row trackpad deltas accumulate via the existing
  `scroll_steps`; app-level because `CodeViewState` derives `Eq`); the center code-branch wrapper gains
  `.on_scroll_wheel` → steps → `scroll_code` → write through `code_view_mut` + notify.
- Harness: a `scrollat:fx,fy,clicks` drive verb (CGEvent scrollWheel).

### Out
- Scrollbars; keyboard paging in the viewer; per-tab remainders (one active code tab scrolls at a time).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — wheel sign matches the terminal: a positive pixel delta (toward older) DECREASES `scroll`; negative
  advances. Same `fallback_cell(TERMINAL_FONT_SIZE).h` row height the lines draw with.
- D2 — the remainder lives on RootView (Eq on CodeViewState precludes an f32; correctness holds — only the
  active tab receives wheel events).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `scroll_code` shall clamp: down within range advances; past-bottom pins to `total-height`; up past 0 pins to 0; a file shorter than `height` always yields 0; zero steps is identity. | unit |
| REQ-002 (visual) | WHEN the wheel scrolls down over a >40-line code tab, later lines shall render (the gutter advances); wheeling far up shall pin the gutter back to 1. | driven capture |
| REQ-003 | gate GREEN; scroll_code at cov/MSI 100; the handler masked. | gate |

## Phase Plan
P2 folded above. P3 implement. P3.5 self-review + gate:5 (small pure fn) with the sign convention double-checked
against the terminal handler. P4 tests + the scrollat-driven capture + gate. P5 docs/AAR/archive.
