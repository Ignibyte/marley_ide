# TICKET-093 — rich focused-pane Details inspector

- **Forge ticket:** #93 `a734efda-d9f1-46cb-a174-41c99471b33b` (feature, M2.F seq-4; sprint #14)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `729d1893-0aee-4497-b7af-d0786824c8eb`
- **Pipeline doc:** ../../pipeline/active/pane-details-inspector.spec.md
- **Status:** closed

## Summary
The Details tab as a per-kind focused-pane inspector. PURE `pane_details.rs`: `DetailRow` + `agent_details`
/ `remote_details` / `terminal_details` (cov/MSI 100); the shim dispatches by focused pane kind (agent /
remote / terminal / empty). Deps #58 + #66 + #85/#86 + #90 + #80.

## Acceptance
The 3 projectors at cov/MSI 100; the Details tab shows the focused pane's kind rows (static live/engine);
FULL gate GREEN. Full EARS in the spec.
