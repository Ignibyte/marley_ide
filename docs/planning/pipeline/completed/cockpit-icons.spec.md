---
pipeline_id: d4551f36-f055-4ad9-a060-17cb935b7b0b
ticket: forge#134 (696ea5d0-ccd2-4fa8-8a2b-89140e2f68d0) · local docs/planning/tickets/open/TICKET-134-cockpit-icons.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: the cockpit as top-bar icons (M7)
type: feature
milestone: M7 — The Warp Top Bar & Sessions
references:
  - crates/marley_app/src/right_dock.rs (PURE: section_icon)
  - crates/marley_app/src/app.rs (SHIM: the #126 cockpit block renders icons)
---

## Title
The Details / Agents / Forge cockpit becomes a compact icon cluster in the top bar, not overlapping text tabs.

## Scope
### In
- PURE `section_icon(RightSection) -> &'static str` (one glyph per section).
- SHIM: the #126 cockpit block renders `section_icon` glyphs instead of `tab.label`; click logic unchanged.

### Out
- Reworking the section bodies / the click behavior (unchanged). The file icon (#133).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `section_icon` maps each `RightSection` to a distinct glyph (Details/Agents/Forge); reuse `top_tabs`
  (#126) for the model + the click handler (right_section + open dock + persist).
- D2 — the active icon renders in the accent color, inactive in muted (as the text tabs did).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `section_icon` is called for each section, it shall return a distinct glyph per section. | unit |
| REQ-002 (visual) | WHEN the app renders, the cockpit shall show as icons in the top-bar right (active accented), not overlapping text. | live capture |
| REQ-003 | gate GREEN, cov/MSI 100 on section_icon; the shim masked. | gate |

## Phase Plan
- **P2** — section_icon; the render swap; test plan.
- **P3** — implement (right_dock.rs + app.rs).
- **P3.5** — 1 self-review: the glyphs distinct; the render swap keeps the click.
- **P4** — section_icon tests (cov/MSI 100) + a LIVE capture (icon cluster) + gate GREEN.
- **P5** — docs, AAR, archive, close #134.
