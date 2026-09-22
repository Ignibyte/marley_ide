---
pipeline_id: 9dad7f1a-f770-4cba-8ba9-3c70818181c4
ticket: forge#63 (ab4d639f-4eff-4a7a-bf80-8ab325df69e6) · local docs/planning/tickets/open/TICKET-063-forge-client.md
aar_id: 20878054-8a0d-441b-9b3f-e478b0e220ca
status: Phase 5 — Complete PASS
title: marley_forge_client — read the current sprint (HTTP seam)
type: feature
milestone: M2.B
references:
  - crates/marley_forge_client/ (NEW crate — endpoint/request/parse + a masked TcpStream fetch)
---

## Title
The seam that lets the RUNNING app read forge (read-only): a `marley_forge_client` crate that reads the
bearer from `.mcp.json`, POSTs a JSON-RPC `tools/call` to the localhost forge over a raw HTTP/1.1
socket, and parses the sprint + its tickets into a `SprintView`. #64 renders it.

## Scope
### In (all `crates/marley_forge_client/`, cov/MSI 100 on the PURE parts)
- **PURE** (`src/lib.rs`): `ForgeEndpoint { url, bearer }` (manual `Debug` REDACTS the bearer);
  `forge_endpoint_from(mcp_json)`; `tool_call_request(endpoint, tool, id)` (raw HTTP/1.1 text);
  `jsonrpc_from_http(response)` (2xx + the SSE `data:` line); `tool_text(jsonrpc)` (`result.content[0].text`);
  `parse_sprint_meta(text)`; `parse_tickets(text)`; `ForgeError` (Http/Protocol/Rpc/Json, Error+Display);
  `SprintView { name, number, tickets }`, `TicketView { number, title, status, kind }`, `SprintMeta`.
- **SHIM/adapter** (mutants::skip + cov-excluded — accepted-untestable like `pty_os`): `fetch(endpoint,
  request)` (TcpStream connect/write/read-to-EOF); `ForgeClient { endpoint }` + `current_sprint()`
  composing the sprint-current + ticket-list calls into a `SprintView`.
- `serde` + `serde_json` deps (already in the workspace lock).

### Out
- Writes from the app (read-only). Auth other than `.mcp.json` bearer. TLS (localhost plain HTTP).
  Streaming/pagination. The Forge PANE UI — #64. Reading `.mcp.json` from disk — the app shim does that
  + passes the string in (keeps the crate pure).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions (SECURITY-SENSITIVE)
- D1 (FORK) — the app authenticates by reading the bearer from `.mcp.json` (chad's local, gitignored
  secret); **localhost 127.0.0.1:8080 only; READ-ONLY; the bearer is NEVER logged** (a manual `Debug`
  on `ForgeEndpoint` prints `Bearer ***`, tested).
- D2 (approach A) — NO new HTTP dependency: a raw HTTP/1.1 POST over `std::net::TcpStream` (plain HTTP
  suffices for localhost). Keeps the clean-room minimal-dep ethos; the request-build + response-parse are
  pure/testable, only the socket I/O is the masked adapter.
- D3 — protocol PROBED against the live forge: `tools/call` works with NO initialize handshake; the
  response is SSE (`data: {jsonrpc}`) with `result.content[0].text` = the inner JSON string.
- D4 — NO `Default` (unviable-mutant hygiene, per #54/#55/#61).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `forge_endpoint_from(mcp_json)` sees a `mcpServers.forge` with a `url` + `headers.Authorization`, it shall return the endpoint; when any is absent it shall return `None`. | unit |
| REQ-002 | WHEN `ForgeEndpoint` is formatted with `{:?}`, it shall NOT reveal the bearer (redacted). | unit |
| REQ-003 | WHEN `tool_call_request(endpoint, tool, id)` runs, it shall produce a well-formed HTTP/1.1 POST (path, Host, Authorization, Content-Length matching the JSON-RPC body carrying `tool`). | unit |
| REQ-004 | WHEN `jsonrpc_from_http` sees a non-2xx status it shall `Err(Http)`; with 2xx + no `data:` line, `Err(Protocol)`; with a `data:` line it shall return that JSON. | unit |
| REQ-005 | WHEN `tool_text` sees a JSON-RPC `error` it shall `Err(Rpc)`; a missing result shall `Err(Protocol)`; a valid envelope shall return `content[0].text`. | unit |
| REQ-006 | WHEN `parse_sprint_meta` / `parse_tickets` parse valid forge JSON they shall map to `SprintMeta` / `Vec<TicketView>` (empty list ok); malformed → `Err(Json)`. | unit |
| REQ-007 | `scripts/gates.sh` GREEN, cov/MSI 100 on the pure parts (the adapter masked). | gate |
| REQ-008 (live) | GIVEN a running forge, `current_sprint()` shall return the live sprint. | best-effort integration (documents if forge down) |

## Phase Plan
- **P2** — the crate + the 8 pure fns + the masked adapter, the HTTP/SSE/JSON shapes, mutation targets,
  the unit + live-integration test plan.
- **P3** — the crate (`Cargo.toml` + `lib.rs`).
- **P3.5** — critic: the endpoint parse, **the redacting Debug (no bearer leak)**, the request shape, the
  error paths, the JSON mapping, the masked adapter seam, mutants; SECURITY lens (secret handling).
- **P4** — unit tests (cov/MSI 100) + the best-effort live check + gate GREEN.
- **P5** — docs (crate-map + CHANGELOG + arch), AAR, archive, close #63.
