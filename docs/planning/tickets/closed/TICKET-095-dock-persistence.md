# TICKET-095 — dock-state persistence + M2.F finale

- **Forge ticket:** #95 `8d2c5397-6c55-4586-844f-5fc7b94e1deb` (feature, M2.F seq-6 FINALE; sprint #14)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **Pipeline doc:** ../../pipeline/active/dock-persistence.spec.md
- **Status:** closed

## Summary
Persist + restore the right dock's active tab. PURE `right_section_key`/`right_section_from_key` (round-trip,
unknown→Details) + `RightSectionSetting` + `AppliedSettings.right_section` + `persist_right_section` (cov/MSI
100); the shim boots from the setting + persists on a tab click. Deps #90-94 + marley_settings. CLOSES M2.F.

## Acceptance
The round-trip + applied_from(right_section) at cov/MSI 100; select a tab → relaunch → the dock opens on it
(engine round-trip); FULL gate GREEN. Full EARS in the spec.
