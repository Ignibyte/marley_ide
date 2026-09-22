---
pipeline_id: 350b6888-20fa-4418-89b2-546bbf9d0ed0
ticket: forge#373 (09c7963a-b3e7-456d-a71c-f8e4da63d123) · local docs/planning/tickets/open/TICKET-373-fleet-reconnect-backoff.md
aar_id: ca7abcba-25a6-4bd6-b78f-cb15dbfa7f01
status: Phase 5 — Complete PASS
title: Fleet subscription — auto-reconnect + jittered backoff for the standing-SSE pump (#368 follow-up)
type: feature
milestone: M23.5
depends_on: forge#372 (queued ahead in this batch, HARD ORDER — its loopback fixture-Forge harness hosts this ticket's kill-and-reconnect tests; its typed disconnect surface + Mcp-Session-Id echo with a typed 404-session signal are what this ticket's reconnect loop consumes)
references:
  - docs/marley_architecture/orchestration-shell.md
  - docs/marley_architecture/fleet-control-plane.md
  - crates/marley_forge_client/src/adapter.rs
  - crates/marley_forge_client/src/fleet.rs
  - crates/marley_fleet/src/reducer.rs
  - crates/marley_fleet/src/attention.rs
  - crates/marley_app/src/fleet_rail.rs
---

## Title
Close the #368 inspect LOW: today a transient Forge bounce permanently kills live fleet updates — the
pump's read loop breaks on server-close (`Ok(0)`, adapter.rs:219) and on any hard mid-stream error
(adapter.rs:232 — the #368 F6 fix: break, never hot-spin a dead socket), and an `open_listen_stream`
failure returns (adapter.rs:208-211); the thread then simply ENDS. No data loss — the durable cursor
(`<config_dir>/fleet-cursor`, adapter.rs:309-322) persists after every applied page (adapter.rs:277-280)
and a fresh `start` replays from `?since=<cursor>` (adapter.rs:197, fleet.rs:487) — but the death is
silent, which is the one failure a monitoring surface must not have ("Marley down ≠ fleet down",
orchestration-shell.md §6/:149). This ticket makes every one of those exits feed a reconnect loop
instead of thread death: a PURE backoff policy seam (full-jitter exponential delay, injected jitter
fraction + injected clock, cov/MSI 100) + the masked pump loop that waits (stop-responsive) →
re-initializes → replays from the cursor → re-subscribes → listens, forever until stopped.

## Scope
### In
1. **A PURE backoff policy seam — NEW `crates/marley_forge_client/src/backoff.rs`** (placement: D4).
   - `next_delay_ms(attempt, jitter_fraction) -> u64`: full jitter over the exponential ceiling —
     `delay = fraction × min(BASE_MS × 2^attempt, CAP_MS)` with saturating arithmetic (an
     until-stopped counter WILL exceed the shift width; overflow is a spec'd behavior, not UB).
     `jitter_fraction ∈ [0,1]` is an INJECTED parameter — no `rand` dep, no `Instant::now`/
     `SystemTime` inside the pure fn (D2; the `marley_fleet::attention` injected-`now_ms` idiom,
     attention.rs:1-2/:37).
   - Attempt-counter lifecycle with an injected clock: stable-connected ≥ `STABLE_RESET_MS` (measured
     from injected connected-at/disconnected-at millis) → attempt resets to 0; a failure before that →
     attempt increments (the next delay climbs).
   - `should_retry(cause) -> bool` over #372's TYPED disconnect surface: true for every transient
     cause (stream closed, hard mid-stream error, open failure, initialize fetch/parse failure,
     404-session-expired — whose recovery IS a re-initialize, Prior art leg 2); false only for the
     immutable config malformation (the builder-returns-`None` unsupported-URL case, adapter.rs:174-177
     — retrying cannot change a bad URL within a run). Exact enum granularity tracks #372's landed
     surface (Phase 2).
   - Suggested small pure helper: slice a delay into ≤1000 ms wait quanta (gives REQ-006's poll
     granularity a pure decision; Phase 2 decides whether it earns a fn).
2. **Pump integration (masked, `#[cfg_attr(test, mutants::skip)]`, coverage-excluded — the accepted-
   untestable adapter layer).** Restructure `run_subscription` (adapter.rs:166-252) into an outer
   until-stopped loop: one connection cycle = today's body (initialize → `notifications/initialized` →
   replay-refresh from the durable cursor → subscribe → post-subscribe gap-window refresh, kept →
   listen), now returning #372's typed disconnect cause instead of dying. On a cause: if
   `should_retry` → wait `next_delay_ms` polling the stop flag at ≤1s granularity (the existing 1s
   read-timeout idiom, adapter.rs:298) → next cycle re-runs `initialize` (fresh `Mcp-Session-Id`),
   replays `resources/read` from the persisted cursor, re-subscribes, listens. The typed 404-session
   signal takes the SAME path (a re-initialize IS the recovery). No event loss or duplication: the
   #367 reducer is idempotent under overlap re-delivery — assignments + a monotone `max`-join
   (reducer.rs:138-140/:199-201, replay-safety test reducer.rs:255-287) — so a cursor'd catch-up that
   overlaps already-applied events converges to the same snapshot. The F6 no-hot-spin property is
   PRESERVED: every retry is separated by a real backoff wait; the hard-error break still exits the
   read loop — it now feeds the retry loop instead of thread exit.
3. **Retry policy: FOREVER at the capped interval until the stop flag (D1 — LOCKED, until-stopped;
   max-attempts-then-dead REJECTED).**

### Out (explicitly deferred)
- **New rail chrome / connection-state UI.** The existing staleness indicator already communicates
  degradation — `STALE_AFTER_MS = 30_000` + the rail's stale rendering
  (crates/marley_app/src/fleet_rail.rs:17/:346-356, on `marley_fleet::attention::is_stale`); while the
  pump is down-and-retrying, seats go stale honestly. No new indicator ships here.
- **Any change to the wire contract or the #368 fixture frames** — the protocol is untouched; this is
  client lifecycle only.
- **#372's harness itself** (the loopback fixture-Forge server, the typed disconnect surface, the
  session-id echo/404 typing) — consumed here, built there. HARD ORDER: #372 lands first.
- Backoff parameters as user config (D3 — constants this slice).

## Reference (§20)
**N/A — Marley-specific reconnect plumbing; no reference-app behavior analog.** This hardens Marley's
own MCP fleet subscription (a Marley-original architecture — orchestration-shell.md §6, shipped in
#368); neither Warp nor Zed has an observable "reconnect to the fleet control plane" behavior to
match, and the maps were checked (Prior art leg 1 — zero relevant hits). The reconnect policy itself
is adopted from PUBLISHED distributed-systems practice (AWS full jitter) + the PUBLISHED MCP spec
(leg 2) — no Warp/Zed source read, no behavior capture needed.

### Prior art
1. **Behavior maps (docs/warp_architecture/, docs/zed_architecture/): N/A — checked.** Grepped both
   maps for reconnect/backoff/SSE/subscription: zero hits; nothing there covers a client's own
   control-plane connection lifecycle. Recorded as checked-and-not-applicable, not skipped.
2. **Published material.** (a) The AWS Architecture Blog **"Exponential Backoff and Jitter"**
   (Brooker) — the **full-jitter** variant: `sleep = random(0, min(cap, base·2^attempt))`; jitter over
   the full interval beats no-jitter and equal-jitter for contention. Adopted with the random input
   INJECTED as a fraction (D2) — the policy stays a pure function. (b) The **MCP spec (2025-06-18,
   Streamable HTTP transport lifecycle)**: the server MAY assign an `Mcp-Session-Id` at `initialize`;
   the client echoes it on every later request; when the server answers a session-bearing request with
   **HTTP 404, the client MUST start a NEW session with a fresh `initialize`** — i.e. re-initialize IS
   the published recovery, exactly what the reconnect cycle does (and why the typed 404 signal from
   #372 routes into the same path).
3. **Our own source (the load-bearing leg — all verified this draft).**
   - **The current on-close behavior** (`crates/marley_forge_client/src/adapter.rs`): read loop
     `while !stop.load` (:217); `Ok(0) => break` — server closed (:219); `WouldBlock|TimedOut =>
     continue` — the expected 1s wakeup to re-check `stop` (:224-231); **any other error `=> break`**
     (:232) — the #368 F6 hot-spin fix ("must break, not spin the dead socket hot", :220-223);
     `open_listen_stream` `Err => return` (:208-211); initialize fetch/parse failure and
     `!supports_subscribe` → `return` (:174-187). Every exit ends the thread — the defect this ticket
     fixes. Stop flag: `Arc<AtomicBool>` (:134, :146-149, Drop joins :152-160); 1s read timeout on the
     standing stream (:298).
   - **The durable cursor + why a manual restart already resumes:** `FleetSync::with_cursor(
     load_cursor_in(config_dir))` seeds each run (:197); `refresh` persists the advanced cursor after
     EVERY applied page (`save_cursor_in`, :277-280); storage is `<config_dir>/fleet-cursor`
     (:309-322). `FleetSync` (fleet.rs:441, `with_cursor` :453, `next_read_uri` :487 →
     `fleet://events?since=<cursor>`; reconnect-replay test `t368_req008` fleet.rs:1074-1087). The
     reconnect loop reuses this machinery verbatim — replay-from-cursor per cycle is already how boot
     works.
   - **Session id today:** `parse_initialize` reads the `mcp-session-id` header (fleet.rs:227-243,
     `Initialized.session_id` :221-222) and every builder echoes it (fleet.rs:164/:181/:195/:201). A
     non-2xx is currently an UNTYPED `ForgeError::Http(status_line)` (lib.rs:252-255) — the typed 404
     discrimination is #372's deliverable, consumed here.
   - **The injected-clock idiom (D2's precedent):** `marley_fleet::attention` — "NO clock lives here —
     `now_ms` and `stale_after_ms` are always injected" (attention.rs:1-2; `is_stale(session, now_ms,
     stale_after_ms)` :37-38; lib.rs:12). Same idiom app-side: `demo_snapshot(now_ms: u64)`
     (fleet_rail.rs:117-121) and `acknowledgment_is_stale(acked, now: &DiskState)`
     (crates/marley_app/src/extchange.rs:82).
   - **Existing backoff/retry helpers: NONE — a no-owner PASS.** Grep backoff/retry/reconnect across
     crates/: the only kin is `terminal_blocks::session`'s `PUMP_RETRY_BUDGET = 8`
     (crates/terminal_blocks/src/session.rs:32/:119-123) — a consecutive-`WouldBlock` PTY-write budget
     (immediate retries, then FAIL — a bounded in-burst affair, not a timed reconnect policy), and the
     LSP host's deliberate does-not-retry posture (crates/marley_app/src/app.rs:4742). Nothing owns a
     timed exponential-backoff seam; this ticket introduces the first one.
   - **The no-dupe/no-gap guarantee:** the #367 reducer — "Idempotent under re-delivery (assignments +
     a `max` join), so a cursor'd catch-up that overlaps already-applied events converges to the same
     snapshot" (crates/marley_fleet/src/reducer.rs:138-140; the monotone max-join :199-201; the
     overlap-replay-is-noop test :255-287).

## Locked-In Decisions
- **D1 — Retry FOREVER at the capped interval until the stop flag (until-stopped). REJECTED:
  max-attempts-then-dead.** (Batch-owner lock; verified nothing contradicts it — the adapter has no
  retry cap today, `PUMP_RETRY_BUDGET` is a different animal (an in-burst write budget), and no doc
  mandates giving up.) Rationale: the fleet rail is a MONITORING surface — "Marley down ≠ fleet down,
  honestly" (orchestration-shell.md §6/:149); a monitor that silently gives up after N attempts
  recreates the very silent-death failure this ticket exists to fix, just later. The capped interval
  (D3) bounds the cost to one cheap localhost attempt per ~30s forever, and the #367/#369 staleness
  indicator (fleet_rail.rs:17) already tells the user the feed is degraded meanwhile. The stop flag +
  Drop (adapter.rs:146-160) remain the ONLY exits besides `should_retry`'s immutable-config-error arm.
- **D2 — Injected jitter fraction + injected clock; full jitter. REJECTED: a `rand` dep (a whole
  dependency for one fraction — and the pure seam would still need injection for determinism);
  REJECTED: jitterless (thundering-herd is trivial to avoid when the fix is one injected parameter —
  and full jitter is the published winner, Prior art leg 2).** The pure fns take `jitter_fraction:
  f64 ∈ [0,1]` and `now_ms: u64` parameters — the `marley_fleet::attention` precedent (attention.rs:
  1-2/:37) — so delay/reset decisions are pure functions of their inputs. The MASKED layer supplies
  ambient authority: the fraction derives from subsecond system-clock nanos (entropy already available
  in the adapter, no new dep), the clock from the existing time source. Ambient authority stays in the
  masked file, decisions stay pure — the crate's charter (adapter.rs:1-2).
- **D3 — Backoff parameters are NAMED CONSTANTS this slice, not config: `BASE_DELAY_MS = 500`,
  `CAP_DELAY_MS = 30_000`, `STABLE_RESET_MS = 30_000`. REJECTED (deferred): a settings knob.** No
  consumer needs tuning yet; a config surface would ship untestable permutations ahead of any demand
  (the #367 "NO default staleness const" fork went the other way because the threshold's OWNER was
  another ticket — here the pump is the only consumer). `CAP/STABLE = 30_000` deliberately resonates
  with `STALE_AFTER_MS = 30_000` (fleet_rail.rs:17): at the cap the pump attempts about once per
  staleness window, and a connection must outlive one full staleness window to count as stable.
  Config = a future ticket if a real tuning need appears.
- **D4 — The pure module lives at `crates/marley_forge_client/src/backoff.rs` (new top-level pure
  sibling). REJECTED: `marley_fleet` (its charter is transport-free — "pure (cov/MSI 100, no gpui, no
  transport)", orchestration-shell.md §7 table :157 — and reconnect policy is transport lifecycle);
  REJECTED: inside `adapter.rs` (masked + coverage-excluded — pure code there would dodge the
  floors); REJECTED: growing `fleet.rs` (~1100 lines of fleet-vocabulary projection; backoff is a
  distinct lifecycle concern).** The crate's layout is exactly "pure decision modules at the top level
  (lib.rs, fleet.rs) + one masked adapter.rs whose header says 'All the decisions live in the pure
  parent module'" (adapter.rs:1-2) — a new small pure sibling matches it and keeps
  `cargo mutants -f backoff.rs` a crisp file target.
- **D5 — Re-initialize EVERY cycle (fresh session), never attempt to resume the old
  session/stream. REJECTED: session-resume/`Last-Event-ID`-style stream resumption.** The 404 rule
  (Prior art leg 2) makes re-initialize the published recovery for an expired session, and the durable
  cursor + idempotent reducer make a full fresh cycle LOSSLESS anyway — so one uniform path covers
  every cause (close, error, open-failure, 404) with zero session-state bookkeeping. This is also
  exactly what a manual app restart already does, i.e. the already-proven path.
- **D6 — Mutant-set honesty: leave the real set to `cargo mutants --list -f` on the ACTUAL touched
  files at Validate** (PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators + the
  syntactic-form rule). Note for Design: whether `should_retry`'s cause enum / any policy struct
  derives `Default` changes body-replacement viability (the #203/#204 lesson — a `Default`-less enum
  return makes the body mutant unviable); choose derives deliberately, then trace.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the pure policy computes delays for attempts 0,1,2,… with the jitter fraction pinned at 1.0, the system shall yield the deterministic doubling sequence `min(500·2^attempt, 30_000)` ms (500, 1000, 2000, 4000, 8000, 16000, 30000, 30000, …) — reaching and HOLDING the cap — with saturating arithmetic at large attempt indices (no overflow/panic at attempt ≥ 64). | pure units: exact expected sequence incl. two capped tail entries + an `attempt = u32::MAX`-region probe == cap |
| REQ-002 | WHEN a jitter fraction `f ∈ [0,1]` is injected, the system shall bound the delay within `[0, min(500·2^attempt, 30_000)]`, scaling full-jitter-style (`f=0 → 0`, `f=1 → ceiling`, monotone in between). | pure units: f = 0 / 0.5 / 1.0 at a below-cap and an at-cap attempt; bounds asserted |
| REQ-003 | WHEN a connection has stayed up ≥ `STABLE_RESET_MS` by the injected clock, the policy shall reset the attempt counter to 0 (the next disconnect backs off from `BASE_DELAY_MS` again); WHEN it fails sooner, the counter shall increment (the next delay climbs). | pure units with injected millis: exactly-at-threshold, just-under, just-over (the boundary mutants `<`/`<=` both orientations — PR-claude-two-comparison-overlap-needs-boundary-per-side) |
| REQ-004 | WHEN the pump observes a #372-typed disconnect cause, `should_retry` shall return true for every transient cause (stream closed, hard mid-stream error, open failure, initialize fetch/parse failure, 404-session-expired) and false for the immutable config malformation (unsupported endpoint URL); the 404-session cause shall route to the re-initialize path. | pure units, one per cause arm (exhaustive over #372's landed enum) |
| REQ-005 | WHEN the #372 loopback fixture-Forge server kills the connection after delivering event 3 of N and the pump auto-reconnects, the final `FleetSnapshot` shall be IDENTICAL to an unbroken run of the same N events — no gap, no duplicate. | integration on the #372 harness (tempdir config_dir for the cursor): kill-after-event-3 run vs unbroken control run, `assert_eq!` on the final snapshots |
| REQ-006 | WHILE the pump is waiting out a backoff delay (up to the 30s cap), WHEN the stop flag is set, the system shall exit the wait and end the thread within ~1s (the wait polls stop at ≤1s granularity — the adapter.rs:298 read-timeout idiom). | integration: trigger a capped backoff on the harness, `stop()` mid-wait, thread joins well under 2s; plus a pure unit on the wait-slicing helper if Phase 2 extracts it (else a masked-behavior note naming the glue) |
| REQ-007 | WHEN the server answers a session-bearing request with the typed 404-session signal, the pump shall re-initialize (obtaining a fresh `Mcp-Session-Id`), replay from the durable cursor, and resume live updates. | integration IF the #372 harness can stage a 404 response (Phase 2 confirms against the landed harness); ELSE REQ-004's pure decision test + a masked-behavior note that the 404→re-init glue is the same masked cycle REQ-005 proves |
| REQ-008 | The pure backoff seam (`backoff.rs`) shall ship at cov 100 / MSI 100; the pump loop shall stay masked (`mutants::skip` + coverage-excluded); test code shall contain no never-run branches. | gate:4/gate:5 exit codes; `cargo mutants --list -f backoff.rs` traced at Validate; inspect checklist |

## Phase Plan
- **P2 Design** — align with #372's LANDED harness + typed-cause enum (this is the hard-order point);
  exact pure signatures (policy fns vs a small `Backoff` state struct — decide, then check
  `Default`-derive viability per D6); the masked-loop restructure sketch for `run_subscription`;
  file manifest: `crates/marley_forge_client/src/backoff.rs` (NEW pure) · `src/lib.rs` (`mod backoff`
  + re-exports) · `src/adapter.rs` (masked outer loop; every `return`/`break`-to-death becomes a typed
  cause into the loop) · the #372 harness test file grows the kill/reconnect + stop-mid-wait cases;
  per-REQ test manifest; run `cargo mutants --list -f` on the ACTUAL touched files at Validate (D6).
- **P3 Implement** — `backoff.rs` + the masked loop per design; constants per D3; no wire/fixture
  changes.
- **P3.5 Inspect** — independent critics vs the diff; hot-spin audit (every retry path provably waits
  or exits — F6 preserved); stop-responsiveness audit on every new wait; D1 audit (no hidden give-up
  path).
- **P4 Validate** — write + RUN the planned tests (pure units + the #372-harness integrations);
  re-run `cargo mutants --list -f backoff.rs` on the ACTUAL code and kill the real set; gate green
  (cov/MSI 100 on the pure seam; adapter stays masked).
- **P5 Complete** — archive; update orchestration-shell.md §7 row (#368 line gains "self-healing");
  AAR capture; close forge #373.
