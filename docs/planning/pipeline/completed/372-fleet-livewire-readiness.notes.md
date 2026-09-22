# Fleet L0 live-wire readiness — Notes

- **Forge ticket:** #372 23b95391-d4b3-4739-9581-bb6fb39b4f89 (chore, sprint #34 "M23.5 — Fleet Layer-1 Consolidation")
- **AAR:** 5067cea0-438b-4218-8572-e087c7065c29 (opened at /work)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-372-fleet-livewire-readiness.md
- **Pipeline spec:** 372-fleet-livewire-readiness.spec.md

## Phase 1 — Plan
- **Request:** The #368 Phase-5 named follow-up ("live-wire verification vs real `seat_events`",
  368.notes:378) — RESPEC'd by the batch owner because Layer 0 (ucsosv2 seat-emitter + the
  `fleet://events` resource on the real Forge server) does not exist and is blocked indefinitely.
  #372 becomes the live-wire READINESS slice: fixture forge server, real-socket integration test of
  the real pump, typed 404/disconnect signals (echo already shipped), going-live runbook. The
  literal real-run = a tiny ticket minted at L0-land.
- **Classification / tier:** chore; medium. New test infrastructure (the workspace's FIRST
  loopback-server test fixture) + a small pure delta (exit-reason enum + classifiers) + store-only
  masked pump edits + a docs subsection. No UI, no new behavior policy (retry = #373).
- **Forge recall (§18.3):** #368 shipped the pump as ACCEPTED-UNTESTABLE with the fake-feed harness
  as its only proof (368.spec "Testing boundary"); its inspect accepted "no auto-reconnect after a
  stream close" as a LOW risk deferred to follow-up (368.notes:270) — that deferral is #373; the
  typed signal it needs is THIS ticket. #371 shipped `[[projects.orchestration]].brain_endpoint`
  (the runbook's settings row). Standing lessons applied: mutants-skip detach trap (fns near masked
  shims); PR-claude-trace-the-real-cargo-mutants-list (run `--list -f` on ACTUAL files at Validate);
  #203/#204 Default-derive viability for the new enum's body mutants; house rule — test code carries
  no never-run branches.
- **Discovery (the sweep's load-bearing evidence — file:line):**
  - Pump wire dialect: `fetch` adapter.rs:93-113 (raw HTTP/1.1 POST over `TcpStream`, 5s timeouts,
    read-to-EOF on `Connection: close`); `mcp_post` lib.rs:198-221 (bearer + `MCP-Protocol-Version`
    + `session_header` lib.rs:242-247 + `Content-Length`); `mcp_listen_get` lib.rs:225-239 (GET,
    `Accept: text/event-stream`, NO `Connection: close`); `open_listen_stream` adapter.rs:287-304
    with the **1s read timeout** (:298) as the stop-flag wakeup; `jsonrpc_from_http` lib.rs:252-265
    (2xx status + first `data:` line); `parse_initialize` fleet.rs:229-246 reading the
    `Mcp-Session-Id` response header (`header_value` :250-260).
  - Pump sequence: `run_subscription` adapter.rs:166-252 — initialize → `supports_subscribe` gate →
    initialized notification → **refresh BEFORE subscribe** (:196-202) → second refresh (:204-206) →
    GET stream → per-notification refresh. Read loop: `Ok(0)` break :219, timeout continue
    :224-231, hard error break :232 — NO surfaced reason. UTF-8 byte-boundary buffering :215-243.
  - Session echo ALREADY SHIPPED: capture fleet.rs:230; echo via every builder; pump threads it
    adapter.rs:188-206; pinned fleet.rs:865 + lib.rs:660-682. Missing: typing — 404 is stringly
    `ForgeError::Http("HTTP/1.1 404 Not Found")` (lib.rs:253-255, `ForgeError` variants :90-99).
  - **Latent gap found:** `open_listen_stream` never checks the GET response status; the HTTP header
    block is consumed by `parse_sse_frames` as a no-`data:` frame and silently skipped
    (fleet.rs:41/:72) — a 404/405 GET idles forever. Fold into the D6 typed-exit design.
  - Frozen contract: `SeatEventRow`/`FleetPage` fleet.rs:91-115/:119-127 (`ts_ms: u64`); `read_uri`
    :207-212; fixture JSONs fleet.rs:1030 + :1063-1137; `FleetSync` :441-490 (the expected-snapshot
    path for REQ-004).
  - Integration precedent: marley_lsp/tests/integration.rs:1-3 (REAL adapter vs scripted `fake_ls`),
    bounded polls :18-49, `CARGO_BIN_EXE_fake_ls` :15 (bin NOT needed here — in-process thread
    suffices); nextest runs `tests/` workspace-wide gates.sh:83; adapter.rs +
    marley_mcp/src/transport.rs already coverage-excluded gates.sh:221-222; **no `TcpListener` in
    any test today** (only app code: transport.rs:82 binds `127.0.0.1:0`, accept loop :95-107);
    `tempfile` dev-dep precedent marley_settings/Cargo.toml + tests/settings.rs:25-28.
  - App truth for the runbook: `fleet_snapshot: Option<FleetSnapshot>` app.rs:387, `None` until "the
    `fleet-demo-feed` verb, or Layer-1 ②'s live client" (:386, :2166) — `FleetSubscription` has NO
    app-side caller yet (workspace grep) → the runbook must name wiring + #373 as preconditions.
  - Crate deps: marley_forge_client = serde/serde_json/marley_fleet + dev-dep mutants only; no tokio
    anywhere in the workspace (368.spec:92-93).
- **Decisions:** D1 respec (readiness now / real-run deferred); D2 fixture = `std::net::TcpListener`
  + thread, test-only, no async (rejected tokio dev-dep, rejected stream mocks); D3 lives in NEW
  `crates/marley_forge_client/tests/livewire.rs` per the marley_lsp precedent (rejected
  `#[cfg(test)]`-in-src, rejected a fixture bin); D4 mirror the pump's ACTUAL dialect only; D5
  session scope = typed 404 signal only (echo shipped; recovery #373); D6 minimal typed exit-reason
  seam (pure enum + classifiers cov/MSI 100; pump store-only; also closes the silent-GET gap); D7
  integration test = behavior proof, pump stays masked; D8 runbook in orchestration-shell.md with
  honest preconditions. Full rationale + rejected alternatives in the spec.
- **Open questions for Phase 2:**
  1. **Exit-reason surfacing:** poll-able `exit_reason()` on `FleetSubscription` (an
     `Arc<Mutex<Option<…>>>` sibling of `stop`) vs an `on_exit` callback parameter on `start`.
     Leaning poll-able — no new callback type, `Drop` already joins, and #373 will poll anyway.
  2. **Where the 404 classifier sits:** a new `ForgeError::SessionExpired` variant (ripples every
     `match`/caller) vs a standalone pure `fn` over the response/status line feeding the exit enum
     only (leaning standalone — the sprint/ticket read path doesn't hold sessions and shouldn't
     change shape).
  3. **GET-status validation:** have `open_listen_stream` (or a pure pre-check on the first bytes)
     reject a non-2xx GET with the typed reason — decide the seam so the check itself is pure and
     the 1s-timeout loop is untouched.
  4. **Fixture server structure:** one scripted server per test (scenario table passed at spawn,
     recording (method, uri, session-header) tuples + connection count) vs a shared server — leaning
     per-test (isolation, no port reuse hazards, `127.0.0.1:0` makes it free). The GET must be
     served on its own thread while later POST connections keep arriving.
  5. **Fixture literals:** duplicate the frozen JSON pages in livewire.rs (pinning by copy — the
     fleet.rs `#[cfg(test)]` fixtures aren't importable from an integration crate) vs promote them
     to `include_str!` files — leaning duplicate-with-a-comment (they are the CONTRACT; a drift is a
     test failure we want loud).
  6. **Batch-owner flag:** which ticket owns the app-side `FleetSubscription::start` wiring
     (`fleet_snapshot` feed) — #373 or its own slice? The runbook precondition list needs the
     answer's ticket number; drafted assuming #373's lane.
  7. Timing budget: bounded polls sized around the pump's 1s read-timeout wakeup (a stop can take
     ~1s to notice) — keep total scenario deadlines ≤ ~5s so the suite stays fast under nextest.

## Phase 2 — Design

### Architecture / approach
The slice has three moving parts: a small **pure typed-signal delta**, a **store-only masked pump edit**,
and a **test-only fixture forge server** that drives the real pump over a real socket. §20 holds — this is
Marley-own infra with no reference-app analog (confirmed still `N/A`); the governing contracts are the MCP
2025-06-18 spec + our shipped #367/#368 code.

**① Pure typed-signal seam (fleet.rs + lib.rs — cov/MSI 100).** Converts the pump's currently-silent
terminations into typed, tested decisions:
- `lib.rs`: `pub fn http_status_code(response: &str) -> Option<u16>` — the status code off a raw response's
  first line (`HTTP/1.1 404 Not Found` → `404`). `status_is_2xx` is refactored to read through it (one
  status-parse, not two). Pure.
- `fleet.rs`:
  - `pub enum SubscriptionExit { Stopped, StreamClosed, Disconnected, SessionExpired, HandshakeFailed }`
    (`Debug, Clone, Copy, PartialEq, Eq`). The typed terminal reason a connection attempt ends with —
    the signal #373's reconnect policy will consume (`Stopped` ⇒ don't reconnect; the rest ⇒ reconnectable;
    `SessionExpired` ⇒ re-initialize).
  - `pub enum ReadStep { Bytes(usize), Closed, Retry, HardError }` + `pub fn read_step(read: Result<usize,
    std::io::ErrorKind>) -> ReadStep` — LIFTS the masked read-loop disposition (adapter.rs:218-233:
    `Ok(0)`→Closed, `Ok(n)`→Bytes, `WouldBlock|TimedOut`→Retry, other→HardError) into a pure classifier.
    The pump becomes `match read_step(stream.read(&mut chunk).map_err(|e| e.kind())) { … }`.
  - `pub fn classify_http_failure(response: &str) -> SubscriptionExit` — `404` ⇒ `SessionExpired`, any other
    non-2xx ⇒ `Disconnected` (via `http_status_code`). Used both for the latent GET-status gap (a non-2xx
    GET response now surfaces instead of idling forever — Prior-art leg 3) and a mid-stream refresh 404.
- **Surfacing (open-Q1 resolved → poll-able):** `FleetSubscription` gains `exit: Arc<Mutex<Option<
  SubscriptionExit>>>`; the thread stores the terminal reason once on return; `pub fn exit_reason(&self)
  -> Option<SubscriptionExit>` reads it (Copy, lock-and-copy). Rejected an `on_exit` callback: no new
  callback type, `Drop` already joins, and #373 will poll this exact accessor. (open-Q2 resolved → a
  standalone classifier, NOT a new `ForgeError::SessionExpired` variant: the sprint/ticket read path holds
  no session and must not change shape; the 404 is typed at the fleet layer only. open-Q3 resolved → the
  GET-status check is a one-time pure `http_status_code`/`classify_http_failure` call on the first bytes,
  leaving the 1s-timeout loop untouched.)

**② Masked pump edit (adapter.rs — `mutants::skip` + coverage-excluded, unchanged floor posture).** The
init→subscribe→listen body is extracted to `fn run_once(endpoint, config_dir, stop, on_update) ->
SubscriptionExit` — **this is the seam #373 wraps in its reconnect loop** (D6/#373 hard-order). Each
termination maps to a reason: handshake steps (bad url / unreachable / `parse_initialize` Err /
`!supports_subscribe` / listen-open Err) → `HandshakeFailed`; the read loop → `read_step`
(`Closed`→`StreamClosed`, `HardError`→`Disconnected`, `Retry`→continue, stop-flag→`Stopped`); a GET whose
first bytes are a non-2xx status → `classify_http_failure`; a refresh POST answered non-2xx →
`classify_http_failure` (404→`SessionExpired`, surfaced + stop, NO re-initialize — that's #373).
`run_subscription` calls `run_once` once and stores the result in `exit`. The integration test is the
behavior proof that the masked wiring maps the right reason (REQ-008/009) — not a mutation target (D7).

**③ Fixture forge server (tests/livewire.rs, NEW — test code, no floor, house never-run-branch rule).**
A `std::net::TcpListener` bound `127.0.0.1:0` + a **thread-per-connection accept loop** (the transport.rs:82/
:95-107 idiom — required because the pump holds the GET open while new POST connections keep arriving). A
`FixtureForge::spawn(Script)` returns the bound `ForgeEndpoint` (built via `forge_endpoint_from` on a
`.mcp.json` string carrying the ephemeral port — `127.0.0.1:<port>` passes the loopback guard) plus shared
handles: `Arc<Mutex<Vec<RequestRecord{method,uri,session}>>>` (order asserts), an `Arc<AtomicUsize>`
GET-connection counter (REQ-008 no-retry), and a scripted read-response queue. Per-connection handler: parse
the request line + headers + body; POST `initialize` → `200` + `Mcp-Session-Id: S` + `capabilities.resources.
subscribe=true`; POST `notifications/initialized` → `200` empty; POST `resources/read` → `200` + the next
scripted `FleetPage` JSON (byte-for-byte the frozen #368 contract, `ts_ms` u64); POST `resources/subscribe`
→ `200` ok; GET → held open, emits scripted SSE `notifications/resources/updated` frames (REQ-005 splits one
across ≥3 `write`+`flush` segments), then per-scenario closes mid-stream (REQ-008) or answers non-2xx
(the GET-status / 404 scenarios). Bounded deadline polls on the `on_update`-collected snapshots (the
integration.rs:18-49 shape), total ≤ ~5s (sized around the pump's 1s GET read-timeout wakeup). Fixture
literals are DUPLICATED from fleet.rs:1030/:1063-1137 with a "this IS the frozen contract; drift = a loud
test failure" comment (open-Q5 resolved — the `#[cfg(test)]` fixtures aren't importable from an integration
crate; per-test server, open-Q4 resolved — isolation, `:0` makes it free).

### File manifest
| # | File | Change |
|---|---|---|
| 1 | `crates/marley_forge_client/src/lib.rs` | ADD `http_status_code`; refactor `status_is_2xx` through it; re-export `SubscriptionExit`, `ReadStep`, `read_step`, `classify_http_failure` |
| 2 | `crates/marley_forge_client/src/fleet.rs` | ADD the two enums + `read_step` + `classify_http_failure` + their unit tests (pure, cov/MSI 100) |
| 3 | `crates/marley_forge_client/src/adapter.rs` | MODIFY (masked): extract `run_once -> SubscriptionExit`; add `exit` field + `exit_reason()`; wire the pure classifiers; the GET-status one-time check |
| 4 | `crates/marley_forge_client/tests/livewire.rs` | NEW — fixture server module + REQ-001..009 integration scenarios |
| 5 | `crates/marley_forge_client/Cargo.toml` | ADD `tempfile = "3"` dev-dep (cursor `config_dir`) |
| 6 | `docs/marley_architecture/orchestration-shell.md` | ADD the "Going live (L0-day) runbook" subsection |
| 7 | `CHANGELOG.md` | Added entry |

### Regression Test Plan
| # | Test | Kind | Proves |
|---|---|---|---|
| U1 | `http_status_code`: `200 OK`→200, `404 Not Found`→404, empty→None, `HTTP/1.1`(no code)→None | pure unit (lib.rs) | classifier input |
| U2 | `read_step`: `Ok(0)`→Closed, `Ok(5)`→Bytes(5), `Err(WouldBlock)`→Retry, `Err(TimedOut)`→Retry, `Err(Other)`→HardError | pure unit | REQ-010, read disposition |
| U3 | `classify_http_failure`: 404-resp→SessionExpired, 500-resp→Disconnected, no-status→Disconnected | pure unit | REQ-009 classify |
| U4 | `SubscriptionExit`/`ReadStep` equality+Copy derive coverage | pure unit | REQ-010 arms |
| I1 | lifecycle order recorded: initialize, initialized, read, subscribe before the GET | integration | REQ-001 |
| I2 | every POST + the GET carries `Mcp-Session-Id: S` server-side | integration | REQ-002 |
| I3 | pre-persisted cursor `K` → first read `?since=K` before subscribe | integration | REQ-003 |
| I4 | final delivered `FleetSnapshot` == same pages through in-process `FleetSync` | integration | REQ-004 |
| I5 | notification split across ≥3 TCP segments → parsed + follow-up read + snapshot matches | integration | REQ-005 |
| I6 | fixture rows byte-identical to the frozen contract + row-content assert on the snapshot | integration | REQ-006 |
| I7 | stop after cursor→C2, restart → reads `?since=C2`; overlap replay == no-overlap snapshot | integration | REQ-007 |
| I8 | fixture closes the GET mid-run → thread ends within bound, no panic, GET-conn-count stays 1, `exit_reason()==StreamClosed` | integration | REQ-008 |
| I9 | a post-init request answered 404 → `exit_reason()==SessionExpired`, no second `initialize` recorded | integration | REQ-009 |
| G1 | `cargo mutants --list -f` on lib.rs+fleet.rs touched fns → kill the real set; gate green | gate | REQ-010 |
| D1 | grep the runbook heading + the (a)–(e) checklist items in orchestration-shell.md | docs presence | REQ-011 |

### Risks / decisions
- **Timing flake** — bounded deadline polls (never fixed sleeps for correctness), generous ~5s deadlines,
  prompt fixture responses. The pump's 1s GET read-timeout means a `stop` can take ≤~1s to notice → sized in.
- **GET held open + concurrent POSTs** — thread-per-connection in the fixture (not a single accept-respond
  loop), mirroring transport.rs.
- **Port/isolation** — `127.0.0.1:0` ephemeral, per-test server; no shared global state across tests.
- **`Arc<Mutex<Option<SubscriptionExit>>>`** — first `Mutex` in this crate; `SubscriptionExit: Copy` so the
  accessor is lock-copy-unlock, no borrow held. Rejected `AtomicU8` (opaque encoding for zero gain).
- **`run_once` seam** — deliberately shaped as the single-attempt function #373 wraps; a design choice that
  pre-pays the hard-ordered next ticket without widening #372's scope (the loop itself is #373).

## Phase 3 — Implement
- **Built exactly to the manifest.** Pure seams first (`lib.rs::http_status_code` + `status_is_2xx`
  refactor-through-it; `fleet.rs::{SubscriptionExit, ReadStep, read_step, classify_http_failure}` + their
  unit tests), then the masked pump (`adapter.rs`: `run_subscription` → thin wrapper storing the exit;
  `run_once -> SubscriptionExit` the extracted single-attempt seam #373 wraps; `refresh -> Option<
  SubscriptionExit>`; the one-time GET-status gate closing the latent silent-idle gap; `exit` field +
  `exit_reason()`), then the fixture + 10 scenarios (`tests/livewire.rs`), then the runbook
  (orchestration-shell.md §12.1).
- **Deviations from design:** none material. The `run_once` GET-status gate waits for a complete first
  line (`buffer.contains(&b'\n')`) before parsing the status — added to avoid mis-classifying a partial
  status line mid-number (a subtlety not spelled out in the design; the safe reading of open-Q3).
- **Compile + run:** `cargo check -p marley_forge_client --tests` clean; 39 lib unit tests pass; all 10
  livewire integration scenarios pass, **stable across 5 back-to-back runs** (no timing flake) in ~1.13s.
- **Floor posture unchanged:** adapter.rs stays `mutants::skip` + coverage-excluded (gates.sh:222); the
  new pure fns carry their own unit tests; the fixture is test code (the `_ =>` POST arm genuinely runs
  for `initialized`+`subscribe`, so it is not a dead branch — the house rule holds).

## Phase 3.5 — Inspect
Three independent critics over the diff (correctness · no-retry/floors · security/reuse/docs). No
high/medium defects. Seven LOW/NIT findings — all confirmed real and FIXED (each cheap, several genuinely
strengthen the slice). Lenses covered: pure-classifier correctness, the masked pump's reason-mapping,
fixture fidelity + concurrency, no-retry/no-idle honesty, mutants-skip attachment, never-run branches,
bearer-redaction, reuse, runbook honesty.

| # | Finding | Verdict | Fix |
|---|---|---|---|
| F1 | **The GET-status gate strict-decoded the whole buffer** (`from_utf8().unwrap_or("")`) — an invalid body byte would discard the ASCII status line and MISS a non-2xx GET (the exact latent gap #372 closes). Self-inconsistent with the valid-prefix decode 7 lines below. | REAL (low) | adapter.rs: decode the VALID prefix (matching the main loop); the status line always sits in it. |
| F2 | **`get_rejection` test closed the socket** after the non-2xx header, so a gate-*less* pump would surface `StreamClosed` (from the close), not idle — the test guarded classification but not the no-idle property it claims. | REAL (low) | livewire.rs: HOLD the rejected GET open (4s) so a regression that dropped the gate idles to the deadline → the expect fails. Test now genuinely guards no-idle. |
| F3 | **The `(200..300)` failure decision was inlined+duplicated** in the masked pump (2 sites), contradicting the module's own "every decision is a pure `fleet` call" invariant. | REAL (low) | Extracted pure `classify_if_failure(response) -> Option<SubscriptionExit>` (fleet.rs, cov/MSI 100 via a new unit test); both pump sites collapse to one call. |
| F4 | **Two redundant dev-deps** (`serde_json`, `marley_fleet`) — already normal `[dependencies]`, hence visible to the integration test. | REAL (low) | Removed; verified the suite still compiles + passes with only `tempfile` added. |
| F5 | **`ForgeClient::new` lacked `mutants::skip`** — currently harmless (no viable mutant), but a future `Default` derive would make it mutation-live while coverage-excluded (a latent mismatch). | REAL (nit) | Added the skip for consistency with every other fn in the excluded adapter. |
| F6 | **No test asserted the `Stopped` exit path** (correct by inspection, but `stop()`→`Stopped` unproven — #373 depends on it). | REAL (nit) | Added `stop_surfaces_stopped_reason` (stop() while the GET is held open → `exit_reason()==Stopped`). |
| F7 | **Runbook headline overclaimed** ("not new Marley code") vs its own precondition 1 (app-side wiring IS unbuilt Marley code). | REAL (low) | Softened to "the client/protocol layer is live-wire ready; app-wiring + #373 remain preconditions". |

Rejected/confirmed-sound (no change): REQ-008 `get_connection_count()==1` IS a sound no-retry proxy (the only GET call site runs once, no wrapping loop); REQ-009 "exactly one initialize" IS sound (no re-initialize path exists in `run_once`); `refresh` skipping a malformed-2xx page is the intended tolerance (a hard error is caught first as Disconnected, a non-2xx classified first); no bearer leak in any new type (`SubscriptionExit`/`ReadStep` carry no secret, `FleetSubscription` has no Debug); the fixture stays loopback via the real `forge_endpoint_from`; clean-room §20 holds (hand-rolled std-only).
- **Post-fix:** 11 integration + 41 lib unit tests green; stable.

## Phase 4 — Validate
- **Tests written + RUN:** 4 new pure unit tests (`t372_http_status_code_parses_first_line`,
  `t372_read_step_arms`, `t372_classify_http_failure_arms` incl. `classify_if_failure`) + 11 integration
  scenarios in `tests/livewire.rs` (REQ-001..009 + the GET-rejection gap + the Stopped path). `cargo test
  -p marley_forge_client`: **41 lib unit + 11 integration + 0 doctest, all pass**; integration suite stable
  across 5 back-to-back runs (no timing flake), ~1.13s.
- **Mutants (the real set via `cargo mutants --list -f`):** lib.rs `http_status_code` (None/Some(0)/Some(1))
  + `status_is_2xx` (true/false) and fleet.rs `classify_http_failure` (delete-arm Some(404)) +
  `classify_if_failure` (guard true/false/delete-!, body-None) are all VIABLE and KILLED by the two new
  unit tests. The `read_step`/`classify_*` `Default::default()` body-mutants are UNVIABLE (no `Default`
  derive — the #203/#204 rule). adapter.rs yields **0 mutants** (fully `mutants::skip` masked +
  coverage-excluded, gates.sh:222).
- **Full gate `scripts/gates.sh --diff`: GATE GREEN [diff] — 15/15** (coverage ≥100% lines, mutation MSI
  ≥100%, clippy -D warnings, machete confirms the redundant dev-deps were correct to drop, gitleaks/audit/
  deny/miri/visual all green). Receipt written for /commit.
- **No live-app drive:** `marley_forge_client` is a library crate with no UI/render/input surface (§7
  N/A — the fixture-driven integration suite IS the behavior proof for the masked pump).
- **Pre-existing:** none touched; no unrelated breakage.

## Phase 5 — Complete
- **CHANGELOG:** entry added under Added (M23.5 ①).
- **Architecture docs:** `orchestration-shell.md` §12.1 "Going live (L0-day) runbook" (REQ-011) + a §12
  M23.5 consolidation breadcrumb (#372→375).
- **Knowledge captured (forge):** failure `BF-372-getstatus-gate-strict-decode` (F1); prevention rule
  `PR-claude-gap-fix-test-must-reproduce-the-original-failure-001` (F2 — the reusable lesson: a gap-fix
  regression test must reproduce the ORIGINAL failure condition, litmus = delete-the-fix-and-it-goes-red);
  AAR `5067cea0` submitted.
- **L0-land obligation re-noted:** the literal real-ucsosv2 (a)–(e) verification = a tiny ticket to mint
  when Layer 0 ships (its checklist IS the §12.1 runbook). App-side `FleetSubscription` wiring + reconnect
  are #373's lane (flagged in the runbook preconditions).
- **Ticket #372 closed + pipeline archived to completed/.**
