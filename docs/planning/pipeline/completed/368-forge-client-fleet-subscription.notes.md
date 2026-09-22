# marley_forge_client — MCP fleet subscription + the seat_events→envelope projection (Adapter #1) — Notes

- **Forge ticket:** #368 (9e447782-6ab1-4acb-be0e-318ff34d4455)
- **AAR:** d8235b74-e3d9-45ac-90f6-6e8a514cb6bd
- **Local ticket doc:** docs/planning/tickets/open/TICKET-368-forge-client-fleet-subscription.md
- **Pipeline spec:** 368-forge-client-fleet-subscription.spec.md

## Phase 1 — Plan
- **Request:** M23 Layer-1 item ② (orchestration-shell.md §12): grow `marley_forge_client` into
  Adapter #1 — (a) MCP resource subscription to Forge's fleet resources on a standing Streamable-HTTP
  connection (`notifications/resources/updated` handled; push = MCP subscription DECIDED, never
  Postgres), (b) cursor/replay catch-up (`resources/read` from the persisted cursor → replay into the
  `marley_fleet` reducer → re-subscribe), (c) the pure table-driven projection of `seat_events` +
  work-state onto #367's generic `SessionEvent`/`Session` envelope. CONTRACT-FIRST: L0 (the ucsosv2
  seat-emitter hook + the `seat_events` table on the Forge server) does not exist yet — this ticket
  ships the client machinery + projection + a fake-feed harness; live-wire verification is a named
  follow-up. AC satisfiable without ucsosv2.
- **Classification / tier:** feature, M23, sprint "M23 — Fleet Control Plane: Layer 1". Protocol/
  adapter work, no rendering, no write verbs. Sibling dependency: #367 creates `marley_fleet`
  (concurrent — the crate is NOT on disk yet; the projection is specified against its contracted
  types, exact signatures deferred to Phase 2).
