---
pipeline_id: 23717b88-38b5-464c-b9bd-7262325e1982
ticket: forge#309 (5bc7247a-15e9-47f7-a2ce-826b5bfddb65) · local docs/planning/tickets/open/TICKET-309-lsp-document-sync.md
aar_id: 697adc7c-9c71-4992-9f42-c9097e72b37c
status: Phase 5 — Complete PASS
title: LSP document sync — didOpen/didChange(incremental)/didSave/didClose + the UTF-16 position bridge
type: feature
milestone: M20
references: [docs/planning/pipeline/completed/308-lsp-client-core.spec.md, docs/zed_architecture/subsystems/05-lsp-language-intelligence.md]
---

## Title
**Feed the server the truth, correctly addressed.** Every editor Buffer edit flows to the language
server as a `textDocument/didChange`, and — the load-bearing part — every position crossing the wire
goes through ONE pure **position bridge** that maps Marley's `char` offsets to the LSP `character`
column in the server's NEGOTIATED encoding (UTF-16 by default). The bridge is where LSP integrations
silently rot (an emoji is 1 char but 2 UTF-16 units); it is a first-class tested seam here because
EVERY M20 feature (#310 diagnostics, #311 hover, #312 goto, #313 completions, #314 format, #317 refs)
addresses through it.

## Scope

### In
- **The position bridge (PURE, `marley_lsp`)** — encoding-aware, over a line's text:
  - `column_in_encoding(line_text, char_col, encoding) -> u32` (UTF-16 `len_utf16` sum / UTF-8 byte /
    UTF-32 char); clamp a `char_col` past line-end to the line's end column (never panic).
  - `char_col_from_column(line_text, column, encoding) -> usize` (the inverse; clamp).
  - `offset_to_lsp_position(buffer, offset, encoding) -> (line, character)` — composes
    `Buffer::line_col` (offset→row,char_col, SHIPPED) with `column_in_encoding`.
  - `lsp_position_to_offset(buffer, line, character, encoding) -> CharOffset` — the inverse (for
    inbound results in #310+); clamp past-EOF.
- **Full-text `didChange` (v1)** — a single change event carrying the whole document text (a
  `{text}`-only `TextDocumentContentChangeEvent`, spec-valid under ANY negotiated sync kind;
  rust-analyzer accepts it). Version bumped monotonically per change. **Incremental (ranged) didChange
  is DEFERRED** — the #269 delta log (`deltas: Vec<(BufferVersion, BufferDelta)>`, unbounded) carries
  NO inserted text and `edit()` already stores that text in the undo stack, so a ranged fold would
  need either a bounded app-side change-capture queue or a snapshot-diff (a separate change); the
  bandwidth win is not worth the memory duplication for v1. The position bridge is built so incremental
  can layer on later with zero rework.
- **Doc lifecycle wiring (`marley_app`/`marley_lsp` shim)** — `didOpen` (languageId, version 0, full
  text) on a fresh editor open (the #308 spawn-trigger site); `didClose` on tab close; `didSave` on
  the save path; `didChange` on the pump when a doc's `version()` advanced past the last-synced
  version — incremental events from `edits_since(last_synced)` when the server's sync kind is
  Incremental, else a single full-text event (the negotiated `TextDocumentSyncKind`, from #308's
  `initialize` result). Monotonic per-doc version; the server never sees didChange before didOpen.
- **A local `Position` type** in the bridge (no `lsp-types` for #309 — hand-built notification JSON,
  the #308 idiom; `lsp-types` returns at #310 with `Diagnostic`).

### Out (explicitly deferred)
- `willSave` / `willSaveWaitUntil` (no consumer until format-on-save #314 chooses to use it).
- Pull diagnostics (`textDocument/diagnostic`) — #310 uses PUSH `publishDiagnostics`.
- Multi-file / multibuffer sync; only the focused editable buffers sync in v1.
- Any USE of the inbound positions (that is each feature ticket #310+).

## Reference (§20)
**Zed (the editor reference) — `docs/zed_architecture/subsystems/05-lsp-language-intelligence.md`
(the `LocalLspStore` doc-sync + the UTF-16 → `Unclipped<PointUtf16>` → clip → Anchor round-trip);
implementation clean-room from the published LSP 3.17 `textDocument` synchronization spec (the UTF-16
default is spec-mandated).** Behavior matched: as the user types, the server's view stays in lockstep
with the buffer, positions addressed in the server's negotiated encoding so a diagnostic under an
emoji lands on the right column. Architecture idea adopted (not code): concentrate ALL coordinate
translation in one place (Zed's single lesson) — Marley's is the pure `marley_lsp` position bridge.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — ONE position bridge, encoding-aware, pure.** Every wire position maps through it; it reads
  the encoding from #308's `Negotiated`. Built on the SHIPPED `Buffer::line_col` + `char::len_utf16`.
- **D2 — Full-text didChange (v1).** A `{text}`-only change event, spec-valid under any sync kind.
  Incremental deferred (see the scope note — the unbounded #269 delta log carries no inserted text).
  The editor crate is NOT touched (its pure core stays as shipped).
- **D3 — Own the position types.** A local `Position{line, character}` (both `u32`) in the bridge; no
  `lsp-types` re-add for #309 (its didOpen/change/save/close bodies are hand-built JSON, the #308
  idiom). `lsp-types` returns at #310 with its first `Diagnostic` consumer.
- **D4 — Per-doc sync state on the LSP host** (`{uri, languageId, version, last_synced_version,
  sync_kind}`), keyed by path; didOpen exactly once before any didChange; didClose drops it.
- **D5 — House seam split**: the bridge + fold + version logic = pure cov/MSI 100; the app-side
  didOpen/Change/Save/Close pump wiring = the masked `lsp_host` shim, behavior-verified.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN mapping a buffer `char` offset on a line containing an emoji (1 char = 2 UTF-16 units) to an LSP position under UTF-16, the bridge shall return the column counted in UTF-16 code units. | pure units, cov/MSI 100 |
| REQ-002 | WHEN the server negotiated UTF-8 (resp. UTF-32) encoding, the bridge shall count the `character` column in bytes (resp. chars) instead of UTF-16 units. | pure units |
| REQ-003 | WHEN a `char_col`/`character` is past the line's end, the bridge shall clamp to the line-end column and never panic. | pure units |
| REQ-004 | WHEN a fresh `.rs` file is opened in the editor, the system shall send exactly one `didOpen` (version 0, full text) before any `didChange`, and `didClose` on tab close. | pure sync-state units + fake_ls integration |
| REQ-005 | WHEN a synced document's buffer version has advanced, the system shall send a `didChange` carrying the whole current document text with a monotonically increasing document version. | pure change-builder + version-monotonic units + fake_ls integration |
| REQ-006 | WHEN the buffer text round-trips through the position bridge (offset→position→offset) for ascii/emoji/multi-line/EOL positions, it shall return the original offset (clamped past-EOF). | pure bridge round-trip units |
| REQ-007 | WHEN a document is saved, the system shall send `didSave` after the successful write. | integration/driven |

## Phase Plan
- **P2 Design** — the exact bridge signatures + the `BufferDelta.new_text` change (+ its editor-crate
  test/mutation impact); the fold's ordering proof; the per-doc sync-state shape on `lsp_host`; the
  `TextDocumentSyncKind` capture from #308's initialize result; the fake_ls "record didChange" mode.
- **P3 Implement** — per design.
- **P3.5 Inspect** — critics vs the diff (the position math is the danger; the ordering fold second).
- **P4 Validate** — pure units to cov/MSI 100; fake_ls integration (didOpen→didChange→didSave order,
  the recorded change events replay to the buffer text); driven re-verify against real rust-analyzer
  (type into a .rs, the server stays in lockstep — provable end-to-end at #310).
- **P5 Complete** — archive, AAR, close #309.
