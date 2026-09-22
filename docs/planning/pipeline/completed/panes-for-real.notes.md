# panes for real — Notes

- **Forge ticket:** #23 `7007e51f-377b-4e86-8ea0-c3ad761897e9`
- **AAR:** `57b31936-1ee9-455c-8619-edc4a01cd413`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-023-panes-for-real.md
- **Pipeline spec:** panes-for-real.spec.md

## Phase 1 — Plan
- **Request:** forge #23 (M1.C seq-2 FLAGSHIP, auto-approved run) — per-pane sessions + real
  split render + focus. Replaces the M1.B status-line placeholder.
- **Classification / tier:** work pipeline, `feature` — one shippable slice (one new pure module
  + the app.rs shim rewrite + spec amendment). Big but coherent; no sub-split needed.
- **Forge recall (§18.3):** bulletins none; operative priors (M1.A/M1.B corpus): the `PtyChannel`
  mock-seam precedent → the registry is generic over the session type with a spawn-fn seam;
  BF-alacritty-pty-drop → drop-kills-PTY is the DESIRED close semantics;
  PR-build-test-fixtures-via-the-real-api → 2x1/2x2 fixtures via real `split` calls;
  PR-write-first-not-research → gpui shim written in-context against app.rs/marley_spike;
  PR-shim-needs-both-cov-exclude-and-mutants-skip; the gate-15 component-dir trap (dir already
  correct); TICKET-022's PR-verify-decoder-parse-order (probe real seams before locking scope).
- **Discovery (verified in-tree):**
  - `app.rs:28-43` — RootView owns ONE `session`/`buffer`/`caret`; `dispatch_action` (:188)
    targets `.panes().first()`; render (:305-327) = flat block rows + the `⬜ panes:N docks:LR`
    status line; the pump-timer (:62) pumps the single session; `on_key_down` routes
    palette → keymap → prompt.
  - `layout.rs` — the pure PaneGroup algebra to render (single/split/close/panes/neighbor/
    equal-ratios); `panes()` is depth-first (the render + invariant anchor).
  - `input.rs` — `apply_key(&mut Buffer, &mut CharOffset, Key)` + `KeyOutcome::{Edited,Submit,
    Ignored}` — per-pane prompt state composes as {Buffer, caret} per pane.
  - `tests/headed_shell.rs` — the headed proof pattern is AX + blank-detector/content-region
    analysis (strict pixel baselines deferred, AD-claude-headed-visual-baseline-text-tolerance).
    NO keystroke-injection helper exists in the harness yet — REQ-006 needs one (os_shim class)
    or an osascript keystroke from the test itself; design decides.
  - Shell integration: one shared ZDOTDIR serves N sessions (`session_env` per spawn points at
    the same rc dir) — no per-pane rc work needed; #22's AnsiCQuoted round-trip already proven.
- **Decisions:** D1–D6 in the spec (spawn-seamed generic registry; pure local Rect + last-child
  remainder; survivor-focus after close; real-API fixtures; blank-detector headed proof;
  registry↔tree invariant).
- **Open questions for Design:** exact focus-after-close rule (collapse survivor vs neighbor());
  module name (`workspace.rs`); Rect f32 vs u32-px; how the shim converts rects to gpui layout
  (absolute positioning vs nested flex); pump fairness over N sessions (one notify per frame);
  the harness keystroke-injection shape; whether click-to-focus is per-pane `on_mouse_down`
  (likely) and its hit-test source (the rect map).
- **AAR id:** `57b31936-1ee9-455c-8619-edc4a01cd413`.

## Phase 2 — Design

### Architecture
NEW pure module `crates/marley_app/src/workspace.rs` (gpui-free; `layout.rs` needs NO change —
`PaneGroup::{Leaf, Split{axis, children, ratios}}` is a pub, directly-matchable enum, binary by
invariant):

