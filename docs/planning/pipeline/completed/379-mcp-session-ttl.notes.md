# marley_mcp — session TTL sweep: idle-session expiry (#375 hardening follow-up) — Notes

- **Forge ticket:** #379 (c5b91d70-f42e-4caf-9e47-9d9a7de7f80d) — chore, milestone M24, sprint #35 "M24 — Fleet Layer 2"
- **AAR:** pending-promotion (opened at /work promotion; Phase 1 drafted docs-only, no forge calls)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-379-mcp-session-ttl.md
- **Pipeline spec:** 379-mcp-session-ttl.spec.md (pipeline_id a53a0c39-62cd-457a-bcca-4ea73e10d318)

<!-- Working scratch for the pipeline. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** #375 shipped the bounded session registry (cap 8, reject-new-never-evict) with
  stream-drop reclamation; the remaining wedge is a client that initializes but never opens a GET
  stream and never returns — its slot lingers until app restart. Add idle expiry: last-seen touch
  on every session-bearing request (injected clock), a lazy pre-dispatch sweep (TTL ~30 min, no
  timers) in `serve_connection`, and expired ids answering with the shipped 404 so clients
  re-initialize (#373's reconnect already classifies `SessionExpired` and recovers). Strictly
  additive to #375's `session_decision` table and tests.
- **Classification / tier:** chore — hardening follow-up; small pure-seam delta (session.rs) + a
  ~5-line masked transport wiring (clock helper, sweep call, touch call). One crate, no UI, no
  settings, no client changes.
- **Forge recall (§18.3):** Phase 1 drafted under a DOCS-ONLY constraint — no forge/MCP calls this
  session; recall to be re-run at /work promotion. Priors applied from disk/memory: the completed
  #375 spec + its D4/D8/REQ-012/REQ-013 invariants (docs/planning/pipeline/completed/
  375-marley-mcp-hardening.spec.md); `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`
  + the syntactic-form lesson (run `--list` on the ACTUAL code); the two-sided-boundary lesson
  (`PR-claude-two-comparison-overlap-needs-boundary-per-side` — applies to the `>=` expiry line);
  the #370 cap-before-alloc lineage.
- **Discovery (sweep findings, with cites):**
  - **The wedge traced to its exact gap:** reclamation today = DELETE-terminate
    (transport.rs:160-166) OR stream-drop reap (`reap_session` transport.rs:221-228, fired only
    from the SSE write-failure arms :260/:279). An initialize-without-GET client hits neither;
    its slot is immortal; at cap every new initialize → 503 (transport.rs:172-176).
  - **The sweep site is already shaped for it:** the session gate takes ONE mutex, decides +
    mutates, releases before socket IO (transport.rs:147-181). Sweeping at the top of that block —
    post-auth, pre-`session_decision` — means an expired id is ABSENT when the gate runs, so the
    shipped `Some(_) => Reject(404)` arms (session.rs:106/:115) answer it: **zero table change,
    zero wire change, no new `SessionDecision` arm.** This is what makes "strictly additive"
    real rather than aspirational.
  - **The clock idiom exists and is proven:** marley_fleet/src/attention.rs — "NO clock lives
    here" (:1-3), `is_stale` = `now_ms.saturating_sub(last_event_ms) >= threshold` (:37-39), with
    a two-sided boundary test (t367_req010). The registry mirrors this shape with a const TTL.
  - **Client recovery is already shipped and tested:** 404 → `SubscriptionExit::SessionExpired`
    (fleet.rs:103, :141, classify_if_failure :151-156); the pump surfaces it from cursor'd reads
    AND the standing GET's own status line (adapter.rs:315-316, :360-365, :385-389); #373
    reconnect re-initializes. The server may expire freely.
  - **Live-stream expiry consequence (D5, accepted):** a standing-GET holder that stays
    fleet-quiet past the TTL expires while its socket is open. Harmless: the L1 stream is not
    session-keyed (pushes the one resource unconditionally, transport.rs:263), hang-up reap on a
    swept id is the idempotent-terminate no-op (session.rs:58-62, tested :146), and the client's
    next request 404s → re-initializes. Touch-on-emit / stream-liveness exemptions REJECTED
    (complexity protecting a case the shipped recovery already handles).
  - **Name-collision guard:** `ServerData.surface_index` holds FLEET seat ids (marley_fleet
    `Session`), NOT `Mcp-Session-Id`s (transport.rs:35 doc). The sweep touches ONLY
    `ServerData.sessions`. Worth one inspect glance in P3.5, zero code interaction.
  - **Skew semantics:** `saturating_sub` ⇒ a backwards wall-clock jump makes sessions look fresh
    (they live longer — safe); a forwards jump expires early (client re-inits — safe). Epoch ms
    matches the fleet-wide convention; `Instant` rejected (D3).
  - **Fresh-8 still refuses the 9th:** the sweep removes nothing idle < TTL, so #375's D4
    reject-new posture is untouched under load (REQ-004 pins it with a new unit).
