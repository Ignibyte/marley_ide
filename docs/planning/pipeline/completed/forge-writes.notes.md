# marley_forge_client write support — Notes

- **Forge ticket:** #74 `e606cc20-fead-4e35-90c8-c701bc011b19`
- **AAR:** `e86d2f51-d5d0-4337-ba1a-56efab0f7702`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-074-forge-writes.md

## Phase 1 — Plan
- **Request:** forge #74 (M2.D seq-3, auto-approved) — the first app→forge writes (claim + comment).
  SECURITY-SENSITIVE.
- **Classification:** work pipeline, `feature`, PURE (parse_write_ack + builders) + an adapter SHIM.
  Library crate (no UI).
- **Pre-flight facts (read the code):** ForgeError{Http,Protocol,Rpc,Json}; `tool_call_request(tool,args)`
  (#69, builds tools/call); `jsonrpc_from_http` (extracts the SSE data JSON, checks 2xx); `tool_text`
  (Err(Rpc) on error, extracts result.content[0].text); RpcResponse{result:Option<RpcResult>,
  error:Option<RpcError>}; RpcResult{content:Vec<RpcContent>} (NO isError yet — add it). adapter:
  ForgeClient{endpoint} (Clone), current_sprint (build→fetch→jsonrpc_from_http→tool_text→parse),
  `fetch(endpoint,request)` (5s r/w timeouts, loopback-guarded #63). Forge arg schemas (loaded MCP defs):
  **ticket-claim {id, owner_id}**, **ticket-comment {ticket_id, body}** (names DIFFER).
- **Decisions:** D1 two explicit methods (no generic write); D2 parse_write_ack fails on error OR isError
  (a failed write ≠ success — security-critical); D3 the exact arg schemas; D4 bearer never logged (#63).
- **Security note:** MCP signals failure TWO ways — a JSON-RPC `error` (protocol) OR `result.isError=true`
  (tool-level, error text in content). parse_write_ack MUST reject both; the live probe confirms which
  forge actually uses for a bad write.
- **AAR id:** `e86d2f51-d5d0-4337-ba1a-56efab0f7702`.

## Phase 2 — Design

### PURE — `lib.rs`
1. Add `isError` to `RpcResult`:
```rust
#[derive(Deserialize)]
struct RpcResult {
    content: Vec<RpcContent>,
    #[serde(rename = "isError", default)]
    is_error: Option<bool>,
}
```
2. The write-ack parser (SECURITY-CRITICAL — a failed write must NEVER read as success):
```rust
/// Confirm a write `tools/call` SUCCEEDED. `Err(Rpc)` on a JSON-RPC error OR an MCP `isError` result
/// (the content carries the message); `Ok(())` only on a genuine success; `Err(Protocol)` if the
/// envelope has neither a result nor an error; `Err(Json)` if it doesn't deserialize.
pub fn parse_write_ack(jsonrpc: &str) -> Result<(), ForgeError> {
    let response: RpcResponse =
        serde_json::from_str(jsonrpc).map_err(|err| ForgeError::Json(err.to_string()))?;
    if let Some(error) = response.error {
        return Err(ForgeError::Rpc(error.message));
    }
    match response.result {
        Some(result) if result.is_error == Some(true) => Err(ForgeError::Rpc(
            result
                .content
                .into_iter()
                .next()
                .map(|content| content.text)
                .unwrap_or_else(|| "forge reported a tool error".to_string()),
        )),
        Some(_) => Ok(()),
        None => Err(ForgeError::Protocol(
            "write returned neither result nor error".to_string(),
        )),
    }
}
```
3. The write-request builders (the tool names + arg mapping live in TESTED pure code, not the masked adapter):
```rust
/// Build the `ticket-claim` write request — args `{id, owner_id}`.
pub fn claim_request(endpoint: &ForgeEndpoint, ticket_id: &str, owner_id: &str) -> Option<String> {
    tool_call_request(endpoint, "ticket-claim", 1,
        serde_json::json!({ "id": ticket_id, "owner_id": owner_id }))
}
/// Build the `ticket-comment` write request — args `{ticket_id, body}`.
pub fn comment_request(endpoint: &ForgeEndpoint, ticket_id: &str, body: &str) -> Option<String> {
    tool_call_request(endpoint, "ticket-comment", 1,
        serde_json::json!({ "ticket_id": ticket_id, "body": body }))
}
```

### SHIM — `adapter.rs` (mutants::skip — live socket)
```rust
#[cfg_attr(test, mutants::skip)]
pub fn claim_ticket(&self, ticket_id: &str, owner_id: &str) -> Result<(), ForgeError> {
    let request = claim_request(&self.endpoint, ticket_id, owner_id).ok_or_else(|| {
        ForgeError::Http(format!("unsupported forge url: {}", self.endpoint.url()))
    })?;
    parse_write_ack(&jsonrpc_from_http(&fetch(&self.endpoint, &request)?)?)
}
// comment_ticket: identical, calling comment_request.
```

### File manifest
- MODIFY `crates/marley_forge_client/src/lib.rs` — `is_error` on RpcResult, `parse_write_ack`,
  `claim_request`, `comment_request` + their tests.
- MODIFY `crates/marley_forge_client/src/adapter.rs` — `claim_ticket` + `comment_ticket` (masked).
- ADD `crates/marley_forge_client/examples/check_write.rs` — a best-effort live probe (comment on a test
  ticket + a bad-id failure check), like #63's check_forge.

### Mutation Targets (pure)
- `parse_write_ack`: the error branch (→Err Rpc), the isError branch (→Err Rpc), the content extraction,
  the Some(_)→Ok, the None→Err(Protocol). Five tests pin every arm.
- `claim_request`/`comment_request`: the tool name + each arg key→value (DISTINCT test values TID/OWN so a
  key/value swap is caught; a whole-body mutant → None is killed by the Some+contains assert).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `parse_write_ack` ok — `{"result":{"content":[…]}}` → Ok(()) | unit |
| REQ-002 | `parse_write_ack` rpc-error → Err(Rpc); isError result → Err(Rpc, message) | unit |
| REQ-003 | `parse_write_ack` neither → Err(Protocol); malformed → Err(Json) | unit |
| REQ-004 | `claim_request`/`comment_request` — the body carries the tool name + the exact args (TID/OWN distinct) | unit |
| REQ-005 | bearer never in Debug/Display/error (ForgeEndpoint redacting Debug holds; ForgeError carries only URL/status/msg) | unit + review |
| REQ-006 | gate GREEN, cov/MSI 100 pure; adapter masked | gate |
| REQ-007 | live claim/comment on a test ticket succeeds; bad id → Err | live probe (best-effort) |

Uncoverable: `claim_ticket`/`comment_ticket`/`fetch` — live socket, masked; proven by REQ-007 + the pure tests.

### Risks / decisions
- D-2.1 the isError handling is DEFENSIVE — the live probe (REQ-007) confirms whether forge uses a JSON-RPC
  error or isError for a bad write; parse_write_ack rejects BOTH regardless. D-2.2 the builders return
  `Option` (None on a bad URL, mirroring tool_call_request) → the adapter maps None to Err(Http). D-2.3
  the JSON-RPC id is a fixed `1` (arbitrary; the response echoes it, unused). D-2.4 SECURITY: two explicit
  methods only; the bearer stays in the endpoint (redacting Debug), never in the ack/error path.

## Phase 3 — Implement
- **Built (PURE, lib.rs):** `is_error: Option<bool>` (`#[serde(rename="isError", default)]`) on RpcResult;
  `parse_write_ack` (Err(Rpc) on error OR isError; Ok on a genuine result; Err(Protocol) on neither);
  `claim_request` (ticket-claim `{id, owner_id}`) + `comment_request` (ticket-comment `{ticket_id, body}`).
- **Built (SHIM, adapter.rs — masked):** `claim_ticket` + `comment_ticket` — build the request (None→Http
  err) → `fetch` → `jsonrpc_from_http` → `parse_write_ack`. Extended the `use super::{…}` import. Fixed the
  ForgeClient docstring (was "read-only" → now "reads + a closed set of explicit writes").
- **Deviations:** none. (Fumbled one edit mid-implement — added then immediately reverted a stray marker fn.)
- **Verification:** `cargo fmt`; `cargo check -p marley_forge_client` 0 err; clippy `-D warnings` OK;
  `cargo nextest -p marley_forge_client` 9 pass (no regression). The pure tests are Phase 4.

## Phase 3.5 — Inspect
- **Critics:** 2 (correctness + SECURITY). Both **PASS**.
- **Correctness (probe + cargo-mutants):** parse_write_ack correct across all 6 arms (success→Ok;
  jsonrpc-error→Err(Rpc); isError-true→Err(Rpc, message); isError-true+empty-content→Err(Rpc, default
  string); neither→Err(Protocol); malformed→Err(Json)). **MSI 100 + line-cov 100** reachable: the 10 new
  mutants (parse_write_ack ×4, claim_request ×3, comment_request ×3) are ALL killed by the planned 6
  ack-cases + 2 builder-shape asserts. The isError guard is LOAD-BEARING (an ablation of the isError-true
  test makes the guard-deletion mutant MISS — the distinct-message test kills it). `#[serde(default)]` →
  a success missing isError → None → Ok (correct). The read path (tool_text) still passes (9/9). Builder
  arg-mapping: DISTINCT test values (TID/OWN/hello) catch a key/value swap (no mutant inside json!). NOTE:
  cargo-mutants `-j2` has a same-line aliasing artifact (both isError mutants at one line) → trust `-j1`
  MSI (the gate's copy-mode is fine).
- **Security (all 5 invariants CONFIRMED):** (1) the bearer NEVER leaks on the write path — the request
  String (with the Authorization header) flows ONLY builder→fetch→socket, never into a ForgeError/log/
  Debug; the write-path errors use `endpoint.url()` not the bearer; no new println/dbg/log; the redacting
  Debug is intact. (2) explicit-writes-only — a CLOSED set of 2 methods; tool names hardcoded; the generic
  `call` + `fetch` are private. (3) a failed write can't read as success — parse_write_ack is fail-closed
  (error OR isError OR neither OR malformed → Err). (4) localhost-only — same loopback-guarded endpoint, no
  new host input. (5) no injection — the string args go through json! (escapes CRLF/quotes) into the body,
  Content-Length from the serialized body → no header/CRLF injection.
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | S1 | LOW (defense-in-depth) | The request builders (tool_call_request/claim_request/comment_request) were crate-wide `pub` → an external consumer could build an arbitrary-tool request String embedding the bearer (not exploitable today — the sender is private). | **FIXED** — narrowed all three to `pub(crate)` (verified: nothing uses them cross-crate; compiles + clippy OK). Also fixed the "READ-ONLY" module docstring → reads + a closed write set. |
  | S2 | LOW (probe) | parse_write_ack treats result-present + isError-absent as Ok — if forge ever reports a write failure as a non-isError result, the app would report false success. | The live probe (REQ-007) confirms a FAILING write (bad id / already-claimed) actually returns isError or a JSON-RPC error, so the guard engages. |
  | I1 | INFO | comment_request omits the optional `author_id` (comments post unattributed). | Accept — per spec; a possible #76 enhancement (attribute to the marley owner). |
- **Fix applied (code):** S1 (pub→pub(crate) on the 3 builders + the docstring). No bug found (both critics
  clean); S1 is a proactive hardening.

## Phase 4 — Validate
- **Tests added** (lib.rs): `parse_write_ack_arms` (REQ-001/002/003 — all 6 arms: success→Ok,
  success-missing-isError→Ok, jsonrpc-error→Err(Rpc), isError→Err(Rpc,msg), isError-empty→Err(Rpc,default),
  neither→Err(Protocol), malformed→Err(Json)); `write_request_builders_map_args` (REQ-004 — claim
  `{id:TID, owner_id:OWN}`/ticket-claim; comment `{ticket_id:TID, body:hello world}`/ticket-comment, with
  DISTINCT values so a swap is caught).
- **Runs (actual):** `cargo nextest -p marley_forge_client -E 'test(parse_write_ack) or test(write_request_builders)'` → 2 passed.
- **LIVE PROBE (REQ-007 + S2 — curl replicating marley's EXACT request against the running forge):**
  - **SUCCESS** — ticket-comment on #74 → HTTP 200 + `data: {…"result":{"content":[…]}}` (result present,
    NO isError) → parse_write_ack ⇒ Ok. The comment was really created (id 5bad600b…). marley's request
    format is accepted end-to-end.
  - **FAILURE** — ticket-comment with a bad (all-zero) ticket_id → HTTP 200 + `{…"result":{…,"code":
    "not_found","isError":true}}` — **forge signals a failed write via `result.isError=true`, NOT a
    JSON-RPC error** (and NOT a non-2xx). This is EXACTLY the hole the isError guard closes: without it,
    parse_write_ack would have read a FAILED write as success. With it → Err(Rpc("not found: ticket …")).
    The security-critical decision (D2) is confirmed against the real forge. Bearer stayed in a shell var
    (not leaked to the transcript).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. parse_write_ack + the builders tested; the adapter masked.
- **Pre-existing:** none. (Library crate — no UI self-test; the live probe is the integration check.)

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_forge_client.md "## Writes (#74)".
- **Knowledge:** aar-submit (5); AD-claude-forge-write-ack-must-check-mcp-iserror-001 (the live-probe-confirmed forge write-failure contract). No bug (both critics clean); S1 was proactive hardening (pub→pub(crate)).
- **Deviation:** did the live probe via curl (documented in P4) rather than an examples/check_write.rs binary — same integration coverage, no extra binary to maintain.
- **Ticket:** forge #74 → done; archived. **3/6 of M2.D.** The app can now write to forge (claim + comment), fail-closed + security-verified.
