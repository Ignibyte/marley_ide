---
pipeline_id: a47d258f-f48d-48e3-9c0f-287696658bf1
ticket: forge#121 (108e9743-4c3b-4518-a0d6-b0d86c13b3ca) · local docs/planning/tickets/open/TICKET-121-pane-render.md
aar_id: a37488ef-1119-4808-b6e2-223d478f14a6
status: Phase 5 — Complete PASS
title: typed-pane render dispatch (M6 seq-2)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/app.rs (SHIM: the per-pane content dispatch + a temporary boot default)
---

## Title
The first VISIBLE Warp step: a pane now renders its content by kind — a Files/Code/Git pane shows a real
body in the grid, not an empty frame. Boots as `[terminal | files pane]` so you can see it.

## Scope
### In
- SHIM: the per-pane content dispatch (Terminal → terminal render; FileTree/CodeView/Git → a kind body
  reusing the existing pure fns) below the per-pane title bar.
- A TEMPORARY boot default opening a FileTree pane beside the terminal (seq-3 replaces it).

### Out
- The rich, interactive file/code/git panes (seq-4/5/6). The config-driven boot layout (seq-3). New pure logic.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the content render dispatches on `state.kind()`; a session-less pane gets a minimal-but-real body
  (files tree / code lines / git summary), reusing `file_icon`/`entry_is_dimmed`/`change_summary`/code lines.
- D2 — the temporary boot default (`open_pane(FileTree)` + refocus the terminal) is clearly marked
  `#121 TEMP → seq-3` so seq-3 swaps it for the serialize/restore boot layout.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN the app boots, a terminal pane AND a Files pane shall render side by side, each with its title bar. | live capture |
| REQ-002 (visual) | WHEN a FileTree pane renders, its body shall show the project tree with file-type icons. | live capture |
| REQ-003 | gate GREEN (shim-only; the render/handlers are masked). | gate |

## Phase Plan
- **P2** — the dispatch `else`-match + each kind body + the temp boot default; risks.
- **P3** — implement (app.rs).
- **P3.5** — 1 self-review: all 4 kinds dispatched, no panic, the temp default marked for seq-3.
- **P4** — gate GREEN + a LIVE capture ([terminal | files pane]).
- **P5** — docs, AAR, archive, close #121.
