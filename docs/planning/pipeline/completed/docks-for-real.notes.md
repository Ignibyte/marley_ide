# docks for real — Notes

- **Forge ticket:** #24 `0764b4e3-affc-45f5-ba73-8f2ae6c7e43f`
- **AAR:** `e928eb68-9ccf-4e72-81af-b764a01a3ddd`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-024-docks-for-real.md
- **Pipeline spec:** docks-for-real.spec.md

## Phase 1 — Plan
- **Request:** forge #24 (M1.C seq-3, auto-approved run) — render the 3 regions for real; wire
  toggle_dock to chords + palette commands.
- **Classification / tier:** work pipeline, `feature` — small slice (one pure fn + keymap rows +
  the shim render). Systems: marley_app only.
- **Forge recall (§18.3):** bulletins none. Fresh in-context from #23: the `pane_rects(bounds)`
  parameter is the ready dock-inset seam (critic D called it); `dock`/`toggle_dock`/`DockState`
  ship unit-tested but render-inert; the keymap extension pattern (#18); the headed lane is
  environmentally blocked this session (fixture-binary baseline proven at #23) — dock baselines
  ride the same deferral.
- **Discovery:** all current (this session rewrote app.rs at #23): `docks: [DockState; 2]` +
  `dock()`/`toggle_dock()` pub on RootView; `cockpit_commands()` static list; render = full-
  viewport pane tiling + occluded palette overlay; `layout.rs` owns DockSide/DockState/toggled.
- **Decisions:** D1–D4 in the spec (pure width fn in layout.rs; cmd-b/cmd-shift-b; one source of
  truth for widths; placeholder dock titles).
- **Open questions for Design:** RegionWidths as struct vs tuple; render = one flex row of three
  sized divs vs absolute (the center's pane tiling is absolute WITHIN its region — nesting
  absolute inside a flex child needs the child `.relative()`); dock title strings; whether
  `region_widths` also returns the center X-OFFSET (the pane bounds origin) or the shim derives
  it (= left width) — mutation-target implications.
- **AAR id:** `e928eb68-9ccf-4e72-81af-b764a01a3ddd`.

## Phase 2 — Design

### Architecture / approach
PURE (layout.rs — it owns `DockSide`/`DockState`):
```rust
pub struct RegionWidths { pub left: f32, pub center: f32, pub right: f32 }  // Copy, PartialEq
pub fn region_widths(window: f32, left: DockState, right: DockState, dock: f32) -> RegionWidths
// per side: Open → dock, Closed → 0.0; center = window − left − right (a remainder, never a
// constant). R31.
```
PURE (keymap.rs): `default_bindings` gains `cmd-b → "toggle-left-dock"` and
`cmd-shift-b → "toggle-right-dock"` (the arg order proven by the existing cmd-shift-p binding;
no collision — only cmd-shift-p/cmd-d/cmd-w exist).

SHIM (app.rs, existing exclude): `const DOCK_WIDTH: f32 = 220.0`; render computes ONE
`region_widths(viewport.w, dock(Left), dock(Right), DOCK_WIDTH)` and sizes everything from it
(D3): left dock div absolute at x=0 w=rw.left (skipped when 0.0), the #23 pane tiling over
`Rect { x: rw.left, y: 0, w: rw.center, h }`, right dock div at x=rw.left+rw.center. Dock panels:
`surface` bg + a title ("Files" left / "Details" right) — placeholder per D4. A small
`toggle_dock_state(side)` shim helper (flip WITHOUT cx) backs both `toggle_dock(side, cx)` (public
R6 API unchanged) and the two new `dispatch_action` arms (the key-handler call site already
notifies after dispatch). `cockpit_commands` gains the two palette entries carrying the chords
(chip data; Enter-dispatch = seq-4).

### File manifest
- M `crates/marley_app/src/layout.rs` — `RegionWidths` + `region_widths` + unit tests.
- M `crates/marley_app/src/keymap.rs` — 2 new default bindings (+ test extension).
- M `crates/marley_app/src/app.rs` — the 3-region render, `toggle_dock_state`, 2 dispatch arms,
  2 palette commands, DOCK_WIDTH.
- M `crates/marley_app/src/lib.rs` — re-export `RegionWidths`/`region_widths` (M2 panels bind).
- M `docs/specs/SPEC-app-shell.spec.md` — R19 amended (5 chords), NEW R31 (region widths), AC
  rows, Test-Plan entries, Mutation-Targets line.
- M `CHANGELOG.md`; M `docs/marley_architecture/app_shell.md` (complete phase).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `region_widths_all_four_combinations` — ASYMMETRIC cases kill the left↔right swap (Open/Closed: left=220, right=0; Closed/Open mirrored; Open/Open; Closed/Closed → center == window); center asserted as the exact remainder for a distinct window width (kills remainder→constant) | unit (layout.rs) |
| REQ-002 | `default_keymap_maps_named_chords` extended to the 5 chords; `action_for_unbound_is_none` unchanged | unit (keymap.rs) |
| REQ-003 | the pure halves above + the existing `DockState::toggled` tests; the dispatch arms + region render are the app.rs shim (existing exclude) — review + the deferred headed lane | unit + review |
| REQ-004 | review (static palette list lives in the skip'd shim fn; Enter-dispatch tests land at seq-4) | review |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the region render + dispatch arms (app.rs — the EXISTING documented exclude; no new
exclude files).

### Risks / decisions
- The left↔right swap mutant is only killable with ASYMMETRIC dock-state fixtures — pinned in the
  test plan (the symmetric-fixture trap class from M0's `w*h` lesson).
- `region_widths` takes `dock` as a parameter (pure); the 220.0 const lives shim-side — M2 panels
  can pass per-side widths without touching the fn's contract.
- gpui absolute rendering nests fine under the `.relative()` root (proven at #23); the center
  tiling keeps using `pane_rects` with an offset origin — already exercised by the off-origin
  unit test (`pane_rects_children_sum_exactly_to_parent` uses x=64/y=32).

## Phase 3 — Implement
- **Built (per manifest):** `layout.rs` — `RegionWidths` + `region_widths` (a `side` closure per
  DockState arm; center = remainder); `keymap.rs` — the two new default bindings (doc updated);
  `app.rs` — `DOCK_WIDTH = 220.0`, `toggle_dock_state(side)` (shared by `toggle_dock` + the two
  new dispatch arms), the two dock panel divs (absolute, `surface` bg, "Files"/"Details" titles,
  skipped at zero width), the center pane tiling inset via `center_bounds` from the ONE
  `region_widths` result (D3), `cockpit_commands` + the two dock commands (ids 4/5, chord chips);
  `lib.rs` re-exports `region_widths`/`RegionWidths`. SPEC-app-shell: R19 amended (5 chords),
  R31 added, AC rows 19/31, Test-Plan R31 line, Mutation-Targets `region_widths` line.
  CHANGELOG entry.
- **Deviations from design:** none.
- **Verification at this phase:** `cargo check -p marley` clean; `cargo clippy -p marley
  --all-targets -- -D warnings` clean; 47 existing lib tests pass. The R31 four-combination test
  + the keymap test extension are Phase 4.

## Phase 3.5 — Inspect
- **Critics run:** 2 (small diff) — (A) region-math + wiring prober (7/7 probes incl. a
  2401-width clamp sweep, gpui/taffy negative-size source reads, the cmd-shift-b keystroke
  mechanics vs the working cmd-shift-p); (B) simplification + provenance + doc-accuracy.
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | MED | Unclamped center goes NEGATIVE below 440pt window width — the right dock paints OVER the left (x = left+center regresses) and the pane tiling REVERSES (negative child widths move origins leftward). Probed end-to-end; reachable by ordinary resize (no window min size). Spec R31 as written MANDATED the bug. | REAL (probed) | `center: (window − l − r).max(0.0)` + R31/AC-31/test-plan/mutation-targets/CHANGELOG co-amended; the clamp is a REACHABLE tested arm (the shrunken-window unit case lands at validate — kills clamp-deletion + max→min). |
  | F2 | MED | CHANGELOG claimed palette "chord chips" — no chip renders anywhere (`Command.binding` has zero readers). | REAL | Reworded: chords registered on the `Command`; chip render + Enter-dispatch = seq-4. |
  | F3 | LOW | `default_keymap_maps_named_chords` asserts 3/5 chords — the two new tuples are silently unasserted (line-covered via the vec literal; mutants can't delete vec tuples, so ONLY an explicit assertion catches a wrong action string). | REAL | Flagged to validate: the test MUST extend to all five (already in the plan; now explicit). |
  | F4 | LOW | Stale keymap.rs module doc ("handlers land in later tickets #19/#20"). | REAL | Doc refreshed (handlers live in dispatch_action). |
  | F5 | LOW | Stale lib.rs crate doc ("the M1.A app shell", pure list missing keymap/layout/palette/workspace). | REAL | Crate doc rewritten to the cockpit reality + full pure list. |
- **Rejected / accepted as-is:** the `DockSide`→index match ×2 (pre-diff count, helper marginal);
  dock divs inline at n=2 (clearer than a closure in the verbatim-conversion shim); `RegionWidths`
  widths-only (3 Rects would triplicate vertical geometry); Open+dock_w=0.0 silently skips a
  panel (unreachable with the const; debug_assert candidate when M2 makes widths user-set —
  noted); fn-modifier ignored in chords (pre-existing, all chords equally).
- **Provenance lens:** CLEAN — generic-descriptive identifiers, not Zed's (PascalCase actions,
  240px, cmd-r right dock) nor VS Code's (cmd-alt-b); no secrets; spawn inputs untouched.
- **Post-fix verification:** fmt/check clean; 47 lib tests pass; critic A's probe P6/P7 shapes
  (clamp sweep + mutant-kill) become the validate-phase unit cases.

## Phase 4 — Validate
- **Tests added:** `region_widths_all_four_combinations` (asymmetric combos kill the left↔right
  swap; distinct window widths kill remainder→constant) + `region_widths_shrunken_window_clamps_center`
  (300pt sub-minimum + the exact 440pt boundary — kills clamp-deletion and `max`→`min`, the F1
  inspect arm); `default_keymap_maps_named_chords` extended to ALL FIVE chords (the F3 flag — the
  two dock chords differ only by shift, pinning the shift bit + action strings).
- **Runs (actual):** `cargo nextest run --workspace` → **459 passed, 6 skipped**; doctests green
  (11 crates ok).
- **Visual (gate:15):** PASS headless (component asserted + harness suite); the headed lane
  remains environmentally blocked this session (#23's fixture-binary proof) — the dock baselines
  ride the same desktop-session deferral (forge #23/#24 comments track it).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15 first try**; coverage 100% lines;
  receipt written.
- **Pre-existing failures:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG entry (clamp wording corrected at inspect); `app_shell.md` — the
  `region_widths` bullet added to the PURE list and the stale "Deferred (M1.B+)" section
  rewritten to the real M1.C-seq-4+/M2 state (shipped items enumerated). SPEC amendments landed
  at implement/inspect.
- **AAR capture:** `BF-claude-unclamped-remainder-region-reverses-layout-under-min-window-001`
  (the spec-mandated-bug class — the fix was spec+code TOGETHER); aar-submit `completed`.
  Lesson: probing critics again beat review — the negative-center bug was invisible in the
  four-combination happy-path table the spec itself prescribed.
- **Ticket:** forge #24 → done; local doc → closed/; pipeline pair archived.
