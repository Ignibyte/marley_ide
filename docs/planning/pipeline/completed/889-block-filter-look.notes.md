# The block filter as the Workflows board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-889-launch-menu-remote-and-block-tools.md
- **Pipeline spec:** 889-block-filter-look.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #888, taken whole.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #528's filter; #527's Launch entries.
- **Discovery:** a probe run (`889-menu-and-filter.sh`): the + menu's Launch section already
  matches the board; the filter has its count in the header and its toggles loose.
- **Decisions:** D1 in the spec.

### Design
- `block_filter.rs`: `header` without the count; `controls`: the field, the count, a bordered
  group of Aa, .* and Invert, Context.
- **File manifest:** `block_filter.rs`; the scenario.

## Phase 2 — Code
- **Built:** `block_filter.rs`: `header` keeps the title and Close; `controls` puts "N of M lines"
  beside the field and Aa, .* and Invert in one bordered group before Context.
- **Gate:** green on the first run, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/889-menu-and-filter.sh` (sway).
- `889-01-menu`: the project's +, "Launch" over Dev and Docs preview. REQ-001 met (as it was).
- `889-02-filter`: "FAILED", "3 of 40 lines" beside the field, Aa, .* and Invert framed, Context,
  the three matching lines. REQ-002 met.
- `889-03-day`: Marley Day, the same. REQ-002 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 8b41d4e0.
- Ticket closed whole; "Keys on a selected block" goes to Chad as a question (where it shows).
