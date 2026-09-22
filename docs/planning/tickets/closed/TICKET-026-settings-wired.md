# TICKET-026 — settings wired: the marley settings schema + boot-load + persist-on-change

- **Forge ticket:** #26 `be140e90-3ddd-41e5-8110-4e76ca89112a` (feature, M1.C — The Wired Cockpit, seq-5, THE CLOSER)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `c91e044c-40ea-4633-9f0c-06f963d41697`
- **Pipeline doc:** ../../pipeline/active/settings-wired.spec.md
- **Source ticket:** forge sprint #3 `b295976b-4f32-4edc-be3c-aecd80b02785` (M1.C — The Wired Cockpit)
- **Status:** closed

## Summary
`marley_settings` (#21) shipped a full framework and NOTHING uses it. Wire marley_app to it:
a typed schema (`appearance.theme`, `docks.left`/`right`, `terminal.rows`/`cols`) loaded at boot
to apply the saved theme + dock states, and persisted on change — so theme + dock state survive a
relaunch. Config-dir isolation via `*_in(dir)` fns (no live ~/.marley in tests); a missing/invalid
file boots with defaults, never a failure. Pane-layout restore is deferred to M2.

## Acceptance
The pure `applied_from` (theme-validate + defaults) + the `*_in(dir)` load/persist seam at cov
100/MSI 100 (each default flip, unknown-theme fallback, dock-side swap, rows↔cols swap, invalid→
defaults); a set→reload→applied round-trip proves persistence; FULL gate GREEN [--diff]. End
state: theme + dock state survive relaunch. Full EARS in the pipeline spec. Closes the M1.C
feature set (#22–#26); #27 (delete marley_spike) is the sprint's cleanup chore.
