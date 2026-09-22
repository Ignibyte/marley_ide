---
pipeline_id: 59b70493-44fe-484a-8c7e-71e0d187f0e0
ticket: forge#30 (1b3f9d1d-c871-40b7-9ebb-679612780807) · local docs/planning/tickets/open/TICKET-030-pty-resize-to-pane.md
aar_id: a50da8f0-5ae6-4828-b951-a46818017dfa
status: Phase 5 — Complete PASS
title: PTY resize-to-pane — winsize follows the pane rect
type: feature
milestone: M1.D
references:
  - crates/marley_app/src/workspace.rs (Rect{x,y,w,h} + pane_rects; PaneState gains pty_size)
  - crates/terminal_blocks/src/session.rs (TerminalSession::resize — R17, ships)
  - crates/marley_app/src/app.rs (the shim: cell metrics + per-pane resize call)
  - docs/specs/SPEC-app-shell.spec.md (gains the pane-size clause)
---

## Title
Size each pane's PTY to its rect instead of a fixed 80×24. Today every pane spawns at
`settings.terminal.cols/rows` (80×24) and NEVER resizes, so output wraps at 80 columns in a wide
pane and a window resize doesn't reflow. Add a PURE `plan_resize(rect, cell, current)` that maps a
pane's `Rect` ÷ the monospace cell size to `(cols, rows)` (clamped ≥1, guarded), folding the
"unchanged" check into an `Option` so a resize fires only when the grid actually changes; the shim
reads the gpui cell metrics + `pane_rects` and calls `TerminalSession::resize` (R17) per changed
pane.

## Scope
### In
- NEW pure sizing in workspace.rs (beside `Rect`/`pane_rects`):
  - `CellSize { w: f32, h: f32 }` — the monospace advance width + line height (from gpui, passed in).
  - `plan_resize(rect: Rect, cell: CellSize, current: (u16, u16)) -> Option<(u16, u16)>` — compute
    `cols = floor(rect.w / cell.w)`, `rows = floor(rect.h / cell.h)`, each clamped ≥1 and guarded
    (a non-positive or non-finite extent/cell → 1); return `Some(new)` only when `new != current`,
    else `None` (the no-op guard).
- `crates/marley_app/src/workspace.rs` — `PaneState` gains `pty_size: (u16, u16)` (init to the spawn
  cols/rows); travels with the pane.
- `crates/marley_app/src/app.rs` (shim) — in render/on-resize: for each pane, compute its `Rect`
  via `pane_rects(group, window_bounds)`, read the gpui monospace `CellSize` (advance × line
  height), and `if let Some((c, r)) = plan_resize(rect, cell, state.pty_size) { state.session
  .resize(c, r); state.pty_size = (c, r); }`. Settings `terminal.cols/rows` stay the initial spawn
  size (fallback before the first layout).
- SPEC-app-shell: the pane-size clause + AC/Test-Plan/Mutation-Targets. CHANGELOG + arch doc.

### Out (explicitly deferred)
- Scrollback viewport is seq-5; ANSI color seq-4; raw-mode grid render seq-6. Debounce beyond the
  unchanged-guard (a resize storm during a live drag is acceptable — the guard already elides
  same-size frames). Font-size / zoom changing the cell metric at runtime (no zoom yet).
- **Monospace font + accurate cell metric (deferred — inspect #3):** the cell metric currently
  reads the ambient (proportional) window font because no monospace font is set yet; the resize
  plumbing is correct but the metric's fidelity is off. Tracked in
  `docs/planning/intake/terminal-monospace-font-and-cell-metric.md`; lands with the terminal-font
  work (likely #31/#33 or an M1.E font ticket).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — PURE `plan_resize` (cov 100/MSI 100); the shim only reads gpui metrics + calls `resize`.
  The unchanged-guard is FOLDED INTO the returned `Option` so it's part of the tested surface (a
  mutation target), not an untestable shim `!=`.
- D2 — Clamp each axis ≥1; guard a non-positive OR non-finite extent/cell → 1 (a zero cell width
  from an unmeasured font, or a zero-area rect during first layout, must not divide-by-zero or
  saturate to 65535).
- D3 — `floor` divide (a partial cell doesn't count as a column) — matches how terminals size.
- D4 — `pty_size` lives on `PaneState` (per-pane last-applied size), like `history` (#29). Init to
  the spawn size so the first real layout only resizes if the pane differs from 80×24.
- D5 — Settings `terminal.cols/rows` remain the INITIAL spawn size (not dropped) — a sensible
  pre-layout default + the fallback if metrics are unavailable.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `plan_resize` is given a rect + cell with a changed grid, it shall return `Some((cols, rows))` where `cols = floor(rect.w / cell.w)` and `rows = floor(rect.h / cell.h)`. | unit tests (exact + floor; a NON-SQUARE rect+cell so a cols/rows swap is caught) |
| REQ-002 | WHERE an axis computes below 1 (extent < cell) OR the extent/cell is non-positive or non-finite, `plan_resize` shall clamp that axis to 1 (never 0, never a saturated value). | unit tests (sub-cell → 1; zero extent → 1; zero cell → 1; NaN/inf → 1) |
| REQ-003 | WHEN the computed `(cols, rows)` equals `current`, `plan_resize` shall return `None` (no resize); WHEN it differs, `Some(new)`. | unit tests (equal → None; differ → Some) |
| REQ-004 | WHEN `scripts/gates.sh --diff` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `plan_resize`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact `plan_resize`/`grid_axis` shape + guards, `CellSize`, the `PaneState`
  field, the shim's metric read + call site, the SPEC clause + mutation targets.
- **P3 Implement** — workspace.rs sizing + PaneState field + app.rs wiring + spec + CHANGELOG.
- **P3.5 Inspect** — critics: the divide/floor/clamp/guard arithmetic, the axis mapping, the
  unchanged-guard, the u16 cast saturation.
- **P4 Validate** — the unit tests + gate GREEN [--diff].
- **P5 Complete** — docs, AAR, archive, close #30.
