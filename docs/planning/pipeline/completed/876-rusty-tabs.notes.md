# Rusty's Memory and Skills chips as the RustyTabs board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-876-rusty-memory-skills-secrets-capture-server.md
- **Pipeline spec:** 876-rusty-tabs.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #875.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #664's and #665's fixtures, #667's secrets.
- **Discovery:** a probe run of Memory, Skills, Secrets and Capture against the RustyTabs board:
  the screens already have the board's parts; the gaps are the importance chip's place and colour
  and the skills' status chips.
- **Decisions:** D1 in the spec.

### Design
- `memory_tab.rs` `render_row`: `Chip::new(importance.label()).label_color(…)` first, the meta
  line source · day.
- `skills_tab.rs`: `status_chip(pending)` on the skill and script rows.
- **File manifest:** `rusty/memory_tab.rs`, `rusty/skills_tab.rs`; the scenario.

## Phase 2 — Code
- **Built:** `memory_tab.rs` `render_row`: the importance as `Chip::new(importance.label())`
  coloured per D1, before the category chip; the meta line source · day. `skills_tab.rs`:
  `status_chip(pending)` on each skill and script row.
- **Process slip:** these edits were made while #875's install still built in the background; it
  may have read them. The install was let finish, the edits stashed, #875 installed again from its
  commit (b93d1b4869), and the edits restored (L-claude-837's trap; the memory note now says to
  check `pgrep -af install-marley` before any crate edit).
- **Gate:** green on the first run, 17 of 17.
- **Review:** layout only; the rows' clicks and forms are unchanged.


## Phase 3 — Test
- **Scenario:** `script/e2e/876-rusty-tabs.sh` (sway).
- `876-01-memory`: four memories, each line led by its importance chip (High orange, Normal plain,
  Low muted) then its category, app or mcp, the day. REQ-001 met.
- `876-02-skills`: quick-shell `pending` (yellow) `auto`, release-notes and tidy-commits `active`
  (green), the script notes-draft `active`. REQ-002 met.
- `876-03-secrets`, `876-04-capture`: shot as they are, the board's shape already.
- `876-05-day`: Marley Day, Memory. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new beyond the memory note. Brain: `brain decide` on consultation 534d2fcc.
- Ticket closed; pipeline archived.
