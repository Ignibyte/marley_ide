---
pipeline_id: f74afe76-b018-43e7-a7ee-8f15ced4e350
ticket: forge#38 (d6bf9b17-cc33-4cd1-8792-599806b52e22) · local docs/planning/tickets/open/TICKET-038-dock-panels.md
aar_id: 47b891b7-03fa-4399-b892-82f46c5217ab
status: Phase 5 — Complete PASS
title: dock panels + titlebar styling
type: feature
milestone: M1.E
references:
  - crates/marley_app/src/layout.rs (DockSide + region_widths; gains the pure dock_title)
  - crates/marley_app/src/app.rs (~612-634 the dock render — shim; ~889 the titlebar)
  - docs/specs/SPEC-app-shell.spec.md (gains the dock-panel clause)
---

## Title
The left/right docks (#24) render as bare `surface` divs with a hardcoded `"Files"`/`"Details"` label
— no header, divider, or panel structure. Give them real panel treatment (a styled header + a divider
+ elevation) and extract the label into a pure `dock_title(DockSide)`, so the window reads as one
cohesive Warp-style workspace. The most SHIM-heavy ticket of the sprint.

## Scope
### In
- `crates/marley_app/src/layout.rs` (PURE — cov/MSI 100): `dock_title(side: DockSide) -> &'static str`
  — `Left → "Files"`, `Right → "Details"` (an exhaustive 2-arm match; extracts the app.rs inline
  hardcode into a tested decision).
- `crates/marley_app/src/app.rs` (SHIM, ~612-634) — replace the two bare `div().bg(surface)
  .child("Files"/"Details")` with a shared dock-PANEL treatment: `surface` bg + a HEADER row
  (`dock_title(side)` in a caption style — smaller/muted — with padding) + a DIVIDER border on the
  inner edge toward the center (right border for the Left dock, left border for the Right) + content
  padding below the header. Confirm the root window bg is `colors.background` (a cohesive surface with
  the native titlebar).
- SPEC-app-shell: the dock-panel clause (R44). CHANGELOG + arch doc.

### Out (explicitly deferred)
- Dock ICONS (a glyph-availability risk in the dock UI font — a later cut once a bundled icon set
  exists). Real dock CONTENT (a file tree, a details inspector — M2; the panels are chrome shells
  now). A custom in-window titlebar strip / wordmark beyond the native macOS titlebar (the native
  titlebar limits this; the window is already themed + titled "Marley"). Resizable/draggable dock
  widths (M2). Hover/focus affordances (#39).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — The pure surface is `dock_title(DockSide)` (the label decision) — thin but genuine; the panel
  layout (header/divider/padding) is SHIM (app.rs, mutants::skip + cov-excluded, masked visual). This
  ticket adds no other pure logic (the layout math is #24's already-tested `region_widths`).
- D2 — The divider is a border on the dock's INNER edge (toward the center pane) — Left dock → right
  border, Right dock → left border — so each dock visually separates from the terminal.
- D3 — The header caption is "muted" via a smaller text size + the `border`/dimmed tone (no new
  ThemeColors role); icons deferred (glyph risk).
- D4 — Titlebar: keep the native macOS titlebar (`TitlebarOptions` title "Marley"); ensure the root
  bg is `background` for cohesion. A custom wordmark strip is out (native-titlebar limit).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `dock_title(side)` is called, it shall return `"Files"` for `Left` and `"Details"` for `Right`. | unit tests (both arms) |
| REQ-002 | WHEN an open dock is rendered, the system shall paint a panel with a header showing `dock_title(side)`, a divider border on its inner edge, and content padding, over the `surface` elevation. | shim + the masked 3-region visual baseline — chad-verified |
| REQ-003 | WHEN the window is rendered, the root background shall be the theme `background`, so the docks/center/native-titlebar read as one cohesive surface. | shim + masked visual |
| REQ-004 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `dock_title` (+ the existing `region_widths`); the 3-region visual rides the masked deferral. | gate exit 0 + receipt + gate:15 |

## Phase Plan
- **P2 Design** — `dock_title`'s exact shape + the app.rs dock-panel shim (header/divider/padding, the
  shared treatment for both sides), the root-bg confirm, the SPEC clause + mutation targets.
- **P3 Implement** — dock_title + the app.rs panel render + the root-bg + spec + CHANGELOG.
- **P3.5 Inspect** — critics: dock_title arms killable, the divider on the correct inner edge per
  side, the render uses dock_title (no re-hardcode), no #24 layout/region regression.
- **P4 Validate** — the dock_title unit test + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #38.
