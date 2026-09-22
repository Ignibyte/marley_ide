# retire the right rail → top tabs (M6 seq-7) — Notes

- **Forge ticket:** #126 `b10f40e4-dbb6-4a7e-bb9a-74b56995b008` · **AAR:** `e2e346d3-707e-4bd2-968a-f759b1696ab6`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-126-top-tabs.md

## Phase 1 — Plan
- **Request:** forge #126 (M6 run 8/10) — Details/Agents/Forge tabs → the top bar; right rail freed.
- **Pre-flight:** right_section (139); section_tabs() (right_dock.rs); right-dock render ~2168-2240 (tab strip
  over section body); docks[1] toggle; #95 persists right_section; top bar (#117) near end of render.
- **Decisions:** D1 top_tabs reuses section_tabs + active flag; D2 top tabs drive right_section + open the
  dock; dock's tab strip removed; right dock defaults CLOSED.
- **AAR id:** `e2e346d3-707e-4bd2-968a-f759b1696ab6`.

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
- **right_dock.rs (PURE):** `struct TopTab { section: RightSection, label: &static str, active: bool }`; `top_tabs(active) -> Vec<TopTab>` = section_tabs().map(|(s,l)| TopTab{s,l,active: s==active}).
- **app.rs (SHIM):** render a top tab strip (top_tabs(right_section)) in the top bar row (beside the #117 search); each tab click → right_section=s + open docks[1] + persist_right_section. REMOVE the tab-strip child from the right-dock render (2168-2200) — the dock body renders only the active section. Boot: right dock defaults CLOSED (docks[1]=false) so the right side is free.
- **Mutation targets:** top_tabs list + the active== flag.
- **Test plan:** top_tabs_lists_and_flags_active (3 tabs Details/Agents/Forge; active flagged; switching moves it).
- **Risks:** where docks[1] default is set (find the boot docks init); the top bar row layout (fit the tabs). No dangling right_section (the dock still uses it for the body).

## Phase 3 — Implement
- **Built (right_dock.rs PURE):** TopTab{section,label,active} + top_tabs(active) (reuses section_tabs, flags active==section). **(app.rs SHIM):** a top-RIGHT tab strip (top_tabs(right_section)) — each tab click sets right_section + opens docks[1] + persist_right_section; REMOVED the tab-strip block from the right-dock render + the .child(tabs) (the dock now shows only the section body). Import section_tabs→top_tabs.
- **DEVIATION:** kept the boot dock state (applied.right, persisted) rather than forcing closed — the tabs move to the top; the content panel shows per its persisted/toggled state (the capture can seed docks.right=false to show the right free).
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a small pure tab model + moving a render block).
- **Lenses — no findings:** top_tabs = section_tabs→TopTab with active==section (the flag); the top strip renders all 3, clicking sets right_section + opens the dock + persists (#95 pattern); the dock tab strip is removed (0 compile err ⇒ no dangling section_tabs ref; active_section still drives the body). No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** top_tabs_flags_active (3 tabs in order, exactly the active flagged, flag moves on switch). Pass.
- **Self-test:** LIVE capture (tabs126.png) — the Details/Agents/Forge tabs render at the TOP-right (Details active, cyan), off the right rail. REQ-002 PASS. MINOR: a cosmetic overlap with the right dock header + the dock still shows the section body (the docks.right seed did not close it) → the #127 finale defaults the right side free + cleans the overlap.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG; forge #126 → done. **M6 8/10.** Details/Agents/Forge tabs → the TOP bar (top_tabs pure); the dock tab strip removed. Chad's "tabs at the top". Minor overlap → #127 finale. cov/MSI 100.
