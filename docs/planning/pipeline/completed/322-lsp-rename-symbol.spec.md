---
pipeline_id: 5a2cb50d-73a2-4c5f-9f1c-7fcc84059f2d
ticket: forge#322 (25a0742a-b2a2-4543-b261-c3b85772f1f9) · local docs/planning/tickets/open/TICKET-322-lsp-rename-symbol.md
aar_id: eadbc5a6-f9a6-4bc4-b879-22982b95d827
status: Phase 5 — Complete PASS
title: LSP rename symbol (F2) — prepareRename + the multi-file WorkspaceEdit engine
type: feature
milestone: M21
references: [forge#314, forge#312, forge#311, forge#310, forge#296]
---

## Title
F2 renames a symbol everywhere it lives — and builds the ONE WorkspaceEdit apply path
that #323 code-actions (and later import-organizing formatting) reuse. The M21 foundation.

## Scope
### In
- **F2** (Editor-scoped) on a symbol → `textDocument/prepareRename` when advertised (validates
  the token, returns its range) → an inline rename **draft** (the `renaming_tab`/`naming_workflow`
  idiom) → Enter sends `textDocument/rename`, Esc cancels.
- **WorkspaceEdit parse (PURE, marley_lsp)**: normalize BOTH `changes:{uri→[TextEdit]}` and
  `documentChanges:[TextDocumentEdit]` into `Vec<FileEdits{path, version:Option<i64>, edits}>`,
  keyed by canonical path via `path_from_file_uri` (the #310 normalize-both-sides pair). Per-element
  skip so one bad edit cannot lose the rename.
- **The single-doc applier primitive (PURE)**: `apply_text_edits(text, edits) -> Result` — sort +
  apply last-to-first over a `String`, overlap → reject the whole batch (return the original). This
  is the #314-shared core, built here (D1).
- **The multi-file applier (SHIM over the pure core)**:
  - OPEN buffers (all projects' `open_docs`): apply as ONE `begin_undo_group`/`end_undo_group` per
    buffer, `EditOrigin::Agent`, over the pure edit set; a single ⌘Z reverts that file.
  - CLOSED files: read → `apply_text_edits` → `fs::write`; skip (with report) a file over the
    `VIEWER_MAX_BYTES`/`is_probably_binary` limits.
  - Version-mismatch (an open doc's synced version ≠ the edit's named version) → skip + report.
- Capability gate off raw `server_caps["renameProvider"]`; not-ready / no-provider / prepareRename
  declines → quiet flash, no buffer changes.
- `RequestPurpose::Rename` + `PrepareRename` on the #311 request recipe + the ONE drain.
- Result flash: `renamed in N files (M skipped)`.

### Out (explicitly deferred)
- **Resource ops** `CreateFile`/`RenameFile`/`DeleteFile` in `documentChanges` → REJECT the whole
  edit with a flash (D2). rust-analyzer's plain rename never emits them; file-ops = a named follow-up.
- **Cross-file undo unification** — undo is per-file for open buffers; disk-only files are not
  undoable (D4, stated not hidden).
- **#314's formatting request + format-on-save** — this ticket builds only the shared applier
  primitive; #314 consumes it.
- Rename **preview**/diff-before-apply; prepareRename `{defaultBehavior}`/placeholder-only variants
  beyond `{range}`/`{range,placeholder}`.

## Reference (§20)
**Zed (the editor)** — the observed universal behavior: F2 on a symbol renames every occurrence
across the whole workspace (including files not open), as one undoable action. Marley matches that
BEHAVIOR via the **published LSP 3.17 spec** (`textDocument/prepareRename`, `textDocument/rename`,
`WorkspaceEdit` with `changes`/`documentChanges`, `TextDocumentEdit`, `OptionalVersionedTextDocument
Identifier`) plus Marley's OWN `Buffer`/undo-group model and `EditOrigin::Agent` write seam. The
replace-everywhere + one-undo semantics come from the spec (a `WorkspaceEdit` names explicit ranges
per file), not from reading anyone's source. No Zed GPL source read — clean-room, confirmed at design.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — build the single-doc `apply_text_edits` primitive HERE.** #322 needs it and #314 (its other
  intended home) is not built; one applier, built once, #314 later consumes it. Avoids two apply paths
  (the convergence the batch is designed around).
- **D2 — resource ops reject the WHOLE edit** with an honest flash (not a partial apply). Plain rename
  doesn't need them; a half-applied file-op rename is worse than none.
- **D3 — closed files use NEW read→apply→`fs::write` machinery** (none exists — the only write path is
  `save_active`'s whole-text write). A file open in ANY project's surface is edited through its
  `Buffer`, never disk (enumerate all surfaces, not just active).
- **D4 — undo scope = per-file for open buffers.** Disk-only files are not undoable in v1; stated in
  the result flash reasoning, not hidden.
- **D5 — key by canonical `PathBuf`** via `path_from_file_uri` → `absolute()`, the SAME normalization
  the store write/read uses (the #310 `PR-claude-external-uri-key-normalize-both-sides` rule).
- **D6 — hand-parse the WorkspaceEdit** (no new `lsp-types` surface beyond the shipped Diagnostic
  seam) — the #312/#313 D3 posture; a typed `from_value` would reject the whole payload on one bad
  element.
- **D7 — prepareRename is OPTIONAL**: a server that omits it or declines → fall back to the client
  word range (`word_query`, #313). The rename still fires from the caret position.
- **D8 — inherit the request guards WHOLE** (`PR-claude-second-consumer-must-inherit-the-first-
  consumers-guards`): the rename response is applied only while the requesting editor identity holds;
  a stale answer is dropped before it edits.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN F2 is pressed on a symbol in an editor tab AND the server advertises `renameProvider`, the system shall send `textDocument/prepareRename` (or fall back to the client word range when unsupported) and open an inline rename draft seeded with that range. | headless drive + LIVE tee (real prepareRename frame) |
| REQ-002 | WHEN the rename draft is confirmed, the system shall send `textDocument/rename` carrying the new name and the caret position. | LIVE tee PAYLOAD assert (newName + position, not method name) |
| REQ-003 | The WorkspaceEdit parser shall normalize BOTH `changes` and `documentChanges` shapes into a per-file edit list keyed by canonical path, skipping any malformed element while keeping the rest. | pure unit cov/MSI 100 |
| REQ-004 | WHEN a rename touches an OPEN buffer, the system shall apply its edits last-to-first as ONE undo group with `EditOrigin::Agent`, and a single ⌘Z shall revert that file's rename. | headless drive + LIVE (⌘Z reverts) |
| REQ-005 | WHEN a rename touches a CLOSED file, the system shall read→apply→write it to disk, skipping with an honest report any file over the binary/size limits. | headless + LIVE (closed file byte-checked on disk) |
| REQ-006 | WHEN edits within one file overlap, the system shall reject that file's batch (leaving it unchanged) rather than write a corrupt result. | pure unit |
| REQ-007 | WHEN no rename is possible (no `renameProvider`, host not ready, prepareRename declines), the system shall show a quiet status flash and change no buffer. | headless + review |
| REQ-008 | The `apply_text_edits` single-document primitive shall apply a `Vec<(range,new_text)>` last-to-first over a `String`, rejecting an overlapping batch (return the original). | pure unit cov/MSI 100 |
| REQ-009 | WHEN a rename names a version for an OPEN doc that differs from its synced version, the system shall skip that file and report it, never applying stale-addressed edits. | pure unit + headless |

## Phase Plan
- **P2 Design** — the pure/shim split (marley_lsp parse + `apply_text_edits`; the marley_app applier
  shim + inline draft + request wiring); the closed-file writer; the file manifest; the ≥1-test-per-REQ
  plan; the mutation surface (apply reorder/overlap boundaries — pin like popup_origin). Confirm §20.
- **P3 Implement** — to the manifest; every new named pure fn gets a direct unit (the extracted-helper
  mutation-surface rule).
- **P3.5 Inspect** — independent critics vs the diff; probe-prove findings; the apply/undo/closed-file
  paths get the hardest look.
- **P4 Validate** — write + RUN tests; gate green [diff]; the LIVE tee drive (rename across one open +
  one closed file, both byte-checked; the real rename frame payload; ⌘Z reverts the open half).
- **P5 Complete** — CHANGELOG + crate-map + editor.md; AAR capture; archive; close #322.
