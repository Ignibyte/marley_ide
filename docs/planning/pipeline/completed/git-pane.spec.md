---
pipeline_id: 879b091c-d408-402d-bdf6-6d3847fe528d
ticket: forge#125 (24f21f98-4fe6-40c2-b411-115250a10446) · local docs/planning/tickets/open/TICKET-125-git-pane.md
aar_id: 8-recorded-in-notes
status: Phase 5 — Complete PASS
title: the git panel as a grid pane (M6 seq-6)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/app.rs (SHIM: full Git pane render; open_git_pane; retire the side-panel + git_panel_open + right_open)
---

## Title
Source control becomes a real grid pane — ⌘⇧C opens a Git pane with the change list, stage toggles, and
commit box. Retires the M5 right-side git panel (clears duplication #3; both side-panels now gone).

## Scope
### In
- SHIM: the full git-panel render moves into the Git pane dispatch; `open_git_pane` (focus-or-open); ⌘⇧C
  opens the Git pane; delete the side-panel render + `git_panel_open` + `right_open`/`viewer_split`.

### Out
- New pure logic (reuses #124 `first_pane_of_kind` + the #115/#116 git adapters, unchanged). The finder/other
  openers (#128).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — ⌘⇧C opens/focuses a single Git pane (no `git_panel_open` toggle); the git-write surface stays confined
  to add/restore/commit (AD-…git-write-confined…), unchanged.
- D2 — with both side-panels retired, `right_open`/`viewer_split` are removed; the center pane grid is full-width.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN a Git pane renders, it shall show Source Control (change summary + commit UI or the empty state). | live capture |
| REQ-002 (visual) | WHEN the app renders, there shall be NO right-side git side-panel. | live capture |
| REQ-003 | the git-write adapters (add/restore/commit) shall be unchanged (still confined). | code review |
| REQ-004 | gate GREEN (shim-only; git_diff cov/MSI stays 100). | gate |

## Phase Plan
- **P2** — move the git render to the pane; open_git_pane; the removals; risks.
- **P3** — implement (app.rs).
- **P3.5** — 1 critic: git-write confinement intact; no dangling git_panel_open/right_open; commit input routes.
- **P4** — gate GREEN + a LIVE capture (Git pane, no side panel).
- **P5** — docs, AAR, archive, close #125.
