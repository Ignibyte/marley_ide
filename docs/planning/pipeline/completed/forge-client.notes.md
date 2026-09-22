# marley_forge_client — Notes

- **Forge ticket:** #63 `ab4d639f-4eff-4a7a-bf80-8ab325df69e6`
- **AAR:** `20878054-8a0d-441b-9b3f-e478b0e220ca`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-063-forge-client.md

## Phase 1 — Plan
- **Request:** forge #63 (M2.B seq-5, auto-approved) — the app-to-forge read-only HTTP seam. The
  SECURITY-sensitive ticket (reads the bearer + opens a socket).
- **Classification:** work pipeline, `feature`, a PURE new crate `marley_forge_client` + a masked
  TcpStream adapter. No UI.
- **Security fork resolved (D1, chad's default + approach A):** app reads the bearer from `.mcp.json`;
  localhost 127.0.0.1:8080 only; read-only; bearer NEVER logged (redacting Debug). The approach question
  was surfaced to chad (away); best judgment = approach A (no HTTP dep, clean-room) — the project ethos.
- **Protocol PROBED live (curl):** POST `http://127.0.0.1:8080/mcp/forge`, headers Authorization +
  Content-Type json + Accept `application/json, text/event-stream` + Connection: close; body JSON-RPC
  `tools/call {name, arguments:{}}`; NO initialize handshake; response SSE `data: {jsonrpc}`, the result's
  `content[0].text` = the inner JSON string (sprint-current → `{sprint:{name,number}}`, ticket-list →
  `{tickets:[{number,title,status,type}]}`).
- **Deps:** serde + serde_json already in the lock (confirmed) — reuse.
- **AAR id:** `20878054-8a0d-441b-9b3f-e478b0e220ca`.

## Phase 2 — Design

### Types (`crates/marley_forge_client/src/lib.rs`)
- `pub struct ForgeEndpoint { url: String, bearer: String }` — fields PRIVATE; **manual `impl Debug`**
  prints `ForgeEndpoint { url, bearer: "***" }` (NEVER the bearer). `pub fn url()/bearer()` accessors for
  the adapter.
- `#[derive(Debug,Clone,PartialEq,Eq,Deserialize)] pub struct SprintMeta { pub name, pub number: u64 }`.
- `#[derive(Debug,Clone,PartialEq,Eq,Deserialize)] pub struct TicketView { pub number: u64, pub title,
  pub status, #[serde(rename="type")] pub kind: String }`.
- `#[derive(Debug,Clone,PartialEq,Eq)] pub struct SprintView { pub sprint: SprintMeta, pub tickets:
  Vec<TicketView> }`.
- `#[derive(Debug,Clone,PartialEq,Eq)] pub enum ForgeError { Http(String), Protocol(String),
  Rpc(String), Json(String) }` + `impl Display` + `impl std::error::Error`.

### PURE fns
- `forge_endpoint_from(mcp_json: &str) -> Option<ForgeEndpoint>` — serde_json::from_str to a `Value`;
  `["mcpServers"]["forge"]["url"].as_str()` + `["headers"]["Authorization"].as_str()`; both present →
  Some; else None.
- `fn split_url(url) -> Option<(&str authority, &str path)>` — `strip_prefix("http://")?`, split at the
  first `/` (default path `/`). Only `http://` (localhost). Used by tool_call_request + the adapter.
- `tool_call_request(endpoint, tool: &str, id: u64) -> Option<String>` — body = serde_json of
  `{"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":tool,"arguments":{}}}`; then the
  HTTP/1.1 text: `POST {path} HTTP/1.1\r\nHost: {authority}\r\nAuthorization: {bearer}\r\nContent-Type:
  application/json\r\nAccept: application/json, text/event-stream\r\nConnection: close\r\nContent-Length:
  {len}\r\n\r\n{body}`. `None` if the url is un-splittable.
- `jsonrpc_from_http(response: &str) -> Result<String, ForgeError>` — first line must contain a 2xx code
  (`Err(Http(status))` else); find a line starting `data:` → return the trimmed remainder
  (`Err(Protocol)` if none).
- `tool_text(jsonrpc: &str) -> Result<String, ForgeError>` — Deserialize to `RpcResponse { result:
  Option<{content: Vec<{text}>}>, error: Option<{message}> }` (`Err(Json)` on parse fail); `error` →
  `Err(Rpc(message))`; `result.content.first().text` → Ok; else `Err(Protocol)`.
- `parse_sprint_meta(text) -> Result<SprintMeta, ForgeError>` — Deserialize `{sprint: SprintMeta}` →
  `.sprint` (`Err(Json)`).
- `parse_tickets(text) -> Result<Vec<TicketView>, ForgeError>` — Deserialize `{tickets: Vec<TicketView>}`
  → `.tickets` (empty ok; `Err(Json)` on malformed).

### SHIM/adapter (mutants::skip + cov-excluded)
- `fetch(endpoint, request: &str) -> Result<String, ForgeError>` — `split_url` → `TcpStream::connect(authority)`,
  `write_all(request)`, `read_to_string`; io::Error → `Err(Http)`.
- `pub struct ForgeClient { endpoint: ForgeEndpoint }` + `pub fn new(endpoint)` + `current_sprint(&self)
  -> Result<SprintView, ForgeError>`: for `sprint-current` and `ticket-list` → build request → fetch →
  jsonrpc_from_http → tool_text → parse_*; assemble `SprintView`.

### File manifest
- NEW `crates/marley_forge_client/Cargo.toml` (serde derive + serde_json).
- NEW `crates/marley_forge_client/src/lib.rs` — the types/fns + tests.

### Mutation Targets (pure)
- `forge_endpoint_from` (both-present vs any-absent → the None branches). `split_url` (strip_prefix, the
  `/` find, default path). `tool_call_request` (the Content-Length = body len; the tool name in the body).
  `jsonrpc_from_http` (the 2xx check — a 500 test; the `data:` find — a no-data test). `tool_text` (error
  BEFORE result; `content.first()` empty → Protocol). `parse_*` — serde-derived (few mutants; coverage +
  assertions guard, hollow-MSI family).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `endpoint_from_present_and_absent` — full .mcp.json → Some(url,bearer); missing forge/url/Authorization → None | unit |
| REQ-002 | `endpoint_debug_redacts_bearer` — `format!("{:?}", ep)` contains "***", NOT the bearer | unit |
| REQ-003 | `request_is_well_formed` — POST line + Host + Authorization + Content-Length == body len + the tool name in the body | unit |
| REQ-004 | `jsonrpc_from_http_status_and_data` — 200+data→Ok(json); 500→Err(Http); 200+no-data→Err(Protocol) | unit |
| REQ-005 | `tool_text_ok_rpc_missing` — valid→text; `{error}`→Err(Rpc); `{result:{content:[]}}`→Err(Protocol); junk→Err(Json) | unit |
| REQ-006 | `parse_sprint_and_tickets` — real sprint/ticket JSON → SprintMeta/Vec; `{tickets:[]}`→empty; malformed→Err(Json); rename type→kind | unit |
| REQ-007 | gate GREEN, cov/MSI 100 pure; adapter masked | gate |
| REQ-008 | `live_current_sprint` — read .mcp.json + current_sprint() → the live sprint name; SKIP+log if forge down | best-effort (serial, `#[ignore]`-style guard) |

Uncoverable: `fetch` + `current_sprint` (live socket) — mutants::skip + cov-excluded, proven by REQ-008.

### Risks / decisions
- D-2.1 the crate is PURE of file IO — the app passes `.mcp.json`'s STRING to `forge_endpoint_from` (the
  app shim reads the file). D-2.2 `read_to_string` relies on `Connection: close` so the server EOFs. D-2.3
  the redacting Debug is the security load-bearing test (REQ-002) — a derived Debug WOULD leak, so it's
  manual + asserted. D-2.4 serde ignores unknown fields (forge sends many) — robust to forge additions.

