---
pipeline_id: 77120848-37b8-4da1-80e6-3834c650a07f
ticket: docs/planning/tickets/open/TICKET-837-side-panels-as-one-inspector.md
status: Phase 4 — Complete PASS
title: Side panels as one inspector
type: feature
slice: The Marley look, W3.1 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Files, Changes and Outline can share one narrow column at the right of the editor, the Inspector,
with a switcher at its top, instead of opening as a full-width tab each. It is a choice, Use the
Inspector (`marley.side_inspector`), beside #747's panel tabs, which stay the default.

## Scope
### In
- A setting `marley.side_inspector` (default off), on the Marley settings page, and an action
  `marley: toggle inspector` that flips it.
- With it on, in the Marley layout, every way the Files, Git or Outline panel is asked for (the
  status bar's buttons, their keys, Reveal in Project Panel) opens or brings forward one Inspector
  item: a pane split to the right of the active pane, about 320 px wide, showing that panel.
- The Inspector's top: a switcher (Files, Changes, Outline) that changes the panel it shows. Its
  tab reads "Inspector".
- Turning the setting on closes the open Files, Git and Outline tabs; turning it off, or choosing
  Zed's layout, closes the Inspector. Fleet, Knowledge and Containers keep their tabs either way.

### Out (explicitly deferred)
- The Inspector after a restart (it is not saved with the workspace; a click opens it again).
- A count beside Changes (the board's "Changes 1"); the Git panel shows its own count.

## Reference (§20)
N/A — Marley-specific. The panel tabs are Marley's own (#747, `panel_tabs.rs`); Zed draws these
panels in its docks, and the Inspector is the design canvas's SidePanels board.

### Prior art
- The code we ship: `panel_tabs.rs` (`dock_changed`, `show_tab`, `retell`, `close_tabs`,
  `workspace_panel`, `PanelTab`'s render and its kept focus handle); `Workspace::split_pane`,
  `Workspace::resize_pane` (the active pane, by an amount, when no dock has the focus);
  `marley.rail_shrunk` and `toggle_rail_strip` (#830) for a Marley setting with an action and a
  settings-page row.
- Ledger: PR-claude-747 (an item hosting a view answers from values it kept), PR-claude-773 (re-tell
  every panel a dock call touched), F-claude-747's drop target, AD-claude-773 (panels by name).
- The brain (consultation 6abe7936): nothing on this seam.

## UI proof
Scenario `script/e2e/837-side-panels-as-one-inspector.sh` (sway): a repository with `src/main.rs`
committed and then changed; `marley.side_inspector` on.
- `837-01-files`: `main.rs` open, the status bar's Files button: the Inspector at the right, the
  editor still beside it, its switcher with Files lit, the file tree under it (REQ-001, REQ-002).
- `837-02-changes`: Changes in the switcher: the Git panel in the column, `src/main.rs` changed
  (REQ-002).
- `837-03-outline`: Outline: `main.rs`'s symbols (REQ-002).
- `837-04-tabs`: the setting off: the Inspector gone; the Files button opens #747's Files tab
  (REQ-003).
- `837-05-day`: Marley Day, the setting on, the Inspector on Files (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — The Inspector is a center item in a pane of its own, split to the right, so it moves, splits
  and closes as any tab does (#747's rule: everything opens in the main pane). Not a dock.
- D2 — One Inspector per group; asking for any of its three panels brings it forward on that panel.
- D3 — Off by default; #747's tabs stay as they are with it off (the ticket's default decision).
- D4 — The Inspector keeps each panel's focus handle from when it was made (PR-claude-747), and
  `retell` counts it as showing its current panel (PR-claude-773).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Use the Inspector is on, WHEN the user asks for Files, Changes or Outline, Marley shall show that panel in one narrow column at the right of the active pane, in Marley Night and Marley Day. | `837-01-files`, `837-05-day` |
| REQ-002 | The Inspector shall offer Files, Changes and Outline at its top, and show the one picked. | `837-01-files`, `837-02-changes`, `837-03-outline` |
| REQ-003 | WHILE Use the Inspector is off, the panels shall open as #747's tabs, and no Inspector shall stay open. | `837-04-tabs` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the setting and action, the Inspector item, the dock observer's choice, the closes;
  a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
