# Fleet dispatch composer — session.send as mailbox data + delivery-state chips (#378) — Notes

- **Forge ticket:** #378 (b40bc0c2-76dd-448f-867b-534cc983408b)
- **AAR:** pending-promotion (aar-open at /work pre-flight)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-378-fleet-dispatch-composer.md
- **Pipeline spec:** 378-fleet-dispatch-composer.spec.md

## Phase 1 — Plan
- **Request:** #378 (feature, M24, sprint #35 "M24 — Fleet Layer 2") — L2 gated-writes ②: compose a
  brief to ONE seat as mailbox DATA (never keystrokes — the two-planes rule,
  orchestration-shell §5:127-132), sent as a proposed `session.send` `tools/call` through
  `marley_forge_client` (contract-first fixtures; live Forge-side support = external follow-up),
  receipt-tracked through the shipped-but-unwired `marley_fleet::dispatch` machine
  (`Deposited → Claimed → Started`, advanced by `seat_events` echoes), rendered as a chip on the
  seat's rail card. v1 composer = the inline-draft idiom (#177 `renaming_tab` / #204
  `naming_workflow`) — no new modal machinery. Kills the evidence-night class fleet-control-plane
  §2:51 names: unsent briefs / dispatch-by-keystroke with no receipt. Sibling: #377 owns the
  question-ANSWER path (separate).
- **Classification / tier:** feature; full pipeline. Wire + state-machine + render-shim ticket: the
  #368 contract-first posture for the wire, the #369/#307 pattern for the rail surface (pure seams
  cov/MSI 100; the excluded app.rs shim proven by headless drives + capture).
