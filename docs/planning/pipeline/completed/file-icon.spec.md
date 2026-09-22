---
pipeline_id: dc5e2ca6-7bb2-47dc-b0c7-53da8e9dff2f
ticket: forge#133 (ee402b99-849d-4993-8a90-a0e6ac168a86) · local docs/planning/tickets/open/TICKET-133-file-icon.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: file-explorer icon in the top bar (M7)
type: feature
milestone: M7 — The Warp Top Bar & Sessions
references:
  - crates/marley_app/src/app.rs (SHIM: a 📁 icon in the top bar → open_files_pane)
---

## Title
A discoverable file-explorer icon in the top bar — click it to open the file tree (no palette needed).

## Scope
### In
- SHIM: a clickable 📁 glyph (reuse `pane_icon(PaneKind::FileTree)`) at the top-left of the top bar; click →
  `self.open_files_pane()` (the #128 opener).

### Out
- New open logic (reuse #128). The cockpit icons (#134). Renaming the left "Files" dock.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — reuse `pane_icon(FileTree)` (tested) for the glyph + `open_files_pane` / `open_or_focus` (tested #128)
  for the action — shim-only, no new pure surface.
- D2 — the icon sits at the top-left of the top bar (absolute, in the `TOP_BAR_H` band), before the search.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN the app renders, a file-explorer icon shall show in the top bar (left). | live capture |
| REQ-002 | WHEN the icon is clicked, it shall open/focus a FileTree pane (via the tested `open_files_pane`). | code-review (click env-blocked) + #128 tests |
| REQ-003 | FULL gate GREEN (shim-only; the open path's cov/MSI 100 is from #128). | gate |

## Phase Plan
- **P2** — the icon render + click; note shim-only.
- **P3** — implement (app.rs).
- **P3.5** — 1 self-review: the glyph + the open_files_pane wire; placement in the bar.
- **P4** — a LIVE capture (the icon in the bar) + gate GREEN.
- **P5** — docs, AAR, archive, close #133.
