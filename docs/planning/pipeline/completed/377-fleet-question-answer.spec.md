---
pipeline_id: 9f2605e5-ad7c-4c3f-9053-3400ee76a023
ticket: forge#377 (d81f8c67-7b4f-4c9d-8f83-b57c73796727) · local docs/planning/tickets/open/TICKET-377-fleet-question-answer.md
aar_id: 839c2b6b-c928-4900-b1db-0e81baf8b5e5
status: Phase 5 — Complete PASS
title: Fleet structured interrupts — answer the Waiting seat's question (receipted verb)
type: feature
milestone: M24
references:
  - docs/marley_architecture/orchestration-shell.md
  - docs/marley_architecture/fleet-control-plane.md
  - docs/planning/intake/mission-control-hypermedia-surface.md
  - docs/planning/pipeline/completed/369-fleet-rail-readonly.spec.md
  - docs/planning/pipeline/completed/368-forge-client-fleet-subscription.spec.md
  - crates/marley_app/src/fleet_rail.rs
  - crates/marley_app/src/app.rs
  - crates/marley_forge_client/src/lib.rs
  - crates/marley_forge_client/src/fleet.rs
  - crates/marley_forge_client/tests/livewire.rs
  - crates/marley_fleet/src/verbs.rs
---

## Title
Make the #369 question-card ANSWERABLE — the first Layer-2 gated write (orchestration-shell.md §12
Layer 2's "question-forms answering") and the in-shell slice of the mission-control AskUserQuestion
loop (docs/planning/intake/mission-control-hypermedia-surface.md, flagship loop #1). Today a Waiting
seat's card renders prompt + options as deliberately inert chips with an "answered in Layer 2" hint
(#369 D6; app.rs:1057-1098). This ticket is that Layer 2: picking an option dispatches exactly ONE
receipted answer via `marley_forge_client` — **contract-first**: a proposed MCP `tools/call` wire +
fixture-server tests are the deliverable (the #368 fixtures-ARE-the-contract pattern; live ucsosv2
support for the tool is an external follow-up). A local pending/answered chip shows until the seat's
own event stream flips its state — **truth stays with the reducer** (reducer.rs:107-115: the reducer
is the sole producer of snapshot state; the send path forges nothing). A failed or refused send
surfaces an error and re-arms the card. The answer is DATA on the control plane, never keystrokes
(fleet-control-plane.md's founding incident: "a menu had to be answered by pressing the digit 4 at a
screenshot" → interrupts carry question + options as data, answered as a receipted typed reply).

## Scope
### In
- **`crates/marley_forge_client`** — the pure wire half (cov/MSI 100): an answer request builder on
  the SHIPPED write lane (`tool_call_request`, lib.rs:170 — the generic `tools/call` framer; builders
  stay `pub(crate)` per the closed-set-of-writes doctrine, lib.rs:5-8) with arguments exactly
  `{session_id, choice}` (D1), exposed as ONE typed `ForgeClient` method (the `claim_ticket`
  adapter.rs:59 shape); receipt parsing rides the SHIPPED fail-closed `parse_write_ack` lane
  (lib.rs:326 — JSON-RPC error / `isError: true` / neither-result-nor-error are ALL failures; D6).
- **`crates/marley_forge_client/tests/livewire.rs`** — extend the #372 fixture `Script`/`FixtureForge`
  with a `tools/call` arm: record `params.name` + `arguments` + the echoed `Mcp-Session-Id` (today
  `parse_rpc` livewire.rs:385 records only method + `params.uri`, and a `tools/call` POST falls into
  the `_ => minimal_ok_response()` catch-all, livewire.rs:280) and script the answer outcome
  (accepted / `isError` refusal / HTTP error). The recorded request IS the frozen proposed contract
  handed to Layer 0 (D7).
- **`crates/marley_app/src/fleet_rail.rs`** — the pure decision half (cov/MSI 100): a per-seat local
  answer state machine (Armed → Pending → Answered / Failed{reason}; Failed re-arms — D2), the pick
  gate (REQ-002), the chip projection (pending/answered/error content), and the clear decision
  (REQ-007: the local record lives only while the SHIPPED `question_card` gate, fleet_rail.rs:85,
  still yields the card for that seat — reuse the same gate, derive nothing new).
- **`crates/marley_app/src/app.rs`** (shim, `mutants::skip`, coverage-excluded): `on_mouse_down` on
  the option chips (the 50-site house click idiom) replacing the inert render + "answered in Layer 2"
  hint (app.rs:1091-1096); an app-state map of seat id → local answer state; the masked send glue on
  a thread (the `claim_forge_ticket` app.rs:6853 idiom) — but UNLIKE claim's fire-and-forget
  (`let _ =` app.rs:6860) the Result MUST come back to the view and set dirty
  (`PR-claude-pump-state-change-must-set-dirty-to-repaint`; D-OPEN-3 picks the mechanism).
- **`crates/marley_fleet/src/verbs.rs`** — IF D-OPEN-2 lands "generic": an `AnswerRequest` mirroring
  `SendRequest`/`SurfaceRequest` (verbs.rs ships Send/Read/Open/Surface + `Receipt<T>` today — NO
  answer verb exists; the sweep's headline gap).
- **Headless + demo story**: drive the pick through `dispatch_for_test`/the real mouse path against
  the demo fixture (`demo_snapshot`'s Waiting seat "dev-4/review" already carries a 3-option
  question, fleet_rail.rs:147-163); the stream-flip half via the snapshot-replace idiom (the #369 D5
  demo-feed verb) since the live ② pump is not yet wired into the app (sweep: `FleetSubscription`
  has zero references in `crates/marley_app/` — `fleet_snapshot` is demo-fed only, app.rs:2171/8146).

### Out (explicitly deferred)
- **Live ucsosv2 server support** for the answer tool — external follow-up; the fixture-recorded
  request is the proposed contract (exactly the #368 `seat_events` handoff pattern).
- **The dispatch composer (#378)** — free-form briefs, delivery states, `session.send` UI. This
  ticket answers a STRUCTURED question only.
- **Free-text answers** — a `Question` with EMPTY `options` (legal per session.rs:46) renders no
  pickable option rows, so it gets no answer affordance in v1; the free-text path is composer (#378)
  territory.
- **Multi-question queues** — the envelope holds at most ONE question per seat (a flat
  `Option<Question>`, session.rs:63-64); nothing to queue.
- **Keyboard focus nav** — the rail is a dock panel with NO focus machinery (sweep-confirmed; focus
  lives with panes/editor); v1 is mouse-only (D4). A rail focus system is its own ticket.
- **Wiring the live `FleetSubscription` into the app** — the ② feed slice; the chip-clear loop is
  proven at the pure layer + via snapshot replacement.
- **Any reducer/event-schema change** — no new `SessionEvent` kind; the answered seat flips via the
  EXISTING stream when the adapter/L0 processes the answer (that is the point of REQ-003/007).
- OS notifications (#226), answer history/audit UI, batching/retry policy beyond re-arm-on-failure.

## Reference (§20)
**N/A — Marley-specific.** The fleet control plane and its question-as-form loop are Marley-original
design: orchestration-shell.md §5 ("`question` is structured … answered as a receipted typed reply.
No digits pressed at screenshots"; the §5 both-directions table row "Structured interrupts
(question-as-form) — event in (consume) → rendered form → answer verb (expose)"), §12 Layer 2
("question-forms answering"); fleet-control-plane.md "Structured interrupts — questions as forms"
(the answer flows back as a receipted reply) and its incident table (answers must be data, not
digits at a screenshot); the mission-control intake's flagship loop #1 (question card → tap an
answer → the run unblocks) — this ticket is that loop's in-shell slice. The behavior maps were
checked (below): Warp's agent protocol attests the question-as-typed-data PRINCIPLE but no
fleet-seat rail surface exists to observe, and no copyleft source (Warp AGPL / Zed GPL) was read.

### Prior art
1. **Behavior maps** — docs/warp_architecture/subsystems/04-agent-ai-mcp.md: Warp's Agent Mode is a
   cloud conversation loop whose typed action vocabulary INCLUDES `AskUserQuestion`
   (04-agent-ai-mcp.md:110, :145) with matching typed action-RESULT types sent back to the server —
   research-level confirmation that "an agent's question and its answer travel as typed protocol
   data, never synthesized input" is attested behavior in the reference app. It is a
   conversation-stream tool call, not a fleet-seat control-plane verb, and our map records no UI
   detail — nothing further to observe or adopt. docs/zed_architecture/ has no agent-question
   surface (crate maps only; #369's sweep conclusion re-verified). No source consulted.
2. **Published — the MCP specification rev 2025-06-18, "Tools → Calling tools"**: a `tools/call`
   request is `{jsonrpc, id, method: "tools/call", params: {name, arguments}}`; the result carries
   `content: [...]` plus the TOOL-level `isError` flag, distinct from a JSON-RPC protocol error —
   both failure planes must be checked. Already adopted verbatim in-tree: `tool_call_request`
   (lib.rs:170-197) builds exactly this shape and `parse_write_ack` (lib.rs:326-346) fail-closes
   over both planes (the #74 lesson: a failed write must NEVER read as success). The answer rides
   this existing adoption; nothing new is taken from the spec.
3. **OUR OWN shipped crates (the load-bearing leg — this sweep settled most of the design):**
   - **`marley_fleet/src/verbs.rs`** — Send/Read/Open/Surface requests + the generic
     `Receipt<T> { Accepted{value} | Refused{reason} }` (a refused verb is a first-class outcome —
     the dialog-up rule, verbs.rs:60-62). **NO answer verb type exists yet**; `Receipt` is the
     receipt vocabulary to reuse, and `SurfaceRequest{id}`/`SendRequest{id,…}` are the naming shape
     if D-OPEN-2 puts `AnswerRequest` here.
   - **`marley_fleet/src/session.rs`** — `Question { prompt, options, context_refs }` (NB the field
     is `context_refs`); `question.is_some() ⇒ State::Waiting` is reducer-enforced one-directional;
     `options` may be empty (the free-text case → scoped out).
   - **`marley_fleet/src/reducer.rs`** — the invariant-authority doc (reducer.rs:107-115): the
     reducer is the SOLE producer of snapshot state; `QuestionCleared` clears the question leaving
     state Waiting; `StateChange`/`Upsert` away from Waiting clear it. REQ-003/007 have a mechanical
     home: the send path never touches this fold.
   - **`marley_app/src/fleet_rail.rs`** — `question_card(&Session) -> Option<&Question>` (:85, pure,
     tested t369_req003) is the ONLY gate REQ-006 needs pinned; `demo_snapshot` (:119) already
     ships a Waiting seat with a 3-option question for the drive/capture.
   - **`marley_app/src/app.rs`** — the inert card render to make live (:1057-1098, the D6 hint at
     :1095); `forge_client: Option<ForgeClient>` built from `.mcp.json` at :1291;
     `claim_forge_ticket` (:6853) = the masked thread-send idiom, whose `let _ =` result-discard
     (:6860) is exactly what REQ-004 must NOT copy; `status_flash` (:541) = an existing transient
     error-surface candidate (D-OPEN-5); 50 `on_mouse_down` sites = the click idiom; **the live
     `FleetSubscription` is NOT wired into marley_app** (zero references; snapshot is demo-fed,
     :2171/:8146) — bounds the honest test story.
   - **`marley_forge_client/src/lib.rs`** — the whole write lane is SHIPPED: `tool_call_request`
     (:170, generic tools/call framer, bearer embedded, `pub(crate)` closed-set doctrine),
     `parse_write_ack` (:326, fail-closed over both failure planes), `claim_request`/
     `comment_request` (:349/:363) as the closed-set precedent, and the distinct-values args-map
     test idiom (:563 — a key swap fails loudly). `mcp_post` (:204) is the OTHER lane: it carries
     `MCP-Protocol-Version` + `Mcp-Session-Id`; `tool_call_request` carries neither → D-OPEN-1.
   - **`marley_forge_client/tests/livewire.rs`** — the `Script`/`FixtureForge` harness (#372):
     thread-per-connection loopback fixture, `RequestRecord{method,uri,session}`, scripted
     failures. `serve_post`'s catch-all would swallow a `tools/call` today (:280); `parse_rpc`
     (:385) needs a `params.name`/`arguments` record — the small, known extension surface.
   - **`marley_mcp`** — the EXPOSE side ships dotted `family.verb` tool names (`fleet.snapshot`,
     `session.surface_to_human`) + the `session.write` grant class, while the forge sidecar's own
     tools are dash-named (`ticket-claim`) — the naming fork D-OPEN-1 must settle.

## Locked-In Decisions
- **D1 — Contract-first minimal wire.** The answer is ONE MCP `tools/call` whose `arguments` are
  exactly `{session_id, choice}` — `choice` = the picked option string VERBATIM (no index: indices
  renumber if the question re-renders; the string is what the human saw). No free-text field, no
  batching, no question-id (the envelope holds one question per seat; the stale-question hazard is
  D-OPEN-4). The exact tool NAME + HTTP lane are D-OPEN-1, but the SHAPE is locked. The fixture
  recording is the proposed contract; live ucsosv2 implementing the tool is an external follow-up
  (the #368 pattern). The answer is control-plane DATA — no keystroke/PTY path exists in this ticket.
  **Rejected:** answering via `session.send` text injection (re-creates the press-digit-4-at-a-
  screenshot incident the control plane exists to kill).
- **D2 — The pending gate is a PURE seam.** A small per-seat answer state machine in `fleet_rail.rs`
  (Armed → Pending → Answered / Failed{reason}; Failed → re-armed) at cov/MSI 100; the shim only
  consults it. One in-flight answer per seat, ever (REQ-002). **Rejected:** gating in the shim
  (untestable — app.rs is coverage-excluded, the #307/#369-D2 doctrine).
- **D3 — No optimistic state forgery.** The send path never constructs, patches, or replaces a
  `Session`/`FleetSnapshot`; the reducer stays the sole producer (reducer.rs:107-115). The
  pending/answered chip is app-LOCAL render state keyed by seat id, resolved per-seat at draw time
  and cleared by REQ-007's pure decision over reducer output. **Rejected:** synthesizing a local
  `QuestionCleared`/`StateChange` on receipt Ok (state forgery — a lost answer would render as an
  answered seat).
- **D4 — v1 interaction = mouse click on the option chip.** The house `on_mouse_down` idiom (50
  sites); the #369-D6 inert chips become interactive and the "answered in Layer 2" hint is retired.
  Keyboard nav DEFERRED: the sweep confirms the rail has no focus machinery. Per-card affordances
  gate on THAT seat's own data (#369 D7 carries over).
- **D5 — Seam split per the house lane.** PURE (cov/MSI 100): the answer request builder + receipt
  parse in `marley_forge_client` (builder `pub(crate)`, one typed `ForgeClient` method — the
  closed-set-of-writes doctrine holds: the app still cannot call an arbitrary mutating tool), and
  the fleet_rail state machine/gate/chip/clear fns. MASKED: the app.rs click arm + thread-send glue
  + result return. UNLIKE `claim_forge_ticket`, the Result crosses back to the view and sets dirty
  (`PR-claude-pump-state-change-must-set-dirty-to-repaint`).
- **D6 — Receipt semantics are fail-closed on the SHIPPED lane.** `parse_write_ack`'s rules govern:
  a JSON-RPC error, an `isError: true` tool result, and a neither-result-nor-error envelope are ALL
  failures; only a genuine success result is Ok. A refusal (the `Receipt::Refused`/dialog-up class)
  therefore surfaces its reason text and re-arms — it must never read as answered. **Rejected:**
  treating HTTP 2xx as success (the #74 lesson).
- **D7 — Fixture-server tests extend the livewire `Script` (the #368/#372 pattern).** A `tools/call`
  arm records `{name, arguments, session header}` and scripts the outcome (accepted / `isError`
  refusal / HTTP error); scenarios assert EXACTLY-ONE recorded call for REQ-001/002 and re-dispatch
  after failure for REQ-004. The recorded literals are the frozen proposed contract, duplicated
  loudly like PAGE_A (livewire.rs:27-33).

### D-OPEN (Phase 2 decides)
- **D-OPEN-1 — Tool name + HTTP lane.** Dash-style `session-answer` (the forge sidecar's own
  `ticket-claim` naming) vs dotted `session.answer` (the marley_mcp `family.verb` idiom); and
  `tool_call_request` (bearer-only, `Connection: close`) vs `mcp_post` (adds `MCP-Protocol-Version`
  + `Mcp-Session-Id`). The `{session_id, choice}` shape is locked either way (D1).
- **D-OPEN-2 — Where the request TYPE lives.** An `AnswerRequest { id, choice }` in
  `marley_fleet::verbs` (the generic MCP-tool-schema seam, mirroring `SendRequest`; wire projection
  in the adapter) vs v1 keeping the shape adapter-local; and whether the success content
  additionally parses into `Receipt<()>` or `parse_write_ack`'s Ok suffices.
- **D-OPEN-3 — The result's path back to the view.** Where the seat-id → answer-state map lives on
  the app struct and how the thread's Result re-enters (the existing pump/poll tick vs a channel
  drained on pump), honoring the dirty-flag rule. `claim`'s fire-and-forget is explicitly NOT
  acceptable here.
- **D-OPEN-4 — The stale-question guard.** While Pending, if the seat's QUESTION CHANGES (a new
  question arrives — legal via `QuestionRaised`), does the local state clear/re-arm? Candidate: the
  clear decision keys on the question VALUE, not merely `question_card` presence. P2 decides with
  the reducer semantics on the table.
- **D-OPEN-5 — Error surfacing form.** A card-local error line (glanceable where the pick happened)
  vs `status_flash` (app.rs:541) vs both. REQ-004 pins only that it surfaces and re-arms.

## Acceptance Criteria (EARS)
One observable behavior per row. Verify: unit = pure fn test (cov/MSI 100); fixture = the livewire
`FixtureForge` scenario (real loopback socket); headless = a drive through the real dispatch/mouse
path against the demo fixture; the app.rs arm itself is a masked shim (the #369 testing boundary).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the operator picks an option on a Waiting seat's question card, the app shall dispatch exactly ONE answer `tools/call` naming that seat's session id and the picked option string — and shall write nothing to any PTY. | unit (builder maps `{session_id, choice}` with distinct values — the lib.rs:563 idiom); fixture (exactly one recorded `tools/call` with the right name + arguments); headless (pick → dispatch observed) |
| REQ-002 | WHILE an answer for a seat's question is in flight, further picks on that card shall be no-ops — no second dispatch, no state-machine motion. | unit (Pending gate refuses a pick; full transition table); fixture (rapid double-pick → ONE recorded call) |
| REQ-003 | WHEN the send returns a success receipt, the card shall show a local pending→answered chip WITHOUT mutating the seat's reducer state — the `FleetSnapshot` (state AND question) compares equal before/after the whole send path until an event flips it. | unit (machine reaches Answered; snapshot equality asserted around the send-path decision fns); headless (seat still renders Waiting + card until the feed changes) |
| REQ-004 | WHEN the send fails — a transport/HTTP error, a JSON-RPC error, or an `isError`/refused tool result — the card shall surface the failure reason and re-arm: a subsequent pick shall dispatch again. | unit (Failed{reason} arm per failure class via the `parse_write_ack` lanes; Failed → pick allowed); fixture (scripted refusal + scripted HTTP error → a second pick is recorded) |
| REQ-005 | WHEN the answer request is built, it shall be a well-formed MCP `tools/call` per the proposed contract: the JSON-RPC envelope `{method: "tools/call", params: {name: <answer tool>, arguments: {session_id, choice}}}` with the bearer header, POSTed to the forge endpoint. | unit (exact envelope keys + header asserts on the builder output); fixture (the recorded request parses as the frozen contract literal) |
| REQ-006 | WHEN a seat is non-Waiting, or Waiting without a question, its card shall offer NO answer affordance; each card's affordance shall gate on that seat's own data only. | existing t369_req003 pinned + unit on the affordance decision (Waiting∧question ⇒ pickable, all other arms ⇒ not); headless (non-Waiting seats in the demo fixture expose no pick) |
| REQ-007 | WHEN the seat's own event stream clears the question or moves the seat out of Waiting, the local pending/answered chip (and its record) shall clear — the reducer's truth replaces the local overlay. | unit (clear decision over reducer outputs: `question_card` gone ⇒ record cleared); headless (snapshot-replace flip → chip gone next frame, dirty set) |

## Floors (constitution)
Pure seams (the fleet_rail answer machine/gate/chip/clear fns; the forge_client builder + receipt
parse) at **cov/MSI 100**; the app.rs click/thread glue masked in the coverage-excluded shim.
Typed errors end-to-end (`ForgeError` carries every failure class), no `unwrap` on any input path;
exhaustive `match` over closed enums (the answer machine's states); distinct-value args-map tests
(a key/value swap fails loudly); run `cargo mutants --list -f` on the ACTUAL touched files before
claiming the kill set (the syntactic-form lesson), and re-check after any fn insertion near a
`mutants::skip` shim (the skip-detach trap).

## Phase Plan
- **P2 Design** — settle D-OPEN-1..5; exact signatures + file manifest (fleet_rail machine fns;
  the `ForgeClient::answer_*` method + builder/parser; the livewire `Script` extension; the app.rs
  state map + click arm + result return); the fixture scenario matrix per REQ; the headless drive
  plan (demo fixture pick → chip → snapshot-replace flip); confirm the #378 composer boundary
  (nothing free-text leaks in).
- **P3 Implement** — pure seams first (fleet_rail machine + forge_client builder/parser + any
  D-OPEN-2 verbs.rs type), then the livewire `tools/call` arm, then the masked app.rs glue
  (click → thread send → result back → dirty).
- **P3.5 Inspect** — adversarial: can ANY path double-dispatch (click re-entrancy, thread races)?
  grep the send path for snapshot/Session construction (D3 forgery check); per-card gating honest
  (a pick on seat A can never mark seat B)? bearer never logged on the new path? the request text
  (which embeds the bearer) never surfaces in the error string? skip-detach re-list; §20 provenance.
- **P4 Validate** — write + RUN the units (cov/MSI 100 on the pure seams), the fixture scenarios,
  and the headless drives; `cargo mutants --list -f` on the actual touched files; gate green
  (`--diff`); the gate-15 capture of the pick→pending→flip loop if the machine is unlocked, else
  the standing mechanism fallback (`PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`).
- **P5 Complete** — CHANGELOG; record the frozen answer-wire contract beside the #368 contract notes
  (the ucsosv2 follow-up handoff); archive, AAR capture, close #377.
