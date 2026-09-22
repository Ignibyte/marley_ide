---
pipeline_id: 5fdd1325-764a-415f-a0b1-53bbd81948bc
ticket: forge#291 (66bf7033-82aa-427c-986a-4652e7cec6df) · local docs/planning/tickets/open/TICKET-291-trace-navigator.md
aar_id: 1c28410d-d18b-4bfc-ad88-367f651bf9e6
status: Phase 5 — Complete PASS
title: Multi-frame stack-trace navigator — parse trace frames (incl. Python) into the diagnostics
type: feature
milestone: M18
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
Parse a panic/traceback/stack into its source frames — including the Python
`File "path", line N` shape #212 misses — and fold the open-file frames into the
#289 diagnostics so the gutter marks them and #290's F8 navigates them.

## Scope
### In
- `links.rs` (pure): `parse_trace_frames(output) -> Vec<(PathBuf, usize, Option<usize>)>` — per line,
  first try the Python frame shape (`File "<path>", line <N>` → `(path, N, None)`); else the first
  `scan_links` `File{line: Some}` (rust panic / backtrace / node — already `path:line`-shaped) →
  `(path, line, col)`; collect every frame in order.
- `app.rs` (shim): `open_file_diagnostic_rows` also unions the `parse_trace_frames` frames resolving to
  the open file (0-based rows) with the existing #289 `diagnostics_for_file` rows.

### Out (explicitly deferred)
- A standalone clickable "call stack" panel / frame-list UI — v1 navigation is the #290 F8 walk + the
  #289 gutter (already shipped); the panel is a follow-up.
- Grouping frames by trace / showing the fn label in a UI — the label is not needed for the row-set fold.
- Non-Python NON-`path:line` shapes beyond the four named — a rare-format follow-up.

## Reference (§20)
Zed / a debugger call-stack: a stack trace's frames are navigable source
locations. Marley matches by parsing every frame's location (incl. Python's) and
surfacing them via the shipped gutter + F8 nav. Clean-room §20: the frame parser
is Marley's own (reusing #212 for the `path:line` shapes); no Zed source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — Python is the gap; the others reuse `scan_links`.** rust panic
  (`panicked at p:l:c`), backtrace (`at p:l`), node (`at fn (p:l:c)`) are
  `path:line`-shaped → `scan_links` already finds them (#212/#289). Python
  `File "p", line N` is NOT → `parse_trace_frames` handles it explicitly.
- **D2 — one frame per line, in order.** A line yields at most one frame (the
  Python shape, else the first `scan_links` File-with-line); a non-frame line yields none.
- **D3 — fold into the #289 rows, no new UI.** The open-file frames union into
  `open_file_diagnostic_rows` → the gutter (#289) + F8 (#290) navigate them. The
  standalone panel is deferred (D-Out).
- **D4 — the Python path is taken verbatim from the quotes** (`File "…"`), then
  `resolve_under_root` (#190) like every other ref.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN output has a Python frame `File "p.py", line 42, in fn`, `parse_trace_frames` shall yield `(p.py, 42, None)`. | links.rs unit: the Python shape → the frame; a rust `panicked at a.rs:10:5` → `(a.rs,10,Some(5))`; a node `at f (b.js:3:1)` → `(b.js,3,Some(1))`. |
| REQ-002 | WHEN a line has no frame, `parse_trace_frames` shall yield none for it; a multi-frame trace yields every frame in order. | links.rs unit: a plain line → none; a 3-line Python traceback → 3 frames ordered. |
| REQ-003 | WHEN a failed block's output has trace frames into the open file, `open_file_diagnostic_rows` shall include those rows (unioned with the #289 diagnostics). | Headless / shim: a Python-traceback block + the open file → the rows include the frame lines. |
| REQ-004 | `parse_trace_frames` shall be pure at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `parse_trace_frames` (the Python matcher + the scan_links fallback) + the `open_file_diagnostic_rows` fold; the test plan.
- **P3 Implement** — the pure parser + the shim union.
- **P3.5 Inspect** — the Python matcher edges (quotes, `, line `, digits), the fallback, the fold.
- **P4 Validate** — the pure fixtures (python/rust/node/no-trace/multi-frame) + a headless fold; gate green [diff].
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #291.
