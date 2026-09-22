---
pipeline_id: 44498681-d80f-443a-9379-280795868bd3
ticket: forge#370 (5b1e4eda-2024-49b4-b981-f7a4f38abbf5) · local docs/planning/tickets/open/TICKET-370-marley-mcp-server-l1.md
aar_id: 309cc4cf-dda6-4a6c-b86e-e083a4f1835c
status: Phase 5 — Complete PASS (shipped 2026-07-21; gate GREEN --diff, 15/15)
title: marley_mcp — the expose-side MCP server (L1) — fleet read slice + session.surface_to_human
type: feature
milestone: M23
references:
  - docs/marley_architecture/orchestration-shell.md
  - docs/marley_architecture/fleet-control-plane.md
  - docs/planning/intake/mcp-first-class-control-plane.md
  - crates/marley_forge_client/src/lib.rs
  - crates/marley_forge_client/src/adapter.rs
  - crates/marley_app/src/headless_drive.rs
---

## Title
NEW crate `marley_mcp` — Marley's own MCP **server** (the expose direction, orchestration-shell.md §4),
so the manager seat — an MCP client, location-independent (§4: hosted inside Marley, in iTerm, or on a
fleet box) — can read the fleet and surface sessions to the human. Layer-1 item ④ of the §12 sequencing:
*insight before control*. One shippable slice: (a) `fleet.snapshot` read tool (the `marley_fleet` types
ARE the schema — one seam, three consumers, §7); (b) the snapshot as a **subscribable MCP resource**
(the same contract shape Forge's push uses, §6 — one contract, N consumers); (c) the ONE gated write,
`session.surface_to_human(id)` — chad's ask verbatim (§5: *"opening new sessions so the user can
view"*), receipted, refused cleanly on unknown id; (d) tool **families** and (e) **permission tiers**
first-class from day one (§10), so the editor/browser families and Layer-2 write verbs arrive without
re-architecture. `headless_drive.rs` proves the whole app surface is already drivable with no window;
`marley_mcp` makes that lane a stable, permissioned, external protocol (§7's crate-map row).

## Scope
### In
- **NEW crate `crates/marley_mcp`** — the server:
  - Pure JSON-RPC 2.0 dispatch for the MCP server subset: `initialize` (capabilities:
    `tools{}`, `resources{subscribe:true}`), `tools/list`, `tools/call`, `resources/list`,
    `resources/read`, `resources/subscribe`, and the outbound `notifications/resources/updated`.
  - **Tool-family registry** — a first-class `(family, verb)` model; L1 ships two families:
    `fleet` (read: `fleet.snapshot`) and `session` (write: `session.surface_to_human`). Adding a
    family is additive (a new registry entry set), never a rework of dispatch/permissions.
  - **Permission tiers** — a pure decision fn: read tools = loose tier (no grant needed); write
    verbs require an explicit per-tool-class grant read from settings (#371's table);
    **deny-by-default** for anything unlisted. Denial = a clean typed refusal.
  - **Transport** — loopback Streamable-HTTP listener (D1): one MCP endpoint, POST for client
    messages, GET for the standing SSE notification stream. Socket accept/read/write loops are
    masked shims; every parse/build/route decision is a pure fn.
- **`marley_fleet` (#367) as the schema seam** — `fleet.snapshot` returns the serialized
  `FleetSnapshot` (`structuredContent` + the back-compat text block); the fleet resource carries the
  same serialization; `session.surface_to_human` uses #367's verb request/receipt types.
- **App wiring (thin, masked)** — `marley_app` hosts the server, feeds it the current snapshot +
  snapshot-change ticks, and executes the surface_to_human effect through the existing shell
  focus/open mechanics (exact wiring is Phase-2 design).
- **Tests** — pure units (cov/MSI 100 on every decision fn); an integration test client speaking real
  HTTP to the loopback server (tools/list, snapshot round-trip, subscribe→notify); headless drives
  (`headless_drive.rs` lane) proving the surface_to_human app-side effect.

### Out (explicitly deferred)
- **Editor / browser tool families** — the registry must make them additive; shipping them is future
  (orchestration-shell §4/§8; the browser family is Phase E).
- **Any additional write verb** — `session.send` / `session.open` / `session.read`, dispatch,
  question-answering = Layer 2 (§12).
- **The manager seat itself** (the client) — out; this ticket only makes Marley reachable by one.
- **Non-loopback exposure** — no 0.0.0.0, no LAN/tunnel topology (mission-control's Tailscale posture
  is a later concern); §10 loopback-only is binding here.
- **#371's settings table itself** — consumed as a contract (the grants this server reads); its
  round-trip ships in the sibling ticket.
- **SSE resumability / `Last-Event-ID` replay** — spec-optional (MAY); unnecessary for L1 because the
  subscribed resource is a *snapshot*: a client that missed a notification re-reads and is current.
- **`resources/templates/list`, pagination, prompts/sampling capabilities** — not needed for a
  two-tool, one-resource surface; the dispatch returns clean method-not-found.

## Reference (§20)
**N/A — Marley-specific; no Warp/Zed behavior analog.** This is control-plane protocol work: Marley
exposing its own MCP server for an orchestrating agent is the orchestration-shell design (§4/§5), not
a Warp terminal/cockpit behavior or a Zed editor behavior to match. The behavior contract being
implemented is the **open MCP specification** (MIT-licensed spec + SDKs — the "LSP-of-M20" posture,
orchestration-shell §13); clean-room §20 is untouched: no Warp (AGPL) or Zed (GPL) source is relevant
or consulted.

### Prior art
1. **Behavior maps / observed captures — N/A (protocol work).** `docs/warp_architecture/` +
   `docs/zed_architecture/` map terminal/editor UX; neither app exposes an "IDE as MCP server"
   surface to map. Stated per the sweep rule rather than skipped.
2. **Published — the MCP specification (2025-06-18), confirmed by fetch 2026-07-20** at
   `modelcontextprotocol.io/specification/2025-06-18/` (`basic/transports`, `server/tools`,
   `server/resources`):
   - **stdio transport:** *"The client launches the MCP server as a subprocess"* — confirmed
     verbatim; the lifecycle is client-owns-server. Decisive for D1 (below).
   - **Streamable HTTP:** *"the server operates as an independent process that can handle multiple
     client connections"*; one MCP endpoint, POST per client message (server replies
     `application/json` or an SSE stream), GET opens the standing SSE stream for server→client
     notifications; sessions via `Mcp-Session-Id`; `MCP-Protocol-Version` header. **Security
     Warning confirmed:** servers **MUST** validate `Origin` (DNS-rebinding), **SHOULD** bind only
     to localhost (127.0.0.1) — never 0.0.0.0 — and **SHOULD** authenticate. Matches §10 verbatim.
   - **Tools:** `tools/list` (name/title/description/`inputSchema`/optional `outputSchema`),
     `tools/call` → `content` + optional `structuredContent` (with `outputSchema`, structured
     results MUST conform); the two-channel error split — protocol errors (unknown tool = JSON-RPC
     `-32602`) vs **tool execution errors** (`isError: true` in the result — the channel for
     business refusals like unknown-session-id and permission denial).
   - **Resources:** capability `resources: {subscribe: true}`; `resources/list`, `resources/read`,
     `resources/subscribe {uri}` → `notifications/resources/updated {uri}`; custom RFC3986 URI
     schemes explicitly allowed (a `fleet://` scheme is legal — the shape Forge's own feed uses,
     orchestration-shell §6 `fleet://seats`); resource-not-found = `-32002`.
   - **Client discovery:** our own `.mcp.json.example` (repo root) is the proof-by-existence — the
     forge server entry is `{"type": "http", "url": "http://127.0.0.1:8080/mcp/forge", "headers":
     {"Authorization": "Bearer …"}}`, i.e. the manager seat's harness (Claude Code) already points
     at loopback Streamable-HTTP servers via exactly the entry shape Marley's server will publish.
3. **OUR DEPS (the load-bearing leg — how Marley speaks MCP as a client today):**
   - **`marley_forge_client` is HAND-ROLLED, not an SDK**: a raw HTTP/1.1 `POST` of JSON-RPC
     `tools/call` over a plain `std::net::TcpStream` to loopback — no rmcp, no async runtime; deps
     are `serde` + `serde_json` only (`crates/marley_forge_client/Cargo.toml`). The two-layer split
     is the house pattern to mirror: PURE request build / response parse / SSE `data:`-line
     extraction at 100% cov/MSI (`src/lib.rs` — `tool_call_request` :157, `jsonrpc_from_http` :189,
     `tool_text` :215, `parse_write_ack` :246, the loopback guard `is_loopback_authority` :132, the
     bearer-redacting `Debug` :29) + a masked live-socket adapter (`src/adapter.rs`, `fetch` :81,
     `Connection: close`, 5s timeouts). The server side needs the same idioms mirrored: request
     PARSE instead of build, response BUILD instead of parse, SSE WRITE instead of read.
   - **No MCP SDK in the tree**: `rmcp` absent from the 764-package `Cargo.lock`; `axum` absent;
     `tokio`/`hyper` present only **transitively** via gpui's http client
     (`gpui → gpui_http_client → zed-reqwest → hyper → tokio`, per `cargo tree -i tokio`) — not a
     runtime any Marley code drives. Adopting `rmcp` (the official Rust SDK, MIT — adoption-eligible
     under §20 leg 3) would add a tokio server runtime + its transport stack to a gpui app. Kept as
     D-OPEN-SDK below with this evidence; Phase 2 may read rmcp's source in the registry (adoption,
     outside the wall) before deciding.
   - **§14 process-spawn confinement** (CONSTITUTION.md §14: spawns live in adapter crates like
     `marley_command`): reinforces D1 — a stdio MCP server exists BY being spawned by its client,
     a posture Marley-the-GUI-app cannot take (evidence in D1).
   - **`headless_drive.rs` is our own prior art for the test lane**: 40+ `#[gpui::test]` drives boot
     the real `RootView` on a TempDir and inject real keystrokes with no window/permission/screen
     (file header + e.g. `cmd_t_adds_a_tab_headless` :238) — the integration lane that proves the
     surface_to_human app-side effect without a headed run.

## Locked-In Decisions
- **D1 — Transport = loopback Streamable-HTTP (LOCKED; the Phase-1 sweep was decisive).**
  Evidence: (1) the spec defines stdio as *client launches the server as a subprocess* — Marley is a
  long-lived GUI app launched by the user, holding the live fleet state and the actual windows;
  a client-spawned "Marley" would be a second, windowless instance, so `surface_to_human` would
  surface a session in an app nobody is looking at — a category error, not a trade-off; (2) stdio
  serves exactly one client per process, but §4's location-independence + §6's "one contract, N
  consumers" require one server serving N concurrent clients (manager seat, hosted agent later,
  human tooling) — the spec says Streamable HTTP is precisely "an independent process that can
  handle multiple client connections"; (3) it is the same transport Forge already exposes, and
  `.mcp.json.example` proves the manager's harness consumes it as-is; (4) §10's binding security
  framing (loopback-only) is written for it. Bindings: **loopback only** (mirror
  `is_loopback_authority`, never 0.0.0.0), **`Origin` validated** on every request, **bearer auth**
  on every request (generation/placement = D-OPEN-LISTEN), bearer never logged (the
  `marley_forge_client` redaction lesson stands).
- **D2 — The `marley_fleet` types ARE the schema (one seam, three consumers).** `fleet.snapshot`
  returns #367's `FleetSnapshot` serialized as `structuredContent` (+ the spec's back-compat
  serialized-JSON text block), with `outputSchema` derived from the same types; the subscribable
  resource serves the identical serialization; `session.surface_to_human` takes/returns #367's verb
  request/receipt types. `marley_mcp` never declares a parallel schema — a drifted hand-written
  schema is the defect class this kills (orchestration-shell §7's crate-map contract).
- **D3 — Subscription = the standard MCP resources contract, not a bespoke channel.** One resource
  (a custom `fleet://` URI — RFC3986-legal per the spec; exact URI fixed in Phase 2 alongside #367
  naming), capability `resources: {subscribe: true}`; on snapshot change the server sends
  `notifications/resources/updated {uri}` on the standing GET/SSE stream and the client re-reads.
  Same contract shape Forge's push uses (§6) — one contract, N consumers; and because the resource
  is a snapshot, missed notifications are self-healing (re-read = current), which is what lets L1
  skip resumability.
- **D4 — Tool families are first-class from day one.** The registry models `(family, verb)` as
  typed data (family = a closed enum; per-family tool tables); dispatch and the permission fn key on
  it. Adding `editor`/`browser` later = adding a variant + its table — additive by construction,
  no re-architecture (REQ-011). The public tool NAME is composed by one pure fn from family + verb;
  the separator charset is a Phase-2 detail (see D-OPEN-SDK note on client tool-name charsets).
- **D5 — Permission tiers day-one, deny-by-default (§10 binding; the intake pillar's model).**
  A pure fn decides every call: read-family tools = loose tier, allowed with zero configuration;
  write verbs = allowed only on an explicit per-tool-class grant read from #371's settings;
  **anything not explicitly listed → deny**. The fn returns typed `Allow`/`Deny(reason)`;
  it never panics and never defaults open.
- **D6 — Error-channel split follows the spec.** Unknown tool / malformed JSON-RPC → protocol error
  (`-32602` etc.). Known tool + business refusal (unknown session id, permission denied) → a
  **tool execution error**: `isError: true` with a typed refusal payload — receipted, clean, never a
  panic. The `parse_write_ack` isError lesson from the client side (a failed write must never read
  as success) applied from the server side.
- **D7 — surface_to_human is the ONLY write in L1**, receipted with a typed result on success and a
  typed refusal on unknown id. Its **v1 UI semantics** (what "focus/open the named session's
  surface" does to project/tab/pane/window state) are decided in Phase 2 per the ticket — the L1
  contract only fixes: known id → the session's surface ends up focused/visible + receipt; unknown
  id → refusal, no app-state change.
- **D8 — Testing doctrine (constitution §7/§0).** Server plumbing shims (listener accept loop, SSE
  write loop, the app-side glue) are masked `#[cfg_attr(test, mutants::skip)]`; every decision fn —
  request parse, JSON-RPC route, registry lookup, permission check, id resolution, snapshot
  serialization, response/notification build — is pure at cov/MSI 100. Integration: a test client
  over a real loopback socket; the app-side effect: headless drives.

### Open decisions (Phase 2)
- **D-OPEN-SDK — hand-rolled server vs adopting `rmcp` (MIT).** Phase-1 evidence: no rmcp/axum in
  the tree; tokio only transitive; the client precedent is a small hand-rolled pure+shim two-layer
  whose idioms mirror onto the server; but the server half (sessions, standing SSE, lifecycle) is
  more spec surface than the client's one-shot POSTs, and rmcp owns exactly that machinery —
  adoption is §20-sanctioned. Phase 2 decides by measuring: rmcp's server-transport dependency
  delta (does streamable-http-server pull axum/tokio-full?), runtime coexistence with gpui's
  executor (a second async runtime on a background thread), and how much of rmcp's surface L1
  actually needs vs the bounded hand-rolled subset (initialize + 5 methods + 1 notification).
  Also settled here: the tool-name separator (`.` vs `_` for `fleet.snapshot`) — verify against
  real client bridging (model-side tool-name charsets commonly exclude `.`; the registry keeps
  family/verb as separate typed fields either way, so the join is one pure fn whichever wins).
- **D-OPEN-LISTEN — listen/auth config + discovery.** Fixed port vs ephemeral-port-advertised; where
  the generated bearer + URL surface so the manager's `.mcp.json` can point at Marley (a #371
  settings field vs a runtime-written file); single- vs per-client session granularity
  (`Mcp-Session-Id`). Constraints already fixed by D1: loopback bind, Origin check, bearer required,
  bearer never logged.
- **D-OPEN-SURFACE-V1 — surface_to_human v1 UI semantics** (ticket decree: Phase 2). Candidate
  shapes: activate the owning project + tab; open a viewer tab/pane for a not-yet-rendered session;
  window-front behavior. Bounded by D7's contract either way.

### Planner sharpenings (Phase 1 review — attacking the draft's confident sentences)
- **S1 — the #370↔#371 coupling is a build-order TRAP; break it by ownership.** D5/Out phrase the grants
  as "read from #371's settings," which reads as a mutual dependency (each ticket needs the other's type).
  Resolution the design MUST honor: **`marley_mcp` OWNS the permission decision AND its input type** (a
  `GrantTable` value / `&[tool-class]` — marley_mcp's type); #371's `[[mcp.*]]` round-trip merely
  DESERIALIZES INTO it. So #370 compiles, ships, and proves REQ-006/007/008 with **fixture grants**, with
  ZERO build-order dependency on #371; #371 conforms to marley_mcp's grant contract, not the reverse. This
  also means #370 can land BEFORE #371 (the numeric/④→⑤ order holds without a deadlock).
- **S2 — the integration lane rides an IN-MEMORY fake transport, not a bound port (Phase-4 risk).** A
  real-loopback-port test is flake-prone and may be gate-excluded (§0 bound-port exclusion); the house
  precedent is to MASK the socket (marley_forge_client) or use a fake child + pipes (marley_lsp). So the
  contract carrier is a **trait-abstracted transport with an in-memory fake** — feed request bytes, capture
  the response + the `notifications/resources/updated` bytes — exercising the FULL parse→route→permission
  →build→notify path with no OS socket. Phase 2 fixes that transport trait so the fake is first-class; a
  real-port smoke is an OPTIONAL extra, never the sole proof of a REQ.
- **S3 — Phase-1 discovery CONFIRMED the draft's load-bearing claims** (independent sweep, this review):
  `marley_forge_client`'s client idioms mirror cleanly to the server (`jsonrpc_from_http`/`tool_text`/
  `parse_write_ack` ↔ parse-request / build-response / write-SSE; `is_loopback_authority` reused verbatim);
  NO `rmcp`/`axum` in the tree (tokio only transitive); the settings macro (`RemoteHosts`/`Workflows`/
  `lsp.servers`, settings.rs:72/77/81) IS #371's round-trip pattern; and #367's `SurfaceRequest { id }` +
  `Receipt<T>` = `Accepted{value}`/`Refused{reason}` (verbs.rs:55/64) EXIST and ground D2/D7 — a `Refused`
  receipt is the first-class outcome that D6 maps to `isError: true`, no new type needed.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a client sends `tools/list`, the server shall return exactly the L1 tool set — `fleet.snapshot` and `session.surface_to_human` (wire names per D4) — each with an `inputSchema` and an `outputSchema`. | integration (test client → loopback server → assert the exact two-tool list) + pure registry unit test |
| REQ-002 | WHEN a client calls `fleet.snapshot`, the server shall return the CURRENT `FleetSnapshot` as `structuredContent` (+ the back-compat text block) conforming to the #367 types. | integration vs a test client (seed a known snapshot → call → assert the round-tripped structure equals the seed) |
| REQ-003 | WHEN a client has subscribed to the fleet resource AND the snapshot changes, the server shall send `notifications/resources/updated` carrying that resource's URI on the client's standing stream. | integration (subscribe → mutate the snapshot source → assert the notification arrives; re-read returns the new snapshot) |
| REQ-004 | WHEN `session.surface_to_human` is called with a KNOWN session id by a granted caller, the server shall return a typed success receipt AND the app shall focus/open that session's surface. | headless drive (seed a session → invoke the verb path → assert the shell's focused surface) + pure receipt-build unit test |
| REQ-005 | WHEN `session.surface_to_human` is called with an UNKNOWN session id, the server shall return a typed refusal as a tool execution error (`isError: true`) — never a panic, never a protocol-level crash, and no app-state change. | pure id-resolution fn test (unknown → typed refusal) + integration (call with a bogus id → assert `isError` refusal + unchanged state) |
| REQ-006 | WHEN a WRITE tool is called and settings carry NO grant for its tool class, the permission fn shall return Deny and the server shall refuse the call (deny-by-default), naming the missing grant in the typed refusal. | pure permission fn test (empty + unrelated grants → Deny) + integration (ungranted call → `isError` refusal) |
| REQ-007 | WHEN a WRITE tool is called and settings carry an explicit grant for its tool class, the permission fn shall return Allow and the call shall proceed to execution. | pure permission fn test (grant present → Allow) + integration (granted call → success receipt) |
| REQ-008 | WHEN a READ-family tool (`fleet.snapshot`) is called with zero grants configured, the server shall proceed (the read tier requires no grant). | pure permission fn test (read tier → Allow with no grants) + integration |
| REQ-009 | WHEN the server starts, it shall bind a loopback address ONLY (never 0.0.0.0), and WHEN a request carries a disallowed `Origin` or a missing/wrong bearer, the server shall refuse it before dispatch. | pure fn tests (bind-addr constructor; Origin/auth check across allowed/disallowed/missing) + integration (reject smoke) |
| REQ-010 | WHEN a request names an UNKNOWN method or tool, the dispatch shall return the spec's protocol error (e.g. `-32602` unknown tool) rather than panicking or hanging the connection. | pure dispatch fn tests (unknown method/tool → typed protocol error) |
| REQ-011 | WHERE a new tool FAMILY is added, the change shall be additive (a new family variant + its tool table); dispatch, permissions, and existing families shall compile and behave unchanged. | type-level claim: §18.1 inspect review of the registry shape (+ a trybuild compile fixture if Phase 2 pins a compile-shape); regression = the full existing suite green after the L1 second family (`session`) proves the pattern once |

## Testing boundary (honest)
The listener/SSE/socket loops and the app-side glue are ACCEPTED-UNTESTABLE masked shims (the
`marley_forge_client` adapter precedent — decisions live above the shim). The integration lane binds
a REAL loopback port in-test; if the gate environment cannot (a bound-port exclusion per §0), the
pure dispatch + a stream-abstracted fake carry the contract and the exclusion is documented, not
hidden. surface_to_human's app-side effect rides the headless drive lane (no window needed);
pixel-level proof is out of scope (gate-15's domain, none needed here).

## Phase Plan
- **P2 Design** — settle D-OPEN-SDK (registry read of rmcp permitted — adoption), D-OPEN-LISTEN,
  D-OPEN-SURFACE-V1; fix the exact #367 type/serialization signatures + #371 grant-key names with
  the sibling pipelines; module map (pure core / masked transport / app glue); the regression test
  plan per REQ row.
- **P3 Implement** — the crate + wiring per design; pure seams first.
- **P3.5 Inspect** — adversarial: deny-by-default actually default? bearer/Origin checked BEFORE
  dispatch? refusals typed on every arm? registry genuinely additive? no Forge-specific string in
  `marley_mcp` (§2's agnosticism defect test)? provenance check (§20).
- **P4 Validate** — write + RUN the units, integration client, headless drives; gate green
  (cov/MSI 100 on the pure seams).
- **P5 Complete** — CHANGELOG + architecture docs (orchestration-shell §7 crate row → shipped;
  a `marley_mcp.md` component doc); AAR capture; archive; close #370.
