# The diff review's header, the guide's Send to, plan review's tabs and a terminal's turns — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-893-diff-review-header-and-turns.md
- **Pipeline spec:** 893-diff-review-header-and-turns.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #896.
- **Classification / tier:** feature, M. Marley crate, and one call in `zed.rs`.
- **Recall (§18.3):** a survey of the seams found most of the behaviour shipped: Viewed (#731),
  findings with Keep and Drop (#730), Send Review to Agent and Copy (#522), turns with Keep and
  Undo (#509, #714, the rail's menu), plan review's version toggle (#726). Missing: a header over
  the diff, the guide's Send To, real tabs, and turns outside the rail. The brain had nothing.
- **Decisions:** D1–D3 in the spec.

### Design
- `diff_header.rs`: `DiffHeader`, a `ToolbarItemView` at `PrimaryLeft` while the active item acts
  as a `ProjectDiff`; it reads the project's active repository (branch, status entries and their
  `diff_stat`), counts the viewed files through `viewed_marks`, and has Copy Notes
  (`review_notes::copy`). `add_toolbar_items(toolbar, window, cx)` is the hook `zed.rs` calls.
- `review_notes.rs`: `send_to(target, ...)` and `copy(...)`, split from the picker's rows.
- `guide_tab.rs`: `render_send_to`: `agent_targets`, a row per agent with its state; a click
  sends the unsent notes of the workspace's project diff.
- `plan_review.rs`: `render_tabs` over the left area (Plan, Changes with its `DiffStat`) and the
  line "Version N · K of M answered"; `render_turns` in the column from `Turns::of(terminal)`,
  observing the global, with Show Diff, Keep and Undo This Turn….
- **File manifest:** `diff_header.rs` (new), `marley_workbench.rs`, `review_notes.rs`,
  `guide_tab.rs`, `plan_review.rs`, `rail.rs` (`open_turn` shared), `crates/zed/src/zed.rs` (Zed
  crate, one line, ledger row), the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001, REQ-002 | two changed files, one viewed, a finding added through the MCP tool, the project diff | `893-01-header` |
| REQ-003 | a guide opened through the MCP tool, an agent stand-in in a terminal | `893-02-guide` |
| REQ-004, REQ-005 | two plans from the stand-in, a turn recorded | `893-03-plan` |
| all | Marley Day | `893-04-day` |

## Phase 2 — Code
- Built as designed: `diff_header.rs` (`DiffHeader`, `add_toolbar_items`), `zed.rs`'s one call
  after the project diff's toolbar, `review_notes::{copy_notes, send_notes_to}`,
  `viewed_marks::viewed`, the guide's `render_send_to`, plan review's `render_tabs`,
  `render_turns` and `show_turn`, and a `Turns` observer on the plan tab.
- Found in Test and fixed: the first run panicked, "cannot read workspace::Workspace while it is
  already being updated": the toolbar hears of the active item from inside the workspace's
  update, and the header read the workspace for its repository. It holds the project now
  (F-claude-893-a-toolbar-item-read-the-workspace-001, below).
- Changed after the first shot: the header counted lines from the status entries' `diff_stat`,
  which leave out untracked files (+3 −3 beside Zed's +4 −3). It takes the diff's own
  `calculate_changed_lines` and its multibuffer's files now, so both counts agree.
- Review: Send To sends through `review_notes::deliver`, so a busy or asking agent gets the
  toast and nothing; Show Diff finds the workspace that holds the tab from inside the window's
  update (`mcp::window_workspaces`).
- Gate: `just gate-diff` 17/17 green on its first run.

## Phase 3 — Test
Scenario `script/e2e/893-diff-review-header-and-turns.sh` (sway, `mcp_agent`, a stand-in Claude
Code idle through the plugin's hook, a turn that writes `todo.txt`, two plans).
- `893-01-header`: "Uncommitted changes main · 3 files · +4 −3 · 1 of 3 viewed · Copy Notes"
  (disabled: a finding is not a note until kept), Zed's +4 −3 at the right agreeing; `other.txt`
  ticked Viewed; the finding "Twelve may stay as it is" with scenario-lint, Keep and Drop
  (REQ-001, REQ-002).
- `893-02-guide`: after Keep, the guide ends with "Send to · 1 note to send" and "Claude Code ·
  repo · ready" (REQ-003).
- `893-03-plan`: Plan and Changes +2 −1 as tabs, "Version 2" under them, the plan; the column's
  "This terminal's turns": "Turn 1 · Write the to-do list", "1 file · to review", Show Diff, Keep,
  Undo This Turn… (REQ-004, REQ-005).
- `893-04-day`, `893-05-day-header`: Marley Day; the header with Copy Notes enabled beside Zed's
  Send Review to Agent (1) (all).
- Not shot: a send from Send To and Keep or Undo from the column (they run #522's and #714's
  paths, which their scenarios show).

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (the diff review's header; the guide; plan review),
  the `zed.rs` ledger row, look-plan W5.
- Knowledge: F-claude-893-a-toolbar-item-read-the-workspace-001.
- Brain decision recorded on consultation 35800d13.
