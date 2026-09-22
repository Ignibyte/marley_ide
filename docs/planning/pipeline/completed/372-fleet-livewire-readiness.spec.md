---
pipeline_id: c03eb0aa-835b-4356-93de-54f456e16617
ticket: forge#372 (23b95391-d4b3-4739-9581-bb6fb39b4f89) · local docs/planning/tickets/open/TICKET-372-fleet-livewire-readiness.md
aar_id: 5067cea0-438b-4218-8572-e087c7065c29
status: Phase 5 — Complete PASS
title: Fleet L0 live-wire READINESS — fixture forge server + real-socket pump integration test + typed disconnect/404 + the going-live runbook
type: chore
milestone: M23.5
references:
  - docs/marley_architecture/orchestration-shell.md
  - crates/marley_forge_client/src/adapter.rs
  - crates/marley_forge_client/src/fleet.rs
  - crates/marley_forge_client/src/lib.rs
  - crates/marley_lsp/tests/integration.rs
  - docs/planning/pipeline/completed/368-forge-client-fleet-subscription.spec.md
---

## Title
**The #368 named follow-up, RESPEC'd (see Scope).** #372 was filed as "live-wire verification against
real `seat_events`" (368.notes:378) — but Layer 0 (the ucsosv2 seat-emitter + the `fleet://events`
resource on the real Forge server) is external and does not exist yet, so the literal run is blocked
indefinitely. This slice makes Marley live-wire READY now: a deterministic test-only loopback fixture
forge server speaking exactly the pump's wire dialect, an end-to-end integration test that drives the
REAL masked `FleetSubscription` pump (adapter.rs) over a REAL socket against it — converting the #368
"accepted-untestable" pump into integration-tested behavior — plus the missing TYPED half of session
handling (a 404-session signal + a typed disconnect reason; recovery stays #373), and a "going live"
runbook so L0-day is config + checklist, zero Marley code.

## Scope
### In
- **The respec itself (recorded here per the batch owner's scope decision):** #372 = readiness now;
  the literal real-ucsosv2 verification run is OUT, deferred to a tiny ticket minted when L0 lands.
- **Fixture forge server (test code only)** — `std::net::TcpListener` bound `127.0.0.1:0` + a thread,
  deterministic, NO tokio/async. It speaks the pump's ACTUAL dialect (verified in Prior art leg 3):
  one TCP connection per POST with `Connection: close` + client read-to-EOF; a standing GET SSE stream
  held open; responses = HTTP/1.1 status line + headers (incl. `Mcp-Session-Id` at initialize) + one
  SSE `data:` line. It replays the frozen #368 fixture pages (`FleetPage{cursor,events}`,
  `SeatEventRow` with `ts_ms: u64`, `fleet://events?since=<cursor>`) and records the method/header
  sequence it observed for order asserts.
- **End-to-end integration test** (`crates/marley_forge_client/tests/livewire.rs`, NEW) driving the
  REAL `FleetSubscription::start` over the real socket: lifecycle order (initialize →
  notifications/initialized → resources/read → resources/subscribe), `Mcp-Session-Id` echo observed
  server-side, catch-up read from a pre-persisted cursor BEFORE subscribe, SSE events split across
  ≥3 TCP write segments, final `FleetSnapshot` EQUAL to the pure `FleetSync` path's snapshot on the
  same pages, restart-resumes-from-persisted-cursor with overlap idempotence, and mid-stream close →
  clean typed end (no panic, no retry).
- **Typed session/disconnect signals (the pure delta):** the `Mcp-Session-Id` echo ALREADY SHIPPED in
  #368 (evidence under D5) — what's missing is typing: a 404 response today collapses into stringly
  `ForgeError::Http(status_line)` (lib.rs:253-255), and the pump ends silently on close/error
  (adapter.rs:219/:232) with no observable reason. Add a pure exit-reason enum + pure classifiers
  (404-session → session-expired; close/hard-error/handshake-failure → their reasons) at cov/MSI 100,
  surfaced from `FleetSubscription` (exact surface = Phase 2). The pump only STORES the value.
- **Runbook** — a named "Going live (L0-day) runbook" subsection in
  docs/marley_architecture/orchestration-shell.md: settings row (`[[projects.orchestration]]` →
  `brain_endpoint`, #371), preconditions honestly listed (see D8), and the original (a)–(e)
  verification checklist to run against real ucsosv2 on L0-day.

### Out (explicitly deferred)
- **The literal real-ucsosv2 verification run** — a tiny ticket minted at L0-land (D1).
- **Reconnect/backoff/re-initialize-on-404** — the recovery POLICY is #373; here only the typed
  signals it will consume. The integration test asserts NO retry happens.
- **Vocabulary reconciliation with L0's actual emitter output** — needs L0 to exist.
- **App-side wiring of `FleetSubscription` into the shell** — `fleet_snapshot` is fed only by the
  demo verb today (app.rs:386-387/:2166); wiring rides the #373 lane (a non-reconnecting pump should
  not be wired into a long-lived app), and the runbook names it as a precondition.
- Layer 0 itself; any UI; any new write verbs.

## Reference (§20)
**N/A — Marley-specific fleet-control-plane infrastructure; no reference-app behavior analog.** This
is test machinery + typed-error plumbing for Marley's own MCP client (a clean-room, hand-rolled
Streamable-HTTP layer) plus an architecture-doc runbook. Neither Warp nor Zed has an observable
behavior to match — there is no user-facing surface at all. The governing references are the
published MCP specification (Prior art leg 2) and Marley's own shipped #367/#368 code and ratified
orchestration-shell.md. No copyleft source consulted; clean-room §20 holds.

