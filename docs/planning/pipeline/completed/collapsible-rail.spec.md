---
pipeline_id: 4397528e-666d-4347-bd54-ddc5f415b5e8
ticket: forge#236 (1c9c86a3-b953-45b2-b68b-8534a1b66ffc) · local docs/planning/tickets/open/TICKET-236-collapsible-rail.md
aar_id: f0bf7067-1fdb-434e-93ee-59496d4c9ac3
status: Phase 5 — Complete PASS
title: Collapsible workspace rail tree + focused-workspace highlight
type: feature
milestone: M13
references: [forge#233, forge#152, forge#155, forge#219]
---

## Title
chad feedback #8 ("collapse workspace tabs + sub-tabs under it") + the rail-highlight ask ("a highlight
when the workspace is open"). The left rail becomes a proper disclosure tree: each workspace (project)
row gets a ▸/▾ chevron that collapses/expands its tabs (+ nested panes), and the FOCUSED workspace (the
active project, #233) gets a clear highlight (a background, not just today's subtle bright-text). Extends
the pure `tabs::rail_rows` (#152) — the collapse projection + the highlight flag stay pure; the shim
renders the chevron + toggle + highlight.

## Scope
### In
- **Pure (tabs.rs, cov/MSI 100):** `rail_rows(ws, collapsed: &HashSet<usize>)` — a collapsed project's
  Tab/Pane rows are OMITTED (its Project row stays, marked collapsed); `RailRow` gains `collapsed: bool`
  (true on a collapsed Project row, false elsewhere).
- **Shim (app.rs rail render):** a `collapsed_projects: HashSet<usize>` on RootView (passed to the 2
  `rail_rows` calls); the Project row renders a ▸ (collapsed) / ▾ (expanded) chevron that toggles
  collapse on click (stop_propagation, like the #184 fold + the #162 × close); the ACTIVE project row
  renders a prominent highlight background (`rail_highlight(active)`), the focused-workspace indicator.

### Out (deferred)
- Persisting collapse state across restart (transient this ticket).
- Collapsing the top Workspace/Session header itself (only project-level collapse; the Session header is
  a single label).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — collapse the PROJECT** (= chad's workspace, #233); its Tab + nested Pane rows hide. The chevron
  lives on the Project row.
- **D2 — the highlight uses the existing `active` flag** (the active project = the focused workspace);
  the shim upgrades it from bright-text-only to a background highlight (chad's "a highlight when open").
- **D3 — collapse state in the shim** (`HashSet<usize>` of collapsed project indices), passed into the
  pure `rail_rows`; no Project-struct change.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE a project index is in `collapsed`, `rail_rows` shall omit that project's Tab + Pane rows and mark its Project row `collapsed = true`; expanded projects emit all children with `collapsed = false`. | tabs.rs unit tests (collapsed hides children + the flag), cov/MSI 100 |
| REQ-002 | The Project row shall render a ▸/▾ chevron that toggles the project's collapse on click (without switching the project). | shim review + driven capture |
| REQ-003 | The ACTIVE project row (the focused workspace) shall render a prominent highlight background. | shim review + driven capture |
| REQ-004 | WHEN a project is collapsed, its tab rows shall disappear from the rail; expanding restores them. | driven capture (collapse → tabs hide; expand → return) |

## Phase Plan
- **P2 Design** — the `rail_rows` signature + `RailRow.collapsed`; the chevron + toggle + highlight
  render; the collapse-toggle helper; run `cargo mutants --list`.
- **P3 Implement** — pure (tabs.rs) then the app.rs rail shim.
- **P3.5 Inspect** — critics: the collapse projection correctness, the render (chevron/highlight/toggle),
  no over-collapse (the active project's children when collapsed — the focused row still shows), clean-room.
- **P4 Validate** — RUN the pure tests; gate green; DRIVEN capture (collapse a project → tabs hide; the
  active highlight).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #236.
