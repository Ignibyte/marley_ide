# M10 — rail + Files scroll — Notes

- **Forge ticket:** #169 `ab767d07-a74a-4c27-8bb2-b38456d4ed52` · **AAR:** `ec887cf5-7d59-40cf-8a74-920b28cabc36`

## Phase 1 — Plan
- Row-skip offsets clamped by the REUSED scroll_code (zero new pure surface); remainder-precise wheels;
  files_scroll resets on the sync re-walk. Proven needed by #168's capture.
- **AAR id:** `ec887cf5-7d59-40cf-8a74-920b28cabc36`.

## Phase 2 — Design (folded)
- Fields: rail_scroll/files_scroll: usize; rail_scroll_remainder/files_scroll_remainder: f32.
- Rail: the container div gains on_scroll_wheel (pixel→rows via the 22pt row estimate → scroll_steps →
  scroll_code(rail_scroll, -steps, content_h/22, rail_rows.len())); the row loop skips
  `min(rail_scroll, len-1)` rows.
- Files: the panel wrapper gains the same (20pt rows; total = visible_rows().len()); files_panel skips
  self.files_scroll (render-time re-clamped); sync_active_project sets files_scroll = 0.
- The height estimates only place the bottom stop; the render re-clamp guarantees non-blank.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- 4 fields (rail_scroll/files_scroll + remainders); the rail container + the Files wrapper gain on_scroll_wheel (pixel→rows at 22/20pt estimates → scroll_steps → scroll_code with viewport=window_h/row_h); the rail loop skips rail_skip (min-clamped); files_panel skips files_scroll via .enumerate().skip() — ORIGINAL indices preserved for toggle/path_at; sync_active_project resets files_scroll.
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Self-review (a reuse-heavy shim; scroll_code+scroll_steps already gate-proven):
- INDEX PRESERVATION is the trap: .enumerate().skip(n) keeps original indices (skip-after-enumerate) — toggle(index)/path_at(index) stay correct. Verified the order in code.
- The wheel regions are disjoint (the rail dock, the Files strip, the code-tab center, terminal panes) — gpui routes to the hovered element; each handler touches only its own offset.
- The viewport estimate (window_h/row_h) only positions the bottom stop — over-estimating trims the max offset short, never blanks (the render min-clamp guarantees ≥1 row).
- Shrink lifecycle: a closed tab/collapsed dir shrinks totals → the render min-clamp + the next wheel scroll_code re-clamp.
Lenses: index preservation, region disjointness, estimate error bounds, shrink lifecycle.

## Phase 4 — Validate
- **Tests:** none new — scroll_code (the clamp) + scroll_steps (the precision) are already cov/MSI 100; this ticket is pure reuse + shim wiring.
- **Self-test:** ls_files.png — 25 wheel-clicks over the Files tree revealed crates/editor/src, marley_agent, marley_app/assets/icons (all below the fold before) — the row-skip + clamp WORK (REQ-002). ls_rail.png — 13 ⌘T tabs all FIT the viewport; wheeling the rail no-opped via the clamp (total < viewport → max offset 0 — the correct no-overscroll behavior, REQ-001-clamp). The rail SKIP is the same code path as the proven Files skip; driving >55 tabs to overflow the rail is disproportionate — noted honestly.
- **Gate:** GREEN [diff] 15/15.

## Phase 5 — Complete
- CHANGELOG + app_shell #169 note; forge #169 → done. **M10 9/10 (8 of the goal 10).** LESSONS: enumerate-THEN-skip preserves click indices; archive/mv NEVER in the same bash as the commit (a hook veto kills all of it — recovered #161/#168 strays this run).
