# M11 #175 — the block context menu — Notes

- **Forge ticket:** #175 `ad4f71a6-e985-4313-bb09-464811148316` · **AAR:** `e21aca98-eb76-453b-b3a8-8576129d75c8`

## Phase 1 — Plan / Phase 2 — Design (folded)
- The #172 spike found the 3 actions already shipped as header buttons (R50) — the ticket narrows to the
  right-click ROUTE + menu unification. block_at_row complements block_boundary_rows in nav.rs; MenuKind
  parametrizes the #166 state; the shim extracts copy_block_text/rerun_block for BOTH routes (no dup).
  The click row = pane_grid_pos row + the viewport start (the exact selection math).

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- nav.rs: block_at_row (header+outputs span; prompt/past-end → None) beside block_boundary_rows.
- context_menu.rs: MenuAction += CopyCommand/CopyOutput/Rerun; MenuKind{Split, Block{block}};
  BLOCK_MENU_ITEMS (6 rows: the block actions then the split three); items_for; the state carries kind
  (with_kind; the dead new() removed per the #152 lesson — its test migrated); wrap/action over items().
- app.rs: the right-click hit-tests the click row (pane_grid_pos — the exact selection math, viewport-
  adjusted) → block_at_row → a Block-kind menu, else Split; run_context_menu_action(action, cx) reads the
  kind pre-clear + the 3 new arms; copy_block_text/rerun_block extracted (the R50 header buttons now call
  them — dedupe, zero behavior change); the render sizes/rows from menu.items().
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: 1 background critic (row-math/alt-screen/lifecycle/height lenses; nextest 8/8 targeted + clippy).

- **Critic verified the core:** the lens walk aligns exactly with the render (header + outputs, prompt →
  None); the kind is read before the clear; menu replacement while open is impossible (the occluding
  backdrop); ⌘-chords are swallowed by the menu branch; heights/labels fit; a captured block index stays
  valid (BlockList is append-only).
- **[medium → FIXED] empty-band clicks opened the Block menu for block 0** — row_at CLAMPS to the first
  row by design (selection wants that); the menu hit-test now uses the NEW pure `row_hit` (None in the
  band above bottom-anchored content AND past the end) → the split menu there. Pixel-proven (bm_empty).
- **[medium → FIXED] alt-screen panes served a stale Block menu** — vim/less show the program grid while
  blocks() still holds invisible cooked history; the hit-test now gates on !is_alt_screen → Split.
- **[low → FIXED] the pump could retarget a block action mid-menu** — a #173 auto-close can shift focus
  while the modal is open; the target now rides the KIND (MenuKind::Block { pane, block }) — bound at
  right-click, immune to focus drift (the #96 pane-binding pattern, second use).
- **[low → FOLLOW-UP #179] the bottom-anchor remainder skew in pane_grid_pos** — capacity floors from the
  FULL rect (title bar included) so the bottom `r.h mod cell_h` px of each row map one row down; INHERITED
  from the R47 selection math and shared with it — fixing it here would silently change selection behavior,
  so it gets its own ticket (filed below) rather than a rider fix.
- **[nit → FIXED] the backdrop now dismisses on right-click too.** **[nit → left]** nth(i) vs get() —
  relocated, not introduced.

Lenses: render-walk alignment, display-mode honesty, modal lifecycle, target binding, inherited-bug scoping.

## Phase 4 — Validate
- **Tests:** block_at_row_maps_and_misses (header/last-output/header-only-block/prompt→None/past→None/
  empty); block_menu_kind_items_and_wrap (3 vs 6, first-entry identity, the pane+block target rides the
  state, wrap over SIX, action at 2 = Rerun); row_hit_bands_and_bounds (the band edge ±1, past-end, the
  scrolled window, the degenerate empty). 3 new, all passing.
- **Self-test:** bm_blocks (2 typed blocks); bm_menu (right-click alpha → the 6-row menu: Copy Command/
  Copy Output/Rerun/Split Right/Split Down/Close Pane); bm_paste (Copy Command → ⌘V → the prompt reads
  `echo alpha-175` — the clipboard round-trip, REQ-003); bm_rerun (right-click beta → ↓↓ → Enter → a FRESH
  `✓ echo beta-175` block — the keyboard path, REQ-004); bm_empty (the empty band → the 3-row split menu —
  the row_hit fix live).
- **Gate:** GREEN [diff] 15/15, MSI 100.

## Phase 5 — Complete
- CHANGELOG + app_shell #175 note; forge #175 → done; backlog #179 filed (the remainder skew, both consumers named). **M11 4/8.** LESSONS: inventory existing affordances first; clamping inverse-maps need MISS siblings for target resolution; pane-binding is now a 2×-proven pattern.
