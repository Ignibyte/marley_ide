---
pipeline_id: ccfa11e9-8c96-4c6d-a50e-53462e5e1cee
ticket: forge#74 (e606cc20-fead-4e35-90c8-c701bc011b19) · local docs/planning/tickets/open/TICKET-074-forge-writes.md
aar_id: e86d2f51-d5d0-4337-ba1a-56efab0f7702
status: Phase 5 — Complete PASS
title: marley_forge_client write support (claim + comment)
type: feature
milestone: M2.D
security_sensitive: true
references:
  - crates/marley_forge_client/src/lib.rs (PURE: parse_write_ack + the request builders + isError)
  - crates/marley_forge_client/src/adapter.rs (SHIM: claim_ticket + comment_ticket)
---

## Title
The FIRST app→forge WRITES: `ForgeClient::claim_ticket` + `comment_ticket` — two EXPLICIT mutating methods
(claim a ticket, add a comment) over the existing localhost, bearer-guarded MCP transport. Read-only until
now; this opens writing, carefully.

## Scope
### In
- PURE (`lib.rs`, cov/MSI 100): `parse_write_ack(jsonrpc) -> Result<(), ForgeError>` (the security-critical
  ack parser — Err on a JSON-RPC error OR an MCP `isError` result; Ok only on a genuine success); add
  `is_error: Option<bool>` (`#[serde(rename="isError")]`) to `RpcResult`; the two request builders (reusing
  `tool_call_request` with the exact forge tool names + args).
- SHIM (`adapter.rs`, masked): `claim_ticket(&self, ticket_id, owner_id)` + `comment_ticket(&self,
  ticket_id, body)` — build → `fetch` (the #63 loopback-guarded, timeout-bounded transport) →
  `jsonrpc_from_http` → `parse_write_ack`. Sync (mirrors current_sprint).

### Out
- A generic `write(tool, args)` — ONLY these two named methods (the app can never call an arbitrary
  mutating tool). The bg-thread wrapping (that's #75/#76's job — the client stays sync). Any UI. Editing /
  deleting comments (forge is append-only). Claiming with `release`.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — TWO explicit methods, not a generic write — the mutating surface is a closed set of 2.
- D2 — `parse_write_ack` treats BOTH a JSON-RPC `error` AND `result.isError == true` as failure (a failed
  write must NEVER read as success — the security-critical invariant). The real forge failure shape is
  confirmed by a live probe at validate.
- D3 — the exact forge arg schemas (from the MCP tool defs): ticket-claim `{id, owner_id}`; ticket-comment
  `{ticket_id, body}` (the names deliberately differ).
- D4 — SECURITY (inherits #63): localhost + loopback-guarded endpoint; the bearer is NEVER logged
  (ForgeEndpoint's redacting Debug; no Debug on ForgeClient; ForgeError carries only URL/status/RPC-msg).
  The writes are user-gated downstream (#75/#76).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `parse_write_ack` sees a `result` present and not `isError`, it shall return `Ok(())`. | unit |
| REQ-002 | WHEN `parse_write_ack` sees a JSON-RPC `error` OR `result.isError == true`, it shall return `Err(Rpc)`. | unit |
| REQ-003 | WHEN `parse_write_ack` sees neither result nor error, it shall `Err(Protocol)`; malformed → `Err(Json)`. | unit |
| REQ-004 | WHEN a claim/comment request is built, the body shall carry the exact tool name + args (claim `{id,owner_id}`; comment `{ticket_id,body}`). | unit |
| REQ-005 (security) | The bearer shall never appear in any Debug/Display/error output of the write path. | unit + review |
| REQ-006 | `scripts/gates.sh` GREEN, cov/MSI 100 on the pure surface; the adapter methods masked. | gate |
| REQ-007 (integration) | A live claim/comment against a test ticket shall succeed; a bad id shall surface as `Err`. | live probe (best-effort) |

## Phase Plan
- **P2** — parse_write_ack + isError + the builders + the adapter methods, mutation targets, test plan.
- **P3** — implement the pure + the adapter.
- **P3.5** — TWO critics (correctness + SECURITY): the ack parser can't false-positive a failed write; the
  bearer never leaks; explicit-methods-only; the arg names/shape; MSI 100.
- **P4** — the unit tests (cov/MSI 100) + the live forge probe (success + failure shape) + gate GREEN.
- **P5** — docs, AAR, archive, close #74.
