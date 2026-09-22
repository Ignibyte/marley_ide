# TICKET-086 — remote connection status (connected / disconnected)

- **Forge ticket:** #86 `ff987d71-bfe9-4117-bb2d-b2ed83862693` (feature, M3.A seq-4; BACKLOG — sprint pending)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `07855e62-39db-494a-a810-02fa6fc696ed`
- **Pipeline doc:** ../../pipeline/active/remote-status.spec.md
- **Milestone:** M3.A — The Remote Seam (tickets in the forge backlog tagged M3.A)
- **Status:** closed

## Summary
When a remote pane's ssh exits, mark it Disconnected + keep it (don't auto-close) + flash — the badge
flips ⇄ → ✗. PURE: `RemoteStatus` + `remote_status_from` + `remote_status_glyph` + `remote_badge(host,
status)`. SHIM: `remotes` value grows to `Remote{host,status}`; the #67 pump loop branches remote panes.
cov/MSI 100 on the pure surface; the pump branch masked + self-test-verified. Deps #84 + #85 + #67 + #77.

## Acceptance
RemoteStatus/from/glyph/badge at cov/MSI 100; a dead ssh pane stays w/ a ✗ badge + a disconnected flash
(self-test); FULL gate GREEN. Full EARS in the spec.
