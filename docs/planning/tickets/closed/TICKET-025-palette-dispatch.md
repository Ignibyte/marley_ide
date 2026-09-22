# TICKET-025 — palette dispatch: ↑/↓ selection + Enter actually runs the command

- **Forge ticket:** #25 `9668f539-15ec-4237-ad5e-c67ecc724126` (feature, M1.C — The Wired Cockpit, seq-4)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `37a1fd65-6965-43d2-90b4-bec585b455b7`
- **Pipeline doc:** ../../pipeline/active/palette-dispatch.spec.md
- **Source ticket:** forge sprint #3 `b295976b-4f32-4edc-be3c-aecd80b02785` (M1.C — The Wired Cockpit)
- **Status:** closed

## Summary
Enter currently just closes the palette — no command has ever dispatched. Build the pure
`PaletteState` selection model (↑/↓ clamp, reset-on-edit, activate → the SELECTED CommandId) and
the pure `action_for_command` dispatch table, wire Enter through the ONE `dispatch_action` verb
router (split/close/toggle-docks/toggle-theme — R23's `set_theme` finally called), drop the inert
"Open Settings", render selection highlight + chord chips.

## Acceptance
PaletteState + dispatch table at cov 100/MSI 100 (clamp both ends, reset-on-edit, shrink-rebound,
activate-on-empty None, a wrong-verb mutant per table row dies); Enter dispatches exactly once
then dismisses; Escape unchanged; FULL gate GREEN [--diff]. Full EARS in the pipeline spec.
