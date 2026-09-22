---
pipeline_id: 3fa42ab7-2aa9-452e-be33-6cb74201d373
ticket: forge#230 (7707f0be-d9d6-43d1-a0c3-a765bc8bd79f) · local docs/planning/tickets/open/TICKET-230-chrome-type-scale.md
aar_id: 11e662ad-a41d-4597-90b7-e18fab7885bf
status: Phase 5 — Complete PASS
title: Calibrate sidebar/files text to Warp size via type_scale (folds #223)
type: chore
milestone: M13
references: []
---

## Title
The files/sidebar chrome text renders at ~13px (too big vs Warp's ~11-12px, chad feedback #2), and ~20
hardcoded `text_size(px(9|11|12|13))` literals scatter the chrome so there's no single size source. Extend
the pure `type_scale` Role table with Warp-calibrated chrome bands and route the chrome literals through
them — calibrating the sidebar/files text DOWN in the process (folds the out-of-range #223).

## Scope
### In
- `marley_app/src/typography.rs` (PURE) — extend `Role` with chrome bands (design decides the set +
  Warp-calibrated sizes); full-value exact-pin tests per new Role.
- `marley_app/src/app.rs` (SHIM) — route the ~20 hardcoded chrome `text_size(px(N))` literals through
  `type_scale(Role::X)`; the sidebar/files literals map to a smaller (Warp) Role.
- `marley_app/src/workspace.rs` — refresh the stale `14.0` in `fallback_cell`'s non-positive-input guard
  default (per #223).

### Out (explicitly deferred)
- The terminal `Output`/`Command`/`Caption` role VALUES (already #195-calibrated; unchanged).
- The terminal font family/metric (#34), `TERMINAL_FONT_SIZE`, and any non-chrome literal.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — the Role taxonomy + exact Warp-calibrated sizes are DESIGN's call** (candidates Body/Label/Meta/Header
  or similar). Keep `Command`/`Output`/`Caption` unchanged. The chrome Roles MAY go below 12px (they're chrome,
  not the terminal body).
- **D2 — full-value exact-pin tests per new Role** (size AND weight — `TextStyle` has no `Default` → `type_scale`
  yields no viable mutant, so assert the full value; mirror `type_scale_by_role`).
- **D3 — no behavior change beyond the calibrated sizes** — the shim re-sources each size through a Role; only
  the sidebar/files (and any other over-large chrome) actually shrinks to the Warp value.
- **D4 — clean-room §20** — observe Warp's sidebar/session-list text size, pick OUR values (no copied hex/pt).
- **D5 — env-considerate driven capture** — chad is actively on a shared desktop; the calibrated VALUES are
  unit-proven (cov/MSI 100) + observe-Warp-referenced; offer chad the visual vibe-check when his screen's free.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the sidebar/files chrome renders, its text shall be sized via a `type_scale` Role calibrated to Warp (~11-12px, down from 13). | typography.rs unit (the Role's size); driven/env-considerate → mechanism + chad vibe-check |
| REQ-002 | WHERE a chrome text element used a hardcoded `text_size(px(N))` literal (the routed set), it shall route through `type_scale(Role)` instead. | grep (no stray hardcoded chrome literal in the routed set); code review |
| REQ-003 | The extended `type_scale` shall return the exact calibrated `{size, weight}` for each Role. | typography.rs full-value unit (cov/MSI 100) |
| REQ-004 | The terminal `Output`/`Command` roles shall remain ≥12px (the legibility floor). | the existing `type_scale_terminal_text_clears_legibility_floor` test stays green |
| REQ-005 | The `workspace.rs::fallback_cell` non-positive-input guard default shall be refreshed from the stale `14.0`. | the `fallback_cell` unit test |

## Phase Plan
- **P2 Design** — decide the Role set + Warp-calibrated sizes (observe the deskcheck Warp captures); map each of
  the ~20 chrome literals to a Role; confirm the scope is one slice (or split); `cargo mutants --list -f
  typography.rs`; the regression test plan.
- **P3 Implement** — extend typography.rs; route the app.rs literals; refresh fallback_cell.
- **P3.5 Inspect** — critics: the calibration values (legibility), the routing completeness (no stray literal /
  no wrong-Role mapping), the mutant coverage, clean-room.
- **P4 Validate** — the type_scale full-value + floor tests + fallback_cell; gate green [diff] cov/MSI 100 on the
  pure seams; driven-or-env-considerate capture.
- **P5 Complete** — CHANGELOG + app_shell.md / the type-scale doc; AAR; close #230 (+ note #223 folded); archive.
