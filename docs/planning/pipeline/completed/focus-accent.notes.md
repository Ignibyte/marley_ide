# pane focus navigation + the Warp focus accent (M6) — Notes

- **Forge ticket:** #131 `b0ed44a3-66fd-4470-867d-43416ab2538e` · **AAR:** `48c84b14-4691-4508-8cae-90be84caad52`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-131-focus-accent.md

## Phase 1 — Plan
- **Request:** forge #131 (M6 run 9/10) — ⌘⌥-arrow focus nav + the Warp left+top focus accent.
- **Pre-flight:** PaneGroup::neighbor (layout.rs:262, tested, tree-based); Workspace::focus (416); keymap
  default_bindings + action_for; the pane border at app.rs ~2316.
- **Decisions:** D1 reuse the tree neighbor (no rect dependency in the key handler); D2 subtle border + a
  focused left+top accent edge.
- **AAR id:** `48c84b14-4691-4508-8cae-90be84caad52`.

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
- **workspace.rs (PURE):** focus_neighbor(dir)->bool = match group.neighbor(focused,dir){Some(id)=>{focus(id);true} None=>false}. Import Direction.
- **keymap.rs (PURE):** 4 bindings chord(true,false,false,true,"left"/"right"/"up"/"down")→"focus-left/right/up/down"; action_for entries + tests.
- **app.rs (SHIM):** dispatch_action 4 arms → focus_neighbor(Direction::Left/Right/Up/Down)+cx.notify; the pane frame border_2→border_1(border) always + on the focused pane a left+top accent edge (2 accent bars, or border_l/t). mutants::skip.
- **Test plan:** focus_neighbor_moves_or_edge ([A|B] Right→true+focused B; Left at A→false); keymap focus_arrows_bound (action_for the 4 chords).
- **Risks:** the focused field name; Direction import; the accent-bar overlay position vs the pane title bar.

## Phase 3 — Implement
- **Built (workspace.rs PURE):** focus_neighbor(dir)->bool (group.neighbor→focus+true / None→false); import Direction. **(keymap.rs PURE):** 4 ⌘⌥-arrow bindings → focus-left/right/up/down (action_for is data-driven, auto-wired). **(app.rs SHIM):** dispatch 4 focus-* arms → focus_neighbor(Direction::…); the pane frame is border_1(border) always + on the focused pane a 2px left + 2px top accent-bar overlay (the Warp edge) instead of the full accent border. Direction imported.
- **DEVIATION:** reused the tested tree neighbor (no geometric pane_in_direction) — no render-time rects needed in the key handler.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a neighbor wrapper + 4 data bindings + an overlay accent).
- **Lenses — no findings:** focus_neighbor — Some(id)→focus(id)+true / None→false (both arms); reuses the tested neighbor. The 4 ⌘⌥-arrow chords carry alt=true so they are DISTINCT from the ⌘-arrow block-jumps (up/down) — no collision. dispatch routes each to focus_neighbor(the matching Direction). The accent: border_1(border) frame + a focused-only left+top accent overlay (occlude-free, drawn after the pane). No unwrap/panic (focus_neighbor swallows the FocusError via let _). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** focus_neighbor_moves_or_edge (Right→true+focused B; edge Right/Up→false, focus unchanged); focus_arrow_chords_bound (the 4 ⌘⌥-arrow → focus-*). Pass.
- **Self-test:** LIVE capture (accent131.png) — grid="H:t,f"; the focused terminal pane shows a cyan accent edge on the LEFT + TOP only (not a full 4-side border); the unfocused FileTree pane has just the subtle frame. REQ-004 PASS. ⌘⌥-arrow nav code-reviewed (tested via the focus_neighbor + keymap units; a live focus shift is hard to capture statically).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG; forge #131 → done. **9/10 goal.** focus_neighbor (reuse tree neighbor) + 4 ⌘⌥-arrow bindings + the Warp left+top focus accent edge. cov/MSI 100.
