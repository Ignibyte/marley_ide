# Rail persistence — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-442-rail-persistence.md
- **Pipeline spec:** 442-rail-persistence.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** the remainder of the plan's rail (workbench-shell W6): closed-rail memory,
  width, rename, close, keyboard, filter, the switcher and reorder, with items carried from
  #438's inspect and #441's promotion. The draft is in git history under
  `docs/planning/pipeline/queued/442-rail-polish.*`.

## Phase 1 — Plan (promoted 2026-09-23, after #450)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] promote, split
  and re-verify · [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Split at promotion.** The queued W6 held eight features and ten carried items. #442 keeps
  persistence, its REQ-001 and REQ-002. The rest:
  - TICKET-451 (W6b): Zed's layout presets, the right dock across a layout switch, a first
    terminal for a project shown with none;
  - TICKET-452 (W6c): rename and close;
  - TICKET-453 (W6d): keyboard navigation, the filter, project reorder;
  - TICKET-454 (W6e): the switcher;
  - `docs/planning/intake/rail-internals.md`: the git branch on a header, closed groups, the
    telemetry event, the swap's subscription pairs, the refresh cost, the restore order, the
    restart check.
  The pair and the ticket are renamed for the scope (`442-rail-persistence`).
- **Recall (§18.3).**
  - `PR-claude-a-swapped-out-zed-entity-is-kept-not-dropped-001` and
    `F-claude-438-b-a-layout-swap-dropped-zeds-sidebar-and-its-state-001`: the rail answers
    `serialized_state` with the kept sidebar's state; the rule asks for the round trip tested
    closed, open and across a restart, which the test plan does.
  - `PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001` (#441): both
    `serialized_state` and `restore_serialized_state` run inside the `MultiWorkspace`'s update,
    so the first reads only the rail's fields and the second defers its close.
  - `PR-claude-a-marley-crate-writes-from-the-contract-not-the-gpl-body-001`: the blob's field
    names are Zed's persisted format, read from `SerializedSidebar`; no code is carried over.
  - Brain: consultation `456b652fec4a4f2eb4ce434ebdde768f`, nothing on this seam.
- **Re-verified.** Every seam the queued spec cited, against today's code; the spec's Prior art
  gives the current lines. The queued design's D2 (a deferred close-state restore) holds; its
  D1 (Zed's field names) holds, with the rail's close flag added.

### Design
- **The blob** (`rail.rs`, pure functions, unit-tested):
  - `RailState { width: Option<f32>, closed: bool }`, where `width` is the width the user
    set, if any.
  - `read_rail_state(blob) -> RailState`: `width` when `width_set_by_user` is true, and
    `marley_rail_closed`; anything unreadable reads as the default.
  - `write_rail_state(base: Option<&str>, state) -> String`: parses `base` into a JSON object
    (an empty one when it is missing or not an object), sets `width` (the width, or `null`),
    `width_set_by_user` and `marley_rail_closed`, and keeps every other field.
- **The rail's fields:** `width_set_by_user: bool` and `closed: bool`, next to `width`.
- **Open state.** The observer `Rail::new` already puts on the `MultiWorkspace` also calls
  `note_open`: while `multi_workspace_enabled`, `closed = !sidebar_open()`. It runs after the
  update that opened or closed the rail, and `serialize_now` reads the blob a turn later.
- **`serialized_state`:** `write_rail_state` over the kept Zed sidebar's own blob, or else the
  restored one (`zed_sidebar_state`), with the rail's fields. It reads the kept sidebar entity
  and the rail's fields, never the `MultiWorkspace`.
- **`restore_serialized_state`:** keeps the blob in `zed_sidebar_state` as today; takes a user
  width from it; and when it says closed, sets `closed` and defers
  `multi_workspace.close_sidebar(window, cx)` with `cx.defer_in`.
- **`set_width`:** `None` resets to `DEFAULT_WIDTH` with `width_set_by_user` off, `Some`
  clamps and sets it on; notifies, and forwards the same value to the kept Zed sidebar's
  `set_width`.
- **`Rail::new` over a kept Zed sidebar** reads that sidebar's blob and starts at its width
  when the user set one.
- **`take_zed_sidebar`** returns `write_rail_state(zed_sidebar_state, …)` in place of the raw
  blob, so a fresh Zed sidebar gets the rail's width.
- **File manifest.** Marley only: `crates/marley_workbench/src/rail.rs`, `rail_tests.rs`, the
  switch's tests in `marley_workbench_tests.rs` if the swap harness lives there; docs at
  Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_workbench.md`,
  `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: a Marley window, `close_sidebar`, then the rail's `serialized_state` says closed; a second Marley window restored through `apply_restored_multiworkspace_state` with `sidebar_open: false` and that blob ends with its rail closed |
| 002 | driven: the same with an open rail's blob and `sidebar_open: true`: the rail stays open |
| 003 | driven: with AI off, `close_sidebar` leaves the blob saying open |
| 004 | unit: `write_rail_state` keeps `active_view` and unknown fields, replaces the width fields, and starts from an empty object for a missing or broken base; `read_rail_state` reads the width only when set by the user. Driven: after a switch to Zed, a fresh Zed sidebar restores the rail's width |
| 005 | driven: `set_width(Some(400))`, switch to Zed (the kept sidebar's width is 400), switch back (the new rail's width is 400); a restored blob with a user width of 333 sizes the rail to 333 |
| 006 | driven: a Zed window whose sidebar the user widened to 321 (`ZED_SIDEBAR_STATE`), switched to Marley: the rail's width is 321 |
| 007 | `script/gates.sh --diff` |

### Risks
- **Telemetry on the restore's close.** `close_sidebar` records "Sidebar Toggled"; Zed has no
  silent close, and a Zed touchpoint for one is out of scope (intake).
- **A flash of an open rail.** A restored window builds its rail open and closes it after the
  restore; the close is deferred to the end of the restore's effect cycle, before the window's
  first frame in practice. The live drive watches for it.
- **Two widths' ranges.** The rail clamps to 180-600 px and Zed's sidebar to its own range, so
  a width outside one range shifts when it crosses to the other.

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] the blob · [x] open-state tracking · [x] the restore's
  close · [x] one width · [x] tests · [x] review · [x] clippy at deny level · [x] coverage.
- **Built** in `crates/marley_workbench/src/rail.rs`, as designed:
  - `RailState`, `SavedRail` (serde, so `serde` joins the crate's dependencies),
    `read_rail_state` and `write_rail_state`;
  - the fields `width_set_by_user` and `closed`; `note_open` from the existing observer;
    `rail_state`;
  - `serialized_state` over Zed's blob, `restore_serialized_state` taking the width and the
    close, `set_width` forwarding to the kept Zed sidebar, `Rail::new` starting at the kept
    sidebar's width, `take_zed_sidebar` handing over the merged blob.
- **Tests.** Three unit tests of the blob in `rail_tests.rs`; seven driven tests in
  `marley_workbench_tests.rs` with the helpers `sidebar_blob`, `saved_closed`,
  `restore_window` (Zed's `apply_restored_multiworkspace_state` on a built
  `MultiWorkspaceState`), `rail_width` and `set_rail_width`, and `assert_keeps_zeds_fields`.
  Two #438 tests asserted the old pass-through (`switching_back_hands_each_window_its_own_zed_sidebar`
  compared the blob byte for byte; `zeds_saved_sidebar_state_is_kept_unread_for_zed` expected
  the rail to ignore the width). Both now check what this slice promises: Zed's fields kept,
  the rail's added, and the width read. The second is renamed
  `zeds_saved_sidebar_state_sizes_the_rail_and_is_kept_for_zed`.
- **A bug the new test found.** The first restore deferred its close with `cx.defer_in` on the
  rail. That callback runs inside an update of the rail, and `close_sidebar` reads the sidebar
  (`sidebar_side`), so `a_closed_rail_stays_closed_when_its_window_is_restored` panicked:
  "cannot read marley_workbench::rail::Rail while it is already being updated". Fixed with
  `window.defer`, which runs with no entity leased, capturing only the `MultiWorkspace`'s weak
  handle. The F block and its rule are written at Complete.
- **Review against the criteria.**
  - Re-entrancy: `serialized_state` and `restore_serialized_state` run inside the
    `MultiWorkspace`'s update and touch only the rail's fields and the kept Zed sidebar;
    `note_open` runs from the observer, after the update; `set_width` updates the kept Zed
    sidebar, whose `set_width` touches nothing else.
  - A refinement: `set_width` first forwarded the raw pointer position, which Zed's sidebar
    clamps to its own range, so the two widths could differ. It now forwards the rail's
    clamped width, or the reset.
  - Provenance: the blob's field names are Zed's persisted format; no code is carried over.
    No Zed crate changed.
- **Negative checks.** Without `note_open`, the closed-rail test fails; without the width
  forwarding, `a_width_set_on_the_rail_carries_to_the_zed_sidebar_it_keeps` fails.
- **Checks.** `cargo clippy -p marley_workbench --all-targets --all-features -- -D warnings`
  clean. `cargo nextest run -p marley_workbench`: 78 passed. Coverage 1783 of 1783 lines.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-006 · [x] the gate · [x] the live drive
  (not run; why below).
- **Tests.**

  | REQ | Test |
  |---|---|
  | 001 | `marley_workbench_tests::a_closed_rail_stays_closed_when_its_window_is_restored` (through `apply_restored_multiworkspace_state`) |
  | 002 | `an_open_rail_stays_open_when_its_window_is_restored` |
  | 003 | `with_ai_off_no_close_is_saved` |
  | 004 | `rail::tests::the_rail_writes_its_fields_into_zeds_blob_and_keeps_the_rest`, `a_width_the_user_did_not_set_is_not_saved`, `an_unreadable_blob_holds_nothing_for_the_rail`; `switching_back_hands_each_window_its_own_zed_sidebar` and `zeds_saved_sidebar_state_sizes_the_rail_and_is_kept_for_zed` (both updated) |
  | 005 | `the_rails_width_holds_through_a_switch_to_zed_and_back` (a fresh Zed sidebar), `a_width_set_on_the_rail_carries_to_the_zed_sidebar_it_keeps` (the kept one), `a_restored_width_sizes_the_rail` |
  | 006 | `a_rail_built_over_zeds_sidebar_starts_at_its_width` |
  | 007 | `script/gates.sh --diff` |
- **Runs.** `cargo nextest run -p marley_workbench`: 78 passed. The fourteen tests that touch
  the blob, the width or the restore five times in a row: 14 passed each time.
- **Gate.** `script/gates.sh --diff`: 19 passed, 0 failed, `GATE GREEN [diff]`.
- **Live drive: not run.** It needs clicks and a relaunch. At 00:57 the desk looked as it did
  for #441, #449 and #450: stay-awake on, so nothing shows Chad away, and his fullscreen Teams
  window on the workspace the headless output borrows. The driven tests restore through Zed's
  own `apply_restored_multiworkspace_state` in its place; the drive is owed with the others to
  the next headless session while he is away.
- **Pre-existing failures:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket
  · [x] archive · [x] commit.
- **Documented.** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (the
  rail's blob, closed-rail memory and one width; the tests; the known limits, pointed at
  #451 to #454 and the intake doc); `docs/marley/workbench-shell.md` (W6 split, W6a shipped).
  No Zed path changed.
- **Knowledge appended.**
  - `F-claude-442-a-close-deferred-on-the-rail-ran-inside-the-rails-update-001`
  - `PR-claude-defer-in-does-not-leave-the-entitys-own-update-001`
  - `L-claude-442-driving-a-window-restore-in-a-test-001`
  - `AD-claude-442-the-rail-adds-its-fields-to-zeds-sidebar-blob-001`
- **Brain.** Consultation `456b652fec4a4f2eb4ce434ebdde768f` closed with `brain decide`:
  `decisions/the-rail-adds-its-width-and-close-to-zeds-sidebar-blob-one-width-for-both-layouts`,
  follow-up by 2026-10-07.
- **Closed and archived.** TICKET-442 in `tickets/closed/`; this pair in `pipeline/completed/`;
  the split's tickets 451 to 454 point at the completed spec.
