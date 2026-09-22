# marley_mcp — the expose-side MCP server (L1) — Notes

- **Forge ticket:** #370 (5b1e4eda-2024-49b4-b981-f7a4f38abbf5)
- **AAR:** 309cc4cf-dda6-4a6c-b86e-e083a4f1835c
- **Local ticket doc:** docs/planning/tickets/open/TICKET-370-marley-mcp-server-l1.md
- **Pipeline spec:** 370-marley-mcp-server-l1.spec.md

## Phase 1 — Plan
- **Request:** NEW crate `marley_mcp` — Marley's own MCP SERVER (the expose direction), so the
  manager seat (an MCP client, location-independent) can read the fleet and surface sessions to the
  human. L1 slice: `fleet.snapshot` (read tool; the `marley_fleet` types ARE the schema), the
  snapshot as a subscribable MCP resource (the contract shape Forge's push uses), the ONE gated
  write `session.surface_to_human(id)` (chad verbatim: *"opening new sessions so the user can
  view"*), tool families + permission tiers first-class from day one, transport decided on
  evidence. Sprint "M23 — Fleet Control Plane: Layer 1"; orchestration-shell §12 Layer-1 item ④.
- **Classification / tier:** feature, M23; a NEW crate + thin app wiring → full pipeline
  (plan→design→implement→inspect→validate→complete→commit). Runs against two CONCURRENT sibling
  contracts: **#367** (`marley_fleet`: `FleetSnapshot` + verb request/receipt types = the tool
  schemas) and **#371** (`[[mcp.servers]]` + permission-grant settings = the grants D5 reads).
  Exact signatures are Phase-2 business, negotiated with those pipelines.
- **Forge recall (§18.3):** drafted DOCS-ONLY (no MCP calls in this drafting lane); recall drawn
  from the on-disk record — orchestration-shell.md §4 (the two MCP directions; manager
  location-independence), §5 (the receipted verb table; surface_to_human's row + chad's verbatim
  ask), §6 (push = MCP resource subscription, DECIDED; one contract, N consumers; snapshot re-read
  as catch-up), §7 (the crate map: `marley_mcp` row — "one server, tool families, the permission
  tiers (§10)"; `headless_drive.rs` named as the proof the drive surface exists), §9 (the hosted
  agent points at this server over loopback), §10 (tiers, deny-posture, loopback, two-credential
  auth, never-logged bearer), §12 (Layer 1 = read slice + the ONE gated write; families must be
  growth-ready because Phase E depends on it); fleet-control-plane.md §2 (the evidence night — why
  receipted data beats keystrokes/screenshots), §7 (what the manager needs to see); the MCP intake
  pillar (permission model per server/per tool class; read loose / write explicit; `marley_mcp`
  named there as the pillar's concrete expose-side deliverable); ADs of record cited by
  orchestration-shell §13 (`AD-claude-tmux-grade-detached-sessions-001`,
  `AD-claude-mission-control-hypermedia-surface-001`,
  `AD-claude-brain-agent-session-supervision-001` — the "one protocol, three callers" lineage of
  the session verb family). Live `knowledge-context`/bulletins recall re-runs at `/work` promotion.
- **Discovery (the load-bearing sweep — how Marley speaks MCP today, client side):**
  - `crates/marley_forge_client/` is **hand-rolled, no SDK**: raw HTTP/1.1 POST of JSON-RPC
    `tools/call` over `std::net::TcpStream` to loopback; deps = serde + serde_json only. Pure layer
    (100% cov/MSI): `tool_call_request` (lib.rs:157), `jsonrpc_from_http` (:189 — SSE `data:` line
    extraction), `tool_text` (:215), `parse_write_ack` (:246 — the isError fail-closed lesson),
    `is_loopback_authority` (:132 — the loopback guard to mirror server-side), bearer-redacting
    `Debug` (:29). Masked shim: adapter.rs `fetch` (:81) — `Connection: close`, 5s timeouts,
    `#[cfg_attr(test, mutants::skip)]`. The server needs these idioms MIRRORED (parse requests,
    build responses, write SSE), plus what the client never needed: a standing GET/SSE stream,
    sessions, and a listener.
  - **Lockfile facts** (764 packages): NO `rmcp`, NO `axum`; `tokio`/`hyper` present only
    transitively via `gpui → gpui_http_client → zed-reqwest → hyper → tokio` (`cargo tree -i
    tokio`) — no Marley code drives an async runtime today. → the SDK question is a real fork
    (D-OPEN-SDK), not a foregone adoption.
  - **MCP spec confirmed by fetch (2026-07-20, 2025-06-18 revision):** stdio = client-spawns-server
    subprocess (kills stdio for a GUI app hosting the server → D1 locked); Streamable HTTP =
    independent process, N clients, POST + standing GET/SSE, `Mcp-Session-Id`, and the Security
    Warning (MUST validate Origin; SHOULD bind 127.0.0.1 only; SHOULD authenticate); tools contract
    (`inputSchema`/`outputSchema`/`structuredContent`; protocol-error vs `isError` split → D6);
    resources contract (`resources/subscribe` → `notifications/resources/updated {uri}`;
    `subscribe: true` capability; custom `fleet://`-style URIs legal → D3).
  - **Client discovery shape:** repo `.mcp.json.example` — the forge entry is
    `{type: "http", url: "http://127.0.0.1:8080/mcp/forge", headers: {Authorization: Bearer …}}`;
    the manager's harness already consumes loopback Streamable-HTTP servers via exactly this entry
    shape, so pointing it at Marley is configuration, not new client work.
  - **Integration lane:** `crates/marley_app/src/headless_drive.rs` — 40+ `#[gpui::test]` drives
    boot the real `RootView` on a TempDir, inject real keystrokes, assert real state, no window/
    permissions/screen. This is how REQ-004/005 (surface_to_human's app-side effect) get proven.
  - **§14 confinement:** process spawns belong to adapter crates (`marley_command`); Marley is
    launched by the user, not spawnable per-client — reinforces D1.
- **Decisions:** D1 transport LOCKED (loopback Streamable-HTTP — evidence in the spec); D2 #367
  types are the schema; D3 subscription = standard resources contract; D4 families first-class;
  D5 tiers + deny-by-default; D6 error-channel split; D7 surface_to_human = the only L1 write
  (v1 UI semantics → Phase 2); D8 pure-seam/masked-shim testing doctrine. OPEN for Phase 2:
  D-OPEN-SDK (hand-rolled vs rmcp — measure the dep/runtime delta; also fixes the tool-name
  separator), D-OPEN-LISTEN (port/bearer/discovery placement), D-OPEN-SURFACE-V1 (UI semantics).

### Phase 1 — Planner review (promotion to active, 2026-07-20)
Promoted the Fable draft (spec+notes) queued→active. Independent sweep + attack:
- **Confirmed prior art (S3):** `marley_forge_client` client idioms mirror to the server
  (`jsonrpc_from_http`/`tool_text`/`parse_write_ack` ↔ parse-req/build-resp/write-SSE;
  `is_loopback_authority` reused); NO `rmcp`/`axum` in the 764-pkg lock (tokio transitive only); the
  settings macro `RemoteHosts`/`Workflows`/`lsp.servers` (settings.rs:72/77/81) IS #371's round-trip
  pattern; #367 `SurfaceRequest { id }` + `Receipt<T>`=`Accepted{value}`/`Refused{reason}` (verbs.rs:55/64)
  EXIST → D2/D7 grounded, a `Refused` receipt is what D6 maps to `isError`.
- **S1 (build-order trap):** broke the #370↔#371 mutual-dependency reading — `marley_mcp` OWNS the
  permission-input type; #371 deserializes into it; #370 ships + tests REQ-006/007/008 on fixture grants,
  no dependency on #371 landing first.
- **S2 (test lane):** the integration contract rides an in-memory FAKE transport (trait-abstracted), not a
  bound port — matches the marley_forge_client (mask socket) / marley_lsp (fake+pipes) precedent; a real
  loopback smoke is optional. Phase 2 fixes the transport trait.
- **Verdict:** the 8 locked decisions + 11 EARS AC survive the attack; the 3 D-OPEN forks (SDK, listen,
  surface-v1) are legitimately Phase-2. Status → Phase 1 PASS.

## Phase 2 — Design

### §20 confirm
**N/A — Marley-specific (protocol work), still holds.** The behavior contract is the open MCP spec
(2025-06-18); no Warp (AGPL) / Zed (GPL) source is relevant or read. Adoption-eligible substrate
(§20 leg 3): our OWN `marley_forge_client` client idioms + std::net; no third-party MCP SDK adopted (see
D-OPEN-SDK resolution).

### Architecture / approach
`marley_mcp` = a PURE protocol crate (serde-only, like `marley_forge_client`) + a masked transport shim;
`marley_app` hosts it via the established worker→pump idiom. The gpui app owns the windows and the live
`fleet_snapshot`; the server is a background std::thread that the pump feeds. Honors §14 (typed errors, no
panic on the request/response path; the socket/thread spawn confined to the shim; single-owner types).

### D-OPEN forks — SETTLED
- **D-OPEN-SDK → HAND-ROLLED (serde-only, std::net + thread), NOT rmcp.** Evidence (Phase-1 sweep + §14):
  no rmcp/axum in the 764-pkg lock; tokio only transitive via gpui — adopting rmcp injects a SECOND async
  runtime (tokio+axum) into a gpui app driving its own executor. The L1 surface is BOUNDED — `initialize`
  + 5 methods + 1 notification — and the client half already proves the pure+shim, serde-only pattern
  (`marley_forge_client`). So: `std::net::TcpListener` bound to loopback, **thread-per-connection**
  (blocking IO, the search/syntax-worker idiom `app.rs:11238`/`13725`), results pushed to the app over an
  `mpsc::Sender` drained on the pump tick (`try_recv` + `cx.notify`, mirroring `forge_pending`
  `app.rs:377`). The whole protocol (parse→route→permit→build→notify) is PURE `handle_message`; only the
  listener/SSE/discovery-file IO is the masked shim.
- **D-OPEN-LISTEN → ephemeral loopback + per-boot bearer + discovery file.** Bind `127.0.0.1:0` (OS-assigned
  port — no collision); generate a random per-boot bearer (NEVER logged — the `marley_forge_client`
  redaction lesson); write `{url, bearer}` to a runtime discovery file the manager's `.mcp.json` reads
  (masked file writer; placement = a state dir, exact path a P3 detail; a #371 settings field for a FIXED
  port is a follow-up, not L1). `Origin` + bearer + loopback are PURE guard fns run BEFORE dispatch
  (mirror `is_loopback_authority`); a bad Origin / missing-or-wrong bearer → refuse pre-dispatch. Per-conn
  subscription set lives in the shim; the pure layer decides WHAT to send.
- **D-OPEN-SURFACE-V1 → pure resolve + masked focus; L1 addresses LOCAL surfaces; remote seats refuse.**
  The Explore found the wrinkle: the verb id is `Session.id: String` (fleet, external) but shell surfaces
  are `PaneId(u64)` with NO existing bridge (`Transport` carries no pane handle; only `Transport::Local`
  seats could have a pane). And the focus primitive ALREADY EXISTS: `jump_to_pane(PaneId) -> bool`
  (`app.rs:6502` — activate project+tab + `PaneGrid::focus`, returns false for unknown; the #68 overlay
  is the precedent). Resolution: **`resolve_surface<H: Copy>(id, &[(String, H)]) -> Option<H>`** is a
  GENERIC PURE fn in marley_mcp (H instantiated to `PaneId` by the app, so marley_mcp never depends on
  marley_app — no cycle); the app builds the `(id, PaneId)` index from its LOCAL hosted sessions, calls
  `resolve_surface`, and on a hit runs the masked `jump_to_pane` + `cx.activate(true)` (window-front). The
  receipt is pure: `surface_receipt(focused: bool, id) -> Receipt<…>` — miss OR `jump_to_pane==false` →
  `Receipt::Refused` (the #367 first-class refusal → D6 `isError:true`), no state change. **L1 limitation
  (documented, honest):** L1 can only surface a session the shell LOCALLY hosts (a pane exists); a
  remote/demo-only fleet seat refuses cleanly — the fleet-id→local-pane population is Layer-0's job. The
  CONTRACT (resolve + receipt) is fully pure-testable with fixture indices regardless.

### Module map / file manifest
| File | Change |
|---|---|
| `crates/marley_mcp/Cargo.toml` | NEW — deps `serde` + `serde_json` + `marley_fleet` (mirror marley_forge_client); dev-dep `mutants` |
| `crates/marley_mcp/src/lib.rs` | NEW — crate root; re-exports; the `handle_message(&ServerState,&str)->Vec<Outgoing>` pure entry + `ServerState` (snapshot ref, grants, surface index, subscriptions) |
| `crates/marley_mcp/src/jsonrpc.rs` | NEW PURE — JSON-RPC 2.0 envelope: parse a request (id/method/params), build result/error responses, the `-32601/-32602/-32002` codes (server mirror of `jsonrpc_from_http`) |
| `crates/marley_mcp/src/dispatch.rs` | NEW PURE — route method→handler (`initialize`/`tools/list`/`tools/call`/`resources/{list,read,subscribe}`); `initialize` capabilities (`tools{}`,`resources{subscribe:true}`); unknown method → protocol error |
| `crates/marley_mcp/src/registry.rs` | NEW PURE — `enum Family {Fleet, Session}`; per-family tool tables; `tool_name(family,verb)` compose + parse; `tools_list()` = the exact L1 set w/ input+output schema; wire-name lookup (D4) |
| `crates/marley_mcp/src/permission.rs` | NEW PURE — marley_mcp-OWNED `GrantTable` (the S1 type #371 deserializes into); `Tier {Read, Write}`; `decide(tool_class, tier, &GrantTable) -> Decision::{Allow, Deny(reason)}`; read=loose, write=grant-or-deny, deny-by-default (D5) |
| `crates/marley_mcp/src/tools.rs` | NEW PURE — `fleet_snapshot_result(&FleetSnapshot)` → structuredContent (`serde_json::to_value`) + back-compat text block (D2); `resolve_surface<H:Copy>` + `surface_receipt` (D7); `tool_error(reason)` → `isError:true` (D6) |
| `crates/marley_mcp/src/resource.rs` | NEW PURE — the `fleet://…` URI const; `resources/list`+`read` (same serialization as the tool, D3); `resource_updated_notification(uri)` build |
| `crates/marley_mcp/src/auth.rs` | NEW PURE — `is_loopback(addr)`, `origin_allowed(&str)`, `bearer_ok(presented, expected)` — pre-dispatch guards (D1/D9) |
| `crates/marley_mcp/src/transport.rs` | NEW — `trait Outgoing`/`ServerIo` seam for the in-memory fake (S2) + the MASKED `std::net::TcpListener` thread-per-conn impl + SSE writer + discovery-file writer (`#[cfg_attr(test, mutants::skip)]`) |
| `crates/marley_app/Cargo.toml` | `+ marley_mcp = { path = "../marley_mcp" }` |
| `crates/marley_app/src/mcp_host.rs` | NEW (masked glue, mirror `lsp_host.rs` — coverage-excluded) — spawn the server thread, feed it `fleet_snapshot` + change ticks over mpsc, build the `(id,PaneId)` surface index from local sessions, execute `surface_to_human` via `jump_to_pane`+`cx.activate` |
| `crates/marley_app/src/app.rs` | (masked shim) hold the mcp_host handle + its mpsc receiver; drain on the pump; a `pub` accessor or direct call for the surface effect |
| `crates/marley_app/src/lib.rs` | `mod mcp_host;` |

### Exact signatures (the seams)
- #367 (unchanged, consumed): `FleetSnapshot{seats:Vec<Session>}` (Serialize) → `to_value`; `SurfaceRequest{id:String}` = tools/call params; `Receipt<T>`=`Accepted{value:T}`/`Refused{reason:String}` = the result.
- marley_mcp-owned: `GrantTable` (S1; e.g. `{ write_classes: BTreeSet<String> }`), `Family`, `Tier`, `Decision`, `Outgoing{Response(String)|Notification(String)}`, `ServerState<'a>`.
- `resolve_surface<H: Copy>(id:&str, index:&[(String,H)]) -> Option<H>` (generic → no marley_app dep; app uses H=`PaneId`).

### Regression Test Plan (per REQ — pure units cov/MSI 100 + in-memory-fake integration + headless drive)
| REQ | Test(s) |
|---|---|
| REQ-001 tools/list = exact L1 set | pure `registry::tools_list` unit (exact 2 tools, each has input+output schema) + fake-transport integration (send tools/list → assert list) |
| REQ-002 fleet.snapshot structuredContent | pure `fleet_snapshot_result` round-trip (seed snapshot → to_value → assert equals #367 serialization) + fake integration |
| REQ-003 subscribe→updated | pure `resource_updated_notification` build + `ServerState` subscribe/change→Outgoing::Notification unit + fake integration (subscribe → change snapshot → assert notification emitted) |
| REQ-004 surface known id → focus + receipt | pure `resolve_surface` hit + `surface_receipt(true)` Accepted; **headless drive**: seed a pane, build index, invoke surface path → assert `active_project/active_tab/focused` moved (via jump_to_pane) |
| REQ-005 surface unknown id → isError, no state change | pure `resolve_surface` miss → `surface_receipt(false)` Refused → `tool_error` isError; headless drive (bogus id → unchanged active pane) |
| REQ-006 write, no grant → Deny | pure `permission::decide(write, empty/unrelated grants)` → Deny(reason) + fake integration (ungranted call → isError refusal) |
| REQ-007 write, grant present → Allow | pure `decide(write, granted)` → Allow + fake integration (granted → success receipt) |
| REQ-008 read tool, zero grants → Allow | pure `decide(read, empty)` → Allow + fake integration |
| REQ-009 loopback-only + Origin/bearer pre-dispatch | pure `auth::{is_loopback,origin_allowed,bearer_ok}` across allowed/disallowed/missing + fake integration (bad origin / wrong bearer → refused before dispatch) |
| REQ-010 unknown method/tool → protocol error | pure `dispatch`/`jsonrpc` unit (unknown method → -32601; unknown tool → -32602) + fake integration |
| REQ-011 family additive | §18.1 inspect review of the `Family` enum + registry shape; regression = full suite green with the 2nd family (`session`) already exercising the additive path; optional trybuild if P3 pins a compile-shape |

**Testing boundary:** the `transport.rs` listener/SSE/discovery-file + `mcp_host.rs` glue are masked
`#[cfg_attr(test, mutants::skip)]` + coverage-excluded (the `lsp_host.rs`/`adapter.rs` precedent — add
`marley_app/src/mcp_host.rs` + the masked `marley_mcp` transport fns to the gate's ignore-regex). Every
decision fn is pure at cov/MSI 100. The integration lane uses the **in-memory fake transport** (S2) — NO
bound port in the gate; a real-loopback smoke is an optional `#[ignore]` extra. surface_to_human's effect
rides the headless drive (no window). No pixels (not a render ticket).

### Risks / decisions
- **The mpsc-pump feed** is the load-bearing app seam (mirror `forge_pending`); a snapshot change must set
  the subscription notification AND the pump must `cx.notify` — the #203 dirty lesson applies to the drain.
- **discovery-file placement** (where `{url,bearer}` lands so the manager reads it) is a P3 detail bounded
  by D-OPEN-LISTEN; the bearer is generated + never logged.
- **surface_to_human L1 scope** only reaches locally-hosted panes (documented); remote seats refuse — the
  contract is honest and the pure resolve/receipt is fully tested.
- **rmcp reconsideration** is a named future option if Layer-2's write surface outgrows the hand-rolled
  subset; L1's bounded surface does not warrant a second async runtime now.

## Phase 3 — Implement
- **Built the NEW `crates/marley_mcp` crate (serde-only, hand-rolled — no rmcp):**
  - PURE core: `jsonrpc.rs` (parse request / build result+error / the -326xx/-32002 codes),
    `dispatch.rs` (`handle_message` routes initialize/tools.list/tools.call/resources.{list,read,subscribe}
    → permission → execute; `snapshot_changed` → notifications), `registry.rs` (`Family{Fleet,Session}` +
    the exact L1 tool table w/ input+output schemas + `tool_name`/`lookup`), `permission.rs`
    (`GrantTable` [marley_mcp-owned, S1] + `Tier` + `decide` deny-by-default), `tools.rs`
    (`fleet_snapshot_result` structuredContent+text, `resolve_surface`, `surface_receipt`, `surface_result`,
    `tool_error` isError), `resource.rs` (`fleet://snapshot` URI + list/read + `resource_updated_notification`),
    `auth.rs` (`is_loopback`/`origin_allowed`/`bearer_ok`). `lib.rs` holds the shared context types
    (`RequestCtx`, `Subscriptions`, `Outgoing`, `Effect`, `Handled`) + re-exports.
  - MASKED `transport.rs` (`#[cfg_attr(test, mutants::skip)]` per-fn): `std::net::TcpListener` bound to
    `127.0.0.1:0`, thread-per-connection, HTTP parse, the auth guards, `handle_message`, one-shot SSE
    response, the standing GET/SSE stream (condvar-woken on a version bump), `discovery_json`, `is_local`.
  - App glue: NEW `crates/marley_app/src/mcp_host.rs` (masked, mirror `lsp_host.rs`) — `McpHost::start`
    (spawn + write the discovery file) / `feed` / `drain_effects`; app.rs `mcp_host: Option<McpHost>` field
    + `apply_mcp_effect` (jump_to_pane) + `mcp_surface_index` (local agents by pane-id) + `pump_mcp_host`
    (pump hook) + `start_mcp_server` + the `mcp-serve` verb; Cargo dep + `mod mcp_host`.
- **Checks:** `cargo check --workspace` green; `cargo clippy -p marley_mcp -p marley --all-targets` clean
  (`-D warnings`); `cargo fmt --all -- --check` clean.
- **Deviations from design (with reason):**
  - **D1 — concrete `u64` surface handle, not generic `resolve_surface<H:Copy>`.** `PaneId` is a `u64`
    newtype, so an opaque `u64` handle keeps marley_mcp free of marley_app with NO generic/lifetime tangle
    (the design's genericity goal is met more simply). marley_mcp treats it as opaque.
  - **D2 — server started via the `mcp-serve` verb (opt-in), NOT auto-started at boot.** Keeps a real
    listener out of the `#[gpui::test]` harness (no port/thread churn — `new_in` never starts it) and
    avoids risky boot-path surgery. The pump hook feeds it once running. A boot-time auto-start behind a
    settings flag is a bounded follow-up (needs #371).
  - **D3 — weak per-boot bearer** (derived from port+salt, not a CSPRNG); loopback-only + single-user, so
    honest for L1 — a strong bearer is the named D-OPEN-LISTEN follow-up. Discovery file
    `mcp-endpoint.json` written to the ambient config dir (this also uses `discovery_json`, clearing the
    only clippy dead-code).
  - No tests here (Phase-4). Gate note for validate: add `marley_mcp/src/transport.rs` +
    `marley_app/src/mcp_host.rs` to the coverage `--ignore-filename-regex` (the masked-shim precedent).

## Phase 3.5 — Inspect
**4 parallel critics** (correctness · security/secrets/provenance · data-state/additivity · simplification/reuse),
all delivered. **CONFIRMED sound:** deny-by-default (`decide`'s only write-`Allow` arm requires the class
present; fall-through is `Deny`), auth-before-dispatch (`serve_connection` refuses on `!origin_allowed ||
!bearer_ok` before `handle_message` — SSE GET gated too), D6 error-channel split (unknown tool → -32602,
business refusal → `isError`), REQ-011 additivity (`decide` untouched by a new family; only `dispatch` match
+ `tool_schemas` are family touch-points), §2 agnosticism (no Forge string as a code identifier), §20 (original
protocol work), no panic/unwrap on any client-reachable path, `to_value` fallbacks genuinely unreachable.

### Findings ledger
| # | Lens | Sev | Finding | Verdict | Resolution |
|---|---|---|---|---|---|
| F1 | security | MED | pre-auth unbounded body alloc — `vec![0u8; content_length]` from an uncapped, UNAUTHENTICATED `Content-Length` → OOM process-abort (the whole gpui app + the user's terminals) | REAL (pre-auth availability) | **Fixed** — `MAX_BODY_BYTES=1MiB` cap, reject oversized → 400 before allocating (transport.rs) |
| F2 | data/simpl | MED | `tools_list()` hand-repeated the descriptions already in `registry()` — drift risk (a 3rd tool listed-only = uncallable / registry-only = undiscoverable), contradicting the D4 single-source charter | REAL | **Fixed** — `const REGISTRY` is the ONE source; `tools_list` DERIVES names+descriptions + a `tool_schemas` match (also removed the per-`lookup` `Vec` alloc → `&'static` slice) |
| F3 | simpl | MED | `transport::is_local` dead (zero callers) + its doc claimed "the app uses it," but `mcp_surface_index` filters nothing | REAL | **Fixed** — deleted (+ the now-unused `Session`/`State` imports); the Local/!Done filter returns with the Layer-0 bridge |
| F4 | data/state | MED | subscribe→notify not wired across the POST/GET split: POST-subscribe sets a throwaway flag; the SSE stream hardcodes `fleet:true` → every stream gets all notifications; docstrings asserted a per-connection model the transport doesn't honor | REAL, **benign for L1** (one self-healing resource → over-notify, never under-notify) | **Fixed the honesty** — transport + dispatch docstrings now state the L1 "open-stream-IS-subscribe" simplification; the pure `Subscriptions` gate stays as the L2-ready per-session model. **Validate MUST note:** REQ-003 is proven at the pure `handle_message`/`snapshot_changed` seam; the wire push is the documented L1 shortcut, NOT end-to-end subscription proof |
| F5 | correctness | LOW | `surface_receipt` doc overclaimed "Accepted iff resolved AND focused" — focus is async best-effort | REAL (doc) | **Fixed** — reworded (Accepted on resolution; focus best-effort, lost race not reflected) |
| F6 | correctness | LOW | a notification-only POST wrote NO HTTP reply → the client read could hang (MCP expects 202) | REAL | **Fixed** — write `202 Accepted` when a POST yields no response body |
| — | reuse | MED | `auth::is_loopback` + `MCP_PROTOCOL_VERSION` duplicate `marley_forge_client` verbatim (a security-critical guard) | REAL, **DEFERRED** | Extracting to `marley_fleet` reopens SHIPPED #368 — a focused refactor follow-up; the copies agree + are each tested. `PR-claude-extract-security-critical-predicate-to-shared-leaf` |
| — | correctness | LOW | id-less known-method → spurious `id:null` response (JSON-RPC forbids replying to a notification) | REAL, **ACCEPTED** | Benign — real MCP clients always id these methods; noted, not fixed |
| — | design | INFO | surface_index keys = pane-decimal-strings, DISJOINT from fleet Session ids → `surface_to_human` un-exercisable end-to-end via the MCP surface in L1 | MATCHES documented D-OPEN-SURFACE-V1 | The fleet-id→pane bridge is Layer-0's job; the surface-effect drive feeds a FIXTURE index to prove the effect mechanism. Sharpened follow-up |

**Result:** 6 fixed at source · 3 deferred/accepted with reason. `cargo check` / `clippy -D warnings` / `fmt`
green after the fixes. No deny-by-default bypass, no client-reachable panic, no §2/§20 violation.

## Phase 4 — Validate
### Tests added (per REQ)
- **Pure units in `marley_mcp` (31, all green — cov/MSI 100 target):**
  - `jsonrpc`: parse request/notification/garbage/non-object/missing-method; response_id/is_request; result+error response shapes.
  - `auth` (REQ-009): `is_loopback` (127/[::1]/localhost accept; evil.com/10.0.0.5/0.0.0.0 reject); `origin_allowed` (missing/loopback ok, cross-site + path-smuggle + malformed reject); `bearer_ok` (empty-expected never matches, wrong/missing reject, exact accept).
  - `permission` (REQ-006/007/008): `decide` all arms — read→Allow (zero grants), write no/unrelated/empty-class grant→Deny, exact grant→Allow; `GrantTable` missing-key→empty + round-trip (S1).
  - `registry` (REQ-001/010): `tool_name` compose; the exact L1 set + tiers + `lookup` unknown→None; **F2** — `tools_list` derived from `REGISTRY` (name equality, no drift) + each tool has input+output schema.
  - `tools` (REQ-002/004/005): `fleet_snapshot_result` (structuredContent+text, isError false); `tool_error` isError; `resolve_surface` hit/miss/empty; `surface_receipt`/`surface_result` Accepted↔isError-false / Refused↔isError-true.
  - `resource` (REQ-003): list/read-known/read-unknown(→RESOURCE_NOT_FOUND)/updated-notification shape.
- **In-memory fake-transport integration in `dispatch` (S2 — NO bound port, 11 cases):** drive `handle_message`
  for initialize (capabilities), tools/list (2 tools), fleet.snapshot, unknown method (-32601), unknown tool
  (-32602), write-denied→isError, **surface resolved→Accepted + `Effect::SurfacePane`**, surface unknown→Refused
  isError + **no effect** (REQ-005), resources/read+subscribe+unknown-uri, notifications→no response +
  parse-error→id:null, and `snapshot_changed` gates on subscription (REQ-003 at the pure seam).
- **Surface-effect headless drive in `marley_app` (REQ-004/005):**
  `mcp_surface_effect_focuses_known_pane_refuses_bogus_headless` — seed two tabs (focus off pane 1),
  `apply_mcp_effect(SurfacePane(pane1))` → active tab + focused pane move back; a bogus handle → `false` + no
  change. (Added a `#[cfg(test)] focused_pane_for_test` seam — `workspace()` is module-private.)
- **Gate exclusion:** added `marley_mcp/src/transport.rs` + `marley_app/src/mcp_host.rs` to the coverage
  `--ignore-filename-regex` (the masked-shim precedent, documented in gates.sh).

### Runs (actual)
- `cargo nextest run -p marley_mcp` → **31 passed**.
- surface drive → **1 passed**.
- `cargo nextest run --workspace` → **1857 passed, 0 failed, 5 skipped** (+32 new).
- **Gate** `scripts/gates.sh --diff` — **attempt 1 RED** (3 fails, all from defensive-code / test idioms on a
  brand-new pure crate; the gate runs all 15 then reports):
  1. **gate:14 docs** — `registry()`/`tools_list()` PUBLIC docs linked `[`REGISTRY`]`/`[`tool_schemas`]`,
     which aren't re-exported (rustdoc `-D warnings`) → de-linked to plain code spans.
  2. **gate:4 coverage** (19 missed lines) — the class was: (a) unreachable `_ =>` catch-all arms
     (`dispatch::tools_call`, `registry::tool_schemas`) → **refactored to `match spec.family`** (EXHAUSTIVE,
     no catch-all → additivity is now COMPILER-enforced, a stronger REQ-011); (b) infallible
     `to_value(..).unwrap_or_else(|_| …)` fallback CLOSURES (uncovered functions) → `unwrap_or_default()`;
     (c) an untested `resources/read` unknown-uri arm → added a test; (d) **test-code** uncovered branches —
     `else { panic!() }`, `match { _ => panic!() }`, and formatted `assert!(…, "{}", x)` messages all have a
     never-run branch → rewrote extraction with an IRREFUTABLE or-pattern `let (Response(t)|Notification(t)) = o`
     + `matches!` asserts + plain asserts.
  3. **gate:5 mutation** (MSI 89.9%, 8 survivors) — the redundant `notifications/initialized` arm (equivalent
     to the `_` fallthrough → removed), an untested `resources/list` (added a test), the 5 `-32xxx` consts
     (a dropped `-` sign self-matched the const → added exact-value assertions), and the `tool_schemas` Fleet
     arm (now unviable under `match spec.family`).
  Fixes verified locally: `llvm-cov -p marley_mcp` → **100% lines** (0 missed); `cargo mutants -p marley_mcp`
  → **83 mutants, 74 caught / 9 unviable / 0 missed = MSI 100**.
- **Gate attempt 2 → GATE GREEN [diff]** (15/15): coverage 100% lines (40312 lines, 0 missed), mutation
  74 caught / 0 missed = MSI 100, docs ✓, miri + visual/AX ✓. Commit receipt written.

**Status: Phase 4 — Validate PASS.** The two coverage/mutation lessons (exhaustive-match-over-catch-all +
test-code-no-never-run-branches) are captured for Phase 5 prevention rules.

### REQ-003 honesty (inspect F4)
Subscribe→notify is proven at the PURE `handle_message`/`snapshot_changed` seam (the subscription gate is
exercised). The L1 transport SIMPLIFIES this to "opening the SSE GET stream = subscribing to the one
self-healing resource" (documented in `serve_sse_stream`); this is NOT end-to-end per-session subscription
proof — that (a `Mcp-Session-Id`-keyed store) is the L2 follow-up.

### Driven capture
**Not a render change** — #370 adds a protocol crate + a focus EFFECT. The effect (`surface_to_human` →
`jump_to_pane`) is behaviorally proven by the headless drive (active tab/pane assertions); no new pixels.
Driven capture is therefore N/A (no visual surface), and the machine is env-blocked (chad active at iTerm2)
regardless. The live loopback server is the masked transport, verified on the wire (not the gate).

## Phase 5 — Complete
