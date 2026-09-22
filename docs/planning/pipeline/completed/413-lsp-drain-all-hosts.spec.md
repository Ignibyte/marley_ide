---
pipeline_id: c2b3974e-ba31-482a-8b0b-a1780eb6abc8
ticket: docs/planning/tickets/open/TICKET-413-lsp-drain-all-hosts-outcomes.md
status: Phase 5 — Complete PASS
title: LSP — drain every host's outcomes per pump tick, route by owning root (#332 inspect follow-up)
type: chore
milestone: M20
references:
  - docs/planning/pipeline/completed/332-lsp-abandoned-terminate.spec.md
  - docs/planning/pipeline/completed/332-lsp-abandoned-terminate.notes.md
  - docs/zed_architecture/subsystems/05-lsp-language-intelligence.md
  - docs/zed_architecture/crates/lsp.md
---

## Title

Every LSP host's queued outcomes drain within one pump tick of arrival, routed
with the OWNING host's root — the active-root-only `consume_lsp_responses` call
dies. Filed from #332 inspect (state critic, F4): today every host `drain()`s
each tick (wire in), but only the active root's host ever `take_responses()`s
(outcomes out), so a backgrounded workspace's Answered AND Abandoned outcomes
defer unboundedly until switch-back — a rename abandonment flashes "Language
server didn't respond" minutes late, and the ⌘T `workspace/symbol` fan-out
(which SENDS to every Ready host) structurally can't merge non-active answers.
Nothing is LOST anymore (#332 made teardown deliver, never destroy) — only
deferred; this pipeline ends the deferral.

The blast radius that makes this a pipeline, not a quick fix: all 12 consumer
arms' stale-guards read ACTIVE editor state (focused uri, live buffer version,
active caret). Draining non-active roots changes WHEN every arm fires — so each
arm gets its own recorded non-active-root decision + test (the per-consumer
sweep IS the work, per
PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001).

## Scope

### In

- `consume_lsp_responses` (app.rs:12672): drain EVERY host's outcome queue each
  pump tick, dispatching each `(purpose, outcome)` with the owning host's root,
  in deterministic order (sorted roots; ids already ascend within a host).
- The pump call site (app.rs:1995): stops being active-root-only; the pump's
  `active_root` stays for reconcile/host-creation (sync policy is out of scope).
- The per-consumer sweep: each of the 12 `RequestPurpose` arms gets an explicit
  non-active-root decision recorded in the Phase 2 per-arm table, and at least
  one test exercising the arm with a non-active owning root.
- ⌘T `workspace/symbol` fan-in completes: answers from every Ready host merge
  into the live query's rows (the merge exists and is root-agnostic — this fix
  un-starves it).

### Out (explicitly deferred)

- Send paths — requests still originate from active-editor gestures + the
  existing fan-out; no new request kinds.
- Document sync (`reconcile`) + host creation (`ensure_lsp_host_for_open_docs`)
  stay active-root — that is didOpen/didChange policy, not outcome delivery.
- No new UI surface: flashes, cards, and picker rows keep their shipped looks;
  only WHEN they fire changes.
