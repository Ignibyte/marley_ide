# add-cursor-below (⌘⌥↓) scroll-follow — Notes

- **Forge ticket:** #360 `c243e4d4-59d2-4d19-8623-faa8c6103a37`
- **AAR:** `98f090dd-d3a3-4e3f-b913-fc9095f0372c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-360-add-cursor-below-scroll-follow.md
- **Pipeline spec:** 360-add-cursor-below-scroll-follow.spec.md
- **pipeline_id:** 49e897c8-e835-4d30-b976-f0b7cac73357
- **Live on:** `3b66660` (FIRST of the goal /work 360,361,362,363,364)

## Phase 1 — Plan
- **Request:** ⌘⌥↓ (add-cursor-below) scroll-follow tracks the stationary PRIMARY, not the new bottom cursor, so
  sustained ⌘⌥↓ marches carets off the bottom edge unseen. A #342 inspect follow-up.
- **Classification / tier:** work pipeline; a small editor/multi-cursor bug; app-shim wiring reusing shipped
  primitives (no new pure seam).
- **Forge recall (§18.3):** bulletins none. `aar-open` → `98f090dd`. `knowledge-context` (Plan) logged 13
  surfacings — the #342 scroll-follow ADs + the `hook-the-shared-primitive` / column-domain PRs. The governing
  prior art is the #342 `add-next-occurrence` arm (the exact same fix).
- **★ Recon (on `3b66660`) — the fix is a verbatim transplant, all seams confirmed:**
  1. The buggy arm (app.rs:7358): `add_cursor_vertical(...)` → `set_active_selections(set)` → the bare
     `self.follow_editor_caret()` (follows `active_caret()` = PRIMARY = member 0).
  2. ★ The TEMPLATE — the `add-next-occurrence` arm (app.rs:7399-7407, #342): `follow = added_member(&before,
     &after).map(|sel| { let head = sel.head(); (s.active_buffer().line_col(head).0, head) }); …; if let
     Some((row, head)) = follow { self.scroll_editor_to(row, Some(head)); }`. #360 mirrors this exactly.
  3. Seams: `added_member(&SelectionSet,&SelectionSet)->Option<Selection>` (multi_cursor.rs:296 + unit :1076) —
     names the new cursor regardless of sort; `scroll_editor_to(row, Option<CharOffset>)` (app.rs:12885,
     column-aware — NOT `scroll_editor_to_row` :12901); `buffer.line_col(c)->(row,col)` (buffer.rs:152, `.0`=row).
  4. ★ The VERTICAL scroll IS headlessly observable: `editor_scroll_y_for_test()` (headless_drive.rs:763) reads a
     MOVED offset after `scroll_editor_to_row` + `run_until_parked` (proven by `per_file_scroll_memory…` :806,
     `a_deep < -100.0`). So #360 proves the vertical fix DIRECTLY (not the #342 horizontal proxy — ⌘⌥↓'s new
     cursor is at the SAME column as the primary, so `scroll_x` wouldn't distinguish them). The scroll settles on
     `run_until_parked` (gpui layout), NOT the app pump → NO mock-clock `tick_pump` needed.
  5. cov/MSI: the arm is inside `fn dispatch_action` (`#[cfg_attr(test, mutants::skip)]`) + app.rs cov-excluded →
     gate-is-headless-drive; the reused primitives are already covered. No new pure seam.
