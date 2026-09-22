---
pipeline_id: 53aba46c-19c0-47aa-8959-487ffeb86868
ticket: forge#378 (b40bc0c2-76dd-448f-867b-534cc983408b) · local docs/planning/tickets/open/TICKET-378-fleet-dispatch-composer.md
aar_id: 57a080c9-40a7-49c2-94bc-99e400c8b1ac
status: Phase 5 — Complete PASS
title: Fleet dispatch composer — session.send as mailbox data + delivery-state chips
type: feature
milestone: M24
references:
  - docs/marley_architecture/orchestration-shell.md
  - docs/marley_architecture/fleet-control-plane.md
  - docs/planning/pipeline/completed/368-forge-client-fleet-subscription.spec.md
  - crates/marley_fleet/src/dispatch.rs
  - crates/marley_fleet/src/verbs.rs
  - crates/marley_forge_client/src/fleet.rs
  - crates/marley_app/src/fleet_rail.rs
---

## Title
Layer-2 gated-writes ②: compose a brief to ONE seat as **mailbox data** — never keystrokes (the
two-planes rule, orchestration-shell §5) — sent as a proposed `session.send` MCP `tools/call`
through `marley_forge_client` (contract-first fixtures; live Forge-side support is an external
follow-up), receipt-tracked through the SHIPPED-BUT-UNWIRED `marley_fleet::dispatch` delivery-state
machine (`Deposited → Claimed → Started`, advanced by `seat_events` echoes), rendered as a
delivery-state chip on the seat's fleet-rail card. The v1 composer reuses the inline-draft input
idiom (#177 `renaming_tab` / #204 `naming_workflow`) targeted from the rail — no new modal
machinery. Kills the evidence-night failure class fleet-control-plane §2 names verbatim: 3 seats sat
idle holding **unsent briefs** because `tmux send-keys` is fire-and-forget with no receipt — this
ticket is dispatch-with-a-receipt, watched live on the rail (§7 item 5's thin end).

## Scope
### In
- **The inline dispatch draft** (marley_app): a compose affordance targeted at exactly one seat
  (entry point D-OPEN-ENTRY) opens an `Option<(seat_id, draft)>` inline draft — key-listener branch
  owning the keyboard (Esc cancels, Enter confirms, Backspace/printables edit, `stop_propagation`),
  overlay card render — byte-for-byte the #177/#204 idiom (app.rs:14287-14343, :17193-17219).
- **The send seam** (marley_forge_client): a pure `session.send` request builder + receipt parser
  (the fleet.rs builder/parser idiom over `mcp_post`/`tool_call_request`), and a THIRD explicit
  masked method on `ForgeClient`'s closed write set mirroring `claim_ticket`/`comment_ticket`
  (adapter.rs:59-73) — no generic tool-name surface (the lib.rs:5-8 doctrine holds).
- **The echo projection** (leaning, D-OPEN-ECHO-CARRIER): a pure adapter-side
  `dispatch_echoes(page) → Vec<(seat_id, DeliveryState)>` mapping the proposed
  `dispatch-claimed`/`dispatch-started` `seat_events` kinds (fleet-control-plane §5's v1 enum
  already names `dispatch-claimed`, :160) to the GENERIC `DeliveryState` — UCSOS vocabulary stays
  confined to the adapter (fleet.rs:6-8).
- **The compose/delivery state machines as PURE seams** (cov/MSI 100): draft→sending→deposited |
  failed(reason, draft preserved); a per-seat delivery tracker advanced ONLY via the shipped
  `DeliveryState::observe` (dispatch.rs:33-45), seeded `Deposited` by an Accepted receipt.
- **The chip render decisions** in the pure `fleet_rail` module (label/tone per `DeliveryState` +
  the transient `sending`), resolved into the seat card by the existing app.rs shim
  (`fleet_rail_body`, app.rs:966; seat card :992-1070).
- **Glue, masked**: the background send via the #69/#72 pattern (clone client + mpsc + pending
  receiver + `thread::spawn`, app.rs:6852-6863); repaint on result arrival
  (`PR-claude-pump-state-change-must-set-dirty-to-repaint`).
- **Contract-first fixtures**: the exact request JSON + receipt envelope + echo rows, pinned by unit
  fixtures that ARE the proposed v1 contract handed to Layer 0 (the #368 idiom; livewire.rs:27-30
  freezes contracts this way). A `fleet-demo` dispatch sequence extends the #369 demo feed so the
  chip lifecycle is drivable headlessly + capturable.

### Out (explicitly deferred)
- **Live Forge-side `session.send` support** — the server half (mailbox deposit, seat routing, echo
  emission) is an external L0 follow-up; this ticket ships against fixtures (the #368 posture).
- **Broadcast / multi-seat send** — v1 targets exactly one seat.
- **A message-history pane** and **attachments** — text-only, latest-dispatch-only.
- **The question-ANSWER path** — #377, a separate sibling ticket (answering a `Waiting` seat's
  structured question is its own receipted verb).
- **The full §7-item-5 composer form** (ticket picker → assignment template → merge-scope field →
  capability auto-refuse pre-check): the pre-check is manager/adapter POLICY
  (orchestration-shell §3, :85-88) — v1 Marley surfaces the server's `Refused` receipt (REQ-005),
  it does not pre-judge.
- **Per-dispatch correlation ids in echoes** — v1 tracks ONE dispatch per seat (latest), attributed
  by seat id (D7 acknowledges the stale-echo window); ids ride a later contract rev.
- **Wiring the live `FleetSubscription` pump into app.rs** — that is #376 (fleet-rail-livewire, a
  queued sibling); the app's fleet feed today is the demo verb (app.rs:8146; no `FleetSubscription`
  reference in app.rs). This ticket's tracker consumes the SAME page/event path either way and is
  proven against the demo feed + fixtures; soft order after #376 is preferred but not required.

