# The Component Library as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-849-components.md
- **Pipeline spec:** 849-components.spec.md

## Phase 1 — Plan
- **Request:** taken by name ahead of TICKET-878 (whose screens need a harness to show).
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #799's library, #801's templates.
- **Discovery:** a probe run: the library already has its title, line, Try it, the groups with
  each component drawn beside its JSON, and the templates; the board adds the navigation column.
- **Decisions:** D1 in the spec.

### Design
- `component_library.rs`: a `find` single-line editor (its edits notify); `matches(entry,
  query)`; `render_navigation` (220 px: the field, each group's heading and names with a match, a
  name's click sets the field to it); the cards filtered the same way; the page an `h_flex` of
  the column and the scrolling main.
- **File manifest:** `component_library.rs`; the scenario.

## Phase 2 — Code
- **Built:** `component_library.rs`: a `find` single-line editor (its edits notify), `query`,
  `matches` (D1), `render_navigation` (the field, each group with a match under its heading, the
  names in the buffer font, a click setting the field to the name); the page an `h_flex` of the
  column and the scrolling main, whose group cards follow the same filter; Try it and the
  templates unchanged.
- **Gate:** green on the first run, 17 of 17.
- **Review:** the scratch box and its parse are unchanged; the column scrolls on its own.


## Phase 3 — Test
- **Scenario:** `script/e2e/849-components.sh` (sway).
- `849-01-library`: the column (Find a component; Layout: row, column, card, divider; Text; Data;
  Charts; Status) beside the library's title, Try it and the Layout cards. REQ-001 met.
- `849-02-found`: "stat" typed: the column shows Data · stat, the main the stat card alone, then
  the templates. REQ-002 met.
- `849-03-day`: Marley Day, the same. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 0b49be4f.
- Ticket closed; pipeline archived.
