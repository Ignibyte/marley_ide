---
pipeline_id: e838eba5-3ccd-490d-82b8-214aca946b19
ticket: forge#376 (f2983e9d-fb30-4df3-a613-0b41d5609afd) · local docs/planning/tickets/open/TICKET-376-fleet-rail-livewire.md
aar_id: 08b3cd95-dcb0-43e7-868b-2027288cdc9e
status: Phase 5 — Complete PASS
title: Fleet rail live wire — app-side FleetSubscription from [[projects.orchestration]]
type: feature
milestone: M24
references:
  - docs/marley_architecture/orchestration-shell.md
  - docs/marley_architecture/fleet-control-plane.md
  - crates/marley_forge_client/src/adapter.rs
  - crates/marley_forge_client/src/backoff.rs
  - crates/marley_forge_client/src/fleet.rs
  - crates/marley_forge_client/tests/livewire.rs
  - crates/marley_app/src/fleet_rail.rs
  - crates/marley_app/src/orchestration.rs
  - crates/marley_app/src/settings.rs
  - crates/marley_app/src/app.rs
---

## Title
Wire the two shipped halves of Layer 1 together — the FIRST live feed into the fleet rail. Today the
rail (#369) renders only the `fleet-demo-feed` fixture (`app.rs:8145-8147`, `CommandId(29)`
:8355-8360): the self-healing `FleetSubscription` (#368; fixture-proven #372; until-stopped
reconnect+backoff #373) has **no app-side caller**, and `orchestration_for` (#371) has **no
consumer** — `marley_app/src/lib.rs:98-99` re-exports it expressly "so it isn't dead-code before its
#368/Phase-E consumers land". Both gaps are the §12.1 runbook's named **precondition 1**
(orchestration-shell.md:305-311). This ticket is that wiring: at boot (the settings-apply path), when
`orchestration_for(active project root)` yields a loopback `brain_endpoint`, start exactly one
subscription on its own background thread; snapshots flow through the existing gpui pump into
`RootView.fleet_snapshot`; the rail header shows connection state (live / reconnecting / off); quit
stops the thread cleanly. Unconfigured roots keep today's behavior verbatim (demo verb intact).

## Scope
### In
- **NEW pure app-side wiring seam** (candidate: `crates/marley_app/src/fleet_live.rs`, cov/MSI 100):
  the **start-gate** decision (`&[ProjectOrchestration]` + active root → the endpoint spec to
  subscribe to, or `None` — composing the shipped `orchestration_for`), the **state→label** fn
  (`Option<ConnectionState>` → `"live"`/`"reconnecting"`/`"off"`; `None` = no subscription = off),
  and the **supersede** decision for live-vs-demo (D5; mechanism D-OPEN-DEMO-MECH). Module placement
  vs folding the label fn into `fleet_rail.rs` is Phase 2's call — the fns are pure either way.
- **`marley_forge_client` grows two small surfaces** (the crate's decisions stay pure, the loop edits
  masked): **(a)** a PUBLIC endpoint constructor from a raw URL + bearer — today `ForgeEndpoint` only
  constructs via `forge_endpoint_from(.mcp.json)` (lib.rs:123-140) — carrying the SAME
  `is_loopback_authority` guard (D6) and the redacted-`Debug` bearer contract (lib.rs:42-49);
  **(b)** a typed `ConnectionState` the `run_subscription` loop publishes through a shared cell and
  the `FleetSubscription` handle exposes — today the handle exposes only `exit_reason()`, which is
  `None` both while healthy AND while reconnecting (adapter.rs:126-129/:171-179), so live-vs-
  reconnecting is NOT observable; the publish points ride the already-masked loop (D-OPEN-STATE-
  POINTS names a pure transition fn as the candidate for the decision half).
- **Boot wiring** (masked, app.rs): after the settings load (app.rs:1256-1265, `applied_from` →
  `AppliedSettings.project_orchestrations`, settings.rs:224-225/:424) and project discovery
  (app.rs:1276-1281), run the start-gate; on `Some`, `FleetSubscription::start` with the config dir
  (the durable-cursor home, adapter.rs:447-463) and a callback that hands each snapshot to the
  handoff cell. Mirrors the `.mcp.json` forge-client build precedent one screen down (app.rs:1287-91).
- **Pump drain** (masked): the existing gpui pump loop (app.rs:1295+, the `pump_mcp_host` take/put
  idiom :6569-6580) drains the handoff into `self.fleet_snapshot` and reads the state cell; any
  change SETS DIRTY (`PR-claude-pump-state-change-must-set-dirty-to-repaint`, the #203 lesson).
- **Rail header indicator**: the right-dock header is `caption_header(dock_title(DockSide::Right))` =
  "Fleet" (app.rs:1130-1133; layout.rs:65-69, tested :373-376). The indicator renders from the pure
  label fn ONLY; placement (header suffix vs a chip atop `fleet_rail_body`) = D-OPEN-INDICATOR.
- **Clean stop on quit**: hold the handle in `RootView`; teardown rides the SHIPPED stop-flag +
  join `Drop` (adapter.rs:165-190), wired on the app's orderly quit path per #375-D7's precedent
  (host-teardown-on-drop; gpui `on_app_quit` as the grounded wiring candidate).
- **App-level proof against a fixture forge server** (the #372 livewire pattern, livewire.rs:46-104):
  a marley_app integration test drives the REAL `FleetSubscription` + the new wiring seam against a
  scripted fixture — snapshot convergence, reconnect retention, stop. Harness shape = D-OPEN-HARNESS.
- **Regression pins:** the #369 rail suite + demo verb unchanged for unconfigured roots; the #373
  stop/reconnect livewire scenarios stay green; the never-logged-bearer + loopback-only postures.

### Out (explicitly deferred)
- **Multi-endpoint fan-out** — one subscription, the active project's entry only (D1); N projects ×
  N brains is a later slice.
- **Seat detail pane** (click-through, per-seat drill-in) — a separate M24 slice.
- **Any write verb** — `session.send`, question answering, dispatch: Layer 2's gated writes.
- **The L0 / live-ucsosv2 run** — §12.1's verify steps (a)-(e) stay the L0-day ticket; this ticket
  makes precondition 1 true, it does not perform the real-forge run.
- **A runtime settings-reload path** — none exists today (settings load once at boot, app.rs:1256);
  inventing one is not this ticket. A future settings-apply re-runs the same start-gate.
- **A bearer field in `[[projects.orchestration]]`** — the #371 schema is untouched (D-OPEN-BEARER
  picks among existing sources only).
- **Surfacing `SubscriptionExit` detail in the UI** beyond the three header states.

## Reference (§20)
**N/A — Marley-specific; no reference-app analog.** The fleet control plane is Marley's own design:
push = MCP subscription is the decided fork ① (docs/marley_architecture/orchestration-shell.md §6/§11),
the crate map is §7, and §12.1's runbook precondition 1 names THIS wiring verbatim ("`FleetSubscription::start`
has NO app-side caller yet… The live client must be wired to start from the configured
`brain_endpoint` and feed the rail's snapshot"). The checked behavior map
`docs/warp_architecture/subsystems/04-agent-ai-mcp.md` shows Warp's multi-agent surface is a CLOUD
service client (`warp_multi_agent_client` SSE → protobuf `ResponseEvent`s to `…/ai/multi-agent`) — a
product-cloud conversation stream, not a local-brain fleet rail; there is no behavior to observe or
match. Clean-room §20 untouched: no Warp (AGPL) / Zed (GPL) source consulted, only our own maps.

### Prior art
1. **Behavior maps — checked, no owner.** `docs/warp_architecture/subsystems/04-agent-ai-mcp.md`
   (Warp: cloud multi-agent SSE client — confirms "standing stream + client-side projection" is the
   normal shape, maps nothing local); `docs/zed_architecture/` has no fleet/agent-rail analog.
   Research only, per the sweep rule.
2. **Published — the MCP spec (2025-06-18) `resources/subscribe` + `notifications/resources/updated`:**
   already adopted wholesale by #368/#370/#375; the push-transport fork is DECIDED
   (orchestration-shell §6 — never raw Postgres). This ticket adds NO protocol surface — it consumes
   the shipped client as-is.
3. **OUR OWN shipped crates — the load-bearing leg; this ticket is almost entirely adoption.**
   Per-seam ownership, recorded:
   - **Subscription lifecycle: OWNED, SHIPPED.** `FleetSubscription::start/stop/Drop`
     (adapter.rs:123-190) spawns/joins the thread; `run_subscription` (:200-234) is the until-stopped
     reconnect loop over pure `backoff` decisions (backoff.rs:30-59, cov/MSI 100); `run_once`
     (:282-383) sequences initialize→replay→subscribe→listen. Nothing to build.
   - **Stop-responsiveness: OWNED.** Stop polled every ≤1s (`backoff_wait` :239-251 +
     `POLL_QUANTUM_MS`/`wait_slice_ms` backoff.rs:20/:57-59; the SSE stream's 1s read timeout
     adapter.rs:439); a mid-`fetch` stop is bounded by the 5s socket timeout (:102). REQ-004 is
     largely verify-and-pin at the app seam.
   - **No-loss retention: OWNED.** ONE `FleetSync` per thread life, carried across reconnects
     (adapter.rs:210-213); the livewire strict-delta scenario (PAGE_C, livewire.rs:40-43) proves a
     delta read keeps the retained seats. REQ-005's crate half is shipped; the app half = never
     clear `fleet_snapshot`.
   - **Endpoint construction: GAP.** No public `ForgeEndpoint` constructor from a raw URL — only
     `forge_endpoint_from(.mcp.json)` (lib.rs:123-140); every request embeds `Authorization:
     {bearer}` in the lib.rs framing (:187/:213). The crate must grow constructor (a); the loopback
     guard carries (§12.1 step 3 pins exactly this: "the endpoint refuses to construct for a
     non-loopback host, lib.rs `is_loopback_authority`").
   - **Connection state: GAP.** `exit_reason()` is the handle's ONLY signal and it cannot
     distinguish live from reconnecting (adapter.rs:126-129 — `None` covers both). The loop must
     publish a typed state (surface (b)).
   - **Start-gate inputs: OWNED.** `orchestration_for` exact-root match (orchestration.rs:25-30,
     tested; re-exported UNUSED at marley_app/src/lib.rs:98-99); `ProjectOrchestration
     { root, brain_endpoint, web_url }` with serde defaults (orchestration.rs:11-21);
     `AppliedSettings.project_orchestrations` round-trip (settings.rs:94-97/:224-225/:424, tolerance
     tests :1055-1099); boot root discovery (app.rs:1276-1281).
   - **Handoff + repaint: OWNED PATTERN.** The gpui pump loop (app.rs:1295+); the
     `pump_mcp_host` take/put idiom (:6569-6580); `PR-claude-pump-state-change-must-set-dirty-to-repaint`.
   - **Rail render: OWNED.** `fleet_rail` pure decisions (state chips, question-card gate, staleness,
     `demo_snapshot` — fleet_rail.rs:20-197, cov/MSI 100) + the `fleet_rail_body` shim (app.rs:962+)
     + the "Fleet" dock header (app.rs:1130-1133, layout.rs:65-69).
   - **Proof harness: OWNED PATTERN.** livewire.rs `FixtureForge`/`Script` (:46-160) — a real
     `TcpListener` scripted forge speaking the pump's actual dialect; its header comment (:26-30)
     establishes duplication-as-contract-pin (`#[cfg(test)]` fixtures aren't importable; "a drift is
     a LOUD test failure we want").

## Locked-In Decisions
- **D1 — v1 subscribes for the ACTIVE project's orchestration entry only; exactly ONE subscription.**
  Trigger = boot, which IS the settings-apply path today (settings load once, app.rs:1256-1265; no
  runtime reload exists). Multi-endpoint fan-out is Out. Whether an active-project SWITCH re-targets
  the one subscription is D-OPEN-RETARGET — either answer preserves the exactly-one invariant.
- **D2 — stop via the EXISTING stop-flag idiom, no new mechanism.** `RootView` holds the
  `FleetSubscription`; quit teardown = the handle's shipped `stop()` + join `Drop`
  (adapter.rs:165-190), wired on the orderly quit path per #375-D7's host-teardown precedent (gpui
  `on_app_quit` the grounded candidate). Bounded: stop honored within the ~1s poll quantum; a
  mid-`fetch` stop within the 5s socket timeout. **Rejected:** detaching the thread (an orphan
  socket past quit); a second flag/channel duplicating the shipped one.
- **D3 — connection state = a typed CLOSED enum in a shared cell the loop updates + a PURE
  state→label fn.** `marley_forge_client` owns the enum + cell + handle accessor (only the loop
  knows the truth); the app maps `Option<state>` → `"live"`/`"reconnecting"`/`"off"` in a pure fn
  (`None` = no subscription = off — ABSENCE is app knowledge, not a loop state). **Rejected:**
  inferring liveness from snapshot age (a quiet-but-healthy fleet reads dead — the #369
  dead-vs-idle line, again); rendering from `exit_reason()` (provably cannot distinguish the two
  live states — adapter.rs:126-129).
- **D4 — snapshot handoff is drained in the EXISTING app pump, and a pump-installed change sets
  dirty** (`PR-claude-pump-state-change-must-set-dirty-to-repaint` — the rule exists because a pump
  state-change on an idle frame otherwise never repaints). The exact container is D-OPEN-HANDOFF;
  whichever wins, the `on_update` callback only deposits-and-returns (it MUST NOT panic — the
  documented pump contract, adapter.rs:136-139).
- **D5 — the demo verb is RETAINED; live supersedes, observably.** `fleet-demo-feed`
  (CommandId(29)) stays dispatchable; whenever the live subscription delivers, the delivered
  snapshot replaces whatever the rail held (demo included). The mechanism (gate the verb while live
  vs plain last-writer-wins) is D-OPEN-DEMO-MECH; the observable is locked.
- **D6 — the loopback guard CARRIES to the settings-sourced endpoint.** A non-loopback
  `brain_endpoint` never constructs an endpoint (reuse `is_loopback_authority`) — a tampered
  settings file must not point a bearer-carrying client at an arbitrary host (the lib.rs D1 posture,
  restated by §12.1 step 3). The redacted-`Debug`/never-logged bearer contract extends to every new
  surface.
- **D7 — a transient disconnect never clears the rail.** The app NEVER resets `fleet_snapshot` on an
  exit/reconnect; the crate half (carried `FleetSync` + delta-keeps-seats) shipped in #373.

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-HANDOFF** — latest-wins `Arc<Mutex<Option<FleetSnapshot>>>` vs an mpsc drain. Candidate:
  latest-wins cell (snapshots are cumulative; dropped intermediates are harmless; no unbounded queue).
- **D-OPEN-BEARER** — the settings-sourced endpoint's Authorization value: (a) reuse the project
  root's `.mcp.json` bearer when present (today's forge-client source, app.rs:1287-1291 — v1's brain
  IS the forge sidecar), (b) empty bearer, (c) `.mcp.json`-when-present-else-empty. Never a new
  settings field (Out).
- **D-OPEN-RETARGET** — restart the subscription when the active project switches, vs
  boot-resolved-only for v1 (stop+start is cheap; the durable cursor is per-config-dir — check the
  #368 D-OPEN-CURSOR-STORE resolution's per-endpoint story before locking).
- **D-OPEN-STATE-POINTS** — the exact publish points for D3's states (handshake-complete vs first
  successful refresh ⇒ live; a transient exit entering backoff ⇒ reconnecting). Candidate: a pure
  transition fn so the decision half is unit-testable, the masked loop only calling it.
- **D-OPEN-DEMO-MECH** — D5's mechanism: verb-gated-while-live vs last-writer-wins.
- **D-OPEN-HARNESS** — the app-level fixture: a MINIMAL scripted forge duplicated into a marley_app
  integration test (the livewire duplication-as-contract precedent, livewire.rs:26-30) vs promoting
  the #372 harness to an exported fixture (leaks test surface into the shipped crate — the reason
  #372 duplicated). Candidate: minimal duplicate driving the extracted wiring seam + a real
  `FleetSubscription`.
- **D-OPEN-INDICATOR** — indicator placement: "Fleet" header suffix vs a chip atop the rail body
  (display detail; the pure label fn is locked either way).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the app boots and the active project's root has an orchestration entry with a loopback `brain_endpoint`, the app shall start EXACTLY ONE fleet subscription against that endpoint, and the rail shall render each delivered live snapshot. | pure start-gate units (entry+endpoint → `Some(spec)`; multi-entry picks the exact root); app-level fixture integration: scripted forge → the rail-bound snapshot converges to the fixture's seats (bounded deadline poll, the livewire idiom); §18.1 inspect: ONE call site on the boot path |
| REQ-002 | WHILE the app runs, the rail shall show a connection-state indicator derived from the TYPED subscription state — "live" when the stream is healthy, "reconnecting" between attempts, "off" when no subscription exists — never inferred from snapshot age. | pure label units (every enum arm + `None`→"off", exhaustive match); fixture: healthy ⇒ live, forge killed ⇒ reconnecting; inspect checkpoint: the pump sets dirty on a state change (PR-claude-pump-state-change-must-set-dirty-to-repaint) |
| REQ-003 | WHEN the active project's root has NO orchestration entry (or an entry without `brain_endpoint`), the app shall start NO subscription and today's behavior shall be unchanged — the #369 rail (empty-state, demo verb `fleet-demo-feed`, toggle) behaves verbatim and the indicator reads "off". | pure start-gate negative units (no entry / `brain_endpoint: None` → `None`); regression: the full #369 `fleet_rail` + demo-verb suites green UNCHANGED; label unit `None`→"off" |
| REQ-004 | WHEN the app quits while a subscription is running, the app shall signal stop and join the thread — the stop flag honored within the ~1s poll quantum (a mid-fetch stop bounded by the 5s socket timeout), no hang, and the thread's exit reason shall be `Stopped`. | fixture integration: stop/drop mid-stream returns promptly with `SubscriptionExit::Stopped` (extends the shipped #373 stop scenarios to the app seam, with a bounded-deadline assert); §18.1 inspect on the quit-path wiring (the handle is dropped on the orderly quit) |
| REQ-005 | WHILE the endpoint is unreachable or bouncing, the rail shall keep rendering the LAST delivered snapshot (never cleared) and the indicator shall read "reconnecting"; WHEN the connection recovers, a strict post-cursor delta read shall leave the previously-known seats intact (the #373 carry-forward — no data loss). | fixture: kill the forge mid-run → snapshot retained + state = reconnecting; revive scripted with a strict-delta page → seats = the union (the livewire PAGE_C scenario replayed at the app seam); inspect: no clear-on-error arm exists on the drain path |
| REQ-006 | WHEN a configured `brain_endpoint` names a NON-loopback host, the app shall construct no endpoint and start no subscription — the bearer never leaves loopback — behaving exactly as unconfigured. | pure constructor units (loopback URL ⇒ `Some`; non-loopback/garbage ⇒ `None` — mirroring the shipped `is_loopback_authority` suite); start-gate negative unit; §18.1 inspect: no alternate construction path bypasses the guard |
| REQ-007 | WHEN the demo verb has installed a snapshot and the live subscription then delivers, the live snapshot shall replace the demo one (live supersedes); the demo verb shall remain dispatchable and fully functional when no subscription exists. | unit on the supersede decision (per D-OPEN-DEMO-MECH's pick); fixture: demo-feed then live delivery → the rail holds the live seats; regression: demo verb with no subscription still installs the fixture snapshot |

## Floors (constitution)
Pure seams at **cov/MSI 100**: the start-gate, the endpoint constructor + loopback guard, the
state→label fn (exhaustive `match` over the closed enum — no defensive catch-all), any state
transition fn, the supersede decision. MASKED: the thread/cell/callback glue, the boot call site, the
pump drain, the quit wiring (adapter.rs + app.rs are already coverage-excluded). Typed errors, no
`unwrap` on input paths; the `on_update` callback deposits-and-returns (no panic — the documented
pump contract). At Validate, run `cargo mutants --list -f` on the ACTUAL touched files (the
syntactic-form lesson) and re-verify neighboring `#[mutants::skip]` bindings (the skip-detach trap —
new fns near masked shims).

## Phase Plan
- **P2 Design** — exact module map + signatures: the wiring seam (fleet_live.rs vs fleet_rail.rs
  placement), the `marley_forge_client` constructor (a) + `ConnectionState` surface (b) with its
  publish points, the handoff container, the boot/pump/quit call sites, the header render delta;
  settle every D-OPEN with evidence (HANDOFF, BEARER, RETARGET, STATE-POINTS, DEMO-MECH, HARNESS,
  INDICATOR); per-REQ test plan incl. the app-level fixture script.
- **P3 Implement** — pure seams first (gate, constructor, label, transition, supersede), then the
  crate's masked loop delta, then the app glue (boot start, pump drain + dirty, header, quit drop).
- **P3.5 Inspect** — adversarial: can TWO subscriptions ever start (double-boot, retarget races)?
  does any path log/format the bearer or endpoint? does the drain path ever clear the snapshot? does
  the indicator ever render from anything but the typed state? is the quit join genuinely bounded?
  provenance (§20).
- **P4 Validate** — write + RUN the units and the fixture integration per REQ; `cargo mutants --list
  -f` on the actual touched files before claiming the kill set; gate green (`--diff`), cov/MSI 100
  on the pure seams; the #369 + #373 regression suites green unchanged.
- **P5 Complete** — CHANGELOG; tick §12.1 precondition 1 in orchestration-shell.md (wiring shipped;
  the L0-day run remains); AAR capture; archive; close #376.
