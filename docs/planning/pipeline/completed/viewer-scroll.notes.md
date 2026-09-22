# scroll + jump-to-line in the viewer — Notes

- **Forge ticket:** #101 `1be96239-3b41-441e-9838-bb8992a876f1` · **AAR:** `93c9d996-fe0d-4dce-a66d-635cd1c96fde`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-101-viewer-scroll.md

## Phase 1 — Plan
- **Request:** forge #101 (M4 5/10) — scroll the viewer + jump_to a line.
- **Pre-flight:** handle_code_view_key (#97) is Esc-only — extend it; the #97 render uses `.take(40)` —
  swap to a visible_range slice; on_scroll_wheel exists (app.rs ~1979) to mirror for the wheel.
- **Decisions:** D1 clamp to total-height; D2 escape before as_mut.
- **AAR id:** `93c9d996-fe0d-4dce-a66d-635cd1c96fde`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **code_view.rs (PURE):** `visible_range(scroll,height,total) -> (usize,usize)` = `let max_start = total.saturating_sub(height); let start = scroll.min(max_start); let end = (start+height).min(total); (start,end)`; `jump_to(line,height,total) -> usize` = `line.min(total.saturating_sub(height))`.
- **app.rs SHIM:** `const VIEWER_ROWS: usize = 40`; handle_code_view_key: escape (as now) returns FIRST; then `let Some(cv)=self.code_view.as_mut() else {return}; let max=cv.lines.len().saturating_sub(VIEWER_ROWS); let cmd=keystroke.modifiers.platform;` match key: up→`cv.scroll = if cmd {0} else {cv.scroll.saturating_sub(1)}`; down→`cv.scroll = if cmd {max} else {(cv.scroll+1).min(max)}`; _→{} ; notify. The overlay render: `let (start,end)=visible_range(cv.scroll,VIEWER_ROWS,cv.lines.len()); for line in &cv.lines[start..end] {…}` (replaces .iter().take(40)); an on_scroll_wheel on the overlay div nudges cv.scroll by ±3 clamped (mirror app.rs ~1979) if clean.
- **Mutation targets:** visible_range saturating_sub + both .min + start+height; jump_to .min + saturating_sub.
- **Test plan:** visible_range_cases (0/mid/past-end/total<height/total0) + jump_to_cases (mid/first/last-clamp/tiny). cov/MSI 100. The keys + wheel + windowed render masked (engine).
- **Risks:** escape handled before as_mut (borrow); scroll clamped in the handler AND visible_range (double-safe); VIEWER_ROWS is the render height contract.

## Phase 3 — Implement
- **Built:** visible_range + jump_to (code_view.rs); app.rs `VIEWER_ROWS=40` + handle_code_view_key scroll keys (↑/↓ step, ⌘↑/⌘↓ = jump_to top/bottom — uses jump_to so it is not dead) + the windowed render (cv.lines[start..end] via visible_range).
- **DEVIATION:** skipped the wheel (design said "if clean") — keyboard scroll suffices; the ScrollWheelEvent-in-overlay wiring is a follow-up if wanted.
- **Verification:** fmt; check 0 err; clippy OK (jump_to consumed by ⌘↑/⌘↓); both scroll tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (two small clamped-arithmetic pure fns + a masked scroll handler/window; gate cargo-mutants is the MSI authority).
- **Lenses — no findings:** visible_range (max_start=total-height saturating; start=scroll.min(max_start); end=(start+height).min(total) — tested 0/mid/past-end/total<height/empty); jump_to (line.min(total-height) — tested mid/first/last-clamp/tiny); the scroll handler bounds scroll (saturating_sub / .min(max)) + escape-before-as_mut (no borrow clash); the render slices [start..end] within bounds (visible_range end<=total). No index panic, no unwrap. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** visible_range_cases + jump_to_cases. `cargo nextest` → pass.
- **Self-test:** scroll needs synthetic keys (ENV-BLOCKED); engine-tested cov/MSI 100; the handler+window masked.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; forge #101 → done. **M4 5/10.** visible_range + jump_to (cov/MSI 100) + ↑/↓/⌘↑/⌘↓ scroll + windowed render. Wheel deferred.
