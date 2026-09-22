---
pipeline_id: e178bf06-12b1-42d7-8c81-3449c7361b9e
ticket: forge#132 (4bed8453-ad63-46fd-840f-1dbadb6e8c59) · local docs/planning/tickets/open/TICKET-132-top-bar.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: the Warp top bar (search in-bar; fixes the close button) [M7]
type: bug
milestone: M7 — The Warp Top Bar & Sessions
references:
  - crates/marley_app/src/layout.rs (PURE: content_band)
  - crates/marley_app/src/app.rs (SHIM: TOP_BAR_H; the bar row; shift the content band down; search in-bar)
---

## Title
A real top bar: the search lives in a dedicated bar at the top (not floating over the terminal), and the
pane grid drops below it so the pane title bars — and their close ×  — are clickable again.

## Scope
### In
- PURE `content_band(window_h, top_bar_h, status_bar_h) -> (content_top, content_h)`.
- SHIM: `TOP_BAR_H`; the top-bar row; shift docks + `center_bounds` down by `content_top`; the #117 search
  rendered in-flow inside the bar (its results dropdown positioned below the bar).

### Out
- Cockpit → top-bar icons (#134). The file-explorer icon (#133). New-session "+" (#135).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `content_band` is the vertical analogue of `region_widths`: `content_top = top_bar_h`,
  `content_h = (window_h - top_bar_h - status_bar_h).max(0.0)`.
- D2 — the docks and the pane grid start at `content_top` (below the bar); the bottom status bar is
  unchanged. The #126 cockpit tabs may keep floating for now (they sit in the bar zone above the moved-down
  panes; #134 converts them to in-bar icons) — the close-× is fixed purely by the panes moving down.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `content_band(1000, 28, 22)` is called, it shall return `(28, 950)`. | unit |
| REQ-002 | WHEN the window is shorter than the two bars, `content_h` shall clamp to 0 (content_top unchanged). | unit |
| REQ-003 (visual) | WHEN the app renders, the search shall sit inside a top bar, NOT floating over the terminal. | live capture |
| REQ-004 (visual) | WHEN the app renders, the pane grid + title row shall start below the top bar (title bars clear of overlays). | live capture |
| REQ-005 | gate GREEN, cov/MSI 100 on content_band; the shim masked. | gate |

## Phase Plan
- **P2** — content_band; TOP_BAR_H + the bar row + shifting the content band; the search relocation; test plan.
- **P3** — implement (layout.rs + app.rs).
- **P3.5** — 1 self-review: the band math; the y-offsets; the search-in-bar; no overlay over the title row.
- **P4** — content_band tests (cov/MSI 100) + a LIVE capture (search in-bar; panes below) + gate GREEN.
- **P5** — docs, AAR, archive, close #132.
