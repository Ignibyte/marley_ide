---
pipeline_id: 5e07f958-b44e-4af2-b429-ed4bba0d2273
ticket: forge#123 (e79c0d80-4794-48cf-bcb5-d9675a74d8b4) · local docs/planning/tickets/open/TICKET-123-files-pane.md
aar_id: (recorded in notes)
status: Phase 5 — Complete PASS
title: the file explorer as a real pane (M6 seq-4)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/app.rs (SHIM: remove the dock FILES section; enrich the FileTree pane)
---

## Title
Retire the duplicate left-dock file tree — the project explorer now lives only in its own grid pane, and the
left sidebar is sessions-only (like Warp). Clears the first on-screen duplication.

## Scope
### In
- Remove the dock "FILES" section (the header + the visible_rows loop).
- The FileTree pane rows get the folder-toggle (dir click) + file-click (⌘-click → viewer), moved from the dock.

### Out
- Rerouting file-open to a CodeView *pane* (that's #128). New pure logic.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the left sidebar renders only sessions (M5 #109-112); the file tree is the FileTree pane alone.
- D2 — the pane's dir rows call `file_tree.toggle(index)`; file rows ⌘-click `open_file_in_viewer` (preserved
  from the dock); a plain file click is a no-op until #128 wires the code-pane open.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN the app renders, the left sidebar shall show NO "FILES" section (sessions only). | live capture |
| REQ-002 (visual) | WHEN a FileTree pane renders, its rows shall show the icon'd, disclosure tree. | live capture |
| REQ-003 | gate GREEN (shim-only; the render/handlers masked). | gate |

## Phase Plan
- **P2** — the dock removal + the pane row handlers; risks.
- **P3** — implement (app.rs).
- **P3.5** — 1 self-review: no dangling dock refs, correct index wiring.
- **P4** — gate GREEN + a LIVE capture (sessions-only sidebar + interactive Files pane).
- **P5** — docs, AAR, archive, close #123.
