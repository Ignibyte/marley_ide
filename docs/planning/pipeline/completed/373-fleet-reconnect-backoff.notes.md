# Fleet subscription — auto-reconnect + jittered backoff for the standing-SSE pump — Notes

- **Forge ticket:** #373 09c7963a-b3e7-456d-a71c-f8e4da63d123
- **AAR:** ca7abcba-25a6-4bd6-b78f-cb15dbfa7f01 (opened at /work)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-373-fleet-reconnect-backoff.md
- **Pipeline spec:** 373-fleet-reconnect-backoff.spec.md
- **Depends on:** forge #372 (queued ahead in this batch, HARD ORDER — harness + typed disconnect
  surface + typed 404-session signal; #373's integration tests and `should_retry` input both come
  from it)

## Phase 1 — Plan
- **Request:** Ticket #373 (M23.5, sprint #34 "Fleet Layer-1 Consolidation"; origin = a #368 inspect
  LOW): a transient Forge bounce permanently kills live fleet updates until the app re-invokes
  `start` — no data loss (durable cursor) but silent death. Fix = (1) a PURE backoff policy seam
  (exponential + injected full jitter + injected clock, cov/MSI 100) and (2) the masked pump loop:
  every current die-path (close / hard error / open failure / typed 404-session) → wait
  (stop-responsive ≤1s) → re-initialize → replay-from-cursor → subscribe → listen, forever until
  stopped.
- **Classification / tier:** feature; one new pure module + a masked restructure of an
  already-masked fn (`run_subscription`) + integration tests on the #372 harness. No UI, no wire
  change, no fixture change. Consumer: the pump itself; beneficiary: the #369 fleet rail (its
  staleness indicator stops being a tombstone).
