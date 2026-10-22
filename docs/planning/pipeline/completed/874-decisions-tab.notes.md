# Rusty's Decisions and Follow Up as the boards draw them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-874-rusty-decisions-and-follow-up.md
- **Pipeline spec:** 874-decisions-tab.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #845.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #659's tab, #660's form, #845's scenario fixtures.
- **Discovery:** a probe run of today's tab: the Due rows and the list already carry the loop
  line, the count, the chips and the dates at the row's end; the form already has its
  segmented outcome. The gaps are the colours, the Due card, the two-line rows and the form's
  buttons.
- **Decisions:** D1 in the spec.

### Design
- `decisions_tab.rs`: `render_row` puts the dates (`decided_line`, `follow_up_line`,
  `followed_up_line`, the successor) under the title; `render_end_slot` keeps the chip and Follow
  Up; `status_color(status)` per D1; the Due rows wrapped in a bordered card.
- `follow_up.rs`: its Cancel and Record drawn by `prompts::render_button` (Quiet, Primary).
- **File manifest:** `rusty/decisions_tab.rs`, `rusty/follow_up.rs`; the scenario.

## Phase 2 — Code
- **Built:** `decisions_tab.rs`: `render_row` draws the title over `render_dates` (decided,
  followed up, the follow-up day in the warning colour once overdue, a replaced decision's
  successor); `render_end_slot` keeps the chip, now coloured by `status_color` (D1), and Follow Up
  as an outlined button; `render_entries` gathers the Due header and rows into a bordered card,
  its border the warning one while a decision is overdue. `follow_up.rs`: Cancel (quiet,
  dismisses) and Record (primary, dimmed and inert while the form lacks an outcome or records)
  through `prompts::render_button`.
- **Gate:** red once on clippy (`unused_self` on `render_end_slot`, now an associated function; `doc_markdown` on `MoreTabs`; `status_color` made `const fn`), fixed at source; the scenario ran again on the fixed code.
- **Review:** the rows' clicks, menus and the form's keys (Enter records, Escape cancels) are
  unchanged.


## Phase 3 — Test
- **Scenario:** `script/e2e/874-decisions-tab.sh` (sway): the stand-in Rusty with six decision
  pages on 2026-10-03; a probe run placed Follow Up.
- `874-01-decisions`: the loop line; the Due card (warning border) with Ship the home page
  (decided 2026-09-20 · follow up by 2026-09-28 · overdue, in the warning colour) and Ask one
  question at a time, each with a green `decided` chip and Follow Up; "6 decisions"; each row its
  title over its dates, `kept` green, `superseded` muted with "replaced by ship-the-home-page".
  REQ-001 met.
- `874-02-follow-up`: the form: "Follow up: Ship the home page", Kept / Revised / Superseded, the
  outcome field, Cancel quiet and Record primary (dimmed: no outcome yet). REQ-002 met.
- `874-03-day`: Marley Day, the same. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md` (the Decisions tab), look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation ea57bb9d.
- Ticket closed; pipeline archived.
