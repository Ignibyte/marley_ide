---
pipeline_id: cce2a066-9fc0-4637-bf04-ee253228e584
ticket: forge#289 (084d5945-7a7f-45d2-a04f-4453e25d9235) · local docs/planning/tickets/open/TICKET-289-diagnostics-gutter.md
aar_id: 9bc05f30-652b-4532-bbcd-1d7b8daacb73
status: Phase 5 — Complete PASS
title: A failed command's file:line refs become inline gutter markers in the open editor
type: feature
milestone: M18
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
When a command block fails and its output references lines in the file open in the
editor, mark those rows in the line-number gutter (a red bar) — errors shown IN the
editor. Reuses #212's ref parser + the #272 render-capture pattern.

## Scope
### In
- `links.rs` (pure): `diagnostics_for_file(output, open_path, root) -> Vec<usize>` — the sorted, deduped
  0-based editor ROWS in `open_path` referenced by `output` (each `File{line: Some}` ref whose
  `resolve_under_root(root, path) == open_path`; the 1-based line → 0-based row via `saturating_sub(1)`).
- `app.rs` (shim): `open_file_diagnostic_rows(&self) -> Vec<usize>` — the diagnostics for the OPEN editor
  file from the LAST failed block across the active project's terminal panes; captured into the editor
  row closure (like `efind`); a gutter marker rendered when the row is in the set.

### Out (explicitly deferred)
- Severity (error vs warning) + the hover message — v1 is a single marker per referenced row.
- `next/prev` navigation across the markers — that is #290 (this ticket produces the row set it navigates).
- Re-run / clear-on-green — that is #292.
- Live incremental update as output streams — recomputed from the finished failed block (a finished
  block's output is immutable).

## Reference (§20)
Zed / VS Code — the inline-diagnostics gutter: a build/lint error shows a marker in
the editor's gutter at the offending line. Marley matches by parsing the failed
block's output for refs into the open file and marking those rows. Clean-room §20:
the ref-filter + the gutter render are Marley's own (reusing #212); no Zed source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the row set is PURE + sorted.** `diagnostics_for_file` returns sorted,
  deduped 0-based rows so the render can `binary_search` per row (the #272 `efind`
  idiom) and #290 can walk them in order.
- **D2 — 1-based line → 0-based row** via `saturating_sub(1)` (compiler lines are
  1-based; the uniform_list rows are 0-based). A ref to line 0 (shouldn't occur)
  clamps to row 0.
- **D3 — source = the LAST failed block** in the active project's terminal panes
  (a Finished block with `StatusKind::Failure`), matched against the open editor
  file. No new pump state (computed in the render from the immutable finished
  blocks); #292 will refine "which failure" + add clear-on-green.
- **D4 — the marker lives in the existing gutter** (reuse `gutter_width`/the row's
  gutter area) — a red bar/dot; no new column geometry.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a failed block's output references `open_path:N`, `diagnostics_for_file` shall include row `N-1`. | links.rs unit: output with `src/a.rs:12` + open_path a.rs → `[11]`; a ref to a DIFFERENT file → excluded; a bare path (no line) → excluded; two refs to the same line → deduped. |
| REQ-002 | The referenced rows shall be sorted ascending and deduped. | links.rs unit (out-of-order refs → sorted; dup line → one entry). |
| REQ-003 | WHEN the open editor file has diagnostic rows, the editor render shall show a gutter marker at each. | Headless: open a file + a failed block referencing it → `open_file_diagnostic_rows()` non-empty at the right rows (the render capture is the shim over this). |
| REQ-004 | WHERE no block has failed (or refs point elsewhere), the diagnostic rows shall be empty. | links.rs unit (empty/success/other-file → `[]`) + headless (a succeeding block → empty). |
| REQ-005 | `diagnostics_for_file` shall be pure at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `diagnostics_for_file` + `open_file_diagnostic_rows` + the gutter-marker render capture; the test plan.
- **P3 Implement** — the pure fn + the shim + the render marker.
- **P3.5 Inspect** — the resolve/filter, the 1-based→0-based, the last-failed pick, the render capture.
- **P4 Validate** — the pure fixtures + a headless "failed block → diagnostic rows" flow; gate green [diff].
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #289.
