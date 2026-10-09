---
pipeline_id: e52b3fe7-bb77-4bc5-9bf7-b805d28b33ee
ticket: docs/planning/tickets/open/TICKET-708-a-restored-group-starts-with-the-keys.md
status: Phase 4 — Complete PASS
title: A restored group starts with the keys
type: bug
slice: the Marley layout (docs/marley/workbench-shell.md), groups (#600, #601, #700)
references:
  - docs/planning/pipeline/completed/702-one-rail-row-for-a-thread-in-a-center-tab.spec.md
  - docs/planning/pipeline/completed/700-home-and-rusty-groups-from-the-start.spec.md
---

## Title
After a relaunch, the window's keys work without a click: the groups Marley reopens behind the
shown workspace no longer take its focus.

## Scope
### In
- A workspace added behind the shown one (a group made by the rail or reopened by
  `groups::reopen`) leaves the window's focus where it is:
  - `new_local`'s `OpenMode::Add` arm and `open_workspace_by_id`'s requesting-window branch give
    the focus back right after `MultiWorkspace::add`, before a frame (`Workspace::new` focused the
    new pane);
  - `zed::initialize_workspace` focuses a new workspace only when it is the shown one
    (`Workspace::marley_is_shown`, over `owns_window_chrome`);
  - a workspace's focus-lost listener refocuses only when it is the shown one.
- #702's rail-side `keep_focus` goes: the Zed side now keeps the focus, and `keep_focus` couldn't
  undo the focus a prompt recorded when it opened.

### Out (explicitly deferred)
- Zed's focus-lost restore target (`focus_lost_restore_target` gives the old focus's nearest
  focusable ancestor, not the old focus): upstream's behaviour, left as is.

## Reference (§20)
Upstream Zed, workspace and zed: `restore_multiworkspace` restores the window's active workspace;
`open_workspace_by_id` and `new_local` with `OpenMode::Add` add a workspace behind the shown one;
`Workspace::new` focuses its center pane, `zed::initialize_workspace` focuses every new workspace,
and every workspace's focus-lost listener refocuses its own pane. Marley keeps all of it for the
shown workspace and stops it for one added behind it. No Warp analog: Warp has no workspaces in one
window.

### Prior art
- **The code we ship:** `Window::focused`, `FocusHandle::downgrade`, `WeakFocusHandle::upgrade`,
  `FocusHandle::is_focused` (gpui); #702's `rail::keep_focus` and its `ensure_groups` use.
- **Marley:** `groups::reopen` (#601), `Rail::new`'s restore path that calls it.
- **Knowledge:** F-claude-702-a-group-made-at-start-took-the-windows-focus-001 and
  PR-claude-702-making-a-workspace-in-the-background-keeps-the-focus-001 name this class;
  L-claude-700-a-restored-folderless-workspace-starts-unfocused-001 is this ticket's symptom.

## UI proof
`script/e2e/708-a-restored-group-starts-with-the-keys.sh`, under `compositor sway`, Rusty on with
the stand-in `rusty-mcp`, so the window has Home, Rusty and a project.

Shots:
- `708-01-before-quit`: Home shown, before the quit.
- `708-02-palette`: after the relaunch, with no click, Ctrl+Shift+P: the command palette open.

## Locked-In Decisions
- **D1:** the fix is in Zed's `workspace` and `zed` crates, three small hunks with their ledger
  rows: a Marley-side give-back after the fact (#702's `keep_focus`) restored a handle taken too
  early, over a prompt that had opened since, and couldn't reach the focus a prompt recorded when
  it opened.
- **D2:** "shown" is `owns_window_chrome()`, the test Zed already uses for the window title.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley relaunches a window whose groups it reopens, the keyboard shall reach the shown workspace with no click first. | Shot 708-02 |
| REQ-002 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the three Zed hunks and their ledger rows, `keep_focus` removed from the rail; a
  review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
