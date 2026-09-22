---
pipeline_id: c7587c3f-8b63-4816-9085-84992abcab43
ticket: docs/planning/tickets/open/TICKET-320-lsp-doc-sync-race-proof.md
status: Phase 5 — Complete PASS
title: LSP doc-sync — race-proof text materialization by construction + app-level Ready (fake_ls) test lane
type: chore
milestone: M20
references:
  - docs/planning/knowledge/prevention-rules.md (PR-claude-two-gated-calls-must-read-the-phase-once-001, PR-claude-pump-materialize-hot-data-only-when-consumed-001, PR-claude-synthetic-response-tests-never-prove-the-live-wire-001)
  - docs/planning/knowledge/failures.md (BF-lsp-didopen-carries-empty-text-ready-race-001)
  - docs/planning/pipeline/completed/309-lsp-document-sync.notes.md
  - docs/planning/pipeline/completed/312-lsp-goto-definition.notes.md
  - docs/zed_architecture/subsystems/05-lsp-language-intelligence.md
---

## Title
Make the #309 didOpen-empty-text bug class impossible by construction, and add the
app-level test lane that would have caught it. #312 fixed the shipped bug with a
minimal ORDERING change (drain before collect) — correct today, silently
re-breakable by any refactor that moves the collect back. This pipeline collapses
the two Ready-gated reads (`needs_text` in the pump, the Ready gate in
`reconcile`) into ONE call that decides AND materializes, deletes `needs_text`,
and adds a headless drive against the real `fake_ls` child process that asserts
the PAYLOAD of the captured outbound didOpen — the lane every M20 pipeline lacked
(all prior "driven proofs" injected responses into a process-less host that never
reached Ready).

## Scope
### In
- **Structural fix:** `reconcile` becomes the single Ready-gated read — it takes
  the open set as `(path, version)` plus a text SOURCE (closure or `&Buffer`;
  Phase 2 decides the form), and materializes text ONLY on the branches that
  actually send (didOpen / didChange). The pump stops pre-collecting text.
- **Delete `needs_text`** — with materialization inside `reconcile` it becomes
  unwired API (the #311-F3 rule); its decision table already lives in `reconcile`.
- **`fake_ls` bridge:** make the #308 fixture reachable from `marley_app`'s test
  lane (today only `marley_lsp`'s own integration tests can spawn it via
  `env!("CARGO_BIN_EXE_fake_ls")`, which does not resolve cross-crate).
- **The regression test:** boot a headless workspace with a `.rs` file open,
  spawn a host against `fake_ls`, pump until the host reaches `Phase::Ready`
  through a REAL `drain()` of a real child's initialize response, and assert the
  captured outbound didOpen carries the buffer's REAL text (`TEXT_LEN > 0` AND
  equal to the buffer content). Payload, not method name.
- **`#[cfg(test)]` capture hook** on the host's `send_body`
  (`sent_bodies_for_test()`) so the lane can observe outbound frames.
- **Standing guard docs:** document the `[[lsp.servers]]` tee-wrapper wire-capture
  trick in `scripts/selftest/README.md`, plus the diagnostic tell: a SAME-FILE
  definition returning empty means the far side has no document — read what you
  SENT before suspecting the parser.

### Out (explicitly deferred)
- No wire-protocol change: didOpen/didChange/didSave/didClose semantics, full-text
  sync mode, and the versioning scheme are untouched.
- No incremental (range-based) didChange — full-text sync stays (its own future
  ticket).
- No new LSP features and no `marley_lsp` protocol-layer work beyond what the
  bridge needs.
- Re-fixing the #312 ordering is a non-goal — the ordering fix already shipped;
  this pipeline removes the possibility of needing it.
- The live rust-analyzer tee capture stays a manual diagnostic procedure (docs),
  not a CI lane — CI uses `fake_ls`.

## Reference (§20)
**Zed (the editor reference — same-gpui-stack), behavior-level.**
`docs/zed_architecture/subsystems/05-lsp-language-intelligence.md` §3.2 "Document
sync": on buffer open/edit/save the store sends `textDocument/didOpen` /
`didChange` / `didSave` / `didClose`, with edits keyed to a document version the
client and server must agree on. Marley already matches that BEHAVIOR (#309's
pump reconcile); this ticket changes nothing on the wire — it hardens WHERE the
document content is materialized so a didOpen can never carry an empty document
for a non-empty buffer. The protocol contract being protected is LSP 3.17
`textDocument/didOpen`: `TextDocumentItem.text` is the document's FULL content —
an empty string for a non-empty file is a silent protocol lie no server can
detect or recover from (observed consequence: rust-analyzer answers null to every
request; wire-proven in BF-lsp-didopen-carries-empty-text-ready-race-001).

### Prior art
1. **Behavior maps** — `docs/zed_architecture/subsystems/05-lsp-language-intelligence.md`
   §3.2 (cited above): didOpen/didChange keyed to an agreed document version;
   §3.1's spawn flow confirms the register-on-open shape Marley's pump reconcile
   mirrors at behavior level. No Warp analog (Warp has no LSP editor train).
2. **Published material** — the LSP 3.17 spec, `textDocument/didOpen` /
   `didChange`: didOpen carries the full text and MUST NOT be sent twice without
   a close; didChange carries `version` + (for full sync) the full new text.
   This is the contract REQ-001 asserts at the payload level.
