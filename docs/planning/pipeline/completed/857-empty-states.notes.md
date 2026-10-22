# Empty and error states as one card with one action — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-857-empty-and-error-states.md
- **Pipeline spec:** 857-empty-states.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next look row once #890 waited on Chad (the Containers grouping) and
  #856 on a language server in the run.
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** an Explore pass listed each state's render site and text today.
- **Decisions:** D1, D2 in the spec.

### Design
- `states.rs`: `EmptyState::new(glyph, title, body).broken().action(label, role, on_click)`,
  `render(id, cx)`.
- The six places swap their muted line for the card; the actions dispatch the existing actions or
  call the view's own function through a weak handle.
- **File manifest:** `states.rs` (new), `marley_workbench.rs`, `home_page.rs`, `rail.rs`,
  `threads_page.rs`, `browser.rs`, `forge_board.rs`, the scenario.

## Phase 2 — Code
- **Built:** `states.rs` (new): `EmptyState` (glyph tile, title, line, one #870 button; `broken`
  gives the tile and border the error's tint). Used by `home_page.rs` (no recent projects: Open a
  Folder), `rail.rs` (the Threads view: New Agent through `new_agent_picker`, or Clear the search;
  the filter: Clear the filter; the Harness section when its state is an error: Open Settings),
  `threads_page.rs` (`render_empty`: New Agent through `agent_ui::NewThread`, or Clear the
  search), `browser.rs` (`render_message` draws a failed start as the card with Open the guide),
  `forge_board.rs` (`render_standing`: no provider with Open Settings, a failed read with Retry
  calling `refresh`).
- **Review:** each action is an existing action or the view's own function through a weak handle;
  clippy asked for `Self::refresh` in place of a closure and a clone of the harness's line before
  the header takes it.
- **Gate:** green on the first run, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/857-empty-states.sh` (sway): `MARLEY_CHROMIUM` at a missing file,
  `marley.harness` at a missing command, no Forge provider.
- `857-01-filter`: the rail filtered for "dpkg": "Nothing matches “dpkg”", the line, Clear the
  filter. REQ-001 met.
- `857-02-threads`: the Threads view searched for "dpkg": the card and Clear the search. REQ-001
  met. Its "No conversations yet" card is not shot: the run's copy of the database keeps one Rusty
  thread (#877's finding).
- `857-03-browser`: "Chromium did not start", the reason, Open the guide, in the error's tint; the
  rail's harness card beside it. REQ-001 met.
- `857-04-forge`: "No Forge provider", the setting, Open Settings. REQ-001 met.
- `857-05-day`: Marley Day. REQ-001 met.
- Not shot: Home's no-recent-projects card (the run's copy of the database has recent projects).


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W6 line.
- Knowledge: none new. Brain: `brain decide` on consultation 03949cea.
- Ticket closed; pipeline archived; TICKET-894 holds Rusty dropped and a missing agent CLI.
