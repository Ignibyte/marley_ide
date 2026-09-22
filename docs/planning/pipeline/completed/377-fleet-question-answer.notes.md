# Fleet structured interrupts — answer the Waiting seat's question (#377) — Notes

- **Forge ticket:** #377 (d81f8c67-7b4f-4c9d-8f83-b57c73796727)
- **AAR:** pending-promotion
- **Local ticket doc:** docs/planning/tickets/open/TICKET-377-fleet-question-answer.md
- **Pipeline spec:** 377-fleet-question-answer.spec.md

## Phase 1 — Plan
- **Request:** #377 (feature, M24, sprint #35 "M24 — Fleet Layer 2") — make the #369 read-only
  question-card answerable: L2 gated-writes ①, the in-shell slice of the mission-control
  AskUserQuestion loop. Picking an option dispatches exactly ONE receipted answer via
  `marley_forge_client` (contract-first: a proposed MCP `tools/call` wire + fixture-server tests,
  the #368/#372 pattern; live ucsosv2 support = external follow-up); a local pending/answered chip
  holds until the seat's own event stream flips its state (truth stays with the reducer — no
  optimistic state forgery); a failed send surfaces an error and re-arms. The answer is DATA on the
  control plane, never keystrokes.
- **Classification / tier:** feature; full pipeline. Splits across the coverage-excluded app.rs shim
  (the #307/#369 pattern: every decision a pure fn) and the `marley_forge_client` pure/masked lane
  (the #74/#368 pattern: builders/parsers pure at cov/MSI 100, socket glue masked, fixture-server
  integration in livewire.rs).
- **Forge recall (§18.3)** — standing rules baked into the spec:
  - `PR-claude-pump-state-change-must-set-dirty-to-repaint` — the thread-send's result re-entering
    the view MUST set dirty (D5/D-OPEN-3); same for the chip clear on a snapshot flip (REQ-007).
  - `PR-claude-per-pane-render-affordance-must-gate-on-is-focused` (per-item-loop form, via #369
    D7) — the pick affordance/chip gates on THAT seat's data; a pick on seat A must never mark B.
  - The #74 fail-closed write lesson — `parse_write_ack`: JSON-RPC error, `isError: true`, and
    neither-result-nor-error are ALL failures; a failed answer must never read as answered (D6).
  - `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators` + the syntactic-form
    lesson — run `cargo mutants --list -f` on the ACTUAL touched files at P4.
  - The `mutants::skip` detach trap — re-run `--list` after inserting fns near the app.rs shims.
  - `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism` — the P4 capture
    fallback if the machine is locked.
- **Discovery (file:line evidence; the sweep that settled the design):**
  - **`marley_fleet/src/verbs.rs` — NO answer verb exists.** Ships `SendRequest{id,text}`,
    `ReadRequest{id,range}`, `OpenRequest{profile}`, `SurfaceRequest{id}`, and the generic
    `Receipt<T>{Accepted{value}|Refused{reason}}` (refusal first-class — the dialog-up rule,
    verbs.rs:60-62). `AnswerRequest` placement = D-OPEN-2.
  - **`marley_fleet/src/session.rs`** — `Question{prompt, options, context_refs}` (field is
    `context_refs`, not `context`); `question ⇒ Waiting` reducer-enforced; empty `options` = the
    free-text case (scoped out to #378).
  - **`marley_fleet/src/reducer.rs`** — the reducer is the SOLE producer of snapshot state
    (reducer.rs:107-115); `QuestionCleared` clears the question leaving state Waiting;
    `StateChange`/`Upsert` away from Waiting clear it; closed v1 `SessionEvent` set (no
    answer-related kind — none needed; the seat flips via the existing stream).
  - **`marley_app/src/fleet_rail.rs`** — `question_card` (:85) is the pure Waiting∧question gate
    (tested t369_req003) — REQ-006 pins it and REQ-007's clear decision reuses it;
    `demo_snapshot` (:119) already carries a Waiting seat ("dev-4/review", 3 options) for the
    headless drive + capture.
  - **`marley_app/src/app.rs`** — the inert card render + "answered in Layer 2" hint to retire
    (:1057-1098, hint :1095, #369 D6); `forge_client: Option<ForgeClient>` (:380, built :1291);
    `claim_forge_ticket` (:6853) = the masked thread-send idiom whose `let _ =` result-discard
    (:6860) REQ-004 must NOT copy; `status_flash` (:541) = an error-surface candidate (D-OPEN-5);
    50 `on_mouse_down` sites = the click idiom (D4). **`FleetSubscription` has ZERO references in
    `crates/marley_app/`** — `fleet_snapshot` is demo-fed only (:2171, :8146): the live ② feed is
    unwired, so the stream-flip half of the loop is proven via snapshot replacement + pure units.
  - **`marley_forge_client/src/lib.rs` — the write lane is ALREADY WHOLE:** `tool_call_request`
    (:170, generic `tools/call` framer, bearer embedded, `pub(crate)` per the closed-set-of-writes
    doctrine :5-8); `parse_write_ack` (:326, fail-closed both failure planes); `claim_request`/
    `comment_request` (:349/:363) = the closed-set precedent; the distinct-values args-map test
    idiom (:563). `mcp_post` (:204) is the fleet lane (adds `MCP-Protocol-Version` +
    `Mcp-Session-Id`) — which lane carries the answer = D-OPEN-1.
  - **`marley_forge_client/tests/livewire.rs`** — the `Script`/`FixtureForge` harness: a
    `tools/call` POST currently falls into `serve_post`'s `_ => minimal_ok_response()` catch-all
    (:280) and `parse_rpc` (:385) records only method + `params.uri` → the small known extension:
    record `params.name` + `arguments` + session header, script accepted/`isError`/HTTP-error
    outcomes. The frozen-literal idiom (PAGE_A, :27-33) is the model for freezing the answer
    contract.
  - **`marley_mcp`** — dotted `family.verb` names shipped (`fleet.snapshot`,
    `session.surface_to_human`; grant class `session.write`) vs the forge sidecar's dash names
    (`ticket-claim`) → the D-OPEN-1 naming fork.
  - (A mid-sweep observation that `tickets/open/` had been emptied by a concurrent process was
    FALSE — a transient misread of the M23.5 `TICKET-372…375` archives in `closed/`. Verified
    post-draft: all five sprint-#35 TICKET docs sit in `open/`, nothing was moved.)
- **Prior-art (§20):** Reference = N/A — Marley-specific (orchestration-shell.md §5 "answered as a
  receipted typed reply / no digits pressed at screenshots" + §5's both-directions table row +
  §12 Layer 2 "question-forms answering"; fleet-control-plane.md "Structured interrupts — questions
  as forms" + the press-digit-4 incident; the mission-control intake's flagship loop #1). Behavior
  maps: docs/warp_architecture/subsystems/04-agent-ai-mcp.md attests `AskUserQuestion` as a typed
  tool call in Warp's cloud conversation loop with typed action-results back (:110, :145) — the
  question-as-typed-data principle confirmed as research, but it is a conversation-stream tool, not
  a fleet-seat rail; no UI detail mapped, nothing to observe/adopt; docs/zed_architecture/ has no
  agent-question surface. No copyleft source consulted. Published: the MCP spec 2025-06-18
  "Tools → Calling tools" (`tools/call` `{name, arguments}` + result `isError`) — already adopted
  in-tree (lib.rs:170/:326); the answer rides that adoption.
- **EARS:** REQ-001 pick ⇒ exactly one answer `tools/call` naming seat + picked option string, no
  PTY writes · REQ-002 in-flight ⇒ further picks no-ops · REQ-003 success receipt ⇒ local
  pending→answered chip, reducer snapshot untouched · REQ-004 failed/refused send ⇒ surface +
  re-arm · REQ-005 wire = well-formed `tools/call` per the proposed contract (fixture-verified) ·
  REQ-006 non-Waiting / question-less Waiting ⇒ no affordance (pin `question_card`) · REQ-007
  stream flips the seat ⇒ chip + local record clear.
- **Decisions:** D1 contract-first minimal wire `{session_id, choice}` (choice = the option string
  verbatim; name/lane D-OPEN-1) · D2 pending gate = a pure per-seat answer state machine
  (Armed→Pending→Answered/Failed; Failed re-arms) · D3 no forgery — send path never touches the
  snapshot; chip is app-local, cleared by a pure decision over reducer output · D4 v1 = mouse click
  (no rail focus machinery exists; keyboard nav deferred) · D5 builders/parsers pure cov/MSI 100,
  send glue masked BUT result returns + dirty · D6 receipts fail-closed on the shipped
  `parse_write_ack` lane · D7 fixture `Script` extension freezes the contract.
  **Open:** D-OPEN-1 tool name + HTTP lane · D-OPEN-2 `AnswerRequest` in `marley_fleet::verbs` vs
  adapter-local (+ `Receipt<()>` content parse?) · D-OPEN-3 the result's path back to the view +
  where the seat-id→state map lives · D-OPEN-4 stale-question guard (clear on question-VALUE change
  while pending?) · D-OPEN-5 error surface form (card-local line vs `status_flash` vs both).

- **Promotion (/work 376-379,335, 2026-07-21):** queued→active, ticket claimed + in-progress/plan,
  AAR `839c2b6b`. **Attack pass:** the write lane verified verbatim (`tool_call_request` embeds the
  bearer + `Connection: close`; `parse_write_ack` fail-closed over BOTH failure planes;
  `claim_request`/`comment_request` = the closed-set precedent); verbs.rs = Send/Read/Open/Surface +
  `Receipt<T>`, NO answer verb ✓; the inert card + D6 hint ✓ (line drift only);
  `claim_forge_ticket` (:6955) confirmed the `let _ =` discard AND surfaced a BONUS precedent — its
  `forge_pending` mpsc-Receiver-drained-on-pump is exactly D-OPEN-3's answer. **Sweep drift noted:**
  the draft pre-dates #376 — `FleetSubscription` IS now app-wired; the demo verb is GATED while a
  subscription exists (headless boots have no orchestration config → no subscription → the demo-fed
  drive story stands unchanged); the Out-item "wiring the live feed" is DONE. Phase 1 PASS.

## Phase 2 — Design
- **D-OPEN resolutions:**
  - **D-OPEN-1 → tool name `session.answer`, lane `tool_call_request`.** The session-verb family is
    dotted BY SHIPPED CONVENTION (`session.surface_to_human` on the marley_mcp expose side; the §5
    vocabulary `session.list/read/send/…`); the forge sidecar's dash names are TICKET-domain tools —
    a different family. Lane: the shipped, live-proven write lane (`tool_call_request`, same as
    claim/comment — session-less `tools/call` is accepted by the real forge today); if L0's stricter
    server later demands `Mcp-Session-Id` on writes, the adapter grows it then (recorded in the
    contract note).
  - **D-OPEN-2 → `AnswerRequest { id, choice }` in `marley_fleet::verbs`** mirroring `SendRequest`
    (the envelope IS the verb vocabulary; #367 built the module for exactly this). Wire projection
    (`{session_id, choice}`) stays in the adapter builder. v1 receipt = `parse_write_ack` Ok/Err
    (no `Receipt<()>` content parse until the server-side contract exists — the isError text IS the
    refusal reason).
  - **D-OPEN-3 → one standing mpsc channel drained on the pump** (the `forge_pending` precedent,
    upgraded): `fleet_answer_tx: Sender<(String, Result<(), String>)>` cloned into each send thread,
    the Receiver drained in `pump_fleet_live` (already the fleet pump seam) → machine apply → dirty.
    The seat-id→record map: `fleet_answers: HashMap<String, AnswerRecord>` on RootView.
  - **D-OPEN-4 → the record stores the QUESTION VALUE** (a `Question` clone taken at pick time); the
    pure `record_current(seat, record)` decision = `question_card(seat) == Some(&record.question)` —
    a CHANGED question (new `QuestionRaised`), a cleared one, or a state flip all read stale → the
    pump scrub drops the record (re-armed for the new question). Scrub runs EVERY pump tick
    (`retain` over the current snapshot — ≤8 seats, cheap) so demo-verb installs and live drains both
    heal without extra plumbing.
  - **D-OPEN-5 → card-local error line.** The `Failed(reason)` chip renders IN the card (danger
    color) — per-seat correct where a global `status_flash` would misattribute with multiple seats.
- **Render-path consequence (the one structural delta):** the #369 card render is a FREE fn (no
  `Context<Self>`), so option chips cannot take `cx.listener` closures there. `fleet_rail_body` +
  the seat-card fn become RootView METHODS (masked, same bodies) so the chips gain
  `on_mouse_down(cx.listener(answer_fleet_question(seat_id, choice)))` — transparent to the #369
  drives (they exercise render via draw).
- **File manifest:**
  1. `crates/marley_fleet/src/verbs.rs` — `AnswerRequest { id, choice }` (serde round-trip).
  2. `crates/marley_forge_client/src/lib.rs` — `pub(crate) fn answer_request(endpoint, session_id, choice)`
     = `tool_call_request(endpoint, "session.answer", 1, {session_id, choice})`.
  3. `crates/marley_forge_client/src/adapter.rs` — masked `ForgeClient::answer_question(&self, session_id, choice) -> Result<(), ForgeError>`
     (the `claim_ticket` shape + `parse_write_ack`).
  4. `crates/marley_app/src/fleet_rail.rs` — pure: `AnswerPhase { Pending, Answered, Failed(String) }`,
     `AnswerRecord { question: Question, phase: AnswerPhase }`, `pick_permitted(Option<&AnswerRecord>) -> bool`
     (None/Failed → true; Pending/Answered → false), `answer_applied(Result<(), String>) -> AnswerPhase`,
     `record_current(&Session, &AnswerRecord) -> bool`.
  5. `crates/marley_app/src/app.rs` — masked: fields (`fleet_answers`, `fleet_answer_tx/rx`),
     `answer_fleet_question(seat_id, choice)` (gate → record Pending w/ question clone → thread send
     via ForgeClient clone → result through the channel), the pump drain + every-tick scrub inside
     `pump_fleet_live` (+ dirty), the card render methodized with live chips + phase/error line
     (retiring the "answered in Layer 2" hint), REQ-006 gate = `question_card` (existing) +
     `pick_permitted`.
  6. `crates/marley_forge_client/tests/livewire.rs` — `Script.answer_outcome` knob + `parse_rpc`
     records `params.name`+`arguments`; scenarios: recorded `session.answer` call w/ exact args;
     scripted `isError` → `Err(Rpc(reason))`; scripted HTTP 404 → `Err(Http)`.
  7. `crates/marley_app/src/headless_drive.rs` — a #377 drive: demo feed → pick via the method →
     (no forge client headless) `Failed` chip + re-arm; Pending-injected double-pick no-op;
     snapshot-replace flip → record scrubbed.
- **Regression Test Plan (per REQ):** REQ-001 unit `answer_request` distinct-values + fixture
  one-recorded-call + drive machine-motion · REQ-002 unit `pick_permitted` full table + drive
  Pending-injected second pick no-op · REQ-003 unit `answer_applied(Ok)` → Answered + drive asserts
  snapshot value-equal around the pick (no forgery — type-level: the machine never sees `&mut`)
  · REQ-004 unit Failed arms + re-arm + fixture refusal/HTTP-error reasons · REQ-005 unit exact
  envelope keys (parse the builder's body JSON) + fixture recorded-request parse · REQ-006 pinned
  t369_req003 + unit gate arms + drive non-Waiting pick no-op · REQ-007 unit `record_current`
  (changed/cleared/flipped → false) + drive snapshot-replace → record gone.
- **Risks:** R1 methodizing the card render must keep the #369 drives green; R2 per-chip closure
  captures (seat id + option clones per render) — house-idiom-consistent; R3 the pump drain MUST set
  dirty (PR-pump-dirty); R4 skip-detach — new methods near masked shims, re-run `--list` (the #376
  5th-strike is one ticket old); R5 `ForgeClient` must be `Clone` for the thread send (it is — claim
  uses `self.forge_client.clone()`).
- **Status: Phase 2 — Design PASS (goal-run confirm); ready for Phase 3 — Implement.**

## Phase 3 — Implement
- Built to the manifest; `cargo check --workspace` clean:
  1. **verbs.rs** — `AnswerRequest { id, choice }` (serde, mirroring `SendRequest`; choice verbatim).
  2. **forge_client lib.rs** — `pub(crate) answer_request` = `tool_call_request("session.answer",
     {session_id, choice})` beside claim/comment.
  3. **adapter.rs** — masked `ForgeClient::answer_question` (the `claim_ticket` shape;
     `parse_write_ack` fail-closed).
  4. **fleet_rail.rs** — pure `AnswerPhase { Pending, Answered, Failed(String) }`, `AnswerRecord
     { question, phase }`, `pick_permitted` (None/Failed → true), `answer_applied`,
     `record_current` (== on `question_card(seat)` vs the recorded question).
  5. **app.rs** — 3 fields (`fleet_answers` map, standing `fleet_answer_tx/rx` channel);
     `answer_fleet_question` (gate → Pending w/ question clone → thread send → tagged result via the
     channel; no-client → immediate visible Failed); `pump_fleet_live` grew the drain (absent record
     drops a late result — no overlay resurrection) + the every-tick stale scrub; the question card's
     chips are PICKABLE (`on_mouse_down` + `cx.listener`, armed/re-armed only) with the phase line
     replacing the "answered in Layer 2" hint.
- **Deviations:** (a) `fleet_rail_body`/`fleet_seat_card` stayed FREE FNS with `answers` +
  `cx: &mut Context<RootView>` threaded through (the design said methodize; a free fn takes
  `Context<RootView>` just fine — same capability, much smaller diff); (b) `Session.id` is a FIELD
  not a method (the `id()` accessor belongs to `SessionEvent`) — two-line fix at compile.
- **Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect
- **2 critics** (correctness/concurrency · security/integrity/provenance) — both independently
  converged on the SAME root: the v1 wire carried no question identity. Ledger:
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1a | MED (security critic) | Question-swap race: the listener captured only (seat_id, choice); a pump tick swapping Q1→Q2 between paint and click passed the "a question exists" belt and dispatched Q1's choice against Q2 — and the record then stored the RE-FETCHED Q2, so `record_current` read true forever (self-concealing: "answered" under a question the human never saw). | REAL | the listener captures the RENDERED `Question` clone; the belt is now exact equality `question_card(seat) == Some(&rendered)`; the record stores the rendered question |
  | F1b | MED (correctness critic) | Cross-send result aliasing: a superseded send's late result landed on the seat's NEW record (channel tagged by seat id only) — T1's stale `Ok` could overwrite T2's `Failed` → a false "answered" LOCKING the seat until the question changes. | REAL | per-pick NONCE: `AnswerRecord.nonce` + `fleet_answer_seq` counter + the channel carries `(seat, nonce, result)`; the drain applies iff `record.nonce == nonce` |
  | F1c | MED (wire leg, both) | `{session_id, choice}` carried NO question identity — after a scrub, an in-flight socket (≤5s) could land a stale answer server-side with nothing for the brain to refuse on. | REAL | the PROPOSED contract (still ours to shape) now carries `prompt` — `{session_id, choice, prompt}`; `AnswerRequest` gains the field; the brain refuses mismatches |
  | F2 | LOW | Server-controlled failure text rendered unclamped (control chars / unbounded length) into a trusted card. | REAL | clamped in the PURE seam: `answer_applied` strips control chars + truncates to 200 (unit-testable) |
  | F3 | LOW | Doc drift ×3: lib.rs "fixed pair" of writes (now a trio), adapter.rs closed-set list, the fleet_rail.rs "READ-ONLY (Layer 1)" header. | REAL | all three updated |
  | F4 | LOW | `AnswerRequest` defined but not re-exported (every sibling is) and unused. | REAL | added to the marley_fleet `pub use`; the P4 round-trip test consumes it |
  | F5 | LOW | adapter doc claims "behavior-proven by the livewire fixture scenarios" before they exist. | REAL (forward-dated) | P4 lands the scenarios (the doc becomes true before commit) |
  | F6 | LOW | Judged residues, spec-conformant, NO code change: a byte-identical re-raised question keeps an `Answered` record (unanswerable until the value changes); a sub-16ms Waiting→Working→Waiting round-trip is swallowed by the latest-wins cell (REQ-007's trigger unobserved). Both unclosable at snapshot level — they need a question NONCE in the L0 envelope. | ACCEPTED | recorded in the contract note as the L0 follow-up |
- **Clean surfaces (verified):** bearer provably cannot surface in any error string (every
  constructor traced; the request text is never formatted into errors); zero state forgery (no
  Session/FleetSnapshot construction anywhere in the diff; the pure fns never see `&mut`); wire
  injection impossible (serde_json end-to-end, body-only placement); double-dispatch airtight
  (synchronous Pending insert in one listener turn); drain→scrub order correct in every constructed
  interleaving; channel try_recv-only + teardown-tolerant; non-pickable chips carry NO handler;
  #369 drives + #376 livewire lane pass unchanged; closed-set-of-writes property intact
  (`answer_request` pub(crate), tool name hardcoded); §20 provenance clean.
- Post-fix: `cargo check` + clippy + fmt clean.
- **Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate
- **Tests written:** fleet_rail units ×3 (`t377_req002` pick-gate full table · `t377_req004`
  fold+clamp — exact-value asserts incl. the 200-char clamp `bad+197a` and the control-strip
  `[31mredline`, no never-run branches · `t377_req007` record_current matrix over DEMO seats);
  verbs round-trip + the `"prompt"` wire-name pin; forge_client `answer_request_envelope_exact`
  (method/name/arguments distinct-values + bearer header); livewire fixture extension (`Script.
  answer_refuse`/`answer_http_fail`, `RequestRecord.name/arguments`, `parse_rpc` 4-tuple, the
  `tools/call` serve arm + `write_ack_response`/`iserror_response`) + 3 real-socket scenarios:
  `t377_answer_call_records_the_frozen_contract` (exactly ONE call; arguments == the frozen JSON),
  `t377_answer_iserror_refusal_surfaces_reason`, `t377_answer_http_error_surfaces`; the headless
  drive `fleet_question_answer_machine_headless` (stale-question refusal → non-Waiting refusal →
  no-client visible-Failed + re-arm → injected-Pending blocks → snapshot value-equal throughout
  (no forgery) → StateChange flip + pump scrub clears, changed=true); 5 `#[cfg(test)]` accessors
  landed WITH their tests (the #371 rule).
- **Runs:** `cargo nextest run -p marley_fleet -p marley_forge_client -p marley` → **878 passed**
  (833→878; all #369/#372/#373/#376 regressions green). **Gate `--diff`: GREEN 15/15 on the FIRST
  attempt** — coverage 100, MSI 100 (the 7 machine + 3 builder mutants all killed), receipt written.
- **Driven live capture — N/A with cause (not silently skipped):** the pick→pending→flip loop needs
  a live brain answering `session.answer` (external, un-landed — the contract is THIS ticket's
  proposed deliverable); the no-client arm + the whole machine run headlessly through the REAL
  dispatch path; the fixture proves the wire; chad is at the machine (the standing synthetic-input
  hazard). Mechanism + units + fixture + headless drive carry it.
- **Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete
- **Docs:** CHANGELOG (first under Added); orchestration-shell §12 Layer-2 "question-forms
  answering" marked SHIPPED with the frozen contract + the L0 question-nonce follow-up recorded.
- **Knowledge:** failure `BF-claude-answer-race-no-question-identity-001` (both critics converged);
  prevention rule `PR-claude-async-answer-carries-question-identity-every-hop-001`; AAR 839c2b6b
  submitted (completed).
- **Ticket:** #377 closed (done) + ship comment; local doc → closed. **Archive:** pair → completed/.
- Lessons: two critics with DIFFERENT lenses independently found the same root (no question
  identity) at different hops — the convergence itself was the signal the wire needed the field;
  fixing the contract while it is still OURS to shape (pre-L0) cost one field, post-L0 it would
  have been a migration.