```rust
pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }   // window points, PartialEq

pub fn pane_rects(group: &PaneGroup, bounds: Rect) -> Vec<(PaneId, Rect)>
// Leaf → (pane, bounds). Split → children tile bounds along `axis`, child i's extent =
// parent_extent * ratios[i], origin accumulating across siblings. NO last-child remainder arm —
// see D-2.2. Depth-first order == PaneGroup::panes() order.

pub struct PaneState<S> { pub session: S, pub buffer: Buffer, pub caret: CharOffset }

pub struct Workspace<S> { group: PaneGroup, panes: HashMap<PaneId, PaneState<S>>,
                          focused: PaneId, next_id: u64 }

pub enum SplitError<E> { Spawn(E) }            // spawn is the ONLY fallible step
pub enum FocusError { PaneNotFound }

impl<S> Workspace<S> {
  pub fn new(initial: S) -> Self                       // PaneId(0), focused, next_id=1
  pub fn split_focused<E>(&mut self, axis, dir, spawn: impl FnOnce() -> Result<S, E>)
      -> Result<PaneId, SplitError<E>>
  // ORDER: spawn FIRST (Err → workspace untouched); then group.split(focused,…) whose success
  // is an invariant — asserted `debug_assert!(ok)` on an always-computed bool (the coverage-model
  // pattern), NOT a dead map_err arm; insert state; FOCUS THE NEW PANE (D-2.4); return id.
  pub fn close_focused(&mut self) -> Result<PaneId, PaneError>
  // group.close(focused)? propagates LastPane (reachable) — layout's own error type, no re-map;
  // drop the state (kills the PTY for S=TerminalSession); focus-after-close per D-2.3; returns
  // the closed id.
  pub fn focus(&mut self, pane: PaneId) -> Result<(), FocusError>   // absent → PaneNotFound
  pub fn focused(&self) -> PaneId
  pub fn focused_state_mut(&mut self) -> Option<&mut PaneState<S>>  // map passthrough, no dead arm
  pub fn states_mut(&mut self) -> impl Iterator<Item = (&PaneId, &mut PaneState<S>)>  // pump-all
  pub fn group(&self) -> &PaneGroup                                  // render + invariant tests
  pub fn pane_ids(&self) -> Vec<PaneId>                              // sorted registry keys
}
```

`app.rs` (shim, ACCEPTED-UNTESTABLE, existing exclude): RootView swaps {session,buffer,caret,
next_pane_id} for `workspace: Workspace<TerminalSession>` + keeps zdotdir/theme/keymap/palette/
docks. `spawn_session(zdotdir) -> Result<TerminalSession, SessionError>` builds the integrated-zsh
SessionOptions (shared ZDOTDIR, cwd=current_dir, 80×24). Render: bounds from
`window.viewport_size()` → `pane_rects` → a `.relative().size_full()` container with one
ABSOLUTE-positioned child div per rect (`.absolute().left/top/w/h(px)`) — the pure rect fn is the
single source of truth; each pane div: 2px border (`accent` when focused else `border` from
ThemeColors — the affordance), that pane's block rows + `▏prompt`, and
`.on_mouse_down(Left, focus(pane_id))`. Pump-timer iterates `states_mut()` pumping every session
(notify if ANY produced events). Key routing: palette branch unchanged; else keymap chord →
dispatch (`split-pane` → `split_focused(Horizontal, After, spawn)`; `close-pane` →
`close_focused()`); else `apply_key` on the FOCUSED pane's buffer/caret; Submit → that pane's
`write_command` + pump.

Harness: additive `send_keystroke(key, cmd_down)` on `HeadedSession` — osascript System Events
`keystroke … using {command down}` — implemented INSIDE the existing excluded shim files
(os_shim.rs + a launch.rs wrapper), so NO gate-exclude change. Headed test `headed_panes.rs`
(#[ignore], headed lane): launch → cmd-d → settle → capture → LEFT half non-blank AND RIGHT half
non-blank below the titlebar (two live panes) + accent-colored pixels present (the affordance) —
the shipped blank-detector pattern (D5).

### Decisions
- D-2.1 Spawn-per-call seam (`impl FnOnce() -> Result<S, E>` argument), not a stored closure —
  no generic-lifetime knots; mock tests pass `|| Ok(MockSession…)` / `|| Err(…)`.
- D-2.2 **REQ-001 amended (equivalent-mutant hazard, caught at design):** the planned
  "last child absorbs rounding" remainder arm is UNKILLABLE today — the tree is binary with
  equal ratios (0.5), and f32 halving is exact (`x*0.5 + x*0.5 == x` for all normal x), so a
  remainder-arm mutant computes identical values (the w*h-loop-bound lesson class). NO remainder
  arm ships: every child extent = `parent * ratios[i]`; tests assert children sum EXACTLY to the
  parent for the binary tree. Revisit when M2 drag-resize introduces non-halving ratios.
- D-2.3 Focus-after-close = the **depth-first predecessor** of the closed pane (the successor
  when it was first): capture `order = group.panes()` BEFORE close; `idx = position(closed)`;
  new focus = `order[idx-1]` if `idx > 0` else `order[1]` (exists — LastPane guard passed).
  Deterministic, pure, index-arithmetic-rich (mutation targets: the -1, the idx==0 arm,
  before-vs-after capture).
- D-2.4 Focus follows the NEW pane on split (standard terminal UX; observable + tested).
- D-2.5 PTY size stays 80×24 per pane at M1.C; resizing the PTY to the pane rect is deferred
  (added to Out) — cosmetic wrap mismatch only, no correctness impact on Blocks.
- D-2.6 Render consumes `pane_rects` via absolute positioning (one source of truth). FALLBACK if
  gpui's absolute API fights write-first: a recursive flex mirror (Split → flex row/col with
  ratio-weighted children) — `pane_rects` then remains the spec'd pure model consumed by tests;
  either way the pure surface is identical.

