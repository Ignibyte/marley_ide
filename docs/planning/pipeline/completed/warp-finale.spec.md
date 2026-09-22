---
pipeline_id: 385a6396-a25d-4e6b-985b-6ed8be2bf969
ticket: forge#127 (fb0f6938-8b86-46fb-9480-6e734c2dbf1c) · local docs/planning/tickets/open/TICKET-127-warp-finale.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: the Warp default arrangement + finale (M6 seq-8 FINALE)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/grid_layout.rs (PURE: pub default_grid + test)
  - crates/marley_app/src/app.rs (SHIM: the right side defaults free; chrome polish)
---

## Title
The Warp arrangement, landed: sessions sidebar (left) | a pane grid (terminal + openable files/code/git
panes) with the cockpit tabs at the top and the right side free. Closes M6.

## Scope
### In
- PURE: make `default_grid()` public + a direct test (the first-run default = one Terminal).
- SHIM: the right dock defaults CLOSED (free right side; fixes the #126 top-tab/dock-header overlap); minor
  chrome polish if the reference compare shows a gap.
- VALIDATE: capture the representative Warp layout + compare to chad's reference screenshots.

### Out
- New panes/features. Pixel-identical Warp cloning (Marley keeps its own cyan/dark identity).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `default_grid()` (already the single-Terminal default behind `restore_grid`) becomes public + directly
  tested; `status_segments` already exists (#94) so nothing is added there.
- D2 — the right dock defaults CLOSED at boot (the cockpit is on-demand via the top tabs) — the right side is
  free for the pane grid, and the #126 overlap is gone.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `default_grid()` is called, it shall return a single-Terminal grid (no split). | unit |
| REQ-002 (visual) | WHEN the app boots, the right side shall be free (no rail) with the cockpit tabs at the top. | live capture |
| REQ-003 (visual) | WHEN compared to a Warp reference screenshot, the arrangement shall match in structure (sidebar left · pane grid · top tabs). | capture compare |
| REQ-004 | gate GREEN, cov/MSI 100 on default_grid; the shim masked. | gate |

## Phase Plan
- **P2** — pub default_grid + test; the right-dock-closed default; the reference-compare plan.
- **P3** — implement (grid_layout.rs + app.rs).
- **P3.5** — 1 self-review: default_grid; the boot dock default; no dangling right-dock refs.
- **P4** — default_grid test (cov/MSI 100) + a LIVE capture compared to the reference + gate GREEN.
- **P5** — docs, AAR, archive, close #127; **close M6 sprint #17**.
