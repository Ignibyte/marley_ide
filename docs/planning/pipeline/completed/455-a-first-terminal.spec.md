---
pipeline_id: a6b051a0-81a6-49c6-8c55-9d92061e591b
ticket: docs/planning/tickets/closed/TICKET-455-a-first-terminal.md
status: Phase 4 — Complete PASS
title: A first terminal for a project opened fresh
type: feature
slice: workbench shell W6f
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/451-marley-layout-presets.spec.md, docs/marley/zed-touchpoints.md]
---

## Title
A folder project opened fresh in the Marley layout, one the user has never opened before,
starts with a terminal at its root, since the terminal is the layout's main surface. A project
opened from saved state keeps exactly what it saved, terminals or none. Zed's
`Workspace::new_local` is the only place that knows which of the two it opened, so it records
that for the hook.

## Scope
### In
- **The Zed touchpoint** (`crates/workspace/src/workspace.rs`):
  `Workspace::opened_from_saved_state() -> Option<bool>`. `new_local` records whether its
  lookup (`workspace_for_roots`) found saved state for the roots: `Some(true)` or `Some(false)`,
  set in the closure that builds the workspace. Every other way of making a workspace leaves
  `None`. It is additive, with a `// Marley:` comment on each hunk and a row in
  `docs/marley/zed-touchpoints.md`.
- **The seed** (`marley_workbench::routing`): the hook the routing already puts on every new
  workspace opens one center terminal at the project's root, through the routing's
  `open_center_terminal`, as New Terminal does. It does so when the layout is the Marley
  layout, the workspace says `Some(false)`, it has a folder, and its center has no terminal.
  The terminal takes focus as a new terminal does, unless a modal holds it, and a file opened
  with the project takes focus after it.

### Out (explicitly deferred)
- A workspace with no folder (`open_new`): nothing is seeded, since there is no root.
- A project opened from saved state with no terminal stays as saved. The gpui-era force-seed
  failure is why (`BF-claude-boot-restore-force-seeds-a-terminal-after-the-runtime-guard-dissolved-001`).
- Switching an open window to the Marley layout: the projects already open get no terminal.
- Workspaces opened by id, remote or through collaboration (`None`).

## Reference (§20)
- **Upstream Zed:** the Terminal Panel's own first terminal
  (`crates/terminal_view/src/terminal_panel.rs`). Opening or activating an empty panel creates
  one (`set_active`, `:1745-1762`; `finish_restoration`, `:293-305`), and it waits while the
  panel restores its own saved terminals (`restoring`, `:251-253`, `:283`). Marley keeps the
  rule for the center: a fresh project's empty center gets one terminal, and a restored
  project's does not. It learns which is which from `new_local`'s own lookup
  (`crates/workspace/src/workspace.rs:2194`).
- **Warp:** N/A. `docs/warp_architecture/` describes PTY spawning
  (`subsystems/03-terminal-session-core.md`), not what a new window starts with.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md`, on the
  workspace and the `MultiWorkspace`.
- **Published material:** none beyond Zed's code.
- **Code we already ship.** The sweep looked for a signal outside `crates/workspace` that
  tells a fresh workspace from a restored one, and found none:
  - `Workspace::is_restoring` is set a turn after the workspace exists, inside the task
    `load_workspace` spawns (`workspace.rs:7857-7862`), and cleared with only a `notify`
    (`:7906-7939`). No event marks a restore's end.
  - `MultiWorkspaceEvent::WorkspaceAdded` means pinned, not created
    (`multi_workspace.rs:764-773`).
  - `database_id` is `Some` for both kinds, and `SerializedWorkspace` and
    `workspace_for_roots` are `pub(crate)`.
  - Reopening any project opened before restores its saved items
    (`new_local` → `open_items` → `load_workspace`, `workspace.rs:2375-2382`, `:9255-9266`),
    even outside a session restore.

  What we reuse:
  - `App::observe_new` runs its callback as an effect after the creating update
    (`crates/gpui/src/app.rs:2973`), so the hook sees the recorded value and a workspace
    already in its window.
  - The routing's `open_center_terminal`, and `TerminalPanel::add_center_terminal`, which
    focuses the new view unless a modal is open (`terminal_panel.rs:835-879`).
- **The ledger:** `PR-claude-restore-is-a-second-constructor-001` (the restore path is a
  second constructor, so the rule reads the constructor's own lookup) and the gpui-era
  force-seed failure above.

## UI proof
UI-AFFECTING.
- **Driven tests,** through Zed's real `Workspace::new_local` with a test `AppState`:
  - a folder opened fresh in the Marley layout shows one center terminal at its root, with
    focus;
  - reopening the same folder from its saved state adds none: its saved terminal comes back
    alone, and without one the center stays empty;
  - in the Zed layout, and for a workspace with no folder, nothing is added;
  - a workspace built by `Workspace::test_new` reports `None` and gets nothing.
- **Live drive:** open a new folder in the Marley layout (`marley <dir>`) and see its terminal,
  then relaunch and see no second one; screenshot each. It needs a relaunch and keys, so it
  runs only while Chad is away from the desk; otherwise the Test phase records why.

## Locked-In Decisions
- D1 — The signal is Zed's own lookup, recorded on the workspace by `new_local` as a one-field
  touchpoint. Rejected:
  - `is_restoring`, set a turn late;
  - `WorkspaceAdded`, which means pinned;
  - `database_id`, `Some` for both kinds;
  - reading the database from Marley, which races the project's first save.
- D2 — Only a fresh folder project is seeded. A project opened from saved state keeps exactly
  what it saved, terminals or none.
- D3 — The seed is the routing's New Terminal at the project's root, from the hook the routing
  already puts on every new workspace.
- D4 — Nothing is seeded on a layout switch, or for a workspace opened another way.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a folder project with no saved state is opened in the Marley layout, it shall show one center terminal at its root, with focus unless a modal holds it | driven test through `Workspace::new_local` |
| REQ-002 | WHEN a project with saved state is opened, with or without saved terminals, no terminal shall be added | driven tests: a restore with and without a saved terminal |
| REQ-003 | WHEN a project is opened in the Zed layout, or a workspace with no folder or not through `new_local` is made, no terminal shall be added | driven tests |
| REQ-004 | `Workspace::opened_from_saved_state` shall be `Some(false)` for a workspace `new_local` opened fresh, `Some(true)` for one it opened from saved state, and `None` for any other | driven tests |
| REQ-005 | The Zed touchpoint shall be additive, carry a `// Marley:` comment on each hunk, and have its row in the touchpoint ledger | gate:16 + review |
| REQ-006 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the ledger row first, then the touchpoint, then the seed in `routing.rs`; fmt
  and clippy clean; a review of the diff.
- **P3 Test** — write and run the tests, with negative checks; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
