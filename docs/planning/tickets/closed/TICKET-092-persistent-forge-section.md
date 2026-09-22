# TICKET-092 — persistent Forge section in the dock

- **Forge ticket:** #92 `2e809554-6495-4b7c-830b-c1c9ce2ea827` (feature, M2.F seq-3; sprint #14)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `dd6db9cb-14bf-4aac-8a4c-1667175489f7`
- **Pipeline doc:** ../../pipeline/active/persistent-forge-section.spec.md
- **Status:** closed

## Summary
The sprint view always-visible in the #90 Forge tab. PURE `forge_empty_hint(loading)` (cov/MSI 100); the
SHIM extracts `forge_ticket_row` (the trio-click row) shared by the ⌘⇧F overlay + the Forge section, which
renders the sprint's rows or the hint. Deps #90 + #64 + #69 + #70/#75/#76 + #77.

## Acceptance
forge_empty_hint at cov/MSI 100; the Forge tab lists the sprint's tickets/the hint + the trio works
(static live/engine + forge read); FULL gate GREEN. Full EARS in the spec.
