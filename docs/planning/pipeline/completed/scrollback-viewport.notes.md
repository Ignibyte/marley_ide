# scrollback viewport — Notes

- **Forge ticket:** #32 `ac48e97c-3937-4e91-9a8d-303013498b2b`
- **AAR:** `d27fac81-be82-4699-a3bf-750630018f42`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-032-scrollback-viewport.md
- **Pipeline spec:** scrollback-viewport.spec.md

## Phase 1 — Plan
- **Request:** forge #32 (M1.D "The Daily Driver" seq-5, auto-approved) — scrollback viewport.
- **Classification / tier:** work pipeline, `feature` — one slice (a pure Viewport + PaneState
  field + the app.rs slice/scroll wiring). marley_app only.
- **Discovery (§18):**
  - The pane render (app.rs:574-597): `overflow_hidden()` + `for block in session.blocks()` →
    per block the command line + `output_styled()` lines → colored spans → `pane.child`, then the
    prompt. All rows top-down, clipped — no scroll. The Viewport slices this flat row sequence.
  - `PaneState { session, buffer, caret, history, pty_size }` (workspace.rs) — add `viewport:
    Viewport` (per-pane, like history #29 / pty_size #30).
  - `capacity` reuses the #30 cell-metric read (the render already computes `cell.h`): rows =
    floor(pane_rect.h / cell.h). `content` = Σ(1 + block.output_styled().len()) + 1 (prompt).
    Both computed in the shim, passed to the pure Viewport (params, like plan_resize #30).
  - Deps #30 (cell metric) + #31 (styled rows) shipped.
- **Decisions:** D1–D5 in the spec (pure Viewport per-pane; following-vs-held anchor; materialise-
  on-first-up; re-anchor on scroll-to-bottom; capacity.max(1)).
- **Open questions for Design:** the exact `scroll_up`/`scroll_down` signatures (do they take
  content+capacity to clamp, or is clamping deferred to `visible`? — leaning: scroll_* take them to
  materialise/clamp `top`); whether `following` re-anchor happens in scroll_down or is recomputed
  in visible; the wheel delta → rows conversion (shim); whether PageUp/Down live in the key handler
  (palette-closed) beside the #29 history ↑/↓ (they must NOT collide — Page keys vs arrows).
- **AAR id:** `d27fac81-be82-4699-a3bf-750630018f42`.

## Phase 2 — Design

### Architecture / approach
NEW pure `crates/marley_app/src/viewport.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport { top: usize, following: bool }   // following = pinned to latest; top = first visible row when held

impl Viewport {
    pub fn new() -> Self { Viewport { top: 0, following: true } }   // start anchored to the bottom

    fn max_scroll(content: usize, cap: usize) -> usize { content.saturating_sub(cap) }

    /// The [start, end) row slice to render.
    pub fn visible(&self, content: usize, capacity: usize) -> (usize, usize) {
        let cap = capacity.max(1);
        let start = if self.following {
            content.saturating_sub(cap)                      // bottom `cap` rows
        } else {
            self.top.min(Self::max_scroll(content, cap))     // held, clamped to max_scroll
        };
        (start, (start + cap).min(content))
    }

    /// Scroll up `n` rows — leave following (materialise `top` at the current bottom first), clamp at 0.
    pub fn scroll_up(&mut self, n: usize, content: usize, capacity: usize) {
        let base = if self.following { Self::max_scroll(content, capacity.max(1)) } else { self.top };
        self.top = base.saturating_sub(n);
        self.following = false;
    }

    /// Scroll down `n` rows — re-anchor (resume following) at/over max_scroll; no-op while following.
    pub fn scroll_down(&mut self, n: usize, content: usize, capacity: usize) {
        if self.following { return; }
        let ms = Self::max_scroll(content, capacity.max(1));
        let next = self.top.saturating_add(n);
        if next >= ms { self.following = true; } else { self.top = next; }
    }
}
```
SHIM (app.rs, existing exclude): `content = blocks.iter().map(|b| 1 + b.output_styled().len()).sum::<usize>() + 1` (the +1 prompt line); `capacity = (pane_rect.h / cell.h) as usize` (the #30 cell-metric read). Then render ONLY rows in `viewport.visible(content, capacity)` — iterate the blocks maintaining a running `row_idx`, emitting an element only when `start <= row_idx < end` (no intermediate Vec). Scroll: `on_scroll_wheel` → `scroll_up`/`scroll_down` by the wheel row-delta; `PageUp`/`PageDown` → ±`capacity`; `shift-up`/`shift-down` → ±1 (palette closed; distinct from the #29 history plain ↑/↓).

### Decisions
- D-2.1 All three methods take `(content, capacity)` — clamping/anchoring needs `max_scroll`, which
  depends on both. Consistent with `plan_resize` (#30) taking its inputs.
- D-2.2 `scroll_up` MATERIALISES `top = max_scroll` when leaving following, so the first up-scroll
  lands `n` rows above the latest (not row 0). D-2.3 `scroll_down` re-anchors on `next >= max_scroll`
  (`>=`, so exactly reaching the bottom resumes following). D-2.4 scroll_down while following is a
  no-op (return). D-2.5 `capacity.max(1)` in every path (zero-height pane → ≥1 row, no underflow).
- D-2.6 `following` (default) auto-shows the bottom as content grows; `top`-held keeps the window as
  content grows (until `top > new max_scroll`, then clamp) — the anchor-unless-scrolled behavior.

### File manifest
- A `crates/marley_app/src/viewport.rs` — `Viewport` + `visible`/`scroll_up`/`scroll_down`/
  `max_scroll` + tests.
- M `crates/marley_app/src/lib.rs` — `mod viewport;` + `pub use viewport::Viewport;`.
- M `crates/marley_app/src/workspace.rs` — `PaneState` gains `viewport: Viewport` (default `new()`).
- M `crates/marley_app/src/app.rs` — content/capacity compute, the row-slice render, the scroll
  events.
- M `docs/specs/SPEC-app-shell.spec.md` — R39 + AC + Test-Plan + Mutation-Targets. CHANGELOG; arch.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `visible_following_shows_bottom_capacity` — content 100/cap 24 → (76,100); content 10/cap 24 → (0,10); content 24/cap 24 → (0,24) | unit |
| REQ-002 | `scroll_up_materialises_then_walks_and_clamps` — from following (100/24, max 76): up(10)→held (66,90); up(10)→(56,80); up(1000)→clamp (0,24) | unit |
| REQ-003 | `scroll_down_reanchors_at_bottom` — held top 56 (100/24, max 76): down(10)→(66,90) held; down(50)→next 116≥76→following (76,100); down while following→no-op (still (76,100)) | unit |
| REQ-004 | `held_window_holds_as_content_grows` — held top 30 (100/24): content→200 → still (30,54); content→40 (max 16) → clamp (16,40) | unit |
| — | `capacity_zero_clamps_to_one` — content 50, cap 0 → cap 1 → following (49,50) | unit |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs content-count + cell-metric capacity + the row-slice render + the scroll
events (existing shim exclude).

### Risks / decisions
- The materialise-on-first-up (D-2.2) is the subtle bit — without it, the first up-scroll from the
  bottom jumps to row 0 (a full-history jump) instead of one screen up. REQ-002 pins it.
- scroll_up when `content <= capacity` (max_scroll 0) sets `top=0, following=false` — harmless
  (visible held-at-0 == following when everything fits); a subsequent scroll_down re-anchors. No
  extra guard (keeps the mutation surface minimal); noted.
- Wheel-delta → rows is a shim conversion (gpui `ScrollWheelEvent` lines vs pixels) — confirmed at
  implement; the pure Viewport takes a row count.
- PageUp/Down + shift-arrows must not collide with #29's plain ↑/↓ history recall — the shim routes
  by the exact key + shift modifier.

## Phase 3 — Implement
- **Built (per manifest):** viewport.rs — `Viewport{top, following}` + Default + `new`/`max_scroll`
  (private)/`visible`/`scroll_up`/`scroll_down`; lib.rs `mod viewport` + export; workspace.rs
  `PaneState.viewport` (default new()) + a NEW `Workspace::state_mut(pane)` accessor (for the
  wheel); app.rs — `content_rows` shim helper, the render row-SLICE (running `row` idx, emit only
  in `[start,end)`), the scroll-KEY block (PageUp/Down ±capacity, Shift+↑/↓ ±1 — BEFORE the #29
  history ↑/↓ so plain arrows still recall), the `on_scroll_wheel` handler (per-pane via state_mut,
  pixel_delta ÷ line_height → rows), and re-anchor `viewport = Viewport::new()` on submit.
  SPEC-app-shell R39 + AC row 39 + Test-Plan + Mutation-Targets; CHANGELOG.
- **Deviations from design:** (D-3.1) capacity = `state.pty_size.1 as usize` (the #30 resize rows
  == floor(pane.h/cell.h) == the viewport capacity) — reused rather than recomputing the metric.
  (D-3.2) added `Workspace::state_mut` (symmetric with `state`) so the wheel scrolls the pane under
  the cursor, not just the focused one — a tested pub fn (Phase 4). (D-3.3) wheel sign: positive
  `delta.y` → scroll_up (toward older) — a shim convention, adjustable.
- **Verification at this phase:** `cargo check -p marley` 0 errors; fmt; 81 lib tests pass. The
  Viewport unit suite + `state_mut` test are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (31 probe assertions + a real cargo-mutants run on viewport.rs). Verdict: PASS,
  no HIGH — the pure Viewport is correct (materialise + held-holds invariants both verified), ZERO
  equivalent/unkillable mutants (all 11 killable), `content_rows` EXACTLY matches the render walk
  (no off-by-one), shim audit clean.
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | MED | The wheel used `rows.abs() as usize` → sub-row trackpad pixel deltas (~0.1) truncate to 0 → precise trackpad scrolling is dead (only large flicks/PageUp/Shift-Up move it). | REAL (daily-driver UX) | Added pure `scroll_steps(remainder, delta)->(i64, f32)` accumulator + `PaneState.scroll_remainder`; the wheel folds each delta so fractions sum to whole rows. |
  | F2 | LOW | `scroll_up` when all content fits (max_scroll 0) detached from following (top=0, following=false) → if content later grows before a submit, the pane stops auto-following. | REAL (edge) | Guard `if max_scroll==0 { return }` at the top of scroll_up — nothing above row 0, stay following. More correct + kills the stuck-state. |
  | F3 | LOW | `content_rows` is mutants::skip + needs a live PTY → its off-by-one rests on inspection. | ACCEPTED | The critic verified it matches the render walk exactly; consistent with the accepted-untestable shim policy. |
  | F4 | LOW | The SPEC R39 mutation catalog lists conceptual mutations cargo-mutants 27.1.0 doesn't emit (the `following` branch, `.max(1)`). | ACCEPTED | Harmless behavioral documentation; the catalog describes intent, the tool sees the 11 real mutants. |
- **CRITICAL Phase-4 note (mutation-invisible behaviors):** cargo-mutants does NOT mutate the
  `if self.following` branch in `visible` nor `capacity.max(1)` — so **REQ-004 (held-holds, the
  ticket's core) and the cap-zero test are protected by ZERO mutants**. MSI 100 will NOT prove
  them. Phase 4 MUST keep both as explicit behavioral assertions regardless of MSI. Also mutant #8
  (`+→*` in `visible`) needs a content≤capacity fixture (100/24 doesn't kill it), and #11 (`>=→<`
  re-anchor) needs the partial-scroll-STAYS-HELD assertion — both in the plan.
- **Post-fix:** the F1 accumulator + F2 guard compile + clippy `-D warnings` clean; 81 lib tests
  pass. `scroll_steps` + the guard get unit tests in Phase 4. Lesson:
  `PR-claude-wheel-delta-truncation-needs-fractional-accumulator`.

## Phase 4 — Validate
- **Tests added (viewport.rs 8 + workspace.rs 1):** `visible_following_shows_bottom_capacity`,
  `scroll_up_materialises_then_walks_and_clamps`, `scroll_down_holds_then_reanchors`,
  `held_window_holds_as_content_grows` (the mutation-invisible CORE invariant — kept explicit per
  inspect), `capacity_zero_clamps_to_one` (mutation-invisible — kept explicit),
  `scroll_up_when_all_fits_stays_following` (the F2 guard), `scroll_steps_accumulates_fractions`
  (the F1 accumulator), `default_is_new_following`; workspace `state_mut_targets_the_named_pane_and
  _none_for_absent`.
- **The gate tail (2 runs):** run 1 — mutation PASS (MSI 100, all 11 Viewport mutants + scroll_steps
  + state_mut killed by the critic's kill-map), coverage FAIL on the `impl Default` (3 lines / 1 fn
  never called — added for clippy's new_without_default). Added `default_is_new_following` → line
  coverage 100%. Run 2 GREEN.
- **Runs (actual):** `cargo nextest run -p marley` → the 9 new PASS (92 marley); full workspace
  green.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100% lines, mutation **26
  caught / 0 missed → MSI 100.0%**, 0 SLOW hangs; receipt written.
- **Pre-existing:** none. (The critic-flagged mutation-invisible behaviors — held-holds + cap-zero
  — are covered by explicit behavioral assertions, not relying on MSI.)

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` gains a `viewport.rs` bullet
  (Viewport + scroll_steps + the shim slice/wheel). SPEC-app-shell R39 at implement.
- **Knowledge captured:** `PR-claude-wheel-delta-truncation-needs-fractional-accumulator` (the F1
  MED — trackpad sub-row deltas truncate without an accumulator). aar-submit `completed`. Win: the
  critic's kill-map (which fixture kills each of the 11 mutants) + its flag that the held-holds +
  cap-zero behaviors are MUTATION-INVISIBLE meant Phase 4 landed MSI 100 first try AND kept the
  ticket's core invariant explicitly tested (not silently relying on the gate). The only gate miss
  was the clippy-required `impl Default` line — a 1-test fix.
- **Ticket:** forge #32 → done; local doc → closed/; pipeline pair archived. 5 of 6 in M1.D — only
  #33 (interactive/raw mode, the flagship) remains.
