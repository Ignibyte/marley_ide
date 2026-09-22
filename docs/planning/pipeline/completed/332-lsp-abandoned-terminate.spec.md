---
pipeline_id: 6f65d651-12a3-4bf1-b8ff-183c9aea8f7d
ticket: docs/planning/tickets/open/TICKET-332-lsp-abandoned-request-terminate.md
status: Phase 5 — Complete PASS
title: LSP — an abandoned request must terminate (typed Abandoned signal per consumer)
type: chore
milestone: M21
references:
  - docs/planning/pipeline/completed/331-inlay-hints.notes.md
  - docs/zed_architecture/crates/lsp.md
  - docs/zed_architecture/subsystems/05-lsp-language-intelligence.md
---

## Title
Every LSP request terminates: both abandonment paths — timeout expire
(`lsp_host.rs:339-354`, removes the purpose and delivers nothing) and
`on_connection_lost` (`lsp_host.rs:365`, clears `purposes` + `responses`) —
gain a typed Abandoned outcome delivered once per dropped purpose through the
normal drain, with an explicit per-consumer arm. Closes the general hole #331's
inspect filed (F6b): today a consumer's apply fn never runs on abandonment, so
its per-feature latch never clears; every event-driven consumer tolerates this
by accident, and the next poll-driven consumer re-discovers the wedge.
`REQUEST_TIMEOUT_TICKS`'s own comment — "a slow server never wedges a
consumer" — becomes true by construction instead of per-consumer luck.

## Scope
### In
- A typed abandonment outcome in the response queue (exact shape is Design's
  D-fork; carrying `reason: Timeout | Disconnected`), delivered through
  `take_responses` / `consume_lsp_responses` like any answer.
- Expire path: one Abandoned (Timeout) per dropped purpose.
- Connection-lost path: one Abandoned (Disconnected) per purpose still pending
  at the loss, with already-queued outcomes KEEPING their delivery (amended at
  inspect F1: the old `responses.clear()` itself ate signals — a dying server's
  final answers arrive in the same drained batch as the disconnect, and their
  purposes have already left the map, so clearing destroyed their only
  terminal outcome; consumer key-guards own staleness).
- An explicit Abandoned arm in all 12 `RequestPurpose` consumers
  (`app.rs consume_lsp_responses` match, `app.rs:12393-12430`): latch clears,
  no semantic-error UI.
- Tests: pure-crate unit coverage for the delivery logic + headless proof that
  a wedged latch clears and re-sends.

### Out (explicitly deferred)
- `$/cancelRequest` emission — already ships: expire sends `build_cancel(id)`
  (`lsp_host.rs:348`).
- Mapping server-sent cancel codes (`REQUEST_CANCELLED -32800` /
  `CONTENT_MODIFIED -32801`) of live, un-expired error replies onto Abandoned —
  real server errors keep today's per-consumer Err behavior.
- Retiring `has_pending_references` — it draw-gates the occluding "Finding
  references…" card (`app.rs:14978`, `app.rs:10654`); any retirement is an
  explicit Design call, default keep.
- Formatting's 2 s save deadline (`check_format_save_deadline`, `app.rs:9349`)
  — already abandons client-side with an honest flash; unchanged.

