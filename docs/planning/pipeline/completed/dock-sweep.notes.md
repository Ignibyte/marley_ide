# #171 — dock-vestige sweep — Notes

- **Forge ticket:** #171 `be697f04-ce9a-4857-9c12-6b5ea7d6422d` (chore) · forge AAR not opened (chore; captured here §19)

## Phase 3 — Implement
- settings.rs: removed DockRight, persist_dock_right, AppliedSettings.right (+ defaults/applied_from/3 test
  ctors/the round-trip persist_dock_right call). app.rs: dropped the persist_dock_right import + the dead
  pub toggle_dock(); toggle_dock_state(side) → toggle_left_dock() (left-only persist); docks[1] documented as
  the retired, permanently-Closed slot. fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Self-review (a mechanical dead-code sweep, no new logic):
- DEAD-PROOF: `AppliedSettings.right` had ZERO readers (the docks init used `DockState::Closed` literally, not
  applied.right — grep confirmed); `persist_dock_right` was called only from toggle_dock_state's Right arm,
  which is unreachable ("toggle-right-dock" was repurposed at #153 to open the Details cockpit tab, never the
  dock); `toggle_dock()` pub fn had no callers. All removed at the source.
- BACK-COMPAT: an old settings.toml with `docks.right = false` now has no matching setting — the manager
  tolerates unknown keys on load (same as any retired key), so no boot error; the key is simply inert.
- KEPT HONEST: `DockSide`/`dock()`/`region_widths`(left,right) are the general 2-region layout primitive
  (dock_title(Right)="Details" still unit-tested in layout.rs); `docks[1]` is a documented Closed constant,
  not dead code — region_widths genuinely computes right=0 from it. Not over-collapsed.
- NO BEHAVIOR CHANGE: the left dock toggle path is byte-equivalent (docks[0].toggled() + persist_dock_left).
Lenses: reader-count dead-proof, settings back-compat, primitive-vs-vestige boundary, behavior invariance.

## Phase 4 — Validate
- **Tests:** no new (dead-code removal); the settings round-trip test still passes without the dropped `right` field + persist_dock_right call; layout.rs dock_title(Right) unit test unchanged (the primitive stays). Full suite green.
- **Self-test:** the app launches + renders the rail + Files panel with no regression (ds_open.png); the left Files panel still toggles via 📁. (The "Release Run" tab name from #177 even persisted across the relaunch — a bonus cross-ticket persistence confirmation.) The left-DOCK toggle path is byte-equivalent (docks[0].toggled()+persist_dock_left).
- **Gate:** GREEN [diff] 15/15, MSI 100.

## Phase 5 — Complete
- CHANGELOG + app_shell #171 note; forge #171 → done. **M11 8/8 — sprint complete.** LESSON: dead-proof by reader-count before deleting; keep the general primitive (region_widths) while removing the vestige (persist_dock_right); tolerated-unknown-key = free settings back-compat.
