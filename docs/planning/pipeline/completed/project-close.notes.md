# M10 — project close — Notes

- **Forge ticket:** #162 `a1b62026-cb96-4d82-8ebb-56b8de86600f` · **AAR:** `3af93b17-5174-4795-95b6-b91e1c26d0f7`

## Phase 1 — Plan
- Shim-only over the tested close_project; mirrors #161's × pattern (stop_propagation, thread-drop, flash).
- **AAR id:** `3af93b17-5174-4795-95b6-b91e1c26d0f7`.

## Phase 2 — Design
- `close_project_at(idx)` (mutants::skip): `self.shell.close_project(idx)` → Ok(removed): thread-drop the whole
  Project (its tabs' grids own the PTYs) + `sync_active_project()` + `persist_grid()`; Err(LastProject): flash
  "can't close the last project"; Err(_): no-op.
- The rail Project arm: flex_row [ label div(flex_1) keeps the switch on_mouse_down | an × div with
  stop_propagation → close_project_at(p) ] — byte-for-byte the #161 Tab-row structure.
- Test plan: no new pure surface; driven — × the only project → the refusal flash; the × renders on the row;
  (if a 2nd project is drivable) close it → subtree gone + resync. Risks: the reap drops a WHOLE project
  (N tabs × M PTYs) on one thread — same policy as a tab (teardown off-main); sync AFTER close so the
  survivor's root drives Files/branch.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- close_project_at(idx): close_project → Ok: last_agent=None (the #161 aliasing rule) + thread-drop the whole Project + sync_active_project + persist_grid; Err(LastProject): "cant close the last project" flash.
- The Project row → flex_row [label(flex_1, switch) + ×(stop_propagation → close_project_at)] — the #161 structure.
- fmt; check 0 err; clippy OK.

## Phase 4 — Validate
- **Tests:** no new pure surface (close_project/adjust_active cov/MSI 100 from #150).
- **Self-test:** pc_refuse.png — the × renders on the Project row ("Marley · main ×"); ×-ing the ONLY project flashes "cant close the last project", nothing closes (REQ-002). REQ-001 (close a 2nd project) is picker-gated (the #156 harness boundary — folder-selection undrivable); the mechanics are the tested algebra + the critic-verified sync order.
- **Gate:** GREEN [diff] 15/15. Critic clean (1 LOW advisory → #158/#167).

## Inspect (Phase 3.5)
Method: 1 background critic (the deltas from #161's inspected pattern) + my review + the driven refusal capture.

- **Critic — clean on all 5 axes; one [Low] advisory.**
  1. REAP: ownership is by-value the whole chain (Project → Vec<Tab> → PaneGrid → PaneState → TerminalSession →
     Box<dyn PtyChannel: Send>) — no Rc/Arc, no custom Drop with main-thread affinity; the thread-drop reaps
     everything; Send enforced by the compiler.
  2. ORDER: adjust_active is clamped+tested → sync reads the correct survivor; persist_grid's expect is
     invariant-guarded (every project seeds a terminal tab; LastTerminal refuses).
  3. LastProject arm consistent with #161 (flash on refusal; silent stale-index no-op).
  4. The ×: byte-for-byte #161's shape; co-fire impossible (child-first bubble + stop_propagation). The
     unconditional last_agent=None is equivalent to #161's guarded form (a project always holds ≥1 terminal).
  5. clippy exit 0; tabs 22/22.
- **[LOW — deferred → #158/#167, commented] stale agents/remotes entries widen with project close** — a dead
  project's N×M pane entries linger (display staleness in Details/Fleet under aliasing; AgentRun is pure
  metadata, no resource leak). Selective removal is impossible under aliasing (ids collide with survivors) and
  draining ALL would clobber valid entries — the per-tab scoping (#167, absorbing #158) is the real fix; its
  ticket now explicitly names the project-close path.

Lenses: reap ownership/Send, sync order, error-arm parity, × routing, map hygiene.

## Phase 5 — Complete
- CHANGELOG + app_shell #162 note; forge #162 → done; #158 commented (project-close path named). **M10 2/10.** LESSON: structurally-identical affordances ship back-to-back — the 2nd inherits the 1st critic pass, review only the deltas.
