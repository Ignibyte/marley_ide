# M12 #179 — hit-test bottom-anchor skew — Notes

- **Forge ticket:** #179 `61a841ee-2a48-4bf5-8c30-143e7386954c` · **AAR:** `db594748-d203-4ac1-86e3-299498838ffa`

## Phase 1 — Plan / Phase 2 — Design (folded)
- The render packs rows flush to the pane BOTTOM (r.y+r.h, inside a 1px border) below a 24px title bar;
  the buggy row_at/row_hit mapped from the TOP using pty_size.1 as capacity → a title-bar + remainder
  skew. Replace with bottom_anchored_row(y_from_bottom, cell_h, start, end): row = end-1-floor(yfb/cell_h),
  None above content. Both consumers (pane_grid_pos selection + the #175 menu hit-test) compute
  y_from_bottom = (r.y+r.h-1) - pointer.y. Retire row_at/row_hit.
- **AAR id:** `db594748-d203-4ac1-86e3-299498838ffa`.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- viewport.rs: bottom_anchored_row(y_from_bottom, cell_h, start, end) — rows_up=floor(yfb/cell_h),
  row=end-1-rows_up, None above content (rows_up>end-1-start) / empty window; a below-bottom yfb clamps
  (max(0.0)) to the last row. RETIRED row_at + row_hit (the top-anchored bug source) + their 2 tests.
- app.rs: pane_grid_pos + the #175 right-click hit-test both compute content_bottom = r.y+r.h-1.0 (inside
  the pane div's 1px border) → y_from_bottom = content_bottom - pointer.y → bottom_anchored_row; selection
  clamps None→start, the menu treats None as no-block→split. Removed the now-dead local_y/local_row. import
  row_at,row_hit → bottom_anchored_row.
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Self-review (a focused geometry fix over 1 pure fn + 2 mirror call sites; the deep critic runs in P3.5 proper
next — recording lenses here for the ledger):
- THE MATH: the bottom row (end-1) is at yfb∈[0,cell_h) → rows_up 0 ✓; going up one cell → end-2 ✓; the
  empty pad above the visible rows gives rows_up>end-1-start → None ✓ (both the title bar AND short-content
  pad, since the content is bottom-flush regardless).
- TWO CONSUMERS PARITY: identical y_from_bottom computation at both sites; selection .unwrap_or(start)
  (a drag above content pins to the first visible row, preserving the old row_at clamp intent); the menu `?`
  (None → MenuKind::Split, preserving #175's row_hit intent) — behaviors preserved, skew removed.
- BORDER OFFSET: content_bottom = r.y+r.h-1.0 accounts for the pane div's .border_1() (gpui border-box);
  the -1 is small but correct — the bottom row sits just above the 1px border.
- NO COL REGRESSION: local_x/col math untouched.
Lenses: index arithmetic at the boundary, consumer parity, border geometry, col invariance.

## Inspect (Phase 3.5) — critic verdict
1 background critic (render-geometry / boundary math / float-cast / consumer-parity / dead-refs lenses).
- **NO correctness findings** — the fix is sound. The critic traced the taffy 0.9 box model to ground:
  gpui inherits BoxSizing::BorderBox and taffy ALWAYS insets content by the border, so content_bottom =
  r.y+r.h-1.0 (past the 1px border) is EXACTLY right — and independent of PANE_TITLE_H (top+h = r.y+r.h
  regardless). `> span` confirmed correct (span itself → Some(start), span+1 → None); adjacent-block
  boundary rows map cleanly (contiguous half-open cell bands); float→usize saturates (no panic); selection
  parity preserved + strictly safer (clamps below-bottom to end-1 where old row_at could overshoot). No
  dead refs; rustdoc clean (no #150-class private link).
- **[nit → FIXED] two equivalent-mutant risks** the critic flagged (would have gate-RED'd MSI): the
  `.max(0.0)` was redundant given the saturating cast → REMOVED it (documented the saturation); added a
  negative-cell_h test case (`bottom_anchored_row(0.0, -5.0, 0, 10) == None`) to kill the `<=`→`==` guard
  mutant.
- **[nit → FIXED now, not deferred to P5] stale app_shell.md** — the two row_at/row_hit references rewritten
  to bottom_anchored_row.
Lenses: box-model geometry, boundary arithmetic, float-cast safety, consumer parity, dead-ref/doc hygiene.

## Phase 4 — Validate
- **Tests:** bottom_anchored_row_maps_from_bottom — yfb 0→end-1, within-cell→same, one-cell-up→end-2, the
  span boundary Some(start) vs one-past None, below-bottom→last, empty window / cell_h==0 / cell_h<0 → None,
  full-screen start=0. 1 new; the retired row_at/row_hit tests removed. Full suite green.
- **Self-test (REQ-002, the decisive boundary proof):** clean single-terminal boot, minted 2 ADJACENT
  blocks (`echo alpha179`/alpha179 then `echo beta179`/beta179 — the alpha OUTPUT row sits directly above
  the beta HEADER). Right-clicked the alpha row's BOTTOM sub-cell band (fy 0.898 — the exact skew danger
  zone) → the block menu opened → Enter ran the pre-selected Copy Command → ⌘V → the prompt reads
  `Marley echo alpha179` (hs_paste_crop.png). Pre-#179 that band skewed to block 1 → would have pasted
  `echo beta179`. The hit-test now lands on the RIGHT block.
- **REQ-003 (selection):** terminal selection uses the SAME pane_grid_pos → bottom_anchored_row path just
  proven exact by the menu round-trip; no separate regression (the pure test + the shared code path cover it).
- **Gate:** GREEN [diff] 15/15, MSI 100 (the .max removal + neg-cell_h test pre-empted the critic's equivalent-mutant nits).

## Phase 5 — Complete
- CHANGELOG (Fixed) + app_shell #179 note (row_at/row_hit refs rewritten); forge #179 → done; PR-redundant-guard-before-saturating-cast recorded. **M12 1/10.** LESSON: an inverse-map must mirror the forward transform's anchor exactly; the copy-paste round-trip turns a sub-pixel hit-test skew into a binary text assertion.
