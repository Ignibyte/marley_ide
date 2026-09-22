# TICKET-371 — Settings: `[[mcp.servers]]` + per-project orchestration config (brain endpoint + web URL) round-trip

- **Forge ticket:** #371 ec7757ce-0d0c-4f4c-b7ac-978fa676a5df (feature, M23)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/371-mcp-servers-settings.spec.md
- **Source ticket:** forge #371 (sprint "M23 — Fleet Control Plane: Layer 1"; orchestration-shell.md §12 Layer-1 slice ⑤)
- **Status:** closed (shipped 2026-07-21; gate GREEN --diff 15/15 first attempt; see pipeline/completed/371-mcp-servers-settings.notes.md)

## Summary
The wiring layer for the fleet control plane, built as the shipped `[[lsp.servers]]` idiom (#308,
`crates/marley_app/src/settings.rs:80-85`) verbatim: (a) a `[[mcp.servers]]` array-of-tables in
settings — name, transport (stdio command+args | http url), enabled, and the per-server permission
grants the `marley_mcp` server (sibling #370) reads (allowed tool classes + write-verb allowlist);
(b) per-project orchestration config — the project's brain MCP endpoint (what the #368 forge-client
subscription reads to know where to subscribe) + its web URL (the future Forge-pane target). This is
the agnosticism face of orchestration-shell.md §2: a non-Forge project points both anywhere, and
NOTHING UCSOS/Forge-specific appears in the schema. Full load/persist/hand-edit-tolerant round-trip
with `#[serde(default)]` on every new field (the #204 RemoteHosts/Workflows lesson — a hand-edited
entry missing a key must not wipe the table). Wiring only: NO live MCP connection behavior ships
here. Consumers: #368 (where to subscribe), #370 (permission grants), the Phase-E browser pane
(web URL).

## Acceptance
`[[mcp.servers]]` and the per-project orchestration table persist → reload as identical values;
a hand-edited entry omitting optional keys loads with declared defaults (enabled=true, empty
args/grants) instead of wiping the table; both transport shapes are representable and a malformed
entry is rejected via a typed error (no panic, siblings survive); the active project's orchestration
config resolves by root and an unconfigured project resolves None; the schema contains no
UCSOS/Forge-specific vocabulary (review); grants are readable through a pure accessor; the whole
seam is pure at cov/MSI 100. Full EARS table in the pipeline spec.
