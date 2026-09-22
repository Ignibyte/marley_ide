# TICKET-404 — forge_web_base — the .mcp.json → Forge web-origin derivation seam (the slice-3 input, pinned)

- **Forge ticket:** #404 e84add90-0452-4bfa-9772-4ca04a54d5ae (feature, M29)
- **Owner:** c484e8af-a328-4d8e-9534-b0e2deacca12 (claimed 2026-08-07, /work 404)
- **AAR:** 62602a55-22db-42ed-9036-2ef6400646d1
- **Pipeline doc:** ../../pipeline/active/404-forge-web-base.spec.md
- **Source ticket:** M29 sprint (forge sprint #40, 6dff19c6-fb61-472b-89c3-98d3215a489b)
- **Status:** closed (done 2026-08-07 — shipped `forge_web_base` at cov/MSI 100; inspect closed a real wall-bypass, BF-claude-wall-admit-vs-parsed-host-divergence-001)

## Summary
Pin the embedded-browser train's slice-3 INPUT ahead of the pane: a pure derivation of the Forge
web-UI base from the SAME `.mcp.json` active-root resolution #381 shipped
(`mcp_config::mcp_json_path`) — one config, THREE consumers (the MCP client and the fleet
brain-endpoint decision already share the one resolved string at app.rs:2400-2410/2426-2428; the
browser URL becomes the third). `mcpServers.forge.url` is the MCP JSON-RPC endpoint (`…/mcp`); the
web base is that endpoint's ORIGIN — scheme+host+port per the WHATWG origin definition, path
dropped. Keyed on the restored ACTIVE root, never the launch cwd
(`PR-claude-boot-decisions-key-the-restored-active-root-001`). Auth inherited, not invented (Q3 D3):
the loopback wall stands and the MCP bearer never enters the URL or page JS (#370/#375 lineage).
Unresolved on any arm → `None` → NO Browser content (the empty rail home, never a broken page — the
presentation-free sibling of #384's misconfigured-state seam). Pure seam at cov/MSI 100 including
hostile inputs; consumer wiring lands with #405.

## Acceptance
`forge_web_base` (exact signature Phase 2's) derives `Some(origin)` from a resolved, wall-passing
`.mcp.json` — path dropped, table-proven — and `None` on every unresolved arm (no file, unreadable,
unparseable, no forge entry, wall-refused url), each arm a named unit; the derived string provably
contains no bearer bytes even when the config carries one; total over hostile inputs (no panic, no
unwrap, §14); gate `--diff` green with cov/MSI 100 on the touched seam. Full EARS in the pipeline
spec.
