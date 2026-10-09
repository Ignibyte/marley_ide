---
pipeline_id: 21ad12be-db6c-43fb-b55a-b39bcec8d2c3
ticket: docs/planning/tickets/open/TICKET-700-home-and-rusty-groups-from-the-start.md
status: Phase 4 — Complete PASS
title: Home and Rusty groups from the start, on top of the rail
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md) and Rusty in Marley (docs/marley/rusty-in-marley.md), on Chad's 2026-10-09 ask
references:
  - docs/planning/pipeline/completed/675-the-rusty-group.spec.md
  - docs/planning/pipeline/completed/699-the-rusty-groups-plus-menu-and-home-page.spec.md
---

## Title
Every window in the Marley layout has its Home group from the start, and its Rusty group while
Rusty is on, listed Home then Rusty at the top of the rail, above the projects.

Chad, 2026-10-09: "Marley starts a fresh install with the home panel existing, rusty existing if
its enabled. The order should be Home then Rusty." Asked top or bottom: "top correct".

## Scope
### In
- A window lacking its Home group gets one; while Rusty is on, a window lacking its Rusty group gets
  one, including when Rusty turns on later.
- A window whose shown workspace is Zed's folderless start workspace (a launch with no folder and no
  session to restore) takes that workspace as its Home group rather than making a second one.
- No group is made while a restored group of the same kind can still come back to the window, so a
  restart neither duplicates nor loses one.
- The rail lists Home, then Rusty, then the projects, then named groups; a header order the user
  dragged (#602) still wins.
- Home's header menu becomes a label, as Rusty's (#675): a group that is always remade has nothing
  to rename or remove.

### Out (explicitly deferred)
- Home's own page (#701): until then an empty Home group shows Zed's Welcome page.
- Dropping the stale group records a window that is never restored leaves in the store.

## Reference (§20)
N/A — Marley-specific: Home and Rusty are Marley's projectless groups (#600, #675) over folderless
Zed workspaces. Zed's own behavior kept: `workspace::open_new` gives a window with nothing to
restore one folderless workspace, and `restore_multiworkspace` restores a window's sidebar state
after the window is open, which is why the rule waits.

### Prior art
- **The code we ship:** `groups::{make, with_group, adopt, reopen, forget_pending}` and
  `Groups::{live, pending, waiting}` (#600, #601, #675, #676); the rail's
  `restore_serialized_state` (the saved group ids, read after the window opens), `rail_groups` and
  `marley_rail::place` (#602); `rusty_changed`; the Rusty group's header label (#675).
- **Zed:** `restore_multiworkspace` and `apply_restored_multiworkspace_state` (the sidebar state
  is applied after awaits), `open_new` for a launch with nothing to restore, `MultiWorkspace::
  project_groups`, which lists no folderless workspace (AD-600).
- **Behavior maps:** `docs/zed_architecture/` has nothing on startup groups.

## UI proof
`script/e2e/700-home-and-rusty-groups-from-the-start.sh`, under `compositor sway`, on a profile
with no database (a fresh install) and the stand-in `rusty-mcp`.

Shots:
- `700-01-fresh`: a fresh start with Rusty off: Home above the project, no Rusty.
- `700-02-rusty-on`: Rusty turned on: Home, Rusty, then the project.
- `700-03-restarted`: after a quit and a relaunch with no path: one Home, one Rusty, then the
  project.
- `700-04-no-folder`: the database removed and Marley started with no path: Home, holding the
  start workspace, and Rusty, no stray workspace.

## Locked-In Decisions
- **D1:** the rail decides when, per window: a kind's group is made only while no record of that
  kind in `Groups::pending` may still come back to the window. After the window's saved state is
  read (`restore_serialized_state`), that is when none of its own saved ids is pending; before
  it, when no record of that kind is pending at all, or once `SETTLE` (3 s) has passed since the
  rail was built (a window that is never restored).
- **D2:** a Home group takes the shown workspace when it is folderless, holds no item, is no
  group's and is not a pending record's; otherwise `with_group` makes one.
- **D3:** the order is Home, Rusty, the projects, the named groups, before `marley_rail::place`
  applies a dragged order.
- **D4:** Home's header menu is a label, as Rusty's.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a window opens in the Marley layout without a Home group, the system shall give it one, listed above the projects. | Shot 700-01 |
| REQ-002 | WHILE Rusty is on, the system shall give a window without a Rusty group one, listed after Home and above the projects, including when Rusty turns on after the window opened. | Shot 700-02 |
| REQ-003 | WHEN Marley restarts, the rail shall list one Home and one Rusty group, in that order, above the projects. | Shot 700-03 |
| REQ-004 | WHEN Marley starts with no folder and nothing to restore, the system shall make the start workspace the Home group. | Shot 700-04 |
| REQ-005 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `groups.rs` (`claim`, `is_home`, the pending checks), `rail.rs` (the rule, the
  order, Home's label); a review of the diff; the gate.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG, architecture docs, ledger, close, archive, commit.
