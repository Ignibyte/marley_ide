# TICKET-085 — remote-pane badge (⇄ host)

- **Forge ticket:** #85 `b17286a1-73ef-4055-9fea-8f4e406478bf` (feature, M3.A seq-3; BACKLOG — sprint pending)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `c8533790-e1cf-48ca-94af-2e676f576786`
- **Pipeline doc:** ../../pipeline/active/remote-badge.spec.md
- **Milestone:** M3.A — The Remote Seam (tickets in the forge backlog tagged M3.A)
- **Status:** closed

## Summary
A "⇄ {host}" corner badge on remote panes (mirror the agent badge #66). PURE: `remote_badge`. SHIM:
`RootView.remotes: HashMap<PaneId, host>` tagged on #84 open + dropped on close (#67); render the badge
top-left. cov/MSI 100 on remote_badge; the tag/render masked + self-test-verified. Deps #83 + #84 + #66 + #67.

## Acceptance
remote_badge at cov/MSI 100 ("⇄ {host}"); the "⇄ localhost" badge shows on a remote pane (self-test);
tag dropped on close; FULL gate GREEN. Full EARS in the spec.