- **Forge recall (§18.3):** docs-only drafting pass — no live forge MCP calls made; recall drawn from
  the on-disk AD register + memory notes. ADs of record: `AD-claude-tmux-grade-detached-sessions-001`,
  `AD-claude-mission-control-hypermedia-surface-001`, `AD-claude-brain-agent-session-supervision-001`
  (orchestration-shell.md §13). Standing lessons applied: the **MCP-bearer-never-logged** lesson
  (lib.rs module header; REQ-011 + D6); `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`
  + the enum-`Default`-derive viability lesson (#203/#204) — the projection returns enum/Vec shapes,
  so run `cargo mutants --list` on the actual code; the **mutants-skip detach trap** (inserting a fn
  above a masked shim rebinds `#[mutants::skip]` — this crate has exactly such a shim in adapter.rs);
  `PR-stat-before-read` for any cursor-file IO.
- **Discovery (what `marley_forge_client` actually is today — the size-determining read):**
  - **Transport is hand-rolled and one-shot.** `tool_call_request` (lib.rs:157-184) builds a raw
    HTTP/1.1 POST string — `Connection: close`, exact `Content-Length`, and (already)
    `Accept: application/json, text/event-stream` (lib.rs:176). The masked `fetch` (adapter.rs:81-101)
    opens a `TcpStream` to the loopback authority, writes, reads to EOF under 5s timeouts. One
    request = one connection. **No `initialize`, no `Mcp-Session-Id`, no standing connection, no
    server-notification path.** The standing GET SSE listen stream is genuinely NEW machinery.
  - **No SDK anywhere.** Crate deps = `serde` + `serde_json` (+ dev `mutants`); no `rmcp`, no tokio,
    no async runtime in the whole workspace. The substrate stays hand-rolled.
  - **Reusable halves already ours:** SSE `data:`-line extraction (`jsonrpc_from_http`,
    lib.rs:189-202) — generalizes to a multi-event stream frame parser; the typed `ForgeError`
    family (Http/Protocol/Rpc/Json, lib.rs:83-105); `.mcp.json` endpoint parse + the loopback-only
    bearer guard (lib.rs:110-142, security D1) + Debug redaction (lib.rs:29-36); the pure/masked
    split (all decisions pure in lib.rs at 100% cov/MSI; adapter.rs socket-only, `mutants::skip`).
  - **Wiring today:** app.rs:1121-1134 reads `.mcp.json` from cwd → `forge_endpoint_from` →
    `ForgeClient`, and runs the boot sprint fetch on gpui's `background_executor` (app.rs:1134) —
    the precedent for D-OPEN-STREAM-DRIVE.
  - **Cursor persistence idiom:** `*_in(dir)` fns over `marley_core::marley_config_dir()`
    (`~/.marley/config` — marley_core/src/paths.rs:38) — `settings_file_in`/`load_manager_in`,
    marley_app/src/settings.rs:327-338; a §14 convention ("File IO is testable: route it through
    `*_in(dir)` functions"). Leading candidate for D-OPEN-CURSOR-STORE.
  - **MCP spec confirmations (modelcontextprotocol.io, revision 2025-06-18, fetched 2026-07-20):**
    subscription is per-resource (`resources/subscribe`, `params:{uri}`, gated on server capability
    `resources.subscribe`); `notifications/resources/updated` carries `params:{uri}` only → the
    client re-reads (`resources/read` → `result.contents[]`); poke-then-read is the spec's own flow;
    server-initiated notifications ride a standing GET SSE stream (`Accept: text/event-stream`, 405
    if unsupported); `Mcp-Session-Id` from the initialize response echoed on all requests (404 →
    re-initialize); SSE `id`/`Last-Event-ID` is per-stream transport resume — distinct from the
    application-level durable cursor (spec D5 keeps correctness on the durable cursor).
  - **Event vocabulary consumed (fleet-control-plane.md §6, the contract):** rows
    {session_id, box, ticket, phase, state, event_type, payload, capabilities, ts, schema}; states
    `working|idle|at-menu|error|done|starting`; v1 kinds `session-start · heartbeat · turn-complete ·
    phase-start · phase-pass · phase-fail · gate-result · halted-with-question · api-error ·
    pr-opened · dispatch-claimed · car-complete · session-end`. Projection table locked in spec D3.
- **Decisions:** D1 push=MCP-subscription/never-Postgres; D2 adapter-boundary absolute (Forge string
  in `marley_fleet` = defect); D3 projection targets #367's contract, mapping table locked
  behaviorally; D4 contract-first (fake feed = verification substrate; fixtures = proposed v1 wire
  contract for L0; live wire = named follow-up); D5 catch-up on the durable application cursor, not
  SSE `Last-Event-ID`; D6 bearer posture inherited (loopback-only, redacted, never logged); D7
  defined unknown-input degrade (no panic, no dropped session); D8 pure/masked split preserved.
  Open for Phase 2: D-OPEN-CURSOR-STORE, D-OPEN-RESOURCE-NAMES, D-OPEN-LIFECYCLE,
  D-OPEN-STREAM-DRIVE (each with sweep evidence in the spec).

## Phase 2 — Design

### Architecture / approach
Grow `marley_forge_client` (NOT a new crate) — Adapter #1. The crate's existing shape is the template:
a PURE parent module (`lib.rs`: request builders + response parsers + `ForgeError` + endpoint/loopback
guard + redacted `Debug`) over a MASKED socket submodule (`adapter.rs`: the one-shot `TcpStream` POST,
`#[cfg_attr(test, mutants::skip)]`). #368 adds a NEW pure module `fleet.rs` (all the framing/builders/
parsers/projection/orchestration decisions) + extends `adapter.rs` with the masked standing-stream pump
+ masked cursor file-IO. Add `marley_fleet` as a path dependency (one-way: forge_client → fleet; fleet
is a serde-only leaf — no cycle). §14 honored: typed `ForgeError` everywhere, no `unwrap`/`expect` on any
response-reachable path, all socket/thread/file IO confined to the masked `adapter`.

**§20 confirmed N/A** — protocol/adapter machinery; the references are the OPEN MCP spec (revision
2025-06-18, cited in the spec's Prior art) + our own hand-rolled layer. No Warp/Zed behavior analog.

### The 4 D-OPEN forks — SETTLED (with evidence)
- **D-OPEN-CURSOR-STORE → a plain file under `marley_core::marley_config_dir()`**, via masked
  `load_cursor_in(dir)`/`save_cursor_in(dir, cursor)` in `adapter.rs`. The cursor is an opaque `String`,
  keyed per config-dir (one Forge per Marley) for v1. Evidence: the crate's convention is "all IO masked
  in the adapter" (lib.rs doc + adapter.rs `fetch`); the `*_in(dir)` shape mirrors settings.rs:327-338.
  The PURE logic (which URI+cursor to read next, how the next cursor is read off a page) is tested; only
  the file read/write is masked (trivial, no branching to cover).
- **D-OPEN-RESOURCE-NAMES → v1 uses `fleet://events` as the single cursor'd durable log**; the reducer
  derives the snapshot (no separate `fleet://seats` snapshot resource in v1 — reserved as a future
  replay-shortcut, noted not built). **The cursor rides as a URI query param**: `resources/read
  {uri: "fleet://events?since=<cursor>"}` (first read omits `since`). The read result is
  `result.contents[0].text` = a `FleetPage` JSON `{ "cursor": "<next>", "events": [<row>…] }`. This IS
  the proposed L0 wire contract (the fixtures pin it).
- **D-OPEN-LIFECYCLE → spec-correct.** Implement `initialize` (assert `capabilities.resources.subscribe`)
  + capture `Mcp-Session-Id` from the initialize response header + echo it (and `MCP-Protocol-Version`)
  on every subsequent request; a 404 → re-initialize. Evidence: the fake feed enforces it and it is the
  correct contract to hand L0; pure builders/parsers make it cheap. (`Last-Event-ID` SSE resume is NOT
  relied on — D5: the durable cursor is the correctness mechanism.)
- **D-OPEN-STREAM-DRIVE → an adapter-owned `std::thread`** holding the standing GET SSE socket, handing
  parsed events to the app via a `Fn` callback (std `mpsc` internally). Evidence: the crate has no async
  runtime and must stay gpui-free (it's a lib crate); a dedicated thread + channel is std-only. Masked.

### Three design refinements the SHIPPED #367 event set forces (flag for inspect)
#367 finalized `marley_fleet`'s event set AFTER this spec was drafted; two locked decisions get a
mechanism refinement (BEHAVIOR preserved), plus a wire-format choice:
- **R1 (refines D3):** most `seat_events` kinds project to **`Upsert` (full-descriptor refresh)**, not
  `StateChange` — because `marley_fleet::SessionEvent::StateChange` carries NO labels (only `Upsert`
  does). To update state + labels together (the D3 "state-change + label updates" intent), `Upsert` with
  the row's full descriptor is the correct event. The observable behavior (right state, opaque labels,
  Error≠Idle, question on at-menu) is exactly D3; the mechanism is Upsert. Consequence: each row carries
  the session's FULL current descriptor (title/state/labels/transport) + the trigger `event_type` — a
  self-contained "authoritative snapshot per row" contract (no delta-merge needed; labels replace
  wholesale, which is correct when the emitter sends the full label set each row).
- **R2 (refines D7):** the unknown-`event_type` / unknown-`state` / future-`schema` degrade =
  **`Heartbeat`** (advance `last_event_ms`, preserve everything), NOT "merge labels" — the envelope has
  no label-merge-only event, and a Heartbeat is the safe, non-corrupting, forward-compatible degrade
  (keeps the session alive + fresh, never drops it, never guesses state).
- **R3 (refines D2):** the proposed v1 wire uses **`ts_ms: u64` epoch-millis**, not RFC3339 — avoids a
  date-crate dependency (the crate is serde-only; #367 chose u64 millis for the same reason) and aligns
  the wire with the envelope. If a future wire sends RFC3339, add a parser then.

### Exact type/fn inventory — `src/fleet.rs` (all PURE, cov/MSI 100)
- **SSE multi-event framing** (generalizes `jsonrpc_from_http`'s single `data:` line):
  `struct SseEvent { event: Option<String>, data: String, id: Option<String> }`;
  `fn parse_sse_frames(buf: &str) -> (Vec<SseEvent>, String)` — split on blank-line frame boundaries,
  collect `event:`/`data:`/`id:` fields (multi-`data:` lines joined with `\n` per the SSE spec), return
  complete frames + the incomplete remainder the pump re-feeds.
- **Wire types** (`Deserialize`): `struct SeatEventRow { event_type: String, session_id: String,
  ts_ms: u64, #[serde(default)] state: Option<String>, #[serde(default)] title: Option<String>,
  #[serde(default)] transport: Option<String>, #[serde(default)] labels: BTreeMap<String,String>,
  #[serde(default)] payload: serde_json::Value }`; `struct FleetPage { #[serde(default)] cursor:
  Option<String>, #[serde(default)] events: Vec<SeatEventRow> }`.
- **MCP request builders** (mirror `tool_call_request`; each takes `session_id: Option<&str>`, embeds the
  bearer, `Connection: close` for the POSTs): `initialize_request`, `subscribe_request(uri)`,
  `read_request(uri)`, and `listen_request` (a GET with `Accept: text/event-stream`, no body).
- **MCP response parsers** (pure, typed `ForgeError`): `parse_initialize(response) ->
  Result<Initialized { session_id: Option<String>, supports_subscribe: bool }>` (2xx + `Mcp-Session-Id`
  header + `result.capabilities.resources.subscribe`); `parse_notification(jsonrpc) ->
  Option<ResourceUpdated { uri }>` (method == `notifications/resources/updated`); `parse_fleet_page(jsonrpc)
  -> Result<FleetPage>` (`result.contents[0].text` → `FleetPage`, reusing the `RpcResponse`/`RpcContent`
  structs, extended for `contents`).
- **The cursor riding:** `fn read_uri(base: &str, cursor: Option<&str>) -> String`
  (`fleet://events` or `fleet://events?since=<cursor>`); `next_cursor(page) = page.cursor.clone()`.
- **The projection** (table-driven, per R1/R2): `fn project_row(row: &SeatEventRow) -> SessionEvent`
  (`heartbeat`→Heartbeat; `session-end`→Ended; `halted-with-question`→QuestionRaised from
  `parse_question(&row.payload)` else Heartbeat; else `map_state(row.state)` Some→Upsert{full descriptor},
  None→Heartbeat [R2]); `fn project_page(page) -> Vec<SessionEvent>`; helpers `map_state(&str) ->
  Option<State>` (`starting|working|idle|waiting|at-menu|error|done`; at-menu→Waiting),
  `map_transport(Option<&str>) -> Option<Transport>`, `parse_question(&Value) -> Option<Question>`.
- **The orchestration model** (pure — makes REQ-008/009/010 testable without the socket):
  `struct FleetSync { snapshot: FleetSnapshot, cursor: Option<String> }` with
  `apply_page(&mut self, page: FleetPage)` (project + `marley_fleet::reduce` into `snapshot`, advance
  `cursor` to `page.cursor`), `snapshot()`, `cursor()`, `next_read_uri(&self) -> String` (= `read_uri`
  with the held cursor); `fn on_notification(uri: &str, subscribed: &str) -> Option<ReadAction>`
  (a matching updated-URI → a read action). The masked pump owns the socket/thread and simply sequences:
  initialize → subscribe → read(next_read_uri) → apply_page → on each frame: on_notification → read →
  apply_page → save_cursor → callback(snapshot).

### File manifest
| File | Change |
|---|---|
| `crates/marley_forge_client/Cargo.toml` | + `marley_fleet = { path = "../marley_fleet" }` dependency |
| `crates/marley_forge_client/src/lib.rs` | `mod fleet;` + re-export the new public types (`SeatEventRow`, `FleetPage`, `FleetSync`, `SseEvent`, the builders/parsers as needed by adapter + tests); the `RpcResponse`/`RpcContent` structs gain a `contents` view for `parse_fleet_page` (additive) |
| `crates/marley_forge_client/src/fleet.rs` | NEW — all the pure seams above + `#[cfg(test)]` unit tests |
| `crates/marley_forge_client/src/adapter.rs` | + the masked standing-stream pump (`FleetSubscription`: spawns the std::thread, holds the socket, sequences init/subscribe/read/notify via the pure fns, threads the cursor, invokes the callback) + masked `load_cursor_in`/`save_cursor_in` — all `#[cfg_attr(test, mutants::skip)]`. WATCH the mutants-skip detach trap: give each new masked fn its OWN `#[cfg_attr(test, mutants::skip)]` and re-run `cargo mutants --list -f adapter.rs` to confirm the skip set didn't shift |
| `crates/marley_forge_client/tests/fleet_harness.rs` | NEW — the fake-feed integration harness: fixture pages/frames driven through the real pure pipeline (parse → project → reduce), asserting the FleetSnapshot for the multi-step scenarios (replay/reconnect/overlap/notify/disconnect) |

### The fake-feed harness (fixtures = the proposed L0 wire contract)
Fixtures are JSON `FleetPage` bodies + SSE notification frames, authored as the v1 contract:
`session-start` × N, `heartbeat`, `halted-with-question` (payload = the question), a state-change
sequence, `api-error` (labels carry status/attempt), `session-end`, an UNKNOWN `event_type`, an unknown
`state`, a future `schema`, a cursor-overlap window, a reconnect (a second read from the persisted
cursor), and a disconnect (a truncated/garbage frame → typed error). The harness runs them through
`parse_sse_frames` → `parse_notification`/`parse_fleet_page` → `FleetSync::apply_page` and asserts the
resulting `FleetSnapshot` — the SAME code the masked pump calls, minus the socket.

### Regression Test Plan (every REQ → named test)
| REQ | Test (location) | Asserts |
|---|---|---|
| REQ-001 | `fleet::t368_req001_session_start_upserts_starting` | session-start → Upsert, state Starting, id = session_id, box/capabilities → opaque labels |
| REQ-002 | `fleet::t368_req002_known_state_maps` | map_state working/idle/done/starting → Working/Idle/Done/Starting (+ at-menu→Waiting, error→Error) |
| REQ-003 | `fleet::t368_req003_halted_question` | halted-with-question → QuestionRaised, question from payload; through reduce → state Waiting + question set |
| REQ-004 | `tests/fleet_harness` `t368_req004_later_row_clears_question` | question-raised then a working row → reduced snapshot's question is None |
| REQ-005 | `fleet::t368_req005_api_error_is_error` | api-error → Upsert state Error (≠ Idle); status/attempt ride labels |
| REQ-006 | `fleet::t368_req006_labels_opaque` + validate grep | ticket/phase/box/capabilities pass as opaque label strings; forge vocab stays in forge_client (D2 boundary — no marley_fleet identifier) |
| REQ-007 | `fleet::t368_req007_unknown_degrades_to_heartbeat` | unknown event_type / unknown state / future schema → Heartbeat (advance last_event, preserve state), never panics |
| REQ-008 | `tests/fleet_harness` `t368_req008_reconnect_replays_from_cursor` | a FleetSync seeded from a persisted cursor issues `read_uri` with `since=<cursor>` and applies the returned page before the (masked) re-subscribe — no gap row lost |
| REQ-009 | `tests/fleet_harness` `t368_req009_overlap_replay_idempotent` | apply_page(overlap) yields a FleetSnapshot equal to the no-overlap snapshot (rides #367's reducer idempotence) |
| REQ-010 | `fleet::t368_req010_notify_triggers_read` + harness | `on_notification(updated_uri, subscribed)` → a ReadAction for that uri; the harness drives subscribe→notify→re-read→reduce |
| REQ-011 | `fleet::t368_req011_bearer_redacted` + validate grep | every new bearer-holding type `Debug`-redacts (`***`); repo grep finds no bearer in any log/error string |
| REQ-012 | `fleet::t368_req012_unreachable_typed_errors` | parse_initialize on a 405, parse_sse on garbage, parse_fleet_page on malformed → typed `ForgeError` (no unwrap/panic); the masked pump's reconnect resumes from the cursor (accepted-untestable orchestration; the pure classifiers it calls ARE tested) |

**Testing boundary:** all of `fleet.rs` (framing, builders, parsers, cursor riding, projection, FleetSync)
is pure → cov/MSI 100, no exclusions. The `adapter.rs` stream pump + socket + cursor file-IO are
ACCEPTED-UNTESTABLE (masked, `mutants::skip` + coverage-excluded), mirroring `fetch`. Run `cargo mutants
--list -f fleet.rs` for the real set (projection returns `SessionEvent`/`Vec` — body-mutant viability
depends on `Default`; keep guard tests). No `trybuild` — no compile-fail type contract; the invariants
are runtime projection/reducer properties.

### Risks / decisions
- **R1's Upsert-per-row** means labels REPLACE (not merge) — correct given the full-descriptor contract;
  documented so L0's emitter knows to send the full label set each row.
- The masked pump is the bulk of the LOC but carries no tested logic (it only sequences pure fns +
  owns the socket) — the honest contract-first shape (the pure pipeline is the proof; the live wire is
  the follow-up).
- `parse_fleet_page` reuses the existing `RpcResponse`/`RpcContent` structs (extended with a `contents`
  vec) — additive, no change to the shipped read/write parsers.

## Phase 3 — Implement
- **Built** to the manifest:
  - `Cargo.toml` — `+ marley_fleet = { path = "../marley_fleet" }` (one-way dep; workspace re-checks
    clean, marley_app builds with the grown adapter).
  - `src/fleet.rs` (NEW, PURE) — `SseEvent` + `parse_sse_frames` (CRLF-normalized, multi-`data:` join,
    incomplete-remainder held); `SeatEventRow` + `FleetPage` (Deserialize, serde defaults, `ts_ms: u64`
    per R3); the request builders `initialize_request`/`subscribe_request`/`read_request`/
    `listen_request` (bodies here, HTTP framing delegated to `crate::mcp_post`/`mcp_listen_get` so the
    bearer never leaves lib.rs); `read_uri` (cursor as `?since=`); the parsers `parse_initialize`
    (Mcp-Session-Id header via `header_value` + `resources.subscribe` capability), `parse_notification`,
    `parse_fleet_page` (`result.contents[0].text` → FleetPage); the projection `project_row`/
    `project_page` + `map_state`/`map_transport`/`parse_question`/`str_array` (R1: known-state kinds →
    Upsert full-descriptor; R2: unknown → Heartbeat degrade); `FleetSync` (apply_page/snapshot/cursor/
    next_read_uri) + `on_notification` + `ReadAction`; module-local Deserialize views.
  - `src/lib.rs` — `mod fleet` + the public re-exports; `pub(crate) mcp_post`/`mcp_listen_get` +
    `session_header` (the fleet HTTP framing, bearer embedded HERE only — REQ-011).
  - `src/adapter.rs` — the MASKED `FleetSubscription` pump (a std::thread owning the standing SSE
    socket, sequencing init → replay-from-cursor → subscribe → notification-driven re-read via the pure
    seams, callback per snapshot; `Drop` stops+joins) + masked `run_subscription`/`refresh`/
    `open_listen_stream`/`load_cursor_in`/`save_cursor_in` — EACH `#[cfg_attr(test, mutants::skip)]`.
- **Deviations from design:** none material. The three design refinements (R1 Upsert-per-row, R2
  Heartbeat degrade, R3 u64 wire ts) are as designed — flagged for inspect since they refine locked
  decisions D3/D7/D2.
- **Checks:** `cargo fmt` clean; `cargo clippy -p marley_forge_client --all-targets` exit 0 (gate:2);
  `cargo check --workspace` green.
- **Mutant surface (traced, not guessed):** `adapter.rs` = **0 viable** (the whole pump is masked — the
  detach trap did NOT fire, each new masked fn carries its own skip); `fleet.rs` = **67** (the pure
  seam); `lib.rs` = 47 (the ~40 existing already-killed + the new mcp_post/mcp_listen_get/session_header).
  Phase 4 must kill the 67 fleet + the handful of new lib.rs mutants.
- **Tests:** none added here (Phase-4 job); production code + doc comments only.

## Phase 3.5 — Inspect
**4 independent general-purpose critics** (projection-correctness · MCP-protocol/parsing ·
security/bearer/agnosticism · mutation-readiness) + **my own independent verification** (two temporary
probes — a projection/FleetSync probe and an SSE/MCP-parser probe, both run green then removed — plus
bearer/agnosticism/panic greps). This inspect earned its keep heavily: **8 confirmed fixes** including a
safety-critical projection bug and an MSI-100-blocking equivalent mutant. (Two critics aborted on first
dispatch with 0 tool-uses; re-dispatched via SendMessage and both returned full reports.)

### Findings ledger
| # | Lens | Sev | Finding | Verdict | Resolution |
|---|---|---|---|---|---|
| F1 | projection | **High** | a `session-end` row with `state:"error"` → bare `Ended` → reducer launders a CRASHED seat to cleanly `Done` (violates "never read as cleanly Done"); amplified by cursor-only reconnect | REAL, safety-critical | **Fixed** — `project_row` → `Vec<SessionEvent>`; session-end+error → `Upsert(Error)` (retained). `BF-…-session-end-error-launders-to-done-001` |
| F2 | projection | Med | `halted-with-question` → bare `QuestionRaised` drops title/labels/transport (a first-seen-via-halt seat renders id-as-title, no chips) | REAL | **Fixed** — halt → `[Upsert(Waiting, descriptor), QuestionRaised]` |
| F3 | projection | Med | a malformed question degrades to `Heartbeat`, hiding an `at-menu` block (attention false-negative) | REAL | **Fixed** — malformed halt still emits `Upsert(Waiting)` (surfaces the block, no question text) |
| F4 | protocol | Med | no `notifications/initialized` — a spec-strict server rejects subscribe/read (D-OPEN-LIFECYCLE "spec-correct" bar) | REAL gap | **Fixed** — added `initialized_notification` builder + fire it after handshake |
| F5 | protocol | Med | masked pump `String::from_utf8_lossy(&chunk[..read])` per-chunk corrupts a multibyte char split across a 4 KB read boundary (silent U+FFFD) | REAL (masked live path) | **Fixed** — byte buffer + decode only the valid-UTF-8 prefix, retain the partial tail |
| F6 | security | Med | listen loop `Err(_) => continue` spins the dead socket hot on a HARD disconnect (ECONNRESET returns Err immediately, not the 1s timeout) — REQ-012 | REAL (masked) | **Fixed** — match `err.kind()`: WouldBlock/TimedOut → continue, else break |
| F7 | mutation | **Blocker** | `fleet.rs` `normalized[..idx + 1]` harbors an EQUIVALENT mutant (`+`→`*`) no test can kill → MSI 100 unreachable (the trailing `\n` is invisible to `.lines()`) | REAL, verified equivalent | **Fixed** — simplified to `[..idx]` (removes the site). `PR-…-redundant-arithmetic-is-an-equivalent-mutant-001` |
| F8 | protocol | Low | `parse_initialize` "neither result nor error" → silent `supports_subscribe=false` (inconsistent with `parse_write_ack`'s `Err(Protocol)`) | REAL (fails safe) | **Fixed** — returns `Err(Protocol)` |
| F9 | protocol | Low | `read_uri`/`apply_page` empty-cursor asymmetry (`Some("")` → `?since=`) + no URL-encoding | REAL (contract-dependent) | **Fixed** — `Some("")` treated as `None`; url-safe-cursor requirement documented on the L0 contract |
| F10 | protocol | Low | `parse_fleet_page` missing-`contents` → `Err(Json)` not `Err(Protocol)` | REAL (cosmetic) | **Fixed** — `#[serde(default)] contents` funnels both to Protocol |
| F11 | protocol | Low | `parse_sse_frames` doesn't handle a bare-CR terminator | REAL (spec-completeness) | **Fixed** — normalize lone `\r` → `\n` too |
| — | security | Low | no auto-reconnect after a stream close | ACCEPTED for L0-absent slice | **Phase-5 follow-up** (a restart already resumes from the cursor — REQ-008 by construction) |
| — | projection | Low | `apply_page` cursor no monotonicity guard; `parse_fleet_page` reads only `contents[0]` | Not reachable (sequential pump; one page = contents[0]) | documented, no change |

### Independent verification I ran myself (corroborates critics before/around their reports)
- **Projection probe** (temp, green, removed): every D3/R1/R2 kind mapped correctly + FleetSync REQ-004
  (question-clear) / REQ-009 (overlap idempotent) / cursor-None-no-regress. (Ran pre-refactor; the F1–F3
  refactor is the fix, re-proven in Phase 4.)
- **SSE/MCP-parser probe** (temp, green, removed): multi-frame/CRLF/incomplete-remainder, notification
  match/non-match, initialize header+capability, fleet-page error arms, the request builders.
- **Greps:** bearer only in lib.rs's request-text builders (never fleet.rs/adapter pump); every adapter
  error string uses `endpoint.url()` never the bearer/request; forge vocab zero in `marley_fleet`; no
  unwrap/expect/panic in fleet.rs prod or the pump. (Critic 3 confirmed all of these independently.)

### The Phase-4 kill map (critic 4 — bake these B-MISS shapes into validate)
`fleet.rs` = **75 mutants** (post-refactor); **9 UNVIABLE** (Default-body on no-Default types: SseEvent,
Initialized, ResourceUpdated, State, Transport, Question, SessionEvent ×2). VIABLE-but-needs-a-guard:
`parse_fleet_page`/`with_cursor`/`snapshot` return Default-deriving types → assert NON-default fields.
The B-MISS test shapes (the obvious REQ test misses the mutant):
- **Request builders** (`initialize`/`subscribe`/`read`/`listen`/`initialized`): assert request CONTENT
  (method + uri + headers), NOT `is_some` — `Some("")`/`Some("xyzzy")` survive an `is_some`-only check.
- **`session_header`** (lib.rs): must be hit via a **Some-session** builder path (initialize passes
  `None`) — assert the request contains `Mcp-Session-Id: s1` AND a None-path omits it.
- **`map_state`** (7 arms incl. the plan-omitted plain `"waiting"`) + **`map_transport`** (3 arms — NO
  named test in the plan!): each arm asserts a DISTINCT `Some(State/Transport::X)`; a shared expected
  value hides an arm-swap.
- **`parse_sse_frames`**: one input with `idx` ≠ 0 and ≠ 2 asserting `events` AND `remainder` AND the
  last frame's data (kills the killable `idx+2` `+→-`/`+→*`); + multi-`data:`, no-`data:`-skip,
  leading-space cases.
- **`parse_notification`/`on_notification`**: BOTH arms (matching → Some, non-matching → None); use a
  query-carrying updated_uri so `split('?')` is exercised.
- **`project_row` heartbeat arm**: a heartbeat row with a KNOWN state (else the arm-delete is equivalent
  to the degrade); the title fallback both ways; session-end+error → Upsert(Error), clean → Ended.
- **`FleetSync`**: `with_cursor(Some("c1")).cursor()==Some("c1")`; `apply_page` asserts an observable
  snapshot change + cursor advance; the cursor-None-page-doesn't-clobber false branch.
- **`parse_initialize`**: assert the EXACT `session_id` (not `is_some`) to kill the 4 `header_value`
  mutants; the neither→`Err(Protocol)` arm (F8).
- `adapter.rs` = **0 mutants** (fully masked — the F5/F6 fixes are accepted-untestable, correct by
  inspection); the detach trap did not fire.

### Result
Zero remaining code defects; 11 findings fixed (1 High, 5 Med, 5 Low) + the equivalent-mutant blocker
removed. `cargo fmt --check`, `cargo clippy --all-targets` (exit 0), `cargo check --workspace` all green
after the fixes. Captured: `BF-…-session-end-error-launders-to-done-001` +
`PR-…-project-rich-row-through-descriptor-event-honor-terminal-state-001` +
`PR-…-redundant-arithmetic-is-an-equivalent-mutant-001`.

## Phase 4 — Validate
- **Tests added: 26 for #368** (24 inline `#[cfg(test)]` in fleet.rs + `mcp_post_and_listen_frame_headers`
  in lib.rs + the harness scenarios inline). One+ per EARS row + critic 4's kill map:
  - `parse_sse_frames` (events+remainder+data with idx≠0,≠2; multi-`data:` join; no-`data:` skip;
    leading-space; `id:` field; entirely-empty inner frame; bare-CR; empty).
  - `map_state` all 7 arms + `map_transport` all 3 arms (each a DISTINCT value).
  - `project_row` via whole-event `assert_eq` (session-start→Upsert Starting, api-error→Upsert Error,
    F1 session-end+error→Upsert(Error)/clean→Ended, F2 halt→[Upsert(Waiting),QuestionRaised], F3
    malformed-halt + non-string-prompt→Upsert(Waiting), heartbeat arm with a KNOWN state, R2 degrade,
    title fallback both ways, options/context extraction).
  - request builders (CONTENT asserts, incl. `initialized_notification` no-id + session header);
    `read_uri` (3 arms incl. empty); `on_notification` both arms (query-carrying uri);
    `parse_initialize` (EXACT session_id, decoy-body-ignored, capability true/false, non-2xx/rpc/neither
    /malformed-JSON error arms); `parse_notification` both arms; `parse_fleet_page` (non-default +
    rpc/empty/missing-contents/inner-bad/outer-malformed arms); `FleetSync` (with_cursor/new/apply
    observable/cursor-None-no-clobber); the inbound-wire deserialize.
  - `lib.rs`: `mcp_post`/`mcp_listen_get` via a **Some-session** path (kills the `session_header` arms).
  - the fake-feed harness scenarios: REQ-004 (question clears), REQ-008 (reconnect-from-cursor replay),
    REQ-009 (overlap idempotent == no-overlap), REQ-010 (subscribe→notify→read→reduce).
- **Runs (real output):**
  - `cargo nextest run -p marley_forge_client` → **36 passed** (26 new + 10 pre-existing).
  - `cargo test --doc` → 0 doctests.
  - **`cargo mutants -p marley_forge_client` → 122 mutants: 108 caught, 14 unviable = MSI 100%** (all
    of critic 4's B-MISS targets killed; the equivalent-mutant blocker was removed at inspect).
  - **REQ-006/011 greps:** forge vocab zero in `marley_fleet` (D2); bearer only in lib.rs's request-text
    builders (never fleet.rs/adapter) — clean.
  - **`scripts/gates.sh --diff` → `GATE GREEN [diff]`, 15/15**: coverage **100% lines** on fleet.rs +
    lib.rs (adapter.rs masked/excluded per the gate's documented shim regex), mutation MSI 100, rustfmt/
    clippy/audit/deny/machete/gitleaks/shellcheck/no-suppressions/SAST/docs/miri/visual. Receipt written.
- **Coverage note (honest):** two `map_err(Json)` closures + a `.as_str()?` needed a malformed-body /
  non-string-prompt test each (the phantom-region quirk llvm-cov shows on `?`/closure chains); LINE
  coverage is 100% (8 residual *region* misses are the closure phantoms, not lines — the gate is
  line-based). Test `assert_eq`-on-whole-event replaced the `other => panic!()` arms so no unreachable
  panic dented coverage.
- **Driven-capture: N/A (honest).** `marley_forge_client` is a protocol/adapter crate with NO UI
  surface; the live socket pump is masked (accepted-untestable, L0 absent). The fleet rail render is
  #369, where the driven capture lives.
- **Pre-existing failures:** none introduced (the `block v0.1.6` future-incompat note is a pre-existing
  transitive-dep warning).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md Added entry; orchestration-shell.md §7 crate row for
  `marley_forge_client` → **fleet-subscription SHIPPED (#368, contract-first)** with the v1 wire
  contract + R1/R2/R3 realizations recorded.
- **AAR** `d8235b74` submitted (completed, effectiveness 5; 3 novel findings → distillation/drift/
  emergence).
  - **What worked:** the pre-authored Fable spec's structure held, but its D3 projection table needed a
    mechanism REFINEMENT (R1/R2) that the SHIPPED #367 event set forced — StateChange carries no labels,
    so descriptor-bearing kinds must project through Upsert; the spec was drafted before #367 finalized
    its enum. This is the Fable→Opus workflow catching a cross-ticket contract drift at design/inspect.
  - **What the inspect bought (heavily):** 11 real issues from 4 critics — the **High F1** (a
    `session-end` error laundering a crashed seat to `Done`, safety-critical), F2/F3 (descriptor-drop on
    the non-Upsert projections), F4/F5 (missing `notifications/initialized`; utf8-split corruption),
    F6 (hot-spin on hard disconnect), and critic 4's **MSI-blocking equivalent mutant** (`[..idx+1]`).
    My own probes + greps independently corroborated projection + security before the critics landed.
  - **Process notes:** 2 critics aborted on first dispatch (0 tool-uses) and were re-dispatched via
    SendMessage — both then returned full reports; worth watching. The phantom-region coverage quirk on
    `?`/closure chains needed a malformed-body test per `map_err(Json)` closure + whole-event
    `assert_eq` (not `match{…,other=>panic!()}`) to reach 100% LINE coverage.
  - **Failures/rules captured:** `BF-…-session-end-error-launders-to-done-001` +
    `PR-…-project-rich-row-through-descriptor-event-honor-terminal-state-001` +
    `PR-…-redundant-arithmetic-is-an-equivalent-mutant-001`.
- **Named follow-ups filed:** **#372** (live-wire verification vs real `seat_events` when Layer 0
  ships — blocked on ucsosv2) + **#373** (auto-reconnect/backoff for the subscription pump — the
  inspect LOW).
- **forge:** #368 closed.
- **Train note:** ticket ② of the M23 Layer-1 fleet train COMPLETE. #367 (the envelope) + #368 (this
  adapter) are shipped; #369 (the rail) renders `FleetSnapshot`, #370 (the MCP server) exposes it.