## Reference (§20)
Zed (the editor — same-gpui-stack reference; research via our behavior map,
never its GPL source). Zed's request model resolves every request future to a
typed `ConnectionResult::{Result, ConnectionReset, Timeout}` — a distinct
terminal outcome per abandonment path, never an error cosplay
(docs/zed_architecture/crates/lsp.md §"The request / response / cancellation
model"; docs/zed_architecture/subsystems/05-lsp-language-intelligence.md).
Marley's tick/pump architecture (purposes table + responses queue, no futures)
reimplements the SHAPE: every request terminates with exactly one typed
outcome — `Answered(Ok | rpc Err)`, or `Abandoned(Timeout | Disconnected)`.

### Prior art
1. **Behavior maps** — docs/zed_architecture/crates/lsp.md documents the typed
   `ConnectionResult` outcome + `$/cancelRequest`-on-drop; direct precedent for
   the Abandoned shape (and for keeping cancel emission, which Marley's expire
   already does via `build_cancel`).
2. **Published** — the LSP spec itself types cancellation distinctly from
   semantic failure: `error_codes::REQUEST_CANCELLED (-32800)`,
   `CONTENT_MODIFIED (-32801)` (read in our shipped `lsp-types-0.97.0`
   source, `src/error_codes.rs:44-48` — adoption, outside the wall).
   "Abandoned ≠ error" is the protocol's own stance.
3. **Our permissive deps** — no crate we ship owns the seam: `marley_lsp` is
   Marley-original plumbing (hand-rolled `serde_json::Value` protocol;
   `lsp-types` confined to the diagnostics parse seam,
   `marley_lsp/src/diagnostics.rs:39-43`). Nothing to adopt beyond the shape;
   no locked decision dissolved.

## React-first (parity)
N/A — no UI delta: no new surface, zone, layout, or affordance. The change
REMOVES phantom semantic toasts on abandonment (rename / code-action /
definition / references arms) and at most routes an honest, timeout-specific
string through the existing status-flash mechanism if Design elects one —
same Flash surface, same chrome, string-only. Nothing to prototype in React.

## Locked-In Decisions
- D1 — The abandonment signal is a DISTINCT typed outcome, never a synthetic
  `RpcError` pushed down the Err lane (#331 F6b: rename/code-action/resolve Err
  arms flash semantic errors; PR-claude-critic-shared-fix-safety-claim-needs-
  per-consumer-check-001).
- D2 — Both abandonment paths deliver exactly one outcome per dropped purpose,
  and the teardown DELIVERS rather than destroys (amended at inspect F1): the
  outcome queue is preserved across `on_connection_lost` — exactly-once holds
  structurally because an outcome enters the queue only by moving its purpose
  out of the map — and every still-pending purpose gains one
  Abandoned(Disconnected). No path may drop a purpose without delivering, and
  no path may destroy a queued outcome.
- D3 — Every consumer handles Abandoned explicitly. No consumer presents a
  semantic-error affordance for abandonment. Rename/code-action family: silent
  latch clear, or an honest timeout-specific message — Design picks per
  consumer (default silent).
- D4 — `has_pending_inlay` stays as belt-and-braces for the poll-driven
  consumer (PR-claude-poll-driven-inflight-check-must-read-the-owner-001);
  retiring either host query is Design's explicit call with the
  references-card draw gate named.
- D5 — Late responses for an already-expired id remain dropped
  (`lsp_host.rs:451-453` `pending.resolve` guard) — regression-locked.
- D6 — The delivery/decision logic lands where gate:5 can kill mutants:
  `lsp_host.rs` is a coverage-excluded, mutation-skipped shim
  (`scripts/gates.sh:229`) and must STAY thin; the testable logic (outcome
  type, expire→deliver, clear→deliver ordering) belongs to the pure surface
  (`marley_lsp` or an equivalently gated seam) per Design's manifest.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a pending request reaches `REQUEST_TIMEOUT_TICKS`, the host shall deliver exactly one typed Abandoned outcome (reason Timeout) for that request's purpose through the normal response drain | pure unit test: register → expire tick → drain yields exactly one Abandoned(Timeout) with the right purpose; second drain yields nothing |
| REQ-002 | WHEN the server connection is lost, the host shall deliver exactly one terminal outcome per in-flight request: outcomes already queued (an answered-then-died request's Answered, an earlier Timeout Abandoned) keep their delivery, and every purpose still pending delivers one Abandoned(Disconnected) — no request delivers zero or two | host-level test: two pending purposes + one queued Answered → after loss the drain yields that Answered plus exactly two Abandoned(Disconnected); a second drain yields nothing |
| REQ-003 | WHEN a consumer receives an Abandoned outcome, the system shall not present any semantic-error affordance ("Cannot rename this", "Rename failed", "Code actions failed", "Code action resolve failed", "No definition found", "No references found") | headless tests on the rename / code-action / resolve / definition / references arms: drive request → abandon → assert no such flash |
| REQ-004 | WHEN a consumer receives an Abandoned outcome, its per-feature pending latch shall clear such that the feature's next trigger sends a fresh request | headless test: request in flight (latch Some) → abandon → latch None → re-trigger observes a new sent request |
| REQ-005 | WHEN an in-flight inlay request is abandoned, the poll-driven refresh shall re-send within one timeout window | headless inlay wedge test (belt-and-braces: Abandoned arm + `has_pending_inlay`) |
| REQ-006 | WHEN a response arrives for an already-expired id, the host shall drop it without delivering anything to any consumer | pure unit test on the resolve guard (regression, `lsp_host.rs:451-453`) |

## Phase Plan
- **P2 Design** — the outcome-type shape fork (queue element type vs a third
  lane; where the type lives per D6); the 12-arm consumer table (per-arm:
  latch, UI, silent-vs-honest-flash per D3); the `WorkspaceSymbol` fan-in arm
  (its latch is deliberately not cleared on success — `app.rs:12864`); test
  manifest (pure unit + headless_drive additions, incl. the missing
  expire→re-request proof).
- **P3 Implement** — per Design's manifest; shim stays thin (D6).
- **P3.5 Inspect** — independent critics vs the diff; per-consumer blast-radius
  check is MANDATORY (the ticket exists because a "siblings are unaffected"
  claim was false).
- **P4 Validate** — write + RUN tests; `scripts/gates.sh --diff` green.
- **P5 Complete** — archive, ledger capture (§19), close TICKET-332.
