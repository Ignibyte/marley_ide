---
pipeline_id: e92f38b3-86d5-41de-bc44-37c77bf0d5f8
ticket: forge#269 (21696db9-3828-40ec-8d10-b7985d8cb8a7) · local docs/planning/tickets/open/TICKET-269-anchors.md
aar_id: a7bc5872-7fde-4970-8382-ca79c045b374
status: Phase 5 — Complete PASS
title: B4 — anchors: a delta-log anchor layer on marley_editor
type: feature
milestone: M16
references:
  - docs/zed_architecture/subsystems/02-text-buffer-anchors.md
---

## Title
Editor frontier B4 — the quiet prerequisite for multi-cursor (B5) and LSP
(B6). Today every stored `CharOffset` goes STALE on any edit (a cursor/mark/
async-LSP result lands in the wrong place). Add an anchor layer: `Anchor
{version, offset, bias}` rebased through the buffer's edit deltas — the
Patch::old_to_new arithmetic WITHOUT the CRDT (no SumTree rewrite, no
Lamport clock; the full Fragment/Locator machinery is deferred to a real
concurrent multi-writer milestone). Discovery correction: the Buffer emits a
`BufferDelta` per edit but does NOT yet store a log — the slice adds the
log (`deltas: Vec<(BufferVersion, BufferDelta)>` fed by the one shared
apply path, so undo/redo rebase too) + `edits_since(version)` (which B3
tree-sitter also needs, per the roadmap note).

## Scope
### In
- `crates/editor` (pure, cov/MSI 100):
  - `Bias { Left, Right }` + `Anchor { version, offset: CharOffset, bias }`
    (new `anchor.rs`; re-exported flat per the crate idiom).
  - The pure single-delta rebase (`rebase_offset(offset, bias, &delta)`)
    and the fold (`resolve` = rebase over every delta since
    `anchor.version`, clamped into `[0, len_chars]`).
  - Buffer additions: the delta log (appended in `apply_edit` — the one
    path every edit/undo/redo shares), `edits_since(BufferVersion)`,
    `anchor_at(offset, bias)`, `resolve_anchor(&Anchor) -> CharOffset`.
  - v1 log policy: unbounded within a buffer's lifetime (documented; a
    compaction epoch is B5/B6 territory when anchor populations grow).

### Out (explicitly deferred)
- The CRDT (Fragment/Locator/Lamport) — a concurrent multi-writer
  (human+agent co-edit) milestone.
- Migrating the EXISTING caret/selection/marks onto anchors (B5 does that
  with multi-cursor); this slice ships the layer + its proofs.
- Snapshots (`BufferSnapshot`) — arrives with B3's off-thread parse.

## Reference (§20)
N/A — Marley-original design. The anchor CONCEPT (a position that survives
edits by rebasing through deltas, with left/right bias at insertion points)
is public CS (the operational-transform/patch literature; UAX-agnostic).
The reference doc `docs/zed_architecture/subsystems/02-text-buffer-anchors.md`
is a behavior-level transcription and its provenance section explicitly
flags the GPL boundary: Marley implements the delta-log-first plan it
recommends, NOT the GPL `text` crate's CRDT rendition (no SumTree, no
Locator, no clock).

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Bias semantics (the public convention): an edit strictly BEFORE the
  anchor shifts it by the length delta; strictly AFTER leaves it; a span
  COVERING it collapses to the span start (Left) or the end of the
  replacement (Right); a pure INSERT exactly AT it stays put (Left) or
  jumps after the insertion (Right).
- D2 — The log lives on the Buffer and is fed ONLY by `apply_edit` (the
  shared non-recording core), so edit/undo/redo all rebase identically.
- D3 — `resolve_anchor` on a version with no logged path (an anchor from a
  FUTURE or pre-log version) clamps to `[0, len]` after folding whatever
  applies — deltas apply iff `delta_version > anchor.version` (each log row
  stores the post-edit version).
- D4 — No public mutation of the log; `edits_since` returns a borrowed
  ordered slice/iterator.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `anchor_at(o, b)` shall stamp the buffer's CURRENT version; `resolve_anchor` at that same version shall return `o` (clamped). | unit |
| REQ-002 | WHEN text is inserted/deleted strictly BEFORE an anchor, resolve shall shift it by exactly the length delta; strictly AFTER, resolve shall leave it unchanged. | units |
| REQ-003 | WHEN an edit REPLACES a span covering the anchor, resolve shall yield the span start (Left) / the replacement end (Right). | units |
| REQ-004 | WHEN a pure insert lands exactly AT the anchor offset, Left shall stay before the insertion and Right shall land after it. | units |
| REQ-005 | Resolving after N edits shall equal resolving stepwise after each edit (fold correctness), demonstrated on a mixed sequence incl. undo/redo. | unit (sequence) |
| REQ-006 | Undo/redo shall move anchors like any other edit (the log records them). | unit |
| REQ-007 | `edits_since(v)` shall yield exactly the deltas recorded after `v`, in application order. | unit |
| REQ-008 | `resolve_anchor` shall always return an offset in `[0, len_chars]`. | units (edge deletions at EOF) |
| REQ-009 | The new surface shall hold the crate bar: coverage 100 / MSI 100 (real `--list`-traced mutants all killed). | gate:4/5 |

## Phase Plan
- **P2 Design** — exact types/signatures, the rebase arithmetic table, log
  row shape, file manifest, mutant-aware test plan.
- **P3 Implement** — anchor.rs + buffer.rs additions + re-exports.
- **P3.5 Inspect** — critics (bias edge cases, fold order, log/version
  fencepost, §20).
- **P4 Validate** — the unit suite; `cargo mutants --list` trace; gate --diff.
- **P5 Complete** — CHANGELOG, editor.md, AAR, archive, close.
