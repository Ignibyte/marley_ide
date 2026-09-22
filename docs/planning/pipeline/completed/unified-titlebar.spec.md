---
pipeline_id: d09af9eb-09dc-4966-a059-08be4f7b5eec
ticket: forge#138 (6b735046-2bd1-4e0a-bc2b-09a73a34689f) · local docs/planning/tickets/open/TICKET-138-unified-titlebar.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: unified macOS title bar [M8]
type: feature
milestone: M8 — Warp Chrome & Fidelity
references:
  - crates/marley_app/src/layout.rs (PURE: topbar_icon_x)
  - crates/marley_app/src/app.rs (SHIM: transparent titlebar + traffic-light position + left-icon inset)
---

## Title
One bar at the top — the icons + search share the row with the macOS traffic lights, and the redundant
"Marley" title is gone (Warp-style unified titlebar).

## Scope
### In
- SHIM: `TitlebarOptions { title: None, appears_transparent: true, traffic_light_position: Some(...) }`;
  inset the left top-bar icons (📁 + 🧠) past the traffic lights.
- PURE `topbar_icon_x(slot, inset, gap)` for the left-cluster icon x positions.

### Out
- cwd/branch title text (#142). The search dropdown reposition (#141). Non-mac window chrome.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `appears_transparent: true` hides the system titlebar so the window content extends to the very top;
  `traffic_light_position` places the lights vertically-centered in the `TOP_BAR_H` row. `title: None`.
- D2 — the left icons start after the traffic lights via `topbar_icon_x(slot, TRAFFIC_LIGHT_INSET, ICON_GAP)`.
  The centered search + right cockpit icons are unchanged. `content_band`/`TOP_BAR_H` unchanged.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `topbar_icon_x(slot, inset, gap)` is called, it shall return `inset + slot·gap`. | unit |
| REQ-002 (visual) | WHEN the app renders, the icons + search shall share the traffic-light row (one bar, no separate row below). | live capture |
| REQ-003 (visual) | WHEN the app renders, the traffic lights shall be unobscured (left of the file icon) and no "Marley" title shown. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on topbar_icon_x; the window/render shim masked. | gate |

## Phase Plan
- **P2** — the titlebar config; topbar_icon_x; the left-icon inset; test plan.
- **P3** — implement (layout.rs + app.rs).
- **P3.5** — 1 self-review: the titlebar flags; the inset; the traffic-light position.
- **P4** — topbar_icon_x tests (cov/MSI 100) + a LIVE capture (unified bar; lights unobscured) + gate GREEN.
- **P5** — docs, AAR, archive, close #138.