## Reference (§20)
**N/A — Marley-specific.** The dispatch composer is item ⑤ of the fleet control plane's first-class
feature set (docs/marley_architecture/fleet-control-plane.md §7 item 5: "Dispatch composer with
delivery states … deposit → watch deposited → claimed → started on the seat rail") and the
"Dispatch (composer, delivery states)" consume-direction row of orchestration-shell §4 (:97), built
on §5's two-planes rule (:127-132): brief text is mailbox DATA on the dispatch plane; keystrokes
shrink to a receipted nudge that is NOT this ticket. There is no reference-app behavior to match:
Warp's AI input is a prompt into its OWN attached agent conversation (behavior map, checked — leg 1
below), not an operator→arbitrary-seat mailbox dispatch with delivery receipts. Clean-room §20
untouched: no Warp (AGPL) / Zed (GPL) source is read or consulted.

### Prior art
1. **Behavior maps — checked, N/A.** `docs/warp_architecture/subsystems/04-agent-ai-mcp.md`: typed
   input is classified shell-vs-natural-language to gate Warp's OWN Agent Mode (:54-56), and
   `RunAgentsRequest` models server-side child-agent dispatch (:158) — a conversation-attached
   prompt, not a fleet-seat mailbox composer with delivery states; no receipt/echo surface to map.
   `docs/zed_architecture/` — no fleet/dispatch analog (editor-domain maps).
2. **Published — the MCP specification (2025-06-18), `tools/call`:** request
   `{jsonrpc:"2.0", id, method:"tools/call", params:{name, arguments}}`; result
   `{content:[{type:"text", text}], isError?}` — both shapes are ALREADY encoded in-tree
   (`tool_call_request` lib.rs:170-197; `RpcResult.isError` lib.rs:383-389), so the send call is a
   new (name, arguments) pair over shipped framing, not new protocol work.
