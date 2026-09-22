# M10 — the split context menu — Notes

- **Forge ticket:** #166 `d783dfa6-f858-420b-975e-104819cc3a44` · **AAR:** `a1f30333-325c-43e9-a52d-14b912674ecd`

## Phase 1 — Plan
- Pure ContextMenuState (mirror PaletteState) + menu_origin clamp; shim overlay + routing; the #155
  right-click repurposes to OPEN (Split Right keeps the old direct behavior one click deeper).
- **AAR id:** `a1f30333-325c-43e9-a52d-14b912674ecd`.

## Phase 2 — Design (folded)
- context_menu.rs: MenuAction (Copy/Eq), MENU_ITEMS 3×(action,label), ContextMenuState{x,y,selected} with
  wrap nav; menu_origin = (x.min(ww-mw).max(0), y.min(wh-mh).max(0)).
- app.rs: the field; right-click → focus + `Some(ContextMenuState::new(event.position.x, .y))`; render LAST
  (above overlays): a full-window transparent backdrop (click → dismiss) then the menu div at the clamped
  origin (w 150, row h 26; selected bg surface); row click stop_propagation → act → dismiss → notify. Keys:
  an early on_key_down branch while open — esc dismiss / up / down / enter act+dismiss / others swallowed
  (return). split_focused_pane(axis: PaneAxis); "close-pane" via dispatch_action.
- Menu metrics consts: MENU_W 150.0, MENU_ROW_H 26.0.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- context_menu.rs (NEW pure): MenuAction, MENU_ITEMS, ContextMenuState (wrap nav, action), menu_origin clamp + tests.
- app.rs: context_menu field (+init+import); the right-click now OPENS the menu at e.position (focus first); split_focused_pane(axis); run_context_menu_action (SplitRight/Down → axis split; ClosePane → dispatch "close-pane" = the #161 semantics); an early on_key_down branch (esc/up/down/enter, others swallowed); the render — LAST: a full-window backdrop (click-away dismiss) + the menu at menu_origin (MENU_W 150, MENU_ROW_H 26; selected highlighted; row click stop_propagation → act).
- lib.rs: mod context_menu (alphabetical).
- fmt; check 0 err; clippy OK.

## Phase 4 — Validate
- **Tests:** selection_wraps_and_maps (REQ-001) + menu_origin_clamps (REQ-002) in context_menu.rs. Pass.
- **Self-test:** cm_menu.png — the menu AT the right-click pointer (Split Right selected/highlighted, Split Down, Close Pane); cm_splitdown.png — clicking Split Down → two STACKED panes (the NEW vertical axis) + pane 1/pane 2 nested in the rail (REQ-003); cm_esc.png — a reopened menu Esc-dismissed, no menu remains (REQ-004). ClosePane inherits the #161-tested ⌘W path.
- **Gate:** first GREEN; re-gated GREEN after the critic fixes (occlude + border height).

## Inspect (Phase 3.5)
Method: 1 background critic (coords, overlay routing, key swallowing, the axis plumbing, the pure state) + the
driven captures.

- **Critic — verified clean on coords (e.position IS window-relative; the root is at the window origin — the
  menu lands at the pointer), row routing (rows paint after the backdrop → hit-test first; stop_propagation
  breaks the whole dispatch — no double-fire), key swallowing (the branch heads the app's ONLY key path; no
  IME leak; no app-menu ⌘Q to regress), the #155 change (split_focused_pane's only callers are the two menu
  arms; ⌘D/"+"/palette use new_terminal_pane — untouched), and the pure state (wrap math mutation-resistant).**
- **[medium → FIXED] the backdrop click FELL THROUGH** — gpui keeps dispatching past a non-occluding hitbox, so
  a dismissing click also started a terminal text selection (and wheel scrolled beneath). The codebase's own
  convention: 12 modal overlays `.occlude()`. Fix: `.occlude()` on the backdrop (rows still clickable —
  painted after). Also changes right-click-while-open over the backdrop to dismiss-only: correct modal intent.
- **[nit → FIXED] the clamp height ignored the 1px borders** — pass `rows*26 + 2.0` so the bottom edge never
  overhangs by 2px.
- **[nit — accepted] modifier-blind menu keys** (⌘-Enter drives the menu) — harmless in a 3-row modal.

Lenses: coordinate space, hit-test/propagation order, modal key routing, caller-graph of the axis change, pure
wrap/clamp. Re-gated after the fixes.

## Phase 5 — Complete
- CHANGELOG + app_shell #166 note; forge #166 → done. **M10 6/10.** LESSON: a modal backdrop MUST .occlude() in gpui (the same mouse-down keeps dispatching past a non-occluding hitbox); when adding overlay N+1, start from the nearest existing overlay, not from memory.
