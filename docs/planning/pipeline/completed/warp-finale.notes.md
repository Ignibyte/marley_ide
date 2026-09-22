# the Warp default arrangement + finale (M6 seq-8 FINALE) — Notes

- **Forge ticket:** #127 `fb0f6938-8b86-46fb-9480-6e734c2dbf1c` · **AAR:** `c58ed9fe-6b8c-45bc-9e39-8d1465c3ad86`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-127-warp-finale.md

## Phase 1 — Plan
- **Request:** forge #127 (M6 run 10/10, FINALE) — land the Warp arrangement + polish; capture vs the references; close M6.
- **Pre-flight:** default_grid EXISTS (grid_layout.rs:83, private, single Terminal); status bar EXISTS (status_bar.rs #94);
  boot restore_grid(&applied.grid) already gives first-run single-Terminal. The right dock defaults open → the #126
  top tabs overlap its header + the right isn't free.
- **Decisions:** D1 pub default_grid + a direct test (status_segments already there); D2 right dock defaults CLOSED
  (free right side + fixes the #126 overlap).
- **AAR id:** `c58ed9fe-6b8c-45bc-9e39-8d1465c3ad86`.

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
- **grid_layout.rs (PURE):** make default_grid pub + a direct test (default_grid() == one Terminal, no split).
- **app.rs (SHIM):** boot docks:[applied.left, applied.right] → [applied.left, DockState::Closed] so the right side is free (Warp arrangement) + the #126 top-tab/dock-header overlap is gone.
- **Test plan:** default_grid_is_single_terminal.
- **Risks:** the boot dock override ignores the persisted right-dock state (intended — cockpit on-demand via the top tabs). Verify no other code assumes right open at boot.

## Phase 3 — Implement
- **Built (grid_layout.rs PURE):** default_grid is now `pub` + documented (the first-run single-Terminal default; also the malformed/empty fallback). **(app.rs SHIM):** boot docks = [applied.left, DockState::Closed] — the right side starts free (Warp arrangement); the top tabs (#126) / ⌘⇧B open the cockpit on demand.
- **DEVIATION:** the persisted right-dock OPEN state is ignored at boot (always starts closed) — intended for the on-demand cockpit; the left/sessions dock keeps its persistence. status_segments not added (status_bar #94 already exists).
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a visibility change + a 1-line boot default).
- **Lenses — no findings:** default_grid pub — unchanged body (one Terminal); still the restore_grid fallback. The boot right-dock default = Closed; applied.right stays computed+tested in settings.rs (not dead) — only the boot ignores it (by design). ⌘⇧B + the top tabs still open the right dock (docks[1]). No dangling refs (0 compile err; clippy OK). No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** default_grid_is_single_terminal (REQ-001). Pass.
- **Reference:** the Warp screenshots (~/Downloads, 2026-07-06) show Warp: LEFT session sidebar · full-width main area with NO right rail · top search bar · bottom status bar. Marley now matches structurally (Files+sessions left · pane grid · top cockpit tabs · bottom status · right free).
- **Self-test:** LIVE capture (finale127.png) — grid="H:t,f,c": Files+sessions sidebar (left) | pane grid (terminal w/ focus edge + files + code, filling to near the right edge) | cockpit tabs at the top | bottom status bar | RIGHT SIDE FREE (no rail). Structurally matches the Warp reference (sidebar · full-width main · top bar · status bar). Marley's own cyan/dark identity. REQ-002/003 PASS. Minor: slight top-right crowding (the code pane title × near the Forge tab) — cosmetic.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG; forge #127 → done. **M6 10/10 — THE WARP LAYOUT COMPLETE.** pub default_grid + the right side defaults free (Warp arrangement; #126 overlap gone). Capture structurally matches the Warp reference. Closes M6 sprint #17.
