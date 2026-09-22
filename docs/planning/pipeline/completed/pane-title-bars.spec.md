---
pipeline_id: 13f9db50-c22a-44a1-b128-a6056e507dfb
ticket: forge#108 (b8cd2f66-f520-40b6-9324-4017b40011a0) · local docs/planning/tickets/open/TICKET-108-pane-title-bars.md
aar_id: b5a055d0-d3de-4df6-83cd-301ea1826534
status: Phase 5 — Complete PASS
title: pane title bars
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/workspace.rs (PURE: pane_title, pane_icon)
  - crates/marley_app/src/app.rs (SHIM: the title-bar strip + the shrunk content pane)
---

## Title
Every pane gets a title bar at its top edge — a type icon + a name + a ⋮ + an × close (chad's "tabs up at
the top"). The Warp pane look.

## Scope
### In
- PURE `pane_title(kind, name)` + `pane_icon(kind)` (4 distinct glyphs).
- SHIM: a title-bar strip atop each pane; the content pane shifts down + shrinks below it; × closes the pane.

### Out
- The ⋮ menu contents (a stub in #108). Per-pane cwd/branch tracking (Terminal shows the project basename for now).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the title bar is a separate absolutely-positioned strip; the content pane's `top`/`h` shrink by
  `PANE_TITLE_H` so nothing clips the prompt (no restructure of the row children).
- D2 — × reuses `workspace.close(pane_id)` (refuses the last pane via the existing algebra).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `pane_title(kind, name)` runs, it shall be the kind's title (Terminal/CodeView use the name, else fixed). | unit |
| REQ-002 | WHEN `pane_icon(kind)` runs, it shall be the kind's distinct glyph. | unit |
| REQ-003 (visual) | WHEN a pane renders, a title bar (icon + name + ×) shall show at its top, above the content. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on pane_title/pane_icon; the shim masked. | gate |

## Phase Plan
- **P2** — pane_title + pane_icon; the title-bar strip + the shrunk content pane; test plan.
- **P3** — implement (workspace.rs + app.rs).
- **P3.5** — 1 critic: pane_title/pane_icon MSI; the strip doesn't clip the prompt; the × close bounded.
- **P4** — the title/icon tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #108.
