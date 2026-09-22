---
pipeline_id: f881ecce-df5a-41d9-9165-8bd149982a99
ticket: forge#124 (efa403e4-ef91-480f-9047-6c4f14aa217d) · local docs/planning/tickets/open/TICKET-124-code-pane.md
aar_id: 2d71d5d5-4c01-4239-98bf-8d28ce529aa7
status: Phase 5 — Complete PASS
title: the code viewer as a grid pane (M6 seq-5)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/workspace.rs (PURE: first_pane_of_kind, set_content)
  - crates/marley_app/src/app.rs (SHIM: full CodeView pane render; open_code_pane; retire the side-panel + field)
---

## Title
The code viewer becomes a real grid pane — open a file and it shows in a CodeView pane beside the terminal,
with gutter + line numbers. Retires the M5 right-side code panel (clears duplication #2).

## Scope
### In
- PURE `first_pane_of_kind(kind)` + `set_content(pane, content)`.
- SHIM: full CodeView pane render; `open_code_pane` (open-or-update a single CodeView pane); reroute the 3
  file-open callers; delete the side-panel render + the `code_view` field + its right_open condition.

### Out
- The git side-panel (that's #125). The finder ⌘↵ / general openers (that's #128, if not covered by the reroute).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — opening a file updates the existing CodeView pane if one exists (no duplicate code panes), else opens one.
- D2 — the code viewer state now lives in the pane's `PaneContent::CodeView`, not the `code_view` field
  (removed); `right_open` becomes just `git_panel_open` until #125 retires that too.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `first_pane_of_kind(CodeView)` runs on panes with a code pane, it shall return that pane's id. | unit |
| REQ-002 | WHEN `set_content(pane, c)` runs on a registered pane, it shall replace its content and return true. | unit |
| REQ-003 (visual) | WHEN a file is opened, a CodeView pane shall render it (gutter + lines), with NO right-side code panel. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on the helpers; the shim masked. | gate |

## Phase Plan
- **P2** — the two helpers; the pane render + open_code_pane; the side-panel/field removal; test plan.
- **P3** — implement (workspace.rs + app.rs).
- **P3.5** — 1 critic: field/side-panel fully retired, no dangling refs, no duplicate code panes.
- **P4** — helper tests (cov/MSI 100) + a LIVE capture (code pane, no side panel) + gate GREEN.
- **P5** — docs, AAR, archive, close #124.
