---
pipeline_id: f0a52485-6ebb-44c5-a228-6f8eaa259a73
ticket: docs/planning/tickets/closed/TICKET-893-diff-review-header-and-turns.md
status: Phase 4 — Complete PASS
title: The diff review's header, the guide's Send to, plan review's tabs and a terminal's turns
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
Over Zed's project diff, a header names what the diff holds (uncommitted changes, the branch, the
files, +/−, how many are viewed) with Copy Notes beside Zed's Send Review to Agent; the review
guide ends with Send To, a row per agent that can take the review; plan review shows Plan and
Changes as tabs over the plan with its version line; and plan review's column lists this
terminal's turns with Show Diff, Keep and Undo This Turn….

## Scope
### In
- `diff_header.rs` (new): a toolbar item, at the toolbar's left while a project diff is active.
- `crates/zed/src/zed.rs`: one call in `initialize_pane` that adds Marley's toolbar items.
- `review_notes.rs`: the send to one chosen agent, and Copy, callable without the picker.
- `guide_tab.rs`: Send To.
- `plan_review.rs`: the tabs and version line; the turns section.

### Out (explicitly deferred)
- The branch diff's header ("Changes since main"): the project diff shows uncommitted changes,
  and its header says so.
- Notes and findings with Keep and Drop: #730 draws them in the diff already; this ticket shoots
  them.

## Reference (§20)
N/A — Marley-specific: Marley's review notes (#522), Viewed (#731), findings (#730), the guide
(#733, #888), plan review (#726) and turns (#509, #714).

### Prior art
- Zed's `ProjectDiffToolbar` (`git_ui::project_diff`), a `ToolbarItemView` at the toolbar's right;
  `initialize_pane` in `zed.rs` adds every toolbar item.
- Zed's status entries carry `diff_stat` (`git::status::DiffStat`); `Repository::branch`.
- `viewed_marks.rs` (#731), `review_notes.rs` (#522), `send_selection::agent_targets`.
- `turns::Turns::of`, `turn_undo::{keep, undo}`, the rail's `open_turn` (#509, #714).
- The brain (consultation 35800d13): nothing on this seam.

## UI proof
Scenario `script/e2e/893-diff-review-header-and-turns.sh` (sway): a scratch repository with two
changed files, a review note on one, an agent stand-in in a terminal, a guide, a plan and a turn.
- `893-01-header`: the project diff with the header, a file viewed, and a finding with Keep and
  Drop (REQ-001, REQ-002).
- `893-02-guide`: the guide's Send To with the agent (REQ-003).
- `893-03-plan`: plan review's tabs and version line, and the turns section (REQ-004, REQ-005).
- `893-04-day`: Marley Day (all).

## Locked-In Decisions
- D1 — The header shows on the project diff only, and names it "Uncommitted changes".
- D2 — Send To in the guide sends the unsent review notes, as the picker's row for that agent
  does; with none it says so.
- D3 — The turns in plan review are the plan's terminal's, newest first, as the rail lists them.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a project diff is the active item, its toolbar shall show "Uncommitted changes", the branch, the files, lines added and removed, how many files are viewed, and Copy Notes. | `893-01-header`, `893-04-day` |
| REQ-002 | An agent's finding in the diff shall show its source with Keep and Drop. | `893-01-header` |
| REQ-003 | The review guide shall end with Send To: a row per agent that can take the review, which sends the unsent notes. | `893-02-guide` |
| REQ-004 | WHEN a plan has an earlier version, plan review shall show Plan and Changes (with lines added and removed) as tabs over it, and the version and answered count on one line. | `893-03-plan` |
| REQ-005 | Plan review's column shall list its terminal's turns, each with its files, Show Diff, Keep and Undo This Turn…. | `893-03-plan` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
