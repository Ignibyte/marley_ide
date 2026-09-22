# TICKET-167 — M10: rail agent icons + unique PaneIds (absorbs #158)

- **Forge ticket:** #167 `409beb81-022d-428d-b68e-7ccda8886411` (feature, M10; sprint #21) — absorbs #158
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `96ea54f1-f7e3-4eca-8705-82c342780fef`
- **Pipeline doc:** ../../pipeline/active/agent-rail.spec.md
- **Status:** closed

## Summary
Every grid mints PaneIds from its own 2³² block (new_with_base; the boot grid keeps 0) — the #158 cross-tab
aliasing dies structurally, the agents/remotes maps stay as-is, and close-cleanup becomes exact. The rail tab
rows then show fleet_status_for's aggregate agent glyph (Working > Waiting > Idle > Exited). Deps #152,
#161, #162.

## Acceptance
new_with_base + fleet_status_for at cov/MSI 100; driven — an agent's tab shows the glyph, a plain tab none,
and it survives a 2nd tab; gate GREEN.
