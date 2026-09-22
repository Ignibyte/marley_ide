---
pipeline_id: d5866f0c-7557-4809-beeb-4249972ec280
ticket: forge#151 (3898b95e-fe1f-40d4-a302-a4b1674efc29) · local docs/planning/tickets/open/TICKET-151-tab-render.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: M9 seq-2 — full-screen active-tab render (retire default tiling)
type: feature
milestone: M9 — Workspace / Project / Tab model
references:
  - crates/marley_app/src/app.rs (SHIM: shell field, workspace()/workspace_mut() accessors, 103-site migration, render, +-adds-tab)
  - crates/marley_app/src/tabs.rs or a small pure fn (next_index wrap)
---

## Title
Wire the tab model into the app — the content area shows the active project's active tab full-screen, and a
new terminal opens as a TAB, not a split. Splitting a tab stays (opt-in).

## Scope
### In
- `shell: Workspace<TerminalSession>` field (replaces the direct `workspace: PaneGrid`), booted with one
  project + one terminal tab wrapping the initial grid.
- `workspace()`/`workspace_mut()` accessors → the active tab's grid; migrate the 103 call sites.
- `+`/new_terminal → add a terminal TAB; `⌘D` → split the active tab's grid.
- Render the active tab full-screen (Terminal → the grid; Cockpit arm = stub until seq-4).
- A temporary next-tab keybinding for validation; pure `next_index` wrap.

### Out
- The clickable rail (seq-3/#152). Cockpit-tab rendering (seq-4). Persistence of the tab layout. The sidebar
  still lists the active grid's panes (transitional — replaced by the rail in seq-3).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — boot: `Workspace::new(project_basename, Project::new(project_basename, project_root, Tab::terminal(grid)))`.
- D2 — `workspace()/workspace_mut()` return the active tab's grid. Pre-seq-4 the active tab is ALWAYS a
  terminal (only + adds tabs, all terminal), so the Terminal-arm access is an internal invariant, not
  input-reachable; seq-4 (cockpit tabs) adds the guard. Design picks expect-with-message vs a fallback.
- D3 — `+` adds a terminal tab (retire default tiling); `⌘D` splits the active tab's grid (opt-in, previews seq-6).
- D4 — a temp `⌘⇧]` "next tab" key → `switch_tab(next_index(active, count))`; `next_index(i, n) = (i+1) % n`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the app boots, the content area shall render one terminal tab full-screen (visually unchanged). | driven capture |
| REQ-002 (visual) | WHEN `+` is driven, a NEW terminal tab shall be added, become active, and fill the area — NOT a split. | driven capture |
| REQ-003 (visual) | WHEN the next-tab key is driven, the active tab shall switch and the full-screen view swap. | driven capture |
| REQ-004 | WHEN `⌘D` is driven, the ACTIVE tab's grid shall split (tiling within a tab preserved). | driven capture |
| REQ-005 | `next_index(i, n)` shall wrap: `(0,1)→0`, `(0,3)→1`, `(2,3)→0`. | unit, cov/MSI 100 |
| REQ-006 | FULL gate GREEN; the shim masked; the new pure bit at cov/MSI 100. | gate |

## Phase Plan
- **P2** — the accessor shape + expect/guard decision; the 103-site migration plan; the boot; the render; test plan.
- **P3** — implement (app.rs shim + next_index).
- **P3.5** — 2 critics (the migration missed nothing / behavior preserved; the boot + accessor invariants).
- **P4** — next_index tests (cov/MSI 100) + driven captures (boot / + adds tab / switch / ⌘D split) + gate GREEN.
- **P5** — docs, AAR, archive, close #151.
