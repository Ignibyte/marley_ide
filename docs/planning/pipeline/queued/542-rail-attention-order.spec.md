---
pipeline_id: a4d805c2-179a-4ca0-824c-83638919b050
ticket: docs/planning/tickets/open/TICKET-542-rail-attention-order.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The rail puts what needs Chad first"
type: feature
slice: prong 2, attention (report 01 §3 item 5, report 05 §3 item 4)
references: [docs/planning/tickets/open/TICKET-519-claude-code-events-in-the-rail.md, docs/orca_architecture/01-agents-and-sessions.md, docs/orca_architecture/05-terminal-and-workspace.md]
---

## Title
The rail orders projects, and the rows under each project, by what they need from Chad: needs
you, done and not yet seen, working, not reporting, idle. A collapsed project's header says what
its agents are doing. The order holds still while the pointer is over the rail, and a setting
keeps the window's order instead.

## Scope
### In
- `crates/marley_rail/src/marley_rail.rs` (pure): an attention class per row, in this order.
  - **Needs you:** a thread `Waiting` on a confirmation; a terminal agent #519 reports waiting on a
    permission or a question; a failed run not yet seen (a thread `Error` with its attention dot,
    a terminal agent #519 reports failed and whose terminal has not been focused since).
  - **Done and not seen:** a thread `Done` with its attention dot; a terminal whose bell or
    unread mark is set (#538's mark, when it exists).
  - **Working:** a thread `Running`; a terminal agent working, by #519's report, or for an agent
    without the plugin by today's quiet timer (`marley_agent::agent_status`).
  - **Not reporting:** a terminal agent #519 shows as `no update in N m`: a working seat with no
    event for 30 minutes while Claude Code is still in the foreground (#519's D7).
  - **Idle:** everything else: shells, seen results, quiet agents without the plugin.
- A project's class is its most demanding row's. Projects sort by class; ties keep the window's
  order, which Move Project Up and Down set. Rows under a project sort by class; ties keep today's
  order (terminals in tab order, then threads newest first). `walk`, and so the rows, the
  selection, the keyboard steps and Next and Previous Project, all follow the sorted order.
- A collapsed project's header shows its agents' counts in class order, after its name:
  `waiting`, `failed`, `done`, `working`, `not reporting` (for example `1 waiting, 2 working`);
  shells and idle agents are not counted, and a project with nothing to count shows nothing.
- The hold: `RailSnapshot` may carry a held order. While the pointer is over the rail
  (`on_hover` on the rail's root, `crates/gpui/src/elements/div.rs:1655`), the workbench keeps the
  order the rail showed when the pointer entered; a row that appears meanwhile goes to the end of
  its project, and a project to the end of the list. When the pointer leaves, the hold ends and
  the rail redraws in attention order.
- A setting in `MarleySettingsContent` (`crates/settings_content/src/marley.rs`): `rail_order`,
  `attention` (the default) or `window` (today's order). On the Marley page once #515 has added
  it.

### Out (explicitly deferred)
- The rail switcher's order and digit shortcuts (report 05 §3 item 4's switcher half):
  `switcher_rows` keeps its recency order.
- A tool step or last message under a working row (report 01 §3 item 5's other half).
- Holding the order for the keyboard: while the rail holds focus and the pointer is elsewhere,
  rows may move; the selected row stays selected, since a selection is a row's identity, not its
  place.
- Mark Unread, per-row unread for threads beyond today's dot, and a board of agents.
- "Not reporting" for Agent Panel threads: ACP reports their state live.

## Reference (§20)
Warp: N/A. Its vertical tabs list tabs with metadata (docs.warp.dev/terminal/windows/vertical-tabs/,
the once-over's item 7) and order them as the user arranges them; no attention order is
documented. Upstream Zed's Threads Sidebar, which the rail replaced, sorts a group's threads by
time (`crates/sidebar/src/sidebar.rs:1800-1804`) and marks a collapsed group with a spinner while
threads run and a warning with "N threads are waiting for confirmation" (`sidebar.rs:2409-2437`);
Marley takes the collapsed group's summary from it, as words. The order is Orca's "Smart" sort,
read from its MIT source (report 01 §2.4): `src/renderer/src/components/sidebar/smart-attention.ts`
ranks 1 needs you, 2 done within 30 minutes, 3 working, 4 unverifiable (a stale row whose PTY is
alive), 5 idle, the most demanding pane deciding the worktree's class, and
`worktree-card-agent-summary.ts` counts agents by state with `not reporting` for unverifiable.
Marley breaks ties by the window's order rather than Orca's attention time, so rows move less.

### Prior art
- **Behavior maps.** Report 01 §2.4 (the glyph set, Smart sort, the card summary), §2.3 (Orca's
  30-minute decay, `src/shared/agent-status-freshness.ts`: `AGENT_STATUS_STALE_AFTER_MS`), §3 item 5
  ("Rows that jump while the pointer is on them; hold the order while the rail is hovered");
  report 05 §2.10 and §3 item 4.
- **Published material.** None beyond Warp's vertical-tabs page above; the order is a UI policy.
- **Code we already ship.** `crates/marley_rail/src/marley_rail.rs`: `thread_status` (118),
  `thread_attention` (140), `walk` (347), `rail_rows` (500), `hidden_rows_need_the_user` (610),
  `has_attention` (636), `Selection::Project` by the group's window index (186), so sorting moves
  rows without changing any row's identity. `crates/marley_fleet/src/attention.rs` already ranks
  Error, Question and Stale for the fleet, with a stable sort and an injected clock. The rail's
  gpui side (`crates/marley_workbench/src/rail.rs`): `build_snapshot` (1734), `note_ended_runs`
  (348), the quiet timers of `note_output` (375), the Move Project menu (1084-1110),
  `render` (2140). Zed's sidebar uses `on_hover` through `cx.listener` (`crates/sidebar/src/sidebar.rs:6304`).

## UI proof
UI-AFFECTING (the rail's order and its headers). `script/e2e/542-rail-attention-order.sh`
(`compositor sway`: the pointer's hover is the point). Fixtures: three scratch repositories opened
into one window through the rail's Add Project; in each a terminal running a fake `claude`
(L-claude-480) that prints the #519 frame its trigger file names. Shots: `542-01-order` (`b`'s
agent waiting, `c`'s finished while not focused, `a`'s working: the rail lists `b`, `c`, `a`),
`542-02-collapsed` (`b` collapsed: its header reads `1 waiting`), `542-03-held` (the pointer on the
rail, `a`'s agent now waiting: the order unchanged), `542-04-released` (the pointer moved off the
rail: `a` moves up beside `b`, in window order), `542-05-not-reporting` (only with `E2E_LONG=1`,
as #519's `519-08-no-update`: `a`'s agent working and silent for 31 minutes, its row below the
working and done rows), `542-06-window-order` (`rail_order: window`: the window's order).

## Locked-In Decisions
- D1 — Five classes, Orca's: needs you, done and not seen, working, not reporting, idle. A failed
  run counts as needing Chad until he has seen it, as `marley_fleet` ranks Error first.
- D2 — Ties keep the window's order for projects and today's order for rows, so the order moves
  only when a class changes, and Move Project Up and Down still mean something.
- D3 — Only a working agent goes quiet. One waiting on Chad sends nothing until answered, so it
  stays in needs you however long it waits.
- D4 — An agent without Marley's plugin cannot say it waits: its quiet-timer "waiting" sorts as
  idle, and only its bell lifts it.
- D5 — The hold covers the whole rail while the pointer is over it, and nothing else; it keeps the
  order it last showed and adds new rows at the ends.
- D6 — The class is computed in `marley_rail`, the gpui-free crate where the rail's other decisions
  already live, and `marley_workbench` only feeds it the states and the hold.
- D7 — `rail_order: attention` is the default, since Chad asked for it; `window` keeps today's
  order for anyone who wants the rail still.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the rail order is `attention`, the system shall list projects by their most demanding row: needs you, done and not seen, working, not reporting, idle. | Shot `542-01-order` |
| REQ-002 | WHERE two projects are in the same class, the system shall keep their window order. | Shot `542-04-released` (`a` and `b` both waiting, in window order) |
| REQ-003 | WHERE the rail order is `attention`, the system shall list the rows under a project by the same classes, ties in today's order. | Shot `542-01-order` (the project with a waiting and an idle terminal lists the waiting one first) |
| REQ-004 | WHILE a project is collapsed and its agents wait, failed, finished unseen, work or do not report, its header shall show their counts in that order. | Shot `542-02-collapsed` |
| REQ-005 | WHILE the pointer is over the rail, the system shall keep the order the rail showed when the pointer entered. | Shot `542-03-held` |
| REQ-006 | WHEN the pointer leaves the rail, the system shall redraw the rail in attention order. | Shot `542-04-released` |
| REQ-007 | WHEN #519 shows a working agent as `no update in N m`, its row shall sort in the not-reporting class. | Shot `542-05-not-reporting` (`E2E_LONG=1`, 31 minutes) |
| REQ-008 | WHERE the rail order is `window`, the system shall list projects and rows in today's order. | Shot `542-06-window-order` |
| REQ-009 | WHEN rows move, the selected row shall stay selected. | Shot `542-04-released`: the selection on the same terminal after the move |

## Phase Plan
- **P1 Plan** — this spec, and the design and the test plan in the notes; `brain_ask` at
  promotion.
- **P2 Code** — the classes, the sort, the hold and the summary in `marley_rail`; the states and
  the hover in `marley_workbench::rail`; the setting and its touchpoint row; fmt and clippy clean;
  a review of the diff.
- **P3 Test** — write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_rail.md` and
  `marley_workbench.md`, ledger capture, close the ticket, archive, commit.
