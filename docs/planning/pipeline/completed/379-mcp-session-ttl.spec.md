---
pipeline_id: a53a0c39-62cd-457a-bcca-4ea73e10d318
ticket: forge#379 (c5b91d70-f42e-4caf-9e47-9d9a7de7f80d) · local docs/planning/tickets/open/TICKET-379-mcp-session-ttl.md
aar_id: 3af8c68c-ca04-46c4-8717-90d5fe02ef8b
status: Phase 5 — Complete PASS
title: marley_mcp — session TTL sweep: idle-session expiry (#375 hardening follow-up)
type: chore
milestone: M24
references:
  - docs/planning/pipeline/completed/375-marley-mcp-hardening.spec.md
  - docs/marley_architecture/orchestration-shell.md
  - crates/marley_mcp/src/session.rs
  - crates/marley_mcp/src/transport.rs
  - crates/marley_mcp/src/auth.rs
  - crates/marley_fleet/src/attention.rs
  - crates/marley_forge_client/src/fleet.rs
---

## Title
Close the last slot-leak wedge in #375's bounded session registry. #375 shipped cap 8,
reject-new-never-evict (D4) WITH stream-drop reclamation: a session whose standing GET stream hangs
up is reaped (`reap_session`, transport.rs:221-228, called from the SSE write-failure arms,
transport.rs:260/:279). The remaining wedge: a client that `initialize`s but never opens a GET
stream and never DELETEs — its slot has NO reclamation path and lingers until app restart; 8 such
ghosts = a full registry refusing every new manager (503 at transport.rs:174). Fix = idle expiry,
three small pieces: (1) a **last-seen timestamp** per session, stamped on assign and refreshed on
every successful validate — via an **injected clock** (`now: u64`), the `marley_fleet::attention`
idiom (attention.rs:1-3, is_stale :37-39); (2) a **lazy sweep** of idle-≥-TTL sessions run
pre-dispatch inside `serve_connection`'s existing session-gate lock block (transport.rs:147-181) —
NO background timers; (3) an expired id thereby becomes ABSENT, so the ALREADY-SHIPPED
`Reject(404)` arms answer it (session.rs:106/:115) — zero wire change; the client half is already
proven to recover (404 → `SubscriptionExit::SessionExpired`, fleet.rs:141 → #373 re-initializes,
adapter.rs:316). **Strictly additive to #375**: the `session_decision` table and every #375 test
expectation are unchanged.

## Scope
### In
- **`last_seen_ms` per session.** The registry's `ids: Vec<String>` (session.rs:23) grows a
  timestamp per entry (shape — tuple vs small struct — is Phase 2's pick). The manual redacting
  `Debug` (session.rs:26-34) keeps redacting ids (`***`); the live COUNT stays shown; timestamps
  are not secrets.
- **Touch points = assign + every successful validate.** `assign` stamps `now`; a request that
  passes session validation (POST `Proceed`, the GET stream open, DELETE `Terminate`'s lookup)
  refreshes its session's last-seen. NOT touched: per-SSE-emit (see D5 for the accepted
  consequence + recovery proof).
- **`SESSION_TTL_MS` + sweep.** A named const beside `SESSION_CAP` (session.rs:12), proposed
  **30 min = 1_800_000 ms** (value D-OPEN-TTL-VALUE). A pure `sweep` removes every session with
  `now.saturating_sub(last_seen) >= SESSION_TTL_MS` (the is_stale boundary shape, attention.rs:38).
  Run site: pre-dispatch in `serve_connection`, INSIDE the existing session-gate mutex block
  (transport.rs:147-181), AFTER the cap/Origin/bearer guards and BEFORE `session_decision` — so an
  expired id is already gone when the gate decides, and the shipped unknown-id 404 answers it.
- **Injected clock end-to-end.** Every pure entry point that reads or stamps time takes
  `now: u64` (epoch ms). The ONLY real-clock read is one new masked helper in transport.rs beside
  `read_entropy` (transport.rs:79-87 — the masked-3-line-helper precedent).
- **Regression pins (every #375 invariant):** cap-8 reject-new for LIVE sessions (a fresh-8
  registry still refuses the 9th — sweep removes nothing fresh); `ct_eq` validation
  (auth.rs:43-53, unchanged); redacted Debug; stream-drop reclamation (`reap_session` of an
  already-swept id = the idempotent-terminate no-op, session.rs:58-62); DELETE terminate; guard
  order cap → Origin → bearer → session → dispatch with the sweep POST-auth (an unauthenticated
  request still never touches the registry — #375 REQ-012 verbatim).

### Out (explicitly deferred)
- LRU / pressure-based eviction — #375 D4 (reject-new, never evict a live session) stands; TTL
  expiry removes only genuinely idle sessions, never to make room under load.
- A configurable TTL setting — #371's settings surface untouched; the TTL is a named in-code const.
- Metrics/logging of expiry events — no new log paths; the bearer and session ids are NEVER logged
  regardless (#375 D8/REQ-013 extends over the new code).
- Touch-on-SSE-emit / stream-liveness exemptions from expiry (D5 documents the accepted behavior).
- Client-half changes — #372/#373 shipped the 404 → re-initialize recovery; it is the proof this
  leans on, not a surface it edits.
- Background sweeper threads / timers of any kind (D1).

## Reference (§20)
**N/A — Marley-specific hardening; no Warp/Zed behavior analog** (the #375 sweep already
established this: Warp's map covers an MCP **client** runtime, Zed has no MCP material — there is
no terminal/editor BEHAVIOR to match for a self-hosted server's session expiry). The behavior
contract is **published material**: the MCP specification rev 2025-06-18, "Transports → Streamable
HTTP → Session Management" — session timeout/expiry is explicitly server discretion, and the 404
signal + client re-initialize is the spec's own recovery loop (Prior art leg 2). Clean-room §20
untouched: no Warp (AGPL) / Zed (GPL) source is relevant or consulted.

### Prior art
1. **Behavior maps — checked, N/A.** Unchanged from the #375 sweep: `docs/warp_architecture/crates/mcp.md`
   maps Warp's MCP **client** (connects OUT; OAuth per server) — no self-hosted-server session-TTL
   surface to observe; `docs/zed_architecture/` has no MCP material. Stated per the sweep rule.
2. **Published — the MCP specification rev 2025-06-18, "Transports → Streamable HTTP → Session
   Management":** the server **MAY terminate the session at any time**, after which it **MUST**
   respond **404 Not Found** to requests containing that session id; the client receiving 404
   **MUST** start a new session by sending a new InitializeRequest. The spec is deliberately silent
   on WHEN/WHY a server terminates — an idle TTL is squarely inside that discretion, and it needs
   NO new protocol surface: the 404 we already emit IS the expiry signal.
3. **OUR OWN shipped code (the load-bearing leg — the audit that justifies the ticket):**
   - **The wedge, traced:** the ONLY reclamation paths today are DELETE-terminate
     (transport.rs:160-166) and stream-drop reap (`reap_session` transport.rs:221-228, fired from
     the SSE write-failure arms :260/:279). A client that initializes (slot assigned,
     transport.rs:167-177) and never opens a GET stream hits NEITHER — the slot is immortal. At
     `SESSION_CAP` (session.rs:12) every new initialize gets 503 (transport.rs:172-176).
   - **The registry (session.rs):** `ids: Vec<String>` (:23); `assign` (:39-45, cap check),
     `validate` (:49-55, ct_eq over ALL entries — no short-circuit), `terminate` (:58-62,
     idempotent — the property that makes reap-after-sweep safe for free); redacting `Debug`
     (:26-34); `session_decision` (:96-117) — its `Some(_) => Reject(404)` arms (:106, :115) are
     the SHIPPED expired-session answer; nothing about them changes.
   - **The sweep site (transport.rs):** the session gate already takes ONE lock, decides + mutates,
     then releases before touching the socket (:147-181, comment :141-143). The sweep is one more
     registry call at the top of that block. Guard-order comment :134-135 pins cap → Origin →
     bearer → session → dispatch; the sweep slots in post-auth.
   - **The clock idiom to mirror (marley_fleet/src/attention.rs):** "NO clock lives here — `now_ms`
     and `stale_after_ms` are always injected" (:1-3); `is_stale` =
     `now_ms.saturating_sub(last_event_ms) >= threshold` (:37-39) with skew tolerance; its t367
     boundary test proves BOTH sides of `>=`. The session TTL is the same shape with a const
     threshold.
   - **The client recovery proof (marley_forge_client):** `SubscriptionExit::SessionExpired`
     (fleet.rs:103, doc :91 — "the MCP 2025-06-18 404-session rule");
     `classify_http_failure`/`classify_if_failure` map 404 → `SessionExpired` (fleet.rs:139-156);
     the pump surfaces it from cursor'd reads AND the standing GET's own status line
     (adapter.rs:315-316, :360-365, :385-389); #373's reconnect re-initializes. The server may
     expire freely — the shipped client self-heals.
   - **Untouched neighbors:** `ct_eq` (auth.rs:43-53) stays the comparator; `mint_secret`
     (secret.rs:36) stays the id mint; `ServerData.surface_index` holds FLEET seat ids (a
     name-collision on "session" — marley_fleet's `Session`, NOT `Mcp-Session-Id`s) and is
     untouched by the sweep.
   - **#375 tests inventoried for churn** (should stay expectation-identical; mechanical `now`
     threading named in the notes): session.rs :126-149, :153-202, :206-216; auth.rs tests
     untouched; transport.rs is masked (no tests).

## Locked-In Decisions
- **D1 — Lazy sweep, pre-dispatch, post-auth, NO timers.** The sweep runs inside the existing
  session-gate lock block of `serve_connection` (transport.rs:147-181), after cap/Origin/bearer,
  before `session_decision` — every authenticated request (POST/GET/DELETE, initialize included)
  triggers it. No background thread, no timer, no new concurrency. Honest consequence: with ZERO
  traffic, expired slots linger in memory — bounded at 8, harmless; the only HARMFUL case (full
  registry + a new client) is self-solving, because that client's own initialize triggers the sweep
  that frees the slots (REQ-003). An unauthenticated request still never touches the registry
  (#375 REQ-012 preserved verbatim). **Rejected:** a sweeper thread/timer (new concurrency for a
  problem request-arrival already solves); sweeping pre-auth (would break the #375 ordering pin).
- **D2 — Expired == unknown at the wire — same 404, indistinguishable.** Sweeping BEFORE the gate
  decides means an expired id is simply absent, and the shipped `Some(_) => Reject(404)` arms
  (session.rs:106/:115) answer it. No new status, no new header, no new reason phrase, no new
  `SessionDecision` arm. A feature, not a leak: no oracle distinguishes "expired" from
  "never existed", and the shipped client classifier (fleet.rs:141) + #373 reconnect recover with
  ZERO client changes. **Rejected:** an expired-specific signal (spec-pointless — 404 IS the
  signal; and it would leak session-liveness history).
- **D3 — Injected clock (the attention.rs idiom), epoch ms.** Pure session code takes `now: u64`;
  NO `SystemTime`/`Instant` outside the masked transport; the single real-clock read is a new
  masked ~3-line helper beside `read_entropy` (transport.rs:79-87), passed down at the sweep/touch
  call sites. Skew semantics via `saturating_sub` (attention.rs:38): a backwards wall-clock jump
  makes sessions look fresh (safe — they just live longer); a forwards jump expires early (safe —
  the client re-initializes, the shipped recovery). **Rejected:** a monotonic `Instant` (not a
  `u64`, not injectable across the lock without conversion, and diverges from the fleet-wide
  epoch-ms convention).
- **D4 — TTL = a named const `SESSION_TTL_MS` beside `SESSION_CAP`, proposed 1_800_000 ms
  (30 min).** The VALUE is D-OPEN-TTL-VALUE for Phase 2, but bounded-and-named: it MUST be a named
  in-code const (not a setting — Out), comfortably above any sane request cadence (minutes, not
  seconds) and at most a few hours (or the wedge returns in slow motion). Expiry boundary: idle
  `>= TTL` expires (mirror `is_stale`'s `>=`, both sides tested).
- **D5 — Touch points = assign + every successful validate; NOT per-SSE-emit.** The GET open IS a
  validated session-bearing request (the gate refuses it otherwise, transport.rs:213) — it touches
  once at open. A standing stream that then sits fleet-quiet past the TTL EXPIRES while its socket
  stays open: accepted. The stream itself is not session-keyed (L1 pushes the one resource
  unconditionally, transport.rs:263), the eventual hang-up reap is an idempotent no-op on the
  swept id, and the client's next request gets 404 → re-initialize → recovered (the shipped #373
  pump — adapter.rs:315-316). **Rejected:** touch-on-emit (per-emit registry writes + stream
  bookkeeping to protect a case the shipped recovery already handles); exempting stream-holders
  (requires stream-liveness tracking in the registry — complexity with no failure it prevents).
- **D6 — TTL expiry is NOT eviction; #375 D4 stands.** Expiry removes only sessions idle ≥ TTL —
  never to make room. A registry full of 8 FRESH sessions still refuses the 9th (`SessionFull` →
  503) with the sweep running (REQ-005 pins it). **Rejected:** "evict-idlest under pressure"
  (reintroduces the evict-a-live-manager hazard D4 exists to prevent).
- **D7 — Strictly additive to #375.** `session_decision`'s decision table is byte-identical in
  behavior (the candidate shape — sweep-then-decide, or a thin pure orchestrator wrapping
  sweep + decide + touch — keeps its signature stable too: D-OPEN-TOUCH-SHAPE). Every existing
  #375 test keeps its expectations; the ONLY permissible edits are mechanical `now`-argument
  threading where a registry method gains a param (the exact sites are inventoried in the notes).
  A shape that forces an expectation change is a design smell to reject in Phase 2.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a session's last-seen is ≥ `SESSION_TTL_MS` in the past at a sweep, the sweep shall remove it, and a subsequent request bearing its id shall receive the SHIPPED 404 (unknown-session) refusal. | pure units: assign at `t` → sweep at `t+TTL` → `validate` false + gate → `Reject(404)`; BOTH boundary sides (`t+TTL-1` survives, `t+TTL` expires — the two-sided-boundary lesson, cf. attention.rs t367_req010) |
| REQ-002 | WHEN a session-bearing request passes validation, that session's last-seen shall be refreshed to the request's `now`. | pure units: assign `t0` → touch/validate at `t1>t0` → sweep at `t0+TTL` (fatal WITHOUT the touch) → still live; sweep at `t1+TTL` → expired (proves the refresh, both directions) |
| REQ-003 | WHEN the registry is FULL of sessions all idle ≥ TTL and a new `initialize` arrives, the pre-dispatch sweep shall free the slots and the initialize shall succeed. | pure units: assign×8 at `t0` (9th refused — the #375 wedge state) → sweep at `t0+TTL` → empty → assign succeeds; the exact ghost-slot scenario end-to-end |
| REQ-004 | WHILE the server runs, every #375 invariant shall hold unchanged: cap-8 reject-new for LIVE sessions, `ct_eq` validation, redacted Debug (ids `***`), stream-drop reclamation, DELETE terminate, and the guard order cap → Origin → bearer → session → dispatch with the sweep POST-auth. | the FULL existing #375 suite green with UNCHANGED expectations (mechanical `now`-threading only — notes inventory); NEW units: 8 FRESH + 9th → `SessionFull` (sweep removes nothing fresh); `terminate` of a swept id → `false` (reap idempotence); redaction test extended over the new entry shape; §18.1 inspect on `serve_connection`'s order |
| REQ-005 | WHILE the crate builds, pure session logic shall carry an injected `now: u64` — no `SystemTime`/`Instant` read outside the masked transport — with cov/MSI 100 on the pure seam. | structural inspect (clock reads grep to transport.rs's masked helper ONLY); gate cov/MSI 100 on session.rs; `cargo mutants --list -f` on the ACTUAL touched file before claiming the kill set (the syntactic-form lesson) |
| REQ-006 | WHEN a request bears an EXPIRED id, the refusal shall be indistinguishable from an unknown-id refusal — the same `Reject(404)` decision, no new status/header/reason/arm. | pure unit: gate outcome for a swept id `==` gate outcome for a never-assigned id (value equality); structural: `SessionDecision` gains NO new arm (inspect) |

## Floors (constitution)
The pure seam (session.rs: registry + sweep + gate) at **cov/MSI 100**; the transport delta rides
the already-masked, coverage-excluded shim (one clock helper + the sweep/touch call sites — same
kind as #375's). Exhaustive `match` over closed enums (no new `SessionDecision` arm to add — D2);
typed errors, no `unwrap` on input paths; both orientations on every comparison boundary
(the `>=` expiry line, per the two-comparison-overlap lesson); test code avoids never-run branches.

## Phase Plan
- **P2 Design** — settle the two D-OPENs (TTL value; touch shape — mutating `validate(now)` vs a
  separate `touch(id, now)` vs a thin pure orchestrator `sweep + decide + touch`, lean =
  orchestrator for zero `session_decision` churn); exact registry entry shape + signatures; the
  exact #375 call sites threading `now` (from the notes inventory); the masked transport delta
  (~5 lines: clock helper, sweep call, touch call); per-REQ test plan.
- **P3 Implement** — session.rs first (entry shape, `SESSION_TTL_MS`, sweep, touch), then the
  masked transport wiring; no other files.
- **P3.5 Inspect** — adversarial: sweep genuinely post-auth? any clock read outside the masked
  helper? any path distinguishing expired-from-unknown? the #375 table/tests truly
  expectation-identical? Debug still redacts over the new entry shape? provenance (§20).
- **P4 Validate** — write + RUN the units per REQ; `cargo mutants --list -f` on the ACTUAL
  session.rs before claiming the kill set; gate green (`--diff`), cov/MSI 100 on the pure seam.
- **P5 Complete** — CHANGELOG; note the expiry posture in orchestration-shell's shipped state;
  AAR (candidate lesson: lazy-sweep-on-ingress beats a timer when the harmful case is
  self-triggering); archive; close #379.
