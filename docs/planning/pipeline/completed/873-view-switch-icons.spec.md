---
pipeline_id: b2072e59-1e66-4761-aa6e-28aa3da31932
ticket: docs/planning/tickets/open/TICKET-873-view-switch-icons.md
status: Phase 4 — Complete PASS
title: The rail's view switch as icons
type: feature
slice: The Marley look (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
The rail header's Projects, Threads and Sessions switch draws as three icon buttons like the
header's others: a folder, a thread and a terminal, each named in its tooltip.

## Scope
### In
- `Rail::render_view_switch`: `IconButton`s (`IconName::Folder`, `Thread`, `TerminalAlt`) at the
  header buttons' size, `toggle_state` on the shown view, `Tooltip::text` with the view's name,
  the attention dot on Projects and the host-down dot on Sessions kept.

### Out (explicitly deferred)
- The header's other buttons; the shrunk rail's strip (#830), which shows no switch.

## Reference (§20)
N/A — Marley-specific: the rail's header is Marley's (#832, #868); Chad's word, 2026-10-21.

### Prior art
- `render_files_button` (#868): an `IconButton` with `IconSize::Small`, `toggle_state` and a
  tooltip, the idiom to match.
- The brain (consultation 7c557f67): nothing on this seam.

## UI proof
Scenario `script/e2e/873-view-switch-icons.sh` (sway): a scratch repository.
- `873-01-projects`: the header: three icons before Files, the folder pressed (REQ-001).
- `873-02-threads`: the thread icon clicked: Threads shown, its icon pressed (REQ-001, REQ-002).
- `873-03-day`: Marley Day, the terminal icon clicked: Sessions shown (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — Folder for Projects, Thread (Zed's thread icon) for Threads, TerminalAlt for Sessions.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail header shall show Projects, Threads and Sessions as icon buttons like its other buttons, the shown view's pressed, each named in its tooltip, in Marley Night and Marley Day. | `873-01-projects`, `873-02-threads`, `873-03-day` |
| REQ-002 | WHEN the user clicks a view's icon, the rail shall show that view as the words did. | `873-02-threads`, `873-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code** (`rail.rs`), **P3 Test**, **P4 Complete**.