3. **Our permissive deps** — `lsp-types` 0.97 (MIT) is ALREADY shipped in
   `marley_lsp`, deliberately confined to the diagnostics parse seam (#310
   decision; #309's inspect PROV row records the doc-sync builders as
   hand-shaped-by-design, no lsp-types). It owns protocol TYPES, not the client
   reconcile/decision seam — no crate in our tree (no tower-lsp / async-lsp)
   owns doc-sync client logic, so the structural fix has no owner to adopt from.
   Checked gpui (UI framework — no LSP surface) and the rest of the ships-today
   list (ropey / regex / alacritty_terminal / tree-sitter): no owner. The one
   adoptable piece — typing the didOpen params via lsp-types — is explicitly out
   of scope here (it would widen #310's confinement decision for zero risk
   reduction on this bug class).

## React-first (parity)
N/A — no UI delta: this is LSP pump/host internals plus a test lane and a
`scripts/selftest` doc note. Nothing the user sees changes — no chrome, overlay,
pane surface, layout, type, color, or affordance is touched. (`marley-web` has no
LSP transport at all; the parity contract's port map has no cell for this seam.)

## Locked-In Decisions
- **D1 — One phase read, by construction.** `reconcile` is the ONLY Ready-gated
  read on the sync path: it receives `(path, version)` pairs plus a text source
  and materializes text INSIDE, on exactly the branches that send (didOpen /
  didChange). No pre-rendered `String` crosses the call boundary, so no second
  gate can disagree with the first (PR-claude-two-gated-calls-must-read-the-phase-once-001).
- **D2 — `needs_text` is DELETED**, not deprecated or kept-for-tests: once
  materialization lives inside `reconcile` it is unwired API (the #311-F3 rule),
  and it is the only reason the pump ever duplicated reconcile's decision table.
- **D3 — The hot path survives.** Idle ticks (synced == version, no send branch)
  materialize NOTHING — the #309-S2 optimization is preserved by construction,
  now provable with a counting text-source in a unit test
  (PR-claude-pump-materialize-hot-data-only-when-consumed-001).
- **D4 — Assert the payload, never the method.** The regression test asserts the
  captured didOpen's text equals the buffer content and is non-empty; the shipped
  bug carried a perfectly correct method name
  (PR-claude-synthetic-response-tests-never-prove-the-live-wire-001).
- **D5 — The new lane reaches Ready through the REAL path**: a spawned `fake_ls`
  child whose initialize response is drained by the host — `push_response_for_test`
  is banned in this lane (a process-less host never reaching Ready is exactly how
  the bug class stayed invisible).
- **D6 — The capture hook is `#[cfg(test)]`-only** (`sent_bodies_for_test()` on
  the host's `send_body`) — zero release-code cost, no runtime flag.
- **Open fork for Phase 2 (deliberately not locked):** the text-source parameter's
  form — `impl Fn(&Path) -> String` closure vs `&Buffer`-bearing entries. The
  host is app-side (`marley_app` already depends on the editor crate), so both
  are dependency-legal; Design picks with the seam map in hand (testability of
  the counting-source vs API surface).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method (gate exit code,
negative smoke, or review).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a host reaches `Phase::Ready` on a pump tick (its initialize response drained that same tick) with a non-empty buffer open, the system shall send a didOpen whose text equals the buffer's full current content | the new fake_ls app-lane regression test: captured outbound didOpen payload compared byte-for-byte to the buffer; `TEXT_LEN > 0` asserted; runs in gate:3 (`cargo nextest`) |
| REQ-002 | WHILE a document's synced version equals its buffer version (no send branch taken), the system shall not materialize the buffer text on that tick | unit test: reconcile driven with a COUNTING text source over an idle tick asserts zero materializations |
| REQ-003 | WHEN reconcile takes a send branch (didOpen or didChange), the system shall materialize the text via its own single call's text source — the host shall expose no `needs_text` (or equivalent pre-collection) API | compile surface: `needs_text` absent (grep + `cargo build`, its one caller removed); the decision table is DRIVE-proven via REQ-001 + REQ-002 — `lsp_host.rs` is coverage-excluded + `mutants::skip` by standing gate posture (shim; notes Discovery) |
| REQ-004 | WHEN the app-level lane boots a headless workspace with a `.rs` file open against a spawned `fake_ls`, the system shall reach `Phase::Ready` via a real `drain()` of the child's initialize response (no injected responses in this lane) | the new regression test reaches Ready with `push_response_for_test` unused (review + the test's own Ready assertion) |
| REQ-005 | WHEN the host sends any body under `#[cfg(test)]`, the system shall record it retrievably via `sent_bodies_for_test()` | unit test: send under test-cfg, read the capture back |
| REQ-006 | `scripts/selftest/README.md` shall document the `[[lsp.servers]]` tee-wrapper wire-capture procedure and the same-file-definition-empty diagnostic tell | review; gate:14 (docs) green |

## Phase Plan
- **P2 Design** — settle the text-source form (closure vs `&Buffer`) with the seam
  map; the fake_ls cross-crate bridge choice (move the lane into `marley_lsp/tests/`
  vs resolve the bin path from `current_exe()` vs a test-only spawn hook); the
  capture-hook shape; the regression test plan per REQ. Plan-time constraints to
  honor (notes Discovery): the `text_of` borrow shape needs a disjoint field
  binding before `lsp_hosts.get_mut` (E0499/E0502); the capture sits BEFORE
  `send_body`'s `if let Some(handle)` so process-less hosts record too;
  `ensure_lsp_host_for_open_docs`'s path feed + the #397 `sort_by` survive the
  open-set reshape.
- **P3 Implement** — reconcile signature change + pump simplification + `needs_text`
  deletion + bridge + capture hook + README note.
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings.
- **P4 Validate** — write + RUN tests; gate green (`--diff`).
- **P5 Complete** — archive, ledger capture (§19), close the ticket (BACKLOG row
  already removed at promotion).
