---
pipeline_id: 6263824d-391c-4f99-9182-f1e725b801f2
ticket: forge#308 (9371f921-9dc4-4c9f-bceb-56c5e38ee9e9) · local docs/planning/tickets/open/TICKET-308-lsp-client-core.md
aar_id: 4556dc66-8ed4-4b5c-88d2-e5224609fcaa
status: Phase 5 — Complete PASS
title: LSP client core — rust-analyzer lifecycle + JSON-RPC framing + initialize handshake
type: feature
milestone: M20
references: [docs/zed_architecture/subsystems/05-lsp-language-intelligence.md, docs/marley_architecture/roadmap.md]
---

## Title
**The M20 foundation: bring up the LSP wire.** A new `marley_lsp` crate — Content-Length framing,
JSON-RPC request/response/notification routing, and a lifecycle state machine — plus the
rust-analyzer spawn shim (settings-configured), driven to a completed `initialize` handshake and a
status-bar readout. NO features (no diagnostics/hover/completions — those are #310-317); this ticket
is the transport every one of them speaks through.

## Scope

### In
- **`crates/marley_lsp`** (new crate):
  - PURE framing codec: `encode_frame` + an incremental `FrameDecoder` (split/coalesced reads).
  - PURE JSON-RPC layer: monotonic request-id allocation, `route(incoming)` →
    Response/Notification/ServerRequest, pending-request table with per-request timeout +
    `$/cancelRequest` on drop/abandon.
  - PURE lifecycle machine: `Starting → Initializing → Ready → Crashed(backoff) → GaveUp/Shutdown`,
    crash-restart capped (3 in 60s), `on_event(state, event) -> (state, Vec<Action>)`.
  - `initialize` handshake: honest client capabilities (only what #309-317 will ship);
    capture the server's `positionEncoding` + capability set for #309's use.
- **Spawn shim** (masked): rust-analyzer as a child process — stdio piped, kill-on-drop,
  `current_dir` = the workspace root; stderr captured to a log ring; pumps wired to the existing
  async runtime; bounded incoming queue (OS-pipe backpressure, not unbounded RAM).
- **Settings**: `LanguageServers: Vec<LanguageServerConfig{language, command, args}>` — the
  #87/#204 round-trip pattern, `#[serde(default)]`-safe; default = `rust-analyzer` on PATH for rust.
- **Status-bar segment**: `LSP: starting…/ready/crashed/failed` (+ absent = muted note), reusing the
  existing status segment pattern.

### Out (explicitly deferred)
- Document sync / any position math (#309 — the transport stays buffer-blind).
- Diagnostics (#310), hover (#311), go-to-def (#312), completions (#313), formatting (#314),
  references (#317).
- Any second language server (the settings SHAPE is per-language; only rust-analyzer ships here).
- Server-initiated requests beyond a clean reply-with-error/ignore (e.g. `workspace/configuration`
  answered with defaults; `window/workDoneProgress` accepted and dropped).

## Reference (§20)
**Zed (the editor reference) — architecture observed via our own source-derived doc
`docs/zed_architecture/subsystems/05-lsp-language-intelligence.md`; implementation clean-room from
the PUBLISHED Microsoft LSP 3.17 specification + the crates.io MIT `lsp-types` crate — explicitly
NOT Zed's `lsp-types` fork and no Zed source read (roadmap B6 lock).** Behavior matched: Zed's LSP
bring-up UX — the server spawns silently in the background on opening a matching file, a small
status readout reports readiness, a crashed server restarts quietly with a capped retry, and an
absent server degrades to a plain editor with a muted note (never a modal, never a crash).
Architecture properties adopted from the spec/doc (ideas, not code): dumb transport / smart
orchestration split, bounded incoming queue for backpressure, cancel-on-drop requests.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — Clean-room from the published spec.** LSP 3.17 + crates.io `lsp-types` (MIT). Zed source
  is never read (§20); our zed_architecture doc is the only Zed artifact consulted.
- **D2 — The transport is DUMB.** `marley_lsp` knows nothing of buffers, languages, files, or
  worktrees; it moves framed JSON-RPC and tracks lifecycle. Orchestration/coordinates arrive in
  #309+, outside this crate's core.
- **D3 — rust-analyzer only, but the settings shape is per-language from day one**
  (`LanguageServers: Vec<…>`) so #315's languages attach servers without a schema break.
- **D4 — House seam split**: framing/route/lifecycle/handshake-shaping = pure fns at cov/MSI 100;
  process spawn + pipe pumping = the masked shim (`#[mutants::skip]`), behavior-verified.
- **D5 — Bounded incoming queue** (backpressure to the server via the OS pipe, cap ~128 frames).
- **D6 — Failure posture**: absent binary → quiet muted status, editor fully functional; crash →
  capped restarts (3/60s) → `LSP: failed` + stop; NEVER a modal, NEVER a panic, NEVER log spam.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a `.rs` file is opened in the editor and a rust-analyzer binary is resolvable (settings or PATH), the system shall spawn it once, complete the `initialize`/`initialized` handshake, and reach `Ready`, with the status segment reading ready. | driven (live app w/ real rust-analyzer) + lifecycle units |
| REQ-002 | WHEN the server process dies unexpectedly, the system shall restart it and re-handshake, at most 3 restarts in any 60 s window, THEN stop and surface `failed`. | pure lifecycle units (cap boundary) + driven kill |
| REQ-003 | WHEN no server binary is resolvable, the system shall keep the editor fully functional and show one muted status note (no crash, no modal, no repeated logging). | driven negative smoke |
| REQ-004 | WHILE stdout bytes arrive at arbitrary chunk boundaries (header split mid-token, frames coalesced, body split), the FrameDecoder shall reassemble exactly the sent message bodies, and shall reject a malformed header without panicking. | pure units, cov/MSI 100 |
| REQ-005 | WHEN a response id matches a pending request, the router shall resolve exactly that request; WHEN an id/method is unknown, the router shall ignore it cleanly. | pure units |
| REQ-006 | WHEN a pending request is abandoned (dropped) or exceeds its timeout, the client shall emit `$/cancelRequest` for that id and resolve the caller with a timeout/cancel result. | pure request-table units |
| REQ-007 | WHEN `LanguageServers` settings are persisted and re-read, they shall round-trip; a hand-edited entry missing optional keys shall not wipe the list (serde defaults). | settings round-trip unit |
| REQ-008 | The `initialize` request shall advertise only implemented capabilities and shall record the server's negotiated `positionEncoding` + capabilities for later phases. | pure handshake-shaping unit + integration |

## Phase Plan
- **P2 Design** — crate layout + exact pure fn signatures; the shim mask boundary; the fake-server
  strategy for integration tests (a scripted child speaking LSP frames, so the gate never depends on
  rust-analyzer being installed); settings schema; status segment wiring; cargo-mutants `--list`
  survey of the planned seams.
- **P3 Implement** — per design; workspace member + deps (`lsp-types`, serde_json) added.
- **P3.5 Inspect** — independent critics vs the diff; fix real findings.
- **P4 Validate** — write + RUN the planned tests; gate green (cov/MSI 100 on pure seams); driven
  REQ-001/002/003 against real rust-analyzer (present on this machine — verified in plan).
- **P5 Complete** — archive, AAR capture, close #308.
