# marley_fleet — the generic Session envelope + FleetSnapshot reducer — Notes

- **Forge ticket:** #367 d8e06631-eb00-4f8e-be7d-f2ce484be23b
- **AAR:** f1f217d1-e5e4-4f25-ae72-fb8c51a2e0f1
- **Local ticket doc:** docs/planning/tickets/open/TICKET-367-marley-fleet-envelope.md
- **Pipeline spec:** 367-marley-fleet-envelope.spec.md

## Phase 1 — Plan
- **Request:** Draft the Phase-1 spec for the M23 Layer-1 opener — a NEW pure crate `marley_fleet`
  carrying the generic `Session` envelope (hardened from the orchestration-shell §3 v1 proposal), the
  `FleetSnapshot` + deterministic replay-safe reducer, staleness/attention derivation, the
  Deposited→Claimed→Started delivery-state types, and the session-verb request/receipt types as data.
  Types + pure fold only; no gpui, no transport, no Forge vocabulary.
- **Classification / tier:** work pipeline — feature; a NEW pure crate (the cleanest tier: no app-code
  surface, no masked shim, cov/MSI 100 with zero exclusions). Sprint "M23 — Fleet Control Plane:
  Layer 1", ticket ① of the five-ticket train.
- **Forge recall (§18.3):** drafted docs-only (no MCP calls this session — per the drafting charter);
  recall satisfied from the on-disk design record, which IS the ratified source for this ticket:
  [orchestration-shell.md](../../../marley_architecture/orchestration-shell.md) §2 (mechanism-not-policy;
  the "Forge string in marley_fleet is a defect" check), §3 (the envelope v1 proposal this ticket
  hardens), §7 (the crate map row: pure, "one seam, three consumers"), §12 (Layer-1 sequencing);
  [fleet-control-plane.md](../../../marley_architecture/fleet-control-plane.md) §2 (the evidence-night
  incident table — every field decision cites it), §6 (the `seat_events` wire contract + derived
  rules); [detached-sessions.md](../../../marley_architecture/detached-sessions.md) (transport/ownership
  framing; "Date::now stays banned" purity doctrine). Known prevention rules honored in the spec:
  PR-claude-two-comparison-overlap-needs-boundary-per-side (REQ-010's dual boundary),
  PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators + the guard-mutant syntactic-form
  lesson (Testing boundary). Formal knowledge-search/bulletins re-run at /work promotion pre-flight.
- **Discovery:**
  - **Proposed crate layout** — `crates/marley_fleet/` (the `crates/*` workspace glob auto-includes it):
    - `src/lib.rs` — crate doc + re-exports.
    - `src/session.rs` — `Session`, `State`, `Question`, `Transport` (the envelope).
    - `src/reducer.rs` — `SessionEvent`, `FleetSnapshot`, the fold.
    - `src/attention.rs` — staleness + attention derivation (pure, injected `now_ms`).
    - `src/dispatch.rs` — the delivery-state machine types.
    - `src/verbs.rs` — the verb request/receipt types.
    - `Cargo.toml` — deps `serde` (derive); dev-deps `serde_json`, `mutants` (mirrors
      `marley_forge_client/Cargo.toml`). Exact module split is Phase 2's to confirm; the seam
      boundaries above are the locked shape.
  - **Consumers (the "one seam, three consumers" doctrine cashing out):** ① the UI model — #369's
    fleet rail renders `FleetSnapshot` + attention directly; ② the test surface — #368's adapter
    projection tests assert against these types; ③ the MCP tool schema — #370 serializes the envelope
    over `marley_mcp`. Plus `marley_forge_client` (#368) as the CONSTRUCTOR of `SessionEvent`s (its
    existing pure-parse/masked-socket split extends naturally; the RFC3339→millis conversion lands in
    its pure layer).
  - **Prior-art sweep findings (details + file:line evidence in the spec's `### Prior art`):** no
    Warp/Zed behavior analog (genuinely N/A); MCP-spec leg informs flat-JSON schema shape only; the
    deps leg settled four things — timestamps as `u64` epoch-millis (NO date crate exists anywhere in
    the workspace), clock-injection as the established idiom (marley_lsp `now_tick: u64`, marley_agent
    "`Date::now` is banned", notify.rs fed-in ticks), `labels` as `BTreeMap<String, String>`
    (workflows.rs:37 precedent), and the Cargo shape (forge_client's serde-only manifest).
- **Decisions:** D1–D13 locked in the spec with evidence; the load-bearing ones — generic-only
  vocabulary (D1), u64 epoch-millis + adapter-owned RFC3339 parse (D2), no clock in the crate /
  read-time attention (D3), BTreeMap labels (D4), flat `Option<Question>` + one-directional
  reducer-enforced invariant (D5), idempotent max-join fold (D6), auto-vivify unknown ids ⇒
  `transport: Option<Transport>` (D7 — a deliberate hardening divergence from the §3 sketch's required
  field), Ended never launders Error (D8), first-seen order (D9), Error > Question > Stale attention
  rank (D10), monotone-join delivery (D11), verbs as data with Accepted/Refused receipts (D12).
  Two genuinely open forks flagged for Phase 2: D-OPEN-READ-RANGE (session.read's range shape),
  D-OPEN-STALE-DEFAULT (whether/what `DEFAULT_STALE_AFTER_MS` const).

## Phase 2 — Design

### Architecture / approach
A NEW leaf crate `crates/marley_fleet` — pure data types + a pure fold, at the very bottom of the
fleet dependency stack (nothing in the workspace depends on it yet; #368 adapter, #369 rail, #370 MCP
server will). No gpui, no transport, no I/O, no Forge vocabulary (D1). §14 honored trivially: there
are no panics-on-input paths (the reducer never panics — unknown ids auto-vivify, D7; no `unwrap`),
no IO to make testable, no process-spawn. The crate is the "one seam, three consumers" doctrine
(spec §Title): the same structs are the UI model, the test surface, and the MCP tool schema, so every
type derives `Serialize + Deserialize`.

**§20 confirmed N/A — Marley-specific.** Fleet orchestration of detached agent seats has no Warp/Zed
behavior analog (re-confirmed the maps at plan: Warp cockpit = single-session blocks, Zed collab =
CRDT buffer-sharing). The behavior source is our own design record (fleet-control-plane.md §2 incident
table + orchestration-shell.md §3 envelope). Only our permissive-dep idioms were adopted: serde
(8-crate workspace idiom), `BTreeMap<String,String>` (workflows.rs:37), u64-epoch-millis +
injected-`now` (marley_lsp/src/rpc.rs:161 `now_tick: u64`; marley_agent lib.rs:94 "`Date::now` is
banned"). No copyleft source consulted.

### Exact type definitions (the design the implementer writes to)

**`src/session.rs`** — the envelope:
- `enum State { Starting, Working, Idle, Waiting, Error, Done }` — `#[derive(Debug, Clone, Copy,
  PartialEq, Eq, Serialize, Deserialize)] #[serde(rename_all = "lowercase")]`. Closed vocabulary;
  Error ≠ Idle first-class.
- `enum Transport { Tmux, Bridge, Local }` — same derives + `rename_all = "lowercase"`. Render hint.
- `struct Question { prompt: String, options: Vec<String>, context_refs: Vec<String> }` — `#[derive(Debug,
  Clone, PartialEq, Eq, Serialize, Deserialize)]`.
- `struct Session { id: String, title: String, state: State, question: Option<Question>, labels:
  BTreeMap<String, String>, last_event_ms: u64, transport: Option<Transport> }` — same derives.
  Serde hygiene for a clean tool schema + round-trip: `#[serde(default, skip_serializing_if =
  "Option::is_none")]` on `question` and `transport`; `#[serde(default, skip_serializing_if =
  "BTreeMap::is_empty")]` on `labels`. `id/title/state/last_event_ms` are always present. (`default`
  makes deserialize fill absent fields; `skip_serializing_if` keeps the wire tidy — round-trip
  equality holds because default⇄skip are inverses.)

**`src/reducer.rs`** — the stream + fold:
- `enum SessionEvent` — `#[serde(tag = "kind", rename_all = "snake_case")]`, every arm carrying
  `id: String, ts_ms: u64`:
  - `Upsert { id, ts_ms, title: String, state: State, #[serde(default)] labels: BTreeMap<String,String>,
    #[serde(default)] transport: Option<Transport> }`
  - `StateChange { id, ts_ms, state: State }`
  - `QuestionRaised { id, ts_ms, question: Question }`
  - `QuestionCleared { id, ts_ms }`
  - `Heartbeat { id, ts_ms }`
  - `Ended { id, ts_ms }`
  - `impl SessionEvent { pub fn id(&self) -> &str; pub fn ts_ms(&self) -> u64 }` (match all arms).
- `struct FleetSnapshot { seats: Vec<Session> }` — `#[derive(Debug, Clone, Default, PartialEq, Eq,
  Serialize, Deserialize)]`. **`seats` is private** to protect the first-seen-order invariant (D9);
  accessors `pub fn seats(&self) -> &[Session]` and `pub fn get(&self, id: &str) -> Option<&Session>`.
  (Deserialize can reconstruct an arbitrary-order snapshot for transport — acceptable; the reducer
  maintains order when folding the live stream.)
- `pub fn apply(snapshot: &mut FleetSnapshot, event: &SessionEvent)` — find seat by `event.id()`
  (linear scan; fleets are small); if absent, push a placeholder (Upsert fills it; non-Upsert
  auto-vivifies `Session { id, title = id.clone(), state: Starting, question: None, labels: empty,
  last_event_ms: 0, transport: None }`, D7). Then match the event:
  - **Upsert**: set `title/state/labels/transport`; if `state != Waiting` → `question = None`.
  - **StateChange**: set `state`; if `state != Waiting` → `question = None`.
  - **QuestionRaised**: `state = Waiting; question = Some(q)`.
  - **QuestionCleared**: `question = None` (state untouched — the seat's own next event moves state).
  - **Heartbeat**: nothing but the timestamp join below.
  - **Ended**: `question = None; state = if state == Error { Error } else { Done }` (D8 — Ended never
    launders Error).
  - **Always, last**: `seat.last_event_ms = seat.last_event_ms.max(event.ts_ms())` (monotone max-join,
    D6 — tolerant of minor clock skew, older re-deliveries don't regress).
- `pub fn reduce(mut snapshot: FleetSnapshot, events: &[SessionEvent]) -> FleetSnapshot` — folds
  `apply` over the slice; returns the snapshot. Idempotent under whole-suffix overlap replay (D6):
  every arm is idempotent w.r.t. the post-fold state (verified per-arm in the design review below).

**`src/attention.rs`** — read-time derivation (no clock; D3):
- `pub fn is_stale(session: &Session, now_ms: u64, stale_after_ms: u64) -> bool` =
  `session.state != State::Done && now_ms.saturating_sub(session.last_event_ms) >= stale_after_ms`.
  (`saturating_sub` guards a now < last_event skew without a panic; `>=` is the threshold boundary.)
- `enum AttentionReason { Error, Question, Stale }` — priority order = declaration order (Error highest).
- `struct Attention<'a> { session: &'a Session, reason: AttentionReason }`.
- `fn reason_for(s: &Session, now_ms, stale_after_ms) -> Option<AttentionReason>`: `Error` if
  `state == Error`; else `Question` if `state == Waiting && question.is_some()`; else `Stale` if
  `is_stale`; else `None`. (Each seat's single HIGHEST reason.)
- `pub fn attention<'a>(snapshot: &'a FleetSnapshot, now_ms, stale_after_ms) -> Vec<Attention<'a>>`:
  map seats (snapshot order) through `reason_for`, collect the Some, then `sort_by_key(|a|
  rank(a.reason))` where `rank: Error=0, Question=1, Stale=2`. **`slice::sort_by_key` is a STABLE
  sort** → within-reason snapshot order preserved (D10). Result: all Errors, then all Questions, then
  all Stales, each in first-seen order, each seat once.

**`src/dispatch.rs`** — the delivery machine (D11):
- `enum DeliveryState { Deposited, Claimed, Started }` — `#[derive(Debug, Clone, Copy, PartialEq, Eq,
  PartialOrd, Ord, Serialize, Deserialize)] #[serde(rename_all = "lowercase")]`. Derived `Ord` follows
  declaration order → Deposited < Claimed < Started.
- `struct DeliveryAdvance { state: DeliveryState, advanced: bool }`.
- `impl DeliveryState { pub fn observe(self, observed: DeliveryState) -> DeliveryAdvance }` = if
  `observed > self` → advance (skips legal); else no-op reported `advanced: false`.

**`src/verbs.rs`** — verb request/receipt types as data (D12):
- `struct SendRequest { id: String, text: String }`.
- `enum ReadRange { Tail { lines: u32 }, Lines { start: u64, end: u64 } }` — `#[serde(tag = "mode",
  rename_all = "snake_case")]` (D-OPEN-READ-RANGE settled: line-oriented).
- `struct ReadRequest { id: String, range: ReadRange }`.
- `struct OpenRequest { profile: String }` (opaque profile — policy, not Marley).
- `struct SurfaceRequest { id: String }`.
- `enum Receipt<T> { Accepted { value: T }, Refused { reason: String } }` — `#[serde(tag = "result",
  rename_all = "snake_case")]`. Generic success payload; struct variants so internal tagging works for
  any `T: Serialize + Deserialize`. Refused is first-class (the dialog-up rule). Concrete
  instantiations at Layer 2 (`Receipt<()>` send/surface, `Receipt<Vec<String>>` read, `Receipt<String>`
  open) — all round-trip-tested.

**`src/lib.rs`** — crate doc (the purity charter) + `pub mod` × 5 + flat `pub use` re-exports.

### Design review — the two load-bearing invariants (proven at design time)
1. **question ⇒ Waiting (one-directional, D5):** every arm that sets a non-Waiting state clears the
   question (Upsert, StateChange, Ended); only QuestionRaised sets a question (and sets Waiting
   simultaneously); QuestionCleared removes a question (safe in any state). So after ANY event the
   invariant holds. REQ-004's sweep tests this over generated sequences.
2. **whole-suffix replay idempotence (D6):** each arm is idempotent w.r.t. the state after a full
   fold — Upsert/StateChange/Ended are assignments (re-assign same value), QuestionRaised/Cleared are
   assignments, Heartbeat is a `max` (no-op when ts ≤ current). A cursor'd catch-up replays a suffix
   IN FULL ORDER, so `reduce(reduce(S, es), es) == reduce(S, es)` even when an intermediate event
   (e.g. a QuestionRaised later cleared by a StateChange in the same suffix) transiently diverges —
   the final snapshot re-converges. REQ-002 asserts this across all six kinds.

### File manifest
| File | Change |
|---|---|
| `crates/marley_fleet/Cargo.toml` | NEW — `name = "marley_fleet"`, `edition.workspace`, `license.workspace`; `[dependencies] serde = { version = "1", features = ["derive"] }`; `[dev-dependencies] serde_json = "1"`, `mutants = "0.0.3"` (mirrors marley_forge_client/Cargo.toml exactly) |
| `crates/marley_fleet/src/lib.rs` | NEW — crate doc + `pub mod` × 5 + re-exports |
| `crates/marley_fleet/src/session.rs` | NEW — `State`, `Transport`, `Question`, `Session` + `#[cfg(test)]` |
| `crates/marley_fleet/src/reducer.rs` | NEW — `SessionEvent`, `FleetSnapshot`, `apply`, `reduce` + `#[cfg(test)]` |
| `crates/marley_fleet/src/attention.rs` | NEW — `is_stale`, `AttentionReason`, `Attention`, `attention` + `#[cfg(test)]` |
| `crates/marley_fleet/src/dispatch.rs` | NEW — `DeliveryState`, `DeliveryAdvance`, `observe` + `#[cfg(test)]` |
| `crates/marley_fleet/src/verbs.rs` | NEW — request types, `ReadRange`, `Receipt<T>` + `#[cfg(test)]` |
| workspace | none — `members = ["crates/*"]` auto-includes the new crate (confirmed Cargo.toml) |

### Regression Test Plan (every REQ → a named `#[cfg(test)]` test in the owning module)
| REQ | Test (module) | Asserts |
|---|---|---|
| REQ-001 | `reducer::t367_req001_fold_is_deterministic` | reduce(default, es) twice → assert_eq |
| REQ-002 | `reducer::t367_req002_overlap_replay_is_noop` | reduce(reduce(d,es),es) == reduce(d,es); es covers all 6 kinds |
| REQ-003 | `reducer::t367_req003_question_raised_atomic` | QR from Working → state==Waiting && question==Some |
| REQ-004 | `reducer::t367_req004_statechange_away_clears_question` + `..._invariant_sweep` | raise→SC(Working) ⇒ None; sweep over sequences asserts question⇒Waiting after each |
| REQ-005 | `reducer::t367_req005_waiting_without_question_and_cleared` | SC(Waiting) alone ⇒ question None; QuestionCleared ⇒ None, state stays Waiting |
| REQ-006 | `reducer::t367_req006_heartbeat_max_join` | forward hb advances last_event_ms; backward (older ts) does not; state/question/labels untouched |
| REQ-007 | `reducer::t367_req007_unknown_id_autovivifies` | StateChange on empty snapshot → placeholder (title==id, labels empty, transport None) |
| REQ-008 | `reducer::t367_req008_ended_done_unless_error` | Ended from Working→Done; from Error→Error; question cleared; seat retained |
| REQ-009 | `reducer::t367_req009_first_seen_order` | upsert "z" then "a" → seats order [z, a] |
| REQ-010 | `attention::t367_req010_staleness_boundaries` | at threshold → stale; threshold−1 → not; Done never stale (both `>=` orientations, PR-…-boundary-per-side) |
| REQ-011 | `attention::t367_req011_attention_reason_order` | mixed fleet incl. a seat both Error+stale → Error before Question before Stale, ties in snapshot order, seat once |
| REQ-012 | `dispatch::t367_req012_delivery_monotone_join` | all 9 ordered pairs of the 3×3 table (incl. Deposited→Started skip; ≤ = not-advanced) |
| REQ-013 | `session/reducer/verbs::t367_req013_serde_round_trip_*` | Session (each state, question ±, each transport + None), each SessionEvent kind, each verb request, Receipt<()>/<Vec<String>>/<String> both arms → JSON round-trip eq |
| REQ-014 | validate-step: `rg 'ticket\|phase\|gate\|SystemTime\|Instant\|::now\(' crates/marley_fleet/src` returns no field/import hits + `cat Cargo.toml` | review + manifest + negative grep (not a unit test — honestly noted) |

**No trybuild cases:** the crate exposes plain data + a runtime reducer; the invariants (question⇒Waiting,
replay idempotence, monotone delivery) are runtime properties, not type-level contracts a compile-fail
case could guard. Stated per §7 rather than adding a hollow trybuild file.

**Mutant hygiene (spec Testing boundary):** run `cargo mutants --list -f <file>` per file before
claiming the set — the real targets are the comparison/join operators (`is_stale`'s `>=`, `observe`'s
`>`, the `max` join, `state != Waiting`, `state == Error`) and the `reason_for` branch order. The
boundary tests (REQ-010 dual orientation, REQ-012 all-pairs, REQ-006 forward+backward, REQ-008
Error-vs-not) kill them; keep any guard tests for enum-arm mutants that lack a `Default`-derived
viable form.

### Risks / decisions
- **Private `seats` vs serde:** deserialize can build an out-of-order snapshot (transport only); the
  live reducer owns order. Accepted — encapsulation over a theoretical wire re-sort.
- **Generic `Receipt<T>`:** DRY + Layer-2-ready; verified serde internal-tagging works with struct
  variants for any `T`. No fns on it → zero mutation surface.
- **`skip_serializing_if` on question/labels/transport:** tidy tool schema; round-trip preserved by
  the paired `default`. Both branches (present/absent) covered by REQ-013.
- **`sort_by_key` stability** is load-bearing for D10 within-reason order — documented; it is a stable
  sort by std contract.

## Phase 3 — Implement
- **Built** exactly the Phase-2 manifest — a new leaf crate `crates/marley_fleet`:
  - `Cargo.toml` — serde (derive) dep; dev-deps serde_json + mutants; `edition.workspace`/
    `license.workspace` (byte-mirrors marley_forge_client/Cargo.toml). `members = ["crates/*"]`
    auto-includes it (confirmed via `cargo metadata`).
  - `src/lib.rs` — the purity charter doc + 5 `pub mod` + flat re-exports.
  - `src/session.rs` — `State`/`Transport` (`rename_all = "lowercase"`), `Question`, `Session`
    (question/transport `default` + `skip_serializing_if = "Option::is_none"`; labels `default` +
    `skip_serializing_if = "BTreeMap::is_empty"`).
  - `src/reducer.rs` — `SessionEvent` (`tag = "kind"`, snake_case) with per-field doc comments,
    `id()`/`ts_ms()` accessors, `FleetSnapshot { seats: Vec<Session> }` (private) + `seats()`/`get()`/
    private `index_of()`, `apply()` (auto-vivify + the 6 arms + the trailing max-join), `reduce()`.
  - `src/attention.rs` — `is_stale` (`saturating_sub` + `>=`), `AttentionReason`, `Attention<'a>`,
    private `reason_for`/`rank`, `attention()` (stable `sort_by_key`).
  - `src/dispatch.rs` — `DeliveryState` (derived `Ord` = declaration order), `DeliveryAdvance`,
    `observe()`.
  - `src/verbs.rs` — `SendRequest`, `ReadRange` (`tag = "mode"`), `ReadRequest`, `OpenRequest`,
    `SurfaceRequest`, generic `Receipt<T>` (`tag = "result"`, struct variants so internal tagging works
    for any `T`).
- **No unwrap/expect anywhere**; no clock; no gpui/transport/Forge vocabulary. `cargo fmt --check`
  clean; `cargo check -p marley_fleet` and `cargo check --workspace` both green (the `block v0.1.6`
  future-incompat note is a pre-existing transitive-dep warning, not from this crate).
- **Deviations from design:** none. (Per the phase contract, the `#[cfg(test)]` REQ-001..013 suites are
  written in Phase 4 — Validate, not here; only production code + doc comments landed in Phase 3.)
- **Field-doc note:** every struct-variant field carries a `///` doc — the workspace lints
  doc-completeness on public items, and the tag-enum variants' fields are public. No behavioral effect.

## Phase 3.5 — Inspect
**4 independent general-purpose critics** (parallel, distinct lenses) + **my own independent
verification** (an exhaustive invariant probe, the real `cargo mutants --list`, and D1/D2/D13 greps).
Lenses: correctness/invariants · attention/delivery · serde-schema/agnosticism · simplification/
mutation-readiness.

### Independent verification I ran myself (cross-check, not just critics)
- **Exhaustive invariant probe** (temporary `tests/_inspect_probe.rs`, run green, then REMOVED): D5
  (`question ⇒ Waiting` after every prefix) and D6 (whole-suffix replay idempotence) over all 4096
  length-3 streams from a 16-event alphabet, + D8/D9 spot cases. All green.
- **Real mutant set** via `cargo mutants --list -p marley_fleet` (v27.1.0): **38 mutants — 33 viable,
  5 auto-unviable** (`Default`-body mutants on `Session`/`AttentionReason`/`Attention`/`DeliveryAdvance`
  — none derive `Default`; the spec's viability lesson, confirmed). `FleetSnapshot` DOES derive
  `Default` so `reduce → Default::default()` is viable (must assert non-empty).
- **Greps:** D1 (no Forge/UCSOS code identifier — all 8 vocab hits are doc-comment examples), D2 (no
  clock read), D13 (serde-only manifest). All clean.

### Findings ledger
| # | Lens | Sev | Finding | Verdict | Resolution |
|---|---|---|---|---|---|
| F1 | correctness | Low | `FleetSnapshot`/`Session` `Deserialize` can materialize a D5-violating or dup-id snapshot the reducer won't repair (the types are the wire schema) | REAL but not a fold bug — in L1 the reducer is the SOLE producer; snapshots flow OUTWARD only. An uncalled `sanitize()` would be dead unwired API | **Doc-fix**: documented the reducer as the invariant authority + the trusted-ingest contract on `FleetSnapshot`; a future untrusted-ingest path adds `sanitize` at that seam (integration note for #368/#370) |
| F2 | correctness | Low | `Upsert{Waiting}` preserves (can't clear) a stale question — by design | REAL, D5-consistent | **Doc-fix**: documented on the `Upsert` variant that question lifecycle is ONLY via QuestionRaised/QuestionCleared (adapter contract for #368) |
| F3 | serde | INFO | `Upsert` event emitted `labels:{}`/`transport:null` while `Session` omits them — schema asymmetry | REAL (cosmetic); these types ARE the MCP schema so uniformity matters | **Code-fix**: added `skip_serializing_if` to `Upsert`'s labels/transport, matching `Session` |
| F4 | attention | INFO | `stale_after_ms == 0` flags every non-`Done` seat | REAL (caller-owned threshold, D-OPEN-STALE-DEFAULT) | **Doc-fix**: caveat on `is_stale` that the threshold is caller-owned and 0 is degenerate |
| F5 | attention | INFO | `rank()` decoupled from `AttentionReason` declaration order — a future enum reorder could silently break D10 | REAL latent maintenance coupling | **Doc-fix**: comment tying `rank` to `reason_for`'s priority as the single sort-order source |
| F6 | attention | INFO | bare `Waiting` (no question) invisible to attention until stale | Not a bug — faithful to D10 (only Waiting-WITH-question is first-class) | Recorded; a D10 policy revisit if the surfacing latency ever matters (out of scope) |
| B1 | mutation | Med | Upsert's question-clear guard (reducer.rs:158 `!=`→`==`) SURVIVES the planned tests — REQ-004 exercises StateChange (line 164), not Upsert | REAL test-plan gap (would fail MSI=100) | **Phase-4 test**: `QuestionRaised(id) → Upsert{id, Working} → assert question==None` |
| B2 | mutation | Med | dispatch `>`→`>=` (dispatch.rs:34) survives if a test asserts only `.state` — `X.observe(X)` gives same state, only `.advanced` differs | REAL test-plan gap | **Phase-4 test**: assert `!X.observe(X).advanced` on the 3 diagonal pairs |
| B3 | mutation | Med | attention `&&`→`||` (attention.rs:40) needs a fresh Waiting-no-question seat asserted absent THROUGH `attention()` (REQ-005 is a reducer test) | REAL test-plan gap | **Phase-4 test**: `attention()` on a Waiting/question:None/fresh seat → EMPTY |
| B4 | mutation | Med | `get()` (reducer.rs:114) exercised by no REQ row (kills `None` + `==`→`!=` + line coverage) | REAL test-plan gap | **Phase-4 test**: `get(existing).id==existing` AND `get("nope").is_none()` on a ≥2-seat snapshot |
| B5 | mutation | Med | REQ-010's Done seat must be TIME-STALE (past threshold); REQ-011's fleet must be OUT-OF-ORDER + include a calm seat (else `rank`→0/1 and the Done-clause mutants survive) | REAL test-plan gap | **Phase-4 test**: Done-past-threshold in REQ-010; REQ-011 fleet ordered [Stale,Question,Error] asserting `attention()`==[Error,Question,Stale] + a calm Idle asserted absent |

### Simplification (both declined, with reason)
- `get`/`index_of` share the `s.id == id` predicate — DRYing `get` via `index_of` removes one mutant
  but adds indirection; LEFT as-is (both killable). The duplicated "clear question if != Waiting" guard
  (Upsert/StateChange): critic recommends LEAVE (3 readable lines; factoring couples the arms). Kept.
- API is adequate for #368/#369/#370: `seats() -> &[Session]` gives len/is_empty/iter/index free;
  `attention()` borrow is ergonomic for the rail; no `get_mut` (the reducer owns mutation — protects
  D9). No dead code.

### The Phase-4 kill map (every viable mutant → its killing assertion)
Baked into validate: attention 12 · dispatch 3 · reducer 18 = 33 viable, plus 5 unviable (keep the
guard tests — they kill the viable siblings). Round-trip EVERY enum variant for line coverage
(State×6, Transport×3+None, SessionEvent×6, ReadRange both, Receipt both) and assert a full
`Vec<Attention>` equality in REQ-011 (exercises `Attention`'s serde-less derive line). NOTE: the
`.max()` join (reducer.rs:186) generates NO mutant in 27.1.0 — REQ-006's backward-heartbeat assertion
is coverage/correctness, not a required kill.

### Result
Zero code defects (the reducer's D5–D9 contract is airtight under brute force — critic 1 ran 579k
sequences × 6 seeds, zero violations; my probe agrees). All findings resolved: 1 code-fix (F3
skip_serializing_if) + 4 doc-fixes (F1/F2/F4/F5) applied; B1–B5 are Phase-4 test-shaping requirements
now baked into the plan; F6 recorded as an out-of-scope policy note. `cargo fmt --check` + `cargo check`
green after all edits.

## Phase 4 — Validate
- **Tests added: 25** inline `#[cfg(test)]` across the 5 modules, one+ per EARS row + the inspect
  B1–B5 kill-shapes:
  - `session.rs` (3): REQ-013 Session round-trip over State×6 × Transport(3+None) × question± × labels±;
    lowercase wire; empty-field-omitted-yet-round-trips (F3).
  - `reducer.rs` (13): REQ-001 determinism, REQ-002 replay-idempotence (all 6 kinds), REQ-003 QR
    atomic, REQ-004 StateChange-clears, **B1** Upsert-clears (kills :158) + Upsert-Waiting-preserves,
    REQ-005 Waiting-no-q + QuestionCleared, REQ-006 heartbeat max-join (ts 100/500), REQ-007
    auto-vivify, REQ-008 Ended, REQ-009 first-seen order, **B4** get() hit/miss, REQ-013 event
    round-trip (all kinds + accessors) + snapshot round-trip.
  - `attention.rs` (4): REQ-010 staleness boundaries + Done-never-stale-past-threshold (B5) + skew,
    REQ-011 out-of-order fleet → [Error,Question,Stale] full `Vec<Attention>` eq + calm-seat-absent
    (B5), **B3** fresh-Waiting-no-question empty, stale-Working surfaces.
  - `dispatch.rs` (2): REQ-012 all 9 pairs + **B2** `.advanced` diagonal assertion; DeliveryState
    round-trip.
  - `verbs.rs` (2): REQ-013 verb requests + ReadRange both arms; Receipt both arms × 3 payloads.
- **Runs (real output):**
  - `cargo nextest run -p marley_fleet` → **25 passed, 0 failed**.
  - `cargo test -p marley_fleet --doc` → 0 doctests (no `” ```rust ”` examples; unit + mutation
    coverage is the plan — no hollow doctest added).
  - **`cargo mutants -p marley_fleet` → 38 mutants: 33 caught, 5 unviable** = **MSI 100%**. The 5
    B1–B5 gaps the inspect flagged are all killed; the 5 unviable are the `Default`-body mutants
    auto-excluded.
  - **REQ-014** (validate-step greps): D1 no Forge/UCSOS code identifier (all vocab hits are
    doc-comment/label-string examples), D2 no clock read, D13 serde-only manifest — all clean.
  - **`scripts/gates.sh --diff` → `GATE GREEN [diff]`, 15/15**: rustfmt · clippy -D · tests · audit ·
    deny · machete · gitleaks · shellcheck · no-suppressions · SAST · **docs (fixed 2 ambiguous
    intra-doc links — `attention` is both a fn and a module → `mod@` disambiguator)** · **coverage
    100% lines** · **mutation MSI 100%** · miri · visual/AX. Receipt written.
- **Driven-capture step: N/A (honest).** `marley_fleet` is a PURE library crate with NO UI surface,
  no render path, no input handler — nothing to drive on the running app. gate:15 (visual/AX)
  skip-cleans for it; the `marley_visual_harness` unit asserts cover the harness itself. The rail that
  renders these types is #369, where the driven capture lives.
- **Pre-existing failures:** none introduced; the workspace `block v0.1.6` future-incompat note is a
  pre-existing transitive-dep warning, out of scope.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md Added entry for `marley_fleet`; orchestration-shell.md §7 crate row →
  **SHIPPED (#367)** with the v1 hardening recorded (u64 epoch-millis; flat `Option<Question>`
  one-directional invariant; `transport: Option` auto-vivify divergence; D-OPEN settled — line-oriented
  `ReadRange`, no default staleness const).
- **AAR** `f1f217d1` submitted (completed, effectiveness 5; 1 novel finding → distillation/drift/
  emergence enqueued).
  - **What worked:** the pre-authored Fable spec was accurate end-to-end — D1–D13 held, both D-OPEN
    forks resolved cleanly in design, zero code deviations through implement. The four adversarial
    critics + my own exhaustive probe (579k+ / 4096 sequences) unanimously confirmed the reducer's
    D5–D9 contract is airtight (zero violations).
  - **What the inspect actually bought (the real value):** B1–B5 — **5 viable mutants that survive the
    tests the EARS rows literally describe** (Upsert's guard vs StateChange's; `get()` vs `seats()`;
    `observe()`'s `.advanced` vs `.state`; a Done seat that must be time-stale; an out-of-order fleet).
    Caught only by tracing the REAL `cargo mutants --list` per file and mapping each viable mutant to a
    test on ITS line — not by trusting EARS coverage. → prevention rule `cb0772c2`
    (`PR-claude-ears-behavior-covered-mutant-still-alive-on-sibling-line-001`).
  - **Failures:** none (zero code defects). The only gate red was cosmetic — 2 ambiguous rustdoc
    intra-doc links (`attention` is both a fn and a module), fixed with the `mod@` disambiguator.
  - **Integration contracts recorded for the sprint** (F1/F2, doc-fixed in code): `FleetSnapshot`
    deserialize is trusted-ingest (the reducer is the invariant authority; a `sanitize` seam is added
    only when an untrusted path appears — avoided as dead API now); the question lifecycle is owned only
    by QuestionRaised/QuestionCleared, never `Upsert` — the #368 adapter contract.
- **forge:** ticket #367 closed; prevention rule `cb0772c2` recorded at inspect.
- **Train note:** ticket ① of the M23 Layer-1 fleet train is COMPLETE. #368 (adapter projection),
  #369 (fleet rail), #370 (MCP read-slice) consume these now-shipped types.