## Phase 3 — Implement
- **Built (PURE, lib.rs):** `ForgeEndpoint` (private fields + manual bearer-redacting Debug + url()/
  bearer() accessors); `forge_endpoint_from`; `split_url` (private); `tool_call_request`;
  `jsonrpc_from_http` + `status_is_2xx`; `tool_text`; `parse_sprint_meta`; `parse_tickets`; `ForgeError`
  (Http/Protocol/Rpc/Json + Display + Error); `SprintView`/`SprintMeta`/`TicketView` (serde Deserialize,
  `type`→`kind` rename); the private RpcResponse/… deserialize structs.
- **Built (MASKED, adapter.rs — mutants::skip + cov-excluded):** `ForgeClient::{new, current_sprint,
  call}` + `fetch` (raw TcpStream POST + read-to-EOF). Uses the pure fns via `super::`.
- **Infra:** added `marley_forge_client/src/adapter\.rs` to the gates.sh coverage `--ignore-filename-regex`
  (the pure/shim seam, like pty_os.rs) + the explanatory comment.
- **Deps:** serde (derive) + serde_json (already in the lock); mutants (dev, for the skip).
- **Deviations:** none — split_url is private (tested directly in the crate's test module).
- **Verification:** `cargo fmt`; `cargo check -p marley_forge_client` 0 err; clippy `-D warnings` OK; in
  the workspace. Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critics:** 2 (CORRECTNESS + SECURITY), each with a verbatim probe crate + real cargo-mutants/llvm-cov.
- **Correctness verdict:** PASS — the pure code is correct, ZERO panics on any malformed input; all edges
  (endpoint None-paths, Content-Length exactness, 2xx/data/error branches, tool_text error-first
  precedence, type→kind rename) confirmed. MSI/cov gaps are P4 test additions (not code defects).
- **Security verdict:** PASS — the bearer redaction is PROBE-PROVEN (20/20: the sentinel secret absent
  from `{:?}`/`{:#?}`/nested, `***` present; no derived Debug/Serialize; the request-string-with-bearer
  only flows to the socket, never an error/log; read-only; no hardcoded secret; §20 clean).
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | S1 | MED (security) | "localhost only" (D1) was DOCUMENTED but NOT ENFORCED — a tampered `.mcp.json` url could send the bearer over cleartext HTTP to an arbitrary host. | **FIXED** — `is_loopback_authority` guard in `forge_endpoint_from`: a non-loopback url → `None` (the endpoint never constructs, so no request is built/sent). D1 is now real, not aspirational. |
  | S2 | LOW (security) | `pub fn bearer()` — a secret-exposing accessor with ZERO callers (tool_call_request reads the private field directly). | **FIXED** — removed. The bearer now has no public getter. |
  | C1 | HIGH-as-stated | `ForgeError` Display untested → 1 survivor (MSI 95.7%) + cov hole. | **P4 test** — assert all 4 Display arms (`ForgeError::Http("d")→"forge HTTP error: d"` etc.). Not a code defect. |
  | C2 | MED | `parse_tickets` malformed-input path uncovered. | **P4 test** — `parse_tickets("not json")→Err(Json)`. |
  | S3/C-LOW | LOW | url not redacted (fine — non-secret + now loopback-guarded); status_is_2xx loose prefix; SSE multi-data assumption. | Accepted; added an SSE-assumption comment. The loopback guard moots the userinfo-leak concern. |
- **Fixes applied (code):** S1 loopback guard (+ `is_loopback_authority`), S2 removed `bearer()`, the SSE
  comment. clippy `-D warnings` OK. **New P4 mutation targets:** `is_loopback_authority` (localhost/
  loopback-IP vs not → tests: 127.0.0.1:8080→ok, localhost→ok, evil.com→None, 8.8.8.8→None) + the
  forge_endpoint_from non-loopback→None branch.

## Phase 4 — Validate
- **Tests added** (lib.rs, 9): endpoint present/absent/**non-loopback→None** (REQ-001+S1); **debug redacts
  bearer** (REQ-002); is_loopback_authority (S1); split_url; request well-formed + exact Content-Length
  (REQ-003); jsonrpc 2xx/data/errors (REQ-004); tool_text ok/rpc/empty/json (REQ-005); sprint+tickets
  incl. type→kind + empty + malformed-both (REQ-006 + C2); **ForgeError Display all 4 arms** (C1).
- **Runs (actual):** `cargo nextest -p marley_forge_client` → 9 passed.
- **LIVE end-to-end (REQ-008):** moved to `examples/check_forge.rs` (kept out of the unit lane so an
  unrun `#[ignore]` test can't dent coverage). Ran it — it hit the REAL forge and printed the live sprint
  **"M2.B — The Agent Cockpit (#10)"** with tickets #65/#64/#63(in-progress)/#52 — proving
  `ForgeClient::current_sprint()` → the raw-TcpStream adapter → parse works end-to-end.
- **Coverage fix:** the first gate RED was the `#[ignore]` live test's un-run body (uncovered) + a rustdoc
  link to the private `adapter` mod. Fixed both (example + plain-text doc ref).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0% (26 caught / 0
  missed). adapter.rs cov-excluded (the gates.sh regex addition).
- **UI:** N/A — library crate.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG; crate-map node+row; new arch doc marley_forge_client.md; gates.sh adapter-exclude comment.
- **Knowledge:** aar-submit (5); ADR-marley-app-to-forge-raw-http-localhost-readonly-redacted-001; PR-claude-enforce-security-invariants-in-code-not-just-docs-001 (HIGH, from the security critic).
- **Ticket:** forge #63 → done; archived. **5/6 of M2.B** — the app can now READ forge (verified live: real M2.B sprint). Only #64 (the Forge pane) remains.
