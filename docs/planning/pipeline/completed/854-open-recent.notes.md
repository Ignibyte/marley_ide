# Open Recent as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-854-search-and-more.md
- **Pipeline spec:** 854-open-recent.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #853; split while planning: Open Recent here, the
  board's other screens TICKET-884.
- **Classification / tier:** feature, S. One Zed crate (`recent_projects`, a hook) and Marley's.
- **Recall (§18.3):** #839's palette hook; #831's project marks.
- **Discovery:** a probe run: Zed's rows are the name over the folder, no mark and no "where";
  This Window's folders and Recent Projects are two row kinds in `render_match`.
- **Decisions:** D1, D2 in the spec.

### Design
- `recent_projects.rs`: `MarleyRecentRow { root, name, opened, host }` and
  `MarleyRecentLook(fn(&MarleyRecentRow, &mut App) -> (AnyElement, Option<SharedString>))`; each
  row kind asks it and adds the mark before the match and the label after it.
- `marley_workbench`: a function in `project_icons.rs`, set as the global
  in `init`: `want` then `project_mark` at 24 px for a local root, the letter for a remote one;
  the label from the host, the window, or the days since.
- **File manifest:** `crates/recent_projects/src/recent_projects.rs` (Zed), `project_icons.rs`
  (Marley), the ledger, the scenario.

## Phase 2 — Code
- **Built:** the ledger row for `recent_projects.rs` first; there `MarleyRecentRow`,
  `MarleyRecentLook` and `marley_recent_look` (the label boxed `ml_auto`), asked by the open
  folder, This Window and Recent Projects rows, each adding `.children(marley_mark)` after Zed's
  icon and `.children(marley_place)` before the tooltip, each hunk `// Marley:`. In
  `project_icons.rs`: `letter_mark` split out of `project_mark`, `recent_look` (`want` then
  `project_mark` at 26 px for a local root, the letter for a remote one or none; "this window",
  "ssh", or the days since), set as the global in `init`.
- **Gate:** the first run failed four `SharedString` literals (`new_static` asked); fixed, the
  second run green, 17 of 17, and the scenario re-run on it.
- **Review:** without the global (Zed alone) nothing changes; the hover buttons in the end slot
  stay; `want` only starts a search the cache throttles.


## Phase 3 — Test
- **Scenario:** `script/e2e/854-open-recent.sh` (sway): `projects: open recent`; Escape closes it.
- `854-01-recent`: This Window's `repo` with its R tile and "this window"; Recent Projects with
  letter tiles and one project's own logo, each with "today" or "15 d ago" at the right. REQ-001
  met.
- `854-02-day`: Marley Day, the same with Day's tints. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), the ledger row checked, `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 6d2556b0.
- Ticket closed; pipeline archived; TICKET-884 holds the rest of the board.
