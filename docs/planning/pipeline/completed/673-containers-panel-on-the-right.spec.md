---
pipeline_id: be0351c7-f969-43fe-8396-af83dfa39d35
ticket: docs/planning/tickets/open/TICKET-673-containers-panel-on-the-right.md
status: Phase 4 — Complete PASS
title: "Containers move to a panel on the right, with a button in the status bar"
type: feature
slice: the rail's ports (#614, #669, #670)
references: [docs/planning/pipeline/completed/614-container-ports-in-the-rail.spec.md, docs/planning/pipeline/completed/670-fold-the-rails-containers-list.spec.md]
---

## Title
The machine's container ports that no project holds leave the rail for a panel in the right dock,
opened from its button in the status bar. Chad, 2026-10-06: "can we just move the containers icon
down as an icon in the botton bar and it opens on the right?"

## Scope
### In
- **The panel**: `ContainersPanel` in the right dock, a Box button in the status bar (tooltip
  Containers), and `marley: toggle containers`. A header (CONTAINERS and the count) at the tab bar's
  height, then a row per port as the rail drew it: the port and its process, the URL, the
  container's name; Open in a Browser Tab, Copy URL and Stop the Container on hover and in the
  row's right-click menu; a double-click opens the URL in a Browser tab of the panel's workspace.
  None found: a line saying so.
- **Shown** only in the Marley layout with `marley.rail_containers` on; otherwise no button and no
  panel, as Rusty's Knowledge panel hides.
- **The scan** runs while the panel is active, as it does while a rail is open.
- **The rail**: the CONTAINERS section, its fold and its saved field go; an older window's blob
  with `marley_containers_open` still restores.
- **The setting's words**: its doc, `default.json`'s comment and the Settings page item say the
  Containers panel; the key stays.

### Out (explicitly deferred)
- A project's own containers, which stay under the project in the rail.
- Keys and a selection inside the panel.

## Reference (§20)
Upstream Zed — a dock `Panel` whose `icon()` puts its button in the status bar through
`PanelButtons` (`workspace/src/dock.rs`), as Zed's own panels and Marley's Fleet (#607) and
Knowledge (#646) panels do. Orca keeps ports in a right-sidebar Ports tab
(`docs/orca_architecture/05-terminal-and-workspace.md`).

### Prior art
- **Behaviour maps:** Orca's right sidebar Ports tab (`05-terminal-and-workspace.md`).
- **Published material:** none applies.
- **The code we ship:** `workspace::Panel` and `PanelButtons`; `FleetPanel` and `KnowledgePanel`
  (registration, `enabled` and `icon` returning `None` to hide); the rail's `row_card`,
  `port_snapshot`, `container_line`, `url_label`, `stop_words` and `container_refused_toast`, used
  from a child module of `rail`; `ports::watch`, `unwatch`, `container_at`, `stop_container`;
  `browser::open_url_tab`.

## UI proof
`script/e2e/673-containers-panel-on-the-right.sh` (`compositor sway`), #670's fake `docker` and
stand-in `docker-proxy` publishing port 673, with `marley.rail_containers` on. Shots:
`673-01-rail` (the rail with no CONTAINERS section; the status bar's Box button);
`673-02-panel` (after a click on the button: the panel on the right, :673 first);
`673-03-browser` (a double-click on :673's row: a Browser tab on its URL); `673-04-off`
(`rail_containers` off: no button, no panel).

## Locked-In Decisions
- D1 — **A child module of `rail`** (`rail_containers.rs`), so the panel reuses the rail's row
  helpers without widening them.
- D2 — **The setting keeps its key**, `marley.rail_containers`: renaming it would drop the users'
  value; its words follow the panel.
- D3 — **The panel opens on a click, not on start**: `starts_open` false.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail shall show no Containers section. | `673-01-rail` |
| REQ-002 | WHILE the Marley layout is on and `marley.rail_containers` is on, the status bar shall show the Containers button. | `673-01-rail` |
| REQ-003 | WHEN the button is clicked, the system shall open the Containers panel in the right dock, listing the container ports no project holds. | `673-02-panel` |
| REQ-004 | WHEN a row is double-clicked, the system shall open its URL in a Browser tab. | `673-03-browser` |
| REQ-005 | WHEN `marley.rail_containers` turns off, the system shall remove the Containers button and close the panel if it is open. | `673-04-off` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rail_containers.rs`, `rail.rs`, the crate root, the setting's words; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture note, ledger capture, close, archive,
  commit.
