---
pipeline_id: 78b479ea-ec55-48bc-817c-eeef6648bfff
ticket: forge#105 (41f545f7-e2b3-4f0f-b736-fc2089cf7ffc) · local docs/planning/tickets/open/TICKET-105-jump-to-file.md
aar_id: d8f8b283-ea9f-4b3f-86a9-0fe14bb6bff1
status: Phase 5 — Complete PASS
title: jump-to-file from a diff
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/code_view.rs (PURE: FileRef, parse_file_ref)
  - crates/marley_app/src/app.rs (SHIM: the diff FileHeader rows click → open + jump)
---

## Title
Click a file reference to open it in the viewer: recognize `path:line` tokens (`parse_file_ref`) and make
the diff overlay's file headers clickable → open that file (at the line) in the code viewer.

## Scope
### In
- PURE `parse_file_ref(text) -> Option<FileRef{path, line}>` (recognize `path[:line[:col]]`; reject non-paths).
- SHIM: the diff overlay's FileHeader rows click → open `project_root.join(path)` in the viewer at the line.

### Out
- Clicking file refs in live terminal/agent output (follow-up). Column positioning. Fuzzy path resolution.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — a "path" is a `/`- or `.`-bearing first colon-field; the 2nd field is the line if numeric (else None).
- D2 — the FileHeader row has no line → opens at the top; the `:line` handling serves future output-token clicks.
- D3 — line is 1-based in text → 0-based scroll index via `jump_to(line-1, …)`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `parse_file_ref(text)` sees a `/`-or-`.` path, it shall return the path + optional numeric line. | unit |
| REQ-002 | WHEN the token is not a path, `parse_file_ref` shall return None. | unit |
| REQ-003 (visual) | WHEN a diff file header is clicked, that file shall open in the viewer. | self-test (engine + #102 live git) |
| REQ-004 | gate GREEN, cov/MSI 100 on parse_file_ref; the shim masked. | gate |

## Phase Plan
- **P2** — parse_file_ref + FileRef; the FileHeader click; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: parse_file_ref MSI (trim/guard/line-parse/None); the click → open+jump+close.
- **P4** — parse_file_ref tests (cov/MSI 100) + gate GREEN.
- **P5** — docs, AAR, archive, close #105.
