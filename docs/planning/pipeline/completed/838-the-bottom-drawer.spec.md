---
pipeline_id: 329fa31f-8aeb-414a-aa44-89902acb7b64
ticket: docs/planning/tickets/open/TICKET-838-the-bottom-drawer.md
status: Phase 4 — Complete PASS
title: The bottom drawer: Problems and Debug
type: feature
slice: The Marley look, W3.2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
In the Marley layout the bottom dock becomes a drawer with tabs: Problems (with its count) and
Debug. Problems is Zed's project diagnostics, drawn in the drawer instead of a center tab; Debug is
Zed's debugger panel (#774 keeps it there). The bottom row's diagnostics count opens the drawer on
Problems, and the Debug button on Debug; a tab switches between them.

## Scope
### In
- A Problems panel (Marley's, in the bottom dock) that draws Zed's `ProjectDiagnosticsEditor` for
  the workspace's project.
- In the Marley layout, the bottom row's diagnostics indicator and `diagnostics: deploy` open the
  bottom dock on Problems instead of a center tab. Zed's layout keeps the center tab.
- A tab strip at the top of the bottom dock while it shows Problems or Debug: "Problems N" (errors
  and warnings) and "Debug", the shown one lit; a click shows the other.

### Out (explicitly deferred)
- Output and Tasks tabs (the ticket's decision 4: a later ticket).
- Restyling the debugger's own panes (call stack, variables, breakpoints) and the board's "Ask an
  agent about this stop" and "Fix with an agent".

## Reference (§20)
Upstream Zed: `workspace::dock` (one panel per dock, chosen by its status-bar button) and
`diagnostics` (`ProjectDiagnosticsEditor`, a center item opened by `Deploy` and the status bar's
`DiagnosticIndicator`). The drawer is the design canvas's BottomPanel board.

### Prior art
- The code we ship: `Dock::panel_index_for_persistent_name`, `activate_panel`, `active_panel`,
  `panel::<T>()`; `ProjectDiagnosticsEditor::new` (private) and `deploy`; `DiagnosticIndicator`'s
  click calls `deploy` directly, not the action; the debugger panel's persistent name `DebugPanel`;
  Marley's hooks into Zed crates as globals (`MarleyTitleBarCenter`, #834) and its own panels
  (`fleet.rs`, `rail_containers.rs`).
- Ledger: #774 (Debug keeps its dock and its button moves left, AD-claude-774), L-claude-707 (an
  opened dock takes the focus), PR-claude-773 (re-tell both panels a dock call touches).
- The brain (consultation cd6086e7): nothing on this seam.

## UI proof
Scenario `script/e2e/838-the-bottom-drawer.sh` (sway): a Cargo crate whose `src/main.rs` has a
syntax error (rust-analyzer reports it), a C program built with `gcc -g`, and `.zed/debug.json`
launching it under GDB, stopping at `main`.
- `838-01-problems`: main.rs open, the bottom row's diagnostics count clicked: the drawer with
  Problems lit and its count, the error listed, no Problems center tab (REQ-001, REQ-002).
- `838-02-debug`: the drawer's Debug tab: the debugger panel under the same strip (REQ-002).
- `838-03-session`: `debugger: start`, the GDB config: the program paused at `main` in the Debug
  tab (REQ-003).
- `838-04-day`: Marley Day, Problems again (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — The drawer is the bottom dock with a Marley tab strip drawn by a hook in `Dock::render`;
  each tab is a panel of the dock, so sizes, focus and the buttons stay Zed's.
- D2 — Problems is a Marley panel hosting Zed's own diagnostics editor (`new` made public), not a
  list of Marley's.
- D3 — Only in the Marley layout; Zed's layout keeps the diagnostics tab and a plain dock.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks the bottom row's diagnostics count in the Marley layout, Marley shall open the bottom drawer on Problems listing the project's diagnostics, in Marley Night and Marley Day. | `838-01-problems`, `838-04-day` |
| REQ-002 | WHILE the bottom dock shows Problems or Debug, it shall draw Problems (with its count) and Debug as tabs, the shown one lit, and a click on the other shall show it. | `838-01-problems`, `838-02-debug` |
| REQ-003 | WHILE a debug session runs, the drawer's Debug tab shall show it as Zed's debugger panel does. | `838-03-session` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the two Zed hooks with ledger rows, the Problems panel, the strip; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
