---
pipeline_id: 8d1626b4-ef6f-45c8-8c1f-44c90e8e2c92
ticket: forge#90 (d9ff256c-ea0c-4130-9ae9-626688d6c8ef) · local docs/planning/tickets/open/TICKET-090-right-dock-sections.md
aar_id: 0423389e-798b-4e91-a1e9-98ca9f99a8b4
status: Phase 5 — Complete PASS
title: right-dock sections — Details / Agents / Forge tabs
type: feature
milestone: M2.F — The Persistent Cockpit (FOUNDATION)
references:
  - crates/marley_app/src/right_dock.rs (NEW PURE: RightSection, section_tabs, section_label)
  - crates/marley_app/src/lib.rs (mod right_dock)
  - crates/marley_app/src/app.rs (SHIM: right_section + the tab strip + body switch)
---

## Title
Turn the right dock into a persistent tabbed cockpit rail — Details / Agents / Forge tabs — so the agent
Fleet + Forge stop being transient ⌘⇧ overlays. The FOUNDATION for M2.F (#91-95 fill the sections).

## Scope
### In
- NEW pure `right_dock.rs`: `enum RightSection { Details, Agents, Forge }` (Copy/Eq) + `section_tabs()` +
  `section_label()`.
- SHIM: `RootView.right_section` (default Details); the right dock gains a 3-tab strip (active highlighted)
  + switches the body on the section; Details keeps #58 block_details; Agents/Forge are placeholders.

### Out
- The Agents section content (#91). The Forge section content (#92). The rich inspector (#93). Persistence
  (#95). Keeping ⌘⇧E/⌘⇧F overlays (they stay as quick toggles).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `RightSection` Copy/Eq; `section_tabs()` fixed order Details/Agents/Forge; `section_label` a match.
- D2 — the tab strip reuses colors.accent for the active tab (the #75/#24 highlight convention).
- D3 — Agents/Forge sections are one-line muted placeholders in #90; #91/#92 fill them.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `section_tabs()` runs, it shall return (Details,"Details"),(Agents,"Agents"),(Forge,"Forge") in order. | unit |
| REQ-002 | WHEN `section_label(s)` runs, it shall return the section's label. | unit |
| REQ-003 (visual) | WHEN a tab is clicked, `right_section` shall switch and the body shall show that section. | self-test (env-blocked → engine) |
| REQ-004 | gate GREEN, cov/MSI 100 on right_dock.rs; the shim masked. | gate |

## Phase Plan
- **P2** — right_dock.rs API; the app.rs field + tab-strip + body-switch render; test plan.
- **P3** — implement (right_dock.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: section_tabs order/labels MSI; the tab click switches; Details unchanged.
- **P4** — section_tabs/section_label tests (cov/MSI 100) + gate GREEN + self-test (env-blocked → engine).
- **P5** — docs, AAR, archive, close #90.
