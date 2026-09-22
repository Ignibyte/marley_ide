# 332 — LSP: an abandoned request must terminate — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-332-lsp-abandoned-request-terminate.md
- **Pipeline spec:** 332-lsp-abandoned-terminate.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan

- **Request:** `/work` next item → TICKET-332 (top of Queue). The #331 inspect
  follow-up (F6b): both abandonment paths in `lsp_host.rs` — timeout `expire`
  (`purposes.remove(&id)`) and `on_connection_lost` (`purposes.clear()` +
  `responses.clear()`) — drop a request's purpose without delivering anything,
  so the consumer's apply fn never runs and its per-feature latch never clears.
  Build the general fix: a typed Abandoned terminal signal delivered once per
  dropped purpose, with an explicit per-consumer arm.
- **Classification / tier:** chore (plumbing hardening), single work pipeline,
  one shippable slice. Systems: `marley_lsp` host + the editor-side consumers.
- **Recall (§18.3):**
  - #331 notes F6/F6b — the filing inspect. F6 fixed inlay the contained way
    (`LspHost::has_pending_inlay()` — the pending table is the one source of
    truth). F6b REJECTED the critic's synthetic-`Err` shortcut: read per-consumer,
    `prepare_rename` Err → "Cannot rename this" flash, `code_action` Err →
    "Code actions failed", `resolve` Err → "Code action resolve failed" — a
    synthetic Err fires those toasts ~10 s after the keypress on three shipped
    features (#322/#323). The general fix must be a DISTINCT terminal signal.
  - PR-claude-poll-driven-inflight-check-must-read-the-owner-001 — poll-driven
    consumers wedge permanently on a dropped purpose; event-driven ones self-heal
    by minting fresh keys. Corollaries: a timeout comment's promise is a contract;
    a destructive `take()` of a refresh flag upstream of a skip swallows the
    healing signal.
  - PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001 — a
    shared-plumbing fix's "siblings are unaffected" claim is the least-verified
    sentence in any report; check every consumer arm before accepting. This
    pipeline exists because that check was done.
- **Prior-art sweep (§20):**
  - **Zed behavior map** (docs/zed_architecture/crates/lsp.md §"request /
    response / cancellation model"; subsystems/05): Zed's request future resolves
    to a typed `ConnectionResult::{Result, ConnectionReset, Timeout}` — a
    distinct terminal outcome per abandonment path, never an error cosplay; plus
    `$/cancelRequest`-on-drop for abandoned as-you-type requests. Marley's
    tick/pump architecture differs (purposes table + responses queue), but the
    outcome SHAPE maps 1:1 onto `Abandoned { Timeout | Disconnected }`.
  - **Published (LSP spec + lsp-types 0.97 source — adoption, outside the wall):**
    the protocol itself types cancellation distinctly from semantic failure —
    `error_codes::REQUEST_CANCELLED (-32800)`, `CONTENT_MODIFIED (-32801)`
    (lsp-types-0.97.0/src/error_codes.rs:44-48). Confirms "cancelled/abandoned ≠
    error" is the protocol's own stance, not an invention.
  - **Deps:** no crate we ship owns the seam — `marley_lsp` is Marley-original
    plumbing (hand-rolled over `lsp-types` serde types). Nothing to adopt beyond
    the shape above; nothing dissolved.
- **Discovery (Explore seam map, §18.2):**
  - **Host:** `crates/marley_app/src/lsp_host.rs` (974 lines; package `marley`,
    lib `marley_app`). `purposes: HashMap<i64, RequestPurpose>` (:137);
    `responses: Vec<(RequestPurpose, Result<Value, RpcError>)>` (:139;
    `RpcError` = `marley_lsp/src/rpc.rs:39`). Send+register: `request()` :548
    (Ready-gated :554 — returns None pre-Ready, nothing registered). Delivery:
    `on_message` :448 → `purposes.remove(&id)` + `responses.push` :467-471.
    Drain: `take_responses` :568 (`mem::take`). Dispatch:
    `consume_lsp_responses` `app.rs:12383` — flat 12-arm match :12393-12430,
    "ONE drain, routed by purpose; each arm owns its stale-guard". Pump order
    per 16 ms tick: `host.drain()` app.rs:1923 → consume :1990 → inlay refresh
    :2038.
  - **Timeout:** `REQUEST_TIMEOUT_TICKS = ticks_for_ms(10_000, 16)` = 625
    ticks ≈ 10 s (:76-77, comment "a slow server never wedges a consumer").
    Expire in `drain` :339-354: `pending.expire(tick)` → per id
    `purposes.remove(&id)` + `send_body(build_cancel(id))` — NOTHING pushed to
    `responses`. Init expire special case → `on_connection_lost`. Pure expire:
    `marley_lsp/src/rpc.rs:161` (ascending ids). Late-answer drop: :451-453
    `pending.resolve(id).is_none() → return` (before purposes lookup).
  - **on_connection_lost** :365 — clears handle, init_req_id, pending, docs,
    diagnostics, `purposes.clear()`, `responses.clear()`, then
    `step(ProcessExited)`. Callers: decode-error :329, EOF :336,
    handshake-timeout :353, initialize-Err :465. Host drop (`:794`, via
    `app.rs:8720` close_project_at) kills everything with the host.
  - **12 consumers** (apply fns in app.rs, all `#[cfg_attr(test, mutants::skip)]`):
    | Purpose | Apply (app.rs) | Latch | Err arm |
    |---|---|---|---|
    | Hover | 12440 | hover_request=None :12449 + hover_anchor.take() | silent (Err→no markdown) |
    | Definition | 12551 | definition_request=None :12560 | Err→vec![] → flash "No definition found" |
    | References | 12690 | references_request=None :12699 | Err→vec![] → flash "No references found" |
    | Completion | 14211 | completion_request=None :14220 | silent (return false) |
    | PrepareRename | 11711 | rename_request=None :11725/:11733 | Err→declined → flash "Cannot rename this" |
    | Rename | 11788 | rename_request=None :11797 | Err→ flash "Rename failed" |
    | CodeAction | 12016 | code_action_request=None :12025 | Err→ flash "Code actions failed" |
    | CodeActionResolve | 12096 | code_action_resolve_request=None :12105 | Err→ flash "Code action resolve failed" |
    | SignatureHelp | 12234 | signature_request=None :12243 | silent (card=None) |
    | WorkspaceSymbol | 12864 | NONE — latch deliberately kept (multi-host fan-in merge) | silent (keep shown rows) |
    | InlayHints | 4907 | inlay_request=None :4916 | silent, no cache write |
    | Formatting | 9164 | formatting_request=None :9173 | Err→ settle_pending_save :9264 (parked save still writes), no toast |
    Guard asymmetry: PrepareRename checks supersede BEFORE clearing (:11719);
    the others clear right after their supersede check.
  - **#331 fixes in place:** `has_pending_inlay` `lsp_host.rs:651` (called
    app.rs:4860, skip gate :4875); `has_pending_references` :662 (draw-gates
    the occluding card, app.rs:14978 + webview mirror :10654).
  - **Supersede-style latch clears** (still-in-flight answer later dropped by
    the arm's own guard — not purpose removal): close_transient_overlays
    :7325, inlay toggles/refresh guards (4693/4795/4808/4818/4825), disk
    reload :9024, format begin/deadline :9321/:9349 (2 s deadline flashes
    "Formatter timed out — saved"), signature Esc :12314, symbols Esc/Enter
    :12908/:12917, dismiss_hover :14485. Plus per-arm generation guards
    (version/uri/nonce/caret checks).
  - **Tests + gate posture:** `lsp_host.rs` is a coverage-excluded,
    mutation-skipped shim (`scripts/gates.sh:229`); its 3 same-file tests are
    reconcile/DocText only. Pure timeout test: `marley_lsp/src/rpc.rs:436`
    `pending_expire_deadline_boundary`. Consumer-arm tests: headless gpui in
    `marley_app/src/headless_drive.rs` (245 tests; e.g. inlay stale-reply
    :5775, references superseded/empty :9625). Host test hooks:
    ingest_message_for_test :720, push_response_for_test :728,
    set_ready_for_test :753, sent_bodies_for_test :763. Wire tests:
    `marley_lsp/tests/integration.rs` (none exercise REQUEST_TIMEOUT_TICKS).
    **GAP:** no test drives expire → purpose dropped → re-request. Design's
    manifest must decide where the new delivery logic lives so gate:5 bites
    (D6: the shim stays thin).
- **Decisions:** carried into the spec (D1–D4): typed Abandoned (not Err), one
  delivery per dropped purpose on both paths, per-consumer explicit arms,
  push-after-clear ordering on `on_connection_lost`.

## Phase 2 — Design

### Architecture (settles the spec's D-fork)

**D-SHAPE — a NEW enum wrapping the old payload, owned by `marley_lsp`:**

```rust
// marley_lsp/src/rpc.rs (pure; #![deny(missing_docs)] — doc every item)
pub enum AbandonReason { Timeout, Disconnected }          // Copy+Clone+Debug+PartialEq+Eq
pub enum RequestOutcome {                                  // Clone+Debug+PartialEq
    Answered(Result<Value, RpcError>),
    Abandoned(AbandonReason),
}
pub fn abandon_ids<P>(purposes: &mut HashMap<i64, P>, ids: &[i64], reason: AbandonReason)
    -> Vec<(P, RequestOutcome)>   // one outcome per id that HAD a purpose; ids without → nothing
pub fn abandon_all<P>(purposes: &mut HashMap<i64, P>, reason: AbandonReason)
    -> Vec<(P, RequestOutcome)>   // drains everything, ascending by id (deterministic)
```

Why a new enum and not a variant inside `RpcError`/the Err lane: every existing
consumer matches `Ok/Err(_)` — a new failure REPRESENTATION inside the existing
`Result` keeps all those catch-alls compiling and silently routes abandonment
into the semantic-error paths (the F6b bug, now invisible). Replacing the
queue's element type breaks every consumer AT COMPILE TIME and forces the
explicit per-arm decision D3 demands. Why not a second queue: the dispatch
comment (`app.rs:12389`) is law — "ONE drain, routed by purpose"; a second
queue re-creates cross-queue ordering hazards.

Why generic `<P>`: `RequestPurpose` is `pub(crate)` in `marley_app` with
app-side key types; the pure crate owns the MECHANISM (remove + pair with
outcome), the shim supplies the purpose type. `abandon_all` sorts ids before
delegating to `abandon_ids` — HashMap iteration order never reaches behavior.

**Shim wiring (`lsp_host.rs` — stays thin, D6):**
- `responses: Vec<(RequestPurpose, RequestOutcome)>`; `on_message` wraps the
  delivered result: `push((purpose, RequestOutcome::Answered(result)))`.
- `drain` expire block: send `build_cancel(id)` per expired id (unchanged),
  then `if init_expired { self.on_connection_lost() } else {
  self.responses.extend(abandon_ids(&mut self.purposes, &expired, Timeout)) }`.
  The init branch no longer pre-removes expired purposes — `on_connection_lost`
  sweeps EVERY pending purpose as Disconnected (robust even though init pending
  ⇒ pre-Ready ⇒ `purposes` empty, since `request()` is Ready-gated at :554).
- `on_connection_lost`: replace `purposes.clear()` + `responses.clear()` with
  **clear-then-extend**: `responses.clear();
  responses.extend(abandon_all(&mut purposes, Disconnected))` (D2 ordering —
  the clear must not eat the signals). Rest of the teardown unchanged.
- `take_responses` return type follows. Test hooks: `push_response_for_test`
  KEEPS its `Result<Value, RpcError>` signature (wraps `Answered` internally —
  zero churn across the existing headless tests); new
  `push_abandoned_for_test(purpose, reason)`.

**Consumer arms (`app.rs` — the 12-arm table, D3 settled):**
Each apply fn's signature changes `Result<Value, RpcError>` → `RequestOutcome`.
The Abandoned arm slots in AFTER the existing supersede/stale guard (an
abandoned answer for a superseded key is dropped exactly like a live answer —
the guard's job is unchanged) and participates in the latch clear at its
existing position:

| Arm | Abandoned behavior | Return |
|---|---|---|
| Hover | markdown=None (fold into the existing `Answered(Err(_)) \| Abandoned(_)` match arm — same diagnostics-only card path) | as today's Err |
| Definition | latch cleared; NO flash (early return before the 0-targets flash) | false |
| References | latch cleared; NO flash; return true so the occluding card (draw-gated on `has_pending_references`, now false) repaints away THIS frame | true |
| Completion | latch cleared; silent | false |
| PrepareRename | clear `rename_request` in the arm (this fn's clears are arm-specific); NO flash | false |
| Rename | latch cleared; honest flash `"Language server didn't respond"` (a committed action — silence reads as success) | true |
| CodeAction | latch cleared; silent (menu never opened; a 10 s-late toast is noise) | false |
| CodeActionResolve | latch cleared; honest flash `"Language server didn't respond"` (user CHOSE the action; its edit never applies) | true |
| SignatureHelp | latch cleared; card=None (same as Err's silent dismiss) | true |
| WorkspaceSymbol | silent, keep shown rows; latch DELIBERATELY untouched (fan-in stale-guard key, not a send-skip — per-keystroke re-query self-heals; verify no `symbol_request.is_some()` send-skip exists at implement) | false |
| InlayHints | latch cleared; no cache write; `has_pending_inlay` stays (D4 belt-and-braces) | false |
| Formatting | latch cleared; `return self.settle_pending_save(&key)` — the parked save STILL WRITES (same as Err; the 2 s deadline is the outer belt) | settle's |

One new flash string, shared by Rename + CodeActionResolve:
`"Language server didn't respond"` — honest, timeout-specific, NOT the semantic
error. Reason-agnostic (Timeout vs Disconnected read the same to the user;
server state has its own segment UI).

**§20 confirm:** Reference stands as planned — Zed's typed
`ConnectionResult::{Result, ConnectionReset, Timeout}` (behavior map only,
docs/zed_architecture/crates/lsp.md; never GPL source). This design matches the
SHAPE 1:1: `Answered(Ok|Err)` ≙ `Result`, `Abandoned(Disconnected)` ≙
`ConnectionReset`, `Abandoned(Timeout)` ≙ `Timeout`. React-first stays N/A
(string-only delta through existing Flash chrome).

**§14 check:** pure total fns (no panics, no unwrap on reachable paths); single
owner for the shared type (`marley_lsp::rpc`, re-exported flat in lib.rs, never
re-declared); no IO, no spawns.

**Reviewed-and-benign:** host Drop via `close_project_at` (`app.rs:8720`) kills
purposes without delivery — the project's editors/latches die with it; the
poll-driven inlay path requires a live host+editor for the root, so no wedge is
reachable. Out of scope, recorded here.

### File manifest
1. `crates/marley_lsp/src/rpc.rs` — add `AbandonReason`, `RequestOutcome`,
   `abandon_ids`, `abandon_all` (+ doc comments) + unit tests.
2. `crates/marley_lsp/src/lib.rs` — extend the `pub use rpc::{…}` re-export.
3. `crates/marley_app/src/lsp_host.rs` — `responses` element type; `on_message`
   `Answered` wrap; `drain` expire branch; `on_connection_lost`
   clear-then-extend; `take_responses` type; `push_abandoned_for_test`; 3 new
   same-file wiring tests (REQ-001/002/006 host level).
4. `crates/marley_app/src/app.rs` — `consume_lsp_responses` passes
   `RequestOutcome`; 12 apply fns per the arm table; the shared honest-flash
   string.
5. `crates/marley_app/src/headless_drive.rs` — new consumer-arm tests
   (REQ-003/004/005); zero churn to existing tests (wrapper hook).

### Regression Test Plan

| AC | Test (location) | Proves |
|---|---|---|
| REQ-001 | `abandon_ids_removes_and_pairs` (rpc.rs unit) | one outcome per id-with-purpose, right reason, map entry gone, ids without purpose yield nothing; kills `->vec![]` / reason-swap / no-remove mutants (assert len + contents + map size) |
| REQ-001 | `expired_request_delivers_exactly_one_abandoned_timeout` (lsp_host same-file: set_ready → request → drain ×(TIMEOUT+1) → take_responses) | the WIRING: expire → exactly one `Abandoned(Timeout)` with the right purpose; second take empty; `$/cancelRequest` still sent (sent_bodies) |
| REQ-002 | `abandon_all_drains_ascending` (rpc.rs unit) | every purpose delivered once, ascending ids, map emptied |
| REQ-002 | `connection_lost_delivers_abandoned_per_pending_after_clear` (lsp_host same-file: 2 requests + 1 queued stale answer → private on_connection_lost) | clear-then-extend ordering: drain yields exactly 2 `Abandoned(Disconnected)`, zero stale `Answered` |
| REQ-003 | `rename_family_abandoned_no_semantic_flash_headless` (headless_drive; one test per arm: prepare-rename / rename / code-action / resolve / definition / references via push_abandoned_for_test) | no "Cannot rename this" / "Rename failed" / "Code actions failed" / "Code action resolve failed" / "No definition found" / "No references found"; rename+resolve assert the honest string INSTEAD; references asserts the occluding card un-draws |
| REQ-004 | `abandoned_latch_clears_and_retrigger_resends_headless` (headless_drive, hover or definition representative + latch asserts inside each REQ-003 test) | latch None after abandon; re-trigger produces a fresh send (sent bodies grow) |
| REQ-005 | `inlay_abandoned_resends_within_window_headless` (headless_drive) | poll-driven refresh re-sends after abandonment (arm + `has_pending_inlay` belt-and-braces) |
| REQ-006 | `late_response_after_expire_is_dropped` (lsp_host same-file: request → expire → ingest_message_for_test the late answer) | drain holds ONLY the Abandoned; no Answered ever appears (`pending.resolve` guard regression) |
| derives | extend rpc.rs `derives_exercised` for the two new enums | Clone/Debug/PartialEq/Copy live in every profile |

- **trybuild:** none — the contract IS enum exhaustiveness, compiler-native; no
  newtype boundary is added. (Documented per §7 rather than silently skipped.)
- **visual/AX (gate:15):** no `visual_acceptance` clause — React-first N/A.
- **Uncoverable:** none new. `lsp_host.rs` additions ride the existing
  documented shim exclusion (`scripts/gates.sh:229`); its same-file tests RUN
  under gate:3 regardless. The MSI-bearing surface (the two pure fns + enums)
  lands in `marley_lsp` at 100%.
- **Mutation posture:** apply fns keep their existing
  `#[cfg_attr(test, mutants::skip)]` (headless-verified shims — posture
  unchanged from #311–#331).

### Risks / decisions
- R1 **init-expire ordering**: solved structurally — the init_expired branch
  routes EVERYTHING through `on_connection_lost`'s sweep (no Timeout push to be
  eaten by its `responses.clear()`).
- R2 **HashMap nondeterminism**: `abandon_all` sorts ids; tests assert order.
- R3 **12-fn signature churn**: wide but compiler-forced; no arm can be
  forgotten (a non-exhaustive match is a compile error).
- R4 **New user-visible string** (Rename/Resolve honest flash): string-only via
  existing Flash chrome; parity N/A holds.
- R5 **Zero-churn hook wrapper**: `push_response_for_test` wraps `Answered` so
  the existing headless suite is untouched.
- R6 **References repaint**: the Abandoned arm returns true so the
  `has_pending_references` draw-gate re-evaluates THIS frame — without
  delivery, the occluding card would linger until an unrelated repaint (the
  observable proof that delivery, not just latch hygiene, matters).

## Phase 3 — Implement

- **React-first: N/A** (spec section: no UI delta — the change removes phantom
  toasts; the one new string rides the existing Flash chrome).
- **Built exactly to the manifest:**
  1. `marley_lsp/src/rpc.rs` — `AbandonReason` (Copy enum), `RequestOutcome`
     (`Answered(Result<Value, RpcError>) | Abandoned(AbandonReason)`),
     `abandon_ids<P>` (order follows `ids`; ids without a purpose yield
     nothing), `abandon_all<P>` (sorts ids, delegates). Placed above
     `ReplyPolicy`; all doc-commented (`deny(missing_docs)`).
  2. `marley_lsp/src/lib.rs` — flat re-export extended.
  3. `marley_app/src/lsp_host.rs` — `responses:
     Vec<(RequestPurpose, RequestOutcome)>`; `on_message` wraps `Answered`;
     `drain`'s expire block now sends the cancels then BRANCHES
     (init_expired → `on_connection_lost()` sweep; else
     `abandon_ids(…, Timeout)` extend); `on_connection_lost` does
     clear-then-extend with `abandon_all(…, Disconnected)`;
     `take_responses` retyped; `push_response_for_test` wraps `Answered`
     (existing suite untouched); new `push_abandoned_for_test`.
  4. `marley_app/src/app.rs` — dispatch loop passes `outcome`; all 12 apply
     fns retyped + explicit Abandoned arms exactly per the Phase 2 table
     (silent latch-clears; honest `"Language server didn't respond"` on
     Rename + CodeActionResolve; references returns true for the card
     repaint; formatting settles the parked save; symbol keeps rows + latch;
     hover/inlay fold Abandoned into their benign
     `Answered(Err(_)) | Abandoned(_)` arm).
- **Deviations from design:** none behavioral. Two mechanical notes:
  (a) `RpcError` import in lsp_host.rs moved under `#[cfg(test)]` — after the
  retype the wire-error type reaches the shim only via the test hook (the lib
  build no longer names it); (b) transient `dead_code` warning on
  `push_abandoned_for_test` until Phase 4's headless tests call it — expected,
  NOT suppressed (§0: no blanket allows; the callers land at validate).
- **Checks run:** `cargo check --workspace --all-targets` green (5.65 s full,
  1.68 s warm); `cargo fmt --all` applied + `--check` clean.

## Phase 3.5 — Inspect

Three independent critics over the diff (correctness/blast-radius,
state-integrity/ordering, simplification/provenance), each fed the prior
failure classes (BF-claude-poll-driven-request-wedges-on-dropped-purpose-001,
BF-claude-references-searching-card-naive-latch-strands-on-timeout-001,
BF-claude-error-reply-writes-no-cache-so-request-loops-forever-001).

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| F1 | HIGH | `on_connection_lost`'s `responses.clear()` destroys queued terminal outcomes: mpsc yields a dying server's final answers BEFORE Disconnected in one drained batch, and their purposes already left the map at delivery — so the clear erased an answered-then-died request's only signal (committed rename → silence), and a backgrounded root's queued Timeout abandons (found independently by BOTH the correctness and state critics; the state critic added the cross-tick trace) | **REAL** — my trace confirmed: queue-entry and purpose-removal are one move, so purposes∩queue=∅ and keeping the queue cannot double-deliver | Clear REMOVED — teardown now delivers: queue preserved + `abandon_all(Disconnected)` sweep. Spec amended (Scope-In, D2, REQ-002): the plan-time "push after the clear" ordering had encoded the bug as a safety rule. Ledger: F-claude-teardown-clear-eats-queued-terminal-outcomes-001, PR-claude-teardown-must-deliver-not-destroy-the-outcome-queue-001 |
| F2 | MED | Shared `rename_request` latch across PrepareRename + Rename purposes with a generation-free key: a superseded prepare's Abandoned (10 s window) can pass the guard against a newer COMMITTED rename's identical key, clear its latch, and the real rename answer then guard-fails → no edit, no flash. (#332 added the silent lane; the Answered-lane confusion pre-existed) | **REAL** — reachable via double-F2 + prepare answered once + Enter within the second prepare's window | `prepare_rename_request` own slot (the F-CORR-2 code-action-resolve precedent): begin_rename/open_rename_draft/apply_prepare_rename_response use the new slot; confirm_rename/apply_rename_response keep `rename_request`. Ledger: F-claude-shared-latch-cross-purpose-abandon-clobber-001 |
| F3 | MED | Doc comments assert the pre-#332 world: `has_pending_inlay`/`has_pending_references` ("delivers no response… waits forever") + the references draw-gate comment in app.rs (both critics flagged independently) | **REAL** (doc-only) | All three rewritten: the queries are the D4 belt; abandonment now delivers |
| F4 | MED | `take_responses` drains only the ACTIVE root → non-active hosts' outcomes defer unboundedly (late honest flash at switch-back; symbol fan-in structurally incomplete for non-active roots — pre-existing) | **REAL, CONTAINED** — the destruction half died with F1 (nothing is lost anymore, only deferred); the deferred-flash-at-switch-back is honest and contextual (strictly better than the pre-existing deferred SEMANTIC toasts); the full fix changes when all 12 arms fire for non-active roots — a per-consumer sweep of its own | **TICKET-413 filed** with the blast radius named (per PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001), backlog row added |
| F5 | LOW | `on_message` body loop has no break after an initialize-Err `on_connection_lost`; later bodies of the dead generation still route (responses are resolve-gated ✓; a publishDiagnostics writes into the fresh store; an inlay-refresh sets one spurious latch) | **PRE-EXISTING, benign** — the diff didn't touch the loop; recorded, not fixed (out of scope) | none (noted for a future lifecycle ticket) |
| F6 | LOW | `LspHost::drop` (workspace close) delivers nothing and clears no app latches — benign today only by belts (deadline settle, host-backed gates, next-gesture overwrites) | **PRE-EXISTING, benign-by-belts** — reviewed as designed-out in Phase 2 ("reviewed-and-benign") | none; the design note stands |
| F7 | LOW | Comment mis-alignment (rustfmt trailing-glue) in `on_connection_lost` | **REAL** (cosmetic) | Dissolved by the F1 rewrite (comment repositioned full-width) |

**Validate checklist from the simplification critic** (mutation reality): the
only viable diff-mode mutants are `abandon_ids → vec![]` / `abandon_all →
vec![]` — sort-removal, remove-elision, and reason-swap are NOT representable
as mutants, so the unit asserts must pin them: exact non-empty contents + map
emptied + second-call-empty + absent-id-yields-nothing; the ascending-order
assert needs ≥5 ids (2 ids pass a removed sort ~50% of runs — flaky pin); the
host-level tests must assert the REASON (Timeout on the expire path,
Disconnected on teardown — the only surface discriminating the lanes); extend
`derives_exercised` for both new enums.

**Post-fix verification:** `cargo check --workspace --all-targets` green;
`cargo fmt --all --check` clean; `rename_request` sites audited (prepare slot:
begin/draft-open/apply-prepare ×4 clears; rename slot: confirm/apply/test-hook
— exactly as designed).

## Phase 4 — Validate

- **Tests added (10 new):**
  - `marley_lsp/src/rpc.rs` (the MSI surface): `abandon_ids_removes_and_pairs`
    (REQ-001 unit: exact contents kill `->vec![]`; absent-id, map-emptied,
    second-call-empty pins), `abandon_all_drains_ascending` (REQ-002 unit: SIX
    ids so a dropped sort can't pass by map-order luck; reason on every
    outcome; map emptied; second sweep empty), + `derives_exercised` extended
    for both new enums (incl. `Answered ≠ Abandoned` distinguishability).
  - `marley_app/src/lsp_host.rs` (same-file wiring, runs under gate:3):
    `expired_request_delivers_exactly_one_abandoned_timeout` (REQ-001: real
    `drain()` ticks to expiry → exactly one Abandoned(TIMEOUT) — the reason
    discriminates the lane — second take empty, `$/cancelRequest` still sent),
    `connection_lost_preserves_queue_and_sweeps_pending` (REQ-002 as amended:
    real `ingest` of the answered-then-died reply + private
    `on_connection_lost()` → the Answered survives + exactly one
    Abandoned(DISCONNECTED) for the pending purpose, nothing twice),
    `late_response_after_expire_is_dropped` (REQ-006).
  - `marley_app/src/headless_drive.rs` (gpui, the REAL consume path):
    `lsp_abandoned_prepare_and_definition_end_quietly_headless` (REQ-003/004),
    `lsp_abandoned_rename_and_resolve_flash_honest_headless` (REQ-003: the
    exact honest string, by name, on both committed-action arms),
    `lsp_abandoned_remaining_arms_silent_headless` (REQ-003: code-action /
    references / signature / completion / formatting — five arms, zero
    flashes, latches cleared, cards/menus not stuck),
    `lsp_abandoned_inlay_real_expire_resends_headless` (REQ-001/004/005
    end-to-end through the GENUINE expire lane: real mint+send → 626 real host
    ticks → Abandoned(Timeout) through the normal drain → latch cleared → next
    poll re-sends; no synthetic push anywhere).
  - New `#[cfg(test)]` hooks (house idiom): `drive_abandoned_for_test`,
    `drive_prepare_rename_for_test`, `drain_lsp_ticks_for_test`, five latch
    accessors. `push_abandoned_for_test`'s transient dead_code warning from
    Phase 3 is resolved (callers landed).
- **REQ-004 note:** the latch-clear asserts live in every arm test; the RESEND
  half is proven on the one consumer with a send-skip (inlay, the real-expire
  test). The event-driven arms have NO send-skip (verified at design/inspect:
  every gesture sends unconditionally and overwrites its latch), so
  latch-clear + the existing supersede suites carry the clause for them.
- **Suites RUN (real output):** `cargo nextest run --workspace` →
  **2091 tests run: 2091 passed, 5 skipped** (14.1 s; includes all 10 new).
  `cargo test --workspace --doc` → 18 suites, all `test result: ok`, 0 failed.
- **Live-app drive:** N/A — no `visual_acceptance` clause; the ticket's visible
  delta is the ABSENCE of phantom toasts plus one honest string through the
  long-shipped Flash chrome. The triggering condition (a server that accepts a
  request and then never answers for 10 s / dies mid-flight) has no
  deterministic live-app reproduction without wedging a real rust-analyzer;
  the headless suite drives the identical consume path end-to-end, and the
  inlay test exercises the genuine expire lane tick-for-tick. Stated per the
  §7 "never silently skip" rule.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — 15 passed,
  0 failed (gate:1 rustfmt, :2 clippy -D warnings, :3 nextest+doctests, :7
  audit, :8 deny, :9 machete, :10 gitleaks, :11 shellcheck, :12
  no-suppressions, :13 SAST, :14 docs, :4 coverage ≥100% lines, :5 mutation
  MSI ≥100%, :6 miri, :15 visual/AX). Exit 0; the commit receipt is written.
- **Pre-existing failures:** none — the suite was green before and after.

## Phase 5 — Complete

- **§21 docs:** CHANGELOG.md entry added (Unreleased → Changed — the full what/why
  incl. the inspect amendments and TICKET-413). `docs/marley_architecture/editor.md`
  updated in both LSP passages: the purpose-tagged path now documents the
  `RequestOutcome` terminal contract + delivery-preserving teardown; the
  poll-vs-event latch passage flips #332 from "carries the fix" to SHIPPED with the
  belt (D4) status. Parity sync: N/A — no UI delta (spec section stands).
- **Ledger appends this pipeline (codes):**
  - Design: `L-claude-new-terminal-state-new-enum-not-err-variant-001` (lessons.md)
  - Inspect: `F-claude-teardown-clear-eats-queued-terminal-outcomes-001`,
    `F-claude-shared-latch-cross-purpose-abandon-clobber-001` (failures.md);
    `PR-claude-teardown-must-deliver-not-destroy-the-outcome-queue-001`
    (prevention-rules.md)
  - Complete: `AD-claude-lsp-request-outcome-typed-terminal-enum-001`
    (architecture-decisions.md)
- **Ticket:** TICKET-332 closed (open → closed, status set); backlog row left at
  promotion (Phase 1); TICKET-413 (inspect follow-up) remains open with its Queue row.
- **Archive:** doc pair → `docs/planning/pipeline/completed/`.
