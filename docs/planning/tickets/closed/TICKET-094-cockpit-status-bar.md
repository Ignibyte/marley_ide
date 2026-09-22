# TICKET-094 — the cockpit status bar (sprint · agents · focus)

- **Forge ticket:** #94 `111a4867-649d-462f-86b5-f84b6b3e04c6` (feature, M2.F seq-5; sprint #14)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `58ee5207-3dc8-47d5-a184-8db01dd3f58b`
- **Pipeline doc:** ../../pipeline/active/cockpit-status-bar.spec.md
- **Status:** closed

## Summary
A persistent bottom footer strip: sprint · agents · focus. PURE `agent_summary` / `sprint_summary` /
`cockpit_status` (cov/MSI 100); the SHIM renders the strip fed each frame. Deps #64 + #67/#68 + #39.

## Acceptance
The 3 summary fns at cov/MSI 100 (0/pluralization/working/Some-None/order); the footer shows the status
(static live/engine); FULL gate GREEN. Full EARS in the spec.
