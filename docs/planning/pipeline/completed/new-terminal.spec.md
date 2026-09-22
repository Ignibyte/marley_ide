---
pipeline_id: 9cc076d0-d628-448d-aaeb-5b063fe7263b
ticket: forge#135 (4f5d3a3a-b1d6-4f16-8d53-8ed924db986d) · local docs/planning/tickets/open/TICKET-135-new-terminal.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: add a new terminal session (the "+") [M7]
type: feature
milestone: M7 — The Warp Top Bar & Sessions
references:
  - crates/marley_app/src/app.rs (SHIM: new_terminal_pane + a "+" in the top bar; DRY split-pane)
---

## Title
A "+" in the top bar spawns a new terminal session — the first discoverable way to add a terminal.

## Scope
### In
- SHIM: extract `new_terminal_pane()` (reuse `split_focused` + `spawn_session` + `persist_grid`); a "+" icon
  in the top-bar left → `new_terminal_pane()`; the `split-pane` dispatch delegates to it (DRY).

### Out
- Agent sessions (#136). A new pure surface (spawn/split already tested).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `new_terminal_pane()` = the existing split-pane body (a fresh PTY terminal via `split_focused` +
  `spawn_session`, then `persist_grid`); `"split-pane"` dispatch calls it (no behavior change).
- D2 — the "+" sits in the top-bar left, after the 📁 file icon.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN the app renders, a "+" affordance shall show in the top bar. | live capture |
| REQ-002 | WHEN the "+" is clicked, it shall spawn a new terminal pane (via the tested `split_focused`+`spawn_session`). | code-review (click env-blocked) + existing tests |
| REQ-003 | FULL gate GREEN (shim-only; spawn/split cov/MSI 100 already). | gate |

## Phase Plan
- **P2** — new_terminal_pane extraction; the "+" render; note shim-only.
- **P3** — implement (app.rs).
- **P3.5** — 1 self-review: the extraction keeps split-pane behavior; the "+" wires to new_terminal_pane.
- **P4** — a LIVE capture (the "+" in the bar) + gate GREEN.
- **P5** — docs, AAR, archive, close #135.
