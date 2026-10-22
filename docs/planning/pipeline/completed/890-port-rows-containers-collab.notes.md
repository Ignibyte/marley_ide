# Port rows, the Containers panel and Collab as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-890-port-rows-containers-collab.md
- **Pipeline spec:** 890-port-rows-containers-collab.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "lets finish up the remaining tickets besides the mac issues";
  his answer on grouping: "as the design shows".
- **Classification / tier:** feature, S. Marley crate, plus one Zed file (`collab_panel.rs`).
- **Recall (§18.3):** the port rows and the panel share `row_card` and `port_snapshot`; the
  ticket's note: a 28 px tile cannot hold a five-digit port, so the port's tile takes its own
  width. The brain had nothing.
- **Design:**
  - `RowKind::Port` (the hint tint the board's tmux and port tiles share) and a tile whose width
    follows its text.
  - The port row: title the URL label in the accent color, a line with the command, then the
    service's unit or the container's line; an end button (Open or Copy) beside the hover
    buttons, which lie left of it.
  - `ContainersPanel`: listeners grouped by engine and name; a row draws a dot, the name, the
    ports as links, and Stop; its menu opens or copies each port and stops the container.
  - `collab_panel.rs`: `MarleyCollabLook`, a unit global set in `search_looks::init`, as `MarleySearchLook` is.
- **Manifest:** `rail.rs`, `rail_containers.rs`, `search_looks.rs` (Marley);
  `crates/collab_ui/src/collab_panel.rs` and its re-export (Zed, ledger row).

## Phase 2 — Code
- `marley_rail`: `PortSnapshot::what` and `PortRow::what`.
- `rail.rs`: the URL title, `RowKind::Port` and its tile, `port_row_shown`, `port_row_buttons`'
  hover group at the end's left; `Cut::Middle` went, unused once the URL left the lines.
- `rail_containers.rs`: `by_container`, `render_container`, `container_menu`, `rail_container`.
- `collab_panel.rs` (Zed, ledger row): `MarleyCollabLook` and its card; `search_looks.rs` sets it;
  `marley_workbench` depends on `collab_ui`.
- Review: clippy's too_many_lines split the end button and the menu out; the tile first followed
  its text's width, which left the URLs out of line (fixed in Phase 3).
- Gate: `just gate-diff` 17 passed, 0 failed, after cargo-sort placed `collab_ui`.

## Phase 3 — Test
- Scenario `script/e2e/890-port-rows-containers-collab.sh` under sway.
- Probe: the tile was grey (the theme's hint) and as wide as its text, and the URL kept its `/`.
  Fixed: the theme's cyan, one width, the slash dropped.
- `890-01-port-row` (REQ-001): under `repo`, `:PORT` on a teal tile, `127.0.0.1:PORT` in the
  accent over `python3 -m http.server…`, Open; `:890` with `container postgres-890` and Copy.
- `890-02-containers` (REQ-002): `mailpit-890` one row with `:891 :892` as links, a green dot and a
  red Stop; the box's own containers follow, grouped by the address they send to.
- `890-03-collab` (REQ-003): the card, its title, its line, Sign In with GitHub.
- `890-04-day`, `890-05-collab-day`: the same in Marley Day.
- `stopped.txt`: mailpit-890's Stop asked the fake docker `stop mailpit-890`.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md` (the port row's look, the Containers panel), the look-plan
  Splits row, the `collab_panel.rs` ledger row.
- Knowledge: nothing new; no bug outside the review's own fixes.
- Brain: decided on consultation b5901b5d85304d3ca3b7203dd36312e8.
