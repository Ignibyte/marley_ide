---
pipeline_id: b5a9b214-e574-43f3-bbf3-aa290b98814d
ticket: forge#274 (522d4028-081c-495f-b2a9-08a4aa639d59) · local docs/planning/tickets/open/TICKET-274-incremental-treesitter.md
aar_id: 9704870c-6528-44fc-b943-05466adc5ca5
status: Phase 5 — Complete PASS
title: Incremental + off-thread tree-sitter re-parse (B3.2)
type: feature
milestone: M17
references:
  - docs/marley_architecture/editor.md
---

## Title
The #268 critic MEASURED the v1 synchronous whole-file parse at 25.9ms
per keystroke on an 8k-line file — past the 16.7ms frame. Two halves:
INCREMENTAL (keep the previous `Tree` + feed `InputEdit`s so only the
damage re-parses) and OFF-THREAD (parse+query on a worker against a
text snapshot, generation-dropped, the UI swaps results in on a later
frame; small files keep the zero-latency sync path).

## Scope
### In
- `marley_syntax`: a `HighlightSession` holding the previous `Tree` +
  the previous TEXT SNAPSHOT (the plan-recon key: `BufferDelta`
  carries lengths, not the removed text or pre-edit points — with the
  pre-edit snapshot in hand, ALL InputEdit point math becomes pure
  string arithmetic). `SyntaxEdit` (6 plain numbers: start/old_end/
  new_end × byte/point) built by a pure
  `syntax_edit(last_src, new_src, delta)` + `point_at(src, byte)`
  (row + BYTE column, the tree-sitter convention; multibyte
  fenceposts). Session API: `highlight_full(src)` (also re-seeds the
  session) and `highlight_incremental(new_src, edits)` →
  `tree.edit(...)` per edit then `parse(new_src, Some(&old))`.
- SINGLE-delta steps go incremental (the typing case); MULTI-delta
  gaps (autorepeat lands 2+ between frames — the #276 lesson; undo
  bursts) fall back to a FULL parse: delta k>1's points would need
  text state k-1, which nobody holds (BufferDelta has no removed
  text). Honest and still off-thread.
- The worker: one thread owning the session (`Parser` is !Sync,
  is Send), a request channel (generation, nonce, version, text
  snapshot, Option<edits>), a response channel the EXISTING pump
  drains (the #203 notify pattern — set cache + dirty). A response
  older than the latest generation (or for a dead nonce) is DROPPED —
  a pure accept decision.
- The app seam: `refresh_syntax_cache` keeps the SYNC path for files
  under the line threshold (small-file latency stays zero) and
  enqueues for large ones; rows render the PREVIOUS spans until the
  swap (stale-but-aligned ≤1 frame).
- A RELATIVE perf pin (build-independent): on an ~8k-line fixture,
  the incremental 1-char re-parse must be < 1/3 of the full parse in
  the SAME build (the ticket's "order of magnitude, not the exact
  number" — a ratio survives debug vs release).

### Out
- Other grammars / language-aware anything (B3 later slices).
- `QueryCursor::set_byte_range` damage-window querying (a further
  optimization — recorded; the parse half dominates the measured
  cost: 14.6 of 25.9ms).
- Multi-file / background-tab sessions (the worker tracks the ACTIVE
  file's nonce, matching the cache's scope today).
- Cancellation of an in-flight parse (generation-drop makes a late
  result harmless; cancellation saves worker CPU only).

## Reference (§20)
Tree-sitter's own documented incremental model (MIT — the public
`Tree::edit` + `Parser::parse(…, Some(&old_tree))` API and its
InputEdit byte/Point contract). Marley-original wiring into Marley's
own delta log + cache seams. No copyleft editor source consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — The session keeps the previous TEXT (one String per active
  Rust file, ~300KB at 8k lines): it converts every InputEdit point
  purely and makes multibyte fenceposts unit-testable; the
  alternative (extending BufferDelta with points/removed text)
  touches the editor crate's hot edit path for one consumer.
- D2 — Single-delta incremental / multi-delta full: correctness
  before cleverness — delta chains need intermediate text states
  nobody holds. Autorepeat's 2-delta frames take the full path
  OFF-THREAD, which is exactly the case the worker exists for.
- D3 — Generation-drop, never cancel: a stale result is ignored on
  arrival; the pump applies only (gen == latest && nonce == active).
- D4 — The sync small-file path stays (threshold in lines, const,
  ~1000): zero added latency for the common case, and the worker
  seam stays exercised by the large path + tests.
- D5 — Stale-but-aligned rendering during the hop: the previous spans
  stay up for ≤1 frame; no blanking, no spinner.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `point_at(src, byte)` shall return (row, BYTE-column) per the tree-sitter convention, exact across multibyte chars and at line boundaries/EOF; `syntax_edit(last, new, delta)` shall produce the 6-field edit exactly (start from the pre-edit text, old_end from the pre-edit text, new_end from the post-edit text) on ASCII, multibyte, newline-inserting, and deleting deltas. | pure units (kill list) |
| REQ-002 | `highlight_incremental` shall produce IDENTICAL spans to a from-scratch `highlight_full` on the same post-edit text, across an edit corpus (insert/delete/replace, mid-line, cross-line, multibyte, at 0, at EOF). | property-style equivalence units |
| REQ-003 | On an ~8k-line fixture, in the same build: (a) the incremental RE-PARSE (tree.edit + parse-with-old-tree) shall run in under ONE THIRD of a from-scratch parse (measured 1.0 vs 9.2 ms release — the tree-sitter win the ticket buys), and (b) the END-TO-END incremental highlight shall run in under THREE QUARTERS of a full highlight (measured 0.46 release / 0.58 debug — the O(file) query walk floors it; the walk-windowing is the recorded follow-up). (Amended at inspect: the ticket's premise "the parse half dominates" was measured BACKWARDS — the query walk is 5.7 of the incremental 7.6 ms; frame-safety itself is REQ-005's off-thread guarantee, not this ratio.) | in-crate timed units (parse-only + end-to-end) |
| REQ-004 | A worker response whose generation is older than the latest request, or whose nonce is not the active file's, shall be DROPPED (the cache unchanged); the matching response shall swap the cache and repaint. | pure accept-decision units + headless |
| REQ-005 | Typing into a small Rust file shall keep the synchronous path (cache updated the same frame — existing behavior byte-identical); typing into a large file shall never block the frame on a parse (the request is enqueued; the previous spans render meanwhile) and the fresh spans shall land on a later frame. | headless (threshold both sides) + the existing #268 flows green |

## Phase Plan
- P2 exact types/channel/threshold + test plan; P3 implement; P3.5
  critic (InputEdit fenceposts vs the tree-sitter contract, the
  generation/nonce races, worker lifecycle/panic containment, the
  fallback completeness); P4 units + equivalence corpus + perf pin +
  headless + gate; P5 docs.
