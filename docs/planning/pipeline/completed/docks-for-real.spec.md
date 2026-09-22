---
pipeline_id: 01f95a48-76f5-47fc-9fc2-9f7f1bc10884
ticket: forge#24 (0764b4e3-affc-45f5-ba73-8f2ae6c7e43f) · local docs/planning/tickets/open/TICKET-024-docks-for-real.md
aar_id: e928eb68-9ccf-4e72-81af-b764a01a3ddd
status: Phase 5 — Complete PASS
title: docks for real — 3-region render + toggle-dock actions
type: feature
milestone: M1.C
references:
  - docs/specs/SPEC-app-shell.spec.md (R4-R6 render side; R19 gains the two chords)
  - crates/marley_app/src/layout.rs (DockSide/DockState — gains the pure region-width fn)
  - crates/marley_app/src/workspace.rs (pane_rects — the center-bounds inset seam, #23)
---

## Title
Make R4–R6 real: `RootView` renders three horizontally-ordered regions — left dock | center
pane-group | right dock — each dock a themed titled panel (placeholder content; real panel
content = the M2 panel system), a Closed dock allocating ZERO width so the center reflows, and
`toggle_dock` finally CALLED via new keymap chords + palette commands. After #23 the docks were
the last render-inert cockpit state.

## Scope
### In
- `crates/marley_app/src/layout.rs` — a PURE `region_widths(window_w, left: DockState,
  right: DockState, dock_w) -> RegionWidths{left, center, right}` (Open → `dock_w`, Closed → 0.0,
  center = remainder) + the `RegionWidths` value type.
- `crates/marley_app/src/keymap.rs` — `default_bindings` gains `cmd-b → "toggle-left-dock"` and
  `cmd-shift-b → "toggle-right-dock"` (proposed; final chord = design call — must not collide
  with prompt-bound keys; cmd-chords never reach the PTY).
- `crates/marley_app/src/app.rs` (shim) — render the 3 regions (dock panels: themed `surface`
  background + a title label; center = the #23 pane tiling over the INSET bounds via
  `region_widths`); `dispatch_action` gains the two toggle arms calling `toggle_dock`;
  `cockpit_commands` gains "Toggle Left Dock"/"Toggle Right Dock" (palette Enter-dispatch is
  seq-4; the chord path works now).
- SPEC-app-shell: amend R19 (the two new default chords) + add the region-width clause (R31)
  with AC/Test-Plan/Mutation-Targets rows.
- CHANGELOG + architecture doc (§21).

### Out (explicitly deferred)
- Real dock content (project tree / agent panels — the M2 panel system).
- Palette Enter-dispatch of the new commands (seq-4); dock-state persistence (seq-5).
- Dock resize/drag; per-side widths; animation. Headed baselines: WRITTEN-pattern assertions
  ride the #23 deferral (this session has no WindowServer access — run in a desktop session).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — The width computation is PURE in `layout.rs` (owns `DockState`); the shim passes a
  `DOCK_WIDTH` const and converts to px. All 4 open/closed combinations unit-tested.
- D2 — Chords: `cmd-b` (left) / `cmd-shift-b` (right) unless design finds a collision.
- D3 — The center pane tiling consumes the SAME `region_widths` result that sizes the dock divs
  (one source of truth — no independent inset math in the render).
- D4 — Dock panels render placeholder title text only ("Files" left, "Details" right — design
  may rename); no interactive content this ticket.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `region_widths` is computed for a window width, two `DockState`s, and a dock width, the system shall give each Open dock exactly the dock width, each Closed dock exactly zero, and the center exactly the remainder — for ALL FOUR state combinations. | unit tests (4 combos; mutation targets: per-side zero arm, left↔right swap, center-remainder-not-constant) |
| REQ-002 | WHEN `Keymap::default_bindings()` is built, the system shall map `cmd-b` to `toggle-left-dock` and `cmd-shift-b` to `toggle-right-dock` while preserving the three existing chords, and `action_for` shall still return `None` for unbound chords. | unit (extended keymap tests) |
| REQ-003 | WHEN a toggle-dock action dispatches (chord path), the system shall flip ONLY that side's `DockState` and mark the view dirty (R6), and the render shall size regions from `region_widths` so a Closed dock vanishes and the center absorbs its width (R4/R5). | pure fn unit tests + `DockState::toggled` (existing) + shim review; headed assertion deferred with #23's environmental note |
| REQ-004 | WHEN the palette lists commands, the system shall include "Toggle Left Dock" and "Toggle Right Dock" bound to the new actions (activation dispatch = seq-4). | review (the static list lives in the shim) + seq-4 tests |
| REQ-005 | WHEN `scripts/gates.sh --diff` runs over the staged change, every gate shall be GREEN with coverage 100%/MSI 100% on the touched pure surface. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — RegionWidths shape, exact render structure (flex row of 3 divs? absolute?),
  chord collision check, SPEC R31 text, test plan.
- **P3 Implement** — layout.rs fn + keymap + app.rs render/dispatch + spec + CHANGELOG.
- **P3.5 Inspect** — critics (small change: 2) on width math edges + render/dispatch wiring.
- **P4 Validate** — tests + gate GREEN [--diff].
- **P5 Complete** — docs, AAR, archive, close #24.
