# M10 — tab close — Notes

- **Forge ticket:** #161 `7056f380-b37e-4cbc-b476-c66c7cc894ae` · **AAR:** `8ad2463e-a192-428f-b143-9af0a1e0460d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-161-tab-close.md · absorbs the #159 guard

## Phase 1 — Plan
- **Request:** forge #161 (M10 run 1/10) — the rail × + ⌘W tab semantics + the LastTerminal guard (#159).
- **Pre-flight:** close_tab exists+tested; TabError lacks LastTerminal; Workspace lacks project_mut(idx); the
  "close-pane" arm @app.rs:2127 (close_focused + thread-drop reap @2135); the rail Tab arm ~2604.
- **Decisions:** D1 guard precedence (LastTab for single; LastTerminal for multi-tab-only-terminal); D2 Warp ⌘W
  (panes first, last pane → tab; cockpit/code → tab); D3 thread-drop reap; D4 refusal flash.
- **AAR id:** `8ad2463e-a192-428f-b143-9af0a1e0460d`.

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
### Approach
- **tabs.rs (PURE, cov/MSI 100):**
  - `TabError::LastTerminal`.
  - `close_tab`: after the IndexOutOfRange + LastTab checks, `if tabs[idx].grid().is_some() &&
    tabs.iter().filter(|t| t.grid().is_some()).count() == 1 → Err(LastTerminal)` (before the remove).
  - `Workspace::project_mut(idx) -> Option<&mut Project<S>>` (`projects.get_mut`).
- **app.rs (SHIM masked):**
  - `close_tab_at(project, tab)`: project_mut → close_tab → Ok(removed): thread-drop `removed` (the dead-pane
    reap pattern — PTY teardown off the UI thread) + persist_grid; Err(LastTerminal|LastTab): a status flash
    "can't close the last terminal"; other Err: no-op.
  - The rail Tab row becomes flex_row: the label (flex_1, keeps the switch click) + an × div whose handler
    calls `cx.stop_propagation()` (already used at app.rs:1822/2990) THEN close_tab_at — so the row's switch
    never fires on an × click.
  - The "close-pane" arm branches: active tab a multi-pane terminal → the EXISTING close_focused path
    (unchanged, incl. its agents/remotes cleanup); else (1-pane terminal OR cockpit/code) →
    close_tab_at(active_project_index, active_tab_index).
