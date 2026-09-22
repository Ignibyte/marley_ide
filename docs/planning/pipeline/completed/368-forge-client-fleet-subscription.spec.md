---
pipeline_id: 513dea1b-6e91-439c-a2e8-5cc99f14efb5
ticket: forge#368 (9e447782-6ab1-4acb-be0e-318ff34d4455) · local docs/planning/tickets/open/TICKET-368-forge-client-fleet-subscription.md
aar_id: d8235b74-e3d9-45ac-90f6-6e8a514cb6bd
status: Phase 5 — Complete PASS
title: marley_forge_client — MCP fleet subscription + the seat_events→envelope projection (Adapter #1)
type: feature
milestone: M23
references:
  - docs/marley_architecture/orchestration-shell.md
  - docs/marley_architecture/fleet-control-plane.md
  - crates/marley_forge_client/src/lib.rs
  - crates/marley_forge_client/src/adapter.rs
  - crates/marley_app/src/settings.rs
---

## Title
Grow the EXISTING `crates/marley_forge_client` into **Adapter #1** of the fleet control plane — the only
crate allowed to know Forge's vocabulary (orchestration-shell.md §2/§7). It gains: **(a)** MCP resource
subscription to Forge's fleet resources on a standing Streamable-HTTP connection, handling
`notifications/resources/updated`; **(b)** cursor/replay catch-up (`resources/read` from the last-seen
cursor → replay into the `marley_fleet` reducer → re-subscribe); **(c)** the pure, table-driven
projection of `seat_events` rows + work-state onto `marley_fleet`'s generic `SessionEvent`/`Session`
envelope (#367's contract). Push = MCP subscription is DECIDED (orchestration-shell.md §6/§11-1):
Marley NEVER opens a Postgres connection — no raw `LISTEN`, no bespoke SSE sidecar.

**Contract-first scope (critical honesty):** Layer 0 — the seat-emitter hook + the `seat_events` table
on the Forge server — is a ucsosv2 (external) deliverable that does NOT exist yet
(orchestration-shell.md §12, "Layer 0 — not Marley"). This ticket lands the client machinery + the pure
projection + a **fake-feed integration harness** (fixture frames driven through the real
subscribe/replay code paths). Every AC row below is satisfiable WITHOUT ucsosv2.

## Scope
### In
- `crates/marley_forge_client` — grown, not replaced. The existing sprint/ticket read + write pair and
  the endpoint/bearer handling (`forge_endpoint_from`, loopback guard, Debug redaction) stay untouched.
- **(a) Subscription machinery (NEW — see Prior art leg 3: today's transport is one-shot POST +
  `Connection: close`):** the standing listen channel (Streamable-HTTP GET SSE stream), a pure
  multi-event SSE frame parser (generalizing the existing single-`data:`-line `jsonrpc_from_http`),
  pure request builders/parsers for `resources/subscribe` + `resources/read` +
  `notifications/resources/updated`, and the minimal MCP lifecycle the flow needs (D-OPEN-LIFECYCLE).
- **(b) Cursor/replay:** a persisted last-seen cursor (storage seam per D-OPEN-CURSOR-STORE, reusing
  the §14 `*_in(dir)` idiom); on (re)connect: `resources/read` from the cursor → replay rows into the
  `marley_fleet` reducer → re-subscribe. `seat_events` is a durable table; state replays from cursor.
- **(c) The projection (pure, table-driven, cov/MSI 100):** `seat_events` rows + work-state → generic
  `SessionEvent`/`Session` per the mapping table under D3. UCSOS vocabulary in, generic envelope out.
- The **fake-feed integration harness**: fixture frames (contracted event kinds, unknown kinds,
  overlap/reconnect windows, disconnects) driven through the REAL subscribe/replay/projection paths
  minus the live socket. The fixtures double as the proposed v1 wire contract handed to L0.

### Out (explicitly deferred)
- **The live ucsosv2 wire.** L0 (seat-emitter hook + `seat_events` on the Forge server) does not exist
  yet — live-wire verification against real `seat_events` is a NAMED FOLLOW-UP ticket filed at Phase 5,
  scheduled when L0 ships.
- **Any rendering.** The fleet rail is Layer-1 item ③ (orchestration-shell.md §12), a separate ticket.
- **Any write verbs.** `session.send`, dispatch, question-answering = Layer 2. This slice is read-only.
- **Postgres anything.** No `LISTEN/NOTIFY`, no DB credentials, no schema knowledge — decided
  (orchestration-shell.md §6/§11-1). `LISTEN/NOTIFY` stays Forge-internal.
- `marley_fleet` itself (#367, concurrent sibling), the `marley_mcp` expose side (Layer-1 item ④), and
  the `[[mcp.servers]]` settings round-trip (item ⑤).
- PTY scrollback replay — a different store with a different answer (`capture-pane -S -`,
  orchestration-shell.md §6); not this cursor's job.

## Reference (§20)
N/A — Marley-specific, no reference-app analog. This is protocol/adapter machinery (an MCP client
subscription + a data projection) with no user-facing surface; neither Warp nor Zed has a behavior to
match here. The governing references are the OPEN MCP specification (published material, cited below)
and Marley's own ratified architecture docs (orchestration-shell.md, fleet-control-plane.md). No
copyleft source consulted; clean-room §20 holds.

### Prior art
1. **Behavior maps (docs/warp_architecture, docs/zed_architecture):** N/A — this is protocol work, not
   UX; there is no observable Warp/Zed behavior to map. Recorded as checked-and-inapplicable.
2. **Published — the MCP specification (modelcontextprotocol.io, revision 2025-06-18; fetched
   2026-07-20).** Confirmed exactly: servers declare `capabilities.resources: { subscribe, listChanged }`
   — `subscribe` = per-resource change notifications, so subscription IS per-resource: one
   `resources/subscribe` request per URI, `params: { uri }`. The server pushes
   `notifications/resources/updated` with `params: { uri }` — the notification carries the URI only,
   NOT the content: the client follows with `resources/read` (`params: { uri }` →
   `result.contents[]: { uri, mimeType, text | blob }`). Poke-then-read is the spec's own flow.
   Standard error `-32002` = resource not found. Transport (Streamable HTTP): every client→server
   message is a NEW HTTP POST with `Accept: application/json, text/event-stream` (exactly the Accept
   line `tool_call_request` already sends, lib.rs:176); **server-initiated notifications arrive on a
   standing GET SSE stream** (client GETs the MCP endpoint with `Accept: text/event-stream`; the server
   MUST answer `text/event-stream` or 405); `Mcp-Session-Id` is assigned in the initialize response
   header and MUST be echoed on every subsequent request (404 → re-initialize); SSE event `id` +
   `Last-Event-ID` provide per-stream transport resume — which is DISTINCT from our application-level
   durable cursor (see D5).
3. **OUR DEPS / our own code (the highest-yield leg — this determines the ticket's real size).**
   `marley_forge_client` does MCP by HAND: a raw HTTP/1.1 `POST` built as a string
   (`tool_call_request`, lib.rs:157-184) over a one-shot `TcpStream` with `Connection: close` +
   read-to-EOF (adapter.rs:81-101). There is NO SDK crate — no `rmcp` anywhere in the workspace (crate
   deps: `serde` + `serde_json` only; workspace has no tokio, no async runtime). It performs NO MCP
   `initialize`, holds NO session, and has NO server-notification path — **the standing listen stream
   is genuinely NEW machinery**, not a flag flip. What IS ours to reuse: the SSE `data:`-line parsing
   (`jsonrpc_from_http`, lib.rs:189-202 — generalizes to a multi-event stream parser), the typed
   `ForgeError` family (lib.rs:83-105), the endpoint parse + loopback-only bearer guard
   (lib.rs:110-142) and the Debug redaction (lib.rs:29-36), and the pure/masked split pattern
   (decisions pure in lib.rs, socket-only masked in adapter.rs). Cursor persistence: the house idiom is
   `*_in(dir)` fns over `marley_core::marley_config_dir()` (`~/.marley/config`) —
   `settings_file_in`/`load_manager_in`, crates/marley_app/src/settings.rs:327-338, a §14 convention.
   Background drive precedent: the boot forge fetch already rides gpui's `background_executor`
   (crates/marley_app/src/app.rs:1134). Verdict: no crate we ship owns the MCP-client seam; the
   substrate is our own hand-rolled layer, and it carries the parsing + security halves already.

## Locked-In Decisions
- **D1 — Push = MCP resource subscription; never Postgres.** No raw `LISTEN`, no DB credentials, no
  bespoke SSE sidecar in Marley (orchestration-shell.md §6 + §11 decided-fork 1, chad-ratified
  2026-07-20). Postgres `LISTEN/NOTIFY` stays Forge-internal — the trigger that fires
  `notifications/resources/updated` outward.
- **D2 — The adapter boundary is absolute.** Forge/UCSOS vocabulary (`seat_events` kinds, `at-menu`,
  ticket/phase/box keys) lives ONLY in `marley_forge_client`. A Forge-specific string appearing in
  `marley_fleet` or the render path is a DEFECT (orchestration-shell.md §2, the mechanical check).
- **D3 — The projection targets #367's `marley_fleet` contract**, named as a dependency: `Session`
  {id, title, state Starting|Working|Idle|Waiting|Error|Done, question iff Waiting, labels (opaque
  string map), last_event, transport} + `SessionEvent` {upsert, state-change, question-raised,
  question-cleared, heartbeat, ended} + `FleetSnapshot`/reducer. Exact signatures are Phase-2 business
  (#367 runs concurrently). The v1 mapping table this spec locks BEHAVIORALLY:

  | `seat_events` in (fleet-control-plane.md §6) | generic envelope out |
  |---|---|
  | `session-start` | upsert — state Starting; `box`/`capabilities.*` → labels |
  | `heartbeat` | heartbeat — `last_event` advances |
  | `turn-complete` · `phase-start` · `phase-pass` · `phase-fail` · `gate-result` · `pr-opened` · `dispatch-claimed` · `car-complete` | state-change per the row's `state` field + label updates (ticket/phase/event detail → labels; Marley renders chips, never interprets) |
  | `halted-with-question` (state `at-menu`) | question-raised — state Waiting + structured `question` {prompt, options, context_refs} |
  | `api-error` | state-change → Error (Error ≠ Idle is first-class); payload status/attempt → labels |
  | `session-end` | ended — state Done |
  | row `state` values `working`/`idle`/`at-menu`/`error`/`done`/`starting` | Working/Idle/Waiting/Error/Done/Starting |

- **D4 — Contract-first verification.** The fake-feed harness (fixture frames through the real
  subscribe/replay/projection paths) is the verification substrate; its fixtures are the proposed v1
  wire contract handed to L0. Live-wire verification = the named follow-up, filed at Phase 5.
- **D5 — Catch-up = the application-level durable cursor, not SSE `Last-Event-ID`.** `seat_events` is
  a durable table; on (re)connect the client does `resources/read` from its persisted cursor → replay
  → re-subscribe (orchestration-shell.md §6: "state replays from cursor"). SSE `Last-Event-ID` is
  per-stream transport resume and MUST NOT be relied on for correctness (a restarted server has no
  stream to resume; the durable cursor always works). PTY scrollback is a different store — out.
- **D6 — Security posture inherited, extended, unchanged in kind.** Loopback-only bearer
  (lib.rs:114-119 guard), Debug-redacted bearer (lib.rs:29-36), bearer NEVER logged — every new
  request builder and stream type keeps the redaction contract. No new logging of the endpoint.
- **D7 — Unknown-input degrade is DEFINED, not accidental.** An unknown `event_type`, unknown `state`,
  or `schema` above the supported version projects to a forward-compatible observation: `last_event`
  advances, labels merge, the session's prior state is preserved (unless the row's `state` field is
  itself a known value, which still applies). Never a panic, never a dropped session.
- **D8 — Pure/masked split preserved (the crate's existing pattern).** All framing, parsing,
  subscription bookkeeping, cursor codec, and the projection are PURE (cov/MSI 100); only the live
  socket/stream pump is masked (`mutants::skip` + coverage-excluded), exactly like adapter.rs today.

**Genuinely open (decide in Phase 2):**
- **D-OPEN-CURSOR-STORE (decide in Phase 2)** — where the cursor persists: a small file under
  `marley_core::marley_config_dir()` via a new `*_in(dir)` fn (the settings.rs:327-338 idiom — leading
  candidate), vs a `marley_settings` key, vs in-memory-only + full replay per boot. Keying (per
  endpoint? per project?) rides the same fork.
- **D-OPEN-RESOURCE-NAMES (decide in Phase 2)** — `fleet://seats` + `fleet://events` are §6's
  examples, not a server contract; the actual URIs and how the cursor rides a read (URI template
  parameter vs argument) are pinned by the Phase-2 fixture design and proposed to L0.
- **D-OPEN-LIFECYCLE (decide in Phase 2)** — how much MCP lifecycle the client implements: today's
  sidecar answers bare `tools/call` with no `initialize`; the spec-correct subscription flow wants
  `initialize` (capability check: `resources.subscribe`) + `Mcp-Session-Id` echo +
  `MCP-Protocol-Version`. Implement spec-correct against the fake feed, or the sidecar's observed
  minimum? (Leaning spec-correct — the fake feed can enforce it; the live sidecar confirm is the
  follow-up's business.)
- **D-OPEN-STREAM-DRIVE (decide in Phase 2)** — who pumps the standing GET stream: a dedicated
  `std::thread` owned by the adapter vs gpui's `background_executor` at the app seam (the boot forge
  fetch's existing pattern, app.rs:1134). The crate has no async runtime; both are viable.

## Acceptance Criteria (EARS)
All rows are satisfiable WITHOUT ucsosv2 (D4). "Harness" = the fake-feed integration harness driving
fixture frames through the real subscribe/replay/projection code paths.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a `session-start` row is projected, the adapter shall emit an upsert whose session has state Starting, id = the row's `session_id` (never a screen/window index), and `box` + `capabilities.*` carried as opaque labels. | unit vs the D3 table |
| REQ-002 | WHEN a row carries a known `state` (`working`/`idle`/`done`/`starting`), the projection shall yield the corresponding envelope state (Working/Idle/Done/Starting). | unit — all four values |
| REQ-003 | WHEN a `halted-with-question` row (state `at-menu`) is projected, the adapter shall emit question-raised with the payload's prompt/options/context refs as the structured question, and the session's state shall be Waiting. | unit + reducer snapshot |
| REQ-004 | WHEN a later row moves a questioned session out of `at-menu`, the projected stream driven through the #367 reducer shall leave that session's `question` null (question-cleared observable). | harness through the reducer |
| REQ-005 | WHEN an `api-error` row is projected, the session's state shall become Error — distinct from Idle — with the payload's status/attempt detail in labels. | unit |
| REQ-006 | WHEN any row carries `ticket`/`phase`/`box`/`capabilities`, the projection shall pass them ONLY as opaque string labels; no Forge/UCSOS-specific string shall appear in `marley_fleet` or the render path. | unit + boundary grep review (D2) |
| REQ-007 | WHEN a row carries an unknown `event_type`, an unknown `state`, or a `schema` above the supported version, the projection shall apply the D7 degrade (advance `last_event`, merge labels, preserve prior state) and shall not panic. | unit — future-kind fixtures |
| REQ-008 | WHEN the client (re)connects holding a persisted cursor, it shall `resources/read` from that cursor and replay the returned rows into the reducer BEFORE re-subscribing, so no gap-window row is lost. | harness — kill/reconnect fixture |
| REQ-009 | WHEN a replay window overlaps rows already applied, driving it through the reducer shall be idempotent — the resulting `FleetSnapshot` equals the no-overlap snapshot (resume, never duplicate). | harness — equal-snapshot assertion |
| REQ-010 | WHEN a subscribed fleet resource changes, the client shall react to `notifications/resources/updated` by issuing `resources/read` for that URI, and the new rows shall reach the reducer (subscribe → notify → re-read). | harness — lifecycle fixture |
| REQ-011 | WHILE any subscription/replay/error path introduced by this ticket executes, the bearer shall never appear in a log, `Debug` render, error string, or panic message (the lib.rs:29-36 redaction contract extends to every new type). | review + repo grep; redaction unit extended to new types |
| REQ-012 | WHEN Forge is unreachable (connect refused, 405 on the listen GET, mid-stream disconnect), the client shall surface a typed `ForgeError` — no panic, no `unwrap()`/`expect()` on any response-reachable path (§14) — and a subsequent reconnect shall resume from the cursor per REQ-008. | unit on pure classifiers + harness disconnect fixture + clippy/review |

## Testing boundary (honest)
The pure seams — SSE multi-event framing, subscribe/read/notification builders + parsers, the cursor
codec, the projection table + D7 degrade — carry cov/MSI 100 (run `cargo mutants --list` on the ACTUAL
code; the projection returns enum/Vec shapes whose body-mutant viability depends on `Default` derives —
the #203/#204 lesson). The live socket + standing-stream pump is ACCEPTED-UNTESTABLE and masked
(`mutants::skip` + coverage-excluded), mirroring adapter.rs today; watch the mutants-skip detach trap
when inserting fns near the masked shim. The fake-feed harness drives the real code paths minus the
socket. Live `seat_events` verification is explicitly the follow-up ticket's job (L0 absent — D4).

## Phase Plan
- **P2 Design** — pin the pure seam inventory + exact fn shapes against #367's landed types; design the
  fake-feed harness + fixture frames (the frames ARE the proposed v1 wire contract for L0); resolve
  D-OPEN-CURSOR-STORE, D-OPEN-RESOURCE-NAMES, D-OPEN-LIFECYCLE, D-OPEN-STREAM-DRIVE with evidence.
- **P3 Implement** — pure seams first (framing, builders/parsers, cursor codec, projection), then the
  masked stream pump; no app-side rendering (out).
- **P3.5 Inspect** — independent critics vs the diff: the D2 boundary grep, the REQ-011 bearer grep,
  §18.1 provenance, idempotence-edge attack (overlap windows, duplicate notifications, cursor
  regression), unwrap-on-response-path sweep.
- **P4 Validate** — write + RUN the units + harness fixtures; cov/MSI 100 on pure seams; full gates
  green.
- **P5 Complete** — CHANGELOG + docs; file the NAMED follow-up (live-wire verification against real
  `seat_events` when L0 ships); AAR capture; archive; close #368.
