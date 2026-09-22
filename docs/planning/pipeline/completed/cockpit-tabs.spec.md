---
pipeline_id: 3f601a43-ce7b-4104-a7f9-571ace34afd6
ticket: forge#153 (334bd265-51fe-4f75-9a57-a7f9671e28d5) · local docs/planning/tickets/open/TICKET-153-cockpit-tabs.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: M9 seq-4 — cockpit as full-screen tabs (retire the right dock)
type: feature
milestone: M9 — Workspace / Project / Tab model
references:
  - crates/marley_app/src/tabs.rs (PURE: cockpit_section, terminal_grid_index, tab_grid, open_or_switch_cockpit)
  - crates/marley_app/src/app.rs (SHIM: workspace() guard, cockpit_body extraction, center render branch, top-icon rewire, retire right dock)
---

## Title
Details / Agents / Forge become full-screen tabs — click a cockpit icon and it opens as a tab in the
workspace, not a panel wedged on the right. The right dock retires.

## Scope
### In
- PURE `tabs.rs`: `Tab::cockpit_section`; `Project::terminal_grid_index` + `tab_grid`/`tab_grid_mut`;
  `Project::open_or_switch_cockpit(section)`.
- SHIM `app.rs`: `workspace()/workspace_mut()` fall back to a terminal tab (no panic on a cockpit-active tab);
  extract `cockpit_body(section)`; the center render branches cockpit vs grid; the top-right icons open a
  cockpit tab; the right-dock cockpit render retires.

### Out
- The full per-tab agent tracking (#158 — this only stops the right-dock stale read). Persisting cockpit tabs.
- Closing a cockpit tab from the rail (a tab-close affordance is a later concern).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `terminal_grid_index`: the active tab if it is a terminal, else the FIRST terminal tab. `workspace()`
  targets that, so with a cockpit tab active the app still operates the focused terminal (no panic). Assumes
  ≥1 terminal tab (the boot terminal; cockpit tabs are ADDED, never replace it).
- D2 — a Cockpit tab is per-project (`open_or_switch_cockpit` switches to an existing one of that section or
  appends). Forge/Agents/Details all become tabs (revisit a global Forge later if wanted).
- D3 — the center render: active tab `Cockpit(section)` → `cockpit_body(section)` full-screen; else the grid.
  The right dock no longer renders the cockpit.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the active tab is a terminal, `terminal_grid_index` shall return the active index; WHEN it is a cockpit, it shall return the first terminal tab's index. | unit |
| REQ-002 | WHEN `open_or_switch_cockpit(Details)` runs and a Details tab exists, it shall switch to it; else it shall append one and activate it. | unit |
| REQ-003 | WHEN `Tab::cockpit_section` is called, it shall return `Some(section)` for a cockpit tab and `None` for a terminal tab. | unit |
| REQ-004 (visual) | WHEN a top-right cockpit icon is clicked, a cockpit tab shall fill the CENTER (not a right panel), and the rail shall show it. | driven capture |
| REQ-005 (visual) | WHEN switching back to a terminal tab, the terminal shall render (no panic from a cockpit-active tab). | driven capture |
| REQ-006 | gate GREEN, cov/MSI 100 on the pure helpers; the render/wiring masked. | gate |

## Phase Plan
- **P2** — the pure helpers; the workspace() guard; the cockpit_body extraction + render branch + top-icon rewire; test plan.
- **P3** — implement (tabs.rs + app.rs).
- **P3.5** — 2 critics (the guard never panics / behavior preserved; the extraction + render branch correctness).
- **P4** — pure tests (cov/MSI 100) + driven captures (cockpit tab full-screen; terminal returns) + gate GREEN.
- **P5** — docs, AAR, archive, close #153.