- Cross-root focus-stealing policy redesigns (e.g. suppressing a background
  root's flash entirely) — each arm's decision is scoped to "correct with the
  owning root", not to inventing new notification UX.

## Reference (§20)

**Zed** (the editor — same-gpui-stack reference), from our behavior map only
(`docs/zed_architecture/subsystems/05-lsp-language-intelligence.md` §2.1–2.2,
`docs/zed_architecture/crates/lsp.md`): each language server has its own
incoming-message task; a response resolves its awaiting requester ON ARRIVAL,
keyed by request id — delivery is a property of the OWNING server, never of
window/worktree focus, and the server registry is per (worktree × language)
with no "active worktree drains first" anywhere. Marley matches that BEHAVIOR
in its own pump architecture: an outcome delivers on the tick it arrives,
routed by its owning root, regardless of which workspace is active. Clean-room:
behavior map + LSP spec only; Zed source (GPL) never read.

### Prior art

1. **Behavior maps** — `docs/zed_architecture/subsystems/05-lsp-language-intelligence.md`
   §2.1–2.2 (per-server incoming task; responses resolve on arrival,
   focus-independent; `LocalLspStore` = per-(worktree×language) registry) and
   `docs/zed_architecture/crates/lsp.md` (responses routed by id to per-connection
   handlers). Both confirm: no editor defers a non-focused root's responses.
2. **Published** — LSP 3.17 spec: request/response correlation is per-connection
   by id; the protocol has NO focus concept, so "active-root-only delivery" is
   client-invented behavior with no spec backing. Multi-root clients (VS Code,
   observable behavior) fan `workspace/symbol` to all applicable servers and
   merge results as they arrive.
3. **Our permissive deps** — no crate we ship owns this seam. `marley_lsp` is
   Marley's own clean-room client (#308's mpsc + tick pump); `lsp-types` (MIT,
   already adopted) is types-only at the parse seam; gpui has no LSP layer.
   Checked gpui / lsp-types / marley_command: no owner — the drain seam is
   Marley-specific.

## React-first (parity)

N/A — no UI delta: this changes WHEN already-shipped surfaces fire (status
flashes, hover/completion cards, ⌘T picker rows), not how anything looks — no
chrome, overlay, layout, type, color, or affordance changes. The ⌘T picker
gains rows it was always specified to show (merged across Ready hosts) in the
existing row style under the existing render cap.

## Locked-In Decisions

- **D1 — Routing, not filtering.** The drain never drops or defers an outcome
  because its root is non-active. Every queued outcome dispatches on the tick
  it drains, with the owning host's root. Per-arm behavior differences are
  arm-local decisions (D2), never a skip of the drain itself.
- **D2 — The per-consumer sweep is the work.** Every one of the 12
  `RequestPurpose` arms gets an explicit non-active-root decision recorded in
  the Phase 2 per-arm table and at least one test on the non-active-root path
  (PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001: a
  shared-plumbing fix's "siblings are unaffected" is the least-verified claim
  in any report — verify it arm by arm). A "defer this arm's outcome anyway"
  verdict is a legal per-arm outcome IF recorded with its reason; silence is not.
- **D3 — ONE drain site stays law.** `take_responses` is called from exactly
  one consume pass per tick (the app.rs:12678 comment survives); the pass
  extends to every host in deterministic sorted-root order (the #396 AAR
  ordering-surface rule), per-host queue (arrival) order within a host —
  #332's abandonment sweeps ascend by id. *(Inspect hedge: answered outcomes
  keep wire-arrival order; "ids ascend" holds only for the abandonment sweeps.)*
- **D4 — Fan-in un-starved, not redesigned.** `apply_workspace_symbol_response`
  (stale-gen key + extend + cap) is already root-agnostic and merge-shaped;
  delivery is the fix. Design confirms the gen/latch reasoning holds when
  answers arrive across multiple ticks from multiple roots.
