# pane-divider drag resizes the boundary you grabbed — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-422-pane-divider-drag-routing.md
- **Pipeline spec:** 422-pane-divider-drag-routing.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** TICKET-422 (the simple-rail shelf's adjacent bug, M20) — Chad: "drag
  and drop works only on the first item and it's very clunky." Classified at shelf
  time and re-verified here: this is the PANE-DIVIDER RESIZE. There is no rail-row
  DnD to be broken — that is a recorded v2 deferral (#398 spec :76-78, D1 :138-141)
  and stays out; nobody scope-creeps rail DnD into this bug. Independent of the
  418-421 rail batch — reorder freely (shelf note, confirmed by Chad 2026-08-12).
- **Classification / tier:** bug, work pipeline, one shippable slice (pure
  descriptor/resize helpers + one render-block rewrite). Systems: `marley_app` pure
  layout core (layout.rs, workspace.rs) + the app.rs render shim.
- **Recall (§18.3):**
  - **L TICKET-020** (lessons.md, DL-cluster-44c704b6e44c): `bool::then_some(x)`
    evaluates `x` EAGERLY — `(i >= 1).then_some(i - 1)` underflowed at i == 0 in
    layout.rs `neighbor`; fixed with `checked_sub`. Clippy's
    `unnecessary_lazy_evaluations` lint actively pushes the bug back. This ticket
    adds boundary/path index math in EXACTLY that file → D5.
  - **#130 origin (M6):** `resize_boundary` shipped top-level-only BY DESIGN — its
    doc comment says so and defers nested-interior resize (layout.rs:201-203). The
    handle loop then painted a handle per flattened leaf boundary anyway, leaking
    the deferral as inert UI. #130 also recorded the headless limit honestly
    (app.rs:20629-20631: gesture needs real mouse input, env-blocked → code-reviewed)
    — this pipeline carries the same honesty (REQ-004, P5).
  - **#398 v2-DnD deferral:** gpui's typed `on_drag`/`on_drag_move<T>`/`on_drop<T>`
    (div.rs:279-499) recorded as the rail-DnD v2 substrate, NOT adopted for verbs
    v1. Re-read this sweep — verdict for the divider gesture: still not adopted
    (spec Prior art leg 3).
  - **F-#386** (shelf pin): widened signatures break `#[cfg(test)]` call sites that
    plain `cargo check` never compiles — layout tests fan-out is plausible here →
    REQ-008 pins `cargo check --tests`.
- **Discovery (the edit surface for Design):**
  - **Handle registration** — app.rs:20627-20654: one handle per flattened leaf
    boundary (`rect_list.iter().enumerate().take(last_boundary)`), strip at
    `left_rect.x + left_rect.w - 3.0`, `w(px(6.0))`, `.top(px(0.0)).h(px(content_h))`,
    `.occlude()`, border fill + accent hover. Band-wrong: pane rects live at
    y ∈ [content_top, content_top + content_h] (`center_bounds.y = content_top`,
    :19229-19234; TOP_BAR_H = 30.0 :768; `content_band` layout.rs:126-130) — the
    strip overlaps the title bar and falls 30px short at the bottom. Axis-wrong:
    always vertical; `pane_rects`' axis branch (workspace.rs:173-201) is unused by
    the loop. `rect_list` = `pane_rects(group, inset_right(center_bounds,
    PANE_GUTTER))`, terminal tabs only (`active_is_terminal`, :19510-19526) — the
    fix stays inside that arm.
  - **Apply** — app.rs:20657-20675: root `on_mouse_move`/`on_mouse_up`; per-frame
    `delta = (x - last_x) / split_w.max(1.0)` — always Δx, always the full center
    width — into `workspace_mut().resize_boundary(boundary, delta, PANE_MIN_RATIO)`.
    `PANE_MIN_RATIO = 0.1` (:1025). `dragging_divider: Option<(usize, f32, f32)>`
    (:642) — becomes the routed-descriptor form.
  - **Sink** — layout.rs `resize_boundary` :204-208 (top-level only, M6 #130) →
    `resize_split` :150-163 no-ops for `boundary + 1 >= ratios.len()`; the Split
    invariant is exactly 2 children (:4-6) so `ratios.len() == 2` and ONLY boundary
    0 ever mutates — every other handle is painted, hoverable, `.occlude()`-ing,
    and inert. Matches the report: first divider works, rest dead.
  - **Working in-repo reference** — the #168 files-edge drag (app.rs:19186-19225):
    same root move/up capture pattern, band-CORRECT strip (`.top(px(content_top))`),
    persists on release. The divider block simply forgot the band offset.
  - **Capture gaps** — root listeners already deliver window-wide move/up (which is
    why the broken drag keeps applying off-strip); the missing piece is a missed
    release. Verified hooks: `MouseMoveEvent.pressed_button: Option<MouseButton>`
    (gpui interactive.rs:335-340) and the existing focus-edge pass
    (app.rs:17189-17194, `last_window_active`). Design picks (D4).
  - **Existing pins** — `resize_split_cases` (layout.rs:550) is the D6 anchor and
    stays green unchanged; `resize_boundary_top_level_only` (:584) literally pins
    today's "nested is a no-op" — Design dispositions it (evolve vs supersede).
- **Prior-art sweep (§20, three legs):** behavior maps — Zed deconstruction 07
  :112-125 (handles belong to split NODES; `resize` takes the axis; cached rects for
  hit-testing) = the adopted model; Warp maps silent on divider mechanics. Published
  — standard split-pane contract, nothing exotic; react-resizable-panels (the POC
  dep) documents the same. Permissive deps — gpui 0.2.2 typed drag READ
  (div.rs:282/:462/:473/:499; ghost render window.rs:2048-2078; central clear on any
  mouse-up :3716-3729): DnD-shaped (mandatory preview `Entity<W>`, drop targets) —
  KEEP `mouse_down` + view state for v1 capture; the typed pipeline stays the #398
  rail-DnD v2 substrate. Paying find: `pressed_button` for missed-release detection.
  No gpui resizable/split example ships (`examples/` swept; `drag_drop.rs` is
  value+ghost DnD).
- **Decisions:** D1 pure descriptor helper {path, axis, rect}; D2 resize at the
  owning split's path, clamp preserved, delta normalized to the owning split's
  extent; D3 shared-edge/axis-correct/band-aligned strips, ~6px; D4 capture
  semantics locked, mechanism to Design; D5 `checked_sub` per L TICKET-020; D6
  top-level resize behavior byte-identical. EARS REQ-001..008 pinned.

## Phase 2 — Design
- Architecture / approach; file manifest; regression test plan; risks.

## Phase 3 — Implement
- What was built; deviations from design (with reason).

## Phase 3.5 — Inspect
- Critics run; findings table (severity / finding / verdict); fixes.

## Phase 4 — Validate
- Tests RUN (with counts) + gate result; negative smokes; pre-existing notes.

## Phase 5 — Complete
- Docs updated; ledger appends (lessons / failures / prevention rules / ADs — codes listed); archive.

- **Promotion (2026-08-12, the `/work 418-422` batch, last slice):** 418–421 all SHIPPED. Seams
  re-verified by symbol (app.rs drifted across the batch): layout.rs `resize_split` :150 /
  `resize_boundary` :204 / `content_band` :126 / `resize_split_cases` :550 /
  `resize_boundary_top_level_only` :584; workspace.rs `pane_rects` :159 (+ its quadrant/sum test
  suite :1174-1275 — the geometry oracle the descriptor tests can lean on); app.rs
  `dragging_divider` :643 (`Option<(usize, f32, f32)>` — boundary/last_x/split_w, the shape D2's
  descriptor replaces), the handle block now ~:20840-20880 (was :20627), `TOP_BAR_H` :769,
  `PANE_MIN_RATIO` :1026, `rect_list` :19719. The spec's gpui sweep verdict stands (mouse_down +
  view state; `pressed_button` for D4). React-first N/A confirmed (gesture bugfix, no visual
  grammar change). BACKLOG row removed (§19).

## Phase 2 — Design

### §20 confirm
As speced: standard split-pane behavior; the POC's react-resizable-panels is the observed feel
reference; Zed map behavior-level only; the gpui sweep verdict (mouse_down + view state;
`pressed_button` for the missed release) stands. No copyleft source.

### The shapes
- **D1 — `divider_rects(group, bounds, hit) -> Vec<DividerRect>`** (workspace.rs, beside
  `pane_rects`, sharing its child-bounds arithmetic):
  `DividerRect { path: Vec<usize>, axis: PaneAxis, rect: Rect, extent: f32 }` — one per `Split`
  (the 2-child invariant ⇒ ONE boundary per split ⇒ N-1 descriptors for N leaves, and **zero
  boundary-index math anywhere** — the L TICKET-020 hazard is designed OUT, not guarded).
  `path` = child indices from the root to the OWNING split; `axis` = the owning split's axis
  (a Horizontal split has a VERTICAL strip); `rect` = the grab strip centered on the shared edge
  (`hit` wide: H → `{x: child0.right - hit/2, y: bounds.y, w: hit, h: bounds.h}`; V →
  transposed); `extent` = the owning split's span along its axis (the D2 delta normalizer).
  **The band is structural:** `center_bounds.y == content_top` (app.rs:19430-19434) already
  flows through the walk — the old block IGNORED `rect.y` (`top(0)/h(content_h)`), which was
  the whole band bug; the rewrite renders `rect` verbatim.
- **D2 — `PaneGroup::resize_at(&mut self, path: &[usize], delta: f32, min: f32)`** (layout.rs):
  descend `path` (`children.get_mut(idx)` — an OOB index is a total no-op), at the end apply
  `resize_split(ratios, 0, delta, min)` — boundary 0 always (2-child invariant). The clamp math
  is `resize_split` UNCHANGED. **`resize_boundary` RETIRES** (its only production caller is the
  rewritten block; a dead pub fn fails honesty) — `resize_boundary_top_level_only`'s top-level
  rows re-pin byte-identically onto `resize_at(&[], …)` (the empty path IS the old
  boundary-0 top-level call; the old "boundary ≥ 1 no-ops" rows die WITH the parameter — that
  semantic is structural now). REQ-007's equivalence pin: identical trajectories.
- **Delta plumbing (shim):** `dragging_divider: Option<DividerDrag>` where
  `DividerDrag { path: Vec<usize>, axis: PaneAxis, last: f32, extent: f32 }` (defined beside
  `DividerRect` in workspace.rs — pure-side shape, shim-held). mouse_down on a strip seeds it
  from the descriptor (last = the axis coordinate of the pointer); each move:
  `delta = (pos_along_axis - last) / extent.max(1.0)` → `resize_at(&path, delta, min)` → update
  `last`. Axis-correct by construction (REQ-002).
- **D4 — the missed release:** primary = `event.pressed_button != Some(MouseButton::Left)` in
  the move listener (verified gpui field) → drop the drag on the next observed evidence
  (REQ-005); mouse_up keeps the normal end. No focus-edge wiring needed — a drag can only
  "stick" until the next move, which self-heals. Recorded: the end condition lives in the
  masked shim (a one-line predicate; REQ-005's verify = the wiring review + the negative smoke
  at P4's code-assert level, recorded honestly per the #130 stance).
- **The handle block rewrite** (app.rs ~:20831-20880): iterate `divider_rects(group,
  center_bounds, 6.0)`; per descriptor render the strip at `rect` verbatim (`.occlude()`, the
  UNCHANGED visual grammar: `colors.border` + accent hover); mouse_down seeds `DividerDrag`
  (clone the path); the root move/up handlers keep the #130 window-wide capture shape, with the
  D4 predicate + axis-aware delta.

### Test matrix (P4 writes)
| Area | Cases |
|---|---|
| `divider_rects` | leaf → []; top-level H 2-pane (1 vertical strip on the ratio edge, extent = bounds.w, path = [], band-y inherited); top-level V (transposed); nested L (H-root, V-child → 2 descriptors, paths []/[1], axes H/V, rects on the true shared edges); 2×2 (3 descriptors); non-zero bounds.y (the band pin); hit-width centering (edge ± hit/2); consistency vs `pane_rects` (the strip's edge == the children's shared edge — lean on the existing quadrant suite's oracle) |
| `resize_at` | empty path == the legacy top-level trajectory (REQ-007); a nested path mutates exactly its split (siblings/root ratios untouched — REQ-001, kills the flattened-routing mutant); OOB path index → total no-op; `Leaf` → no-op; min-clamp at depth (REQ-006, both directions + the `total < 2·min` guard); V-axis delta sign via extent (REQ-002's pure half) |
| re-pins | `resize_split_cases` green UNCHANGED; `resize_boundary_top_level_only` → `resize_at` re-pin (top-level rows verbatim; the boundary-param rows retire with the param) |

### File manifest
- `crates/marley_app/src/layout.rs` — `resize_at` in; `resize_boundary` out; doc updates
  (the #130 "top-level only" comment superseded honestly).
- `crates/marley_app/src/workspace.rs` — `DividerRect` + `DividerDrag` + `divider_rects` (+ the
  unit suite beside the `pane_rects` tests).
- `crates/marley_app/src/app.rs` — the field's new shape, the handle-block rewrite, the
  move/up handlers (axis + D4), the #130 comment refresh. All shim (masked).

### Risks
- **R1** f32 edge drift (the `pane_rects` ULP caveat) — descriptors reuse the same arithmetic;
  the consistency unit compares against `pane_rects`' own child edges (same drift, no false red).
- **R2** the `.occlude()` strips vs the #228 focused-border draw order — the rewrite keeps the
  same z-order (strips before the border, border non-occluding) — reviewed at inspect.
- **R3** `DividerDrag.path` is a Vec (the old state was Copy) — the mouse_down listener clones
  per seed; per-move mutates in place; no per-frame allocation beyond the seed.

## Phase 3 — Implement

- **React-first: N/A** (per the spec — a gesture/hit-target bugfix with no visual-grammar change;
  the POC's react-resizable-panels behavior is the observed reference, already correct).
- **workspace.rs:** `DividerRect` + `DividerDrag` + `divider_rects(group, bounds, hit)` — the
  depth-first walk sharing `pane_rects`' exact child-bounds arithmetic; one descriptor per Split
  (path/axis/band-inheriting rect/extent); zero boundary-index math (L TICKET-020 designed out).
  The PaneGrid wrapper `resize_boundary` → `resize_at(&[usize], …)`.
- **layout.rs:** `PaneGroup::resize_at(path, delta, min)` — descend `children.get_mut(idx)`
  (OOB/Leaf total no-ops), apply `resize_split(ratios, 0, …)` at the owning split;
  `resize_boundary` RETIRED (the #130 comment superseded honestly in the new doc).
- **app.rs:** `dragging_divider: Option<DividerDrag>`; the handle block iterates
  `divider_rects(ws.group(), center_bounds, 6.0)` and renders each descriptor's rect VERBATIM
  (band + axis structural — `center_bounds.y == content_top`); mouse_down seeds the drag with
  the pointer's axis coordinate; the move handler applies Δ-along-axis / extent at the grabbed
  path and D4's missed-release predicate (`pressed_button != Some(Left)` ends the drag on the
  next observed evidence); mouse_up unchanged. Visual grammar untouched (border fill, accent
  hover, ~6px, `.occlude()`), z-order vs the #228 focused border unchanged.
- **D6/F-#386:** the predicted 3-site test fan-out re-pinned at compile time
  (`resize_at_empty_path_is_the_legacy_top_level_resize` — the REQ-007 equivalence half —
  and the PaneGrid delegate test onto the empty path); `cargo check --workspace --tests` 0
  errors; fmt clean.
- **Deviations:** none.

## Phase 3.5 — Inspect

Two critics (geometry/routing with numeric traces; shim/wiring). Ledger:

| # | Finding | Verdict | Fix |
|---|---|---|---|
| 1 | **Wrong bounds at the ONE call site**: panes tile `inset_right(center_bounds, PANE_GUTTER)` (w−8) but the walk got raw `center_bounds` — every strip sat `PANE_GUTTER·ratio` off the true seam (fully disjoint from the 6px strip at ratio ≥ 0.75) and the extent lagged the pointer at (w−8)/w | **REAL (high)** — both critics, numerically traced | the walk gets the SAME `tiling_bounds` binding the panes use (extent fixed for free) |
| 2 | **Wrong gate**: `try_workspace()` falls back to the FIRST terminal grid when a code/cockpit tab is active → occluding strips leaked over non-terminal surfaces, stealing mouse and resizing a hidden grid | **REAL (high)** | the block is gated on `active_is_terminal` — the tiling's own predicate |
| 3 | **Mid-drag panic**: the move handler's `workspace_mut()` `expect`s ≥1 terminal; the #395/#173 pump can close the last terminal mid-drag → the next move panicked | **REAL (med)** | `try_workspace_mut()`, clearing the drag on None (total) |
| 4 | The field's decl doc still described the deleted `(boundary, last_x, split_w)` tuple + `resize_boundary` | **REAL (doc)** | rewritten |
| 5 | The missed-release predicate hand-rolled what gpui ships: `!event.dragging()` (`pressed_button == Some(Left)`, verified in gpui source) — and the hand-rolled form ended a drag on a CHORDED right/middle press (mac NS*MouseDragged events carry that button) | **REAL (low)** | `!event.dragging()` — identical semantics for the miss case, self-documenting, chord-tolerant… (same event mapping; the chord still ends it — benign, re-grab works — recorded) |
| 6 | A stale `path` after a mid-drag tree restructure can misroute deltas until release (in-range positional aliasing; out-of-range/leaf paths are total no-ops) | **recorded edge** — requires closing a pane WHILE dragging and continuing the drag; self-heals at release; wiring drag-clears into every restructure site is disproportionate — recorded with the mitigation option |
| 7 | `divider_rects` has ZERO tests — the critics' concrete mutant list + fixture requirements (non-zero origin, unequal ratios, w≠h — `split_focused` only makes 0.5/0.5, so Split literals required) | **P4 obligation** — the matrix now carries the exact kill list |
Verified clean (numeric traces in the critic reports): the edge formula ≡ `pane_rects`' child-0 far
edge (both axes, token-identical recursion — bit-identical GIVEN the same bounds, which fix 1 now
guarantees); the band inheritance exact (old strip [0,648] vs panes [30,678] — the ticket's 30px
over/short, now [30,678] exactly); routing traces (nested path → exactly its split; leaf/OOB paths
total no-ops); extent = the OWNING split's span at every depth; the 2-child invariant structural
(unconstructible otherwise); z-order vs the #228 border unchanged; borrow/NLL clean; per-frame cost
noise at N≤8. Post-fix: `cargo check --workspace --tests` 0 errors, fmt clean.

## Phase 4 — Validate

**Tests written + RUN:**
- `divider_rects` suite (workspace.rs — the inspect kill-list executed): leaf → []; top-level H
  strip on the ratio edge (unequal [0.3,0.7], origin (10,30), 1000×500 — kills edge±, ratios[0]→[1],
  the y→0 BAND mutant, extent w↔h); the V twin transposed; the nested L (paths []/[1], the child's
  extent, the accumulated child bounds); the 2×2 (3 descriptors, walk order); **the AGREEMENT pin**
  (divider centers ≡ `pane_rects` shared edges over the SAME awkward bounds — the
  PR-…-coordinate-world-001 class made structural).
- `resize_at` suite (layout.rs): nested-path routes to EXACTLY that split (root + sibling
  untouched — REQ-001; kills get_mut idx→0, rest→path); bad paths total (OOB / onto-a-Leaf /
  deep garbage — REQ-005's pure half); min-clamp at depth + the no-room guard (REQ-006, f32
  `approx` per the house idiom); + the P3 re-pins (empty-path ≡ legacy — REQ-007; the PaneGrid
  delegate).
- **Run:** `cargo nextest run --workspace` → **2146/2146 passed** (5 skipped); doctests clean;
  `cargo check --workspace --tests` green (REQ-008's check half); fmt clean.

**Live geometry (fresh bundle, the restored 2-cell split):** pixel-sampled — the divider strip
occupies EXACTLY the content band (y=15 title bar clean, y=45..740 the border-color strip, y=760
status bar clean — the old code's 30px-over/30px-short band bug is dead in the pixels) and sits
centered on the pane seam beside the #228 focus border. ⌘⇧L on the right cell → a NESTED
3-cell arrangement: **both dividers render, each on its own seam** (the second at x 912-915,
pixel-verified — the old code's inert-handles state is gone). ⌘W restored the 2-cell state
(the second strip vanished with its split ✓). Captures read at every step
(/tmp/marley-422-*.png).

**The drag gesture itself (press-move-release): env-blocked headless** — drive.swift has no
drag verb and the #130 stance is recorded in the spec; REQ-004/005's wiring is code-reviewed
(the inspect shim critic traced the predicate order, the gpui `dragging()` semantics, and the
total mid-drag mutate) and their PURE halves are unit-pinned (the bad-path totality rows).

**Gate:** first run RED at gate:14 (a rustdoc link to the private `resize_split` — de-linked),
gate:5 (4 survivors on the V-arm child-bounds math — executed but UNOBSERVED; killed by the
V-root nested fixture whose nested-divider rect observes the accumulation), and gate:4 (7
uncovered lines = MY OWN tests' let-else `unreachable!` arms — rewritten to the house
whole-tree-equality pattern, the arms now nonexistent). Re-run → **GATE GREEN [diff],
15 passed, 0 failed**. Receipt written.

## Phase 5 — Complete
- **§21:** CHANGELOG (Fixed) entry; app_shell.md M20 #422 entry. React-first N/A (recorded; the
  POC's react-resizable-panels was the behavior reference — no POC change).
- **§19:** F-claude-422-pure-walk-right-call-site-wrong-twice-001 +
  PR-claude-a-pure-helper-inherits-its-callers-coordinate-world-001 (inspect);
  L-claude-422-test-code-is-coverage-code-avoid-destructuring-arms-001 (complete — the gate:4/5
  lesson). The headless-gesture limitation recorded plainly in P4 (the #130 stance).
- **Ticket** → closed/; shelf ticked (the batch is now 5/5 ✅); BACKLOG clean; pair → completed/.