3. **OUR OWN source (the load-bearing leg — the audit this spec stands on):**
   - **`marley_fleet::dispatch` — the real surface, and it MATCHES the ticket's premise:**
     `DeliveryState { Deposited, Claimed, Started }` ordered by declaration (dispatch.rs:9-18);
     `observe(self, observed) -> DeliveryAdvance { state, advanced }` — a monotone join: a strictly
     further-along observation advances, the `Deposited → Started` skip is legal, an at-or-behind
     (duplicate / out-of-order) observation is an idempotent no-op (dispatch.rs:29-46). Lowercase
     serde; the full 3×3 observe table + round-trip are already tested (t367_req012/013).
     **Genuinely unwired:** zero references outside `marley_fleet` (workspace grep) — #367 shipped
     the machine "Layer-2-ready"; this ticket IS Layer 2 picking it up.
   - **`marley_fleet::verbs` — the send request is ALREADY TYPED:** `SendRequest { id, text }`
     (verbs.rs:9-14 — field `id`, NOT `session_id`; the intake sketch's `{session_id, text}` yields
     to the code's truth, D1) and `Receipt<T> = Accepted{value} | Refused{reason}` internally
     tagged `result`/snake_case (verbs.rs:62-75), both round-trip-tested. The module doc pre-states
     the semantics: delivery is receipted, "Enter as a separate write" is the DELIVERY side's
     concern (bridge, pre-solved — fleet-control-plane :122), and a refused send (a dialog is up)
     is a first-class `Receipt` outcome, not an error to swallow (verbs.rs:6-8) → REQ-005's arm.
   - **`marley_fleet::reducer`/`session` — NO dispatch arm exists:** `SessionEvent` is a CLOSED v1
     six-kind set (reducer.rs:11-14: Upsert/StateChange/QuestionRaised/QuestionCleared/Heartbeat/
     Ended) and `Session` carries no delivery field (session.rs:54-76). An unknown `event_type` row
     degrades through `project_row` to `Upsert(known state)` or `Heartbeat` (fleet.rs:479-483) — a
     dispatch echo would be SILENTLY absorbed today (liveness advances, delivery state lost). The
     echo channel must therefore be ADDED at a seam — the D-OPEN-ECHO-CARRIER fork, with the
     vocabulary-boundary rule (fleet.rs:6-8, reducer's deliberate "Closed v1 set") as tiebreaker.
     Pleasant property of the leaning option: the generic degrade path still advances
     `last_event_ms` on echo rows with zero reducer change.
   - **`marley_forge_client` — the whole wire idiom is shipped:** pure builders over
     `crate::mcp_post`/`tool_call_request` with content-asserting fixture tests
     (fleet.rs:210-287, :992-1015); `parse_write_ack` is FAIL-CLOSED (JSON-RPC error, then
     `isError:true` → `Err(Rpc)` carrying the content text; neither-result-nor-error → Protocol —
     lib.rs:326-346) — the receipt parser layers `Receipt<()>` on this posture; the closed-write-set
     doctrine ("Writes are a fixed pair of methods — no generic tool-name surface", lib.rs:5-8)
     extends to a fixed TRIPLE; `ForgeClient::claim_ticket`/`comment_ticket` are the masked-method
     template (adapter.rs:56-73, 5s-bounded `fetch` :95-104).
   - **`tests/livewire.rs` — the scripted fixture forge:** `Script`/`FixtureForge` records every
     request and scripts responses (:65-172); `serve_post`'s catch-all currently answers any
     non-initialize POST with a minimal OK (:267-281) — a scripted `tools/call` arm (record
     name+arguments; respond Accepted/Refused/isError/404) is a small extension IF Phase 2's test
     plan elects a livewire send scenario (unit fixtures are the required floor, REQ-006).
   - **app.rs — every UI ingredient is pre-built:** `fleet_snapshot: Option<FleetSnapshot>`
     (app.rs:391) fed by the `fleet-demo-feed` verb (:8146); the pure-shim split `fleet_rail_body`
     (:952-1070 — chip FILL tones :1002-1008, label chips :1049, question card :1057; render call
     :15755); the inline-draft idiom TWICE over — `renaming_tab` (:421, key branch :14472) and
     `naming_workflow` (:163, begin :2961-2980 with the blank-guard `status_flash`, key branch
     :14287-14343 ending `stop_propagation` + `notify`, overlay card :17193-17219, the
     draft-in-flight gate :8728-8731); the #69/#72 background-write pattern (`claim_forge_ticket`
     :6852-6863: clone client → mpsc → pending receiver → `thread::spawn`); headless reachability
     via `dispatch_for_test` + palette verbs (the #369 lane).
   - **The design docs pre-commit the shape:** orchestration-shell §4:97 (deposit →
     `deposited → claimed → started`, rendered live), §5:115 (`session.send(id, text)` — receipted;
     refuses when a dialog is up), §5:127-132 (the two planes); fleet-control-plane §2:51 (the
     unsent-briefs failure row), :60-61 (the mailbox is the half that NEVER failed — this ticket
     routes dispatch onto it), §5:157-160 (`dispatch-claimed` already in the v1 event enum),
     §7:190-193 (item ⑤).

## Locked-In Decisions
- **D1 — Wire contract v1 = the SHIPPED types, contract-first.** `params.arguments` serializes the
  shipped `marley_fleet::verbs::SendRequest { id, text }` (the code's truth — NOT the sketch's
  `{session_id, text}`; `session_id` is the INBOUND `seat_events` row field, `id` is the outbound
  verb schema, one authoritative type already round-trip-tested). The receipt = the fail-closed
  envelope discipline (`parse_write_ack` idiom: JSON-RPC error / `isError` first) + the tool text
  deserializing to `Receipt<()>` (`Accepted`/`Refused`). The fixtures ARE the proposed contract
  handed to L0 (the #368 posture); live server support is an external follow-up. **Rejected:** a
  second send-request type; renaming the shipped field; custom framing.
- **D2 — Mailbox DATA through the closed write set ONLY (the two-planes rule as code).** The send is
  a THIRD explicit masked `ForgeClient` method mirroring `claim_ticket`/`comment_ticket` — no
  generic tool-name surface (lib.rs:5-8 doctrine). The draft text NEVER reaches any PTY/`send-keys`/
  spawn path in Marley — dispatch-by-keystroke is the failure class this ticket exists to kill;
  §18.1 inspect owes an explicit sweep for it. **Rejected:** typing into the seat's terminal;
  fire-and-forget anything.
- **D3 — Chip truth is machine-only (no local state forgery).** A seat's delivery chip renders
  EXCLUSIVELY from (a) the compose machine's transient `sending` and (b) a per-seat delivery
  tracker advanced ONLY via the shipped `DeliveryState::observe`, seeded `Deposited` by an
  Accepted receipt. No clock-driven advance, no render-side mutation, no optimistic `deposited`
  before the receipt. **Rejected:** advancing on send-initiate; any chip state the machine didn't
  produce.
- **D4 — Pure seams, masked glue.** The compose state machine (draft→sending→deposited |
  failed(reason, draft PRESERVED)), the per-seat tracker, the request builder, the receipt parser,
  the echo projection, and the `fleet_rail` chip decisions are pure, gpui-free, cov/MSI 100 (the
  fleet_rail.rs charter). The socket call + thread + channel + repaint are masked shims matching
  their siblings (`mutants::skip`; the whole-body thread mutant is timing-flaky — app.rs:6851).
- **D5 — v1 composer = the inline-draft idiom, verbatim.** `Option<(seat_id, draft)>` + a
  key-listener branch that owns the keyboard (`stop_propagation`) + an overlay card + the
  blank-confirm guard (#204's `status_flash` shape) — the #177/#204 pattern, third instantiation.
  NO new modal machinery; the full §7.5 composer form stays deferred (Scope Out). **Rejected:** a
  new modal framework; a free-floating text input in the rail card (the draft gate at app.rs:8728
  wants ONE in-flight-draft convention).
- **D7 — v1 correlation = one tracked dispatch per seat (the latest).** A new Accepted send
  RE-SEEDS that seat's tracker at `Deposited` — a fresh machine instance, not a monotone regression
  of the old one. Echo attribution is by seat id; a stale echo from a superseded dispatch can
  advance the new one's chip — acknowledged narrow window, resolved by per-dispatch ids in a later
  contract rev (Scope Out). **Rejected:** blocking a re-send while a prior dispatch is un-Started.
- **D-OPEN-TOOL-NAME** — the exact `tools/call` name. Proposal: `session.send`
  (orchestration-shell §5:115); counter-signal: Forge's in-tree tools are kebab-case
  (`ticket-claim`, `sprint-current`). One constant either way; Phase 2 locks it, the fixtures pin
  it, L0 ratifies at the external follow-up.
- **D-OPEN-ECHO-CARRIER** — how delivery echoes travel from `seat_events` to the tracker.
  **(a) LEANING:** a pure adapter-side `dispatch_echoes(page)` mapping proposed
  `dispatch-claimed`/`dispatch-started` row kinds → generic `(seat_id, DeliveryState)` — UCSOS
  vocabulary stays confined to `marley_forge_client::fleet` (its charter, fleet.rs:6-8),
  `marley_fleet`'s CLOSED v1 `SessionEvent` set stays closed, and the generic degrade path still
  advances liveness. **(b):** extend `SessionEvent` + `Session` with a delivery arm/field — a
  wire-schema ripple (reducer invariants, snapshot serde, the #370 tool schema those types feed).
  Phase 2 decides with the vocabulary boundary as the tiebreaker. Sub-fork: `dispatch-started` is
  OUR symmetric proposal (`dispatch-claimed` is already in the doc's v1 enum, fleet-control-plane
  :160) — pinned by the same fixtures.
- **D-OPEN-ENTRY** — the compose entry point: a rail-card affordance vs a palette verb (the #87
  dynamic-command idiom). Behavior LOCKED regardless: targeted at exactly ONE seat, and the chip
  lands on that seat's rail card. Phase 2 picks (the card affordance must respect
  `PR-claude-per-pane-render-affordance-must-gate-on-is-focused`-class per-item gating).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the operator invokes the compose affordance for a seat, the system shall open an inline draft bound to exactly that seat which owns the keyboard (Esc cancels and closes, Backspace/printables edit — the #177/#204 idiom). | pure draft-state units (open binds the seat id; edit ops); headless `dispatch_for_test` drive; driven capture |
| REQ-002 | WHEN a NON-blank draft is confirmed (Enter), the system shall dispatch exactly ONE `session.send` `tools/call` carrying the seat id + the draft text as DATA (`SendRequest {id, text}`); a blank draft shall dispatch nothing; no code path shall write the draft text to a PTY. | pure builder fixture asserts (locked tool name; `"id"`/`"text"` keys with DISTINCT values so a swap fails); compose-machine unit — confirm from `sending` is a no-op (the exactly-once gate); blank-guard unit; §18.1 inspect sweep: draft text reaches ONLY the builder |
| REQ-003 | WHEN the send returns an Accepted receipt, the dispatch shall render on that seat's rail card as a delivery-state chip at `deposited`. | pure units: receipt-Ok transition seeds the tracker `Deposited`; `fleet_rail` chip label/tone fns; driven capture of the card |
| REQ-004 | WHEN `seat_events` echo the dispatch's progress, the chip shall advance `deposited → claimed → started` via the existing `marley_fleet::dispatch` machine (`DeliveryState::observe`) — duplicate and out-of-order echoes shall not regress it, and the `Deposited → Started` skip shall be legal. | pure units: echo fixture pages → projection → tracker fold (in-order, duplicate, behind, skip); the shipped 3×3 observe table stays green; headless demo-feed drive advancing a chip |
| REQ-005 | IF the send fails (transport error, JSON-RPC error, `isError:true`, or a `Refused{reason}` receipt), THEN the system shall surface the error with its reason AND preserve the draft for edit/retry — a failed send shall never render as `deposited`. | pure per-arm units: each failure arm → `failed(reason)` with the draft intact + tracker NOT seeded; the Refused arm carries the server's reason verbatim; capture of the failed-state surface |
| REQ-006 | WHERE the wire is concerned, the outbound call shall be a well-formed MCP `tools/call` (`jsonrpc:"2.0"`, `method:"tools/call"`, `params.name` = the locked tool name, `params.arguments` = the `SendRequest` schema) and the receipt envelope parse shall be fixture-pinned across all arms — the fixtures being the proposed v1 contract (the #368 idiom). | builder/parser unit fixtures asserting exact JSON (the lib.rs `write_request_builders_map_args` style); the frozen-contract literals commented as handed-to-L0; a livewire scripted `tools/call` scenario IF Phase 2's test plan elects it |
| REQ-007 | WHILE dispatches are tracked, chip state shall be a pure function of the compose machine + delivery tracker only (no clock-driven or render-side advancement), and a seat with NO tracked dispatch shall render NO delivery chip (the #369 read-only rail unchanged for it). | structural: the tracker's only mutators are receipt-Accepted-seed and `observe` (inspect checkpoint); pure unit: absent tracker entry → `None` chip; #369 rail tests stay green |

## Floors (constitution)
Pure seams (request builder, receipt parser, echo projection, compose machine, per-seat tracker,
`fleet_rail` chip decisions) at **cov/MSI 100**; the socket/thread/repaint glue masked in the
already-excluded shims (matching siblings — and re-run `cargo mutants --list` after edits near
skipped shims, the detach trap). Run `--list` on the ACTUAL touched files before claiming a kill
set (the syntactic-form lesson). Exhaustive `match` over closed enums (`DeliveryState`,
`Receipt`, the compose machine — no defensive catch-alls). Typed errors end-to-end; fail-closed
receipt handling (a failed send NEVER reads as sent — the `parse_write_ack` posture). The bearer
rides the request text and is never logged (regression, lib.rs discipline). A pump/background state
change that affects render sets dirty (`PR-claude-pump-state-change-must-set-dirty-to-repaint`).

## Phase Plan
- **P2 Design** — lock D-OPEN-TOOL-NAME / D-OPEN-ECHO-CARRIER / D-OPEN-ENTRY; module map + exact
  signatures: `marley_forge_client` (send builder + receipt parser + echo projection if (a);
  `ForgeClient::send_session` masked), the pure compose/tracker module in `marley_app` (placement:
  `fleet_rail.rs` vs a sibling `dispatch_composer.rs`), `fleet_rail` chip decisions, the app.rs shim
  deltas (draft branch, overlay, background send + pending receiver, tracker fold point, demo-feed
  dispatch sequence); decide the livewire `tools/call` arm; per-REQ test plan.
- **P3 Implement** — pure seams first (builder/parser/projection/machines/chip decisions), then the
  masked glue; the demo-feed extension last.
- **P3.5 Inspect** — adversarial: does the draft text reach ANY PTY/spawn path? any chip mutation
  outside the machine (forgery)? is the exactly-once confirm gate airtight under key-repeat? does a
  Refused receipt genuinely preserve the draft? vocabulary boundary held (no UCSOS string outside
  the adapter)? D7's stale-echo window stated honestly? provenance (§20).
- **P4 Validate** — write + RUN the units per REQ; `cargo mutants --list -f` on the actual files;
  headless drives green; the driven capture (or the locked-screen mechanism fallback,
  `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`); gate green (`--diff`).
- **P5 Complete** — CHANGELOG; record the frozen v1 send/echo contract for the L0 follow-up ticket;
  orchestration-shell §12 shipped-state note; AAR; archive; close #378.
