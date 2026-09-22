# TICKET-063 — marley_forge_client: read the current sprint (HTTP seam)

- **Forge ticket:** #63 `ab4d639f-4eff-4a7a-bf80-8ab325df69e6` (feature, M2.B seq-5)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `20878054-8a0d-441b-9b3f-e478b0e220ca`
- **Pipeline doc:** ../../pipeline/active/forge-client.spec.md
- **Source ticket:** forge sprint #10 `4c988c99-56be-4434-8017-6909db864935` (M2.B — The Agent Cockpit)
- **Status:** closed

## Summary
A new `marley_forge_client` crate: read-only app→forge HTTP seam. PURE (cov/MSI 100): endpoint parse
from `.mcp.json` (bearer-redacting Debug), the raw HTTP/1.1 tools/call request builder, SSE/JSON-RPC
envelope + sprint/ticket JSON parsing → `SprintView`, typed `ForgeError`. Masked adapter: a TcpStream
`fetch` + `current_sprint()`. SECURITY: localhost only, read-only, bearer never logged. Deps: none.

## Acceptance
The 8 pure fns at cov/MSI 100 (endpoint present/missing; redacting Debug; request shape; http/rpc/json
errors; sprint+ticket mapping) + a best-effort live `current_sprint()` check; FULL gate GREEN. Full EARS
in the pipeline spec.
