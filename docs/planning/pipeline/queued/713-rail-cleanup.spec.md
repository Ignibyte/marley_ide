---
pipeline_id: b9132431-1950-4c33-b5d4-84a7eaf9e936
ticket: docs/planning/tickets/open/TICKET-713-rail-cleanup.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Rail cleanup - one sidebar's subscriptions, refreshes on row changes, a silent swap, saves after the restore
type: chore
slice: the Marley layout (docs/marley/workbench-shell.md), the rail; the finish-line list's row 8 (docs/planning/design-notes/finish-line-2026-10-11.md)
references:
  - docs/planning/intake/rail-internals.md
  - docs/planning/pipeline/completed/438-marley-layout-and-rail.notes.md
  - docs/planning/pipeline/completed/442-rail-persistence.spec.md
  - docs/planning/pipeline/completed/601-projectless-groups-survive-a-restart.spec.md
  - docs/planning/pipeline/active/743-the-rails-top.spec.md
---

## Title
Four leftovers from #438's inspect, carried through #442 and the 2026-10-09 audit
(`docs/planning/intake/rail-internals.md`): a swap leaves the old sidebar's subscriptions behind,
terminal output refreshes the rail three or four times a chunk, Marley's own sidebar opens and
closes record a "Sidebar Toggled" event, and a window in the Marley layout saves itself before its
restore is done. Chad, 2026-10-11: "8.) yes to all".

## Scope
### In
- **One sidebar's subscriptions.** `MultiWorkspace::register_sidebar` keeps the registered
  sidebar's two subscriptions in a field of their own and replaces them at each call, so a swap
  ends the old sidebar's pair (Zed touchpoint, `crates/workspace/src/multi_workspace.rs:400-412`).
  Home's page holds the rail weakly and finds the window's rail again when the sidebar is another
  one (`crates/marley_workbench/src/home_page.rs:106`, `192-206`), so a swapped-out rail is
  dropped.
- **Refreshes on row changes.** A terminal's `Wakeup` and its `ItemEvent`s stamp the output time
  and build that terminal's row alone (`terminal_snapshot`, `rail.rs:7758`); the rail refreshes
  only when the row differs from the one its last refresh built. A pane's `ChangeItemTitle` and a
  workspace's `ActiveItemChanged` refresh only when that pane's tabs (ids, titles, dirty marks, the
  active one) differ from the last refresh's. The agent's quiet timer stays. Every other event
  refreshes as today. Each refresh writes a debug line naming the rail.
- **A silent swap.** Marley's opens and closes of the sidebar (a new window, a swap, a restored
  closed rail) record no "Sidebar Toggled" event: `restore_open_sidebar` becomes `pub`, and
  `close_sidebar` gets a silent twin the way `open_sidebar` has one (Zed touchpoint,
  `multi_workspace.rs:500-551`). The user's toggles keep the event.
- **Saves after the restore.** A rail built with its window holds the window's saves until its
  restore is done: every group the saved state lists is back or dropped; for a window no restore
  reaches, the startup settle (3 s) has passed; and at most 10 s after the rail was built. The hold
  is a flag on `MultiWorkspace` that `serialize` and `flush_serialization` respect (Zed
  touchpoint); the release saves once. Until then a reorder keeps the saved order of the groups
  still reopening (`rail_order.rs:445-454`).
- **Docs:** `docs/marley_architecture/marley_workbench.md` (the status and persistence notes), the
  `multi_workspace.rs` row in `docs/marley/zed-touchpoints.md`, and `rail-internals.md` marked done
  except its last item.

### Out (explicitly deferred)
- The intake's restart check of center and Terminal Panel terminals: a check, not a change.
- Refreshes from other sources (git, project and global events): only the output path is gated.
- Telemetry itself, off by default since #514.
- #743's retire step for an old Rusty group, active now; this ticket builds after it.

## Reference (§20)
Upstream Zed, the `workspace` crate's `MultiWorkspace`: one registered sidebar, its open state, the
silent `restore_open_sidebar` upstream keeps for a restore (`multi_workspace.rs:509-513`), and the
window's saved state written by `serialize`. Behavior kept: Zed's sidebar and its saved blob, the
restore order in `apply_restored_multiworkspace_state` (`crates/workspace/src/workspace.rs:10495-10512`),
and the event on the user's own toggles. The rail itself is Marley-specific (#438, #468); no Warp
or Orca behavior applies to these four.

