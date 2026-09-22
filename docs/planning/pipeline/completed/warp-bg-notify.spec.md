---
pipeline_id: 745b769c-f486-4c17-95e7-a22f7af327e7
ticket: forge#203 (4f5fac35-0985-4986-888e-203710700ea5) · local docs/planning/tickets/open/TICKET-203-warp-bg-notify.md
aar_id: ee7be81c-460e-474d-a06f-12f1394d2c0a
status: Phase 5 — Complete PASS
title: background-pane command-finish notification (tab flash)
type: feature
milestone: M12.2
references: []
---

## Title
Signal when a long-running command finishes in a NON-focused pane (forge #203). Today the only cue a
background command completed is watching its pane. Flash the pane's TAB (success vs failure) when a
command finishes in a background pane after running past a duration threshold, so you can start a long
job, switch away, and get told when it's done.

## Scope
### In
- A pure `should_notify(pane_focused, status, elapsed_secs, threshold_secs) -> Notify` decision seam
  (cov/MSI 100): no notify when the pane is focused or the command was quick (below threshold);
  otherwise `Succeeded`/`Failed` per the exit status. Boundary defined at exactly the threshold.
- The masked shim: stamp per-pane command start times, detect the Running→Finished transition in the
  #173 pump-all-grids loop, and on a background pane's finish-past-threshold set a per-tab flash
  (reusing `flash.rs` + `block_status::exit_status_kind`), shown in the rail and cleared when the tab is
  focused (or when its countdown expires).
- Driven capture: a `sleep`-past-threshold in a background pane flashes its tab (success); a failing
  command flashes failure; a focused-pane finish and a quick command do NOT flash.

### Out (explicitly deferred)
- **The optional OS notification** (macOS `NSUserNotification`/`UNUserNotification`). The ticket says
  "optionally"; a platform adapter is unsafe/platform code that can't be headlessly drive-validated and
  drags a dependency — the tab flash is the testable core. → a follow-up ticket if wanted.
- **A user-configurable threshold setting** (`notify.threshold_secs`). A const default keeps the slice
  bounded (a new persisted setting needs its own round-trip + mutation surface). → follow-up.
- Agent panes specifically (they already carry live status badges, #66/#67); this is terminal-command
  panes. The mechanism could extend later.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — Pure `should_notify(pane_focused: bool, status: StatusKind, elapsed_secs: u64,
  threshold_secs: u64) -> Notify`** with `enum Notify { No, Succeeded, Failed }`. Marley-native types
  (`u64` secs + the existing `StatusKind`), NOT `std::time::Duration` — `Instant`/`Duration` are banned
  in pure code (determinism; the #187 "Date::now banned" stance). The shim converts.
- **D2 — Threshold boundary: `elapsed_secs >= threshold_secs` notifies** (a command that ran AT the
  threshold is "long enough"). Pinned with exact-value tests at threshold−1 (No), threshold (notify),
  threshold+1 (notify).
- **D3 — The shim stamps elapsed; the Block model is unchanged.** A terminal `Block` tracks no time
  (only agents do, #187) — the ticket's "elapsed already tracked" is inaccurate. The shim keeps a
  per-pane start `Instant` (in a `RootView` map, NOT on the serialized `PaneState`), set on the
  false→true `is_command_running()` edge; on the true→false edge it computes `elapsed_secs` and reads
  the last block's `exit_status_kind`. No `Instant`/`SystemTime` in the serialized model.
- **D4 — Per-tab flash via a `RootView` map** (reuse `Flash` + its pump-tick `tick()`); the rail render
  shows it (success vs failure styled); it clears when the tab is focused or the countdown expires.
- **D5 — Clean-room (§20).** Behavior-only; no Warp source/assets. Enum transforms + a rail badge.
- **D6 — Persistent-until-viewed badge, NOT a timed flash** (design refinement). A brief flash while
  you're away is easy to miss; a completion ● that stays on the background tab's rail row until you
  switch to that tab is the useful "did it finish?" cue. Drops the countdown → the pure surface is just
  `Notify` + `should_notify`. (REQ-005 adjusted accordingly.)
- **D7 — Elapsed via a per-pane pump-tick counter, NOT wall-clock in the model.** A `RootView`
  `HashMap<PaneId, u32>` counts 16ms pump ticks while a command runs; `elapsed_secs = ticks*16/1000`.
  Deterministic, integer, no `Instant` anywhere, no `PaneState`/serialization change, no cross-field
  borrow. Threshold const = 10s. The finish badge keys by `(proj,tab)` via `locate_pane` (two-phase
  after the grids loop); the active foreground tab's badge is pruned each tick (clear-on-view).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a command finishes in a non-focused pane after running ≥ the threshold, the system shall flag that pane's tab with a completion flash. | unit: `should_notify(false, Success, threshold, threshold)` == `Succeeded`; driven: `sleep 3` in a bg pane → its tab flashes. |
| REQ-002 | WHERE a background finish is notified, the system shall distinguish success (exit 0) from failure (non-zero/none). | unit: `should_notify(false, Success, …)` == `Succeeded` AND `should_notify(false, Failure, …)` == `Failed`; driven: `sleep 2; false` in a bg pane → failure-styled flash. |
| REQ-003 | WHEN the finishing pane is the focused pane, the system shall NOT notify. | unit: `should_notify(true, Success, huge, threshold)` == `No`; driven: a focused pane's finish → no flash. |
| REQ-004 | WHEN the command ran less than the threshold, the system shall NOT notify. | unit: `should_notify(false, Success, threshold−1, threshold)` == `No` (boundary vs REQ-001's `== threshold` → notify). |
| REQ-005 | WHEN a flagged tab is viewed (becomes the active foreground tab), the system shall clear its completion badge. | driven: switch to the flagged tab → badge gone; review: the clear-on-view at the pump-tick site. |

## Phase Plan
- **P2 Design** — pick the `should_notify` module + exact `Notify` shape; the shim's per-pane
  start-time storage (RootView map) + the Running→Finished edge detection in the pump; the per-tab flash
  map + rail render + focus-clear; the threshold const value. `cargo mutants --list` for the real set.
- **P3 Implement** — the pure fn + the shim wiring.
- **P3.5 Inspect** — critics vs the diff (the edge-detection correctness, the focused-pane test, no
  double-notify, the boundary); fix real findings.
- **P4 Validate** — the exact-value matrix + RUN it; driven captures for REQ-001/002/003/004; gate green.
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; archive; close the ticket.