### Prior art
1. **Behavior maps (docs/warp_architecture, docs/zed_architecture): N/A — checked, inapplicable.**
   Protocol/test infrastructure, not terminal/editor UX; the maps carry nothing about MCP transports
   or test fixtures. Recorded as checked-and-not-applicable, not skipped.
2. **Published — the MCP specification, revision 2025-06-18** (already fetched + recorded in the #368
   sweep, 368.spec:74-88; cited by section, no re-fetch needed). *Transports → Streamable HTTP*:
   every client→server message is a new HTTP POST with `Accept: application/json, text/event-stream`;
   server-initiated notifications arrive on a standing GET SSE stream (`Accept: text/event-stream`;
   the server MUST answer `text/event-stream` or 405). *Session Management*: `Mcp-Session-Id` is
   assigned in the initialize response header and MUST be echoed on every subsequent request; a
   server MAY answer **404** for an expired session, upon which the client MUST re-initialize — the
   echo + the typed 404 signal are THIS ticket; the mandated re-initialize is #373's recovery loop.
   *Lifecycle*: `initialize` → `notifications/initialized` before any other request.
3. **OUR OWN source (the load-bearing leg — the fixture must mirror what the pump actually speaks):**
   - **The pump's wire dialect.** `fetch` (adapter.rs:93-113): raw HTTP/1.1 POST over
     `std::net::TcpStream`, 5s read/write timeout, read-to-EOF relying on the request's
     `Connection: close`. POST framing = `mcp_post` (lib.rs:198-221 — bearer, `MCP-Protocol-Version`,
     optional `Mcp-Session-Id` via `session_header` lib.rs:242-247, `Content-Length`). The standing
     stream = `open_listen_stream` (adapter.rs:287-304): `mcp_listen_get` (lib.rs:225-239 — GET,
     `Accept: text/event-stream`, NO `Connection: close`), **1s read timeout** (adapter.rs:298) as
     the stop-flag wakeup. Responses are parsed by `jsonrpc_from_http` (lib.rs:252-265: 2xx status
     line + first SSE `data:` line) and `parse_initialize` (fleet.rs:229-246, reading the
     `Mcp-Session-Id` response HEADER via `header_value` fleet.rs:250-260).
   - **The pump's sequence.** `run_subscription` (adapter.rs:166-252): initialize → gate on
     `supports_subscribe` → `notifications/initialized` → **refresh (cursor'd `resources/read`)
     BEFORE `resources/subscribe`** (adapter.rs:196-202) → a second refresh (:204-206) → GET stream →
     per-notification refresh. Read loop: `Ok(0)` → break (:219), timeout → continue (:224-231),
     hard error → break (:232) — **no reason is surfaced anywhere; the thread just ends.** UTF-8
     byte-boundary buffering at :215-243 is the seam the ≥3-segment scenario proves over real TCP.
   - **Session echo: ALREADY SHIPPED.** Capture at fleet.rs:230; echo on every builder via
     `mcp_post`/`mcp_listen_get`; the pump threads it (adapter.rs:188-206); pinned by tests
     fleet.rs:865 (`Mcp-Session-Id: s1\r\n` in the notification) and lib.rs:660-682 (POST + GET, Some
     and None arms). The typed-404 half is genuinely absent: `jsonrpc_from_http` yields
     `Err(Http("HTTP/1.1 404 Not Found"))` — matchable only by string.
   - **A latent listen-stream gap (Phase-2 input):** `open_listen_stream` never inspects the GET
     response's status line — the HTTP header block is consumed by `parse_sse_frames` as a frame with
     no `data:` and silently skipped (fleet.rs:41/:72), so a 404/405 on the GET idles forever instead
     of surfacing. The typed exit-reason work should close this (D6).
   - **The frozen contract.** `FleetPage`/`SeatEventRow` (fleet.rs:119-127/:91-115, `ts_ms: u64`),
     `read_uri` `?since=` (fleet.rs:207-212), and the fixture JSONs in fleet.rs tests (:1030, :1063-
     :1137) — the proposed v1 wire contract handed to L0 (368.spec D4). The integration test replays
     THESE, and its expected snapshot is computed through the same pure `FleetSync` (fleet.rs:441-490).
   - **Integration-test precedent (the workspace decision D3 rests on).** Nine crates have `tests/`
     dirs; the closest analog is **crates/marley_lsp/tests/integration.rs:1-3** — "drive the REAL
     process adapter (`spawn_server`) + the REAL framing/route/lifecycle against the scripted
     `fake_ls` fixture" — with bounded `poll_message`/`poll_disconnect` deadline helpers (:18-49).
     nextest runs `tests/` workspace-wide (scripts/gates.sh:83). **No test anywhere stands up a
     `TcpListener` today** (grep: the only hits are app code — marley_mcp/src/transport.rs:82's
     masked loopback server, itself modeled on this adapter and coverage-excluded at gates.sh:222) —
     so this fixture server is the workspace's first, composing the marley_lsp scripted-fixture shape
     with transport.rs's bind-`127.0.0.1:0` + thread idiom. Temp-dir precedent for the cursor
     `config_dir`: `tempfile` dev-dep (marley_settings/Cargo.toml dev-dependencies;
     crates/marley_settings/tests/settings.rs:25-28).
   - **No crate we ship owns this seam** — no tokio/async runtime anywhere in the workspace, no rmcp
     (368.spec:92-93); the substrate is our own hand-rolled layer. Nothing to adopt beyond it.

## Locked-In Decisions
- **D1 — The respec: readiness now, the real run deferred.** L0 is external and indefinitely blocked;
  #372 lands everything provable without it, and the literal real-ucsosv2 run becomes a tiny ticket
  minted at L0-land (its checklist = the runbook, D8). *Rejected:* leaving the ticket blocked-open (a
  sprint slot pinned on an external dependency, and the pump stays untested-in-integration meanwhile);
  closing as unactionable (loses the named verification obligation).
- **D2 — Fixture server = `std::net::TcpListener` + a thread, in TEST code, no async runtime.**
  Deterministic scripted scenarios, blocking IO, bind `127.0.0.1:0` (the transport.rs:82 idiom).
  *Rejected:* a tokio dev-dep (the workspace has NO async runtime anywhere — 368.spec:93; importing
  one to test a `std::net` client adds a second concurrency substrate for zero fidelity gain);
  mocking the stream behind a trait (the whole point is a REAL socket — byte-split, timeouts, and
  close semantics are exactly what a mock fakes away).
- **D3 — It lives in `crates/marley_forge_client/tests/livewire.rs` (a NEW `tests/` integration
  dir), fixture server as a module inside the test crate.** Precedent: marley_lsp/tests/
  integration.rs (real seam vs scripted fixture, bounded polls); nextest already runs `tests/`
  (gates.sh:83). Unlike `fake_ls` no separate bin is needed — the counterpart is an in-process
  thread, not a subprocess. *Rejected:* a `#[cfg(test)]` module in src/ (drags a server into the
  unit lane, puts test plumbing in mutation/coverage territory, and bloats the crate's source);
  a `fake_forge` bin (subprocess indirection with no isolation benefit here).
- **D4 — The fixture mirrors the pump's ACTUAL dialect, not generic HTTP.** Exactly the shapes in
  Prior art leg 3: per-POST connections read-to-EOF on close, one `data:` line per response, the
  session id as a response header at initialize, a held-open GET with incremental writes. No chunked
  encoding, no keep-alive, no header canon beyond what `header_value`/`jsonrpc_from_http` parse.
  *Rejected:* a general HTTP test server (fidelity to the real peer is the test's value; generality
  is dead code by the house never-run-branch rule).
- **D5 — Session-id scope = the typed 404 signal ONLY; the echo is already shipped; recovery is
  #373.** Evidence: capture fleet.rs:230, echo lib.rs:242-247 via every builder, pump threading
  adapter.rs:188-206, pinned at fleet.rs:865 + lib.rs:660-682. The delta: a pure classifier typing a
  404-session response (today a stringly `Http` error, lib.rs:253-255) + its surfacing as a typed
  exit reason. *Rejected:* re-implementing the echo (it exists — the "if missing" check in the batch
  brief resolves to NO); building the re-initialize loop here (#373 owns recovery policy — this
  ticket's test asserts NO retry).
- **D6 — A typed exit-reason seam, minimal.** A pure enum (session-expired · stream-closed ·
  hard-disconnect · handshake-failed, exact arms Phase 2) + pure classifiers at cov/MSI 100;
  `FleetSubscription` exposes the terminal reason (poll-able accessor vs `on_exit` callback =
  Phase 2); the masked pump only maps-and-stores. Covers the latent silent-GET-rejection gap (Prior
  art leg 3). *Rejected:* leaving the end silent (then #373 has no typed signal to act on, and
  REQ-008/009 have nothing observable to assert).
- **D7 — The integration test is a BEHAVIOR proof, not a mutation target.** adapter.rs stays
  `#[cfg_attr(test, mutants::skip)]` + coverage-excluded (gates.sh:222 already lists it); the new
  pure seams land in lib.rs/fleet.rs at cov/MSI 100. Fixture-server test code follows the house
  rule: no never-run branches (every scripted arm fires in some scenario; bounded deadline polls,
  the integration.rs:18-49 shape — no `else { panic! }` arms that never fire).
- **D8 — The runbook lands IN orchestration-shell.md as a named subsection, honest about
  preconditions.** Checklist: the #371 settings row (`brain_endpoint`), the NOT-yet-landed
  preconditions (app-side `FleetSubscription` wiring — app.rs:386-387/:2166 is demo-fed today — and
  #373 reconnect), then the original (a)–(e) verification list against real ucsosv2. *Rejected:* a
  separate new doc (orchestration-shell.md §12 already owns sequencing; a second home drifts).

## Acceptance Criteria (EARS)
"Fixture" = the D2/D3 loopback server; "the pump" = the real `FleetSubscription` thread.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the pump is started against the fixture, the fixture shall observe the MCP lifecycle in order — `initialize`, `notifications/initialized`, `resources/read`, `resources/subscribe` — before the standing GET arrives. | integration: assert on the fixture's recorded method sequence |
| REQ-002 | WHEN the fixture's initialize response carries `Mcp-Session-Id: S`, every subsequent request (each POST and the GET) shall carry `Mcp-Session-Id: S`. | integration: server-side header assert on every recorded request |
| REQ-003 | WHEN a cursor `K` is pre-persisted in the pump's config dir, the FIRST `resources/read` shall target `fleet://events?since=K` and shall be observed BEFORE `resources/subscribe`. | integration: URI + sequence-index assert |
| REQ-004 | WHEN the fixture replays the frozen #368 fixture pages (initial read + notified re-reads), the pump's final delivered `FleetSnapshot` shall EQUAL the snapshot from driving the same pages through the pure `FleetSync` in-process. | integration: `assert_eq!` of the two snapshots (bounded poll for the last update) |
| REQ-005 | WHEN the fixture writes an SSE `notifications/resources/updated` split across ≥3 separate TCP write segments (flush between), the pump shall parse it identically and issue the follow-up read (byte-split invariance over a real socket). | integration: segmented-write scenario → the re-read arrives + snapshot matches |
| REQ-006 | The fixture pages' rows shall be byte-for-byte the frozen #368 contract JSON (`FleetPage{cursor,events}`, `ts_ms` as u64), and shall deserialize into `SeatEventRow` across the real wire. | integration: fixture literals mirror fleet.rs:1030/:1063-1137 + a row-content assert on the delivered snapshot |
| REQ-007 | WHEN the pump is stopped after advancing the cursor to `C2` and restarted against the fixture, it shall read from `?since=C2` (the persisted cursor, observed server-side), and an overlap-window replay shall leave the snapshot EQUAL to the no-overlap snapshot (no gaps, no duplicates). | integration: two-run scenario — persisted-file + `since=` assert + snapshot equality |
| REQ-008 | WHEN the fixture closes the standing GET stream mid-run, the pump thread shall end cleanly within a bounded wait — no panic, and NO new connection attempt observed by the fixture (retry is #373) — and the subscription shall surface the typed stream-closed reason. | integration: bounded join + fixture connection-count assert + typed-reason assert |
| REQ-009 | WHEN a post-initialize request is answered 404, the classifier shall type it as session-expired (matchable in type, not by string), the pump shall surface that typed reason and stop without re-initializing. | pure classifier units + integration: 404 scenario — typed reason + no further `initialize` recorded |
| REQ-010 | The new pure seams (exit-reason enum, 404/exit classifiers, any touched header/status logic) shall sit at cov 100 / MSI 100; adapter.rs shall remain masked + coverage-excluded. | gate:4/gate:5 exit codes; `cargo mutants --list -f` on the ACTUAL touched files |
| REQ-011 | docs/marley_architecture/orchestration-shell.md shall contain the named "Going live (L0-day) runbook" subsection with the settings-row step, the named preconditions, and the (a)–(e) verification checklist. | docs presence: grep the heading + checklist items; Phase-3.5 review for honesty |

## Testing boundary (floors, stated plainly)
Pure seams — SSE framing, builders/parsers, `FleetSync`, the NEW exit-reason enum + classifiers, any
touched session/status logic — stay at cov/MSI 100 (run `cargo mutants --list -f` on the ACTUAL
touched files; enum-return body-mutant viability depends on Default derives — the #203/#204 lesson;
watch the mutants-skip detach trap when adding fns near masked shims). The pump (adapter.rs) stays
`mutants::skip` + coverage-excluded (gates.sh:222) — the integration test is its behavior proof, not
a mutation target. The fixture server is TEST code: no never-run branches, bounded deadline polls
(the integration.rs:18-49 shape), deterministic scripts.

## Phase Plan
- **P2 Design** — exact exit-reason enum + classifier signatures and the surfacing choice (poll vs
  callback); the fixture server's scenario-script structure (per-test server vs shared, GET-thread
  handling while POSTs continue); the scenario→REQ manifest; settle the GET-status-validation gap;
  file manifest sketch: `tests/livewire.rs` (NEW), `src/lib.rs` + `src/fleet.rs` (pure deltas),
  `src/adapter.rs` (store-only pump edits, masked), `Cargo.toml` (+`tempfile` dev-dep),
  `docs/marley_architecture/orchestration-shell.md` (runbook), CHANGELOG.
- **P3 Implement** — pure seams first (enum, classifiers), then the masked store-only pump edits,
  then the fixture server + scenarios, then the runbook subsection.
- **P3.5 Inspect** — independent critics vs the diff: no-retry honesty (REQ-008/009), bearer-redaction
  contract on any new type (the lib.rs:29-36 posture), never-run-branch sweep of the fixture,
  runbook-precondition honesty, mutants-skip attachment check.
- **P4 Validate** — write + RUN units and the integration scenarios; run `cargo mutants --list -f`
  on the ACTUAL touched files for the real mutant set and kill it; full gates green.
- **P5 Complete** — CHANGELOG + docs; re-note the L0-land tiny-ticket obligation; AAR capture;
  archive; close #372.
