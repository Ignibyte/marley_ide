# Forge around Marley — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-843-forge-around-marley.md
- **Pipeline spec:** 843-forge-around-marley.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #842.
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** #841/#842's Forge client, mapping and stand-in.
- **Discovery (Explore):** the rail's body (`impl Render for Rail`, `render_inbox` then
  `render_rows`), its global observers (`observe_marks`); Home's layout (`MarleyHome::render`,
  `card`, `row`, `_activity` observers); the bottom row's registration (`status_buttons::init`,
  `FocusRow`); Rusty's keeper loop as the model for a polling global.
- **Decisions:** D1–D3 in the spec.

### Design
- **`forge_watch.rs`:** `ForgeWatch { snapshot: Option<ForgeSnapshot>, dismissed, keeper }`
  (Global); `init` follows `SettingsStore`: a `forge` entry starts the keeper, none drops it and
  clears the snapshot. The keeper connects through `fleet_providers::connect` and every 30 s reads
  `agent-list` (`forge::summaries`), `sprint-current`, the sprint's `ticket-list` and
  `bulletin-list` into `ForgeSnapshot { agents, sprint (number, name, days left), done, total,
  qa_failed: Vec<(number, title)>, bulletins }`, written with `update_global` only when it
  changed; a failed read keeps the last snapshot and marks it unreachable.
- **Rail:** `render_forge` beside `render_inbox`: FORGE heading, "N online", a row per agent
  (a tile with CC/CX by runtime, name, "host · ticket · state"), "Open the sprint board"
  dispatching `OpenForgeBoard`; `observe_marks` grows by one.
- **Home:** `render_forge` card: the sprint line, three counts, bulletins with Dismiss.
- **Bottom row:** `ForgeItem` (StatusItemView) after `FocusRow`: "Forge · N online · #828 QA
  failed", click → `OpenForgeBoard`; empty without a snapshot.
- **File manifest:** `forge_watch.rs` (new), `rail.rs`, `home_page.rs`, `status_buttons.rs`,
  `forge_board.rs` (`forge_entry` and its types shared), `marley_workbench.rs`; the scenario.
- **Visual check plan:** per the spec's UI proof. Not reached: Forge unreachable (the snapshot's
  mark), a bulletin's Dismiss.

## Phase 2 — Code
- **Built:** `forge_watch.rs`: `ForgeWatch` (Global: snapshot, dismissed bulletins, the watched
  entry, the keeper task); `init` follows `SettingsStore` (`forge_board::forge_entry`), starting
  the keeper for a `forge` entry and dropping it (and the snapshot) when the entry goes; `keep`
  connects through `fleet_providers::connect` and every 30 s reads `agent-list`,
  `sprint-current`, the sprint's `ticket-list` and `bulletin-list` into a `ForgeSnapshot`,
  `store`d only when it changed; a failed read keeps the last snapshot marked unreachable.
  `render_rail_section` (FORGE, "N online", a row per agent with a CC/CX tile and "host · ticket
  · state" coloured by state, Open the sprint board), `render_home_card` (the sprint line, Board,
  the three counts, the bulletins with Dismiss), and `ForgeItem` (a `StatusItemView`: "Forge · N
  online · #N QA failed", click → `OpenForgeBoard`). The rail calls the section between Needs you
  and the projects and observes the global (`observe_marks` 14 → 15); Home adds the card after
  Rusty's and observes it (`_activity` 3 → 4); `status_buttons::init` puts the item after
  `FocusRow`. `forge_board`'s `Ticket`, `Tickets`, `Sprint`, `SprintAnswer`, `forge_entry`,
  `days_left` and `TICKET_LIMIT` became `pub(crate)` for the watch.
- **Deviation:** the three renderers live in `forge_watch.rs`, so `rail.rs` and `home_page.rs`
  change by a few lines each.
- **Gate:** red once on clippy (`render_rail_section` 102 lines, its row now `rail_row`; `needless_pass_by_ref_mut` on `keep`), fixed at source; the scenario ran again on the fixed code.
- **Review:** the keeper is dropped with its task when the entry changes; the watch writes the
  global only on a change, so the rail does not redraw every 30 s for nothing.


## Phase 3 — Test
- **Scenario:** `script/e2e/843-forge-around-marley.sh` (sway): #842's stand-in grown two
  bulletins; the provider set from the start. A probe run (`STOP_AT=1`) placed the bottom row's
  item.
- `843-01-rail`: the rail's FORGE · 4 online, code-1 `forge · #826 · working` (yellow), code-2
  `agent1 · #828 · silent 7 m` (red), qa-1 (CX) `agent2 · #829 · working`, docs-1 `agent3 ·
  idle`, Open the sprint board; the bottom row's `Forge · 4 online · #828 QA failed`. REQ-001,
  REQ-003 met.
- `843-02-home`: Home's FORGE card: `Sprint 14 · 3 of 11 done · 4 days left`, Board, `2 working`,
  `1 needs you`, `1 QA failed`, the critical and the warning bulletin with Dismiss. REQ-002 met.
- `843-03-board`: the bottom row's item clicked: the Forge board tab. REQ-003 met. (The first run
  clicked where the item sits beside a terminal's focus segment; with Home focused that segment
  is empty and the item sits further left. The scenario clicks it there.)
- `843-04-day`: Marley Day, the rail's section and the bottom row. REQ-001 met.
- Not reached: Forge unreachable ("unreachable" on the heading), a bulletin dismissed.


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (Forge around Marley), look-plan W4.3 done.
- Knowledge: none new. Brain: `brain decide` on consultation 3c7ad8d4.
- Ticket closed; pipeline archived.
