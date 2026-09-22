---
pipeline_id: 20dfe523-8bff-4ef9-bfa6-04f7135b0fe2
ticket: forge#98 (b3be0115-7d23-43ef-b115-51ebb7728260) · local docs/planning/tickets/open/TICKET-098-tree-open-viewer.md
aar_id: aa68318f-2216-464a-8bec-efd5ecb96fff
status: Phase 5 — Complete PASS
title: open a file from the tree into the viewer
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/code_view.rs (PURE: viewer_open_path)
  - crates/marley_app/src/app.rs (SHIM: open_file_in_viewer + the tree ⌘-click + the finder refactor)
---

## Title
Open a file into the #97 viewer from the Files tree (⌘-click a file row) — plus a shared open helper the
finder ⌘↵ (#97) and the tree both use, and a dir/non-file guard.

## Scope
### In
- PURE `viewer_open_path(path, is_dir) -> Option<PathBuf>` (dir → None; file → Some).
- SHIM: a shared `open_file_in_viewer(&mut self, path)`; the finder ⌘↵ refactors onto it; the tree file-row
  ⌘-click opens the viewer (plain-click keeps the #59 path-insert).

### Out
- Syntax (seq-3). Gutter (seq-4). Scroll (seq-5). Binary/size guards (seq-10). Non-⌘ tree click behaviour.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — ⌘-click a file row = open in viewer; plain-click = insert the path (#59). Mirrors the finder ⌘↵ split.
- D2 — `open_file_in_viewer` reads on Ok only (a read error → no open, no crash); shared by both sites.
- D3 — `viewer_open_path` is the pure guard (dir → None).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `viewer_open_path(path, is_dir)` runs, it shall be None for a dir and Some(path) for a file. | unit |
| REQ-002 (visual) | WHEN a file row is ⌘-clicked, the viewer shall open on it. | self-test (engine; synthetic click env-blocked) |
| REQ-003 | gate GREEN, cov/MSI 100 on viewer_open_path; the shim masked. | gate |

## Phase Plan
- **P2** — viewer_open_path; open_file_in_viewer + the finder refactor + the tree ⌘-click; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: viewer_open_path MSI; the shared refactor output-identical; the ⌘-click vs plain split.
- **P4** — viewer_open_path tests (cov/MSI 100) + gate GREEN (open masked, engine).
- **P5** — docs, AAR, archive, close #98.