- **#375 test-churn inventory (the additive-only flag):** expectations change NOWHERE; the only
  permissible edits are mechanical `now`-argument threading IF Phase 2 gives a method a `now`
  param. Exact sites, by shape:
  - `assign` gains `now` (needed to stamp last-seen in every candidate shape) → thread a literal
    at: session.rs `registry_caps_at_eight_and_rejects_without_eviction` (:126-149 — ~11 `assign`
    calls), `session_decision_covers_every_arm` (:155 — 1 call), `debug_redacts_session_ids`
    (:208 — 1 call).
  - `validate`/`terminate`/`session_decision` — UNCHANGED under the lean orchestrator shape
    (sweep + decide + touch wrapped in a new pure fn; `session_decision` keeps `&SessionRegistry`
    and its exact signature). If Phase 2 instead picks a mutating `validate(&mut self, id, now)`,
    the ~14 `validate` call sites and the 10 `session_decision` sites in the #375 tests would need
    threading — a reason to prefer the orchestrator.
  - auth.rs tests (ct_eq/bearer_ok): untouched. transport.rs: masked, no tests. dispatch tests:
    no session surface, untouched.
- **EARS drafted (full table in the spec):** REQ-001 idle-≥-TTL expired at next sweep + shipped
  404; REQ-002 successful validation refreshes last-seen; REQ-003 sweep frees capacity (the
  full-of-ghosts initialize succeeds); REQ-004 every #375 invariant unchanged; REQ-005 injected
  `now`, no SystemTime in pure code, cov/MSI 100; REQ-006 expired ≡ unknown at the wire (added
  during Phase 1 — it pins D2 testably).
- **Decisions:** D1 lazy pre-dispatch post-auth sweep, no timers; D2 expired == unknown (shipped
  404, indistinguishable — a feature); D3 injected epoch-ms clock, one masked reader; D4
  `SESSION_TTL_MS` named const, proposed 1_800_000; D5 touch = assign + successful validate only;
  D6 expiry ≠ eviction (#375 D4 stands); D7 strictly additive (table byte-stable, tests
  expectation-identical).
- **Open questions (D-OPEN for Phase 2):**
  - **D-OPEN-TTL-VALUE** — 30 min proposed; bounds locked (named const, minutes-not-seconds,
    ≤ a few hours); pick + justify in Design.
  - **D-OPEN-TOUCH-SHAPE** — mutating `validate(now)` vs separate `touch(id, now)` vs a thin pure
    orchestrator (`sweep + decide + touch` in one covered fn the transport calls once). Lean:
    orchestrator — keeps `session_decision`'s signature + table byte-stable and shrinks the #375
    test churn to the `assign` sites only.
- **Forge ids:** ticket #379 c5b91d70-f42e-4caf-9e47-9d9a7de7f80d; sprint #35 "M24 — Fleet
  Layer 2"; predecessor #375 b20f9b7b-b210-4310-810d-3399eabd7d45 (closed); client-half proof
  #373 (closed). AAR: pending-promotion.

- **Promotion (/work 376-379,335, 2026-07-21):** queued→active, claimed + in-progress/plan, AAR
  `3af8c68c`. The queued spec's design was COMPLETE and code-verified (registry shape confirmed);
  Phase 1+2 fold: the drafter's two D-OPENs resolved as proposed — TTL = 30min
  (`SESSION_TTL_MS = 1_800_000` beside `SESSION_CAP`), touch-shape = the ORCHESTRATOR
  (`session_gate`: sweep→decide→touch-on-Proceed; `session_decision`/`validate` signatures
  untouched → zero threading of the ~24 sibling call sites). Phase 1+2 PASS.

## Phase 2 — Design
- (pending promotion to active)

- (Folded above — the queued draft WAS the design; the resolutions are recorded in the promotion
  entry.)

## Phase 3 — Implement
- Built: `SessionEntry { id, last_seen_ms }` (no Debug — cannot leak), `SESSION_TTL_MS`,
  `assign(id, now)`, `touch` (ct_eq scan, all entries), `sweep` (`saturating_sub < TTL` retain —
  boundary expires, skew reads fresh), the `session_gate` orchestrator; transport: masked
  `now_epoch_ms` (its own doc — the first insertion swapped docs with `read_entropy`, caught +
  repaired immediately), ONE `now` per request shared by gate + assign, the call site swapped to
  `session_gate`. Existing #375 tests: mechanical `, 0` threading only (expectations unchanged).
