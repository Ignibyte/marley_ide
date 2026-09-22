---
pipeline_id: 790215b3-a8ca-439c-9046-6d64a086f32b
ticket: forge#126 (b10f40e4-dbb6-4a7e-bb9a-74b56995b008) · local docs/planning/tickets/open/TICKET-126-top-tabs.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: retire the right rail → top tabs (M6 seq-7)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/right_dock.rs (PURE: TopTab, top_tabs)
  - crates/marley_app/src/app.rs (SHIM: top tab strip; move tabs off the dock; dock defaults closed)
---

## Title
Tabs at the top — the Details / Agents / Forge tabs move from the right rail up to the top bar; their content
shows in a panel on demand, so the right side is free for the pane grid. Chad's "tabs at the top."

## Scope
### In
- PURE `TopTab` + `top_tabs(active)` (the 3 tabs, the active one flagged).
- SHIM: render the tab strip at the top; a tab click selects the section + opens the right dock; the dock
  shows only the section body; the right dock defaults closed at boot.

### Out
- Reworking the section bodies (Details/Agents/Forge content unchanged). Folding content into panes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `top_tabs(active)` reuses `section_tabs()`'s (section, label); the active one flagged (== active).
- D2 — the top tabs drive `right_section` + open the right dock (persist via #95); the dock's own tab strip
  is removed; the right dock defaults CLOSED (the right side is free until a tab is clicked).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `top_tabs(active)` runs, it shall list Details/Agents/Forge in order with exactly the active flagged. | unit |
| REQ-002 (visual) | WHEN the app renders, the Details/Agents/Forge tabs shall show at the TOP (not a right rail). | live capture |
| REQ-003 | gate GREEN, cov/MSI 100 on top_tabs; the shim masked. | gate |

## Phase Plan
- **P2** — TopTab/top_tabs; the top strip + the dock changes; test plan.
- **P3** — implement (right_dock.rs + app.rs).
- **P3.5** — 1 self-review: no dangling right_section; tab click switches + persists.
- **P4** — top_tabs tests (cov/MSI 100) + a LIVE capture (tabs at top) + gate GREEN.
- **P5** — docs, AAR, archive, close #126.
