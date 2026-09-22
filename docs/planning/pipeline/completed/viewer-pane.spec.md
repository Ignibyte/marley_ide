---
pipeline_id: 7e725642-7686-4d26-9ba7-c92715d152e5
ticket: forge#114 (861a81d5-1115-4c12-bbc5-22325f1940f7) · local docs/planning/tickets/open/TICKET-114-viewer-pane.md
aar_id: 0c16ff56-2e29-4a17-8f80-ed19bafedd9b
status: Phase 5 — Complete PASS
title: the code viewer as a side-by-side panel
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/code_view.rs (PURE: viewer_split)
  - crates/marley_app/src/app.rs (SHIM: the terminal shrinks; the viewer is a right panel)
---

## Title
Open a file and the code viewer sits BESIDE the terminal (a right-side panel with a filename title + close),
not a centered modal that covers it — the Warp code pane.

## Scope
### In
- PURE `viewer_split(center_w) -> (terminal_w, viewer_w)` (a proportional split).
- SHIM: when the viewer is open, the terminals tile in `terminal_w` + the viewer renders in the right `viewer_w`.

### Out
- Making the viewer a full PaneGroup pane (needs the session-per-pane refactor — a larger follow-up). A
  draggable split ratio. Multiple viewer panes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `viewer_split(center_w)` = viewer 45% of the center, terminal the remainder.
- D2 — a side-PANEL toggled by `code_view` (not a PaneGroup pane) — reuses all of the M4 viewer render,
  repositioned from the centered modal to the right rect; the session-per-pane refactor is deferred.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `viewer_split(center_w)` runs, it shall split the center into (terminal, viewer) widths. | unit |
| REQ-002 (visual) | WHEN a file is open, the viewer shall sit beside the terminal (right panel, titled), not over it. | live capture |
| REQ-003 | gate GREEN, cov/MSI 100 on viewer_split; the shim masked. | gate |

## Phase Plan
- **P2** — viewer_split; the center-shrink + the right-panel viewer render; test plan.
- **P3** — implement (code_view.rs + app.rs).
- **P3.5** — 1 critic: viewer_split MSI; the terminal shrinks + the viewer fits; close works.
- **P4** — viewer_split tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #114.