### File manifest
- A `crates/marley_app/src/workspace.rs` — everything above + full unit tests.
- M `crates/marley_app/src/lib.rs` — `mod workspace;` + re-exports (Workspace, PaneState, Rect,
  pane_rects, SplitError, FocusError).
- M `crates/marley_app/src/app.rs` — the shim rewrite (fields, spawn_session, absolute render +
  affordance + click-to-focus, pump-all, focused routing, dispatch retarget).
- M `crates/marley_visual_harness/src/os_shim.rs` (+ `launch.rs` wrapper) — additive
  `send_keystroke` (both files already coverage/mutation-excluded).
- A `crates/marley_app/tests/headed_panes.rs` — the #[ignore] headed split proof.
- M `crates/marley_app/tests/integration.rs` — ADD `workspace_two_real_sessions_are_independent`
  (#[serial]; real spawn ×2 via split_focused; write to focused only; other pane block-free;
  close_focused; survivor still works).
- M `docs/specs/SPEC-app-shell.spec.md` — R27 (rect tiling) / R28 (per-pane lifecycle) / R29
  (focus routing + after-close + follows-split) / R30 (registry↔tree invariant) + AC rows +
  Test-Plan + Mutation-Targets entries.
- M `CHANGELOG.md`; M `docs/marley_architecture/app_shell.md` (complete phase).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `pane_rects_single_fills_bounds`; `pane_rects_2x1_partitions_along_axis` (real split API, BOTH axes — kills the H↔V arm swap + ratio→w-vs-h mapping); `pane_rects_nested_2x2_tiles_exact_quadrants` (real API: split H then V; 4 exact quadrant rects, order == panes()); `pane_rects_children_sum_exactly_to_parent` (origin accumulation — kills the `+= extent` drop) | unit |
| REQ-002 | `split_spawns_via_seam_and_inserts_state` (mock spawn counter = 1, new state independent); `close_drops_state` (Rc<Cell> drop-tracking mock hits 1); `split_spawn_failure_leaves_workspace_unchanged` (Err spawn → tree/registry/focus/next_id all unchanged — kills spawn-ordering mutants); `workspace_two_real_sessions_are_independent` (integration, #[serial]) | unit + integration |
| REQ-003 | `focused_state_mut_routes_to_focused_only` (mutate via accessor; other panes' buffers untouched); `split_focuses_new_pane` (D-2.4); `close_targets_focused` (returned closed id == previously focused) | unit |
| REQ-004 | `close_moves_focus_to_depth_first_predecessor` (3 panes, close middle); `close_first_pane_focuses_successor` (idx==0 arm); `close_last_pane_rejected_unchanged` (LastPane; everything intact) | unit |
| REQ-005 | `registry_matches_tree_after_every_op` (scripted split/split/focus/close/close sequence; after EACH op assert sorted pane_ids == sorted group.panes()) | unit |
| REQ-006 | `headed_split_shows_two_live_panes` (#[ignore]; cmd-d via send_keystroke; left+right halves non-blank; accent border pixels present) — run in the headed lane during validate | headed |
| REQ-007 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Mutation-target inventory (beyond the table): `focus()` absent-arm, `focused` reassignment on
split, `next_id` increment, the `debug_assert!(split_ok)` always-computed bool, `pane_ids`
sort, `states_mut` passthrough. Uncoverable: app.rs + the two harness shim files (EXISTING
excludes — no gate change).

### Risks
- gpui absolute-positioning API shape — write-first + `cargo check` probe; D-2.6 fallback ready.
- Headed keystroke needs macOS Accessibility for System Events keystrokes — Warp already granted
  (M0 precedent); the headed test stays #[ignore] so the per-commit gate never depends on it.
- Two real PTY spawns in one #[serial] test — existing per-test PTY pattern, modest timeout.
- `PartialEq` on f32 Rects — exact math only (binary halving); tests use power-of-two-friendly
  bounds; no epsilon comparisons needed.

## Phase 3 — Implement
- **Built (per manifest):**
  - `workspace.rs` — `Rect`, `pane_rects` (depth-first, axis-arm cursor accumulation, no
    remainder arm per D-2.2), `PaneState<S>`, `Workspace<S>` with `new`/`split_focused`
    (spawn-first, focus-follows-split, `debug_assert!(split_ok)` on an always-computed bool)/
    `close_focused` (pre-close order capture → predecessor-or-successor)/`focus`/`focused`/
    `focused_state_mut`/`states_mut`/`group`/`pane_ids`, `SplitError<E>`, `FocusError`.
  - `lib.rs` — `mod workspace` + re-exports.
  - `app.rs` — full shim rewrite: `Workspace<TerminalSession>` field (single session/buffer/caret
    fields GONE), `spawn_session(zdotdir)`, pump-ALL timer, focused key routing +
    `on_submit`-to-focused, dispatch split/close on FOCUSED, absolute-positioned per-rect pane
    divs (accent/border focus affordance, `overflow_hidden`, click-to-focus listeners), palette
    overlay now absolute (¼-inset, half-width — closer to R14's "centered" than the old flow
    child), the status-line placeholder REMOVED.
  - `marley_visual_harness` — `os_shim::send_keystroke(pid, key, command_down)` (osascript
    System Events, frontmost-by-pid + keystroke) + `HeadedSession::send_keystroke` wrapper
    (io err → `Spawn`; osascript failure → `AccessibilityDenied`). Both in the EXISTING excluded
    shim files with `mutants::skip` + justifications — zero gate-config change.
  - SPEC-app-shell — R27–R30 + AC rows 27–30 + Test-Plan entries (unit/integration/headed) +
    4 Mutation-Targets lines + ACCEPTED-UNTESTABLE additions + the stale "does not drive
    sessions" Out bullet corrected. CHANGELOG `### Added` entry.
- **Deviations from design (with reason):**
  - D-3.1 `layout.rs` touched after all: `PaneId` gains a `Hash` derive (HashMap key) — a pure
    derive addition, no behavior change (design said "layout.rs needs NO change"; the derive was
    unforeseen).
  - D-3.2 `Workspace::state(pane)` read accessor added (the render's per-pane read path) —
    listed in the design code block but not the manifest bullet; called out for completeness.
  - D-3.3 `f32::from(Pixels)` (the `.0` field is private in gpui 0.2.2).
- **Verification at this phase:** `cargo check --workspace` clean; `cargo clippy -p marley
  -p marley_visual_harness --all-targets -- -D warnings` clean; `cargo fmt` applied; the 31
  existing marley lib tests still pass. Workspace tests + integration + headed are Phase 4.

## Phase 3.5 — Inspect
- **Critics run:** 4 parallel general-purpose critics, all instructed to RUN probes (the
  TICKET-022 lesson): (A) workspace-model prober — 960-op seeded differential fuzz + exact-tiling
  checker + close-focus rule at every position + spawn-atomicity + drop accounting; (B) gpui shim
  reviewer — verified every styled/interactivity API against the gpui 0.2.2 registry sources +
  a compile probe; (C) lifecycle prober — real-PTY isolation/close/churn/pump-cost measurements;
  (D) provenance + simplification — registry scan for Zed pane sources (none exist locally),
  identifier-leak grep, injection audit.
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | HIGH | pump-all costs ~12 ms per IDLE pane per 16 ms tick (98 ms measured at 8 panes — 6× frame budget at 8, blown at 2): `pump()`'s retry budget sleeps ~1 ms × 8 on every idle call | REAL (measured) | session.rs idle fast-path: a LEADING `WouldBlock` returns immediately; retry sleeps only mid-burst (after bytes read this call). Re-pinned `pump_leading_wouldblock_is_idle_fast_path_not_fatal` + NEW `pump_midburst_wouldblock_retries_within_budget`. RE-MEASURED with the critic's probe: 8 idle panes = **19.7 µs**/tick (~5000×). |
  | F2 | MED | close-pane blocks the main thread in `Pty::drop`'s unbounded `child.wait()` — 603 ms measured with a running foreground child; FOREVER for a HUP-immune child (window-close hangs too) | REAL (measured) | `Workspace::close(pane)` (generalizes close_focused) RETURNS the state — the caller owns teardown; app.rs drops it on a detached reaper thread; `PtyChannel` gains `Send` (supertrait → `Box<dyn PtyChannel>: Send` → `TerminalSession: Send`, compiler-verified). |
  | F3 | MED | a pane whose shell exits lingers dead forever — pump events discarded, writes swallowed | REAL | app.rs pump loop collects `ChildExited`/`Err` panes and auto-closes them (reaper-thread drop) unless last; the last pane stays visibly dead (window-close-on-exit = a later cut, documented). |
  | F4 | MED | palette overlay is mouse-transparent (gpui hit-testing is not topmost-wins; no hitbox without listeners/occlude) — clicks on the open palette refocus the pane beneath | REAL (verified against gpui src) | `.occlude()` on the overlay (`HitboxBehavior::BlockMouse`). |
  | F5 | MED | AppleScript injection: `send_keystroke`'s `key` interpolates raw into the script — a `"` breaks out (pub API, dev-only today) | REAL | input validated: single ASCII-alphanumeric or `InvalidInput`; multi-char/named-key footgun documented. |
  | F6 | MED | R27's "tiles exactly" is edge-false on non-dyadic fractional bounds: origin ACCUMULATION drifts ≤1 f32 ULP (41/80 fractional viewports; areas always exact — 180/180) | REAL (measured; doc-level) | R27 + fn doc reworded: area-exact + sibling-boundary-exact; ≤1 ULP far-edge caveat recorded; revisit at M2 drag-resize. |
  | F7 | LOW | `pane_rects` silently drops panes on a malformed hand-built Split (`ratios.len() < children.len()` — unreachable via Workspace, reachable via pub fields) | REAL | `debug_assert_eq!(children.len(), ratios.len())` in the Split arm. |
  | F8 | LOW | `unwrap_or_default()` on the close position lookup = silent-wrong on a future refactor | REAL | `.expect("the closed pane is in the pre-close order")` (loud invariant; internal, not an input path). |
  | F9 | LOW | boundary-exact clicks focus the earlier pane (inclusive far edges, both hitboxes fire) | REJECTED-as-fix / accepted | needs an exact f32 hit — cosmetic-rare; `stop_propagation` would break root focus tracking. Documented, left. |
  | F10 | LOW | docks render-inert after the status line's removal | ACCEPTED (intended) | seq-3 renders docks; R6 state contract stays unit-covered; `pane_rects` bounds is the ready seam. |
  | F11 | LOW | PTY stays 80×24 regardless of rect; prompt clips below the pane fold; zdotdir never cleaned | ACCEPTED (pre-existing modalities) | D-2.5 deferral (resize API exists — M2 wiring); scrollback = M1 gap (backlog); zdotdir shared-single-dir is not worsened. |
  | F12 | LOW | `zdotdir.clone()` per split; `theme.colors.clone()` per frame | REAL (style) | clone dropped via 2021 disjoint capture; colors clone kept (pre-existing idiom, stack copy). |
- **Provenance lens:** CLEAN — no Zed workspace/pane source exists anywhere locally (registry
  scanned; gpui 0.2.2 has zero PaneGroup/Member/pane_group hits; only permissive `Axis`);
  structural design is dissimilar at every joint (binary Leaf/Split + separate generic registry +
  spawn seam vs Zed's n-ary Member tree with embedded entities); zero fork-identifier hits; no
  secrets; spawn inputs fixed.
- **Post-fix verification:** workspace + terminal fmt/clippy/check clean; marley_terminal 69+5
  tests green (2 pump tests re-pinned/new); the critic's OWN probes re-run post-fix: pumpcost
  **19.7 µs**/8-pane tick PASS, isolation PASS, churn PASS, close PASS (exited-pane close 56 µs).

## Phase 4 — Validate
- **Tests added (per the Phase 2 plan + inspect additions):**
  - `workspace.rs` (17 unit tests): R27 — `pane_rects_single_fills_bounds`,
    `pane_rects_2x1_partitions_along_axis` (both axes + Before-ordering),
    `pane_rects_nested_2x2_tiles_exact_quadrants` (real-API fixture, order == panes()),
    `pane_rects_children_sum_exactly_to_parent` (off-origin accumulation); R28 —
    `split_spawns_via_seam_and_inserts_state`, `split_spawn_failure_leaves_workspace_unchanged`
    (incl. next_id non-consumption), `close_returns_state_and_caller_drops_it` (drop-tracking
    mock, the reaper seam); R29 — `split_focuses_new_pane`,
    `focused_state_mut_routes_to_focused_only`, the 3 close-focus-rule tests
    (predecessor/middle/first-successor), `close_unfocused_keeps_focus`,
    `close_last_pane_rejected_unchanged`, `absent_pane_ops_rejected_unchanged`; R30 —
    `registry_matches_tree_after_every_op` (asserted after EACH op of a scripted sequence).
  - `session.rs`: `pump_burst_then_endless_wouldblock_exhausts_budget_cleanly` (kills the
    `||`→`&&` debug-underflow + `-=`→`+=` timeout mutants on the idle fast-path); the idle
    fast-path + midburst tests landed at inspect; the stale budget-drain comment re-pinned.
  - `tests/integration.rs`: `workspace_two_real_sessions_are_independent` (`#[serial]`, two real
    zsh sessions via the spawn seam — isolation both ways, close, survivor still executes).
  - `tests/headed_panes.rs`: `headed_split_shows_two_live_panes` (`#[ignore]`, cmd-d via
    `send_keystroke`, two non-blank halves + accent-affordance pixel check).
- **Runs (actual):** `cargo nextest run --workspace` → **457 passed, 6 skipped** (all new tests
  listed PASS); doctests green.
- **Headed lane (REQ-006): BLOCKED — environmental, documented.** This session's process tree
  has NO WindowServer access: the app runs but `count of windows = 0` via AX — and the
  M0-proven `marley_harness_selftest` fixture binary is ALSO windowless here (conclusive
  baseline); `launchctl asuser` bootstrap didn't help; osascript/AX itself works (71 processes
  queried). Previous sprints' headed lanes ran inside Warp's Aqua session. The headed test is
  WRITTEN per the proven blank-detector pattern; the run is deferred to the next desktop-context
  session (`cargo test -p marley --test headed_panes --test headed_shell -- --ignored`) —
  forge #23 carries the follow-up comment. gate-15 (headless by design) is unaffected; every
  non-paint behavior is proven by the pure suite + the real-PTY integration test.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15 first try**. Coverage 100% lines
  (workspace.rs fully in the denominator); mutation **29 caught / 0 missed → MSI 100.0%**
  (diff-scoped over workspace.rs + session.rs + the shim-adjacent edits); receipt
  `36b6ea79f9e8104ebabfbcfabb1fd9f357473a02`.
- **Pre-existing failures:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` entry (with the inspect-hardening addendum);
  `app_shell.md` — workspace.rs joins the PURE list, the shim paragraph rewritten to the
  multi-pane render/pump-all/reaper/auto-close reality; `terminal_blocks.md` — the pump idle
  fast-path + `PtyChannel: Send` recorded with the measured numbers. SPEC-app-shell R27–R30 +
  SPEC-terminal-blocks test-plan additions landed at implement/inspect.
- **AAR capture:** 2 failures + 1 prevention rule (inspect) +
  `AD-claude-workspace-spawn-seam-and-caller-owned-teardown-001`; aar-submit `completed`
  (4 novel findings). Lessons: measuring critics (real probes, real PTYs, real timing) caught
  BOTH ship-blockers code review missed — the 12 ms idle-pump floor and the blocking reap;
  the design-phase equivalent-mutant analysis (remainder arm) saved a validate-phase MSI failure;
  the headed lane requires an Aqua-session context — this session's process tree cannot reach
  the WindowServer (fixture-binary baseline proven), so headed runs belong to desktop sessions.
- **Ticket:** forge #23 → done (with the headed-lane follow-up comment); local doc →
  `tickets/closed/`. Pipeline pair archived to `pipeline/completed/`.