- **D5 — Focused-root guard on the focused-editor arms (inspect amendment).**
  The 8 focused-editor arms (hover, definition, references, completion,
  prepare-rename, code-action menu, signature help, inlay) drop an outcome
  whose owning root is not the active project's root BEFORE any latch/UI
  interaction, latch untouched (`outcome_is_for_focused_root`). Reason: uri
  equality is FILE identity, not INSTANCE identity — a NESTED-root twin spells
  the same uri under both hosts, latches survive project switches, and twin
  version counters collide at equal counts (the #354 F3 class); without the
  guard every uri-guarded arm could deliver cross-instance (inspect F1/F2
  HIGH). The committed/global arms (rename, resolve, symbol, formatting) do
  NOT take the guard — background completion is their semantics (D1 intact:
  the drain itself still never filters).
- **D6 — Owning-instance edit routing (inspect amendment; the #354 D12.5
  class).** `apply_one_file` prefers the OWNING root's instance for a twin
  (the version was validated against that root's host) and falls back to the
  all-roots scan only when the owning root holds no instance. Pre-existing gap
  (equal severity pre-#413), fixed here because the drain change put the seam
  in scope.

## Acceptance Criteria (EARS)

One observable behavior per row, with a verification method (gate exit code,
negative smoke, or review).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE two or more LSP hosts hold queued outcomes, WHEN a pump tick runs the consume pass, the app shall empty every host's outcome queue that tick (a second `take_responses` on each host returns empty). | App-level test: two hosts with queued outcomes → one consume pass → both queues empty. |
| REQ-002 | WHEN an outcome drains from a host, the app shall dispatch it to its purpose arm with the OWNING host's root, never the active root. | Test: an outcome queued on a non-active root's host produces its arm's root-dependent effect under the owning root (e.g. a rename edit applies to the owning root's files). |
| REQ-003 | WHEN a non-active root's request expires (timeout) or its server disconnects, the app shall fire the arm's abandonment effect within one pump tick of the outcome being queued, not at switch-back. | Test: rename/resolve request on root A → activate root B → tick to expiry+1 → the flash is present while B is still active. |
| REQ-004 | WHEN `workspace/symbol` answers arrive from multiple Ready hosts for the live query, the picker shall merge every host's rows regardless of which root is active. | Test: fan-out to two Ready hosts → answers from both → `open_symbols.results` contains both hosts' rows. |
| REQ-005 | Each of the 12 `RequestPurpose` arms shall have its non-active-root behavior recorded in the design per-arm table and exercised by at least one test whose outcome's owning root is non-active. | Review at inspect (table complete, 12/12) + the per-arm test list at validate. |
| REQ-006 | WHEN the consume pass runs with no hosts or only empty queues, the app shall report no repaint (return false). | Unit/app test: zero hosts / empty queues → consume returns false. |
| REQ-007 | WHEN outcomes from multiple hosts drain in one tick, the app shall dispatch them in deterministic order (hosts sorted by root; per-host queue/arrival order within a host — #332's abandonment sweeps ascend by id). | Test: two hosts' queued outcomes → dispatch order observed stable across runs (assert exact sequence). |
| REQ-008 | *(inspect, D5)* WHEN a focused-editor arm receives an outcome whose owning root is not the active project's root — including a NESTED-root twin whose uri and version match the focused instance — the arm shall drop it before any latch or UI interaction. | Tests: nested-twin definition answer executes no jump; nested-twin completion answer opens no menu; per-arm cross-root drop tests double as the `outcome_is_for_focused_root` mutation kills. |
| REQ-009 | *(inspect, D6)* WHEN a WorkspaceEdit file is open under multiple roots, the app shall apply its edits to the OWNING root's instance, never the lexicographically-first twin. | Test: rename with the file open under a parent AND the owning nested root → the owning instance's buffer changes, the parent twin's does not. |

## Phase Plan

- **P2 Design** — the per-arm sweep table (12 arms × what the arm's guards read
  from active state × what a non-active-root outcome does today vs after ×
  verdict + test shape); the `consume_lsp_responses` signature/iteration change
  (collect-then-dispatch to satisfy the borrow checker; sorted roots;
  `WorkspaceSymbol`'s rootless arm); confirm the fan-in gen reasoning (D4);
  regression test plan mapping every REQ + arm to a named test.
- **P3 Implement** — per design; Rust only (no React leg — see React-first: N/A).
- **P3.5 Inspect** — independent critics vs the diff (correctness / state
  integrity / provenance / simplification); the per-arm table is the core
  review surface; fix real findings; ledger appends.
- **P4 Validate** — write + RUN the planned tests; `scripts/gates.sh --diff`
  green (receipt written).
- **P5 Complete** — CHANGELOG + architecture docs (editor/LSP section), ledger
  capture (§19), archive the pair, close TICKET-413 (row already left the
  queue at promotion).
