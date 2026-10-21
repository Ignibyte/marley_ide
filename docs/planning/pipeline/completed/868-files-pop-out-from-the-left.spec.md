---
pipeline_id: 17a0b22f-e836-4c7e-a81a-20f3813afb32
ticket: docs/planning/tickets/open/TICKET-868-files-pop-out-from-the-left.md
status: Phase 4 — Complete PASS
title: Files pops out from the left
type: feature
slice: The Marley look (Chad's ask, 2026-10-21)
references: [docs/marley/look-plan.md]
---

## Title
Chad, 2026-10-21: "the project panel shouldnt be in the main tab it should be a left hand nav item
you can view that pops out instead." In the Marley layout the project panel leaves the center
tabs (#747) and the Inspector (#837): a Files button in the rail's header pops the file tree out
in the left dock, between the rail and the editor, and the button, its key or Escape's dock close
puts it away.

## Scope
### In
- The Marley layout puts the project panel in the left dock (a default, as it does the Agent
  Panel's; Zed's layout gets the user's own back).
- The project panel is no longer a moved panel (#747): its button, Ctrl+Shift+E and Reveal in
  Project Panel open the left dock on it; no Files tab opens, and an open one closes.
- The Inspector (#837) holds Changes and Outline; Files left it.
- A Files button in the rail's header, before Add Project, toggles the tree.

### Out (explicitly deferred)
- An overlay that slides over the editor and closes on a click away (the ticket's default decision
  is the dock, pending Chad's answer).

## Reference (§20)
Upstream Zed: the project panel is a left-dock panel by default (`project_panel.dock`), toggled by
`project_panel::ToggleFocus`; Marley's #747 moved it into a center tab, which this undoes for it.

### Prior art
- The code we ship: `panel_tabs::MOVED`, `inspector::INSPECTED`; `apply_defaults` and
  `LayoutState` (the Agent Panel's dock default per layout, #460/#696); the rail header's buttons
  (`render_header`, `rail.rs:4820`); an action built by name (`marley::OpenComponentLibrary`).
- Ledger: L-claude-707 (the rail is not the left dock, which is empty in a fresh profile),
  AD-claude-773 (no `project_panel` dependency for one lookup).
- The brain (consultation 0da97eab): nothing on this seam.

## UI proof
Scenario `script/e2e/868-files-pop-out-from-the-left.sh` (sway): a repository with a source file.
- `868-01-closed`: the rail's header with the Files button, no tree (REQ-003).
- `868-02-open`: the Files button: the tree beside the rail, the editor still in view, no Files tab
  (REQ-001, REQ-003).
- `868-03-key`: the button again, then Ctrl+Shift+E: the tree back in the left dock (REQ-002).
- `868-04-inspector`: `marley.side_inspector` on, `git panel: toggle focus`: the Inspector with
  Changes and Outline only (REQ-004).
- `868-05-day`: Marley Day, the tree out (REQ-001).

## Locked-In Decisions
- D1 — The tree pops out in Zed's left dock (resizable, stays until closed), pending Chad.
- D2 — Only the project panel changes; Git, Outline and the others keep #747's tabs or #837's
  Inspector.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks the rail's Files button in the Marley layout, Marley shall show the file tree beside the rail with the editor still in view, in Marley Night and Marley Day. | `868-02-open`, `868-05-day` |
| REQ-002 | WHEN the project panel is asked for by its key or Zed's other ways, Marley shall show it in the left dock and open no Files tab. | `868-03-key` |
| REQ-003 | The rail's header shall offer a Files button that opens and closes the tree. | `868-01-closed`, `868-02-open` |
| REQ-004 | WHILE the Inspector is chosen, it shall offer Changes and Outline only. | `868-04-inspector` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the dock default, MOVED and INSPECTED, the button; review; gate.
- **P3 Test** — the visual check above.
- **P4 Complete** — docs (#747's and #837's text too), ledger, close, archive, commit.
