---
pipeline_id: c15dc9fb-b39b-4989-8043-ee9a740c8ce2
ticket: forge#104 (9acb6d71-1661-4727-ab57-ca07ab288f29) · local docs/planning/tickets/open/TICKET-104-agent-diff.md
aar_id: 7bea4cd9-8d1f-492b-a3d5-107bc18f573f
status: Phase 5 — Complete PASS
title: the agent-diff view
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/git_diff.rs (PURE: agent_diff_summary)
  - crates/marley_app/src/app.rs (SHIM: ⌘-click agent → diff; the overlay header summary)
---

## Title
Review what your agents changed: ⌘-click an agent in the dock's Agents section to open the working-tree
diff, headed by a `N files · +A −R` summary. The payoff of the command center.

## Scope
### In
- PURE `agent_diff_summary(files) -> String` ("N files · +A −R" / "no changes").
- SHIM: the ⌘⇧D diff overlay header uses agent_diff_summary; ⌘-click an Agents-section row opens the diff.

### Out
- Per-agent-since-launch scoping (v1 = the current working-tree diff, repo-wide). A snapshot at launch.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `agent_diff_summary` counts Add/Remove lines across all files/hunks; empty → "no changes".
- D2 — v1 shows the CURRENT working diff (the same git_working_diff #102 spawns); a per-agent snapshot is
  a follow-up.
- D3 — ⌘-click an agent row = open the diff; plain-click still focuses the pane (mirrors the tree #98).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_diff_summary(files)` runs, it shall be "N files · +A −R" (or "no changes" for empty). | unit |
| REQ-002 (visual) | WHEN an agent row is ⌘-clicked, the working diff shall open with the summary. | self-test (engine + #102 live git) |
| REQ-003 | gate GREEN, cov/MSI 100 on agent_diff_summary; the shim masked. | gate |

## Phase Plan
- **P2** — agent_diff_summary; the ⌘-click affordance + the header summary; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: agent_diff_summary MSI (empty/len/counts); the ⌘-click→diff + the header.
- **P4** — agent_diff_summary tests (cov/MSI 100) + gate GREEN.
- **P5** — docs, AAR, archive, close #104.
