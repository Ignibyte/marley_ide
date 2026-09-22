---
pipeline_id: 6df7a9da-f321-4038-8bc4-28a77f0b2728
ticket: forge#239 (e5fde138-6d0f-4b55-91a5-60ac3d9f9d50) · local docs/planning/tickets/open/TICKET-239-rail-remove-session-header.md
aar_id: 9d29dd82-6701-474a-88a1-61e1f8b59c52
status: Phase 5 — Complete PASS
title: Remove the redundant "MARLEY" session header from the left rail
type: chore
milestone: M14
references: [forge#152, forge#233, forge#236]
---

## Title
Remove the redundant "MARLEY" session header from the left rail (chad live issue #1).

The left rail is a Workspace → Project → Tab → Pane tree (#152). Its very first row is a `RailLevel::Workspace`
header rendered as an all-caps muted caption — the top-level session/workspace container's name ("Marley" →
"MARLEY"). With a single session open that header is pure redundancy: the status bar (`~/…/Marley · main`) and
the project row + tabs already identify the workspace (chad: "workspaces can be identified by the bottom row and
the tabs"). Remove it.

## Scope
### In
- **`tabs::rail_rows`** no longer emits the top-of-tree `RailLevel::Workspace` row; its output for a workspace
  begins with the first project's `RailLevel::Project` row. Project / Tab / Pane rows are unchanged.
- The shim render loop follows (the `RailLevel::Workspace => { … }` arm at app.rs:~4258 is removed), and — if
  the pure removal leaves the variant unconstructed — the now-unused `RailLevel::Workspace` variant too.
- Update the `rail_rows` unit tests (row indices shift by one; no Workspace row asserted).

### Out
- A multi-workspace GROUP header — if/when true multi-workspace lands, a workspace/group header can be
  reintroduced (from git history) as its own ticket. Not now.
- Any change to Project / Tab / Pane rows, the #236 collapse logic, the #169 rail scroll, or click/switch
  behavior. Purely the removal of the one header row.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — remove the Workspace-level rail row entirely** (the session has exactly one workspace today; the header
  is redundant per chad #1). The status bar + the project row + tabs carry workspace identity.
- **D2 — the removal LAYER is a Phase-2 (Design) decision.** Two options: (a) drop the row in the PURE
  `tabs::rail_rows` AND remove the now-unused `RailLevel::Workspace` variant + its render arm (cleanest — no
  dead pure output; reintroduce from git history when multi-workspace needs a group header); (b) a shim-only
  skip that keeps the pure row + variant. **Lean = (a).** Confirm nothing else reads the Workspace row (grep at
  design: only the render arm + the two rail_rows tests do).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `rail_rows` shall not emit a `RailLevel::Workspace` row — its output for a workspace shall begin with the first project's `RailLevel::Project` row. | rail_rows unit test (`rows[0].level == Project`; no `Workspace` row; count == prior − 1) |
| REQ-002 | The Project / Tab / Pane rows shall be emitted unchanged (labels, active flags, project/tab/pane indices, collapse flag). | existing rail_rows unit tests (project/tab/pane assertions pass, re-indexed) |
| REQ-003 | The left rail shall render no all-caps session header above the project row. | driven capture — the rail's top row is the "Marley · main" project row; no "MARLEY" caption |

## Phase Plan
- **P2 Design** — pick the removal layer (D2 a vs b); the file manifest (tabs.rs rail_rows + enum, app.rs render
  arm); the test-plan delta (which rail_rows tests re-index; a new "no Workspace row" assert). Confirm no other
  consumer. `cargo mutants --list -f tabs.rs` for the pure seam.
- **P3 Implement** — pure `rail_rows` (drop the Workspace row) → the enum + render arm → re-index tests to compile.
- **P3.5 Inspect** — critic: completeness (all Workspace-row consumers handled), no rail-scroll/index regression,
  clean-room, cov/MSI.
- **P4 Validate** — RUN the rail_rows tests; gate green; DRIVEN capture (rail shows no "MARLEY" header; project +
  tabs intact).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #239; archive.
