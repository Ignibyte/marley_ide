---
pipeline_id: 6f0a5221-b71c-4146-8012-cdcad162fba6
ticket: docs/planning/tickets/closed/TICKET-818-drag-a-thread-to-another-group.md
status: Phase 4 — Complete PASS
title: A thread's row drags to another group
type: bug
slice: the Marley layout's rail (#602 drag, #613 move, #734 threads in tabs)
references: []
---

## Title
A terminal's row dropped on another group's header moves the terminal there (#613); a thread's
row only reorders inside its own group. Chad dragged a Rusty thread toward Default and nothing
took it. Every thread's row should move the same way: its tab goes to the group's workspace,
which since #734 is where the rail lists it, and its agent keeps the folder it works in.

## Scope
### In
- `DraggedRailRow::moves_to` takes a thread from another group as it takes a terminal; the
  header's drop moves a thread with `Rail::move_thread`.
- `move_thread`: a thread open in a tab moves that tab to the target workspace's active pane
  through `workspace::move_item`; a thread with no tab opens in a tab of the target workspace.
  The target group then shows.
- `ThreadTab::added_to_workspace` re-points the conversation's workspace handles to the new
  workspace (`ConversationView::marley_set_workspace`, a Zed hunk in `agent_ui`), so its links
  and its thread views act there; the agent keeps its project and its folder.

### Out (explicitly deferred)
- A Move to Project entry in a thread row's menu, as a terminal row's has (#613): not asked for.
- Closed groups (#617) as targets: their headers take no drop, as for terminals.

## Reference (§20)
Upstream Zed: `workspace::move_item` moves an item between panes, and across the workspaces of one
window, with its state; Zed's Threads Sidebar lists a thread by its folders and moves none.
Marley's #613 uses the same seam for terminals (`TerminalView::added_to_workspace` re-points its
workspace and project); a thread re-points its workspace only, since its agent's session belongs
to the project it started in.

### Prior art
- `crates/marley_workbench/src/rail_order.rs`: `DraggedRailRow::{takes, moves_to}`, the header's
  `can_drop` and `on_drop`.
- `crates/marley_workbench/src/rail.rs`: `move_terminal` (#613), `group_threads` (a tabbed thread
  is listed under the group whose workspace holds its tab, #734), `tab_homes`, `open_thread_with`
  and `load_thread`.
- `crates/marley_workbench/src/thread_tab.rs`: `ThreadTab` (a `SerializableItem`, so a moved tab
  comes back with its new workspace), `tab_threads`.
- L-claude-613 (lessons.md): `move_item` works across one window's workspaces, and an item that
  keeps workspace handles goes stale unless `added_to_workspace` re-points them.
- `crates/agent_ui/src/conversation_view.rs`: `ConversationView::workspace` (private) and
  `ConnectedServerState::threads`, whose `ThreadView::workspace` is public.

## UI proof
`script/e2e/818-drag-a-thread-to-another-group.sh`, under sway, with #807's set-up (the Marley
entry on a scripted agent, Rusty on through its stand-in) and `repo-b` handed to the window:
- `818-01-rusty-in-repo`: a Rusty conversation started in `repo`, its row under `repo`.
- `818-02-rusty-in-default`: its row dragged onto Default's header: the row under Default, the
  window on Default with the thread's tab (REQ-001, REQ-003).
- `818-03-answers`: a prompt sent there is answered: the conversation came along (REQ-004).
- `818-04-marley-in-repo-b`: a Marley conversation dragged from `repo` onto `repo-b`'s header:
  its row under `repo-b` (REQ-002).
- `818-05-restored`: Marley quit and started: both rows are where they were moved (REQ-005).

## Locked-In Decisions
- D1 — Every thread moves (Chad, 2026-10-19: "Every thread"), whatever its agent.
- D2 — The live view moves, not a reload: the conversation and its running turn carry on, as a
  moved terminal keeps its shell. Its agent keeps its project and folder; only the workspace the
  view acts in changes.
- D3 — A thread with no tab opens in a tab of the target group, which is then where it is listed.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user drops a Rusty thread's row on the Default group's header, the rail shall list the thread under Default. | Shot `818-02-rusty-in-default` |
| REQ-002 | WHEN the user drops an agent thread's row on another project's header, the rail shall list the thread under that project. | Shot `818-04-marley-in-repo-b` |
| REQ-003 | WHEN a thread moves, the window shall show the target group with the thread's tab. | Shot `818-02-rusty-in-default` |
| REQ-004 | WHEN a moved conversation gets a prompt, its agent shall answer in it. | Shot `818-03-answers`; `expect` on the agent's log |
| REQ-005 | WHEN Marley starts again, a moved thread shall be listed under the group it was moved to. | Shot `818-05-restored` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the drag, `move_thread`, the re-point (a ledger row); a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the scenario on the debug build, every shot read.
- **P4 Complete** — CHANGELOG, the crate note, the ledger, close, archive, commit.
