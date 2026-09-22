---
pipeline_id: 81827455-a7cf-4ac4-b7f0-b6a79bfd416a
ticket: forge#317 (51426ddb-f716-4de2-9865-42ce0b17c1d2) · local docs/planning/tickets/open/TICKET-317-find-references.md
aar_id: fe3478a7-7ca2-4ee9-a922-3f9baa3cda28
status: Phase 5 — Complete PASS
title: Find references (⇧F12) — every usage, grouped by file, one Enter from the caret
type: feature
milestone: M20
references: [text_document_position_params — its doc already names references as a future user (rpc.rs:221), the position-keyed stale guard = the Definition pattern VERBATIM (DefinitionKey, app.rs:8858), the capability-reader template (inlay.rs:118), THE PICKER FINDING: DefPicker is definition-shaped — flat, no preview, no grouping (editor_nav.rs:70; DefLocation has no text) → a richer row type is required, cap_with_tail (editor_symbols.rs:42), the NavStack jump idiom (jump_to_definition app.rs:8906), lsp_position_for (app.rs:7919), ⇧F12 verified FREE]
---

## Title
⇧F12 on a symbol answers "who uses this?" — every reference in a picker, grouped by file, each row showing
its line's text with the match span emphasized; Enter jumps (centered, NavStack-pushed so ⌃- returns).
The read-only sibling of go-to-definition and the last leg of navigate-by-meaning (#312 def + #304
file-symbols + this).

**The recon's shaping finding:** the #312 `DefPicker` canNOT carry this — it is definition-shaped (a flat
`Vec<(String, DefLocation)>`, `path:line` labels only, no line text, no grouping). References need a NEW
row model; the JUMP path (`jump_to_definition`'s open+center+push) and the stale-guard/capability
patterns are the genuine reuse.

## Scope
### In
- **The request:** `RequestPurpose::References(ReferencesKey { uri, line, character })` —
  **position-keyed, the Definition pattern verbatim** (no version: an edit doesn't move which symbol was
  asked about; a NEWER ⇧F12 supersedes). Params = `text_document_position_params(...)` + `context:
  { includeDeclaration: true }` (the base builder's doc already anticipates this caller). Capability:
  `references_support(server_caps)` reading `referencesProvider` (the inlay template) + host wrapper; no
  capability → quiet flash.
- **Result shaping (PURE, cov/MSI 100):** `group_references(Vec<Location>, current_file) -> RefResults`
  — normalize, sort by (path, line, col), DEDUPE exact duplicates (servers send them), **current-file
  rows first** (the local-first read), a computed count line (`N references in M files` — derived, never
  counted twice), and an honest cap (**~200 rendered rows + a "+K more" tail via the shipped
  `cap_with_tail`** — no silent truncation; #325 caps at 64 with the same helper, this list runs longer).
- **The picker (NEW type, the #325 modal shape):** `OpenReferences { finder: FinderState, results:
  RefResults }` — file HEADER rows + reference rows (`line: text` with the match span accent-emphasized
  via the styled-runs the finder rows already use); ↑/↓ skip headers; type-to-filter fuzzy over
  path+line-text (`marley_search_core::fuzzy_score` — LOCAL, the list is in hand; unlike #325's
  server-side re-query). Esc closes. Opens instantly with a `searching…` row that RESOLVES when the
  response lands (async; a superseded query's answer drops — the stale guard).
- **The line text:** an OPEN buffer's rows read LIVE text (an unsaved edit shows the truth); cold files
  read from disk ONCE PER FILE (batched by the grouping, never per row); an unreadable file degrades to
  a path-only row — never a panic, never a stall.
- **The jump:** Enter → the `jump_to_definition` idiom (open + `position_to_offset` at the host's
  encoding + center + **PUSH the NavStack** with the pre-jump origin). Zero results → the quiet status
  flash, never an empty picker.
- **The chord:** ⇧F12 `(F,F,F,T,"f12")` verified FREE (plain F12 = go-to-definition), Editor-scoped
  (roster 67→68, scoped 22→23, individual assert first) + a palette row ("Find All References").
### Out (explicitly)
- A persistent references PANEL (this is a picker); call hierarchy (`callHierarchy/*`); highlight-all-
  occurrences-in-file (⌘⇧L shipped the multi-cursor cousin); rename-from-references (F2 shipped);
  streaming partial results (`partialResultToken` — one response v1).

## Reference (§20)
LSP 3.17 published spec (`textDocument/references`, `ReferenceContext.includeDeclaration`,
`referencesProvider`). VS Code / Zed = OBSERVED (⇧F12; grouped-by-file with line previews; declaration
included; local-file-first reading order).

### Prior art
1. **Behavior maps / observed** — the picker semantics above.
2. **Published material** — LSP 3.17 references; `includeDeclaration` is the client's honest default
   (you asked "who uses this", the definition anchors the list).
3. **OUR OWN CODE — the sweep separated reuse from build:** the stale guard (Definition's verbatim), the
   capability reader (inlay's template), the jump+NavStack (`jump_to_definition`), `cap_with_tail`,
   `FinderState`, and `fuzzy_score` are all shipped; the params builder's own doc names this caller.
   What is genuinely NEW — proven by reading `DefPicker`/`DefLocation` — is the grouped row model with
   line text (no shipped picker carries preview text). No new deps.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-NEW-ROW-MODEL** — a references-shaped picker type; do NOT stretch DefPicker past its shape.
- **D-POSITION-KEYED-GUARD** — the Definition pattern; newest ⇧F12 wins; stale answers drop.
- **D-INCLUDE-DECLARATION** — true, always (observed + honest).
- **D-CURRENT-FILE-FIRST** — the local-first sort, pinned in the pure table.
- **D-LIVE-TEXT-FOR-OPEN-BUFFERS** — open = live, cold = one disk read per file, unreadable = path-only.
- **D-PUSH-NAVSTACK** — a jump is a navigation; ⌃- returns.
- **D-HONEST-CAP** — ~200 + "+K more"; silent truncation is the named anti-goal.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | group/sort references by (path, line, col), current file first, exact duplicates deduped | pure table |
| REQ-002 | compute the `N references in M files` line from the grouped result (never counted separately) | pure |
| REQ-003 | cap rendered rows with an honest "+K more" tail | pure (cap_with_tail) |
| REQ-004 | show LIVE text for an open buffer's rows and disk text (one read per file) for cold ones; degrade unreadable to path-only | headless |
| REQ-005 | jump on Enter, centered, PUSHING the NavStack (⌃- returns), across files | headless |
| REQ-006 | drop a response for a superseded query (a newer ⇧F12 raced it) | headless — the guard row |
| REQ-007 | flash quietly on zero results / no capability — never an empty picker | headless |
| REQ-008 | resolve the `searching…` row when the async answer lands | headless (fake_ls lane) |
| REQ-009 | resolve ⇧F12 + the palette row to the verb (roster/palette guards; plain F12 untouched) | unit |

## Phase Plan
P2 confirm the fake_ls lane can serve `textDocument/references` (extend the fixture if not) + the row
model's exact shape (headers vs rows, the skip-header navigation) + the emphasis mechanism on the row
text; P3 the pure grouping first (truth tables), then request/guard/capability, then the picker + jump +
chord; P3.5 critics on the disk-read batching (a 500-reference symbol), the live-vs-disk truth split, the
superseded-query drop, the header-skip navigation edges; P4 tables + fake_ls drives + gate; P5 docs.
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
