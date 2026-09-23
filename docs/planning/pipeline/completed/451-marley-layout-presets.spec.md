---
pipeline_id: 63caad57-aacb-4cb4-ae32-659244fa1bd0
ticket: docs/planning/tickets/open/TICKET-451-marley-layout-presets.md
status: Phase 4 — Complete PASS
title: Zed's layout presets in the Marley layout
type: bug
slice: workbench shell W6b
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/441-marley-terminal-routing.spec.md, docs/planning/tickets/closed/TICKET-455-a-first-terminal.md, docs/planning/tickets/closed/TICKET-456-docks-across-a-layout-switch.md]
---

## Title
In the Marley layout, Zed's Panel Layout presets no longer rewrite the settings the layout
depends on. Classic writes every panel but the agent to the left, and Agentic writes
`agent.dock: left`, which overrides the Marley layout's right-hand Agent Panel for good. In the
Marley layout both now show a message with a way to Zed's layout instead. Narrowed at
promotion (2026-09-23): the docks across a round trip are TICKET-456, a first terminal
TICKET-455.

## Scope
### In
- Capture-phase listeners on the workspace's root, installed from the crate's `init`, for
  `workspace::UseClassicLayout` and `workspace::UseAgenticLayout` (declared in `title_bar`,
  which joins the crate's dependencies). In the Marley layout they stop propagation and show a
  toast: the presets belong to Zed's layout, with a button that switches to it
  (`marley::UseZedLayout`). In the Zed layout they return and Zed's handlers run.

### Out (explicitly deferred)
- The title bar's Panel Layout submenu itself: it still lists Classic and Agentic, with
  "Custom" checked, in the Marley layout. Hiding or relabelling it needs a `title_bar`
  touchpoint; choosing an entry now explains itself.
- TICKET-455 (a first terminal) and TICKET-456 (the docks across a layout round trip).

## Reference (§20)
- **Warp:** N/A for the presets themselves, which are Zed's. Warp keeps one layout and no
  preset rewrites it.
- **Upstream Zed:** the Panel Layout presets (`crates/title_bar/src/title_bar.rs:94-126`, the
  submenu at `:1403-1432`) and `AgentSettings::set_layout` (`crates/agent_settings/src/agent_settings.rs:370-396`)
  are kept as Zed has them in the Zed layout. The capture follows #441's
  (`marley_workbench::routing`).

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (the
  palette and the title bar's menus).
- **Published material:** none; the presets are internal to Zed's settings.
- **Code we already ship.**
  - The presets are workspace actions (`workspace.register_action`, `title_bar.rs:120-126`)
    that call `AgentSettings::set_layout`, which writes `agent.dock` and the other panels'
    docks to the user's settings file (`agent_settings.rs:370-396`); `get_layout` reads
    "Custom" for any mix the presets do not name (`:338-356`).
  - The title bar hides the presets from the palette only while AI is off and re-applies that
    on every settings change (`title_bar.rs:188-203`), so a Marley-side filter would be undone.
  - `Workspace::register_action_renderer` and `capture_action`, with `cx.stop_propagation()`,
    as #441 uses them.
  - `workspace::Toast` with `on_click`, shown by `Workspace::show_toast`
    (`crates/workspace/src/workspace.rs:729-760`, `notifications.rs:196`).

## UI proof
UI-AFFECTING.
- **Driven tests:** in the Marley layout, each preset dispatched from the center pane leaves
  the user's settings file and `AgentSettings::dock` untouched and shows the toast, and the
  toast's button switches to the Zed layout; in the Zed layout a stand-in for Zed's handler
  runs and no toast shows.
- **Live drive:** in the Marley layout choose Classic and Agentic from the title bar's Panel
  Layout menu; screenshot the toast and confirm the Agent Panel stays on the right. It needs
  clicks, so it runs only while Chad is away from the desk; otherwise the Test phase records
  why.

## Locked-In Decisions
- D1 — Catch the actions, not the menu, as #441 does: the palette and any binding route the
  same way, and no Zed crate changes.
- D2 — A message rather than silence: the entry exists in the menu, so choosing it says why
  nothing changed and how to get Zed's layout.
- D3 — The layout is read at each call.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the layout is `marley`, `workspace::UseClassicLayout` and `workspace::UseAgenticLayout` shall change no setting and shall show a message saying the presets belong to Zed's layout | driven test |
| REQ-002 | WHEN the message's button is clicked, the layout shall switch to `zed` | driven test |
| REQ-003 | WHILE the layout is `zed`, both actions shall reach Zed's handlers as upstream, with no message | driven test |
| REQ-004 | The diff gate shall be green, with no path outside the Marley-owned set changed | `script/gates.sh --diff`, gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the listeners and the toast in `marley_workbench`; fmt and clippy clean; a
  review of the diff.
- **P3 Test** — write and run the tests; the live drive; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
