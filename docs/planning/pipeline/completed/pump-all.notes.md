# M11 #173 — pump every grid — Notes

- **Forge ticket:** #173 `48330513-d9a8-43d5-acc2-b6ca9faff3da` · **AAR:** `d87d0f69-4d6c-4a8e-b263-195b6c803a83`

## Phase 1 — Plan / Phase 2 — Design (folded)
- grids()/grids_mut() flat_map over projects→tabs→grid (private-field access inside tabs.rs); locate_pane
  built HERE for the dead-close (and #174 reuses it). The pump's per-grid loop collects (PaneId, dead)
  across all grids; the dead-close resolves the owning grid via locate_pane; refresh_agent_statuses
  find_maps across grids(). Render untouched.
- **AAR id:** `d87d0f69-4d6c-4a8e-b263-195b6c803a83`.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- tabs.rs: grids()/grids_mut() (projects→tabs→grid flat_map; cockpit/code skipped) + locate_pane (enumerated
  double find_map; unique ids ⇒ ≤1 match; #174 will reuse).
- app.rs: the pump's pane loop is now per-grid over grids_mut(); the dead-close resolves the OWNING grid via
  locate_pane→project_mut→tab_grid_mut (per-grid last-pane stays visibly dead; agents/remotes dropped;
  threaded reap); refresh_agent_statuses resolves each agent via grids().find_map(terminal).
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: 1 background critic (semantics/remote-rule/resize/borrows/perf — it also ran the CPU sampling +
full nextest 260/260 + clippy clean).

- **Critic verified the core:** gpui coalesces notify() (a pending set — one draw/frame, no repaint storm);
  the remote Connected→Disconnected flash stays once-only across grids; resize interplay SAFE (background
  panes process at their spawn/last size — alacritty's Term and the PTY winsize were built together; no
  zero-size path); borrows clean (the grids_mut loop ends before the dead loop; multi-dead-same-tick safe —
  indices don't shift on pane close); dead panes don't spin dirty (the Err arm never sets it).
- **[LOW → FIXED] the pump close lacked the last_agent clear** (parity with close_tab_at/close_project_at) —
  a background agent auto-close would have left ⌘⇧S silently no-oping. Added the targeted clear.
- **[LOW → FIXED] the pump close never persisted** (pre-existing) — a relaunch resurrected the dead pane.
  Added persist_grid() in the close branch.
- **[INFO → routed to #174] send-to-agent still resolves in the ACTIVE grid only** — #174 (which reuses
  locate_pane) will fix the send path with the jump.
- **Perf (critic-measured, isolated HOME):** idle 1 tab ≈ 1.7–2.0% CPU vs 5 tabs ≈ 1.9–2.2% — +0.2–0.3%
  absolute for 4 extra PTYs (the idle fast-path returns on the first WouldBlock). HONEST CAVEAT: the
  mid-burst retry budget (8×1ms per pane per tick, main-thread) now applies to every tab — N
  simultaneously-streaming panes can block a frame ~8N ms; pre-existing class, staggering deferred until
  measured hot.
- Test-set note applied: the (1,2) locate case is what kills the index swap ((0,0) is swap-symmetric) —
  the landed test has (0,0), (1,0), (1,2), None + the ordered grids() firsts.

Lenses: repaint semantics, remote lifecycle, PTY-size invariants, borrow shape, cleanup parity, perf.

## Phase 4 — Validate
- **Tests:** grids_and_locate_pane_across_projects (3 grids in project→tab order over a mixed 2-project
  shell; locate (0,0)/(1,0)/(1,2)/None — swap + contains + always-None mutants killed). 1 new + 260/260.
- **Self-test (the background-freeze un-freeze, REQ-002):** ⌘⇧A in the claude tab → ⌘T away IMMEDIATELY.
  pa_t0_rail2.png (t≈2s backgrounded): the rail glyph is ◔ Waiting (muted — claude still silent).
  pa_t18_rail2.png (t≈20s, NEVER revisited): ● Working in ACCENT — the transition happened entirely while
  backgrounded (pre-#173 the glyph froze at its last foreground state). pa_back_pane.png (⌘1 back): the
  FULL Claude Code welcome banner — all of it streamed + processed while background (the drain proof).
- **Gate:** GREEN [diff] 15/15, MSI 100.

## Phase 5 — Complete
- CHANGELOG (Fixed) + app_shell #173 note; forge #173 → done. **M11 2/8.** LESSONS: prove background liveness via a transition captured while hidden; close-path side-effect parity keeps re-breaking — a close_pane_cleanup helper is the structural fix when the next site appears; send-to-agent gap routed to #174.
