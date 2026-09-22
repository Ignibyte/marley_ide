# TICKET-074 — marley_forge_client write support (claim + comment)

- **Forge ticket:** #74 `e606cc20-fead-4e35-90c8-c701bc011b19` (feature, M2.D seq-3, SECURITY-sensitive)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `e86d2f51-d5d0-4337-ba1a-56efab0f7702`
- **Pipeline doc:** ../../pipeline/active/forge-writes.spec.md
- **Source ticket:** forge sprint #12 `c93f9693-5569-4208-bb6a-20d38afec99b` (M2.D — The Controlling Cockpit)
- **Status:** closed

## Summary
The first app→forge WRITES: `ForgeClient::claim_ticket` + `comment_ticket` (two explicit mutating methods).
PURE: `parse_write_ack` (fails on a JSON-RPC error OR result.isError — a failed write never reads as
success) + the request builders. SHIM: build→fetch→parse_write_ack. localhost + loopback-guarded bearer,
never logged. cov/MSI 100 on the pure surface; 2 critics (correctness + security). Deps #63 + #69.

## Acceptance
parse_write_ack + the builders at cov/MSI 100 (Ok / Err(Rpc)-on-error-or-isError / Err(Protocol) / Err(Json);
exact tool names + args); bearer never leaks; a live claim/comment succeeds + a bad id → Err; FULL gate
GREEN. Full EARS in the spec.
