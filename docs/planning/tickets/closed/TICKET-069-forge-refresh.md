# TICKET-069 — Forge pane: refresh + honest states

- **Forge ticket:** #69 `2baadc57-69db-4cdd-a190-0cdd2dc22b16` (feature, M2.C seq-4)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `654f86a6-ad26-4abb-9194-56589cc5b6f1`
- **Pipeline doc:** ../../pipeline/active/forge-refresh.spec.md
- **Source ticket:** forge sprint #11 `fc38f0c0-0704-40e6-9530-3402f8b4821c` (M2.C — The Living Cockpit)
- **Status:** closed

## Summary
The ⌘⇧F forge pane re-fetches the sprint on OPEN (was frozen at startup) + shows a consistent header /
"forge unreachable" placeholder. PURE: `forge_status_line`. SHIM: store the ForgeClient in RootView +
re-fetch in the toggle-forge dispatch (sync, #63-timeout-bounded). cov/MSI 100 on forge_status_line; the
re-fetch is masked + self-test-verified. Deps #63 + #64.

## Acceptance
forge_status_line at cov/MSI 100 (Some→header; None→unreachable); ⌘⇧F re-fetches + shows the sprint
(self-test); FULL gate GREEN. Full EARS in the spec.
