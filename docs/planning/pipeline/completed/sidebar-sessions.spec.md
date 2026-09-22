---
pipeline_id: 58588abe-e697-488e-905d-d8b63f926b91
ticket: forge#129 (7102ac4f-ca8e-44e5-a741-c26ef6a0a680) · local docs/planning/tickets/open/TICKET-129-sidebar-sessions.md
aar_id: 3-recorded-in-notes
status: Phase 5 — Complete PASS
title: the sessions sidebar for the pane world (M6)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/workspace.rs (PURE: panes_of_kind)
  - crates/marley_app/src/app.rs (SHIM: the sidebar builds sessions from terminal panes only)
---

## Title
The sidebar shows real terminal sessions again — a files/code/git pane no longer appears as a phantom
"terminal 2". Fixes a visible bug from #121.

## Scope
### In
- PURE `panes_of_kind(kind)` → the pane ids of that kind (in order).
- SHIM: the sidebar `sessions` list builds from `panes_of_kind(Terminal)` (filter then number).

### Out
- Reworking the session rows / grouping (unchanged). Sidebar for non-terminal panes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `panes_of_kind(kind)` generalizes `first_pane_of_kind`; the sidebar uses `panes_of_kind(Terminal)`.
- D2 — numbering ("terminal 1/2") is over the FILTERED terminals; the row's PaneId is the terminal's (focus).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `panes_of_kind(Terminal)` runs on [Terminal, FileTree, Terminal, Git], it shall return the 2 terminal ids. | unit |
| REQ-002 | WHEN no pane of a kind exists, `panes_of_kind` shall return empty. | unit |
| REQ-003 (visual) | WHEN a Files/Git pane is open, the sidebar shall NOT show a phantom "terminal N" for it. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on panes_of_kind; the shim masked. | gate |

## Phase Plan
- **P2** — panes_of_kind; the sidebar filter; test plan.
- **P3** — implement (workspace.rs + app.rs).
- **P3.5** — 1 self-review: terminal-only filter; numbering + focus id.
- **P4** — panes_of_kind tests (cov/MSI 100) + a LIVE capture (no phantom terminal) + gate GREEN.
- **P5** — docs, AAR, archive, close #129.