- **Status: Phase 3 — Implement PASS.**

## Phase 3.5 — Inspect
- **2 critics** (correctness — with a REAL `cargo mutants -f session.rs` run: 32 tested, 30 caught,
  2 unviable [`Default::default()` on the two decision fns — SessionDecision has no Default, the
  #203 rule], **0 missed**; security — line-level guard-order trace). Ledger:
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | LOW (both) | `lib.rs` exported only the TTL-less `session_decision`; `session_gate`/`SESSION_TTL_MS` unexported — a future external caller would silently bypass TTL; ALSO the root of a gate:14 RED (`sweep`'s public doc linked the private const). | REAL | both exported; rustdoc clean |
  | F2 | INFO | REQ-002's letter ("passes validation → refreshed") vs shipped touch-on-Proceed-only: a `Terminate` validates but is REMOVED under the same lock in the same request — no observable state where the refresh matters. | RECONCILED (this line) | none |
  | F3 | INFO | `touch`'s per-entry branch vs `validate`'s branch-free fold — judged WITHIN the D8 posture (branches on the ct_eq RESULT post-Proceed; only the caller's own slot position could leak — not a secret; mirrors `terminate`'s retain). | ACCEPT + DOCUMENT | the touch doc gained the precise clause |
  | F4 | NIT | double `now_epoch_ms()` on the Initialize path. | polish | hoisted to ONE `now` per request |
  | F5 | INFO (security) | a session swept mid-stream doesn't sever its live SSE stream — PRE-EXISTING symmetry with #375's DELETE-terminate (the stream was authenticated at open; reap-on-drop idempotent; the D5 acceptance verified TRUE of the shipped code, client recovery real at fleet.rs:141/adapter re-init) | ACCEPTED | none |
- **Clean (verified):** guard order intact (an unauthenticated request never reaches the registry —
  line-traced); no new attacker capability (expiry is a pure fn of (last_seen, now) — who sweeps is
  irrelevant; concurrent-live stays ≤8 always); redaction intact (SessionEntry has NO Debug; the
  manual registry Debug shows count only); expired ≡ unknown BYTE-identical at the wire (one
  status_reason producer); broken-clock direction verified fail-OPEN to the #375 behavior (TTL is
  hygiene, not an auth boundary — bearer/Origin/loopback/cap all hold with a dead clock) and
  self-healing; full-of-FRESH initialize still 503s via assign under ONE lock (no TOCTOU);
  DELETE-of-swept → the idempotent 404.
- **Status: Phase 3.5 — Inspect PASS.**
- (pending)

## Phase 4 — Validate
- **Tests:** 5 new t379 units (REQ-001 two-sided boundary + the shipped-404 gate outcome ·
  REQ-002 gate-touch both directions [fatal-without-the-touch sweep + expiry-from-the-touched-stamp]
  · REQ-003 the wedge end-to-end [full-of-idle → gate sweeps → Initialize → assign succeeds] ·
  REQ-004 invariants [fresh-8 unswept, 9th refused, terminate-of-swept idempotent-false, u64::MAX
  skew reads fresh] · REQ-006 expired ≡ unknown value-equality); the #375 suite green with
  EXPECTATIONS UNCHANGED (mechanical `, 0` assign threading only — the additive-only flag held).
  One test bug self-caught on first run: the idempotence assert targeted the TOUCHED seat (which
  correctly survives) — retargeted to a genuinely swept sibling with the survival asserted too.
- **Runs:** marley_mcp 58/58; the critic's REAL mutants run: 32 tested, 30 caught, 2 unviable,
  0 missed. **Gate `--diff`: first attempt RED on gate:14 only** (rustdoc: `sweep`'s public doc
  linked the then-private `SESSION_TTL_MS` — fixed at source by EXPORTING `session_gate` +
  `SESSION_TTL_MS`, which was also the critics' F1). **Second attempt: GATE GREEN 15/15** —
  coverage 100, MSI 100, receipt written. No UI surface — no driven capture applies (a pure
  registry + one masked wiring line).
- **Status: Phase 4 — Validate PASS.**

- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG (first under Added); orchestration-shell §12 hardening line gains the TTL.
- **Knowledge:** no new failure class (the inspect found zero MED+; the F1 export/doc-link fix is
  the #379 story's only source change beyond the plan) — AAR 3af8c68c submitted (completed).
- **Ticket:** #379 closed (done) + ship comment; local doc → closed; pair → completed/.
- Lesson: the sweep-before-decide construction let "expired ≡ unknown" be true BY ABSENCE (no new
  arm, no wire change) — making an invariant structural beats testing it in; the orchestrator shape
  kept ~24 sibling call sites untouched.

- (pending)
