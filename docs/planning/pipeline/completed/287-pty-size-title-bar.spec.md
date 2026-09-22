---
pipeline_id: 58a2676a-b943-4ac5-85b1-83ab00827ea1
ticket: forge#287 (defe74a1-3684-4e49-83ca-7df32bc5d332) · local docs/planning/tickets/open/TICKET-287-pty-size-title-bar.md
aar_id: 4497d877-6c70-4a36-b822-ce994a5d5c3c
status: Phase 5 — Complete PASS
title: Size the PTY to the carved content rect (subtract the pane title bar)
type: bug
milestone: M17
references: []
---

## Title
Size a terminal pane's PTY grid to its CONTENT rect (pane height minus the 24px
title bar), not the full pane. The resize loop feeds `plan_resize` the full rect
while the render carves `PANE_TITLE_H` off the content, so the grid is ~1 row too
tall — the TUI's true top row clips off-screen and #280's mouse-row mapping skews.

## Scope
### In
- A pure `inset_top(rect, inset) -> Rect` in `workspace.rs` (mirrors `inset_right`): carve the TOP
  edge (`y += inset`, `h = (h - inset).max(0)`).
- `app.rs` resize loop: `plan_resize(inset_top(*rect, PANE_TITLE_H), cell, term.pty_size)`.

### Out (explicitly deferred)
- The render's content carving (`.top(r.y + PANE_TITLE_H).h(…)`) and `pane_mouse_cell` are
  UNCHANGED — they already trust `pty_size` correctly; fixing `pty_size` heals both consequences.
- Any change to `plan_resize`'s floor math or `PANE_TITLE_H`'s value.

## Reference (§20)
Warp (terminal). Observed behavior: a full-screen TUI (htop/vim/less) fills the
terminal's CONTENT area below the pane chrome — its top row is visible and mouse
clicks land on the right cell. Marley matches by sizing the PTY grid to the carved
content rect (the same `PANE_TITLE_H` the render already subtracts). Clean-room
§20: observed behavior only; the rect arithmetic is Marley's own; no Warp source
read. (`docs/warp_architecture/` covers the Blocks/cockpit model; this is the
standard "grid = content area" every terminal honors.)

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — `inset_top` carves the TOP edge** (`y += inset`, `h = (h - inset).max(0)`,
  `x`/`w` unchanged) — byte-for-byte the render's content carving at app.rs
  7285-7287. Mirrors the existing `inset_right` (right-edge gutter).
- **D2 — the inset is `PANE_TITLE_H`, passed by the app.** The const stays in
  `app.rs`; `workspace.rs` stays gpui-free and takes the inset as a param (exactly
  like `inset_right(rect, inset)`), so the title-bar height is single-sourced.
- **D3 — ONLY the resize loop changes.** The render + `pane_mouse_cell` already
  trust `pty_size`; correcting `pty_size` fixes BOTH the top-row clip AND the
  mouse-row skew with no render/mouse edits (the ticket's "one fix heals both").

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a pane's PTY is resized, the row count shall be `floor((pane.h - PANE_TITLE_H) / cell_h)` (the carved content height), not `floor(pane.h / cell_h)`. | `plan_resize` golden fed `inset_top(rect, PANE_TITLE_H)` vs the full rect (rows differ by ~1). |
| REQ-002 | `inset_top` shall reduce the height by `inset` and drop `y` by `inset` (clamped ≥0), leaving `x`/`w` unchanged. | `inset_top` unit truth-table (normal + `inset > h` clamp). |
| REQ-003 | WHEN an alt-screen TUI renders in a tall pane, its top row shall be visible (not clipped) and a top-row click shall map to the top grid row. | Driven `htop` capture (top status row visible + click → row 1) if the harness is reachable; else the mechanism (pty_size now equals the content grid the render + `pane_mouse_cell` consume). |
| REQ-004 | `inset_top` shall be a pure fn at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `inset_top` signature + the one resize-loop call change; the regression test plan.
- **P3 Implement** — the pure fn + the call-site + the import.
- **P3.5 Inspect** — inline: matches the render carving exactly? clamp? no other resize sites?
- **P4 Validate** — `inset_top` unit + the `plan_resize` carved-height golden; gate green [diff]; DRIVE htop if reachable.
- **P5 Complete** — CHANGELOG + workspace/app-shell doc, AAR, close #287.
