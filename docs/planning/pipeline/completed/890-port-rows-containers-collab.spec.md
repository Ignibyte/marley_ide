---
pipeline_id: ed80de1a-1a11-4ce3-96f9-828db4dc96a8
ticket: docs/planning/tickets/open/TICKET-890-port-rows-containers-collab.md
status: Phase 4 — Complete PASS
title: Port rows, the Containers panel and Collab as the board draws them
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
The rail's port rows, the Containers panel and Collab's sign-in as the Splits board draws them:
a port row leads with its port on a tile, reads its URL as a link over what serves it, and ends
with Open or Copy; the Containers panel lists a row per container with its ports and Stop; a
signed-out Collab panel is a card saying what an account is for, with Sign In with GitHub.

## Scope
### In
- `rail.rs`: the port row's tile, its two lines and its end button; the row's hover buttons stay.
- `rail_containers.rs`: one row per container (Chad, 2026-10-23), its ports as links, Stop.
- `collab_panel.rs` (Zed): a look hook Marley sets so the signed-out panel is the board's card.

### Out (explicitly deferred)
- Naming the terminal that serves a port ("cargo run in repo — bash" on the board): Marley does
  not map a listener's process to a terminal, and doing so is new behaviour. The row reads the
  process's command line.
- Telling a web port from another by probing it: the end button follows the board's own split,
  Open for a process's or a service's port and Copy for a container's.

## Reference (§20)
N/A — Marley-specific: the rail's port rows (#521, #603, #614), the Containers panel (#673), and
Zed's own Collab panel (`collab_ui`), whose sign-in action is kept as is.

### Prior art
- `Rail::render_port_row`, `port_row_buttons`, `PortMenu`; `row_card` and `RowKind`'s tints
  (#827); `tab_tiles::tile` (#883) draws a glyph on a tile.
- `ContainersPanel::render_row` draws a row per port with `port_snapshot`; `Ports::containers`.
- `CollabPanel::render_signed_out` (Zed) draws the blurb and the Sign In with GitHub button; the
  look hooks of #891 and #899 (`MarleySearchLook`, `MarleyCodeActionsLook`) are the pattern.
- The brain (consultation b5901b5d): nothing on this seam.

## UI proof
Scenario `script/e2e/890-port-rows-containers-collab.sh` (sway): a scratch project with a
`python3 -m http.server` in its terminal; a stand-in `docker` on Marley's PATH whose `ps` names
`postgres-890` (Compose folder the project) and `mailpit-890` (no project, two ports), their
command lines held by stand-in `docker-proxy` processes.
- `890-01-port-row`: the rail's port row under the project: `:PORT` on its tile, the URL as a
  link over the command line, Open at its end (REQ-001).
- `890-02-containers`: the Containers panel: one row for the container, its two ports, Stop
  (REQ-002).
- `890-03-collab`: the Collab panel signed out: the card (REQ-003).
- `890-04-day`: the rail and the panel in Marley Day (REQ-001, REQ-002).
- `890-05-collab-day`: Collab in Marley Day (REQ-003).

## Locked-In Decisions
- D1 — The Containers panel groups by container (Chad, 2026-10-23); a port's link opens it in a
  Browser tab, and Stop stops the container.
- D2 — The port row's visible button is Open for a process's or a service's port and Copy for a
  container's, as on the board; Open, Copy and Stop stay on hover.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | A rail port row shall show its port on a tile, its URL in the accent color over what serves it, and an Open or Copy button, in Marley Night and Marley Day. | `890-01-port-row`, `890-04-day` |
| REQ-002 | The Containers panel shall list one row per container with its ports as links and a Stop button. | `890-02-containers`, `890-04-day` |
| REQ-003 | WHILE signed out, the Collab panel shall draw a card titled "Work with others in Zed", a line on what the account is for, and Sign In with GitHub. | `890-03-collab`, `890-05-collab-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
