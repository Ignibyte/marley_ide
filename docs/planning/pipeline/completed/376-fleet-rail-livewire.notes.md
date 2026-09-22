# Fleet rail live wire — Notes

- **Forge ticket:** #376 f2983e9d-fb30-4df3-a613-0b41d5609afd (feature, sprint #35 "M24 — Fleet Layer 2")
- **AAR:** 08b3cd95-dcb0-43e7-868b-2027288cdc9e (opened at /work promotion, 2026-07-21)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-376-fleet-rail-livewire.md
- **Pipeline spec:** 376-fleet-rail-livewire.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** M24 Layer-2 opener — the §12.1 runbook's precondition 1 made real. The fleet rail
  (#369) still renders only the `fleet-demo-feed` fixture; `FleetSubscription` (#368, hardened
  #372/#373) has NO app-side caller and `orchestration_for` (#371) has NO consumer. Wire them: boot
  start when the active root's `[[projects.orchestration]]` entry has a loopback `brain_endpoint`,
  snapshots through the existing pump into `RootView.fleet_snapshot`, a live/reconnecting/off header
  indicator, clean stop on quit, unconfigured roots byte-identical to today.
- **Classification / tier:** feature, ONE slice, medium. Almost entirely ADOPTION of shipped seams —
  the genuine deltas are (i) the app-side start-gate/handoff/indicator decisions (new pure module),
  (ii) two small `marley_forge_client` growths (a public URL+bearer endpoint constructor with the
  loopback guard; a typed `ConnectionState` published by the loop — `exit_reason()` provably cannot
  distinguish live from reconnecting), (iii) masked glue (boot start, pump drain + dirty, quit drop).
- **Forge recall (§18.3):** live forge calls deferred to /work promotion (Phase-1 draft is docs-only);
  recall from the standing record: §12.1 runbook preconditions 1-2 (orchestration-shell.md:305-311 —
  precondition 2, reconnect, SHIPPED as #373, so wiring is unblocked); the #372 notes pinned
  "`FleetSubscription` has NO app-side caller yet (workspace grep)"; standing lessons applied:
  `PR-claude-pump-state-change-must-set-dirty-to-repaint` (#203 — a pump state-change on an idle
  frame never repaints without dirty), the mutants-skip detach trap (new fns near masked shims —
  re-run `--list` after), `PR-claude-trace-the-real-cargo-mutants-list` (run `--list -f` on the
  ACTUAL files at Validate), the #369 dead-vs-idle line (never infer liveness from age), the
  never-logged-bearer contract (#64/#375).
- **Discovery (the sweep's load-bearing evidence — file:line):**
  - **Subscription lifecycle SHIPPED, callerless:** `FleetSubscription::start/stop/exit_reason/Drop`
    adapter.rs:123-190 (spawn + stop flag + join); `run_subscription` :200-234 (until-stopped
    reconnect over pure backoff.rs:30-59); `run_once` :282-383; durable cursor
    `load_cursor_in`/`save_cursor_in` :447-463 (needs a config_dir from the app). Workspace grep
    confirms zero app-side callers.
  - **Live-vs-reconnecting NOT observable today:** `exit` is written ONCE when the thread ends;
    `exit_reason()` is `None` while running AND while reconnecting (adapter.rs:126-129/:171-179) →
    the indicator REQUIRES a new published state (spec D3, D-OPEN-STATE-POINTS).
  - **Stop bounds (REQ-004 evidence):** stop polled per ≤1s granule (`backoff_wait` adapter.rs:239-251,
    `POLL_QUANTUM_MS` backoff.rs:20, `wait_slice_ms` :57-59); SSE stream 1s read timeout
    adapter.rs:439; `fetch` 5s timeouts :102 — worst-case join ≈ one in-flight fetch.
  - **No-loss retention (REQ-005 crate half):** ONE carried `FleetSync` adapter.rs:210-213; the
    strict-delta PAGE_C scenario livewire.rs:40-43 proves delta-on-reconnect keeps known seats.
  - **Endpoint construction GAP:** `ForgeEndpoint{url,bearer}` constructs ONLY via
    `forge_endpoint_from(.mcp.json)` lib.rs:123-140 (loopback guard `is_loopback_authority`,
    redacted Debug :42-49); EVERY request embeds `Authorization: {bearer}` in the lib.rs framing
    (:187/:213) → the settings-sourced URL needs a new public constructor + a bearer source
    (D-OPEN-BEARER; candidates incl. the `.mcp.json` build precedent app.rs:1287-1291).
  - **Start-gate inputs SHIPPED:** `orchestration_for` exact-root orchestration.rs:25-30 (tested;
    re-exported UNUSED marley_app/src/lib.rs:98-99 — "so it isn't dead-code before its #368/Phase-E
    consumers land"); `ProjectOrchestration` serde-default fields orchestration.rs:11-21;
    `AppliedSettings.project_orchestrations` settings.rs:94-97/:207-225/:424 (+ tolerance tests
    :1055-1099); boot settings load app.rs:1256-1265 (ONCE — no runtime reload path exists; "boot IS
    settings-apply", spec D1); boot root discovery app.rs:1276-1281.
  - **Rail + demo sites:** `fleet_snapshot: Option<FleetSnapshot>` app.rs:389-391, init `None`
    :2171, test accessor :2280-2283; verb `"fleet-demo-feed"` :8145-8147 (installs
    `fleet_rail::demo_snapshot(now_epoch_ms())`); palette `CommandId(29)` "Fleet Demo Feed"
    :8355-8360 (+ `CommandId(28)` toggle :8344-8354); render `fleet_rail_body` :962-1060 (pure
    decisions in fleet_rail.rs:20-197, cov/MSI 100); right-dock header = `caption_header(dock_title
    (Right))` = "Fleet" app.rs:1130-1133 + layout.rs:65-69 (tested :373-376);
    `STALE_AFTER_MS = 30_000` fleet_rail.rs:17 (== `CAP_DELAY_MS` backoff.rs:15 — deliberate).
  - **Pump idiom:** the gpui pump loop app.rs:1295+ (skips when `project_count() == 0` :1304 — the
    launcher state; note for the drain placement); `pump_mcp_host` take/put :6569-6580 reads
    `self.fleet_snapshot` :6574 (the MCP expose side ALREADY forwards whatever snapshot the app
    holds — live data will flow to `fleet://snapshot` consumers for free).
  - **Quit precedent:** #375-D7 — McpHost teardown on Drop, gpui `on_app_quit` the grounded wiring
    candidate (375.spec D7). `FleetSubscription`'s Drop already stops+joins (adapter.rs:182-190).
  - **Proof harness precedent:** livewire.rs `Script`/`FixtureForge` :46-160 (real `TcpListener`
    scripted forge, thread-per-connection, bounded deadline polls); duplication-as-contract-pin
    rationale :26-30 (`#[cfg(test)]` fixtures not importable; drift = LOUD failure) → the marley_app
    integration test duplicates a MINIMAL fixture rather than exporting the harness (D-OPEN-HARNESS).
  - **Behavior maps checked:** docs/warp_architecture/subsystems/04-agent-ai-mcp.md — Warp's
    multi-agent surface is a cloud SSE client (`warp_multi_agent_client` → `…/ai/multi-agent`); no
    local fleet-rail analog; docs/zed_architecture/ none. Reference = N/A, Marley-specific.
- **EARS drafted (7):** REQ-001 configured root ⇒ exactly one subscription + live rail; REQ-002
  typed live/reconnecting/off indicator (never age-inferred); REQ-003 unconfigured ⇒ no subscription,
  #369 behavior verbatim (demo verb intact), "off"; REQ-004 quit ⇒ stop+join bounded, `Stopped`;
  REQ-005 unreachable ⇒ last snapshot retained + "reconnecting", delta reconnect loses no seats;
  REQ-006 non-loopback `brain_endpoint` ⇒ never constructs, behaves unconfigured; REQ-007 live
  supersedes demo, demo verb still works alone.
- **Decisions:** locked D1 (active-project-only, one subscription, boot trigger), D2 (existing
  stop-flag idiom, handle Drop on quit), D3 (typed closed state enum in a loop-updated shared cell +
  pure `Option<state>`→label fn; crate owns the enum, app owns the label), D4 (drain in the existing
  pump; change sets dirty — PR-pump-dirty), D5 (demo retained, live supersedes — observable locked),
  D6 (loopback guard + never-logged bearer carry to the settings-sourced endpoint), D7 (never clear
  the rail on disconnect).
- **Open design questions (Phase 2):** D-OPEN-HANDOFF (latest-wins cell vs mpsc — candidate cell;
  snapshots cumulative), D-OPEN-BEARER (`.mcp.json`-when-present vs empty vs both — never a new
  settings field), D-OPEN-RETARGET (restart on active-project switch vs boot-only; check the cursor
  store's per-endpoint story first), D-OPEN-STATE-POINTS (which loop points publish live vs
  reconnecting; pure transition fn candidate), D-OPEN-DEMO-MECH (verb-gated-while-live vs
  last-writer-wins), D-OPEN-HARNESS (minimal duplicated fixture vs exported harness — candidate
  duplicate), D-OPEN-INDICATOR (header suffix vs rail-top chip).
- **Forge ids:** ticket #376 `f2983e9d-fb30-4df3-a613-0b41d5609afd`; sprint #35 "M24 — Fleet
  Layer 2"; pipeline `e838eba5-3ccd-490d-82b8-214aca946b19`; AAR pending-promotion (mint at /work).

- **Promotion (/work 376-379,335, 2026-07-21):** queued→active (`git mv`), ticket claimed +
  in-progress/plan, AAR `08b3cd95` opened, `knowledge-context` Plan recall logged (13 surfacings).
  **Attack pass on the drafted spec — every load-bearing claim re-verified against the code:**
  `FleetSubscription::start(endpoint, config_dir, on_update)` + MUST-NOT-panic callback contract +
  Drop stop+join (adapter.rs:135-190 read); `exit_reason()` None-during-reconnect confirmed by its
  own doc ("transient causes … are NOT observable here"); `forge_endpoint_from` is the ONLY
  constructor and `is_loopback_authority` is PRIVATE — so the new raw-URL constructor must land in
  lib.rs beside the guard (it cannot be reused from outside); boot path lines exact (settings :1256,
  discovery :1276, forge-client-from-.mcp.json :1287, pump spawn :1295); no runtime settings-reload
  path (applied_from called once); `orchestration_for` exact-match + serde-default entry verified.
  No wrong claims found — spec PASSES as drafted. Phase 1 PASS.

## Phase 2 — Design
- **Approach:** adoption-first — the shipped `FleetSubscription` loop + `orchestration_for` +
  `fleet_rail` render are wired by ONE new pure app module (`fleet_live.rs`), one new pure
  crate fn (`endpoint_for_brain`), a typed `ConnectionState` published by the already-masked loop,
  and ~5 masked app.rs glue sites. §20 stance unchanged: N/A — Marley-specific (re-confirmed).
- **D-OPEN resolutions (all seven, evidence-backed):**
  - **HANDOFF → latest-wins `Arc<Mutex<Option<FleetSnapshot>>>`.** Snapshots are cumulative
    (dropped intermediates harmless); tolerates non-drained periods (the launcher state skips the
    pump at app.rs:1304 — a queue would grow, a cell can't); mirrors the take/put idiom.
  - **BEARER → new pure `endpoint_for_brain(brain_url, mcp_json: Option<&str>) -> Option<ForgeEndpoint>`
    in marley_forge_client lib.rs** — bearer = the `.mcp.json` forge bearer IFF its url == brain_url
    (v1's brain IS the forge sidecar), else empty; the `is_loopback_authority` guard applies to
    brain_url (non-loopback → `None`). Subsumes spec-surface (a): constructor + bearer decision in
    ONE pure fn, private fields stay private, no bearer accessor is ever exported. Rejected: a raw
    public `new(url, bearer)` (invites unguarded construction).
  - **RETARGET → boot-resolved-only v1** (a project switch does NOT retarget; documented). PLUS the
    promotion-found hazard fixed: the cursor store is a SINGLE shared `<dir>/fleet-cursor`
    (adapter.rs:449-463) — two different brains across runs would poison each other's replay
    window. Fix: pure `fleet_cursor_dir(config_dir, url) -> PathBuf` (sanitized per-endpoint
    subdir, e.g. `fleet-http-127-0-0-1-8790-mcp/`); the old shared file is simply orphaned (one
    harmless full replay; the #367 reducer is idempotent).
  - **STATE-POINTS → crate `ConnectionState { Reconnecting, Live }`** (Copy/Eq/Debug) in a
    `Arc<Mutex<_>>` cell on the handle; seeded `Reconnecting` at `start()`, `Live` published at
    listen-entry (subscribe 2xx + initial replay delivered), `Reconnecting` re-published on each
    transient cycle end. NO forced pure transition fn — it would be degenerate
    (`_ => Reconnecting`); the fixture integration proves both transitions (REQ-002). Publish
    lines ride already-masked fns; accessor `connection_state()` masked like `exit_reason()`.
  - **DEMO-MECH → verb-gated-while-live**, pure `demo_feed_permitted(subscription_active) -> bool`.
    Last-writer-wins REJECTED with a concrete failure: a demo dispatch after a live delivery would
    show fixture seats until the NEXT live event — silently wrong for minutes on a quiet fleet.
  - **HARNESS → minimal duplicated fixture** in NEW `marley_app/tests/fleet_livewire.rs`
    (duplication-as-contract, livewire.rs:26-30), driving the REAL `FleetSubscription` + the real
    handoff cell + the pure seams. The app.rs glue lines stay masked + §18.1-inspected; the rail
    RENDER of a live snapshot is the same `fleet_snapshot` field #369's render-executing drives
    already prove. marley_app already deps forge_client/fleet/tempfile — no Cargo.toml delta.
  - **INDICATOR → header-title suffix** via pure
    `fleet_header_title(Option<ConnectionState>) -> String`: `None` → `"Fleet"` (byte-identical to
    today — REQ-003's verbatim clause), `Live` → `"Fleet · live"`, `Reconnecting` →
    `"Fleet · reconnecting"`. A chip = bigger render delta, zero info gain; rejected.
- **File manifest:**
  1. `crates/marley_forge_client/src/lib.rs` — pure `endpoint_for_brain` beside
     `forge_endpoint_from` + its `#[cfg(test)]` units; re-export `ConnectionState` on line 18.
  2. `crates/marley_forge_client/src/adapter.rs` — `ConnectionState` enum; `FleetSubscription`
     gains the state cell + masked `connection_state()`; 3 publish sites (start-seed,
     listen-entry, transient-cycle-end) threaded through `run_subscription`/`run_once` params.
  3. `crates/marley_app/src/fleet_live.rs` — NEW pure module: `subscription_target(&[ProjectOrchestration], root) -> Option<String>`
     (empty-string endpoint → None), `fleet_header_title`, `demo_feed_permitted`,
     `fleet_cursor_dir` + full unit suites (cov/MSI 100).
  4. `crates/marley_app/src/lib.rs` — `mod fleet_live;` + flat `pub use` of its fns (the
     orchestration idiom, needed by the integration test).
  5. `crates/marley_app/src/app.rs` — masked glue: 3 fields (`fleet_subscription`,
     `fleet_incoming`, `fleet_conn_last`), the boot block (refactor the `.mcp.json` read to ONE
     string reused by forge_client + brain endpoint; target → `endpoint_for_brain` → `start` with
     `fleet_cursor_dir` + deposit-only closure), the pump drain (+ state-change dirty — PR-pump-dirty),
     the demo-verb gate, the header-title call. Quit teardown = RootView field drop (the #375
     McpHost precedent — `FleetSubscription::Drop` stops+joins, adapter.rs:182-190; no new wiring).
  6. `crates/marley_app/tests/fleet_livewire.rs` — NEW integration: minimal scripted fixture
     (initialize/read/subscribe/notify/close subset) + 4 scenarios (converge / state-transitions /
     stop-bounded / retain+delta-union).
- **Regression Test Plan (per REQ):**
  | REQ | Tests |
  |---|---|
  | REQ-001 | units: `subscription_target` exact-root / multi-entry / None / `brain_endpoint:None` / empty-string; integration: converge to fixture seats (bounded-deadline poll) |
  | REQ-002 | units: `fleet_header_title` all 3 arms (exhaustive match, no catch-all); integration: Live while healthy → fixture killed → Reconnecting |
  | REQ-003 | units: negative target arms; `fleet_header_title(None) == "Fleet"` byte-pin; regression: #369 fleet_rail + demo-verb suites untouched |
  | REQ-004 | integration: stop mid-stream returns promptly (bounded assert), `exit_reason == Some(Stopped)` |
  | REQ-005 | integration: kill fixture → deposited snapshot retained (cell non-cleared) + state Reconnecting; revive with strict-delta page → seat union (PAGE_C idiom at the app seam) |
  | REQ-006 | units: `endpoint_for_brain` loopback+matching-url→bearer-carried / loopback+mismatched-url→empty-bearer / non-loopback→None / garbage-url→None / no-mcp-json→empty-bearer |
  | REQ-007 | units: `demo_feed_permitted(true)=false, (false)=true`; §18.1 inspect on the verb-gate line; regression: demo verb works with no subscription |
- **Risks:** R1 the mutants-skip detach trap — adapter.rs gains code near `#[cfg_attr(test, mutants::skip)]`
  fns → re-run `--list -f` after; R2 the `.mcp.json` single-read refactor must keep the
  forge_client build byte-equal (same parse, same guard); R3 integration timing — bounded-deadline
  polls with generous deadlines (the livewire idiom), assert on state not sleep; R4 a launcher-state
  boot still subscribes for the boot-discovered root; the drain defers until a workspace opens
  (latest-wins tolerates) — documented, not a defect; R5 the cursor relocation orphans the old
  shared file → one full replay, idempotent.
- **Status: Phase 2 — Design PASS (goal-run confirm); ready for Phase 3 — Implement.**

## Phase 3 — Implement
- Built exactly to the manifest; `cargo check --workspace` clean (one pre-existing transitive
  `block v0.1.6` future-incompat note, not ours):
  1. **adapter.rs** — `ConnectionState { Reconnecting, Live }` (Copy/Eq/Debug) + the `state` cell on
     `FleetSubscription` (seeded `Reconnecting` in `start`), masked `connection_state()` accessor
     (poisoned lock → `Reconnecting`, the conservative arm), masked `set_state` helper, publish
     sites: `Live` after `open_listen_stream` Ok (replay done + stream established), `Reconnecting`
     re-published on each transient cycle end in `run_subscription`; `run_once`/`run_subscription`
     signatures thread `state: &Mutex<ConnectionState>`.
  2. **forge_client lib.rs** — pure `endpoint_for_brain(brain_url, mcp_json)` beside
     `forge_endpoint_from` (loopback wall via the SAME private `is_loopback_authority`; bearer
     carried IFF the `.mcp.json` forge url == brain_url, else empty); `ConnectionState` re-exported.
  3. **fleet_live.rs** (NEW pure) — `subscription_target` (exact-root + non-empty filter),
     `fleet_header_title` (exhaustive 3-arm match, `None`→"Fleet" byte-pin), `demo_feed_permitted`,
     `fleet_cursor_dir` (alnum-sanitized per-endpoint subdir).
  4. **marley_app lib.rs** — `mod fleet_live;` + flat `pub use` (the orchestration idiom; the
     integration lane needs public items).
  5. **app.rs** — import block; 3 fields (`fleet_subscription`/`fleet_incoming`/`fleet_conn_last`);
     boot: `fleet_config_dir` clone BEFORE the settings match consumes the Option, `.mcp.json` read
     refactored to ONE `mcp_json: Option<String>` reused by forge_client + `endpoint_for_brain`,
     subscription started via target→endpoint→cursor-dir with a deposit-only closure; pump:
     `pump_fleet_live()` (masked; drains the cell + tracks state, returns changed) called right
     after `let mut dirty = false;`; demo verb gated by `demo_feed_permitted`; `dock_panel` gained
     `title: String` (caption_header needs `'static`-able — first attempt with `&str` failed E0521,
     fixed by owning), Left passes `dock_title(...).to_string()`, Right passes
     `fleet_header_title(state)`.
- **Deviations from design:** none semantic; the only mechanical delta is `dock_panel(title: String)`
  instead of `&str` (gpui `SharedString` conversion requires ownership).
- **Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect
- **3 critics** (correctness/concurrency · security/secrets/provenance · integrity/reuse), each
  instructed to verify concretely. Ledger:
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | HIGH | `pump_fleet_live` inserted BETWEEN `pump_mcp_host`'s doc+`mutants::skip` and its `fn` — the skip-detach trap's **5th strike**; `--list` PROVED `replace pump_mcp_host with ()` (unkillable → MSI RED; worse, `--diff` would slip it now and detonate on a later full run, misattributed). Found by 2 critics independently. | REAL | relocated `pump_fleet_live` (own doc+skip) ABOVE the block, restored `pump_mcp_host`'s adjacency; `--list` re-run: BOTH pumps now 0 mutants |
  | F2 | HIGH | subscription keyed to the launch-cwd-discovered root, but the shell restore RE-SYNCS `project_root` to the restored ACTIVE project afterward — a Finder/Dock boot (cwd=/ or $HOME) would never subscribe though the restored workspace IS configured; a cwd-A/active-B boot feeds A's data under B; a launcher boot span a thread for an unopened workspace. | REAL | moved the start block BELOW the shell build, gated `shell.project_count() > 0`, keyed `shell.active_project().root` (launcher → no subscription — cleanly D1) |
  | F3 | MED | `Live` published after `open_listen_stream` Ok — but that only connect+WRITES; a server that accepts the GET and never answers → **unbounded** false "Fleet · live" (the 404 sub-case was a bounded flap). Write success ≠ established stream. | REAL | moved `set_state(Live)` into the `status_checked` pass, after `classify_if_failure` types the GET's own status line non-failure |
  | F4 | MED | render re-read the LIVE state cell (`connection_state()`, a per-frame mutex) while dirty tracked `fleet_conn_last` — two read points for one datum; header could run a tick ahead of body+dirty. | REAL | render reads `self.fleet_conn_last`; seeded `Some(Reconnecting)` at construction when a subscription exists (no bare-"Fleet" first tick) |
  | F5 | LOW | `fleet_cursor_dir` sanitizer lossy — `:9000` vs `/9000` (all 3 critics) share a subdir → the cross-brain cursor poisoning the fn exists to fix (a foreign HIGHER cursor silently SKIPS rows — worse than a replay). Also the "orphaned legacy file" comment was vacuous (no pre-#376 caller ever shipped a cursor). | REAL (contrived reach, cheap closure) | appended an inline FNV-1a 32-bit discriminator (deterministic across toolchains, no dep); comment rewritten honest |
  | F6 | LOW | `pump_mcp_host` fed BEFORE the fleet drain — `fleet://snapshot` consumers lagged one pump tick. | REAL | reordered: drain first, then feed |
  | F7 | LOW | `dock_panel(title: String)` forced a per-frame alloc for the Left dock's `&'static str` (`caption_header` already takes `impl Into<SharedString>`). | REAL | `title: impl Into<gpui::SharedString>`; Left passes the static, Right the computed String |
  | F8 | LOW | three silent-failure arms share one symptom (gated demo verb no-ops silently; non-loopback/malformed endpoint indistinguishable from unconfigured; a bearer string-mismatch → 401 → eternal "reconnecting" with the cause unobservable). | ACCEPTED — each is the ratified locked decision (D5/D6/D-OPEN-BEARER(a)); compounded diagnosability noted as a follow-up candidate (a "Fleet · misconfigured" state = new UI surface, not v1) |
- **Clean surfaces (verified, per critic):** loopback wall airtight (check-what-you-dial: guard and
  socket parse the SAME authority; every crafted-url divergence lands fail-safe — userinfo tricks,
  case, embedded-port forms all traced); bearer rule fail-safe (string-equal ⇒ identical dial
  target; mismatch ⇒ empty bearer); zero secret logging/Debug surface (`FleetSubscription` has no
  Debug and never retains the endpoint); cursor-dir traversal impossible (sanitizer total); tamper
  worst-case bounded (own-user loopback SSRF-shape, no credential movement); double-subscription
  impossible (one call site, one field, never reassigned); `.mcp.json` refactor byte-equal;
  deadlock impossible (no lock nesting anywhere); latest-wins handoff race-clean (one depositor,
  monotone within thread); full replay harmless (reducer max-join verified); REQ-003 byte-identity
  holds (`fleet_header_title(None)` == `dock_title(Right)` == "Fleet"); no duplicated helper found;
  export surface = the #371 idiom; provenance clean (§20 — all house idiom).
- **Post-fix verification:** `cargo check --workspace` clean; `--list` on app.rs → 0 pump mutants;
  fleet_live.rs = 14 viable mutants, `endpoint_for_brain` = 4 — the ACTUAL Phase-4 kill set.
- **Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate
- **Tests written:** `fleet_live.rs` units ×5 (`t376_req001` exact-root · `t376_req003` 4 negative
  arms · `t376_req002` all 3 title arms w/ the "Fleet" byte-pin · `t376_req007` demo gate ·
  `t376_req005` cursor-dir exact FNV pins `0583df2b`/`e1475d80` + the collision-pair distinctness);
  forge_client `brain_endpoint_loopback_wall_and_bearer_match` (REQ-006: bearer-carried-on-match
  asserted through the REQUEST FRAMING [`initialize_request` contains `Authorization: Bearer …`],
  empty-on-mismatch, None-json→empty, non-loopback/garbage/empty → None — kills the delete-! and
  ==→!= mutants); NEW `marley_app/tests/fleet_livewire.rs` — a minimal duplicated fixture forge
  driving the EXACT production chain (`subscription_target` → `endpoint_for_brain` →
  `fleet_cursor_dir` → `FleetSubscription::start` + the latest-wins cell): `req001` converge + Live
  + the cursor persisting under the per-endpoint home · `req002_005` forge-death → Reconnecting +
  snapshot RETAINED · `req004` stop bounded → `Stopped` · `req005` delta-union {a,b,c} across an
  in-fixture reconnect + return to Live.
- **Runs:** `cargo nextest run -p marley_forge_client -p marley` → **845 passed** (incl. all 12
  #372/#373 livewire regressions + the #369 rail/demo suites unchanged). Full gate `--diff`:
  first attempt RED on gate:1 (rustfmt — the trailing pump comment made the next block comment
  continuation-aligned; restructured above-the-call) + gate:2 (clippy `too_many_arguments 8/7` on
  `dock_panel` after the title param; fixed AT SOURCE by grouping the f32 quad into `PanelRect` —
  no suppression; `--list` confirms the struct adds zero mutants). **Second attempt: GATE GREEN
  [diff] 15/15 — coverage 100, MSI 100, receipt written.**
- **Driven live capture — explicitly N/A, not silently skipped:** the only DEFAULT-state visual
  delta is NONE (unconfigured roots render byte-identically — unit-pinned AND executed by the #369
  render-executing headless drives, which now run through the new `dock_panel`/`fleet_header_title`
  path in gate:3/15). The configured-state title requires a live orchestration entry + running
  brain; its machinery is proven by the 4 real-socket integration scenarios. Synthetic-input drive
  is contraindicated right now: chad is actively at the machine (the recorded M21 hazard — synthetic
  events land in HIS frontmost window; `PR-claude-selftest-focus-marley-before-driving-input`'s
  failure mode), and configuring the LIVE app would mean writing his real settings file. Mechanism +
  units + headless render carry it (the #204/#205/#214 precedent).
- **Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete
- **Docs:** CHANGELOG entry (first under Added); orchestration-shell §12.1 preconditions rewritten —
  BOTH SHIPPED (①=#376 with the full wiring summary, ②=#373), the runbook is now config+verify only.
- **Knowledge:** failures `BF-claude-skip-detach-pump-fleet-live-001` (the trap's 5th strike — new
  edge: INSERT-BETWEEN-ATTR-AND-FN, and `--diff`'s `--in-diff` would have slipped it to detonate
  later misattributed) + `BF-claude-boot-root-resolved-before-restore-001`; prevention rules
  `PR-claude-boot-decisions-key-the-restored-active-root-001` +
  `PR-claude-liveness-published-only-after-peer-status-verified-001`; AAR 08b3cd95 submitted
  (completed).
- **Ticket:** #376 closed (done) + ship comment; local doc → tickets/closed (status closed).
- **Archive:** the spec/notes pair → pipeline/completed/.
- Lessons: the 3-critic inspect caught BOTH commit-blockers my implement pass missed (the detach
  trap even though the spec's Floors section NAMED it, and the wrong-root keying even though D1
  SAID "active project") — naming a hazard is not the same as checking it; the fixture-forge
  pattern ported to the app seam in ~200 lines and immediately proved the retention/delta story.