- **Forge recall (§18.3):** drafted offline (docs-only Phase-1; no MCP calls) — re-run
  bulletins/knowledge-context at /work pre-flight. Standing rules already baked into the spec:
  `PR-claude-pump-state-change-must-set-dirty-to-repaint` (the send result + echo folds land off
  the render loop → set dirty); the mutants::skip-detach trap (new fns near masked shims → re-run
  `--list`); `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators` /
  guard-mutants-depend-on-syntactic-form (run `--list` on the ACTUAL files);
  `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism` (Validate fallback);
  the fail-closed write-ack posture (#74: a failed write must never read as success);
  `PR-claude-per-pane-render-affordance-must-gate-on-is-focused` (if D-OPEN-ENTRY lands on the
  card, per-seat gating).
- **Discovery (sweep findings; file:line evidence):**
  - **dispatch.rs — the premise HOLDS, and it is genuinely unwired.** `DeliveryState { Deposited <
    Claimed < Started }` (declaration-order `Ord`, dispatch.rs:9-18); `observe(observed) →
    DeliveryAdvance { state, advanced }` — monotone join: strictly-further advances, the
    `Deposited → Started` skip is legal, at-or-behind is an idempotent no-op (:29-46); lowercase
    serde; the 3×3 table + round-trip already tested (t367_req012/013). Workspace grep: ZERO
    references outside `marley_fleet` (the marley_mcp `dispatch` module hit is its own message
    dispatcher, unrelated).
  - **verbs.rs — the send verb is already typed.** `SendRequest { id, text }` (:9-14 — field `id`,
    not the intake sketch's `session_id`; spec D1 adapts to the code) + `Receipt<T> =
    Accepted{value} | Refused{reason}` (tag `result`, snake_case, :62-75), round-trip-tested.
    Module doc pre-states: receipted delivery, Enter-as-separate-write is the DELIVERY side
    (bridge, pre-solved), a refused send (dialog up) is a first-class outcome (:6-8) → REQ-005.
  - **reducer.rs / session.rs — NO dispatch arm.** `SessionEvent` = closed v1 six-kind set
    (reducer.rs:11-14); `Session` has no delivery field (session.rs:54-76). An unknown row kind
    degrades via `project_row` to Upsert(known-state)/Heartbeat (fleet.rs:479-483) — an echo row
    would be silently absorbed today → the echo channel must be added at a seam
    (D-OPEN-ECHO-CARRIER; leaning: adapter-level pure projection, vocabulary boundary
    fleet.rs:6-8; alternative: extend the closed set = wire-schema ripple into reducer/snapshot/
    the #370 tool schema).
  - **Echo vocabulary:** fleet-control-plane §5's v1 event enum ALREADY names `dispatch-claimed`
    (:160); `dispatch-started` is our symmetric proposal — both pinned by fixtures, ratified with
    L0 at the follow-up.
  - **marley_forge_client — the wire idiom is fully shipped.** `tool_call_request` builds a generic
    `tools/call` (lib.rs:170-197, pub(crate)); `parse_write_ack` is fail-closed incl. `isError`
    (:326-346); the closed-write-set doctrine (:5-8) — send = the THIRD explicit masked method
    mirroring `claim_ticket`/`comment_ticket` (adapter.rs:56-73; bounded `fetch` :95-104).
    Builder/parser fixture-test style to mirror: `write_request_builders_map_args` (lib.rs:563-573,
    distinct values so a key swap fails) + fleet.rs:992-1015.
  - **livewire.rs — the Script harness.** `Script`/`FixtureForge` record + script responses
    (:65-172); `serve_post`'s catch-all answers any non-initialize POST minimal-OK (:267-281) → a
    scripted `tools/call` arm (record name+arguments, respond Accepted/Refused/isError) is cheap IF
    P2 elects a livewire send scenario; unit fixtures are the REQ-006 floor either way.
  - **app.rs — all UI ingredients pre-built.** `fleet_snapshot: Option<FleetSnapshot>` (:391), fed
    ONLY by the `fleet-demo-feed` verb today (:8146 — no `FleetSubscription` reference in app.rs;
    live-pump wiring is NOT this ticket, Scope Out); rail shim `fleet_rail_body` (:952-1070; chip
    FILL tones :1002-1008; render call :15755); inline-draft idiom twice over — `renaming_tab`
    (:421/:14472) and `naming_workflow` (:163; begin :2961-2980 incl. the blank-guard
    `status_flash`; key branch :14287-14343 ending `stop_propagation`+`notify`; overlay
    :17193-17219; the in-flight-draft gate :8728-8731); background-write pattern
    `claim_forge_ticket` (:6852-6863, clone client → mpsc → pending receiver → thread); headless
    lane `dispatch_for_test` + palette verbs (#369's proof path).
  - **Design-doc anchors:** orchestration-shell §4:97 (the dispatch consume row), §5:115
    (`session.send(id, text)` receipted, refuses when a dialog is up), §5:127-132 (two planes),
    §3:85-88 (capability pre-checks are POLICY — Marley surfaces the Refused receipt);
    fleet-control-plane §2:51 (unsent briefs), :60-61 (the mailbox never failed), §7:190-193
    (item ⑤ — v1 is its thin end: no template/merge-scope/auto-refuse form).
- **EARS (7):** REQ-001 seat-targeted inline draft (the idiom); REQ-002 confirm → exactly ONE
  `session.send` carrying `{id, text}` as DATA, blank = no-op, draft text never touches a PTY;
  REQ-003 Accepted receipt → chip at `deposited`; REQ-004 echoes advance
  deposited→claimed→started via the SHIPPED `observe` (dup/behind no-op, skip legal); REQ-005 any
  failure arm (transport/RPC/isError/Refused) → reason surfaced + draft PRESERVED, never renders
  deposited; REQ-006 wire shape fixture-pinned (well-formed `tools/call` + receipt arms — the
  fixtures ARE the proposed contract, #368 idiom); REQ-007 chips are a pure function of the
  machines only; no tracked dispatch → no chip (#369 rail unchanged).
- **Decisions:** D1 wire = shipped `SendRequest{id,text}` + fail-closed `Receipt<()>`; D2 data
  plane through the closed write set only (never PTY); D3 chip truth machine-only (no forgery —
  transient `sending` + observe-driven tracker seeded by receipt); D4 pure seams cov/MSI 100,
  masked glue; D5 the inline-draft idiom, no new modal machinery; D7 one tracked dispatch per seat
  (latest; re-send re-seeds; stale-echo window acknowledged, per-dispatch ids deferred).
  **D-OPEN:** TOOL-NAME (`session.send` vs Forge kebab-case — one constant, P2 locks, L0
  ratifies); ECHO-CARRIER (adapter projection [leaning] vs extending the closed `SessionEvent`
  set); ENTRY (rail-card affordance vs palette verb — behavior locked: one seat).
- **Open questions for P2:** the livewire `tools/call` arm (elect or defer); compose-module
  placement (`fleet_rail.rs` vs a sibling); the pending-receiver shape (reuse `forge_pending`'s
  pattern vs a dedicated channel carrying `(seat_id, Result)`); how the demo feed scripts a
  dispatch sequence for the capture. RESOLVED during draft: #376 (fleet-rail-livewire — its queued
  spec landed alongside this one) owns the live-pump→app wiring; Scope Out names it, soft order
  after #376 preferred, not required.

- **Promotion (/work 376-379,335, 2026-07-21):** queued→active, claimed + in-progress/plan, AAR
  `57a080c9`. **Attack pass + drift:** the draft PRE-DATES #376 and #377 shipping IN THIS RUN —
  (a) the livewire `tools/call` arm + name/arguments recording it plans as an extension ALREADY
  EXISTS (#377 landed `Script.answer_refuse`/`answer_http_fail` + the recording; the send scenarios
  reuse those knobs); (b) the fleet feed IS live-wired (#376) and the demo verb is gated while a
  subscription exists (headless boots unaffected); (c) #377's inspect lessons (per-pick NONCE, the
  clamp, the overlay precedent in the card) apply verbatim and are adopted PREEMPTIVELY. dispatch.rs
  re-verified: `Deposited < Claimed < Started` declaration-order Ord + monotone `observe` exactly as
  specced. **One design problem the draft missed:** the app receives SNAPSHOTS, not pages — the
  `dispatch_echoes(page)` projection has no app-side input; the pages live only inside `run_once`.
  Resolved in P2 below (a crate-side echo queue on the handle). Phase 1 PASS.

## Phase 2 — Design
- **D-OPEN resolutions:**
  - **TOOL-NAME → `session.send`** (dotted — the #377 in-run precedent for the session-verb family;
    §5:115 names it verbatim). Same lane as #377: `tool_call_request` (the shipped, live-proven
    write framing).
  - **ECHO-CARRIER → (a) adapter-side pure projection + a crate-side ECHO QUEUE on the handle.**
    Pure `dispatch_echoes(&FleetPage) -> Vec<(String, DeliveryState)>` in forge_client's fleet.rs
    maps the proposed `dispatch-claimed`/`dispatch-started` row kinds (UCSOS vocabulary stays in the
    adapter; `SessionEvent` stays closed; the generic degrade path still advances liveness). BUT the
    app never sees pages (`on_update` carries snapshots; pages are consumed inside `run_once`) — so
    the masked `refresh` also appends projected echoes into a `Arc<Mutex<Vec<_>>>` cell on the
    handle, drained by the app pump via `FleetSubscription::drain_dispatch_echoes()` (the
    `connection_state()` polling idiom, no callback-signature change, #376's glue untouched).
  - **ENTRY → a per-card affordance.** A small muted "send" chip in each seat card's header row
    (`on_mouse_down` + `cx.listener` — the #377 chip idiom) opening the inline draft targeted at
    THAT seat. A palette verb can't name a seat; per-card gating is inherent (each chip captures its
    own seat id). v1: every seat card offers it (an invalid target = the server's `Refused`).
- **The #377 lessons adopted preemptively:** each send carries a NONCE (`fleet_dispatch_seq`), the
  result drain applies iff `record.nonce` matches (a superseded send's late result is dropped); the
  failure reason is CLAMPED via the same pure chain (shared `clamp_reason` helper, two callers).
- **Contract deviation from the draft's D1 tail (recorded):** refusals ride the `isError` lane
  (`parse_write_ack` surfaces the reason — the #377 wire posture, one lane, zero new parse code);
  `Receipt<T>` stays the ENVELOPE vocabulary, not a second wire parse. The fixture pins refusal-as-
  isError as the proposed contract.
- **Pure seams (fleet_rail.rs, cov/MSI 100):** `DispatchPhase { Sending, Failed(String),
  Delivery(DeliveryState) }`; `DispatchRecord { phase, nonce }`; `dispatch_send_applied(Result<(),
  String>) -> DispatchPhase` (Ok → `Delivery(Deposited)` — the receipt seeds, D3; Err → clamped
  `Failed`); `dispatch_observe(phase, observed) -> DispatchPhase` (advances ONLY a `Delivery` phase
  via the shipped monotone `observe`; `Sending`/`Failed` unchanged — an echo racing ahead of the
  receipt is dropped and the NEXT echo heals it, the skip being legal; honest residue noted);
  `dispatch_chip(phase) -> (String, ChipTone)` (labels: "sending…"/"send failed: {r}"/"deposited"/
  "claimed"/"started"; tones: Pending/Failure/Pending/Attention/Active); `dispatch_confirmable
  (&str) -> bool` (blank guard); shared private `clamp_reason` (killed via both public callers).
  In forge_client fleet.rs: pure `dispatch_echoes`.
- **File manifest:**
  1. `crates/marley_forge_client/src/fleet.rs` — pure `dispatch_echoes(&FleetPage)`.
  2. `crates/marley_forge_client/src/lib.rs` — `pub(crate) send_request_verb(endpoint, id, text)`
     serializing the SHIPPED `marley_fleet::SendRequest` via serde (D1: one authoritative type);
     re-export `dispatch_echoes`.
  3. `crates/marley_forge_client/src/adapter.rs` — masked `ForgeClient::send_session(id, text)`
     (the answer_question twin); `FleetSubscription` gains the echo cell + masked
     `drain_dispatch_echoes()`; `refresh` appends projections (threaded like the state cell).
  4. `crates/marley_app/src/fleet_rail.rs` — the pure seams above.
  5. `crates/marley_app/src/app.rs` — masked: fields (`fleet_dispatch_draft: Option<(String,
     String)>`, `fleet_dispatches: HashMap<String, DispatchRecord>`, `fleet_dispatch_tx/rx`,
     `fleet_dispatch_seq`); the card "send" affordance; the inline-draft overlay + key branch
     (the #177/#204 idiom, third instantiation); `confirm_fleet_dispatch` (blank-guard → nonce →
     Sending → thread send → tagged result); the pump drain (results + echo queue + seat-vanish
     scrub — the #377 shape); the dispatch chip line in the seat card.
  6. `crates/marley_forge_client/tests/livewire.rs` — send scenarios reusing the #377 knobs
     (recorded contract JSON `{id, text}`; isError refusal; HTTP failure) + an echo-page scenario
     (a `dispatch-claimed` row page → `drain_dispatch_echoes` yields `(seat, Claimed)`).
  7. `crates/marley_app/src/headless_drive.rs` — the composer machine drive (open→edit→Esc;
     blank no-op; confirm→no-client Failed + draft PRESERVED; nonce-gated result apply; echo fold
     advances Deposited→Claimed→Started incl. duplicate no-op; seat-vanish scrub) + accessors.
- **Regression Test Plan (per REQ):** REQ-001 draft-state units + drive (open binds seat, Esc
  closes, edits) · REQ-002 builder fixture (locked name `session.send`, `{id, text}` distinct
  values) + confirm-gate unit (Sending → no second dispatch) + blank unit + inspect PTY sweep ·
  REQ-003 `dispatch_send_applied(Ok)` → `Delivery(Deposited)` + chip fn units · REQ-004 echo
  projection units (claimed/started/unknown-ignored) + `dispatch_observe` fold (in-order, dup,
  behind, skip) + the livewire echo scenario · REQ-005 per-arm Failed units (draft preserved is an
  app-state assert in the drive; reason verbatim-then-clamped) + livewire isError/HTTP scenarios ·
  REQ-006 exact envelope JSON + the frozen-contract literal + livewire recorded-args assert ·
  REQ-007 structural (tracker mutated only by seed/observe — inspect checkpoint) + absent-entry →
  no chip unit + #369 suite green.
- **Risks:** R1 skip-detach (adapter + app edits near masked fns — `--list` after); R2 the echo
  cell adds a second lock on the pump path (no nesting: drain takes echoes THEN applies — verify at
  inspect); R3 the draft key-branch must not collide with the existing renaming branches (mirror
  their guard order); R4 the demo-gate interplay unchanged (headless has no subscription).
- **Status: Phase 2 — Design PASS (goal-run confirm); ready for Phase 3 — Implement.**

## Phase 3 — Implement
- Built to the manifest; `cargo check --workspace` clean:
  1. **fleet.rs** — pure `dispatch_echoes(&FleetPage)` (claimed/started only; Deposited is
     receipt-seeded, D3); re-exported.
  2. **lib.rs** — `pub(crate) send_request_verb` serializing the SHIPPED `SendRequest {id, text}`
     (one authoritative type, D1) via `tool_call_request("session.send", …)`.
  3. **adapter.rs** — masked `ForgeClient::send_session` (the answer_question twin);
     `FleetSubscription` gained the echo queue cell + masked `drain_dispatch_echoes()`;
     `run_subscription`/`run_once`/`refresh` thread the cell; `refresh` appends projections after
     `apply_page` (pages never reach the app — the queue is the echoes' only route out).
  4. **fleet_rail.rs** — `DispatchPhase { Sending, Failed, Delivery(DeliveryState) }`,
     `DispatchRecord { phase, nonce, draft }`, `dispatch_confirmable`, `dispatch_send_applied`
     (Ok → `Delivery(Deposited)`), `dispatch_observe` (only `Delivery` advances, monotone),
     `dispatch_chip` (label+`ChipTone`), `dispatch_retry_draft` (Failed+draft → prefill); the #377
     clamp extracted to a shared private `clamp_reason` (two public callers kill its mutants).
  5. **app.rs** — 5 fields (draft, tracker map, `DispatchOutcome` channel pair, seq); the
     `DispatchOutcome` type alias (the Err arm carries (reason, draft) — REQ-005's preservation
     without a clippy-suppression); `text_input_blocked` gains the draft; the key branch (the #204
     mirror, `stop_propagation`); `begin_fleet_dispatch(seat, prefill)` +
     `confirm_fleet_dispatch` (blank close-discard → nonce → Sending → thread send); the pump
     drain (nonce-gated, Err restores the brief) + echo fold (`dispatch_observe`) + seat-vanish
     scrub; the card's "send" header affordance + the dispatch chip line (Failed+preserved-brief =
     clickable prefilled retry); the centered draft overlay (the #204 card mirror).
- **Deviations:** (a) REQ-005's draft preservation implemented via the record's `draft` field + the
  clickable-retry chip (the design's channel shape grew `(reason, draft)` on Err — the confirm
  consumes the draft, so failure must carry it BACK); (b) blank-confirm = close-discard (the exact
  #204 sibling behavior; REQ-002 only demands "dispatches nothing"); (c) a `DispatchOutcome` type
  alias instead of `#[allow(clippy::type_complexity)]` (gate:12 bans suppressions).
- **Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect
- **2 critics** (correctness/concurrency · security/integrity/provenance). Ledger:
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | MED (both critics) | The echo queue was an UNBOUNDED `Vec` whose only drain (the pump) parks in the launcher state (`project_count()==0` early-return) while the boot-started subscription keeps refreshing — hours of fleet dispatches (including OTHER actors') accumulate; breaks the #376 bounded-cell discipline on the same thread. | REAL | the queue became a PER-SEAT MAX-JOIN `HashMap<String, DeliveryState>` (bounded by fleet size; lossless — the sole consumer folds the same monotone max, so coalescing [Claimed, Started]→Started is behavior-identical) |
  | F2 | MED | The "next echo heals it" justification had an UNHEALABLE case: a fast seat claims+starts while the send's HTTP response is in flight → both echoes drop against `Sending` → receipt lands `Deposited` → `dispatch-started` is TERMINAL, no later echo ever comes → the chip reads "deposited" forever while the seat runs the brief. | REAL | `DispatchRecord.pending: Option<DeliveryState>` — echoes observed while `Sending` are max-joined into the buffer (nothing RENDERED pre-receipt, so D3 holds); `dispatch_result_applied` seeds `Deposited.observe(pending)` on Ok; the old phase-only fns replaced by record-level `dispatch_result_applied`/`dispatch_observe_record` |
  | F3 | MED | The write-surface DOCTRINE docs undercounted the closed set ("fixed trio" / a three-item list) — a provenance defect: reviewers audit the write surface against that sentence. | REAL | lib.rs header → QUARTET with all four writes named; the ForgeClient doc lists `send_session` |
  | F4 | LOW | `begin_fleet_dispatch` clobbered an open draft's typed text (any send/retry chip stays clickable beside the overlay); and naming_workflow + the dispatch draft could both be Some (the ladder feeds naming while the dispatch card paints on top — the screen lies). | REAL | one guard: begin refuses while `naming_workflow` is open OR a non-empty draft exists (typed work is never clobbered) |
  | F5 | LOW | The overlay rendered the server-controlled seat id unclamped (inconsistent with the #377-F2 stance). | REAL | `clamp_reason` generalized to pub `clamp_card_text`; the overlay routes the id through it |
  | F6 | LOW | The livewire "behavior-proven" doc claims are ahead of the tree (no send scenarios yet). | REAL (forward-dated) | Phase 4 lands them (binding) |
  | F7 | LOW | A vanished seat would silently discard a preserved FAILED brief. | ACCEPTED — verified UNREACHABLE today (the reducer never removes seats; `Ended` keeps them; the scrub is a forward-guard) |
  | F8 | note | The nonce-less echo fold means ANOTHER ACTOR's dispatch to the same seat advances our chip — broader than D7's re-send framing but covered by its per-dispatch-ids deferral. | ACCEPTED | recorded here + the contract note |
- **Clean surfaces (verified):** two-planes rule airtight (zero PTY/spawn/send-keys reach; the key
  branch stops propagation on ALL keys; `text_input_blocked` gates the IME plane); bearer hygiene
  (no error path formats the request text); wire integrity (serde end-to-end, body-only, byte-length
  correct for multibyte); no state forgery (the tracker's full mutator inventory verified:
  confirm-seed / nonce-gated result / get_mut-gated observe / scrub); hostile echoes can neither
  create records nor regress states; locks all leaf-level on both threads (no nesting/inversion);
  the #377 clamp extraction behavior-identical; zero suppressions; §20 provenance clean; 88
  relevant tests green during inspect.
- Post-fix: `cargo check --workspace` clean.
- **Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate
- **Tests written:** fleet_rail units ×5 (`t378_req002` blank guard · `t378_req003_005` receipt
  fold incl. the F2 buffered-Started consumption + clamp-on-Err + pending-cleared ·
  `t378_req004` the full observe matrix — advance/dup/behind/skip/Sending-buffers-max-join/
  Failed-inert · `t378_req007` all 5 chip arms exact · `t378_req005` the retry gate 4 arms);
  fleet.rs `t378_dispatch_echoes_projects_the_two_kinds` (claimed/started map, session-start AND a
  forged dispatch-deposited ignored, empty page); lib.rs `send_request_envelope_exact` (name +
  `{id, text}` distinct values + bearer + the CRLF-injection NEGATIVE — a sneaky draft stays
  JSON-escaped in the body); livewire ×3 (`t378_send_call_records_the_frozen_contract` — the L0
  handoff literal `{id, text}` · `t378_send_iserror_refusal_surfaces_reason` ·
  `t378_dispatch_echoes_drain_coalesced_from_pages` — a claimed+started page drains as exactly ONE
  `(a, Started)`, proving the F1 max-join map end-to-end over the real socket); the headless drive
  `fleet_dispatch_composer_machine_headless` (begin binds · non-empty-draft refuses a clobbering
  begin (F4) · blank confirm dispatches nothing · no-client confirm → visible Failed + brief
  PRESERVED + retry-prefill round-trip · snapshot value-equal throughout — no forgery); 6
  `#[cfg(test)]` accessors landed with their tests.
- **Runs:** 889 passed (878→889; all #369/#372/#373/#376/#377 suites green). **Gate `--diff`:
  GREEN 15/15 on the FIRST attempt** — coverage 100, MSI 100, receipt written.
- **Driven live capture — N/A with cause:** the chip lifecycle needs a live brain emitting dispatch
  echoes (external, un-landed — the echo kinds are THIS ticket's proposed contract); the composer +
  machine run headlessly through the real paths; the fixture proves wire + drain; chad is at the
  machine (the standing synthetic-input hazard). Mechanism + units + fixture + drive carry it.
- **Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete
- **Docs:** CHANGELOG (first under Added); orchestration-shell §12 Layer-2 `session.send`+composer
  marked SHIPPED with the frozen contract, the echo kinds, the buffer rule, and the
  per-dispatch-ids deferral.
- **Knowledge:** failure `BF-claude-terminal-echo-dropped-during-transient-state-001` + prevention
  rule `PR-claude-buffer-dont-drop-events-during-transient-states-001`; AAR 57a080c9 submitted.
- **Ticket:** #378 closed (done) + ship comment; local doc → closed. **Archive:** pair → completed/.
- Lessons: the critics' convergence pattern held a third time (both found the echo queue); the
  "heals later" justification failed exactly at the SEQUENCE-TERMINAL event — check the last event
  of any lifecycle when arguing drops are safe; the in-run #377 lessons (nonce, clamp) applied
  preemptively cost nothing and the critics confirmed them correct rather than finding them missing.
