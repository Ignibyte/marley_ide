# PTY resize-to-pane — Notes

- **Forge ticket:** #30 `1b3f9d1d-c871-40b7-9ebb-679612780807`
- **AAR:** `a50da8f0-5ae6-4828-b951-a46818017dfa`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-030-pty-resize-to-pane.md
- **Pipeline spec:** pty-resize-to-pane.spec.md

## Phase 1 — Plan
- **Request:** forge #30 (M1.D "The Daily Driver" seq-3, auto-approved) — size the PTY to the pane.
- **Classification / tier:** work pipeline, `feature` — one slice (a pure sizing fn + a PaneState
  field + the app.rs metric-read/resize call). marley_app only (marley_terminal's `resize` ships).
- **Forge recall (§18.3) + discovery:**
  - `TerminalSession::resize(cols: u16, rows: u16) -> Result<(), SessionError>` SHIPS (session.rs:208,
    R17 — sets the winsize ioctl + resizes the alacritty grid; already tested `resize_success` +
    `resize_maps_winsize_error`).
  - `Rect { x, y, w, h: f32 }` + `pane_rects(group, bounds) -> Vec<(PaneId, Rect)>` (workspace.rs
    19/39) — the per-pane geometry the size derives from.
  - app.rs holds `term_cols/term_rows: u16` (from `applied.cols/rows`) + `spawn_session(zdotdir,
    cols, rows)`; panes spawn at 80×24 and never resize (app.rs:102/167/301). So the fixed-size
    deferral from #23/#26 is exactly here.
  - `PaneState` (workspace.rs) — add `pty_size: (u16,u16)` like #29's `history` field.
- **Decisions:** D1–D5 in the spec (pure plan_resize with the Option-folded unchanged-guard; clamp
  ≥1 + non-finite guard; floor divide; pty_size on PaneState; settings stay the initial size).
- **Open questions for Design:** where the gpui cell metric comes from (window `line_height()` +
  measuring an 'M' advance via the text system — a shim read; confirm the gpui-0.2.2 API at
  implement); whether the resize call belongs in `render` (every frame, guarded) or a dedicated
  resize handler (simpler: in render, the unchanged-guard makes it cheap); the u16 cast for a huge
  rect (saturate — but clamp the floored value; a 10000-col pane is fine as u16).
- **AAR id:** `a50da8f0-5ae6-4828-b951-a46818017dfa`.

## Phase 2 — Design

### Architecture / approach
PURE (workspace.rs, beside `Rect`/`pane_rects`):
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellSize { pub w: f32, pub h: f32 }  // monospace advance width + line height (from gpui)

/// Grid count along one axis: floor(extent/cell), clamped ≥1. A non-positive/NaN cell → 1.
fn grid_axis(extent: f32, cell: f32) -> u16 {
    if cell > 0.0 {
        (extent / cell).floor().max(1.0) as u16
    } else {
        1
    }
}

pub fn plan_resize(rect: Rect, cell: CellSize, current: (u16, u16)) -> Option<(u16, u16)> {
    let want = (grid_axis(rect.w, cell.w), grid_axis(rect.h, cell.h));
    (want != current).then_some(want)
}
```
**Why only ONE guard** (the #26 inert-code lesson): the `cell > 0.0` guard is the only branch that
needs to exist — everything else is handled by language semantics that no mutant can kill, so
adding extra guards would be inert code:
- **High end** — a huge `floor()` result saturates on the `as u16` cast (Rust float→int is
  saturating: `75000.0_f32 as u16 == 65535`). No explicit high clamp (that would be an unkillable
  branch).
- **Negative / zero extent** — `.floor().max(1.0)` → 1 (e.g. `-5/8 → -0.625 → max(1.0) → 1`).
- **NaN extent** — `NaN.max(1.0) == 1.0` (Rust `f32::max` returns the non-NaN operand) → 1.
- **cell ≤ 0 / NaN cell** — the `cell > 0.0` guard (NaN fails `> 0.0`) → 1; WITHOUT it, `x/0.0 →
  inf → 65535` (observably wrong), which is exactly what makes the guard's `>` mutation-killable.

SHIM (app.rs, existing exclude): in `render`, the center-region `Rect` (already computed for
painting via `region_widths` #24) → `pane_rects(group, center_rect)` → per `(pane_id, rect)`: read
the gpui monospace `CellSize` (advance of an 'M' × line height, from the window text system), then
`if let Some((c, r)) = plan_resize(rect, cell, state.pty_size) { let _ = state.session.resize(c,
r); state.pty_size = (c, r); }`.

### Decisions
- D-2.1 `grid_axis` PRIVATE; tested THROUGH `plan_resize` (pub) with `current=(0,0)` so each axis's
  clamp/guard is reachable + asserted via the returned `Some`.
- D-2.2 ONLY the `cell > 0.0` guard (minimal killable surface — no inert extent guard / high clamp).
- D-2.3 `PaneState.pty_size: (u16, u16)` inits to `(0, 0)` — a sentinel no real grid can equal
  (grid_axis clamps ≥1), so the FIRST real layout always applies (at most one redundant resize).
  Avoids threading the spawn cols/rows through `PaneState::new`.
- D-2.4 The resize call lives in `render` (guarded cheap by the Option) — no separate resize event
  path needed; the unchanged-guard elides same-size frames.

### File manifest
- M `crates/marley_app/src/workspace.rs` — `CellSize`, `plan_resize`, `grid_axis`, `PaneState
  .pty_size` (+ init in `new`), + unit tests.
- M `crates/marley_app/src/lib.rs` — export `plan_resize` + `CellSize`.
- M `crates/marley_app/src/app.rs` — the render metric-read + per-pane resize call.
- M `docs/specs/SPEC-app-shell.spec.md` — R37 + AC row + Test-Plan + Mutation-Targets. CHANGELOG;
  arch doc at complete.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `plan_resize_floor_divides_each_axis` — NON-SQUARE `rect{w:800,h:240}` + `cell{w:8,h:16}`, `current:(0,0)` → `Some((100,15))` (cols≠rows + all of 800/240/8/16 distinct → kills an axis swap AND divide→multiply, which would give (6400,…)); floor: `w:805` → still `100` (ceil would be 101) | unit |
| REQ-002 | `grid_axis_clamps_below_one_and_guards_bad_cell` — sub-cell `rect{w:5,h:240}` → cols `1` (kills `.max(1.0)`→`.max(0.0)`/`min`); zero cell `cell{w:0,h:16}` → cols `1` (kills `cell>0.0`→`>=`, which would divide-by-zero → 65535); NaN extent `rect{w:NAN,h:240}` → cols `1` | unit |
| REQ-003 | `plan_resize_none_when_unchanged` — `current==(100,15)` computed → `None`; `current==(99,15)` → `Some((100,15))` (kills `!=`→`==`/drop) | unit |
| REQ-004 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs cell-metric read (gpui text system) + `pane_rects`-in-render + the `resize`
call (existing shim exclude; `resize` itself is `marley_terminal`'s tested R17 surface).

### Risks
- The gpui-0.2.2 monospace advance API (measure 'M' via `window.text_system()`) — confirm the exact
  call at implement (shim; worst case the cell size falls back to a constant, still correct-shaped).
- A resize storm during a live window drag — accepted: the `!= current` guard elides same-grid
  frames; a genuine per-column change during drag is the intended behavior (output reflows live).
- `pty_size=(0,0)` sentinel → one first-layout resize even if it lands on 80×24; harmless (resize
  to the current size is a no-op ioctl).

## Phase 3 — Implement
- **Built (per manifest):** workspace.rs — `CellSize{w,h}`, `grid_axis` (`if cell>0.0
  {(e/c).floor().max(1.0) as u16} else {1}`), `plan_resize` (Option-folded unchanged-guard),
  `PaneState.pty_size: (u16,u16)` init `(0,0)`; lib.rs exports `plan_resize` + `CellSize`; app.rs
  render — reads the gpui cell metric (`window.text_system().em_advance(resolve_font(text_style
  .font()), font_size.to_pixels(rem_size))` for w, `window.line_height()` for h; `unwrap_or(px(8.0))`
  fallback), then a resize pass over `states_mut()` looking up each pane's rect from `pane_rects`
  → `plan_resize` → `session.resize` + update `pty_size`. SPEC R37 + AC row 37 + Test-Plan +
  Mutation-Targets; CHANGELOG.
- **Deviations from design:** (D-3.1) the render loop now iterates `&rect_list` and copies
  `pane_id`/`r` out of the refs at the top (both `Copy`), so the loop BODY is byte-identical — the
  resize pass reuses the same `pane_rects` result. (D-3.2) per-pane rect lookup in the resize pass
  is a linear `rect_list.iter().find` (≤ handful of panes) instead of a HashMap — avoids an import,
  trivial cost.
- **Verification at this phase:** `cargo check -p marley` 0 errors; `cargo fmt`. The R37 unit suite
  is Phase 4.

## Phase 3.5 — Inspect
- **Critics:** 2 — (A) arithmetic (33 probes + 26 real cargo-mutants runs across 2 suites), (B)
  shim wiring (build + borrow trace + read of session.rs::resize).
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | A1 | LOW (test-design, LOAD-BEARING) | The `> → >=` gate mutant is killable by EXACTLY ONE fixture: `cell==0` asserting the exact tuple `Some((1,15))`. A weakened `.is_some()`/`cols≥1` assertion → the mutant survives → MSI 12/13. (cell==0 with `>=` divides-by-zero → inf → 65535, still Some+≥1.) | REAL (guardrail) | Phase-4 zero-cell test asserts exact `Some((1,15))` — baked into the test. |
  | A2 | INFO | The high-range fixture does NOT kill `/→*` (600000/8 and 600000*8 both saturate to 65535); the NORMAL fixture (800/8=100 vs 6400) is the load-bearing `/→*` killer. | REAL (test-design) | REQ-001 uses the normal 800×240 fixture (already planned). |
  | A3 | — | MSI 100 VERIFIED: 13/13 mutants caught, NO equivalent/unkillable mutant; the "only `cell>0.0` guard" design claim holds (saturating cast + f32::max-ignores-NaN handle the rest — no inert code). | CONFIRMED | none |
  | B8 | MED | `pty_size` was advanced even when `session.resize()` FAILED (`let _ = resize(); pty_size = new`) → the guard's shadow records the target while the PTY stays old-sized, eliding all retries → pane silently stuck mis-sized (contradicts the "last APPLIED" docstring). | REAL | Advance the shadow only on success: `if state.session.resize(c,r).is_ok() { state.pty_size = (c,r) }`. A persistently-failing (dead) pane now re-attempts 1 cheap ioctl/frame instead of lying. |
  | B3 | MED (follow-up) | The cell metric reads the AMBIENT (proportional) window font — no monospace font is set in marley_app, and block_row renders plain Strings in it — so `cols` won't match the drawn glyphs. Resize plumbing is valid; only metric fidelity is off. Harmless today; a real gap at TUI time. | REAL (deferred) | Captured as intake `terminal-monospace-font-and-cell-metric.md` + noted in the spec Out; lands with the terminal-font work (#31/#33/M1.E). |
  | B1 | — | Resize-spam (headline concern): the `floor()`→integer-grid guard elides steady-state frames (f32 jitter absorbed by floor; metric deterministic for a fixed font). | VERIFIED-SAFE | none |
  | B-misc | LOW | render-side-effect ioctl (gated change-only), O(N²) find (single-digit N), (0,0) sentinel unreachable as real grid — all acceptable. | ACCEPTED | none |
- **Fixes verified:** the B8 one-liner compiles + clippy clean; the A1/A2 constraints are baked into
  the Phase-4 test plan. Lesson: advance a cached/shadow of an external state ONLY when the
  operation that mutates that state SUCCEEDED (`PR-claude-advance-shadow-only-on-success`).

## Phase 4 — Validate
- **Tests added (workspace.rs, 3):** `plan_resize_floor_divides_each_axis` (non-square 800×240 /
  8×16 → (100,15); floor via 805→100), `grid_axis_clamps_below_one_and_guards_bad_cell` (sub-cell
  →(1,15); zero-cell → EXACT `Some((1,15))` per Critic A's load-bearing constraint that kills
  `>`→`>=`; NaN extent →(1,15)), `plan_resize_none_when_unchanged` (equal→None, differ→Some).
- **Runs (actual):** `cargo nextest run -p marley` → the 3 new PASS (81 in marley); full workspace
  green; doctests green.
- **Visual (gate:15):** PASS (the resize is a PTY-side effect; no new render surface — the pane
  divs are unchanged).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15 first try**, 0 SLOW hangs; coverage
  100%; mutation **13 caught / 0 missed → MSI 100.0%** (exactly the 13 mutants Critic A enumerated
  — no equivalent/unkillable mutant). Receipt written.
- **Pre-existing failures:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (done at implement); `app_shell.md` gains a workspace.rs
  sizing bullet (plan_resize + the shim + the advance-on-success). SPEC-app-shell R37 at implement.
- **Knowledge captured:** `BF-claude-pty-size-shadow-advanced-on-failed-resize-elides-retries-001`
  (inspect #8) + `PR-claude-advance-shadow-only-on-success-001` (advance a cached shadow of an
  external state ONLY on the mutating op's success) + intake
  `terminal-monospace-font-and-cell-metric.md` (inspect #3 follow-up). aar-submit `completed`.
  Wins: the "minimal guard" design (only `cell > 0.0`; let the saturating cast + f32-max-NaN do the
  rest) gave a clean 13/13 MSI with ZERO inert code — the arithmetic critic PRE-RAN cargo-mutants
  and proved 13/13 + the exact load-bearing zero-cell fixture BEFORE Phase 4, so the tests landed
  green first try.
- **Ticket:** forge #30 → done; local doc → closed/; pipeline pair archived. 3 of 6 in M1.D.
