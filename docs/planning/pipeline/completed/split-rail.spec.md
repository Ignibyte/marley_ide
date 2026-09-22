---
pipeline_id: 10c8b61b-1a77-4de6-948f-5b620ae36b95
ticket: forge#155 (60e0f233-3ec5-4e2d-bbdb-6189da9f6dc7) · local docs/planning/tickets/open/TICKET-155-split-rail.md
aar_id: d826f887-0acb-4b7c-be87-d3ca8b8632d9
status: Phase 5 — Complete PASS
title: M9 seq-6 — right-click → Split + nested pane rows in the rail
type: feature
milestone: M9 — Workspace / Project / Tab model
references:
  - crates/marley_app/src/tabs.rs (PURE: RailLevel::Pane, RailRow.pane, rail_rows nested pane rows)
  - crates/marley_app/src/app.rs (SHIM: right-click → split_focused; the rail Pane render arm)
---

## Title
Splitting is opt-in per terminal tab — right-click a terminal to tile it (reusing the M6 split), and the split
panes appear NESTED under the terminal tab in the rail.

## Scope
### In
- PURE `tabs.rs`: `RailLevel::Pane`; `RailRow.pane: Option<usize>`; `rail_rows` emits a nested Pane row per pane
  when a terminal tab's grid has >1 pane (the focused pane marked active).
- SHIM `app.rs`: a right-click (MouseButton::Right) on a terminal pane → `split_focused` (reuse the ⌘D spawn) +
  persist; the rail render gets a `RailLevel::Pane` arm (deeper indent; click → focus that pane).

### Out
- A full context menu (Split Right / Down / Close) — a right-click splits directly (Horizontal/After) for seq-6.
- Renaming/closing a pane from the rail; per-pane titles beyond "pane N".

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — a single-pane terminal tab shows NO nested rows (only a split tab nests). The `>1 pane` guard.
- D2 — right-click splits Horizontal/After (the ⌘D direction), reusing the existing spawn closure + persist_grid.
- D3 — Pane-row label = `"pane {k+1}"` (1-based); active = the tab is active AND `pane_ids[k] == grid.focused()`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a terminal tab's grid has >1 pane, `rail_rows` shall emit one `Pane` row per pane after that Tab row; WHEN it has 1 pane, it shall emit none. | unit |
| REQ-002 | WHEN a Pane row is emitted, `active` shall be true iff its tab is the active tab AND its pane is the grid's focused pane; `pane` shall be `Some(k)` (the 0-based index). | unit |
| REQ-003 (visual) | WHEN a terminal pane is right-clicked, the terminal tab shall tile into 2 panes AND the rail shall show 2 nested pane rows under the tab. | driven capture |
| REQ-004 | gate GREEN, cov/MSI 100 on the pure rail_rows change; the render/right-click masked. | gate |

## Phase Plan
- **P2** — RailLevel::Pane + RailRow.pane + rail_rows nesting; the right-click + rail Pane arm; test plan.
- **P3** — implement (tabs.rs + app.rs).
- **P3.5** — 2 critics (rail_rows nesting correctness + the >1 guard; the right-click split + focus routing).
- **P4** — pure tests (cov/MSI 100) + a driven capture (right-click → 2 panes + 2 nested rows) + gate.
- **P5** — docs, AAR, archive, close #155.