- **Forge recall (§18.3):** drafted offline (Phase-1 doc pass — no live forge calls from this
  drafter); recall satisfied from the on-disk record and MUST be re-run live at `/work` promotion.
  Applied here: the #368 F6 hot-spin lesson (break-don't-spin — this ticket must preserve it: every
  retry separated by a real wait); `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`
  + the syntactic-form rule + the #203/#204 `Default`-derive viability lesson (spec D6);
  `PR-claude-two-comparison-overlap-needs-boundary-per-side` (baked into REQ-003's verify);
  orchestration-shell.md §6 (push = MCP subscription; "Marley down ≠ fleet down" :149) as the D1
  frame; the `marley_fleet::attention` injected-clock idiom as the D2 precedent.

### Discovery — the prior-art sweep (evidence, file:line)
**Leg 1 — behavior maps: N/A (checked).** Grepped docs/warp_architecture/ + docs/zed_architecture/
for reconnect/backoff/SSE/subscription — zero hits. Neither map covers a client's control-plane
connection lifecycle. Checked, not skipped.

**Leg 2 — published material (no fetch needed; standard practice).**
- AWS Architecture Blog, **"Exponential Backoff and Jitter"** (Marc Brooker): compares no-jitter /
  equal-jitter / decorrelated / **full jitter** (`sleep = random(0, min(cap, base·2^attempt))`);
  full jitter wins on contention + total work. We adopt full jitter with the random input INJECTED
  as a fraction — the policy stays a deterministic pure fn; the masked layer supplies entropy.
- **MCP spec 2025-06-18, Streamable HTTP transport lifecycle:** server MAY assign `Mcp-Session-Id`
  on `initialize`; client echoes it on all later requests; a request answered **404** because the
  session expired ⇒ the client MUST start a NEW session (fresh `initialize` →
  `notifications/initialized`). So "re-initialize IS the recovery" is the published rule, not our
  invention — the typed 404 signal (#372) routes into the same reconnect cycle as a plain drop (D5).

**Leg 3 — our own source (verified this draft).**
- **Current on-close behavior — `crates/marley_forge_client/src/adapter.rs` (the defect, verified):**
  - Read loop `while !stop.load(Ordering::Relaxed)` :217.
  - `Ok(0) => break` — "the server closed the stream" :219.
  - `WouldBlock | TimedOut => continue` :224-231 — the expected 1s wakeup so the loop re-checks
    `stop` (read timeout set at :298, `Duration::from_secs(1)`).
  - `Err(_) => break` :232 — the **#368 F6 hot-spin fix**: "any OTHER error (a hard mid-stream
    disconnect — reset / broken pipe) must break, not spin the dead socket hot. A restart resumes
    from the persisted cursor" (:220-223).
  - `open_listen_stream` failure → `return` :208-211; initialize fetch/parse failure → `return`
    :174-184; `!supports_subscribe` → `return` :185-187; a bad URL makes `initialize_request`
    return `None` → `return` :174-177 (the one IMMUTABLE-within-a-run cause — `should_retry` false).
  - Every one of these ends the thread. `FleetSubscription` keeps the stop flag
    (`Arc<AtomicBool>` :134; `stop()` :146-149; Drop stores + joins :152-160).
- **Durable cursor + why a manual restart already resumes correctly:** each run seeds
  `FleetSync::with_cursor(load_cursor_in(config_dir))` :197; `refresh` applies a page then persists
  the advanced cursor EVERY time (:277-280 `save_cursor_in`); storage `<config_dir>/fleet-cursor`
  (:309-322). `FleetSync` at fleet.rs:441 (`with_cursor` :453 — doc: "a reconnect resumes from where
  it left off" :452; `apply_page` :461; `cursor()` :482; `next_read_uri` :487 →
  `fleet://events?since=<cursor>`); reconnect-replay test `t368_req008_reconnect_replays_from_cursor`
  fleet.rs:1074-1087. ⇒ The reconnect loop is "do what boot already does, again, without being
  restarted" — the replay machinery needs ZERO new code.
- **Session-id plumbing today:** `parse_initialize` reads the `mcp-session-id` header
  (fleet.rs:227-243; `Initialized.session_id` :221-222 — "echoed on every later request"); the
  builders thread it (fleet.rs:164/:181/:195/:200-201 via `mcp_post`/`mcp_listen_get`,
  lib.rs:198/:225). A non-2xx is an UNTYPED `ForgeError::Http(status_line)` today
  (`jsonrpc_from_http` lib.rs:252-255, `status_is_2xx` :268) — #372 delivers the typed 404-session
  discrimination this ticket consumes.
- **Injected-clock precedent (D2):** `marley_fleet::attention` — "NO clock lives here — `now_ms` and
  `stale_after_ms` are always injected, so a snapshot's attention is a pure function of (snapshot,
  now, threshold)" (crates/marley_fleet/src/attention.rs:1-2; `is_stale(session, now_ms,
  stale_after_ms)` :37-38; module doc lib.rs:12). Also app-side: `demo_snapshot(now_ms: u64)` —
  "`now_ms` is injected (pure)" (crates/marley_app/src/fleet_rail.rs:117-121) and
  `acknowledgment_is_stale(acked, now: &DiskState)` (crates/marley_app/src/extchange.rs:82).
- **Existing backoff/retry owner: NONE (a no-owner PASS).** Workspace grep backoff/retry/reconnect:
  the only kin is `terminal_blocks::session` `PUMP_RETRY_BUDGET: u32 = 8`
  (crates/terminal_blocks/src/session.rs:32) with `WriteOutcome::Retry` classification (:112,
  :119-123) — consecutive-`WouldBlock` PTY-write retries inside one burst, then FAIL; no timing, no
  exponential growth, not a reconnect policy. Plus the LSP host's deliberate no-retry posture
  (crates/marley_app/src/app.rs:4742). Nothing to reuse; `backoff.rs` is the first owner.
- **No-gap/no-dupe guarantee (cited in the spec):** the #367 reducer is idempotent under
  re-delivery — "assignments + a `max` join, so a cursor'd catch-up that overlaps already-applied
  events converges to the same snapshot" (crates/marley_fleet/src/reducer.rs:138-140; monotone
  max-join :199-201; `reduce` replay-safe :204-212; test `t367_req002_overlap_replay_is_noop`
  :255-287).
- **Scope-out justification:** the staleness indicator already covers degradation UI —
  `STALE_AFTER_MS: u64 = 30_000` (crates/marley_app/src/fleet_rail.rs:17) driving `is_stale` at
  :346-356; a down-and-retrying pump reads as stale seats, honestly.

### Decisions
D1-D6 locked in the spec: **D1** until-stopped forever at the capped interval (batch-owner lock;
verified uncontradicted — rationale: monitoring surface, orchestration-shell.md §6/:149; the
staleness indicator communicates meanwhile; rejected max-attempts-then-dead). **D2** injected jitter
fraction + injected clock, full jitter (rejected: `rand` dep; rejected: jitterless). **D3** constants
`BASE_DELAY_MS=500` / `CAP_DELAY_MS=30_000` / `STABLE_RESET_MS=30_000` (config = future; the 30s
values resonate with `STALE_AFTER_MS`). **D4** new pure `crates/marley_forge_client/src/backoff.rs`
(rejected: marley_fleet — transport-free charter, orchestration-shell.md §7 :157; rejected: inside
the masked adapter — dodges the floors; rejected: growing fleet.rs — distinct concern). **D5**
re-initialize every cycle, one uniform recovery path (the MCP 404 rule + the cursor/reducer make it
lossless; rejected: session/stream resume bookkeeping). **D6** real mutant set via
`cargo mutants --list -f` at Validate; `Default`-derive choices made deliberately (viability).

### Open questions (for Phase 2, against #372's LANDED state)
- **O1 — the typed-cause enum's exact shape** is #372's to land; `should_retry`'s match goes
  exhaustive over whatever ships (REQ-004 "one unit per arm" tracks it). If #372 lands a cause this
  draft didn't anticipate, classify it transient-unless-provably-immutable (the D1 posture).
- **O2 — can the #372 harness stage a 404?** If yes, REQ-007 verifies by integration; if not, the
  REQ-007 fallback (pure decision test + masked-behavior note) is pre-authorized in the spec — do
  NOT grow #372's harness from this pipeline (scope Out).
- **O3 — pure API shape:** free fns (`next_delay_ms`, `next_attempt`, `should_retry`) vs a small
  `Backoff` state struct holding `attempt` + `connected_at`. Lean: struct for the counter lifecycle
  (REQ-003 reads naturally), free fn for the delay formula. Check `Default`-derive viability either
  way (D6) and whether the wait-slicing helper (≤1000 ms quanta, REQ-006) earns a pure fn.
- **O4 — where the integration tests live:** wherever #372 puts the harness (expect
  `crates/marley_forge_client/tests/`); the kill-after-event-3 + stop-mid-wait cases grow that file.
  Use a tempdir `config_dir` so the cursor round-trips per test (`load_cursor_in`/`save_cursor_in`).
- **O5 — masked-layer jitter source:** subsecond nanos of the existing time read (no `rand`, D2).
  Confirm at implement that the derivation stays in adapter.rs and feeds the pure fn a plain `f64`.

## Phase 2 — Design

### Reconciliation with #372's LANDED surface (the hard-order alignment point)
The spec was drafted before #372 shipped. What #372 actually landed (verified in `adapter.rs`/`fleet.rs`):
- `run_once(endpoint, config_dir, stop, on_update) -> SubscriptionExit` — the SINGLE-attempt seam, exactly
  the shape #373 wraps (the spec's D6-anticipated seam is real).
- `SubscriptionExit { Stopped, StreamClosed, Disconnected, SessionExpired, HandshakeFailed }` — the typed
  cause. **Key delta from the spec's REQ-004:** #372 folded the "immutable config malformation
  (unsupported URL)" case into `HandshakeFailed` — AND that case is UNREACHABLE post-construction
  (`forge_endpoint_from` already validated http+loopback, so `initialize_request` never returns `None`);
  `HandshakeFailed` in practice means "forge unreachable / not-ready / no subscribe capability" — all
  TRANSIENT. So **`should_retry` simplifies to `!Stopped`** (retry every non-clean-stop cause) — there is
  no reachable "give up permanently within a run" cause, which matches D1 (until-stopped) exactly. Recorded
  as a deliberate design adaptation to the landed enum, not a deviation from intent.
- `exit_reason()` is poll-able; the thread stored the single attempt's reason. **Post-#373 the thread only
  ends on `Stopped`** (every other cause retries), so `exit_reason()` becomes `Some(Stopped)` at end / `None`
  while running-or-reconnecting. The old #372 tests that asserted `exit_reason == StreamClosed/SessionExpired/
  Disconnected` + "no retry" therefore MUST be converted to reconnect-behavior assertions (below) — the
  correct consequence of crossing the "retry is #373" boundary #372 documented.

### Architecture / approach
§20 still N/A (Marley-own reconnect plumbing; policy adopted from published AWS full-jitter + the MCP 404
rule). Two parts: a pure backoff policy seam + the masked outer loop.

**① Pure `backoff.rs` (NEW top-level sibling — D4; cov/MSI 100).** Stateless pure decisions (state lives in
the masked pump, the attention.rs injected-clock idiom):
- consts `BASE_DELAY_MS=500`, `CAP_DELAY_MS=30_000`, `STABLE_RESET_MS=30_000`, `POLL_QUANTUM_MS=1000` (D3).
- `should_retry(cause: SubscriptionExit) -> bool` = `!matches!(cause, Stopped)` (per the reconciliation).
- `next_attempt(attempt: u32, connected_ms: u64, disconnected_ms: u64) -> u32` — uptime
  `disconnected.saturating_sub(connected) >= STABLE_RESET_MS` → `0`, else `attempt.saturating_add(1)`.
- `next_delay_ms(attempt: u32, jitter_fraction: f64) -> u64` — full jitter over a SATURATING ceiling:
  `ceiling = min(CAP, BASE.saturating_mul(2u64.saturating_pow(attempt)))`; `(jitter.clamp(0,1) * ceiling as
  f64) as u64` (attempt≥64 saturates to CAP, no overflow — REQ-001).
- `wait_slice_ms(remaining_ms: u64) -> u64 = remaining.min(POLL_QUANTUM_MS)` — the ≤1s stop-poll granule.

**② Masked outer loop (`adapter.rs` `run_subscription` — stays `mutants::skip` + coverage-excluded).**
Wrap #372's `run_once` in an until-stopped loop; ambient authority (clock + jitter) stays masked, decisions
call the pure fns:
```
let mut attempt = 0u32;
let reason = loop {
    if stop.load() { break Stopped; }
    let connected_at = now_ms();                     // masked clock
    let cause = run_once(endpoint, config_dir, stop, on_update);
    if !should_retry(cause) { break cause; }         // Stopped → clean exit
    attempt = next_attempt(attempt, connected_at, now_ms());
    if !backoff_wait(next_delay_ms(attempt, jitter_fraction()), stop) { break Stopped; }
};                                                    // store `reason` in `exit`
```
`backoff_wait(delay, stop) -> bool` (masked): sleep in `wait_slice_ms` granules, re-checking `stop` each
granule; returns `false` (→ Stopped) if stop fires mid-wait. `now_ms()`/`jitter_fraction()` (masked): the
subsecond system-clock — no new dep (the crate already has no `rand`; the D2 injection keeps the pure fns
deterministic). **F6 no-hot-spin PRESERVED:** every retry is separated by a real backoff wait; `run_once`'s
hard-error break still exits its read loop — it now feeds the outer loop, not thread death.

### File manifest
| # | File | Change |
|---|---|---|
| 1 | `crates/marley_forge_client/src/backoff.rs` | NEW pure module — 4 fns + 4 consts + unit tests (cov/MSI 100) |
| 2 | `crates/marley_forge_client/src/lib.rs` | `mod backoff;` + re-export the fns/consts |
| 3 | `crates/marley_forge_client/src/adapter.rs` | MODIFY (masked): `run_subscription` outer loop + `backoff_wait` + `now_ms`/`jitter_fraction` helpers |
| 4 | `crates/marley_forge_client/tests/livewire.rs` | Fixture: `close_after`→`close_gets_before: usize` + `fail_initialize: bool` + hold-open-until-fixture-stop; CONVERT req008/req009/get_rejection to reconnect assertions; ADD REQ-005/006/007 |
| 5 | `CHANGELOG.md` | Added entry |
| — | `orchestration-shell.md` | §7 row: the #368/#372 line gains "self-healing (reconnect+backoff)" |

### Regression Test Plan
| # | Test | Kind | Proves |
|---|---|---|---|
| U1 | `next_delay_ms` jitter=1.0 for attempts 0..8 → `500,1000,2000,4000,8000,16000,30000,30000`; attempt=`u32::MAX`→`CAP` (no panic) | pure | REQ-001 |
| U2 | `next_delay_ms` jitter 0/0.5/1.0 at a below-cap + at-cap attempt → bounds `[0,ceiling]`, monotone | pure | REQ-002 |
| U3 | `next_attempt` uptime exactly `STABLE_RESET_MS`→0, just-under→+1, just-over→0 (both `<`/`>=` boundary orientations) | pure | REQ-003 |
| U4 | `should_retry`: Stopped→false; StreamClosed/Disconnected/SessionExpired/HandshakeFailed→true (exhaustive over the landed enum) | pure | REQ-004 |
| U5 | `wait_slice_ms`: `>1000`→1000, `≤1000`→itself, `0`→0 | pure | REQ-006 pure |
| I1 | kill-after-notification (close first GET, hold the reconnect) → final snapshot == unbroken control run; `get_conns ≥ 2` | integration | REQ-005 |
| I2 | stop() mid-backoff-wait (fixture fails `initialize` repeatedly → pump is backing off) → thread joins < 2s, `exit_reason()==Stopped` | integration | REQ-006 |
| I3 | fixture answers a read 404 → pump re-initializes (initialize count ≥ 2) → resumes; snapshot recovers | integration | REQ-007 |
| I4 | rejected GET held open → pump reconnects (`get_conns ≥ 2`), does not idle | integration | latent-gap under reconnect |
| — | req001/002/003/004(seg)/006(frozen)/007(restart)/stop kept green (hold-open-until-fixture-stop default → no spurious reconnect) | integration | #372 parity |
| G1 | `cargo mutants --list -f backoff.rs` real set killed; gate green; adapter stays masked | gate | REQ-008 |

### Risks / decisions
- **The exit_reason semantics shift** (thread only ends on Stopped) — documented; the 3 converted tests
  assert reconnect *behavior* (connection/initialize counts + snapshot recovery), a stronger proof than the
  old terminal-reason asserts.
- **Fixture must not infinite-close** — `close_gets_before: usize` closes only the first N GETs then holds
  the reconnect open, so REQ-005 converges instead of looping forever. Hold-open now waits on the fixture's
  stop flag (not a fixed 4s) so the happy-path tests never see a spurious reconnect.
- **Timing** — REQ-006 relies on the ≤1s stop-poll granule; join deadline 2s absorbs one granule + jitter.
  Integration tests assert *reconnect happened* (counts) + *snapshot converged*, never exact delays (those
  are the pure units with injected jitter) — so real-jitter non-determinism can't flake them.
- **`now_ms`/`jitter_fraction` masked** — first `SystemTime` use in the crate; confined to the masked
  adapter (ambient authority), pure fns stay deterministic.

## Phase 3 — Implement
- **Built to the manifest.** NEW `backoff.rs` (pure: `should_retry`/`next_attempt`/`next_delay_ms`/
  `wait_slice_ms` + 4 consts + 5 unit tests); `lib.rs` `mod backoff` + re-exports; `adapter.rs` masked
  restructure — `run_subscription` is now the until-stopped reconnect loop wrapping `run_once`, plus masked
  `backoff_wait` (stop-responsive granular sleep) + `now_ms`/`jitter_fraction` (ambient authority, first
  `SystemTime` use in the crate).
- **Fixture evolution (`tests/livewire.rs`):** `close_after: bool` → `close_gets_before: usize` (close the
  first N GETs, hold the reconnect — avoids an infinite-close loop), + `fail_initialize: bool`; GET now
  holds open until the fixture is dropped (`hold_until_stopped`) so a healthy connection never triggers a
  spurious reconnect; the fixture stop flag is threaded to `serve_get`.
- **#372 test conversions (the correct consequence of crossing the "retry is #373" boundary):**
  `req008` (close→StreamClosed, no-retry) → `req373_reconnects_after_midstream_close_and_converges`
  (close→reconnect→converge to a,b,c, seat c arrives ONLY via the reconnect); `req009` (404→SessionExpired,
  no-reinit) → `req373_read_404_triggers_reinitialize_and_resumes` (404→re-initialize→resume, ≥2
  initializes); `get_rejection` (→Disconnected terminal) → `rejected_get_reconnects_instead_of_idling`
  (→reconnect, ≥2 GET conns). ADDED `req373_stop_during_backoff_wait_is_honored`. The 8 happy-path/parity
  tests (req001-007 + stop) stay green (hold-open default).
- **Deviation from the drafted spec (documented in the reconciliation above):** `should_retry` = `!Stopped`
  rather than a per-transient-cause enum, because #372 folded the immutable-config case into an
  unreachable `HandshakeFailed`. REQ-005's "identical to an unbroken run" is realized as "converges to the
  same final snapshot with seat c delivered only via the reconnect" + the re-initialize proof (a separate
  control-run structure would diverge in read count; convergence-to-the-pure-oracle is the stronger check).
- **One test-assertion fix mid-implement:** REQ-005 initially asserted `get_conns ≥ 2`, which raced (seat c
  arrives via the reconnect's *replay read*, before its GET opens) → switched to `initializes ≥ 2` (recorded
  before the read). 12/12 green, stable across 5 runs.

## Phase 3.5 — Inspect
Two independent critics (correctness/hot-spin/stop-responsiveness · test-honesty/floors). The core
reconnect+backoff behavior verified SOUND — critic 2 empirically reverted the loop and confirmed exactly
the 4 reconnect tests go red + the 8 kept tests stay green (litmus passed). Findings:

| # | Finding | Sev | Verdict | Fix |
|---|---|---|---|---|
| F1 | **Reconnect rebuilt a fresh EMPTY `FleetSync` every cycle** (`run_once`) — a reconnect reading only a post-cursor DELTA (the proposed `?since=<cursor>` contract) would DROP every seat introduced before the cursor → the fleet rail collapses to just-the-delta on every transient bounce (the opposite of "no gap"). The old fixture served cumulative pages, masking it. | **MED** | REAL | CARRY the snapshot forward: `run_subscription` builds ONE `FleetSync` (seeded from the persisted cursor) and passes `&mut sync` into every `run_once`; a reconnect resumes from the held cursor AND retains known seats. Reworked REQ-005 to serve a strict DELTA (`PAGE_C`, only seat c) on reconnect → snapshot must still be `{a,b,c}`; **empirically verified the test FAILS (delivers `{c}`) without the fix, passes with it**. |
| F2 | `exit_reason()` + the `exit` field docs still described pre-#373 per-attempt semantics ("a stream close surfaces StreamClosed") — post-#373 the thread only ends on `Stopped`, so it's `None`-while-running / `Some(Stopped)`-after-stop; a downstream poller trusting the doc waits forever. | LOW | REAL | Rewrote both docs to the reconnect-loop semantics (transient causes handled internally, not observable). |
| F3 | An `on_update` callback panic unwinds the pump thread → `exit_reason()` stays `None` forever, silently defeating the self-healing. Pre-existing, more impactful now the thread is long-lived. | LOW | REAL (doc) | Documented the "MUST NOT panic" contract on `start` (a `catch_unwind`-into-reconnect would re-panic on the same snapshot — a pointless loop; the callback is the app's responsibility). |
| F4 | REQ-005 "no gap" ultimately rests on the forge's `?since=` semantics the fixture can't validate against a real server. | LOW | TRACK | The F1 carry-forward makes losslessness hold for BOTH a cumulative-since AND a delta-since forge, removing the dependency for the reconnect path; fresh-app-restart/boot completeness stays a deferred L0-day concern. |

Confirmed sound (no change): F6 no-hot-spin preserved (every retry path waits or exits; near-0 delay only after a ≥30s-stable drop = once/30s, not a spin); stop-responsive within ≤1s on every wait (no underflow, ≤30 iters); saturating arithmetic panic-free at attempt 0/63/64/u32::MAX + a backward clock; `should_retry = !Stopped` correct + total; all 6 Script fields + both serve_post arms exercised; adapter.rs = 0 mutants (masked), backoff.rs 12/12 caught.
- **Post-fix:** 12 integration + 44 unit tests green; stable across runs.

## Phase 4 — Validate
- **Tests RUN:** 5 new pure `backoff.rs` units (delay sequence+cap+saturation, jitter bounds, stable-reset
  boundary both orientations, should_retry exhaustive, wait-slice cap) + the 12 livewire integration
  scenarios (8 kept #372 parity + 4 #373 reconnect). `cargo test -p marley_forge_client`: **44 lib unit +
  12 integration + 0 doctest, all pass**; livewire stable across ~6 runs (~1.1–1.9s).
- **Carry-forward fix EMPIRICALLY guarded:** temp-reverted `run_once` to rebuild a fresh empty FleetSync →
  `req373_reconnects_after_midstream_close_and_converges` FAILED (delivered `{c}` only); restored → passes.
- **Mutants (`cargo mutants --list -f backoff.rs`):** 12 viable, ALL caught by the 5 units (should_retry
  true/false/delete-!; next_attempt 0/1/`>=`→`<`; next_delay_ms 0/1/`*`→`+`/`*`→`/`; wait_slice_ms 0/1).
  Concrete-default returns (bool/u32/u64) → viable+killable (#204). adapter.rs = 0 mutants (all masked incl.
  the new run_subscription/backoff_wait/now_ms/jitter_fraction).
- **Full gate `scripts/gates.sh --diff`: GATE GREEN [diff] — 15/15** (coverage ≥100% incl. the new pure
  `backoff.rs`, mutation MSI ≥100%, clippy -D warnings, machete/gitleaks/audit/deny/miri/visual green).
  Receipt written.
- **No live-app drive:** library crate, no UI surface (§7 N/A — the fixture-driven suite is the pump's
  behavior proof).
- **Pre-existing:** none touched.

## Phase 5 — Complete
- **CHANGELOG:** entry under Added (M23.5 ②, self-healing pump).
- **Architecture docs:** `orchestration-shell.md` §7 `marley_forge_client` row updated (SHIPPED + integration-
  tested + self-healing #372/#373; the backoff/carry-forward note) + the §12 M23.5 breadcrumb already names
  #373.
- **Knowledge captured (forge):** failure `BF-373-reconnect-rebuilds-empty-fleetsync` (F1, MED); prevention
  rule `PR-claude-carry-accumulated-state-across-a-reconnect-for-delta-streams-001` (HIGH — the reusable
  lesson: carry reduced state across a reconnect on a delta stream, and write the test with a STRICT delta
  so the drop is caught); AAR `ca7abcba` submitted.
- **Ticket #373 closed + pipeline archived to completed/.**