- **Decisions:** D1 verbatim mirror of the add-next-occurrence arm; D2 gate BOTH dirs through `added_member`
  (removes ⌘⌥↑'s luck-dependency); D3 no new cov/MSI surface; D4 vertical observable via `editor_scroll_y`, no
  pump.
- **Prior art:** the #342 add-next-occurrence arm is the same fix; `PR-claude-hook-the-shared-primitive` (#336).
  gpui owns the scroll (no new adoption). §20 N/A.
- **EARS:** REQ-DOWN-FOLLOWS-NEW, REQ-UP-FOLLOWS-NEW, REQ-SINGLE-PRESS-NOOP (see spec).

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture / approach.** One app-shim arm (`crates/marley_app/src/app.rs`, inside `dispatch_action` which is
`#[cfg_attr(test, mutants::skip)]`), rewired to reuse the shipped editor/multi-cursor primitives. No crate/type
added. §14: no panics on the path (`added_member` returns `Option`, the follow is gated on `Some`); the reused
`line_col`/`scroll_editor_to`/`add_cursor_vertical`/`added_member` are all existing + tested. §20 N/A (Marley's
own multi-cursor scroll-follow — confirmed; no reference-app behavior).

**★ Ratified — the arm rewrite (D1/D2).** `add_cursor_vertical(set: &SelectionSet, buffer: &Buffer, dir) ->
SelectionSet` (multi_cursor.rs) takes `&SelectionSet` and sources `VDir::Up => set.primary()` / `VDir::Down =>
set.last()` — confirming the bug (Down grows from `last`, sorts to the bottom, primary stays member 0). The
rewrite is byte-for-byte the `add-next-occurrence` template (app.rs:7399-7407):
```
"add-cursor-above" | "add-cursor-below" => {
    let dir = match action { "add-cursor-above" => VDir::Up, _ => VDir::Down };
    let mut follow = None;
    if let Some(s) = self.active_editor_mut() {
        s.clear_marked();
        let before = s.active_selections().clone();
        let set = marley_editor::add_cursor_vertical(&before, s.active_buffer(), dir);
        follow = marley_editor::added_member(&before, &set).map(|sel| {
            let head = sel.head();
            (s.active_buffer().line_col(head).0, head)
        });
        s.set_active_selections(set);
    }
    if let Some((row, head)) = follow { self.scroll_editor_to(row, Some(head)); }
}
```
- Borrow shape: `before` cloned first (needed for `added_member` AFTER the set is built — one cheap SelectionSet
  clone, exactly as the template does); `follow` a local applied AFTER the `if let Some(s)` block (so the `&mut
  self.scroll_editor_to` doesn't overlap the `active_editor_mut` borrow). The template proves this compiles.
- The bare `self.follow_editor_caret()` at :7372 is REMOVED (replaced by the gated `scroll_editor_to`).
- D2: `added_member(&before, &set)` names the new cursor regardless of sort → correct for BOTH ⌘⌥↑ (new top,
  sorts to member 0) and ⌘⌥↓ (new bottom) — removes the luck-dependency; ⌘⌥↑ still follows the new cursor.

**File manifest (1 file):**
- `crates/marley_app/src/app.rs` — rewrite the `"add-cursor-above" | "add-cursor-below"` arm (:7358) to the
  `added_member` + `scroll_editor_to` pattern (above); remove the bare `follow_editor_caret()`.
- (Tests land in headless_drive.rs at Phase 4 — the drives below.)

**★ Test plan (headless; the arm is mutants::skip + cov-excluded → NO new cov/MSI surface; the drives are the
proof). Mechanism confirmed:** `dispatch_for_test("add-cursor-below")` in a loop; `editor_scroll_y` reads the
settled vertical offset; the scroll settles on `run_until_parked` ALONE (NO `tick_pump`/`advance_clock` — the
deferred `scroll_to_item` applies on the gpui layout, NOT the app pump; proven by `per_file_scroll_memory` :806).
The fixture mirrors `per_file_scroll_memory`'s scale (row 100 of 121 is comfortably off-screen, `a_deep < -100`).

| # | Proves | Test |
|---|---|---|
| T1 | REQ-DOWN-FOLLOWS-NEW | tall fixture (~130 rows `// line {n}`), caret row 0, ~100× `dispatch_for_test("add-cursor-below")` in one `window.update`, `run_until_parked` → assert `editor_scroll_y < -100.0` (the list followed the new bottom cursor). ★ NON-VACUOUS: the OLD bare `follow_editor_caret()` (primary at row 0, on-screen) leaves it ~0 → fails on the old code. |
| T2 | REQ-UP-FOLLOWS-NEW (regression guard) | caret deep (`caret_at(view,100,0)` + `set_single_caret`) + `scroll_editor_to_row(100)` → `editor_scroll_y` deep-negative; ~100× `add-cursor-above` + `run_until_parked` → assert `editor_scroll_y` moved toward 0 (the view followed the new TOP cursor up). A regression guard (⌘⌥↑ worked by luck before; proves the added_member path keeps it working). |
| T3 | REQ-SINGLE-PRESS-NOOP | caret row 0, ONE `add-cursor-below`, `run_until_parked` → assert `editor_scroll_y` unchanged (~0 — the new cursor at row 1 is on-screen, the non-strict Center no-op). |

- **Uncoverable-honestly:** none new. The arm is app-shim (mutants::skip, cov-excluded); the reused primitives
  (`added_member` unit multi_cursor.rs:1076, `scroll_editor_to`/`line_col`) are already covered by #342/#297. The
  drives carry the wiring proof; the `--diff` mutation gate has no new mutable crate lines beyond the skip'd arm.

**Risks / decisions.** (a) borrow shape — the template proves the `before`-clone + `follow`-local + apply-after
pattern compiles. (b) T1 non-vacuous — the ~130-row fixture + ~100 presses guarantees the bottom cursor exceeds
the viewport (per_file_scroll_memory proved row 100 is off-screen); if a healthy box's viewport were somehow
≥100 rows the test would under-assert, but 100 rows >> any plausible test-window height. (c) `editor_scroll_y`
(vertical) is the honest axis, NOT `scroll_x` (⌘⌥↓ keeps the column). (d) `run_until_parked` ALONE — NO
mock-clock pump (D4; the inverse of the #321/#349 pump trap). (e) both dirs through `added_member` (D2 — ⌘⌥↑
guarded by T2). §14; §20 N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

**Built (1 file, `crates/marley_app/src/app.rs`, 1 arm):** rewrote the `"add-cursor-above" | "add-cursor-below"`
arm (:7358) to the `added_member` + `scroll_editor_to` pattern — a verbatim mirror of the `add-next-occurrence`
arm. The bare `self.follow_editor_caret()` is GONE; the follow is now `added_member(&before, &set).map(|sel| (…
line_col(head).0, head))` applied via `scroll_editor_to(row, Some(head))` after the borrow block. A `#360` doc
comment records the mechanism.

**Borrow shape (confirmed compiling):** `before = s.active_selections().clone()` (one owned SelectionSet clone,
as the template does — needed for `added_member` after the set is built); `follow` a local `Option<(usize,
CharOffset)>`; the `self.scroll_editor_to(...)` applied AFTER the `if let Some(s) = self.active_editor_mut()`
block closes (so the `&mut self` call doesn't overlap the editor borrow — E0502-safe, exactly as the template).

**Checks:** `cargo fmt --all` clean; `cargo clippy -p marley --all-targets -- -D warnings` **exit 0** (no
unused-`mut`/needless-clone; `add_cursor_vertical(&before, …)` + `line_col(head).0` resolve). Diff = ONLY app.rs
(the one arm).

**Deviations from design:** none. Byte-for-byte the ratified rewrite.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**1 critic (Agent general-purpose) over the diff + my own step-2 review. Verdict: CLEAN — no HIGH/MEDIUM; 5 LOW,
all confirmations, no fix. No source change.**

**Critic — the arm rewrite is correct + behavior-preserving both directions. VERDICT: correct.**
- (a) BOTH dirs via `added_member` CONFIRMED: ⌘⌥↓ (`add_cursor_vertical` sources `set.last()` → new caret sorts
  to the bottom via `from_selections`' `sort_by_key(start())`) → `added_member` names it → `scroll_editor_to`
  follows it (the fix). ⌘⌥↑ (sources `set.primary()` → new caret sorts to member 0) → the OLD `follow_editor_caret()`
  followed member 0 = the same new top caret (by luck); the NEW `added_member` names the same caret → NO
  regression, now robust. Extra proof (critic): `added_member` can't mis-fire on a moved existing member — the
  gesture changes no text, and a bare caret can't extend a range (`head > cur.end()` for extension contradicts
  `head <= cur.end()` for overlap) → exactly one `after` member is new.
- (b) the no-add `None` case: real but BENIGN behavior delta — `add_cursor_vertical` returns `set.clone()` at the
  buffer edge (multi_cursor.rs:84-86) → `added_member` None → no scroll. OLD re-centered on the primary
  unconditionally; NEW leaves the view put on a dead keypress (nothing added = nothing to reveal). Matches the
  shipped #342 sibling + REQ-SINGLE-PRESS-NOOP/D2. No fix. (My own read reached the identical conclusion.)
- (c) `line_col(head).0` = row (buffer.rs:152); `scroll_editor_to` (column-aware :12885) not `scroll_editor_to_row`;
  `head: CharOffset` matches the `Option<CharOffset>` param. CONFIRMED.
- (d) borrow: `before` owned clone → `add_cursor_vertical(&before)`; `follow` local applied after the
  `active_editor_mut` block (E0502-safe; note `scroll_editor_to` is `&self`); no `unwrap`/`expect`; the bare
  `follow_editor_caret()` REMOVED (only in the doc comment now); single hunk; `git status` = app.rs + 3 docs; no
  Zed/Warp/secrets. CONFIRMED.

**My own step-2 review (independent):** read `add_cursor_vertical` (the edge `set.clone()` → None), traced both
directions + the no-add case → benign/better. Agrees with the critic.

**Findings acted on:** none — no HIGH/MEDIUM; all 5 LOWs are confirmations of a correct, design-ratified transplant.
`failure-record`: none (no bug — a clean mirror of the shipped #342 arm). No source edit at Inspect.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests WRITTEN (3 headless drives in headless_drive.rs, after the #342 scroll-follow family):**
- T1 `add_cursor_below_follows_the_new_bottom_cursor_headless` (REQ-DOWN-FOLLOWS-NEW): 130-row fixture, caret row
  0, 100× `dispatch_for_test("add-cursor-below")` → assert `cursors == 101` (the adds happened) AND `editor_scroll_y
  < -100.0` (the list followed the new bottom cursor down).
- T2 `add_cursor_above_follows_the_new_top_cursor_headless` (REQ-UP-FOLLOWS-NEW, regression guard): caret deep
  (row 120) + `scroll_editor_to_row(120)` → `editor_scroll_y < -100` (sanity); 100× `add-cursor-above` →
  `editor_scroll_y` moved toward 0 (the view followed the new top cursor up).
- T3 `add_cursor_below_single_press_on_screen_does_not_scroll_headless` (REQ-SINGLE-PRESS-NOOP): one
  `add-cursor-below` at row 0 → `editor_scroll_y == 0.0` (row 1 is on-screen, the non-strict Center no-op).

**★ One iteration (a test-harness detail, NOT the fix):** the first run failed with `editor_scroll_y == 0` in T1/T2
— the mutating `window.update` blocks lacked `cx.notify()`, so `run_until_parked` never rendered a frame and the
DEFERRED `scroll_to_item` never applied against a layout. `per_file_scroll_memory` calls `cx.notify()` after its
scroll (:802) for exactly this reason. Added `cx.notify()` to each block → all 3 pass. ★ T1 is NON-VACUOUS: with
the notify present, the OLD bare `follow_editor_caret()` would scroll to the primary at row 0 (on-screen → the
non-strict Center no-op → `scroll_y == 0`), so `assert!(scroll_y < -100)` fails on the pre-fix code and passes
only because the fix routes the follow to the new bottom cursor.

**Tests RUN:**
- Targeted: `cargo nextest run -p marley -E 'test(add_cursor_below_follows) + test(add_cursor_above_follows) +
  test(add_cursor_below_single_press)'` → **3 passed** (T3 0.086s, T1 0.093s, T2 0.094s).
- App regression: `cargo nextest run -p marley` → **736 passed, 2 skipped** (was 733 for #334 → +3, the new
  drives). No regression; the `search_open…` flake did not recur.

**Live drive:** NONE — stated deliberately. The change is a keystroke-handler follow-target rewrite; the headless
drives run through the REAL `dispatch_action` + the REAL `editor_scroll` gpui handle (not a mock), so the vertical
follow is proven end-to-end. A live synthetic drive is off-limits (chad may be at the machine) and adds nothing
over the `editor_scroll_y` assertion.

**Full `--diff` gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15 PASS**, receipt
`dda5cde67a240388e6d0b17ea750f78f72a58230`. gate:4 coverage ≥100% (no new pure surface — the arm is inside
`dispatch_action` `mutants::skip` + app.rs cov-excluded; the drives are `#[cfg(test)]`); gate:5 mutation MSI ≥100%
(no new mutable crate lines); gate:14 docs PASS. No red, no pre-existing exclusions.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21).**
- **CHANGELOG.md** — a `### Fixed` entry (⌘⌥↓ follows the new cursor via `added_member` + `scroll_editor_to`;
  ⌘⌥↑ was correct only by luck; the #342 seam transplant).
- **docs/marley_architecture/editor.md** — extended the `added_member` scroll-follow paragraph (:257) to note #360
  routes the ⌘⌥↑/⌘⌥↓ add-cursor gesture through the same `added_member` + column-aware `scroll_editor_to`.

**Capture (forge wired).**
- `aar-submit` 98f090dd, outcome completed, effectiveness 4 (1 novel, 13 verdicts). Headline lessons: (a) ★ the
  fix was a PRIOR-ART TRANSPLANT — the sibling `add-next-occurrence` arm (#342) already solved the identical
  "follow the added cursor, not the sorted member-0 primary" problem; the recon's first move (read the sibling)
  turned "design a fix" into "copy the proven pattern". (b) ★ D2 — gating BOTH directions through `added_member`
  removed the "⌘⌥↑ correct by luck" (member-0-sort) dependency. (c) ★ a DEFERRED gpui handle op
  (`scroll_to_item`) is headlessly observable via `editor_scroll_y` ONLY after a render — the mutating
  `window.update` block must `cx.notify()` so `run_until_parked` renders the frame that applies it (the first-run
  failure: scroll_y stayed 0). (d) the arm is `dispatch_action` `mutants::skip` + app.rs cov-excluded →
  gate-is-headless-drive; non-vacuity comes from the fixture putting the new cursor off-screen. 1-critic inspect +
  own review = unanimous, 0 HIGH/MEDIUM.
- `prevention-rule-record` **PR-claude-deferred-gpui-handle-op-needs-notify-in-headless-001** (bb5c5a00): a
  headless assertion on a deferred gpui handle op must `cx.notify()` in the mutating update block so the frame
  that applies it renders — else the accessor reads the pre-op value and the test is vacuous. Distinct from the
  mock-clock pump trap (there it's `advance_clock`; here it's the notify-driven render).
- `failure-record`: none SHIPPED — the bug was pre-existing (#342's D3-⌘D-only scope); the inspect found 0
  HIGH/MEDIUM; the `cx.notify` iteration was a test-authoring detail caught + fixed in-phase.
- No follow-up ticket — this IS the #342 follow-up; the fix is complete + symmetric.

**Close + archive.** forge #360 → done; TICKET-360 → closed/ (`status: closed`); the spec/notes pair archived
active/ → completed/; spec `status: Phase 5 — Complete PASS`.

**Status: Phase 5 — Complete PASS.** Run `/commit` to deliver (LOCAL — push un-OK'd). FIRST of the goal /work
360…364 (1 of 5).
