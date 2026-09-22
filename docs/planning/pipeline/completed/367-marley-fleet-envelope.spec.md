---
pipeline_id: ed212558-91e1-4a84-8420-d76ff36334f8
ticket: forge#367 (d8e06631-eb00-4f8e-be7d-f2ce484be23b) · local docs/planning/tickets/open/TICKET-367-marley-fleet-envelope.md
aar_id: f1f217d1-e5e4-4f25-ae72-fb8c51a2e0f1
status: Phase 5 — Complete PASS
title: marley_fleet — the generic Session envelope + FleetSnapshot reducer (pure crate)
type: feature
milestone: M23
references:
  - docs/marley_architecture/orchestration-shell.md
  - docs/marley_architecture/fleet-control-plane.md
  - docs/marley_architecture/detached-sessions.md
---

## Title
NEW pure crate `marley_fleet` — the generic session envelope + fleet snapshot the whole control plane
stands on. orchestration-shell §2 names the stakes: *"the one schema agnosticism lives or dies on"* —
Marley is the project-AGNOSTIC mechanism shell, and every fleet surface (the rail, the tests, the MCP
tool schema) reads this one seam. The crate is types + a pure reducer, nothing else: no gpui, no
transport, no Forge vocabulary (**a Forge-specific string in this crate is a defect** — it belongs in
the adapter, `marley_forge_client`; §2's mechanical check, adopted verbatim). Ticket ① of the Layer-1
train (orchestration-shell §12); it forces the envelope v1 field-level decisions while they are cheap.

## Scope
### In
- New crate `crates/marley_fleet` (pure; cov/MSI 100; deps = serde only — see D13):
  - **(a) the `Session` envelope**, hardened from the orchestration-shell §3 v1 proposal: stable string
    `id` (never a tmux window index — the evidence-night lesson), `title`, `state` (CLOSED vocabulary
    `Starting | Working | Idle | Waiting | Error | Done`; Error ≠ Idle first-class), `question:
    Option<Question { prompt, options, context_refs }>` (present ⇒ Waiting; D5), `labels` (opaque
    ordered string map; D4), `last_event_ms` (D2), `transport: Option<Tmux | Bridge | Local>` (render
    hint only; Option per D7).
  - **(b) `FleetSnapshot`** — seats in first-seen order (D9) + the read-time attention derivation (D3/D10).
  - **(c) the reducer** — a generic `SessionEvent` stream (`Upsert · StateChange · QuestionRaised ·
    QuestionCleared · Heartbeat · Ended`, each carrying `id` + `ts_ms`) folded into the snapshot;
    deterministic + replay-safe: a cursor'd catch-up replays overlapping events through the same fn as
    a no-op (D6).
  - **(d) staleness/attention derivation** — a pure fn over (snapshot, injected `now_ms`,
    `stale_after_ms`): absence-of-heartbeat past the threshold flags Stale; Error and
    Waiting-with-question rank as attention (D3/D10).
  - **(e) the dispatch delivery-state machine TYPES** — `Deposited → Claimed → Started`, monotone-join
    semantics, Layer-2-ready (D11).
  - **(f) the verb TYPES** — `session.send` / `session.read` / `session.open` /
    `session.surface_to_human` request/receipt shapes as serde data (D12). Types only.
- serde derives throughout — the envelope doubles as the MCP tool schema (one seam, three consumers:
  UI model + test surface + tool schema).

### Out (explicitly deferred)
- **Anything Layer-2+**: verb *implementations*, receipted `session.send` delivery, the dispatch
  composer, question-form answering (orchestration-shell §12 Layer 2).
- **Live transports** — no tmux/bridge/local code, no sockets, no MCP subscription (that is #368's
  `marley_forge_client` growth); no RFC3339 parsing (the wire's ISO `ts` strings are the ADAPTER's to
  parse into `ts_ms` — this crate takes integers; D2).
- **Rendering** — no rail, no chips, no gpui (#369).
- **The MCP server** (`marley_mcp`, #370) — this crate only makes the types serde-ready for it.
- Settings wiring (`[[mcp.servers]]`, per-project brain config — Layer-1 ticket ⑤).
- The `marley_agent` ownership inversion (a parallel prerequisite refactor, orchestration-shell §7 —
  not this crate); unifying `AgentStatus` with `Session.state` (different seams: owned child vs
  external seat; revisit at the inversion).
- Event retention/pruning policy (an open fork, orchestration-shell §11 — the reducer RETAINS ended
  seats; pruning is consumer policy, D8).

## Reference (§20)
N/A — Marley-specific. Fleet orchestration of detached agent seats is Marley-original design with no
Warp/Zed behavior analog to observe; the behavior reference IS our own design record —
docs/marley_architecture/fleet-control-plane.md (the 2026-07-19 evidence night: every field in this
envelope traces to a named incident) and orchestration-shell.md (the 2026-07-20 ratified synthesis,
§3 envelope proposal + §7 crate map). No copyleft source consulted; the design rides open protocols
(MCP) and our own permissive deps only (orchestration-shell §13 provenance: `[Marley-original]`).

### Prior art
1. **Behavior maps (Warp/Zed):** checked the premise — neither docs/warp_architecture/ nor
   docs/zed_architecture/ covers fleet/seat orchestration (Warp's cockpit is single-session blocks;
   Zed's collab is CRDT buffer-sharing, a different problem). No analog; the reference-app leg is
   genuinely N/A. The in-house behavior source is fleet-control-plane.md §2's incident table — each
   envelope decision below cites it.
2. **Published — the MCP spec:** relevant only insofar as the envelope doubles as a tool/resource
   schema: MCP payloads are plain JSON, so the bar is serde-serializable types with a flat, stable
   JSON shape (flat `state` string + nullable `question` object beats a tagged mixed-representation
   enum — informs D5). The subscription/cursor model itself is #368's problem, not this crate's.
3. **OUR DEPS (the highest-yield leg — swept the workspace):**
   - **serde is already the workspace idiom** — `serde = { version = "1", features = ["derive"] }` in
     8 crates (marley_core, marley_forge_client, marley_settings, marley_util, …); no new dependency
     class. `marley_forge_client/Cargo.toml` (serde + serde_json, dev `mutants`) is the Cargo shape to
     mirror (D13).
   - **NO date/time crate exists anywhere in the workspace** — grepped every `crates/*/Cargo.toml`:
     no chrono, no time, no jiff. Timestamps must be plain integers or the crate imports a new
     dependency class for zero benefit (D2).
   - **The clock-injection idiom is established, three times over:**
     `crates/marley_lsp/src/rpc.rs:161` `expire(&mut self, now_tick: u64)` (+ `lifecycle.rs:116
     on_event(…, now_tick: u64)`); `crates/marley_agent/src/lib.rs:66` `agent_status_from(exited,
     active, quiet_ticks: u32)` with the lib.rs:94 comment *"`Date::now` is banned"*;
     `crates/marley_app/src/notify.rs` `should_notify(…, elapsed_ticks, threshold)`. detached-
     sessions.md states it as doctrine: *"nothing here needs a clock inside the domain crate."*
     Staleness therefore takes `now_ms` as a parameter (D3) — confirmed idiom, not invention.
   - **Ordered-string-map precedent:** `crates/marley_app/src/workflows.rs:37` already uses
     `BTreeMap<String, String>` for exactly this shape (deterministic iteration, JSON-object serde) —
     adopt for `labels` (D4).
   - **The consuming adapter's existing shape** (`crates/marley_forge_client/src/lib.rs` +
     `adapter.rs`): a PURE parse layer (typed `Deserialize` views, e.g. `TicketView`) over a masked
     socket adapter. #368 will extend that pattern — parse wire `seat_events` JSON in its pure layer
     and CONSTRUCT `marley_fleet::SessionEvent`s (including the RFC3339→millis conversion). Nothing in
     the existing client blocks the envelope shape; no fleet field needs to be stringly-typed for it.
   - No crate we ship owns an event-fold/state-machine seam to adopt — checked; the reducer is ~200
     lines of plain Rust over std + serde. **Found no owner beyond the pieces above; the sweep's yield
     is the timestamp representation, the clock idiom, the labels map, and the Cargo shape.**

## Locked-In Decisions
- **D1 — GENERIC-ONLY vocabulary.** No Forge/UCSOS string anywhere in the crate ("ticket", "phase",
  "gate" appear only as opaque label VALUES in tests, never as field names or match arms). Mechanical
  check adopted verbatim from orchestration-shell §2: a Forge-specific string in `marley_fleet` is a
  DEFECT; projection lives in `marley_forge_client` (#368).
- **D2 — timestamps are `u64` epoch-millis** (`ts_ms` on every event, `last_event_ms` on the seat).
  Evidence: no date crate exists in the workspace (prior-art leg 3); the integer-fed-in idiom is
  established (marley_lsp `now_tick: u64`). The wire's RFC3339 `ts` (fleet-control-plane §6) is parsed
  by the adapter, never here. Deliberate divergence from orchestration-shell §3's jsonc sketch (an ISO
  string) — that was a proposal sketch, not a wire lock; the MCP schema carries an integer.
- **D3 — no clock in the crate; attention is a READ-TIME derivation.** The reducer folds only
  event-carried timestamps; staleness/attention is a pure fn `(snapshot, now_ms, stale_after_ms) →
  attention set`, never stored in the fold — so snapshots stay bit-equal under replay regardless of
  when they are rebuilt. Evidence: the three-crate clock-injection idiom + detached-sessions.md's
  "Date::now stays banned" doctrine.
- **D4 — `labels: BTreeMap<String, String>`.** Deterministic sorted iteration (stable render + stable
  serialization), serializes as a JSON object for the tool schema, no duplicate keys. Precedent:
  workflows.rs:37. Values ride through UNINTERPRETED (the adapter writes dotted keys like
  `capabilities.mode`; Marley renders chips — orchestration-shell §3).
- **D5 — `question` is a separate `Option<Question>` field with a flat serde schema; the invariant is
  reducer-enforced and ONE-directional.** question present ⇒ `state == Waiting`; Waiting WITHOUT a
  question is legal (a seat gone quiet needing input — `agent_status_from`'s Waiting — vs AT_MENU
  which maps to Waiting + payload, fleet-control-plane §5/§6). A state-embedded `Waiting { question }`
  enum variant was considered and REJECTED: it type-enforces the invariant but serializes `state` as a
  mixed string/object — a worse MCP tool schema for the three-consumers seam (prior-art leg 2).
  `Question { prompt: String, options: Vec<String>, context_refs: Vec<String> }` (the
  `halted-with-question` payload, fleet-control-plane §6). `state`/`transport` serialize as lowercase
  strings.
- **D6 — the reducer is a pure, idempotent fold over an ordered stream.** Event set (closed, v1):
  `Upsert · StateChange · QuestionRaised · QuestionCleared · Heartbeat · Ended`, each carrying `id` +
  `ts_ms`. Every arm is idempotent under re-delivery (assignments + max-joins), so a cursor'd catch-up
  that overlaps already-applied events is a no-op — orchestration-shell §6's replay-from-cursor,
  honored by construction. `QuestionRaised` atomically sets `state = Waiting` + the question; any
  `StateChange` away from Waiting (and `Ended`) clears the question; `QuestionCleared` clears only the
  question and leaves state Waiting (the seat's own next event moves state — mechanism, not guessing).
  `last_event_ms = max(prev, ts_ms)` — monotone, tolerant of minor cross-box clock skew.
- **D7 — never drop a live seat: unknown-id events auto-vivify.** A non-Upsert event naming an unknown
  `id` creates a placeholder seat (`title = id`, empty labels, `transport = None`) rather than being
  dropped. Evidence: the UNREACHABLE incident (fleet-control-plane §2 — a live seat invisible is the
  cardinal failure) + retention pruning is anticipated (orchestration-shell §11), so mid-life stream
  starts WILL occur. Consequence: **`transport: Option<Transport>`** (None = no render hint) — a
  field-level hardening of the §3 proposal's required field, forced by honest placeholders.
- **D8 — `Ended` is not a launderer.** `Ended` sets `state = Done` UNLESS the seat is in `Error`, which
  is retained (a crashed seat must not read as cleanly Done when its process exits — the dead-vs-idle
  blindness, fleet-control-plane §2, + detached-sessions' remain-on-exit evidence rule). A recovered
  seat (`Error → StateChange(Working) → … → Ended`) ends Done normally. `Ended` clears any question;
  the seat is RETAINED in the snapshot (pruning is consumer policy).
- **D9 — snapshot order = first-seen order.** Seats list in order of the first event mentioning their
  `id` — a deterministic function of the stream, and a stable rail (no alphabetical re-sort jumps).
- **D10 — attention rank: Error → Waiting-with-question → Stale; ties in snapshot order; each seat
  appears once at its highest reason; Done seats are never stale** (their silence is legitimate).
  Rationale from the evidence night: an errored seat is stopped AND silent (the 15-min-unnoticed
  API-529 incident); a questioning seat has articulated its need (a bounded action); stale is
  suspicion, not declaration. Stale = non-Done seat with `now_ms − last_event_ms ≥ stale_after_ms`
  (threshold always injected).
- **D11 — delivery state = monotone join.** `Deposited < Claimed < Started`; an observation greater
  than current advances to it (multi-step forward legal — a missed intermediate must not wedge the
  machine); an observation ≤ current is an idempotent no-op reported as not-advanced. Deterministic,
  duplicate-tolerant, replay-safe. Types only, Layer-2-ready — mirrors the mailbox contract that
  "never failed" (fleet-control-plane §2/§5).
- **D12 — verbs as data only.** Request + receipt types for `session.send` / `session.read` /
  `session.open` / `session.surface_to_human` (orchestration-shell §5's table). Receipts are explicit
  `Accepted | Refused { reason }` — a refused send is a first-class outcome (the dialog-up rule);
  `open` takes an OPAQUE profile string (profile vocabulary is policy). No transport, no handlers,
  no I/O.
- **D13 — deps: `serde` (derive) only; dev-deps `serde_json` + `mutants`.** Mirrors
  `marley_forge_client/Cargo.toml`. No gpui, no tokio, no chrono — the purity is checkable from the
  manifest.
- **D-OPEN-READ-RANGE → SETTLED P2:** `session.read`'s range is **line-oriented**, a two-variant enum
  `ReadRange::Tail { lines: u32 } | Lines { start: u64, end: u64 }` (half-open absolute range). Tail is
  the ergonomic "what did it just output?" default; absolute Lines covers a precise window. **Byte
  offsets rejected** — tmux `capture-pane` is line-oriented and byte ranges leak text-encoding concerns
  into a transport-agnostic envelope. Both transports (tmux/bridge) can honor a line range. (Phase-2
  detail in the notes.)
- **D-OPEN-STALE-DEFAULT → SETTLED P2:** **no default const in v1.** The threshold stays a required
  injected parameter (D3) — a shipped `DEFAULT_STALE_AFTER_MS` would become a de-facto standard with
  zero evidence to back its value, and no consumer is blocked (staleness display is a rendering-policy
  choice #369's rail owns as its own documented constant). Adding a const later, once Layer-0
  heartbeat-cadence data exists, is a trivial non-breaking change.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the same ordered `SessionEvent` stream is folded from the empty snapshot twice, the reducer shall produce equal `FleetSnapshot`s (derived `PartialEq`). | unit: fold twice, assert_eq |
| REQ-002 | WHEN a cursor'd catch-up re-delivers an already-applied suffix of the stream, re-folding it shall leave the snapshot unchanged (idempotent overlap replay). | unit: `fold(fold(S, es), es) == fold(S, es)` across all six event kinds |
| REQ-003 | WHEN `QuestionRaised` is applied, the seat shall atomically hold `state == Waiting` AND the question. | unit: raise from Working, assert both |
| REQ-004 | WHEN a `StateChange` away from Waiting is applied to a seat holding a question, the question shall become `None` (the invariant question ⇒ Waiting holds after every event). | unit: raise → StateChange(Working), assert question None; property-style sweep over event sequences |
| REQ-005 | WHEN `StateChange(Waiting)` alone is applied, the question shall remain `None` (Waiting-without-question is legal); WHEN `QuestionCleared` is applied, the question shall clear while `state` stays Waiting. | unit: two cases |
| REQ-006 | WHEN `Heartbeat` is applied, only `last_event_ms` shall change, and an older-`ts_ms` heartbeat shall not regress it (`max` join). | unit: heartbeat forward + backward, assert state/question/labels untouched |
| REQ-007 | WHEN a non-`Upsert` event names an unknown id, the reducer shall auto-vivify a placeholder seat (`title = id`, empty labels, `transport = None`) rather than drop the event. | unit: StateChange on empty snapshot |
| REQ-008 | WHEN `Ended` is applied, the seat shall become `Done` — UNLESS its state is `Error`, which shall be retained — with any question cleared and the seat kept in the snapshot. | unit: Ended-from-Working → Done; Ended-from-Error → Error; question cleared |
| REQ-009 | WHEN seats are first mentioned in stream order, the snapshot shall list them in that first-seen order regardless of id sort order. | unit: upsert "z" then "a", assert order z, a |
| REQ-010 | WHEN attention is derived with an injected `now_ms`, a non-Done seat with `now_ms − last_event_ms ≥ stale_after_ms` shall flag Stale, a seat one millisecond under shall not, and a Done seat shall never flag Stale. | unit: boundary at threshold, threshold−1, and Done (both boundary orientations per PR-claude-two-comparison-overlap-needs-boundary-per-side) |
| REQ-011 | WHEN multiple seats need attention, the attention set shall order Error seats before Waiting-with-question seats before Stale seats, ties in snapshot order, each seat at most once at its highest reason. | unit: mixed fleet incl. a seat both Error and stale |
| REQ-012 | WHEN a delivery-state observation greater than the current state arrives, the machine shall advance to it (including a skip, Deposited→Started); WHEN an observation ≤ current arrives, the state shall not change and the result shall report not-advanced. | unit: every ordered pair of the 3×3 transition table |
| REQ-013 | WHEN a `Session` (each state, question present and absent, each transport including `None`), a `SessionEvent` of each kind, and each verb request/receipt type round-trips through serde_json, the deserialized value shall equal the original. | unit: round-trip suite |
| REQ-014 | The crate shall contain no Forge-specific vocabulary, no clock read, and no dependency beyond serde (dev: serde_json, mutants). | review + manifest inspection (D1/D13); negative grep for `SystemTime`/`Instant`/`now()` in src |

## Testing boundary (honest)
The entire crate is pure — no masked shim, no adapter module, no I/O — so the cov/MSI 100 bar applies
to every line with no exclusions. Mutant hygiene notes carried in from prior runs: run `cargo mutants
--list` on the ACTUAL code before claiming the set (the guard-mutant syntactic-form lesson); enum-return
fns without `Default` derives yield unviable body mutants — keep the guard tests regardless.

## Phase Plan
- **P2 Design** — the exact struct/enum definitions (field lists for the six event kinds + the verb
  request/receipt types, settling D-OPEN-READ-RANGE and D-OPEN-STALE-DEFAULT); module layout inside
  `crates/marley_fleet` (proposal in the notes: `session` / `reducer` / `attention` / `dispatch` /
  `verbs`); the serde casing/shape spec; the full test plan mapped to REQ rows.
- **P3 Implement** — the crate per design; workspace member addition only (`crates/*` glob already
  covers it).
- **P3.5 Inspect** — independent critics vs the diff: invariant holes (an event sequence that leaves
  question ≠ None outside Waiting), replay non-idempotence, any Forge vocabulary leak (D1), serde
  schema regressions.
- **P4 Validate** — write + RUN the REQ-named tests; `cargo mutants` on the crate; gate green at
  cov/MSI 100.
- **P5 Complete** — CHANGELOG + architecture docs (orchestration-shell §7 crate row → "shipped";
  envelope v1 hardening recorded, incl. the D7 `Option<Transport>` divergence); archive; AAR; close #367.
- **Train note** — ticket ① of the Layer-1 train (orchestration-shell §12): NOTHING depends on it yet;
  #368 (adapter projection), #369 (fleet rail), #370 (MCP read-slice) consume its types, so the
  envelope decisions locked here are load-bearing for the rest of the sprint.
