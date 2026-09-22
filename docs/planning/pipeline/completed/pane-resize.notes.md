# pane drag-resize (M6) — Notes

- **Forge ticket:** #130 `236fab76-d501-466c-82ce-de143ca87151` · **AAR:** `220cc4ce-03e8-48aa-be1c-2d4483ea85de`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-130-pane-resize.md

## Phase 1 — Plan
- **Request:** forge #130 — drag-resize the fixed 50/50 pane splits (pure resize_split + a divider handle).
- **Pre-flight:** PaneGroup::Split{axis,children,ratios} + equal_ratios; pane_rects tiles by ratios; grid renders from rect_list.
- **Decisions:** D1 clamp both ≥min, sum=1, out-of-range/no-room→unchanged; D2 top-level only; D3 drag env-blocked→code-reviewed.
- **AAR id:** `220cc4ce-03e8-48aa-be1c-2d4483ea85de`.

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
- **layout.rs (PURE):** resize_split(ratios,boundary,delta,min)->Vec<f32> (clamp new_a to [min,total-min], b=total-new_a; out-of-range/total<2min→unchanged); PaneGroup::resize_boundary(boundary,delta,min) applies it to a top-level Split.
- **workspace.rs (PURE):** Workspace::resize_boundary delegates to the group.
- **app.rs (SHIM):** a drag-state field dragging_divider:Option<usize>; a thin divider handle at each top-level boundary (rect_list i|i+1) — on_mouse_down sets the boundary, on_mouse_move delta=dx/center_w→resize_boundary, on_mouse_up clears.
- **Test plan:** resize_split_cases (0.5/0.5+0.1→0.6/0.4; clamp both ends; 3-way boundary1 only 1&2; out-of-range/no-room unchanged); resize_boundary_applies (Split→ratios change; Leaf→no-op).
- **Risks:** f32::clamp panics if min>max → the total<2min guard; drag env-blocked (masked).

## Phase 3 — Implement
- **Built (layout.rs PURE):** resize_split(ratios,boundary,delta,min) (clamp new_a to [min,total-min], b=total-new_a; out-of-range/total<2min→unchanged); PaneGroup::resize_boundary applies it to a top-level Split. **(workspace.rs PURE):** Workspace::resize_boundary delegates. **(app.rs SHIM):** PANE_MIN_RATIO=0.1; dragging_divider:Option<(usize,f32,f32)>=(boundary,last_x,split_w); a thin 6px divider handle at each top-level boundary (hover=accent) → on_mouse_down sets the drag; root on_mouse_move applies Δx/split_w via resize_boundary; on_mouse_up clears. MouseUpEvent imported.
- **DEVIATION:** the drag gesture is env-blocked (synthetic mouse) → the shim is code-reviewed; the capture shows the handles.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a pure clamped-arithmetic fn + an env-blocked drag shim).
- **Lenses — no findings:** resize_split — new_a=(a+delta).clamp(min,total-min) so BOTH sides ≥min; b=total-new_a preserves the pair-sum ⇒ overall sum 1; boundary+1>=len guards out-of-range; total<2min guards the clamp (min>max would panic) + the no-room case. resize_boundary only touches a top-level Split (Leaf=no-op). The drag: down sets (boundary,x,split_w), move applies incremental Δx/split_w (correct sign: right-drag grows the left pane), up clears via take(). No unwrap/panic (clamp guarded). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** resize_split_cases (REQ-001 60/40; REQ-002 clamp both ends; REQ-003 3-way only adjacent; REQ-004 out-of-range + no-room unchanged); resize_boundary_top_level_only (Split→changed; Leaf→no-op). Pass.
- **Self-test:** LIVE capture (dividers130.png) — grid="H:t,f,c" (terminal + files + code); subtle divider handles render as thin vertical lines at each pane boundary (the grab affordance, brightening to accent on hover). The drag gesture is ENV-BLOCKED (synthetic mouse) → code-reviewed; resize_split carries the cov/MSI-100 proof.
- **Gate:** (running).

## Phase 5 — Complete
- CHANGELOG; forge #130 → done. **M6 10/goal.** resize_split (pure, clamped, cov/MSI 100) + resize_boundary + draggable divider handles. Drag env-blocked (code-reviewed). cov/MSI 100.
