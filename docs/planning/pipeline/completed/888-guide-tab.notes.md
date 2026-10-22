# The guided review as the DiffReview board draws its guide — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-888-diff-review-and-turns.md
- **Pipeline spec:** 888-guide-tab.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #887.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #733's scenario: its Allow click was stale since #871 reversed the ask's
  buttons (Allow for This Session now at 1460, 929).
- **Discovery:** the guide tab already draws a title, "K of N reviewed", and sections with a
  Reviewed box, summary and chips; the diff review is Zed's project diff with Marley's notes, and
  the turns have no list of their own.
- **Decisions:** D1 in the spec.

### Design
- `guide_tab.rs`: an accent eyebrow; the count beside a 4 px bar whose fill is the share
  reviewed; each section an elevated card, its number and title then Reviewed at the right.
- **File manifest:** `guide_tab.rs`; the scenario.

## Phase 2 — Code
- **Built:** `guide_tab.rs`: the accent "Guide · from the agent" over the title; the count beside
  `progress_bar` (160 × 4 px, its fill the share reviewed, counted through `u16` so no float cast);
  each section an elevated card with its number and title, then Reviewed at its right.
- **Gate:** green on the first run, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/888-guide-tab.sh` (sway): #733's setup with the browser fixture's
  `mcp_agent` (the first run, without the fixture sourced, stopped with 127); Allow for This
  Session at (1460, 929).
- `888-01-guide`: the eyebrow, "Shout the second lines", "0 of 2 reviewed" and an empty bar, two
  cards with Reviewed at their right. REQ-001 met.
- `888-02-ticked`: "1 of 2 reviewed", the bar half full, the first card folded to its title with
  its tick. REQ-001 met.
- `888-03-day`: Marley Day, the same. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 4dcd9eba.
- Ticket closed; pipeline archived; TICKET-893 holds the diff review's header and the turns.
