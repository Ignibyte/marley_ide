---
pipeline_id: b2afe478-3f5c-47e4-b19a-8c6cc94f5bc6
ticket: docs/planning/tickets/open/TICKET-442-rail-persistence.md
status: Phase 4 — Complete PASS
title: Rail persistence
type: feature
slice: workbench shell W6a
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/438-marley-layout-and-rail.spec.md, docs/planning/intake/rail-internals.md]
---

## Title
A rail Chad closes stays closed across restarts, and one rail width holds across restarts and
across a switch to Zed's sidebar and back. Today the Marley layout opens the rail whenever a
window is built, so a close is forgotten, and the rail's width is never saved. Narrowed at
promotion (2026-09-23): W6's other features are TICKET-451 to TICKET-454.

## Scope
### In
- **The rail's fields in the saved sidebar blob.** While the rail stands in, it answers
  `serialized_state` with the kept Zed sidebar's blob, or the blob restored into a window that
  opened in the Marley layout (#438). It now adds its fields to that JSON object rather than
  replacing it: `width` and `width_set_by_user`, the names Zed's sidebar reads, and
  `marley_rail_closed`, which Zed's sidebar ignores.
- **Closed-rail memory.** The rail notes whether the user closed it when the
  `MultiWorkspace` notifies, and only while the sidebar can be shown (AI on). A restored window
  whose blob says closed closes the rail again after the restore, deferred out of the
  `MultiWorkspace` update the restore runs in.
- **One width.** A width set on the rail is forwarded to the kept Zed sidebar and written into
  the blob a fresh Zed sidebar is given. A rail built over a kept Zed sidebar starts at that
  sidebar's width. A restored window's rail takes the blob's width when the user set it.

### Out (explicitly deferred)
- The rest of W6: TICKET-451 (Zed's layout presets, the right dock across a switch, a first
  terminal), TICKET-452 (rename and close), TICKET-453 (keyboard, filter, reorder),
  TICKET-454 (the switcher).
- The rail's internals in `docs/planning/intake/rail-internals.md`: among them the partial
  state saved during a restore, the swap's subscription pairs and the telemetry event.
- A silent close: Zed has none, so the restore's close records Zed's "Sidebar Toggled" event.

## Reference (§20)
- **Warp:** a window's layout comes back as it was left: Warp keeps its windows, tabs and panels
  in its local database between launches (`docs/warp_architecture/subsystems/06-platform-settings-infra.md`,
  the `persistence` crate's `windows`, `tabs` and `panels` tables; behavior only).
- **Upstream Zed:** the `MultiWorkspace` saves `sidebar_open` and an opaque sidebar blob per
  window and restores them in `apply_restored_multiworkspace_state`
  (`crates/workspace/src/workspace.rs:10285-10352`); Zed's sidebar keeps its width in that blob
  (`crates/sidebar/src/sidebar.rs:119-133`, `:7856-7882`). Marley keeps Zed's fields and adds
  one of its own, so either sidebar reads what the other wrote.

### Prior art
- **Behavior maps:** the Warp persistence map above; `docs/zed_architecture/subsystems/07-workspace-panes-palette.md`
  (the window's persistence).
- **Published material:** none beyond Zed's code; the blob is internal to Zed.
- **Code we already ship.**
  - `MultiWorkspace::serialize` spawns the write, and `serialize_now` reads the sidebar's
    `serialized_state` in a later turn, inside the `MultiWorkspace`'s update
    (`crates/workspace/src/multi_workspace.rs:1443-1470`).
  - `close_sidebar` and `apply_open_sidebar` both serialize and notify (`:487-537`); the rail
    already observes the `MultiWorkspace` (`crates/marley_workbench/src/rail.rs`, `Rail::new`).
  - `apply_restored_multiworkspace_state` only opens the sidebar (`restore_open_sidebar`) and
    then restores the blob inside a `MultiWorkspace` update (`workspace.rs:10336-10352`).
  - The drag handle calls `set_width` on each move without saving; the width goes out with the
    next save, the quit flush included (`multi_workspace.rs:2155-2168`, `flush_serialization`).
  - Zed's `SerializedSidebar` has `width: Option<f32>`, `width_set_by_user: bool` and
    `active_view`, all `#[serde(default)]`, and no `deny_unknown_fields`
    (`sidebar.rs:119-133`); its `set_width` touches nothing but the sidebar (`:7813-7820`).
  - `serde_json::Value` for merging the blob; `workspace::MultiWorkspaceState` has public
    fields, so a test drives the real restore.

## UI proof
UI-AFFECTING.
- **Driven tests:** close the rail and read its blob; restore that blob through
  `apply_restored_multiworkspace_state` into a fresh Marley window and the rail is closed, and
  a blob left open keeps it open; set the rail's width, switch to Zed's sidebar and back, and
  the width holds on both sides; a restored blob's width sizes the rail; with AI off no close
  is recorded. Unit tests for the blob's merge and read.
- **Live drive:** close the rail, quit, relaunch: the rail stays closed; drag it wider, quit,
  relaunch, switch layouts: the width holds. It needs clicks, so it runs only while Chad is
  away from the desk; otherwise the Test phase records why.

## Locked-In Decisions
- D1 — The rail adds `width`, `width_set_by_user` and `marley_rail_closed` to Zed's blob and
  keeps every other field, so Zed's sidebar still restores its own state
  (PR-claude-a-swapped-out-zed-entity-is-kept-not-dropped-001).
- D2 — The rail learns its open state from the `MultiWorkspace`'s notify, outside any update;
  `serialized_state`, which runs inside the `MultiWorkspace`'s update, reads only the rail's own
  fields.
- D3 — A restore's close is deferred out of the update that restores the blob.
- D4 — One width: the rail forwards its width to the kept Zed sidebar, and a rail built over one
  starts at its width.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user closes the rail, the window's saved sidebar blob shall record it, and WHEN that window is restored in the Marley layout, the rail shall be closed | driven test through `apply_restored_multiworkspace_state` |
| REQ-002 | WHEN a window whose rail was open is restored, the rail shall stay open | driven test |
| REQ-003 | WHILE AI is disabled, the rail shall record no close | driven test |
| REQ-004 | The rail's blob shall keep every field of the Zed sidebar's blob it stands in for, and Zed's sidebar shall restore from it | unit tests; driven test through a switch to Zed |
| REQ-005 | WHEN the user sets the rail's width, that width shall hold after a switch to Zed's sidebar and back, and after a restore | driven tests |
| REQ-006 | WHEN Zed's sidebar has a width set by the user, a rail built over it shall start at that width | driven test |
| REQ-007 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the blob, the open-state tracking, the deferred close and the width in
  `rail.rs`; fmt and clippy clean; a review of the diff for re-entrancy in the restore and in
  `serialized_state`.
- **P3 Test** — write and run the tests; the live drive; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