### Prior art
- **The code we ship:**
  - Upstream already split open into the event and a silent body: `open_sidebar` →
    `apply_open_sidebar`, and `restore_open_sidebar` → `apply_open_sidebar`
    (`multi_workspace.rs:500-526`). The close gets the same shape.
  - gpui ends a subscription when its `Subscription` drops, so replacing the field ends the old
    pair; `WeakEntity` lets Home's page see a rail go.
  - `MultiWorkspace::serialize` is one replaced task (`multi_workspace.rs:1456-1463`), and
    `serialize_now` writes `project_groups` and `active_workspace_id` from Zed's own state
    (`1465-1486`), so a hold must sit in front of it; the rail's blob alone cannot fix the write.
  - The rail already redraws only on a changed snapshot (`rail.rs:1143`,
    F-claude-504-a-favicons-arrival-changed-nothing-the-rail-compares-001); the gate moves a
    smaller comparison in front of the rebuild. `terminal_snapshot` (`rail.rs:7758`) already
    builds one terminal's row on its own.
  - `StartupGroups` and `SETTLE` (`rail.rs:119`, `262-281`) time the start; `groups::reopen` and
    `forget_pending` (`groups.rs:217-262`) say when each saved group is back or dropped.
  - L-claude-572-an-observed-global-written-at-every-wakeup-redraws-the-rail-001: what a row draws
    is kept apart from what changes at every wakeup.
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` §1.7
  (`MultiWorkspace`, many workspaces per window and a switcher sidebar); nothing on these four in
  the Warp or Orca maps.
- **Published material:** none applies. Zed's docs do not describe `MultiWorkspace`'s sidebar or
  its persistence; the code is the record.

## UI proof
`script/e2e/713-rail-cleanup.sh`, under `compositor sway` (#601's group clicks), with
`ZED_LOG=marley_workbench::rail=debug` so the run's `Marley.log` counts refreshes, a scratch
project, and a stand-in agent CLI that prints for three seconds and then waits.

Shots and checks:
- `713-01-printing`: a plain terminal in the project prints 300 lines over six seconds; the shot
  mid-print shows its row running the command. Check: the refresh lines written during the print
  number ten or fewer.
- `713-02-finished`: the command ends with a bell: the row's finished state and the bell's dot.
- `713-03-agent-working` and `713-04-agent-waiting`: the stand-in agent while it prints, then two
  seconds after it stops.
- `713-05-home-after-swap`: Home's page open; `marley: use zed layout`, then `marley: use marley
  layout`; a second stand-in agent started: Home's page lists it. Check: after the swap back,
  every refresh line names one rail, not the one before the swap.
- `713-06-restored`: two groups made from the rail's empty space (#601); Marley quits and starts
  with no path; it is killed (SIGKILL) as soon as its window maps; it starts again: both groups are
  back, in their order. Check: the log of the killed launch shows the saves held and no release
  before the kill; the last launch's shows the release once the groups are back.

## Locked-In Decisions
- **D1 — Register replaces.** A `sidebar_subscriptions` field on `MultiWorkspace` is set, not
  pushed to, at each `register_sidebar`. Rejected: an `unregister_sidebar` Marley calls before each
  register (a second call to forget, and a window left with the old pair if one path misses it).
- **D2 — Home's page holds the rail weakly** and re-finds it when the window's sidebar is another
  entity. Rejected: dropping the page's observe (its agents at work would stop following the
  rail).
- **D3 — The output gate is per row.** Terminal events compare that terminal's row; pane titles and
  active-item changes compare that pane's tabs. Rejected: a throttle (it still refreshes on every
  tick while output runs and delays a real change); dropping the item-event subscription (a
  terminal's title reaches the rail only as `UpdateTab`, `terminal_view.rs:1625-1627`); a Zed
  change so a wakeup sends no `UpdateTab` (`terminal_view.rs:1565-1570`; other items rely on it).
- **D4 — The silent close mirrors upstream's silent open**: `close_sidebar`'s body moves into
  `apply_close_sidebar`, `close_sidebar` keeps the event, and a `pub` silent entry calls the body;
  `restore_open_sidebar` becomes `pub`. Rejected: a Marley copy of close's body (a Zed function
  body in a Marley crate, §20's code-layer wall).
- **D5 — The hold is a flag on `MultiWorkspace`**: `serialize` and `flush_serialization` skip the
  write while it is set and remember that one was asked; clearing it writes once. Rejected: the
  rail answering its restored blob while restoring (Zed writes the empty `project_groups` and the
  active workspace itself); opening the rail only after the restore (a fresh window would show no
  rail until the settle).
- **D6 — The release:** every saved group back or dropped, else the startup settle for a window no
  restore reaches, and at most 10 s after the rail was built (default; a remote group's reopen can
  wait on SSH). A rail built by a swap holds nothing. A quit during the hold keeps the window's
  previous saved state.
- **D7 — One debug line per refresh**, `rail <entity id> refresh`, at `debug`, as #489's scenario reads
  its own (`ZED_LOG=marley_browser=debug`).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a sidebar is registered on a window, the system shall end the previous sidebar's subscriptions on that window. | Review of the diff; the check after `713-05-home-after-swap` |
| REQ-002 | WHEN the layout swaps back to Marley's, Home's page shall follow the new rail, and the previous rail shall refresh no more. | Shot `713-05-home-after-swap` and its log check |
| REQ-003 | WHILE a terminal prints and nothing its row shows changes, the system shall not refresh the rail. | The log count with `713-01-printing` |
| REQ-004 | WHEN a terminal's row changes (its command starts or ends, a bell, its title, its folder, an agent turning working or waiting), the system shall redraw the row as before. | Shots `713-01` to `713-04` |
| REQ-005 | WHEN Marley opens or closes the sidebar for a new window, a swap or a restored closed rail, the system shall record no "Sidebar Toggled" event, and the user's own toggles shall still record it. | Review of the diff (telemetry is off, #514) |
| REQ-006 | WHILE a window's restore is not done, the system shall not write that window's saved state, and WHEN the restore is done it shall write it once. | Shot `713-06-restored` and its log checks |
| REQ-007 | WHEN the rail is reordered during the restore, the system shall keep the saved order of the groups still reopening. | Review of the diff (a reorder timed inside the restore is not reliable in a scenario) |
| REQ-008 | The change shall pass `script/gates.sh --diff`. | The gate's exit code |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, with `refresh`, `ensure_groups`
  and #743's retire step re-read.
- **P2 Code** — the ledger row first (`multi_workspace.rs`); the subscriptions field, the silent
  close, `restore_open_sidebar` public and the hold in `multi_workspace.rs`; in `marley_workbench`,
  `register_sidebar`'s silent calls, the rail's output gate, its hold and release, the reorder's
  keep, the debug line, and Home's weak rail; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run `script/e2e/713-rail-cleanup.sh`, read every shot.
- **P4 Complete** — CHANGELOG and the architecture note (§21), ledger capture (§19), close the
  ticket, archive, commit.
