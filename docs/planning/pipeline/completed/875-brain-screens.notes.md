# Rusty's Brain tab, page and Knowledge as the boards draw them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-875-rusty-brain-and-knowledge.md
- **Pipeline spec:** 875-brain-screens.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #874 (and #877, which this planning found).
- **Classification / tier:** feature, S–M. Marley crate only.
- **Recall (§18.3):** #645's vault fixture; F-659 (a `ListSubHeader` grows in a column);
  L-claude-877 (old scenarios' rail clicks are stale).
- **Discovery:** probe runs of today's screens: the Brain tab (`marley: toggle brain view`)
  already has the search, the + and the tree with counts; a page has its properties, body,
  outline, star and Edit; the Knowledge panel lists the page, Backlinks and Links flat; the local
  graph needs a page focused first.
- **Decisions:** D1 in the spec.

### Design
- `brain.rs`: `render_body` gets a `ListSubHeader::new("Vault")` (in a `flex_none` div, F-659).
- `page.rs`: `render_outline` gets an "On this page" label at its top.
- `knowledge_panel.rs`: `card(title, count, children)`; `render_page` draws Following (title,
  slug, tags), Backlinks (count, rows) and Links out (count, rows) as cards; `outgoing_row`'s
  missing link adds "not written yet".
- **File manifest:** `rusty/brain.rs`, `rusty/page.rs`, `rusty/knowledge_panel.rs`; the scenario.

## Phase 2 — Code
- **Built:** `brain.rs`: a "Vault" `ListSubHeader` (in a `flex_none` div) over the tree while no
  search shows. `page.rs`: "On this page" (XSmall, muted) at the outline column's top.
  `knowledge_panel.rs`: `card(selector, cx)`; `render_page` draws Following (its title, slug and
  tags), Backlinks (its count, the rows with their context) and Links out (its count, the rows) as
  cards; `outgoing_row`'s missing link reads its target and "not written yet", Create kept.
- **Deviation:** the scenario's first draft opened the Brain tab with `rusty: open brain`, an
  action that does not exist: the Brain tab is `marley: toggle brain view` (#678).
- **Gate:** green on the first run, 17 of 17.
- **Review:** clicks, the tag search and Create are unchanged; the headings are layout only.


## Phase 3 — Test
- **Scenario:** `script/e2e/875-brain-screens.sh` (sway): #645's three pages, `tmux_hosts` empty
  (a probe run without it listed the user's local tmux session in the run).
- `875-01-brain`: the Brain tab: the search and +, "Vault", decisions / people / projects with
  their counts, "Pick a page on the left.". REQ-001 met.
- `875-02-page`: Atlas: the properties, the body, and the outline column headed "On this page"
  (Atlas, Tasks, Code, Table). REQ-001 met.
- `875-03-knowledge`: the Knowledge panel: Following (Atlas, projects/atlas, #rust #gpui #brain),
  Backlinks 0, Links out 2 (Use Zed renderer; ideas/later "not written yet" with Create), each a
  card. REQ-002 met.
- `875-04-day`: Marley Day, the page. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md` (the Brain view, the page, the Knowledge panel),
  look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 2ee15070 (the spec's ee22b1df lapsed).
- Ticket closed; pipeline archived.
