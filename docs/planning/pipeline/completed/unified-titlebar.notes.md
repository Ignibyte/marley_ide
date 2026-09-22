# unified macOS title bar [M8] — Notes

- **Forge ticket:** #138 `6b735046-2bd1-4e0a-bc2b-09a73a34689f` · **AAR:** `f1af3314-c227-4502-adc2-f20d3e6465d3`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-138-unified-titlebar.md

## Phase 1 — Plan
- **Request:** forge #138 (M8 run 2/8) — merge the top bar into the macOS traffic-light row (transparent titlebar).
- **Pre-flight (gpui 0.2.2):** TitlebarOptions { title: Option<SharedString>, appears_transparent: bool, traffic_light_position: Option<Point<Pixels>> }; app entry app.rs:3646 sets title Some("Marley"); the top bar (#132) + icons (#137) at left:12/44/76.
- **Decisions:** D1 appears_transparent+traffic_light_position, title None; D2 inset left icons via topbar_icon_x.
- **AAR id:** `f1af3314-c227-4502-adc2-f20d3e6465d3`.

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
- **layout.rs (PURE):** topbar_icon_x(slot,inset,gap)->f32 = inset + slot·gap.
- **app.rs (SHIM):** import gpui point; TITLEBAR titlebar=Some(TitlebarOptions{title:None, appears_transparent:true, traffic_light_position:Some(point(px(19.),px(TOP_BAR_H/2.-6.5)))}); TRAFFIC_LIGHT_INSET(~90)+ICON_GAP(~30) consts; the left icons 📁/+/🧠 use left(px(topbar_icon_x(0/1/2,INSET,GAP))). Search+cockpit unchanged.
- **Test plan:** topbar_icon_x_cases ((0,90,28)=90;(1,..)=118;(2,..)=146).
- **Risks:** appears_transparent + window drag region (empty top-bar stays OS-draggable) — verify capture; traffic-light y-center in TOP_BAR_H(30).

## Phase 3 — Implement
- **Built (layout.rs PURE):** topbar_icon_x(slot,inset,gap)=inset+slot·gap. **(app.rs SHIM):** titlebar=Some(TitlebarOptions{title:None, appears_transparent:true, traffic_light_position:Some(point(19, TOP_BAR_H/2-6.5))}); TRAFFIC_LIGHT_INSET=92 + ICON_GAP=30; the left 📁/+/🧠 icons use left(px(topbar_icon_x(0/1/2,INSET,GAP))). Imports: gpui point; layout topbar_icon_x.
- **Verification:** fmt; check 0 err; clippy OK. (appears_transparent behavior verified at P4 capture.)

## Phase 3.5 — Inspect
- **Method:** self-review (a layout fn + a window-config change). Key risk: appears_transparent (proven at P4).
- **Lenses — no findings:** topbar_icon_x = inset + slot·gap (the + and ·). The titlebar: appears_transparent hides the system bar so the window content extends to the top (the top bar IS the titlebar row); traffic_light_position centers the lights in TOP_BAR_H; title None. The left icons inset past the lights (92px) so they do not collide. Search (centered) + cockpit (right) untouched. content_band/TOP_BAR_H unchanged → the pane grid still starts at content_top. No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** topbar_icon_x_steps_by_gap ((0,90,28)=90;(1)=118;(2)=146). Pass.
- **Self-test:** LIVE capture (below) — THE check: the unified titlebar (icons+search in the traffic-light row, one bar, lights unobscured, no "Marley" title).
- **Gate:** (running).
- **Result:** LIVE capture (titlebar138.png) — ONE bar at the top: traffic lights · 📁/+/🧠 icons · centered search · cockpit icons. No separate row, no "Marley" title, lights unobscured (icons inset 92px past them). REQ-002/003 PASS. **Gate GREEN [diff] 15/15, cov/MSI 100.**

## Phase 5 — Complete
- CHANGELOG; forge #138 → done. **M8 2/8.** Unified titlebar: appears_transparent + traffic_light_position → icons+search in the traffic-light row; pure topbar_icon_x insets the left icons. cov/MSI 100.
