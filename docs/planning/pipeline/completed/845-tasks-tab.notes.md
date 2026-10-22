# Rusty's Tasks tab as the Tasks board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-845-rusty-screens.md
- **Pipeline spec:** 845-tasks-tab.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #844; split while planning: TICKET-845 is ten screens,
  so this pipeline is the Tasks tab, and TICKET-874, 875 and 876 take the rest.
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** #658's Tasks tab and its scenario's stand-in Rusty; #659's decision pages.
- **Discovery:** the tab reads `list_task_groups` then `list_tasks` for the chosen list; Rusty's
  tasks carry no due date or note (`rusty-core` `UserTask`); `brain_due` and `list_memories` are
  read by the Decisions and Memory tabs.
- **Decisions:** D1–D3 in the spec.

### Design
- **`tasks_tab.rs`:** the read gains `list_tasks` per list (counts), `brain_due {days: 0}` and
  `list_memories` (kept as `side: Option<Side>`); `render_lists` draws a dot and the count;
  `render_header` adds "N open · M done"; the task list draws the open tasks, then "Done" and the
  done ones; `render_side` (new) the two cards when Rusty answered and the tab is wide enough.
  Follow Up and Open the page reuse the Decisions tab's handlers (made `pub(super)`).
- **File manifest:** `rusty/tasks_tab.rs`, `rusty/decisions_tab.rs` (shared handlers); the
  scenario (from #658's and #659's).
- **Visual check plan:** per the spec's UI proof. Not reached: a failed side read (the column
  simply does not show).

## Phase 2 — Code
- **Built:** in `tasks_tab.rs`, the read gains `read_counts` (one `list_tasks` per list, archived
  aside) and `read_side` (`brain_due {days: 0}`, `list_memories`, the three newest), kept as
  `counts` and `side`; `render_lists` gives each list a dot (`LIST_DOTS` by position) and its open
  count; `render_header` adds `open_done_line` ("N open · M done", archived aside); the task list
  draws the open tasks, then "Done" and the done ones (each keeps its index, so selection, drag
  and the menu work as before); `render_side` draws the Decisions due card (an overdue count,
  each decision's title, its decided and follow-up line, Decisions and Open the page) and the
  Memory card, when the window is at least 1300 px wide.
- **Deviations:**
  - D2's width is the window's (1300 px), not the tab's: the tab's own width is not known while
    it renders.
  - The decision card offers Decisions (opening the Decisions tab, where Follow Up lives) in
    place of the board's Follow Up: the follow-up form is bound to the Decisions tab's entity.
- **Gate:** red once on clippy (`sort_by` → `sort_by_key` with `Reverse`; `render` 107 lines, its rows now `task_rows`), fixed at source; the scenario ran again on the fixed code.
- **Review:** the side reads fail quietly (no column); the counts read one list at a time, after
  the lists.


## Phase 3 — Test
- **Scenario:** `script/e2e/845-tasks-tab.sh` (sway): the stand-in Rusty with #658's lists, two
  decision pages due on 2026-10-03 (one overdue), three memories.
- `845-01-tasks`: Home chosen: Home 4, Work 3, Someday 1, each with its dot; "Home 4 open · 1
  done"; the four open tasks, then Done with Call the plumber; the Rusty column: Decisions due, "1
  overdue", Ship the home page (decided 2026-09-20 · follow up by 2026-09-28 · overdue, in the
  warning colour) and Ask one question at a time, each with Decisions and Open the page; Memory with
  the three newest. REQ-001, REQ-002 met.
- `845-02-work`: Work chosen: "3 open · 0 done", its three tasks. REQ-001 met.
- `845-03-day`: Marley Day, the same. REQ-001, REQ-002 met.
- Not reached: the narrow window (no column), Open the page's click.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md` (the Tasks tab), look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation b417c36c.
- Ticket closed (its other screens are TICKET-874, 875, 876); pipeline archived.
