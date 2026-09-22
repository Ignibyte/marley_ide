# TICKET-374 — marley_mcp: wire configured `[mcp.expose]` grants into `start_mcp_server` (S3 — #370/#371 follow-up)

- **Forge ticket:** #374 1200f35c-e444-4c16-aa68-c1347269684b (feature, M23.5)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/374-mcp-expose-grants-wiring.spec.md
- **Source ticket:** forge #374 (sprint #34 "M23.5 — Fleet Layer-1 Consolidation"; the S3 deferral recorded in pipeline/completed/371-mcp-servers-settings.notes.md)
- **Status:** closed

## Summary
The deliberately-deferred S3 slice from #371: the `[[mcp.servers]]` table shipped with per-server
`allow`/`allow_write` grants and a pure `grants() -> GrantTable` accessor, but #370's live loopback
expose server still starts with the hardcoded default table —
`McpHost::start(salt, marley_mcp::GrantTable::default())` at `crates/marley_app/src/app.rs:6589` —
so NO operator configuration reaches the running permission check and every write verb is denied
regardless of settings. The blocker #371 named was the entry-selection convention (which config row is
"us", Marley's own expose server, vs the client-side entries Marley subscribes to). Resolution: a
separate singleton `[mcp.expose]` TOML table — a transport-less expose row does NOT belong in
`[[mcp.servers]]`, where the #371 `transport()` resolver types no-command-and-no-url as the ERROR
`McpConfigError::NoTransport` (config.rs:117). A minimal `ExposeConfig { allow, allow_write }` in
`marley_mcp`, wired through the #371 `define_setting!` idiom, built into a `GrantTable` via the
EXISTING `GrantTable::from_classes(allow_write)` semantics, and passed into the unchanged
`McpHost::start` by the `mcp-serve` verb callsite. Absent table ⇒ exactly today's default (a
regression pin). Soft-ordered before #375 (hardening pins the final server-start surface).

## Acceptance
`[mcp.expose] allow_write = ["session.write"]` makes a `tools/call session.surface_to_human` pass the
live permission check while every unlisted write class stays denied (deny-by-default preserved); an
absent `[mcp.expose]` yields a GrantTable byte-equal to today's default; full #371-shape settings
round-trip + missing-key + malformed-value tolerance. Full EARS table in the pipeline spec.