- **D5 (deferred cleanup):** agents/remotes map entries for a CLOSED TAB's panes are NOT removed here —
  PaneIds alias across tabs (#158), so removing by id could clobber another tab's entries. The per-tab map
  rework (#167, absorbing #158) makes that cleanup safe + trivial; a stale Fleet row until then is the known
  #158-class limitation.

### File manifest
- `crates/marley_app/src/tabs.rs` — LastTerminal + the close_tab guard + project_mut + tests.
- `crates/marley_app/src/app.rs` — close_tab_at; the rail × ; the ⌘W branch.

### Regression Test Plan
| Test (tabs.rs) | AC |
|---|---|
| close_last_terminal_refused — [T,C]: close T → Err(LastTerminal), tabs unchanged; close C → Ok | REQ-001 |
| close_terminal_among_two_ok — [T,T]: close 0 → Ok, active follows; [T]: close → Err(LastTab) | REQ-002 |
| project_mut_cases — Some(0), None(past) | REQ-003 |
| DRIVEN: + a tab → × the first → the rail shrinks; × the last terminal → refusal flash | REQ-004 |
| DRIVEN: ⌘W at 1 pane → the TAB closes; at 2 panes → the PANE closes | REQ-005 |

### Risks
- Guard ORDER: LastTab (len==1) must stay first so a single-tab project keeps its existing error; the terminal
  count uses the PRE-remove list.
- × bubbling: stop_propagation is load-bearing — without it the row switch fires after the close with a stale
  index (safe via switch_tab's bounds check, but wrong UX). The critic checks it.
- ⌘W on a cockpit/code tab previously closed a pane in the FALLBACK grid (#153's hidden-grid note) — the new
  branch makes it close the visible tab instead: an intentional, documented behavior improvement.

## Phase 3 — Implement
- **tabs.rs PURE:** TabError::LastTerminal; close_tab guard (terminal + only-terminal → LastTerminal, AFTER IndexOutOfRange/LastTab so precedence holds); Workspace::project_mut(idx)->Option.
- **app.rs SHIM (masked):** close_tab_at(project,tab) — project_mut → close_tab → Ok: thread-drop the removed tab + persist_grid; Err(LastTerminal|LastTab): "cant close the last terminal" flash. The rail Tab row → flex_row (label flex_1 keeps the switch click) + an × child (stop_propagation THEN close_tab_at). The "close-pane" arm branches: multi-pane terminal → the EXISTING close_focused path (agents/remotes cleanup intact); else → close_tab_at(active). D5: no agents/remotes cleanup on tab close (PaneId aliasing — deferred to #167/#158).
- **Verify:** fmt; check --all-targets 0 err; clippy -D warnings OK.

## Phase 4 — Validate
- **Tests:** close_last_terminal_guard (REQ-001/002: [T,C] close T → LastTerminal + unchanged, close C → Ok; [T,T] → Ok; [T] → LastTab precedence); project_mut_cases (REQ-003). 22 tab tests pass.
- **Self-test:** tc_x.png — + a 2nd tab, × terminal 1 → the rail shows only terminal 2 (with its ×), the tab closed (REQ-004). tc_refuse.png — after + and ⌘W (the added tab closed → only terminal 2 remains, REQ-005) a final ⌘W flashes "cant close the last terminal" (the guard, REQ-004).
- **Gate:** GREEN [diff] 15/15 (first run GREEN; re-gated GREEN after the 2 critic fixes — last_agent clear + pane-close persist).

## Inspect (Phase 3.5)
Method: 1 background critic (guard + × routing + ⌘W + reap) + my review + the driven captures + the gate.

- **Critic — core verified clean on all 5 checks:** guard order exact ([T]→LastTab, [T,C] close-T→LastTerminal
  BEFORE any mutation, close-C→Ok, [T,T]→Ok; one production caller, nothing bypasses; workspace()'s expects
  unreachable via close). The × bubbling confirmed against gpui 0.2.2 internals (child fires first, stop_propagation
  suppresses the row switch — the app.rs:1843 idiom). The ⌘W branch FIXES the #153 hidden-pane surprise (the
  >1-pane path only runs when the active tab IS a terminal, so the fallback grid is unreachable from ⌘W).
- **[medium → FIXED] stale `last_agent` after a terminal-tab close.** PaneIds alias across tabs (#158), so a
  closed tab's agent left `last_agent` routable INTO a same-numbered pane of another tab (⌘⇧S/broadcast inject
  + Enter — worse than D5's "stale Fleet row" framing). Fix: close_tab_at clears `last_agent` when the removed
  tab had a grid (conservative; no clobber hazard). The full map scoping stays #167/#158; D5 note expanded here.
- **[low → FIXED] the multi-pane ⌘W path never persisted** — a closed PANE resurrected on restart while a closed
  TAB stayed closed. Added persist_grid after the pane-close reap (matches persist_grid's own doc).
- **[info — accepted] a rapid double-click on × acts on a frame-stale (p,t)** — bounded by the close_tab guards
  (IndexOutOfRange/LastTerminal); listeners re-register per paint. No action.

Lenses: guard precedence + invariant, bubbling/routing, ⌘W branch order, reap/persist state, map hygiene.

## Phase 5 — Complete
- (Archive deferred two commits — the changelog-hook veto killed the combined bash; recovered with #169.) CHANGELOG + app_shell #161 notes shipped in 25a8471.
